use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, IconName, Sizable, StyledExt};
use gpui_kit::{
    Anchor, Context, Entity, MouseButton, SharedString, Subscription, Window, anchored, deferred,
    div, point, prelude::*, px,
};

use rally_core::KeyValueRow;

const MAX_SUGGESTIONS: usize = 6;
const ROW_HEIGHT: f32 = 36.;
const CHECK_COL: f32 = 40.;
const REMOVE_COL: f32 = 36.;

/// Which field of a row a suggestion dropdown belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Key,
    Value,
}

/// Looks up suggested values for a key.
pub type ValueSuggestions = fn(&str) -> &'static [&'static str];

struct HeaderRow {
    id: usize,
    enabled: bool,
    key: Entity<InputState>,
    value: Entity<InputState>,
    /// Free-form note for the row; never sent with the request.
    description: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

/// Key/value table (headers, query params). Always ends with one empty row.
pub struct KeyValueEditor {
    key_placeholder: &'static str,
    /// Key names offered by the autocomplete dropdown; empty disables it.
    suggestions: &'static [&'static str],
    /// Values offered for a key's value field; `None` disables it.
    value_suggestions: Option<ValueSuggestions>,
    rows: Vec<HeaderRow>,
    next_id: usize,
    /// Field currently showing suggestions.
    active: Option<(usize, Field)>,
    hovering_suggestions: bool,
}

