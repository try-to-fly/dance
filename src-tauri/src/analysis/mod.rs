#![allow(unused_imports)]

pub mod contract;
pub mod rebuild;
pub mod repository;
pub mod service;

pub use contract::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisDiagnosticSeverity, AnalysisMetadata,
    AnalysisSnapshot, AnalysisStatus, AnalysisSubtype, Base64Metadata, CodeMetadata, ColorMetadata,
    CommandMetadata, EmailMetadata, IpAddressMetadata, IpAddressVersion, JsonMetadata,
    JsonRootKind, MarkdownMetadata, PlainTextMetadata, TimestampMetadata, UrlMetadata,
    UrlQueryParam, ANALYSIS_CONTRACT_VERSION, TEXT_ANALYSIS_VERSION,
};
pub use rebuild::{EntryAnalysisRebuilder, RebuildEntryAnalysisResult, DEFAULT_REBUILD_BATCH_SIZE};
pub use repository::{
    list_stale_entry_ids, load_entry_analysis_for_history, upsert_entry_analysis,
};
pub use service::TextAnalysisService;
