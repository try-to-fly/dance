use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use dance_lib::headless::{ClipboardEntry, ClipboardHistoryQuery, HeadlessApp};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::cap_parser::QueryStdioOptions;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};
use std::collections::{HashSet, VecDeque};
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

const HISTORY_LIMIT: i32 = 100;
const DAEMON_TICK_MS: u64 = 500;
const INPUT_POLL_MS: u64 = 30;
const AUTO_REFRESH_MS: u64 = 750;
const SEARCH_DEBOUNCE_MS: u64 = 180;
const IMAGE_PREVIEW_DEBOUNCE_MS: u64 = 0;
const IMAGE_PICKER_QUERY_TIMEOUT_MS: u64 = 120;
const IMAGE_PROTOCOL_CACHE_LIMIT: usize = 8;
const APP_ICON_PROTOCOL_WIDTH: u16 = 2;
const APP_ICON_PROTOCOL_HEIGHT: u16 = 1;
const APP_ICON_PROTOCOL_CACHE_LIMIT: usize = 48;
const ATTRIBUTE_PANEL_MAX_HEIGHT: u16 = 10;
const ATTRIBUTE_PANEL_MIN_CONTENT_HEIGHT: u16 = 4;

#[derive(Parser)]
#[command(name = "dance-tui", about = "Dance clipboard TUI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Daemon,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if matches!(cli.command, Some(Command::Daemon)) {
        init_cli_logger();
    }

    let app = HeadlessApp::new_default().await?;

    match cli.command {
        Some(Command::Daemon) => run_daemon(app).await,
        None => run_tui(app).await,
    }
}

struct CliLogger;

impl log::Log for CliLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "[{}] {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static CLI_LOGGER: CliLogger = CliLogger;

fn init_cli_logger() {
    let level = match std::env::var("DANCE_TUI_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase()
        .as_str()
    {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        "off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    };

    if log::set_logger(&CLI_LOGGER).is_ok() {
        log::set_max_level(level);
    }
}

async fn run_daemon(app: HeadlessApp) -> Result<()> {
    println!("dance-tui daemon started for {}", app.owner());
    log::info!("[dance-tui daemon] 已启动，owner={}", app.owner());
    let mut last_state: Option<&'static str> = None;

    loop {
        if app.is_tauri_active() {
            if app.is_capture_running().await {
                app.stop_capture().await?;
            }
            app.write_daemon_status("paused_by_tauri")?;
            if last_state != Some("paused_by_tauri") {
                log::info!("[dance-tui daemon] Tauri 主应用活跃，暂停 daemon 监听");
                last_state = Some("paused_by_tauri");
            }
        } else {
            if !app.is_capture_running().await {
                app.start_capture().await?;
            }
            app.write_daemon_status("listening")?;
            if last_state != Some("listening") {
                log::info!("[dance-tui daemon] daemon 正在监听剪贴板");
                last_state = Some("listening");
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                app.stop_capture().await?;
                app.write_daemon_status("stopped")?;
                log::info!("[dance-tui daemon] 已停止");
                println!("dance-tui daemon stopped");
                return Ok(());
            }
            _ = tokio::time::sleep(Duration::from_millis(DAEMON_TICK_MS)) => {}
        }
    }
}

async fn run_tui(app: HeadlessApp) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut state = TuiState::new(app);
    state.refresh_entries().await?;

    let result = run_tui_loop(&mut terminal, &mut state).await;
    restore_terminal(&mut terminal)?;
    result
}

async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    let mut pending_refresh_at: Option<Instant> = None;
    let mut next_auto_refresh_at: Option<Instant> =
        Some(Instant::now() + Duration::from_millis(AUTO_REFRESH_MS));
    let mut search_job: Option<SearchJob> = None;
    let mut pending_image_refresh_at: Option<Instant> = Some(Instant::now());
    let mut image_job: Option<ImageJob> = None;
    let mut icon_job: Option<IconJob> = None;

    loop {
        terminal.draw(|frame| draw(frame, state))?;

        if state.take_image_refresh_requested() {
            schedule_image_refresh(state, &mut image_job, &mut pending_image_refresh_at);
        }

        if let Some(job) = search_job.as_ref() {
            if job.handle.is_finished() {
                let job = search_job.take().expect("search job disappeared");
                state.searching = false;
                match job.handle.await {
                    Ok((text, result)) => {
                        if state.apply_search_result(&text, result) {
                            schedule_image_refresh(
                                state,
                                &mut image_job,
                                &mut pending_image_refresh_at,
                            );
                        }
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        state.error = Some(format!("检索任务失败: {}", error));
                    }
                }
                next_auto_refresh_at =
                    Some(Instant::now() + Duration::from_millis(AUTO_REFRESH_MS));
            }
        }

        if pending_refresh_at.is_some_and(|deadline| Instant::now() >= deadline) {
            pending_refresh_at = None;
            start_search_job(state, &mut search_job, true);
            next_auto_refresh_at = None;
        }

        if next_auto_refresh_at.is_some_and(|deadline| Instant::now() >= deadline) {
            next_auto_refresh_at = None;
            if search_job.is_none() && pending_refresh_at.is_none() {
                start_search_job(state, &mut search_job, false);
            } else {
                next_auto_refresh_at =
                    Some(Instant::now() + Duration::from_millis(AUTO_REFRESH_MS));
            }
        }

        if let Some(job) = image_job.as_ref() {
            if job.handle.is_finished() {
                let job = image_job.take().expect("image job disappeared");
                state.image_loading = false;
                match job.handle.await {
                    Ok((path, size, result)) => {
                        state.apply_image_result(path, size, result);
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        state.error = Some(format!("图片预览任务失败: {}", error));
                    }
                }
                schedule_image_refresh(state, &mut image_job, &mut pending_image_refresh_at);
            }
        }

        if pending_image_refresh_at.is_some_and(|deadline| Instant::now() >= deadline) {
            pending_image_refresh_at = None;
            start_image_job(state, &mut image_job);
        }

        if let Some(job) = icon_job.as_ref() {
            if job.handle.is_finished() {
                let job = icon_job.take().expect("icon job disappeared");
                match job.handle.await {
                    Ok((bundle_id, result)) => {
                        state.apply_icon_result(bundle_id, result);
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        state.error = Some(format!("应用图标任务失败: {}", error));
                    }
                }
            }
        }

        start_icon_job(state, &mut icon_job);

        let poll_timeout = next_deadline_duration(&[
            pending_refresh_at,
            pending_image_refresh_at,
            next_auto_refresh_at,
        ])
        .unwrap_or_else(|| Duration::from_millis(INPUT_POLL_MS));
        if !event::poll(poll_timeout)? {
            continue;
        }

        let event = event::read()?;
        let Event::Key(key) = event else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if is_copy_shortcut(&key) {
            state.copy_selected()?;
            continue;
        }

        if is_open_shortcut(&key) {
            state.open_selected()?;
            continue;
        }

        if is_quit_shortcut(&key) {
            abort_search_job(state, &mut search_job);
            abort_image_job(state, &mut image_job);
            abort_icon_job(&mut icon_job);
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') => {
                abort_search_job(state, &mut search_job);
                abort_image_job(state, &mut image_job);
                abort_icon_job(&mut icon_job);
                return Ok(());
            }
            KeyCode::Esc => {
                state.input = Input::default();
                state.preview_scroll = 0;
                pending_refresh_at = None;
                start_search_job(state, &mut search_job, true);
                next_auto_refresh_at = None;
            }
            KeyCode::Up => {
                state.select_previous();
                schedule_image_refresh(state, &mut image_job, &mut pending_image_refresh_at);
            }
            KeyCode::Down => {
                state.select_next();
                schedule_image_refresh(state, &mut image_job, &mut pending_image_refresh_at);
            }
            KeyCode::PageUp => {
                state.preview_scroll = state.preview_scroll.saturating_sub(8);
            }
            KeyCode::PageDown => {
                state.preview_scroll = state.preview_scroll.saturating_add(8);
            }
            _ => {
                let before = state.input.value().to_string();
                state.input.handle_event(&Event::Key(key));
                if state.input.value() != before {
                    state.preview_scroll = 0;
                    abort_search_job(state, &mut search_job);
                    pending_refresh_at =
                        Some(Instant::now() + Duration::from_millis(SEARCH_DEBOUNCE_MS));
                    next_auto_refresh_at = None;
                }
            }
        }
    }
}

