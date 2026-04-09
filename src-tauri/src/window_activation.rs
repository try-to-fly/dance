use anyhow::{anyhow, Result};
use tauri::{
    AppHandle, Manager, Monitor, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow,
    Window,
};

const FALLBACK_WINDOW_WIDTH: u32 = 900;
const FALLBACK_WINDOW_HEIGHT: u32 = 700;

pub fn show_main_window(app: &AppHandle) {
    if let Err(error) = move_window_to_active_monitor_for_label(app, "main") {
        log::warn!("Failed to move main window to active monitor: {}", error);
    }

    let Some(window) = app.get_webview_window("main") else {
        log::error!("Main window not found");
        return;
    };

    if let Err(error) = window.show() {
        log::error!("Failed to show window: {}", error);
    }

    if let Err(error) = window.unminimize() {
        log::error!("Failed to unminimize window: {}", error);
    }

    if let Err(error) = window.set_focus() {
        log::error!("Failed to focus window: {}", error);
    }

    #[cfg(target_os = "macos")]
    pulse_window_to_front(&window);
}

pub fn move_window_to_active_monitor_for_label(app: &AppHandle, label: &str) -> Result<()> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| anyhow!("window '{}' not found", label))?;

    move_window_to_active_monitor(&window)
}

pub fn centered_position_for_active_monitor(
    app: &AppHandle,
    window_size: PhysicalSize<u32>,
) -> Result<PhysicalPosition<i32>> {
    let target_monitor = resolve_active_monitor(app)?;
    Ok(calculate_centered_position(
        target_monitor.work_area(),
        window_size,
    ))
}

pub fn centered_position_for_invoker_window(
    window: &Window,
    window_size: PhysicalSize<u32>,
) -> Result<PhysicalPosition<i32>> {
    let target_monitor = resolve_active_monitor_from_window(window)?;
    Ok(calculate_centered_position(
        target_monitor.work_area(),
        window_size,
    ))
}

pub fn move_window_to_invoker_monitor(
    app: &AppHandle,
    invoker_window: &Window,
    target_label: &str,
) -> Result<()> {
    let target_window = app
        .get_webview_window(target_label)
        .ok_or_else(|| anyhow!("window '{}' not found", target_label))?;

    let window_size = target_window.outer_size().unwrap_or_else(|error| {
        log::warn!(
            "Failed to read window outer size, using fallback size {}x{}: {}",
            FALLBACK_WINDOW_WIDTH,
            FALLBACK_WINDOW_HEIGHT,
            error
        );
        fallback_window_size()
    });

    let target_monitor = resolve_active_monitor_from_window(invoker_window)?;
    let next_position = calculate_centered_position(target_monitor.work_area(), window_size);

    log::info!(
        "Moving window '{}' to invoker monitor {:?} at ({}, {})",
        target_label,
        target_monitor.name(),
        next_position.x,
        next_position.y
    );

    target_window
        .set_position(next_position)
        .map_err(|error| anyhow!("failed to set window position: {}", error))
}

fn move_window_to_active_monitor(window: &WebviewWindow) -> Result<()> {
    let window_size = window.outer_size().unwrap_or_else(|error| {
        log::warn!(
            "Failed to read window outer size, using fallback size {}x{}: {}",
            FALLBACK_WINDOW_WIDTH,
            FALLBACK_WINDOW_HEIGHT,
            error
        );
        fallback_window_size()
    });

    let next_position = centered_position_for_active_monitor(window.app_handle(), window_size)?;

    log::info!(
        "Moving window '{}' to monitor {:?} at ({}, {})",
        window.label(),
        resolve_active_monitor(window.app_handle())?.name(),
        next_position.x,
        next_position.y
    );

    window
        .set_position(next_position)
        .map_err(|error| anyhow!("failed to set window position: {}", error))
}

fn fallback_window_size() -> PhysicalSize<u32> {
    PhysicalSize::new(FALLBACK_WINDOW_WIDTH, FALLBACK_WINDOW_HEIGHT)
}

fn resolve_active_monitor(app: &AppHandle) -> Result<Monitor> {
    let monitor_from_cursor = match app.cursor_position() {
        Ok(cursor_position) => app
            .monitor_from_point(cursor_position.x, cursor_position.y)
            .map_err(|error| {
                anyhow!("failed to resolve monitor from cursor position: {}", error)
            })?,
        Err(error) => {
            log::warn!(
                "Failed to read cursor position, falling back to primary monitor: {}",
                error
            );
            None
        }
    };

    monitor_from_cursor
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or_else(|| anyhow!("no monitor available for positioning"))
}

fn resolve_active_monitor_from_window(window: &Window) -> Result<Monitor> {
    let monitor_from_cursor = match window.cursor_position() {
        Ok(cursor_position) => window
            .monitor_from_point(cursor_position.x, cursor_position.y)
            .map_err(|error| {
                anyhow!("failed to resolve monitor from cursor position: {}", error)
            })?,
        Err(error) => {
            log::warn!(
                "Failed to read cursor position from window '{}', falling back to current monitor: {}",
                window.label(),
                error
            );
            None
        }
    };

    monitor_from_cursor
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| anyhow!("no monitor available for positioning"))
}

fn calculate_centered_position(
    work_area: &PhysicalRect<i32, u32>,
    window_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    let horizontal_space = work_area.size.width.saturating_sub(window_size.width) as i32;
    let vertical_space = work_area.size.height.saturating_sub(window_size.height) as i32;

    PhysicalPosition::new(
        work_area.position.x + horizontal_space / 2,
        work_area.position.y + vertical_space / 2,
    )
}

#[cfg(target_os = "macos")]
fn pulse_window_to_front(window: &WebviewWindow) {
    if window.set_always_on_top(true).is_err() {
        return;
    }

    let window_clone = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        let _ = window_clone.set_always_on_top(false);
    });
}

#[cfg(test)]
mod tests {
    use super::calculate_centered_position;
    use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize};

    #[test]
    fn centers_window_inside_work_area() {
        let work_area = PhysicalRect {
            position: PhysicalPosition::new(100, 50),
            size: PhysicalSize::new(1600, 900),
        };

        let position = calculate_centered_position(&work_area, PhysicalSize::new(900, 700));

        assert_eq!(position, PhysicalPosition::new(450, 150));
    }

    #[test]
    fn clamps_to_work_area_origin_when_window_is_larger() {
        let work_area = PhysicalRect {
            position: PhysicalPosition::new(-1728, 23),
            size: PhysicalSize::new(1512, 945),
        };

        let position = calculate_centered_position(&work_area, PhysicalSize::new(1800, 1100));

        assert_eq!(position, PhysicalPosition::new(-1728, 23));
    }
}
