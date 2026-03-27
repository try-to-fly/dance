#[cfg(test)]
mod tests {
    use crate::app_paths::AppPaths;
    use crate::database::Database;
    use crate::models::{ClipboardEntry, ContentType};
    use crate::state::AppState;
    use crate::test_support::{create_temp_app_roots, TestAppRoots};
    use std::sync::Arc;

    async fn create_test_state() -> (Arc<AppState>, TestAppRoots) {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let db = Database::new_in(paths.clone()).await.unwrap();

        let state = AppState {
            paths: paths.clone(),
            db: Arc::new(db),
            monitor: Arc::new(tokio::sync::RwLock::new(None)),
            tx: tokio::sync::broadcast::channel(100).0,
            _rx: Arc::new(tokio::sync::Mutex::new(
                tokio::sync::broadcast::channel(100).1,
            )),
            app_handle: Arc::new(tokio::sync::Mutex::new(None)),
            processor: Arc::new(crate::clipboard::ContentProcessor::new_in(paths.clone()).unwrap()),
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
    async fn test_state_creation() {
        let (_state, _temp_dir) = create_test_state().await;
        // If we reach here without panicking, state creation is successful
    }

    #[tokio::test]
    async fn test_get_clipboard_history_empty() {
        let (state, _temp_dir) = create_test_state().await;

        let result = state.get_clipboard_history(Some(10), Some(0), None).await;
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_get_clipboard_history_with_entries() {
        let (state, _temp_dir) = create_test_state().await;

        // Insert test entries directly into the database
        let test_entries: Vec<ClipboardEntry> = (0..15)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Test content {}", i)),
                    format!("hash_{}", i),
                    Some(format!("App{}", i)),
                    None,
                )
            })
            .collect();

        for entry in &test_entries {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Test getting all entries
        let result = state.get_clipboard_history(None, None, None).await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 15);

        // Test pagination
        let result = state.get_clipboard_history(Some(10), Some(0), None).await;
        assert!(result.is_ok());
        let first_page = result.unwrap();
        assert_eq!(first_page.len(), 10);

        let result = state.get_clipboard_history(Some(10), Some(10), None).await;
        assert!(result.is_ok());
        let second_page = result.unwrap();
        assert_eq!(second_page.len(), 5);

        // Verify no overlap between pages
        let first_page_ids: std::collections::HashSet<String> =
            first_page.iter().map(|e| e.id.clone()).collect();
        let second_page_ids: std::collections::HashSet<String> =
            second_page.iter().map(|e| e.id.clone()).collect();
        assert!(first_page_ids.is_disjoint(&second_page_ids));
    }

    #[tokio::test]
    async fn test_get_clipboard_history_with_search() {
        let (state, _temp_dir) = create_test_state().await;

        let test_entries = vec![
            ClipboardEntry::new(
                ContentType::Text,
                Some("Hello world".to_string()),
                "search_hash_1".to_string(),
                Some("TestApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Text,
                Some("Python programming".to_string()),
                "search_hash_2".to_string(),
                Some("TestApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Text,
                Some("JavaScript code".to_string()),
                "search_hash_3".to_string(),
                Some("TestApp".to_string()),
                None,
            ),
        ];

        for entry in &test_entries {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Search for "Python"
        let result = state
            .get_clipboard_history(None, None, Some("Python".to_string()))
            .await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].content_data.as_ref().unwrap().contains("Python"));

        // Search for "script" (should match JavaScript)
        let result = state
            .get_clipboard_history(None, None, Some("script".to_string()))
            .await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .content_data
            .as_ref()
            .unwrap()
            .contains("JavaScript"));

        // Search with no results
        let result = state
            .get_clipboard_history(None, None, Some("nonexistent".to_string()))
            .await;
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_toggle_favorite() {
        let (state, _temp_dir) = create_test_state().await;

        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("Favorite test".to_string()),
            "fav_hash".to_string(),
            Some("TestApp".to_string()),
            None,
        );

        // Insert entry
        sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
        .execute(state.db.pool())
        .await
        .unwrap();

        // Initially should not be favorite
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(state.db.pool())
                .await
                .unwrap();
        assert!(!stored_entry.is_favorite);

        // Toggle to favorite
        let result = state.toggle_favorite(entry.id.clone()).await;
        assert!(result.is_ok());

        // Verify it's now favorite
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(state.db.pool())
                .await
                .unwrap();
        assert!(stored_entry.is_favorite);

        // Toggle back to not favorite
        let result = state.toggle_favorite(entry.id.clone()).await;
        assert!(result.is_ok());

        // Verify it's no longer favorite
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .fetch_one(state.db.pool())
                .await
                .unwrap();
        assert!(!stored_entry.is_favorite);
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let (state, _temp_dir) = create_test_state().await;

        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("To be deleted".to_string()),
            "delete_hash".to_string(),
            Some("TestApp".to_string()),
            None,
        );

