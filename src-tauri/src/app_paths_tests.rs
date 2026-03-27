use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::app_paths::AppPaths;
use crate::clipboard::ContentProcessor;
use crate::config::ConfigManager;
use crate::database::Database;
use crate::test_support::create_temp_app_roots;

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
#[ignore = "implemented in 01-02"]
fn test_app_paths_migrate_legacy_roots() {}
