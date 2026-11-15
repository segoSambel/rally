use gpui_kit::Hsla;
use gpui_kit::component::Theme;

/// Green for 2xx, red for 4xx and 5xx, amber for everything else.
pub fn status_color(theme: &Theme, status: u16) -> Hsla {
    match status {
        200..=299 => theme.success,
        400..=599 => theme.danger,
        _ => theme.warning,
    }
}
