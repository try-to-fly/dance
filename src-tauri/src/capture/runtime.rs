use crate::analysis::upsert_entry_analysis;
use crate::clipboard::ClipboardMonitor;
use crate::database::Database;
use crate::models::ClipboardEntry;
use crate::retrieval::upsert_entry_search_document;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressionEntry {
    pub content_hash: String,
    pub expires_at_ms: i64,
}

pub struct CaptureRuntime {
    cancel: CancellationToken,
    monitor_task: JoinHandle<()>,
    save_task: JoinHandle<()>,
    last_observed_hash: Arc<Mutex<Option<String>>>,
    suppression_registry: Arc<Mutex<Vec<SuppressionEntry>>>,
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
        let suppression_registry = Arc::new(Mutex::new(Vec::new()));
        let monitor_cancel = cancel.child_token();
        let save_cancel = cancel.child_token();
        let observed_hash_for_monitor = Arc::clone(&last_observed_hash);
        let suppression_registry_for_monitor = Arc::clone(&suppression_registry);
        let mut rx = tx.subscribe();

        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = monitor_cancel.cancelled() => break,
                    _ = interval.tick() => {
                        if let Err(error) = monitor
                            .poll_once(
                                &observed_hash_for_monitor,
                                &suppression_registry_for_monitor,
                            )
                            .await
                        {
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
                            Ok(entry) => match persist_entry(&db, entry).await {
                                Ok(stored_entry) => {
                                    emit_clipboard_update(&app_handle, &stored_entry).await;
                                }
                                Err(error) => {
                                    log::error!("[CaptureRuntime] 保存剪贴板条目失败: {}", error);
                                }
                            },
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
            suppression_registry,
        }
    }

    pub async fn stop(self) {
        self.cancel.cancel();
        let _ = self.monitor_task.await;
        let _ = self.save_task.await;
    }

    pub async fn remember_observed_hash(&self, content_hash: String) {
        remember_observed_hash(&self.last_observed_hash, content_hash).await;
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn observed_hash(&self) -> Option<String> {
        self.last_observed_hash.lock().await.clone()
    }

    pub async fn register_suppression_key(&self, content_hash: String, ttl_ms: i64) {
        self.remember_observed_hash(content_hash.clone()).await;

        let now_ms = now_timestamp_ms();
        let mut registry = self.suppression_registry.lock().await;
        purge_expired_suppression_entries(&mut registry, now_ms);
        registry.retain(|entry| entry.content_hash != content_hash);
        registry.push(SuppressionEntry {
            content_hash,
            expires_at_ms: now_ms + ttl_ms,
        });
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn has_suppression_key(&self, content_hash: &str) -> bool {
        let now_ms = now_timestamp_ms();
        let mut registry = self.suppression_registry.lock().await;
        purge_expired_suppression_entries(&mut registry, now_ms);
        registry
            .iter()
            .any(|entry| entry.content_hash == content_hash)
    }
}

pub async fn remember_observed_hash(
    last_observed_hash: &Arc<Mutex<Option<String>>>,
    content_hash: String,
) {
    let mut last = last_observed_hash.lock().await;
    *last = Some(content_hash);
}

pub async fn consume_suppression_key(
    suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
    content_hash: &str,
) -> bool {
    let now_ms = now_timestamp_ms();
    let mut registry = suppression_registry.lock().await;
    purge_expired_suppression_entries(&mut registry, now_ms);

    if let Some(index) = registry
        .iter()
        .position(|entry| entry.content_hash == content_hash)
    {
        registry.remove(index);
        true
    } else {
        false
    }
}

fn purge_expired_suppression_entries(registry: &mut Vec<SuppressionEntry>, now_ms: i64) {
    registry.retain(|entry| entry.expires_at_ms > now_ms);
}

fn now_timestamp_ms() -> i64 {
    Utc::now().timestamp_millis()
}

async fn persist_entry(db: &Database, entry: ClipboardEntry) -> anyhow::Result<ClipboardEntry> {
    log::debug!(
        "[CaptureRuntime] 收到新条目: {} ({})",
        short_hash(&entry.content_hash),
        entry.content_type
    );

    let snapshot = entry.analysis.clone();
    let mut tx = db.pool().begin().await?;

    sqlx::query(
        r#"
        INSERT INTO clipboard_entries
        (id, content_hash, content_type, content_data, source_app,
         created_at, copy_count, file_path, is_favorite, content_subtype, metadata, app_bundle_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(content_hash) DO UPDATE SET
            content_data = COALESCE(excluded.content_data, clipboard_entries.content_data),
            copy_count = clipboard_entries.copy_count + 1,
            created_at = excluded.created_at,
            source_app = COALESCE(excluded.source_app, clipboard_entries.source_app),
            content_subtype = COALESCE(excluded.content_subtype, clipboard_entries.content_subtype),
            metadata = COALESCE(excluded.metadata, clipboard_entries.metadata),
            app_bundle_id = COALESCE(excluded.app_bundle_id, clipboard_entries.app_bundle_id),
            file_path = COALESCE(excluded.file_path, clipboard_entries.file_path)
        "#,
    )
    .bind(&entry.id)
    .bind(&entry.content_hash)
    .bind(&entry.content_type)
    .bind(&entry.content_data)
    .bind(&entry.source_app)
    .bind(entry.created_at)
    .bind(entry.copy_count)
    .bind(&entry.file_path)
    .bind(entry.is_favorite as i32)
    .bind(&entry.content_subtype)
    .bind(&entry.metadata)
    .bind(&entry.app_bundle_id)
    .execute(&mut *tx)
    .await?;

    let mut stored_entry = sqlx::query_as::<_, ClipboardEntry>(
        "SELECT * FROM clipboard_entries WHERE content_hash = ?",
    )
    .bind(&entry.content_hash)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(snapshot) = snapshot {
        upsert_entry_analysis(
            &mut *tx,
            &stored_entry.id,
            &stored_entry.content_hash,
            &snapshot,
        )
        .await?;
        stored_entry.attach_analysis(snapshot);
    }

    upsert_entry_search_document(&mut tx, &stored_entry).await?;

    tx.commit().await?;

    Ok(stored_entry)
}

async fn emit_clipboard_update(app_handle: &Arc<Mutex<Option<AppHandle>>>, entry: &ClipboardEntry) {
    if let Some(handle) = app_handle.lock().await.as_ref() {
        if let Err(error) = handle.emit("clipboard-update", entry) {
            log::error!("[CaptureRuntime] 发送更新事件失败: {}", error);
        }
    }
}

fn short_hash(content_hash: &str) -> &str {
    let length = content_hash.len().min(8);
    &content_hash[..length]
}

pub fn calculate_content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