fn is_copy_shortcut(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(value) if value.eq_ignore_ascii_case(&'c'))
}

fn is_open_shortcut(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char(value) if value.eq_ignore_ascii_case(&'o')))
}

fn is_quit_shortcut(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(value) if value.eq_ignore_ascii_case(&'q'))
}

fn next_deadline_duration(deadlines: &[Option<Instant>]) -> Option<Duration> {
    deadlines
        .iter()
        .copied()
        .flatten()
        .min()
        .map(|deadline| deadline.saturating_duration_since(Instant::now()))
}

fn start_search_job(state: &mut TuiState, search_job: &mut Option<SearchJob>, show_status: bool) {
    abort_search_job(state, search_job);
    *search_job = Some(state.spawn_search_job());
    state.searching = show_status;
}

fn abort_search_job(state: &mut TuiState, search_job: &mut Option<SearchJob>) {
    if let Some(job) = search_job.take() {
        job.handle.abort();
        state.searching = false;
    }
}

struct SearchJob {
    handle: JoinHandle<(String, std::result::Result<Vec<ClipboardEntry>, String>)>,
}

fn schedule_image_refresh(
    state: &mut TuiState,
    image_job: &mut Option<ImageJob>,
    pending_image_refresh_at: &mut Option<Instant>,
) {
    let Some(path) = state.selected_image_path() else {
        state.image_protocol = None;
        *pending_image_refresh_at = None;
        return;
    };
    let Some(size) = state.image_target_size else {
        state.image_protocol = None;
        *pending_image_refresh_at = None;
        return;
    };

    if state
        .image_protocol
        .as_ref()
        .is_some_and(|protocol| protocol.path == path && protocol.size == size)
    {
        *pending_image_refresh_at = None;
        return;
    }

    if state.apply_cached_image_protocol(&path, size) {
        *pending_image_refresh_at = None;
        return;
    }

    state.image_protocol = None;
    state.image_loading = state.image_picker.is_some();
    if image_job.is_some() {
        *pending_image_refresh_at = None;
        return;
    }

    *pending_image_refresh_at =
        Some(Instant::now() + Duration::from_millis(IMAGE_PREVIEW_DEBOUNCE_MS));
}

fn start_image_job(state: &mut TuiState, image_job: &mut Option<ImageJob>) {
    if image_job.is_some() {
        state.image_loading = state.image_picker.is_some();
        return;
    }

    let Some(path) = state.selected_image_path() else {
        state.image_protocol = None;
        state.image_loading = false;
        return;
    };
    let Some(size) = state.image_target_size else {
        state.image_protocol = None;
        state.image_loading = false;
        return;
    };

    if state
        .image_protocol
        .as_ref()
        .is_some_and(|protocol| protocol.path == path && protocol.size == size)
        || state.apply_cached_image_protocol(&path, size)
    {
        state.image_loading = false;
        return;
    }

    let Some(picker) = state.image_picker.clone() else {
        state.image_protocol = None;
        state.image_loading = false;
        return;
    };

    *image_job = Some(ImageJob {
        handle: tokio::task::spawn_blocking(move || load_image_protocol(picker, path, size)),
    });
    state.image_loading = true;
}

fn abort_image_job(state: &mut TuiState, image_job: &mut Option<ImageJob>) {
    if let Some(job) = image_job.take() {
        job.handle.abort();
        state.image_loading = false;
    }
}

struct ImageJob {
    handle: JoinHandle<(PathBuf, Size, std::result::Result<Protocol, String>)>,
}

fn start_icon_job(state: &mut TuiState, icon_job: &mut Option<IconJob>) {
    if icon_job.is_some() {
        return;
    }

    let Some(bundle_id) = state.next_uncached_icon_bundle_id() else {
        return;
    };
    let Some(picker) = state.image_picker.clone() else {
        return;
    };

    let app = state.app.clone();
    let job_bundle_id = bundle_id.clone();
    *icon_job = Some(IconJob {
        handle: tokio::task::spawn_blocking(move || {
            load_app_icon_protocol(app, picker, job_bundle_id)
        }),
    });
}

fn abort_icon_job(icon_job: &mut Option<IconJob>) {
    if let Some(job) = icon_job.take() {
        job.handle.abort();
    }
}

struct IconJob {
    handle: JoinHandle<(String, std::result::Result<Option<Protocol>, String>)>,
}

struct TuiState {
    app: HeadlessApp,
    input: Input,
    entries: Vec<ClipboardEntry>,
    selected: usize,
    list_scroll: usize,
    preview_scroll: u16,
    image_target_size: Option<Size>,
    image_refresh_requested: bool,
    image_picker: Option<Picker>,
    image_protocol: Option<ImageProtocolState>,
    image_protocol_cache: VecDeque<ImageProtocolState>,
    icon_protocol_cache: VecDeque<IconProtocolState>,
    failed_icon_bundle_ids: HashSet<String>,
    image_loading: bool,
    error: Option<String>,
    notice: Option<String>,
    searching: bool,
}

struct ImageProtocolState {
    path: PathBuf,
    size: Size,
    protocol: Protocol,
}

