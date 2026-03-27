use crate::config::AppConfig;
use crate::models::{ClipboardEntry, Statistics};
use crate::state::AppState;
use crate::updater::{UpdateInfo, UpdateManager};
use crate::utils::app_icon_extractor::AppIconExtractor;
use crate::utils::app_list::{AppListManager, InstalledApp};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{State, Window};
use tauri_plugin_aptabase::EventTracker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub db_size_bytes: u64,
    pub images_size_bytes: u64,
    pub total_entries: i64,
    pub text_entries: i64,
    pub image_entries: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub entries_removed: u32,
    pub images_removed: u32,
    pub size_freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreviewKind {
    PlainText,
    Code,
    Markdown,
    Json,
    Image,
    Audio,
    Video,
    UrlCard,
    FileCard,
    Base64Text,
    Base64Binary,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecodedKind {
    Text,
    Json,
    Image,
    Audio,
    Video,
    Binary,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaInspection {
    pub source: String,
    pub source_kind: String,
    pub kind: Option<String>,
    pub mime: Option<String>,
    pub format: Option<String>,
    pub duration: Option<String>,
    pub bitrate: Option<String>,
    pub codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<String>,
    pub sample_rate: Option<String>,
    pub size_bytes: Option<u64>,
    pub ffprobe_used: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Base64DecodedPreview {
    pub decoded_kind: Option<DecodedKind>,
    pub mime: Option<String>,
    pub text_preview: Option<String>,
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolvedPreviewData {
    pub source_kind: String,
    pub mime: Option<String>,
    pub file_name: Option<String>,
    pub extension: Option<String>,
    pub size_bytes: Option<u64>,
    pub text_content: Option<String>,
    pub json_content: Option<Value>,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub video_url: Option<String>,
    pub media: Option<MediaInspection>,
    pub base64: Option<Base64DecodedPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPreviewResolution {
    pub final_url: String,
    pub status: Option<u16>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub preview_kind: PreviewKind,
    pub resolved: ResolvedPreviewData,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64PreviewResolution {
    pub preview_kind: PreviewKind,
    pub decoded_kind: DecodedKind,
    pub resolved: ResolvedPreviewData,
    pub filename_suggestion: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn start_monitoring(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let result = state.start_monitoring().await.map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("monitoring_started", None);
        });
    }
    result
}

#[tauri::command]
pub async fn stop_monitoring(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let result = state.stop_monitoring().await.map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("monitoring_stopped", None);
        });
    }
    result
}

#[tauri::command]
pub async fn get_clipboard_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
    offset: Option<i32>,
    search: Option<String>,
) -> Result<Vec<ClipboardEntry>, String> {
    state
        .get_clipboard_history(limit, offset, search)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_favorite(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let result = state.toggle_favorite(id).await.map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("favorite_toggled", None);
        });
    }
    result
}

#[tauri::command]
pub async fn delete_entry(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let result = state.delete_entry(id).await.map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("entry_deleted", None);
        });
    }
    result
}

#[tauri::command]
pub async fn clear_history(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let result = state.clear_history().await.map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("history_cleared", None);
        });
    }
    result
}

#[tauri::command]
pub async fn get_statistics(state: State<'_, AppState>) -> Result<Statistics, String> {
    state.get_statistics().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn copy_to_clipboard(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    content: String,
) -> Result<(), String> {
    let result = state
        .copy_to_clipboard(content)
        .await
        .map_err(|e| e.to_string());
    if result.is_ok() {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = app_handle.track_event("item_copied", None);
        });
    }
    result
}

