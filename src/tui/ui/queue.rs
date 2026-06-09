use crate::tui::app::{AppState, Focus, Tab};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use super::theme::{ACCENT, DIM_ACCENT, DARK_GREY, WHITE};

pub(crate) fn render_queue(frame: &mut Frame, area: Rect, state: &AppState) {
    let border_color = if state.focus == Focus::List && state.active_tab == Tab::Queue {
        ACCENT
    } else {
        DARK_GREY
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = if state.queue.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  queue empty  |  browse library to add tracks",
            Style::default().fg(DARK_GREY),
        )))]
    } else {
        state
            .queue
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let is_playing = i == state.current_track_idx;
                let is_cursor = i == state.queue_cursor
                    && state.focus == Focus::List
                    && state.active_tab == Tab::Queue;
                let icon = if is_playing { "▶️" } else { " " };
                let style = match (is_playing, is_cursor) {
                    (true, _) => Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    (false, true) => Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD),
                    _ => Style::default().fg(WHITE),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", icon),
                        if is_playing {
                            Style::default().fg(ACCENT)
                        } else {
                            Style::default().fg(DARK_GREY)
                        },
                    ),
                    Span::styled(format!("{:>2}", i + 1), Style::default().fg(DARK_GREY)),
                    Span::styled(format!("  {}", track.title), style),
                ]))
            })
            .collect()
    };
    let mut ls = ListState::default();
    if !items.is_empty() {
        ls.select(Some(state.queue_cursor));
    }
    frame.render_stateful_widget(List::new(items).block(block), area, &mut ls);
}