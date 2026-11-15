#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod client;

use gpui_kit::component::{Root, Theme, ThemeMode, ThemeRegistry};
use gpui_kit::{App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::client::{ClientModel, ClientView};

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);
            apply_theme(cx);
            open_main_window(cx);
            cx.activate(true);
        });
}

/// Registers the Rally light and dark themes and starts in dark mode.
fn apply_theme(cx: &mut App) {
    ThemeRegistry::global_mut(cx)
        .load_themes_from_str(include_str!("../assets/theme/rally.json"))
        .expect("assets/theme/rally.json is not a valid theme");

    let registry = ThemeRegistry::global(cx);
    let light = registry.themes().get("Rally Light").cloned();
    let dark = registry.themes().get("Rally Dark").cloned();
    let (light, dark) = (
        light.expect("Rally Light theme missing"),
        dark.expect("Rally Dark theme missing"),
    );

    let theme = Theme::global_mut(cx);
    theme.light_theme = light;
    theme.dark_theme = dark;
    Theme::change(ThemeMode::Dark, None, cx);
}

fn open_main_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(960.), px(720.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Rally".into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    cx.open_window(options, |window, cx| {
        let model = cx.new(|_cx| ClientModel::new());
        let view = cx.new(|cx| ClientView::new(model, window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    })
    .expect("failed to open main window");
}
