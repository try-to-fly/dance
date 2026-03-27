#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct TestAppRoots {
    pub temp_dir: TempDir,
    pub config_root: PathBuf,
    pub data_root: PathBuf,
    pub cache_root: PathBuf,
    pub log_root: PathBuf,
}

impl TestAppRoots {
    fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn sqlite_url(&self, file_name: &str) -> String {
        let db_path = self.data_root.join(file_name);
        format!("sqlite:{}?mode=rwc", db_path.display())
    }

    pub fn seed_file(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root_path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory for seeded file");
        }
        fs::write(&path, bytes).expect("seed file into temporary app root");
        path
    }

    pub fn create_dir(&self, relative: &str) -> PathBuf {
        let path = self.root_path().join(relative);
        fs::create_dir_all(&path).expect("create directory inside temporary app root");
        path
    }
}

pub fn create_temp_app_roots() -> TestAppRoots {
    let temp_dir = TempDir::new().expect("create temporary app root");
    let config_root = temp_dir.path().join("config");
    let data_root = temp_dir.path().join("data");
    let cache_root = temp_dir.path().join("cache");
    let log_root = temp_dir.path().join("logs");

    for root in [&config_root, &data_root, &cache_root, &log_root] {
        fs::create_dir_all(root).expect("create temporary app subdirectory");
    }

    TestAppRoots {
        temp_dir,
        config_root,
        data_root,
        cache_root,
        log_root,
    }
}
