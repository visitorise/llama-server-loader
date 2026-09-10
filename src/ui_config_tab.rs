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

pub const COMMON_FIELDS: &[(&str, fn(&CommonSettings) -> String, fn(&mut CommonSettings, String))] = &[
    ("llama_server_path (master)",
     |c| c.llama_server_path.clone(),
     |c, v| c.llama_server_path = v),
    ("model_path", |c| c.model_dir.clone(), |c, v| c.model_dir = v),
    ("host", |c| c.host.clone(), |c, v| c.host = v),
    ("port", |c| c.port.to_string(), |c, v| { if let Ok(n) = v.parse() { c.port = n } }),
    ("mid_pane_height", |c| c.mid_pane_height.to_string(), |c, v| { if let Ok(n) = v.parse() { c.mid_pane_height = n } }),
    ("update_script_path", |c| c.update_script_path.clone(), |c, v| c.update_script_path = v),
    ("extra_args", |c| c.extra_args.clone(), |c, v| c.extra_args = v),
];

pub type ModelFieldGetter = fn(&crate::model::ModelSettings) -> String;
pub type ModelFieldSetter = fn(&mut crate::model::ModelSettings, String);

pub struct ModelField {
    pub label: &'static str,
    pub get: ModelFieldGetter,
    pub set: ModelFieldSetter,
    pub enabled_get: Option<fn(&crate::model::ModelSettings) -> bool>,
    pub enabled_toggle: Option<fn(&mut crate::model::ModelSettings)>,
    pub fallback_get: Option<fn(&CommonSettings) -> String>,
}

impl ModelField {
    const fn new(
        label: &'static str,
        get: ModelFieldGetter,
        set: ModelFieldSetter,
        enabled_get: Option<fn(&crate::model::ModelSettings) -> bool>,
        enabled_toggle: Option<fn(&mut crate::model::ModelSettings)>,
    ) -> Self {
        Self { label, get, set, enabled_get, enabled_toggle, fallback_get: None }
    }

    const fn with_fallback(mut self, fallback: fn(&CommonSettings) -> String) -> Self {
        self.fallback_get = Some(fallback);
        self
    }
}

