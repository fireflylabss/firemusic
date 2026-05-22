use crate::audio::eq::EqState;
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute, queue,
    style::Stylize,
    terminal::{self, ClearType},
};
use libmpv2::{Mpv, events::Event as MEvent};
use std::io::{self, Write};
use std::time::Duration;

pub enum PlayLoopResult {
    Quit,
    SearchAgain,
    EndReached,
}

pub fn render_ui(mpv: &Mpv, is_loop: bool) -> Result<()> {
    let mut stdout = io::stdout();
    let (width, _) = terminal::size().unwrap_or((80, 24));
    let width = width as usize;

    let time = mpv.get_property::<f64>("time-pos").unwrap_or(0.0);
    let duration = mpv.get_property::<f64>("duration").unwrap_or(0.0);
    let paused = mpv.get_property::<bool>("pause").unwrap_or(false);
    let mute = mpv.get_property::<bool>("mute").unwrap_or(false);
    let volume = mpv.get_property::<f64>("volume").unwrap_or(0.0);
    let speed = mpv.get_property::<f64>("speed").unwrap_or(1.0);
    let pitch = mpv.get_property::<f64>("pitch").unwrap_or(1.0);
    let title = mpv
        .get_property::<String>("media-title")
        .unwrap_or_else(|_| "...".to_string());
    let bitrate = mpv.get_property::<f64>("audio-bitrate").unwrap_or(0.0) / 1000.0;

    let af = mpv.get_property::<String>("af").unwrap_or_default();
    let eq_label = if af.is_empty() {
        "off"
    } else if af.contains("bass") && af.contains("treble") {
        "rock"
    } else if af.contains("bass") {
        "bass+"
    } else if af.contains("treble") {
        "treble+"
    } else if af.contains("1000") && !af.contains("300") {
        "vocal"
    } else if af.contains("300") {
        "lofi"
    } else if af.contains("equalizer") {
        "custom"
    } else {
        "off"
    };

    let status_base = if paused { "\u{23F8}" } else { "\u{25B6}" };
    let status_str = if mute {
        format!("{} (mute)", status_base)
    } else {
        status_base.to_string()
    };
    let loop_tag = if is_loop { " \u{00B7} loop" } else { "" };
    let tech_tags = format!("  \u{00B7}  {:.0}kbps{}", bitrate, loop_tag);

    queue!(stdout, cursor::MoveToColumn(0))?;
    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let status_styled = format!("{} ", status_str);
    let available_for_title =
        width.saturating_sub(status_styled.chars().count() + tech_tags.chars().count() + 2);
    let display_title = if title.chars().count() > available_for_title {
        format!(
            "{}...",
            &title
                .chars()
                .take(available_for_title.saturating_sub(3))
                .collect::<String>()
        )
    } else {
        title
    };
    print!(
        "{}{}{}",
        status_styled.dark_red().bold(),
        display_title.white().bold(),
        tech_tags.dark_grey()
    );
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    let progress = if duration > 0.0 { time / duration } else { 0.0 };
    let time_str = format!(
        "{:02}:{:02} / {:02}:{:02}",
        (time / 60.) as i32,
        (time % 60.) as i32,
        (duration / 60.) as i32,
        (duration % 60.) as i32
    );
    let specs_str = format!(
        "  \u{00B7}  vol {:>3.0}%  \u{00B7}  spd {:.1}x  \u{00B7}  ptch {:.1}x  \u{00B7}  eq {}",
        volume, speed, pitch, eq_label
    );
    let bar_width = width
        .saturating_sub(time_str.len() + specs_str.len() + 4)
        .max(10);
    let filled = (progress * bar_width as f64) as usize;
    print!("{} | ", time_str.dark_grey());
    print!("{}", "\u{2501}".repeat(filled.saturating_sub(1)).white());
    if filled > 0 {
        print!("{}", "\u{2588}".dark_red());
    } else {
        print!("{}", "\u{2501}".white());
    }
    print!(
        "{}",
        "\u{2500}"
            .repeat(bar_width.saturating_sub(filled))
            .dark_grey()
    );
    print!("{}", specs_str.dark_grey());
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(0))?;

    execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

    let is_at_end = duration > 0.0 && time >= duration - 0.5;
    let show_s = is_at_end && !is_loop;

    let shortcuts = if show_s {
        "[space] pause | [\u{2190}/\u{2192}] seek | [\u{2191}/\u{2193}] vol | [+/-] spd | [,/.] pitch | [e] eq | [E] eq-mode | [s] search | [q] quit"
    } else {
        "[space] pause | [\u{2190}/\u{2192}] seek | [\u{2191}/\u{2193}] vol | [+/-] spd | [,/.] pitch | [e] eq | [E] eq-mode | [q] quit"
    };

    let pad = (width.saturating_sub(shortcuts.len()) / 2) as u16;
    queue!(stdout, cursor::MoveToColumn(pad))?;
    print!("{}", shortcuts.dark_grey());

    queue!(stdout, cursor::MoveToColumn(0), cursor::MoveUp(2))?;
    stdout.flush()?;
    Ok(())
}

