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

/// `%APPDATA%\Rally\history.json`, or `None` when `APPDATA` is not set.
pub fn default_history_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("Rally").join("history.json"))
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