pub const MODEL_FIELDS: &[ModelField] = &[
    ModelField::new("llama_server_path", |m| m.llama_server_path.clone(), |m, v| m.llama_server_path = v, Some(|m| m.llama_server_path_enabled), Some(|m| m.llama_server_path_enabled = !m.llama_server_path_enabled))
        .with_fallback(|c| c.llama_server_path.clone()),
    ModelField::new("model", |m| m.file.clone(), |m, v| m.file = v, Some(|m| m.file_enabled), Some(|m| m.file_enabled = !m.file_enabled)),
    ModelField::new("model_draft", |m| m.model_draft.clone(), |m, v| m.model_draft = v, Some(|m| m.model_draft_enabled), Some(|m| m.model_draft_enabled = !m.model_draft_enabled)),
    ModelField::new("alias", |m| m.name.clone(), |m, v| m.name = v, Some(|m| m.name_enabled), Some(|m| m.name_enabled = !m.name_enabled)),
    ModelField::new("spec_type", |m| m.spec_type.clone(), |m, v| m.spec_type = v, Some(|m| m.spec_type_enabled), Some(|m| m.spec_type_enabled = !m.spec_type_enabled)),
    ModelField::new("spec_draft_n_max", |m| m.spec_draft_n_max.to_string(), |m, v| { if let Ok(n) = v.parse() { m.spec_draft_n_max = n } }, Some(|m| m.spec_draft_n_max_enabled), Some(|m| m.spec_draft_n_max_enabled = !m.spec_draft_n_max_enabled)),
    ModelField::new("cache_dir", |m| m.cache_dir.clone(), |m, v| m.cache_dir = v, Some(|m| m.cache_dir_enabled), Some(|m| m.cache_dir_enabled = !m.cache_dir_enabled)),
    ModelField::new("gpu_layers", |m| m.gpu_layers.clone(), |m, v| m.gpu_layers = v, Some(|m| m.gpu_layers_enabled), Some(|m| m.gpu_layers_enabled = !m.gpu_layers_enabled)),
    ModelField::new("cpu_moe", |m| m.cpu_moe.to_string(), |m, v| { if let Ok(n) = v.parse() { m.cpu_moe = n } }, Some(|m| m.cpu_moe_enabled), Some(|m| m.cpu_moe_enabled = !m.cpu_moe_enabled)),
    ModelField::new("threads", |m| m.threads.to_string(), |m, v| { if let Ok(n) = v.parse() { m.threads = n } }, Some(|m| m.threads_enabled), Some(|m| m.threads_enabled = !m.threads_enabled)),
    ModelField::new("moe_expert_cache_size", |m| m.moe_expert_cache_size.to_string(), |m, v| { if let Ok(n) = v.parse() { m.moe_expert_cache_size = n } }, Some(|m| m.moe_expert_cache_size_enabled), Some(|m| m.moe_expert_cache_size_enabled = !m.moe_expert_cache_size_enabled)),
    ModelField::new("ctx_size", |m| m.ctx_size.to_string(), |m, v| { if let Ok(n) = v.parse() { m.ctx_size = n } }, Some(|m| m.ctx_size_enabled), Some(|m| m.ctx_size_enabled = !m.ctx_size_enabled)),
    ModelField::new("kv_k", |m| m.kv_k.clone(), |m, v| m.kv_k = v, Some(|m| m.kv_k_enabled), Some(|m| m.kv_k_enabled = !m.kv_k_enabled)),
    ModelField::new("kv_v", |m| m.kv_v.clone(), |m, v| m.kv_v = v, Some(|m| m.kv_v_enabled), Some(|m| m.kv_v_enabled = !m.kv_v_enabled)),
    ModelField::new("temperature", |m| m.temperature.to_string(), |m, v| { if let Ok(f) = v.parse() { m.temperature = f } }, Some(|m| m.temperature_enabled), Some(|m| m.temperature_enabled = !m.temperature_enabled)),
    ModelField::new("top_k", |m| m.top_k.to_string(), |m, v| { if let Ok(n) = v.parse() { m.top_k = n } }, Some(|m| m.top_k_enabled), Some(|m| m.top_k_enabled = !m.top_k_enabled)),
    ModelField::new("top_p", |m| m.top_p.to_string(), |m, v| { if let Ok(f) = v.parse() { m.top_p = f } }, Some(|m| m.top_p_enabled), Some(|m| m.top_p_enabled = !m.top_p_enabled)),
    ModelField::new("min_p", |m| format!("{:.2}", m.min_p), |m, v| { if let Ok(f) = v.parse() { m.min_p = f } }, Some(|m| m.min_p_enabled), Some(|m| m.min_p_enabled = !m.min_p_enabled)),
    ModelField::new("repeat_penalty", |m| format!("{:.2}", m.repeat_penalty), |m, v| { if let Ok(f) = v.parse() { m.repeat_penalty = f } }, Some(|m| m.repeat_penalty_enabled), Some(|m| m.repeat_penalty_enabled = !m.repeat_penalty_enabled)),
    ModelField::new("presence_penalty", |m| format!("{:.2}", m.presence_penalty), |m, v| { if let Ok(f) = v.parse() { m.presence_penalty = f } }, Some(|m| m.presence_penalty_enabled), Some(|m| m.presence_penalty_enabled = !m.presence_penalty_enabled)),
    ModelField::new("batch_size", |m| m.batch_size.to_string(), |m, v| { if let Ok(n) = v.parse() { m.batch_size = n } }, Some(|m| m.batch_size_enabled), Some(|m| m.batch_size_enabled = !m.batch_size_enabled)),
    ModelField::new("ubatch_size", |m| m.ubatch_size.to_string(), |m, v| { if let Ok(n) = v.parse() { m.ubatch_size = n } }, Some(|m| m.ubatch_size_enabled), Some(|m| m.ubatch_size_enabled = !m.ubatch_size_enabled)),
    ModelField::new("no_mmap", |m| m.no_mmap.to_string(), |m, v| m.no_mmap = v == "true", Some(|m| m.no_mmap_enabled), Some(|m| m.no_mmap_enabled = !m.no_mmap_enabled)),
    ModelField::new("flash_attn", |m| m.flash_attn.clone(), |m, v| m.flash_attn = v, Some(|m| m.flash_attn_enabled), Some(|m| m.flash_attn_enabled = !m.flash_attn_enabled)),
    ModelField::new("cache_ram", |m| m.cache_ram.to_string(), |m, v| { if let Ok(n) = v.parse() { m.cache_ram = n } }, Some(|m| m.cache_ram_enabled), Some(|m| m.cache_ram_enabled = !m.cache_ram_enabled)),
    ModelField::new("load_mode", |m| m.load_mode.clone(), |m, v| m.load_mode = v, Some(|m| m.load_mode_enabled), Some(|m| m.load_mode_enabled = !m.load_mode_enabled)),
    ModelField::new("parallel", |m| m.parallel.to_string(), |m, v| { if let Ok(n) = v.parse() { m.parallel = n } }, Some(|m| m.parallel_enabled), Some(|m| m.parallel_enabled = !m.parallel_enabled)),
    ModelField::new("fit", |m| m.fit.clone(), |m, v| m.fit = v, Some(|m| m.fit_enabled), Some(|m| m.fit_enabled = !m.fit_enabled)),
    ModelField::new("kv_offload", |_| String::new(), |_, _| {}, Some(|m| m.kv_offload_enabled), Some(|m| m.kv_offload_enabled = !m.kv_offload_enabled)),
    ModelField::new("jinja", |_| String::new(), |_, _| {}, Some(|m| m.jinja_enabled), Some(|m| m.jinja_enabled = !m.jinja_enabled)),
    ModelField::new("extra_args", |m| m.extra_args.clone(), |m, v| m.extra_args = v, Some(|m| m.extra_args_enabled), Some(|m| m.extra_args_enabled = !m.extra_args_enabled)),
];

