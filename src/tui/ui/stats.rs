use crate::tui::app::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::theme::{stat_line, ACCENT, DARK_GREY, GREY};

pub(crate) fn render_stats(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let total_duration: f64 = state
        .queue
        .iter()
        .map(|track| track.duration)
        .filter(|d| *d > 0.0)
        .sum();
    let hours = (total_duration / 3600.0) as i32;
    let minutes = ((total_duration % 3600.0) / 60.0) as i32;
    let duration_text = if total_duration > 0.0 {
        format!("{}h {}m", hours, minutes)
    } else {
        "unknown".to_string()
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Library Stats",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        stat_line(
            "🎵",
            GREY,
            format!("Total Tracks      {}", state.queue.len()),
        ),
        stat_line(
            "⏱️",
            Color::Yellow,
            format!("Total Duration    {}", duration_text),
        ),
        stat_line(
            "📋",
            Color::Green,
            format!("Playlists         {}", state.playlists.playlists.len()),
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Playback Stats",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        stat_line(
            "🔊",
            Color::Blue,
            format!("Volume            {:.0}%", state.playback.volume),
        ),
        stat_line(
            "🚀",
            Color::Blue,
            format!("Speed             {:.1}x", state.playback.speed),
        ),
        stat_line(
            "🎚️",
            Color::Blue,
            format!("Pitch             {:.1}x", state.playback.pitch),
        ),
        stat_line(
            "🎵",
            GREY,
            format!("Bitrate           {:.0} kbps", state.playback.bitrate_kbps),
        ),
        Line::from(""),
        stat_line(
            "🔁",
            Color::Green,
            format!(
                "Loop Mode         {}",
                if state.playback.is_loop {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
        ),
        stat_line(
            "↔",
            Color::Yellow,
            format!(
                "Crossfade         {}",
                if state.crossfade.enabled {
                    format!("{:.1}s", state.crossfade.duration)
                } else {
                    "disabled".to_string()
                }
            ),
        ),
        stat_line(
            "🔇",
            Color::Red,
            format!(
                "Muted             {}",
                if state.playback.muted { "yes" } else { "no" }
            ),
        ),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Current Track",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}", state.playback.title),
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "  {:02}:{:02} / {:02}:{:02}",
                (state.playback.time / 60.) as i32,
                (state.playback.time % 60.) as i32,
                (state.playback.duration / 60.) as i32,
                (state.playback.duration % 60.) as i32
            ),
            Style::default().fg(DARK_GREY),
        )]),
        Line::from(""),
    ];

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}