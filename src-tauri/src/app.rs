//! Testable application core. Every mutation goes through here: compute the
//! new [`AppState`], render the managed section, stage the file, write it
//! through the [`PrivilegedWriter`], and only then persist state. A cancelled
//! or failed privileged write therefore never changes persisted state.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::baseline::fetcher::Fetcher;
use crate::baseline::sources::SourceId;
use crate::baseline::stevenblack::Extension;
use crate::domain::block_entry::block_lines;
use crate::domain::hostname::Hostname;
use crate::domain::mode::BlockingMode;
use crate::error::{AppError, AppResult};
use crate::hosts_file::render::{merge, render_section};
use crate::hosts_file::writer::{default_writer, PrivilegedWriter, WriteError};
use crate::store::{AppState, BaselineMeta, Store};

/// Everything the UI needs, in one serialisable snapshot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiState {
    pub mode: BlockingMode,
    pub sites: Vec<String>,
    pub baseline: Option<BaselineView>,
    /// Number of hosts-file lines HostBlocker currently blocks.
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineView {
    /// Which curated list is in use.
    pub source: SourceId,
    /// Prebuilt-variant slug without the source prefix, e.g.
    /// `fakenews-gambling`; `None` for a plain list.
    pub variant: Option<String>,
    pub extensions: Vec<Extension>,
    pub host_count: usize,
    pub fetched_at: DateTime<Utc>,
}

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct App {
    store: Store,
    fetcher: Fetcher,
    writer: Arc<dyn PrivilegedWriter>,
    hosts_path: PathBuf,
    backup_path: PathBuf,
    staging_dir: PathBuf,
}

impl App {
    /// Production wiring: real privileged writer, real list origins.
    pub fn production(
        state_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        hosts_path: impl Into<PathBuf>,
    ) -> Self {
        Self::with_fetcher(
            state_dir,
            cache_dir,
            hosts_path,
            Arc::from(default_writer()),
            Fetcher::production(),
        )
    }

    /// Dependency-injecting constructor shared by tests: every source is
    /// fetched from `base_url` (typically a local HTTP stub).
    pub fn with_parts(
        state_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        hosts_path: impl Into<PathBuf>,
        writer: Arc<dyn PrivilegedWriter>,
        base_url: impl Into<String>,
    ) -> Self {
        Self::with_fetcher(
            state_dir,
            cache_dir,
            hosts_path,
            writer,
            Fetcher::new(base_url),
        )
    }

    pub fn with_fetcher(
        state_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        hosts_path: impl Into<PathBuf>,
        writer: Arc<dyn PrivilegedWriter>,
        fetcher: Fetcher,
    ) -> Self {
        let state_dir = state_dir.into();
        let cache_dir = cache_dir.into();
        let hosts_path = hosts_path.into();
        let backup_path = default_backup_path(&hosts_path);
        Self {
            store: Store::with_dirs(state_dir, cache_dir.clone()),
            fetcher,
            writer,
            staging_dir: cache_dir,
            hosts_path,
            backup_path,
        }
    }