#[tauri::command]
pub async fn paste_text(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    content: String,
) -> Result<(), String> {
    state
        .paste_text(content, Some(app_handle))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn paste_image(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    state
        .paste_image(file_path, Some(app_handle))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_file_with_system(file_path: String) -> Result<(), String> {
    use std::path::PathBuf;
    use std::process::Command;

    log::info!("[open_file_with_system] 打开文件: {}", file_path);

    // 如果是相对路径（如 imgs/xxx.png），转换为绝对路径
    let absolute_path = if file_path.starts_with("imgs/") {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Unable to get config directory".to_string())?;
        let app_dir = config_dir.join("clipboard-app");
        app_dir.join(&file_path)
    } else {
        PathBuf::from(&file_path)
    };

    if !absolute_path.exists() {
        return Err(format!("File not found: {:?}", absolute_path));
    }

    // 在 macOS 上使用 open 命令
    #[cfg(target_os = "macos")]
    {
        let result = Command::new("open").arg(&absolute_path).spawn();

        match result {
            Ok(_) => {
                log::info!("[open_file_with_system] 成功打开文件");
                Ok(())
            }
            Err(e) => {
                log::error!("[open_file_with_system] 打开文件失败: {}", e);
                Err(format!("Failed to open file: {}", e))
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("This feature is only supported on macOS".to_string())
    }
}

#[tauri::command]
pub async fn get_image_url(file_path: String) -> Result<String, String> {
    use base64::Engine;
    use std::fs;
    use std::path::PathBuf;

    // println!("[get_image_url] 请求加载图片: {}", file_path);

    // 如果是相对路径（如 imgs/xxx.png），转换为绝对路径
    let absolute_path = if file_path.starts_with("imgs/") {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Unable to get config directory".to_string())?;
        let app_dir = config_dir.join("clipboard-app");

        // 确保 imgs 目录存在
        let imgs_dir = app_dir.join("imgs");
        if !imgs_dir.exists() {
            log::info!("[get_image_url] 创建 imgs 目录: {:?}", imgs_dir);
            if let Err(e) = fs::create_dir_all(&imgs_dir) {
                return Err(format!("Failed to create imgs directory: {}", e));
            }
        }

        app_dir.join(&file_path)
    } else {
        PathBuf::from(&file_path)
    };

    // println!("[get_image_url] 绝对路径: {:?}", absolute_path);

    if !absolute_path.exists() {
        log::warn!("[get_image_url] 文件不存在: {:?}", absolute_path);
        // 列出 imgs 目录中的文件帮助调试
        if let Some(parent) = absolute_path.parent() {
            if parent.exists() {
                log::debug!("[get_image_url] 目录 {:?} 中的文件:", parent);
                if let Ok(entries) = fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        log::debug!("  - {:?}", entry.file_name());
                    }
                }
            }
        }
        return Err(format!("File not found: {:?}", absolute_path));
    }

    match fs::read(&absolute_path) {
        Ok(data) => {
            // println!("[get_image_url] 成功读取文件，大小: {} 字节", data.len());

            let extension = absolute_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("png");

            let mime_type = match extension.to_lowercase().as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "bin" => {
                    // 对于 .bin 文件，尝试检测实际格式
                    if data.len() >= 4 {
                        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                            "image/png"
                        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                            "image/jpeg"
                        } else if data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
                            "image/gif"
                        } else if data.starts_with(&[0x52, 0x49, 0x46, 0x46])
                            && data.len() >= 12
                            && &data[8..12] == b"WEBP"
                        {
                            "image/webp"
                        } else {
                            "image/png" // 默认使用 PNG
                        }
                    } else {
                        "image/png"
                    }
                }
                _ => "image/png",
            };

            // println!("[get_image_url] MIME 类型: {}", mime_type);

            let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(format!("data:{};base64,{}", mime_type, base64_data))
        }
        Err(e) => {
            log::error!("[get_image_url] 读取文件失败: {}", e);
            Err(format!("Failed to read file: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_app_icon(bundle_id: String) -> Result<Option<String>, String> {
    use base64::Engine;
    use std::fs;

    // println!("[get_app_icon] 请求应用图标: {}", bundle_id);

    let extractor = AppIconExtractor::new().map_err(|e| e.to_string())?;

    // 首先检查缓存
    if let Some(cached_path) = extractor.get_cached_icon_path(&bundle_id) {
        // println!("[get_app_icon] 找到缓存图标: {:?}", cached_path);

        match fs::read(&cached_path) {
            Ok(data) => {
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
                return Ok(Some(format!("data:image/png;base64,{}", base64_data)));
            }
            Err(e) => {
                log::warn!("[get_app_icon] 读取缓存图标失败: {}", e);
                // 继续尝试提取新图标
            }
        }
    }

    // 提取并缓存图标
    match extractor.extract_and_cache_icon(&bundle_id) {
        Ok(Some(icon_path)) => {
            log::info!("[get_app_icon] 成功提取图标: {:?}", icon_path);

            match fs::read(&icon_path) {
                Ok(data) => {
                    let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
                    Ok(Some(format!("data:image/png;base64,{}", base64_data)))
                }
                Err(e) => {
                    log::error!("[get_app_icon] 读取图标文件失败: {}", e);
                    Ok(None)
                }
            }
        }
        Ok(None) => {
            log::warn!("[get_app_icon] 无法为 {} 获取图标", bundle_id);
            Ok(None)
        }
        Err(e) => {
            log::error!("[get_app_icon] 提取图标出错: {}", e);
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn convert_and_scale_image(
    file_path: String,
    format: String,
    scale: f32,
    _skip_recording: bool,
) -> Result<String, String> {
    use image::DynamicImage;
    use std::fs;
    use std::path::PathBuf;

    log::info!(
        "[convert_and_scale_image] 转换图片: {}, 格式: {}, 缩放: {}%",
        file_path,
        format,
        (scale * 100.0) as i32
    );

    // 转换为绝对路径
    let absolute_path = if file_path.starts_with("imgs/") {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Unable to get config directory".to_string())?;
        config_dir.join("clipboard-app").join(&file_path)
    } else {
        PathBuf::from(&file_path)
    };

    if !absolute_path.exists() {
        return Err(format!("File not found: {:?}", absolute_path));
    }

    // 读取原始图片
    let img_data = fs::read(&absolute_path).map_err(|e| format!("Failed to read image: {}", e))?;

    let img =
        image::load_from_memory(&img_data).map_err(|e| format!("Failed to decode image: {}", e))?;

    // 缩放图片
    let (width, height) = (img.width(), img.height());
    let new_width = ((width as f32) * scale) as u32;
    let new_height = ((height as f32) * scale) as u32;

    let scaled_img = if scale != 1.0 {
        log::debug!(
            "[convert_and_scale_image] 缩放从 {}x{} 到 {}x{}",
            width,
            height,
            new_width,
            new_height
        );
        img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // 转换格式并编码
    let mut buffer = Vec::new();
    let output_format = match format.to_lowercase().as_str() {
        "jpeg" | "jpg" => {
            // JPEG不支持透明度，需要先转换
            let rgb_img = DynamicImage::ImageRgb8(scaled_img.to_rgb8());
            rgb_img
                .write_to(
                    &mut std::io::Cursor::new(&mut buffer),
                    image::ImageFormat::Jpeg,
                )
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
            "jpeg"
        }
        "webp" => {
            // WebP支持透明度
            scaled_img
                .write_to(
                    &mut std::io::Cursor::new(&mut buffer),
                    image::ImageFormat::WebP,
                )
                .map_err(|e| format!("Failed to encode WebP: {}", e))?;
            "webp"
        }
        _ => {
            // 默认PNG
            scaled_img
                .write_to(
                    &mut std::io::Cursor::new(&mut buffer),
                    image::ImageFormat::Png,
                )
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
            "png"
        }
    };

    log::info!(
        "[convert_and_scale_image] 转换完成，输出大小: {} 字节",
        buffer.len()
    );

    // 返回base64编码的图片数据
    let base64_data = general_purpose::STANDARD.encode(&buffer);
    Ok(format!(
        "data:image/{};base64,{}",
        output_format, base64_data
    ))
}

#[tauri::command]
pub async fn copy_converted_image(
    state: State<'_, AppState>,
    base64_data: String,
    _skip_recording: bool,
) -> Result<(), String> {
    log::info!("[copy_converted_image] 复制转换后的图片到剪贴板");

    // 解析base64数据
    let data_parts: Vec<&str> = base64_data.split(',').collect();
    if data_parts.len() != 2 {
        return Err("Invalid base64 data format".to_string());
    }

    let base64_content = data_parts[1];
    let image_data = general_purpose::STANDARD
        .decode(base64_content)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // 将图片数据写入临时文件
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("clipboard_temp_{}.png", uuid::Uuid::new_v4()));

    std::fs::write(&temp_file, &image_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // 暂时禁用监控以避免记录
    state.set_skip_next_clipboard_change(true).await;

    // 复制到剪贴板
    let result = state
        .copy_image_to_clipboard(temp_file.to_str().unwrap().to_string())
        .await;

    // 清理临时文件
    let _ = std::fs::remove_file(&temp_file);

    result.map_err(|e| e.to_string())
}

const URL_PREVIEW_MAX_BYTES: usize = 256 * 1024;
const BASE64_TEXT_PREVIEW_MAX_CHARS: usize = 8192;
const BASE64_DATA_URL_MAX_BYTES: usize = 2 * 1024 * 1024;

struct ParsedBase64Input {
    mime: Option<String>,
    payload: String,
}

fn normalize_mime(input: &str) -> String {
    input
        .split(';')
        .next()
        .unwrap_or(input)
        .trim()
        .to_lowercase()
}

fn extension_from_mime(mime: &str) -> Option<String> {
    let ext = match normalize_mime(mime).as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "audio/mp4" => "m4a",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "application/json" => "json",
        "text/plain" => "txt",
        "text/markdown" => "md",
        "application/pdf" => "pdf",
        "application/octet-stream" => "bin",
        _ => return None,
    };
    Some(ext.to_string())
}

fn preview_kind_from_mime(mime: &str) -> PreviewKind {
    let mime = normalize_mime(mime);
    if mime.starts_with("image/") {
        return PreviewKind::Image;
    }
    if mime.starts_with("video/") {
        return PreviewKind::Video;
    }
    if mime.starts_with("audio/") {
        return PreviewKind::Audio;
    }
    if mime == "application/json" || mime.ends_with("+json") {
        return PreviewKind::Json;
    }
    if mime == "text/markdown" {
        return PreviewKind::Markdown;
    }
    if mime.starts_with("text/html")
        || mime.starts_with("text/css")
        || mime.starts_with("application/javascript")
        || mime.starts_with("text/javascript")
        || mime.starts_with("application/xml")
        || mime.starts_with("text/xml")
    {
        return PreviewKind::Code;
    }
    if mime.starts_with("text/") {
        return PreviewKind::PlainText;
    }
    if mime == "application/pdf" {
        return PreviewKind::FileCard;
    }
    PreviewKind::UrlCard
}

fn preview_kind_from_url_path(url: &url::Url) -> PreviewKind {
    let path = url.path().to_lowercase();
    if path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
    {
        return PreviewKind::Image;
    }
    if path.ends_with(".mp4")
        || path.ends_with(".webm")
        || path.ends_with(".mov")
        || path.ends_with(".mkv")
    {
        return PreviewKind::Video;
    }
    if path.ends_with(".mp3")
        || path.ends_with(".wav")
        || path.ends_with(".ogg")
        || path.ends_with(".m4a")
        || path.ends_with(".flac")
    {
        return PreviewKind::Audio;
    }
    if path.ends_with(".json") {
        return PreviewKind::Json;
    }
    if path.ends_with(".md") || path.ends_with(".markdown") {
        return PreviewKind::Markdown;
    }
    if path.ends_with(".js")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".jsx")
        || path.ends_with(".css")
        || path.ends_with(".html")
        || path.ends_with(".xml")
    {
        return PreviewKind::Code;
    }
    PreviewKind::UrlCard
}

fn parse_base64_input(input: &str) -> Result<ParsedBase64Input, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Input is empty".to_string());
    }

    if !trimmed.starts_with("data:") {
        let payload: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
        if payload.is_empty() {
            return Err("Base64 payload is empty".to_string());
        }
        return Ok(ParsedBase64Input {
            mime: None,
            payload,
        });
    }

    let comma_idx = trimmed
        .find(',')
        .ok_or_else(|| "Invalid data URL: missing comma".to_string())?;
    let header = &trimmed[..comma_idx];
    let payload_raw = &trimmed[comma_idx + 1..];
    if !header.to_ascii_lowercase().contains(";base64") {
        return Err("Only base64 data URLs are supported".to_string());
    }

    let mime = header.strip_prefix("data:").and_then(|h| {
        let first = h.split(';').next().unwrap_or("").trim();
        if first.is_empty() {
            None
        } else {
            Some(normalize_mime(first))
        }
    });
    let payload: String = payload_raw.chars().filter(|c| !c.is_whitespace()).collect();
    if payload.is_empty() {
        return Err("Base64 payload is empty".to_string());
    }

    Ok(ParsedBase64Input { mime, payload })
}

fn decode_base64_payload(payload: &str) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD
        .decode(payload)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(payload))
        .or_else(|_| general_purpose::URL_SAFE.decode(payload))
        .map_err(|e| format!("Failed to decode base64: {}", e))
}

fn truncate_by_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect()
}

