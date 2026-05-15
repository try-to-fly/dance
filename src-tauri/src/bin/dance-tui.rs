use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use dance_lib::headless::{ClipboardEntry, ClipboardHistoryQuery, HeadlessApp};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};
use std::collections::VecDeque;
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
const SOURCE_APP_ICON: &str = "▣";

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

        match key.code {
            KeyCode::Char('q') => {
                abort_search_job(state, &mut search_job);
                abort_image_job(state, &mut image_job);
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
            KeyCode::Char('o') => {
                state.open_selected()?;
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

struct TuiState {
    app: HeadlessApp,
    input: Input,
    entries: Vec<ClipboardEntry>,
    selected: usize,
    preview_scroll: u16,
    image_picker: Option<Picker>,
    image_protocol: Option<ImageProtocolState>,
    image_protocol_cache: VecDeque<ImageProtocolState>,
    image_loading: bool,
    error: Option<String>,
    searching: bool,
}

struct ImageProtocolState {
    path: PathBuf,
    protocol: Protocol,
}

impl TuiState {
    fn new(app: HeadlessApp) -> Self {
        Self {
            app,
            input: Input::default(),
            entries: Vec::new(),
            selected: 0,
            preview_scroll: 0,
            image_picker: if is_kitty_terminal() {
                Picker::from_query_stdio().ok()
            } else {
                None
            },
            image_protocol: None,
            image_protocol_cache: VecDeque::new(),
            image_loading: false,
            error: None,
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
                self.error = None;
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
            return Ok(());
        };
        let Some(path) = openable_path(entry) else {
            return Ok(());
        };
        let absolute_path = self.app.resolve_file_path(path)?;
        opener::open(&absolute_path)
            .with_context(|| format!("无法打开 {}", absolute_path.display()))?;
        Ok(())
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

    let items = state
        .entries
        .iter()
        .map(|entry| ListItem::new(result_lines(entry)))
        .collect::<Vec<_>>();
    let mut list_state = ListState::default();
    if !state.entries.is_empty() {
        list_state.select(Some(state.selected));
    }
    let list = List::new(items)
        .block(Block::default().title("历史").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let daemon_state = state
        .app
        .read_daemon_status()
        .map(|status| status.state)
        .unwrap_or_else(|| "daemon_unknown".to_string());
    let status = if let Some(error) = state.error.as_deref() {
        format!("{} | {}", daemon_state, error)
    } else if state.searching {
        format!("{} | 检索中...", daemon_state)
    } else {
        daemon_state
    };
    frame.render_widget(Paragraph::new(status), chunks[2]);
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
    let reader =
        image::ImageReader::open(path).map_err(|error| format!("图片读取失败: {}", error))?;
    let image = reader
        .decode()
        .map_err(|error| format!("图片解码失败: {}", error))?;

    picker
        .new_protocol(
            image,
            Size::new(IMAGE_PROTOCOL_WIDTH, IMAGE_PROTOCOL_HEIGHT),
            Resize::Fit(None),
        )
        .map_err(|error| format!("图片协议初始化失败: {}", error))
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

fn result_lines(entry: &ClipboardEntry) -> Vec<Line<'static>> {
    let title = entry_title(entry);
    let meta = format!(
        "{} · {} · {}",
        entry
            .content_subtype
            .as_deref()
            .unwrap_or(entry.content_type.as_str()),
        source_app_label(entry),
        format_time(entry.created_at)
    );

    vec![
        Line::from(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(meta, Style::default().fg(Color::DarkGray))),
    ]
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
    format!(
        "{} {}",
        SOURCE_APP_ICON,
        entry.source_app.as_deref().unwrap_or("未知来源")
    )
}

fn entry_title(entry: &ClipboardEntry) -> String {
    if entry.content_type.contains("image") {
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

fn openable_path(entry: &ClipboardEntry) -> Option<&str> {
    if entry.content_type.contains("image") {
        return entry.file_path.as_deref().or(entry.content_data.as_deref());
    }
    if entry.content_type == "file" {
        return single_file_path(entry);
    }
    None
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
