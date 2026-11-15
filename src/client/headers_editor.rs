use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, IconName, Sizable, StyledExt};
use gpui_kit::{
    Anchor, Context, Entity, MouseButton, SharedString, Subscription, Window, anchored, deferred,
    div, point, prelude::*, px,
};

const COMMON_HEADERS: &[&str] = &[
    "Accept",
    "Accept-Charset",
    "Accept-Encoding",
    "Accept-Language",
    "Authorization",
    "Cache-Control",
    "Connection",
    "Content-Encoding",
    "Content-Length",
    "Content-Type",
    "Cookie",
    "DNT",
    "Host",
    "If-Match",
    "If-Modified-Since",
    "If-None-Match",
    "Origin",
    "Pragma",
    "Range",
    "Referer",
    "TE",
    "User-Agent",
    "X-Api-Key",
    "X-Forwarded-For",
    "X-Requested-With",
];
const MAX_SUGGESTIONS: usize = 6;

struct HeaderRow {
    id: usize,
    enabled: bool,
    key: Entity<InputState>,
    value: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

/// Key/value table of request headers. Always ends with one empty row.
pub struct HeadersEditor {
    rows: Vec<HeaderRow>,
    next_id: usize,
    /// Row whose key field is showing suggestions.
    active: Option<usize>,
    hovering_suggestions: bool,
}

impl HeadersEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            rows: Vec::new(),
            next_id: 0,
            active: None,
            hovering_suggestions: false,
        };
        this.push_row(window, cx);
        this
    }

    /// Enabled rows with a non-empty key.
    pub fn headers(&self, cx: &gpui_kit::App) -> Vec<(String, String)> {
        self.rows
            .iter()
            .filter(|row| row.enabled)
            .map(|row| {
                (
                    row.key.read(cx).value().trim().to_string(),
                    row.value.read(cx).value().trim().to_string(),
                )
            })
            .filter(|(key, _)| !key.is_empty())
            .collect()
    }

    fn push_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        let key = cx.new(|cx| InputState::new(window, cx).placeholder("Header"));
        let value = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        let subscriptions = vec![
            cx.subscribe_in(&key, window, move |this, _, event, window, cx| {
                this.on_key_event(id, event, window, cx)
            }),
            cx.subscribe_in(&value, window, |this, _, event, window, cx| {
                this.on_value_event(event, window, cx)
            }),
        ];
        self.rows.push(HeaderRow {
            id,
            enabled: true,
            key,
            value,
            _subscriptions: subscriptions,
        });
    }

    /// Keeps exactly one empty row at the end of the table.
    fn ensure_trailing_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let last_is_empty = self.rows.last().is_some_and(|row| {
            row.key.read(cx).value().is_empty() && row.value.read(cx).value().is_empty()
        });
        if !last_is_empty {
            self.push_row(window, cx);
        }
    }

    fn row(&self, id: usize) -> Option<&HeaderRow> {
        self.rows.iter().find(|row| row.id == id)
    }

    fn suggestions(&self, id: usize, cx: &gpui_kit::App) -> Vec<&'static str> {
        if self.active != Some(id) {
            return Vec::new();
        }
        let Some(row) = self.row(id) else {
            return Vec::new();
        };
        let typed = row.key.read(cx).value().trim().to_lowercase();
        if typed.is_empty() {
            return Vec::new();
        }
        COMMON_HEADERS
            .iter()
            .copied()
            .filter(|name| {
                let name = name.to_lowercase();
                name.starts_with(&typed) && name != typed
            })
            .take(MAX_SUGGESTIONS)
            .collect()
    }

    fn accept(
        &mut self,
        id: usize,
        name: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.row(id) else { return };
        let (key, value) = (row.key.clone(), row.value.clone());
        key.update(cx, |state, cx| state.set_value(name, window, cx));
        value.update(cx, |state, cx| state.focus(window, cx));
        self.active = None;
        self.hovering_suggestions = false;
        cx.notify();
    }

    fn on_key_event(
        &mut self,
        id: usize,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Focus => self.active = Some(id),
            InputEvent::Blur => {
                // A click on a suggestion blurs the field first; keep the list alive for it.
                if !self.hovering_suggestions && self.active == Some(id) {
                    self.active = None;
                }
            }
            InputEvent::Change => {
                self.active = Some(id);
                self.ensure_trailing_row(window, cx);
            }
            InputEvent::PressEnter { .. } => match self.suggestions(id, cx).first() {
                Some(name) => return self.accept(id, name, window, cx),
                None => {
                    if let Some(row) = self.row(id) {
                        let value = row.value.clone();
                        value.update(cx, |state, cx| state.focus(window, cx));
                    }
                    self.active = None;
                }
            },
        }
        cx.notify();
    }

    fn on_value_event(&mut self, event: &InputEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            InputEvent::Focus => self.active = None,
            InputEvent::Change => self.ensure_trailing_row(window, cx),
            _ => {}
        }
        cx.notify();
    }

    fn remove(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        if self.active == Some(id) {
            self.active = None;
        }
        self.ensure_trailing_row(window, cx);
        cx.notify();
    }
}

