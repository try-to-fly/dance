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
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

const HISTORY_LIMIT: i32 = 100;
const DAEMON_TICK_MS: u64 = 500;

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
    state.refresh_image_protocol();

    let result = run_tui_loop(&mut terminal, &mut state).await;
    restore_terminal(&mut terminal)?;
    result
}

async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, state))?;

        if !event::poll(Duration::from_millis(80))? {
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
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Esc => {
                state.input = Input::default();
                state.preview_scroll = 0;
                state.refresh_entries().await?;
                state.refresh_image_protocol();
            }
            KeyCode::Up => {
                state.select_previous();
                state.refresh_image_protocol();
            }
            KeyCode::Down => {
                state.select_next();
                state.refresh_image_protocol();
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
                    state.refresh_entries().await?;
                    state.refresh_image_protocol();
                }
            }
        }
    }
}

struct TuiState {
    app: HeadlessApp,
    input: Input,
    entries: Vec<ClipboardEntry>,
    selected: usize,
    preview_scroll: u16,
    image_picker: Option<Picker>,
    image_protocol: Option<ImageProtocolState>,
    error: Option<String>,
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
            error: None,
        }
    }

    async fn refresh_entries(&mut self) -> Result<()> {
        let text = self.input.value().trim();
        let query = ClipboardHistoryQuery {
            text: (!text.is_empty()).then(|| text.to_string()),
            limit: Some(HISTORY_LIMIT),
            offset: Some(0),
            ..Default::default()
        };

        match self.app.search_clipboard_history(query).await {
            Ok(entries) => {
                self.entries = entries;
                if self.selected >= self.entries.len() {
                    self.selected = self.entries.len().saturating_sub(1);
                }
                self.error = None;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.to_string());
                Ok(())
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

    fn refresh_image_protocol(&mut self) {
        let Some(path) = self.selected_image_path() else {
            self.image_protocol = None;
            return;
        };

        if self
            .image_protocol
            .as_ref()
            .is_some_and(|state| state.path == path)
        {
            return;
        }

        let Some(picker) = self.image_picker.as_mut() else {
            self.image_protocol = None;
            return;
        };

        let reader = match image::ImageReader::open(&path) {
            Ok(reader) => reader,
            Err(error) => {
                self.error = Some(format!("图片读取失败: {}", error));
                self.image_protocol = None;
                return;
            }
        };
        let image = match reader.decode() {
            Ok(image) => image,
            Err(error) => {
                self.error = Some(format!("图片解码失败: {}", error));
                self.image_protocol = None;
                return;
            }
        };

        match picker.new_protocol(image, Size::new(80, 40), Resize::Fit(None)) {
            Ok(protocol) => {
                self.image_protocol = Some(ImageProtocolState { path, protocol });
            }
            Err(error) => {
                self.error = Some(format!("图片协议初始化失败: {}", error));
                self.image_protocol = None;
            }
        }
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
    } else {
        daemon_state
    };
    frame.render_widget(Paragraph::new(status), chunks[2]);
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
        entry.source_app.as_deref().unwrap_or("未知来源"),
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
        entry.source_app.as_deref().unwrap_or("未知来源"),
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
