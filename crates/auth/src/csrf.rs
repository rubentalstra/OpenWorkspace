//! Synchronizer-token CSRF protection as a tower middleware.
//!
//! # FAIL-CLOSED: mutating Leptos `#[server]` functions
//!
//! This middleware is **fail-closed** for every unsafe method, including the
//! Leptos server-fn endpoints under `/api/*`. The Leptos client does **not**
//! attach an `X-CSRF-Token` header out of the box, so the first mutating
//! `#[server]` function (a urlencoded `POST`) will 403 until a client-side token
//! path is wired in. As of Leptos 0.8.20 there is no cleanly-documented public
//! hook to inject a per-request header (read from the rendered
//! `<meta name="csrf-token">`) into the generated server-fn client, so that work
//! is a **required follow-up for the first server-fn phase**. P5 ships no
//! mutating server fns, so nothing breaks today.
//!
//! Do **not** "fix" this by exempting `/api/*` from CSRF — that would reopen the
//! hole this layer closes. The first mutating server fn MUST attach the token.
//!
//! A per-session token is minted (CSPRNG → base64url) and stored in the session
//! under [`SESSION_TOKEN_KEY`]. The token is also placed in the request
//! extensions as [`CsrfToken`] so the SSR shell can render it into a `<meta>` tag
//! and a hidden form field. Safe methods (`GET`/`HEAD`/`OPTIONS`/`TRACE`) bypass
//! the check; unsafe methods must echo the token either in the `X-CSRF-Token`
//! header (JS) or — for `application/x-www-form-urlencoded` bodies with no header
//! — in a `csrf_token` form field (no-JS progressive enhancement). Comparison is
//! constant-time. Absence or mismatch is `403 Forbidden`.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use rand::TryRngCore as _;
use subtle::ConstantTimeEq as _;
use tower_sessions::Session;

/// Session key under which the per-session CSRF token is stored.
const SESSION_TOKEN_KEY: &str = "csrf.token";
/// Header carrying the CSRF token on JS-driven requests.
const CSRF_HEADER: &str = "x-csrf-token";
/// Form field carrying the CSRF token on no-JS submissions.
const CSRF_FIELD: &str = "csrf_token";
/// Number of random bytes behind a token (→ ~43 base64url chars).
const TOKEN_BYTES: usize = 32;
/// Maximum form body size buffered when extracting the field token (256 KiB).
const MAX_FORM_BODY: usize = 256 * 1024;

/// The per-session CSRF token, placed in request extensions for the view layer.
#[derive(Clone, Debug)]
pub struct CsrfToken(String);

