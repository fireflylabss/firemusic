use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::tui::state::{AppState, Focus, LibraryEntry, Tab};

fn supports_graphics_protocol() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    
    // Check for Kitty terminal
    if term == "xterm-kitty" || term_program == "kitty" {
        return true;
    }
    
    // Check for other terminals with graphics support
    // WezTerm, iTerm2, etc. could be added here
    
    false
}

const ACCENT: Color = Color::Red;
const DIM_ACCENT: Color = Color::Rgb(100, 20, 20);
const WHITE: Color = Color::White;
const GREY: Color = Color::Gray;
const DARK_GREY: Color = Color::DarkGray;

pub fn render(frame: &mut Frame, state: &AppState) {
    let bg = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(bg, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(frame.area());

    render_sidebar(frame, state, chunks[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Min(0),    // content
            Constraint::Length(6), // now playing bar
            Constraint::Length(3), // status bar
        ])
        .split(chunks[1]);

    render_titlebar(frame, state, right[0]);

    match state.active_tab {
        Tab::Queue => render_queue(frame, right[1], state),
        Tab::Library => render_library(frame, right[1], state),
        Tab::Playlists => render_playlists(frame, right[1], state),
        Tab::Stats => render_stats(frame, right[1], state),
    }

    // Kitty cover art overlay — writes raw escape sequences after ratatui render
    if let Some(ref data) = state.playback.cover_art {
        render_kitty_cover(right[1], state, data);
    }

    render_now_playing_bar(frame, state, right[2]);
    render_statusbar(frame, state, right[3]);

    // Render help popup if active
    if state.show_help_popup {
        render_help_popup(frame, right[1]);
    }
}

fn render_kitty_cover(area: Rect, state: &AppState, data: &[u8]) {
    use std::io::Write;
    let mut stdout = std::io::stdout();

    // Only attempt on terminals with graphics support
    if !supports_graphics_protocol() {
        return;
    }

    // Place cover art on the right side of the now playing area
    let img_w = state.playback.cover_w;
    let img_h = state.playback.cover_h;
    if img_w == 0 || img_h == 0 { return; }

    let avail_w = area.width.saturating_sub(2) as u32;
    let avail_h = area.height.saturating_sub(2) as u32;
    let cell_w = 8u32;  // approximate pixels per column
    let cell_h = 16u32; // approximate pixels per row

    let scale_w = avail_w * cell_w;
    let scale_h = avail_h * cell_h;
    let ratio = (scale_w as f64 / img_w as f64).min(scale_h as f64 / img_h as f64);
    let cols = ((img_w as f64 * ratio) / cell_w as f64) as u32;
    let rows = ((img_h as f64 * ratio) / cell_h as f64) as u32;
    if cols == 0 || rows == 0 { return; }

    let id = state.playback.cover_id;

    // Transmit image
    let encoded = base64_encode(data);
    let transmit = format!("\x1b_Ga=t,f=100,s={},v={},i={},q=2;{}\x1b\\", img_w, img_h, id, encoded);
    let _ = stdout.write_all(transmit.as_bytes());
    let _ = stdout.flush();

    // Place image in the area
    let col = (area.x + area.width.saturating_sub(cols as u16)).max(area.x + 1);
    let row = area.y + 1;
    let place = format!("\x1b_Ga=p,i={},p=1,q=2,c={},r={},C=1;\x1b\\", id, cols, rows);
    // Move cursor to position
    let cursor_pos = format!("\x1b[{};{}H", row, col);
    let _ = stdout.write_all(cursor_pos.as_bytes());
    let _ = stdout.write_all(place.as_bytes());
    let _ = stdout.flush();
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(n & 0x3F) as usize] as char); } else { out.push('='); }
    }
    out
}

