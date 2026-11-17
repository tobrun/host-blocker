//! StevenBlack/hosts: category selection, prebuilt-variant URLs, and the
//! parser that turns a downloaded hosts file into baseline entries.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Optional StevenBlack extensions. The unified base list is always on and
/// has no extension value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Extension {
    FakeNews,
    Gambling,
    Porn,
    Social,
}

/// Default canonical base for the raw list downloads.
pub const DEFAULT_BASE_URL: &str = "https://raw.githubusercontent.com/StevenBlack/hosts/master";

/// Sorted, de-duplicated slug of the selection, e.g. `fakenews-gambling`.
/// `None` for an empty selection (the plain unified list).
pub fn variant_slug(exts: &[Extension]) -> Option<String> {
    let mut sorted: Vec<Extension> = exts
        .iter()
        .copied()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    sorted.sort();
    if sorted.is_empty() {
        return None;
    }
    Some(
        sorted
            .iter()
            .map(|e| e.as_slug().to_string())
            .collect::<Vec<_>>()
            .join("-"),
    )
}

impl Extension {
    fn as_slug(self) -> &'static str {
        match self {
            Extension::FakeNews => "fakenews",
            Extension::Gambling => "gambling",
            Extension::Porn => "porn",
            Extension::Social => "social",
        }
    }
}

/// URL of the prebuilt list for the selection under `base`.
pub fn download_url_with_base(base: &str, exts: &[Extension]) -> String {
    match variant_slug(exts) {
        None => format!("{base}/hosts"),
        Some(slug) => format!("{base}/alternates/{slug}/hosts"),
    }
}

/// URL of the prebuilt list for the selection on GitHub.
pub fn download_url(exts: &[Extension]) -> String {
    download_url_with_base(DEFAULT_BASE_URL, exts)
}

/// Human-readable label written into the managed section header, e.g.
/// `stevenblack` or `stevenblack fakenews-gambling`.
pub fn baseline_label(exts: &[Extension]) -> String {
    match variant_slug(exts) {
        None => "stevenblack".to_string(),
        Some(slug) => format!("stevenblack {slug}"),
    }
}

/// Hosts a baseline line must not resolve (kept out of the baseline).
const SKIPPED_HOSTS: &[&str] = &[
    "0.0.0.0",
    "localhost",
    "localhost.localdomain",
    "localhost4",
    "localhost4.localdomain4",
    "localhost6",
    "localhost6.localdomain6",
    "local",
    "broadcasthost",
    "ip6-localhost",
    "ip6-loopback",
    "ip6-localnet",
    "ip6-mcastprefix",
    "ip6-allnodes",
    "ip6-allrouters",
    "ip6-allhosts",
];

/// Parse a StevenBlack-style hosts file, keeping one `0.0.0.0 <host>` line
/// per unique host of every valid entry, in first-seen order. These lines
/// are written into the managed section verbatim.
///
/// Dropped: comments, blank lines, inline comments, non-`0.0.0.0` lines,
/// `0.0.0.0 0.0.0.0`, loopback/localhost entries and malformed hosts.
pub fn parse_baseline(content: &str) -> Vec<String> {
    use crate::hosts_file::constants::SINK_V4;
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for raw in content.lines() {
        let line = raw.trim().trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.split('#').next().unwrap_or(line).trim();
        let mut fields = line.split_whitespace();
        let (Some(addr), Some(host), None) = (fields.next(), fields.next(), fields.next()) else {
            continue;
        };
        if addr != SINK_V4 {
            continue;
        }
        if !is_plausible_host(host) || SKIPPED_HOSTS.contains(&host) {
            continue;
        }
        if seen.insert(host.to_string()) {
            lines.push(format!("{SINK_V4} {host}"));
        }
    }
    lines
}

