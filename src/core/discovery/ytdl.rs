use super::tiktok::search_tiktok;
use super::types::{normalize_soundcloud_url, SearchProvider, SearchResult, PROVIDERS};
use std::process::Command;

pub(crate) fn playlist_items_range(page: usize) -> String {
    format!("{}-{}", (page - 1) * 10 + 1, page * 10)
}

pub(crate) fn build_search_query(provider: &SearchProvider, query: &str, page: usize) -> String {
    if provider.id == "ym" {
        format!("{}{}{}", provider.prefix, query, provider.suffix)
    } else {
        format!("{}{}:{}", provider.prefix, 10 * page, query)
    }
}

pub(crate) fn parse_search_lines(out: &str, pid: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for line in out.lines() {
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
    results
}

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
        let items_range = playlist_items_range(page);
        args.extend(["--playlist-items", &items_range]);

        if provider.id == "yt" || provider.id == "sc" {
            args.push("--flat-playlist");
        }

        let query_str = build_search_query(provider, query, page);
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
            results.extend(parse_search_lines(&out_str, pid));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playlist_items_range_pages() {
        assert_eq!(playlist_items_range(1), "1-10");
        assert_eq!(playlist_items_range(2), "11-20");
        assert_eq!(playlist_items_range(3), "21-30");
    }

    #[test]
    fn build_search_query_youtube() {
        let yt = &PROVIDERS[0];
        assert_eq!(build_search_query(yt, "daft punk", 1), "ytsearch10:daft punk");
        assert_eq!(build_search_query(yt, "daft punk", 2), "ytsearch20:daft punk");
    }

    #[test]
    fn build_search_query_youtube_music() {
        let ym = &PROVIDERS[1];
        assert_eq!(
            build_search_query(ym, "lofi", 1),
            "https://music.youtube.com/search?q=lofi#songs"
        );
    }

    #[test]
    fn parse_search_lines_assigns_provider() {
        let line = r#"{"title":"Track","url":"https://example.com/track"}"#;
        let results = parse_search_lines(line, "yt");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].provider, "yt");
        assert_eq!(results[0].title, "Track");
    }
}