        // Insert entry
        sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
        .execute(state.db.pool())
        .await
        .unwrap();

        // Verify entry exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries WHERE id = ?")
            .bind(&entry.id)
            .fetch_one(state.db.pool())
            .await
            .unwrap();
        assert_eq!(count, 1);

        // Delete entry
        let result = state.delete_entry(entry.id.clone()).await;
        assert!(result.is_ok());

        // Verify entry is gone
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries WHERE id = ?")
            .bind(&entry.id)
            .fetch_one(state.db.pool())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let (state, _temp_dir) = create_test_state().await;

        // Insert multiple entries
        let test_entries: Vec<ClipboardEntry> = (0..5)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Content {}", i)),
                    format!("hash_{}", i),
                    Some("TestApp".to_string()),
                    None,
                )
            })
            .collect();

        for entry in &test_entries {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Verify entries exist
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .unwrap();
        assert_eq!(count, 5);

        // Clear history
        let result = state.clear_history().await;
        assert!(result.is_ok());

        // Verify all entries are gone
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let (state, _temp_dir) = create_test_state().await;

        let test_entries = vec![
            // Entry that will be most copied
            {
                let mut entry = ClipboardEntry::new(
                    ContentType::Text,
                    Some("Most popular".to_string()),
                    "popular_hash".to_string(),
                    Some("PopularApp".to_string()),
                    None,
                );
                entry.copy_count = 10;
                entry
            },
            // Regular entries
            ClipboardEntry::new(
                ContentType::Text,
                Some("Regular content 1".to_string()),
                "regular_hash_1".to_string(),
                Some("RegularApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Text,
                Some("Regular content 2".to_string()),
                "regular_hash_2".to_string(),
                Some("RegularApp".to_string()),
                None,
            ),
            ClipboardEntry::new(
                ContentType::Image,
                None,
                "image_hash".to_string(),
                Some("ImageApp".to_string()),
                Some("imgs/test.png".to_string()),
            ),
        ];

        for entry in &test_entries {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, file_path)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            .bind(&entry.file_path)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        let result = state.get_statistics().await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_entries, 4);
        assert_eq!(stats.total_copies, 13); // 10 + 1 + 1 + 1

        // Most copied entry should be the popular one
        assert!(!stats.most_copied.is_empty());
        assert_eq!(stats.most_copied[0].copy_count, 10);

        // Should have app usage statistics
        assert!(!stats.recent_apps.is_empty());

        // Find RegularApp usage (should appear twice)
        let regular_app_usage = stats
            .recent_apps
            .iter()
            .find(|app| app.app_name == "RegularApp");
        assert!(regular_app_usage.is_some());
        assert_eq!(regular_app_usage.unwrap().count, 2);
    }

    #[tokio::test]
    async fn test_get_cache_statistics() {
        let (state, _temp_dir) = create_test_state().await;

        // Insert test entries
        let text_entries: Vec<ClipboardEntry> = (0..3)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Text content {}", i)),
                    format!("text_hash_{}", i),
                    Some("TextApp".to_string()),
                    None,
                )
            })
            .collect();

        let image_entries: Vec<ClipboardEntry> = (0..2)
            .map(|i| {
                ClipboardEntry::new(
                    ContentType::Image,
                    None,
                    format!("image_hash_{}", i),
                    Some("ImageApp".to_string()),
                    Some(format!("imgs/image_{}.png", i)),
                )
            })
            .collect();

        for entry in text_entries.iter().chain(image_entries.iter()) {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite, file_path)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            .bind(&entry.file_path)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        let result = state.get_cache_statistics().await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.text_entries, 3);
        assert_eq!(stats.image_entries, 2);
        assert!(stats.db_size_bytes > 0);
        // images_size_bytes might be 0 if no actual image files exist
    }

    #[tokio::test]
    async fn test_monitoring_state() {
        let (state, _temp_dir) = create_test_state().await;

        // Initially not monitoring
        assert!(!state.is_monitoring().await);

        // Note: We can't easily test start_monitoring/stop_monitoring without
        // actually creating the clipboard monitor, which requires system access
        // These would be better suited for integration tests
    }

    #[tokio::test]
    async fn test_skip_next_clipboard_change() {
        let (state, _temp_dir) = create_test_state().await;

        // Initially should not skip
        {
            let skip_guard = state.skip_next_change.lock().await;
            assert!(!*skip_guard);
        }

        // Set to skip
        state.set_skip_next_clipboard_change(true).await;

        {
            let skip_guard = state.skip_next_change.lock().await;
            assert!(*skip_guard);
        }

        // Set back to not skip
        state.set_skip_next_clipboard_change(false).await;

        {
            let skip_guard = state.skip_next_change.lock().await;
            assert!(!*skip_guard);
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (state, _temp_dir) = create_test_state().await;
        let state = Arc::clone(&state);

        // Create test entry
        let entry = ClipboardEntry::new(
            ContentType::Text,
            Some("Concurrent test".to_string()),
            "concurrent_hash".to_string(),
            Some("TestApp".to_string()),
            None,
        );

        sqlx::query(
            r#"
            INSERT INTO clipboard_entries 
            (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
        .execute(state.db.pool())
        .await
        .unwrap();

        let entry_id = entry.id.clone();
        let mut handles = vec![];

        // Spawn multiple concurrent toggle operations
        for _ in 0..10 {
            let state_clone = Arc::clone(&state);
            let id_clone = entry_id.clone();
            let handle = tokio::spawn(async move {
                let _ = state_clone.toggle_favorite(id_clone).await;
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify the entry still exists and is in a valid state
        let stored_entry =
            sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                .bind(&entry_id)
                .fetch_one(state.db.pool())
                .await
                .unwrap();

        // The final favorite state could be either true or false,
        // but the entry should exist and be valid
        assert_eq!(stored_entry.id, entry_id);
        assert_eq!(
            stored_entry.content_data,
            Some("Concurrent test".to_string())
        );
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (state, _temp_dir) = create_test_state().await;

        // Test toggle favorite with non-existent ID
        let _result = state.toggle_favorite("nonexistent_id".to_string()).await;
        // This should either succeed (no-op) or return an appropriate error
        // The exact behavior depends on implementation

        // Test delete with non-existent ID
        let _result = state.delete_entry("nonexistent_id".to_string()).await;
        // Similar to above, should handle gracefully

        // Test getting statistics with empty database
        let result = state.get_statistics().await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_copies, 0);
    }

    #[tokio::test]
    async fn test_config_operations() {
        let (state, _temp_dir) = create_test_state().await;

        // Test getting default config
        let result = state.get_config().await;
        assert!(result.is_ok());
        let config = result.unwrap();

        // Default values should be set
        assert!(config.text.max_size_mb > 0.0);

        // Test updating config
        let mut new_config = config.clone();
        new_config.text.max_size_mb = 2.0;
        new_config.auto_update = true;

        let result = state.update_config(new_config.clone()).await;
        assert!(result.is_ok());

        // Verify config was updated
        let result = state.get_config().await;
        assert!(result.is_ok());
        let updated_config = result.unwrap();
        assert_eq!(updated_config.text.max_size_mb, 2.0);
        assert!(updated_config.auto_update);
    }

    #[tokio::test]
    async fn test_large_scale_operations() {
        let (state, _temp_dir) = create_test_state().await;

        // Insert a large number of entries
        let num_entries = 1000;
        for i in 0..num_entries {
            let entry = ClipboardEntry::new(
                ContentType::Text,
                Some(format!("Large scale content {}", i)),
                format!("large_hash_{}", i),
                Some(format!("App{}", i % 10)), // 10 different apps
                None,
            );

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries 
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, is_favorite)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at + i as i64) // Different timestamps
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Test pagination with large dataset
        let start = std::time::Instant::now();
        let result = state.get_clipboard_history(Some(100), Some(0), None).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 100);

        // Should complete reasonably quickly
        assert!(duration.as_millis() < 1000);

        // Test search with large dataset
        let start = std::time::Instant::now();
        let result = state
            .get_clipboard_history(None, None, Some("500".to_string()))
            .await;
        let search_duration = start.elapsed();

        assert!(result.is_ok());
        let search_results = result.unwrap();
        assert!(!search_results.is_empty()); // Should find entries containing "500"

        // Search should also complete reasonably quickly
        assert!(search_duration.as_millis() < 1000);

        // Test statistics with large dataset
        let start = std::time::Instant::now();
        let result = state.get_statistics().await;
        let stats_duration = start.elapsed();

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_entries, num_entries as i64);
        assert_eq!(stats.total_copies, num_entries as i64); // Each entry has copy_count = 1

        // Statistics should complete reasonably quickly
        assert!(stats_duration.as_millis() < 1000);
    }
}
