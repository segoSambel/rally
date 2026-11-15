use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{
    Editor, EditorState, Input, InputEvent, InputState, Textarea, TextareaState,
};
use gpui_kit::component::searchable_list::SearchableListItem;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, IndexPath, Sizable, StyledExt, Theme, ThemeMode,
};
use gpui_kit::{
    AnyElement, App, Context, Entity, Hsla, SharedString, Subscription, Window, div, prelude::*,
    px, rgb,
};

use rally_core::{
    ApiRequest, ApiResponse, COMMON_HEADER_NAMES, Method, format_body, format_duration, format_size,
};

use super::auth_editor::AuthEditor;
use super::client_model::{ClientModel, RequestState};
use super::key_value_editor::KeyValueEditor;

type MethodSelect = Vec<MethodItem>;

/// Postman-style color for each HTTP method.
fn method_color(method: Method) -> Hsla {
    match method {
        Method::Get => rgb(0x0cbb52),
        Method::Post => rgb(0xe8a000),
        Method::Put => rgb(0x097bed),
        Method::Patch => rgb(0x7d65e8),
        Method::Delete => rgb(0xf05050),
    }
    .into()
}

#[derive(Clone)]
struct MethodItem(Method);

impl SearchableListItem for MethodItem {
    type Value = Method;

    fn title(&self) -> SharedString {
        self.0.as_str().into()
    }

    fn display_title(&self) -> Option<AnyElement> {
        Some(self.render_label().into_any_element())
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.render_label()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

impl MethodItem {
    fn render_label(&self) -> impl IntoElement {
        div()
            .font_semibold()
            .text_color(method_color(self.0))
            .child(self.0.as_str())
    }
}

pub struct ClientView {
    model: Entity<ClientModel>,
    method: Method,
    method_select: Entity<SelectState<MethodSelect>>,
    url: Entity<InputState>,
    params: Entity<KeyValueEditor>,
    auth: Entity<AuthEditor>,
    headers: Entity<KeyValueEditor>,
    body: Entity<TextareaState>,
    response_body: Entity<EditorState>,
    request_tab: usize,
    response_tab: usize,
    _subscriptions: Vec<Subscription>,
}

impl ClientView {
    pub fn new(model: Entity<ClientModel>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let methods: MethodSelect = Method::ALL.into_iter().map(MethodItem).collect();
        let method_select =
            cx.new(|cx| SelectState::new(methods, Some(IndexPath::default()), window, cx));
        let url = cx.new(|cx| InputState::new(window, cx).placeholder("https://api.example.com"));
        let params = cx.new(|cx| KeyValueEditor::new("Parameter", &[], window, cx));
        let auth = cx.new(|cx| AuthEditor::new(window, cx));
        let headers = cx.new(|cx| KeyValueEditor::new("Header", COMMON_HEADER_NAMES, window, cx));
        let body = cx.new(|cx| TextareaState::new(window, cx).placeholder("Request body"));
        let response_body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("json")
                .soft_wrap(true)
        });

        let subscriptions = vec![
            cx.observe_in(&model, window, Self::on_model_change),
            cx.subscribe_in(&url, window, Self::on_url_event),
            cx.subscribe_in(&method_select, window, Self::on_method_event),
            // Re-render when a table changes so the tab counts stay current.
            cx.observe(&params, |_, _, cx| cx.notify()),
            cx.observe(&headers, |_, _, cx| cx.notify()),
        ];
        Self {
            model,
            method: Method::Get,
            method_select,
            url,
            params,
            auth,
            headers,
            body,
            response_body,
            request_tab: 0,
            response_tab: 0,
            _subscriptions: subscriptions,
        }
    }

    fn on_model_change(
        &mut self,
        model: Entity<ClientModel>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (text, is_json) = match model.read(cx).state() {
            RequestState::Done(response) => format_body(&response.body),
            _ => (String::new(), false),
        };
        self.response_body.update(cx, |state, cx| {
            state.set_highlighter(if is_json { "json" } else { "text" }, cx);
            state.set_value(text, window, cx);
        });
        cx.notify();
    }

    fn on_method_event(
        &mut self,
        _select: &Entity<SelectState<MethodSelect>>,
        event: &SelectEvent<MethodSelect>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(method)) = event {
            self.method = *method;
            cx.notify();
        }
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

        let request = ApiRequest::build(
            self.method,
            &url,
            self.params.read(cx).pairs(cx),
            self.headers.read(cx).pairs(cx),
            &self.auth.read(cx).auth(cx),
            self.body.read(cx).value().to_string(),
        );
        self.model.update(cx, |model, cx| model.send(request, cx));
    }
}

impl ClientView {
    fn render_url_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        let sending = self.model.read(cx).is_sending();

