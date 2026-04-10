use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app_paths::AppPaths;
use crate::clipboard::ContentProcessor;
use crate::commands::{app_log_file_path, clear_log_file_in, read_log_content_in};
use crate::config::ConfigManager;
use crate::database::Database;
use crate::test_support::create_temp_app_roots;
use serde_json::Value;

#[test]
fn test_app_paths_temp_roots_are_isolated() {
    let roots = create_temp_app_roots();
    let temp_root = roots.temp_dir.path().to_path_buf();

    for root in [
        &roots.config_root,
        &roots.data_root,
        &roots.cache_root,
        &roots.log_root,
    ] {
        assert!(root.starts_with(&temp_root));
        assert!(root.is_dir());
    }

    let distinct_roots: HashSet<PathBuf> = [
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    ]
    .into_iter()
    .collect();

    assert_eq!(distinct_roots.len(), 4);
}

#[test]
fn test_app_paths_from_roots_uses_fixed_layout() {
    let roots = create_temp_app_roots();
    let paths = AppPaths::from_roots(
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    );

    assert_eq!(
        paths.config_file_path(),
        roots.config_root.join("config.json")
    );
    assert_eq!(
        paths.history_db_path(),
        roots.data_root.join("clipboard.db")
    );
    assert_eq!(paths.image_assets_dir(), roots.data_root.join("imgs"));
    assert_eq!(paths.icon_cache_dir(), roots.cache_root.join("icons"));
    assert_eq!(paths.log_dir(), roots.log_root);
    assert_eq!(
        paths.migration_marker_path(),
        roots
            .data_root
            .join("migrations")
            .join("capt04-storage-roots.json")
    );
}

#[test]
fn test_app_paths_from_storage_owner_identifier_uses_bundle_scoped_layout() {
    let paths = AppPaths::from_storage_owner_identifier("com.dance.app").unwrap();

    assert!(paths
        .config_file_path()
        .ends_with(PathBuf::from("com.dance.app").join("config.json")));
    assert!(paths
        .history_db_path()
        .ends_with(PathBuf::from("com.dance.app").join("clipboard.db")));
    assert!(paths
        .image_assets_dir()
        .ends_with(PathBuf::from("com.dance.app").join("imgs")));
}

#[tokio::test]
async fn test_app_paths_injected_roots_drive_core_modules() {
    let roots = create_temp_app_roots();
    let paths = Arc::new(AppPaths::from_roots(
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    ));

    let database = Database::new_in(paths.clone()).await.unwrap();
    let config_manager = ConfigManager::new_in(paths.clone()).await.unwrap();
    let processor = ContentProcessor::new_in(paths.clone()).unwrap();

    assert!(database.db_path().starts_with(&roots.data_root));
    assert_eq!(database.db_path(), paths.history_db_path());
    assert!(config_manager.config_path().starts_with(&roots.config_root));
    assert_eq!(config_manager.config_path(), paths.config_file_path());
    assert!(processor.imgs_dir().starts_with(&roots.data_root));
    assert_eq!(processor.imgs_dir(), paths.image_assets_dir());

    assert!(paths.config_file_path().is_file());
    assert!(paths.history_db_path().is_file());
    assert!(paths.image_assets_dir().is_dir());
}

