use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_config_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[0]);

    render_common_settings(frame, top_chunks[0], app);
    render_model_list_config(frame, top_chunks[1], app);

    render_model_settings(frame, chunks[1], app);
}

fn render_common_settings(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.config.common;
    let lines = vec![
        Line::from(format!(" llama-server path: {}", c.llama_server_path)),
        Line::from(format!(" Host: {}  Port: {}", c.host, c.port)),
        Line::from(format!(" Cache dir: {}", c.cache_dir)),
        Line::from(format!(" No mmap: {}  Flash attn: {}", c.no_mmap, c.flash_attn)),
        Line::from(format!(" Spec type: {}  Draft n-max: {}", c.spec_type, c.spec_draft_n_max)),
        Line::from(format!(" Nvtop: {}  Cmd: {}", c.nvtop_enabled, c.nvtop_cmd)),
        Line::from(format!(" Update script: {}", c.update_script_path)),
        Line::from(format!(" Extra args: {}", c.extra_args)),
        Line::from(format!(" Mid pane height: {}", c.mid_pane_height)),
    ];

    let block = Block::default()
        .title(" Common Settings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_model_list_config(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .config
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == app.config_selected_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.config_selected_idx { " > " } else { "   " };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{}", m.name), style)))
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(app.config_selected_idx));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Models ")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_model_settings(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Per-Model Settings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    if let Some(model) = app.config.models.get(app.config_selected_idx) {
        let lines = vec![
            Line::from(format!(
                " GPU Layers: {}              Context Size: {}",
                model.gpu_layers, model.ctx_size
            )),
            Line::from(format!(
                " KV Cache K: {}              KV Cache V: {}",
                model.kv_k, model.kv_v
            )),
            Line::from(format!(
                " CPU MoE: {}                 Temperature: {}",
                model.cpu_moe, model.temperature
            )),
            Line::from(format!(
                " Top-K: {}                   Top-P: {}",
                model.top_k, model.top_p
            )),
            Line::from(format!(
                " Min-P: {:.2}                Repeat Penalty: {}",
                model.min_p, model.repeat_penalty
            )),
            Line::from(format!(" Presence Penalty: {}", model.presence_penalty)),
        ];
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(Line::from("No model selected or no models configured."))
            .block(block);
        frame.render_widget(paragraph, area);
    }
}
