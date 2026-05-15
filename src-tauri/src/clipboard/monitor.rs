use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

use crate::analysis::{AnalysisSnapshot, TextAnalysisService};
use crate::capture::macos_files::read_pasteboard_file_paths;
use crate::capture::macos_markers::read_pasteboard_markers;
use crate::capture::{
    calculate_content_hash, consume_suppression_key, decide_capture, remember_observed_hash,
    CaptureDisposition, PasteboardMarkers, SuppressionEntry,
};
use crate::clipboard::content_detector::ContentDetector;
use crate::clipboard::processor::ContentProcessor;
use crate::config::ConfigManager;
use crate::models::{ClipboardEntry, ContentType};
use crate::utils::app_detector::{get_active_app_info, AppInfo};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub struct ClipboardMonitor {
    tx: broadcast::Sender<ClipboardEntry>,
    processor: Arc<ContentProcessor>,
    config_manager: Arc<Mutex<ConfigManager>>,
    analysis_service: TextAnalysisService,
    self_bundle_identifier: String,
}

impl ClipboardMonitor {
    pub fn new(
        tx: broadcast::Sender<ClipboardEntry>,
        processor: Arc<ContentProcessor>,
        config_manager: Arc<Mutex<ConfigManager>>,
        self_bundle_identifier: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            tx,
            processor,
            config_manager,
            analysis_service: TextAnalysisService::new(),
            self_bundle_identifier: self_bundle_identifier.into(),
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

    fn matches_self_bundle_identifier(
        self_bundle_identifier: &str,
        source_bundle_id: Option<&str>,
    ) -> bool {
        source_bundle_id == Some(self_bundle_identifier)
    }

    fn is_self_generated(&self, source_bundle_id: Option<&str>) -> bool {
        Self::matches_self_bundle_identifier(&self.self_bundle_identifier, source_bundle_id)
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
            self.is_self_generated(source_bundle_id),
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
            self.is_self_generated(source_bundle_id),
            excluded_app,
            true,
        )
    }

    async fn process_file_capture(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        app_info: Option<&AppInfo>,
        markers: &PasteboardMarkers,
        file_paths: Vec<PathBuf>,
    ) -> Result<bool> {
        if file_paths.is_empty() {
            return Ok(false);
        }

        let joined_paths = file_paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        let hash = calculate_content_hash(joined_paths.as_bytes());

        let disposition = self.image_capture_disposition(markers, app_info).await;
        if disposition != CaptureDisposition::Persist {
            remember_observed_hash(last_observed_hash, hash).await;
            log::debug!(
                "[ClipboardMonitor] 文件命中 capture policy: {}",
                Self::capture_disposition_label(disposition)
            );
            return Ok(true);
        }

        if !Self::should_emit_hash(last_observed_hash, &hash).await {
            log::debug!("[ClipboardMonitor] 重复文件Hash，跳过处理");
            return Ok(true);
        }

        let file_path = if file_paths.len() == 1 {
            Some(file_paths[0].to_string_lossy().to_string())
        } else {
            None
        };
        let metadata = if file_paths.len() == 1 {
            build_file_metadata_json(&file_paths[0]).map(|value| value.to_string())
        } else {
            None
        };

        let mut entry = ClipboardEntry::new(
            ContentType::File,
            Some(joined_paths),
            hash,
            app_info.map(|info| info.name.clone()),
            file_path,
        );
        entry.app_bundle_id = Self::resolve_source_bundle_id(markers, app_info)
            .map(|bundle_id| bundle_id.to_string());
        entry.metadata = metadata;

        log::info!(
            "[ClipboardMonitor] 发现文件剪贴板内容: {} 个文件 | 来源: {}",
            file_paths.len(),
            app_info
                .map(|info| info.name.as_str())
                .unwrap_or("未知应用")
        );

        let _ = self.tx.send(entry);
        Ok(true)
    }

