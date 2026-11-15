/// Appends `params` to `url`'s query string, keeping any existing query and `#fragment`.
pub(crate) fn with_query(url: &str, params: &[(String, String)]) -> String {
    if params.is_empty() {
        return url.to_string();
    }
    let (base, fragment) = match url.split_once('#') {
        Some((base, fragment)) => (base, Some(fragment)),
        None => (url, None),
    };
    let query = form_encode(params);
    let separator = match base.find('?') {
        None => "?",
        Some(_) if base.ends_with('?') || base.ends_with('&') => "",
        Some(_) => "&",
    };
    let fragment = fragment.map(|f| format!("#{f}")).unwrap_or_default();
    format!("{base}{separator}{query}{fragment}")
}

/// `name=value` pairs joined with `&`, percent-encoded, for form bodies and query strings.
pub(crate) fn form_encode(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
