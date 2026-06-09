use crate::tui::app::{AppState, Tab};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::theme::{keybinds, DARK_GREY};

pub(crate) fn render_statusbar(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let spans = if state.loading {
        vec![
            Span::styled(" ⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled(&state.loading_msg, Style::default().fg(Color::White)),
        ]
    } else if let Some(msg) = &state.status_msg {
        vec![
            Span::styled(" ", Style::default()),
            Span::styled(msg, Style::default().fg(Color::Green)),
        ]
    } else {
        match state.active_tab {
            Tab::Queue => keybinds(&[
                ("F1-F4", "tabs"),
                ("↑↓", "nav"),
                ("Enter", "play"),
                ("d", "remove"),
                ("Space", "pause"),
                ("m", "mute"),
                ("?", "help"),
                ("q", "quit"),
            ]),
            Tab::Library => keybinds(&[
                ("F1-F4", "tabs"),
                ("↑↓", "nav"),
                ("Enter", "add"),
                ("c", "cd"),
                ("r", "rescan"),
                ("?", "help"),
                ("q", "quit"),
            ]),
            Tab::Playlists => keybinds(&[
                ("F1-F4", "tabs"),
                ("↑↓", "nav"),
                ("Enter", "load/sel"),
                ("n", "new"),
                ("s", "save"),
                ("x", "delete"),
                ("Esc", "back"),
                ("?", "help"),
                ("q", "quit"),
            ]),
            Tab::Stats => keybinds(&[("F1-F4", "tabs"), ("?", "help"), ("q", "quit")]),
        }
    };

    let p = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(p, area);
}