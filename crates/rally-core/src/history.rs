use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use serde::{Deserialize, Serialize};

use crate::draft::RequestDraft;

/// Entries kept; the oldest are dropped first.
pub const MAX_ENTRIES: usize = 100;

/// A sent request and how it ended. Response bodies are not stored.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryEntry {
    pub id: u64,
    /// Unix time in seconds.
    pub at: u64,
    pub draft: RequestDraft,
    pub status: Option<u16>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// Sent requests, newest first.
#[derive(Default)]
pub struct History {
    entries: Vec<HistoryEntry>,
    next_id: u64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct HistoryFile {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Adds an entry at the front, dropping the oldest beyond [`MAX_ENTRIES`].
    pub fn push(
        &mut self,
        draft: RequestDraft,
        status: Option<u16>,
        elapsed_ms: u64,
        error: Option<String>,
    ) {
        let entry = HistoryEntry {
            id: self.next_id,
            at: now_unix(),
            draft,
            status,
            elapsed_ms,
            error,
        };
        self.next_id += 1;
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_ENTRIES);
    }

    pub fn remove(&mut self, id: u64) {
        self.entries.retain(|entry| entry.id != id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Loads history from `path`. A missing file is an empty history. A file that cannot
    /// be parsed is moved aside to `<path>.bak` and an empty history is returned.
    pub fn load(path: &Path) -> Self {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(error) => {
                eprintln!("rally: cannot read {}: {error}", path.display());
                return Self::default();
            }
        };

        match serde_json::from_str::<HistoryFile>(&text) {
            Ok(file) => Self::from_entries(file.entries),
            Err(error) => {
                let backup = backup_path(path);
                eprintln!(
                    "rally: {} is not valid history ({error}); moved to {}",
                    path.display(),
                    backup.display()
                );
                if let Err(error) = fs::rename(path, &backup) {
                    eprintln!("rally: cannot move {} aside: {error}", path.display());
                }
                Self::default()
            }
        }
    }

    /// Writes the history to a temporary file and renames it over `path`.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = HistoryFile {
            entries: self.entries.clone(),
        };
        let text = serde_json::to_string_pretty(&file).map_err(io::Error::other)?;

        let mut temp = path.as_os_str().to_owned();
        temp.push(".tmp");
        let temp = PathBuf::from(temp);
        fs::write(&temp, text)?;
        fs::rename(&temp, path)
    }

    fn from_entries(mut entries: Vec<HistoryEntry>) -> Self {
        entries.truncate(MAX_ENTRIES);
        let next_id = entries
            .iter()
            .map(|entry| entry.id)
            .max()
            .map_or(0, |id| id + 1);
        Self { entries, next_id }
    }
}

/// The operating systems whose conventions for per-user data Rally follows.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    /// Linux and other Unix systems, following the XDG base directory specification.
    Linux,
}

impl Platform {
    /// The platform this program was compiled for.
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }
}

/// Where history is saved on this machine, or `None` when the environment does not say where
/// the user's data directory is. See [`history_path_for`].
pub fn default_history_path() -> Option<PathBuf> {
    history_path_for(Platform::current(), |name| std::env::var_os(name))
}

/// The history file for `platform`, using `env` to read environment variables:
///
/// - Windows: `%APPDATA%\Rally\history.json`
/// - macOS: `~/Library/Application Support/Rally/history.json`
/// - Linux: `$XDG_DATA_HOME/rally/history.json`, or `~/.local/share/rally/history.json` when
///   `XDG_DATA_HOME` is unset, empty or not an absolute path
///
/// Empty variables count as unset.
pub fn history_path_for(
    platform: Platform,
    env: impl Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    let var = |name: &str| {
        env(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };

    match platform {
        Platform::Windows => Some(var("APPDATA")?.join("Rally").join("history.json")),
        Platform::MacOs => Some(
            var("HOME")?
                .join("Library")
                .join("Application Support")
                .join("Rally")
                .join("history.json"),
        ),
        Platform::Linux => {
            // The XDG specification says to ignore relative paths.
            let data = var("XDG_DATA_HOME")
                .filter(|dir| dir.is_absolute())
                .or_else(|| Some(var("HOME")?.join(".local").join("share")))?;
            Some(data.join("rally").join("history.json"))
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_owned();
    backup.push(".bak");
    PathBuf::from(backup)
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
