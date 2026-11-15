use gpui_kit::Context;

use rally_core::{Console, ConsoleEntry};

/// Requests sent this session. Kept in memory only, never written to disk.
pub struct ConsoleModel {
    console: Console,
}

impl ConsoleModel {
    pub fn new() -> Self {
        Self {
            console: Console::default(),
        }
    }

    // reads
    pub fn entries(&self) -> &[ConsoleEntry] {
        self.console.entries()
    }

    // writes
    pub fn record(&mut self, entry: ConsoleEntry, cx: &mut Context<Self>) {
        self.console.push(entry);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.console.clear();
        cx.notify();
    }
}
