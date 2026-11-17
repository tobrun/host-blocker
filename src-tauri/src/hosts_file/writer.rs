//! The single platform-specific module: replacing `/etc/hosts` through a
//! one-shot elevated shell invocation (admin prompt each time, no daemon).

use std::path::Path;
use std::process::Command;

/// Result of trying to install a staged hosts file with elevated rights.
#[derive(Debug)]
pub enum WriteError {
    /// The user dismissed or failed the privilege prompt.
    Cancelled,
    /// Anything else that went wrong.
    Failed(String),
}

pub trait PrivilegedWriter: Send + Sync {
    /// Replace `hosts` with the contents of the world-readable staging
    /// file `staged` (final mode 644), first copying `hosts` to `backup`
    /// when no backup exists yet, then flushing the system DNS cache.
    fn install_hosts(&self, staged: &Path, hosts: &Path, backup: &Path) -> Result<(), WriteError>;
}

/// Pick the writer for the platform HostBlocker is running on.
pub fn default_writer() -> Box<dyn PrivilegedWriter> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacWriter)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxWriter)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Box::new(UnsupportedWriter)
    }
}

/// Common shell body, run as root. Positional parameters keep paths out of
/// the script text: `$1` staged, `$2` hosts, `$3` backup.
const INSTALL_SCRIPT: &str = "set -e\n\
    [ -f \"$3\" ] || cp \"$2\" \"$3\"\n\
    install -m 644 \"$1\" \"$2\"\n\
    __HB_FLUSH__";

#[cfg(target_os = "macos")]
const FLUSH_MACOS: &str = "dscacheutil -flushcache; killall -HUP mDNSResponder";
#[cfg(target_os = "linux")]
const FLUSH_LINUX: &str =
    "if command -v resolvectl >/dev/null 2>&1; then resolvectl flush-caches; fi";

/// macOS: `osascript … with administrator privileges`.
#[cfg(target_os = "macos")]
pub struct MacWriter;

#[cfg(target_os = "macos")]
impl PrivilegedWriter for MacWriter {
    fn install_hosts(&self, staged: &Path, hosts: &Path, backup: &Path) -> Result<(), WriteError> {
        let script = INSTALL_SCRIPT.replace("__HB_FLUSH__", FLUSH_MACOS);
        let shell = format!(
            "{script}\nargs {} {} {}",
            posix_quote(staged),
            posix_quote(hosts),
            posix_quote(backup)
        );
        // `do shell script` takes one double-quoted AppleScript string.
        let apple = format!("do shell script \"{}\"", escape_applescript(&shell));
        let output = Command::new("osascript")
            .args(["-e", &apple])
            .output()
            .map_err(|e| WriteError::Failed(e.to_string()))?;
        classify(output, &["user canceled", "user cancelled", "errcore -128"])
    }
}

/// Linux: `pkexec sh -c …` with the paths as positional arguments.
#[cfg(target_os = "linux")]
pub struct LinuxWriter;

#[cfg(target_os = "linux")]
impl PrivilegedWriter for LinuxWriter {
    fn install_hosts(&self, staged: &Path, hosts: &Path, backup: &Path) -> Result<(), WriteError> {
        let script = INSTALL_SCRIPT.replace("__HB_FLUSH__", FLUSH_LINUX);
        let output = Command::new("pkexec")
            .arg("sh")
            .arg("-c")
            .arg(&script)
            .arg("hostblocker")
            .arg(staged)
            .arg(hosts)
            .arg(backup)
            .output()
            .map_err(|e| WriteError::Failed(e.to_string()))?;
        // pkexec exit codes: 126 = authentication dialog dismissed,
        // 127 = not authorised / authentication failed.
        if matches!(output.status.code(), Some(126) | Some(127)) {
            return Err(WriteError::Cancelled);
        }
        classify(
            output,
            &["user cancelled", "user canceled", "not authorized"],
        )
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
struct UnsupportedWriter;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl PrivilegedWriter for UnsupportedWriter {
    fn install_hosts(
        &self,
        _staged: &Path,
        _hosts: &Path,
        _backup: &Path,
    ) -> Result<(), WriteError> {
        Err(WriteError::Failed(
            "HostBlocker only supports macOS and Linux".into(),
        ))
    }
}

fn classify(output: std::process::Output, cancel_markers: &[&str]) -> Result<(), WriteError> {
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if cancel_markers.iter().any(|m| stderr.contains(m)) {
        return Err(WriteError::Cancelled);
    }
    Err(WriteError::Failed(format!(
        "privileged command failed with {}: {}",
        output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "a signal".to_string()),
        stderr.trim()
    )))
}

#[cfg(target_os = "macos")]
fn posix_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', r"'\''"))
}

#[cfg(target_os = "macos")]
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
