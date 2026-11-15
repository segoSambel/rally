use std::path::PathBuf;

use gpui_kit::Context;

use rally_core::{History, HistoryEntry, RequestDraft, default_history_path};

/// Sent requests, kept on disk so they survive restarts.
pub struct HistoryModel {
    history: History,
    path: Option<PathBuf>,
}

impl HistoryModel {
    pub fn new() -> Self {
        let path = default_history_path();
        let history = match &path {
            Some(path) => History::load(path),
            None => {
                eprintln!("rally: APPDATA is not set; history will not be saved");
                History::default()
            }
        };
        Self { history, path }
    }

    // reads
    pub fn entries(&self) -> &[HistoryEntry] {
        self.history.entries()
    }

    // writes
    pub fn record(
        &mut self,
        draft: RequestDraft,
        status: Option<u16>,
        elapsed_ms: u64,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.history.push(draft, status, elapsed_ms, error);
        self.persist();
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.persist();
        cx.notify();
    }

    fn persist(&self) {
        let Some(path) = &self.path else { return };
        if let Err(error) = self.history.save(path) {
            eprintln!("rally: cannot save history to {}: {error}", path.display());
        }
    }
}