struct IconProtocolState {
    bundle_id: String,
    protocol: Protocol,
}

enum OpenTarget {
    File(PathBuf),
    Url(String),
}

enum CopyItem {
    Text(String),
    Image(PathBuf),
}

impl TuiState {
    fn new(app: HeadlessApp) -> Self {
        Self {
            app,
            input: Input::default(),
            entries: Vec::new(),
            selected: 0,
            list_scroll: 0,
            preview_scroll: 0,
            image_target_size: None,
            image_refresh_requested: false,
            image_picker: build_image_picker(),
            image_protocol: None,
            image_protocol_cache: VecDeque::new(),
            icon_protocol_cache: VecDeque::new(),
            failed_icon_bundle_ids: HashSet::new(),
            image_loading: false,
            error: None,
            notice: None,
            searching: false,
        }
    }

    async fn refresh_entries(&mut self) -> Result<()> {
        let text = self.search_text();
        let result = search_entries(self.app.clone(), text.clone()).await.1;
        self.apply_search_result(&text, result);
        Ok(())
    }

    fn spawn_search_job(&self) -> SearchJob {
        let app = self.app.clone();
        let text = self.search_text();
        let handle = tokio::spawn(search_entries(app, text));
        SearchJob { handle }
    }

    fn search_text(&self) -> String {
        self.input.value().trim().to_string()
    }

    fn entry_signature(entry: &ClipboardEntry) -> (&str, i64, i32, bool) {
        (
            entry.id.as_str(),
            entry.created_at,
            entry.copy_count,
            entry.is_favorite,
        )
    }

