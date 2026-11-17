//! Pure rendering of HostBlocker's managed section and its merge into an
//! existing `/etc/hosts` document. Everything outside the managed markers is
//! preserved byte-for-byte.

use crate::error::{AppError, AppResult};
use crate::hosts_file::constants::{BEGIN_MARKER, CUSTOM_HEADER, END_MARKER};

/// Render the managed section, ending without a trailing newline.
///
/// * `custom_lines` — the four-line entries for the user's own sites.
/// * `baseline` — optional `(label, hosts)` pair, e.g.
///   `("stevenblack fakenews-gambling", ["0.0.0.0 ad.example.net", …])`.
pub fn render_section(custom_lines: &[String], baseline: Option<(&str, &[String])>) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(BEGIN_MARKER.to_string());
    lines.push(CUSTOM_HEADER.to_string());
    lines.extend(custom_lines.iter().cloned());
    if let Some((label, hosts)) = baseline {
        lines.push(format!("# --- baseline: {label} ---"));
        lines.extend(hosts.iter().cloned());
    }
    lines.push(END_MARKER.to_string());
    lines.join("\n")
}

/// Merge `section` (or remove the managed section when `None`) into
/// `existing`, preserving everything outside the markers byte-for-byte.
///
/// Returns an error — and leaves the caller's file untouched — when the
/// markers are damaged (one without the other, duplicated, or crossed).
pub fn merge(existing: &str, section: Option<&str>) -> AppResult<String> {
    let begin_count = existing.matches(BEGIN_MARKER).count();
    let end_count = existing.matches(END_MARKER).count();

    if begin_count == 0 && end_count == 0 {
        return Ok(append_new(existing, section));
    }
    if begin_count != 1 || end_count != 1 {
        return Err(AppError::HostsFile(format!(
            "found {begin_count} begin markers and {end_count} end markers; \
             expected one of each"
        )));
    }

    let begin_idx = existing.find(BEGIN_MARKER).expect("counted above");
    let end_idx = existing.find(END_MARKER).expect("counted above");
    if end_idx < begin_idx {
        return Err(AppError::HostsFile(
            "the managed section's end marker appears before its begin marker".into(),
        ));
    }

    let prefix = &existing[..line_start(existing, begin_idx)];
    let suffix = &existing[line_end(existing, end_idx)..];
    let middle = match section {
        Some(s) => format!("{s}\n"),
        None => String::new(),
    };
    Ok(format!("{prefix}{middle}{suffix}"))
}

fn append_new(existing: &str, section: Option<&str>) -> String {
    let Some(section) = section else {
        // Nothing to remove; return the file unchanged.
        return existing.to_string();
    };
    if existing.is_empty() {
        return format!("{section}\n");
    }
    let sep = if existing.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{existing}{sep}{section}\n")
}

/// Index of the first byte of the line containing `idx`.
fn line_start(s: &str, idx: usize) -> usize {
    s[..idx].rfind('\n').map_or(0, |i| i + 1)
}

