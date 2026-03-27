#[cfg(test)]
mod integration_tests {
    use crate::app_paths::AppPaths;
    use crate::clipboard::content_detector::{ContentDetector, ContentSubType};
    use crate::clipboard::ContentProcessor;
    use crate::database::Database;
    use crate::models::{ClipboardEntry, ContentType};
    use crate::state::AppState;
    use crate::test_support::{create_temp_app_roots, TestAppRoots};
    use std::sync::Arc;
    use tokio::sync::broadcast;

    async fn create_integration_test_env() -> (Arc<AppState>, TestAppRoots) {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let db = Database::new_in(paths.clone()).await.unwrap();

        let (tx, rx) = broadcast::channel(100);
        let state = AppState {
            paths: paths.clone(),
            db: Arc::new(db),
            monitor: Arc::new(tokio::sync::RwLock::new(None)),
            tx,
            _rx: Arc::new(tokio::sync::Mutex::new(rx)),
            app_handle: Arc::new(tokio::sync::Mutex::new(None)),
            processor: Arc::new(ContentProcessor::new_in(paths.clone()).unwrap()),
            skip_next_change: Arc::new(tokio::sync::Mutex::new(false)),
            config_manager: Arc::new(tokio::sync::Mutex::new(
                crate::config::ConfigManager::new_in(paths).await.unwrap(),
            )),
            current_shortcut: Arc::new(tokio::sync::Mutex::new(None)),
            last_cleanup_date: Arc::new(tokio::sync::Mutex::new(None)),
        };

        (Arc::new(state), roots)
    }