        div()
            .h_flex()
            .gap_2()
            .px_4()
            .py_3()
            .bg(cx.theme().tab_bar)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                // One bordered field holding the method picker and the URL, like Bruno.
                div()
                    .h_flex()
                    .flex_1()
                    .h(px(40.))
                    .px_1()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        div()
                            .w(px(112.))
                            .child(Select::new(&self.method_select).appearance(false)),
                    )
                    .child(div().w(px(1.)).h(px(20.)).bg(cx.theme().border))
                    .child(
                        div().flex_1().child(
                            Input::new(&self.url)
                                .appearance(false)
                                .focus_bordered(false),
                        ),
                    ),
            )
            .child(
                Button::new("send")
                    .primary()
                    .label("Send")
                    .loading(sending)
                    .on_click(cx.listener(|this, _, _, cx| this.send(cx))),
            )
            .child(
                Button::new("theme-toggle")
                    .ghost()
                    .icon(if cx.theme().is_dark() {
                        IconName::Sun
                    } else {
                        IconName::Moon
                    })
                    .tooltip("Toggle light/dark theme")
                    .on_click(|_, window, cx| {
                        let mode = if cx.theme().is_dark() {
                            ThemeMode::Light
                        } else {
                            ThemeMode::Dark
                        };
                        Theme::change(mode, Some(window), cx);
                    }),
            )
    }

    fn tab_with_count(&self, label: &'static str, count: usize, cx: &Context<Self>) -> Tab {
        let tab = Tab::new().label(label);
        if count == 0 {
            return tab;
        }
        tab.suffix(
            div()
                .ml_1()
                .text_xs()
                .text_color(cx.theme().primary)
                .child(count.to_string()),
        )
    }

    fn render_request_pane(&self, cx: &Context<Self>) -> impl IntoElement {
        let editor = match self.request_tab {
            0 => self.params.clone().into_any_element(),
            1 => self.auth.clone().into_any_element(),
            2 => self.headers.clone().into_any_element(),
            _ => Textarea::new(&self.body).h_full().into_any_element(),
        };

        div()
            .v_flex()
            .flex_1()
            .min_w_0()
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                TabBar::new("request-tabs")
                    .underline()
                    .prefix(div().w(px(12.)))
                    .selected_index(self.request_tab)
                    .on_click(cx.listener(|this, index: &usize, _, cx| {
                        this.request_tab = *index;
                        cx.notify();
                    }))
                    .child(self.tab_with_count("Params", self.params.read(cx).count(cx), cx))
                    .child(Tab::new().label("Authorization"))
                    .child(self.tab_with_count("Headers", self.headers.read(cx).count(cx), cx))
                    .child(Tab::new().label("Body")),
            )
            .child(div().flex_1().min_h_0().p_4().child(editor))
    }

    fn render_response_pane(&self, cx: &Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let header_count = match model.state() {
            RequestState::Done(response) => response.headers.len(),
            _ => 0,
        };

        div()
            .v_flex()
            .flex_1()
            .min_w_0()
            .child(
                TabBar::new("response-tabs")
                    .underline()
                    .prefix(div().w(px(12.)))
                    .suffix(self.render_status(cx))
                    .selected_index(self.response_tab)
                    .on_click(cx.listener(|this, index: &usize, _, cx| {
                        this.response_tab = *index;
                        cx.notify();
                    }))
                    .child(Tab::new().label("Body"))
                    .child(Tab::new().label(format!("Headers ({header_count})"))),
            )
            .child(self.render_response_content(cx))
    }

    fn render_status(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let row = div()
            .h_flex()
            .gap_3()
            .px_3()
            .items_center()
            .ml_auto()
            .text_sm();

        match self.model.read(cx).state() {
            RequestState::Done(response) => {
                let color = match response.status {
                    200..=299 => theme.success,
                    400..=599 => theme.danger,
                    _ => theme.warning,
                };
                let status = format!("{} {}", response.status, response.reason);
                row.child(
                    div()
                        .font_semibold()
                        .text_color(color)
                        .child(SharedString::from(status)),
                )
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(format_duration(response.elapsed)),
                )
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(format_size(response.body.len())),
                )
            }
            _ => row,
        }
    }

    fn render_empty_state(
        icon: impl IntoElement,
        title: &'static str,
        hint: Option<SharedString>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .items_center()
            .justify_center()
            .gap_2()
            .text_color(cx.theme().muted_foreground)
            .child(icon)
            .child(div().font_semibold().child(title))
            .children(hint.map(|hint| div().text_sm().child(hint)))
    }

    fn render_response_content(&self, cx: &Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let container = div().id("response-content").flex_1().min_h_0();

        match self.model.read(cx).state() {
            RequestState::Idle => container.p_4().child(Self::render_empty_state(
                Icon::new(IconName::Globe).size_8(),
                "Ready when you are",
                Some("Enter a URL and press Send or Enter.".into()),
                cx,
            )),
            RequestState::Sending => container.p_4().child(Self::render_empty_state(
                Spinner::new().large().color(muted),
                "Sending request...",
                None,
                cx,
            )),
            RequestState::Failed(message) => container.p_4().child(Self::render_empty_state(
                Icon::new(IconName::CircleAlert)
                    .size_8()
                    .text_color(cx.theme().danger),
                "Request failed",
                Some(SharedString::from(message.clone())),
                cx,
            )),
            RequestState::Done(response) if self.response_tab == 0 => {
                if response.body.is_empty() {
                    container.p_4().child(Self::render_empty_state(
                        Icon::new(IconName::Inbox).size_8(),
                        "No response body",
                        None,
                        cx,
                    ))
                } else {
                    container.child(
                        Editor::new(&self.response_body)
                            .readonly(true)
                            .bordered(false)
                            .h_full(),
                    )
                }
            }
            RequestState::Done(response) => container
                .p_4()
                .overflow_y_scroll()
                .child(self.render_response_headers(response, cx)),
        }
    }

    fn render_response_headers(&self, response: &ApiResponse, cx: &Context<Self>) -> gpui_kit::Div {
        div()
            .v_flex()
            .gap_1()
            .text_sm()
            .children(response.headers.iter().map(|(name, value)| {
                div()
                    .h_flex()
                    .gap_3()
                    .items_start()
                    .child(
                        div()
                            .w(px(180.))
                            .flex_shrink_0()
                            .font_semibold()
                            .text_color(cx.theme().muted_foreground)
                            .child(SharedString::from(name.clone())),
                    )
                    .child(div().flex_1().child(SharedString::from(value.clone())))
            }))
    }
}

impl Render for ClientView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_url_bar(cx))
            .child(
                div()
                    .h_flex()
                    .items_stretch()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_request_pane(cx))
                    .child(self.render_response_pane(cx)),
            )
    }
}
