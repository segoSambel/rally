/// Where an API key is sent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyLocation {
    Header,
    Query,
}

/// Authorization scheme for a request. Empty fields add nothing.
pub enum Auth {
    None,
    Bearer {
        token: String,
    },
    Basic {
        username: String,
        password: String,
    },
    ApiKey {
        name: String,
        value: String,
        location: ApiKeyLocation,
    },
}

/// What an [`Auth`] scheme adds to a request.
#[derive(Default)]
pub(crate) struct AuthParts {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}

impl Auth {
    pub(crate) fn parts(&self) -> AuthParts {
        let mut parts = AuthParts::default();

        match self {
            Auth::None => {}
            Auth::Bearer { token } => {
                if !token.is_empty() {
                    parts
                        .headers
                        .push(("Authorization".into(), format!("Bearer {token}")));
                }
            }
            Auth::Basic { username, password } => {
                if !username.is_empty() || !password.is_empty() {
                    let encoded = base64_encode(format!("{username}:{password}").as_bytes());
                    parts
                        .headers
                        .push(("Authorization".into(), format!("Basic {encoded}")));
                }
            }
            Auth::ApiKey {
                name,
                value,
                location,
            } => {
                if !name.is_empty() {
                    let target = match location {
                        ApiKeyLocation::Header => &mut parts.headers,
                        ApiKeyLocation::Query => &mut parts.query,
                    };
                    target.push((name.clone(), value.clone()));
                }
            }
        }
        parts
    }
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}