    fn apply_search_result(
        &mut self,
        text: &str,
        result: std::result::Result<Vec<ClipboardEntry>, String>,
    ) -> bool {
        if text != self.search_text() {
            return false;
        }

        match result {
            Ok(entries) => {
                let previous_selected_id = self.selected_entry().map(|entry| entry.id.clone());
                let entries_changed = self.entries.len() != entries.len()
                    || self
                        .entries
                        .iter()
                        .zip(entries.iter())
                        .any(|(left, right)| {
                            Self::entry_signature(left) != Self::entry_signature(right)
                        });

                self.entries = entries;
                if self.selected >= self.entries.len() {
                    self.selected = self.entries.len().saturating_sub(1);
                }
                if self.list_scroll > self.selected {
                    self.list_scroll = self.selected;
                }
                let selected_changed =
                    previous_selected_id != self.selected_entry().map(|entry| entry.id.clone());
                self.error = None;
                self.notice = None;
                entries_changed || selected_changed
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    fn selected_entry(&self) -> Option<&ClipboardEntry> {
        self.entries.get(self.selected)
    }

    fn next_uncached_icon_bundle_id(&self) -> Option<String> {
        self.entries.iter().find_map(|entry| {
            let bundle_id = entry.app_bundle_id.as_deref()?.trim();
            if bundle_id.is_empty()
                || self.failed_icon_bundle_ids.contains(bundle_id)
                || self
                    .icon_protocol_cache
                    .iter()
                    .any(|state| state.bundle_id == bundle_id)
            {
                return None;
            }

            Some(bundle_id.to_string())
        })
    }

    fn apply_icon_result(
        &mut self,
        bundle_id: String,
        result: std::result::Result<Option<Protocol>, String>,
    ) {
        match result {
            Ok(Some(protocol)) => {
                self.icon_protocol_cache
                    .retain(|state| state.bundle_id != bundle_id);
                self.icon_protocol_cache.push_back(IconProtocolState {
                    bundle_id,
                    protocol,
                });
                while self.icon_protocol_cache.len() > APP_ICON_PROTOCOL_CACHE_LIMIT {
                    self.icon_protocol_cache.pop_front();
                }
            }
            Ok(None) => {
                self.failed_icon_bundle_ids.insert(bundle_id);
            }
            Err(_) => {
                self.failed_icon_bundle_ids.insert(bundle_id);
            }
        }
    }

    fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.preview_scroll = 0;
    }

    fn select_next(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
        self.preview_scroll = 0;
    }

    fn apply_image_result(
        &mut self,
        path: PathBuf,
        size: Size,
        result: std::result::Result<Protocol, String>,
    ) {
        if self.selected_image_path().as_ref() != Some(&path)
            || self.image_target_size != Some(size)
        {
            return;
        }

        match result {
            Ok(protocol) => {
                self.store_image_protocol(path, size, protocol);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
                self.image_protocol = None;
            }
        }
    }

    fn apply_cached_image_protocol(&mut self, path: &Path, size: Size) -> bool {
        let Some(index) = self
            .image_protocol_cache
            .iter()
            .position(|state| state.path == path && state.size == size)
        else {
            return false;
        };

        let Some(cached) = self.image_protocol_cache.remove(index) else {
            return false;
        };
        let protocol = cached.protocol.clone();
        self.image_protocol_cache.push_back(cached);
        self.image_protocol = Some(ImageProtocolState {
            path: path.to_path_buf(),
            size,
            protocol,
        });
        self.image_loading = false;
        true
    }

    fn store_image_protocol(&mut self, path: PathBuf, size: Size, protocol: Protocol) {
        self.image_protocol_cache
            .retain(|state| state.path != path || state.size != size);
        self.image_protocol_cache.push_back(ImageProtocolState {
            path: path.clone(),
            size,
            protocol: protocol.clone(),
        });
        while self.image_protocol_cache.len() > IMAGE_PROTOCOL_CACHE_LIMIT {
            self.image_protocol_cache.pop_front();
        }
        self.image_protocol = Some(ImageProtocolState {
            path,
            size,
            protocol,
        });
        self.image_loading = false;
    }

    fn update_image_target_size(&mut self, size: Size) {
        let size = if size.width == 0 || size.height == 0 {
            None
        } else {
            Some(size)
        };

        if self.image_target_size == size {
            return;
        }

        self.image_target_size = size;

        let Some(path) = self.selected_image_path() else {
            self.image_protocol = None;
            return;
        };

        let Some(size) = self.image_target_size else {
            self.image_protocol = None;
            return;
        };

        if self
            .image_protocol
            .as_ref()
            .is_some_and(|protocol| protocol.path == path && protocol.size == size)
        {
            return;
        }

        self.image_protocol = None;
        self.image_loading = self.image_picker.is_some();
        self.image_refresh_requested = true;
    }

    fn take_image_refresh_requested(&mut self) -> bool {
        let requested = self.image_refresh_requested;
        self.image_refresh_requested = false;
        requested
    }

    fn selected_image_path(&self) -> Option<PathBuf> {
        let entry = self.selected_entry()?;
        let path = if entry.content_type.contains("image") {
            entry
                .file_path
                .as_deref()
                .or(entry.content_data.as_deref())?
        } else if entry.content_type == "file" {
            let file_path = single_file_path(entry)?;
            if !is_image_path(Path::new(file_path)) {
                return None;
            }
            file_path
        } else {
            return None;
        };

        self.app.resolve_file_path(path).ok()
    }

    fn open_selected(&mut self) -> Result<()> {
        let Some(entry) = self.selected_entry() else {
            self.notice = Some("没有可打开的剪贴板记录".to_string());
            return Ok(());
        };
        let Some(target) = open_target(entry, &self.app)? else {
            self.notice = Some("当前条目不支持系统打开".to_string());
            return Ok(());
        };
        match target {
            OpenTarget::File(path) => {
                opener::open(&path).with_context(|| format!("无法打开 {}", path.display()))?;
            }
            OpenTarget::Url(url) => {
                opener::open(&url).with_context(|| format!("无法打开 {}", url))?;
            }
        }
        self.error = None;
        self.notice = Some("已交给系统应用打开".to_string());
        Ok(())
    }

    fn copy_selected(&mut self) -> Result<()> {
        let Some(entry) = self.selected_entry() else {
            self.notice = Some("没有可复制的剪贴板记录".to_string());
            return Ok(());
        };
        let Some(item) = copy_item(entry, &self.app)? else {
            self.notice = Some("当前条目没有可复制内容".to_string());
            return Ok(());
        };
        match item {
            CopyItem::Text(content) => {
                let mut clipboard = arboard::Clipboard::new()?;
                clipboard.set_text(content)?;
                self.notice = Some("已复制文本".to_string());
            }
            CopyItem::Image(path) => {
                copy_image_file_to_clipboard(&path)?;
                self.notice = Some("已复制图片".to_string());
            }
        }
        self.error = None;
        Ok(())
    }

    fn ensure_selected_visible(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            self.list_scroll = 0;
            return;
        }

        if self.selected < self.list_scroll {
            self.list_scroll = self.selected;
        } else if self.selected >= self.list_scroll + visible_rows {
            self.list_scroll = self.selected + 1 - visible_rows;
        }
    }

    fn icon_protocol_for(&self, bundle_id: &str) -> Option<&Protocol> {
        self.icon_protocol_cache
            .iter()
            .find(|state| state.bundle_id == bundle_id)
            .map(|state| &state.protocol)
    }
}

fn draw(frame: &mut Frame<'_>, state: &mut TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(frame.area());

    draw_left(frame, chunks[0], state);
    draw_preview(frame, chunks[1], state);
}

fn draw_left(frame: &mut Frame<'_>, area: Rect, state: &mut TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let input = Paragraph::new(state.input.value())
        .block(Block::default().title("搜索").borders(Borders::ALL));
    frame.render_widget(input, chunks[0]);

    draw_history_list(frame, chunks[1], state);

    let daemon_state = state
        .app
        .read_daemon_status()
        .map(|status| status.state)
        .unwrap_or_else(|| "daemon_unknown".to_string());
    let action_hint = "↑/↓ 选择 · Enter 打开 · Ctrl+C 复制 · Esc 清空搜索 · q 退出";
    let status = if let Some(error) = state.error.as_deref() {
        format!("{} | {}", daemon_state, error)
    } else if let Some(notice) = state.notice.as_deref() {
        format!("{} | {} | {}", daemon_state, notice, action_hint)
    } else if state.searching {
        format!("{} | 检索中... | {}", daemon_state, action_hint)
    } else {
        format!("{} | {}", daemon_state, action_hint)
    };
    frame.render_widget(Paragraph::new(status), chunks[2]);
}

fn draw_history_list(frame: &mut Frame<'_>, area: Rect, state: &mut TuiState) {
    let block = Block::default().title("历史").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_rows = usize::from(inner.height / 2);
    state.ensure_selected_visible(visible_rows);

    if state.entries.is_empty() || visible_rows == 0 {
        return;
    }

    for (row_index, entry_index) in (state.list_scroll..state.entries.len())
        .take(visible_rows)
        .enumerate()
    {
        let entry = &state.entries[entry_index];
        let is_selected = entry_index == state.selected;
        let y = inner.y + (row_index as u16 * 2);
        let title_area = Rect::new(inner.x, y, inner.width, 1);
        let meta_area = Rect::new(inner.x, y.saturating_add(1), inner.width, 1);
        let row_style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        let meta_style = if is_selected {
            row_style
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let title_prefix = if is_selected { "> " } else { "  " };
        let title = format!("{}{}", title_prefix, entry_title(entry));
        let meta_prefix = "  ";
        let meta_type = entry
            .content_subtype
            .as_deref()
            .unwrap_or(entry.content_type.as_str());
        let source_name = source_app_name(entry);
        let meta = format!(
            "{}{} ·   {} · {}",
            meta_prefix,
            meta_type,
            source_name,
            format_time(entry.created_at)
        );

        frame.render_widget(Paragraph::new(title).style(row_style), title_area);
        frame.render_widget(Paragraph::new(meta).style(meta_style), meta_area);

        let icon_x = inner.x.saturating_add(
            (meta_prefix.chars().count() + meta_type.chars().count() + " · ".chars().count())
                as u16,
        );
        if icon_x.saturating_add(APP_ICON_PROTOCOL_WIDTH) <= inner.x.saturating_add(inner.width) {
            if let Some(bundle_id) = entry.app_bundle_id.as_deref() {
                if let Some(protocol) = state.icon_protocol_for(bundle_id) {
                    let icon_area = Rect::new(
                        icon_x,
                        meta_area.y,
                        APP_ICON_PROTOCOL_WIDTH,
                        APP_ICON_PROTOCOL_HEIGHT,
                    );
                    frame.render_widget(Image::new(protocol), icon_area);
                }
            }
        }
    }
}

async fn search_entries(
    app: HeadlessApp,
    text: String,
) -> (String, std::result::Result<Vec<ClipboardEntry>, String>) {
    let query = ClipboardHistoryQuery {
        text: (!text.is_empty()).then(|| text.clone()),
        limit: Some(HISTORY_LIMIT),
        offset: Some(0),
        ..Default::default()
    };
    let result = app
        .search_clipboard_history(query)
        .await
        .map_err(|error| error.to_string());
    (text, result)
}

fn load_image_protocol(
    picker: Picker,
    path: PathBuf,
    size: Size,
) -> (PathBuf, Size, std::result::Result<Protocol, String>) {
    let result = load_image_protocol_result(&picker, &path, size);
    (path, size, result)
}

fn load_image_protocol_result(
    picker: &Picker,
    path: &Path,
    size: Size,
) -> std::result::Result<Protocol, String> {
    load_protocol_result(picker, path, size)
}

fn load_protocol_result(
    picker: &Picker,
    path: &Path,
    size: Size,
) -> std::result::Result<Protocol, String> {
    let reader =
        image::ImageReader::open(path).map_err(|error| format!("图片读取失败: {}", error))?;
    let image = reader
        .decode()
        .map_err(|error| format!("图片解码失败: {}", error))?;

    picker
        .new_protocol(image, size, Resize::Fit(None))
        .map_err(|error| format!("图片协议初始化失败: {}", error))
}

fn load_app_icon_protocol(
    app: HeadlessApp,
    picker: Picker,
    bundle_id: String,
) -> (String, std::result::Result<Option<Protocol>, String>) {
    let result = app
        .app_icon_path(&bundle_id)
        .map_err(|error| format!("应用图标读取失败: {}", error))
        .and_then(|path| {
            path.map(|path| {
                load_protocol_result(
                    &picker,
                    &path,
                    Size::new(APP_ICON_PROTOCOL_WIDTH, APP_ICON_PROTOCOL_HEIGHT),
                )
            })
            .transpose()
        });
    (bundle_id, result)
}

fn draw_preview(frame: &mut Frame<'_>, area: Rect, state: &mut TuiState) {
    let block = Block::default().title("预览").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(entry) = state.selected_entry().cloned() else {
        frame.render_widget(Paragraph::new("没有剪切板记录"), inner);
        return;
    };

    let attribute_lines = attribute_lines(&entry, inner.width.saturating_sub(2));
    let (attribute_area, content_area) = split_preview_sections(inner, attribute_lines.len());
    state.update_image_target_size(Size::new(content_area.width, content_area.height));

    if let Some(attribute_area) = attribute_area {
        let block = Block::default().title("属性").borders(Borders::ALL);
        let inner = block.inner(attribute_area);
        frame.render_widget(block, attribute_area);
        let paragraph = Paragraph::new(attribute_lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    if let Some(image_state) = state.image_protocol.as_ref() {
        frame.render_widget(Clear, content_area);
        frame.render_widget(Image::new(&image_state.protocol), content_area);
        return;
    }

    if state.image_loading && state.selected_image_path().is_some() {
        frame.render_widget(Paragraph::new("图片预览加载中..."), content_area);
        return;
    }

    let lines = preview_lines(&entry);
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.preview_scroll, 0));
    frame.render_widget(paragraph, content_area);
}

fn split_preview_sections(area: Rect, attribute_line_count: usize) -> (Option<Rect>, Rect) {
    if attribute_line_count == 0
        || area.height < ATTRIBUTE_PANEL_MIN_CONTENT_HEIGHT.saturating_add(3)
    {
        return (None, area);
    }

    let wanted_attribute_height = (attribute_line_count as u16).saturating_add(2);
    let max_attribute_height = area
        .height
        .saturating_sub(ATTRIBUTE_PANEL_MIN_CONTENT_HEIGHT)
        .min(ATTRIBUTE_PANEL_MAX_HEIGHT);
    let attribute_height = wanted_attribute_height.min(max_attribute_height);

    if attribute_height < 3 {
        return (None, area);
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(attribute_height),
            Constraint::Min(ATTRIBUTE_PANEL_MIN_CONTENT_HEIGHT),
        ])
        .split(area);

    (Some(chunks[0]), chunks[1])
}

fn preview_lines(entry: &ClipboardEntry) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if entry.content_subtype.as_deref() == Some("json") {
        if let Some(content) = entry.content_data.as_deref() {
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(value) => push_multiline(
                    &mut lines,
                    &serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string()),
                ),
                Err(error) => {
                    lines.push(Line::from(format!("JSON 解析失败: {}", error)));
                    push_multiline(&mut lines, content);
                }
            }
        }
        return lines;
    }

    if entry.content_type == "file" {
        push_file_preview(&mut lines, entry);
        return lines;
    }

    if entry.content_type.contains("image") {
        lines.push(Line::from(
            "当前终端不支持 kitty 图片预览，或图片加载失败。",
        ));
        if let Some(path) = entry.file_path.as_deref().or(entry.content_data.as_deref()) {
            lines.push(Line::from(path.to_string()));
        }
        return lines;
    }

    if let Some(content) = entry.content_data.as_deref() {
        push_multiline(&mut lines, content);
    }

    lines
}

