use crate::model::GpuMetrics;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph,
    },
    Frame,
};
use std::collections::VecDeque;

// ── helpers ──

fn format_memory(mb: f64) -> String {
    if mb >= 1024.0 {
        format!("{:.2}Gi", mb / 1024.0)
    } else {
        format!("{:.2}Mi", mb)
    }
}

fn format_throughput(kbps: u32) -> String {
    let kibps = kbps as f64;
    if kibps < 1024.0 {
        format!("{:.1} KiB/s", kibps)
    } else {
        let mibps = kibps / 1024.0;
        format!("{:.3} MiB/s", mibps)
    }
}

fn value_to_braille(value: u32) -> char {
    match value {
        0..=16 => '⣀',
        17..=33 => '⣄',
        34..=50 => '⣤',
        51..=66 => '⣴',
        67..=83 => '⣶',
        _ => '⣿',
    }
}

fn value_to_bottom_braille(v: u32) -> char {
    value_to_braille(v.min(50) * 100 / 50)
}

fn value_to_top_braille(v: u32) -> char {
    let excess = v.saturating_sub(50);
    if excess == 0 {
        return ' ';
    }
    value_to_braille(excess * 100 / 50)
}

fn build_braille_graph_row(history: &VecDeque<u32>, max_chars: usize, top: bool) -> Vec<Span<'static>> {
    let taken: Vec<(&u32, char)> = history
        .iter()
        .rev()
        .take(max_chars)
        .map(|v| {
            let c = if top {
                value_to_top_braille(*v)
            } else {
                value_to_bottom_braille(*v)
            };
            (v, c)
        })
        .collect();
    let used = taken.len();
    // taken is newest→oldest, reverse → chronological (oldest→newest)
    let data_spans: Vec<Span<'static>> = taken
        .into_iter()
        .rev()
        .map(|(&v, c)| {
            let v = v.min(100);
            let r: u8 = if v <= 50 { (v * 255 / 50) as u8 } else { 255 };
            let g: u8 = if v <= 50 { 255 } else { (255 - ((v - 50) * 255 / 50)) as u8 };
            Span::styled(c.to_string(), Style::default().fg(Color::Rgb(r, g, 0)))
        })
        .collect();
    let mut spans = Vec::with_capacity(max_chars);
    if used < max_chars {
        spans.push(Span::raw(" ".repeat(max_chars - used)));
    }
    spans.extend(data_spans);
    spans
}

fn util_bar_spans(percent: u32, total_chars: usize) -> Vec<Span<'static>> {
    if total_chars == 0 {
        return vec![];
    }
    let fill = ((percent as f32 / 100.0) * total_chars as f32).round() as usize;
    let fill = fill.min(total_chars);
    (0..total_chars)
        .map(|i| {
            let char_pct = if i < fill {
                (i as f32 / total_chars as f32 * 100.0).round() as u32
            } else {
                0
            };
            let c = if i < fill { '|' } else { ' ' };
            let v = char_pct.min(100);
            let r: u8 = if v <= 50 { (v * 255 / 50) as u8 } else { 255 };
            let g: u8 = if v <= 50 { 255 } else { (255 - ((v - 50) * 255 / 50)) as u8 };
            Span::styled(c.to_string(), Style::default().fg(Color::Rgb(r, g, 0)))
        })
        .collect()
}

// ── render──

