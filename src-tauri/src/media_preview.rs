use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

pub const URL_PREVIEW_MAX_BYTES: usize = 256 * 1024;
pub const BASE64_TEXT_PREVIEW_MAX_CHARS: usize = 8192;
pub const BASE64_DATA_URL_MAX_BYTES: usize = 2 * 1024 * 1024;
const REMOTE_IMAGE_METADATA_MAX_BYTES: usize = 10 * 1024 * 1024;

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
    pub title: Option<String>,
    pub description: Option<String>,
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

pub(crate) struct ParsedBase64Input {
    pub mime: Option<String>,
    pub payload: String,
}

pub fn normalize_mime(input: &str) -> String {
    input
        .split(';')
        .next()
        .unwrap_or(input)
        .trim()
        .to_lowercase()
}

pub fn extension_from_mime(mime: &str) -> Option<String> {
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

pub fn preview_kind_from_mime(mime: &str) -> PreviewKind {
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

pub fn is_html_content_type(mime: &str) -> bool {
    let mime = normalize_mime(mime);
    mime.starts_with("text/html") || mime == "application/xhtml+xml"
}

pub fn preview_kind_from_url_content_type(mime: &str) -> PreviewKind {
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

    PreviewKind::UrlCard
}

pub fn preview_kind_from_url_path(url: &url::Url) -> PreviewKind {
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
    PreviewKind::UrlCard
}

fn html_title_regex() -> &'static Regex {
    static TITLE_RE: OnceLock<Regex> = OnceLock::new();
    TITLE_RE.get_or_init(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap())
}

fn html_meta_tag_regex() -> &'static Regex {
    static META_TAG_RE: OnceLock<Regex> = OnceLock::new();
    META_TAG_RE.get_or_init(|| Regex::new(r"(?is)<meta\b[^>]*>").unwrap())
}

fn html_attr_regex() -> &'static Regex {
    static ATTR_RE: OnceLock<Regex> = OnceLock::new();
    ATTR_RE.get_or_init(|| {
        Regex::new(
            r#"(?is)([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))"#,
        )
        .unwrap()
    })
}

fn html_tag_regex() -> &'static Regex {
    static TAG_RE: OnceLock<Regex> = OnceLock::new();
    TAG_RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").unwrap())
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn normalize_html_summary_text(value: &str, max_chars: usize) -> Option<String> {
    let without_tags = html_tag_regex().replace_all(value, " ");
    let decoded = decode_basic_html_entities(&without_tags);
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }

    Some(truncate_by_chars(&collapsed, max_chars))
}

pub fn extract_html_preview_summary(html: &str) -> (Option<String>, Option<String>) {
    let title = html_title_regex()
        .captures(html)
        .and_then(|captures| captures.get(1))
        .and_then(|value| normalize_html_summary_text(value.as_str(), 160));

    let mut description = None;
    for tag_match in html_meta_tag_regex().find_iter(html) {
        let tag = tag_match.as_str();
        let mut attr_name = None::<String>;
        let mut attr_property = None::<String>;
        let mut attr_content = None::<String>;

        for attr_match in html_attr_regex().captures_iter(tag) {
            let Some(key_match) = attr_match.get(1) else {
                continue;
            };
            let key = key_match.as_str().to_ascii_lowercase();
            let value = attr_match
                .get(2)
                .or_else(|| attr_match.get(3))
                .or_else(|| attr_match.get(4))
                .map(|capture| capture.as_str().to_string())
                .unwrap_or_default();

            match key.as_str() {
                "name" => attr_name = Some(value.to_ascii_lowercase()),
                "property" => attr_property = Some(value.to_ascii_lowercase()),
                "content" => attr_content = Some(value),
                _ => {}
            }
        }

        let is_description_tag = matches!(
            attr_name.as_deref().or(attr_property.as_deref()),
            Some("description" | "og:description" | "twitter:description")
        );

        if is_description_tag {
            description = attr_content
                .as_deref()
                .and_then(|value| normalize_html_summary_text(value, 280));
            if description.is_some() {
                break;
            }
        }
    }

    if title.is_some() && title == description {
        return (title, None);
    }

    (title, description)
}

pub(crate) fn parse_base64_input(input: &str) -> Result<ParsedBase64Input, String> {
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

pub(crate) fn decode_base64_payload(payload: &str) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD
        .decode(payload)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(payload))
        .or_else(|_| general_purpose::URL_SAFE.decode(payload))
        .map_err(|e| format!("Failed to decode base64: {}", e))
}

