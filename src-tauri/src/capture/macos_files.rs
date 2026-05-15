use std::path::PathBuf;

pub fn read_pasteboard_file_paths() -> Vec<PathBuf> {
    #[cfg(all(target_os = "macos", not(test)))]
    {
        read_pasteboard_file_paths_macos()
    }

    #[cfg(any(not(target_os = "macos"), test))]
    {
        Vec::new()
    }
}

#[cfg(all(target_os = "macos", not(test)))]
fn read_pasteboard_file_paths_macos() -> Vec<PathBuf> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    std::panic::catch_unwind(|| unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard == nil {
            return Vec::new();
        }

        let items: id = msg_send![pasteboard, pasteboardItems];
        if items == nil {
            return Vec::new();
        }

        let count: usize = msg_send![items, count];
        let file_url_type = NSString::alloc(nil).init_str("public.file-url");
        let mut paths = Vec::new();

        for index in 0..count {
            let item: id = msg_send![items, objectAtIndex: index];
            if item == nil {
                continue;
            }

            let value: id = msg_send![item, stringForType: file_url_type];
            if value == nil {
                continue;
            }

            let c_str: *const i8 = msg_send![value, UTF8String];
            if c_str.is_null() {
                continue;
            }

            let Ok(raw_url) = std::ffi::CStr::from_ptr(c_str).to_str() else {
                continue;
            };
            let Ok(parsed) = url::Url::parse(raw_url) else {
                continue;
            };
            let Ok(path) = parsed.to_file_path() else {
                continue;
            };
            paths.push(path);
        }

        paths
    })
    .unwrap_or_else(|_| {
        log::error!("[macos_files] 读取 pasteboard 文件 URL 失败，回退为空列表");
        Vec::new()
    })
}