fn render_sidebar(frame: &mut Frame, state: &AppState, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(" Stats", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        stat_line("🎵", Color::Cyan, format!("Tracks     {}", state.queue.len())),
        stat_line("▶️", Color::Green, format!("Playing    {}", if state.playback.paused { "no" } else { "yes" })),
        stat_line("⏱️", Color::Yellow, format!("Time       {:02}:{:02}", (state.playback.time/60.) as i32, (state.playback.time%60.) as i32)),
    ];

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Playback", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));
    lines.push(stat_line("🔊", Color::Blue, format!("Volume     {:.0}%", state.playback.volume)));
    lines.push(stat_line("🚀", Color::Blue, format!("Speed      {:.1}x", state.playback.speed)));
    lines.push(stat_line("🎚️", Color::Blue, format!("Pitch      {:.1}x", state.playback.pitch)));

    if state.playback.muted {
        lines.push(stat_line("🔇", Color::Red, "Muted".to_string()));
    }
    if state.playback.is_loop {
        lines.push(stat_line("🔁", Color::Green, "Loop".to_string()));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Library", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(format!("  {}", state.library.displayed_path()), Style::default().fg(Color::White))));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn stat_line(icon: &str, color: Color, label: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {} ", icon), Style::default().fg(color)),
        Span::styled(label, Style::default().fg(Color::White)),
    ])
}

fn render_titlebar(frame: &mut Frame, state: &AppState, area: Rect) {
    let title = match state.active_tab {
        Tab::Queue => " 🎵 Queue",
        Tab::Library => " 📁 Library",
        Tab::Playlists => " 📋 Playlists",
        Tab::Stats => " 📊 Stats",
    };

    let mut spans = vec![
        Span::styled(title, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ];

    if let Some(msg) = &state.status_msg {
        spans.push(Span::styled(format!("  |  {}", msg), Style::default().fg(Color::Green)));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let p = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(p, area);
}

fn render_queue(frame: &mut Frame, area: Rect, state: &AppState) {
    let border_color = if state.focus == Focus::List && state.active_tab == Tab::Queue { ACCENT } else { DARK_GREY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = if state.queue.is_empty() {
        vec![ListItem::new(Line::from(Span::styled("  queue empty  |  browse library to add tracks", Style::default().fg(DARK_GREY))))]
    } else {
        state.queue.iter().enumerate().map(|(i, track)| {
            let is_playing = i == state.current_track_idx;
            let is_cursor = i == state.queue_cursor && state.focus == Focus::List && state.active_tab == Tab::Queue;
            let icon = if is_playing { "▶️" } else { " " };
            let style = match (is_playing, is_cursor) {
                (true, _) => Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                (false, true) => Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD),
                _ => Style::default().fg(WHITE),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", icon), if is_playing { Style::default().fg(ACCENT) } else { Style::default().fg(DARK_GREY) }),
                Span::styled(format!("{:>2}", i + 1), Style::default().fg(DARK_GREY)),
                Span::styled(format!("  {}", track.title), style),
            ]))
        }).collect()
    };
    let mut ls = ListState::default();
    if !items.is_empty() { ls.select(Some(state.queue_cursor)); }
    frame.render_stateful_widget(List::new(items).block(block), area, &mut ls);
}

fn render_library(frame: &mut Frame, area: Rect, state: &AppState) {
    let bc = if state.focus == Focus::List && state.active_tab == Tab::Library { ACCENT } else { DARK_GREY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bc));
    let is_root = state.library.current_dir == state.library.root_dir;
    let mut items: Vec<ListItem> = Vec::new();
    if !is_root { items.push(ListItem::new(Line::from(Span::styled(" ⬅️  ..", Style::default().fg(DARK_GREY))))); }
    for (i, entry) in state.library.entries.iter().enumerate() {
        let di = if !is_root { i + 1 } else { i };
        let sel = di == state.library.selected_idx && state.focus == Focus::List && state.active_tab == Tab::Library;
        let style = if sel { Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD) } else { Style::default().fg(WHITE) };
        let (icon, name, istyle) = match entry {
            LibraryEntry::Folder(n) => ("📁", n.clone(), if sel { Style::default().fg(ACCENT) } else { Style::default().fg(DARK_GREY) }),
            LibraryEntry::Track(t) => ("🎵", t.title.clone(), if sel { Style::default().fg(ACCENT) } else { Style::default().fg(GREY) }),
        };
        items.push(ListItem::new(Line::from(vec![Span::styled(format!(" {} ", icon), istyle), Span::styled(name, style)])));
    }
    if items.is_empty() { items.push(ListItem::new(Span::styled("  empty  |  [c] dir  [r] rescan", Style::default().fg(DARK_GREY)))); }
    let mut ls = ListState::default();
    if !items.is_empty() { ls.select(Some(state.library.selected_idx)); }
    frame.render_stateful_widget(List::new(items).block(block), area, &mut ls);
}

