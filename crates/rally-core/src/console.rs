use std::time::Duration;

use crate::history::now_unix;
use crate::request::{ApiRequest, Method};
use crate::response::ApiResponse;

/// Entries kept in the console; the oldest are dropped first.
pub const MAX_ENTRIES: usize = 200;
/// Longest response body kept for display, in bytes.
pub const MAX_BODY_PREVIEW: usize = 64 * 1024;

/// A request as it was sent.
#[derive(Clone)]
pub struct ConsoleRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// A response as it was received. The body is cut to [`MAX_BODY_PREVIEW`].
#[derive(Clone)]
pub struct ConsoleResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body_preview: String,
    /// Full length of the body in bytes, before cutting.
    pub body_len: usize,
    pub elapsed: Duration,
}

#[derive(Clone)]
pub enum ConsoleOutcome {
    Response(ConsoleResponse),
    Failed(String),
}

/// One request and how it ended.
#[derive(Clone)]
pub struct ConsoleEntry {
    pub id: u64,
    /// Unix time in seconds.
    pub at: u64,
    pub request: ConsoleRequest,
    pub outcome: ConsoleOutcome,
}

impl ConsoleEntry {
    /// Records `request` and its `result`. The id and time are set when it is pushed.
    pub fn new(request: &ApiRequest, result: &Result<ApiResponse, String>) -> Self {
        let outcome = match result {
            Ok(response) => ConsoleOutcome::Response(ConsoleResponse {
                status: response.status,
                reason: response.reason.clone(),
                headers: response.headers.clone(),
                body_preview: preview(&response.body, MAX_BODY_PREVIEW).to_string(),
                body_len: response.body.len(),
                elapsed: response.elapsed,
            }),
            Err(message) => ConsoleOutcome::Failed(message.clone()),
        };

        Self {
            id: 0,
            at: 0,
            request: ConsoleRequest {
                method: request.method,
                url: request.url.trim().to_string(),
                headers: request.headers.clone(),
                body: request.body.clone(),
            },
            outcome,
        }
    }
}

impl ConsoleRequest {
    /// A `curl` command that repeats this request.
    pub fn to_curl(&self) -> String {
        let mut parts = vec![
            "curl".to_string(),
            "-X".to_string(),
            self.method.as_str().to_string(),
            shell_quote(&self.url),
        ];
        for (name, value) in &self.headers {
            parts.push("-H".to_string());
            parts.push(shell_quote(&format!("{name}: {value}")));
        }
        if !self.body.is_empty() {
            parts.push("--data-raw".to_string());
            parts.push(shell_quote(&self.body));
        }
        parts.join(" ")
    }
}

/// Requests sent this session, newest first. Kept in memory only.
#[derive(Default)]
pub struct Console {
    entries: Vec<ConsoleEntry>,
    next_id: u64,
}

impl Console {
    pub fn entries(&self) -> &[ConsoleEntry] {
        &self.entries
    }

    /// Adds an entry at the front, dropping the oldest beyond [`MAX_ENTRIES`].
    pub fn push(&mut self, mut entry: ConsoleEntry) {
        entry.id = self.next_id;
        entry.at = now_unix();
        self.next_id += 1;
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_ENTRIES);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// The longest prefix of `text` that is at most `max` bytes and ends on a character boundary.
fn preview(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Wraps `text` in single quotes for a POSIX shell.
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}
