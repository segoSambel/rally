use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable, Sizable, StyledExt};
use gpui_kit::{Context, Entity, EventEmitter, SharedString, Window, div, prelude::*, px};

use rally_core::{HistoryEntry, RequestDraft, format_ago, format_duration, now_unix};

use super::model::HistoryModel;
use crate::widgets::{method_color, status_color};

/// Emitted when the user clicks a history entry.
pub struct OpenEntry(pub RequestDraft);

/// Sidebar listing sent requests, newest first.
pub struct HistoryPanel {
    model: Entity<HistoryModel>,
    /// Entry last opened from the list; cleared when the history changes.
    selected: Option<u64>,
}

impl EventEmitter<OpenEntry> for HistoryPanel {}

impl HistoryPanel {
    pub fn new(model: Entity<HistoryModel>, cx: &mut Context<Self>) -> Self {
        cx.observe(&model, |this, _, cx| {
            this.selected = None;
            cx.notify();
        })
        .detach();
        Self {
            model,
            selected: None,
        }
    }

    fn render_entry(&self, entry: &HistoryEntry, now: u64, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let draft = entry.draft.clone();
        let id = entry.id;
        let is_selected = self.selected == Some(id);
        let url = entry.draft.url.trim();
        let url = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        let (status, status_color) = match entry.status {
            Some(status) => (status.to_string(), status_color(theme, status)),
            None => ("Failed".to_string(), theme.danger),
        };
        let mut details = vec![format_ago(now, entry.at)];
        if entry.status.is_some() {
            details.insert(
                0,
                format_duration(std::time::Duration::from_millis(entry.elapsed_ms)),
            );
        }

        div()
            .id(("history-entry", entry.id as usize))
            .v_flex()
            .gap_1()
            .px_3()
            .py_2()
            .border_l_2()
            .border_color(if is_selected {
                theme.primary
            } else {
                gpui_kit::transparent_black()
            })
            .when(is_selected, |this| this.bg(theme.muted.opacity(0.5)))
            .cursor_pointer()
            .hover(|style| style.bg(theme.muted.opacity(0.4)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(id);
                cx.emit(OpenEntry(draft.clone()));
                cx.notify();
            }))
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(
                        div()
                            .w(px(48.))
                            .flex_shrink_0()
                            .text_xs()
                            .font_semibold()
                            .text_color(method_color(entry.draft.method))
                            .child(entry.draft.method.as_str()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .child(SharedString::from(url.to_string())),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .pl(px(56.))
                    .text_xs()
                    .child(
                        div()
                            .text_color(status_color)
                            .child(SharedString::from(status)),
                    )
                    .child(
                        div()
                            .text_color(theme.muted_foreground)
                            .child(SharedString::from(details.join(" · "))),
                    ),
            )
    }
}

impl Render for HistoryPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted, chrome) = (theme.border, theme.muted_foreground, theme.tab_bar);
        let model = self.model.read(cx);
        let is_empty = model.entries().is_empty();
        let count = model.entries().len();
        let now = now_unix();

        div()
            .size_full()
            .v_flex()
            .bg(chrome)
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .child(div().text_sm().font_semibold().child("History"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(SharedString::from(count.to_string())),
                            ),
                    )
                    .child(
                        Button::new("history-clear")
                            .ghost()
                            .xsmall()
                            .label("Clear")
                            .disabled(is_empty)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.model.update(cx, |model, cx| model.clear(cx))
                            })),
                    ),
            )
            .child(if is_empty {
                div()
                    .flex_1()
                    .p_4()
                    .text_sm()
                    .text_color(muted)
                    .child("No history yet. Requests you send will show up here.")
                    .into_any_element()
            } else {
                div()
                    .id("history-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(
                        model
                            .entries()
                            .iter()
                            .map(|entry| self.render_entry(entry, now, cx)),
                    )
                    .into_any_element()
            })
    }
}
