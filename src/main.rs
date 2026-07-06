mod app;
mod config;
mod model;
mod server_manager;
mod ui_config_tab;
mod ui_log;
mod ui_mid;
mod ui_server_tab;
mod ui_update_popup;

const VERSION: &str = "0.1.0";

use app::{App, AppTab, ServerState};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    let mut update_rx: Option<mpsc::Receiver<String>> = None;
    let mut update_handle: Option<std::thread::JoinHandle<()>> = None;

    loop {
        app.poll_server_events();

        terminal.draw(|frame| {
            let area = frame.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(22),
                    Constraint::Length(app.config.common.mid_pane_height),
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
                )
                .block(Block::default().borders(Borders::TOP));
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
            );

            ui_log::render_log_pane(
                frame,
                main_chunks[3],
                &app.log_lines,
                app.server_state == ServerState::Running,
            );

            if app.show_update_popup {
                ui_update_popup::render_update_popup(frame, area, &app.update_output);
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
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code, &mut update_rx, &mut update_handle);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    if app.server_state == ServerState::Running {
        let _ = app.stop_server();
    }

    // App::drop() handles nvtop cleanup
    drop(app);

    Ok(())
}

fn handle_key(
    app: &mut App,
    key: KeyCode,
    update_rx: &mut Option<mpsc::Receiver<String>>,
    update_handle: &mut Option<std::thread::JoinHandle<()>>,
) {
    if app.show_update_popup {
        if key == KeyCode::Esc || key == KeyCode::Enter {
            app.show_update_popup = false;
        }
        return;
    }

    match app.tab {
        AppTab::Server => handle_server_tab_key(app, key),
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
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_config_tab_key(
    app: &mut App,
    key: KeyCode,
    update_rx: &mut Option<mpsc::Receiver<String>>,
    update_handle: &mut Option<std::thread::JoinHandle<()>>,
) {
    match key {
        KeyCode::Tab => {
            app.tab = AppTab::Server;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let idx = app.config_selected_idx.saturating_sub(1);
            if idx < app.config.models.len() {
                app.config_selected_idx = idx;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let idx = (app.config_selected_idx + 1)
                .min(app.config.models.len().saturating_sub(1));
            if idx < app.config.models.len() {
                app.config_selected_idx = idx;
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
        _ => {}
    }
}

