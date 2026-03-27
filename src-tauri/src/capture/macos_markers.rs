#![allow(dead_code)]

use crate::capture::PasteboardMarkers;

pub fn read_pasteboard_markers() -> PasteboardMarkers {
    #[cfg(target_os = "macos")]
    {
        todo!("implemented in 01-05")
    }

    #[cfg(not(target_os = "macos"))]
    {
        PasteboardMarkers::default()
    }
}
