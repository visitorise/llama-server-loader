use ansi_to_tui::IntoText;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Return the number of visible ASCII columns in `s`, ignoring ANSI escape
/// sequences (CSI SGR codes like `\x1b[32m`).  llama-server output is
/// ASCII-only, so every byte that isn't part of an escape sequence counts
/// as one display column — CJK wide characters are not expected.
fn display_width(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut len = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            // CSI sequence: ESC [ params… letter (terminator 0x40–0x7E)
            i += 1;
            while i < bytes.len() {
                let b = bytes[i];
                i += 1;
                if (0x40..=0x7E).contains(&b) {
                    break;
                }
            }
            continue;
        }
        // Count one display column, advance past UTF-8 continuation bytes
        let c = bytes[i];
        i += 1;
        if c & 0x80 == 0 {
            // 1-byte ASCII
        } else if c & 0xE0 == 0xC0 {
            i += 1; // 2-byte
        } else if c & 0xF0 == 0xE0 {
            i += 2; // 3-byte
        } else if c & 0xF8 == 0xF0 {
            i += 3; // 4-byte
        }
        len += 1;
    }
    len
}

/// Estimate how many visual rows a log line occupies after word-wrapping
/// at `width` columns.
fn visual_rows(line: &str, width: usize) -> usize {
    if line.is_empty() || width == 0 {
        return 1;
    }
    let effective = display_width(line).max(1);
    ((effective + width - 1) / width).max(1)
}

/// Render the server log pane.
///
/// We walk backward through `lines`, counting estimated visual rows, and
/// render only the slice that fits the visible area.  This avoids the
/// `Paragraph::scroll()` mismatch between logical and visual line counts
/// while keeping auto-scroll reliable: the newest lines always appear at
/// the bottom of the pane.
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

    let visible_height = area.height.saturating_sub(1) as usize;
    let content_width = area.width.max(1) as usize;

    let offset = if log_auto_scroll {
        0usize
    } else {
        log_scroll as usize
    };
    let target_rows = visible_height + offset;

    let mut rows = 0usize;
    let mut start = lines.len();

    for i in (0..lines.len()).rev() {
        let vr = visual_rows(&lines[i], content_width);
        // Always include at least the last line so the pane is never blank
        // when lines exist.
        if rows + vr > target_rows && start < lines.len() {
            break;
        }
        rows += vr;
        start = i;
    }

    let visible = &lines[start..];
    let joined = visible.join("\n");
    let text = joined
        .as_bytes()
        .into_text()
        .unwrap_or_else(|_| ratatui::text::Text::from(joined));

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
