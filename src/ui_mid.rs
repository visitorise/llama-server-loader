use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_mid_pane(frame: &mut Frame, area: Rect, is_running: bool) {
    let border_style = if is_running {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" GPU / System Monitor ")
        .borders(Borders::TOP)
        .border_style(border_style);

    let text = if is_running {
        "Launch nvtop or htop in a separate terminal (planned for v0.2.0)"
    } else {
        "Server idle - start a model to see GPU metrics here"
    };

    let paragraph = Paragraph::new(Line::from(text)).block(block);
    frame.render_widget(paragraph, area);
}