fn push_file_preview(lines: &mut Vec<Line<'static>>, entry: &ClipboardEntry) {
    if let Some(content) = entry.content_data.as_deref() {
        lines.push(Line::from("paths:"));
        push_multiline(lines, content);
    }
}

fn attribute_lines(entry: &ClipboardEntry, width: u16) -> Vec<Line<'static>> {
    build_attribute_text_lines(entry, width)
        .into_iter()
        .map(Line::from)
        .collect()
}

fn build_attribute_text_lines(entry: &ClipboardEntry, width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let metadata = parse_metadata_value(entry);

    if entry.content_type.contains("image") {
        append_image_attributes(&mut lines, entry, metadata.as_ref(), width);
        return lines;
    }

    if entry.content_type == "file" {
        append_file_attributes(&mut lines, entry, metadata.as_ref(), width);
        return lines;
    }

    let Some(metadata) = metadata.as_ref() else {
        append_content_fallback_attributes(&mut lines, entry, width);
        return lines;
    };

    match entry.content_subtype.as_deref() {
        Some("url") => append_url_attributes(&mut lines, metadata, width),
        Some("color") => append_color_attributes(&mut lines, metadata, width),
        Some("base64") => append_base64_attributes(&mut lines, metadata, width),
        Some("timestamp") => append_timestamp_attributes(&mut lines, metadata, width),
        Some("code") => append_code_attributes(&mut lines, metadata, width),
        Some("command") => append_command_attributes(&mut lines, metadata, width),
        Some("json") => append_json_attributes(&mut lines, metadata, width),
        Some("markdown") => append_markdown_attributes(&mut lines, metadata, width),
        Some("email") => append_email_attributes(&mut lines, metadata, width),
        Some("ip_address") => append_ip_attributes(&mut lines, metadata, width),
        Some("plain_text") | None | Some("") => {
            append_content_fallback_attributes(&mut lines, entry, width)
        }
        _ => append_text_metadata_attributes(&mut lines, metadata, width),
    }

    lines
}

fn parse_metadata_value(entry: &ClipboardEntry) -> Option<serde_json::Value> {
    entry
        .metadata
        .as_deref()
        .and_then(|metadata| serde_json::from_str::<serde_json::Value>(metadata).ok())
}

