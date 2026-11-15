use std::time::Duration;

/// Pretty-prints JSON and reports whether the body was JSON; other bodies are unchanged.
pub fn format_body(body: &str) -> (String, bool) {
    match serde_json::from_str::<serde_json::Value>(body)
        .and_then(|value| serde_json::to_string_pretty(&value))
    {
        Ok(pretty) => (pretty, true),
        Err(_) => (body.to_string(), false),
    }
}

pub fn format_duration(elapsed: Duration) -> String {
    if elapsed.as_millis() >= 1000 {
        format!("{:.1}s", elapsed.as_secs_f64())
    } else {
        format!("{}ms", elapsed.as_millis())
    }
}

pub fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// "just now", "5m ago", "3h ago", "2d ago".
pub fn format_ago(now: u64, at: u64) -> String {
    let secs = now.saturating_sub(at);
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86400),
    }
}
