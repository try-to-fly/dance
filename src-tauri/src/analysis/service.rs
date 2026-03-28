#![allow(dead_code)]

use crate::analysis::contract::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisDiagnosticSeverity, AnalysisMetadata,
    AnalysisSnapshot, AnalysisSubtype, Base64Metadata, CodeMetadata, ColorMetadata,
    CommandMetadata, EmailMetadata, IpAddressMetadata, IpAddressVersion, JsonMetadata,
    JsonRootKind, MarkdownMetadata, PlainTextMetadata, TimestampMetadata, UrlMetadata,
    UrlQueryParam,
};
use crate::clipboard::content_detector::{
    Base64Metadata as DetectorBase64Metadata, ColorFormats as DetectorColorFormats,
    ContentDetector, ContentMetadata as DetectorContentMetadata, ContentSubType,
    TimestampFormats as DetectorTimestampFormats, UrlParts as DetectorUrlParts,
};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
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
        let trimmed = text.trim();
        if let Some(diagnostic) = detect_explicit_fallback(trimmed) {
            return self.fallback_plain_text(trimmed, vec![diagnostic]);
        }

        let (detected_subtype, detector_metadata) = ContentDetector::detect(trimmed);
        let subtype = map_subtype(detected_subtype);
        match self.build_metadata(subtype, trimmed, detector_metadata.as_ref()) {
            Ok(metadata) => AnalysisSnapshot::matched(subtype, metadata),
            Err(diagnostic) => self.fallback_plain_text(trimmed, vec![diagnostic]),
        }
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
    ) -> Result<AnalysisMetadata, AnalysisDiagnostic> {
        match subtype {
            AnalysisSubtype::PlainText => Ok(AnalysisMetadata::PlainText(
                PlainTextMetadata::from_text(text),
            )),
            AnalysisSubtype::Url => Ok(AnalysisMetadata::Url(build_url_metadata(
                detector_metadata.and_then(|metadata| metadata.url_parts.as_ref()),
                text,
            )?)),
            AnalysisSubtype::IpAddress => Ok(AnalysisMetadata::IpAddress(
                build_ip_address_metadata(text)?,
            )),
            AnalysisSubtype::Email => Ok(AnalysisMetadata::Email(build_email_metadata(text)?)),
            AnalysisSubtype::Color => Ok(AnalysisMetadata::Color(build_color_metadata(
                detector_metadata.and_then(|metadata| metadata.color_formats.as_ref()),
            )?)),
            AnalysisSubtype::Code => Ok(AnalysisMetadata::Code(CodeMetadata::from_text(
                text,
                detector_metadata.and_then(|metadata| metadata.detected_language.clone()),
            ))),
            AnalysisSubtype::Command => Ok(AnalysisMetadata::Command(build_command_metadata(text))),
            AnalysisSubtype::Timestamp => {
                Ok(AnalysisMetadata::Timestamp(build_timestamp_metadata(
                    detector_metadata.and_then(|metadata| metadata.timestamp_formats.as_ref()),
                )?))
            }
            AnalysisSubtype::Json => Ok(AnalysisMetadata::Json(build_json_metadata(text)?)),
            AnalysisSubtype::Markdown => {
                Ok(AnalysisMetadata::Markdown(build_markdown_metadata(text)))
            }
            AnalysisSubtype::Base64 => Ok(AnalysisMetadata::Base64(build_base64_metadata(
                detector_metadata.and_then(|metadata| metadata.base64_metadata.as_ref()),
            )?)),
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

fn build_url_metadata(
    url_parts: Option<&DetectorUrlParts>,
    text: &str,
) -> Result<UrlMetadata, AnalysisDiagnostic> {
    if let Some(url_parts) = url_parts {
        return Ok(UrlMetadata {
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
        });
    }

    if let Ok(parsed) = url::Url::parse(text) {
        return Ok(UrlMetadata {
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
        });
    }

    Err(metadata_unavailable(
        AnalysisSubtype::Url,
        AnalysisDiagnosticCode::UrlMalformed,
        format!("failed to derive url metadata for '{}'", text.trim()),
    ))
}

fn build_ip_address_metadata(text: &str) -> Result<IpAddressMetadata, AnalysisDiagnostic> {
    let parsed = IpAddr::from_str(text.trim()).map_err(|_| {
        metadata_unavailable(
            AnalysisSubtype::IpAddress,
            AnalysisDiagnosticCode::MetadataUnavailable,
            format!("failed to parse ip address '{}'", text.trim()),
        )
    })?;

    Ok(match parsed {
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
    })
}

fn build_email_metadata(text: &str) -> Result<EmailMetadata, AnalysisDiagnostic> {
    let trimmed = text.trim();
    let (local_part, domain) = trimmed.split_once('@').ok_or_else(|| {
        metadata_unavailable(
            AnalysisSubtype::Email,
            AnalysisDiagnosticCode::MetadataUnavailable,
            format!("failed to parse email '{}'", trimmed),
        )
    })?;

    Ok(EmailMetadata {
        local_part: local_part.to_string(),
        domain: domain.to_string(),
    })
}

fn build_color_metadata(
    color_formats: Option<&DetectorColorFormats>,
) -> Result<ColorMetadata, AnalysisDiagnostic> {
    if let Some(color_formats) = color_formats {
        return Ok(ColorMetadata {
            hex: color_formats.hex.clone(),
            rgb: color_formats.rgb.clone(),
            rgba: color_formats.rgba.clone(),
            hsl: color_formats.hsl.clone(),
        });
    }

    Err(metadata_unavailable(
        AnalysisSubtype::Color,
        AnalysisDiagnosticCode::MetadataUnavailable,
        "failed to derive color formats",
    ))
}

fn build_command_metadata(text: &str) -> CommandMetadata {
    let trimmed = text.trim();
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let has_sudo_prefix = tokens.first().is_some_and(|token| *token == "sudo");
    let command_name = if has_sudo_prefix {
        tokens.get(1).map(|token| token.to_string())
    } else {
        tokens.first().map(|token| token.to_string())
    };
    let shell_family = infer_shell_family(trimmed);

    CommandMetadata {
        command_name,
        shell_family,
        has_pipeline: trimmed.contains('|'),
        has_sudo_prefix,
    }
}

fn build_timestamp_metadata(
    timestamp_formats: Option<&DetectorTimestampFormats>,
) -> Result<TimestampMetadata, AnalysisDiagnostic> {
    if let Some(timestamp_formats) = timestamp_formats {
        return Ok(TimestampMetadata {
            unix_ms: timestamp_formats.unix_ms,
            iso8601: timestamp_formats.iso8601.clone(),
            date_string: timestamp_formats.date_string.clone(),
        });
    }

    Err(metadata_unavailable(
        AnalysisSubtype::Timestamp,
        AnalysisDiagnosticCode::MetadataUnavailable,
        "failed to derive timestamp formats",
    ))
}

fn build_json_metadata(text: &str) -> Result<JsonMetadata, AnalysisDiagnostic> {
    let value: Value = serde_json::from_str(text).map_err(|error| {
        metadata_unavailable(
            AnalysisSubtype::Json,
            AnalysisDiagnosticCode::JsonMalformed,
            format!("json parse failed: {}", error),
        )
    })?;

    Ok(match value {
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
    })
}

fn build_markdown_metadata(text: &str) -> MarkdownMetadata {
    let trimmed = text.trim();

    MarkdownMetadata {
        has_heading: trimmed
            .lines()
            .any(|line| line.trim_start().starts_with('#')),
        has_list: trimmed.lines().any(|line| {
            let candidate = line.trim_start();
            candidate.starts_with("- ")
                || candidate.starts_with("* ")
                || candidate.starts_with("+ ")
                || Regex::new(r"^\d+\.\s+").unwrap().is_match(candidate)
        }),
        has_code_fence: trimmed.contains("```"),
        has_link: trimmed.contains("]("),
    }
}

fn build_base64_metadata(
    base64_metadata: Option<&DetectorBase64Metadata>,
) -> Result<Base64Metadata, AnalysisDiagnostic> {
    if let Some(base64_metadata) = base64_metadata {
        return Ok(Base64Metadata {
            estimated_original_size: base64_metadata.estimated_original_size,
            encoded_size: base64_metadata.encoded_size,
            content_hint: base64_metadata.content_hint.clone(),
            encoding_efficiency: base64_metadata.encoding_efficiency,
        });
    }

    Err(metadata_unavailable(
        AnalysisSubtype::Base64,
        AnalysisDiagnosticCode::Base64Malformed,
        "failed to decode base64 metadata",
    ))
}

fn infer_shell_family(command: &str) -> Option<String> {
    if command.contains("&&")
        || command.contains("||")
        || command.contains("$(")
        || command.contains("${")
        || command.contains("~/")
    {
        return Some("posix".to_string());
    }

    if command.contains(".exe")
        || command.contains('\\')
        || command.to_ascii_lowercase().starts_with("powershell ")
    {
        return Some("windows".to_string());
    }

    None
}

fn detect_explicit_fallback(text: &str) -> Option<AnalysisDiagnostic> {
    if looks_like_malformed_json(text) {
        return Some(metadata_unavailable(
            AnalysisSubtype::Json,
            AnalysisDiagnosticCode::JsonMalformed,
            "structured json candidate could not be parsed",
        ));
    }

    if looks_like_malformed_url(text) {
        return Some(metadata_unavailable(
            AnalysisSubtype::Url,
            AnalysisDiagnosticCode::UrlMalformed,
            "url candidate is missing a valid host or scheme",
        ));
    }

    if looks_like_malformed_base64(text) {
        return Some(metadata_unavailable(
            AnalysisSubtype::Base64,
            AnalysisDiagnosticCode::Base64Malformed,
            "base64 candidate could not be decoded",
        ));
    }

    None
}

fn looks_like_malformed_json(text: &str) -> bool {
    let trimmed = text.trim();
    (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<Value>(trimmed).is_err()
}

fn looks_like_malformed_url(text: &str) -> bool {
    let trimmed = text.trim();
    if !trimmed.starts_with("http://")
        && !trimmed.starts_with("https://")
        && !trimmed.starts_with("ftp://")
    {
        return false;
    }

    url::Url::parse(trimmed)
        .map(|parsed| parsed.host_str().is_none())
        .unwrap_or(true)
}

fn looks_like_malformed_base64(text: &str) -> bool {
    let trimmed = text.trim();
    if let Some((_, encoded)) = trimmed.split_once(";base64,") {
        return general_purpose::STANDARD.decode(encoded.trim()).is_err();
    }

    if trimmed.len() < 16 || !trimmed.contains('=') || trimmed.chars().any(char::is_whitespace) {
        return false;
    }

    let is_base64ish = trimmed
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '+' || char == '/' || char == '=');
    is_base64ish && general_purpose::STANDARD.decode(trimmed).is_err()
}

fn metadata_unavailable(
    subtype: AnalysisSubtype,
    code: AnalysisDiagnosticCode,
    message: impl Into<String>,
) -> AnalysisDiagnostic {
    let severity = match code {
        AnalysisDiagnosticCode::JsonMalformed
        | AnalysisDiagnosticCode::Base64Malformed
        | AnalysisDiagnosticCode::UrlMalformed => AnalysisDiagnosticSeverity::Error,
        AnalysisDiagnosticCode::HeuristicFallback | AnalysisDiagnosticCode::MetadataUnavailable => {
            AnalysisDiagnosticSeverity::Warning
        }
    };

    AnalysisDiagnostic::new(
        code,
        severity,
        format!(
            "{} metadata unavailable: {}",
            subtype.as_str(),
            message.into()
        ),
    )
}
