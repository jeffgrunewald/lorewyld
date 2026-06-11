//! Cached, paginated access to the Open5e API.
//!
//! Raw pages land in a cache directory keyed by URL so re-runs are
//! offline and deterministic against a fixed upstream snapshot.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Page<T> {
    next: Option<String>,
    results: Vec<T>,
}

pub struct Fetcher {
    client: reqwest::blocking::Client,
    cache_dir: PathBuf,
}

impl Fetcher {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("creating cache dir {}", cache_dir.display()))?;
        let client = reqwest::blocking::Client::builder()
            .user_agent("lorewyld-bundle-gen/0.1")
            .build()?;
        Ok(Self { client, cache_dir })
    }

    fn cache_path(&self, url: &str) -> PathBuf {
        let sanitized: String = url
            .trim_start_matches("https://")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        self.cache_dir.join(format!("{sanitized}.json"))
    }

    fn get_raw(&self, url: &str) -> Result<String> {
        let path = self.cache_path(url);
        if let Ok(cached) = fs::read_to_string(&path) {
            return Ok(cached);
        }
        eprintln!("GET {url}");
        let body = self
            .client
            .get(url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .with_context(|| format!("fetching {url}"))?
            .text()?;
        fs::write(&path, &body).with_context(|| format!("caching {}", path.display()))?;
        Ok(body)
    }

    /// Fetches every page of a list endpoint, following `next` links.
    pub fn fetch_all<T: serde::de::DeserializeOwned>(&self, first_url: &str) -> Result<Vec<T>> {
        let mut url = first_url.to_string();
        let mut out = Vec::new();
        loop {
            let body = self.get_raw(&url)?;
            let page: Page<T> = serde_json::from_str(&body)
                .with_context(|| format!("decoding page from {url}"))?;
            out.extend(page.results);
            match page.next {
                Some(next) => url = next,
                None => return Ok(out),
            }
        }
    }
}