impl CsrfToken {
    /// The token as a string, for rendering into HTML.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// CSRF middleware errors. Mapped to `403`/`500` responses; vendor session
/// errors never leak.
#[derive(Debug, thiserror::Error)]
pub enum CsrfError {
    /// The session layer was not installed before this middleware.
    #[error("session middleware missing")]
    MissingSession,
    /// Reading or writing the session token failed.
    #[error("session token access failed")]
    Session,
    /// The submitted token was absent or did not match.
    #[error("csrf token missing or invalid")]
    Invalid,
    /// Buffering the form body for field extraction failed or exceeded the cap.
    #[error("request body could not be read")]
    Body,
    /// Generating a new token failed.
    #[error("csrf token generation failed")]
    Generate,
}

impl IntoResponse for CsrfError {
    fn into_response(self) -> Response {
        // The specific variant is logged server-side; the client body is generic
        // so it never reveals whether the failure was CSRF, session, or internal.
        tracing::warn!(error = %self, "csrf middleware rejected request");
        let (status, body) = match self {
            Self::Invalid => (StatusCode::FORBIDDEN, "Forbidden"),
            Self::Body => (StatusCode::BAD_REQUEST, "Bad Request"),
            Self::MissingSession | Self::Session | Self::Generate => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        };
        (status, body).into_response()
    }
}

/// Renders the per-session token as a hidden `<input>` for inclusion in a
/// no-JS-capable form. The token is base64url (no special characters), so it is
/// safe to embed in the attribute unescaped.
#[must_use]
pub fn hidden_field(token: &CsrfToken) -> String {
    format!(
        r#"<input type="hidden" name="{CSRF_FIELD}" value="{}">"#,
        token.as_str()
    )
}

/// Whether a method mutates state and therefore requires a CSRF token.
fn is_unsafe(method: &Method) -> bool {
    !matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

/// Mints a fresh token, base64url-encoded.
fn mint_token() -> Result<String, CsrfError> {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| CsrfError::Generate)?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// Rotates the per-session CSRF token across an authentication boundary.
///
/// `axum-login`'s `login`/`logout` cycle the session **id** but keep the session
/// **data**, so a pre-auth (anonymous) CSRF token would otherwise survive into
/// the authenticated session — token fixation. Call this immediately after a
/// successful `auth_session.login(...)` and on `auth_session.logout()`: it
/// removes the stored token so the next request mints a fresh one bound to the
/// new session.
///
/// Takes the first-party [`AuthSession`](crate::AuthSession) so no
/// `tower_sessions` type leaks to the caller.
///
/// # Errors
///
/// Returns [`CsrfError::Session`] if removing the stored token fails.
pub async fn rotate_csrf_token(auth_session: &crate::AuthSession) -> Result<(), CsrfError> {
    remove_token(&auth_session.session).await
}

/// Removes the stored CSRF token from a raw session (the rotation primitive).
async fn remove_token(session: &Session) -> Result<(), CsrfError> {
    session
        .remove::<String>(SESSION_TOKEN_KEY)
        .await
        .map(|_| ())
        .map_err(|_| CsrfError::Session)
}

/// Loads the session token, minting and persisting one if absent.
async fn ensure_token(session: &Session) -> Result<String, CsrfError> {
    if let Some(existing) = session
        .get::<String>(SESSION_TOKEN_KEY)
        .await
        .map_err(|_| CsrfError::Session)?
    {
        return Ok(existing);
    }
    let token = mint_token()?;
    session
        .insert(SESSION_TOKEN_KEY, &token)
        .await
        .map_err(|_| CsrfError::Session)?;
    Ok(token)
}

/// Whether the request carries `application/x-www-form-urlencoded`.
fn is_form_urlencoded(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|ct| ct.split(';').next())
        .map(str::trim)
        .is_some_and(|mime| mime.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
}

/// Extracts the `csrf_token` field from a buffered urlencoded body.
fn field_token(body: &[u8]) -> Option<String> {
    form_urlencoded::parse(body)
        .find(|(k, _)| k == CSRF_FIELD)
        .map(|(_, v)| v.into_owned())
}

/// Constant-time equality of two tokens.
fn tokens_match(expected: &str, submitted: &str) -> bool {
    expected.as_bytes().ct_eq(submitted.as_bytes()).into()
}

/// The CSRF middleware. See the module docs for the protocol.
///
/// # Errors
///
/// Returns a [`CsrfError`] (mapped to a 4xx/5xx response) when the session is
/// missing, the token is absent/invalid, or the body cannot be read.
pub async fn csrf_layer(mut request: Request, next: Next) -> Result<Response, CsrfError> {
    let session = request
        .extensions()
        .get::<Session>()
        .cloned()
        .ok_or(CsrfError::MissingSession)?;

    let expected = ensure_token(&session).await?;

    if !is_unsafe(request.method()) {
        request.extensions_mut().insert(CsrfToken(expected.clone()));
        return Ok(next.run(request).await);
    }

    // Unsafe method: a header token wins; otherwise fall back to a form field.
    if let Some(header_token) = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
    {
        if tokens_match(&expected, &header_token) {
            request.extensions_mut().insert(CsrfToken(expected));
            return Ok(next.run(request).await);
        }
        return Err(CsrfError::Invalid);
    }

    if is_form_urlencoded(request.headers()) {
        let (parts, body) = request.into_parts();
        let bytes = axum::body::to_bytes(body, MAX_FORM_BODY)
            .await
            .map_err(|_| CsrfError::Body)?;
        let Some(submitted) = field_token(&bytes) else {
            return Err(CsrfError::Invalid);
        };
        if !tokens_match(&expected, &submitted) {
            return Err(CsrfError::Invalid);
        }
        // Re-inject the buffered body so downstream handlers still parse it.
        let mut request = Request::from_parts(parts, Body::from(bytes));
        request.extensions_mut().insert(CsrfToken(expected));
        return Ok(next.run(request).await);
    }

    Err(CsrfError::Invalid)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse as _;
    use http_body_util::BodyExt as _;

    use super::CsrfError;

    async fn body_text(err: CsrfError) -> String {
        let resp = err.into_response();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn invalid_body_is_generic_and_omits_csrf() {
        let body = body_text(CsrfError::Invalid).await;
        assert_eq!(body, "Forbidden");
        assert!(
            !body.to_lowercase().contains("csrf"),
            "403 body must not mention csrf"
        );
    }

    #[tokio::test]
    async fn session_body_is_generic_and_omits_session() {
        let body = body_text(CsrfError::Session).await;
        assert_eq!(body, "Internal Server Error");
        assert!(
            !body.to_lowercase().contains("session"),
            "500 body must not mention session"
        );
    }
}
