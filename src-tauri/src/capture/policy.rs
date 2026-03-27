#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureDisposition {
    Persist,
    CurrentOnly,
    Skip,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PasteboardMarkers {
    pub is_transient: bool,
    pub is_concealed: bool,
    pub is_auto_generated: bool,
    pub is_remote: bool,
    pub source_bundle_id: Option<String>,
}

#[allow(dead_code)]
pub fn decide_capture(
    _markers: &PasteboardMarkers,
    _self_generated: bool,
    _excluded_app: bool,
    _text_size_valid: bool,
) -> CaptureDisposition {
    todo!("implemented in 01-05")
}
