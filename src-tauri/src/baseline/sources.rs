//! Catalog of the third-party blocklists a baseline can come from.
//!
//! Every source is a single curated list of a manageable size (the whole
//! point is `/etc/hosts`, where very large lists degrade every DNS lookup).
//! New sources only need a variant here plus a URL and format entry.

use serde::{Deserialize, Serialize};

use crate::baseline::stevenblack::{download_url_with_base, Extension, DEFAULT_BASE_URL};

/// How the downloaded file is laid out on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormat {
    /// Classic hosts file (`0.0.0.0 <host>` or `127.0.0.1 <host>` lines).
    Hosts,
    /// Adblock Plus element-hiding rules (`||<host>^` lines).
    Adblock,
}

/// One selectable baseline list. The serde names are stable wire
/// identifiers: they appear in Tauri commands, the UI and state files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceId {
    #[serde(rename = "stevenblack")]
    StevenBlack,
    #[serde(rename = "oisd-small")]
    OisdSmall,
    #[serde(rename = "adaway")]
    AdAway,
    #[serde(rename = "blocklistproject-phishing")]
    BlocklistProjectPhishing,
}

impl SourceId {
    pub const ALL: [SourceId; 4] = [
        SourceId::StevenBlack,
        SourceId::OisdSmall,
        SourceId::AdAway,
        SourceId::BlocklistProjectPhishing,
    ];

    /// Display name shown in the UI and written into the hosts header.
    pub fn label(self) -> &'static str {
        match self {
            SourceId::StevenBlack => "stevenblack",
            SourceId::OisdSmall => "oisd small",
            SourceId::AdAway => "adaway",
            SourceId::BlocklistProjectPhishing => "blocklistproject phishing",
        }
    }

    pub fn homepage(self) -> &'static str {
        match self {
            SourceId::StevenBlack => "https://github.com/StevenBlack/hosts",
            SourceId::OisdSmall => "https://oisd.nl/",
            SourceId::AdAway => "https://adaway.org/",
            SourceId::BlocklistProjectPhishing => "https://github.com/blocklistproject/Lists",
        }
    }

    /// Category extensions are a StevenBlack-only concept.
    pub fn supports_extensions(self) -> bool {
        matches!(self, SourceId::StevenBlack)
    }

    pub fn format(self) -> ListFormat {
        match self {
            SourceId::OisdSmall => ListFormat::Adblock,
            _ => ListFormat::Hosts,
        }
    }

    /// Download URL. `base_override` (used by tests against a local HTTP
    /// stub) replaces each source's own origin, keeping the trailing path.
    pub fn url_with_base(self, base_override: Option<&str>, exts: &[Extension]) -> String {
        let origin = |production: &str, test_path: &str| match base_override {
            Some(base) => format!("{base}/{test_path}"),
            None => production.to_string(),
        };
        match self {
            SourceId::StevenBlack => {
                download_url_with_base(base_override.unwrap_or(DEFAULT_BASE_URL), exts)
            }
            SourceId::OisdSmall => origin("https://small.oisd.nl/", "oisd_hostsmall.txt"),
            SourceId::AdAway => origin("https://adaway.org/hosts.txt", "adaway.txt"),
            SourceId::BlocklistProjectPhishing => origin(
                "https://raw.githubusercontent.com/blocklistproject/Lists/master/phishing.txt",
                "phishing.txt",
            ),
        }
    }

    /// Production download URL for the given extension selection.
    pub fn url(self, exts: &[Extension]) -> String {
        self.url_with_base(None, exts)
    }

    /// Label written into the managed hosts section header, e.g.
    /// `stevenblack fakenews-gambling` or `oisd small`.
    pub fn baseline_label(self, exts: &[Extension]) -> String {
        let exts = if self.supports_extensions() {
            exts
        } else {
            &[]
        };
        match crate::baseline::stevenblack::variant_slug(exts) {
            None => self.label().to_string(),
            Some(slug) => format!("{} {slug}", self.label()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Extension::Gambling;

    #[test]
    fn production_urls() {
        assert_eq!(
            SourceId::StevenBlack.url(&[]),
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"
        );
        assert_eq!(
            SourceId::StevenBlack.url(&[Gambling]),
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/gambling/hosts"
        );
        assert_eq!(SourceId::OisdSmall.url(&[]), "https://small.oisd.nl/");
        assert_eq!(SourceId::AdAway.url(&[]), "https://adaway.org/hosts.txt");
        assert_eq!(
            SourceId::BlocklistProjectPhishing.url(&[]),
            "https://raw.githubusercontent.com/blocklistproject/Lists/master/phishing.txt"
        );
    }

    #[test]
    fn base_override_replaces_origin_for_every_source() {
        for source in SourceId::ALL {
            let url = source.url_with_base(Some("http://127.0.0.1:9999"), &[]);
            assert!(
                url.starts_with("http://127.0.0.1:9999/"),
                "{source:?} -> {url}"
            );
        }
        // StevenBlack keeps its alternates path under the override.
        assert_eq!(
            SourceId::StevenBlack.url_with_base(Some("http://stub"), &[Gambling]),
            "http://stub/alternates/gambling/hosts"
        );
        // Extensions never leak into extension-less sources.
        assert_eq!(
            SourceId::OisdSmall.url_with_base(Some("http://stub"), &[Gambling]),
            "http://stub/oisd_hostsmall.txt"
        );
    }

    #[test]
    fn only_stevenblack_supports_extensions() {
        assert!(SourceId::StevenBlack.supports_extensions());
        assert!(!SourceId::OisdSmall.supports_extensions());
    }

    #[test]
    fn labels_append_extensions_only_for_supported_sources() {
        assert_eq!(
            SourceId::StevenBlack.baseline_label(&[Gambling]),
            "stevenblack gambling"
        );
        assert_eq!(
            SourceId::OisdSmall.baseline_label(&[Gambling]),
            "oisd small"
        );
    }

    #[test]
    fn serde_ids_are_stable_wire_names() {
        assert_eq!(
            serde_json::to_string(&SourceId::BlocklistProjectPhishing).unwrap(),
            "\"blocklistproject-phishing\""
        );
        assert_eq!(
            serde_json::to_string(&SourceId::StevenBlack).unwrap(),
            "\"stevenblack\""
        );
        assert_eq!(
            serde_json::to_string(&SourceId::AdAway).unwrap(),
            "\"adaway\""
        );
        let id: SourceId = serde_json::from_str("\"oisd-small\"").unwrap();
        assert_eq!(id, SourceId::OisdSmall);
    }

    #[test]
    fn formats_match_parsers() {
        assert_eq!(SourceId::OisdSmall.format(), ListFormat::Adblock);
        assert_eq!(SourceId::AdAway.format(), ListFormat::Hosts);
    }

    #[test]
    fn homepages_are_the_upstream_projects() {
        // Shown next to each list in the UI: pin them to the real projects.
        assert_eq!(
            SourceId::StevenBlack.homepage(),
            "https://github.com/StevenBlack/hosts"
        );
        assert_eq!(SourceId::OisdSmall.homepage(), "https://oisd.nl/");
        assert_eq!(SourceId::AdAway.homepage(), "https://adaway.org/");
        assert_eq!(
            SourceId::BlocklistProjectPhishing.homepage(),
            "https://github.com/blocklistproject/Lists"
        );
    }
}
