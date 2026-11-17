use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// A normalised, bare hostname that HostBlocker can block.
///
/// Invariants (established by [`Hostname::parse`]):
/// - lower-case and punycoded (IDNA)
/// - never an IP literal, `localhost`, or a single-label name
/// - at most one leading `www.` label has been stripped
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hostname(String);

impl Hostname {
    /// Parse a user-entered string (page URL, bare domain, anything in between)
    /// into the bare host it points at.
    pub fn parse(input: &str) -> AppResult<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(AppError::User("Enter a website address.".into()));
        }

        let with_scheme = if has_scheme(trimmed) {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        };

        let parsed = Url::parse(&with_scheme)
            .map_err(|_| AppError::User(format!("“{trimmed}” is not a valid website address.")))?;

        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(AppError::User(
                "Only http and https addresses are supported.".into(),
            ));
        }

        let host = match parsed.host() {
            Some(url::Host::Domain(domain)) => domain.to_string(),
            // `host()` returns IPv4/IPv6 variants for IP literals.
            Some(url::Host::Ipv4(_) | url::Host::Ipv6(_)) => {
                return Err(AppError::User(
                    "Enter a website address, not an IP address.".into(),
                ))
            }
            None => {
                return Err(AppError::User(format!(
                    "“{trimmed}” does not point at a website."
                )))
            }
        };
        // `url` already lower-cases and punycodes domains, but be explicit so
        // the invariant survives a dependency change.
        let host = host.to_lowercase();
        // A trailing dot is the DNS root label; the bare host omits it.
        let host = host.strip_suffix('.').unwrap_or(&host).to_string();

        if host == "localhost" {
            return Err(AppError::User("“localhost” cannot be blocked.".into()));
        }
        if !host.contains('.') {
            return Err(AppError::User(format!(
                "“{host}” is not a full website address (no domain suffix)."
            )));
        }

        let bare = host.strip_prefix("www.").unwrap_or(&host);
        if bare.is_empty() || !bare.contains('.') {
            return Err(AppError::User(format!(
                "“{trimmed}” is not a full website address (no domain suffix)."
            )));
        }

        Ok(Self(bare.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Hostname {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Hostname::parse(&value)
    }
}

impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.0
    }
}

/// True when the string starts with a RFC 3986 scheme followed by `://`,
/// e.g. `https://` or `ftp://`.
fn has_scheme(input: &str) -> bool {
    match input.find("://") {
        Some(idx) if idx > 0 => input[..idx].bytes().enumerate().all(|(i, b)| match b {
            b'a'..=b'z' | b'A'..=b'Z' => true,
            b'0'..=b'9' | b'+' | b'-' | b'.' => i > 0,
            _ => false,
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(input: &str) -> String {
        Hostname::parse(input).unwrap().as_str().to_string()
    }

    #[test]
    fn parses_full_page_urls() {
        assert_eq!(host("https://www.reddit.com/r/rust"), "reddit.com");
        assert_eq!(
            host("http://news.bbc.co.uk:8080/sport?x=1#frag"),
            "news.bbc.co.uk"
        );
    }

    #[test]
    fn parses_without_scheme() {
        assert_eq!(host("reddit.com/r/rust"), "reddit.com");
        assert_eq!(host("x.com"), "x.com");
    }

    #[test]
    fn lowercases_input() {
        assert_eq!(host("HTTPS://WWW.Reddit.COM/"), "reddit.com");
    }

    #[test]
    fn strips_one_leading_www() {
        assert_eq!(host("www.example.com"), "example.com");
        // Deeper subdomains stay exactly as typed.
        assert_eq!(host("m.example.com"), "m.example.com");
        assert_eq!(host("www.www.example.com"), "www.example.com");
        // `www` as the only subdomain still keeps a registrable remainder.
        assert_eq!(host("www.example.de"), "example.de");
    }

    #[test]
    fn punycodes_idn() {
        assert_eq!(host("https://bücher.de/shelf"), "xn--bcher-kva.de");
        assert_eq!(host("日本語.jp"), "xn--wgv71a119e.jp");
    }

    #[test]
    fn accepts_trailing_dot() {
        assert_eq!(host("example.com."), "example.com");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(host("  example.com  "), "example.com");
    }

    #[test]
    fn rejects_ip_literals() {
        assert!(Hostname::parse("192.168.0.1").is_err());
        assert!(Hostname::parse("http://1.2.3.4/path").is_err());
        assert!(Hostname::parse("[::1]").is_err());
        assert!(Hostname::parse("http://[2001:db8::1]/").is_err());
    }

    #[test]
    fn rejects_localhost_and_single_label() {
        assert!(Hostname::parse("localhost").is_err());
        assert!(Hostname::parse("http://localhost:3000").is_err());
        assert!(Hostname::parse("intranet").is_err());
        assert!(Hostname::parse("www.localhost").is_err());
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert!(Hostname::parse("").is_err());
        assert!(Hostname::parse("   ").is_err());
    }

    #[test]
    fn rejects_invalid_characters() {
        assert!(Hostname::parse("exa mple.com").is_err());
        assert!(Hostname::parse("example.com bad").is_err());
        assert!(Hostname::parse("exa%mple.com").is_err());
        assert!(Hostname::parse("http://exa[mple.com").is_err());
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert!(Hostname::parse("ftp://files.example.com").is_err());
        assert!(Hostname::parse("file:///etc/hosts").is_err());
        assert!(Hostname::parse("data:text/plain,example.com").is_err());
    }

    #[test]
    fn serde_round_trips_as_string() {
        let h = Hostname::parse("https://www.example.com/a").unwrap();
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, "\"example.com\"");
        let back: Hostname = serde_json::from_str(&json).unwrap();
        assert_eq!(back, h);
    }
}
