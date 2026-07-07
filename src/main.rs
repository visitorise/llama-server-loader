mod app;
mod config;
mod model;
mod server_manager;
mod ui_config_tab;
mod ui_log;
mod ui_mid;
mod ui_server_tab;
mod ui_update_popup;

const VERSION: &str = "0.2.0";

use app::{App, AppTab, ServerState};
use ui_config_tab::{COMMON_FIELDS, MODEL_FIELDS};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Tabs, Widget},
    Terminal,
};
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

struct ScreenCapture<'a> {
    lines: &'a std::cell::RefCell<Vec<String>>,
}

impl Widget for ScreenCapture<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::with_capacity(area.height as usize);
        for row in 0..area.height {
            let mut line = String::with_capacity(area.width as usize);
            for col in 0..area.width {
                line.push_str(buf[(col, row)].symbol());
            }
            lines.push(line);
        }
        *self.lines.borrow_mut() = lines;
    }
}

struct SelectionOverlay {
    start: (u16, u16),
    end: (u16, u16),
}

impl Widget for SelectionOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (c1, r1) = self.start;
        let (c2, r2) = self.end;
        let min_col = c1.min(c2).min(area.width.saturating_sub(1));
        let max_col = c1.max(c2).min(area.width.saturating_sub(1));
        let min_row = r1.min(r2).min(area.height.saturating_sub(1));
        let max_row = r1.max(r2).min(area.height.saturating_sub(1));
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                buf[(col, row)].set_style(Style::default().add_modifier(Modifier::REVERSED));
            }
        }
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    let mut update_rx: Option<mpsc::Receiver<String>> = None;
    let mut update_handle: Option<std::thread::JoinHandle<()>> = None;

    let screen_lines: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());

    loop {
        app.poll_server_events();
        app.poll_gpu();

        terminal.draw(|frame| {
            let area = frame.area();

            let server_tab_h = if app.server_state == ServerState::Running {
                3u16
            } else {
                22u16
            };
            let mid_h = if app.graph_mode {
                app.config.common.mid_pane_height
            } else {
                6u16
            };
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(server_tab_h),
                    Constraint::Length(mid_h),
                    Constraint::Min(3),
                ])
                .split(area);

            let tab_titles = vec![" Server ", " Configure "];
            let selected = match app.tab {
                AppTab::Server => 0,
                AppTab::Configure => 1,
            };
            let tabs = Tabs::new(tab_titles)
                .select(selected)
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_widget(tabs, main_chunks[0]);

            let version_str = format!(" llama-server Loader v{VERSION} ");
            let version_paragraph = Paragraph::new(Line::from(Span::styled(
                &version_str,
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Right);
            frame.render_widget(version_paragraph, main_chunks[0]);

            match app.tab {
                AppTab::Server => {
                    ui_server_tab::render_server_tab(frame, main_chunks[1], &app);
                }
                AppTab::Configure => {
                    ui_config_tab::render_config_tab(frame, main_chunks[1], &app);
                }
            }

            ui_mid::render_mid_pane(
                frame,
                main_chunks[2],
                app.server_state == ServerState::Running,
                &app.gpu_metrics,
                &app.gpu_util_history,
                &app.gpu_mem_history,
                app.gpu_available,
                app.graph_mode,
            );

            ui_log::render_log_pane(
                frame,
                main_chunks[3],
                &app.log_lines,
                app.server_state == ServerState::Running,
                app.log_scroll,
                app.log_auto_scroll,
            );

            if app.show_update_popup {
                ui_update_popup::render_update_popup(frame, area, &app.update_output);
            }

            frame.render_widget(
                ScreenCapture { lines: &screen_lines },
                frame.area(),
            );

            if let (Some(start), Some(end)) = (app.mouse_select_start, app.mouse_select_end) {
                frame.render_widget(
                    SelectionOverlay { start, end },
                    frame.area(),
                );
            }
        })?;

        if app.show_update_popup {
            if let Some(ref rx) = update_rx {
                loop {
                    match rx.try_recv() {
                        Ok(line) => {
                            app.update_output.push(line);
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => break,
                    }
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(&mut app, key, &mut update_rx, &mut update_handle);
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            app.mouse_select_start = Some((mouse.column, mouse.row));
                            app.mouse_select_end = Some((mouse.column, mouse.row));
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            app.mouse_select_end = Some((mouse.column, mouse.row));
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            let start = app.mouse_select_start.take();
                            let end = app.mouse_select_end.take();
                            if let (Some(start), Some(end)) = (start, end) {
                                if let Ok(lines) = screen_lines.try_borrow() {
                                    let text = extract_text(&lines, start, end);
                                    if !text.is_empty() {
                                        std::thread::spawn(move || {
                                            let _ = std::process::Command::new("wl-copy")
                                                .stdin(std::process::Stdio::piped())
                                                .stdout(std::process::Stdio::null())
                                                .stderr(std::process::Stdio::null())
                                                .spawn()
                                                .and_then(|mut child| {
                                                    use std::io::Write;
                                                    if let Some(stdin) = child.stdin.as_mut() {
                                                        let _ = stdin.write_all(text.as_bytes());
                                                        let _ = stdin.flush();
                                                    }
                                                    child.wait()
                                                });
                                        });
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if app.tab == AppTab::Server {
                                app.scroll_up(3);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.tab == AppTab::Server {
                                app.scroll_down(3);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

fn extract_text(lines: &[String], start: (u16, u16), end: (u16, u16)) -> String {
    let min_row = (start.1).min(end.1) as usize;
    let max_row = (start.1).max(end.1) as usize;
    let min_col = (start.0).min(end.0) as usize;
    let max_col = (start.0).max(end.0) as usize;

    let mut result = String::new();
    for row in min_row..=max_row {
        if row >= lines.len() {
            break;
        }
        let line = &lines[row];
        let ncols = line.chars().count();
        let col_end = (max_col + 1).min(ncols);
        let col_start = min_col.min(col_end);
        let start_byte = line.char_indices().nth(col_start).map(|(i, _)| i).unwrap_or(line.len());
        let end_byte = line.char_indices().nth(col_end).map(|(i, _)| i).unwrap_or(line.len());
        result.push_str(&line[start_byte..end_byte]);
        if row < max_row && row + 1 < lines.len() {
            result.push('\n');
        }
    }
    result
}

        if app.should_quit {
            break;
        }
    }

    if app.server_state == ServerState::Running {
        let _ = app.stop_server();
    }

    drop(app);

    Ok(())
}

fn handle_key(
    app: &mut App,
    key: KeyEvent,
    update_rx: &mut Option<mpsc::Receiver<String>>,
    update_handle: &mut Option<std::thread::JoinHandle<()>>,
) {
    if app.show_update_popup {
        if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
            app.show_update_popup = false;
        }
        return;
    }

    match app.tab {
        AppTab::Server => handle_server_tab_key(app, key.code),
        AppTab::Configure => handle_config_tab_key(app, key, update_rx, update_handle),
    }
}

fn handle_server_tab_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Tab => {
            app.tab = AppTab::Configure;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.server_state != ServerState::Running {
                let idx = app.selected_model_idx.saturating_sub(1);
                app.select_model(idx);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.server_state != ServerState::Running {
                let idx = (app.selected_model_idx + 1)
                    .min(app.config.models.len().saturating_sub(1));
                app.select_model(idx);
            }
        }
        KeyCode::Enter | KeyCode::Char('r') | KeyCode::Char('R') => {
            if app.server_state == ServerState::Idle {
                if let Err(e) = app.start_server() {
                    app.log_lines.push(format!("[error] {e}"));
                }
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if app.server_state == ServerState::Running {
                if let Err(e) = app.stop_server() {
                    app.log_lines.push(format!("[error] {e}"));
                }
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.graph_mode = !app.graph_mode;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_config_tab_key(
    app: &mut App,
    key: KeyEvent,
    update_rx: &mut Option<mpsc::Receiver<String>>,
    update_handle: &mut Option<std::thread::JoinHandle<()>>,
) {
    use app::ConfigSection;

    // === EDIT MODE ===
    if app.config_edit.editing {
        match key.code {
            KeyCode::Enter => {
                let new_val = app.config_edit.input.value().to_string();
                match app.config_edit.section {
                    app::ConfigSection::Common => {
                        if let Some((_, _, set)) =
                            ui_config_tab::COMMON_FIELDS.get(app.config_edit.common_idx)
                        {
                            set(&mut app.config.common, new_val);
                        }
                    }
                    app::ConfigSection::ModelSettings => {
                        if let Some(model) =
                            app.config.models.get_mut(app.config_edit.model_list_idx)
                        {
                            if let Some((_, _, set)) =
                                ui_config_tab::MODEL_FIELDS.get(app.config_edit.model_field_idx)
                            {
                                set(model, new_val);
                            }
                        }
                    }
                    app::ConfigSection::ModelList => {}
                }
                app.config_edit.editing = false;
                app.config_edit.input = tui_input::Input::default();
            }
            KeyCode::Esc => {
                app.config_edit.editing = false;
                app.config_edit.input = tui_input::Input::default();
            }
            _ => {
                use tui_input::backend::crossterm::EventHandler;
                app.config_edit.input.handle_event(&Event::Key(key));
            }
        }
        return;
    }

    // === NAVIGATION MODE ===
    match key.code {
        KeyCode::Tab => {
            app.tab = AppTab::Server;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.config_edit.section = match app.config_edit.section {
                ConfigSection::ModelSettings => ConfigSection::ModelList,
                ConfigSection::ModelList => ConfigSection::Common,
                ConfigSection::Common => ConfigSection::Common,
            };
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.config_edit.section = match app.config_edit.section {
                ConfigSection::Common => ConfigSection::ModelList,
                ConfigSection::ModelList => ConfigSection::ModelSettings,
                ConfigSection::ModelSettings => ConfigSection::ModelSettings,
            };
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.config_edit.section {
                ConfigSection::Common => {
                    let idx = app.config_edit.common_idx.saturating_sub(1);
                    app.config_edit.common_idx = idx;
                }
                ConfigSection::ModelList => {
                    let idx = app.config_edit.model_list_idx.saturating_sub(1);
                    app.config_edit.model_list_idx = idx;
                    app.config_edit.model_field_idx = 0;
                }
                ConfigSection::ModelSettings => {
                    let idx = app.config_edit.model_field_idx.saturating_sub(1);
                    app.config_edit.model_field_idx = idx;
                }
            }
            app.config_edit.follow_selection();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.config_edit.section {
                ConfigSection::Common => {
                    let idx = (app.config_edit.common_idx + 1)
                        .min(COMMON_FIELDS.len().saturating_sub(1));
                    app.config_edit.common_idx = idx;
                }
                ConfigSection::ModelList => {
                    let idx = (app.config_edit.model_list_idx + 1)
                        .min(app.config.models.len().saturating_sub(1));
                    app.config_edit.model_list_idx = idx;
                    app.config_edit.model_field_idx = 0;
                }
                ConfigSection::ModelSettings => {
                    let idx = (app.config_edit.model_field_idx + 1)
                        .min(MODEL_FIELDS.len().saturating_sub(1));
                    app.config_edit.model_field_idx = idx;
                }
            }
            app.config_edit.follow_selection();
        }
        KeyCode::PageUp => {
            match app.config_edit.section {
                ConfigSection::Common => {
                    app.config_edit.common_scroll = app.config_edit.common_scroll.saturating_sub(5);
                    app.config_edit.common_idx = app.config_edit.common_idx.saturating_sub(5);
                }
                ConfigSection::ModelSettings => {
                    app.config_edit.model_scroll = app.config_edit.model_scroll.saturating_sub(5);
                    app.config_edit.model_field_idx = app.config_edit.model_field_idx.saturating_sub(5);
                }
                _ => {}
            }
            app.config_edit.follow_selection();
        }
        KeyCode::PageDown => {
            match app.config_edit.section {
                ConfigSection::Common => {
                    app.config_edit.common_scroll =
                        app.config_edit.common_scroll.saturating_add(5)
                            .min(COMMON_FIELDS.len().saturating_sub(1) as u16);
                    app.config_edit.common_idx =
                        (app.config_edit.common_idx + 5)
                            .min(COMMON_FIELDS.len().saturating_sub(1));
                }
                ConfigSection::ModelSettings => {
                    app.config_edit.model_scroll =
                        app.config_edit.model_scroll.saturating_add(5)
                            .min(MODEL_FIELDS.len().saturating_sub(1) as u16);
                    app.config_edit.model_field_idx =
                        (app.config_edit.model_field_idx + 5)
                            .min(MODEL_FIELDS.len().saturating_sub(1));
                }
                _ => {}
            }
            app.config_edit.follow_selection();
        }
        KeyCode::Enter => {
            // Start editing current field
            let value = match app.config_edit.section {
                ConfigSection::Common => {
                    if let Some((_, get, _)) = COMMON_FIELDS.get(app.config_edit.common_idx) {
                        get(&app.config.common)
                    } else {
                        return;
                    }
                }
                ConfigSection::ModelSettings => {
                    if let Some(model) = app.config.models.get(app.config_edit.model_list_idx) {
                        if let Some((_, get, _)) = MODEL_FIELDS.get(app.config_edit.model_field_idx)
                        {
                            get(model)
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                }
                ConfigSection::ModelList => {
                    app.config_edit.section = ConfigSection::ModelSettings;
                    app.config_edit.model_field_idx = 0;
                    return;
                }
            };
            app.config_edit.input = tui_input::Input::from(value);
            app.config_edit.editing = true;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if let Err(e) = crate::config::save_config(&app.config) {
                app.log_lines.push(format!("[error] Save failed: {e}"));
            } else {
                app.log_lines.push("[info] Config saved.".to_string());
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if !app.show_update_popup {
                let script = app.config.common.update_script_path.clone();
                let (rx, handle) = ui_update_popup::start_update_check(&script);
                app.update_output.clear();
                app.show_update_popup = true;
                *update_rx = Some(rx);
                *update_handle = Some(handle);
            }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

