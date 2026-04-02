use crate::analysis::{
    load_entry_analysis_for_history, EntryAnalysisRebuilder, RebuildEntryAnalysisResult,
};
use crate::app_paths::AppPaths;
use crate::capture::{calculate_content_hash, CaptureRuntime};
use crate::clipboard::content_detector::ContentDetector;
use crate::clipboard::{ClipboardMonitor, ContentProcessor};
use crate::commands::{CacheStatistics, CleanupResult};
use crate::config::{AppConfig, ConfigManager};
use crate::database::Database;
use crate::models::{AppUsage, ClipboardEntry, Statistics};
use crate::retrieval::{
    refresh_favorite_search_document, search_clipboard_history, ClipboardHistoryQuery,
};
use crate::shortcuts::parse_shortcut_string;
use anyhow::Result;
use arboard::Clipboard;
use chrono::Utc;
use sqlx::Row;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tokio::sync::Mutex;
use tokio::sync::{broadcast, RwLock};

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, KeyCode};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

#[cfg(target_os = "macos")]
const MACOS_KEYCODE_V: u16 = 0x09;

#[cfg(target_os = "macos")]
struct FrontmostApplication {
    pid: i32,
    name: String,
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
fn ensure_macos_accessibility_permission() -> Result<()> {
    let is_trusted = unsafe { AXIsProcessTrusted() };
    if is_trusted {
        return Ok(());
    }

    let current_exe = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    log::error!(
        "[paste] 当前进程未通过辅助功能校验，executable={}",
        current_exe
    );

    let settings_url =
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
    if let Err(error) = std::process::Command::new("open")
        .arg(settings_url)
        .output()
    {
        log::error!("[paste] 打开辅助功能设置页失败: {}", error);
    } else {
        log::error!("[paste] Dance 缺少辅助功能权限，已尝试打开辅助功能设置页");
    }

    Err(anyhow::anyhow!(
        "Dance 缺少辅助功能权限，无法向其他应用发送粘贴按键。当前实际执行文件是：{}。如果你在用 tauri dev，请在“系统设置 > 隐私与安全性 > 辅助功能”里允许这个可执行文件，而不只是安装版 Dance.app。已尝试打开设置页。",
        current_exe
    ))
}

#[cfg(target_os = "macos")]
fn get_frontmost_application() -> Option<FrontmostApplication> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    std::panic::catch_unwind(|| unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }

        let active_app: id = msg_send![workspace, frontmostApplication];
        if active_app == nil {
            return None;
        }

        let pid: i32 = msg_send![active_app, processIdentifier];

        let localized_name: id = msg_send![active_app, localizedName];
        let name = if localized_name != nil {
            let name_c_str: *const i8 = msg_send![localized_name, UTF8String];
            if name_c_str.is_null() {
                "Unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(name_c_str)
                    .to_str()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|_| "Unknown".to_string())
            }
        } else {
            "Unknown".to_string()
        };

        Some(FrontmostApplication { pid, name })
    })
    .unwrap_or_else(|_| {
        log::error!("[paste] 读取前台应用失败，已安全回退");
        None
    })
}