pub fn render_mid_pane(
    frame: &mut Frame,
    area: Rect,
    is_running: bool,
    gpu_metrics: &[GpuMetrics],
    util_history: &VecDeque<u32>,
    mem_history: &VecDeque<u32>,
    gpu_available: bool,
    graph_mode: bool,
) {
    let border_style = if is_running {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if !gpu_available || gpu_metrics.is_empty() {
        let block = Block::default()
            .title(" GPU / System Monitor ")
            .borders(Borders::TOP)
            .border_style(border_style);
        let text = if is_running {
            " No NVIDIA GPU detected or NVML unavailable \u{2014} GPU metrics not available"
        } else {
            " Server idle \u{2014} start a model to see GPU metrics here"
        };
        let paragraph = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let mode_tag = if graph_mode { "graph" } else { "simple" };

    // ── Outer frame block (only TOP border) ──
    let outer_block = Block::default()
        .title_top(Line::from(" GPU / System Monitor "))
        .title_top(
            Line::from(format!(" [g] {} ", mode_tag))
                .alignment(Alignment::Right),
        )
        .borders(Borders::TOP)
        .border_style(border_style);

    let inner = outer_block.inner(area);
    let first = &gpu_metrics[0];

    // Line 1: Device index + name + PCIe / P-state
    let line1 = if first.pcie_link_gen > 0 && first.pcie_link_width > 0 {
        let rx_str = format_throughput(first.pcie_rx_kbps);
        let tx_str = format_throughput(first.pcie_tx_kbps);
        Line::from(vec![
            Span::styled(
                format!(" Device {} ", first.index),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}]", first.name), Style::default().fg(Color::Rgb(160, 160, 160))),
            Span::styled(" PCIe ", Style::default().fg(Color::Cyan)),
            Span::styled("GEN", Style::default().fg(Color::Magenta)),
            Span::styled(
                format!(" {}@{}x ", first.pcie_link_gen, first.pcie_link_width),
                Style::default().fg(Color::Rgb(160, 160, 160)),
            ),
            Span::styled("RX", Style::default().fg(Color::Magenta)),
            Span::styled(format!(": {} ", rx_str), Style::default().fg(Color::Rgb(160, 160, 160))),
            Span::styled("TX", Style::default().fg(Color::Magenta)),
            Span::styled(format!(": {}", tx_str), Style::default().fg(Color::Rgb(160, 160, 160))),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!(" Device {} ", first.index),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}]", first.name), Style::default().fg(Color::Rgb(160, 160, 160))),
            Span::styled("  P-state", Style::default().fg(Color::Cyan)),
            Span::styled(format!(": {}", first.pstate), Style::default().fg(Color::Rgb(160, 160, 160))),
        ])
    };

    // Line 2: Clocks / Temp / Fan / Power  — labels=cyan, values=gray, TEMP value=conditional
    let temp_color = if first.temp >= 85 { Color::Red } else if first.temp >= 70 { Color::Yellow } else { Color::Green };
    let line2 = Line::from(vec![
        Span::styled(" GPU ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:>4}MHz", first.gpu_clock), Style::default().fg(Color::Rgb(160, 160, 160))),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(" MEM ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:>4}MHz", first.mem_clock), Style::default().fg(Color::Rgb(160, 160, 160))),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(" TEMP ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:>2}\u{b0}C", first.temp), Style::default().fg(temp_color)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(" FAN ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:>3}%", first.fan_speed), Style::default().fg(Color::Rgb(160, 160, 160))),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(" POW ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:>3.0}/{:>3.0}W", first.power_draw, first.power_limit), Style::default().fg(Color::Rgb(160, 160, 160))),
    ]);

    // ── graph mode: bar header + chart ──
    if graph_mode {
        let avail = inner.width as usize;
        let bar_chars = avail.saturating_sub(40).max(4);
        let gpu_bar_chars = bar_chars / 2;
        let mem_bar_chars = bar_chars - gpu_bar_chars;
        let gpu_bar_spans = util_bar_spans(first.gpu_util, gpu_bar_chars);
        let mem_bar_spans = util_bar_spans(first.mem_util, mem_bar_chars);

        let mut line3_spans: Vec<Span> = Vec::new();
        line3_spans.push(Span::styled(" GPU", Style::default().fg(Color::Cyan)));
        line3_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line3_spans.extend(gpu_bar_spans);
        line3_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        let gpu_color = if first.gpu_util >= 90 { Color::Red } else if first.gpu_util >= 70 { Color::Yellow } else { Color::Green };
        line3_spans.push(Span::styled(format!(" {:>3}%", first.gpu_util), Style::default().fg(gpu_color)));
        line3_spans.push(Span::raw("  "));
        line3_spans.push(Span::styled(" MEM", Style::default().fg(Color::Cyan)));
        line3_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line3_spans.extend(mem_bar_spans);
        line3_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        let mem_color = if first.mem_util >= 90 { Color::Red } else if first.mem_util >= 70 { Color::Yellow } else { Color::Green };
        line3_spans.push(Span::styled(format!(" {:>3}%", first.mem_util), Style::default().fg(mem_color)));
        line3_spans.push(Span::raw(" "));
        line3_spans.push(Span::styled(
            format!("{}/{}", format_memory(first.mem_used_mb), format_memory(first.mem_total_mb)),
            Style::default().fg(Color::Rgb(160, 160, 160)),
        ));
        let line3 = Line::from(line3_spans);
        let line4 = Line::from("");

        let header_height: u16 = 4;
        let chart_height = inner.height.saturating_sub(header_height);
        let header_rect = Rect { x: inner.x, y: inner.y, width: inner.width, height: header_height };
        let chart_rect = if chart_height >= 5 {
            Rect { x: inner.x, y: inner.y + header_height, width: inner.width, height: chart_height }
        } else {
            Rect::default()
        };

        frame.render_widget(outer_block, area);
        frame.render_widget(Paragraph::new(vec![line1, line2, line3, line4]), header_rect);

        if chart_rect.width < 16 || chart_rect.height < 5 { return; }

        let data_len = util_history.len().max(mem_history.len());
        if data_len < 2 { return; }
        let x_max = (data_len - 1) as f64;

        let gpu_data: Vec<(f64, f64)> = util_history.iter().enumerate().map(|(i, &v)| (i as f64, v as f64)).collect();
        let mem_data: Vec<(f64, f64)> = mem_history.iter().enumerate().map(|(i, &v)| (i as f64, v as f64)).collect();

        let gpu_dataset = Dataset::default()
            .name(format!("{} GPU%", first.name))
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&gpu_data);
        let mem_dataset = Dataset::default()
            .name(format!("{} MEM%", first.name))
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .graph_type(GraphType::Line)
            .data(&mem_data);

        let total_secs = data_len;
        let x_labels = vec![
            Span::styled(format!("{}s", total_secs), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}s", total_secs * 3 / 4), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}s", total_secs / 2), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}s", total_secs / 4), Style::default().fg(Color::DarkGray)),
            Span::styled("0s", Style::default().fg(Color::DarkGray)),
        ];
        let y_labels = vec![
            Span::styled("0", Style::default().fg(Color::DarkGray)),
            Span::styled("25", Style::default().fg(Color::DarkGray)),
            Span::styled("50", Style::default().fg(Color::DarkGray)),
            Span::styled("75", Style::default().fg(Color::DarkGray)),
            Span::styled("100", Style::default().fg(Color::DarkGray)),
        ];
        let x_axis = Axis::default().bounds([0.0, x_max.max(1.0)]).labels(x_labels);
        let y_axis = Axis::default().bounds([0.0, 100.0]).labels(y_labels);
        let chart = Chart::new(vec![mem_dataset, gpu_dataset])
            .block(Block::default().borders(Borders::ALL))
            .x_axis(x_axis)
            .y_axis(y_axis)
            .legend_position(Some(LegendPosition::TopLeft));
        frame.render_widget(chart, chart_rect);

    // ── simple mode: 2-line braille history graph (GPU/MEM side-by-side) ──
    } else {
        let avail = inner.width as usize;

        let gpu_val = format!("{:>3}%", first.gpu_util);
        let mem_val = format!("{:>3}%", first.mem_util);
        let mem_info = format!(" {}/{}", format_memory(first.mem_used_mb), format_memory(first.mem_total_mb));

        let overhead = 1 + 4 + 1 + gpu_val.len() + 2 + 4 + 1 + mem_val.len() + mem_info.len();
        let braille_total = avail.saturating_sub(overhead);
        let braille_per_side = braille_total / 2;

        let gpu_top = build_braille_graph_row(util_history, braille_per_side, true);
        let gpu_bot = build_braille_graph_row(util_history, braille_per_side, false);
        let mem_top = build_braille_graph_row(mem_history, braille_per_side, true);
        let mem_bot = build_braille_graph_row(mem_history, braille_per_side, false);

        let gpu_color = if first.gpu_util >= 90 { Color::Red } else if first.gpu_util >= 70 { Color::Yellow } else { Color::Green };
        let mem_color = if first.mem_util >= 90 { Color::Red } else if first.mem_util >= 70 { Color::Yellow } else { Color::Green };

        // line 3: top-half braille (per-point color) + GPU/MEM labels
        let mut line3_spans: Vec<Span> = Vec::new();
        line3_spans.push(Span::raw(" "));
        line3_spans.push(Span::styled("GPU", Style::default().fg(Color::Cyan)));
        line3_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line3_spans.extend(gpu_top);
        line3_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        line3_spans.push(Span::raw("      "));
        line3_spans.push(Span::styled("MEM", Style::default().fg(Color::Cyan)));
        line3_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line3_spans.extend(mem_top);
        line3_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        let line3 = Line::from(line3_spans);

        // line 4: bottom-half braille (per-point color) + values
        let mut line4_spans: Vec<Span> = Vec::new();
        line4_spans.push(Span::raw(" "));
        line4_spans.push(Span::raw("   "));
        line4_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line4_spans.extend(gpu_bot);
        line4_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::styled(gpu_val, Style::default().fg(gpu_color)));
        line4_spans.push(Span::raw("  "));
        line4_spans.push(Span::raw("   "));
        line4_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line4_spans.extend(mem_bot);
        line4_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::styled(mem_val, Style::default().fg(mem_color)));
        line4_spans.push(Span::styled(mem_info, Style::default().fg(Color::Rgb(160, 160, 160))));
        let line4 = Line::from(line4_spans);

        let header_lines = vec![line1, line2, line3, line4];
        frame.render_widget(outer_block, area);
        frame.render_widget(Paragraph::new(header_lines), inner);
    }
}
