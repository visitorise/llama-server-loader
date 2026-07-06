use crate::app::{App, ServerState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_server_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    render_model_list(frame, chunks[0], app);
    render_buttons(frame, chunks[1], app);
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

fn render_buttons(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Length(18),
            Constraint::Min(0),
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

    let exit = Paragraph::new(Line::from(Span::styled(
        "[ Exit ]",
        Style::default().fg(Color::White),
    )));
    frame.render_widget(exit, chunks[3]);
}
