use crate::tui::app::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::theme::{ACCENT, DARK_GREY};

pub(crate) fn render_titlebar(frame: &mut Frame, state: &AppState, area: Rect) {
    let title = format!(" 🔥 Firemusic ({})", state.active_tab.title());

    let spans = vec![Span::styled(
        title,
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let p = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(p, area);
}