pub fn render_config_tab(frame: &mut Frame, area: Rect, app: &App) {
    let common_h = (area.height / 2).saturating_sub(2);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(common_h), Constraint::Min(0)])
        .split(area);

    let model_w = (chunks[1].width / 2).saturating_sub(5);
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(model_w), Constraint::Min(0)])
        .split(chunks[1]);

    render_common_settings(frame, chunks[0], app);
    render_model_list_config(frame, bottom_chunks[0], app);
    render_model_settings(frame, bottom_chunks[1], app);
    render_save_hint(frame, bottom_chunks[1]);
}

fn render_save_hint(frame: &mut Frame, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        " [s] Save  [c] Update check  [Space] Toggle checkbox  [Tab] Server tab  [← →] Section  [Enter] Edit",
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
        for (i, field) in MODEL_FIELDS.iter().enumerate() {
            let val = if (field.get)(model).is_empty() {
                match field.fallback_get {
                    Some(fb) => fb(&app.config.common),
                    None => String::new(),
                }
            } else {
                (field.get)(model)
            };
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

            let checkbox = match field.enabled_get {
                Some(get) => {
                    let mark = if get(model) { "x" } else { " " };
                    let fg = if get(model) { Color::White } else { Color::DarkGray };
                    Span::styled(format!("[{mark}] "), Style::default().fg(fg))
                }
                None => Span::styled("    ", Style::default().fg(Color::DarkGray)),
            };

            let (value_display, value_style) = if editing {
                let available = (inner_width as usize).saturating_sub(7 + field.label.len());
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
                checkbox,
                Span::styled(format!(" {label}", label = field.label), label_style),
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