impl HeadersEditor {
    fn render_suggestions(
        &self,
        id: usize,
        suggestions: Vec<&'static str>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        deferred(
            anchored()
                .anchor(Anchor::TopLeft)
                .offset(point(px(0.), px(38.)))
                .snap_to_window()
                .child(
                    div()
                        .id(("suggestions", id))
                        .v_flex()
                        .w(px(240.))
                        .py_1()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.background)
                        .shadow_md()
                        .occlude()
                        .on_hover(cx.listener(|this, hovering: &bool, _, _| {
                            this.hovering_suggestions = *hovering;
                        }))
                        .children(suggestions.into_iter().enumerate().map(|(index, name)| {
                            div()
                                .id(("suggestion", index))
                                .px_3()
                                .py_1()
                                .text_sm()
                                .cursor_pointer()
                                .hover(|style| style.bg(theme.accent))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, window, cx| {
                                        this.accept(id, name, window, cx)
                                    }),
                                )
                                .child(SharedString::from(name))
                        })),
                ),
        )
        .with_priority(1)
    }

    fn render_row(&self, row: &HeaderRow, is_last: bool, cx: &Context<Self>) -> impl IntoElement {
        let id = row.id;
        let suggestions = self.suggestions(id, cx);

        div()
            .h_flex()
            .gap_2()
            .px_1()
            .py_1()
            .rounded_md()
            .hover(|style| style.bg(cx.theme().muted.opacity(0.5)))
            .when(!row.enabled, |this| this.opacity(0.5))
            .child(div().w(px(20.)).when(!is_last, |this| {
                this.child(
                    Checkbox::new(("header-enabled", id))
                        .checked(row.enabled)
                        .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                            if let Some(row) = this.rows.iter_mut().find(|row| row.id == id) {
                                row.enabled = *checked;
                                cx.notify();
                            }
                        })),
                )
            }))
            .child(
                div()
                    .flex_1()
                    .child(Input::new(&row.key))
                    .when(!suggestions.is_empty(), |this| {
                        this.child(self.render_suggestions(id, suggestions, cx))
                    }),
            )
            .child(div().flex_1().child(Input::new(&row.value)))
            .child(div().w(px(32.)).when(!is_last, |this| {
                this.child(
                    Button::new(("header-remove", id))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Close)
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.remove(id, window, cx)),
                        ),
                )
            }))
    }
}

impl Render for HeadersEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let last = self.rows.len().saturating_sub(1);

        let muted = cx.theme().muted_foreground;

        div()
            .id("headers-editor")
            .size_full()
            .v_flex()
            .gap_2()
            .overflow_y_scroll()
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .text_xs()
                    .text_color(muted)
                    .child(div().w(px(20.)))
                    .child(div().flex_1().child("Key"))
                    .child(div().flex_1().child("Value"))
                    .child(div().w(px(32.))),
            )
            .children(
                self.rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| self.render_row(row, index == last, cx)),
            )
    }
}
