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

fn build_braille_graph_row(history: &VecDeque<u32>, max_chars: usize, top: bool) -> String {
    let taken: Vec<char> = history
        .iter()
        .rev()
        .take(max_chars)
        .map(|&v| {
            if top {
                value_to_top_braille(v)
            } else {
                value_to_bottom_braille(v)
            }
        })
        .collect();
    let used = taken.len();
    let chars: String = taken.into_iter().rev().collect();
    if used < max_chars {
        chars + &" ".repeat(max_chars - used)
    } else {
        chars
    }
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
            let color = if char_pct >= 90 {
                Color::Red
            } else if char_pct >= 70 {
                Color::Yellow
            } else {
                Color::Green
            };
            Span::styled(c.to_string(), Style::default().fg(color))
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
        Line::from(vec![
            Span::styled(
                format!(" Device {} ", first.index),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}]", first.name), Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    " PCIe GEN {}@{}x RX: {} TX: {}",
                    first.pcie_link_gen,
                    first.pcie_link_width,
                    format_throughput(first.pcie_rx_kbps),
                    format_throughput(first.pcie_tx_kbps),
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!(" Device {} ", first.index),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}]", first.name), Style::default().fg(Color::White)),
            Span::styled(
                format!("  P-state: {}", first.pstate),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    };

    // Line 2: Clocks / Temp / Fan / Power
    let temp_color = if first.temp >= 85 { Color::Red } else if first.temp >= 70 { Color::Yellow } else { Color::Green };
    let line2 = Line::from(vec![
        Span::styled(format!(" GPU {:>4}MHz", first.gpu_clock), Style::default().fg(Color::White)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" MEM {:>4}MHz", first.mem_clock), Style::default().fg(Color::White)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" TEMP {:>2}\u{b0}C", first.temp), Style::default().fg(temp_color)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" FAN {:>3}%", first.fan_speed), Style::default().fg(Color::White)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" POW {:>3.0}/{:>3.0}W", first.power_draw, first.power_limit), Style::default().fg(Color::White)),
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
            Style::default().fg(Color::White),
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

        // line 3: top-half braille + GPU/MEM labels, brackets in DarkGray
        // gap after GPU `]` = 6 spaces → aligns MEM `[` with line 4's `[`
        let line3 = Line::from(vec![
            Span::raw(" "),
            Span::styled("GPU", Style::default().fg(Color::Cyan)),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::raw(gpu_top),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
            Span::raw("      "),
            Span::styled("MEM", Style::default().fg(Color::Cyan)),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::raw(mem_top),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]);

        // line 4: bottom-half braille + values, labels→spaces for alignment, brackets in DarkGray
        let mut line4_spans: Vec<Span> = Vec::new();
        line4_spans.push(Span::raw(" "));
        line4_spans.push(Span::raw("   "));
        line4_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::raw(gpu_bot));
        line4_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::styled(gpu_val, Style::default().fg(gpu_color)));
        line4_spans.push(Span::raw("  "));
        line4_spans.push(Span::raw("   "));
        line4_spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::raw(mem_bot));
        line4_spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
        line4_spans.push(Span::styled(mem_val, Style::default().fg(mem_color)));
        line4_spans.push(Span::styled(mem_info, Style::default().fg(Color::White)));
        let line4 = Line::from(line4_spans);

        let header_lines = vec![line1, line2, line3, line4];
        frame.render_widget(outer_block, area);
        frame.render_widget(Paragraph::new(header_lines), inner);
    }
}
