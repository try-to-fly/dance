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
    markers: &PasteboardMarkers,
    self_generated: bool,
    excluded_app: bool,
    text_size_valid: bool,
) -> CaptureDisposition {
    if self_generated
        || markers.is_transient
        || markers.is_concealed
        || markers.is_remote
        || excluded_app
        || !text_size_valid
    {
        return CaptureDisposition::Skip;
    }

    if markers.is_auto_generated {
        return CaptureDisposition::CurrentOnly;
    }

    CaptureDisposition::Persist
}