fn parse_json_if_possible(text: &str) -> Option<Value> {
    serde_json::from_str::<Value>(text).ok()
}

async fn read_response_text_limited(
    response: &mut reqwest::Response,
    max_bytes: usize,
) -> Result<(String, bool), String> {
    let mut buffer = Vec::new();
    let mut truncated = false;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?
    {
        let remaining = max_bytes.saturating_sub(buffer.len());
        if remaining == 0 {
            truncated = true;
            break;
        }

        if chunk.len() > remaining {
            buffer.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            break;
        }

        buffer.extend_from_slice(&chunk);
    }

    Ok((String::from_utf8_lossy(&buffer).to_string(), truncated))
}

fn printable_ratio(text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let printable = text
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .count();
    printable as f32 / text.chars().count() as f32
}

fn detect_decoded_kind(
    bytes: &[u8],
    mime_hint: Option<&str>,
) -> (DecodedKind, PreviewKind, Option<String>) {
    let mut mime = mime_hint.map(normalize_mime);
    if mime.is_none() {
        mime = infer::get(bytes).map(|kind| kind.mime_type().to_string());
    }

    if let Some(ref known_mime) = mime {
        let preview_kind = preview_kind_from_mime(known_mime);
        match preview_kind {
            PreviewKind::Image => return (DecodedKind::Image, PreviewKind::Image, mime),
            PreviewKind::Audio => return (DecodedKind::Audio, PreviewKind::Audio, mime),
            PreviewKind::Video => return (DecodedKind::Video, PreviewKind::Video, mime),
            PreviewKind::Json => return (DecodedKind::Json, PreviewKind::Json, mime),
            PreviewKind::Code => return (DecodedKind::Text, PreviewKind::Code, mime),
            PreviewKind::Markdown => return (DecodedKind::Text, PreviewKind::Markdown, mime),
            PreviewKind::PlainText => return (DecodedKind::Text, PreviewKind::Base64Text, mime),
            _ => {}
        }
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        if parse_json_if_possible(text).is_some() {
            return (
                DecodedKind::Json,
                PreviewKind::Json,
                mime.or(Some("application/json".into())),
            );
        }
        if printable_ratio(text) > 0.85 {
            return (
                DecodedKind::Text,
                PreviewKind::Base64Text,
                mime.or(Some("text/plain".into())),
            );
        }
    }

    (
        DecodedKind::Binary,
        PreviewKind::Base64Binary,
        mime.or(Some("application/octet-stream".into())),
    )
}

