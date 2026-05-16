use anyhow::Result;
use crossterm::{cursor, execute, style::Stylize};
use dialoguer::Input;
use serde::Deserialize;
use std::io;
use std::process::Command;

use crate::player::{play_loop, PlayLoopResult};
use crate::tactical_select::tactical_select;
use crate::download::handle_download;

#[derive(Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: Option<String>,
    pub webpage_url: Option<String>,
    pub id: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub channel: Option<String>,
    pub artist: Option<String>,
    pub subtitles: Option<serde_json::Value>,
    pub automatic_captions: Option<serde_json::Value>,
    #[serde(skip)]
    pub provider: String,
}

impl SearchResult {
    pub fn has_subs(&self) -> bool {
        self.subtitles
            .as_ref()
            .map_or(false, |s| s.is_object() && !s.as_object().unwrap().is_empty())
            || self
                .automatic_captions
                .as_ref()
                .map_or(false, |s| s.is_object() && !s.as_object().unwrap().is_empty())
    }

    pub fn get_playable_url(&self) -> String {
        if let Some(u) = &self.url {
            if u.starts_with("http") {
                return u.clone();
            }
        }
        if let Some(u) = &self.webpage_url {
            if u.starts_with("http") {
                return u.clone();
            }
        }
        if let Some(id) = &self.id {
            if self.provider == "yt" || self.provider == "ym" {
                return format!("https://www.youtube.com/watch?v={}", id);
            }
        }
        self.url.clone().unwrap_or_default()
    }

    pub fn get_uploader(&self) -> &str {
        if let Some(a) = &self.artist {
            return a;
        }
        if let Some(c) = &self.channel {
            return c;
        }
        if let Some(u) = &self.uploader {
            return u;
        }
        "?"
    }
}

pub struct SearchProvider {
    pub id: &'static str,
    pub label: &'static str,
    pub prefix: &'static str,
    pub suffix: &'static str,
}

pub const PROVIDERS: &[SearchProvider] = &[
    SearchProvider {
        id: "yt",
        label: "youtube",
        prefix: "ytsearch",
        suffix: "",
    },
    SearchProvider {
        id: "ym",
        label: "youtube-music",
        prefix: "https://music.youtube.com/search?q=",
        suffix: "#songs",
    },
    SearchProvider {
        id: "sc",
        label: "soundcloud",
        prefix: "scsearch",
        suffix: "",
    },
];

#[derive(Deserialize, Debug)]
pub struct YtdlFormat {
    #[allow(dead_code)]
    pub format_id: String,
    #[allow(dead_code)]
    pub ext: String,
    pub resolution: Option<String>,
    pub vcodec: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct YtdlInfo {
    pub title: String,
    pub formats: Vec<YtdlFormat>,
}

pub fn handle_search(mut query: String, loop_mode: bool, volume: f64, speed: f64) -> Result<()> {
    let mut active_providers;
    let mut current_page = 1;

    if query.is_empty() {
        query = Input::new()
            .with_prompt("\u{1F50D} search music")
            .interact_text()?;

        let p_labels: Vec<String> = PROVIDERS.iter().map(|p| p.label.to_string()).collect();
        if let Some(p_indices) =
            tactical_select("\u{1F310} select a provider", &p_labels, false)?
        {
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
        let mut results = Vec::new();
        for pid in &active_providers {
            let provider = PROVIDERS.iter().find(|p| p.id == pid).unwrap();
            println!(
                "\u{1F50D} searching {} for \"{}\" (page {})...",
                provider.label.cyan(),
                query.clone().white().bold(),
                current_page
            );

            let mut args = vec!["-j"];
            let items_range = format!(
                "{}-{}",
                (current_page - 1) * 10 + 1,
                current_page * 10
            );
            args.extend(["--playlist-items", &items_range]);

            if provider.id == "yt" || provider.id == "sc" {
                args.push("--flat-playlist");
            }

            let query_str = if provider.id == "ym" {
                format!("{}{}{}", provider.prefix, query, provider.suffix)
            } else {
                format!("{}{}:{}", provider.prefix, 10 * current_page, query)
            };
            args.push(&query_str);

            let output = Command::new("yt-dlp").args(&args).output()?;

            if output.status.success() {
                let out_str = String::from_utf8_lossy(&output.stdout);
                for line in out_str.lines() {
                    match serde_json::from_str::<SearchResult>(line) {
                        Ok(mut res) => {
                            res.provider = pid.to_string();
                            results.push(res);
                        }
                        Err(_) => {}
                    }
                }
            } else {
                eprintln!(
                    "yt-dlp error for {}: {}",
                    pid,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

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
            .map(|r| {
                let dur = r
                    .duration
                    .map(|d| {
                        format!(
                            "({:02}:{:02})",
                            (d / 60.) as i32,
                            (d % 60.) as i32
                        )
                    })
                    .unwrap_or_default();
                let cc = if r.has_subs() {
                    " [CC]".green().to_string()
                } else {
                    "".to_string()
                };
                format!(
                    "[{}] {} {}{} | {}",
                    r.provider.clone().cyan(),
                    r.title.clone().white(),
                    dur.yellow(),
                    cc,
                    r.get_uploader().dark_grey()
                )
            })
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
                        if let Some(q_idx) = tactical_select(
                            "\u{1F3A7} select audio quality",
                            &q_options,
                            false,
                        )? {
                            current_quality = match q_idx[0] {
                                0 => "bestaudio/best".to_string(),
                                1 => "bestaudio[abr<=128]/best".to_string(),
                                2 => "worstaudio/worst".to_string(),
                                _ => "bestaudio/best".to_string(),
                            };
                        }
                    }

                    loop {
                        let mut mpv = libmpv2::Mpv::new()
                            .map_err(|e| anyhow::anyhow!("mpv init: {:?}", e))?;
                        mpv.set_property("video", "no").ok();
                        mpv.set_property("volume", volume).ok();
                        mpv.set_property("speed", speed).ok();
                        mpv.set_property("ytdl", "yes").ok();
                        mpv.set_property("ytdl-format", current_quality.as_str())
                            .ok();
                        if loop_mode {
                            mpv.set_property("loop-file", "inf").ok();
                        }
                        mpv.command("loadfile", &[&current_url, "replace"]).ok();

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
                                        handle_download(
                                            "interactive",
                                            vec![current_url.clone()],
                                        )?;
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
