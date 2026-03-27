use std::collections::HashSet;
use std::path::PathBuf;

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
#[ignore = "implemented in 01-02"]
fn test_app_paths_migrate_legacy_roots() {}
