use gpui_kit::component::Sizable;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ActiveTheme, IndexPath, StyledExt};
use gpui_kit::{Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};

use rally_core::{AuthDraft, AuthKind};

const KINDS: [&str; 4] = ["No Auth", "Bearer Token", "Basic Auth", "API Key"];
const LABEL_WIDTH: f32 = 96.;
const ROW_HEIGHT: f32 = 36.;

pub struct AuthEditor {
    kind: AuthKind,
    in_query: bool,
    kind_select: Entity<SelectState<Vec<&'static str>>>,
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
        let token = cx.new(|cx| InputState::new(window, cx).placeholder("Token"));
        let username = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Password")
                .masked(true)
        });
        let key_name = cx.new(|cx| InputState::new(window, cx).placeholder("X-API-Key"));
        let key_value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));

        let subscriptions = vec![cx.subscribe_in(&kind_select, window, Self::on_kind_event)];
        Self {
            kind: AuthKind::None,
            in_query: false,
            kind_select,
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

    /// Replaces everything entered. The kind is assigned directly because setting a select's
    /// value does not emit a selection event.
    pub fn set_draft(&mut self, draft: &AuthDraft, window: &mut Window, cx: &mut Context<Self>) {
        self.kind = draft.kind;
        self.in_query = draft.key_in_query;

        let kind = KINDS[match draft.kind {
            AuthKind::None => 0,
            AuthKind::Bearer => 1,
            AuthKind::Basic => 2,
            AuthKind::ApiKey => 3,
        }];
        self.kind_select.update(cx, |select, cx| {
            select.set_selected_value(&kind, window, cx)
        });

        for (input, text) in [
            (&self.token, &draft.token),
            (&self.username, &draft.username),
            (&self.password, &draft.password),
            (&self.key_name, &draft.key_name),
            (&self.key_value, &draft.key_value),
        ] {
            input.update(cx, |input, cx| input.set_value(text.clone(), window, cx));
        }
        cx.notify();
    }

    /// Everything currently entered, including fields of schemes that are not selected.
    pub fn draft(&self, cx: &gpui_kit::App) -> AuthDraft {
        let read = |input: &Entity<InputState>| input.read(cx).value().to_string();

        AuthDraft {
            kind: self.kind,
            token: read(&self.token),
            username: read(&self.username),
            password: read(&self.password),
            key_name: read(&self.key_name),
            key_value: read(&self.key_value),
            key_in_query: self.in_query,
        }
    }
}

impl AuthEditor {
    /// A form row: muted label on the left, the control filling the rest.
    fn row(
        &self,
        label: &'static str,
        content: impl IntoElement,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .h_flex()
            .gap_3()
            .min_h(px(ROW_HEIGHT))
            .child(
                div()
                    .w(px(LABEL_WIDTH))
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(div().flex_1().min_w_0().child(content))
    }

    /// What the selected scheme does, in the words of the tab it replaces in Postman.
    fn help(&self) -> &'static [&'static str] {
        match (self.kind, self.in_query) {
            (AuthKind::None, _) => &[],
            (AuthKind::ApiKey, true) => &["The key is appended to the request's query string."],
            (AuthKind::ApiKey, false) => &[
                "The key is added as a request header.",
                "A header with the same name set in Headers takes precedence.",
            ],
            _ => &[
                "The authorization header will be automatically generated when you send the request.",
                "An Authorization header set in Headers takes precedence.",
            ],
        }
    }
}

impl Render for AuthEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted) = (theme.border, theme.muted_foreground);

        let fields = match self.kind {
            AuthKind::None => div()
                .flex_1()
                .v_flex()
                .items_center()
                .justify_center()
                .gap_1()
                .text_color(muted)
                .child(div().font_semibold().child("No Auth"))
                .child(
                    div()
                        .text_sm()
                        .child("This request does not use any authorization."),
                ),
            AuthKind::Bearer => {
                div()
                    .v_flex()
                    .gap_3()
                    .child(self.row("Token", Input::new(&self.token), cx))
            }
            AuthKind::Basic => div()
                .v_flex()
                .gap_3()
                .child(self.row("Username", Input::new(&self.username), cx))
                .child(self.row("Password", Input::new(&self.password).mask_toggle(), cx)),
            AuthKind::ApiKey => div()
                .v_flex()
                .gap_3()
                .child(self.row("Key", Input::new(&self.key_name), cx))
                .child(self.row("Value", Input::new(&self.key_value), cx))
                .child(
                    self.row(
                        "Add to",
                        div().w(px(260.)).child(
                            TabBar::new("api-key-location")
                                .segmented()
                                .small()
                                .selected_index(usize::from(self.in_query))
                                .on_click(cx.listener(|this, index: &usize, _, cx| {
                                    this.in_query = *index == 1;
                                    cx.notify();
                                }))
                                .child(Tab::new().label("Header"))
                                .child(Tab::new().label("Query Params")),
                        ),
                        cx,
                    ),
                ),
        };

        div()
            .id("auth-editor")
            .size_full()
            .v_flex()
            .gap_3()
            .overflow_y_scroll()
            .child(self.row(
                "Auth Type",
                div().w(px(220.)).child(Select::new(&self.kind_select)),
                cx,
            ))
            .when(self.kind != AuthKind::None, |this| {
                this.child(
                    div()
                        .v_flex()
                        .gap_1()
                        .pl(px(LABEL_WIDTH + 12.))
                        .text_xs()
                        .text_color(muted)
                        .children(self.help().iter().map(|line| SharedString::from(*line))),
                )
                .child(div().h(px(1.)).bg(border))
            })
            .child(fields)
    }
}
