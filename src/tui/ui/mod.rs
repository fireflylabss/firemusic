mod library;
mod now_playing;
mod playlists;
mod popups;
mod queue;
mod sidebar;
mod stats;
mod statusbar;
mod theme;
mod titlebar;

use crate::tui::app::{AppState, Tab};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Block,
};

use library::render_library;
use now_playing::render_now_playing_bar;
use playlists::render_playlists;
use popups::{render_help_popup, render_input_popup, render_kitty_cover};
use queue::render_queue;
use sidebar::render_sidebar;
use stats::render_stats;
use statusbar::render_statusbar;
use titlebar::render_titlebar;

pub fn render(frame: &mut Frame, state: &AppState) {
    let bg = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(bg, frame.area());

    let area = frame.area();
    let show_sidebar = area.width >= 96;
    let sidebar_width = if area.width >= 120 { 30 } else { 26 };
    let chunks = if show_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_width), Constraint::Min(0)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(0), Constraint::Min(0)])
            .split(area)
    };

    if show_sidebar {
        render_sidebar(frame, state, chunks[0]);
    }

    let now_playing_height = if area.height >= 16 { 5 } else { 3 };
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(now_playing_height),
            Constraint::Length(3),
        ])
        .split(chunks[1]);

    render_titlebar(frame, state, right[0]);

    match state.active_tab {
        Tab::Queue => render_queue(frame, right[1], state),
        Tab::Library => render_library(frame, right[1], state),
        Tab::Playlists => render_playlists(frame, right[1], state),
        Tab::Stats => render_stats(frame, right[1], state),
    }

    render_now_playing_bar(frame, state, right[2]);

    if right[2].width >= 90 {
        if let Some(ref data) = state.playback.cover_art {
            render_kitty_cover(right[2], state, data);
        }
    }

    render_statusbar(frame, state, right[3]);

    if state.show_help_popup {
        render_help_popup(frame, right[1]);
    }

    if state.input_mode.is_some() {
        render_input_popup(frame, area, state);
    }
}