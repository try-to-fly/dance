use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Manager};

#[cfg_attr(not(test), allow(dead_code))]
pub const CAPT04_STORAGE_ROOTS_VERSION: &str = "capt04-storage-roots";

#[derive(Debug, Clone)]
pub struct AppPaths {
    config_root: PathBuf,
    data_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    cache_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    log_root: PathBuf,
}

impl AppPaths {
    pub fn from_app(app: &AppHandle) -> Result<Self> {
        let resolver = app.path();
        Ok(Self::from_roots(
            resolver.app_config_dir()?,
            resolver.app_data_dir()?,
            resolver.app_cache_dir()?,
            resolver.app_log_dir()?,
        ))
    }

    pub fn from_roots(
        config_dir: PathBuf,
        data_dir: PathBuf,
        cache_dir: PathBuf,
        log_dir: PathBuf,
    ) -> Self {
        Self {
            config_root: config_dir,
            data_root: data_dir,
            cache_root: cache_dir,
            log_root: log_dir,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_default_roots() -> Result<Self> {
        let base_dir =
            dirs::config_dir().ok_or_else(|| anyhow!("Unable to get config directory"))?;

        Ok(Self::from_roots(
            base_dir.join("clipboard-app").join("config"),
            base_dir.join("clipboard-app").join("data"),
            base_dir.join("clipboard-app").join("cache"),
            base_dir.join("clipboard-app").join("logs"),
        ))
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_root.join("config.json")
    }

    pub fn history_db_path(&self) -> PathBuf {
        self.data_root.join("clipboard.db")
    }

    pub fn image_assets_dir(&self) -> PathBuf {
        self.data_root.join("imgs")
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn icon_cache_dir(&self) -> PathBuf {
        self.cache_root.join("icons")
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn log_dir(&self) -> PathBuf {
        self.log_root.clone()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn migration_marker_path(&self) -> PathBuf {
        self.data_root
            .join("migrations")
            .join(format!("{CAPT04_STORAGE_ROOTS_VERSION}.json"))
    }

    pub fn resolve_relative_asset_path(&self, relative: &str) -> Result<PathBuf> {
        let normalized = Self::normalize_relative_path(relative)?;
        let mut components = normalized.components();

        match components.next() {
            Some(Component::Normal(first)) if first == OsStr::new("imgs") => {
                let mut resolved = self.image_assets_dir();
                let mut saw_tail = false;

                for component in components {
                    if let Component::Normal(part) = component {
                        resolved.push(part);
                        saw_tail = true;
                    }
                }

                if !saw_tail {
                    return Err(anyhow!(
                        "Relative asset path must target a file inside imgs/"
                    ));
                }

                Ok(resolved)
            }
            _ => Ok(self.data_root.join(normalized)),
        }
    }

    fn normalize_relative_path(relative: &str) -> Result<PathBuf> {
        let path = Path::new(relative);
        if path.is_absolute() {
            return Err(anyhow!("Absolute paths are not allowed"));
        }

        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => normalized.push(part),
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(anyhow!("Parent directory traversal is not allowed"));
                }
            }
        }

        if normalized.as_os_str().is_empty() {
            return Err(anyhow!("Relative asset path cannot be empty"));
        }

        Ok(normalized)
    }
}
