//! Single source of truth for every string HostBlocker writes into
//! `/etc/hosts`, plus the well-known paths it touches.

/// Opening marker of HostBlocker's managed section.
pub const BEGIN_MARKER: &str = "# >>> HostBlocker BEGIN (managed, do not edit)";
/// Closing marker of HostBlocker's managed section.
pub const END_MARKER: &str = "# <<< HostBlocker END";
/// Header line inside the managed section above the user's own sites.
pub const CUSTOM_HEADER: &str = "# --- custom ---";

/// Sink address used for blocked IPv4 lookups.
pub const SINK_V4: &str = "0.0.0.0";
/// Sink address used for blocked IPv6 lookups.
pub const SINK_V6: &str = "::";

/// The system hosts file.
pub const HOSTS_PATH: &str = "/etc/hosts";
/// One-time backup written before the first HostBlocker modification.
pub const HOSTS_BACKUP_PATH: &str = "/etc/hosts.hostblocker.bak";

/// Prefix of the baseline header line inside the managed section,
/// e.g. `# --- baseline: stevenblack fakenews-gambling ---`.
pub const BASELINE_HEADER_PREFIX: &str = "# --- baseline: stevenblack";
