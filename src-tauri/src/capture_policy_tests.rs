#[test]
fn test_capture_policy_marker_matrix() {
    use crate::capture::{decide_capture, CaptureDisposition, PasteboardMarkers};

    let baseline = PasteboardMarkers::default();

    assert_eq!(
        decide_capture(&baseline, false, false, true),
        CaptureDisposition::Persist
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_transient: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_concealed: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(
            &PasteboardMarkers {
                is_remote: true,
                ..PasteboardMarkers::default()
            },
            false,
            false,
            true,
        ),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, true, false, true),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, false, true, true),
        CaptureDisposition::Skip
    );
    assert_eq!(
        decide_capture(&baseline, false, false, false),
        CaptureDisposition::Skip
    );
}

#[test]
fn test_capture_policy_current_only_is_non_persistent_in_v1() {
    use crate::capture::{decide_capture, CaptureDisposition, PasteboardMarkers};

    let auto_generated = PasteboardMarkers {
        is_auto_generated: true,
        ..PasteboardMarkers::default()
    };

    assert_eq!(
        decide_capture(&auto_generated, false, false, true),
        CaptureDisposition::CurrentOnly
    );
    assert_ne!(
        decide_capture(&auto_generated, false, false, true),
        CaptureDisposition::Persist
    );
}
