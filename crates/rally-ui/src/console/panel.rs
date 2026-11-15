use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use gpui_kit::{
    ClipboardItem, Context, Entity, EventEmitter, Hsla, SharedString, Window, div, prelude::*, px,
    relative,
};

use rally_core::{
    ConsoleEntry, ConsoleOutcome, MAX_BODY_PREVIEW, format_ago, format_body, format_duration,
    format_size, now_unix,
};

use super::model::ConsoleModel;
use crate::widgets::{method_color, status_color};

/// Emitted when the user closes the panel.
pub struct CloseRequested;

/// Bottom panel listing requests sent this session, with the raw exchange of the selected one.
pub struct ConsolePanel {
    model: Entity<ConsoleModel>,
    /// Entry picked from the list; the newest one is shown until then.
    selected: Option<u64>,
}

impl EventEmitter<CloseRequested> for ConsolePanel {}

impl ConsolePanel {
    pub fn new(model: Entity<ConsoleModel>, cx: &mut Context<Self>) -> Self {
        cx.observe(&model, |_, _, cx| cx.notify()).detach();
        Self {
            model,
            selected: None,
        }
    }

    fn render_row(
        &self,
        entry: &ConsoleEntry,
        selected: bool,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let id = entry.id;
        let (status, color, took) = match &entry.outcome {
            ConsoleOutcome::Response(response) => (
                response.status.to_string(),
                status_color(theme, response.status),
                format_duration(response.elapsed),
            ),
            ConsoleOutcome::Failed(_) => ("Failed".to_string(), theme.danger, String::new()),
        };

        div()
            .id(("console-entry", id as usize))
            .h_flex()
            .gap_3()
            .px_3()
            .py_1()
            .text_xs()
            .border_l_2()
            .border_color(if selected {
                theme.primary
            } else {
                gpui_kit::transparent_black()
            })
            .when(selected, |this| this.bg(theme.muted.opacity(0.5)))
            .cursor_pointer()
            .hover(|style| style.bg(theme.muted.opacity(0.4)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(id);
                cx.notify();
            }))
            .child(
                div()
                    .w(px(48.))
                    .flex_shrink_0()
                    .font_semibold()
                    .text_color(method_color(entry.request.method))
                    .child(entry.request.method.as_str()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(SharedString::from(entry.request.url.clone())),
            )
            .child(
                div()
                    .w(px(44.))
                    .flex_shrink_0()
                    .text_color(color)
                    .child(SharedString::from(status)),
            )
            .child(
                div()
                    .w(px(56.))
                    .flex_shrink_0()
                    .text_color(theme.muted_foreground)
                    .child(SharedString::from(took)),
            )
    }

