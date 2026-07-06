use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_log_pane(frame: &mut Frame, area: Rect, lines: &[String], is_running: bool) {
    let border_style = if is_running {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Server Log ")
        .borders(Borders::TOP)
        .border_style(border_style);

    let styled_lines: Vec<Line> = lines
        .iter()
        .map(|s| {
            let style = if s.contains("[stderr]") || s.contains("error") || s.contains("exited with code") {
                Style::default().fg(Color::Red)
            } else if s.contains("started") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(s, style))
        })
        .collect();

    let paragraph = Paragraph::new(styled_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
