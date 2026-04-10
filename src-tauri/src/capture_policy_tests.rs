use crate::analysis::TextAnalysisService;
use crate::app_paths::AppPaths;
use crate::capture::{
    calculate_content_hash, decide_capture, CaptureDisposition, PasteboardMarkers,
};
use crate::clipboard::ClipboardMonitor;
use crate::state::AppState;
use crate::test_support::{create_temp_app_roots, TestAppRoots};
use sqlx::query_scalar;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

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

#[test]
fn test_capture_policy_marker_matrix() {
    let baseline = PasteboardMarkers::default();

    assert_eq!(
        decide_capture(&baseline, false, false, true),
        CaptureDisposition::Persist
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_transient: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_concealed: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_remote: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, true, false, true),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, false, true, true),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, false, false, false),
        CaptureDisposition::Skip
    );
}

#[tokio::test]
async fn test_capture_policy_current_only_is_non_persistent_in_v1() {
    let (state, _roots) = create_test_state().await;
    let monitor = ClipboardMonitor::new(
        state.tx.clone(),
        Arc::clone(&state.processor),
        Arc::clone(&state.config_manager),
        "com.dance.app",
    )
    .expect("create clipboard monitor");
    let mut rx = state.tx.subscribe();
    let last_observed_hash = Arc::new(Mutex::new(None));
    let suppression_registry = Arc::new(Mutex::new(Vec::new()));
    let auto_generated = PasteboardMarkers {
        is_auto_generated: true,
        ..PasteboardMarkers::default()
    };
    let detection_calls = Arc::new(AtomicUsize::new(0));
    let detection_calls_for_detector = Arc::clone(&detection_calls);
    let analysis_service = TextAnalysisService::new();
    let content = "auto generated text";

    let disposition = monitor
        .process_text_capture_for_test(
            &last_observed_hash,
            &suppression_registry,
            None,
            &auto_generated,
            content,
            move |text| {
                detection_calls_for_detector.fetch_add(1, Ordering::SeqCst);
                analysis_service.analyze(text)
            },
        )
        .await
        .expect("process auto-generated clipboard text");

    let content_hash = calculate_content_hash(content.as_bytes());
    assert_eq!(disposition, CaptureDisposition::CurrentOnly);
    assert_eq!(detection_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        last_observed_hash.lock().await.clone(),
        Some(content_hash.clone())
    );
    assert!(timeout(Duration::from_millis(100), rx.recv())
        .await
        .is_err());
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM clipboard_entries")
            .fetch_one(state.db.pool())
            .await
            .expect("count clipboard entries"),
        0
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn test_capture_policy_non_macos_markers_are_noop() {
    let markers = crate::capture::macos_markers::read_pasteboard_markers();
    assert_eq!(markers, PasteboardMarkers::default());
    assert_eq!(
        decide_capture(&markers, false, false, true),
        CaptureDisposition::Persist
    );
}
