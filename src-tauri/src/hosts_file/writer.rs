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
        let apple = mac_applescript(&script, staged, hosts, backup);
        let output = Command::new("osascript")
            .args(["-e", &apple])
            .output()
            .map_err(|e| WriteError::Failed(e.to_string()))?;
        classify(output, &["user canceled", "user cancelled", "errcore -128"])
    }
}

/// Build the `osascript` one-liner. `do shell script` cannot pass
/// positional parameters, so the script itself starts with `set --` and
/// `with administrator privileges` is what triggers the admin prompt.
#[cfg(any(target_os = "macos", test))]
fn mac_applescript(script: &str, staged: &Path, hosts: &Path, backup: &Path) -> String {
    let shell = format!(
        "set -- {} {} {}\n{}",
        posix_quote(staged),
        posix_quote(hosts),
        posix_quote(backup),
        script
    );
    // `do shell script` takes one double-quoted AppleScript string.
    format!(
        "do shell script \"{}\" with administrator privileges",
        escape_applescript(&shell)
    )
}

#[cfg(test)]
mod mac_script_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn positional_parameters_are_set_before_the_script_body() {
        // Regression: `cp: : No such file or directory` when `$2`/`$3`
        // were still empty at the time `cp` ran.
        let script = INSTALL_SCRIPT.replace("__HB_FLUSH__", "dscacheutil -flushcache");
        let apple = mac_applescript(
            &script,
            Path::new("/tmp/staged"),
            Path::new("/etc/hosts"),
            Path::new("/etc/hosts.hostblocker.bak"),
        );
        let set_idx = apple
            .find("set -- '/tmp/staged' '/etc/hosts'")
            .expect("set --");
        let cp_idx = apple
            .find(&escape_applescript("cp \"$2\" \"$3\""))
            .expect("cp");
        assert!(
            set_idx < cp_idx,
            "set -- must run before the script body: {apple}"
        );
        assert!(
            apple.contains("\" with administrator privileges"),
            "{apple}"
        );
    }

    #[test]
    fn quotes_single_quotes_in_paths() {
        let apple = mac_applescript(
            "true",
            Path::new("/tmp/it's"),
            Path::new("/etc/hosts"),
            Path::new("/etc/x"),
        );
        assert!(
            apple.contains(&escape_applescript(r"/tmp/it'\''s")),
            "{apple}"
        );
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

#[cfg(all(test, unix))]
mod install_script_tests {
    use super::*;

    /// Runs INSTALL_SCRIPT two ways: with real positional parameters (the
    /// Linux `pkexec sh -c` form) or with a `set --` prefix (the macOS
    /// `do shell script` form, which cannot pass arguments).
    #[test]
    fn script_installs_with_positional_arguments_pkexec_form() {
        run_script(false);
    }

    #[test]
    fn script_installs_with_set_dash_prefix_osascript_form() {
        run_script(true);
    }

    fn run_script(use_set_prefix: bool) {
        let dir = tempfile::tempdir().unwrap();
        let staged = dir.path().join("staged");
        let hosts = dir.path().join("hosts");
        let backup = dir.path().join("hosts.bak");
        std::fs::write(&staged, "new contents\n").unwrap();
        std::fs::write(&hosts, "original contents\n").unwrap();

        let body = INSTALL_SCRIPT.replace("__HB_FLUSH__", "true");
        let script = if use_set_prefix {
            format!(
                "set -- {} {} {}\n{}",
                posix_quote(&staged),
                posix_quote(&hosts),
                posix_quote(&backup),
                body
            )
        } else {
            body
        };
        let status = Command::new("sh")
            .arg("-c")
            .arg(&script)
            .arg("hostblocker")
            .arg(&staged)
            .arg(&hosts)
            .arg(&backup)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(std::fs::read_to_string(&hosts).unwrap(), "new contents\n");
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "original contents\n"
        );
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(&hosts).unwrap().permissions().mode() & 0o777
        };
        assert_eq!(mode, 0o644);

        // Second run: an existing backup must not be overwritten.
        std::fs::write(&hosts, "second original\n").unwrap();
        let status = Command::new("sh")
            .arg("-c")
            .arg(&script)
            .arg("hostblocker")
            .arg(&staged)
            .arg(&hosts)
            .arg(&backup)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "original contents\n"
        );
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

#[cfg(any(target_os = "macos", test))]
fn posix_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', r"'\''"))
}

#[cfg(any(target_os = "macos", test))]
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
