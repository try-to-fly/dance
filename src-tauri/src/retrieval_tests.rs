#![cfg(test)]

use crate::analysis::{upsert_entry_analysis, TextAnalysisService};
use crate::app_paths::AppPaths;
use crate::database::Database;
use crate::models::{ClipboardEntry, ClipboardRetrievalMatchKind, ContentType};
use crate::retrieval::{upsert_entry_search_document, ClipboardHistoryQuery};
use crate::state::AppState;
use crate::test_support::{create_temp_app_roots, TestAppRoots};
use chrono::{Duration, Utc};
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
        capture_runtime: Arc::new(tokio::sync::RwLock::new(None)),
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

async fn insert_text_entry(
    state: &AppState,
    content: &str,
    source_app: Option<&str>,
    is_favorite: bool,
    created_at: Option<i64>,
) -> ClipboardEntry {
    let snapshot = TextAnalysisService::new().analyze(content);
    let mut entry = ClipboardEntry::new(
        ContentType::Text,
        Some(content.to_string()),
        format!("retrieval_hash_{}", uuid::Uuid::new_v4()),
        source_app.map(|value| value.to_string()),
        None,
    );
    if let Some(created_at) = created_at {
        entry.created_at = created_at;
    }
    entry.is_favorite = is_favorite;
    entry.attach_analysis(snapshot.clone());

    sqlx::query(
        r#"
        INSERT INTO clipboard_entries
        (id, content_hash, content_type, content_data, source_app, created_at, copy_count, file_path, is_favorite, content_subtype, metadata, app_bundle_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&entry.id)
    .bind(&entry.content_hash)
    .bind(&entry.content_type)
    .bind(&entry.content_data)
    .bind(&entry.source_app)
    .bind(entry.created_at)
    .bind(entry.copy_count)
    .bind(&entry.file_path)
    .bind(entry.is_favorite)
    .bind(&entry.content_subtype)
    .bind(&entry.metadata)
    .bind(&entry.app_bundle_id)
    .execute(state.db.pool())
    .await
    .unwrap();

    upsert_entry_analysis(state.db.pool(), &entry.id, &entry.content_hash, &snapshot)
        .await
        .unwrap();

    let mut connection = state.db.pool().acquire().await.unwrap();
    upsert_entry_search_document(&mut connection, &entry)
        .await
        .unwrap();

    entry
}

#[tokio::test]
async fn retrieval_search_matches_structured_url_tokens_and_fuzzy_fragments() {
    let (state, _temp_dir) = create_test_state().await;

    let url_entry = insert_text_entry(
        &state,
        "https://api.example.com/docs/install?debug=1",
        Some("Arc"),
        false,
        None,
    )
    .await;
    let _json_entry = insert_text_entry(
        &state,
        r#"{"deploy":{"service":"clipboard"}}"#,
        Some("Warp"),
        false,
        None,
    )
    .await;

    let host_results = state
        .search_clipboard_history(ClipboardHistoryQuery {
            text: Some("api.example".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        host_results.first().map(|entry| entry.id.as_str()),
        Some(url_entry.id.as_str())
    );
    assert_eq!(
        host_results[0]
            .retrieval
            .as_ref()
            .map(|value| &value.match_kind),
        Some(&ClipboardRetrievalMatchKind::UrlHost)
    );

    let fuzzy_query_results = state
        .search_clipboard_history(ClipboardHistoryQuery {
            text: Some("dbg".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        fuzzy_query_results.first().map(|entry| entry.id.as_str()),
        Some(url_entry.id.as_str())
    );
    assert_eq!(
        fuzzy_query_results[0]
            .retrieval
            .as_ref()
            .map(|value| &value.match_kind),
        Some(&ClipboardRetrievalMatchKind::UrlQuery)
    );
}

#[tokio::test]
async fn retrieval_search_matches_json_key_paths_and_color_formats() {
    let (state, _temp_dir) = create_test_state().await;

    let json_entry = insert_text_entry(
        &state,
        r#"{"deploy":{"service":"clipboard","region":"apac"}}"#,
        Some("Warp"),
        true,
        None,
    )
    .await;
    let color_entry = insert_text_entry(&state, "#0EA5E9", Some("Figma"), false, None).await;

    let json_results = state
        .search_clipboard_history(ClipboardHistoryQuery {
            text: Some("deploy.service".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        json_results.first().map(|entry| entry.id.as_str()),
        Some(json_entry.id.as_str())
    );
    assert_eq!(
        json_results[0]
            .retrieval
            .as_ref()
            .map(|value| &value.match_kind),
        Some(&ClipboardRetrievalMatchKind::JsonKey)
    );

    let color_results = state
        .search_clipboard_history(ClipboardHistoryQuery {
            text: Some("#0ea5e9".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(
        color_results.first().map(|entry| entry.id.as_str()),
        Some(color_entry.id.as_str())
    );
    assert_eq!(
        color_results[0]
            .retrieval
            .as_ref()
            .map(|value| &value.match_kind),
        Some(&ClipboardRetrievalMatchKind::ColorValue)
    );
}

#[tokio::test]
async fn retrieval_query_applies_type_source_favorite_and_recency_filters() {
    let (state, _temp_dir) = create_test_state().await;
    let now = Utc::now().timestamp_millis();

    let matching_entry = insert_text_entry(
        &state,
        r#"{"deploy":{"service":"clipboard"}}"#,
        Some("Warp"),
        true,
        Some(now - Duration::hours(6).num_milliseconds()),
    )
    .await;
    let _wrong_source = insert_text_entry(
        &state,
        r#"{"deploy":{"service":"clipboard"}}"#,
        Some("Arc"),
        true,
        Some(now - Duration::hours(6).num_milliseconds()),
    )
    .await;
    let _stale_entry = insert_text_entry(
        &state,
        r#"{"deploy":{"service":"clipboard"}}"#,
        Some("Warp"),
        true,
        Some(now - Duration::days(40).num_milliseconds()),
    )
    .await;
    let _wrong_type = insert_text_entry(
        &state,
        "pnpm exec vitest run retrieval-tests",
        Some("Warp"),
        true,
        Some(now - Duration::hours(2).num_milliseconds()),
    )
    .await;

    let filtered = state
        .search_clipboard_history(ClipboardHistoryQuery {
            selected_type: Some("text:json".to_string()),
            source_app: Some("warp".to_string()),
            favorites_only: Some(true),
            recency_days: Some(7),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, matching_entry.id);
}

#[tokio::test]
async fn list_clipboard_source_apps_returns_recent_distinct_apps() {
    let (state, _temp_dir) = create_test_state().await;
    let now = Utc::now().timestamp_millis();

    let _older_arc = insert_text_entry(
        &state,
        "https://archive.example.com/release-notes",
        Some("Arc"),
        false,
        Some(now - Duration::days(2).num_milliseconds()),
    )
    .await;
    let _newer_warp = insert_text_entry(
        &state,
        "pnpm exec vitest run src-tauri/src/retrieval_tests.rs",
        Some("Warp"),
        false,
        Some(now - Duration::hours(1).num_milliseconds()),
    )
    .await;

    let source_apps = state.list_clipboard_source_apps(Some(10)).await.unwrap();

    assert_eq!(source_apps, vec!["Warp".to_string(), "Arc".to_string()]);
}
