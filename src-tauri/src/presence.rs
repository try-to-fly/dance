use crate::app_paths::AppPaths;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const TAURI_PRESENCE_FILE: &str = "tauri-presence.json";
const DAEMON_STATUS_FILE: &str = "daemon-status.json";
const HEARTBEAT_INTERVAL_MS: u64 = 500;
const PRESENCE_TTL_MS: i64 = 2_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TauriPresence {
    pub owner: String,
    pub pid: u32,
    pub started_at_ms: i64,
    pub heartbeat_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonStatus {
    pub owner: String,
    pub pid: u32,
    pub state: String,
    pub updated_at_ms: i64,
}

#[allow(dead_code)]
pub struct PresenceHeartbeat {
    cancel: CancellationToken,
    task: Option<JoinHandle<()>>,
}

impl PresenceHeartbeat {
    #[allow(dead_code)]
    pub async fn stop(mut self) {
        self.cancel.cancel();
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for PresenceHeartbeat {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

pub fn spawn_tauri_presence_heartbeat(
    paths: Arc<AppPaths>,
    owner: impl Into<String>,
) -> PresenceHeartbeat {
    let owner = owner.into();
    let cancel = CancellationToken::new();
    let task_cancel = cancel.child_token();
    let task = tokio::spawn(async move {
        let started_at_ms = now_ms();
        loop {
            if let Err(error) = write_tauri_presence(
                paths.as_ref(),
                &TauriPresence {
                    owner: owner.clone(),
                    pid: std::process::id(),
                    started_at_ms,
                    heartbeat_at_ms: now_ms(),
                },
            ) {
                log::warn!("[presence] 写入 Tauri 心跳失败: {}", error);
            }

            tokio::select! {
                _ = task_cancel.cancelled() => {
                    let _ = remove_tauri_presence(paths.as_ref());
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(HEARTBEAT_INTERVAL_MS)) => {}
            }
        }
    });

    PresenceHeartbeat {
        cancel,
        task: Some(task),
    }
}

pub fn is_tauri_presence_fresh(paths: &AppPaths) -> bool {
    read_tauri_presence(paths)
        .is_ok_and(|presence| now_ms() - presence.heartbeat_at_ms <= PRESENCE_TTL_MS)
}

pub fn read_daemon_status(paths: &AppPaths) -> Option<DaemonStatus> {
    let path = daemon_status_path(paths);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_daemon_status(paths: &AppPaths, state: &str, owner: &str) -> Result<()> {
    let status = DaemonStatus {
        owner: owner.to_string(),
        pid: std::process::id(),
        state: state.to_string(),
        updated_at_ms: now_ms(),
    };
    write_json(daemon_status_path(paths), &status)
}

fn read_tauri_presence(paths: &AppPaths) -> Result<TauriPresence> {
    let content = fs::read_to_string(tauri_presence_path(paths))?;
    Ok(serde_json::from_str(&content)?)
}

fn write_tauri_presence(paths: &AppPaths, presence: &TauriPresence) -> Result<()> {
    write_json(tauri_presence_path(paths), presence)
}

fn remove_tauri_presence(paths: &AppPaths) -> Result<()> {
    let path = tauri_presence_path(paths);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn write_json<T: Serialize>(path: std::path::PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn tauri_presence_path(paths: &AppPaths) -> std::path::PathBuf {
    paths.runtime_dir().join(TAURI_PRESENCE_FILE)
}

fn daemon_status_path(paths: &AppPaths) -> std::path::PathBuf {
    paths.runtime_dir().join(DAEMON_STATUS_FILE)
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::create_temp_app_roots;

    #[test]
    fn stale_tauri_presence_is_not_fresh() {
        let roots = create_temp_app_roots();
        let paths = AppPaths::from_roots(
            roots.config_root,
            roots.data_root,
            roots.cache_root,
            roots.log_root,
        );
        write_tauri_presence(
            &paths,
            &TauriPresence {
                owner: "com.dance.app".to_string(),
                pid: 1,
                started_at_ms: now_ms() - 5_000,
                heartbeat_at_ms: now_ms() - 5_000,
            },
        )
        .unwrap();

        assert!(!is_tauri_presence_fresh(&paths));
    }

    #[test]
    fn daemon_status_round_trips() {
        let roots = create_temp_app_roots();
        let paths = AppPaths::from_roots(
            roots.config_root,
            roots.data_root,
            roots.cache_root,
            roots.log_root,
        );
        write_daemon_status(&paths, "listening", "com.dance.app").unwrap();

        let status = read_daemon_status(&paths).unwrap();
        assert_eq!(status.state, "listening");
        assert_eq!(status.owner, "com.dance.app");
    }
}