#[test]
fn test_app_paths_migrate_legacy_roots() {
    let roots = create_temp_app_roots();
    let paths = AppPaths::from_roots(
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    )
    .with_legacy_config_base_for_tests(roots.temp_dir.path().to_path_buf());

    roots.seed_file("dance/config.json", br#"{"language":"legacy"}"#);
    roots.seed_file("dance/clipboard.db", b"legacy-db");
    roots.seed_file("clipboard-app/imgs/legacy-image.png", b"legacy-image");
    roots.seed_file("clipboard-app/icons/legacy-icon.png", b"legacy-icon");

    paths.migrate_legacy_roots().unwrap();

    assert_eq!(
        fs::read(paths.config_file_path()).unwrap(),
        br#"{"language":"legacy"}"#
    );
    assert_eq!(fs::read(paths.history_db_path()).unwrap(), b"legacy-db");
    assert_eq!(
        fs::read(paths.image_assets_dir().join("legacy-image.png")).unwrap(),
        b"legacy-image"
    );
    assert_eq!(
        fs::read(paths.icon_cache_dir().join("legacy-icon.png")).unwrap(),
        b"legacy-icon"
    );

    let marker: Value =
        serde_json::from_slice(&fs::read(paths.migration_marker_path()).unwrap()).unwrap();
    assert_eq!(marker["version"], "capt04-storage-roots");
    assert!(marker["completed_at"].as_str().is_some());
    let migrated_from = marker["migrated_from"].as_array().unwrap();
    assert_eq!(migrated_from.len(), 2);

    let second_roots = create_temp_app_roots();
    let second_paths = AppPaths::from_roots(
        second_roots.config_root.clone(),
        second_roots.data_root.clone(),
        second_roots.cache_root.clone(),
        second_roots.log_root.clone(),
    )
    .with_legacy_config_base_for_tests(second_roots.temp_dir.path().to_path_buf());

    second_roots.seed_file("dance/config.json", br#"{"language":"legacy"}"#);
    second_roots.seed_file("dance/clipboard.db", b"legacy-db");
    second_roots.seed_file(
        "clipboard-app/imgs/conflict-image.png",
        b"legacy-conflict-image",
    );
    second_roots.seed_file(
        "clipboard-app/icons/conflict-icon.png",
        b"legacy-conflict-icon",
    );
    second_roots.seed_file("config/config.json", br#"{"language":"current"}"#);
    second_roots.seed_file("data/clipboard.db", b"current-db");
    second_roots.seed_file("data/imgs/conflict-image.png", b"current-image");
    second_roots.seed_file("cache/icons/conflict-icon.png", b"current-icon");

    second_paths.migrate_legacy_roots().unwrap();
    second_paths.migrate_legacy_roots().unwrap();

    assert_eq!(
        fs::read(second_paths.config_file_path()).unwrap(),
        br#"{"language":"current"}"#
    );
    assert_eq!(
        fs::read(second_paths.history_db_path()).unwrap(),
        b"current-db"
    );
    assert_eq!(
        fs::read(second_paths.image_assets_dir().join("conflict-image.png")).unwrap(),
        b"current-image"
    );
    assert_eq!(
        fs::read(second_paths.icon_cache_dir().join("conflict-icon.png")).unwrap(),
        b"current-icon"
    );
    assert!(second_paths.migration_marker_path().is_file());
}

#[test]
fn test_app_paths_resolve_relative_asset_path_for_nested_imgs_assets() {
    let roots = create_temp_app_roots();
    let paths = AppPaths::from_roots(
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    );

    let resolved = paths
        .resolve_relative_asset_path("imgs/nested/example.png")
        .unwrap();

    assert_eq!(
        resolved,
        roots
            .data_root
            .join("imgs")
            .join("nested")
            .join("example.png")
    );
}

#[test]
fn test_app_paths_log_commands_follow_log_dir() {
    let roots = create_temp_app_roots();
    let paths = AppPaths::from_roots(
        roots.config_root.clone(),
        roots.data_root.clone(),
        roots.cache_root.clone(),
        roots.log_root.clone(),
    );

    roots.seed_file("logs/clipboard-app.log", b"line1\nline2");

    let log_path = app_log_file_path(&paths);
    assert_eq!(log_path, roots.log_root.join("clipboard-app.log"));
    assert_eq!(read_log_content_in(&paths).unwrap(), "line1\nline2");

    clear_log_file_in(&paths).unwrap();
    assert_eq!(fs::read_to_string(&log_path).unwrap(), "");
    assert_eq!(read_log_content_in(&paths).unwrap(), "");

    clear_log_file_in(&paths).unwrap();
    assert_eq!(read_log_content_in(&paths).unwrap(), "");
}
