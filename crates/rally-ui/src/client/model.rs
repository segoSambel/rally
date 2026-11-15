use gpui_kit::{Context, EventEmitter};

use rally_core::{ApiRequest, ApiResponse, ConsoleEntry, RequestDraft, send};

pub enum RequestState {
    Idle,
    Sending,
    Done(ApiResponse),
    Failed(String),
}

pub struct ClientModel {
    state: RequestState,
}

/// Emitted when a sent request finishes, carrying the draft it was sent from.
pub struct RequestCompleted {
    pub draft: RequestDraft,
    /// The request as sent and its outcome, for the console.
    pub entry: ConsoleEntry,
    pub status: Option<u16>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

impl EventEmitter<RequestCompleted> for ClientModel {}

impl ClientModel {
    pub fn new() -> Self {
        Self {
            state: RequestState::Idle,
        }
    }

    // reads
    pub fn state(&self) -> &RequestState {
        &self.state
    }

    pub fn is_sending(&self) -> bool {
        matches!(self.state, RequestState::Sending)
    }

    // writes
    pub fn fail(&mut self, message: String, cx: &mut Context<Self>) {
        self.state = RequestState::Failed(message);
        cx.notify();
    }

    pub fn send(&mut self, request: ApiRequest, draft: RequestDraft, cx: &mut Context<Self>) {
        if self.is_sending() {
            return;
        }
        self.state = RequestState::Sending;
        cx.notify();

        let sent = request.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { send(request) })
                .await;
            this.update(cx, |this, cx| {
                let entry = ConsoleEntry::new(&sent, &result);
                let completed = match &result {
                    Ok(response) => RequestCompleted {
                        draft,
                        entry,
                        status: Some(response.status),
                        elapsed_ms: response.elapsed.as_millis() as u64,
                        error: None,
                    },
                    Err(message) => RequestCompleted {
                        draft,
                        entry,
                        status: None,
                        elapsed_ms: 0,
                        error: Some(message.clone()),
                    },
                };
                this.state = match result {
                    Ok(response) => RequestState::Done(response),
                    Err(message) => RequestState::Failed(message),
                };
                cx.emit(completed);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
