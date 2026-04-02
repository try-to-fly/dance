use crate::app_paths::AppPaths;
use crate::capture::calculate_content_hash;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::AppHandle;

const WEBPAGE_PREVIEW_VERSION: &str = "webpage-preview-v2";

fn preview_cache_path(paths: &AppPaths, url: &str) -> PathBuf {
    let cache_key = format!("{WEBPAGE_PREVIEW_VERSION}:{url}");
    let hash = calculate_content_hash(cache_key.as_bytes());
    paths
        .image_assets_dir()
        .join("url-previews")
        .join(format!("{hash}.png"))
}

fn png_file_to_data_url(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!(
        "data:image/png;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

pub fn get_cached_webpage_preview_data_url(paths: &AppPaths, url: &str) -> Result<Option<String>> {
    let cache_path = preview_cache_path(paths, url);
    if !cache_path.exists() {
        return Ok(None);
    }

    png_file_to_data_url(&cache_path).map(Some)
}

pub async fn get_or_create_webpage_preview_data_url(
    app_handle: &AppHandle,
    paths: Arc<AppPaths>,
    url: &str,
) -> Result<Option<String>> {
    if let Some(cached) = get_cached_webpage_preview_data_url(paths.as_ref(), url)? {
        return Ok(Some(cached));
    }

    let png_bytes = capture_webpage_preview_png(app_handle, url).await?;
    let cache_path = preview_cache_path(paths.as_ref(), url);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&cache_path, &png_bytes)?;

    Ok(Some(format!(
        "data:image/png;base64,{}",
        general_purpose::STANDARD.encode(png_bytes)
    )))
}

#[cfg(not(target_os = "macos"))]
async fn capture_webpage_preview_png(_app_handle: &AppHandle, _url: &str) -> Result<Vec<u8>> {
    Err(anyhow!(
        "Webpage screenshot previews are only supported on macOS"
    ))
}

#[cfg(target_os = "macos")]
async fn capture_webpage_preview_png(app_handle: &AppHandle, url: &str) -> Result<Vec<u8>> {
    use block::ConcreteBlock;
    use cocoa::appkit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
    use cocoa::base::{id, nil, NO, YES};
    use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    use std::collections::HashMap;
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;
    use tokio::sync::oneshot;

    const CAPTURE_WIDTH: f64 = 1280.0;
    const CAPTURE_HEIGHT: f64 = 900.0;
    const CAPTURE_TIMEOUT_SECS: u64 = 24;
    const SNAPSHOT_DELAY_SECS: f64 = 1.8;
    const SAFARI_USER_AGENT: &str =
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15";
    type PreviewCaptureResult = Result<Vec<u8>, String>;
    type PreviewSender = oneshot::Sender<PreviewCaptureResult>;
    type SessionStartError = (PreviewSender, String);

    struct PreviewSession {
        sender: Option<PreviewSender>,
        delegate: usize,
        web_view: usize,
        window: usize,
    }

    static SESSION_ID: AtomicUsize = AtomicUsize::new(1);
    static SESSION_STORE: OnceLock<Mutex<HashMap<usize, PreviewSession>>> = OnceLock::new();
    static DELEGATE_CLASS: OnceLock<usize> = OnceLock::new();

    fn session_store() -> &'static Mutex<HashMap<usize, PreviewSession>> {
        SESSION_STORE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn nsstring_to_string(value: id) -> String {
        if value == nil {
            return String::new();
        }

        unsafe {
            let c_str: *const i8 = msg_send![value, UTF8String];
            if c_str.is_null() {
                return String::new();
            }

            CStr::from_ptr(c_str).to_string_lossy().into_owned()
        }
    }

    fn ns_error_to_string(error: id) -> String {
        if error == nil {
            return "Unknown webpage preview error".to_string();
        }

        unsafe {
            let description: id = msg_send![error, localizedDescription];
            let message = nsstring_to_string(description);
            if message.is_empty() {
                "Unknown webpage preview error".to_string()
            } else {
                message
            }
        }
    }

    fn cleanup_session(session: PreviewSession) {
        unsafe {
            let window = session.window as id;
            let web_view = session.web_view as id;
            let delegate = session.delegate as id;
            if window != nil {
                let _: () = msg_send![window, close];
                let _: () = msg_send![window, release];
            }
            if web_view != nil {
                let _: () = msg_send![web_view, stopLoading];
                let _: () = msg_send![web_view, release];
            }
            if delegate != nil {
                let _: () = msg_send![delegate, release];
            }
        }
    }

    fn finish_session(session_id: usize, result: PreviewCaptureResult) {
        let session = session_store().lock().unwrap().remove(&session_id);
        if let Some(mut session) = session {
            if let Some(sender) = session.sender.take() {
                let _ = sender.send(result);
            }
            cleanup_session(session);
        }
    }

    fn nsimage_to_png_bytes(image: id) -> Result<Vec<u8>, String> {
        unsafe {
            if image == nil {
                return Err("Webpage preview snapshot returned no image".to_string());
            }

            let tiff_data: id = msg_send![image, TIFFRepresentation];
            if tiff_data == nil {
                return Err("Failed to get TIFF data from webpage snapshot".to_string());
            }

            let image_rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff_data];
            if image_rep == nil {
                return Err(
                    "Failed to create bitmap representation for webpage snapshot".to_string(),
                );
            }

            let png_data: id = msg_send![
                image_rep,
                representationUsingType: 4
                properties: nil
            ];
            if png_data == nil {
                return Err("Failed to encode webpage snapshot as PNG".to_string());
            }

            let length: usize = msg_send![png_data, length];
            let bytes_ptr: *const u8 = msg_send![png_data, bytes];
            if bytes_ptr.is_null() || length == 0 {
                return Err("Encoded webpage snapshot PNG was empty".to_string());
            }

            Ok(std::slice::from_raw_parts(bytes_ptr, length).to_vec())
        }
    }

    fn take_snapshot(session_id: usize, web_view: id) {
        let completion = ConcreteBlock::new(move |image: id, error: id| {
            if error != nil {
                finish_session(session_id, Err(ns_error_to_string(error)));
                return;
            }

            finish_session(session_id, nsimage_to_png_bytes(image));
        })
        .copy();

        unsafe {
            let configuration: id = msg_send![class!(WKSnapshotConfiguration), new];
            let snapshot_width: id = msg_send![class!(NSNumber), numberWithDouble: CAPTURE_WIDTH];
            let _: () = msg_send![configuration, setSnapshotWidth: snapshot_width];
            let _: () = msg_send![configuration, setAfterScreenUpdates: YES];
            let _: () = msg_send![
                web_view,
                takeSnapshotWithConfiguration: configuration
                completionHandler: &*completion
            ];
            let _: () = msg_send![configuration, release];
        }
    }

    extern "C" fn did_finish_navigation(this: &Object, _: Sel, web_view: id, _: id) {
        let _: id = web_view;
        unsafe {
            let _: () = msg_send![
                this,
                performSelector: sel!(danceCaptureSnapshot)
                withObject: nil
                afterDelay: SNAPSHOT_DELAY_SECS
            ];
        }
    }

    extern "C" fn did_fail_navigation(this: &Object, _: Sel, _: id, _: id, error: id) {
        let session_id = unsafe { *this.get_ivar::<usize>("sessionId") };
        finish_session(session_id, Err(ns_error_to_string(error)));
    }

    extern "C" fn delayed_capture_snapshot(this: &Object, _: Sel) {
        let session_id = unsafe { *this.get_ivar::<usize>("sessionId") };
        let web_view = session_store()
            .lock()
            .unwrap()
            .get(&session_id)
            .map(|session| session.web_view as id);

        if let Some(web_view) = web_view {
            unsafe {
                let _: () = msg_send![web_view, displayIfNeeded];
            }
            take_snapshot(session_id, web_view);
        } else {
            finish_session(
                session_id,
                Err("Webpage preview session disappeared before snapshot".to_string()),
            );
        }
    }

    fn preview_delegate_class() -> *const Class {
        *DELEGATE_CLASS.get_or_init(|| unsafe {
            let superclass = class!(NSObject);
            let mut decl = ClassDecl::new("DanceWebPagePreviewNavigationDelegate", superclass)
                .expect("failed to create webpage preview delegate class");
            decl.add_ivar::<usize>("sessionId");
            decl.add_method(
                sel!(webView:didFinishNavigation:),
                did_finish_navigation as extern "C" fn(&Object, Sel, id, id),
            );
            decl.add_method(
                sel!(webView:didFailNavigation:withError:),
                did_fail_navigation as extern "C" fn(&Object, Sel, id, id, id),
            );
            decl.add_method(
                sel!(webView:didFailProvisionalNavigation:withError:),
                did_fail_navigation as extern "C" fn(&Object, Sel, id, id, id),
            );
            decl.add_method(
                sel!(danceCaptureSnapshot),
                delayed_capture_snapshot as extern "C" fn(&Object, Sel),
            );
            decl.register() as *const Class as usize
        }) as *const Class
    }

    fn start_capture_session(
        session_id: usize,
        url: &str,
        sender: PreviewSender,
    ) -> std::result::Result<(), SessionStartError> {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            let frame = NSRect::new(
                NSPoint::new(0., 0.),
                NSSize::new(CAPTURE_WIDTH, CAPTURE_HEIGHT),
            );

            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                frame,
                NSWindowStyleMask::NSBorderlessWindowMask,
                NSBackingStoreType::NSBackingStoreBuffered,
                NO,
            );
            if window == nil {
                return Err((
                    sender,
                    "Failed to create hidden window for webpage preview".to_string(),
                ));
            }

            let _: () = msg_send![window, setReleasedWhenClosed: NO];
            let _: () = msg_send![window, setFrameOrigin: NSPoint::new(-16000., -16000.)];
            let _: () = msg_send![window, setOpaque: YES];
            let _: () = msg_send![window, setHasShadow: NO];
            let _: () = msg_send![window, setAlphaValue: 0.01f64];

            let web_view: id = msg_send![class!(WKWebView), alloc];
            let web_view: id = msg_send![web_view, initWithFrame: frame];
            if web_view == nil {
                let _: () = msg_send![window, release];
                return Err((
                    sender,
                    "Failed to create WKWebView for webpage preview".to_string(),
                ));
            }

            let delegate_class = preview_delegate_class();
            let delegate: id = msg_send![delegate_class, new];
            if delegate == nil {
                let _: () = msg_send![web_view, release];
                let _: () = msg_send![window, release];
                return Err((
                    sender,
                    "Failed to create WKNavigationDelegate for webpage preview".to_string(),
                ));
            }

            (&mut *delegate).set_ivar("sessionId", session_id);

            let user_agent = NSString::alloc(nil).init_str(SAFARI_USER_AGENT);
            let _: () = msg_send![web_view, setCustomUserAgent: user_agent];
            let _: () = msg_send![user_agent, release];
            let _: () = msg_send![web_view, setNavigationDelegate: delegate];
            let _: () = msg_send![window, setContentView: web_view];
            let _: () = msg_send![window, orderFront: nil];
            let _: () = msg_send![window, displayIfNeeded];
            let _: () = msg_send![web_view, displayIfNeeded];

            session_store().lock().unwrap().insert(
                session_id,
                PreviewSession {
                    sender: Some(sender),
                    delegate: delegate as usize,
                    web_view: web_view as usize,
                    window: window as usize,
                },
            );

            let url_ns = NSString::alloc(nil).init_str(url);
            let ns_url: id = msg_send![class!(NSURL), URLWithString: url_ns];
            let _: () = msg_send![url_ns, release];
            if ns_url == nil {
                finish_session(
                    session_id,
                    Err("Invalid URL for webpage preview".to_string()),
                );
                return Ok(());
            }

            let request: id = msg_send![class!(NSURLRequest), requestWithURL: ns_url];
            if request == nil {
                finish_session(
                    session_id,
                    Err("Failed to create URL request for webpage preview".to_string()),
                );
                return Ok(());
            }

            let _: id = msg_send![web_view, loadRequest: request];
            Ok(())
        }
    }

    let session_id = SESSION_ID.fetch_add(1, Ordering::Relaxed);
    let url = url.trim().to_string();
    let (sender, receiver) = oneshot::channel::<PreviewCaptureResult>();
    let handle = app_handle.clone();

    app_handle.run_on_main_thread(move || {
        if let Err((sender, error)) = start_capture_session(session_id, &url, sender) {
            let _ = sender.send(Err(error));
        }
    })?;

    match tokio::time::timeout(Duration::from_secs(CAPTURE_TIMEOUT_SECS), receiver).await {
        Ok(Ok(Ok(bytes))) => Ok(bytes),
        Ok(Ok(Err(error))) => Err(anyhow!(error)),
        Ok(Err(_)) => Err(anyhow!("Webpage preview session ended unexpectedly")),
        Err(_) => {
            let _ = handle.run_on_main_thread(move || {
                finish_session(
                    session_id,
                    Err("Timed out while capturing webpage preview".to_string()),
                );
            });
            Err(anyhow!("Timed out while capturing webpage preview"))
        }
    }
}