fn to_u32_value(value: Option<&Value>) -> Option<u32> {
    if let Some(v) = value {
        if let Some(num) = v.as_u64() {
            return u32::try_from(num).ok();
        }
        if let Some(text) = v.as_str() {
            return text.parse::<u32>().ok();
        }
    }
    None
}

fn source_kind_from_input(source: &str) -> String {
    let trimmed = source.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return "remote".to_string();
    }
    if trimmed.starts_with("data:") {
        return "data_url".to_string();
    }
    "local".to_string()
}

async fn inspect_media_source_internal(
    source: &str,
    mime_hint: Option<&str>,
    size_hint: Option<u64>,
) -> MediaInspection {
    let mut inspection = MediaInspection {
        source: source.to_string(),
        source_kind: source_kind_from_input(source),
        mime: mime_hint.map(normalize_mime),
        size_bytes: size_hint,
        ..Default::default()
    };

    if source.starts_with("data:") {
        match parse_base64_input(source).and_then(|parsed| {
            let decoded = decode_base64_payload(&parsed.payload)?;
            Ok((parsed, decoded))
        }) {
            Ok((parsed, decoded)) => {
                inspection.mime = parsed
                    .mime
                    .clone()
                    .or_else(|| infer::get(&decoded).map(|kind| kind.mime_type().to_string()));
                inspection.size_bytes = Some(decoded.len() as u64);
                if let Some(ref mime) = inspection.mime {
                    inspection.kind = match preview_kind_from_mime(mime) {
                        PreviewKind::Image => Some("image".to_string()),
                        PreviewKind::Audio => Some("audio".to_string()),
                        PreviewKind::Video => Some("video".to_string()),
                        _ => None,
                    };
                    inspection.format = extension_from_mime(mime);
                }
                if inspection.kind.as_deref() == Some("image") {
                    if let Ok(img) = image::load_from_memory(&decoded) {
                        inspection.width = Some(img.width());
                        inspection.height = Some(img.height());
                    }
                }
            }
            Err(err) => {
                inspection.error = Some(err);
            }
        }
        return inspection;
    }

    if inspection.source_kind == "local" {
        let local_path = std::path::PathBuf::from(source);
        if let Ok(meta) = std::fs::metadata(&local_path) {
            inspection.size_bytes = Some(meta.len());
        }
        if inspection.mime.is_none() {
            if let Ok(bytes) = std::fs::read(&local_path) {
                inspection.mime = infer::get(&bytes).map(|k| k.mime_type().to_string());
            }
        }
    }

    if let Some(ref mime) = inspection.mime {
        inspection.kind = match preview_kind_from_mime(mime) {
            PreviewKind::Image => Some("image".to_string()),
            PreviewKind::Audio => Some("audio".to_string()),
            PreviewKind::Video => Some("video".to_string()),
            _ => inspection.kind.clone(),
        };
        inspection.format = extension_from_mime(mime).or_else(|| inspection.format.clone());
    }

    match extract_media_metadata(source.to_string()).await {
        Ok(metadata) => {
            inspection.ffprobe_used = true;
            inspection.duration = metadata
                .get("duration")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            inspection.bitrate = metadata
                .get("bitrate")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            inspection.codec = metadata
                .get("codec")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            inspection.fps = metadata
                .get("fps")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            inspection.sample_rate = metadata.get("sample_rate").and_then(|v| {
                v.as_str()
                    .map(ToString::to_string)
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
            });
            inspection.width = to_u32_value(metadata.get("width"));
            inspection.height = to_u32_value(metadata.get("height"));

            if inspection.kind.is_none() {
                if inspection.width.is_some() && inspection.height.is_some() {
                    inspection.kind = Some("video".to_string());
                } else if inspection.sample_rate.is_some() {
                    inspection.kind = Some("audio".to_string());
                }
            }
        }
        Err(err) => {
            inspection.error = Some(err);
        }
    }

    if inspection.kind.as_deref() == Some("image")
        && inspection.width.is_none()
        && inspection.source_kind == "local"
    {
        if let Ok(img) = image::open(source) {
            inspection.width = Some(img.width());
            inspection.height = Some(img.height());
        }
    }

    inspection
}