    fn render_detail(&self, entry: &ConsoleEntry, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (muted, border) = (theme.muted_foreground, theme.border);
        let mono = theme.mono_font_family.clone();
        let curl = entry.request.to_curl();

        let title = |text: &'static str| {
            div()
                .pb_1()
                .border_b_1()
                .border_color(border)
                .font_semibold()
                .text_color(muted)
                .child(text)
        };
        let lines = |headers: &[(String, String)]| {
            div()
                .v_flex()
                .gap_0p5()
                .font_family(mono.clone())
                .children(headers.iter().map(|(name, value)| {
                    div()
                        .h_flex()
                        .gap_2()
                        .items_start()
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_color(muted)
                                .child(SharedString::from(format!("{name}:"))),
                        )
                        .child(div().min_w_0().child(SharedString::from(value.clone())))
                }))
        };
        let block = |text: String| {
            div()
                .p_2()
                .rounded_md()
                .bg(theme.muted.opacity(0.3))
                .font_family(mono.clone())
                .child(SharedString::from(text))
        };

        let mut request = div().v_flex().gap_2().child(title("Request"));
        request = request.child(lines(&entry.request.headers));
        if !entry.request.body.is_empty() {
            request = request.child(block(entry.request.body.clone()));
        }
        request = request.child(div().text_color(muted).child(
            "The HTTP client also adds Host, User-Agent, Accept, Accept-Encoding and Content-Length.",
        ));

        let response = match &entry.outcome {
            ConsoleOutcome::Failed(message) => {
                div().v_flex().gap_2().child(title("Response")).child(
                    div()
                        .text_color(theme.danger)
                        .child(SharedString::from(message.clone())),
                )
            }
            ConsoleOutcome::Response(response) => {
                let truncated = response.body_len > response.body_preview.len();
                // Truncated JSON would not parse, so only pretty-print whole bodies.
                let body = if truncated {
                    response.body_preview.clone()
                } else {
                    format_body(&response.body_preview).0
                };
                let status: Hsla = status_color(theme, response.status);

                let mut section = div()
                    .v_flex()
                    .gap_2()
                    .child(title("Response"))
                    .child(
                        div()
                            .h_flex()
                            .gap_2()
                            .child(div().font_semibold().text_color(status).child(
                                SharedString::from(format!(
                                    "{} {}",
                                    response.status, response.reason
                                )),
                            ))
                            .child(div().text_color(muted).child(SharedString::from(format!(
                                "{} · {}",
                                format_duration(response.elapsed),
                                format_size(response.body_len)
                            )))),
                    )
                    .child(lines(&response.headers));
                if !body.is_empty() {
                    section = section.child(block(body));
                }
                if truncated {
                    section =
                        section.child(div().text_color(muted).child(SharedString::from(format!(
                            "Showing the first {} of {}.",
                            format_size(MAX_BODY_PREVIEW),
                            format_size(response.body_len)
                        ))));
                }
                section
            }
        };

        div()
            .id("console-detail")
            .size_full()
            .v_flex()
            .gap_4()
            .p_3()
            .text_xs()
            .overflow_y_scroll()
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_flex()
                            .gap_2()
                            .items_start()
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .font_semibold()
                                    .text_color(method_color(entry.request.method))
                                    .child(entry.request.method.as_str()),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .font_family(mono.clone())
                                    .child(SharedString::from(entry.request.url.clone())),
                            ),
                    )
                    .child(
                        Button::new("console-copy-curl")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Copy)
                            .label("Copy cURL")
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(curl.clone()))
                            })),
                    ),
            )
            .child(
                div()
                    .text_color(muted)
                    .child(SharedString::from(format_ago(now_unix(), entry.at))),
            )
            .child(request)
            .child(response)
    }
}

impl Render for ConsolePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted, chrome) = (theme.border, theme.muted_foreground, theme.tab_bar);
        let model = self.model.read(cx);
        let entries = model.entries();
        let shown = self
            .selected
            .and_then(|id| entries.iter().find(|entry| entry.id == id))
            .or(entries.first());
        let shown_id = shown.map(|entry| entry.id);

        let body =
            match shown {
                None => div()
                    .flex_1()
                    .v_flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_color(muted)
                    .child(div().font_semibold().child("Console is empty"))
                    .child(div().text_sm().child(
                        "Requests you send appear here with their raw request and response.",
                    ))
                    .into_any_element(),
                Some(shown) => div()
                    .h_flex()
                    .items_stretch()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .id("console-list")
                            .w(relative(0.46))
                            .flex_shrink_0()
                            .border_r_1()
                            .border_color(border)
                            .overflow_y_scroll()
                            .children(entries.iter().map(|entry| {
                                self.render_row(entry, Some(entry.id) == shown_id, cx)
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(self.render_detail(shown, cx)),
                    )
                    .into_any_element(),
            };

        div()
            .size_full()
            .v_flex()
            .bg(theme.background)
            .child(
                div()
                    .h_flex()
                    .h(px(32.))
                    .px_3()
                    .gap_2()
                    .bg(chrome)
                    .border_b_1()
                    .border_color(border)
                    .child(div().text_sm().font_semibold().child("Console"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(SharedString::from(entries.len().to_string())),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_1()
                            .ml_auto()
                            .child(
                                Button::new("console-clear")
                                    .ghost()
                                    .xsmall()
                                    .label("Clear")
                                    .disabled(entries.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.selected = None;
                                        this.model.update(cx, |model, cx| model.clear(cx))
                                    })),
                            )
                            .child(
                                Button::new("console-close")
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Close)
                                    .tooltip("Close console")
                                    .on_click(cx.listener(|_, _, _, cx| cx.emit(CloseRequested))),
                            ),
                    ),
            )
            .child(body)
    }
}
