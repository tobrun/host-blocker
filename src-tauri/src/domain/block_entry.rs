use super::hostname::Hostname;
use crate::hosts_file::constants::{SINK_V4, SINK_V6};

/// The four `/etc/hosts` lines that block one custom host:
/// IPv4 and IPv6 sinks for both the bare host and its `www.` variant.
///
/// The order is fixed: `0.0.0.0 host`, `0.0.0.0 www.host`, `:: host`,
/// `:: www.host`.
pub fn block_lines(host: &Hostname) -> Vec<String> {
    let bare = host.as_str();
    vec![
        format!("{SINK_V4} {bare}"),
        format!("{SINK_V4} www.{bare}"),
        format!("{SINK_V6} {bare}"),
        format!("{SINK_V6} www.{bare}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(input: &str) -> Vec<String> {
        block_lines(&Hostname::parse(input).unwrap())
    }

    #[test]
    fn emits_exactly_four_lines_in_order() {
        assert_eq!(
            lines("example.com"),
            vec![
                "0.0.0.0 example.com",
                "0.0.0.0 www.example.com",
                ":: example.com",
                ":: www.example.com",
            ]
        );
    }

    #[test]
    fn no_duplicate_when_input_already_had_www() {
        assert_eq!(lines("www.example.com"), lines("example.com"));
    }

    #[test]
    fn keeps_deeper_subdomains() {
        assert_eq!(
            lines("m.example.com"),
            vec![
                "0.0.0.0 m.example.com",
                "0.0.0.0 www.m.example.com",
                ":: m.example.com",
                ":: www.m.example.com",
            ]
        );
    }
}
