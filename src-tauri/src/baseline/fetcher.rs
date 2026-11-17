//! Downloads StevenBlack prebuilt lists over HTTPS.
//!
//! The base URL is injectable so integration tests can point this at a
//! local HTTP stub instead of GitHub.

use std::time::Duration;

use crate::baseline::stevenblack::{download_url_with_base, parse_baseline, Extension};
use crate::error::{AppError, AppResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub struct Fetcher {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Fetcher {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .user_agent(concat!("HostBlocker/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("reqwest client"),
        }
    }

    pub fn url_for(&self, exts: &[Extension]) -> String {
        download_url_with_base(&self.base_url, exts)
    }

    /// Download and parse the list for the selected extensions.
    /// Blocking; callers run this off the main thread.
    pub fn fetch(&self, exts: &[Extension]) -> AppResult<Vec<String>> {
        let url = self.url_for(exts);
        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| AppError::Network(format!("could not download {url}: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            return Err(AppError::Network(format!(
                "{url} responded with HTTP {status}"
            )));
        }
        let body = response
            .text()
            .map_err(|e| AppError::Network(format!("could not read {url}: {e}")))?;
        let hosts = parse_baseline(&body);
        if hosts.is_empty() {
            return Err(AppError::Network(format!(
                "{url} contained no usable entries"
            )));
        }
        Ok(hosts)
    }
}
