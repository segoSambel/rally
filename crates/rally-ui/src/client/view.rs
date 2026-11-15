use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Editor, EditorState, Input, InputEvent, InputState};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, IconName, IndexPath, Sizable, StyledExt, Theme, ThemeMode,
};
use gpui_kit::{
    ClipboardItem, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px,
};

use rally_core::{
    ApiResponse, BodyKind, COMMON_HEADER_NAMES, Method, RequestDraft, common_header_values,
    format_body, format_duration, format_size,
};

use super::model::{ClientModel, RequestCompleted, RequestState};
use crate::history::{HistoryModel, HistoryPanel, OpenEntry};
use crate::widgets::{
    AuthEditor, BodyDraft, BodyEditor, KeyValueEditor, MethodItem, MethodSelect, status_color,
};

pub struct ClientView {
    model: Entity<ClientModel>,
    history: Entity<HistoryModel>,
    history_panel: Entity<HistoryPanel>,
    sidebar_open: bool,
    /// Briefly true after copying, to show a confirmation icon.
    copied: bool,
    method: Method,
    method_select: Entity<SelectState<MethodSelect>>,
    url: Entity<InputState>,
    params: Entity<KeyValueEditor>,
    auth: Entity<AuthEditor>,
    headers: Entity<KeyValueEditor>,
    body: Entity<BodyEditor>,
    response_body: Entity<EditorState>,
    request_tab: usize,
    response_tab: usize,
    _subscriptions: Vec<Subscription>,
}

