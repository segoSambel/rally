//! Window chrome around the request editor: the tab strip on top and the status bar below.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, IconName, Sizable, StyledExt, Theme, ThemeMode};
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
    /// A single request tab (method and title) and a button for a new request.
    pub(super) fn render_tab_strip(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let title = tab_title(&self.url.read(cx).value());

        div()
            .h_flex()
            .items_end()
            .h(px(40.))
            .px_2()
            .gap_1()
            .bg(theme.tab_bar)
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .h(px(34.))
                    .px_3()
                    .rounded_t_md()
                    .border_1()
                    .border_b_0()
                    .border_color(theme.border)
                    .bg(theme.background)
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(method_color(self.method))
                            .child(self.method.as_str()),
                    )
                    .child(
                        div()
                            .max_w(px(320.))
                            .truncate()
                            .text_sm()
                            .child(SharedString::from(title)),
                    ),
            )
            .child(
                div().h(px(34.)).h_flex().child(
                    Button::new("new-request")
                        .ghost()
                        .small()
                        .icon(IconName::Plus)
                        .tooltip("New request")
                        .on_click(cx.listener(|this, _, window, cx| this.new_request(window, cx))),
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
