use anyhow::Result;
use serde_json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

use crate::clipboard::content_detector::ContentDetector;
use crate::clipboard::processor::ContentProcessor;
use crate::config::ConfigManager;
use crate::models::{ClipboardEntry, ContentType};
use crate::utils::app_detector::get_active_app_info;

pub struct ClipboardMonitor {
    last_hash: Arc<Mutex<Option<String>>>,
    tx: broadcast::Sender<ClipboardEntry>,
    processor: Arc<ContentProcessor>,
    config_manager: Arc<Mutex<ConfigManager>>,
}

impl ClipboardMonitor {
    pub fn new(
        tx: broadcast::Sender<ClipboardEntry>,
        processor: Arc<ContentProcessor>,
        config_manager: Arc<Mutex<ConfigManager>>,
    ) -> Result<Self> {
        let last_hash = Arc::new(Mutex::new(None));

        Ok(Self {
            last_hash,
            tx,
            processor,
            config_manager,
        })
    }

    fn get_saved_file_size(file_path: &str) -> Option<u64> {
        // 将相对路径转换为绝对路径
        let absolute_path = if file_path.starts_with("imgs/") {
            let config_dir = dirs::config_dir()?;
            let app_dir = config_dir.join("clipboard-app");
            app_dir.join(file_path)
        } else {
            std::path::PathBuf::from(file_path)
        };

        std::fs::metadata(absolute_path).ok().map(|meta| meta.len())
    }

    fn should_skip_data_url_recording(text: &str, source_bundle_id: Option<&str>) -> bool {
        let is_base64_image_data_url = text.starts_with("data:image/") && text.contains(";base64,");
        if !is_base64_image_data_url {
            return false;
        }

        // 仅在来源是本应用时跳过，避免把用户真实复制的 data URL 全部丢弃。
        matches!(source_bundle_id, Some("com.dance.app"))
    }

