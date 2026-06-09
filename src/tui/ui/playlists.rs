use crate::tui::app::{AppState, Focus, Tab};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use super::theme::{ACCENT, DIM_ACCENT, DARK_GREY, WHITE};

pub(crate) fn render_playlists(frame: &mut Frame, area: Rect, state: &AppState) {
    let bc = if state.focus == Focus::List && state.active_tab == Tab::Playlists {
        ACCENT
    } else {
        DARK_GREY
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bc));
    let items: Vec<ListItem> = if state.playlists.current_tracks.is_empty() {
        state
            .playlists
            .playlists
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let sel = i == state.playlists.selected_idx
                    && state.focus == Focus::List
                    && state.active_tab == Tab::Playlists;
                let style = if sel {
                    Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(WHITE)
                };
                ListItem::new(Line::from(Span::styled(format!("  📋 {}", name), style)))
            })
            .collect()
    } else {
        state
            .playlists
            .current_tracks
            .iter()
            .enumerate()
            .map(|(i, track)| {
                let sel = i == state.playlists.selected_idx
                    && state.focus == Focus::List
                    && state.active_tab == Tab::Playlists;
                let style = if sel {
                    Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(WHITE)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("  {:>2}  {}", i + 1, track.title),
                    style,
                )))
            })
            .collect()
    };
    let empty = if items.is_empty() {
        vec![ListItem::new(Span::styled(
            "  none  |  [n] new  [s] save queue  [enter] load",
            Style::default().fg(DARK_GREY),
        ))]
    } else {
        items
    };
    let mut ls = ListState::default();
    if !empty.is_empty() {
        ls.select(Some(state.playlists.selected_idx));
    }
    frame.render_stateful_widget(List::new(empty).block(block), area, &mut ls);
}