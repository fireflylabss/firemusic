use crate::tui::app::{AppState, Focus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::theme::{ACCENT, DARK_GREY};

pub(crate) fn render_now_playing_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    let border_color = if state.focus == Focus::NowPlaying {
        ACCENT
    } else {
        DARK_GREY
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let progress = if state.playback.duration > 0.0 {
        state.playback.time / state.playback.duration
    } else {
        0.0
    };

    let row_constraints: Vec<Constraint> = match inner.height {
        0 => return,
        1 => vec![Constraint::Length(1)],
        2 => vec![Constraint::Length(1), Constraint::Length(1)],
        _ => vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ],
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);

    let title_line = Line::from(vec![
        Span::styled("▶ ", Style::default().fg(ACCENT)),
        Span::styled(
            &state.playback.title,
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(title_line), rows[0]);

    if rows.len() == 1 {
        return;
    }

    let bar_width = rows[1].width as usize;
    let filled = (progress * bar_width as f64) as usize;
    let bar = Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(ACCENT)),
        Span::styled(
            "░".repeat(bar_width.saturating_sub(filled)),
            Style::default().fg(DARK_GREY),
        ),
    ]);
    frame.render_widget(Paragraph::new(bar), rows[1]);

    if rows.len() == 2 {
        return;
    }

    let time_str = format!(
        "{:02}:{:02} / {:02}:{:02}",
        (state.playback.time / 60.) as i32,
        (state.playback.time % 60.) as i32,
        (state.playback.duration / 60.) as i32,
        (state.playback.duration % 60.) as i32
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            time_str,
            Style::default().fg(DARK_GREY),
        ))),
        rows[2],
    );
}