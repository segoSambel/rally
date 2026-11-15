use std::time::{Duration, Instant};

use crate::request::ApiRequest;
use crate::response::ApiResponse;

/// Blocking; call from a background thread.
pub fn send(request: ApiRequest) -> Result<ApiResponse, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();

    let mut builder = ureq::http::Request::builder()
        .method(request.method.as_str())
        .uri(request.url.trim());
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    let http_request = builder.body(request.body).map_err(|e| e.to_string())?;

    let started = Instant::now();
    let mut response = agent.run(http_request).map_err(|e| e.to_string())?;
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let elapsed = started.elapsed();

    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();

    Ok(ApiResponse {
        status: status.as_u16(),
        reason: status.canonical_reason().unwrap_or("").to_string(),
        headers,
        body,
        elapsed,
    })
}
