use crate::app_paths::AppPaths;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSString;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

pub struct AppIconExtractor {
    icons_dir: PathBuf,
}

impl AppIconExtractor {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new() -> Result<Self> {
        Self::new_in(Arc::new(AppPaths::from_default_roots()?))
    }

    pub fn new_in(paths: Arc<AppPaths>) -> Result<Self> {
        let icons_dir = paths.icon_cache_dir();

        // 确保图标目录存在
        if !icons_dir.exists() {
            fs::create_dir_all(&icons_dir)?;
        }

        Ok(Self { icons_dir })
    }

    /// 提取应用图标并保存到本地缓存
    pub fn extract_and_cache_icon(&self, bundle_id: &str) -> Result<Option<PathBuf>> {
        let icon_path = self.icons_dir.join(format!("{}.png", bundle_id));

        // 如果图标已经缓存，直接返回路径
        if icon_path.exists() {
            return Ok(Some(icon_path));
        }

        // 根据平台提取图标数据
        let icon_data = self.extract_icon_data(bundle_id)?;

        if let Some(data) = icon_data {
            // 保存图标到文件
            fs::write(&icon_path, data)?;
            return Ok(Some(icon_path));
        }

        Ok(None)
    }

    /// 提取图标数据（跨平台）
    fn extract_icon_data(&self, bundle_id: &str) -> Result<Option<Vec<u8>>> {
        #[cfg(target_os = "macos")]
        {
            self.extract_icon_data_macos(bundle_id)
        }

        #[cfg(target_os = "windows")]
        {
            self.extract_icon_data_windows(bundle_id)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Ok(None)
        }
    }

    /// 使用macOS NSWorkspace API提取图标数据
    #[cfg(target_os = "macos")]
    fn extract_icon_data_macos(&self, bundle_id: &str) -> Result<Option<Vec<u8>>> {
        std::panic::catch_unwind(|| {
            unsafe {
                // 获取NSWorkspace实例
                let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
                if workspace == nil {
                    return None;
                }

                // 创建NSString用于Bundle ID
                let bundle_id_nsstring = NSString::alloc(nil).init_str(bundle_id);

                // 通过Bundle ID获取应用路径
                let app_path: id = msg_send![
                    workspace,
                    absolutePathForAppBundleWithIdentifier: bundle_id_nsstring
                ];

                if app_path == nil {
                    return None;
                }

                // 获取应用图标
                let icon: id = msg_send![workspace, iconForFile: app_path];
                if icon == nil {
                    return None;
                }

                // 设置图标大小（128x128 适合显示）
                let size = cocoa::foundation::NSSize {
                    width: 128.0,
                    height: 128.0,
                };
                let _: () = msg_send![icon, setSize: size];

                // 获取TIFF数据
                let tiff_data: id = msg_send![icon, TIFFRepresentation];
                if tiff_data == nil {
                    return None;
                }

                // 转换为NSBitmapImageRep
                let image_rep: id = msg_send![
                    class!(NSBitmapImageRep),
                    imageRepWithData: tiff_data
                ];

                if image_rep == nil {
                    return None;
                }

                // 转换为PNG数据
                let png_data: id = msg_send![
                    image_rep,
                    representationUsingType: 4 // NSBitmapImageFileTypePNG
                    properties: nil
                ];

                if png_data == nil {
                    return None;
                }

                // 获取数据长度和指针
                let length: usize = msg_send![png_data, length];
                let bytes_ptr: *const u8 = msg_send![png_data, bytes];

                if bytes_ptr.is_null() || length == 0 {
                    return None;
                }

                // 复制数据到Vector
                let data = std::slice::from_raw_parts(bytes_ptr, length).to_vec();
                Some(data)
            }
        })
        .unwrap_or_else(|_| {
            log::error!("提取应用图标时发生异常：{}", bundle_id);
            None
        })
        .ok_or_else(|| anyhow::anyhow!("Failed to extract icon data"))
        .map(Some)
        .or_else(|_| Ok(None))
    }

