use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event as CEvent, KeyCode, KeyModifiers},
    execute, queue,
    style::Stylize,
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::io::{self, Write};
use std::time::Duration;

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!(
            "{}...",
            s.chars()
                .take(max_len.saturating_sub(3))
                .collect::<String>()
        )
    } else {
        s.to_string()
    }
}

pub fn tactical_select(
    prompt: &str,
    items: &[String],
    is_multi: bool,
) -> Result<Option<Vec<usize>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, cursor::Hide)?;

    let mut selected = HashSet::new();
    let mut hovered = 0;

    // Auto-paginate if there are more than 15 items to prevent blowing up the screen
    let page_size = if items.len() <= 15 { items.len() } else { 10 };
    let max_lines_needed = page_size + 4;

    for _ in 0..max_lines_needed {
        println!();
    }
    execute!(stdout, cursor::MoveUp(max_lines_needed as u16))?;

    let res = loop {
        queue!(stdout, cursor::SavePosition)?;
        queue!(
            stdout,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        let (term_width, _) = terminal::size().unwrap_or((80, 24));
        let term_width = term_width as usize;

        let page = hovered / page_size;
        let start_idx = page * page_size;
        let end_idx = (start_idx + page_size).min(items.len());

        let mut prompt_str = prompt.to_string();
        if items.len() > page_size {
            prompt_str = format!(
                "{} (Page {}/{})",
                prompt_str,
                page + 1,
                (items.len() + page_size - 1) / page_size
            );
        }

        let safe_prompt = truncate(&prompt_str, term_width);
        queue!(
            stdout,
            crossterm::style::Print(format!("{}\n", safe_prompt.yellow().bold()))
        )?;

        for idx in start_idx..end_idx {
            let is_hovered = idx == hovered;
            let is_selected = selected.contains(&idx);

            let prefix = if is_hovered {
                ">".cyan().bold()
            } else {
                " ".white()
            };
            let check = if is_multi {
                if is_selected {
                    "[x]".cyan()
                } else {
                    "[ ]".dark_grey()
                }
            } else {
                "   ".white()
            };

            let raw_item = &items[idx];
            let max_item_len = term_width.saturating_sub(6);
            let safe_item = truncate(raw_item, max_item_len);

            let item_styled = if is_hovered {
                safe_item.white().bold()
            } else {
                safe_item.dark_grey()
            };

            queue!(stdout, cursor::MoveToColumn(0))?;
            queue!(
                stdout,
                crossterm::style::Print(format!("{} {} {}\n", prefix, check, item_styled))
            )?;
        }

        if items.len() > page_size {
            queue!(stdout, cursor::MoveToColumn(0))?;
            queue!(
                stdout,
                crossterm::style::Print(format!(
                    "\n  {} use ←/→ to change pages\n",
                    "•".dark_grey()
                ))
            )?;
        }

        stdout.flush()?;

        if event::poll(Duration::from_millis(50))? {
            if let CEvent::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break None;
                }
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break None,
                    KeyCode::Up | KeyCode::Char('k') => {
                        if hovered > 0 {
                            hovered -= 1;
                        } else {
                            hovered = items.len() - 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if hovered < items.len() - 1 {
                            hovered += 1;
                        } else {
                            hovered = 0;
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        if items.len() > page_size {
                            let current_page = hovered / page_size;
                            if current_page > 0 {
                                hovered = (current_page - 1) * page_size;
                            } else {
                                hovered = ((items.len() - 1) / page_size) * page_size;
                            }
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if items.len() > page_size {
                            let current_page = hovered / page_size;
                            let last_page = (items.len() - 1) / page_size;
                            if current_page < last_page {
                                hovered = (current_page + 1) * page_size;
                            } else {
                                hovered = 0;
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        if is_multi {
                            if selected.contains(&hovered) {
                                selected.remove(&hovered);
                            } else {
                                selected.insert(hovered);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        selected.insert(hovered);
                        // Visual feedback before exit
                        queue!(stdout, cursor::RestorePosition)?;
                        queue!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
                        queue!(
                            stdout,
                            crossterm::style::Print(format!(
                                "{}\n",
                                truncate(&prompt_str, term_width).yellow().bold()
                            ))
                        )?;
                        for idx in start_idx..end_idx {
                            let is_hovered = idx == hovered;
                            let is_selected = selected.contains(&idx);
                            let prefix = if is_hovered {
                                ">".cyan().bold()
                            } else {
                                " ".white()
                            };
                            let check = if is_multi {
                                if is_selected {
                                    "[x]".cyan()
                                } else {
                                    "[ ]".dark_grey()
                                }
                            } else {
                                if is_selected {
                                    "[x]".cyan()
                                } else {
                                    "   ".white()
                                }
                            };
                            let item_styled = if is_hovered {
                                truncate(&items[idx], term_width.saturating_sub(6))
                                    .white()
                                    .bold()
                            } else {
                                truncate(&items[idx], term_width.saturating_sub(6)).dark_grey()
                            };
                            queue!(
                                stdout,
                                cursor::MoveToColumn(0),
                                crossterm::style::Print(format!(
                                    "{} {} {}\n",
                                    prefix, check, item_styled
                                ))
                            )?;
                        }
                        stdout.flush()?;
                        std::thread::sleep(Duration::from_millis(100));

                        let mut sorted: Vec<usize> = selected.into_iter().collect();
                        sorted.sort();
                        break Some(sorted);
                    }
                    _ => {}
                }
            }
        }
        execute!(stdout, cursor::RestorePosition)?;
    };

    execute!(stdout, cursor::RestorePosition)?;
    let page = hovered / page_size;
    let start_idx = page * page_size;
    let end_idx = (start_idx + page_size).min(items.len());
    let mut lines_to_move_down = 1 + (end_idx - start_idx);
    if items.len() > page_size {
        lines_to_move_down += 2;
    }

    execute!(
        stdout,
        cursor::MoveDown(lines_to_move_down as u16),
        cursor::MoveToColumn(0)
    )?;
    execute!(stdout, terminal::Clear(ClearType::FromCursorDown))?;
    execute!(stdout, cursor::Show)?;
    disable_raw_mode()?;
    Ok(res)
}