pub(crate) fn truncate_by_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect()
}

pub(crate) fn parse_json_if_possible(text: &str) -> Option<Value> {
    serde_json::from_str::<Value>(text).ok()
}

pub async fn read_response_text_limited(
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

async fn read_response_bytes_limited(
    response: &mut reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?
    {
        let remaining = max_bytes.saturating_sub(buffer.len());
        if remaining == 0 || chunk.len() > remaining {
            return Err(format!(
                "Remote image is too large to inspect: more than {} bytes",
                max_bytes
            ));
        }

        buffer.extend_from_slice(&chunk);
    }

    Ok(buffer)
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

pub(crate) fn detect_decoded_kind(
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

fn to_u64_value(value: Option<&Value>) -> Option<u64> {
    if let Some(v) = value {
        if let Some(num) = v.as_u64() {
            return Some(num);
        }
        if let Some(text) = v.as_str() {
            return text.parse::<u64>().ok();
        }
    }
    None
}

fn infer_kind_and_format_from_remote_path(source: &str) -> (Option<String>, Option<String>) {
    let Ok(parsed) = url::Url::parse(source) else {
        return (None, None);
    };

    let extension = std::path::Path::new(parsed.path())
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase());

    let kind = match preview_kind_from_url_path(&parsed) {
        PreviewKind::Image => Some("image".to_string()),
        PreviewKind::Audio => Some("audio".to_string()),
        PreviewKind::Video => Some("video".to_string()),
        _ => None,
    };

    (kind, extension)
}

async fn inspect_remote_image_bytes(
    source: &str,
) -> Result<(Option<String>, u64, u32, u32), String> {
    use std::time::Duration;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Dance/media-inspector")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut response = client
        .get(source)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    if let Some(content_length) = response.content_length() {
        if content_length > REMOTE_IMAGE_METADATA_MAX_BYTES as u64 {
            return Err(format!(
                "Remote image is too large to inspect: {} bytes",
                content_length
            ));
        }
    }

    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(normalize_mime);
    let bytes = read_response_bytes_limited(&mut response, REMOTE_IMAGE_METADATA_MAX_BYTES).await?;

    let image =
        image::load_from_memory(&bytes).map_err(|e| format!("Failed to decode image: {}", e))?;

    Ok((mime, bytes.len() as u64, image.width(), image.height()))
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

pub async fn inspect_media_source_internal(
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

    if inspection.source_kind == "remote" {
        let (path_kind, path_format) = infer_kind_and_format_from_remote_path(source);
        if inspection.kind.is_none() {
            inspection.kind = path_kind;
        }
        if inspection.format.is_none() {
            inspection.format = path_format;
        }
    }

    match extract_media_metadata(source.to_string()).await {
        Ok(metadata) => {
            inspection.ffprobe_used = true;
            if inspection.format.is_none() {
                inspection.format = metadata
                    .get("format")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
            }
            if inspection.size_bytes.is_none() {
                inspection.size_bytes = to_u64_value(metadata.get("size_bytes"));
            }
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

    if inspection.kind.as_deref() == Some("image")
        && (inspection.width.is_none() || inspection.height.is_none())
        && inspection.source_kind == "remote"
    {
        match inspect_remote_image_bytes(source).await {
            Ok((mime, size_bytes, width, height)) => {
                if inspection.mime.is_none() {
                    inspection.mime = mime;
                }
                if inspection.size_bytes.is_none() {
                    inspection.size_bytes = Some(size_bytes);
                }
                inspection.width = Some(width);
                inspection.height = Some(height);
                inspection.error = None;
            }
            Err(err) => {
                if inspection.error.is_none() {
                    inspection.error = Some(err);
                }
            }
        }
    }

    inspection
}

pub async fn resolve_direct_url_preview(url: &str) -> Result<UrlPreviewResolution, String> {
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

    let response = client
        .get(parsed_url.clone())
        .send()
        .await
        .map_err(|err| format!("Network request failed: {}", err))?;
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
        .map(preview_kind_from_url_content_type)
        .filter(|kind| *kind != PreviewKind::UrlCard)
        .unwrap_or(fallback_kind);

    let mut resolution = UrlPreviewResolution {
        final_url: final_url.to_string(),
        status: Some(status.as_u16()),
        content_type: content_type.clone(),
        content_length,
        title: None,
        description: None,
        preview_kind: preview_kind.clone(),
        resolved: ResolvedPreviewData {
            source_kind: "remote".to_string(),
            mime: content_type.clone(),
            size_bytes: content_length,
            extension: content_type
                .as_deref()
                .and_then(extension_from_mime)
                .or_else(|| {
                    std::path::Path::new(final_url.path())
                        .extension()
                        .and_then(|v| v.to_str())
                        .map(|v| v.to_lowercase())
                }),
            file_name: final_url
                .path_segments()
                .and_then(|mut seg| seg.next_back())
                .filter(|name| !name.is_empty())
                .map(ToString::to_string),
            ..Default::default()
        },
        error: None,
    };

    if !status.is_success() {
        resolution.error = Some(format!("HTTP error: {}", status));
        return Ok(resolution);
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
        _ => {}
    }

    Ok(resolution)
}

pub async fn check_ffprobe_available() -> Result<bool, String> {
    use std::process::Command;

    log::debug!("[check_ffprobe_available] 检查 ffprobe 是否可用");

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

pub async fn extract_media_metadata(url: String) -> Result<serde_json::Value, String> {
    use std::process::Command;

    log::info!("[extract_media_metadata] 提取媒体元数据: {}", url);

    if !check_ffprobe_available().await? {
        return Err("FFprobe not available".to_string());
    }

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
    let metadata: serde_json::Value = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse ffprobe JSON output: {}", e))?;

    Ok(extract_basic_media_metadata(&metadata))
}

pub fn extract_basic_media_metadata(metadata: &serde_json::Value) -> serde_json::Value {
    let mut result = serde_json::Map::new();

    if let Some(format) = metadata.get("format") {
        if let Some(format_name) = format.get("format_name").and_then(Value::as_str) {
            let normalized_format = format_name
                .split(',')
                .find(|value| !value.trim().is_empty())
                .unwrap_or(format_name)
                .trim();
            if !normalized_format.is_empty() {
                result.insert(
                    "format".to_string(),
                    serde_json::Value::String(normalized_format.to_string()),
                );
            }
        }
        if let Some(size_bytes) = to_u64_value(format.get("size")) {
            result.insert(
                "size_bytes".to_string(),
                serde_json::Value::Number(serde_json::Number::from(size_bytes)),
            );
        }
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
    serde_json::Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_basic_video_metadata_from_ffprobe_json() {
        let metadata = json!({
            "format": {
                "duration": "125.4",
                "bit_rate": "3200000",
                "size": "5242880",
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2"
            },
            "streams": [
                {
                    "codec_type": "video",
                    "width": 1920,
                    "height": 1080,
                    "codec_name": "h264",
                    "r_frame_rate": "30000/1001"
                },
                {
                    "codec_type": "audio",
                    "sample_rate": "48000",
                    "codec_name": "aac"
                }
            ]
        });

        let result = extract_basic_media_metadata(&metadata);

        assert_eq!(result.get("width").and_then(Value::as_u64), Some(1920));
        assert_eq!(result.get("height").and_then(Value::as_u64), Some(1080));
        assert_eq!(result.get("duration").and_then(Value::as_str), Some("2:05"));
        assert_eq!(
            result.get("bitrate").and_then(Value::as_str),
            Some("3200 kbps")
        );
        assert_eq!(result.get("codec").and_then(Value::as_str), Some("h264"));
        assert_eq!(result.get("fps").and_then(Value::as_str), Some("29.97"));
        assert_eq!(
            result.get("sample_rate").and_then(Value::as_str),
            Some("48000")
        );
        assert_eq!(
            result.get("size_bytes").and_then(Value::as_u64),
            Some(5242880)
        );
        assert_eq!(result.get("format").and_then(Value::as_str), Some("mov"));
    }

    #[tokio::test]
    async fn inspects_data_url_image_size_and_dimensions_without_ffprobe() {
        use base64::Engine as _;

        let image = image::RgbaImage::from_pixel(2, 3, image::Rgba([255, 0, 0, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode png");
        let data_url = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&bytes)
        );

        let inspection = inspect_media_source_internal(&data_url, None, None).await;

        assert_eq!(inspection.kind.as_deref(), Some("image"));
        assert_eq!(inspection.mime.as_deref(), Some("image/png"));
        assert_eq!(inspection.format.as_deref(), Some("png"));
        assert_eq!(inspection.size_bytes, Some(bytes.len() as u64));
        assert_eq!(inspection.width, Some(2));
        assert_eq!(inspection.height, Some(3));
        assert!(!inspection.ffprobe_used);
    }
}