pub fn play_loop(mpv: &mut Mpv, mut is_loop: bool) -> Result<PlayLoopResult> {
    loop {
        if let Some(event_result) = mpv.wait_event(0.0) {
            match event_result.map_err(|e| anyhow::anyhow!("mpv error: {:?}", e))? {
                MEvent::EndFile(_reason) => {
                    if !is_loop {
                        let remaining: i64 = mpv.get_property("playlist-count").unwrap_or(0);
                        let pos: i64 = mpv.get_property("playlist-pos").unwrap_or(0);
                        if pos + 1 >= remaining || pos < 0 {
                            return Ok(PlayLoopResult::EndReached);
                        }
                    }
                }
                MEvent::Shutdown => break,
                _ => {}
            }
        }
        if event::poll(Duration::from_millis(0))? {
            if let CEvent::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('s') => {
                        let time: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
                        let duration: f64 = mpv.get_property("duration").unwrap_or(0.0);
                        let is_at_end = duration > 0.0 && time >= duration - 1.0;
                        if is_at_end && !is_loop {
                            return Ok(PlayLoopResult::SearchAgain);
                        }
                    }
                    KeyCode::Char('e') => {
                        let af: String = mpv.get_property("af").unwrap_or_default();
                        if af.is_empty() {
                            mpv.set_property("af", "bass=g=10").ok();
                        } else if af.contains("bass") && !af.contains("treble") {
                            mpv.set_property("af", "treble=g=10").ok();
                        } else if af.contains("treble") && !af.contains("bass") {
                            mpv.set_property("af", "bass=g=10,treble=g=10").ok();
                        } else if af.contains("bass") && af.contains("treble") {
                            mpv.set_property("af", "equalizer=f=1000:width_type=h:width=200:g=10")
                                .ok();
                        } else if af.contains("1000") {
                            mpv.set_property(
                                "af",
                                "equalizer=f=300:width_type=h:width=200:g=-10,equalizer=f=3000:width_type=h:width=200:g=-10",
                            )
                            .ok();
                        } else {
                            mpv.set_property("af", "").ok();
                        }
                    }
                    KeyCode::Char('E') => {
                        eq_mode_overlay(mpv)?;
                    }
                    KeyCode::Char(',') => {
                        let cur: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                        mpv.set_property("pitch", (cur - 0.05).max(0.5)).ok();
                    }
                    KeyCode::Char('.') => {
                        let cur: f64 = mpv.get_property("pitch").unwrap_or(1.0);
                        mpv.set_property("pitch", (cur + 0.05).min(2.0)).ok();
                    }
                    KeyCode::Char('{') => {
                        mpv.command("seek", &["-60", "relative"]).ok();
                    }
                    KeyCode::Char('}') => {
                        mpv.command("seek", &["60", "relative"]).ok();
                    }
                    KeyCode::Char(' ') => {
                        let p: bool = mpv.get_property("pause").unwrap_or(false);
                        mpv.set_property("pause", !p).ok();
                    }
                    KeyCode::Char('m') => {
                        let m: bool = mpv.get_property("mute").unwrap_or(false);
                        mpv.set_property("mute", !m).ok();
                    }
                    KeyCode::Char('l') => {
                        is_loop = !is_loop;
                        mpv.set_property("loop-file", if is_loop { "inf" } else { "no" })
                            .ok();
                    }
                    KeyCode::Char('0') => {
                        mpv.set_property("speed", 1.0).ok();
                        mpv.set_property("pitch", 1.0).ok();
                    }
                    KeyCode::Right => {
                        mpv.command("seek", &["5", "relative"]).ok();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        mpv.command("seek", &["-5", "relative"]).ok();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                        mpv.set_property("volume", (v + 5.0).min(100.0)).ok();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let v: f64 = mpv.get_property("volume").unwrap_or(100.0);
                        mpv.set_property("volume", (v - 5.0).max(0.0)).ok();
                    }
                    KeyCode::Char(c) if c.is_digit(10) && c != '0' => {
                        let pct = c.to_digit(10).unwrap() * 10;
                        mpv.command("seek", &[&pct.to_string(), "absolute-percent"])
                            .ok();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                        mpv.set_property("speed", (s + 0.1).min(10.0)).ok();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let s: f64 = mpv.get_property("speed").unwrap_or(1.0);
                        mpv.set_property("speed", (s - 0.1).max(0.1)).ok();
                    }
                    _ => {}
                }
            }
        }
        render_ui(mpv, is_loop)?;
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(PlayLoopResult::Quit)
}