    pub async fn start_monitoring(&self) {
        log::info!("[ClipboardMonitor] 启动剪贴板监控");

        let last_hash = Arc::clone(&self.last_hash);
        let tx = self.tx.clone();
        let processor = Arc::clone(&self.processor);
        let config_manager = Arc::clone(&self.config_manager);

        tokio::spawn(async move {
            loop {
                // 获取当前应用信息，用于记录剪贴板内容来源
                let app_info = get_active_app_info();
                if let Some(ref info) = app_info {
                    log::trace!(
                        "[ClipboardMonitor] 当前活跃应用: {} ({})",
                        info.name,
                        info.bundle_id.as_deref().unwrap_or("unknown")
                    );
                } else {
                    log::trace!("[ClipboardMonitor] 无法获取当前活跃应用信息");
                }

                if let Err(e) =
                    Self::check_clipboard(&last_hash, &tx, &processor, &config_manager).await
                {
                    log::error!("剪切板检查错误: {}", e);
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
    }

    async fn check_clipboard(
        last_hash: &Arc<Mutex<Option<String>>>,
        tx: &broadcast::Sender<ClipboardEntry>,
        processor: &Arc<ContentProcessor>,
        config_manager: &Arc<Mutex<ConfigManager>>,
    ) -> Result<()> {
        // 获取当前活跃应用信息
        let app_info = get_active_app_info();

        // 检查文本内容 - 使用独立的剪切板实例，避免长时间锁定
        let text_result = tokio::task::spawn_blocking(|| match arboard::Clipboard::new() {
            Ok(mut temp_clipboard) => temp_clipboard.get_text(),
            Err(e) => Err(e),
        })
        .await
        .unwrap_or(Err(arboard::Error::ClipboardNotSupported));

        if let Ok(text) = text_result {
            // 先trim处理文本
            let trimmed_text = text.trim();
            if !trimmed_text.is_empty() {
                log::debug!(
                    "[ClipboardMonitor] 检测到文本内容 ({}字符)",
                    trimmed_text.len()
                );

                if Self::should_skip_data_url_recording(
                    trimmed_text,
                    app_info.as_ref().and_then(|info| info.bundle_id.as_deref()),
                ) {
                    log::debug!("[ClipboardMonitor] 跳过本应用产生的base64图片URL，避免循环记录");
                    return Ok(());
                }

                let hash = Self::calculate_hash(trimmed_text.as_bytes());
                log::debug!("[ClipboardMonitor] 计算内容Hash: {}", &hash[..8]);

                let should_send = {
                    let mut last = last_hash.lock().await;
                    if last.as_ref() != Some(&hash) {
                        *last = Some(hash.clone());
                        log::debug!("[ClipboardMonitor] 新内容Hash，准备处理");
                        true
                    } else {
                        log::debug!("[ClipboardMonitor] 重复内容Hash，跳过处理");
                        false
                    }
                };

                if should_send {
                    // 检查是否是被排除的应用
                    if let Some(ref app_info) = app_info {
                        if let Some(bundle_id) = &app_info.bundle_id {
                            let config_guard = config_manager.lock().await;
                            if config_guard.is_app_excluded(bundle_id) {
                                log::debug!(
                                    "[ClipboardMonitor] 应用 {} 在排除列表中，跳过",
                                    app_info.name
                                );
                                return Ok(());
                            }

                            // 检查文本大小限制
                            if !config_guard.is_text_size_valid(trimmed_text) {
                                log::warn!(
                                    "[ClipboardMonitor] 文本大小超限 ({}字符)，跳过",
                                    trimmed_text.len()
                                );
                                return Ok(());
                            }
                        }
                    }

                    // 检测内容子类型
                    let (subtype, metadata) = ContentDetector::detect(trimmed_text);
                    log::debug!("[ClipboardMonitor] 内容检测结果: {:?}", subtype);

                    // 将metadata转换为JSON字符串
                    let metadata_json = metadata.and_then(|m| serde_json::to_string(&m).ok());

                    let mut entry = ClipboardEntry::new(
                        ContentType::Text,
                        Some(trimmed_text.to_string()),
                        hash,
                        app_info.as_ref().map(|info| info.name.clone()),
                        None,
                    );

                    // 设置子类型、元数据和bundle ID
                    // 使用serde_json::to_value获取正确的snake_case字符串
                    let subtype_str = serde_json::to_value(&subtype)
                        .ok()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "plain_text".to_string());
                    entry.content_subtype = Some(subtype_str);
                    entry.metadata = metadata_json;
                    entry.app_bundle_id = app_info.as_ref().and_then(|info| info.bundle_id.clone());

                    log::info!(
                        "[ClipboardMonitor] 发现新文本内容: {} | 来源: {} | 类型: {:?}",
                        if trimmed_text.chars().count() > 50 {
                            format!("{}...", trimmed_text.chars().take(50).collect::<String>())
                        } else {
                            trimmed_text.to_string()
                        },
                        app_info
                            .as_ref()
                            .map(|info| info.name.as_str())
                            .unwrap_or("未知应用"),
                        subtype
                    );

                    let _ = tx.send(entry);
                    return Ok(());
                }
            }
        }

        // 检查图片内容 - 使用独立的剪切板实例
        let image_result = tokio::task::spawn_blocking(|| match arboard::Clipboard::new() {
            Ok(mut temp_clipboard) => temp_clipboard.get_image(),
            Err(e) => Err(e),
        })
        .await
        .unwrap_or(Err(arboard::Error::ClipboardNotSupported));

        if let Ok(image_data) = image_result {
            // arboard 返回的图片数据包含宽高信息
            let width = image_data.width;
            let height = image_data.height;
            let bytes = image_data.bytes.as_ref();

            log::debug!(
                "[ClipboardMonitor] 检测到图片内容: {}x{} ({}字节)",
                width,
                height,
                bytes.len()
            );

            let hash = Self::calculate_hash(bytes);
            log::debug!("[ClipboardMonitor] 计算图片Hash: {}", &hash[..8]);

            let should_send = {
                let mut last = last_hash.lock().await;
                if last.as_ref() != Some(&hash) {
                    *last = Some(hash.clone());
                    log::debug!("[ClipboardMonitor] 新图片Hash，准备处理");
                    true
                } else {
                    log::debug!("[ClipboardMonitor] 重复图片Hash，跳过处理");
                    false
                }
            };

            if should_send {
                // 检查是否是被排除的应用
                if let Some(ref app_info) = app_info {
                    if let Some(bundle_id) = &app_info.bundle_id {
                        let config_guard = config_manager.lock().await;
                        if config_guard.is_app_excluded(bundle_id) {
                            log::debug!(
                                "[ClipboardMonitor] 图片来源应用 {} 在排除列表中，跳过",
                                app_info.name
                            );
                            return Ok(());
                        }
                    }
                }

                // 使用宽高信息处理图片
                match processor
                    .process_image_with_dimensions(bytes, width as u32, height as u32)
                    .await
                {
                    Ok(image_info) => {
                        log::info!(
                            "[ClipboardMonitor] 图片处理成功: {}x{} -> {} ({}字节) | 来源: {}",
                            image_info.width,
                            image_info.height,
                            image_info.file_path,
                            image_info.actual_size,
                            app_info
                                .as_ref()
                                .map(|info| info.name.as_str())
                                .unwrap_or("未知应用")
                        );

                        // 创建图片元数据，使用实际压缩后的文件大小
                        let image_metadata = serde_json::json!({
                            "image_metadata": {
                                "width": image_info.width,
                                "height": image_info.height,
                                "file_size": image_info.actual_size,
                                "format": "png"
                            }
                        });

                        let mut entry = ClipboardEntry::new(
                            ContentType::Image,
                            Some(image_info.file_path.clone()),
                            hash,
                            app_info.as_ref().map(|info| info.name.clone()),
                            Some(image_info.file_path),
                        );
                        entry.app_bundle_id =
                            app_info.as_ref().and_then(|info| info.bundle_id.clone());
                        entry.metadata = Some(image_metadata.to_string());

                        let _ = tx.send(entry);
                    }
                    Err(e) => {
                        log::warn!(
                            "[ClipboardMonitor] 指定尺寸图片处理失败，降级到自动检测: {}",
                            e
                        );
                        // 降级到自动检测
                        match processor.process_image(bytes).await {
                            Ok(file_path) => {
                                // 获取实际保存的文件大小
                                let actual_size = Self::get_saved_file_size(&file_path)
                                    .unwrap_or(bytes.len() as u64);

                                log::info!("[ClipboardMonitor] 图片降级处理成功: {}x{} -> {} ({}字节) | 来源: {}", 
                                    width, height,
                                    file_path,
                                    actual_size,
                                    app_info.as_ref().map(|info| info.name.as_str()).unwrap_or("未知应用")
                                );

                                // 创建图片元数据（使用压缩后的文件大小）
                                let image_metadata = serde_json::json!({
                                    "image_metadata": {
                                        "width": width as u32,
                                        "height": height as u32,
                                        "file_size": actual_size,
                                        "format": "png"
                                    }
                                });

                                let mut entry = ClipboardEntry::new(
                                    ContentType::Image,
                                    Some(file_path.clone()),
                                    hash,
                                    app_info.as_ref().map(|info| info.name.clone()),
                                    Some(file_path),
                                );
                                entry.app_bundle_id =
                                    app_info.as_ref().and_then(|info| info.bundle_id.clone());
                                entry.metadata = Some(image_metadata.to_string());

                                let _ = tx.send(entry);
                            }
                            Err(fallback_error) => {
                                log::error!(
                                    "[ClipboardMonitor] 图片处理完全失败: 原始错误={}, 降级错误={}",
                                    e,
                                    fallback_error
                                );
                            }
                        }
                    }
                }
                return Ok(());
            }
        }

        Ok(())
    }

    fn calculate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::ClipboardMonitor;

    #[test]
    fn test_should_skip_data_url_recording_for_self_app_only() {
        let value = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA";
        assert!(ClipboardMonitor::should_skip_data_url_recording(
            value,
            Some("com.dance.app")
        ));
        assert!(!ClipboardMonitor::should_skip_data_url_recording(
            value,
            Some("com.other.app")
        ));
        assert!(!ClipboardMonitor::should_skip_data_url_recording(
            value, None
        ));
    }

    #[test]
    fn test_should_not_skip_non_data_url() {
        assert!(!ClipboardMonitor::should_skip_data_url_recording(
            "https://example.com/a.png",
            Some("com.dance.app")
        ));
    }
}