impl KeyValueEditor {
    pub fn new(
        key_placeholder: &'static str,
        suggestions: &'static [&'static str],
        value_suggestions: Option<ValueSuggestions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            key_placeholder,
            suggestions,
            value_suggestions,
            rows: Vec::new(),
            next_id: 0,
            active: None,
            hovering_suggestions: false,
        };
        this.push_row(None, window, cx);
        this
    }

    /// The table contents, skipping rows that are completely empty.
    pub fn rows(&self, cx: &gpui_kit::App) -> Vec<KeyValueRow> {
        self.rows
            .iter()
            .map(|row| KeyValueRow {
                enabled: row.enabled,
                key: row.key.read(cx).value().to_string(),
                value: row.value.read(cx).value().to_string(),
                description: row.description.read(cx).value().to_string(),
            })
            .filter(|row| {
                !(row.key.is_empty() && row.value.is_empty() && row.description.is_empty())
            })
            .collect()
    }

    /// Number of rows that will be sent.
    pub fn count(&self, cx: &gpui_kit::App) -> usize {
        self.rows(cx)
            .iter()
            .filter(|row| row.enabled && !row.key.trim().is_empty())
            .count()
    }

    /// Replaces the table contents. Inputs are built with their text already set, so no
    /// change events fire and the autocomplete dropdown stays closed.
    pub fn set_rows(&mut self, rows: &[KeyValueRow], window: &mut Window, cx: &mut Context<Self>) {
        self.rows.clear();
        self.active = None;
        self.hovering_suggestions = false;
        for row in rows {
            self.push_row(Some(row), window, cx);
        }
        self.ensure_trailing_row(window, cx);
        cx.notify();
    }

    fn push_row(
        &mut self,
        initial: Option<&KeyValueRow>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let text =
            |select: fn(&KeyValueRow) -> &String| initial.map(select).cloned().unwrap_or_default();
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(self.key_placeholder)
                .default_value(text(|row| &row.key))
        });
        let value = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Value")
                .default_value(text(|row| &row.value))
        });
        let description = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Description")
                .default_value(text(|row| &row.description))
        });
        let subscriptions = vec![
            cx.subscribe_in(&key, window, move |this, _, event, window, cx| {
                this.on_key_event(id, event, window, cx)
            }),
            cx.subscribe_in(&value, window, move |this, _, event, window, cx| {
                this.on_value_event(id, event, window, cx)
            }),
            cx.subscribe_in(&description, window, |this, _, event, window, cx| {
                this.on_description_event(event, window, cx)
            }),
        ];
        self.rows.push(HeaderRow {
            id,
            enabled: initial.is_none_or(|row| row.enabled),
            key,
            value,
            description,
            _subscriptions: subscriptions,
        });
    }

    /// Keeps exactly one empty row at the end of the table.
    fn ensure_trailing_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let last_is_empty = self.rows.last().is_some_and(|row| {
            [&row.key, &row.value, &row.description]
                .iter()
                .all(|input| input.read(cx).value().is_empty())
        });
        if !last_is_empty {
            self.push_row(None, window, cx);
        }
    }

    fn row(&self, id: usize) -> Option<&HeaderRow> {
        self.rows.iter().find(|row| row.id == id)
    }

    fn suggestions(&self, id: usize, field: Field, cx: &gpui_kit::App) -> Vec<&'static str> {
        if self.active != Some((id, field)) {
            return Vec::new();
        }
        let Some(row) = self.row(id) else {
            return Vec::new();
        };

        let (candidates, typed): (&[&'static str], String) = match field {
            Field::Key => {
                let typed = row.key.read(cx).value().trim().to_lowercase();
                if typed.is_empty() {
                    return Vec::new();
                }
                (self.suggestions, typed)
            }
            Field::Value => {
                let Some(lookup) = self.value_suggestions else {
                    return Vec::new();
                };
                // An empty value lists every suggestion; the text is not trimmed so that
                // an accepted "Bearer " does not match itself again.
                let key = row.key.read(cx).value();
                (
                    lookup(key.trim()),
                    row.value.read(cx).value().to_lowercase(),
                )
            }
        };

        candidates
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
        field: Field,
        name: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.row(id) else { return };
        let (key, value) = (row.key.clone(), row.value.clone());
        match field {
            Field::Key => {
                key.update(cx, |state, cx| state.set_value(name, window, cx));
                // Focusing the value field opens its own suggestions.
                value.update(cx, |state, cx| state.focus(window, cx));
            }
            Field::Value => value.update(cx, |state, cx| state.set_value(name, window, cx)),
        }
        self.active = None;
        self.hovering_suggestions = false;
        cx.notify();
    }

    /// Closes the dropdown on blur, unless the pointer is over it (a click on a suggestion
    /// blurs the field first and must still land).
    fn close_on_blur(&mut self, id: usize, field: Field) {
        if !self.hovering_suggestions && self.active == Some((id, field)) {
            self.active = None;
        }
    }

    fn on_key_event(
        &mut self,
        id: usize,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Focus => self.active = Some((id, Field::Key)),
            InputEvent::Blur => self.close_on_blur(id, Field::Key),
            InputEvent::Change => {
                self.active = Some((id, Field::Key));
                self.ensure_trailing_row(window, cx);
            }
            InputEvent::PressEnter { .. } => match self.suggestions(id, Field::Key, cx).first() {
                Some(name) => return self.accept(id, Field::Key, name, window, cx),
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

    fn on_value_event(
        &mut self,
        id: usize,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Focus => self.active = Some((id, Field::Value)),
            InputEvent::Blur => self.close_on_blur(id, Field::Value),
            InputEvent::Change => {
                self.active = Some((id, Field::Value));
                self.ensure_trailing_row(window, cx);
            }
            InputEvent::PressEnter { .. } => {
                // Enter on an empty value must not pick a suggestion by accident.
                let typed = self
                    .row(id)
                    .is_some_and(|row| !row.value.read(cx).value().is_empty());
                if typed && let Some(name) = self.suggestions(id, Field::Value, cx).first() {
                    return self.accept(id, Field::Value, name, window, cx);
                }
            }
        }
        cx.notify();
    }

    fn on_description_event(
        &mut self,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Focus => self.active = None,
            InputEvent::Change => self.ensure_trailing_row(window, cx),
            _ => {}
        }
        cx.notify();
    }

    fn remove(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        if self.active.is_some_and(|(row, _)| row == id) {
            self.active = None;
        }
        self.ensure_trailing_row(window, cx);
        cx.notify();
    }
}