pub fn eq_mode_overlay(mpv: &Mpv) -> Result<()> {
    let mut stdout = io::stdout();
    let mut eq = EqState::new();

    loop {
        let (width, _) = terminal::size().unwrap_or((80, 24));
        let w = width as usize;

        queue!(stdout, cursor::SavePosition)?;
        queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;

        print!(
            "{}",
            "\n".repeat(2.max(w as u16).saturating_sub(2) as usize)
        );

        let bar_width = (w.saturating_sub(14)).max(10);
        for i in 0..10 {
            let bar = eq.eq_bar(i, bar_width);
            let label = eq.eq_bar_label(i);
            if i == eq.selected_band {
                print!("  {}", ">".cyan());
            } else {
                print!("   ");
            }
            println!("{:>8} {}", label, bar);
        }

        let controls = format!(
            " [\u{2190}/\u{2192},h/l] band  [\u{2191}/\u{2193},k/j] gain  [r] reset  [s] save  [L] load  [esc/enter] back  presets: {}",
            EqState::list_presets().join(", ")
        );
        let pad = (w.saturating_sub(controls.len()) / 2) as u16;
        queue!(stdout, cursor::MoveToColumn(pad))?;
        println!("{}", controls.dark_grey());

        stdout.flush()?;

        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        eq.apply(mpv);
                        break;
                    }
                    KeyCode::Right => eq.next_band(),
                    KeyCode::Left => eq.prev_band(),
                    KeyCode::Up | KeyCode::Char('k') => {
                        eq.adjust_band(1.0);
                        eq.apply(mpv);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        eq.adjust_band(-1.0);
                        eq.apply(mpv);
                    }
                    KeyCode::Char('h') => eq.prev_band(),
                    KeyCode::Char('l') => eq.next_band(),
                    KeyCode::Char('r') => {
                        eq.reset();
                        eq.apply(mpv);
                    }
                    KeyCode::Char('s') => {
                        execute!(stdout, cursor::RestorePosition)?;
                        queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
                        print!(" save preset name: ");
                        stdout.flush()?;
                        let mut name = String::new();
                        read_line_raw(&mut name)?;
                        let name = name.trim();
                        if !name.is_empty() {
                            if let Err(e) = eq.save_preset(name) {
                                println!("  error: {}", e);
                            } else {
                                println!("  saved '{}'", name.cyan());
                            }
                            std::thread::sleep(Duration::from_millis(600));
                        }
                        continue;
                    }
                    KeyCode::Char('L') => {
                        let presets = EqState::list_presets();
                        if presets.is_empty() {
                            continue;
                        }
                        execute!(stdout, cursor::RestorePosition)?;
                        queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
                        for (i, p) in presets.iter().enumerate() {
                            println!("  {}  {}", format!("[{}]", i + 1).cyan(), p);
                        }
                        print!(" load preset (1-{}): ", presets.len());
                        stdout.flush()?;
                        let mut input = String::new();
                        read_line_raw(&mut input)?;
                        if let Ok(idx) = input.trim().parse::<usize>() {
                            if idx > 0 && idx <= presets.len() {
                                if let Ok(loaded) = EqState::load_preset(&presets[idx - 1]) {
                                    eq = loaded;
                                    eq.apply(mpv);
                                }
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            }
        }
        execute!(stdout, cursor::RestorePosition)?;
    }
    Ok(())
}

fn read_line_raw(buf: &mut String) -> Result<()> {
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        buf.clear();
                        break;
                    }
                    KeyCode::Char(c) => {
                        buf.push(c);
                        print!("{}", c);
                        io::stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        buf.pop();
                        print!("\x08 \x08");
                        io::stdout().flush()?;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
