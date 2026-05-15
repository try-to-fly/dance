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
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::Picker;
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
const SEARCH_DEBOUNCE_MS: u64 = 180;
const IMAGE_PREVIEW_DEBOUNCE_MS: u64 = 80;
const IMAGE_PROTOCOL_WIDTH: u16 = 80;
const IMAGE_PROTOCOL_HEIGHT: u16 = 40;
const IMAGE_PROTOCOL_CACHE_LIMIT: usize = 8;
const APP_ICON_PROTOCOL_WIDTH: u16 = 2;
const APP_ICON_PROTOCOL_HEIGHT: u16 = 1;
const APP_ICON_PROTOCOL_CACHE_LIMIT: usize = 48;

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
    let app = HeadlessApp::new_default().await?;

    match cli.command {
        Some(Command::Daemon) => run_daemon(app).await,
        None => run_tui(app).await,
    }
}

async fn run_daemon(app: HeadlessApp) -> Result<()> {
    println!("dance-tui daemon started for {}", app.owner());

    loop {
        if app.is_tauri_active() {
            if app.is_capture_running().await {
                app.stop_capture().await?;
            }
            app.write_daemon_status("paused_by_tauri")?;
        } else {
            if !app.is_capture_running().await {
                app.start_capture().await?;
            }
            app.write_daemon_status("listening")?;
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                app.stop_capture().await?;
                app.write_daemon_status("stopped")?;
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
    let mut search_job: Option<SearchJob> = None;
    let mut pending_image_refresh_at: Option<Instant> = Some(Instant::now());
    let mut image_job: Option<ImageJob> = None;
    let mut icon_job: Option<IconJob> = None;

    loop {
        terminal.draw(|frame| draw(frame, state))?;

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
            }
        }

        if pending_refresh_at.is_some_and(|deadline| Instant::now() >= deadline) {
            pending_refresh_at = None;
            start_search_job(state, &mut search_job);
        }

        if let Some(job) = image_job.as_ref() {
            if job.handle.is_finished() {
                let job = image_job.take().expect("image job disappeared");
                state.image_loading = false;
                match job.handle.await {
                    Ok((path, result)) => {
                        state.apply_image_result(path, result);
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        state.error = Some(format!("图片预览任务失败: {}", error));
                    }
                }
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

        let poll_timeout = next_deadline_duration(pending_refresh_at, pending_image_refresh_at)
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
                start_search_job(state, &mut search_job);
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

fn next_deadline_duration(first: Option<Instant>, second: Option<Instant>) -> Option<Duration> {
    [first, second]
        .into_iter()
        .flatten()
        .min()
        .map(|deadline| deadline.saturating_duration_since(Instant::now()))
}

fn start_search_job(state: &mut TuiState, search_job: &mut Option<SearchJob>) {
    abort_search_job(state, search_job);
    *search_job = Some(state.spawn_search_job());
    state.searching = true;
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
    abort_image_job(state, image_job);

    let Some(path) = state.selected_image_path() else {
        state.image_protocol = None;
        *pending_image_refresh_at = None;
        return;
    };

    if state
        .image_protocol
        .as_ref()
        .is_some_and(|protocol| protocol.path == path)
    {
        *pending_image_refresh_at = None;
        return;
    }

    if state.apply_cached_image_protocol(&path) {
        *pending_image_refresh_at = None;
        return;
    }

    state.image_protocol = None;
    state.image_loading = state.image_picker.is_some();
    *pending_image_refresh_at =
        Some(Instant::now() + Duration::from_millis(IMAGE_PREVIEW_DEBOUNCE_MS));
}

fn start_image_job(state: &mut TuiState, image_job: &mut Option<ImageJob>) {
    abort_image_job(state, image_job);

    let Some(path) = state.selected_image_path() else {
        state.image_protocol = None;
        state.image_loading = false;
        return;
    };

    if state
        .image_protocol
        .as_ref()
        .is_some_and(|protocol| protocol.path == path)
        || state.apply_cached_image_protocol(&path)
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
        handle: tokio::task::spawn_blocking(move || load_image_protocol(picker, path)),
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
    handle: JoinHandle<(PathBuf, std::result::Result<Protocol, String>)>,
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
            image_picker: if is_kitty_terminal() {
                Picker::from_query_stdio().ok()
            } else {
                None
            },
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
                self.entries = entries;
                if self.selected >= self.entries.len() {
                    self.selected = self.entries.len().saturating_sub(1);
                }
                if self.list_scroll > self.selected {
                    self.list_scroll = self.selected;
                }
                self.error = None;
                self.notice = None;
                true
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

    fn apply_image_result(&mut self, path: PathBuf, result: std::result::Result<Protocol, String>) {
        if self.selected_image_path().as_ref() != Some(&path) {
            return;
        }

        match result {
            Ok(protocol) => {
                self.store_image_protocol(path, protocol);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
                self.image_protocol = None;
            }
        }
    }

    fn apply_cached_image_protocol(&mut self, path: &Path) -> bool {
        let Some(index) = self
            .image_protocol_cache
            .iter()
            .position(|state| state.path == path)
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
            protocol,
        });
        self.image_loading = false;
        true
    }

    fn store_image_protocol(&mut self, path: PathBuf, protocol: Protocol) {
        self.image_protocol_cache.retain(|state| state.path != path);
        self.image_protocol_cache.push_back(ImageProtocolState {
            path: path.clone(),
            protocol: protocol.clone(),
        });
        while self.image_protocol_cache.len() > IMAGE_PROTOCOL_CACHE_LIMIT {
            self.image_protocol_cache.pop_front();
        }
        self.image_protocol = Some(ImageProtocolState { path, protocol });
        self.image_loading = false;
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
) -> (PathBuf, std::result::Result<Protocol, String>) {
    let result = load_image_protocol_result(&picker, &path);
    (path, result)
}

fn load_image_protocol_result(
    picker: &Picker,
    path: &Path,
) -> std::result::Result<Protocol, String> {
    load_protocol_result(
        picker,
        path,
        Size::new(IMAGE_PROTOCOL_WIDTH, IMAGE_PROTOCOL_HEIGHT),
    )
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

    let Some(entry) = state.selected_entry() else {
        frame.render_widget(Paragraph::new("没有剪切板记录"), inner);
        return;
    };

    if let Some(image_state) = state.image_protocol.as_ref() {
        frame.render_widget(Clear, inner);
        frame.render_widget(Image::new(&image_state.protocol), inner);
        return;
    }

    if state.image_loading && state.selected_image_path().is_some() {
        frame.render_widget(Paragraph::new("图片预览加载中..."), inner);
        return;
    }

    let lines = preview_lines(entry);
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.preview_scroll, 0));
    frame.render_widget(paragraph, inner);
}

fn preview_lines(entry: &ClipboardEntry) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        entry_title(entry),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(format!(
        "{} · {} · copied {}",
        entry.content_type,
        source_app_label(entry),
        entry.copy_count
    )));
    if let Some(retrieval) = entry.retrieval.as_ref() {
        lines.push(Line::from(format!(
            "命中: {} ({:.1}) {}",
            retrieval.label,
            retrieval.score,
            retrieval.snippet.as_deref().unwrap_or("")
        )));
    }
    lines.push(Line::from(""));

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
    if let Some(metadata) = entry.metadata.as_deref() {
        lines.push(Line::from("metadata:"));
        push_multiline(lines, metadata);
        lines.push(Line::from(""));
    }

    if let Some(content) = entry.content_data.as_deref() {
        lines.push(Line::from("paths:"));
        push_multiline(lines, content);
    }
}

fn push_multiline(lines: &mut Vec<Line<'static>>, text: &str) {
    lines.extend(text.lines().map(|line| Line::from(line.to_string())));
}

fn source_app_label(entry: &ClipboardEntry) -> String {
    source_app_name(entry).to_string()
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

fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff")
    )
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