fn append_image_attributes(
    lines: &mut Vec<String>,
    entry: &ClipboardEntry,
    metadata: Option<&serde_json::Value>,
    available_width: u16,
) {
    let image_metadata = metadata.and_then(|value| value.get("image_metadata"));
    let image_width = image_metadata.and_then(|value| json_u64(value, "width"));
    let height = image_metadata.and_then(|value| json_u64(value, "height"));
    let path = entry.file_path.as_deref().or(entry.content_data.as_deref());

    push_compact_fields(
        lines,
        [
            ("文件名", path.and_then(file_name).map(str::to_string)),
            ("尺寸", format_dimensions(image_width, height)),
            (
                "格式",
                image_metadata
                    .and_then(|value| json_string(value, "format"))
                    .or_else(|| path.and_then(path_extension))
                    .map(|value| value.to_uppercase()),
            ),
            (
                "大小",
                image_metadata
                    .and_then(|value| json_u64(value, "file_size"))
                    .map(format_binary_size),
            ),
        ],
        available_width,
    );
    push_long_field(lines, "路径", path.map(str::to_string));
}

fn append_file_attributes(
    lines: &mut Vec<String>,
    entry: &ClipboardEntry,
    metadata: Option<&serde_json::Value>,
    width: u16,
) {
    let file_metadata = metadata.and_then(|value| value.get("file_metadata"));
    let path = single_file_path(entry);

    push_compact_fields(
        lines,
        [
            (
                "文件名",
                file_metadata
                    .and_then(|value| json_string(value, "name"))
                    .or_else(|| path.and_then(file_name).map(str::to_string)),
            ),
            (
                "扩展名",
                file_metadata
                    .and_then(|value| json_string(value, "extension"))
                    .or_else(|| path.and_then(path_extension)),
            ),
            (
                "MIME",
                file_metadata.and_then(|value| json_string(value, "mime")),
            ),
            (
                "大小",
                file_metadata
                    .and_then(|value| json_u64(value, "size_bytes"))
                    .map(format_binary_size),
            ),
            (
                "目录",
                file_metadata
                    .and_then(|value| json_bool(value, "is_directory"))
                    .map(format_bool),
            ),
        ],
        width,
    );
    push_compact_fields(
        lines,
        [(
            "修改时间",
            file_metadata
                .and_then(|value| json_i64(value, "modified_at"))
                .map(format_time),
        )],
        width,
    );
    push_long_field(lines, "路径", path.map(str::to_string));
}

fn append_url_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    let Some(url_parts) = metadata.get("url_parts") else {
        return;
    };

    push_compact_fields(
        lines,
        [
            ("协议", json_string(url_parts, "protocol")),
            ("Host", json_string(url_parts, "host")),
            ("Path", json_string(url_parts, "path")),
        ],
        width,
    );

    let query_params = url_parts
        .get("query_params")
        .and_then(|value| value.as_array());
    if let Some(query_params) = query_params {
        let query_preview = query_params
            .iter()
            .take(6)
            .filter_map(format_query_param)
            .collect::<Vec<_>>()
            .join(", ");
        push_compact_fields(
            lines,
            [
                ("Query 数量", Some(query_params.len().to_string())),
                (
                    "Query",
                    (!query_preview.is_empty()).then_some(query_preview),
                ),
            ],
            width,
        );
    }
}

fn append_color_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    let Some(color_formats) = metadata.get("color_formats") else {
        return;
    };

    push_compact_fields(
        lines,
        [
            ("HEX", json_string(color_formats, "hex")),
            ("RGB", json_string(color_formats, "rgb")),
            ("RGBA", json_string(color_formats, "rgba")),
            ("HSL", json_string(color_formats, "hsl")),
        ],
        width,
    );
}

fn append_base64_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    let Some(base64_metadata) = metadata.get("base64_metadata") else {
        return;
    };

    push_compact_fields(
        lines,
        [
            (
                "Encoded",
                json_u64(base64_metadata, "encoded_size").map(|value| format!("{} bytes", value)),
            ),
            (
                "Decoded",
                json_u64(base64_metadata, "estimated_original_size")
                    .map(|value| format!("{} bytes", value)),
            ),
            ("Hint", json_string(base64_metadata, "content_hint")),
            (
                "Efficiency",
                json_f64(base64_metadata, "encoding_efficiency")
                    .map(|value| format!("{:.2}", value)),
            ),
        ],
        width,
    );
}

fn append_timestamp_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    let Some(timestamp_formats) = metadata.get("timestamp_formats") else {
        return;
    };

    push_compact_fields(
        lines,
        [
            (
                "Unix ms",
                json_i64(timestamp_formats, "unix_ms").map(|value| value.to_string()),
            ),
            ("ISO8601", json_string(timestamp_formats, "iso8601")),
            ("日期", json_string(timestamp_formats, "date_string")),
        ],
        width,
    );
}

fn append_code_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            ("语言", json_string(metadata, "detected_language")),
            (
                "行数",
                json_u64(metadata, "line_count").map(|value| value.to_string()),
            ),
        ],
        width,
    );
}

fn append_command_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            ("命令", json_string(metadata, "command_name")),
            ("Shell", json_string(metadata, "shell_family")),
            (
                "Pipeline",
                json_bool(metadata, "has_pipeline").map(format_bool),
            ),
            (
                "sudo",
                json_bool(metadata, "has_sudo_prefix").map(format_bool),
            ),
        ],
        width,
    );
}

fn append_json_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            ("根类型", json_string(metadata, "root_kind")),
            (
                "Key 数量",
                json_u64(metadata, "key_count").map(|value| value.to_string()),
            ),
        ],
        width,
    );
}

fn append_markdown_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            (
                "Heading",
                json_bool(metadata, "has_heading").map(format_bool),
            ),
            ("List", json_bool(metadata, "has_list").map(format_bool)),
            (
                "Code fence",
                json_bool(metadata, "has_code_fence").map(format_bool),
            ),
            ("Link", json_bool(metadata, "has_link").map(format_bool)),
        ],
        width,
    );
}

fn append_email_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            ("Local", json_string(metadata, "local_part")),
            ("Domain", json_string(metadata, "domain")),
        ],
        width,
    );
}

fn append_ip_attributes(lines: &mut Vec<String>, metadata: &serde_json::Value, width: u16) {
    push_compact_fields(
        lines,
        [
            ("版本", json_string(metadata, "version")),
            (
                "Loopback",
                json_bool(metadata, "is_loopback").map(format_bool),
            ),
            (
                "Private",
                json_bool(metadata, "is_private").map(format_bool),
            ),
        ],
        width,
    );
}

fn append_text_metadata_attributes(
    lines: &mut Vec<String>,
    metadata: &serde_json::Value,
    width: u16,
) {
    push_compact_fields(
        lines,
        [
            ("语言", json_string(metadata, "detected_language")),
            (
                "行数",
                json_u64(metadata, "line_count").map(|value| value.to_string()),
            ),
        ],
        width,
    );
}

