use anyhow::Result;
use serde_json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

use crate::capture::macos_markers::read_pasteboard_markers;
use crate::capture::{
    calculate_content_hash, consume_suppression_key, decide_capture, remember_observed_hash,
    CaptureDisposition, PasteboardMarkers, SuppressionEntry,
};
use crate::clipboard::content_detector::{ContentDetector, ContentMetadata, ContentSubType};
use crate::clipboard::processor::ContentProcessor;
use crate::config::ConfigManager;
use crate::models::{ClipboardEntry, ContentType};
use crate::utils::app_detector::{get_active_app_info, AppInfo};

pub struct ClipboardMonitor {
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
        Ok(Self {
            tx,
            processor,
            config_manager,
        })
    }

    fn get_saved_file_size(&self, file_path: &str) -> Option<u64> {
        let absolute_path = if file_path.starts_with("imgs/") {
            self.processor.resolve_relative_asset_path(file_path).ok()?
        } else {
            std::path::PathBuf::from(file_path)
        };

        std::fs::metadata(absolute_path).ok().map(|meta| meta.len())
    }

    fn resolve_source_bundle_id<'a>(
        markers: &'a PasteboardMarkers,
        app_info: Option<&'a AppInfo>,
    ) -> Option<&'a str> {
        markers
            .source_bundle_id
            .as_deref()
            .or_else(|| app_info.and_then(|info| info.bundle_id.as_deref()))
    }

    fn is_self_generated(source_bundle_id: Option<&str>) -> bool {
        matches!(source_bundle_id, Some("com.dance.app"))
    }

    async fn should_emit_hash(
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        content_hash: &str,
    ) -> bool {
        let is_duplicate = {
            let last = last_observed_hash.lock().await;
            last.as_deref() == Some(content_hash)
        };

        if is_duplicate {
            false
        } else {
            remember_observed_hash(last_observed_hash, content_hash.to_string()).await;
            true
        }
    }

    async fn text_capture_disposition(
        &self,
        markers: &PasteboardMarkers,
        app_info: Option<&AppInfo>,
        trimmed_text: &str,
    ) -> CaptureDisposition {
        let source_bundle_id = Self::resolve_source_bundle_id(markers, app_info);
        let config_guard = self.config_manager.lock().await;
        let excluded_app = source_bundle_id
            .map(|bundle_id| config_guard.is_app_excluded(bundle_id))
            .unwrap_or(false);
        let text_size_valid = config_guard.is_text_size_valid(trimmed_text);

        decide_capture(
            markers,
            Self::is_self_generated(source_bundle_id),
            excluded_app,
            text_size_valid,
        )
    }

    async fn image_capture_disposition(
        &self,
        markers: &PasteboardMarkers,
        app_info: Option<&AppInfo>,
    ) -> CaptureDisposition {
        let source_bundle_id = Self::resolve_source_bundle_id(markers, app_info);
        let config_guard = self.config_manager.lock().await;
        let excluded_app = source_bundle_id
            .map(|bundle_id| config_guard.is_app_excluded(bundle_id))
            .unwrap_or(false);

        decide_capture(
            markers,
            Self::is_self_generated(source_bundle_id),
            excluded_app,
            true,
        )
    }

    fn capture_disposition_label(disposition: CaptureDisposition) -> &'static str {
        match disposition {
            CaptureDisposition::Persist => "persist",
            CaptureDisposition::CurrentOnly => "current_only",
            CaptureDisposition::Skip => "skip",
        }
    }

    async fn process_text_capture_with_detector<F>(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
        app_info: Option<&AppInfo>,
        markers: &PasteboardMarkers,
        trimmed_text: &str,
        detector: F,
    ) -> Result<Option<CaptureDisposition>>
    where
        F: FnOnce(&str) -> (ContentSubType, Option<ContentMetadata>),
    {
        let hash = calculate_content_hash(trimmed_text.as_bytes());
        log::debug!("[ClipboardMonitor] 计算内容Hash: {}", &hash[..8]);

        if consume_suppression_key(suppression_registry, &hash).await {
            remember_observed_hash(last_observed_hash, hash).await;
            log::debug!("[ClipboardMonitor] 命中 suppression key，跳过本次文本持久化");
            return Ok(Some(CaptureDisposition::Skip));
        }

        let disposition = self
            .text_capture_disposition(markers, app_info, trimmed_text)
            .await;
        if disposition != CaptureDisposition::Persist {
            remember_observed_hash(last_observed_hash, hash).await;
            log::debug!(
                "[ClipboardMonitor] 文本命中 capture policy: {}",
                Self::capture_disposition_label(disposition)
            );
            return Ok(Some(disposition));
        }

        if !Self::should_emit_hash(last_observed_hash, &hash).await {
            log::debug!("[ClipboardMonitor] 重复内容Hash，跳过处理");
            return Ok(None);
        }

        let (subtype, metadata) = detector(trimmed_text);
        log::debug!("[ClipboardMonitor] 内容检测结果: {:?}", subtype);

        let metadata_json = metadata.and_then(|value| serde_json::to_string(&value).ok());
        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some(trimmed_text.to_string()),
            hash,
            app_info.map(|info| info.name.clone()),
            None,
        );

        let subtype_str = serde_json::to_value(&subtype)
            .ok()
            .and_then(|value| value.as_str().map(|value| value.to_string()))
            .unwrap_or_else(|| "plain_text".to_string());
        entry.content_subtype = Some(subtype_str);
        entry.metadata = metadata_json;
        entry.app_bundle_id = Self::resolve_source_bundle_id(markers, app_info)
            .map(|bundle_id| bundle_id.to_string());

        log::info!(
            "[ClipboardMonitor] 发现新文本内容: {} | 来源: {} | 类型: {:?}",
            if trimmed_text.chars().count() > 50 {
                format!("{}...", trimmed_text.chars().take(50).collect::<String>())
            } else {
                trimmed_text.to_string()
            },
            app_info
                .map(|info| info.name.as_str())
                .unwrap_or("未知应用"),
            subtype
        );

        let _ = self.tx.send(entry);
        Ok(Some(CaptureDisposition::Persist))
    }

    pub async fn poll_once(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
    ) -> Result<()> {
        self.check_clipboard(
            last_observed_hash,
            suppression_registry,
            &self.tx,
            &self.processor,
        )
        .await
    }

    async fn check_clipboard(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
        tx: &broadcast::Sender<ClipboardEntry>,
        processor: &Arc<ContentProcessor>,
    ) -> Result<()> {
        // 获取当前活跃应用信息
        let app_info = get_active_app_info();
        let markers = read_pasteboard_markers();

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

                if self
                    .process_text_capture_with_detector(
                        last_observed_hash,
                        suppression_registry,
                        app_info.as_ref(),
                        &markers,
                        trimmed_text,
                        ContentDetector::detect,
                    )
                    .await?
                    .is_some()
                {
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

            let hash = calculate_content_hash(bytes);
            log::debug!("[ClipboardMonitor] 计算图片Hash: {}", &hash[..8]);

            if consume_suppression_key(suppression_registry, &hash).await {
                remember_observed_hash(last_observed_hash, hash).await;
                log::debug!("[ClipboardMonitor] 命中 suppression key，跳过本次图片持久化");
                return Ok(());
            }

            let disposition = self
                .image_capture_disposition(&markers, app_info.as_ref())
                .await;
            if disposition != CaptureDisposition::Persist {
                remember_observed_hash(last_observed_hash, hash).await;
                log::debug!(
                    "[ClipboardMonitor] 图片命中 capture policy: {}",
                    Self::capture_disposition_label(disposition)
                );
                return Ok(());
            }

            let should_send = Self::should_emit_hash(last_observed_hash, &hash).await;

            if should_send {
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
                            Self::resolve_source_bundle_id(&markers, app_info.as_ref())
                                .map(|bundle_id| bundle_id.to_string());
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
                                let actual_size = self
                                    .get_saved_file_size(&file_path)
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
                                    Self::resolve_source_bundle_id(&markers, app_info.as_ref())
                                        .map(|bundle_id| bundle_id.to_string());
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

            log::debug!("[ClipboardMonitor] 重复图片Hash，跳过处理");
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) async fn process_text_capture_for_test<F>(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
        app_info: Option<&crate::utils::app_detector::AppInfo>,
        markers: &crate::capture::PasteboardMarkers,
        trimmed_text: &str,
        detector: F,
    ) -> Result<crate::capture::CaptureDisposition>
    where
        F: FnOnce(
            &str,
        ) -> (
            crate::clipboard::content_detector::ContentSubType,
            Option<crate::clipboard::content_detector::ContentMetadata>,
        ),
    {
        Ok(self
            .process_text_capture_with_detector(
                last_observed_hash,
                suppression_registry,
                app_info,
                markers,
                trimmed_text,
                detector,
            )
            .await?
            .unwrap_or(CaptureDisposition::Persist))
    }
}

#[cfg(test)]
mod tests {
    use super::ClipboardMonitor;
    use crate::capture::PasteboardMarkers;
    use crate::utils::app_detector::AppInfo;

    #[test]
    fn test_resolve_source_bundle_id_prefers_markers() {
        let markers = PasteboardMarkers {
            source_bundle_id: Some("com.marker.source".to_string()),
            ..PasteboardMarkers::default()
        };
        let app_info = AppInfo {
            name: "Test App".to_string(),
            bundle_id: Some("com.active.app".to_string()),
        };

        assert_eq!(
            ClipboardMonitor::resolve_source_bundle_id(&markers, Some(&app_info)),
            Some("com.marker.source")
        );
    }

    #[test]
    fn test_is_self_generated_uses_bundle_id() {
        assert!(ClipboardMonitor::is_self_generated(Some("com.dance.app")));
        assert!(!ClipboardMonitor::is_self_generated(Some("com.other.app")));
        assert!(!ClipboardMonitor::is_self_generated(None));
    }
}
