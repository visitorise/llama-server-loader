use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_llama_args_popup(
    frame: &mut Frame,
    area: Rect,
    server_path: &str,
    args: &[String],
) {
    let popup_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" llama-server CLI Args ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut lines: Vec<Line> = Vec::new();

    let cmd_line = args
        .iter()
        .map(|a| format!(" {a}"))
        .collect::<String>();
    lines.push(Line::from(Span::styled(
        format!("$ {server_path}{cmd_line}"),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let mut i = 0usize;
    while i < args.len() {
        let arg = &args[i];
        let has_value = i + 1 < args.len() && !args[i + 1].starts_with('-');
        if has_value {
            lines.push(Line::from(vec![
                Span::styled(
                    arg,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", args[i + 1]), Style::default().fg(Color::White)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                arg,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        i += if has_value { 2 } else { 1 };
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Press Esc or Enter to close ",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
