use anyhow::{Result, anyhow};
use crossterm::{cursor, execute, style::Stylize};
use dialoguer::Input;
use serde::Deserialize;
use serde_json::Value;
use std::io;
use std::process::Command;
use std::time::Duration;

use crate::download::handle_download;
use crate::player::{PlayLoopResult, play_loop};
use crate::tactical_select::tactical_select;

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
    pub search_url: Option<String>,
    #[serde(skip)]
    pub provider: String,
}

impl SearchResult {
    pub fn has_subs(&self) -> bool {
        self.subtitles.as_ref().map_or(false, |s| {
            s.is_object() && !s.as_object().unwrap().is_empty()
        }) || self.automatic_captions.as_ref().map_or(false, |s| {
            s.is_object() && !s.as_object().unwrap().is_empty()
        })
    }

    pub fn get_playable_url(&self) -> String {
        if let Some(u) = &self.search_url {
            if u.starts_with("http") {
                return u.clone();
            }
        }
        if self.provider == "sc" || self.provider == "tk" {
            if let Some(u) = &self.webpage_url {
                if u.starts_with("http") {
                    return u.clone();
                }
            }
        }
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
        if self.provider == "tk" {
            if let Some(id) = &self.id {
                if id.chars().all(|c| c.is_ascii_digit()) {
                    return format!("https://www.tiktok.com/@_/video/{}", id);
                }
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
    SearchProvider {
        id: "tk",
        label: "tiktok",
        prefix: "",
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
    #[serde(default)]
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
        let mut results = Vec::new();
        for pid in &active_providers {
            let provider = PROVIDERS.iter().find(|p| p.id == pid).unwrap();
            println!(
                "\u{1F50D} searching {} for \"{}\" (page {})...",
                provider.label.red(),
                query.clone().white().bold(),
                current_page
            );

            if provider.id == "tk" {
                match search_tiktok(&query, current_page) {
                    Ok(mut found) => results.append(&mut found),
                    Err(err) => eprintln!("tiktok search error: {}", err),
                }
                continue;
            }

            let mut args = vec!["-j"];
            let items_range = format!("{}-{}", (current_page - 1) * 10 + 1, current_page * 10);
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
                            if res.provider == "sc" {
                                res.search_url = normalize_soundcloud_url(&res);
                            }
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

fn search_tiktok(query: &str, page: usize) -> Result<Vec<SearchResult>> {
    let offset = page.saturating_sub(1) * 10;
    let mut urls = Vec::new();

    if let Some(key) = std::env::var("BRAVE_SEARCH_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty())
    {
        urls.extend(search_tiktok_brave(query, offset, &key)?);
    }

    if urls.is_empty() {
        urls.extend(search_tiktok_bing(query, offset)?);
    }

    if urls.is_empty() {
        urls.extend(search_tiktok_duckduckgo(query, offset)?);
    }

    if urls.is_empty() {
        return Err(anyhow!(
            "no TikTok results returned; set BRAVE_SEARCH_API_KEY for reliable TikTok search"
        ));
    }

    Ok(urls
        .into_iter()
        .take(10)
        .map(|url| tiktok_result_from_url(&url))
        .collect())
}

fn search_tiktok_brave(query: &str, offset: usize, key: &str) -> Result<Vec<String>> {
    let q = format!(
        "site:tiktok.com/@ inurl:/video/ {}",
        query.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count=10&offset={}",
        urlencoding::encode(&q),
        offset
    );
    let body = http_client()?
        .get(url)
        .header("X-Subscription-Token", key)
        .send()?
        .error_for_status()?
        .text()?;
    let json: Value = serde_json::from_str(&body)?;

    let mut urls = Vec::new();
    if let Some(results) = json.pointer("/web/results").and_then(|v| v.as_array()) {
        for item in results {
            if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                push_tiktok_url(&mut urls, url);
            }
        }
    }
    Ok(urls)
}

fn search_tiktok_bing(query: &str, offset: usize) -> Result<Vec<String>> {
    let q = format!(
        "site:tiktok.com/@ inurl:/video/ {}",
        query.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    let url = format!(
        "https://www.bing.com/search?q={}&first={}",
        urlencoding::encode(&q),
        offset + 1
    );
    let body = http_client()?.get(url).send()?.error_for_status()?.text()?;
    Ok(extract_tiktok_urls(&body))
}

fn search_tiktok_duckduckgo(query: &str, offset: usize) -> Result<Vec<String>> {
    let q = format!(
        "site:tiktok.com/@ inurl:/video/ {}",
        query.split_whitespace().collect::<Vec<_>>().join(" ")
    );
    let url = format!(
        "https://duckduckgo.com/html/?q={}&s={}",
        urlencoding::encode(&q),
        offset
    );
    let body = http_client()?.get(url).send()?.error_for_status()?.text()?;

    if body.contains("anomaly.js") || body.contains("challenge-form") {
        return Ok(Vec::new());
    }

    let mut urls = extract_tiktok_urls(&body);
    for part in body.split("uddg=").skip(1) {
        let encoded = part
            .split('&')
            .next()
            .unwrap_or_default()
            .replace("&amp;", "");
        if let Ok(decoded) = urlencoding::decode(&encoded) {
            push_tiktok_url(&mut urls, decoded.as_ref());
        }
    }
    Ok(urls)
}

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36")
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(Into::into)
}

fn extract_tiktok_urls(body: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for marker in [
        "https://www.tiktok.com/@",
        "https%3A%2F%2Fwww.tiktok.com%2F%40",
    ] {
        for part in body.split(marker).skip(1) {
            let raw = format!("{}{}", marker, part);
            let raw = raw
                .split(['"', '\'', '<', '>', ' ', '\\'])
                .next()
                .unwrap_or_default()
                .replace("&amp;", "&");
            if let Ok(decoded) = urlencoding::decode(&raw) {
                push_tiktok_url(&mut urls, decoded.as_ref());
            } else {
                push_tiktok_url(&mut urls, &raw);
            }
        }
    }
    urls
}

fn push_tiktok_url(urls: &mut Vec<String>, url: &str) {
    let clean = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    if !clean.contains("tiktok.com/@") || !clean.contains("/video/") {
        return;
    }
    let Some(id) = clean.rsplit('/').next() else {
        return;
    };
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return;
    }
    let clean = clean.to_string();
    if !urls.iter().any(|u| u == &clean) {
        urls.push(clean);
    }
}

fn tiktok_result_from_url(url: &str) -> SearchResult {
    let id = url.rsplit('/').next().map(|s| s.to_string());
    let uploader = url
        .split("tiktok.com/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .filter(|s| s.starts_with('@'))
        .map(|s| s.to_string());
    let title = id
        .as_deref()
        .map(|id| format!("TikTok video {}", id))
        .unwrap_or_else(|| "TikTok video".to_string());
    SearchResult {
        title,
        url: Some(url.to_string()),
        webpage_url: Some(url.to_string()),
        id,
        duration: None,
        uploader,
        channel: None,
        artist: None,
        subtitles: None,
        automatic_captions: None,
        search_url: Some(url.to_string()),
        provider: "tk".to_string(),
    }
}

fn normalize_soundcloud_url(result: &SearchResult) -> Option<String> {
    result
        .webpage_url
        .as_ref()
        .filter(|url| url.starts_with("https://soundcloud.com/"))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(provider: &str, url: Option<&str>, webpage_url: Option<&str>) -> SearchResult {
        SearchResult {
            title: "title".to_string(),
            url: url.map(str::to_string),
            webpage_url: webpage_url.map(str::to_string),
            id: None,
            duration: None,
            uploader: None,
            channel: None,
            artist: None,
            subtitles: None,
            automatic_captions: None,
            search_url: None,
            provider: provider.to_string(),
        }
    }

    #[test]
    fn soundcloud_search_prefers_public_webpage_url() {
        let mut res = result(
            "sc",
            Some("https://api.soundcloud.com/tracks/soundcloud%3Atracks%3A1250206453"),
            Some("https://soundcloud.com/lofi-beats-1/example"),
        );
        res.search_url = normalize_soundcloud_url(&res);
        assert_eq!(
            res.get_playable_url(),
            "https://soundcloud.com/lofi-beats-1/example"
        );
    }

    #[test]
    fn tiktok_url_extraction_decodes_and_deduplicates() {
        let html = r#"
            <a href="https://www.tiktok.com/@artist/video/7519269824963317022?lang=en">one</a>
            <a href="https%3A%2F%2Fwww.tiktok.com%2F%40artist%2Fvideo%2F7519269824963317022%3Fis_from_webapp%3D1">two</a>
            <a href="https://www.tiktok.com/@artist/photo/7519269824963317022">photo</a>
        "#;
        assert_eq!(
            extract_tiktok_urls(html),
            vec!["https://www.tiktok.com/@artist/video/7519269824963317022"]
        );
    }

    #[test]
    fn tiktok_id_fallback_uses_extractor_compatible_url() {
        let mut res = result("tk", None, None);
        res.id = Some("7519269824963317022".to_string());
        assert_eq!(
            res.get_playable_url(),
            "https://www.tiktok.com/@_/video/7519269824963317022"
        );
    }
}
