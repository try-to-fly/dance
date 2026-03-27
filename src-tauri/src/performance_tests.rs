#[cfg(test)]
mod performance_tests {
    use crate::app_paths::AppPaths;
    use crate::clipboard::content_detector::ContentDetector;
    use crate::clipboard::ContentProcessor;
    use crate::database::Database;
    use crate::models::{ClipboardEntry, ContentType};
    use crate::state::AppState;
    use crate::test_support::{create_temp_app_roots, TestAppRoots};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::task::JoinSet;

    async fn create_perf_test_env() -> (Arc<AppState>, TestAppRoots) {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let db = Database::new_in(paths.clone()).await.unwrap();

        let (tx, rx) = tokio::sync::broadcast::channel(1000);
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
    #[ignore] // Use --ignored flag to run performance tests
    async fn test_content_detection_performance() {
        // Pre-allocate the longer strings to avoid borrowing issues
        let medium_text = "A".repeat(1000);
        let medium_url = format!("https://example.com/{}", "path/".repeat(100));
        let medium_json = format!(r#"{{"data": "{}"}}"#, "B".repeat(500));
        let large_text = "C".repeat(50000);
        let large_code = format!(
            "function test() {{\n{}\n}}",
            "  console.log('test');\n".repeat(1000)
        );

        let test_cases = vec![
            // Small content
            ("Hello world", 1000),
            ("https://example.com", 1000),
            ("#ff0000", 1000),
            (r#"{"key": "value"}"#, 1000),
            // Medium content
            (medium_text.as_str(), 100),
            (medium_url.as_str(), 100),
            (medium_json.as_str(), 100),
            // Large content
            (large_text.as_str(), 10),
            (large_code.as_str(), 10),
        ];

        for (content, iterations) in test_cases {
            let content_size = content.len();
            let mut total_duration = Duration::new(0, 0);
            let mut detection_results = Vec::new();

            for _ in 0..iterations {
                let start = Instant::now();
                let (subtype, metadata) = ContentDetector::detect(content);
                let duration = start.elapsed();

                total_duration += duration;
                detection_results.push((subtype, metadata));
            }

            let avg_duration = total_duration / iterations as u32;
            let throughput = (content_size as f64 / avg_duration.as_secs_f64()) / 1024.0; // KB/s

            println!(
                "Content size: {}B, Iterations: {}, Avg time: {:?}, Throughput: {:.2}KB/s",
                content_size, iterations, avg_duration, throughput
            );

            // Performance assertions
            if content_size < 1000 {
                assert!(
                    avg_duration.as_millis() < 5,
                    "Small content detection should be < 5ms, got {:?}",
                    avg_duration
                );
            } else if content_size < 10000 {
                assert!(
                    avg_duration.as_millis() < 50,
                    "Medium content detection should be < 50ms, got {:?}",
                    avg_duration
                );
            } else {
                assert!(
                    avg_duration.as_millis() < 500,
                    "Large content detection should be < 500ms, got {:?}",
                    avg_duration
                );
            }

            // Verify consistency - all results should be the same
            let first_result = &detection_results[0];
            for result in &detection_results[1..] {
                assert_eq!(
                    result.0, first_result.0,
                    "Detection results should be consistent"
                );
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_database_insertion_performance() {
        let (state, _temp_dir) = create_perf_test_env().await;
        let batch_sizes = vec![1, 10, 100, 1000];

        for batch_size in batch_sizes {
            let entries: Vec<ClipboardEntry> = (0..batch_size)
                .map(|i| {
                    ClipboardEntry::new(
                        ContentType::Text,
                        Some(format!("Performance test content {}", i)),
                        format!("perf_hash_{}", i),
                        Some("PerfApp".to_string()),
                        None,
                    )
                })
                .collect();

            let start = Instant::now();

            for entry in &entries {
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

            let duration = start.elapsed();
            let per_item = duration / batch_size as u32;
            let throughput = batch_size as f64 / duration.as_secs_f64();

            println!(
                "Batch size: {}, Total time: {:?}, Per item: {:?}, Throughput: {:.2} items/s",
                batch_size, duration, per_item, throughput
            );

            // Performance assertions
            assert!(
                per_item.as_millis() < 100,
                "Individual insertion should be < 100ms, got {:?}",
                per_item
            );

            if batch_size >= 100 {
                assert!(
                    throughput > 10.0,
                    "Large batch throughput should be > 10 items/s, got {:.2}",
                    throughput
                );
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_query_performance_with_large_dataset() {
        let (state, _temp_dir) = create_perf_test_env().await;
        let num_entries = 10000;

        // Insert large dataset
        println!("Creating {} test entries...", num_entries);
        let setup_start = Instant::now();

        for i in 0..num_entries {
            let content = match i % 5 {
                0 => format!("https://example-{}.com/path/{}", i, i),
                1 => format!("function test{}() {{ return {}; }}", i, i),
                2 => format!(
                    r#"{{"id": {}, "name": "item{}", "data": "value{}"}}'"#,
                    i, i, i
                ),
                3 => format!("git commit -m 'Update item {}'", i),
                _ => format!("Plain text content number {}", i),
            };

            let entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content),
                format!("large_hash_{}", i),
                Some(format!("App{}", i % 10)),
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
            .bind(entry.created_at + i as i64)
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        let setup_duration = setup_start.elapsed();
        println!("Setup completed in {:?}", setup_duration);

        // Test various query patterns
        let query_tests = vec![
            ("Get recent 100 entries", Some(100), Some(0), None),
            ("Get page 2 (100 entries)", Some(100), Some(100), None),
            ("Get page 10 (100 entries)", Some(100), Some(1000), None),
            (
                "Search for 'function'",
                None,
                None,
                Some("function".to_string()),
            ),
            (
                "Search for 'example-500'",
                None,
                None,
                Some("example-500".to_string()),
            ),
            (
                "Search for common term 'git'",
                None,
                None,
                Some("git".to_string()),
            ),
        ];

        for (test_name, limit, offset, search) in query_tests {
            let start = Instant::now();
            let results = state
                .get_clipboard_history(limit, offset, search)
                .await
                .unwrap();
            let duration = start.elapsed();

            println!("{}: {} results in {:?}", test_name, results.len(), duration);

            // Performance assertions
            assert!(
                duration.as_millis() < 1000,
                "{} should complete in < 1s, got {:?}",
                test_name,
                duration
            );

            // For pagination tests, verify we get expected number of results
            if let Some(expected_limit) = limit {
                if offset.unwrap_or(0) + expected_limit <= num_entries {
                    assert_eq!(
                        results.len(),
                        expected_limit as usize,
                        "{} should return {} results",
                        test_name,
                        expected_limit
                    );
                }
            }
        }

        // Test statistics performance
        let stats_start = Instant::now();
        let stats = state.get_statistics().await.unwrap();
        let stats_duration = stats_start.elapsed();

        println!("Statistics query: {:?}", stats_duration);
        assert!(
            stats_duration.as_millis() < 2000,
            "Statistics should complete in < 2s with large dataset"
        );
        assert_eq!(stats.total_entries, num_entries as i64);
    }

    #[tokio::test]
    #[ignore]
    async fn test_concurrent_access_performance() {
        let (state, _temp_dir) = create_perf_test_env().await;
        let state = Arc::clone(&state);
        let num_threads = 10;
        let operations_per_thread = 100;

        println!(
            "Testing concurrent access with {} threads, {} ops each",
            num_threads, operations_per_thread
        );

        let start = Instant::now();
        let mut join_set = JoinSet::new();

        // Spawn concurrent tasks
        for thread_id in 0..num_threads {
            let state_clone = Arc::clone(&state);

            join_set.spawn(async move {
                let mut thread_results = Vec::new();


                for op_id in 0..operations_per_thread {
                    let op_start = Instant::now();


                    // Mix of operations
                    match op_id % 4 {
                        0 => {
                            // Insert
                            let entry = ClipboardEntry::new(
                                ContentType::Text,
                                Some(format!("Concurrent content {}-{}", thread_id, op_id)),
                                format!("concurrent_hash_{}_{}", thread_id, op_id),
                                Some("ConcurrentApp".to_string()),
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
                            .execute(state_clone.db.pool())
                            .await
                            .unwrap();
                        },
                        1 => {
                            // Query recent entries
                            let _results = state_clone.get_clipboard_history(Some(10), None, None)
                                .await
                                .unwrap();
                        },
                        2 => {
                            // Search
                            let _results = state_clone.get_clipboard_history(
                                None, None, Some(format!("{}", thread_id))
                            ).await.unwrap();
                        },
                        3 => {
                            // Statistics
                            let _stats = state_clone.get_statistics().await.unwrap();
                        },
                        _ => unreachable!(),
                    }

                    let op_duration = op_start.elapsed();
                    thread_results.push(op_duration);
                }

                thread_results
            });
        }

        // Wait for all threads to complete
        let mut all_results = Vec::new();
        while let Some(thread_results) = join_set.join_next().await {
            all_results.extend(thread_results.unwrap());
        }

        let total_duration = start.elapsed();
        let total_operations = num_threads * operations_per_thread;
        let avg_duration: Duration =
            all_results.iter().sum::<Duration>() / all_results.len() as u32;
        let throughput = total_operations as f64 / total_duration.as_secs_f64();

        println!("Concurrent test completed:");
        println!("  Total time: {:?}", total_duration);
        println!("  Average operation time: {:?}", avg_duration);
        println!("  Throughput: {:.2} ops/s", throughput);

        // Performance assertions
        assert!(
            avg_duration.as_millis() < 100,
            "Average concurrent operation should be < 100ms, got {:?}",
            avg_duration
        );
        assert!(
            throughput > 50.0,
            "Concurrent throughput should be > 50 ops/s, got {:.2}",
            throughput
        );

        // Verify data integrity
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .unwrap();

        // Should have at least the insertions (25% of operations are inserts)
        let expected_min_inserts = (total_operations / 4) as i64;
        assert!(
            final_count >= expected_min_inserts,
            "Should have at least {} entries, got {}",
            expected_min_inserts,
            final_count
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_memory_usage_large_content() {
        let (state, _temp_dir) = create_perf_test_env().await;

        // Test memory efficiency with large content
        let large_contents = vec![
            ("1KB text", "A".repeat(1024)),
            ("10KB text", "B".repeat(10 * 1024)),
            ("100KB text", "C".repeat(100 * 1024)),
            ("1MB text", "D".repeat(1024 * 1024)),
        ];

        for (description, content) in large_contents {
            let content_size = content.len();
            println!("Testing {}: {} bytes", description, content_size);

            // Measure processing time
            let detection_start = Instant::now();
            let (subtype, metadata) = ContentDetector::detect(&content);
            let detection_duration = detection_start.elapsed();

            // Create entry
            let mut entry = ClipboardEntry::new(
                ContentType::Text,
                Some(content.clone()),
                format!("large_hash_{}", content_size),
                Some("LargeContentApp".to_string()),
                None,
            );

            let subtype_str = serde_json::to_value(&subtype)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "plain_text".to_string());
            entry.content_subtype = Some(subtype_str);

            if let Some(meta) = metadata {
                entry.metadata = serde_json::to_string(&meta).ok();
            }

            // Measure storage time
            let storage_start = Instant::now();
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
            let storage_duration = storage_start.elapsed();

            // Measure retrieval time
            let retrieval_start = Instant::now();
            let stored_entry =
                sqlx::query_as::<_, ClipboardEntry>("SELECT * FROM clipboard_entries WHERE id = ?")
                    .bind(&entry.id)
                    .fetch_one(state.db.pool())
                    .await
                    .unwrap();
            let retrieval_duration = retrieval_start.elapsed();

            println!(
                "  Detection: {:?}, Storage: {:?}, Retrieval: {:?}",
                detection_duration, storage_duration, retrieval_duration
            );

            // Performance assertions based on content size
            let size_category = if content_size < 10 * 1024 {
                "small"
            } else if content_size < 100 * 1024 {
                "medium"
            } else {
                "large"
            };

            match size_category {
                "small" => {
                    assert!(detection_duration.as_millis() < 10);
                    assert!(storage_duration.as_millis() < 100);
                    assert!(retrieval_duration.as_millis() < 50);
                }
                "medium" => {
                    assert!(detection_duration.as_millis() < 50);
                    assert!(storage_duration.as_millis() < 500);
                    assert!(retrieval_duration.as_millis() < 200);
                }
                "large" => {
                    assert!(detection_duration.as_millis() < 200);
                    assert!(storage_duration.as_millis() < 2000);
                    assert!(retrieval_duration.as_millis() < 1000);
                }
                _ => unreachable!(),
            }

            // Verify content integrity
            assert_eq!(
                stored_entry.content_data.as_ref().unwrap().len(),
                content_size
            );
            assert_eq!(*stored_entry.content_data.as_ref().unwrap(), content);

            // Clean up to free memory
            sqlx::query("DELETE FROM clipboard_entries WHERE id = ?")
                .bind(&entry.id)
                .execute(state.db.pool())
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_regex_performance() {
        // Test performance of various regex patterns used in content detection
        let test_patterns = vec![
            (
                "URL",
                r"^(https?|ftp)://[^\s/$.?#].[^\s]*$",
                vec![
                    "https://example.com",
                    "http://test.org/path?query=value",
                    "not a url at all",
                ],
            ),
            (
                "Email",
                r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
                vec![
                    "user@example.com",
                    "test.email+tag@domain.co.uk",
                    "not an email",
                ],
            ),
            (
                "IPv4",
                r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
                vec!["192.168.1.1", "255.255.255.255", "not an ip"],
            ),
            (
                "JSON Object",
                r"^\s*\{.*\}\s*$",
                vec![
                    r#"{"key": "value"}"#,
                    r#"  {"nested": {"deep": true}}  "#,
                    "not json",
                ],
            ),
        ];

        for (pattern_name, pattern_str, test_inputs) in test_patterns {
            println!("Testing {} regex pattern", pattern_name);
            let regex = regex::Regex::new(pattern_str).unwrap();

            let iterations = 10000;
            let mut total_duration = Duration::new(0, 0);

            for input in &test_inputs {
                let start = Instant::now();

                for _ in 0..iterations {
                    let _ = regex.is_match(input);
                }

                let duration = start.elapsed();
                total_duration += duration;

                let per_match = duration / iterations;
                let throughput = iterations as f64 / duration.as_secs_f64();

                println!(
                    "  Input '{}': {:?} per match, {:.0} matches/s",
                    input, per_match, throughput
                );

                // Performance assertion
                assert!(
                    per_match.as_nanos() < 10_000, // < 10 microseconds
                    "Regex match should be < 10μs, got {:?}",
                    per_match
                );
            }

            let avg_duration = total_duration / (test_inputs.len() as u32 * iterations);
            println!("  Average: {:?} per match\n", avg_duration);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_database_index_effectiveness() {
        let (state, _temp_dir) = create_perf_test_env().await;
        let num_entries = 50000;

        println!(
            "Creating {} entries for index performance test...",
            num_entries
        );

        // Insert test data
        for i in 0..num_entries {
            let entry = ClipboardEntry::new(
                ContentType::Text,
                Some(format!("Index test content number {}", i)),
                format!("index_hash_{}", i),
                Some(format!("App{}", i % 100)), // 100 different apps
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
            .bind(entry.created_at + i as i64) // Unique timestamps
            .bind(entry.copy_count)
            .bind(entry.is_favorite)
            .execute(state.db.pool())
            .await
            .unwrap();
        }

        // Test indexed queries
        let index_tests = vec![
            ("ORDER BY created_at DESC LIMIT 100", "idx_created_at"),
            (
                "WHERE content_hash = 'index_hash_25000'",
                "idx_content_hash",
            ),
            ("WHERE created_at > ? ORDER BY created_at", "idx_created_at"),
        ];

        for (query_desc, index_name) in index_tests {
            println!("Testing {}", query_desc);

            let start = Instant::now();
            match query_desc {
                "ORDER BY created_at DESC LIMIT 100" => {
                    let _results = sqlx::query_as::<_, ClipboardEntry>(
                        "SELECT * FROM clipboard_entries ORDER BY created_at DESC LIMIT 100",
                    )
                    .fetch_all(state.db.pool())
                    .await
                    .unwrap();
                }
                "WHERE content_hash = 'index_hash_25000'" => {
                    let _results = sqlx::query_as::<_, ClipboardEntry>(
                        "SELECT * FROM clipboard_entries WHERE content_hash = ?",
                    )
                    .bind("index_hash_25000")
                    .fetch_all(state.db.pool())
                    .await
                    .unwrap();
                }
                "WHERE created_at > ? ORDER BY created_at" => {
                    let mid_timestamp = num_entries as i64 / 2;
                    let _results = sqlx::query_as::<_, ClipboardEntry>(
                        "SELECT * FROM clipboard_entries WHERE created_at > ? ORDER BY created_at LIMIT 100"
                    )
                    .bind(mid_timestamp)
                    .fetch_all(state.db.pool())
                    .await
                    .unwrap();
                }
                _ => unreachable!(),
            }
            let duration = start.elapsed();

            println!(
                "  {} completed in {:?} (using {})",
                query_desc, duration, index_name
            );

            // With proper indexing, these queries should be fast even with large datasets
            assert!(
                duration.as_millis() < 100,
                "Indexed query should complete in < 100ms, got {:?}",
                duration
            );
        }

        // Test query without index (full table scan)
        println!("Testing unindexed query for comparison");
        let start = Instant::now();
        let _results = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE source_app LIKE '%App5%'",
        )
        .fetch_all(state.db.pool())
        .await
        .unwrap();
        let unindexed_duration = start.elapsed();

        println!("  Unindexed query completed in {:?}", unindexed_duration);

        // This might be slower as it's not indexed, but should still be reasonable
        assert!(
            unindexed_duration.as_millis() < 1000,
            "Even unindexed query should complete reasonably quickly"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_stress_test_rapid_insertions() {
        let (state, _temp_dir) = create_perf_test_env().await;

        println!("Starting stress test: rapid insertions");

        let batches = 100;
        let batch_size = 100;
        let total_entries = batches * batch_size;

        let start = Instant::now();

        for batch in 0..batches {
            let batch_start = Instant::now();

            for i in 0..batch_size {
                let global_id = batch * batch_size + i;
                let entry = ClipboardEntry::new(
                    ContentType::Text,
                    Some(format!("Stress test content {}", global_id)),
                    format!("stress_hash_{}", global_id),
                    Some(format!("StressApp{}", global_id % 10)),
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
            }

            let batch_duration = batch_start.elapsed();
            if batch % 10 == 0 {
                println!("  Completed batch {} in {:?}", batch, batch_duration);
            }

            // Verify we can still query efficiently during heavy insertion
            if batch % 20 == 0 {
                let query_start = Instant::now();
                let _recent = state
                    .get_clipboard_history(Some(10), None, None)
                    .await
                    .unwrap();
                let query_duration = query_start.elapsed();

                assert!(
                    query_duration.as_millis() < 200,
                    "Queries should remain fast during insertion stress test"
                );
            }
        }

        let total_duration = start.elapsed();
        let throughput = total_entries as f64 / total_duration.as_secs_f64();

        println!("Stress test completed:");
        println!(
            "  {} entries inserted in {:?}",
            total_entries, total_duration
        );
        println!("  Throughput: {:.2} entries/s", throughput);

        // Verify all entries were inserted
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .unwrap();

        assert_eq!(final_count, total_entries as i64);

        // Performance assertion
        assert!(
            throughput > 100.0,
            "Stress test throughput should be > 100 entries/s, got {:.2}",
            throughput
        );
    }
}