/// Minimal shape check for a hosts-file entry name (lists are plain ASCII).
fn is_plausible_host(host: &str) -> bool {
    !host.is_empty()
        && host.contains('.')
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !host.contains("..")
        && host
            .split('.')
            .all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Extension::*;

    #[test]
    fn url_for_no_extension_is_the_unified_list() {
        assert_eq!(
            download_url_with_base("https://example.test/repo", &[]),
            "https://example.test/repo/hosts"
        );
    }

    #[test]
    fn url_for_single_extensions() {
        for (ext, slug) in [
            (FakeNews, "fakenews"),
            (Gambling, "gambling"),
            (Porn, "porn"),
            (Social, "social"),
        ] {
            assert_eq!(
                download_url(&[ext]),
                format!("https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/{slug}/hosts")
            );
        }
    }

    #[test]
    fn url_for_all_four_in_canonical_order() {
        assert_eq!(
            variant_slug(&[Social, Porn, Gambling, FakeNews]).as_deref(),
            Some("fakenews-gambling-porn-social")
        );
    }

    #[test]
    fn slug_sorts_and_dedupes_unsorted_input() {
        assert_eq!(
            variant_slug(&[Porn, Gambling, Porn, Gambling]).as_deref(),
            Some("gambling-porn")
        );
        assert_eq!(variant_slug(&[]), None);
    }

    #[test]
    fn real_default_urls_exist() {
        // Smoke-check the canonical shapes against the upstream repository.
        assert_eq!(
            download_url(&[]),
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"
        );
        assert_eq!(
            download_url(&[FakeNews, Gambling]),
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/alternates/fakenews-gambling/hosts"
        );
    }

    #[test]
    fn baseline_labels() {
        assert_eq!(baseline_label(&[]), "stevenblack");
        assert_eq!(baseline_label(&[Gambling]), "stevenblack gambling");
    }

    fn parsed(content: &str) -> Vec<String> {
        parse_baseline(content)
    }

    #[test]
    fn parser_keeps_plain_0_0_0_0_entries() {
        assert_eq!(
            parsed("0.0.0.0 ads.example.com\n0.0.0.0 tracker.example.net"),
            vec!["0.0.0.0 ads.example.com", "0.0.0.0 tracker.example.net"]
        );
    }

    #[test]
    fn parser_drops_comments_and_blanks() {
        assert_eq!(
            parsed("# header\n\n   \n0.0.0.0 a.example.com\n  # 0.0.0.0 commented"),
            vec!["0.0.0.0 a.example.com"]
        );
    }

    #[test]
    fn parser_drops_inline_comments_but_keeps_the_host() {
        assert_eq!(
            parsed("0.0.0.0 a.example.com # added 2024"),
            vec!["0.0.0.0 a.example.com"]
        );
    }

    #[test]
    fn parser_drops_zero_address_and_localhost_entries() {
        assert_eq!(
            parsed(
                &[
                    "0.0.0.0 0.0.0.0",
                    "127.0.0.1 localhost",
                    "127.0.1.1 myhost.local",
                    "0.0.0.0 localhost",
                    "0.0.0.0 localhost.localdomain",
                    "0.0.0.0 broadcasthost",
                    "::1 localhost",
                    "0.0.0.0 real.example.com",
                ]
                .join("\n"),
            ),
            vec!["0.0.0.0 real.example.com"]
        );
    }

    #[test]
    fn parser_accepts_valid_host_shapes() {
        assert_eq!(parsed("0.0.0.0 a.b"), ["0.0.0.0 a.b"]);
        assert_eq!(parsed("0.0.0.0 9.9"), ["0.0.0.0 9.9"]);
        assert_eq!(parsed("0.0.0.0 1.2.3.4"), ["0.0.0.0 1.2.3.4"]);
        assert_eq!(
            parsed("0.0.0.0 1.2.3.4.example"),
            ["0.0.0.0 1.2.3.4.example"]
        );
        assert_eq!(
            parsed("0.0.0.0 a-b.example.com"),
            ["0.0.0.0 a-b.example.com"]
        );
        assert_eq!(
            parsed("0.0.0.0 a_b.example.com"),
            ["0.0.0.0 a_b.example.com"]
        );
        assert_eq!(parsed("0.0.0.0 -a.example.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 a-.example.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 a.-b"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 a.b-"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 a..b"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 .example.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 example.com."), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 exam_ple.com"), ["0.0.0.0 exam_ple.com"]);
        assert_eq!(parsed("0.0.0.0 exam ple.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 é.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 exa*mple.com"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 example/com"), Vec::<String>::new());
        // A fourth field makes the line ambiguous: drop it.
        assert_eq!(parsed("0.0.0.0 a.b extra"), Vec::<String>::new());
        assert_eq!(parsed("0.0.0.0 9a.b9.c"), ["0.0.0.0 9a.b9.c"]);
    }

    #[test]
    fn parser_drops_malformed_lines() {
        assert_eq!(
            parsed(
                &[
                    "0.0.0.0",     // address only
                    "0.0.0.0 a b", // extra field
                    "0.0.0.0 nodots",
                    "0.0.0.0 ..bad..",
                    "0.0.0.0 -bad.example.com",
                    "0.0.0.0 bad-.example.com",
                    "not-an-ip ok.example.com",
                    "0.0.0.0 good.example.com",
                ]
                .join("\n"),
            ),
            vec!["0.0.0.0 good.example.com"]
        );
    }

    #[test]
    fn parser_dedupes_preserving_first_seen_order() {
        assert_eq!(
            parsed("0.0.0.0 b.example.com\n0.0.0.0 a.example.com\n0.0.0.0 b.example.com"),
            vec!["0.0.0.0 b.example.com", "0.0.0.0 a.example.com"]
        );
    }

    #[test]
    fn parser_handles_crlf() {
        assert_eq!(
            parsed("0.0.0.0 a.example.com\r\n0.0.0.0 b.example.com\r\n"),
            vec!["0.0.0.0 a.example.com", "0.0.0.0 b.example.com"]
        );
    }

    #[test]
    fn extension_serde_is_lowercase() {
        assert_eq!(serde_json::to_string(&Porn).unwrap(), "\"porn\"");
        let exts: Vec<Extension> = serde_json::from_str("[\"social\",\"fakenews\"]").unwrap();
        assert_eq!(exts, vec![Social, FakeNews]);
    }
}
