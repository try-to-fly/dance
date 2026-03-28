#![allow(dead_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const ANALYSIS_CONTRACT_VERSION: i32 = 1;
pub const TEXT_ANALYSIS_VERSION: i32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisSubtype {
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

impl AnalysisSubtype {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlainText => "plain_text",
            Self::Url => "url",
            Self::IpAddress => "ip_address",
            Self::Email => "email",
            Self::Color => "color",
            Self::Code => "code",
            Self::Command => "command",
            Self::Timestamp => "timestamp",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Base64 => "base64",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "plain_text" => Some(Self::PlainText),
            "url" => Some(Self::Url),
            "ip_address" => Some(Self::IpAddress),
            "email" => Some(Self::Email),
            "color" => Some(Self::Color),
            "code" => Some(Self::Code),
            "command" => Some(Self::Command),
            "timestamp" => Some(Self::Timestamp),
            "json" => Some(Self::Json),
            "markdown" => Some(Self::Markdown),
            "base64" => Some(Self::Base64),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Matched,
    Fallback,
}

impl AnalysisStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::Fallback => "fallback",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "matched" => Some(Self::Matched),
            "fallback" => Some(Self::Fallback),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDiagnosticCode {
    HeuristicFallback,
    JsonMalformed,
    Base64Malformed,
    UrlMalformed,
    MetadataUnavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisDiagnostic {
    pub code: AnalysisDiagnosticCode,
    pub severity: AnalysisDiagnosticSeverity,
    pub message: String,
}

impl AnalysisDiagnostic {
    pub fn new(
        code: AnalysisDiagnosticCode,
        severity: AnalysisDiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlainTextMetadata {
    pub char_count: usize,
    pub line_count: usize,
}

impl PlainTextMetadata {
    pub fn from_text(text: &str) -> Self {
        Self {
            char_count: text.chars().count(),
            line_count: count_lines(text),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UrlQueryParam {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UrlMetadata {
    pub protocol: String,
    pub host: String,
    pub path: String,
    pub query_params: Vec<UrlQueryParam>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IpAddressVersion {
    V4,
    V6,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpAddressMetadata {
    pub version: IpAddressVersion,
    pub is_loopback: bool,
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailMetadata {
    pub local_part: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorMetadata {
    pub hex: Option<String>,
    pub rgb: Option<String>,
    pub rgba: Option<String>,
    pub hsl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeMetadata {
    pub detected_language: Option<String>,
    pub line_count: usize,
}

impl CodeMetadata {
    pub fn from_text(text: &str, detected_language: Option<String>) -> Self {
        Self {
            detected_language,
            line_count: count_lines(text),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandMetadata {
    pub command_name: Option<String>,
    pub shell_family: Option<String>,
    pub has_pipeline: bool,
    pub has_sudo_prefix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimestampMetadata {
    pub unix_ms: Option<i64>,
    pub iso8601: Option<String>,
    pub date_string: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonRootKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonMetadata {
    pub root_kind: JsonRootKind,
    pub key_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownMetadata {
    pub has_heading: bool,
    pub has_list: bool,
    pub has_code_fence: bool,
    pub has_link: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Base64Metadata {
    pub estimated_original_size: usize,
    pub encoded_size: usize,
    pub content_hint: Option<String>,
    pub encoding_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum AnalysisMetadata {
    PlainText(PlainTextMetadata),
    Url(UrlMetadata),
    IpAddress(IpAddressMetadata),
    Email(EmailMetadata),
    Color(ColorMetadata),
    Code(CodeMetadata),
    Command(CommandMetadata),
    Timestamp(TimestampMetadata),
    Json(JsonMetadata),
    Markdown(MarkdownMetadata),
    Base64(Base64Metadata),
}

impl AnalysisMetadata {
    pub fn subtype(&self) -> AnalysisSubtype {
        match self {
            AnalysisMetadata::PlainText(_) => AnalysisSubtype::PlainText,
            AnalysisMetadata::Url(_) => AnalysisSubtype::Url,
            AnalysisMetadata::IpAddress(_) => AnalysisSubtype::IpAddress,
            AnalysisMetadata::Email(_) => AnalysisSubtype::Email,
            AnalysisMetadata::Color(_) => AnalysisSubtype::Color,
            AnalysisMetadata::Code(_) => AnalysisSubtype::Code,
            AnalysisMetadata::Command(_) => AnalysisSubtype::Command,
            AnalysisMetadata::Timestamp(_) => AnalysisSubtype::Timestamp,
            AnalysisMetadata::Json(_) => AnalysisSubtype::Json,
            AnalysisMetadata::Markdown(_) => AnalysisSubtype::Markdown,
            AnalysisMetadata::Base64(_) => AnalysisSubtype::Base64,
        }
    }

    pub fn to_legacy_metadata_json(&self) -> Option<String> {
        let value = match self {
            AnalysisMetadata::PlainText(_) => return None,
            AnalysisMetadata::Url(metadata) => json!({
                "url_parts": {
                    "protocol": metadata.protocol,
                    "host": metadata.host,
                    "path": metadata.path,
                    "query_params": metadata
                        .query_params
                        .iter()
                        .map(|param| (&param.key, &param.value))
                        .collect::<Vec<_>>(),
                }
            }),
            AnalysisMetadata::IpAddress(metadata) => json!({
                "version": match metadata.version {
                    IpAddressVersion::V4 => "v4",
                    IpAddressVersion::V6 => "v6",
                },
                "is_loopback": metadata.is_loopback,
                "is_private": metadata.is_private,
            }),
            AnalysisMetadata::Email(metadata) => json!({
                "local_part": metadata.local_part,
                "domain": metadata.domain,
            }),
            AnalysisMetadata::Color(metadata) => json!({
                "color_formats": {
                    "hex": metadata.hex,
                    "rgb": metadata.rgb,
                    "rgba": metadata.rgba,
                    "hsl": metadata.hsl,
                }
            }),
            AnalysisMetadata::Code(metadata) => json!({
                "detected_language": metadata.detected_language,
                "line_count": metadata.line_count,
            }),
            AnalysisMetadata::Command(metadata) => json!({
                "command_name": metadata.command_name,
                "shell_family": metadata.shell_family,
                "has_pipeline": metadata.has_pipeline,
                "has_sudo_prefix": metadata.has_sudo_prefix,
            }),
            AnalysisMetadata::Timestamp(metadata) => json!({
                "timestamp_formats": {
                    "unix_ms": metadata.unix_ms,
                    "iso8601": metadata.iso8601,
                    "date_string": metadata.date_string,
                }
            }),
            AnalysisMetadata::Json(metadata) => json!({
                "root_kind": match metadata.root_kind {
                    JsonRootKind::Object => "object",
                    JsonRootKind::Array => "array",
                    JsonRootKind::String => "string",
                    JsonRootKind::Number => "number",
                    JsonRootKind::Boolean => "boolean",
                    JsonRootKind::Null => "null",
                },
                "key_count": metadata.key_count,
            }),
            AnalysisMetadata::Markdown(metadata) => json!({
                "has_heading": metadata.has_heading,
                "has_list": metadata.has_list,
                "has_code_fence": metadata.has_code_fence,
                "has_link": metadata.has_link,
            }),
            AnalysisMetadata::Base64(metadata) => json!({
                "base64_metadata": {
                    "estimated_original_size": metadata.estimated_original_size,
                    "encoded_size": metadata.encoded_size,
                    "content_hint": metadata.content_hint,
                    "encoding_efficiency": metadata.encoding_efficiency,
                }
            }),
        };

        Some(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisSnapshot {
    pub contract_version: i32,
    pub analysis_version: i32,
    pub status: AnalysisStatus,
    pub subtype: AnalysisSubtype,
    pub metadata: AnalysisMetadata,
    pub diagnostics: Vec<AnalysisDiagnostic>,
    pub analyzed_at: i64,
}

impl AnalysisSnapshot {
    pub fn matched(subtype: AnalysisSubtype, metadata: AnalysisMetadata) -> Self {
        debug_assert_eq!(subtype, metadata.subtype());

        Self {
            contract_version: ANALYSIS_CONTRACT_VERSION,
            analysis_version: TEXT_ANALYSIS_VERSION,
            status: AnalysisStatus::Matched,
            subtype,
            metadata,
            diagnostics: Vec::new(),
            analyzed_at: Utc::now().timestamp_millis(),
        }
    }

    pub fn fallback_plain_text(text: &str, diagnostics: Vec<AnalysisDiagnostic>) -> Self {
        Self {
            contract_version: ANALYSIS_CONTRACT_VERSION,
            analysis_version: TEXT_ANALYSIS_VERSION,
            status: AnalysisStatus::Fallback,
            subtype: AnalysisSubtype::PlainText,
            metadata: AnalysisMetadata::PlainText(PlainTextMetadata::from_text(text)),
            diagnostics,
            analyzed_at: Utc::now().timestamp_millis(),
        }
    }
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count().max(1)
    }
}
