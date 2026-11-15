//! Window chrome around the request editor: the tab strip on top and the status bar below.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{
    ActiveTheme, IconName, Selectable, Sizable, StyledExt, Theme, ThemeMode,
};
use gpui_kit::{Context, SharedString, div, prelude::*, px};

use super::view::ClientView;
use crate::widgets::method_color;

/// `https://host/path` becomes `host/path`; an empty URL is a new request.
fn tab_title(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return "New Request".to_string();
    }
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .to_string()
}

impl ClientView {
    /// One tab per request, each with a close button, and a button to open another.
    pub(super) fn render_tab_strip(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let many = self.tabs.len() > 1;

        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| {
            let active = index == self.active_tab;
            // The live editors hold the active tab's request.
            let (method, url) = if active {
                (self.method, self.url.read(cx).value().to_string())
            } else {
                (tab.draft.method, tab.draft.url.clone())
            };

            div()
                .h_flex()
                .flex_shrink_0()
                .h(px(34.))
                .rounded_t_md()
                .border_1()
                .border_b_0()
                .border_color(if active {
                    theme.border
                } else {
                    gpui_kit::transparent_black()
                })
                .when(active, |this| this.bg(theme.background))
                .when(!active, |this| {
                    this.text_color(theme.muted_foreground)
                        .hover(|style| style.bg(theme.muted.opacity(0.3)))
                })
                .child(
                    div()
                        .id(("request-tab", index))
                        .h_flex()
                        .gap_2()
                        .h_full()
                        .pl_3()
                        .when(!many, |this| this.pr_3())
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_tab(index, window, cx)
                        }))
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(method_color(method))
                                .child(method.as_str()),
                        )
                        .child(
                            div()
                                .max_w(px(180.))
                                .truncate()
                                .text_sm()
                                .child(SharedString::from(tab_title(&url))),
                        ),
                )
                .when(many, |this| {
                    this.child(
                        div().px_1().child(
                            Button::new(("close-tab", index))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Close)
                                .tooltip("Close tab")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.close_tab(index, window, cx)
                                })),
                        ),
                    )
                })
        });

        div()
            .h_flex()
            .items_end()
            .h(px(40.))
            .px_2()
            .gap_1()
            .bg(theme.tab_bar)
            .child(
                div()
                    .id("request-tabs-scroll")
                    .h_flex()
                    .items_end()
                    .gap_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .children(tabs),
            )
            .child(
                div().h(px(34.)).h_flex().child(
                    Button::new("new-tab")
                        .ghost()
                        .small()
                        .icon(IconName::Plus)
                        .tooltip("New tab")
                        .on_click(cx.listener(|this, _, window, cx| this.new_tab(window, cx))),
                ),
            )
    }

    /// Utility toggles on the left, request status and version on the right.
    pub(super) fn render_status_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let sending = self.model.read(cx).is_sending();
        let saved = self.history.read(cx).entries().len();

        div()
            .h_flex()
            .h(px(28.))
            .px_2()
            .gap_1()
            .bg(theme.tab_bar)
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(muted)
            .child(
                Button::new("sidebar-toggle")
                    .ghost()
                    .xsmall()
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
                Button::new("theme-toggle")
                    .ghost()
                    .xsmall()
                    .icon(if theme.is_dark() {
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
            .child(
                Button::new("console-toggle")
                    .ghost()
                    .xsmall()
                    .icon(IconName::SquareTerminal)
                    .label("Console")
                    .selected(self.console_open)
                    .tooltip("Toggle console")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.console_open = !this.console_open;
                        cx.notify();
                    })),
            )
            .when(sending, |this| {
                this.child(
                    div()
                        .h_flex()
                        .gap_1()
                        .ml_2()
                        .child(Spinner::new().xsmall().color(muted))
                        .child("Sending..."),
                )
            })
            .child(
                div()
                    .h_flex()
                    .gap_3()
                    .ml_auto()
                    .px_2()
                    .child(SharedString::from(format!("{saved} in history")))
                    .child(SharedString::from(format!(
                        "v{}",
                        env!("CARGO_PKG_VERSION")
                    ))),
            )
    }
}
