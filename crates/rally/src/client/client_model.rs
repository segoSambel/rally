use gpui_kit::Context;

use rally_core::{ApiRequest, ApiResponse, send};

pub enum RequestState {
    Idle,
    Sending,
    Done(ApiResponse),
    Failed(String),
}

pub struct ClientModel {
    state: RequestState,
}

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

    pub fn send(&mut self, request: ApiRequest, cx: &mut Context<Self>) {
        if self.is_sending() {
            return;
        }
        self.state = RequestState::Sending;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { send(request) })
                .await;
            this.update(cx, |this, cx| {
                this.state = match result {
                    Ok(response) => RequestState::Done(response),
                    Err(message) => RequestState::Failed(message),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}
