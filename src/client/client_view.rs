use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::{ActiveTheme, Selectable, StyledExt};
use gpui_kit::{Context, Entity, SharedString, Subscription, Window, div, prelude::*};

use super::client_model::{ClientModel, RequestState};
use super::http::{self, ApiRequest, ApiResponse, Method};

pub struct ClientView {
    model: Entity<ClientModel>,
    method: Method,
    url: Entity<InputState>,
    headers: Entity<TextareaState>,
    body: Entity<TextareaState>,
    response_body: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
}

impl ClientView {
    pub fn new(model: Entity<ClientModel>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url = cx.new(|cx| InputState::new(window, cx).placeholder("https://api.example.com"));
        let headers = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(4)
                .placeholder("Content-Type: application/json")
        });
        let body = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(6)
                .placeholder("Request body")
        });
        let response_body = cx.new(|cx| TextareaState::new(window, cx).rows(12));

        let subscriptions = vec![
            cx.observe_in(&model, window, Self::on_model_change),
            cx.subscribe_in(&url, window, Self::on_url_event),
        ];
        Self {
            model,
            method: Method::Get,
            url,
            headers,
            body,
            response_body,
            _subscriptions: subscriptions,
        }
    }

    fn on_model_change(
        &mut self,
        model: Entity<ClientModel>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = match model.read(cx).state() {
            RequestState::Done(response) => pretty_body(&response.body),
            _ => String::new(),
        };
        self.response_body
            .update(cx, |state, cx| state.set_value(text, window, cx));
        cx.notify();
    }

    fn on_url_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::PressEnter { .. } = event {
            self.send(cx);
        }
    }

    fn send(&mut self, cx: &mut Context<Self>) {
        let url = self.url.read(cx).value().trim().to_string();
        if url.is_empty() {
            self.model
                .update(cx, |model, cx| model.fail("Enter a URL first.".into(), cx));
            return;
        }

        let headers = match http::parse_headers(&self.headers.read(cx).value()) {
            Ok(headers) => headers,
            Err(message) => {
                self.model.update(cx, |model, cx| model.fail(message, cx));
                return;
            }
        };
        let request = ApiRequest {
            method: self.method,
            url,
            headers,
            body: self.body.read(cx).value().to_string(),
        };
        self.model.update(cx, |model, cx| model.send(request, cx));
    }
}

/// Pretty-prints JSON; any other body is shown unchanged.
fn pretty_body(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .unwrap_or_else(|_| body.to_string())
}

impl ClientView {
    fn render_request_row(&self, cx: &Context<Self>) -> impl IntoElement {
        let sending = self.model.read(cx).is_sending();

        div()
            .h_flex()
            .gap_2()
            .child(div().h_flex().gap_1().children(Method::ALL.map(|method| {
                Button::new(method.as_str())
                    .ghost()
                    .label(method.as_str())
                    .selected(self.method == method)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.method = method;
                        cx.notify();
                    }))
            })))
            .child(div().flex_1().child(Input::new(&self.url)))
            .child(
                Button::new("send")
                    .primary()
                    .label("Send")
                    .loading(sending)
                    .on_click(cx.listener(|this, _, _, cx| this.send(cx))),
            )
    }

    fn render_section(
        &self,
        title: &'static str,
        content: impl IntoElement,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .v_flex()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(title),
            )
            .child(content)
    }

    fn render_response(&self, cx: &Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let muted = cx.theme().muted_foreground;

        match model.state() {
            RequestState::Idle => div()
                .text_color(muted)
                .child("Send a request to see the response."),
            RequestState::Sending => div().text_color(muted).child("Sending..."),
            RequestState::Failed(message) => div()
                .text_color(cx.theme().danger)
                .child(SharedString::from(message.clone())),
            RequestState::Done(response) => div()
                .v_flex()
                .gap_2()
                .child(self.render_status(response, cx))
                .child(
                    self.render_section(
                        "Headers",
                        div()
                            .v_flex()
                            .text_sm()
                            .children(response.headers.iter().map(|(name, value)| {
                                div().child(SharedString::from(format!("{name}: {value}")))
                            })),
                        cx,
                    ),
                )
                .child(self.render_section(
                    "Body",
                    Textarea::new(&self.response_body).readonly(true),
                    cx,
                )),
        }
    }

    fn render_status(&self, response: &ApiResponse, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let color = match response.status {
            200..=299 => theme.success,
            400..=599 => theme.danger,
            _ => theme.warning,
        };
        let summary = format!(
            "{} {}  ·  {} ms  ·  {} B",
            response.status,
            response.reason,
            response.elapsed.as_millis(),
            response.body.len()
        );

        div().text_color(color).child(SharedString::from(summary))
    }
}

impl Render for ClientView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .gap_4()
            .p_4()
            .id("client-scroll")
            .overflow_y_scroll()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_request_row(cx))
            .child(
                div()
                    .h_flex()
                    .gap_4()
                    .child(div().flex_1().child(self.render_section(
                        "Headers",
                        Textarea::new(&self.headers),
                        cx,
                    )))
                    .child(div().flex_1().child(self.render_section(
                        "Body",
                        Textarea::new(&self.body),
                        cx,
                    ))),
            )
            .child(self.render_response(cx))
    }
}
