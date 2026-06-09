use super::tiktok::search_tiktok;
use super::types::{normalize_soundcloud_url, SearchResult, PROVIDERS};
use std::process::Command;

pub fn search_providers(pids: &[String], query: &str, page: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for pid in pids {
        let provider = PROVIDERS.iter().find(|p| p.id == pid).unwrap();

        if provider.id == "tk" {
            match search_tiktok(query, page) {
                Ok(mut found) => results.append(&mut found),
                Err(err) => eprintln!("tiktok search error: {}", err),
            }
            continue;
        }

        let mut args = vec!["-j"];
        let items_range = format!("{}-{}", (page - 1) * 10 + 1, page * 10);
        args.extend(["--playlist-items", &items_range]);

        if provider.id == "yt" || provider.id == "sc" {
            args.push("--flat-playlist");
        }

        let query_str = if provider.id == "ym" {
            format!("{}{}{}", provider.prefix, query, provider.suffix)
        } else {
            format!("{}{}:{}", provider.prefix, 10 * page, query)
        };
        args.push(&query_str);

        let output = match Command::new("yt-dlp").args(&args).output() {
            Ok(output) => output,
            Err(err) => {
                eprintln!("yt-dlp error for {}: {}", pid, err);
                continue;
            }
        };

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

    results
}