fn append_content_fallback_attributes(lines: &mut Vec<String>, entry: &ClipboardEntry, width: u16) {
    if let Some(content) = entry.content_data.as_deref() {
        push_compact_fields(
            lines,
            [
                ("字符数", Some(content.chars().count().to_string())),
                ("行数", Some(count_text_lines(content).to_string())),
            ],
            width,
        );
    }
}

fn push_compact_fields<const N: usize>(
    lines: &mut Vec<String>,
    fields: [(&str, Option<String>); N],
    width: u16,
) {
    let max_width = usize::from(width.max(24));
    let segments = fields
        .into_iter()
        .filter_map(|(label, value)| format_field_segment(label, value))
        .collect::<Vec<_>>();
    let mut current = String::new();

    for segment in segments {
        if current.is_empty() {
            current = segment;
            continue;
        }

        let next_len = current.chars().count() + 5 + segment.chars().count();
        if next_len <= max_width {
            current.push_str("  |  ");
            current.push_str(&segment);
        } else {
            lines.push(current);
            current = segment;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
}

fn push_long_field(lines: &mut Vec<String>, label: &str, value: Option<String>) {
    if let Some(segment) = format_field_segment(label, value) {
        lines.push(segment);
    }
}

fn format_field_segment(label: &str, value: Option<String>) -> Option<String> {
    let value = value?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(format!("{} {}", label, value))
}

fn format_dimensions(width: Option<u64>, height: Option<u64>) -> Option<String> {
    match (width, height) {
        (Some(width), Some(height)) => Some(format!("{}x{}", width, height)),
        (Some(width), None) => Some(format!("{}w", width)),
        (None, Some(height)) => Some(format!("{}h", height)),
        (None, None) => None,
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_string)
}

fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key)?.as_u64()
}

fn json_i64(value: &serde_json::Value, key: &str) -> Option<i64> {
    value.get(key)?.as_i64()
}

fn json_bool(value: &serde_json::Value, key: &str) -> Option<bool> {
    value.get(key)?.as_bool()
}

fn json_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key)?.as_f64()
}

fn format_query_param(value: &serde_json::Value) -> Option<String> {
    let array = value.as_array()?;
    let key = array.first()?.as_str()?;
    let value = array.get(1)?.as_str()?;
    Some(format!("{}={}", key, value))
}

fn format_binary_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }

    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }

    if bytes < 1024 * 1024 * 1024 {
        return format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0));
    }

    format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn format_bool(value: bool) -> String {
    if value {
        "是".to_string()
    } else {
        "否".to_string()
    }
}

fn count_text_lines(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count().max(1)
    }
}

fn push_multiline(lines: &mut Vec<Line<'static>>, text: &str) {
    lines.extend(text.lines().map(|line| Line::from(line.to_string())));
}

fn source_app_name(entry: &ClipboardEntry) -> &str {
    entry.source_app.as_deref().unwrap_or("未知来源")
}

fn entry_title(entry: &ClipboardEntry) -> String {
    if entry.content_type.contains("image") || entry.content_type.contains("video") {
        return entry
            .file_path
            .as_deref()
            .and_then(file_name)
            .unwrap_or("Image")
            .to_string();
    }

    if entry.content_type == "file" {
        return single_file_path(entry)
            .and_then(file_name)
            .unwrap_or("Files")
            .to_string();
    }

    entry
        .content_data
        .as_deref()
        .map(normalize_preview)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| entry.content_type.clone())
}

fn normalize_preview(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > 96 {
        format!("{}...", normalized.chars().take(96).collect::<String>())
    } else {
        normalized
    }
}

fn single_file_path(entry: &ClipboardEntry) -> Option<&str> {
    entry.file_path.as_deref().or_else(|| {
        entry
            .content_data
            .as_deref()
            .filter(|value| !value.contains('\n'))
    })
}

fn open_target(entry: &ClipboardEntry, app: &HeadlessApp) -> Result<Option<OpenTarget>> {
    if entry.content_subtype.as_deref() == Some("url") {
        return Ok(entry
            .content_data
            .as_deref()
            .and_then(normalize_open_url)
            .map(OpenTarget::Url));
    }

    if entry.content_type.contains("image") || entry.content_type.contains("video") {
        return entry
            .file_path
            .as_deref()
            .or(entry.content_data.as_deref())
            .map(|path| app.resolve_file_path(path).map(OpenTarget::File))
            .transpose();
    }

    if entry.content_type == "file" {
        return single_file_path(entry)
            .map(|path| app.resolve_file_path(path).map(OpenTarget::File))
            .transpose();
    }

    Ok(None)
}

fn copy_item(entry: &ClipboardEntry, app: &HeadlessApp) -> Result<Option<CopyItem>> {
    if entry.content_type.contains("image") {
        return entry
            .file_path
            .as_deref()
            .or(entry.content_data.as_deref())
            .map(|path| app.resolve_file_path(path).map(CopyItem::Image))
            .transpose();
    }

    if let Some(content) = entry.content_data.as_deref() {
        return Ok(Some(CopyItem::Text(content.to_string())));
    }

    if entry.content_type == "file" {
        return Ok(single_file_path(entry).map(|path| CopyItem::Text(path.to_string())));
    }

    Ok(None)
}

fn normalize_open_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return None;
    }

    if let Ok(parsed) = url::Url::parse(trimmed) {
        if matches!(parsed.scheme(), "http" | "https" | "ftp") && parsed.host_str().is_some() {
            return Some(parsed.to_string());
        }
        return None;
    }

    if !trimmed.contains('.') {
        return None;
    }

    url::Url::parse(&format!("https://{}", trimmed))
        .ok()
        .filter(|parsed| parsed.host_str().is_some())
        .map(|parsed| parsed.to_string())
}

fn copy_image_file_to_clipboard(path: &Path) -> Result<()> {
    let image_data =
        std::fs::read(path).with_context(|| format!("无法读取图片 {}", path.display()))?;
    let image = image::load_from_memory(&image_data)
        .with_context(|| format!("无法解码图片 {}", path.display()))?;
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let image_data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: rgba_image.into_raw().into(),
    };
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_image(image_data)?;
    Ok(())
}

fn file_name(path: &str) -> Option<&str> {
    Path::new(path).file_name().and_then(|value| value.to_str())
}

fn path_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_string)
}

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff")
    )
}

fn build_image_picker() -> Option<Picker> {
    if !is_kitty_terminal() {
        return None;
    }

    let options = QueryStdioOptions {
        timeout: Duration::from_millis(IMAGE_PICKER_QUERY_TIMEOUT_MS),
        ..Default::default()
    };

    let mut picker =
        Picker::from_query_stdio_with_options(options).unwrap_or_else(|_| Picker::halfblocks());
    picker.set_protocol_type(ProtocolType::Kitty);
    Some(picker)
}

