use anyhow::Result;
use crossterm::{cursor, execute, style::Stylize};
use dialoguer::Input;
use std::io;

use super::types::{SearchResult, PROVIDERS};
use super::ytdl::search_providers;
use crate::core::download::handle_download;
use crate::core::mpv::{create_player, load_inputs, MpvConfig};
use crate::core::paths::validate_url;
use crate::core::player::{play_loop, PlayLoopResult};
use crate::core::tactical_select::tactical_select;

pub fn handle_search(mut query: String, loop_mode: bool, volume: f64, speed: f64) -> Result<()> {
    let mut active_providers;
    let mut current_page = 1;

    if query.is_empty() {
        query = Input::new()
            .with_prompt("\u{1F50D} search music")
            .interact_text()?;

        let p_labels: Vec<String> = PROVIDERS.iter().map(|p| p.label.to_string()).collect();
        if let Some(p_indices) = tactical_select("\u{1F310} select a provider", &p_labels, false)? {
            if !p_indices.is_empty() {
                active_providers = vec![PROVIDERS[p_indices[0]].id.to_string()];
            } else {
                active_providers = vec!["yt".to_string()];
            }
        } else {
            return Ok(());
        }
    } else if query.contains(':') {
        let (prefix, rest) = if let Some((p, r)) = query.split_once(':') {
            if PROVIDERS.iter().any(|prov| prov.id == p) {
                (vec![p.to_string()], r.to_string())
            } else {
                (vec!["yt".to_string()], query.clone())
            }
        } else {
            (vec!["yt".to_string()], query.clone())
        };
        query = rest;
        active_providers = prefix;
    } else {
        active_providers = vec!["yt".to_string()];
    }

    loop {
        for pid in &active_providers {
            let provider = PROVIDERS.iter().find(|p| p.id == pid).unwrap();
            println!(
                "\u{1F50D} searching {} for \"{}\" (page {})...",
                provider.label.red(),
                query.clone().white().bold(),
                current_page
            );
        }

        let results = search_providers(&active_providers, &query, current_page);

        if results.is_empty() {
            println!("\u{274C} no results found.");
            let action_items = vec![
                "new search".to_string(),
                "previous page".to_string(),
                "quit".to_string(),
            ];
            let action = match tactical_select("no results. what now?", &action_items, false)? {
                Some(s) if !s.is_empty() => s[0],
                _ => 2,
            };
            match action {
                0 => {
                    query = Input::new()
                        .with_prompt("\u{1F50D} search music")
                        .interact_text()?;
                    current_page = 1;
                    continue;
                }
                1 => {
                    if current_page > 1 {
                        current_page -= 1;
                    }
                    continue;
                }
                _ => return Ok(()),
            }
        }

        let mut items: Vec<String> = results
            .iter()
            .map(format_result_line)
            .collect();

        items.push(
            "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
                .dark_grey()
                .to_string(),
        );
        items.push("[\u{2794} next page]".to_string());
        if current_page > 1 {
            items.push("[\u{2794} previous page]".to_string());
        }
        items.push("[\u{2794} change provider]".to_string());
        items.push("[\u{2794} new search]".to_string());

        let selections = match tactical_select(
            &format!(
                "\u{1F525} results for \"{}\" (Page {})",
                query.clone(),
                current_page
            ),
            &items,
            true,
        )? {
            Some(s) => s,
            None => return Ok(()),
        };

        if selections.is_empty() {
            return Ok(());
        }

        let mut final_urls = Vec::new();
        let mut switch_provider = false;
        let mut new_search = false;
        let mut next_page = false;
        let mut prev_page = false;

        for &idx in &selections {
            if idx < results.len() {
                let url = results[idx].get_playable_url();
                if !url.is_empty() {
                    final_urls.push(url);
                }
            } else {
                let label = &items[idx];
                if label.contains("next page") {
                    next_page = true;
                } else if label.contains("previous page") {
                    prev_page = true;
                } else if label.contains("change provider") {
                    switch_provider = true;
                } else if label.contains("new search") {
                    new_search = true;
                }
            }
        }

        if next_page {
            current_page += 1;
            continue;
        }
        if prev_page {
            if current_page > 1 {
                current_page -= 1;
            }
            continue;
        }

        if switch_provider {
            let p_labels: Vec<String> = PROVIDERS.iter().map(|p| p.label.to_string()).collect();
            if let Some(p_indices) =
                tactical_select("\u{1F310} select a provider", &p_labels, false)?
            {
                if !p_indices.is_empty() {
                    active_providers = vec![PROVIDERS[p_indices[0]].id.to_string()];
                    current_page = 1;
                }
            }
            continue;
        }

        if new_search {
            query = Input::new()
                .with_prompt("\u{1F50D} search music")
                .interact_text()?;
            current_page = 1;
            continue;
        }

        if final_urls.len() == 1 {
            let action_items = vec![
                "play now".to_string(),
                "select quality & play".to_string(),
                "download".to_string(),
                "back".to_string(),
            ];
            let action = match tactical_select(
                &format!("\u{1F3B5} selected: {}", results[selections[0]].title),
                &action_items,
                false,
            )? {
                Some(s) if !s.is_empty() => s[0],
                _ => continue,
            };

            match action {
                0 | 1 => {
                    let current_url = final_urls[0].clone();
                    let mut current_quality = "bestaudio/best".to_string();

                    if action == 1 {
                        let q_options = vec![
                            "high (best)".to_string(),
                            "medium (128k)".to_string(),
                            "low (data saving)".to_string(),
                        ];
                        if let Some(q_idx) =
                            tactical_select("\u{1F3A7} select audio quality", &q_options, false)?
                        {
                            current_quality = match q_idx[0] {
                                0 => "bestaudio/best".to_string(),
                                1 => "bestaudio[abr<=128]/best".to_string(),
                                2 => "worstaudio/worst".to_string(),
                                _ => "bestaudio/best".to_string(),
                            };
                        }
                    }

                    validate_url(&current_url)?;

                    loop {
                        let config =
                            MpvConfig::for_stream(volume, speed, loop_mode, &current_quality);
                        let mut mpv = create_player(&config)?;
                        load_inputs(&mpv, &[current_url.clone()])?;

                        crossterm::terminal::enable_raw_mode().ok();
                        println!("\n\n\n");
                        execute!(io::stdout(), cursor::Hide, cursor::MoveUp(3))?;
                        let play_res = play_loop(&mut mpv, loop_mode)?;
                        execute!(
                            io::stdout(),
                            cursor::Show,
                            cursor::MoveToColumn(0),
                            cursor::MoveDown(3)
                        )?;
                        crossterm::terminal::disable_raw_mode().ok();

                        match play_res {
                            PlayLoopResult::SearchAgain => {
                                query = Input::new()
                                    .with_prompt("\u{1F50D} search music")
                                    .interact_text()?;
                                current_page = 1;
                                break;
                            }
                            PlayLoopResult::EndReached => {
                                let end_options = vec![
                                    "[\u{2794} repeat this track]".to_string(),
                                    "[\u{2794} new search]".to_string(),
                                    "[\u{2794} back to results]".to_string(),
                                    "[\u{2794} download this track]".to_string(),
                                    "[q] quit".to_string(),
                                ];
                                match tactical_select(
                                    "\u{1F3C1} track ended. what now?",
                                    &end_options,
                                    false,
                                )? {
                                    Some(s) if s[0] == 0 => continue,
                                    Some(s) if s[0] == 1 => {
                                        query = Input::new()
                                            .with_prompt("\u{1F50D} search music")
                                            .interact_text()?;
                                        current_page = 1;
                                        break;
                                    }
                                    Some(s) if s[0] == 2 => break,
                                    Some(s) if s[0] == 3 => {
                                        handle_download("interactive", vec![current_url.clone()])?;
                                        return Ok(());
                                    }
                                    _ => return Ok(()),
                                }
                            }
                            PlayLoopResult::Quit => return Ok(()),
                        }
                    }
                }
                2 => {
                    handle_download("interactive", final_urls)?;
                    return Ok(());
                }
                _ => continue,
            }
        } else {
            handle_download("interactive", final_urls)?;
            return Ok(());
        }
    }
}

fn format_result_line(r: &SearchResult) -> String {
    let dur = r
        .duration
        .map(|d| format!("({:02}:{:02})", (d / 60.) as i32, (d % 60.) as i32))
        .unwrap_or_default();
    let cc = if r.has_subs() {
        " [CC]".green().to_string()
    } else {
        "".to_string()
    };
    format!(
        "[{}] {} {}{} | {}",
        r.provider.clone().red(),
        r.title.clone().white(),
        dur.yellow(),
        cc,
        r.get_uploader().dark_grey()
    )
}