use serde::{Deserialize, Serialize};

use crate::auth::{ApiKeyLocation, Auth};
use crate::request::{ApiRequest, Method};
use crate::url::form_encode;

/// One row of a key/value table (headers or query params).
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyValueRow {
    pub enabled: bool,
    pub key: String,
    pub value: String,
    pub description: String,
}

impl Default for KeyValueRow {
    fn default() -> Self {
        Self {
            enabled: true,
            key: String::new(),
            value: String::new(),
            description: String::new(),
        }
    }
}

/// Which authorization scheme is selected in the editor.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthKind {
    #[default]
    None,
    Bearer,
    Basic,
    ApiKey,
}

/// Everything entered in the authorization tab, including fields of unselected schemes.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthDraft {
    pub kind: AuthKind,
    pub token: String,
    pub username: String,
    pub password: String,
    pub key_name: String,
    pub key_value: String,
    pub key_in_query: bool,
}

impl AuthDraft {
    /// The scheme that will be applied to the request.
    pub fn to_auth(&self) -> Auth {
        match self.kind {
            AuthKind::None => Auth::None,
            AuthKind::Bearer => Auth::Bearer {
                token: self.token.trim().to_string(),
            },
            AuthKind::Basic => Auth::Basic {
                username: self.username.trim().to_string(),
                password: self.password.clone(),
            },
            AuthKind::ApiKey => Auth::ApiKey {
                name: self.key_name.trim().to_string(),
                value: self.key_value.trim().to_string(),
                location: if self.key_in_query {
                    ApiKeyLocation::Query
                } else {
                    ApiKeyLocation::Header
                },
            },
        }
    }
}

/// How the request body is entered and encoded.
///
/// `Text` is the default so history saved before body types existed keeps its body.
#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    None,
    Json,
    #[default]
    Text,
    Form,
}

/// The full editor state of a request: what the user typed, not what is sent.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestDraft {
    pub method: Method,
    pub url: String,
    pub params: Vec<KeyValueRow>,
    pub headers: Vec<KeyValueRow>,
    pub auth: AuthDraft,
    pub body_kind: BodyKind,
    /// Raw text for the `Json` and `Text` kinds.
    pub body: String,
    /// Rows for the `Form` kind.
    pub form: Vec<KeyValueRow>,
}

impl RequestDraft {
    /// The request to send: enabled rows with a key, trimmed, plus the auth scheme.
    ///
    /// The body kind sets a default `Content-Type` when there is a body; a header the user
    /// set explicitly wins.
    pub fn build(&self) -> ApiRequest {
        let (body, content_type) = match self.body_kind {
            BodyKind::None => (String::new(), None),
            BodyKind::Json => (self.body.clone(), Some("application/json")),
            BodyKind::Text => (self.body.clone(), Some("text/plain")),
            BodyKind::Form => (
                form_encode(&enabled_pairs(&self.form)),
                Some("application/x-www-form-urlencoded"),
            ),
        };
        let content_type = content_type.filter(|_| !body.is_empty());

        ApiRequest::build(
            self.method,
            self.url.trim(),
            enabled_pairs(&self.params),
            enabled_pairs(&self.headers),
            &self.auth.to_auth(),
            body,
            content_type,
        )
    }
}

fn enabled_pairs(rows: &[KeyValueRow]) -> Vec<(String, String)> {
    rows.iter()
        .filter(|row| row.enabled)
        .map(|row| (row.key.trim().to_string(), row.value.trim().to_string()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}
