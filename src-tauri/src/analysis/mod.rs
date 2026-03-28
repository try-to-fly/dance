#![allow(unused_imports)]

pub mod contract;
pub mod service;

pub use contract::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisDiagnosticSeverity, AnalysisMetadata,
    AnalysisSnapshot, AnalysisStatus, AnalysisSubtype, Base64Metadata, CodeMetadata, ColorMetadata,
    CommandMetadata, EmailMetadata, IpAddressMetadata, IpAddressVersion, JsonMetadata,
    JsonRootKind, MarkdownMetadata, PlainTextMetadata, TimestampMetadata, UrlMetadata,
    UrlQueryParam, ANALYSIS_CONTRACT_VERSION, TEXT_ANALYSIS_VERSION,
};
pub use service::TextAnalysisService;
