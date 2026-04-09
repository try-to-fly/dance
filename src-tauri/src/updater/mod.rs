use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    pub available: bool,
}

pub struct UpdateManager;

impl UpdateManager {
    /// Check if we should check for updates (once per day)
    pub fn should_check_for_updates(last_check: Option<&str>) -> bool {
        if let Some(last_check_str) = last_check {
            if let Ok(last_check_time) = DateTime::parse_from_rfc3339(last_check_str) {
                let now = Utc::now();
                let duration = now.signed_duration_since(last_check_time);
                // Check if more than 24 hours have passed
                return duration.num_hours() >= 24;
            }
        }
        // If no last check or parsing failed, we should check
        true
    }

    /// Get current timestamp in ISO 8601 format
    pub fn get_current_timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    /// Check for updates
    pub async fn check_for_updates(app: &AppHandle) -> Result<Option<UpdateInfo>> {
        log::info!("[UpdateManager] Starting update check...");
        log::info!(
            "[UpdateManager] Current app version: {}",
            app.package_info().version
        );

        let updater = app.updater_builder().build()?;
        log::debug!("[UpdateManager] Updater built successfully");

        match updater.check().await {
            Ok(Some(update)) => {
                log::info!("[UpdateManager] Update available: {}", update.version);
                log::info!(
                    "[UpdateManager] Update notes: {}",
                    update.body.as_ref().unwrap_or(&"No notes".to_string())
                );
                log::info!("[UpdateManager] Update date: {:?}", update.date);
                let info = UpdateInfo {
                    version: update.version.clone(),
                    notes: update.body.clone(),
                    pub_date: update.date.map(|d| d.format(&Rfc3339).unwrap_or_default()),
                    available: true,
                };
                Ok(Some(info))
            }
            Ok(None) => {
                log::info!("[UpdateManager] No updates available - current version is up to date");
                log::debug!("[UpdateManager] This could mean:");
                log::debug!("  - Remote version is same or older than current version");
                log::debug!("  - No release manifest found at the endpoint");
                log::debug!(
                    "  - Current version {} is already the latest",
                    app.package_info().version
                );
                Ok(None)
            }
            Err(e) => {
                log::error!("[UpdateManager] Failed to check for updates: {}", e);
                log::error!("[UpdateManager] Error details: {:?}", e);
                log::error!("[UpdateManager] This could be due to:");
                log::error!("  - Network connection issues");
                log::error!("  - Invalid or unreachable update endpoints");
                log::error!("  - Malformed update manifest");
                log::error!("  - Authentication/permission issues");
                // Propagate the error to frontend for better error handling
                Err(e.into())
            }
        }
    }

    /// Download and install update
    pub async fn download_and_install(app: &AppHandle) -> Result<()> {
        log::info!("[UpdateManager] Starting update download and install flow");

        let updater = app.updater_builder().build()?;

        if let Some(update) = updater.check().await? {
            log::info!(
                "[UpdateManager] Downloading update {} for installation",
                update.version
            );

            // Emit progress events to frontend
            let app_handle = app.clone();
            let mut downloaded_bytes = 0usize;

            update
                .download_and_install(
                    move |chunk_length, content_length| {
                        downloaded_bytes = downloaded_bytes.saturating_add(chunk_length);

                        let progress = if let Some(total) = content_length {
                            ((downloaded_bytes as f64 / total as f64) * 100.0)
                                .round()
                                .clamp(0.0, 100.0) as u32
                        } else {
                            0
                        };

                        let _ = app_handle.emit("update-download-progress", progress);
                    },
                    || {
                        log::info!("[UpdateManager] Update package downloaded, applying update");
                    },
                )
                .await?;

            log::info!("[UpdateManager] Update installed successfully, restarting app");
            app.restart();
        } else {
            log::info!("[UpdateManager] Install requested but no update is currently available");
        }

        Ok(())
    }

    /// Manually trigger update check
    #[allow(dead_code)]
    pub async fn manual_check_and_update(app: &AppHandle) -> Result<UpdateInfo> {
        if let Some(info) = Self::check_for_updates(app).await? {
            Ok(info)
        } else {
            Ok(UpdateInfo {
                version: String::new(),
                notes: None,
                pub_date: None,
                available: false,
            })
        }
    }
}
