use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::io::BufRead;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub fn start_update_check(script_path: &str) -> (mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let script = script_path.to_string();
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = tx.send("Checking for updates...\n".to_string());
        match Command::new(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                if let Some(out) = stdout {
                    let tx_out = tx.clone();
                    thread::spawn(move || {
                        let reader = std::io::BufReader::new(out);
                        for line in reader.lines().flatten() {
                            if tx_out.send(line + "\n").is_err() {
                                break;
                            }
                        }
                    });
                }
                if let Some(err) = stderr {
                    let tx_err = tx.clone();
                    thread::spawn(move || {
                        let reader = std::io::BufReader::new(err);
                        for line in reader.lines().flatten() {
                            if tx_err.send("[stderr] ".to_string() + &line + "\n").is_err() {
                                break;
                            }
                        }
                    });
                }

                match child.wait() {
                    Ok(status) => {
                        let code = status.code().unwrap_or(-1);
                        let _ = tx.send(format!("\nDone. Exit code: {code}\n"));
                    }
                    Err(e) => {
                        let _ = tx.send(format!("\nError: {e}\n"));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(format!("Failed to launch update script: {e}\n"));
            }
        }
    });
    (rx, handle)
}

pub fn render_update_popup(frame: &mut Frame, area: Rect, lines: &[String]) {
    let popup_area = centered_rect(70, 60, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Update Check ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let text: Vec<Line> = lines
        .iter()
        .map(|s| Line::from(Span::raw(s)))
        .collect();

    let paragraph = Paragraph::new(text)
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