fn format_time(timestamp_ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(timestamp_ms)
        .map(|value| {
            value
                .with_timezone(&chrono::Local)
                .format("%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn is_kitty_terminal() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM")
            .map(|value| value.to_lowercase().contains("kitty"))
            .unwrap_or(false)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_entry(
        content_type: &str,
        content_subtype: Option<&str>,
        content_data: Option<&str>,
        file_path: Option<&str>,
        metadata: Option<String>,
    ) -> ClipboardEntry {
        ClipboardEntry {
            id: "entry-test".to_string(),
            content_hash: "hash-test".to_string(),
            content_type: content_type.to_string(),
            content_data: content_data.map(str::to_string),
            source_app: Some("Terminal".to_string()),
            created_at: 0,
            copy_count: 2,
            file_path: file_path.map(str::to_string),
            is_favorite: false,
            content_subtype: content_subtype.map(str::to_string),
            metadata,
            app_bundle_id: Some("com.apple.Terminal".to_string()),
            analysis: None,
            retrieval: None,
        }
    }

    #[test]
    fn image_attributes_show_dimensions_size_format_and_path() {
        let entry = test_entry(
            "image/png",
            None,
            None,
            Some("imgs/captured.png"),
            Some(
                json!({
                    "image_metadata": {
                        "width": 1440,
                        "height": 900,
                        "file_size": 2048,
                        "format": "png"
                    }
                })
                .to_string(),
            ),
        );

        let lines = build_attribute_text_lines(&entry, 120);

        assert!(lines
            .iter()
            .any(|line| line.contains("文件名 captured.png")));
        assert!(lines.iter().any(|line| line.contains("尺寸 1440x900")));
        assert!(lines.iter().any(|line| line.contains("大小 2.0 KB")));
        assert!(lines.iter().any(|line| line.contains("格式 PNG")));
        assert!(lines.contains(&"路径 imgs/captured.png".to_string()));
    }

    #[test]
    fn file_attributes_use_structured_metadata_without_raw_metadata_preview() {
        let entry = test_entry(
            "file",
            None,
            Some("/tmp/report.csv"),
            Some("/tmp/report.csv"),
            Some(
                json!({
                    "file_metadata": {
                        "name": "report.csv",
                        "extension": "csv",
                        "mime": "text/csv",
                        "size_bytes": 1536,
                        "modified_at": 0,
                        "is_directory": false
                    }
                })
                .to_string(),
            ),
        );

        let lines = build_attribute_text_lines(&entry, 120);
        let preview = preview_lines(&entry)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert!(lines.iter().any(|line| line.contains("文件名 report.csv")));
        assert!(lines.iter().any(|line| line.contains("扩展名 csv")));
        assert!(lines.iter().any(|line| line.contains("MIME text/csv")));
        assert!(lines.iter().any(|line| line.contains("大小 1.5 KB")));
        assert!(lines.iter().any(|line| line.contains("目录 否")));
        assert!(!preview.iter().any(|line| line == "metadata:"));
        assert!(preview.iter().any(|line| line == "paths:"));
    }

    #[test]
    fn url_and_base64_attributes_show_existing_parseable_fields() {
        let url_entry = test_entry(
            "text",
            Some("url"),
            Some("https://example.com/docs?a=1&b=2"),
            None,
            Some(
                json!({
                    "url_parts": {
                        "protocol": "https",
                        "host": "example.com",
                        "path": "/docs",
                        "query_params": [["a", "1"], ["b", "2"]]
                    }
                })
                .to_string(),
            ),
        );
        let base64_entry = test_entry(
            "text",
            Some("base64"),
            Some("aGVsbG8="),
            None,
            Some(
                json!({
                    "base64_metadata": {
                        "encoded_size": 8,
                        "estimated_original_size": 5,
                        "content_hint": "text",
                        "encoding_efficiency": 1.0
                    }
                })
                .to_string(),
            ),
        );

        let url_lines = build_attribute_text_lines(&url_entry, 120);
        let base64_lines = build_attribute_text_lines(&base64_entry, 120);

        assert!(url_lines.iter().any(|line| line.contains("协议 https")));
        assert!(url_lines
            .iter()
            .any(|line| line.contains("Host example.com")));
        assert!(url_lines.iter().any(|line| line.contains("Path /docs")));
        assert!(url_lines.iter().any(|line| line.contains("Query 数量 2")));
        assert!(url_lines.iter().any(|line| line.contains("Query a=1, b=2")));
        assert!(base64_lines
            .iter()
            .any(|line| line.contains("Encoded 8 bytes")));
        assert!(base64_lines
            .iter()
            .any(|line| line.contains("Decoded 5 bytes")));
        assert!(base64_lines.iter().any(|line| line.contains("Hint text")));
        assert!(base64_lines
            .iter()
            .any(|line| line.contains("Efficiency 1.00")));
    }

    #[test]
    fn legacy_analysis_metadata_fields_are_rendered_by_exact_names() {
        let command_entry = test_entry(
            "text",
            Some("command"),
            Some("sudo pnpm test | cat"),
            None,
            Some(
                json!({
                    "command_name": "pnpm",
                    "shell_family": "posix",
                    "has_pipeline": true,
                    "has_sudo_prefix": true
                })
                .to_string(),
            ),
        );
        let json_entry = test_entry(
            "text",
            Some("json"),
            Some(r#"{"a":1,"b":2}"#),
            None,
            Some(
                json!({
                    "root_kind": "object",
                    "key_count": 2
                })
                .to_string(),
            ),
        );

        let command_lines = build_attribute_text_lines(&command_entry, 120);
        let json_lines = build_attribute_text_lines(&json_entry, 120);

        assert!(command_lines.iter().any(|line| line.contains("命令 pnpm")));
        assert!(command_lines
            .iter()
            .any(|line| line.contains("Shell posix")));
        assert!(command_lines
            .iter()
            .any(|line| line.contains("Pipeline 是")));
        assert!(command_lines.iter().any(|line| line.contains("sudo 是")));
        assert!(json_lines.iter().any(|line| line.contains("根类型 object")));
        assert!(json_lines.iter().any(|line| line.contains("Key 数量 2")));
    }

    #[test]
    fn invalid_metadata_does_not_block_plain_text_attributes() {
        let entry = test_entry(
            "text",
            Some("plain_text"),
            Some("first\nsecond"),
            None,
            Some("{not-json".to_string()),
        );

        let lines = build_attribute_text_lines(&entry, 120);

        assert!(!lines.iter().any(|line| line.contains("类型 text")));
        assert!(!lines.iter().any(|line| line.contains("子类型 plain_text")));
        assert!(lines.iter().any(|line| line.contains("字符数 12")));
        assert!(lines.iter().any(|line| line.contains("行数 2")));
    }
}
