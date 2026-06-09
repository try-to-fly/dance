use std::borrow::Cow;

use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentSubType {
    PlainText,
    Url,
    IpAddress,
    Email,
    Color,
    Code,
    Command,
    Timestamp,
    Json,
    Markdown,
    Base64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub detected_language: Option<String>,
    pub url_parts: Option<UrlParts>,
    pub color_formats: Option<ColorFormats>,
    pub timestamp_formats: Option<TimestampFormats>,
    pub base64_metadata: Option<Base64Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlParts {
    pub protocol: String,
    pub host: String,
    pub path: String,
    pub query_params: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorFormats {
    pub hex: Option<String>,
    pub rgb: Option<String>,
    pub rgba: Option<String>,
    pub hsl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampFormats {
    pub unix_ms: Option<i64>,
    pub iso8601: Option<String>,
    pub date_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64Metadata {
    pub estimated_original_size: usize,
    pub encoded_size: usize,
    pub content_hint: Option<String>,
    pub encoding_efficiency: f32,
}

pub struct ContentDetector;

impl ContentDetector {
    const DOMINANT_URL_RATIO_THRESHOLD: f32 = 0.8;

    pub fn detect(text: &str) -> (ContentSubType, Option<ContentMetadata>) {
        let normalized = Self::normalize_clipboard_text(text);
        let trimmed = normalized.as_ref();
        log::debug!(
            "[ContentDetector] 开始检测内容类型，长度: {}字符",
            trimmed.len()
        );
        log::trace!(
            "[ContentDetector] 内容前100字符: {}",
            if trimmed.len() > 100 {
                &trimmed[..100]
            } else {
                trimmed
            }
        );

        // URL检测
        if Self::is_url(trimmed) {
            log::debug!("[ContentDetector] 检测到URL类型");
            let metadata = Self::parse_url_metadata(trimmed);
            return (ContentSubType::Url, Some(metadata));
        }

        // IP地址检测
        if Self::is_ip_address(trimmed) {
            log::debug!("[ContentDetector] 检测到IP地址类型");
            return (ContentSubType::IpAddress, None);
        }

        // 邮箱检测
        if Self::is_email(trimmed) {
            log::debug!("[ContentDetector] 检测到邮箱地址类型");
            return (ContentSubType::Email, None);
        }

        // 颜色检测
        if let Some(color_formats) = Self::detect_color(trimmed) {
            log::debug!("[ContentDetector] 检测到颜色类型: {:?}", color_formats);
            let metadata = ContentMetadata {
                detected_language: None,
                url_parts: None,
                color_formats: Some(color_formats),
                timestamp_formats: None,
                base64_metadata: None,
            };
            return (ContentSubType::Color, Some(metadata));
        }

        // JSON检测
        if Self::is_json(trimmed) {
            log::debug!("[ContentDetector] 检测到JSON类型");
            return (ContentSubType::Json, None);
        }

        // 命令行检测
        if Self::is_command(trimmed) {
            log::debug!("[ContentDetector] 检测到命令行类型");
            return (ContentSubType::Command, None);
        }

        // 时间戳检测
        if let Some(timestamp_formats) = Self::detect_timestamp(trimmed) {
            log::debug!(
                "[ContentDetector] 检测到时间戳类型: {:?}",
                timestamp_formats
            );
            let metadata = ContentMetadata {
                detected_language: None,
                url_parts: None,
                color_formats: None,
                timestamp_formats: Some(timestamp_formats),
                base64_metadata: None,
            };
            return (ContentSubType::Timestamp, Some(metadata));
        }

        // Markdown检测
        if Self::is_markdown(trimmed) {
            log::debug!("[ContentDetector] 检测到Markdown类型");
            return (ContentSubType::Markdown, None);
        }

        // Base64检测
        if let Some(base64_metadata) = Self::detect_base64(trimmed) {
            log::debug!(
                "[ContentDetector] 检测到Base64类型: {} -> {}, 内容: {:?}",
                base64_metadata.encoded_size,
                base64_metadata.estimated_original_size,
                base64_metadata.content_hint
            );
            let metadata = ContentMetadata {
                detected_language: None,
                url_parts: None,
                color_formats: None,
                timestamp_formats: None,
                base64_metadata: Some(base64_metadata),
            };
            return (ContentSubType::Base64, Some(metadata));
        }

        // 代码检测
        if let Some(language) = Self::detect_code_language(trimmed) {
            log::debug!("[ContentDetector] 检测到代码类型，语言: {}", language);
            let metadata = ContentMetadata {
                detected_language: Some(language),
                url_parts: None,
                color_formats: None,
                timestamp_formats: None,
                base64_metadata: None,
            };
            return (ContentSubType::Code, Some(metadata));
        }

        // 默认为纯文本
        log::debug!("[ContentDetector] 未匹配到特定类型，归类为纯文本");
        (ContentSubType::PlainText, None)
    }

    // 只在 URL 几乎占满整个 token，且移除内容仅限首尾噪声标点/包裹符时才做归一化。
    pub fn normalize_clipboard_text<'a>(text: &'a str) -> Cow<'a, str> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Cow::Borrowed(trimmed);
        }

        if let Some(candidate) = Self::extract_dominant_url_candidate(trimmed) {
            return Cow::Borrowed(candidate);
        }

        Cow::Borrowed(trimmed)
    }

    fn is_url(text: &str) -> bool {
        if text.is_empty() || text.chars().any(char::is_whitespace) {
            return false;
        }

        // 优先识别带协议的完整URL，避免把任意包含冒号的文本误判成链接
        if let Ok(parsed) = url::Url::parse(text) {
            return matches!(parsed.scheme(), "http" | "https" | "ftp")
                && parsed.host_str().is_some();
        }

        if text.contains('@') {
            return false; // 可能是邮箱，不是URL
        }

        Self::normalize_bare_http_url(text).is_some()
    }

    fn extract_dominant_url_candidate(text: &str) -> Option<&str> {
        if text.is_empty() || text.chars().any(char::is_whitespace) {
            return None;
        }

        let sanitized = Self::sanitize_url_candidate(text);
        if sanitized != text
            && Self::url_candidate_ratio(sanitized, text) >= Self::DOMINANT_URL_RATIO_THRESHOLD
            && Self::is_url(sanitized)
        {
            return Some(sanitized);
        }

        if Self::is_url(text) {
            return Some(text);
        }

        None
    }

    fn sanitize_url_candidate(mut text: &str) -> &str {
        loop {
            if let Some(unwrapped) = Self::strip_matching_url_wrapper(text) {
                text = unwrapped;
                continue;
            }

            let without_leading_noise = Self::strip_leading_url_noise(text);
            if without_leading_noise != text {
                text = without_leading_noise;
                continue;
            }

            let without_trailing_noise = Self::strip_trailing_url_noise(text);
            if without_trailing_noise != text {
                text = without_trailing_noise;
                continue;
            }

            break;
        }

        text
    }

    fn strip_matching_url_wrapper(text: &str) -> Option<&str> {
        let first = text.chars().next()?;
        let last = text.chars().next_back()?;
        let expected_closer = match first {
            '"' => '"',
            '\'' => '\'',
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '<' => '>',
            '“' => '”',
            '‘' => '’',
            '（' => '）',
            '【' => '】',
            '《' => '》',
            '「' => '」',
            '『' => '』',
            _ => return None,
        };

        if last != expected_closer || text.chars().count() < 3 {
            return None;
        }

        Some(&text[first.len_utf8()..text.len() - last.len_utf8()])
    }

    fn strip_leading_url_noise(mut text: &str) -> &str {
        while let Some(first) = text.chars().next() {
            if !Self::is_leading_url_noise(first) {
                break;
            }
            text = &text[first.len_utf8()..];
        }

        text
    }

    fn strip_trailing_url_noise(mut text: &str) -> &str {
        while let Some(last) = text.chars().next_back() {
            if !Self::is_trailing_url_noise(last) {
                break;
            }
            text = &text[..text.len() - last.len_utf8()];
        }

        text
    }

    fn is_leading_url_noise(ch: char) -> bool {
        matches!(
            ch,
            '"' | '\''
                | '`'
                | ','
                | ';'
                | ':'
                | '，'
                | '；'
                | '：'
                | '('
                | '['
                | '{'
                | '<'
                | '“'
                | '‘'
                | '（'
                | '【'
                | '《'
                | '「'
                | '『'
        )
    }

    fn is_trailing_url_noise(ch: char) -> bool {
        matches!(
            ch,
            '"' | '\'' | '`' | ',' | ';' | ':' | '，' | '；' | '：' | '”' | '’'
        )
    }

    fn url_candidate_ratio(candidate: &str, original: &str) -> f32 {
        let original_chars = original.chars().count();
        if original_chars == 0 {
            return 0.0;
        }

        candidate.chars().count() as f32 / original_chars as f32
    }

    fn parse_url_metadata(url: &str) -> ContentMetadata {
        let mut metadata = ContentMetadata {
            detected_language: None,
            url_parts: None,
            color_formats: None,
            timestamp_formats: None,
            base64_metadata: None,
        };

        let parsed_url = url::Url::parse(url).ok().or_else(|| {
            Self::normalize_bare_http_url(url).and_then(|value| url::Url::parse(&value).ok())
        });

        if let Some(parsed) = parsed_url {
            let query_params: Vec<(String, String)> = parsed
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            log::trace!(
                "[ContentDetector] URL解析成功: {} -> {}://{}{}",
                url,
                parsed.scheme(),
                parsed.host_str().unwrap_or(""),
                parsed.path()
            );

            let host_with_port = if let Some(port) = parsed.port() {
                format!("{}:{}", parsed.host_str().unwrap_or(""), port)
            } else {
                parsed.host_str().unwrap_or("").to_string()
            };

            metadata.url_parts = Some(UrlParts {
                protocol: parsed.scheme().to_string(),
                host: host_with_port,
                path: parsed.path().to_string(),
                query_params,
            });
        } else {
            log::trace!("[ContentDetector] URL解析失败: {}", url);
        }

        metadata
    }

    fn normalize_bare_http_url(text: &str) -> Option<String> {
        let (host, has_extra_parts) = Self::extract_host_candidate(text)?;
        let domain_regex =
            Regex::new(r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,24}$")
                .unwrap();

        if !domain_regex.is_match(host) {
            return None;
        }

        let labels: Vec<&str> = host.split('.').collect();
        if labels.len() < 2 {
            return None;
        }

        let tld = labels.last()?.to_ascii_lowercase();
        if Self::is_known_file_extension(&tld) {
            return None;
        }

        if !has_extra_parts
            && !host.starts_with("www.")
            && labels.len() == 2
            && !Self::is_common_bare_domain_tld(&tld)
        {
            return None;
        }

        Some(format!("https://{}", text))
    }

    fn extract_host_candidate(text: &str) -> Option<(&str, bool)> {
        let mut boundary = text.len();
        for separator in ['/', '?', '#'] {
            if let Some(index) = text.find(separator) {
                boundary = boundary.min(index);
            }
        }

        let host_and_port = &text[..boundary];
        if host_and_port.is_empty() {
            return None;
        }

        let (host, has_port) = if let Some((host, port)) = host_and_port.rsplit_once(':') {
            if !host.is_empty() && !port.is_empty() && port.chars().all(|ch| ch.is_ascii_digit()) {
                (host, true)
            } else {
                (host_and_port, false)
            }
        } else {
            (host_and_port, false)
        };

        if host.is_empty() {
            return None;
        }

        Some((host, has_port || boundary < text.len()))
    }

    fn is_common_bare_domain_tld(tld: &str) -> bool {
        matches!(
            tld,
            "ai" | "app"
                | "au"
                | "biz"
                | "ca"
                | "cc"
                | "ch"
                | "cn"
                | "co"
                | "com"
                | "de"
                | "dev"
                | "es"
                | "fr"
                | "info"
                | "in"
                | "io"
                | "it"
                | "jp"
                | "kr"
                | "me"
                | "net"
                | "nl"
                | "no"
                | "online"
                | "org"
                | "ru"
                | "se"
                | "sh"
                | "site"
                | "store"
                | "tech"
                | "tv"
                | "uk"
                | "us"
                | "xyz"
        )
    }

    fn is_known_file_extension(ext: &str) -> bool {
        matches!(
            ext,
            "7z" | "aac"
                | "avi"
                | "bat"
                | "bmp"
                | "bz2"
                | "c"
                | "conf"
                | "cpp"
                | "css"
                | "csv"
                | "doc"
                | "docx"
                | "flac"
                | "flv"
                | "gif"
                | "go"
                | "gz"
                | "h"
                | "hpp"
                | "htm"
                | "html"
                | "ico"
                | "ini"
                | "java"
                | "jpeg"
                | "jpg"
                | "js"
                | "json"
                | "jsx"
                | "log"
                | "m4a"
                | "md"
                | "mkv"
                | "mov"
                | "mp3"
                | "mp4"
                | "ogg"
                | "pdf"
                | "php"
                | "png"
                | "ppt"
                | "pptx"
                | "py"
                | "rar"
                | "rb"
                | "rs"
                | "sh"
                | "sql"
                | "svg"
                | "tar"
                | "tif"
                | "tiff"
                | "toml"
                | "ts"
                | "tsx"
                | "txt"
                | "wav"
                | "webm"
                | "webp"
                | "xls"
                | "xlsx"
                | "xml"
                | "xz"
                | "yaml"
                | "yml"
                | "zip"
        )
    }

    fn is_ip_address(text: &str) -> bool {
        // IPv4
        let ipv4_regex = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
        if ipv4_regex.is_match(text) {
            return true;
        }

        // IPv6
        let ipv6_regex = Regex::new(r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$").unwrap();
        ipv6_regex.is_match(text)
    }

    fn is_email(text: &str) -> bool {
        let email_regex = Regex::new(r"^[\w._%+-]+@[\w.-]+\.[\w]{2,}$").unwrap();
        email_regex.is_match(text)
    }

    fn detect_color(text: &str) -> Option<ColorFormats> {
        let mut formats = ColorFormats {
            hex: None,
            rgb: None,
            rgba: None,
            hsl: None,
        };

        // HEX颜色 - 支持 #333, #ffffff 等格式
        if text.starts_with('#') && text.len() >= 4 {
            let hex_part = &text[1..];
            if (hex_part.len() == 3 || hex_part.len() == 6)
                && hex_part.chars().all(|c| c.is_ascii_hexdigit())
            {
                formats.hex = Some(text.to_string());
                return Some(formats);
            }
        }

        // RGB/RGBA颜色
        let rgb_regex = Regex::new(
            r"^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(?:,\s*([\d.]+))?\s*\)$",
        )
        .unwrap();
        if let Some(captures) = rgb_regex.captures(text) {
            // 验证RGB值是否在有效范围内 (0-255)
            let _r: u8 = captures[1].parse().ok()?;
            let _g: u8 = captures[2].parse().ok()?;
            let _b: u8 = captures[3].parse().ok()?;

            // 如果有alpha通道，验证它在0.0-1.0范围内
            if let Some(alpha_str) = captures.get(4) {
                let alpha: f32 = alpha_str.as_str().parse().ok()?;
                if !(0.0..=1.0).contains(&alpha) {
                    return None;
                }
                formats.rgba = Some(text.to_string());
            } else {
                formats.rgb = Some(text.to_string());
            }
            return Some(formats);
        }

        // HSL颜色
        let hsl_regex =
            Regex::new(r"^hsl\(\s*(\d{1,3})\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*\)$").unwrap();
        if hsl_regex.is_match(text) {
            formats.hsl = Some(text.to_string());
            return Some(formats);
        }

        None
    }

    fn is_json(text: &str) -> bool {
        let trimmed = text.trim();

        // 检查是否以 { 或 [ 开头并以相应字符结尾
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            // 尝试解析JSON
            return serde_json::from_str::<Value>(trimmed).is_ok();
        }

        false
    }

    fn is_command(text: &str) -> bool {
        let commands = [
            "git ", "npm ", "yarn ", "pnpm ", "docker ", "kubectl ", "cargo ", "python ", "pip ",
            "brew ", "apt ", "yum ", "ls", "cd ", "mkdir ", "rm ", "cp ", "mv ", "cat ", "grep ",
            "sed ", "awk ", "curl ", "wget ", "ssh ",
        ];

        commands.iter().any(|cmd| text.starts_with(cmd))
    }

    fn detect_timestamp(text: &str) -> Option<TimestampFormats> {
        let mut formats = TimestampFormats {
            unix_ms: None,
            iso8601: None,
            date_string: None,
        };

        // Unix时间戳（秒或毫秒）
        if let Ok(num) = text.parse::<i64>() {
            // 检查是否在合理的时间戳范围内
            if (946684800..4102444800).contains(&num) {
                // 秒级时间戳（2000-2100年）
                formats.unix_ms = Some(num * 1000);
                return Some(formats);
            } else if (946684800000..7258118400000).contains(&num) {
                // 毫秒级时间戳（2000-2200年）
                formats.unix_ms = Some(num);
                return Some(formats);
            }
        }

        // ISO 8601格式
        let iso_regex =
            Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?$")
                .unwrap();
        if iso_regex.is_match(text) {
            formats.iso8601 = Some(text.to_string());
            return Some(formats);
        }

        // 常见日期格式
        let date_regex =
            Regex::new(r"^\d{4}[-/]\d{2}[-/]\d{2}(?:\s+\d{2}:\d{2}(?::\d{2})?)?$").unwrap();
        if date_regex.is_match(text) {
            formats.date_string = Some(text.to_string());
            return Some(formats);
        }

        None
    }

    fn is_markdown(text: &str) -> bool {
        let patterns = [
            r"^#{1,6}\s+",    // 标题
            r"\*\*[^*]+\*\*", // 粗体
            r"\*[^*]+\*",     // 斜体
            r"\[.+\]\(.+\)",  // 链接
            r"!\[.*\]\(.+\)", // 图片
            r"^[-*+]\s+",     // 列表
            r"^\d+\.\s+",     // 有序列表
            r"^>\s+",         // 引用
            r"```",           // 代码块
            r"`[^`]+`",       // 行内代码
        ];

        patterns
            .iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(text))
    }

    fn detect_code_language(text: &str) -> Option<String> {
        // 简单的代码语言检测
        let patterns = vec![
            // More specific patterns first to avoid conflicts
            // Rust has very specific keywords
            (
                r"\b(?:fn|impl|struct|enum|match|trait|pub|use|mut)\b",
                "rust",
            ),
            // Java has specific combinations
            (
                r"\b(?:public|private|protected)\s+(?:class|static|final)|static void main|import java|\bfinal\s+\w+|\w+\s*\[\s*\]",
                "java",
            ),
            // Python specific patterns
            (r"\b(?:def |import |from |if __name__|print\()", "python"),
            // JavaScript patterns
            (
                r"\b(?:function|const|let|var|async|await|console\.log)\b|=>",
                "javascript",
            ),
            // C patterns
            (r"#include|int main|\bvoid\b|\bprintf\b|\bscanf\b", "c"),
            // SQL patterns
            (
                r"\b(?:SELECT|FROM|WHERE|INSERT|UPDATE|DELETE|CREATE TABLE)\b",
                "sql",
            ),
            // HTML patterns
            (r"(?:<html|<div|<span|<body|<head|<script|<style)", "html"),
            // CSS patterns
            (
                r"(?:^\.[\w-]+\s*\{|^#[\w-]+\s*\{|color:|background:|margin:|padding:)",
                "css",
            ),
        ];

        for (pattern, language) in patterns {
            if Regex::new(pattern).unwrap().is_match(text) {
                log::trace!(
                    "[ContentDetector] 代码语言匹配: {} -> {}",
                    pattern,
                    language
                );
                return Some(language.to_string());
            }
        }

        log::trace!("[ContentDetector] 未检测到已知代码语言");
        None
    }

    fn detect_base64(text: &str) -> Option<Base64Metadata> {
        let data_url_payload = Self::extract_base64_data_url_payload(text);
        let explicit_data_url = data_url_payload.is_some();
        let candidate = data_url_payload.unwrap_or(text);

        // 最小长度检查 - 对于短字符串需要更严格的验证
        if candidate.len() < 4 {
            return None;
        }

        // 对于较短的字符串（4-40字符），需要更严格的base64格式检查
        let is_short = candidate.len() <= 40;

        // 排除明显的URL；显式 data:*;base64,... 由上面的 payload 分支处理
        if !explicit_data_url && (text.starts_with("http://") || text.starts_with("https://")) {
            return None;
        }

        // 检查是否主要由base64字符组成
        let base64_chars = candidate
            .chars()
            .filter(|c| Self::is_standard_base64_char(*c))
            .count();

        let total_chars = candidate.chars().count();
        let base64_ratio = base64_chars as f32 / total_chars as f32;

        // Base64字符占比需要足够高
        let required_ratio = if is_short { 1.0 } else { 0.95 };
        if base64_ratio < required_ratio {
            return None;
        }

        // 对于短字符串，额外检查：不应该是常见的英文单词或简单文本
        if is_short {
            // 排除常见的英文单词和简单模式
            let lowercase_text = candidate.to_lowercase();
            let common_words = [
                "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
                "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new",
                "now", "old", "see", "two", "who", "boy", "did", "man", "car", "run", "way", "use",
                "yes", "too", "big", "end", "far", "off", "own", "say", "she", "try", "ask", "job",
                "let", "put", "sit", "top", "win", "cut", "lot", "eat", "god", "hit", "lot", "son",
                "got", "red", "hot", "air", "bit", "box", "buy", "eye", "few", "fix", "key", "lay",
                "leg", "low", "map", "mix", "oil", "pay", "pop", "raw", "row", "sad", "sea", "set",
                "six", "sky", "tax", "tea", "ten", "tie", "tip", "war", "wet", "add", "bad", "bag",
                "bar", "bat", "bed", "bid", "bus", "cat", "cop", "cup", "die", "dig", "dog", "dot",
                "dry", "ear", "egg", "fan", "fly", "fun", "gap", "gas", "gun", "hat", "ice", "kid",
                "lab", "lap", "lie", "lip", "log", "mad", "mom", "mud", "net", "pan", "pen", "pet",
                "pie", "pin", "pot", "rat", "red", "rid", "rip", "rob", "rod", "run", "sad", "sit",
                "sun", "tap", "toy", "van", "web", "win", "zip",
            ];
            if common_words.contains(&lowercase_text.as_str()) {
                return None;
            }

            // 排除看起来像普通词汇的纯字母模式，避免把 ANALYSIS 之类的单词误报成 base64
            let is_alpha_only = candidate.chars().all(|c| c.is_ascii_alphabetic());
            let is_title_case = {
                let mut chars = candidate.chars();
                matches!(chars.next(), Some(first) if first.is_ascii_uppercase())
                    && chars.all(|c| c.is_ascii_lowercase())
            };
            if is_alpha_only
                && candidate.len() >= 3
                && (candidate.chars().all(|c| c.is_ascii_lowercase())
                    || candidate.chars().all(|c| c.is_ascii_uppercase())
                    || is_title_case)
            {
                return None;
            }

            // 排除简单的重复模式
            if candidate.len() <= 8
                && candidate
                    .chars()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    <= 2
            {
                return None;
            }
        }

        // 检查是否是长字符串中的重复模式（如10000个"A"）
        if candidate.len() > 100 {
            let unique_chars: std::collections::HashSet<char> = candidate.chars().collect();
            if unique_chars.len() <= 3 {
                // 只有3个或更少的不同字符，可能是重复模式
                return None;
            }
        }

        // 检查换行符数量，排除格式化的代码或文档
        let newlines = candidate.chars().filter(|c| *c == '\n').count();
        if newlines > candidate.len() / 50 {
            // 如果换行符过多，可能是格式化文本
            return None;
        }

        // 清理空白字符后再验证
        let cleaned: String = candidate.chars().filter(|c| !c.is_whitespace()).collect();
        if cleaned.len() < 4 || cleaned.chars().any(|c| !Self::is_standard_base64_char(c)) {
            return None;
        }

        if !Self::has_valid_base64_padding(&cleaned) {
            return None;
        }

        // 验证base64格式：去除padding后，长度应该符合base64规则
        let without_padding: String = cleaned.trim_end_matches('=').to_string();
        // Base64编码后，去除padding的长度 mod 4 应该是 0, 2, 或 3 (对应0, 2, 1个padding字符)
        // 如果是1，说明格式不正确
        let remainder = without_padding.len() % 4;
        if remainder == 1 {
            // 这种情况不符合base64规则
            return None;
        }

        // 尝试解码
        match general_purpose::STANDARD
            .decode(&cleaned)
            .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&cleaned))
        {
            Ok(decoded) => {
                let encoded_size = cleaned.len();
                let decoded_size = decoded.len();

                // 计算编码效率：base64编码后长度应该是原始长度的4/3倍，然后向上对齐到4的倍数
                let expected_encoded_size = ((decoded_size * 4).div_ceil(3) + 3) & !3; // 向上对齐到4的倍数
                let size_ratio = encoded_size as f32 / expected_encoded_size as f32;

                // 对于正确的base64，实际大小应该等于或接近期望大小
                // 放宽一点限制，特别是对于短字符串
                let tolerance = if decoded_size <= 10 { 0.5 } else { 0.2 };
                if size_ratio < (1.0 - tolerance) || size_ratio > (1.0 + tolerance) {
                    return None;
                }

                // 分析解码后的内容特征
                let content_hint = Self::analyze_decoded_content(&decoded);
                let has_padding = cleaned.ends_with('=');
                let has_content_signal = content_hint.is_some();
                if !explicit_data_url && !has_padding && !has_content_signal {
                    return None;
                }

                log::debug!(
                    "[ContentDetector] Base64检测成功: {}字节 -> {}字节, 效率: {:.2}",
                    encoded_size,
                    decoded_size,
                    size_ratio
                );

                Some(Base64Metadata {
                    estimated_original_size: decoded_size,
                    encoded_size,
                    content_hint,
                    encoding_efficiency: size_ratio,
                })
            }
            Err(_) => None,
        }
    }

    fn extract_base64_data_url_payload(text: &str) -> Option<&str> {
        let trimmed = text.trim();
        if !trimmed
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return None;
        }

        let comma_index = trimmed.find(',')?;
        let header = &trimmed[..comma_index];
        if !header.to_ascii_lowercase().contains(";base64") {
            return None;
        }

        let payload = &trimmed[comma_index + 1..];
        if payload.trim().is_empty() {
            None
        } else {
            Some(payload)
        }
    }

    fn is_standard_base64_char(char: char) -> bool {
        char.is_ascii_alphanumeric() || char == '+' || char == '/' || char == '='
    }

    fn has_valid_base64_padding(cleaned: &str) -> bool {
        if let Some(first_padding_index) = cleaned.find('=') {
            let padding = &cleaned[first_padding_index..];
            if padding.len() > 2 || padding.chars().any(|char| char != '=') {
                return false;
            }
        }

        cleaned.trim_end_matches('=').len() % 4 != 1
    }

    fn analyze_decoded_content(data: &[u8]) -> Option<String> {
        // 检查是否是常见的二进制格式
        if data.len() >= 4 {
            // PNG文件签名
            if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Some("PNG图片".to_string());
            }
            // JPEG文件签名
            if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Some("JPEG图片".to_string());
            }
            // PDF文件签名
            if data.starts_with(b"%PDF") {
                return Some("PDF文档".to_string());
            }
            // GIF文件签名
            if data.starts_with(b"GIF8") {
                return Some("GIF图片".to_string());
            }
            // ZIP文件签名
            if data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
                || data.starts_with(&[0x50, 0x4B, 0x05, 0x06])
            {
                return Some("ZIP压缩包".to_string());
            }
        }

        // 检查是否是文本内容
        if let Ok(text) = std::str::from_utf8(data) {
            if text
                .chars()
                .all(|c| c.is_ascii() && (!c.is_control() || c.is_whitespace()))
                && text.len() > 10
                && Self::decoded_text_has_content_signal(text)
            {
                return Some("文本内容".to_string());
            }
        }

        // 分析字节分布
        if data.len() > 100 {
            let zero_bytes = data.iter().filter(|&&b| b == 0).count();
            let zero_ratio = zero_bytes as f32 / data.len() as f32;

            if zero_ratio > 0.1 {
                return Some("二进制数据".to_string());
            }
        }

        None
    }

    fn decoded_text_has_content_signal(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }

        if serde_json::from_str::<Value>(trimmed).is_ok() {
            return true;
        }

        trimmed.chars().any(char::is_whitespace)
            || trimmed.chars().any(|char| {
                matches!(
                    char,
                    '{' | '}'
                        | '['
                        | ']'
                        | '('
                        | ')'
                        | ':'
                        | ';'
                        | ','
                        | '.'
                        | '\''
                        | '"'
                        | '<'
                        | '>'
                        | '='
                        | '+'
                        | '-'
                        | '*'
                        | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '|'
                        | '\\'
                        | '`'
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic text detection tests
    #[test]
    fn test_content_detector_supported_subtypes_are_stable() {
        let cases = vec![
            ("https://example.com/path?foo=bar", ContentSubType::Url),
            ("192.168.1.10", ContentSubType::IpAddress),
            ("developer@example.com", ContentSubType::Email),
            ("#ff8800", ContentSubType::Color),
            (r#"{"hello":"world","count":2}"#, ContentSubType::Json),
            ("git status --short", ContentSubType::Command),
            ("2025-01-01T12:00:00Z", ContentSubType::Timestamp),
            ("# Header\n\n- item", ContentSubType::Markdown),
            ("Y29uc29sZS5sb2coJ2hlbGxvJyk7", ContentSubType::Base64),
            (
                "function greet(name) {\n  return name;\n}",
                ContentSubType::Code,
            ),
            ("ordinary clipboard note", ContentSubType::PlainText),
        ];

        for (input, expected_subtype) in cases {
            let (sub_type, _) = ContentDetector::detect(input);
            assert_eq!(
                sub_type, expected_subtype,
                "unexpected subtype for stable corpus input '{}'",
                input
            );
        }
    }

    #[test]
    fn test_plain_text_detection() {
        let test_cases = vec![
            "Hello, world!",
            "This is a simple text message.",
            "你好，世界！",
            "こんにちは世界",
            "مرحبا بالعالم",
            "Привет, мир!",
            "12345 mixed with text",
            "Special characters: @#$%^&*()",
            "Line breaks\nare\nhere",
            "Tabs\tand\tspaces",
        ];

        for text in test_cases {
            let (sub_type, _metadata) = ContentDetector::detect(text);
            match sub_type {
                ContentSubType::PlainText => {}
                // Some texts might be detected as other types, which is acceptable
                // The detection prioritizes more specific patterns
                _ => {
                    // For debugging, log what was detected
                    println!("Text '{}' was detected as {:?}", text, sub_type);
                }
            }
        }
    }

    #[test]
    fn test_empty_and_whitespace() {
        let test_cases = vec!["", " ", "\n", "\t", "   \n\t  "];

        for text in test_cases {
            let (sub_type, _) = ContentDetector::detect(text);
            assert!(matches!(sub_type, ContentSubType::PlainText));
        }
    }

    // URL detection tests
    #[test]
    fn test_url_detection() {
        let valid_urls = vec![
            "https://www.example.com",
            "http://example.com",
            "ftp://files.example.com",
            "https://sub.example.co.uk/path/to/resource?param=value#anchor",
            "http://localhost:3000",
            "https://192.168.1.1:8080/api",
            "github.com/user/repo",
            "www.example.com",
            "example.org",
            "test-site.example.com",
        ];

        for url in valid_urls {
            let (sub_type, metadata) = ContentDetector::detect(url);
            assert!(
                matches!(sub_type, ContentSubType::Url),
                "Failed to detect '{}' as URL",
                url
            );

            // Only check metadata for URLs that have valid protocols
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("ftp://")
            {
                if let Some(meta) = metadata {
                    assert!(
                        meta.url_parts.is_some(),
                        "URL metadata missing for '{}'",
                        url
                    );
                }
            }
        }
    }

    #[test]
    fn test_normalize_clipboard_text_strips_dominant_url_wrappers_and_noise() {
        let cases = vec![
            (
                r#"https://www.right.codes/dashboard""#,
                "https://www.right.codes/dashboard",
            ),
            (
                ";https://www.right.codes/dashboard",
                "https://www.right.codes/dashboard",
            ),
            (
                r#"("https://example.com/docs?tab=api")"#,
                "https://example.com/docs?tab=api",
            ),
            ("，github.com/user/repo；", "github.com/user/repo"),
            ("《https://example.com/path》", "https://example.com/path"),
            (" https://example.com/docs ", "https://example.com/docs"),
        ];

        for (input, expected) in cases {
            assert_eq!(
                ContentDetector::normalize_clipboard_text(input).as_ref(),
                expected
            );
        }
    }

    #[test]
    fn test_normalize_clipboard_text_preserves_non_url_inputs_and_low_ratio_cases() {
        let cases = vec![
            r#"README.md""#,
            "git clone https://example.com/repo.git",
            "ordinary clipboard note",
            "((((((https://example.com))))))",
            "https://en.wikipedia.org/wiki/Function_(mathematics))",
        ];

        for input in cases {
            assert_eq!(
                ContentDetector::normalize_clipboard_text(input).as_ref(),
                input
            );
        }
    }

    #[test]
    fn test_url_detection_uses_normalized_dominant_url_candidate() {
        let (sub_type, metadata) = ContentDetector::detect(r#"("https://example.com/docs")"#);
        assert!(matches!(sub_type, ContentSubType::Url));

        let url_parts = metadata
            .and_then(|meta| meta.url_parts)
            .expect("expected normalized url metadata");
        assert_eq!(url_parts.protocol, "https");
        assert_eq!(url_parts.host, "example.com");
        assert_eq!(url_parts.path, "/docs");
    }

    #[test]
    fn test_url_metadata_parsing() {
        let (_, metadata) = ContentDetector::detect(
            "https://example.com:8080/path/to/resource?param1=value1&param2=value2#anchor",
        );

        if let Some(meta) = metadata {
            if let Some(url_parts) = meta.url_parts {
                assert_eq!(url_parts.protocol, "https");
                assert_eq!(url_parts.host, "example.com:8080");
                assert_eq!(url_parts.path, "/path/to/resource");
                assert_eq!(url_parts.query_params.len(), 2);
                assert_eq!(
                    url_parts.query_params[0],
                    ("param1".to_string(), "value1".to_string())
                );
                assert_eq!(
                    url_parts.query_params[1],
                    ("param2".to_string(), "value2".to_string())
                );
            }
        }
    }

    #[test]
    fn test_file_names_are_not_detected_as_urls() {
        let file_names = vec![
            "code-plan.toml",
            "notes.txt",
            "README.md",
            "config.json",
            "archive.tar.gz",
        ];

        for file_name in file_names {
            let (sub_type, _) = ContentDetector::detect(file_name);
            assert!(
                matches!(sub_type, ContentSubType::PlainText),
                "File name '{}' should not be detected as URL, got {:?}",
                file_name,
                sub_type
            );
        }
    }

    // IP address detection tests
    #[test]
    fn test_ip_detection() {
        let valid_ipv4 = vec![
            "192.168.1.1",
            "127.0.0.1",
            "255.255.255.255",
            "0.0.0.0",
            "8.8.8.8",
            "10.0.0.1",
        ];

        for ip in valid_ipv4 {
            let (sub_type, _) = ContentDetector::detect(ip);
            assert!(
                matches!(sub_type, ContentSubType::IpAddress),
                "Failed to detect '{}' as IP",
                ip
            );
        }

        let valid_ipv6 = vec![
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "2001:db8:85a3::8a2e:370:7334",
            "::1",
            "fe80::1%lo0",
        ];

        for ip in valid_ipv6 {
            let (sub_type, _) = ContentDetector::detect(ip);
            assert!(
                matches!(sub_type, ContentSubType::IpAddress),
                "Failed to detect '{}' as IPv6",
                ip
            );
        }

        let invalid_ips = vec![
            "256.256.256.256",
            "192.168.1",
            "192.168.1.1.1",
            "not.an.ip.address",
        ];

        for ip in invalid_ips {
            let (sub_type, _) = ContentDetector::detect(ip);
            assert!(
                !matches!(sub_type, ContentSubType::IpAddress),
                "Incorrectly detected '{}' as IP",
                ip
            );
        }
    }

    // Email detection tests
    #[test]
    fn test_email_detection() {
        let valid_emails = vec![
            "user@example.com",
            "test.email@domain.co.uk",
            "user+tag@example.org",
            "firstname.lastname@company.com",
            "user123@test-domain.com",
        ];

        for email in valid_emails {
            let (sub_type, _) = ContentDetector::detect(email);
            assert!(
                matches!(sub_type, ContentSubType::Email),
                "Failed to detect '{}' as email",
                email
            );
        }

        let invalid_emails = vec![
            "@example.com",
            "user@",
            "user@@example.com",
            "user@.com",
            "user space@example.com",
        ];

        for email in invalid_emails {
            let (sub_type, _) = ContentDetector::detect(email);
            assert!(
                !matches!(sub_type, ContentSubType::Email),
                "Incorrectly detected '{}' as email",
                email
            );
        }
    }

    // Color detection tests
    #[test]
    fn test_color_detection() {
        let hex_colors = vec![
            "#fff", "#000", "#ff0000", "#00FF00", "#0000ff", "#123abc", "#ABCDEF",
        ];

        for color in hex_colors {
            let (sub_type, metadata) = ContentDetector::detect(color);
            assert!(
                matches!(sub_type, ContentSubType::Color),
                "Failed to detect '{}' as color",
                color
            );

            if let Some(meta) = metadata {
                if let Some(color_formats) = meta.color_formats {
                    assert!(color_formats.hex.is_some());
                }
            }
        }

        let rgb_colors = vec![
            "rgb(255, 0, 0)",
            "rgb(0, 255, 0)",
            "rgb(0, 0, 255)",
            "rgba(255, 255, 255, 0.5)",
            "rgba(0, 0, 0, 1.0)",
        ];

        for color in rgb_colors {
            let (sub_type, metadata) = ContentDetector::detect(color);
            assert!(
                matches!(sub_type, ContentSubType::Color),
                "Failed to detect '{}' as color",
                color
            );

            if let Some(meta) = metadata {
                if let Some(color_formats) = meta.color_formats {
                    assert!(color_formats.rgb.is_some() || color_formats.rgba.is_some());
                }
            }
        }

        let hsl_colors = vec![
            "hsl(0, 100%, 50%)",
            "hsl(120, 50%, 25%)",
            "hsl(240, 100%, 100%)",
        ];

        for color in hsl_colors {
            let (sub_type, metadata) = ContentDetector::detect(color);
            assert!(
                matches!(sub_type, ContentSubType::Color),
                "Failed to detect '{}' as color",
                color
            );

            if let Some(meta) = metadata {
                if let Some(color_formats) = meta.color_formats {
                    assert!(color_formats.hsl.is_some());
                }
            }
        }
    }

    // JSON detection tests
    #[test]
    fn test_json_detection() {
        let valid_json = vec![
            r#"{"key": "value"}"#,
            r#"{"number": 42, "boolean": true, "null": null}"#,
            r#"[1, 2, 3, 4, 5]"#,
            r#"{"nested": {"object": {"deep": true}}}"#,
            r#"{"array": [{"item": 1}, {"item": 2}]}"#,
            "{}",
            "[]",
            r#"{"unicode": "你好世界"}"#,
        ];

        for json in valid_json {
            let (sub_type, _) = ContentDetector::detect(json);
            assert!(
                matches!(sub_type, ContentSubType::Json),
                "Failed to detect JSON: {}",
                json
            );
        }

        let invalid_json = vec![
            r#"{"key": value}"#,    // Unquoted value
            r#"{"key": "value",}"#, // Trailing comma
            r#"{key: "value"}"#,    // Unquoted key
            r#"{"key": "value"#,    // Incomplete
        ];

        for json in invalid_json {
            let (sub_type, _) = ContentDetector::detect(json);
            assert!(
                !matches!(sub_type, ContentSubType::Json),
                "Incorrectly detected invalid JSON: {}",
                json
            );
        }
    }

    // Command detection tests
    #[test]
    fn test_command_detection() {
        let valid_commands = vec![
            "git status",
            "npm install",
            "cargo build",
            "docker run image",
            "kubectl get pods",
            "python script.py",
            "ls -la",
            "cd /home/user",
            "mkdir new_directory",
            "curl -X GET https://api.example.com",
        ];

        for command in valid_commands {
            let (sub_type, _) = ContentDetector::detect(command);
            assert!(
                matches!(sub_type, ContentSubType::Command),
                "Failed to detect command: {}",
                command
            );
        }
    }

    // Timestamp detection tests
    #[test]
    fn test_timestamp_detection() {
        // Unix timestamps (seconds)
        let unix_seconds = vec![
            "1640995200", // 2022-01-01 00:00:00
            "946684800",  // 2000-01-01 00:00:00
        ];

        for ts in unix_seconds {
            let (sub_type, metadata) = ContentDetector::detect(ts);
            assert!(
                matches!(sub_type, ContentSubType::Timestamp),
                "Failed to detect timestamp: {}",
                ts
            );

            if let Some(meta) = metadata {
                if let Some(timestamp_formats) = meta.timestamp_formats {
                    assert!(timestamp_formats.unix_ms.is_some());
                }
            }
        }

        // Unix timestamps (milliseconds)
        let unix_millis = vec![
            "1640995200000", // 2022-01-01 00:00:00.000
            "1754568465706", // Future timestamp
        ];

        for ts in unix_millis {
            let (sub_type, _metadata) = ContentDetector::detect(ts);
            assert!(
                matches!(sub_type, ContentSubType::Timestamp),
                "Failed to detect timestamp: {}",
                ts
            );
        }

        // ISO 8601 timestamps
        let iso_timestamps = vec![
            "2022-01-01T00:00:00Z",
            "2022-01-01T12:30:45.123Z",
            "2022-01-01T12:30:45+08:00",
            "2022-01-01T12:30:45-05:00",
        ];

        for ts in iso_timestamps {
            let (sub_type, metadata) = ContentDetector::detect(ts);
            assert!(
                matches!(sub_type, ContentSubType::Timestamp),
                "Failed to detect ISO timestamp: {}",
                ts
            );

            if let Some(meta) = metadata {
                if let Some(timestamp_formats) = meta.timestamp_formats {
                    assert!(timestamp_formats.iso8601.is_some());
                }
            }
        }

        // Date strings
        let date_strings = vec![
            "2022-01-01",
            "2022/01/01",
            "2022-01-01 12:30:45",
            "2022/01/01 12:30:45",
        ];

        for ts in date_strings {
            let (sub_type, _metadata) = ContentDetector::detect(ts);
            assert!(
                matches!(sub_type, ContentSubType::Timestamp),
                "Failed to detect date string: {}",
                ts
            );
        }
    }

    // Markdown detection tests
    #[test]
    fn test_markdown_detection() {
        let markdown_samples = vec![
            "# Header 1",
            "## Header 2",
            "### Header 3",
            "**Bold text**",
            "*Italic text*",
            "[Link text](https://example.com)",
            "![Alt text](image.png)",
            "- List item",
            "* Another list item",
            "1. Ordered list",
            "> Quote block",
            "```code block```",
            "`inline code`",
        ];

        for markdown in markdown_samples {
            let (sub_type, _) = ContentDetector::detect(markdown);
            assert!(
                matches!(sub_type, ContentSubType::Markdown),
                "Failed to detect markdown: {}",
                markdown
            );
        }
    }

    // Code detection tests
    #[test]
    fn test_code_language_detection() {
        let code_samples = vec![
            // JavaScript
            ("function hello() { return 'world'; }", "javascript"),
            (
                "const arr = [1, 2, 3]; const result = arr.map(x => x * 2);",
                "javascript",
            ),
            (
                "async function fetchData() { await fetch('/api'); }",
                "javascript",
            ),
            // Python
            ("def hello_world():\n    print('Hello, World!')", "python"),
            ("import numpy as np\nfrom sklearn import datasets", "python"),
            ("if __name__ == '__main__':\n    main()", "python"),
            // Rust
            ("fn main() { println!(\"Hello, world!\"); }", "rust"),
            (
                "impl Display for MyStruct { fn fmt(&self, f: &mut Formatter) -> Result { } }",
                "rust",
            ),
            ("pub struct Config { pub name: String, }", "rust"),
            // Java
            (
                "public class HelloWorld { public static void main(String[] args) { } }",
                "java",
            ),
            ("private static final String CONSTANT = \"value\";", "java"),
            ("import java.util.List;", "java"),
            // C
            (
                "#include <stdio.h>\nint main() { printf(\"Hello\"); return 0; }",
                "c",
            ),
            ("void* malloc(size_t size);", "c"),
            // SQL
            ("SELECT * FROM users WHERE age > 18", "sql"),
            (
                "INSERT INTO table (column1, column2) VALUES ('a', 'b')",
                "sql",
            ),
            ("CREATE TABLE users (id INTEGER PRIMARY KEY)", "sql"),
            // HTML
            ("<html><body><h1>Title</h1></body></html>", "html"),
            ("<div class=\"container\"><span>Text</span></div>", "html"),
            // CSS
            (".container { margin: 10px; padding: 5px; }", "css"),
            ("#header { background-color: blue; }", "css"),
        ];

        for (code, expected_lang) in code_samples {
            let (sub_type, metadata) = ContentDetector::detect(code);
            assert!(
                matches!(sub_type, ContentSubType::Code),
                "Failed to detect code: {}",
                code
            );

            if let Some(meta) = metadata {
                if let Some(detected_lang) = meta.detected_language {
                    assert_eq!(
                        detected_lang, expected_lang,
                        "Wrong language detected for code: {}",
                        code
                    );
                }
            }
        }
    }

    // Base64 detection tests
    #[test]
    fn test_base64_detection() {
        // Test valid base64 with sufficient length
        let test_data = "Hello, World! This is a test message for base64 encoding. ".repeat(5);
        let encoded = base64::engine::general_purpose::STANDARD.encode(&test_data);

        let (sub_type, metadata) = ContentDetector::detect(&encoded);
        assert!(matches!(sub_type, ContentSubType::Base64));

        if let Some(meta) = metadata {
            if let Some(base64_meta) = meta.base64_metadata {
                assert_eq!(base64_meta.estimated_original_size, test_data.len());
                assert!(base64_meta.content_hint.is_some());
            } else {
                panic!("Base64 metadata should be present");
            }
        }
    }

    #[test]
    fn test_base64_false_positives() {
        // URLs should not be detected as base64
        let (sub_type, _) = ContentDetector::detect("https://example.com/path?param=value");
        assert!(matches!(sub_type, ContentSubType::Url));

        // Short plain text should not be detected as base64
        let short_text = "Hello world";
        let (sub_type, _) = ContentDetector::detect(short_text);
        assert!(matches!(sub_type, ContentSubType::PlainText));

        // Code with newlines should not be detected as base64
        let code = "function test() {\n    console.log('hello');\n    return true;\n}";
        let (sub_type, _) = ContentDetector::detect(code);
        assert!(matches!(sub_type, ContentSubType::Code));

        // Common English words should not be detected as base64
        let common_words = [
            "hello", "world", "test", "cat", "dog", "run", "yes", "ANALYSIS", "Analysis",
        ];
        for word in common_words {
            let (sub_type, _) = ContentDetector::detect(word);
            assert!(
                !matches!(sub_type, ContentSubType::Base64),
                "Incorrectly detected '{}' as base64",
                word
            );
        }

        let developer_texts = [
            "/admin/organizations",
            "/api/users",
            "/v1/chat/completions",
            "foo/bar/baz",
            "clientSecret",
            "userSettings",
            "CookieHeader",
            "BearerTokenX",
            "MTIz",
        ];
        for text in developer_texts {
            let (sub_type, _) = ContentDetector::detect(text);
            assert!(
                matches!(sub_type, ContentSubType::PlainText),
                "Expected '{}' to be plain text, got {:?}",
                text,
                sub_type
            );
        }
    }

    #[test]
    fn test_base64_with_whitespace() {
        // Test base64 with formatting (newlines)
        let test_data = "This is a longer test message for base64 encoding that will result in a multi-line base64 string when formatted.";
        let encoded = base64::engine::general_purpose::STANDARD.encode(test_data);

        // Add some newlines (but not too many)
        let formatted = encoded
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 64 == 0 {
                    vec!['\n', c]
                } else {
                    vec![c]
                }
            })
            .collect::<String>();

        let (sub_type, metadata) = ContentDetector::detect(&formatted);
        assert!(matches!(sub_type, ContentSubType::Base64));

        if let Some(meta) = metadata {
            assert!(meta.base64_metadata.is_some());
        }
    }

    #[test]
    fn test_base64_short_strings() {
        // Test short valid base64 strings
        let test_cases = [
            ("YWI=", "ab"),
            ("SGVsbG8=", "Hello"),
            ("VGVzdA==", "Test"),
            ("data:text/plain;base64,SGVsbG8=", "Hello"),
        ];

        for (encoded, expected_decoded) in test_cases {
            let (sub_type, metadata) = ContentDetector::detect(encoded);
            assert!(
                matches!(sub_type, ContentSubType::Base64),
                "Failed to detect '{}' as base64",
                encoded
            );

            if let Some(meta) = metadata {
                if let Some(base64_meta) = meta.base64_metadata {
                    assert_eq!(base64_meta.estimated_original_size, expected_decoded.len());
                }
            }
        }
    }

    #[test]
    fn test_base64_binary_data() {
        // Test binary data base64 encoding
        let binary_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        let encoded = base64::engine::general_purpose::STANDARD.encode(&binary_data);

        let (sub_type, metadata) = ContentDetector::detect(&encoded);
        if matches!(sub_type, ContentSubType::Base64) {
            if let Some(meta) = metadata {
                if let Some(base64_meta) = meta.base64_metadata {
                    // Should be recognized as PNG format
                    assert!(base64_meta
                        .content_hint
                        .as_ref()
                        .is_some_and(|h| h.contains("PNG")));
                }
            }
        }
    }

    // Edge cases and boundary tests
    #[test]
    fn test_extremely_long_text() {
        let long_text = "A".repeat(10000);
        let (sub_type, _) = ContentDetector::detect(&long_text);
        // Should be detected as plain text
        assert!(matches!(sub_type, ContentSubType::PlainText));
    }

    #[test]
    fn test_mixed_content_prioritization() {
        // Test that more specific types take priority
        let json_with_url = r#"{"url": "https://example.com", "data": "value"}"#;
        let (sub_type, _) = ContentDetector::detect(json_with_url);
        assert!(matches!(sub_type, ContentSubType::Json)); // JSON should take priority over URL

        let markdown_with_code = "# Header\n```javascript\nfunction test() {}\n```";
        let (sub_type, _) = ContentDetector::detect(markdown_with_code);
        assert!(matches!(sub_type, ContentSubType::Markdown)); // Markdown should be detected first
    }

    #[test]
    fn test_special_characters_and_unicode() {
        let unicode_text = "🌟 Unicode symbols and emojis 🚀 测试中文 тест кириллица";
        let (sub_type, _) = ContentDetector::detect(unicode_text);
        assert!(matches!(sub_type, ContentSubType::PlainText));

        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        let (sub_type, _) = ContentDetector::detect(special_chars);
        assert!(matches!(sub_type, ContentSubType::PlainText));
    }

    #[test]
    fn test_malformed_inputs() {
        let malformed_inputs = vec![
            "http://",          // Incomplete URL
            "#gg",              // Invalid hex color
            "rgb(256, 0, 0)",   // Invalid RGB values
            "{incomplete json", // Malformed JSON
            "not-an-email@",    // Incomplete email
        ];

        for input in malformed_inputs {
            let (sub_type, _) = ContentDetector::detect(input);
            // Should fall back to PlainText for malformed inputs
            assert!(
                matches!(sub_type, ContentSubType::PlainText),
                "Malformed input '{}' should be PlainText, got {:?}",
                input,
                sub_type
            );
        }
    }
}
