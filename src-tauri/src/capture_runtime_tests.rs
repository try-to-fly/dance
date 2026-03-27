#[cfg(test)]
mod capture_runtime_tests {
    use crate::app_paths::AppPaths;
    use crate::models::{ClipboardEntry, ContentType};
    use crate::state::AppState;
    use crate::test_support::{create_temp_app_roots, TestAppRoots};
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
    #[ignore = "implemented in 01-03"]
    async fn test_capture_runtime_single_worker_and_suppression() {}

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
}