    fn capture_disposition_label(disposition: CaptureDisposition) -> &'static str {
        match disposition {
            CaptureDisposition::Persist => "persist",
            CaptureDisposition::CurrentOnly => "current_only",
            CaptureDisposition::Skip => "skip",
        }
    }

    async fn process_text_capture_with_analysis<F>(
        &self,
        last_observed_hash: &Arc<Mutex<Option<String>>>,
        suppression_registry: &Arc<Mutex<Vec<SuppressionEntry>>>,
        app_info: Option<&AppInfo>,
        markers: &PasteboardMarkers,
        trimmed_text: &str,
        analyzer: F,
    ) -> Result<Option<CaptureDisposition>>
    where
        F: FnOnce(&str) -> AnalysisSnapshot,
    {
        let normalized_text = ContentDetector::normalize_clipboard_text(trimmed_text);
        let captured_text = normalized_text.as_ref();
        if captured_text != trimmed_text {
            log::debug!(
                "[ClipboardMonitor] 归一化高占比URL文本: '{}' -> '{}'",
                trimmed_text,
                captured_text
            );
        }

        let hash = calculate_content_hash(captured_text.as_bytes());
        log::debug!("[ClipboardMonitor] 计算内容Hash: {}", &hash[..8]);

        if consume_suppression_key(suppression_registry, &hash).await {
            remember_observed_hash(last_observed_hash, hash).await;
            log::debug!("[ClipboardMonitor] 命中 suppression key，跳过本次文本持久化");
            return Ok(Some(CaptureDisposition::Skip));
        }

        let disposition = self
            .text_capture_disposition(markers, app_info, captured_text)
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

        let snapshot = analyzer(captured_text);
        log::debug!(
            "[ClipboardMonitor] authoritative analysis 结果: {:?} ({:?})",
            snapshot.subtype,
            snapshot.status
        );

        let mut entry = ClipboardEntry::new(
            ContentType::Text,
            Some(captured_text.to_string()),
            hash,
            app_info.map(|info| info.name.clone()),
            None,
        );
        entry.app_bundle_id = Self::resolve_source_bundle_id(markers, app_info)
            .map(|bundle_id| bundle_id.to_string());
        entry.attach_analysis(snapshot.clone());

        log::info!(
            "[ClipboardMonitor] 发现新文本内容: {} | 来源: {} | 类型: {:?} | 状态: {:?}",
            if captured_text.chars().count() > 50 {
                format!("{}...", captured_text.chars().take(50).collect::<String>())
            } else {
                captured_text.to_string()
            },
            app_info
                .map(|info| info.name.as_str())
                .unwrap_or("未知应用"),
            snapshot.subtype,
            snapshot.status
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

        let file_paths = read_pasteboard_file_paths();
        if self
            .process_file_capture(last_observed_hash, app_info.as_ref(), &markers, file_paths)
            .await?
        {
            return Ok(());
        }

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
                    .process_text_capture_with_analysis(
                        last_observed_hash,
                        suppression_registry,
                        app_info.as_ref(),
                        &markers,
                        trimmed_text,
                        |text| self.analysis_service.analyze(text),
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
        F: FnOnce(&str) -> AnalysisSnapshot,
    {
        Ok(self
            .process_text_capture_with_analysis(
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

fn build_file_metadata_json(path: &Path) -> Option<Value> {
    let metadata = std::fs::metadata(path).ok();
    let mut file_metadata = Map::new();

    if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
        file_metadata.insert("name".to_string(), Value::String(name.to_string()));
    }
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        file_metadata.insert(
            "extension".to_string(),
            Value::String(extension.to_lowercase()),
        );
    }
    if let Some(mime) = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|kind| kind.mime_type().to_string())
    {
        file_metadata.insert("mime".to_string(), Value::String(mime));
    }
    if let Some(metadata) = metadata.as_ref() {
        file_metadata.insert("is_directory".to_string(), Value::Bool(metadata.is_dir()));
        if metadata.is_file() {
            file_metadata.insert(
                "size_bytes".to_string(),
                Value::Number(serde_json::Number::from(metadata.len())),
            );
        }
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                file_metadata.insert(
                    "modified_at".to_string(),
                    Value::Number(serde_json::Number::from(duration.as_millis() as u64)),
                );
            }
        }
    }

    if file_metadata.is_empty() {
        return None;
    }

    let mut root = Map::new();
    root.insert("file_metadata".to_string(), Value::Object(file_metadata));
    Some(Value::Object(root))
}

#[cfg(test)]
mod tests {
    use super::{build_file_metadata_json, ClipboardMonitor};
    use crate::app_paths::AppPaths;
    use crate::capture::{
        calculate_content_hash, CaptureDisposition, PasteboardMarkers, SuppressionEntry,
    };
    use crate::clipboard::ContentProcessor;
    use crate::config::ConfigManager;
    use crate::test_support::create_temp_app_roots;
    use crate::utils::app_detector::AppInfo;
    use std::fs;
    use std::sync::Arc;
    use tokio::sync::{broadcast, Mutex};

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
        assert!(ClipboardMonitor::matches_self_bundle_identifier(
            "com.dance.app",
            Some("com.dance.app")
        ));
        assert!(!ClipboardMonitor::matches_self_bundle_identifier(
            "com.dance.app",
            Some("com.other.app")
        ));
        assert!(!ClipboardMonitor::matches_self_bundle_identifier(
            "com.dance.app",
            None
        ));
    }

    #[tokio::test]
    async fn test_process_text_capture_with_analysis_normalizes_dominant_url_before_emit() {
        let roots = create_temp_app_roots();
        let paths = Arc::new(AppPaths::from_roots(
            roots.config_root.clone(),
            roots.data_root.clone(),
            roots.cache_root.clone(),
            roots.log_root.clone(),
        ));
        let processor =
            Arc::new(ContentProcessor::new_in(paths.clone()).expect("create processor"));
        let config_manager = Arc::new(Mutex::new(
            ConfigManager::new_in(paths)
                .await
                .expect("create config manager"),
        ));
        let (tx, mut rx) = broadcast::channel(1);
        let monitor = ClipboardMonitor::new(tx, processor, config_manager, "com.dance.app.dev")
            .expect("create monitor");
        let last_observed_hash = Arc::new(Mutex::new(None));
        let suppression_registry = Arc::new(Mutex::new(Vec::<SuppressionEntry>::new()));

        let disposition = monitor
            .process_text_capture_with_analysis(
                &last_observed_hash,
                &suppression_registry,
                None,
                &PasteboardMarkers::default(),
                r#"https://www.right.codes/dashboard""#,
                |text| monitor.analysis_service.analyze(text),
            )
            .await
            .expect("process normalized url capture");

        assert_eq!(disposition, Some(CaptureDisposition::Persist));

        let entry = rx.recv().await.expect("receive normalized clipboard entry");
        let normalized_url = "https://www.right.codes/dashboard";
        assert_eq!(entry.content_data.as_deref(), Some(normalized_url));
        assert_eq!(
            entry.content_hash,
            calculate_content_hash(normalized_url.as_bytes())
        );
        assert_eq!(entry.content_subtype.as_deref(), Some("url"));
    }

    #[test]
    fn test_build_file_metadata_json_uses_existing_file_metadata_contract() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let file_path = temp_dir.path().join("preview.txt");
        fs::write(&file_path, "hello").expect("write file");

        let metadata = build_file_metadata_json(&file_path).expect("build file metadata");
        let file_metadata = metadata
            .get("file_metadata")
            .and_then(|value| value.as_object())
            .expect("file_metadata object");

        assert_eq!(
            file_metadata.get("name").and_then(|value| value.as_str()),
            Some("preview.txt")
        );
        assert_eq!(
            file_metadata
                .get("extension")
                .and_then(|value| value.as_str()),
            Some("txt")
        );
        assert_eq!(
            file_metadata
                .get("is_directory")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            file_metadata
                .get("size_bytes")
                .and_then(|value| value.as_u64()),
            Some(5)
        );
    }
}
