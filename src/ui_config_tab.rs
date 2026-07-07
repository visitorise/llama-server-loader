use crate::app::{App, ConfigSection};
use crate::model::CommonSettings;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Compute the visible portion of an edited value string, accounting for horizontal
/// scroll via `tui_input::Input`, and return a display string with a cursor marker.
fn edit_value_display(input: &tui_input::Input, available_width: usize) -> String {
    if available_width < 3 {
        return "…".to_string();
    }
    let val = input.value();
    if val.is_empty() {
        return " ▏".to_string();
    }

    let scroll = input.visual_scroll(available_width).min(val.len());
    let visible = &val[scroll..];

    let cursor_col = input.visual_cursor();
    let vis_cursor = cursor_col.saturating_sub(scroll).min(visible.len());

    let (bef, aft) = visible.split_at(vis_cursor);
    format!("{bef}▏{aft}")
}

/// Labels + getter/setter for common settings fields.
pub const COMMON_FIELDS: &[(&str, fn(&CommonSettings) -> String, fn(&mut CommonSettings, String))] = &[
    ("llama_server_path",
     |c| c.llama_server_path.clone(),
     |c, v| c.llama_server_path = v),
    ("host", |c| c.host.clone(), |c, v| c.host = v),
    ("port", |c| c.port.to_string(), |c, v| { if let Ok(n) = v.parse() { c.port = n } }),
    ("cache_dir", |c| c.cache_dir.clone(), |c, v| c.cache_dir = v),
    ("model_dir", |c| c.model_dir.clone(), |c, v| c.model_dir = v),
    ("no_mmap", |c| c.no_mmap.to_string(), |c, v| c.no_mmap = v == "true"),
    ("flash_attn", |c| c.flash_attn.clone(), |c, v| c.flash_attn = v),
    ("spec_type", |c| c.spec_type.clone(), |c, v| c.spec_type = v),
    ("spec_draft_n_max", |c| c.spec_draft_n_max.to_string(), |c, v| { if let Ok(n) = v.parse() { c.spec_draft_n_max = n } }),
    ("mid_pane_height", |c| c.mid_pane_height.to_string(), |c, v| { if let Ok(n) = v.parse() { c.mid_pane_height = n } }),
    ("update_script_path", |c| c.update_script_path.clone(), |c, v| c.update_script_path = v),
    ("extra_args", |c| c.extra_args.clone(), |c, v| c.extra_args = v),
];

/// Labels + getter/setter for model settings fields.
pub const MODEL_FIELDS: &[(&str, fn(&crate::model::ModelSettings) -> String, fn(&mut crate::model::ModelSettings, String))] = &[
    ("name", |m| m.name.clone(), |m, v| m.name = v),
    ("file", |m| m.file.clone(), |m, v| m.file = v),
    ("gpu_layers", |m| m.gpu_layers.to_string(), |m, v| { if let Ok(n) = v.parse() { m.gpu_layers = n } }),
    ("ctx_size", |m| m.ctx_size.to_string(), |m, v| { if let Ok(n) = v.parse() { m.ctx_size = n } }),
    ("kv_k", |m| m.kv_k.clone(), |m, v| m.kv_k = v),
    ("kv_v", |m| m.kv_v.clone(), |m, v| m.kv_v = v),
    ("cpu_moe", |m| m.cpu_moe.to_string(), |m, v| { if let Ok(n) = v.parse() { m.cpu_moe = n } }),
    ("temperature", |m| m.temperature.to_string(), |m, v| { if let Ok(f) = v.parse() { m.temperature = f } }),
    ("top_k", |m| m.top_k.to_string(), |m, v| { if let Ok(n) = v.parse() { m.top_k = n } }),
    ("top_p", |m| m.top_p.to_string(), |m, v| { if let Ok(f) = v.parse() { m.top_p = f } }),
    ("min_p", |m| format!("{:.2}", m.min_p), |m, v| { if let Ok(f) = v.parse() { m.min_p = f } }),
    ("repeat_penalty", |m| format!("{:.1}", m.repeat_penalty), |m, v| { if let Ok(f) = v.parse() { m.repeat_penalty = f } }),
    ("presence_penalty", |m| format!("{:.1}", m.presence_penalty), |m, v| { if let Ok(f) = v.parse() { m.presence_penalty = f } }),
];