impl ClientView {
    pub fn new(model: Entity<ClientModel>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let history = cx.new(|_| HistoryModel::new());
        let history_panel = cx.new(|cx| HistoryPanel::new(history.clone(), cx));
        let methods: MethodSelect = Method::ALL.into_iter().map(MethodItem).collect();
        let method_select =
            cx.new(|cx| SelectState::new(methods, Some(IndexPath::default()), window, cx));
        let url = cx.new(|cx| InputState::new(window, cx).placeholder("https://api.example.com"));
        let params = cx.new(|cx| KeyValueEditor::new("Parameter", &[], None, window, cx));
        let auth = cx.new(|cx| AuthEditor::new(window, cx));
        let headers = cx.new(|cx| {
            KeyValueEditor::new(
                "Header",
                COMMON_HEADER_NAMES,
                Some(common_header_values),
                window,
                cx,
            )
        });
        let body = cx.new(|cx| BodyEditor::new(window, cx));
        let response_body = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("json")
                .soft_wrap(true)
        });

        let subscriptions = vec![
            cx.observe_in(&model, window, Self::on_model_change),
            cx.subscribe_in(&url, window, Self::on_url_event),
            cx.subscribe_in(&method_select, window, Self::on_method_event),
            cx.subscribe(&model, Self::on_request_completed),
            cx.subscribe_in(&history_panel, window, Self::on_open_entry),
            // Re-render when a table changes so the tab counts stay current.
            cx.observe(&params, |_, _, cx| cx.notify()),
            cx.observe(&headers, |_, _, cx| cx.notify()),
            cx.observe(&body, |_, _, cx| cx.notify()),
        ];
        Self {
            model,
            history,
            history_panel,
            sidebar_open: true,
            copied: false,
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

    fn on_request_completed(
        &mut self,
        _model: Entity<ClientModel>,
        event: &RequestCompleted,
        cx: &mut Context<Self>,
    ) {
        self.history.update(cx, |history, cx| {
            history.record(
                event.draft.clone(),
                event.status,
                event.elapsed_ms,
                event.error.clone(),
                cx,
            )
        });
    }

    fn on_open_entry(
        &mut self,
        _panel: &Entity<HistoryPanel>,
        event: &OpenEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.restore(&event.0, window, cx);
    }

    /// Loads a draft back into the editors.
    fn restore(&mut self, draft: &RequestDraft, window: &mut Window, cx: &mut Context<Self>) {
        // The select does not emit an event when set from code, so assign the field too.
        self.method = draft.method;
        self.method_select.update(cx, |select, cx| {
            select.set_selected_value(&draft.method, window, cx)
        });
        self.url
            .update(cx, |url, cx| url.set_value(draft.url.clone(), window, cx));
        self.params
            .update(cx, |params, cx| params.set_rows(&draft.params, window, cx));
        self.headers.update(cx, |headers, cx| {
            headers.set_rows(&draft.headers, window, cx)
        });
        self.auth
            .update(cx, |auth, cx| auth.set_draft(&draft.auth, window, cx));
        let body = BodyDraft {
            kind: draft.body_kind,
            text: draft.body.clone(),
            form: draft.form.clone(),
        };
        self.body
            .update(cx, |editor, cx| editor.set_draft(&body, window, cx));
        cx.notify();
    }

    /// Copies what the active response tab shows: the body, or the headers as `Name: value`.
    fn copy_response(&mut self, cx: &mut Context<Self>) {
        let model = self.model.read(cx);
        let RequestState::Done(response) = model.state() else {
            return;
        };
        let text = if self.response_tab == 0 {
            self.response_body.read(cx).value().to_string()
        } else {
            response
                .headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.copied = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            this.update(cx, |this, cx| {
                this.copied = false;
                cx.notify();
            })
            .ok();
        })
        .detach();
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

    /// The current editor contents.
    fn snapshot(&self, cx: &Context<Self>) -> RequestDraft {
        let body = self.body.read(cx).draft(cx);
        RequestDraft {
            method: self.method,
            url: self.url.read(cx).value().to_string(),
            params: self.params.read(cx).rows(cx),
            headers: self.headers.read(cx).rows(cx),
            auth: self.auth.read(cx).draft(cx),
            body_kind: body.kind,
            body: body.text,
            form: body.form,
        }
    }

    fn send(&mut self, cx: &mut Context<Self>) {
        let draft = self.snapshot(cx);
        if draft.url.trim().is_empty() {
            self.model
                .update(cx, |model, cx| model.fail("Enter a URL first.".into(), cx));
            return;
        }

        let request = draft.build();
        self.model
            .update(cx, |model, cx| model.send(request, draft, cx));
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
                Button::new("sidebar-toggle")
                    .ghost()
                    .icon(if self.sidebar_open {
                        IconName::PanelLeftClose
                    } else {
                        IconName::PanelLeftOpen
                    })
                    .tooltip("Toggle history")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_open = !this.sidebar_open;
                        cx.notify();
                    })),
            )
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

    /// The Body tab, with a dot when the request has a body.
    fn body_tab(&self, cx: &Context<Self>) -> Tab {
        let tab = Tab::new().label("Body");
        if self.body.read(cx).kind() == BodyKind::None {
            return tab;
        }
        tab.suffix(
            div()
                .ml_1()
                .text_xs()
                .text_color(cx.theme().primary)
                .child("●"),
        )
    }

    fn render_request_pane(&self, cx: &Context<Self>) -> impl IntoElement {
        let editor = match self.request_tab {
            0 => self.params.clone().into_any_element(),
            1 => self.auth.clone().into_any_element(),
            2 => self.headers.clone().into_any_element(),
            _ => self.body.clone().into_any_element(),
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
                    .child(self.body_tab(cx)),
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
                let color = status_color(theme, response.status);
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
                .child(
                    Button::new("copy-response")
                        .ghost()
                        .xsmall()
                        .icon(if self.copied {
                            IconName::Check
                        } else {
                            IconName::Copy
                        })
                        .tooltip(if self.response_tab == 0 {
                            "Copy body"
                        } else {
                            "Copy headers"
                        })
                        .disabled(self.response_tab == 0 && response.body.is_empty())
                        .on_click(cx.listener(|this, _, _, cx| this.copy_response(cx))),
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
        let border = cx.theme().border;

        div()
            .size_full()
            .h_flex()
            .items_stretch()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .when(self.sidebar_open, |this| {
                this.child(
                    div()
                        .w(px(240.))
                        .flex_shrink_0()
                        .border_r_1()
                        .border_color(border)
                        .child(self.history_panel.clone()),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .v_flex()
                    .child(self.render_url_bar(cx))
                    .child(
                        div()
                            .h_flex()
                            .items_stretch()
                            .flex_1()
                            .min_h_0()
                            .child(self.render_request_pane(cx))
                            .child(self.render_response_pane(cx)),
                    ),
            )
    }
}
