#![allow(dead_code)]

use crate::analysis::contract::{
    AnalysisDiagnostic, AnalysisMetadata, AnalysisSnapshot, AnalysisSubtype, Base64Metadata,
    CodeMetadata, ColorMetadata, CommandMetadata, EmailMetadata, IpAddressMetadata,
    IpAddressVersion, JsonMetadata, JsonRootKind, MarkdownMetadata, PlainTextMetadata,
    TimestampMetadata, UrlMetadata, UrlQueryParam,
};
use crate::clipboard::content_detector::{
    Base64Metadata as DetectorBase64Metadata, ColorFormats as DetectorColorFormats,
    ContentDetector, ContentMetadata as DetectorContentMetadata, ContentSubType,
    TimestampFormats as DetectorTimestampFormats, UrlParts as DetectorUrlParts,
};
use serde_json::Value;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct TextAnalysisService;

impl TextAnalysisService {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, text: &str) -> AnalysisSnapshot {
        let (detected_subtype, detector_metadata) = ContentDetector::detect(text);
        let subtype = map_subtype(detected_subtype);
        let metadata = self.build_metadata(subtype, text, detector_metadata.as_ref());

        AnalysisSnapshot::matched(subtype, metadata)
    }

    pub fn fallback_plain_text(
        &self,
        text: &str,
        diagnostics: Vec<AnalysisDiagnostic>,
    ) -> AnalysisSnapshot {
        AnalysisSnapshot::fallback_plain_text(text, diagnostics)
    }

    fn build_metadata(
        &self,
        subtype: AnalysisSubtype,
        text: &str,
        detector_metadata: Option<&DetectorContentMetadata>,
    ) -> AnalysisMetadata {
        match subtype {
            AnalysisSubtype::PlainText => {
                AnalysisMetadata::PlainText(PlainTextMetadata::from_text(text))
            }
            AnalysisSubtype::Url => AnalysisMetadata::Url(build_url_metadata(
                detector_metadata.and_then(|metadata| metadata.url_parts.as_ref()),
                text,
            )),
            AnalysisSubtype::IpAddress => {
                AnalysisMetadata::IpAddress(build_ip_address_metadata(text))
            }
            AnalysisSubtype::Email => AnalysisMetadata::Email(build_email_metadata(text)),
            AnalysisSubtype::Color => AnalysisMetadata::Color(build_color_metadata(
                detector_metadata.and_then(|metadata| metadata.color_formats.as_ref()),
            )),
            AnalysisSubtype::Code => AnalysisMetadata::Code(CodeMetadata::from_text(
                text,
                detector_metadata.and_then(|metadata| metadata.detected_language.clone()),
            )),
            AnalysisSubtype::Command => AnalysisMetadata::Command(build_command_metadata(text)),
            AnalysisSubtype::Timestamp => AnalysisMetadata::Timestamp(build_timestamp_metadata(
                detector_metadata.and_then(|metadata| metadata.timestamp_formats.as_ref()),
            )),
            AnalysisSubtype::Json => AnalysisMetadata::Json(build_json_metadata(text)),
            AnalysisSubtype::Markdown => AnalysisMetadata::Markdown(build_markdown_metadata(text)),
            AnalysisSubtype::Base64 => AnalysisMetadata::Base64(build_base64_metadata(
                detector_metadata.and_then(|metadata| metadata.base64_metadata.as_ref()),
            )),
        }
    }
}

fn map_subtype(subtype: ContentSubType) -> AnalysisSubtype {
    match subtype {
        ContentSubType::PlainText => AnalysisSubtype::PlainText,
        ContentSubType::Url => AnalysisSubtype::Url,
        ContentSubType::IpAddress => AnalysisSubtype::IpAddress,
        ContentSubType::Email => AnalysisSubtype::Email,
        ContentSubType::Color => AnalysisSubtype::Color,
        ContentSubType::Code => AnalysisSubtype::Code,
        ContentSubType::Command => AnalysisSubtype::Command,
        ContentSubType::Timestamp => AnalysisSubtype::Timestamp,
        ContentSubType::Json => AnalysisSubtype::Json,
        ContentSubType::Markdown => AnalysisSubtype::Markdown,
        ContentSubType::Base64 => AnalysisSubtype::Base64,
    }
}

