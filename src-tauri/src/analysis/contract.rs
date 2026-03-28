#![allow(dead_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStatus {
    Matched,
    Fallback,
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
    pub executable: Option<String>,
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
    pub has_fenced_code_block: bool,
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
