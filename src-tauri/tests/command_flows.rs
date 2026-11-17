//! Command-flow integration tests: real [`App`] logic against a fake
//! [`PrivilegedWriter`], a temp hosts file, and a local HTTP stub standing
//! in for the StevenBlack repository.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use host_blocker_lib::app::App;
use host_blocker_lib::baseline::stevenblack::Extension;
use host_blocker_lib::domain::mode::BlockingMode;
use host_blocker_lib::error::AppError;
use host_blocker_lib::hosts_file::writer::{PrivilegedWriter, WriteError};
use host_blocker_lib::store::Store;
use serde_json::Value;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Records every installed hosts-file body and applies it to the target
/// path, like the real privileged writer would.
struct FakeWriter {
    installs: Mutex<Vec<String>>,
    cancelled: Mutex<bool>,
}

impl FakeWriter {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            installs: Mutex::new(Vec::new()),
            cancelled: Mutex::new(false),
        })
    }
    fn last(&self) -> Option<String> {
        self.installs.lock().unwrap().last().cloned()
    }
    fn fail_with_cancel(&self) {
        *self.cancelled.lock().unwrap() = true;
    }
}

impl PrivilegedWriter for FakeWriter {
    fn install_hosts(&self, staged: &Path, hosts: &Path, backup: &Path) -> Result<(), WriteError> {
        if *self.cancelled.lock().unwrap() {
            return Err(WriteError::Cancelled);
        }
        let content = fs::read_to_string(staged).map_err(|e| WriteError::Failed(e.to_string()))?;
        // First write creates the one-time backup, like the real script.
        if !backup.exists() {
            if hosts.exists() {
                fs::copy(hosts, backup).map_err(|e| WriteError::Failed(e.to_string()))?;
            } else {
                fs::write(backup, "").map_err(|e| WriteError::Failed(e.to_string()))?;
            }
        }
        fs::write(hosts, &content).map_err(|e| WriteError::Failed(e.to_string()))?;
        self.installs.lock().unwrap().push(content);
        Ok(())
    }
}

/// Cheaply cloneable test environment; `App` values (and the blocking HTTP
/// client they own) are built inside `spawn_blocking` so that client is
/// never created or dropped on an async-runtime thread.
#[derive(Clone)]
struct Env {
    dir: Arc<tempfile::TempDir>,
    hosts: PathBuf,
    writer: Arc<FakeWriter>,
    initial_hosts: String,
}

impl Env {
    fn new() -> Self {
        let dir = Arc::new(tempfile::tempdir().unwrap());
        let hosts = dir.path().join("hosts");
        let initial_hosts = "127.0.0.1 localhost\n::1 localhost\n127.0.1.1 mybox\n".to_string();
        fs::write(&hosts, &initial_hosts).unwrap();
        Self {
            dir,
            hosts,
            writer: FakeWriter::new(),
            initial_hosts,
        }
    }

    fn app(&self, base_url: &str) -> App {
        let w: Arc<dyn PrivilegedWriter> = self.writer.clone();
        App::with_parts(
            self.dir.path().join("state"),
            self.dir.path().join("cache"),
            self.hosts.clone(),
            w,
            base_url,
        )
    }

    fn hosts_content(&self) -> String {
        fs::read_to_string(&self.hosts).unwrap()
    }

    fn persisted(&self) -> Value {
        let store = Store::new(self.dir.path().join("state"));
        serde_json::to_value(store.load().unwrap()).unwrap()
    }
}

/// `off!(env, base_url, |app| app.call(..))` — builds an App on a plain
/// blocking thread and runs the call there.
macro_rules! off {
    ($env:expr, $base:expr, |$a:ident| $body:expr) => {{
        let env = $env.clone();
        let base = $base.to_string();
        tokio::task::spawn_blocking(move || {
            let $a = env.app(&base);
            $body
        })
        .await
        .unwrap()
    }};
}

const UNUSED_BASE: &str = "http://127.0.0.1:1/unused";
const BASELINE_BODY: &str = "# comments are dropped\n0.0.0.0 ads.example.net\n0.0.0.0 track.example.net\n0.0.0.0 0.0.0.0\n127.0.0.1 localhost\n";

