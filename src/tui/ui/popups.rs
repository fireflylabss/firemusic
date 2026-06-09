use crate::tui::app::AppState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::theme::{base64_encode, centered_rect, supports_graphics_protocol, ACCENT, DARK_GREY};

pub(crate) fn render_kitty_cover(area: Rect, state: &AppState, data: &[u8]) {
    use std::io::Write;
    let mut stdout = std::io::stdout();

    if !supports_graphics_protocol() {
        return;
    }

    let img_w = state.playback.cover_w;
    let img_h = state.playback.cover_h;
    if img_w == 0 || img_h == 0 {
        return;
    }

    let avail_w = area.width.saturating_sub(4).min(24) as u32;
    let avail_h = area.height.saturating_sub(2) as u32;
    let cell_w = 8u32;
    let cell_h = 16u32;

    let scale_w = avail_w * cell_w;
    let scale_h = avail_h * cell_h;
    let ratio = (scale_w as f64 / img_w as f64).min(scale_h as f64 / img_h as f64);
    let cols = ((img_w as f64 * ratio) / cell_w as f64) as u32;
    let rows = ((img_h as f64 * ratio) / cell_h as f64) as u32;
    if cols == 0 || rows == 0 {
        return;
    }

    let id = state.playback.cover_id;

    let encoded = base64_encode(data);
    let transmit = format!(
        "\x1b_Ga=t,f=100,s={},v={},i={},q=2;{}\x1b\\",
        img_w, img_h, id, encoded
    );
    let _ = stdout.write_all(transmit.as_bytes());
    let _ = stdout.flush();

    let col = (area.x + area.width.saturating_sub(cols as u16 + 1)).max(area.x + 1);
    let row = area.y + 1;
    let place = format!(
        "\x1b_Ga=p,i={},p=1,q=2,c={},r={},C=1;\x1b\\",
        id, cols, rows
    );
    let cursor_pos = format!("\x1b[{};{}H", row, col);
    let _ = stdout.write_all(cursor_pos.as_bytes());
    let _ = stdout.write_all(place.as_bytes());
    let _ = stdout.flush();
}

pub(crate) fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(70, 20, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .margin(1)
        .split(inner);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Tabs",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("        ", Style::default()),
            Span::styled("F1 Queue", Style::default().fg(Color::White)),
            Span::styled("  ", Style::default()),
            Span::styled("F2 Library", Style::default().fg(Color::White)),
            Span::styled("  ", Style::default()),
            Span::styled("F3 Playlists", Style::default().fg(Color::White)),
            Span::styled("  ", Style::default()),
            Span::styled("F4 Stats", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Navigation",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default()),
            Span::styled("Tab", Style::default().fg(Color::White)),
            Span::styled("        ", Style::default()),
            Span::styled("Cycle focus", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("↑↓", Style::default().fg(Color::White)),
            Span::styled("           ", Style::default()),
            Span::styled("Navigate", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Enter", Style::default().fg(Color::White)),
            Span::styled("       ", Style::default()),
            Span::styled("Play/Select", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Color::White)),
            Span::styled("         ", Style::default()),
            Span::styled("Back/Close", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Playback",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    ", Style::default()),
            Span::styled("Space", Style::default().fg(Color::White)),
            Span::styled("       ", Style::default()),
            Span::styled("Play/Pause", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("m", Style::default().fg(Color::White)),
            Span::styled("           ", Style::default()),
            Span::styled("Mute", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("←→", Style::default().fg(Color::White)),
            Span::styled("          ", Style::default()),
            Span::styled("Seek ±5s", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("↑↓", Style::default().fg(Color::White)),
            Span::styled("          ", Style::default()),
            Span::styled("Volume ±5", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("+/-", Style::default().fg(Color::White)),
            Span::styled("          ", Style::default()),
            Span::styled("Speed ±0.1", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("0", Style::default().fg(Color::White)),
            Span::styled("           ", Style::default()),
            Span::styled("Reset speed", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Other",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("      ", Style::default()),
            Span::styled("?", Style::default().fg(Color::White)),
            Span::styled("          ", Style::default()),
            Span::styled("Toggle help", Style::default().fg(DARK_GREY)),
            Span::styled("  ", Style::default()),
            Span::styled("q", Style::default().fg(Color::White)),
            Span::styled("           ", Style::default()),
            Span::styled("Quit", Style::default().fg(DARK_GREY)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press ? or Esc to close",
            Style::default().fg(DARK_GREY),
        )),
        Line::from(""),
    ];

    frame.render_widget(Paragraph::new(help_text), chunks[0]);
}

pub(crate) fn render_input_popup(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(input) = &state.input_mode else {
        return;
    };
    let width = area.width.saturating_sub(4).min(72).max(24);
    let popup = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(5) / 2,
        width,
        height: 5.min(area.height),
    };

    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(format!(" {} ", input.prompt))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let text = Line::from(vec![
        Span::styled(
            "> ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(&input.value, Style::default().fg(Color::White)),
        Span::styled(" ", Style::default().bg(ACCENT)),
    ]);
    frame.render_widget(Paragraph::new(text), inner);
}