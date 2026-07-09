use ansi_to_tui::IntoText;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the server log pane.
///
/// Uses `Paragraph::scroll()` with visual-line-based offset to anchor the
/// view.  `Paragraph` handles word-wrapping internally, so we only need to
/// compute the total visual line count and derive the scroll offset from it.
pub fn render_log_pane(
    frame: &mut Frame,
    area: Rect,
    lines: &[String],
    is_running: bool,
    log_scroll: u16,
    log_auto_scroll: bool,
) {
    let border_style = if is_running {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Server Log ")
        .borders(Borders::TOP)
        .border_style(border_style);

    if lines.is_empty() {
        frame.render_widget(Paragraph::new("").block(block), area);
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize;

    let joined = lines.join("\n");
    let text = joined
        .as_bytes()
        .into_text()
        .unwrap_or_else(|_| ratatui::text::Text::from(joined));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });

    // Exact total visual rows after word-wrapping (includes block border).
    // line_count(&self) borrows — consume/rebuild is not needed.
    let total_visual = paragraph.line_count(inner_width as u16) as usize;

    // Full area height includes the border row.  Scroll is in the paragraph's
    // visual coordinate space (borders included, see Paragraph::line_count).
    //
    // Auto-scroll: skip everything above the last `area.height` rows.
    //
    // Manual scroll: reduce skip by `log_scroll` visual rows, shifting the
    // view downward to reveal older content above.
    let scroll_y = if log_auto_scroll {
        total_visual.saturating_sub(area.height as usize)
    } else {
        total_visual
            .saturating_sub(area.height as usize)
            .saturating_sub(log_scroll as usize)
    };

    frame.render_widget(paragraph.scroll((scroll_y as u16, 0)), area);
}
