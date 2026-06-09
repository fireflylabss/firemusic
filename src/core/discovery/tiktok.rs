use super::types::SearchResult;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::Duration;

pub fn search_tiktok(query: &str, page: usize) -> Result<Vec<SearchResult>> {
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

pub(crate) fn extract_tiktok_urls(body: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::extract_tiktok_urls;

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
}