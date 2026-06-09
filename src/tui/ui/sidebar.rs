use crate::tui::app::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::theme::{stat_line, ACCENT, DARK_GREY, GREY};

pub(crate) fn render_sidebar(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(
            " Stats",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        stat_line("🎵", GREY, format!("Tracks     {}", state.queue.len())),
        stat_line(
            "▶️",
            Color::Green,
            format!(
                "Playing    {}",
                if state.playback.paused { "no" } else { "yes" }
            ),
        ),
        stat_line(
            "⏱️",
            Color::Yellow,
            format!(
                "Time       {:02}:{:02}",
                (state.playback.time / 60.) as i32,
                (state.playback.time % 60.) as i32
            ),
        ),
    ];

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Playback",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(stat_line(
        "🔊",
        Color::Blue,
        format!("Volume     {:.0}%", state.playback.volume),
    ));
    lines.push(stat_line(
        "🚀",
        Color::Blue,
        format!("Speed      {:.1}x", state.playback.speed),
    ));
    lines.push(stat_line(
        "🎚️",
        Color::Blue,
        format!("Pitch      {:.1}x", state.playback.pitch),
    ));

    if state.playback.muted {
        lines.push(stat_line("🔇", Color::Red, "Muted".to_string()));
    }
    if state.playback.is_loop {
        lines.push(stat_line("🔁", Color::Green, "Loop".to_string()));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Library",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", state.library.displayed_path()),
        Style::default().fg(Color::White),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}