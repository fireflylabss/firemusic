use crate::tui::app::{AppState, Focus, LibraryEntry, Tab};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use super::theme::{ACCENT, DIM_ACCENT, DARK_GREY, GREY, WHITE};

pub(crate) fn render_library(frame: &mut Frame, area: Rect, state: &AppState) {
    let bc = if state.focus == Focus::List && state.active_tab == Tab::Library {
        ACCENT
    } else {
        DARK_GREY
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bc));
    let is_root = state.library.current_dir == state.library.root_dir;
    let mut items: Vec<ListItem> = Vec::new();
    if !is_root {
        items.push(ListItem::new(Line::from(Span::styled(
            " ⬅️  ..",
            Style::default().fg(DARK_GREY),
        ))));
    }
    for (display_i, (_, entry)) in state.library.visible_entries().iter().enumerate() {
        let i = display_i;
        let di = if !is_root { i + 1 } else { i };
        let sel = di == state.library.selected_idx
            && state.focus == Focus::List
            && state.active_tab == Tab::Library;
        let style = if sel {
            Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(WHITE)
        };
        let (icon, name, istyle) = match entry {
            LibraryEntry::Folder(n) => (
                "📁",
                n.clone(),
                if sel {
                    Style::default().fg(ACCENT)
                } else {
                    Style::default().fg(DARK_GREY)
                },
            ),
            LibraryEntry::Track(t) => (
                "🎵",
                t.title.clone(),
                if sel {
                    Style::default().fg(ACCENT)
                } else {
                    Style::default().fg(GREY)
                },
            ),
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), istyle),
            Span::styled(name, style),
        ])));
    }
    if items.is_empty() {
        let text = if state.library.filter.is_empty() {
            "  empty  |  [c] dir  [r] rescan"
        } else {
            "  no matches  |  [/] filter  [Esc] clear"
        };
        items.push(ListItem::new(Span::styled(
            text,
            Style::default().fg(DARK_GREY),
        )));
    }
    let mut ls = ListState::default();
    if !items.is_empty() {
        ls.select(Some(state.library.selected_idx));
    }
    frame.render_stateful_widget(List::new(items).block(block), area, &mut ls);
}