async fn stub_hosts(server: &MockServer, path_str: &str, body: &str) {
    Mock::given(path(path_str))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

#[tokio::test]
async fn add_site_writes_four_lines_and_persists_state() {
    let env = Env::new();
    let state = off!(env, UNUSED_BASE, |a| a
        .add_site("https://www.reddit.com/r/rust"))
    .unwrap();

    let written = env.writer.last().unwrap();
    let expected_tail = [
        "# >>> HostBlocker BEGIN (managed, do not edit)",
        "# --- custom ---",
        "0.0.0.0 reddit.com",
        "0.0.0.0 www.reddit.com",
        ":: reddit.com",
        ":: www.reddit.com",
        "# <<< HostBlocker END",
    ]
    .join("\n");
    assert!(written.ends_with(&format!("{expected_tail}\n")));
    assert!(written.starts_with(&env.initial_hosts));

    assert_eq!(state.sites, vec!["reddit.com".to_string()]);
    assert_eq!(state.blocked_count, 4);
    assert_eq!(env.hosts_content(), written);
    assert_eq!(env.persisted()["sites"][0], "reddit.com");
    // One-time backup preserved the pristine file.
    assert_eq!(
        fs::read_to_string(env.dir.path().join("hosts.hostblocker.bak")).unwrap(),
        env.initial_hosts
    );
}

#[tokio::test]
async fn duplicate_add_is_rejected_without_writing() {
    let env = Env::new();
    off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap();
    let err = off!(env, UNUSED_BASE, |a| a.add_site("https://www.reddit.com/x")).unwrap_err();
    assert!(matches!(err, AppError::User(_)));
    assert_eq!(env.writer.installs.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn remove_site_deletes_the_four_lines() {
    let env = Env::new();
    off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap();
    let state = off!(env, UNUSED_BASE, |a| a.remove_site("reddit.com")).unwrap();
    assert!(state.sites.is_empty());
    let written = env.writer.last().unwrap();
    assert!(!written.contains("reddit.com"));
}

#[tokio::test]
async fn remove_unknown_site_is_an_error() {
    let env = Env::new();
    let err = off!(env, UNUSED_BASE, |a| a.remove_site("nope.example")).unwrap_err();
    assert!(matches!(err, AppError::User(_)));
}

#[tokio::test]
async fn mode_transitions_control_what_is_written() {
    let env = Env::new();
    let server = MockServer::start().await;
    stub_hosts(&server, "/hosts", BASELINE_BODY).await;
    stub_hosts(&server, "/alternates/gambling/hosts", BASELINE_BODY).await;
    let base = server.uri();

    off!(env, &base, |a| a.add_site("reddit.com")).unwrap();
    off!(env, &base, |a| a.import_baseline(&[Extension::Gambling])).unwrap();

    // CustomPaused: baseline stays, custom disappears.
    off!(env, &base, |a| a.set_mode(BlockingMode::CustomPaused)).unwrap();
    let written = env.writer.last().unwrap();
    assert!(!written.contains("reddit.com"));
    assert!(written.contains("0.0.0.0 ads.example.net"));
    assert!(written.contains("# --- baseline: stevenblack gambling ---"));

    // AllPaused: markers only.
    let state = off!(env, &base, |a| a.set_mode(BlockingMode::AllPaused)).unwrap();
    let written = env.writer.last().unwrap();
    assert_eq!(
        written,
        format!(
            "{}\n\n# >>> HostBlocker BEGIN (managed, do not edit)\n# --- custom ---\n# <<< HostBlocker END\n",
            env.initial_hosts.trim_end()
        )
    );
    assert_eq!(state.blocked_count, 0);

    // Active: both again.
    let state = off!(env, &base, |a| a.set_mode(BlockingMode::Active)).unwrap();
    let written = env.writer.last().unwrap();
    assert!(written.contains("0.0.0.0 www.reddit.com"));
    assert!(written.contains("ads.example.net"));
    assert_eq!(state.blocked_count, 4 + 2);
}

#[tokio::test]
async fn import_update_remove_baseline_flow() {
    let env = Env::new();
    let server = MockServer::start().await;
    stub_hosts(&server, "/hosts", BASELINE_BODY).await;
    let v2_server = MockServer::start().await;
    stub_hosts(&v2_server, "/hosts", "# v2\n0.0.0.0 new-only.example\n").await;

    // Import (no extensions): parser drops comments, 0.0.0.0 and localhost.
    let state = off!(env, server.uri(), |a| a.import_baseline(&[])).unwrap();
    let view = state.baseline.clone().unwrap();
    assert_eq!(view.variant, None);
    assert_eq!(view.host_count, 2);
    assert_eq!(state.blocked_count, 2);
    assert!(env
        .writer
        .last()
        .unwrap()
        .contains("# --- baseline: stevenblack ---"));

    // Update re-downloads the same variant URL and refreshes the cache.
    let state = off!(env, v2_server.uri(), |a| a.update_baseline()).unwrap();
    assert_eq!(state.baseline.unwrap().host_count, 1);
    let written = env.writer.last().unwrap();
    assert!(written.contains("new-only.example"));
    assert!(!written.contains("ads.example.net"));

    // Remove drops baseline entries and metadata.
    let state = off!(env, UNUSED_BASE, |a| a.remove_baseline()).unwrap();
    assert!(state.baseline.is_none());
    assert!(!env.writer.last().unwrap().contains("baseline:"));

    // Updating without a baseline errors.
    let err = off!(env, UNUSED_BASE, |a| a.update_baseline()).unwrap_err();
    assert!(matches!(err, AppError::User(_)));
}

#[tokio::test]
async fn cancelled_prompt_leaves_persisted_state_unchanged() {
    let env = Env::new();
    off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap();

    env.writer.fail_with_cancel();
    let err = off!(env, UNUSED_BASE, |a| a.add_site("evil.example")).unwrap_err();
    assert!(matches!(err, AppError::Cancelled));

    // Persisted state still shows only reddit.com and the hosts file is
    // untouched.
    let persisted = env.persisted();
    assert_eq!(persisted["sites"].as_array().unwrap().len(), 1);
    assert_eq!(persisted["sites"][0].as_str().unwrap(), "reddit.com");
    assert!(!env.hosts_content().contains("evil.example"));

    // The UI can still read back the previous state.
    let state = off!(env, UNUSED_BASE, |a| a.get_state()).unwrap();
    assert_eq!(state.sites, vec!["reddit.com".to_string()]);
}

#[tokio::test]
async fn fetch_failure_keeps_previous_baseline() {
    let env = Env::new();
    let server = MockServer::start().await;
    stub_hosts(&server, "/hosts", BASELINE_BODY).await;
    let base = server.uri();

    off!(env, &base, |a| a.import_baseline(&[])).unwrap();

    let failing_server = MockServer::start().await;
    Mock::given(path("/hosts"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&failing_server)
        .await;
    let err = off!(env, failing_server.uri(), |a| a.update_baseline()).unwrap_err();
    assert!(matches!(err, AppError::Network(_)));

    // Baseline metadata and cache still hold the original list.
    let state = off!(env, &base, |a| a.get_state()).unwrap();
    assert_eq!(state.baseline.unwrap().host_count, 2);
    assert!(env.hosts_content().contains("ads.example.net"));
}

#[tokio::test]
async fn remove_all_entries_restores_the_file_and_resets_state() {
    let env = Env::new();
    off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap();
    off!(env, UNUSED_BASE, |a| a.remove_all_entries()).unwrap();
    // The original bytes plus a blank line remain where the section was;
    // no HostBlocker traces anywhere.
    let content = env.hosts_content();
    assert!(content.starts_with(&env.initial_hosts));
    assert!(!content.contains("HostBlocker"));
    let persisted = env.persisted();
    assert_eq!(persisted["mode"], "active");
    assert!(persisted["sites"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn corrupt_state_file_surfaces_as_error() {
    let env = Env::new();
    fs::create_dir_all(env.dir.path().join("state")).unwrap();
    fs::write(env.dir.path().join("state/state.json"), "!!corrupt").unwrap();
    let err = off!(env, UNUSED_BASE, |a| a.get_state()).unwrap_err();
    assert!(matches!(err, AppError::Data(_)));
}

#[tokio::test]
async fn missing_hosts_file_is_treated_as_empty() {
    // App::read_hosts maps NotFound to "" so a fresh box still works; a
    // guard mutant that instead errors must fail this.
    let env = Env::new();
    fs::remove_file(&env.hosts).unwrap();
    let state = off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap();
    assert_eq!(state.sites, vec!["reddit.com".to_string()]);
    let written = env.writer.last().unwrap();
    assert!(written.contains("0.0.0.0 reddit.com"));
    assert!(!written.contains("mybox"));
}

#[tokio::test]
async fn unreadable_hosts_file_reports_the_io_error_before_writing() {
    // A directory at the hosts path fails with a non-NotFound kind; the
    // NotFound fallback must not swallow it into an "empty" file.
    let env = Env::new();
    fs::remove_file(&env.hosts).unwrap();
    fs::create_dir(&env.hosts).unwrap();
    let err = off!(env, UNUSED_BASE, |a| a.add_site("reddit.com")).unwrap_err();
    assert_eq!(err.kind(), "io");
    assert!(env.writer.last().is_none(), "no privileged write attempted");
}

#[allow(dead_code)]
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn app_is_send_and_sync() {
    assert_send_sync::<App>();
}