fn render_playlists(frame: &mut Frame, area: Rect, state: &AppState) {
    let bc = if state.focus == Focus::List && state.active_tab == Tab::Playlists { ACCENT } else { DARK_GREY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bc));
    let items: Vec<ListItem> = if state.playlists.current_tracks.is_empty() {
        state.playlists.playlists.iter().enumerate().map(|(i, name)| {
            let sel = i == state.playlists.selected_idx && state.focus == Focus::List && state.active_tab == Tab::Playlists;
            let style = if sel { Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD) } else { Style::default().fg(WHITE) };
            ListItem::new(Line::from(Span::styled(format!("  📋 {}", name), style)))
        }).collect()
    } else {
        state.playlists.current_tracks.iter().enumerate().map(|(i, track)| {
            let sel = i == state.playlists.selected_idx && state.focus == Focus::List && state.active_tab == Tab::Playlists;
            let style = if sel { Style::default().bg(DIM_ACCENT).add_modifier(Modifier::BOLD) } else { Style::default().fg(WHITE) };
            ListItem::new(Line::from(Span::styled(format!("  {:>2}  {}", i + 1, track.title), style)))
        }).collect()
    };
    let empty = if items.is_empty() { vec![ListItem::new(Span::styled("  none  |  [n] new  [s] save queue  [enter] load", Style::default().fg(DARK_GREY)))] } else { items };
    let mut ls = ListState::default();
    if !empty.is_empty() { ls.select(Some(state.playlists.selected_idx)); }
    frame.render_stateful_widget(List::new(empty).block(block), area, &mut ls);
}

fn render_statusbar(frame: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let spans: Vec<Span> = if state.loading {
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
                ("F1-F4", "tabs"), ("↑↓", "nav"), ("Enter", "play"),
                ("d", "remove"), ("Space", "pause"), ("m", "mute"),
                ("?", "help"), ("q", "quit"),
            ]),
            Tab::Library => keybinds(&[
                ("F1-F4", "tabs"), ("↑↓", "nav"), ("Enter", "add"),
                ("c", "cd"), ("r", "rescan"), ("?", "help"),
                ("q", "quit"),
            ]),
            Tab::Playlists => keybinds(&[
                ("F1-F4", "tabs"), ("↑↓", "nav"), ("Enter", "load/sel"),
                ("n", "new"), ("s", "save"), ("x", "delete"),
                ("Esc", "back"), ("?", "help"), ("q", "quit"),
            ]),
            Tab::Stats => keybinds(&[
                ("F1-F4", "tabs"), ("?", "help"), ("q", "quit"),
            ]),
        }
    };

    let p = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(p, area);
}