    #[tokio::test]
    async fn test_text_detection_to_storage_pipeline() {
        let (state, _temp_dir) = create_integration_test_env().await;

        let test_cases = vec![
            ("https://www.example.com", ContentSubType::Url),
            ("192.168.1.1", ContentSubType::IpAddress),
            ("user@example.com", ContentSubType::Email),
            ("#ff0000", ContentSubType::Color),
            (r#"{"key": "value", "number": 42}"#, ContentSubType::Json),
            ("git status", ContentSubType::Command),
            ("1640995200000", ContentSubType::Timestamp),
            ("# Markdown Header", ContentSubType::Markdown),
            ("function test() { return true; }", ContentSubType::Code),
            ("Hello, world!", ContentSubType::PlainText),
        ];

        for (content, expected_subtype) in test_cases {
            // Step 1: Content Detection
            let (detected_subtype, metadata) = ContentDetector::detect(content);
            assert_eq!(
                detected_subtype, expected_subtype,
                "Failed to detect correct subtype for: {}",
                content
            );

            // Step 2: Create Entry with Detection Results
            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("hash_{}", content.len()),
                Some("TestApp".to_string()),
                None,
            );

            // Apply detection results
            let subtype_str = serde_json::to_value(&detected_subtype)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Step 3: Store in Database
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();

            // Step 4: Retrieve and Verify
            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(state.db.pool())
                    .await
                    .unwrap();

            assert_eq!(stored_entry.content_data.as_ref().unwrap(), content);
            assert!(stored_entry.content_subtype.is_some());

            // Verify metadata exists for types that should have it
            match expected_subtype {
                ContentSubType::Url | ContentSubType::Color | ContentSubType::Timestamp => {
                    assert!(
                        stored_entry.metadata.is_some(),
                        "Metadata should exist for {:?}",
                        expected_subtype
                    );
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_complex_content_processing() {
        let (state, _temp_dir) = create_integration_test_env().await;

        let complex_contents = vec![
            // JSON with embedded URL
            (r#"{"api_endpoint": "https://api.example.com/v1/users", "timeout": 5000}"#,
             ContentSubType::Json),


            // Code with embedded email
            ("const email = 'developer@company.com';\nconsole.log(email);",
             ContentSubType::Code),


            // Markdown with multiple elements
            ("# Project README\n\n![Logo](https://example.com/logo.png)\n\n```javascript\nfunction hello() {}\n```",
             ContentSubType::Markdown),


            // Command with URL
            ("curl -X GET https://httpbin.org/json", 
             ContentSubType::Command),
        ];

        for (content, expected_primary_type) in complex_contents {
            // Process content
            let (detected_type, metadata) = ContentDetector::detect(content);
            assert_eq!(
                detected_type, expected_primary_type,
                "Primary type detection failed for: {}",
                content
            );

            // Create and store entry
            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("complex_hash_{}", content.len()),
                Some("ComplexApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Store
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();

            // Verify full round-trip
            let retrieved = state.get_clipboard_history(None, None, None).await.unwrap();
            let stored_entry = retrieved
                .iter()
                .find(|e| e.id == entry.id)
                .expect("Entry should be found");

            assert_eq!(stored_entry.content_data.as_ref().unwrap(), content);
            assert_eq!(stored_entry.content_subtype, entry.content_subtype);
        }
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters_pipeline() {
        let (state, _temp_dir) = create_integration_test_env().await;

        let unicode_test_cases = vec![
            (
                "🌟 JavaScript emoji test: console.log('✨');",
                ContentSubType::Code,
            ),
            (
                "# 标题 中文 Markdown 测试\n\n- 列表项目 📝",
                ContentSubType::Markdown,
            ),
            (
                r#"{"🔑": "中文键", "value": "Русский текст", "emoji": "🚀"}"#,
                ContentSubType::Json,
            ),
            ("https://例え.テスト/パス?クエリ=値", ContentSubType::Url),
            ("用户@中文域名.测试", ContentSubType::Email),
            (
                "git status --porcelain | grep '中文文件.txt'",
                ContentSubType::Command,
            ),
        ];

        for (content, expected_type) in unicode_test_cases {
            // Full pipeline test
            let (detected_type, metadata) = ContentDetector::detect(content);
            assert_eq!(
                detected_type, expected_type,
                "Unicode content type detection failed for: {}",
                content
            );

            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("unicode_hash_{}", content.chars().count()),
                Some("UnicodeApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Store and verify
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();

            // Verify unicode content is preserved exactly
            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(state.db.pool())
                    .await
                    .unwrap();

            assert_eq!(stored_entry.content_data.as_ref().unwrap(), content);
            assert_eq!(
                stored_entry.content_data.as_ref().unwrap().chars().count(),
                content.chars().count()
            );
        }
    }

    #[tokio::test]
    async fn test_large_content_processing_pipeline() {
        let (state, _temp_dir) = create_integration_test_env().await;

        // Test different large content scenarios
        let large_text = "A".repeat(50000);
        let large_json = format!(r#"{{"data": "{}", "size": {}}}"#, "B".repeat(10000), 10000);
        let large_code = format!(
            "// Large comment\n{}\nfunction process() {{\n  return true;\n}}",
            "// ".repeat(5000)
        );

        let test_cases = vec![
            (large_text, ContentSubType::PlainText),
            (large_json, ContentSubType::Json),
            (large_code, ContentSubType::Code),
        ];

        for (content, expected_type) in test_cases {
            let content_length = content.len();

            // Performance timing
            let start_time = std::time::Instant::now();

            // Detection phase
            let (detected_type, metadata) = ContentDetector::detect(&content);
            let detection_time = start_time.elapsed();

            assert_eq!(detected_type, expected_type);
            assert!(
                detection_time.as_millis() < 1000,
                "Detection took too long: {}ms for {}KB",
                detection_time.as_millis(),
                content_length / 1024
            );

            // Entry creation
            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.clone()),
                format!("large_hash_{}", content_length),
                Some("LargeApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Storage phase
            let storage_start = std::time::Instant::now();

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();

            let storage_time = storage_start.elapsed();
            assert!(
                storage_time.as_millis() < 2000,
                "Storage took too long: {}ms for {}KB",
                storage_time.as_millis(),
                content_length / 1024
            );

            // Retrieval phase
            let retrieval_start = std::time::Instant::now();

            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(state.db.pool())
                    .await
                    .unwrap();

            let retrieval_time = retrieval_start.elapsed();
            assert!(
                retrieval_time.as_millis() < 1000,
                "Retrieval took too long: {}ms for {}KB",
                retrieval_time.as_millis(),
                content_length / 1024
            );

            // Verify content integrity
            assert_eq!(
                stored_entry.content_data.as_ref().unwrap().len(),
                content_length
            );
            assert_eq!(*stored_entry.content_data.as_ref().unwrap(), content);
        }
    }

    #[tokio::test]
    async fn test_search_integration() {
        let (state, _temp_dir) = create_integration_test_env().await;

        // Create entries with various content types
        let test_entries = vec![
            (
                "https://github.com/rust-lang/rust",
                ContentSubType::Url,
                vec!["github", "rust"],
            ),
            (
                "function fibonacci(n) { return n < 2 ? n : fibonacci(n-1) + fibonacci(n-2); }",
                ContentSubType::Code,
                vec!["function", "fibonacci", "javascript"],
            ),
            (
                r#"{"name": "John Doe", "email": "john@example.com", "age": 30}"#,
                ContentSubType::Json,
                vec!["john", "email", "json"],
            ),
            (
                "git clone https://github.com/user/repo.git",
                ContentSubType::Command,
                vec!["git", "clone", "github"],
            ),
            (
                "# Project Documentation\n\nThis is a **bold** statement.",
                ContentSubType::Markdown,
                vec!["project", "documentation", "bold"],
            ),
        ];

        // Store all entries
        for (content, expected_type, _search_terms) in &test_entries {
            let (detected_type, metadata) = ContentDetector::detect(content);
            assert_eq!(detected_type, *expected_type);

            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("search_hash_{}", content.len()),
                Some("SearchTestApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Test searches
        let search_tests = vec![
            ("github", 2),      // Should find URL and command
            ("function", 1),    // Should find JavaScript code
            ("john", 1),        // Should find JSON
            ("project", 1),     // Should find Markdown
            ("nonexistent", 0), // Should find nothing
        ];

        for (search_term, expected_count) in search_tests {
            let results = state
                .get_clipboard_history(None, None, Some(search_term.to_string()))
                .await
                .unwrap();

            assert_eq!(
                results.len(),
                expected_count,
                "Search for '{}' expected {} results, got {}",
                search_term,
                expected_count,
                results.len()
            );

            // Verify all results contain the search term
            for result in &results {
                assert!(
                    result
                        .content_data
                        .as_ref()
                        .unwrap()
                        .to_lowercase()
                        .contains(&search_term.to_lowercase()),
                    "Search result doesn't contain search term: {}",
                    search_term
                );
            }
        }
    }

    #[tokio::test]
    async fn test_favorite_and_statistics_integration() {
        let (state, _temp_dir) = create_integration_test_env().await;

        // Create entries with different types and usage patterns
        let entries_data = vec![
            (
                "Popular URL",
                "https://www.popular-site.com",
                ContentSubType::Url,
                10,
            ),
            ("Common Command", "git status", ContentSubType::Command, 5),
            (
                "Useful JSON",
                r#"{"config": "production"}"#,
                ContentSubType::Json,
                3,
            ),
            ("Regular Text", "Hello world", ContentSubType::PlainText, 1),
            (
                "Important Code",
                "function main() {}",
                ContentSubType::Code,
                7,
            ),
        ];

        let mut entry_ids = Vec::new();

        // Create and store entries
        for (name, content, expected_type, copy_count) in entries_data {
            let (detected_type, metadata) = ContentDetector::detect(content);
            assert_eq!(detected_type, expected_type);

            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("fav_hash_{}", name.replace(" ", "_")),
                Some(format!("{}App", name.replace(" ", ""))),
                None,
            );

            entry.copy_count = copy_count;
            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await
            .unwrap();

            entry_ids.push(entry.id);
        }

        // Test favorite toggling
        let first_entry_id = &entry_ids[0];
        let second_entry_id = &entry_ids[1];

        // Toggle favorites
        state.toggle_favorite(first_entry_id.clone()).await.unwrap();
        state
            .toggle_favorite(second_entry_id.clone())
            .await
            .unwrap();

        // Verify favorites
        let favorites = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE is_favorite = 1",
        )
        .fetch_all(state.db.pool())
        .await
        .unwrap();

        assert_eq!(favorites.len(), 2);
        let favorite_ids: std::collections::HashSet<String> =
            favorites.iter().map(|e| e.id.clone()).collect();
        assert!(favorite_ids.contains(first_entry_id));
        assert!(favorite_ids.contains(second_entry_id));

        // Test statistics integration
        let stats = state.get_statistics().await.unwrap();

        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.total_copies, 10 + 5 + 3 + 1 + 7); // Sum of all copy_counts

        // Most copied should be the popular URL (copy_count = 10)
        assert!(!stats.most_copied.is_empty());
        assert_eq!(stats.most_copied[0].copy_count, 10);
        assert!(stats.most_copied[0]
            .content_data
            .as_ref()
            .unwrap()
            .contains("popular-site.com"));

        // Verify app usage statistics
        assert!(!stats.recent_apps.is_empty());
        let app_names: std::collections::HashSet<String> = stats
            .recent_apps
            .iter()
            .map(|app| app.app_name.clone())
            .collect();
        assert!(app_names.contains("PopularURLApp"));
        assert!(app_names.contains("CommonCommandApp"));
    }

    #[tokio::test]
    async fn test_concurrent_processing_pipeline() {
        let (state, _temp_dir) = create_integration_test_env().await;
        let state = Arc::clone(&state);

        // Test concurrent content processing
        let test_contents = vec![
            "https://concurrent-test-1.com",
            "function concurrent1() { return 'test'; }",
            r#"{"concurrent": "test1", "id": 1}"#,
            "git concurrent test",
            "# Concurrent Test 1",
        ];

        let mut handles = vec![];

        // Process content concurrently
        for (i, content) in test_contents.into_iter().enumerate() {
            let state_clone = Arc::clone(&state);
            let content_clone = content.to_string();

            let handle = tokio::spawn(async move {
                // Detection
                let (detected_type, metadata) = ContentDetector::detect(&content_clone);

                // Entry creation
                let mut entry = ClipboardEntry::new(
                    ContentType::Text,
                    Some(content_clone.clone()),
                    format!("concurrent_hash_{}", i),
                    Some(format!("ConcurrentApp{}", i)),
                    None,
                );

                let subtype_str = serde_json::to_value(&detected_type)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "plain_text".to_string());
                entry.content_subtype = Some(subtype_str);

                if let Some(meta) = metadata {
                    entry.metadata = serde_json::to_string(&meta).ok();
                }

                // Storage
                sqlx::query(
                    r#"
                    INSERT INTO clipboard_entries 
                    (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(&entry.id)
                .bind(&entry.content_hash)
                .bind(&entry.content_type)
                .bind(&entry.content_data)
                .bind(&entry.source_app)
                .bind(entry.created_at)
                .bind(entry.copy_count)
                .bind(entry.is_favorite)
                .bind(&entry.content_subtype)
                .bind(&entry.metadata)
                .execute(state_clone.db.pool())
                .await
                .unwrap();

                entry.id
            });

            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        let entry_ids: Vec<String> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|result| result.unwrap())
            .collect();

        // Verify all entries were stored correctly
        let stored_entries = state.get_clipboard_history(None, None, None).await.unwrap();
        assert_eq!(stored_entries.len(), 5);

        // Verify all concurrent entries exist
        let stored_ids: std::collections::HashSet<String> =
            stored_entries.iter().map(|e| e.id.clone()).collect();

        for entry_id in entry_ids {
            assert!(
                stored_ids.contains(&entry_id),
                "Concurrent entry should exist: {}",
                entry_id
            );
        }
    }

    #[tokio::test]
    async fn test_error_recovery_pipeline() {
        let (state, _temp_dir) = create_integration_test_env().await;

        // Test various edge cases and error conditions
        let large_content = "a".repeat(1_000_000);
        let edge_cases = vec![
            "",                     // Empty content
            " ",                    // Whitespace only
            "\n\n\n",               // Newlines only
            "🚀",                   // Single emoji
            large_content.as_str(), // Very large content
            "http://",              // Malformed URL
            "{incomplete json",     // Malformed JSON
            "#",                    // Minimal markdown
        ];

        for (i, content) in edge_cases.into_iter().enumerate() {
            // This should not panic or fail
            let (detected_type, metadata) = ContentDetector::detect(content);

            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.to_string()),
                format!("edge_hash_{}", i),
                Some("EdgeCaseApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&detected_type)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Should be able to store any content
            let result = sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, content_subtype, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .execute(state.db.pool())
            .await;

            assert!(
                result.is_ok(),
                "Should be able to store edge case content: {:?}",
                content
            );

            // Should be able to retrieve it
            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(state.db.pool())
                    .await
                    .unwrap();

            assert_eq!(stored_entry.content_data.as_ref().unwrap(), content);
        }

        // Verify all edge cases were processed
        let total_entries: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .unwrap();

        assert_eq!(total_entries, 8); // All edge cases stored
    }
}