#[tauri::command]
pub async fn resolve_url_preview(url: String) -> Result<UrlPreviewResolution, String> {
    use std::time::Duration;

    let parsed_url = url::Url::parse(url.trim())
        .map_err(|_| "Only absolute HTTP(S) URLs are supported".to_string())?;
    if !matches!(parsed_url.scheme(), "http" | "https") || parsed_url.host_str().is_none() {
        return Err("Only absolute HTTP(S) URLs are supported".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Dance/preview-resolver")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut resolution = UrlPreviewResolution {
        final_url: parsed_url.to_string(),
        status: None,
        content_type: None,
        content_length: None,
        preview_kind: PreviewKind::UrlCard,
        resolved: ResolvedPreviewData {
            source_kind: "remote".to_string(),
            ..Default::default()
        },
        error: None,
    };

    let mut response = match client.get(parsed_url.clone()).send().await {
        Ok(resp) => resp,
        Err(err) => return Err(format!("Network request failed: {}", err)),
    };

    let final_url = response.url().clone();
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(normalize_mime);
    let content_length = response.content_length();

    let fallback_kind = preview_kind_from_url_path(&final_url);
    let preview_kind = content_type
        .as_deref()
        .map(preview_kind_from_mime)
        .filter(|kind| *kind != PreviewKind::UrlCard)
        .unwrap_or(fallback_kind);

    resolution.final_url = final_url.to_string();
    resolution.status = Some(status.as_u16());
    resolution.content_type = content_type.clone();
    resolution.content_length = content_length;
    resolution.preview_kind = preview_kind.clone();

    resolution.resolved.mime = content_type.clone();
    resolution.resolved.size_bytes = content_length;
    resolution.resolved.extension = content_type
        .as_deref()
        .and_then(extension_from_mime)
        .or_else(|| {
            let path = final_url.path();
            std::path::Path::new(path)
                .extension()
                .and_then(|v| v.to_str())
                .map(|v| v.to_lowercase())
        });
    resolution.resolved.file_name = final_url
        .path_segments()
        .and_then(|mut seg| seg.next_back())
        .filter(|name| !name.is_empty())
        .map(ToString::to_string);

    if !status.is_success() {
        return Err(format!("HTTP error: {}", status));
    }

    match preview_kind {
        PreviewKind::Image => {
            resolution.resolved.image_url = Some(final_url.to_string());
            resolution.resolved.media = Some(
                inspect_media_source_internal(
                    final_url.as_str(),
                    content_type.as_deref(),
                    content_length,
                )
                .await,
            );
        }
        PreviewKind::Audio => {
            resolution.resolved.audio_url = Some(final_url.to_string());
            resolution.resolved.media = Some(
                inspect_media_source_internal(
                    final_url.as_str(),
                    content_type.as_deref(),
                    content_length,
                )
                .await,
            );
        }
        PreviewKind::Video => {
            resolution.resolved.video_url = Some(final_url.to_string());
            resolution.resolved.media = Some(
                inspect_media_source_internal(
                    final_url.as_str(),
                    content_type.as_deref(),
                    content_length,
                )
                .await,
            );
        }
        PreviewKind::Json | PreviewKind::Markdown | PreviewKind::PlainText | PreviewKind::Code => {
            let (text, truncated) =
                read_response_text_limited(&mut response, URL_PREVIEW_MAX_BYTES).await?;
            resolution.resolved.text_content = Some(text.clone());
            if preview_kind == PreviewKind::Json {
                resolution.resolved.json_content = parse_json_if_possible(&text);
            }
            if truncated {
                resolution.error = Some(format!(
                    "Content truncated to {} bytes for preview",
                    URL_PREVIEW_MAX_BYTES
                ));
            }
        }
        _ => {}
    }

    Ok(resolution)
}

#[tauri::command]
pub async fn decode_base64_preview(input: String) -> Result<Base64PreviewResolution, String> {
    let parsed = parse_base64_input(&input)?;
    let decoded_bytes = decode_base64_payload(&parsed.payload)?;
    let (detected_decoded_kind, detected_preview_kind, detected_mime) =
        detect_decoded_kind(&decoded_bytes, parsed.mime.as_deref());
    let mut decoded_kind = detected_decoded_kind.clone();
    let mut preview_kind = detected_preview_kind;
    let mime = detected_mime.or(parsed.mime.clone());
    let extension = mime.as_deref().and_then(extension_from_mime);

    let mut resolved = ResolvedPreviewData {
        source_kind: "decoded".to_string(),
        mime: mime.clone(),
        extension: extension.clone(),
        size_bytes: Some(decoded_bytes.len() as u64),
        ..Default::default()
    };

    let mut base64_preview = Base64DecodedPreview {
        mime: mime.clone(),
        ..Default::default()
    };

    let mut error = None;
    match detected_decoded_kind {
        DecodedKind::Json => {
            let text = String::from_utf8_lossy(&decoded_bytes).to_string();
            resolved.json_content = parse_json_if_possible(&text);
            resolved.text_content = Some(truncate_by_chars(&text, URL_PREVIEW_MAX_BYTES));
            base64_preview.text_preview =
                Some(truncate_by_chars(&text, BASE64_TEXT_PREVIEW_MAX_CHARS));
        }
        DecodedKind::Text => {
            let text = String::from_utf8_lossy(&decoded_bytes).to_string();
            resolved.text_content = Some(truncate_by_chars(&text, URL_PREVIEW_MAX_BYTES));
            base64_preview.text_preview =
                Some(truncate_by_chars(&text, BASE64_TEXT_PREVIEW_MAX_CHARS));
        }
        DecodedKind::Image => {
            if let Some(ref m) = mime {
                if decoded_bytes.len() <= BASE64_DATA_URL_MAX_BYTES {
                    let data_url = format!("data:{};base64,{}", m, parsed.payload);
                    resolved.image_url = Some(data_url.clone());
                    base64_preview.data_url = Some(data_url);
                    resolved.media = Some(
                        inspect_media_source_internal(
                            &format!("data:{};base64,{}", m, parsed.payload),
                            Some(m),
                            Some(decoded_bytes.len() as u64),
                        )
                        .await,
                    );
                } else {
                    error = Some("Decoded media is too large to inline as data URL".to_string());
                    decoded_kind = DecodedKind::Binary;
                    preview_kind = PreviewKind::Base64Binary;
                }
            }
        }
        DecodedKind::Audio => {
            if let Some(ref m) = mime {
                if decoded_bytes.len() <= BASE64_DATA_URL_MAX_BYTES {
                    let data_url = format!("data:{};base64,{}", m, parsed.payload);
                    resolved.audio_url = Some(data_url.clone());
                    base64_preview.data_url = Some(data_url);
                } else {
                    error = Some("Decoded media is too large to inline as data URL".to_string());
                    decoded_kind = DecodedKind::Binary;
                    preview_kind = PreviewKind::Base64Binary;
                }
            }
        }
        DecodedKind::Video => {
            if let Some(ref m) = mime {
                if decoded_bytes.len() <= BASE64_DATA_URL_MAX_BYTES {
                    let data_url = format!("data:{};base64,{}", m, parsed.payload);
                    resolved.video_url = Some(data_url.clone());
                    base64_preview.data_url = Some(data_url);
                } else {
                    error = Some("Decoded media is too large to inline as data URL".to_string());
                    decoded_kind = DecodedKind::Binary;
                    preview_kind = PreviewKind::Base64Binary;
                }
            }
        }
        DecodedKind::Binary | DecodedKind::Unknown => {}
    }

    base64_preview.decoded_kind = Some(decoded_kind.clone());
    resolved.base64 = Some(base64_preview);
    let filename_suggestion = extension
        .as_deref()
        .map(|ext| format!("decoded.{}", ext))
        .or_else(|| Some("decoded.bin".to_string()));

    Ok(Base64PreviewResolution {
        preview_kind,
        decoded_kind,
        resolved,
        filename_suggestion,
        error,
    })
}

#[tauri::command]
pub async fn inspect_media_source(source: String) -> Result<MediaInspection, String> {
    let source_trimmed = source.trim();
    if source_trimmed.is_empty() {
        return Err("Source is empty".to_string());
    }
    Ok(inspect_media_source_internal(source_trimmed, None, None).await)
}

#[tauri::command]
pub async fn fetch_url_content(url: String) -> Result<String, String> {
    use std::time::Duration;

    log::info!("[fetch_url_content] 请求获取URL内容: {}", url);

    let parsed_url = url::Url::parse(url.trim())
        .map_err(|_| "Only absolute HTTP(S) URLs are supported".to_string())?;

    if !matches!(parsed_url.scheme(), "http" | "https") || parsed_url.host_str().is_none() {
        return Err("Only absolute HTTP(S) URLs are supported".to_string());
    }

    // 创建HTTP客户端，配置超时
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 发起HTTP请求
    match client.get(parsed_url).send().await {
        Ok(mut response) => {
            if response.status().is_success() {
                match read_response_text_limited(&mut response, URL_PREVIEW_MAX_BYTES).await {
                    Ok((content, truncated)) => {
                        log::info!(
                            "[fetch_url_content] 成功获取内容，长度: {} 字符，截断: {}",
                            content.len(),
                            truncated
                        );
                        Ok(content)
                    }
                    Err(e) => {
                        log::error!("[fetch_url_content] 读取响应内容失败: {}", e);
                        Err(e)
                    }
                }
            } else {
                log::error!("[fetch_url_content] HTTP错误状态: {}", response.status());
                Err(format!("HTTP error: {}", response.status()))
            }
        }
        Err(e) => {
            log::error!("[fetch_url_content] 网络请求失败: {}", e);
            Err(format!("Network request failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn check_ffprobe_available() -> Result<bool, String> {
    use std::process::Command;

    log::debug!("[check_ffprobe_available] 检查 ffprobe 是否可用");

    // 尝试执行 ffprobe -version 命令
    match Command::new("ffprobe").arg("-version").output() {
        Ok(output) => {
            let available = output.status.success();
            log::debug!("[check_ffprobe_available] ffprobe 可用: {}", available);
            Ok(available)
        }
        Err(e) => {
            log::debug!("[check_ffprobe_available] ffprobe 不可用: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn extract_media_metadata(url: String) -> Result<serde_json::Value, String> {
    use std::process::Command;

    log::info!("[extract_media_metadata] 提取媒体元数据: {}", url);

    // 首先检查 ffprobe 是否可用
    if !check_ffprobe_available().await? {
        return Err("FFprobe not available".to_string());
    }

    // 使用 ffprobe 提取媒体信息
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        log::error!("[extract_media_metadata] ffprobe 执行失败: {}", error_msg);
        return Err(format!("FFprobe execution failed: {}", error_msg));
    }

    let json_output = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output as UTF-8: {}", e))?;

    // 解析 JSON 输出
    let metadata: serde_json::Value = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse ffprobe JSON output: {}", e))?;

    // 提取关键信息
    let mut result = serde_json::Map::new();

    if let Some(format) = metadata.get("format") {
        if let Some(duration) = format.get("duration") {
            if let Some(duration_str) = duration.as_str() {
                if let Ok(duration_f64) = duration_str.parse::<f64>() {
                    let minutes = (duration_f64 / 60.0) as u32;
                    let seconds = (duration_f64 % 60.0) as u32;
                    result.insert(
                        "duration".to_string(),
                        serde_json::Value::String(format!("{}:{:02}", minutes, seconds)),
                    );
                }
            }
        }
        if let Some(bit_rate) = format.get("bit_rate") {
            if let Some(bit_rate_str) = bit_rate.as_str() {
                if let Ok(bit_rate_i64) = bit_rate_str.parse::<i64>() {
                    let kbps = bit_rate_i64 / 1000;
                    result.insert(
                        "bitrate".to_string(),
                        serde_json::Value::String(format!("{} kbps", kbps)),
                    );
                }
            }
        }
    }

    if let Some(streams) = metadata.get("streams") {
        if let Some(streams_array) = streams.as_array() {
            for stream in streams_array {
                if let Some(codec_type) = stream.get("codec_type") {
                    if codec_type == "video" {
                        if let Some(width) = stream.get("width") {
                            result.insert("width".to_string(), width.clone());
                        }
                        if let Some(height) = stream.get("height") {
                            result.insert("height".to_string(), height.clone());
                        }
                        if let Some(codec_name) = stream.get("codec_name") {
                            result.insert("codec".to_string(), codec_name.clone());
                        }
                        if let Some(r_frame_rate) = stream.get("r_frame_rate") {
                            if let Some(fps_str) = r_frame_rate.as_str() {
                                // 处理分数形式的帧率，如 "30/1"
                                if let Some(slash_pos) = fps_str.find('/') {
                                    let numerator: f64 =
                                        fps_str[..slash_pos].parse().unwrap_or(0.0);
                                    let denominator: f64 =
                                        fps_str[slash_pos + 1..].parse().unwrap_or(1.0);
                                    if denominator != 0.0 {
                                        let fps = numerator / denominator;
                                        result.insert(
                                            "fps".to_string(),
                                            serde_json::Value::String(format!("{:.2}", fps)),
                                        );
                                    }
                                }
                            }
                        }
                    } else if codec_type == "audio" {
                        if let Some(sample_rate) = stream.get("sample_rate") {
                            result.insert("sample_rate".to_string(), sample_rate.clone());
                        }
                        if result.get("codec").is_none() {
                            if let Some(codec_name) = stream.get("codec_name") {
                                result.insert("codec".to_string(), codec_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    log::info!("[extract_media_metadata] 成功提取元数据: {:?}", result);
    Ok(serde_json::Value::Object(result))
}

// Configuration commands
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.get_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    state.update_config(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cache_statistics(state: State<'_, AppState>) -> Result<CacheStatistics, String> {
    state
        .get_cache_statistics()
        .await
        .map_err(|e| e.to_string())
}

// Global shortcut commands
#[tauri::command]
pub async fn register_global_shortcut(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    shortcut: String,
) -> Result<(), String> {
    state
        .register_global_shortcut(app_handle, shortcut)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unregister_global_shortcut(state: State<'_, AppState>) -> Result<(), String> {
    state
        .unregister_global_shortcut()
        .await
        .map_err(|e| e.to_string())
}

// Auto startup commands
#[tauri::command]
pub async fn set_auto_startup(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state
        .set_auto_startup(enabled)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_auto_startup_status(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .get_auto_startup_status()
        .await
        .map_err(|e| e.to_string())
}

// Cache cleanup command
#[tauri::command]
pub async fn cleanup_expired_entries(state: State<'_, AppState>) -> Result<CleanupResult, String> {
    state
        .cleanup_expired_entries()
        .await
        .map_err(|e| e.to_string())
}

// App list commands
#[tauri::command]
pub async fn get_installed_applications() -> Result<Vec<InstalledApp>, String> {
    log::info!("[get_installed_applications] Starting to load applications...");

    match AppListManager::get_installed_applications() {
        Ok(apps) => {
            log::info!(
                "[get_installed_applications] Successfully loaded {} applications",
                apps.len()
            );
            Ok(apps)
        }
        Err(e) => {
            log::error!(
                "[get_installed_applications] Error loading applications: {}",
                e
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_common_excluded_apps() -> Result<Vec<InstalledApp>, String> {
    Ok(AppListManager::get_common_excluded_apps())
}

// Shortcut validation command
#[tauri::command]
pub async fn validate_shortcut(shortcut: String) -> Result<bool, String> {
    // Basic validation for shortcut format
    if shortcut.is_empty() {
        return Ok(false);
    }

    // Check for required modifier keys (at least one)
    let has_modifier = shortcut.contains("Cmd")
        || shortcut.contains("Ctrl")
        || shortcut.contains("Alt")
        || shortcut.contains("Shift");

    if !has_modifier {
        return Ok(false);
    }

    // Check for system shortcut conflicts (basic check)
    let system_shortcuts = [
        "CmdOrCtrl+Q",   // Quit
        "CmdOrCtrl+W",   // Close window
        "CmdOrCtrl+H",   // Hide window
        "CmdOrCtrl+M",   // Minimize
        "CmdOrCtrl+Tab", // Switch apps
    ];

    if system_shortcuts.contains(&shortcut.as_str()) {
        return Ok(false);
    }

    Ok(true)
}

// Update commands
#[tauri::command]
pub async fn check_for_update(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<UpdateInfo, String> {
    log::info!("[check_for_update] Manual update check requested");

    // Update last check time in config
    let mut config = state.get_config().await.map_err(|e| e.to_string())?;
    config.last_update_check = Some(UpdateManager::get_current_timestamp());
    let _ = state.update_config(config).await;

    match UpdateManager::check_for_updates(&app_handle).await {
        Ok(Some(update_info)) => {
            log::info!("[check_for_update] Check completed successfully - update available");
            Ok(update_info)
        }
        Ok(None) => {
            log::info!("[check_for_update] Check completed successfully - no updates");
            Ok(UpdateInfo {
                version: app_handle.package_info().version.to_string(),
                notes: None,
                pub_date: None,
                available: false,
            })
        }
        Err(e) => {
            log::error!("[check_for_update] Update check failed: {}", e);
            Err(format!("更新检查失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn install_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    UpdateManager::download_and_install(&app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn should_check_for_updates(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.get_config().await.map_err(|e| e.to_string())?;

    // Check if auto-update is enabled
    if !config.auto_update {
        return Ok(false);
    }

    // Check if enough time has passed since last check
    Ok(UpdateManager::should_check_for_updates(
        config.last_update_check.as_deref(),
    ))
}

#[tauri::command]
pub async fn set_window_title(window: Window, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}

// Log management commands
#[tauri::command]
pub async fn get_log_content() -> Result<String, String> {
    use dirs;
    use std::fs;

    // Tauri v2 LogDir location on macOS: ~/Library/Logs/{app_identifier}/
    let log_file = dirs::home_dir()
        .ok_or("无法获取用户目录")?
        .join("Library")
        .join("Logs")
        .join("com.dance.app")
        .join("dance.log");

    if !log_file.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(&log_file).map_err(|e| format!("读取日志文件失败: {}", e))
}

#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    use dirs;
    use std::fs;

    // Tauri v2 LogDir location on macOS: ~/Library/Logs/{app_identifier}/
    let log_file = dirs::home_dir()
        .ok_or("无法获取用户目录")?
        .join("Library")
        .join("Logs")
        .join("com.dance.app")
        .join("dance.log");

    if log_file.exists() {
        fs::write(&log_file, "").map_err(|e| format!("清空日志文件失败: {}", e))?;
        log::info!("日志文件已清空");
    }

    Ok(())
}

#[tauri::command]
pub async fn set_log_level(level: String) -> Result<(), String> {
    let _log_level = match level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => return Err("无效的日志级别".to_string()),
    };

    // Note: Tauri log plugin doesn't support runtime level changes directly
    // This is a placeholder - the actual implementation would need to be done differently
    log::info!("请求设置日志级别为: {}", level);
    Ok(())
}

#[tauri::command]
pub async fn get_current_log_level() -> Result<String, String> {
    // This is a simplified implementation
    // In practice, you'd need to store the current level somewhere
    Ok("info".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_base64_input_with_data_url() {
        let input = "data:text/plain;base64,SGVsbG8=";
        let parsed = parse_base64_input(input).expect("data url should parse");
        assert_eq!(parsed.mime, Some("text/plain".to_string()));
        assert_eq!(parsed.payload, "SGVsbG8=");
    }

    #[test]
    fn test_parse_base64_input_with_plain_base64() {
        let input = " SGVsbG8= ";
        let parsed = parse_base64_input(input).expect("base64 should parse");
        assert_eq!(parsed.mime, None);
        assert_eq!(parsed.payload, "SGVsbG8=");
    }

    #[test]
    fn test_detect_decoded_kind_json() {
        let bytes = br#"{"name":"dance","ok":true}"#;
        let (decoded_kind, preview_kind, mime) =
            detect_decoded_kind(bytes, Some("application/json"));
        assert_eq!(decoded_kind, DecodedKind::Json);
        assert_eq!(preview_kind, PreviewKind::Json);
        assert_eq!(mime, Some("application/json".to_string()));
    }

    #[test]
    fn test_detect_decoded_kind_image() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let (decoded_kind, preview_kind, _) = detect_decoded_kind(&png_header, Some("image/png"));
        assert_eq!(decoded_kind, DecodedKind::Image);
        assert_eq!(preview_kind, PreviewKind::Image);
    }

    #[test]
    fn test_preview_kind_from_mime() {
        assert_eq!(
            preview_kind_from_mime("application/json"),
            PreviewKind::Json
        );
        assert_eq!(
            preview_kind_from_mime("text/markdown"),
            PreviewKind::Markdown
        );
        assert_eq!(preview_kind_from_mime("image/webp"), PreviewKind::Image);
        assert_eq!(preview_kind_from_mime("audio/mpeg"), PreviewKind::Audio);
        assert_eq!(preview_kind_from_mime("video/mp4"), PreviewKind::Video);
    }

    #[test]
    fn test_parse_json_if_possible_keeps_scalar_values() {
        assert_eq!(parse_json_if_possible("0"), Some(Value::from(0)));
        assert_eq!(parse_json_if_possible("false"), Some(Value::from(false)));
        assert_eq!(parse_json_if_possible("null"), Some(Value::Null));
    }

    #[tokio::test]
    async fn test_decode_base64_preview_downgrades_large_media_to_binary() {
        let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.resize(BASE64_DATA_URL_MAX_BYTES + 1, 0);
        let encoded = general_purpose::STANDARD.encode(bytes);
        let input = format!("data:image/png;base64,{}", encoded);

        let result = decode_base64_preview(input)
            .await
            .expect("base64 preview should decode");

        assert_eq!(result.decoded_kind, DecodedKind::Binary);
        assert_eq!(result.preview_kind, PreviewKind::Base64Binary);
        assert_eq!(
            result.error.as_deref(),
            Some("Decoded media is too large to inline as data URL")
        );
        assert_eq!(
            result
                .resolved
                .base64
                .as_ref()
                .and_then(|preview| preview.decoded_kind.as_ref()),
            Some(&DecodedKind::Binary)
        );
        assert!(result
            .resolved
            .base64
            .as_ref()
            .and_then(|preview| preview.data_url.as_ref())
            .is_none());
    }
}
