use crate::app_paths::AppPaths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedApp {
    pub name: String,
    pub bundle_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub text: TextConfig,
    pub image: ImageConfig,
    #[serde(default)]
    pub excluded_apps: Vec<String>, // Keep for backward compatibility
    #[serde(default)]
    pub excluded_apps_v2: Vec<ExcludedApp>, // New format with name and bundle_id
    pub global_shortcut: String,
    pub auto_startup: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub last_update_check: Option<String>, // ISO 8601 date string
    #[serde(default = "default_language")]
    pub language: String, // Language preference (zh or en)
    #[serde(default)]
    pub llm: LlmConfig,
}

fn default_language() -> String {
    "system".to_string()
}

fn default_llm_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_llm_model() -> String {
    "gpt-4.1-mini".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ExpiryOption {
    Days(u32),
    #[default]
    Never,
}

impl ExpiryOption {
    pub fn as_days(&self) -> Option<u32> {
        match self {
            ExpiryOption::Days(days) => Some(*days),
            ExpiryOption::Never => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    pub max_size_mb: f64,
    pub expiry: ExpiryOption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub expiry: ExpiryOption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_llm_base_url")]
    pub base_url: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_llm_base_url(),
            model: default_llm_model(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            text: TextConfig {
                max_size_mb: 1.0,
                expiry: ExpiryOption::Never,
            },
            image: ImageConfig {
                expiry: ExpiryOption::Never,
            },
            excluded_apps: vec![], // Legacy format, migrate to excluded_apps_v2
            excluded_apps_v2: vec![
                ExcludedApp {
                    name: "1Password 7 - Password Manager".to_string(),
                    bundle_id: "com.1password.1password7".to_string(),
                },
                ExcludedApp {
                    name: "Keychain Access".to_string(),
                    bundle_id: "com.apple.keychainaccess".to_string(),
                },
            ],
            global_shortcut: "CmdOrCtrl+Shift+V".to_string(),
            auto_startup: false,
            auto_update: true,
            last_update_check: None,
            language: default_language(),
            llm: LlmConfig::default(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    pub config: AppConfig,
}

impl ConfigManager {
    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn new() -> Result<Self> {
        Self::new_in(Arc::new(AppPaths::from_default_roots()?)).await
    }

    pub async fn new_in(paths: Arc<AppPaths>) -> Result<Self> {
        let config_path = paths.config_file_path();

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let config = if config_path.exists() {
            Self::load_config(&config_path).await?
        } else {
            let default_config = AppConfig::default();
            Self::save_config(&config_path, &default_config).await?;
            default_config
        };

        // Migrate old excluded_apps format to new format if needed
        let mut migrated_config = config.clone();
        let needs_migration = !migrated_config.excluded_apps.is_empty()
            && migrated_config.excluded_apps_v2.is_empty();

        if needs_migration {
            log::info!("Migrating excluded apps to new format...");
            migrated_config.excluded_apps_v2 =
                Self::migrate_excluded_apps(&migrated_config.excluded_apps).await;
            migrated_config.excluded_apps.clear(); // Clear old format
        }

        // Always save the config after loading to ensure it's in the latest format
        if config_path.exists() || needs_migration {
            Self::save_config(&config_path, &migrated_config).await?;
        }

        Ok(Self {
            config_path,
            config: migrated_config,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub async fn update_config(&mut self, new_config: AppConfig) -> Result<()> {
        self.config = new_config.clone();
        Self::save_config(&self.config_path, &new_config).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn reload(&mut self) -> Result<()> {
        self.config = Self::load_config(&self.config_path).await?;
        Ok(())
    }

    async fn load_config(path: &PathBuf) -> Result<AppConfig> {
        let content = fs::read_to_string(path).await?;

        // Try to parse as new format first
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => Ok(config),
            Err(_) => {
                // Try to migrate from old format
                log::info!("Migrating config from old format...");
                Self::migrate_old_config(&content).await
            }
        }
    }

    async fn save_config(path: &PathBuf, config: &AppConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content).await?;
        Ok(())
    }

    pub fn is_app_excluded(&self, bundle_id: &str) -> bool {
        // Check both old and new format for backward compatibility
        self.config
            .excluded_apps
            .iter()
            .any(|excluded| excluded == bundle_id)
            || self
                .config
                .excluded_apps_v2
                .iter()
                .any(|excluded| excluded.bundle_id == bundle_id)
    }

    pub fn is_text_size_valid(&self, content: &str) -> bool {
        let size_bytes = content.len() as f64;
        let size_mb = size_bytes / (1024.0 * 1024.0);
        size_mb <= self.config.text.max_size_mb
    }

    async fn migrate_excluded_apps(old_excluded_apps: &[String]) -> Vec<ExcludedApp> {
        use crate::utils::app_list::AppListManager;

        let mut migrated_apps = Vec::new();

        // Try to get app names from the system
        if let Ok(installed_apps) = AppListManager::get_installed_applications() {
            for bundle_id in old_excluded_apps {
                if let Some(app) = installed_apps
                    .iter()
                    .find(|app| &app.bundle_id == bundle_id)
                {
                    migrated_apps.push(ExcludedApp {
                        name: app.name.clone(),
                        bundle_id: app.bundle_id.clone(),
                    });
                } else {
                    // Fallback to just using bundle_id as name
                    migrated_apps.push(ExcludedApp {
                        name: bundle_id.clone(),
                        bundle_id: bundle_id.clone(),
                    });
                }
            }
        } else {
            // Fallback: use bundle_ids as names
            for bundle_id in old_excluded_apps {
                migrated_apps.push(ExcludedApp {
                    name: bundle_id.clone(),
                    bundle_id: bundle_id.clone(),
                });
            }
        }

        migrated_apps
    }

    async fn migrate_old_config(content: &str) -> Result<AppConfig> {
        // Parse as generic JSON first
        let mut json: Value = serde_json::from_str(content)?;

        // Migrate text.expiry_days to text.expiry
        if let Some(text) = json.get_mut("text") {
            if let Some(expiry_days) = text.get("expiry_days").and_then(|v| v.as_u64()) {
                text.as_object_mut().unwrap().remove("expiry_days");
                if expiry_days == 0 {
                    text.as_object_mut().unwrap().insert(
                        "expiry".to_string(),
                        serde_json::Value::String("Never".to_string()),
                    );
                } else {
                    text.as_object_mut().unwrap().insert(
                        "expiry".to_string(),
                        serde_json::json!({"Days": expiry_days}),
                    );
                }
            }
        }

        // Migrate image.expiry_days to image.expiry
        if let Some(image) = json.get_mut("image") {
            if let Some(expiry_days) = image.get("expiry_days").and_then(|v| v.as_u64()) {
                image.as_object_mut().unwrap().remove("expiry_days");
                if expiry_days == 0 {
                    image.as_object_mut().unwrap().insert(
                        "expiry".to_string(),
                        serde_json::Value::String("Never".to_string()),
                    );
                } else {
                    image.as_object_mut().unwrap().insert(
                        "expiry".to_string(),
                        serde_json::json!({"Days": expiry_days}),
                    );
                }
            }
        }

        // Convert back to AppConfig
        let migrated_config: AppConfig = serde_json::from_value(json)?;
        log::info!("Config migration completed successfully");
        Ok(migrated_config)
    }
}