impl KeyValueEditor {
    fn render_suggestions(
        &self,
        id: usize,
        field: Field,
        suggestions: Vec<&'static str>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let width = match field {
            Field::Key => 240.,
            Field::Value => 300.,
        };

        deferred(
            anchored()
                .anchor(Anchor::TopLeft)
                .offset(point(px(0.), px(ROW_HEIGHT)))
                .snap_to_window()
                .child(
                    div()
                        .id(("suggestions", id * 2 + field as usize))
                        .v_flex()
                        .w(px(width))
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
                                        this.accept(id, field, name, window, cx)
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
        let key_suggestions = self.suggestions(id, Field::Key, cx);
        let value_suggestions = self.suggestions(id, Field::Value, cx);
        let border = cx.theme().border;

        let cell = |input: &Entity<InputState>| {
            div().flex_1().min_w_0().h_full().h_flex().child(
                div()
                    .flex_1()
                    .child(Input::new(input).appearance(false).focus_bordered(false)),
            )
        };
        let divider = || div().w(px(1.)).h_full().bg(border);

        div()
            .h_flex()
            .h(px(ROW_HEIGHT))
            .border_t_1()
            .border_color(border)
            .hover(|style| style.bg(cx.theme().muted.opacity(0.3)))
            .when(!row.enabled, |this| this.opacity(0.5))
            .child(
                div()
                    .w(px(CHECK_COL))
                    .h_full()
                    .h_flex()
                    .justify_center()
                    .when(!is_last, |this| {
                        this.child(
                            Checkbox::new(("header-enabled", id))
                                .checked(row.enabled)
                                .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                                    if let Some(row) = this.rows.iter_mut().find(|r| r.id == id) {
                                        row.enabled = *checked;
                                        cx.notify();
                                    }
                                })),
                        )
                    }),
            )
            .child(divider())
            .child(cell(&row.key).when(!key_suggestions.is_empty(), |this| {
                this.child(self.render_suggestions(id, Field::Key, key_suggestions, cx))
            }))
            .child(divider())
            .child(
                cell(&row.value).when(!value_suggestions.is_empty(), |this| {
                    this.child(self.render_suggestions(id, Field::Value, value_suggestions, cx))
                }),
            )
            .child(divider())
            .child(cell(&row.description))
            .child(divider())
            .child(
                div()
                    .w(px(REMOVE_COL))
                    .h_full()
                    .h_flex()
                    .justify_center()
                    .when(!is_last, |this| {
                        this.child(
                            Button::new(("header-remove", id))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Close)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.remove(id, window, cx)
                                })),
                        )
                    }),
            )
    }
}

impl Render for KeyValueEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let last = self.rows.len().saturating_sub(1);
        let theme = cx.theme();
        let (border, muted) = (theme.border, theme.muted_foreground);

        let title = |text: &'static str| div().flex_1().px_3().h_full().h_flex().child(text);

        div()
            .id("key-value-editor")
            .size_full()
            .v_flex()
            .overflow_y_scroll()
            .child(
                div()
                    .v_flex()
                    .rounded_md()
                    .border_1()
                    .border_color(border)
                    .child(
                        div()
                            .h_flex()
                            .h(px(ROW_HEIGHT))
                            .rounded_t_md()
                            .bg(theme.tab_bar)
                            .text_sm()
                            .font_semibold()
                            .text_color(muted)
                            .child(div().w(px(CHECK_COL)))
                            .child(title("Name"))
                            .child(title("Value"))
                            .child(title("Description"))
                            .child(div().w(px(REMOVE_COL))),
                    )
                    .children(
                        self.rows
                            .iter()
                            .enumerate()
                            .map(|(index, row)| self.render_row(row, index == last, cx)),
                    ),
            )
    }
}
