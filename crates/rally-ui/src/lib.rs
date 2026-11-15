mod client;
mod history;
mod theme;
mod widgets;

use gpui_kit::component::Root;
use gpui_kit::{App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size};

use crate::client::{ClientModel, ClientView};

/// Starts the Rally desktop app and blocks until it exits.
pub fn run() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx: &mut App| {
            gpui_kit::init(cx);
            theme::apply(cx);
            open_main_window(cx);
            cx.activate(true);
        });
}

fn open_main_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1160.), px(720.)), cx);
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
