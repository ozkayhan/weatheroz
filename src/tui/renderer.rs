use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use crate::tui::ProcessState;

pub fn draw_dashboard(f: &mut ratatui::Frame, state: &ProcessState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.size());

    let header_text = vec![Line::from(vec![
        Span::styled("☁️  WEATHER RUNNER v0.1.0 — Canlı Süreç Konsolu", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])];
    let header_widget = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(header_widget, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    let mut steps_lines = Vec::new();

    let loc_str = if let Some(ref r) = state.resolved_location {
        format!("📍 Konum: {}", r)
    } else {
        format!("📍 Arama: \"{}\"", state.query)
    };
    steps_lines.push(Line::from(vec![
        Span::styled(loc_str, Style::default().fg(if state.resolved_location.is_some() { Color::Green } else { Color::Yellow }).add_modifier(Modifier::BOLD)),
    ]));

    let filled = (state.global_progress / 5) as usize;
    let mut bar_chars = String::new();
    for _ in 0..filled {
        bar_chars.push('▰');
    }
    for _ in filled..20 {
        bar_chars.push('▱');
    }
    let bar_color = if state.global_progress >= 90 {
        Color::Green
    } else if state.global_progress >= 50 {
        Color::Yellow
    } else {
        Color::Blue
    };
    steps_lines.push(Line::from(vec![
        Span::styled(
            format!("\n📊 İlerleme: {} {}%\n", bar_chars, state.global_progress),
            Style::default().fg(bar_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    let get_step_span = |name: &str, status: &str| -> Line<'static> {
        let name_owned = name.to_string();
        match status {
            "completed" => Line::from(vec![
                Span::styled("  ✅ ", Style::default().fg(Color::Green)),
                Span::styled(name_owned, Style::default().fg(Color::Green)),
            ]),
            "running" => Line::from(vec![
                Span::styled("  ⠋ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(name_owned, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            "failed" => Line::from(vec![
                Span::styled("  ❌ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(name_owned, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]),
            _ => Line::from(vec![
                Span::styled("  🕒 ", Style::default().fg(Color::DarkGray)),
                Span::styled(name_owned, Style::default().fg(Color::DarkGray)),
            ]),
        }
    };

    steps_lines.push(get_step_span("Argümanlar Ayrıştırıldı", &state.step_parsing));
    steps_lines.push(get_step_span("Konum Çözümleniyor", &state.step_geocoding));
    steps_lines.push(get_step_span("API Sağlayıcı Yarışı", &state.step_race));
    steps_lines.push(get_step_span("Veri Harmanlama & Analiz", &state.step_blending));

    let steps_widget = Paragraph::new(steps_lines)
        .block(Block::default().borders(Borders::ALL).title("📋 İşlem Adımları").border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(steps_widget, body_chunks[0]);

    let header_cells = vec!["Sağlayıcı", "Durum", "Süre"];
    let header_row = Row::new(header_cells)
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .height(1);

    let mut rows = Vec::new();
    let mut sorted_keys: Vec<&String> = state.providers.keys().collect();
    sorted_keys.sort();

    for prov_name in sorted_keys {
        if let Some(prov_info) = state.providers.get(prov_name) {
            let status_cell = match prov_info.status.as_str() {
                "completed" => Row::new(vec![
                    Cell(prov_name.to_string(), Style::default()),
                    Cell("🏆 Başarılı".to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Cell(format!("{:.1}ms", prov_info.time.unwrap_or(0.0)), Style::default().fg(Color::Green)),
                ]),
                "running" => Row::new(vec![
                    Cell(prov_name.to_string(), Style::default()),
                    Cell("🏎️ Yarışıyor...".to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Cell("-".to_string(), Style::default().fg(Color::DarkGray)),
                ]),
                "failed" => Row::new(vec![
                    Cell(prov_name.to_string(), Style::default()),
                    Cell("❌ Başarısız".to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Cell(format!("{:.1}ms", prov_info.time.unwrap_or(0.0)), Style::default().fg(Color::Red)),
                ]),
                _ => Row::new(vec![
                    Cell(prov_name.to_string(), Style::default()),
                    Cell("🕒 Beklemede".to_string(), Style::default().fg(Color::DarkGray)),
                    Cell("-".to_string(), Style::default().fg(Color::DarkGray)),
                ]),
            };
            rows.push(status_cell);
        }
    }

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
    ];

    let table_widget = Table::new(rows, widths)
        .header(header_row)
        .block(Block::default().borders(Borders::ALL).title("🏁 API Sağlayıcı Yarışı").border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(table_widget, body_chunks[1]);

    let log_style = if state.last_log.contains("hata") || state.last_log.contains("failed") || state.last_log.contains("limit") || state.last_log.contains("❌") {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if state.last_log.contains("kazandı") || state.last_log.contains("başarıyla") || state.last_log.contains("hit") || state.last_log.contains("✅") || state.last_log.contains("🏆") {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let footer_text = vec![Line::from(vec![
        Span::styled(state.last_log.clone(), log_style),
    ])];
    let footer_widget = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).title("⚡ Son Aktivite").border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(footer_widget, chunks[2]);
}

struct Cell(String, Style);

impl From<Cell> for ratatui::widgets::Cell<'_> {
    fn from(c: Cell) -> Self {
        ratatui::widgets::Cell::from(c.0).style(c.1)
    }
}