#[cfg(target_os = "macos")]
fn wait_for_macos_paste_target(current_process_id: i32) -> Result<FrontmostApplication> {
    for _ in 0..20 {
        if let Some(frontmost_app) = get_frontmost_application() {
            if frontmost_app.pid != current_process_id {
                return Ok(frontmost_app);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Err(anyhow::anyhow!(
        "隐藏 Dance 后没有检测到可接收粘贴的目标应用"
    ))
}

#[cfg(target_os = "macos")]
fn create_keyboard_event(
    source: &CGEventSource,
    keycode: u16,
    keydown: bool,
    flags: CGEventFlags,
) -> Result<CGEvent> {
    let event = CGEvent::new_keyboard_event(source.clone(), keycode, keydown)
        .map_err(|_| anyhow::anyhow!("Failed to create macOS keyboard event"))?;
    event.set_flags(flags);
    Ok(event)
}

#[cfg(target_os = "macos")]
fn run_macos_paste(current_process_id: u32, label: &str, log_prefix: &str) -> Result<()> {
    let target_app = wait_for_macos_paste_target(current_process_id as i32)?;
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| anyhow::anyhow!("Failed to create macOS event source"))?;
    let command_flags = CGEventFlags::CGEventFlagCommand;

    let command_down = create_keyboard_event(&source, KeyCode::COMMAND, true, command_flags)?;
    let v_down = create_keyboard_event(&source, MACOS_KEYCODE_V, true, command_flags)?;
    let v_up = create_keyboard_event(&source, MACOS_KEYCODE_V, false, command_flags)?;
    let command_up = create_keyboard_event(
        &source,
        KeyCode::COMMAND,
        false,
        CGEventFlags::CGEventFlagNull,
    )?;

    command_down.post(CGEventTapLocation::HID);
    std::thread::sleep(std::time::Duration::from_millis(10));
    v_down.post(CGEventTapLocation::HID);
    std::thread::sleep(std::time::Duration::from_millis(10));
    v_up.post(CGEventTapLocation::HID);
    std::thread::sleep(std::time::Duration::from_millis(10));
    command_up.post(CGEventTapLocation::HID);

    log::info!(
        "[{}] Pasted {} to: {} (pid={})",
        log_prefix,
        label,
        target_app.name,
        target_app.pid
    );

    Ok(())
}

pub struct AppState {
    pub paths: Arc<AppPaths>,
    pub db: Arc<Database>,
    pub capture_runtime: Arc<RwLock<Option<CaptureRuntime>>>,
    pub tx: broadcast::Sender<ClipboardEntry>,
    pub _rx: Arc<Mutex<broadcast::Receiver<ClipboardEntry>>>,
    pub app_handle: Arc<Mutex<Option<AppHandle>>>,
    pub processor: Arc<ContentProcessor>,
    pub skip_next_change: Arc<Mutex<bool>>,
    pub config_manager: Arc<Mutex<ConfigManager>>,
    pub current_shortcut: Arc<Mutex<Option<String>>>,
    pub last_cleanup_date: Arc<Mutex<Option<chrono::DateTime<Utc>>>>,
}

impl AppState {
    pub async fn new(paths: Arc<AppPaths>) -> Result<Self> {
        let db = Arc::new(Database::new_in(paths.clone()).await?);
        let (tx, rx) = broadcast::channel(100);
        let processor = Arc::new(ContentProcessor::new_in(paths.clone())?);
        let config_manager = Arc::new(Mutex::new(ConfigManager::new_in(paths.clone()).await?));

        let instance = Self {
            paths,
            db,
            capture_runtime: Arc::new(RwLock::new(None)),
            tx: tx.clone(),
            _rx: Arc::new(Mutex::new(rx)),
            app_handle: Arc::new(Mutex::new(None)),
            processor,
            skip_next_change: Arc::new(Mutex::new(false)),
            config_manager,
            current_shortcut: Arc::new(Mutex::new(None)),
            last_cleanup_date: Arc::new(Mutex::new(None)),
        };

        // 初始化清理日期
        instance.check_and_cleanup_daily().await?;

        Ok(instance)
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let app_handle = Arc::clone(&self.app_handle);
        tauri::async_runtime::spawn(async move {
            let mut guard = app_handle.lock().await;
            *guard = Some(handle);
        });
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let mut runtime_guard = self.capture_runtime.write().await;

        if runtime_guard.is_none() {
            let monitor = ClipboardMonitor::new(
                self.tx.clone(),
                Arc::clone(&self.processor),
                Arc::clone(&self.config_manager),
            )?;
            let runtime = CaptureRuntime::spawn(
                monitor,
                self.tx.clone(),
                Arc::clone(&self.db),
                Arc::clone(&self.app_handle),
            );
            *runtime_guard = Some(runtime);
        }

        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<()> {
        let runtime = {
            let mut runtime_guard = self.capture_runtime.write().await;
            runtime_guard.take()
        };

        if let Some(runtime) = runtime {
            runtime.stop().await;
        }

        Ok(())
    }

    pub async fn is_monitoring(&self) -> bool {
        let runtime_guard = self.capture_runtime.read().await;
        runtime_guard.is_some()
    }

    pub async fn get_clipboard_history(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        search: Option<String>,
    ) -> Result<Vec<ClipboardEntry>> {
        if search
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            return search_clipboard_history(
                self.db.pool(),
                ClipboardHistoryQuery {
                    text: search,
                    limit,
                    offset,
                    ..Default::default()
                },
            )
            .await;
        }

        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        load_entry_analysis_for_history(self.db.pool(), limit, offset, None).await
    }

    pub async fn search_clipboard_history(
        &self,
        query: ClipboardHistoryQuery,
    ) -> Result<Vec<ClipboardEntry>> {
        search_clipboard_history(self.db.pool(), query).await
    }

    pub async fn list_clipboard_source_apps(&self, limit: Option<i32>) -> Result<Vec<String>> {
        let limit = limit.unwrap_or(24).max(1);

        sqlx::query_scalar::<_, String>(
            r#"
            SELECT source_app
            FROM clipboard_entries
            WHERE source_app IS NOT NULL
              AND TRIM(source_app) != ''
            GROUP BY source_app
            ORDER BY MAX(created_at) DESC, COUNT(*) DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(Into::into)
    }

    pub async fn rebuild_entry_analysis(
        &self,
        batch_size: Option<usize>,
    ) -> Result<RebuildEntryAnalysisResult> {
        EntryAnalysisRebuilder::new()
            .rebuild(self.db.pool(), batch_size)
            .await
    }

    pub async fn toggle_favorite(&self, id: String) -> Result<()> {
        sqlx::query("UPDATE clipboard_entries SET is_favorite = NOT is_favorite WHERE id = ?")
            .bind(&id)
            .execute(self.db.pool())
            .await?;
        refresh_favorite_search_document(self.db.pool(), &id).await?;

        Ok(())
    }

    pub async fn delete_entry(&self, id: String) -> Result<()> {
        sqlx::query("DELETE FROM clipboard_entries WHERE id = ?")
            .bind(&id)
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    pub async fn clear_history(&self) -> Result<()> {
        sqlx::query("DELETE FROM clipboard_entries")
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<Statistics> {
        // 总条目数
        let total_entries: i64 = sqlx::query("SELECT COUNT(*) as count FROM clipboard_entries")
            .fetch_one(self.db.pool())
            .await?
            .get("count");

        // 总复制次数
        let total_copies: i64 = sqlx::query("SELECT SUM(copy_count) as sum FROM clipboard_entries")
            .fetch_one(self.db.pool())
            .await?
            .try_get("sum")
            .unwrap_or(0);

        // 最多复制的条目
        let most_copied = sqlx::query_as::<_, ClipboardEntry>(
            "SELECT * FROM clipboard_entries ORDER BY copy_count DESC LIMIT 10",
        )
        .fetch_all(self.db.pool())
        .await?;

        // 最近使用的应用
        let recent_apps = sqlx::query(
            r#"
            SELECT source_app, COUNT(*) as count 
            FROM clipboard_entries 
            WHERE source_app IS NOT NULL 
            GROUP BY source_app 
            ORDER BY count DESC 
            LIMIT 10
            "#,
        )
        .fetch_all(self.db.pool())
        .await?
        .into_iter()
        .map(|row| AppUsage {
            app_name: row.get("source_app"),
            count: row.get("count"),
        })
        .collect();

        Ok(Statistics {
            total_entries,
            total_copies,
            most_copied,
            recent_apps,
        })
    }

    pub async fn copy_to_clipboard(&self, content: String) -> Result<()> {
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(content)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn register_suppression_for_text(&self, content: &str, ttl_ms: i64) -> String {
        let normalized = ContentDetector::normalize_clipboard_text(content);
        let content_hash = calculate_content_hash(normalized.as_ref().as_bytes());

        let runtime_guard = self.capture_runtime.read().await;
        if let Some(runtime) = runtime_guard.as_ref() {
            runtime
                .register_suppression_key(content_hash.clone(), ttl_ms)
                .await;
        }

        content_hash
    }

    pub async fn copy_image_to_clipboard(&self, _file_path: String) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            tokio::task::spawn_blocking(move || -> Result<()> {
                // 使用osascript复制图片到剪贴板
                let script = format!(
                    r#"
                    set the clipboard to (read (POSIX file "{}") as «class PNGf»)
                    "#,
                    _file_path
                );

                let output = Command::new("osascript").arg("-e").arg(&script).output()?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow::anyhow!("Failed to copy image: {}", stderr));
                }

                Ok(())
            })
            .await??;
        }

        #[cfg(target_os = "windows")]
        {
            // Windows图片复制支持 - 使用arboard
            tokio::task::spawn_blocking(move || -> Result<()> {
                use image::ImageReader;
                use std::fs;

                let img_data = fs::read(&_file_path)?;
                let img = ImageReader::new(std::io::Cursor::new(&img_data))
                    .with_guessed_format()?
                    .decode()?;

                let rgba_img = img.to_rgba8();
                let (width, height) = rgba_img.dimensions();

                let img_data = arboard::ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: rgba_img.into_raw().into(),
                };

                let mut clipboard = Clipboard::new()?;
                clipboard.set_image(img_data)?;

                Ok(())
            })
            .await??;
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            return Err(anyhow::anyhow!("Image copy not supported on this platform"));
        }

        Ok(())
    }

    pub async fn set_skip_next_clipboard_change(&self, skip: bool) {
        let mut skip_guard = self.skip_next_change.lock().await;
        *skip_guard = skip;
    }

    pub async fn paste_text(
        &self,
        content: String,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<()> {
        log::info!("[paste_text] 收到粘贴请求");

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(content)?;
            Ok(())
        })
        .await??;

        // 切换应用焦点并粘贴（macOS）
        #[cfg(target_os = "macos")]
        {
            if let Err(error) = ensure_macos_accessibility_permission() {
                log::error!("[paste_text] 权限检查失败: {}", error);
                return Err(error);
            }

            let current_process_id = std::process::id();

            if let Some(handle) = app_handle.as_ref() {
                handle
                    .hide()
                    .map_err(|e| anyhow::anyhow!("Failed to hide app before pasting: {}", e))?;
            }

            let paste_result = tokio::task::spawn_blocking(move || -> Result<()> {
                log::info!("[paste_text] 开始执行粘贴流程");
                run_macos_paste(current_process_id, "text", "paste_text")
            })
            .await?;

            if let Err(error) = paste_result {
                log::error!("[paste_text] 执行失败: {}", error);
                return Err(error);
            }
        }

        Ok(())
    }

    pub async fn paste_image(
        &self,
        file_path: String,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<()> {
        log::info!("[paste_image] 收到粘贴请求");

        use std::fs;
        use std::path::PathBuf;

        // 解析文件路径
        let absolute_path = if file_path.starts_with("imgs/") {
            self.paths.resolve_relative_asset_path(&file_path)?
        } else {
            PathBuf::from(&file_path)
        };

        if !absolute_path.exists() {
            return Err(anyhow::anyhow!("File not found: {:?}", absolute_path));
        }

        tokio::task::spawn_blocking(move || -> Result<()> {
            let image_data = fs::read(&absolute_path)?;

            // 使用arboard设置图片到剪切板
            let mut clipboard = Clipboard::new()?;

            // 确定图片格式
            let img = image::load_from_memory(&image_data)
                .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e))?;

            // 转换为RGB格式
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();

            let img_data = arboard::ImageData {
                width: width as usize,
                height: height as usize,
                bytes: rgba_img.into_raw().into(),
            };

            clipboard
                .set_image(img_data)
                .map_err(|e| anyhow::anyhow!("Failed to set image to clipboard: {}", e))?;

            Ok(())
        })
        .await??;

        // 切换应用焦点并粘贴（macOS）
        #[cfg(target_os = "macos")]
        {
            if let Err(error) = ensure_macos_accessibility_permission() {
                log::error!("[paste_image] 权限检查失败: {}", error);
                return Err(error);
            }

            let current_process_id = std::process::id();

            if let Some(handle) = app_handle.as_ref() {
                handle
                    .hide()
                    .map_err(|e| anyhow::anyhow!("Failed to hide app before pasting: {}", e))?;
            }

            let paste_result = tokio::task::spawn_blocking(move || -> Result<()> {
                log::info!("[paste_image] 开始执行图片粘贴流程");
                run_macos_paste(current_process_id, "image", "paste_image")
            })
            .await?;

            if let Err(error) = paste_result {
                log::error!("[paste_image] 执行失败: {}", error);
                return Err(error);
            }
        }

        Ok(())
    }

    // Configuration management methods
    pub async fn get_config(&self) -> Result<AppConfig> {
        let config_manager = self.config_manager.lock().await;
        Ok(config_manager.config.clone())
    }

    pub async fn update_config(&self, config: AppConfig) -> Result<()> {
        let mut config_manager = self.config_manager.lock().await;
        config_manager.update_config(config).await?;
        Ok(())
    }

    // Global shortcut methods
    pub async fn register_global_shortcut(
        &self,
        app_handle: AppHandle,
        shortcut: String,
    ) -> Result<()> {
        let (normalized_shortcut, parsed_shortcut) =
            parse_shortcut_string(&shortcut).map_err(anyhow::Error::msg)?;

        let global_shortcut_manager = app_handle.global_shortcut();

        // Unregister existing shortcut if any
        if let Some(current) = self.current_shortcut.lock().await.as_ref() {
            let (_, current_shortcut) =
                parse_shortcut_string(current).map_err(anyhow::Error::msg)?;
            global_shortcut_manager
                .unregister(current_shortcut)
                .map_err(|e| anyhow::anyhow!("Failed to unregister shortcut: {}", e))?;
        }

        // Register new shortcut - the API only takes the shortcut, callback is handled via events
        global_shortcut_manager
            .register(parsed_shortcut)
            .map_err(|e| anyhow::anyhow!("Failed to register shortcut: {}", e))?;

        // Update stored shortcut
        let mut current_shortcut = self.current_shortcut.lock().await;
        *current_shortcut = Some(normalized_shortcut);

        Ok(())
    }

    pub async fn unregister_global_shortcut(&self) -> Result<()> {
        if let Some(app_handle) = self.app_handle.lock().await.as_ref() {
            if let Some(current) = self.current_shortcut.lock().await.as_ref() {
                let (_, current_shortcut) =
                    parse_shortcut_string(current).map_err(anyhow::Error::msg)?;
                let global_shortcut_manager = app_handle.global_shortcut();
                global_shortcut_manager
                    .unregister(current_shortcut)
                    .map_err(|e| anyhow::anyhow!("Failed to unregister shortcut: {}", e))?;

                let mut current_shortcut_guard = self.current_shortcut.lock().await;
                *current_shortcut_guard = None;
            }
        }
        Ok(())
    }

    // Auto startup methods
    pub async fn set_auto_startup(&self, enabled: bool) -> Result<()> {
        if let Some(app_handle) = self.app_handle.lock().await.as_ref() {
            let autolaunch_manager = app_handle.autolaunch();
            if enabled {
                autolaunch_manager
                    .enable()
                    .map_err(|e| anyhow::anyhow!("Failed to enable auto startup: {}", e))?;
            } else {
                autolaunch_manager
                    .disable()
                    .map_err(|e| anyhow::anyhow!("Failed to disable auto startup: {}", e))?;
            }
        }
        Ok(())
    }

    pub async fn get_auto_startup_status(&self) -> Result<bool> {
        if let Some(app_handle) = self.app_handle.lock().await.as_ref() {
            let autolaunch_manager = app_handle.autolaunch();
            let status = autolaunch_manager
                .is_enabled()
                .map_err(|e| anyhow::anyhow!("Failed to get auto startup status: {}", e))?;
            Ok(status)
        } else {
            Ok(false)
        }
    }

    // Cache statistics
    pub async fn get_cache_statistics(&self) -> Result<CacheStatistics> {
        // Get database size
        let db_path = self.get_db_path()?;
        let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

        // Get images directory size
        let images_path = self.get_images_path()?;
        let images_size = if images_path.exists() {
            self.calculate_directory_size(&images_path)?
        } else {
            0
        };

        // Get entry counts
        let total_entries: i64 = sqlx::query("SELECT COUNT(*) as count FROM clipboard_entries")
            .fetch_one(self.db.pool())
            .await?
            .get("count");

        let text_entries: i64 = sqlx::query(
            "SELECT COUNT(*) as count FROM clipboard_entries WHERE content_type LIKE 'text%'",
        )
        .fetch_one(self.db.pool())
        .await?
        .get("count");

        let image_entries: i64 = sqlx::query(
            "SELECT COUNT(*) as count FROM clipboard_entries WHERE content_type LIKE 'image%'",
        )
        .fetch_one(self.db.pool())
        .await?
        .get("count");

        Ok(CacheStatistics {
            db_size_bytes: db_size,
            images_size_bytes: images_size,
            total_entries,
            text_entries,
            image_entries,
        })
    }

    // Cleanup methods
    pub async fn check_and_cleanup_daily(&self) -> Result<()> {
        let now = Utc::now();
        let mut last_cleanup = self.last_cleanup_date.lock().await;

        let should_cleanup = match *last_cleanup {
            Some(last) => {
                let days_since = (now.date_naive() - last.date_naive()).num_days();
                log::debug!("[Cleanup] 距离上次清理已过 {} 天", days_since);
                days_since >= 1
            }
            None => {
                log::debug!("[Cleanup] 首次运行，需要执行清理");
                true
            }
        };

        if should_cleanup {
            log::info!("[Cleanup] 开始执行每日清理任务");
            let result = self.cleanup_expired_entries().await?;
            log::info!(
                "[Cleanup] 清理完成: 删除 {} 条记录, {} 张图片, 释放 {} 字节空间",
                result.entries_removed,
                result.images_removed,
                result.size_freed_bytes
            );
            *last_cleanup = Some(now);
        } else {
            log::debug!("[Cleanup] 今日已清理过，跳过清理任务");
        }

        Ok(())
    }

    pub async fn cleanup_expired_entries(&self) -> Result<CleanupResult> {
        let config = self.get_config().await?;
        let now = Utc::now().timestamp_millis();

        // Get cutoff times, skip cleanup for Never expiry
        let text_cutoff = match config.text.expiry.as_days() {
            Some(days) => {
                let expiry_ms = (days as i64) * 24 * 60 * 60 * 1000;
                Some(now - expiry_ms)
            }
            None => None, // Never expire
        };

        let image_cutoff = match config.image.expiry.as_days() {
            Some(days) => {
                let expiry_ms = (days as i64) * 24 * 60 * 60 * 1000;
                Some(now - expiry_ms)
            }
            None => None, // Never expire
        };

        // Get entries to remove
        let expired_text_entries = match text_cutoff {
            Some(cutoff) => sqlx::query("SELECT id, file_path FROM clipboard_entries WHERE content_type LIKE 'text%' AND created_at < ?")
                .bind(cutoff)
                .fetch_all(self.db.pool())
                .await?,
            None => vec![], // Never expire text
        };

        let expired_image_entries = match image_cutoff {
            Some(cutoff) => sqlx::query("SELECT id, file_path FROM clipboard_entries WHERE content_type LIKE 'image%' AND created_at < ?")
                .bind(cutoff)
                .fetch_all(self.db.pool())
                .await?,
            None => vec![], // Never expire images
        };

        let mut entries_removed = 0;
        let mut images_removed = 0;
        let mut size_freed = 0u64;

        // Remove text entries
        for row in expired_text_entries {
            let id: String = row.get("id");
            sqlx::query("DELETE FROM clipboard_entries WHERE id = ?")
                .bind(&id)
                .execute(self.db.pool())
                .await?;
            entries_removed += 1;
        }

        // Remove image entries and files
        for row in expired_image_entries {
            let id: String = row.get("id");
            let file_path: Option<String> = row.get("file_path");

            sqlx::query("DELETE FROM clipboard_entries WHERE id = ?")
                .bind(&id)
                .execute(self.db.pool())
                .await?;
            entries_removed += 1;

            // Remove image file if exists
            if let Some(relative_path) = file_path {
                let images_dir = self.get_images_path()?;
                let full_path = images_dir.join(relative_path.replace("imgs/", ""));

                if full_path.exists() {
                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        size_freed += metadata.len();
                    }
                    let _ = std::fs::remove_file(&full_path);
                    images_removed += 1;
                }
            }
        }

        Ok(CleanupResult {
            entries_removed,
            images_removed,
            size_freed_bytes: size_freed,
        })
    }

    // Helper methods
    fn get_db_path(&self) -> Result<PathBuf> {
        Ok(self.paths.history_db_path())
    }

    fn get_images_path(&self) -> Result<PathBuf> {
        Ok(self.paths.image_assets_dir())
    }

    fn calculate_directory_size(&self, path: &PathBuf) -> Result<u64> {
        calculate_directory_size_impl(path)
    }
}

#[cfg(all(test, target_os = "macos"))]
mod macos_paste_permission_tests {
    use super::ensure_macos_accessibility_permission;

    #[test]
    fn accessibility_error_message_is_actionable() {
        if let Err(error) = ensure_macos_accessibility_permission() {
            let message = error.to_string();
            assert!(message.contains("辅助功能"));
            assert!(message.contains("Dance"));
        }
    }
}

fn calculate_directory_size_impl(path: &PathBuf) -> Result<u64> {
    let mut size = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                size += metadata.len();
            } else if metadata.is_dir() {
                size += calculate_directory_size_impl(&entry.path())?;
            }
        }
    }
    Ok(size)
}
