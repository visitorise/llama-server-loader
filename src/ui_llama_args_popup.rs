use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_llama_args_popup(frame: &mut Frame, area: Rect, args: &[String]) {
    let popup_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" llama-server CLI Args ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let mut lines: Vec<Line> = Vec::new();

    let cmd_line = args
        .iter()
        .enumerate()
        .map(|(i, a)| {
            if i == 0 {
                a.clone()
            } else {
                format!(" {a}")
            }
        })
        .collect::<String>();
    lines.push(Line::from(Span::styled(
        format!("$ llama-server {cmd_line}"),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let mut col = 0usize;
    let mut row_spans: Vec<Span> = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        let pair = if i + 1 < args.len() {
            format!("{arg} {}", args[i + 1])
        } else {
            arg.clone()
        };
        let w = pair.len() + 2;
        if col + w > 76 && col > 0 {
            lines.push(Line::from(row_spans.clone()));
            row_spans.clear();
            col = 0;
        }
        let style = if arg.starts_with('-') {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        row_spans.push(Span::styled(pair, style));
        row_spans.push(Span::raw("  "));
        col += w;
        if arg.starts_with('-') {
            // skip next if it's the value (non-dash)
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                // already included in pair
            }
        }
    }
    if !row_spans.is_empty() {
        lines.push(Line::from(row_spans));
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