    /// `/etc/hosts` → `/etc/hosts.hostblocker.bak`.
    pub fn backup_path(&self) -> &Path {
        &self.backup_path
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    pub fn get_state(&self) -> AppResult<UiState> {
        let state = self.store.load()?;
        self.build_ui_state(&state)
    }

    // ------------------------------------------------------------------
    // Mutations (each one applies to /etc/hosts)
    // ------------------------------------------------------------------

    /// Add one custom site from a user-entered page URL.
    pub fn add_site(&self, input: &str) -> AppResult<UiState> {
        let host = Hostname::parse(input)?;
        let mut state = self.store.load()?;
        if state.sites.contains(&host) {
            return Err(AppError::User(format!("{host} is already blocked.")));
        }
        state.sites.push(host);
        state.sites.sort();
        self.apply(&state, None)
    }

    /// Remove a custom site (accepts the same input forms as [`add_site`]).
    pub fn remove_site(&self, input: &str) -> AppResult<UiState> {
        let host = Hostname::parse(input)?;
        let mut state = self.store.load()?;
        let before = state.sites.len();
        state.sites.retain(|h| h != &host);
        if state.sites.len() == before {
            return Err(AppError::User(format!("{host} is not in your list.")));
        }
        self.apply(&state, None)
    }

    /// Download a curated list and start using it as the baseline.
    /// `extensions` only apply to [`SourceId::StevenBlack`].
    pub fn import_baseline(
        &self,
        source: SourceId,
        extensions: &[Extension],
    ) -> AppResult<UiState> {
        let extensions = if source.supports_extensions() {
            canonical_extensions(extensions)
        } else {
            Vec::new()
        };
        let hosts = self.fetcher.fetch(source, &extensions)?;
        let mut state = self.store.load()?;
        state.baseline = Some(BaselineMeta {
            source,
            extensions,
            fetched_at: Utc::now(),
        });
        self.apply(&state, Some(&hosts))
    }

    /// Re-download the currently selected list. A failed download leaves
    /// the previous baseline untouched.
    pub fn update_baseline(&self) -> AppResult<UiState> {
        let state = self.store.load()?;
        let meta = state
            .baseline
            .as_ref()
            .ok_or_else(|| AppError::User("No baseline list is imported.".into()))?;
        let hosts = self.fetcher.fetch(meta.source, &meta.extensions)?;
        let mut state = state.clone();
        if let Some(m) = state.baseline.as_mut() {
            m.fetched_at = Utc::now();
        }
        self.apply(&state, Some(&hosts))
    }

    /// Drop the baseline entirely (state, cache and hosts-file entries).
    pub fn remove_baseline(&self) -> AppResult<UiState> {
        let mut state = self.store.load()?;
        if state.baseline.is_none() {
            return Err(AppError::User("No baseline list is imported.".into()));
        }
        state.baseline = None;
        self.apply(&state, Some(&[]))
    }

    pub fn set_mode(&self, mode: BlockingMode) -> AppResult<UiState> {
        let mut state = self.store.load()?;
        state.mode = mode;
        self.apply(&state, None)
    }

    /// Remove the entire managed section from the hosts file and reset all
    /// persisted state (Settings → “Remove all HostBlocker entries”).
    pub fn remove_all_entries(&self) -> AppResult<UiState> {
        let existing = self.read_hosts()?;
        let merged = merge(&existing, None)?;
        if merged != existing {
            self.write_privileged(&merged)?;
        }
        let state = AppState::default();
        self.store.save(&state)?;
        self.store.delete_baseline_hosts()?;
        self.build_ui_state(&state)
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    /// Render + merge + privileged write, then persist state (and baseline
    /// cache when `baseline_hosts` is given). Nothing is persisted when the
    /// privileged write fails or is cancelled.
    fn apply(&self, state: &AppState, baseline_hosts: Option<&[String]>) -> AppResult<UiState> {
        let rendered = self.render_full(state, baseline_hosts)?;
        let existing = self.read_hosts()?;
        if rendered != existing {
            self.write_privileged(&rendered)?;
        }
        self.store.save(state)?;
        if let Some(hosts) = baseline_hosts {
            self.store.save_baseline_hosts(hosts)?;
        }
        self.build_ui_state(state)
    }

    fn write_privileged(&self, content: &str) -> AppResult<()> {
        fs::create_dir_all(&self.staging_dir)?;
        let seq = STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let staged =
            self.staging_dir
                .join(format!("hostblocker-hosts.{}.{}", std::process::id(), seq));
        fs::write(&staged, content)?;
        let result = self
            .writer
            .install_hosts(&staged, &self.hosts_path, &self.backup_path);
        let _ = fs::remove_file(&staged);
        match result {
            Ok(()) => Ok(()),
            Err(WriteError::Cancelled) => Err(AppError::Cancelled),
            Err(WriteError::Failed(msg)) => Err(AppError::Privileged(msg)),
        }
    }

    /// The complete new `/etc/hosts` document for `state`.
    fn render_full(
        &self,
        state: &AppState,
        baseline_override: Option<&[String]>,
    ) -> AppResult<String> {
        let custom_lines: Vec<String> = if state.mode.writes_custom() {
            state.sites.iter().flat_map(block_lines).collect()
        } else {
            Vec::new()
        };
        let mut baseline_label_text: Option<String> = None;
        let mut baseline_hosts: Vec<String> = Vec::new();
        if let Some(meta) = &state.baseline {
            if state.mode.writes_baseline() {
                baseline_label_text = Some(meta.source.baseline_label(&meta.extensions));
                baseline_hosts = match baseline_override {
                    Some(h) => h.to_vec(),
                    None => self.store.load_baseline_hosts()?,
                };
            }
        }
        let baseline = baseline_label_text
            .as_deref()
            .map(|label| (label, baseline_hosts.as_slice()));
        let section = render_section(&custom_lines, baseline);
        let existing = self.read_hosts()?;
        merge(&existing, Some(&section))
    }

    fn read_hosts(&self) -> AppResult<String> {
        match fs::read_to_string(&self.hosts_path) {
            Ok(text) => Ok(text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(e.into()),
        }
    }

    fn build_ui_state(&self, state: &AppState) -> AppResult<UiState> {
        let baseline = match &state.baseline {
            Some(meta) => Some(BaselineView {
                source: meta.source,
                variant: crate::baseline::stevenblack::variant_slug(&meta.extensions),
                extensions: meta.extensions.clone(),
                host_count: self.store.load_baseline_hosts()?.len(),
                fetched_at: meta.fetched_at,
            }),
            None => None,
        };
        let custom_count = if state.mode.writes_custom() {
            state.sites.len() * 4
        } else {
            0
        };
        let baseline_count = baseline
            .as_ref()
            .filter(|_| state.mode.writes_baseline())
            .map(|b| b.host_count)
            .unwrap_or(0);
        Ok(UiState {
            mode: state.mode,
            sites: state.sites.iter().map(|h| h.to_string()).collect(),
            baseline,
            blocked_count: custom_count + baseline_count,
        })
    }
}

/// De-duplicated, canonically ordered extension list for persistence.
fn canonical_extensions(exts: &[Extension]) -> Vec<Extension> {
    let mut v: Vec<Extension> = exts
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    v.sort();
    v
}

fn default_backup_path(hosts_path: &Path) -> PathBuf {
    let name = hosts_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "hosts".to_string());
    hosts_path.with_file_name(format!("{name}.hostblocker.bak"))
}
