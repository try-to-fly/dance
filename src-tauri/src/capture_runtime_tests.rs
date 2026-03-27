#[cfg(test)]
mod capture_runtime_tests {
    use crate::app_paths::AppPaths;
    use crate::database::Database;
    use crate::models::{ClipboardEntry, ContentType};
    use crate::state::AppState;
    use crate::test_support::{create_temp_app_roots, TestAppRoots};
    use sqlx::{sqlite::SqlitePool, Row};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    async fn create_test_state() -> (Arc<AppState>, TestAppRoots) {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let state = AppState::new(paths).await.expect("create test app state");
        (Arc::new(state), roots)
    }

    fn create_text_entry(content_hash: &str, content: &str) -> ClipboardEntry {
        ClipboardEntry::new(
            ContentType::Text,
            Some(content.to_string()),
            content_hash.to_string(),
            Some("CaptureRuntimeTests".to_string()),
            None,
        )
    }

    async fn count_entries(state: &AppState) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .expect("count clipboard entries")
    }

    async fn fetch_entry_by_hash(state: &AppState, content_hash: &str) -> ClipboardEntry {
        sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_hash = ?",
        )
        .bind(content_hash)
        .fetch_one(state.db.pool())
        .await
        .expect("fetch clipboard entry by content hash")
    }

    #[tokio::test]
    async fn test_capture_runtime_stop_cancels_tasks() {
        let (state, _roots) = create_test_state().await;

        state.start_monitoring().await.expect("start monitoring");
        state.stop_monitoring().await.expect("stop monitoring");

        let sent = state.tx.send(create_text_entry(
            "stop-cancel-hash",
            "should not persist after stop",
        ));
        assert!(sent.is_ok(), "send synthetic clipboard entry");

        sleep(Duration::from_millis(250)).await;

        assert_eq!(count_entries(&state).await, 0);
    }

    #[tokio::test]
    async fn test_capture_runtime_single_worker_and_suppression() {
        let (state, _roots) = create_test_state().await;
        state.start_monitoring().await.expect("start monitoring");

        let content_hash = "runtime-upsert-contract-hash";
        let mut first_entry = create_text_entry(content_hash, "same payload");
        first_entry.created_at = 100;
        first_entry.source_app = Some("Initial App".to_string());
        first_entry.content_subtype = None;
        first_entry.metadata = None;
        first_entry.app_bundle_id = None;
        first_entry.file_path = None;

        let mut second_entry = create_text_entry(content_hash, "same payload");
        second_entry.created_at = 200;
        second_entry.source_app = None;
        second_entry.content_subtype = Some("json".to_string());
        second_entry.metadata = Some(r#"{"merged":true}"#.to_string());
        second_entry.app_bundle_id = Some("com.example.merged".to_string());
        second_entry.file_path = Some("imgs/merged.png".to_string());

        assert!(state.tx.send(first_entry).is_ok());
        assert!(state.tx.send(second_entry).is_ok());

        sleep(Duration::from_millis(300)).await;

        let stored_entry = fetch_entry_by_hash(&state, content_hash).await;
        assert_eq!(count_entries(&state).await, 1);
        assert_eq!(stored_entry.copy_count, 2);
        assert_eq!(stored_entry.created_at, 200);
        assert_eq!(stored_entry.source_app, Some("Initial App".to_string()));
        assert_eq!(stored_entry.content_subtype, Some("json".to_string()));
        assert_eq!(
            stored_entry.metadata,
            Some(r#"{"merged":true}"#.to_string())
        );
        assert_eq!(
            stored_entry.app_bundle_id,
            Some("com.example.merged".to_string())
        );
        assert_eq!(stored_entry.file_path, Some("imgs/merged.png".to_string()));
    }

    #[tokio::test]
    async fn test_capture_runtime_restart_is_single_owner() {
        let (state, _roots) = create_test_state().await;
        let baseline_receivers = state.tx.receiver_count();

        state.start_monitoring().await.expect("start monitoring");
        assert_eq!(state.tx.receiver_count(), baseline_receivers + 1);

        state.stop_monitoring().await.expect("stop monitoring");
        assert_eq!(state.tx.receiver_count(), baseline_receivers);

        state.start_monitoring().await.expect("restart monitoring");
        state
            .start_monitoring()
            .await
            .expect("second start should be a no-op");
        assert_eq!(state.tx.receiver_count(), baseline_receivers + 1);
    }

    #[tokio::test]
    async fn test_capture_runtime_dedupe_migration_merges_existing_duplicates() {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let db_path = paths.history_db_path();
        let parent = db_path.parent().expect("history db parent directory");
        std::fs::create_dir_all(parent).expect("create history db directory");

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let seed_pool = SqlitePool::connect(&database_url)
            .await
            .expect("open seed database");

        sqlx::query(
            r#"
            CREATE TABLE clipboard_entries (
                id TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_data TEXT,
                source_app TEXT,
                created_at INTEGER NOT NULL,
                copy_count INTEGER DEFAULT 1,
                file_path TEXT,
                is_favorite INTEGER DEFAULT 0,
                content_subtype TEXT,
                metadata TEXT,
                app_bundle_id TEXT
            )
            "#,
        )
        .execute(&seed_pool)
        .await
        .expect("create brownfield clipboard_entries table");

        for (
            id,
            created_at,
            copy_count,
            source_app,
            content_subtype,
            metadata,
            app_bundle_id,
            file_path,
        ) in [
            (
                "a-1",
                100_i64,
                2_i32,
                Some("Oldest App"),
                Some("plain_text"),
                Some(r#"{"source":"oldest"}"#),
                Some("bundle.oldest"),
                Some("imgs/oldest.png"),
            ),
            (
                "b-2",
                300_i64,
                4_i32,
                Some("Tie Loser App"),
                None,
                Some(r#"{"source":"tie"}"#),
                Some("bundle.tie"),
                None,
            ),
            (
                "c-3",
                300_i64,
                1_i32,
                None,
                Some("json"),
                None,
                None,
                Some("imgs/survivor.png"),
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO clipboard_entries
                (id, content_hash, content_type, content_data, source_app, created_at, copy_count, file_path, is_favorite, content_subtype, metadata, app_bundle_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(id)
            .bind("dedupe-migration-hash")
            .bind("text")
            .bind(Some("same payload"))
            .bind(source_app)
            .bind(created_at)
            .bind(copy_count)
            .bind(file_path)
            .bind(0_i32)
            .bind(content_subtype)
            .bind(metadata)
            .bind(app_bundle_id)
            .execute(&seed_pool)
            .await
            .expect("seed duplicate brownfield row");
        }

        drop(seed_pool);

        let db = Database::new_in(paths)
            .await
            .expect("reopen migrated database");

        let rows = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries WHERE content_hash = ?",
        )
        .bind("dedupe-migration-hash")
        .fetch_all(db.pool())
        .await
        .expect("load migrated rows");

        assert_eq!(rows.len(), 1);
        let stored_entry = &rows[0];
        assert_eq!(stored_entry.id, "c-3");
        assert_eq!(stored_entry.copy_count, 7);
        assert_eq!(stored_entry.created_at, 300);
        assert_eq!(stored_entry.source_app, Some("Tie Loser App".to_string()));
        assert_eq!(stored_entry.content_subtype, Some("json".to_string()));
        assert_eq!(
            stored_entry.metadata,
            Some(r#"{"source":"tie"}"#.to_string())
        );
        assert_eq!(stored_entry.app_bundle_id, Some("bundle.tie".to_string()));
        assert_eq!(
            stored_entry.file_path,
            Some("imgs/survivor.png".to_string())
        );

        let index_row =
            sqlx::query("SELECT name FROM sqlite_master WHERE type = 'index' AND name = ?")
                .bind("idx_clipboard_entries_content_hash_unique")
                .fetch_one(db.pool())
                .await
                .expect("load content hash unique index");

        assert_eq!(
            index_row.get::<String, _>("name"),
            "idx_clipboard_entries_content_hash_unique"
        );
    }
}