fn keybinds<'a>(pairs: &[(&'a str, &'a str)]) -> Vec<Span<'a>> {
    let mut spans = vec![Span::raw(" ")];
    for (key, desc) in pairs {
        spans.push(Span::styled(*key, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}  ", desc), Style::default().fg(DARK_GREY)));
    }
    spans
}

fn render_stats(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DARK_GREY));

    let total_duration: f64 = state.queue.iter().map(|_| 180.0).sum(); // Approx 3min per track
    let hours = (total_duration / 3600.0) as i32;
    let minutes = ((total_duration % 3600.0) / 60.0) as i32;

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Library Stats", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        stat_line("🎵", Color::Cyan, format!("Total Tracks      {}", state.queue.len())),
        stat_line("⏱️", Color::Yellow, format!("Total Duration    {}h {}m", hours, minutes)),
        stat_line("📋", Color::Green, format!("Playlists         {}", state.playlists.playlists.len())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Playback Stats", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        stat_line("🔊", Color::Blue, format!("Volume            {:.0}%", state.playback.volume)),
        stat_line("🚀", Color::Blue, format!("Speed             {:.1}x", state.playback.speed)),
        stat_line("🎚️", Color::Blue, format!("Pitch             {:.1}x", state.playback.pitch)),
        stat_line("🎵", Color::Cyan, format!("Bitrate           {:.0} kbps", state.playback.bitrate_kbps)),
        Line::from(""),
        stat_line("🔁", Color::Green, format!("Loop Mode         {}", if state.playback.is_loop { "enabled" } else { "disabled" })),
        stat_line("🔇", Color::Red, format!("Muted             {}", if state.playback.muted { "yes" } else { "no" })),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Current Track", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {}", state.playback.title), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:02}:{:02} / {:02}:{:02}",
                (state.playback.time/60.) as i32,
                (state.playback.time%60.) as i32,
                (state.playback.duration/60.) as i32,
                (state.playback.duration%60.) as i32),
                Style::default().fg(DARK_GREY)),
        ]),
        Line::from(""),
    ];

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn render_now_playing_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    let border_color = if state.focus == Focus::NowPlaying { ACCENT } else { DARK_GREY };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(area);

    // Progress bar and info
    let progress = if state.playback.duration > 0.0 {
        state.playback.time / state.playback.duration
    } else {
        0.0
    };

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .margin(1)
        .split(chunks[0]);

    // Track title
    let title_line = Line::from(vec![
        Span::styled("▶ ", Style::default().fg(ACCENT)),
        Span::styled(&state.playback.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(title_line), left_chunks[0]);

    // Progress bar
    let bar_width = left_chunks[2].width as usize;
    let filled = (progress * bar_width as f64) as usize;
    let bar = Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(ACCENT)),
        Span::styled("░".repeat(bar_width.saturating_sub(filled)), Style::default().fg(DARK_GREY)),
    ]);
    frame.render_widget(Paragraph::new(bar), left_chunks[1]);

    // Time
    let time_str = format!("{:02}:{:02} / {:02}:{:02}",
        (state.playback.time/60.) as i32,
        (state.playback.time%60.) as i32,
        (state.playback.duration/60.) as i32,
        (state.playback.duration%60.) as i32);
    frame.render_widget(Paragraph::new(Line::from(Span::styled(time_str, Style::default().fg(DARK_GREY)))), left_chunks[2]);

    // Right side: controls
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .margin(1)
        .split(chunks[1]);

    let controls = vec![
        Line::from(Span::styled(format!("🔊 {:.0}%", state.playback.volume), Style::default().fg(Color::Blue))),
        Line::from(Span::styled(format!("🚀 {:.1}x", state.playback.speed), Style::default().fg(Color::Blue))),
        Line::from(Span::styled(if state.playback.paused { "⏸️" } else { "▶️" }, Style::default().fg(ACCENT))),
    ];

    for (i, line) in controls.iter().enumerate() {
        frame.render_widget(Paragraph::new(line.clone()), right_chunks[i]);
    }

    frame.render_widget(block, area);
}

fn render_help_popup(frame: &mut Frame, area: Rect) {
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
            Span::styled("  Tabs", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
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
            Span::styled("  Navigation", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
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
            Span::styled("  Playback", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
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
            Span::styled("  Other", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
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
        Line::from(Span::styled("  Press ? or Esc to close", Style::default().fg(DARK_GREY))),
        Line::from(""),
    ];

    frame.render_widget(Paragraph::new(help_text), chunks[0]);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let w = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width: w, height: height.min(area.height) }
}
