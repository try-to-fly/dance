pub mod macos_markers;
pub mod policy;
pub mod runtime;

#[allow(unused_imports)]
pub use policy::{decide_capture, CaptureDisposition, PasteboardMarkers};
pub use runtime::{
    calculate_content_hash, consume_suppression_key, CaptureRuntime, SuppressionEntry,
};