    /// 使用Windows Shell API提取图标数据
    #[cfg(target_os = "windows")]
    fn extract_icon_data_windows(&self, bundle_id: &str) -> Result<Option<Vec<u8>>> {
        use std::ffi::OsString;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        use winapi::shared::minwindef::{DWORD, UINT};
        use winapi::shared::windef::HICON;
        use winapi::um::shellapi::SHGetFileInfoW;
        use winapi::um::shellapi::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
        use winapi::um::wingdi::{DeleteObject, GetObjectW, BITMAP};
        use winapi::um::winuser::ICONINFO;
        use winapi::um::winuser::{DestroyIcon, GetIconInfo};

        // Try to find executable by bundle_id (simplified approach)
        let exe_name = format!("{}.exe", bundle_id);
        let mut exe_path_wide: Vec<u16> = OsString::from(&exe_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mut file_info: SHFILEINFOW = std::mem::zeroed();

            let result = SHGetFileInfoW(
                exe_path_wide.as_ptr(),
                0,
                &mut file_info,
                std::mem::size_of::<SHFILEINFOW>() as UINT,
                SHGFI_ICON | SHGFI_LARGEICON,
            );

            if result != 0 && file_info.hIcon as isize != 0 {
                // Convert HICON to PNG data (simplified)
                let icon_handle = file_info.hIcon;

                // Get icon information
                let mut icon_info: ICONINFO = std::mem::zeroed();
                if GetIconInfo(icon_handle, &mut icon_info) != 0 {
                    // Get bitmap information
                    let mut bitmap: BITMAP = std::mem::zeroed();
                    if GetObjectW(
                        icon_info.hbmColor as *mut winapi::ctypes::c_void,
                        std::mem::size_of::<BITMAP>() as i32,
                        &mut bitmap as *mut BITMAP as *mut winapi::ctypes::c_void,
                    ) != 0
                    {
                        // For simplicity, we'll create a placeholder PNG
                        // In a real implementation, you'd convert the bitmap to PNG
                        let placeholder_png = Self::create_placeholder_icon();

                        // Cleanup
                        DeleteObject(icon_info.hbmColor as *mut winapi::ctypes::c_void);
                        DeleteObject(icon_info.hbmMask as *mut winapi::ctypes::c_void);
                        DestroyIcon(icon_handle);

                        return Ok(Some(placeholder_png));
                    }

                    // Cleanup on failure
                    if icon_info.hbmColor as isize != 0 {
                        DeleteObject(icon_info.hbmColor as *mut winapi::ctypes::c_void);
                    }
                    if icon_info.hbmMask as isize != 0 {
                        DeleteObject(icon_info.hbmMask as *mut winapi::ctypes::c_void);
                    }
                }

                DestroyIcon(icon_handle);
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "windows")]
    fn create_placeholder_icon() -> Vec<u8> {
        // Simple 32x32 PNG placeholder (transparent with border)
        // This is a minimal PNG file - in production you'd want to generate proper icons
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x73, 0x7A, 0x7A, 0xF4, 0x00, 0x00, 0x00, 0x19, 0x74, 0x45, 0x58, 0x74, 0x53,
            0x6F, 0x66, 0x74, 0x77, 0x61, 0x72, 0x65, 0x00, 0x41, 0x64, 0x6F, 0x62, 0x65, 0x20,
            0x49, 0x6D, 0x61, 0x67, 0x65, 0x52, 0x65, 0x61, 0x64, 0x79, 0x71, 0xC9, 0x65, 0x3C,
            0x00, 0x00, 0x00, 0x25, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0xED, 0xC1, 0x01, 0x0D,
            0x00, 0x00, 0x00, 0xC2, 0xA0, 0xF7, 0x4F, 0x6D, 0x0E, 0x37, 0xA0, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xBE, 0x0D, 0x21, 0x00, 0x00, 0x01, 0x9A, 0x60, 0xE1, 0xD5, 0x00, 0x00, 0x00, 0x00,
            0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    /// 获取缓存的图标路径
    pub fn get_cached_icon_path(&self, bundle_id: &str) -> Option<PathBuf> {
        let icon_path = self.icons_dir.join(format!("{}.png", bundle_id));
        if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        }
    }

    /// 清理过期的图标缓存（超过30天的文件）
    #[allow(dead_code)]
    pub fn cleanup_old_icons(&self) -> Result<()> {
        let now = std::time::SystemTime::now();
        let thirty_days = std::time::Duration::from_secs(30 * 24 * 60 * 60);

        if let Ok(entries) = fs::read_dir(&self.icons_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > thirty_days {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
