use gpui_kit::component::StyledExt;
use gpui_kit::component::searchable_list::SearchableListItem;
use gpui_kit::{AnyElement, App, Hsla, SharedString, Window, div, prelude::*, rgb};
use rally_core::Method;

pub type MethodSelect = Vec<MethodItem>;

/// Postman-style color for each HTTP method.
pub fn method_color(method: Method) -> Hsla {
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
pub struct MethodItem(pub Method);

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
