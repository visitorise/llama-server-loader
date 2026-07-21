use crate::app::{App, ServerState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_server_tab(frame: &mut Frame, area: Rect, app: &App) {
    let running = app.server_state == ServerState::Running;

    let (model_con, btn_con) = if running {
        (Constraint::Length(1), Constraint::Length(1))
    } else {
        (Constraint::Min(5), Constraint::Length(3))
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([model_con, btn_con, Constraint::Length(1)])
        .split(area);

    if running {
        render_model_wave(frame, chunks[0], app);
    } else {
        render_model_list(frame, chunks[0], app);
    }
    render_buttons(frame, chunks[1], app);
    render_server_hint(frame, chunks[2]);
}

fn render_model_wave(frame: &mut Frame, area: Rect, app: &App) {
    let model = app
        .config
        .models
        .get(app.selected_model_idx)
        .map(|m| m.name.as_str())
        .unwrap_or("-");

    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;

    let mut spans: Vec<Span> = Vec::new();
    let pad = " ".repeat(area.width.saturating_sub(model.len() as u16 + 6) as usize / 2);
    spans.push(Span::raw(&pad));
    spans.push(Span::styled(">>  ", Style::default().fg(Color::White)));

    for (i, c) in model.chars().enumerate() {
        let phase = elapsed / 480.0 + i as f64 * 0.5;
        let r = (phase.sin() * 40.0 + 215.0) as u8;
        let g = ((phase + 2.094).sin() * 40.0 + 215.0) as u8;
        let b = ((phase + 4.188).sin() * 40.0 + 215.0) as u8;
        spans.push(Span::styled(c.to_string(), Style::default().fg(Color::Rgb(r, g, b))));
    }

    spans.push(Span::styled("  <<", Style::default().fg(Color::White)));

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

fn render_server_hint(frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        " [Tab] Configure  [↑↓] Select  [Enter/r] Run  [s] Stop  [l] llama Args  [q] Quit",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hint, area);
}

fn render_model_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(format!(" Models ({}) ", app.config.models.len()))
        .borders(Borders::ALL);

    let items: Vec<ListItem> = app
        .config
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_running = app.server_state == ServerState::Running;
            let style = if is_running {
                Style::default().fg(Color::DarkGray)
            } else if i == app.selected_model_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if i == app.selected_model_idx { " > " } else { "   " };
            let line = Line::from(Span::styled(format!("{prefix}{}", m.name), style));
            ListItem::new(line)
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.selected_model_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_buttons(frame: &mut Frame, area: Rect, app: &App) -> Vec<(Rect, &'static str)> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Length(18),
            Constraint::Min(0),
            Constraint::Length(16),
            Constraint::Length(10),
        ])
        .split(area);

    let is_running = app.server_state == ServerState::Running;

    let run_style = if is_running {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };
    let run_label = if is_running { "[ Run ] (inactive)" } else { "[ Run ]" };
    let run = Paragraph::new(Line::from(Span::styled(run_label, run_style)));
    frame.render_widget(run, chunks[0]);

    let stop_style = if is_running {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
    };
    let stop_label = if is_running { "[ Stop ]" } else { "[Stop] (inactive)" };
    let stop = Paragraph::new(Line::from(Span::styled(stop_label, stop_style)));
    frame.render_widget(stop, chunks[1]);

    let llama_args = Paragraph::new(Line::from(Span::styled(
        "[llama Args]",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(llama_args, chunks[3]);

    let exit = Paragraph::new(Line::from(Span::styled(
        "[ Exit ]",
        Style::default().fg(Color::White),
    )));
    frame.render_widget(exit, chunks[4]);

    vec![
        (chunks[0], "run"),
        (chunks[1], "stop"),
        (chunks[3], "llama_args"),
        (chunks[4], "exit"),
    ]
}
