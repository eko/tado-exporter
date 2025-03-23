use reqwest::{
    Error as HttpError,
    StatusCode,
    Url,
};

/// Authentication Errors.
#[derive(Debug)]
pub enum AuthError {
    /// Authentication failed because of an HTTP client error.
    Http(HttpError),

    /// The device authentication flow took too long to complete.
    Timeout,

    /// Unexpected status from the Auth API.
    UnexpectedStatus(StatusCode, Url),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Http(inner) => std::fmt::Display::fmt(inner, f),
            AuthError::Timeout => write!(f, "device auth flow took too long to complete"),
            AuthError::UnexpectedStatus(status, url) => write!(
                f, "unexpected auth API status {} for URL {}",
                status, url,
            ),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthError::Http(ref inner) => Some(inner),
            AuthError::Timeout => None,
            AuthError::UnexpectedStatus(_, _) => None,
        }
    }
}

impl From<HttpError> for AuthError {
    fn from(value: HttpError) -> Self {
        AuthError::Http(value)
    }
}
