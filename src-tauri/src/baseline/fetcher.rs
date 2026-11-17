//! Downloads the selected baseline list over HTTPS and turns it into
//! `0.0.0.0 <host>` lines.
//!
//! The base URL is injectable so integration tests can point every source
//! at a local HTTP stub instead of the real origins.

use std::time::Duration;

use crate::baseline::sources::SourceId;
use crate::baseline::stevenblack::{parse_baseline, parse_domain_list, Extension};
use crate::error::{AppError, AppResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// `/etc/hosts` is read on every lookup; lists beyond this size are refused
/// rather than making the whole machine slow.
pub const MAX_BASELINE_HOSTS: usize = 250_000;

pub struct Fetcher {
    base_override: Option<String>,
    client: reqwest::blocking::Client,
}

impl Fetcher {
    fn with_base(base_override: Option<String>) -> Self {
        Self {
            base_override,
            client: reqwest::blocking::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .user_agent(concat!("HostBlocker/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("reqwest client"),
        }
    }

    /// Fetch from the real origins (production).
    pub fn production() -> Self {
        Self::with_base(None)
    }

    /// Fetch every source from `base_url` (integration tests against a stub).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_base(Some(base_url.into()))
    }

    pub fn url_for(&self, source: SourceId, exts: &[Extension]) -> String {
        source.url_with_base(self.base_override.as_deref(), exts)
    }

    /// Download and parse the list for the given source (and StevenBlack
    /// extensions). Blocking; callers run this off the main thread.
    pub fn fetch(&self, source: SourceId, exts: &[Extension]) -> AppResult<Vec<String>> {
        let url = self.url_for(source, exts);
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
        let hosts = match source.format() {
            crate::baseline::sources::ListFormat::Hosts => parse_baseline(&body),
            crate::baseline::sources::ListFormat::Adblock => parse_domain_list(&body),
        };
        if hosts.is_empty() {
            return Err(AppError::Network(format!(
                "{url} contained no usable entries"
            )));
        }
        if hosts.len() > MAX_BASELINE_HOSTS {
            return Err(AppError::Network(format!(
                "{url} lists {} entries; HostBlocker supports at most {MAX_BASELINE_HOSTS} in /etc/hosts",
                hosts.len()
            )));
        }
        Ok(hosts)
    }
}
