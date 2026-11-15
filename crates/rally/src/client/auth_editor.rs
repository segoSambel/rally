use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::{ActiveTheme, IndexPath, StyledExt};
use gpui_kit::{Context, Entity, Subscription, Window, div, prelude::*, px};

use rally_core::base64_encode;

const KINDS: [&str; 4] = ["No Auth", "Bearer Token", "Basic Auth", "API Key"];
const LOCATIONS: [&str; 2] = ["Header", "Query Params"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthKind {
    None,
    Bearer,
    Basic,
    ApiKey,
}

/// What the chosen auth scheme adds to a request.
#[derive(Default)]
pub struct AuthParts {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}

pub struct AuthEditor {
    kind: AuthKind,
    in_query: bool,
    kind_select: Entity<SelectState<Vec<&'static str>>>,
    location_select: Entity<SelectState<Vec<&'static str>>>,
    token: Entity<InputState>,
    username: Entity<InputState>,
    password: Entity<InputState>,
    key_name: Entity<InputState>,
    key_value: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl AuthEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let kind_select =
            cx.new(|cx| SelectState::new(KINDS.to_vec(), Some(IndexPath::default()), window, cx));
        let location_select = cx
            .new(|cx| SelectState::new(LOCATIONS.to_vec(), Some(IndexPath::default()), window, cx));
        let token = cx.new(|cx| InputState::new(window, cx).placeholder("Token"));
        let username = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Password")
                .masked(true)
        });
        let key_name = cx.new(|cx| InputState::new(window, cx).placeholder("X-API-Key"));
        let key_value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));

        let subscriptions = vec![
            cx.subscribe_in(&kind_select, window, Self::on_kind_event),
            cx.subscribe_in(&location_select, window, Self::on_location_event),
        ];
        Self {
            kind: AuthKind::None,
            in_query: false,
            kind_select,
            location_select,
            token,
            username,
            password,
            key_name,
            key_value,
            _subscriptions: subscriptions,
        }
    }

    fn on_kind_event(
        &mut self,
        _select: &Entity<SelectState<Vec<&'static str>>>,
        event: &SelectEvent<Vec<&'static str>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(name)) = event else {
            return;
        };
        self.kind = match KINDS.iter().position(|kind| kind == name) {
            Some(1) => AuthKind::Bearer,
            Some(2) => AuthKind::Basic,
            Some(3) => AuthKind::ApiKey,
            _ => AuthKind::None,
        };
        cx.notify();
    }

    fn on_location_event(
        &mut self,
        _select: &Entity<SelectState<Vec<&'static str>>>,
        event: &SelectEvent<Vec<&'static str>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(name)) = event {
            self.in_query = *name == LOCATIONS[1];
            cx.notify();
        }
    }

    /// Headers or query params the selected scheme contributes. Empty fields add nothing.
    pub fn parts(&self, cx: &gpui_kit::App) -> AuthParts {
        let read = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let mut parts = AuthParts::default();

        match self.kind {
            AuthKind::None => {}
            AuthKind::Bearer => {
                let token = read(&self.token);
                if !token.is_empty() {
                    parts
                        .headers
                        .push(("Authorization".into(), format!("Bearer {token}")));
                }
            }
            AuthKind::Basic => {
                let (user, pass) = (read(&self.username), self.password.read(cx).value());
                if !user.is_empty() || !pass.is_empty() {
                    let encoded = base64_encode(format!("{user}:{pass}").as_bytes());
                    parts
                        .headers
                        .push(("Authorization".into(), format!("Basic {encoded}")));
                }
            }
            AuthKind::ApiKey => {
                let (name, value) = (read(&self.key_name), read(&self.key_value));
                if !name.is_empty() {
                    let target = if self.in_query {
                        &mut parts.query
                    } else {
                        &mut parts.headers
                    };
                    target.push((name, value));
                }
            }
        }
        parts
    }
}

impl AuthEditor {
    fn field(
        &self,
        label: &'static str,
        content: impl IntoElement,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .v_flex()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(content)
    }
}

impl Render for AuthEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fields = match self.kind {
            AuthKind::None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("This request does not use any authorization."),
            AuthKind::Bearer => {
                div()
                    .v_flex()
                    .gap_3()
                    .child(self.field("Token", Input::new(&self.token), cx))
            }
            AuthKind::Basic => div()
                .v_flex()
                .gap_3()
                .child(self.field("Username", Input::new(&self.username), cx))
                .child(self.field("Password", Input::new(&self.password), cx)),
            AuthKind::ApiKey => div()
                .v_flex()
                .gap_3()
                .child(self.field("Key", Input::new(&self.key_name), cx))
                .child(self.field("Value", Input::new(&self.key_value), cx))
                .child(self.field(
                    "Add to",
                    div().w(px(200.)).child(Select::new(&self.location_select)),
                    cx,
                )),
        };

        div()
            .id("auth-editor")
            .size_full()
            .v_flex()
            .gap_4()
            .overflow_y_scroll()
            .child(self.field(
                "Auth type",
                div().w(px(200.)).child(Select::new(&self.kind_select)),
                cx,
            ))
            .child(div().max_w(px(420.)).child(fields))
    }
}
