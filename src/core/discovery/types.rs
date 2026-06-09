use serde::Deserialize;

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

pub(crate) fn normalize_soundcloud_url(result: &SearchResult) -> Option<String> {
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
    fn tiktok_id_fallback_uses_extractor_compatible_url() {
        let mut res = result("tk", None, None);
        res.id = Some("7519269824963317022".to_string());
        assert_eq!(
            res.get_playable_url(),
            "https://www.tiktok.com/@_/video/7519269824963317022"
        );
    }
}