pub fn render_config_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[0]);

    render_common_settings(frame, top_chunks[0], app);
    render_model_list_config(frame, top_chunks[1], app);
    render_model_settings(frame, chunks[1], app);
    render_save_hint(frame, chunks[1]);
}

fn render_save_hint(frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        " [s] Save to disk  [c] Update check  [Tab] Server tab  [← →] Section  [Enter] Edit",
        Style::default().fg(Color::DarkGray),
    )))
    .style(Style::default());
    frame.render_widget(hint, Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1));
}

fn render_common_settings(frame: &mut Frame, area: Rect, app: &App) {
    let is_active = app.config_edit.section == ConfigSection::Common;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let inner_width = area.width.saturating_sub(2);

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, get, _)) in COMMON_FIELDS.iter().enumerate() {
        let val = get(&app.config.common);
        let selected = is_active && i == app.config_edit.common_idx;
        let editing = selected && app.config_edit.editing;

        let label_style = if selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let separator_style = Style::default().fg(Color::DarkGray);

        let (value_display, value_style) = if editing {
            let available = (inner_width as usize).saturating_sub(4 + label.len());
            let display = edit_value_display(&app.config_edit.input, available);
            (display, Style::default().add_modifier(Modifier::REVERSED))
        } else {
            let style = if selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            (val, style)
        };

        let line = Line::from(vec![
            Span::styled(format!(" {label}"), label_style),
            Span::styled(": ", separator_style),
            Span::styled(value_display, value_style),
        ]);
        lines.push(line);
    }

    let block = Block::default()
        .title(" Common Settings ")
        .borders(Borders::ALL)
        .style(border_style);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((app.config_edit.common_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn render_model_list_config(frame: &mut Frame, area: Rect, app: &App) {
    let is_active = app.config_edit.section == ConfigSection::ModelList;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let items: Vec<ListItem> = app
        .config
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if is_active && i == app.config_edit.model_list_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_active && i == app.config_edit.model_list_idx {
                " > "
            } else {
                "   "
            };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{}", m.name), style)))
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.config_edit.model_list_idx));

    let list = List::new(items)
        .block(Block::default().title(" Models ").borders(Borders::ALL).style(border_style))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_model_settings(frame: &mut Frame, area: Rect, app: &App) {
    let is_active = app.config_edit.section == ConfigSection::ModelSettings;
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let block = Block::default()
        .title(" Per-Model Settings ")
        .borders(Borders::ALL)
        .style(border_style);

    let inner_width = area.width.saturating_sub(2);

    if let Some(model) = app.config.models.get(app.config_edit.model_list_idx) {
        let mut lines: Vec<Line> = Vec::new();
        for (i, (label, get, _)) in MODEL_FIELDS.iter().enumerate() {
            let val = get(model);
            let selected = is_active && i == app.config_edit.model_field_idx;
            let editing = selected && app.config_edit.editing;

            let label_style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            let separator_style = Style::default().fg(Color::DarkGray);

            let (value_display, value_style) = if editing {
                let available = (inner_width as usize).saturating_sub(4 + label.len());
                let display = edit_value_display(&app.config_edit.input, available);
                (display, Style::default().add_modifier(Modifier::REVERSED))
            } else {
                let style = if selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                (val, style)
            };

            let line = Line::from(vec![
                Span::styled(format!(" {label}"), label_style),
                Span::styled(": ", separator_style),
                Span::styled(value_display, value_style),
            ]);
            lines.push(line);
        }
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((app.config_edit.model_scroll, 0));
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(Line::from("No model selected or no models configured."))
            .block(block);
        frame.render_widget(paragraph, area);
    }
}
