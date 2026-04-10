use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::{AppHandle, Manager};

#[cfg_attr(not(test), allow(dead_code))]
pub const CAPT04_STORAGE_ROOTS_VERSION: &str = "capt04-storage-roots";
pub const STORAGE_OWNER_IDENTIFIER_ENV: &str = "DANCE_STORAGE_OWNER_IDENTIFIER";

#[derive(Debug, Clone)]
pub struct AppPaths {
    config_root: PathBuf,
    data_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    cache_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    log_root: PathBuf,
    legacy_config_base_dir: Option<PathBuf>,
}

impl AppPaths {
    pub fn from_app(app: &AppHandle) -> Result<Self> {
        if let Some(storage_owner_identifier) = storage_owner_identifier_override() {
            return Self::from_storage_owner_identifier(&storage_owner_identifier);
        }

        let resolver = app.path();
        Ok(Self::from_roots(
            resolver.app_config_dir()?,
            resolver.app_data_dir()?,
            resolver.app_cache_dir()?,
            resolver.app_log_dir()?,
        ))
    }

    pub fn from_storage_owner_identifier(identifier: &str) -> Result<Self> {
        let identifier = identifier.trim();
        if identifier.is_empty() {
            return Err(anyhow!("Storage owner identifier cannot be empty"));
        }

        let config_root = dirs::config_dir()
            .ok_or_else(|| anyhow!("Unable to get config directory"))?
            .join(identifier);
        let data_root = dirs::data_dir()
            .ok_or_else(|| anyhow!("Unable to get data directory"))?
            .join(identifier);
        let cache_root = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Unable to get cache directory"))?
            .join(identifier);
        let log_root = app_log_root_for_identifier(identifier)?;

        Ok(Self::from_roots(
            config_root,
            data_root,
            cache_root,
            log_root,
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
            legacy_config_base_dir: None,
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

    pub fn migrate_legacy_roots(&self) -> Result<()> {
        let marker_path = self.migration_marker_path();
        if marker_path.exists() {
            return Ok(());
        }

        fs::create_dir_all(&self.config_root)?;
        fs::create_dir_all(&self.data_root)?;
        fs::create_dir_all(self.image_assets_dir())?;
        fs::create_dir_all(self.icon_cache_dir())?;
        fs::create_dir_all(self.log_dir())?;

        let legacy_base_dir = match &self.legacy_config_base_dir {
            Some(path) => path.clone(),
            None => dirs::config_dir().ok_or_else(|| anyhow!("Unable to get config directory"))?,
        };

        let dance_root = legacy_base_dir.join("dance");
        let clipboard_app_root = legacy_base_dir.join("clipboard-app");
        let dance_config = dance_root.join("config.json");
        let dance_db = dance_root.join("clipboard.db");
        let legacy_images = clipboard_app_root.join("imgs");
        let legacy_icons = clipboard_app_root.join("icons");

        let mut migrated_from = Vec::new();

        if dance_config.exists() || dance_db.exists() {
            migrated_from.push(dance_root.to_string_lossy().to_string());
            Self::copy_file_if_missing(&dance_config, &self.config_file_path())?;
            Self::copy_file_if_missing(&dance_db, &self.history_db_path())?;
        }

        if legacy_images.exists() || legacy_icons.exists() {
            migrated_from.push(clipboard_app_root.to_string_lossy().to_string());
            Self::copy_directory_contents_if_missing(&legacy_images, &self.image_assets_dir())?;
            Self::copy_directory_contents_if_missing(&legacy_icons, &self.icon_cache_dir())?;
        }

        if migrated_from.is_empty() {
            return Ok(());
        }

        if let Some(parent) = marker_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let marker = json!({
            "version": CAPT04_STORAGE_ROOTS_VERSION,
            "migrated_from": migrated_from,
            "completed_at": Utc::now().to_rfc3339(),
        });

        fs::write(marker_path, serde_json::to_vec_pretty(&marker)?)?;

        Ok(())
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

    fn copy_file_if_missing(source: &Path, target: &Path) -> Result<()> {
        if !source.exists() || target.exists() {
            return Ok(());
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(source, target)?;
        Ok(())
    }

    fn copy_directory_contents_if_missing(source_dir: &Path, target_dir: &Path) -> Result<()> {
        if !source_dir.exists() {
            return Ok(());
        }

        fs::create_dir_all(target_dir)?;

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            let source_path = entry.path();
            let target_path = target_dir.join(entry.file_name());

            if entry.file_type()?.is_dir() {
                Self::copy_directory_contents_if_missing(&source_path, &target_path)?;
            } else {
                Self::copy_file_if_missing(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    #[cfg(test)]
    pub fn with_legacy_config_base_for_tests(mut self, base_dir: PathBuf) -> Self {
        self.legacy_config_base_dir = Some(base_dir);
        self
    }
}

fn storage_owner_identifier_override() -> Option<String> {
    std::env::var(STORAGE_OWNER_IDENTIFIER_ENV)
        .ok()
        .or_else(|| option_env!("DANCE_STORAGE_OWNER_IDENTIFIER").map(|value| value.to_string()))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "macos")]
fn app_log_root_for_identifier(identifier: &str) -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("Unable to get home directory"))?
        .join("Library/Logs")
        .join(identifier))
}

#[cfg(not(target_os = "macos"))]
fn app_log_root_for_identifier(identifier: &str) -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Unable to get local data directory"))?
        .join(identifier)
        .join("logs"))
}