/// Index just past the newline that ends the line containing `idx`
/// (or the end of the string when the line is unterminated).
fn line_end(s: &str, idx: usize) -> usize {
    match s[idx..].find('\n') {
        Some(off) => idx + off + 1,
        None => s.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosts_file::constants::BASELINE_HEADER_PREFIX;

    fn custom() -> Vec<String> {
        vec![
            "0.0.0.0 example.com".into(),
            "0.0.0.0 www.example.com".into(),
            ":: example.com".into(),
            ":: www.example.com".into(),
        ]
    }

    fn section() -> String {
        render_section(
            &custom(),
            Some((
                "stevenblack fakenews-gambling",
                &["0.0.0.0 ad.example.net".into()],
            )),
        )
    }

    #[test]
    fn section_layout_matches_spec() {
        assert_eq!(
            section(),
            [
                "# >>> HostBlocker BEGIN (managed, do not edit)",
                "# --- custom ---",
                "0.0.0.0 example.com",
                "0.0.0.0 www.example.com",
                ":: example.com",
                ":: www.example.com",
                "# --- baseline: stevenblack fakenews-gambling ---",
                "0.0.0.0 ad.example.net",
                "# <<< HostBlocker END",
            ]
            .join("\n")
        );
    }

    #[test]
    fn renders_empty_custom_list_and_missing_baseline() {
        let s = render_section(&[], None);
        assert_eq!(s, [BEGIN_MARKER, CUSTOM_HEADER, END_MARKER].join("\n"));
    }

    #[test]
    fn appends_to_file_without_section() {
        let existing = "127.0.0.1 localhost\n::1 localhost\n";
        let merged = merge(existing, Some(&section())).unwrap();
        assert!(merged.starts_with(existing));
        assert!(merged.ends_with(&format!("{}\n", END_MARKER)));
        assert_eq!(merged.matches(BEGIN_MARKER).count(), 1);
    }

    #[test]
    fn replaces_existing_section_without_duplicating() {
        let first = merge("127.0.0.1 localhost\n", Some(&section())).unwrap();
        let new_section = render_section(&["0.0.0.0 other.com".into()], None);
        let second = merge(&first, Some(&new_section)).unwrap();
        assert_eq!(second.matches(BEGIN_MARKER).count(), 1);
        assert!(!second.contains("example.com"));
        assert!(second.contains("other.com"));
        assert!(second.starts_with("127.0.0.1 localhost\n\n"));
    }

    #[test]
    fn merging_twice_is_idempotent() {
        let once = merge("127.0.0.1 localhost\n", Some(&section())).unwrap();
        let twice = merge(&once, Some(&section())).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn preserves_content_around_markers_byte_for_byte() {
        let odd = "# my comment  \n\t weird \r\n127.0.1.1 myhost  # laptop\n\n";
        let merged = merge(odd, Some(&section())).unwrap();
        assert!(merged.starts_with(odd));
        // Replace once more; the odd prefix must still be byte-identical.
        let merged2 = merge(&merged, Some(&section())).unwrap();
        assert!(merged2.starts_with(odd));
    }

    #[test]
    fn preserves_content_after_the_section() {
        let existing = format!(
            "127.0.0.1 localhost\n{}\nold line\n{}\ntrailing 1.2.3.4 kept\n",
            BEGIN_MARKER, END_MARKER
        );
        let merged = merge(&existing, Some(&section())).unwrap();
        assert!(!merged.contains("old line"));
        assert!(merged.ends_with("trailing 1.2.3.4 kept\n"));
        assert!(merged.starts_with("127.0.0.1 localhost\n"));
    }

    #[test]
    fn handles_missing_trailing_newline() {
        let merged = merge("127.0.0.1 localhost", Some(&section())).unwrap();
        assert!(merged.starts_with("127.0.0.1 localhost\n\n"));
        assert!(merged.ends_with(&format!("{}\n", END_MARKER)));
    }

    #[test]
    fn handles_crlf_file() {
        let existing = "127.0.0.1 localhost\r\n::1 localhost\r\n";
        let merged = merge(existing, Some(&section())).unwrap();
        assert!(merged.starts_with(existing));
        let again = merge(&merged, Some(&section())).unwrap();
        assert_eq!(merged, again);
    }

    #[test]
    fn replaces_section_in_crlf_file_between_crlf_lines() {
        let existing = format!(
            "127.0.0.1 localhost\r\n{}\r\nold\r\n{}\r\nend\r\n",
            BEGIN_MARKER, END_MARKER
        );
        let merged = merge(&existing, Some(&section())).unwrap();
        assert!(merged.starts_with("127.0.0.1 localhost\r\n"));
        assert!(merged.ends_with("end\r\n"));
        // The LF-terminated replacement section is followed by the untouched
        // CRLF tail of the file.
        assert!(merged.ends_with(&format!("{END_MARKER}\nend\r\n")));
        assert!(!merged.contains("old"));
    }

    #[test]
    fn begin_without_end_is_an_error() {
        let existing = format!("127.0.0.1 localhost\n{BEGIN_MARKER}\n0.0.0.0 stale\n");
        let err = merge(&existing, Some(&section())).unwrap_err();
        assert!(matches!(err, AppError::HostsFile(_)));
    }

    #[test]
    fn end_without_begin_is_an_error() {
        let existing = format!("127.0.0.1 localhost\n{END_MARKER}\n");
        assert!(merge(&existing, Some(&section())).is_err());
    }

    #[test]
    fn duplicate_markers_are_an_error() {
        let existing = format!("{BEGIN_MARKER}\n{BEGIN_MARKER}\nx\n{END_MARKER}\n");
        assert!(merge(&existing, Some(&section())).is_err());
        let crossed = format!("{END_MARKER}\nx\n{BEGIN_MARKER}\n");
        assert!(merge(&crossed, Some(&section())).is_err());
    }

    #[test]
    fn removal_deletes_only_the_section() {
        let odd = "127.0.0.1 localhost\n";
        let merged = merge(odd, Some(&section())).unwrap();
        let removed = merge(&merged, None).unwrap();
        assert_eq!(removed, format!("{odd}\n"));
        // No section at all: removal is a no-op.
        assert_eq!(merge(odd, None).unwrap(), odd);
    }

    #[test]
    fn empty_file_gets_the_section() {
        let merged = merge("", Some(&section())).unwrap();
        assert_eq!(merged, format!("{}\n", section()));
    }

    #[test]
    fn baseline_header_uses_prefix_constant() {
        let s = section();
        assert!(s.contains(&format!("{BASELINE_HEADER_PREFIX} fakenews-gambling ---")));
    }
}
