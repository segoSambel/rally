use gpui_kit::App;
use gpui_kit::component::{Theme, ThemeMode, ThemeRegistry};

/// Registers the Rally light and dark themes and starts in dark mode.
pub fn apply(cx: &mut App) {
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
