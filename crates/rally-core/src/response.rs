use std::time::Duration;

pub struct ApiResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub elapsed: Duration,
}