fn build_url_metadata(url_parts: Option<&DetectorUrlParts>, text: &str) -> UrlMetadata {
    if let Some(url_parts) = url_parts {
        return UrlMetadata {
            protocol: url_parts.protocol.clone(),
            host: url_parts.host.clone(),
            path: url_parts.path.clone(),
            query_params: url_parts
                .query_params
                .iter()
                .map(|(key, value)| UrlQueryParam {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        };
    }

    if let Ok(parsed) = url::Url::parse(text) {
        return UrlMetadata {
            protocol: parsed.scheme().to_string(),
            host: parsed.host_str().unwrap_or_default().to_string(),
            path: parsed.path().to_string(),
            query_params: parsed
                .query_pairs()
                .map(|(key, value)| UrlQueryParam {
                    key: key.into_owned(),
                    value: value.into_owned(),
                })
                .collect(),
        };
    }

    UrlMetadata {
        protocol: String::new(),
        host: String::new(),
        path: text.to_string(),
        query_params: Vec::new(),
    }
}

fn build_ip_address_metadata(text: &str) -> IpAddressMetadata {
    let parsed = IpAddr::from_str(text.trim())
        .expect("content detector reported ip_address but parsing failed");

    match parsed {
        IpAddr::V4(address) => IpAddressMetadata {
            version: IpAddressVersion::V4,
            is_loopback: address.is_loopback(),
            is_private: address.is_private(),
        },
        IpAddr::V6(address) => IpAddressMetadata {
            version: IpAddressVersion::V6,
            is_loopback: address.is_loopback(),
            is_private: address.is_unique_local(),
        },
    }
}

fn build_email_metadata(text: &str) -> EmailMetadata {
    let trimmed = text.trim();
    let (local_part, domain) = trimmed
        .split_once('@')
        .expect("content detector reported email but parsing failed");

    EmailMetadata {
        local_part: local_part.to_string(),
        domain: domain.to_string(),
    }
}

fn build_color_metadata(color_formats: Option<&DetectorColorFormats>) -> ColorMetadata {
    if let Some(color_formats) = color_formats {
        return ColorMetadata {
            hex: color_formats.hex.clone(),
            rgb: color_formats.rgb.clone(),
            rgba: color_formats.rgba.clone(),
            hsl: color_formats.hsl.clone(),
        };
    }

    ColorMetadata {
        hex: None,
        rgb: None,
        rgba: None,
        hsl: None,
    }
}

fn build_command_metadata(text: &str) -> CommandMetadata {
    let trimmed = text.trim();
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let has_sudo_prefix = tokens.first().is_some_and(|token| *token == "sudo");
    let executable = if has_sudo_prefix {
        tokens.get(1).map(|token| token.to_string())
    } else {
        tokens.first().map(|token| token.to_string())
    };

    CommandMetadata {
        executable,
        has_pipeline: trimmed.contains('|'),
        has_sudo_prefix,
    }
}

fn build_timestamp_metadata(
    timestamp_formats: Option<&DetectorTimestampFormats>,
) -> TimestampMetadata {
    if let Some(timestamp_formats) = timestamp_formats {
        return TimestampMetadata {
            unix_ms: timestamp_formats.unix_ms,
            iso8601: timestamp_formats.iso8601.clone(),
            date_string: timestamp_formats.date_string.clone(),
        };
    }

    TimestampMetadata {
        unix_ms: None,
        iso8601: None,
        date_string: None,
    }
}

fn build_json_metadata(text: &str) -> JsonMetadata {
    let value: Value = serde_json::from_str(text)
        .expect("content detector reported json but serde_json parsing failed");

    match value {
        Value::Object(object) => JsonMetadata {
            root_kind: JsonRootKind::Object,
            key_count: Some(object.len()),
        },
        Value::Array(items) => JsonMetadata {
            root_kind: JsonRootKind::Array,
            key_count: Some(items.len()),
        },
        Value::String(_) => JsonMetadata {
            root_kind: JsonRootKind::String,
            key_count: None,
        },
        Value::Number(_) => JsonMetadata {
            root_kind: JsonRootKind::Number,
            key_count: None,
        },
        Value::Bool(_) => JsonMetadata {
            root_kind: JsonRootKind::Boolean,
            key_count: None,
        },
        Value::Null => JsonMetadata {
            root_kind: JsonRootKind::Null,
            key_count: None,
        },
    }
}

fn build_markdown_metadata(text: &str) -> MarkdownMetadata {
    let trimmed = text.trim();

    MarkdownMetadata {
        has_heading: trimmed
            .lines()
            .any(|line| line.trim_start().starts_with('#')),
        has_fenced_code_block: trimmed.contains("```"),
        has_link: trimmed.contains("]("),
    }
}

fn build_base64_metadata(base64_metadata: Option<&DetectorBase64Metadata>) -> Base64Metadata {
    if let Some(base64_metadata) = base64_metadata {
        return Base64Metadata {
            estimated_original_size: base64_metadata.estimated_original_size,
            encoded_size: base64_metadata.encoded_size,
            content_hint: base64_metadata.content_hint.clone(),
            encoding_efficiency: base64_metadata.encoding_efficiency,
        };
    }

    Base64Metadata {
        estimated_original_size: 0,
        encoded_size: 0,
        content_hint: None,
        encoding_efficiency: 0.0,
    }
}
