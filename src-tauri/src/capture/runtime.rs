use crate::clipboard::ClipboardMonitor;
use crate::database::Database;
use crate::models::ClipboardEntry;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

pub struct CaptureRuntime {
    cancel: CancellationToken,
    monitor_task: JoinHandle<()>,
    save_task: JoinHandle<()>,
    #[allow(dead_code)]
    last_observed_hash: Arc<Mutex<Option<String>>>,
}

impl CaptureRuntime {
    pub fn spawn(
        monitor: ClipboardMonitor,
        tx: broadcast::Sender<ClipboardEntry>,
        db: Arc<Database>,
        app_handle: Arc<Mutex<Option<AppHandle>>>,
    ) -> Self {
        let cancel = CancellationToken::new();
        let last_observed_hash = Arc::new(Mutex::new(None));
        let monitor_cancel = cancel.child_token();
        let save_cancel = cancel.child_token();
        let observed_hash_for_monitor = Arc::clone(&last_observed_hash);
        let mut rx = tx.subscribe();

        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = monitor_cancel.cancelled() => break,
                    _ = interval.tick() => {
                        if let Err(error) = monitor.poll_once(&observed_hash_for_monitor).await {
                            log::error!("[CaptureRuntime] 剪贴板轮询失败: {}", error);
                        }
                    }
                }
            }
        });

        let save_task = tokio::spawn(async move {
            log::info!("[CaptureRuntime] 启动受控数据库保存任务");

            loop {
                tokio::select! {
                    _ = save_cancel.cancelled() => break,
                    recv_result = rx.recv() => {
                        match recv_result {
                            Ok(entry) => {
                                let updated_entry =
                                    persist_entry(&db, entry).await.unwrap_or_else(|error| {
                                        log::error!("[CaptureRuntime] 保存剪贴板条目失败: {}", error);
                                        None
                                    });

                                if let Some(entry) = updated_entry {
                                    emit_clipboard_update(&app_handle, &entry).await;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                log::warn!(
                                    "[CaptureRuntime] 数据保存任务丢失了 {} 条广播消息",
                                    skipped
                                );
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        });

        Self {
            cancel,
            monitor_task,
            save_task,
            last_observed_hash,
        }
    }

    pub async fn stop(self) {
        self.cancel.cancel();
        let _ = self.monitor_task.await;
        let _ = self.save_task.await;
    }

    #[allow(dead_code)]
    pub async fn remember_observed_hash(&self, content_hash: String) {
        let mut last = self.last_observed_hash.lock().await;
        *last = Some(content_hash);
    }
}

async fn persist_entry(
    db: &Database,
    entry: ClipboardEntry,
) -> anyhow::Result<Option<ClipboardEntry>> {
    log::debug!(
        "[CaptureRuntime] 收到新条目: {} ({})",
        &entry.content_hash[..8],
        entry.content_type
    );

    let existing =
        sqlx::query("SELECT id, copy_count FROM clipboard_entries WHERE content_hash = ?")
            .bind(&entry.content_hash)
            .fetch_optional(db.pool())
            .await?;

    let mut updated_entry = entry.clone();

    match existing {
        Some(row) => {
            let id: String = row.get("id");
            let count: i32 = row.get("copy_count");
            let new_count = count + 1;

            sqlx::query("UPDATE clipboard_entries SET copy_count = ?, created_at = ? WHERE id = ?")
                .bind(new_count)
                .bind(entry.created_at)
                .bind(&id)
                .execute(db.pool())
                .await?;

            updated_entry.id = id;
            updated_entry.copy_count = new_count;
        }
        None => {
            updated_entry.copy_count = 1;

            sqlx::query(
                r#"
                INSERT INTO clipboard_entries
                (id, content_hash, content_type, content_data, source_app,
                 created_at, copy_count, file_path, is_favorite, content_subtype, metadata, app_bundle_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&entry.id)
            .bind(&entry.content_hash)
            .bind(&entry.content_type)
            .bind(&entry.content_data)
            .bind(&entry.source_app)
            .bind(entry.created_at)
            .bind(1)
            .bind(&entry.file_path)
            .bind(entry.is_favorite as i32)
            .bind(&entry.content_subtype)
            .bind(&entry.metadata)
            .bind(&entry.app_bundle_id)
            .execute(db.pool())
            .await?;
        }
    }

    Ok(Some(updated_entry))
}

async fn emit_clipboard_update(app_handle: &Arc<Mutex<Option<AppHandle>>>, entry: &ClipboardEntry) {
    if let Some(handle) = app_handle.lock().await.as_ref() {
        if let Err(error) = handle.emit("clipboard-update", entry) {
            log::error!("[CaptureRuntime] 发送更新事件失败: {}", error);
        }
    }
}

pub fn calculate_content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
