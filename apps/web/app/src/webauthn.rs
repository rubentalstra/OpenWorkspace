//! The browser-side WebAuthn ceremony — the first-party facade over
//! `navigator.credentials`.
//!
//! The server sends a `CreationChallengeResponse` / `RequestChallengeResponse` as
//! JSON (its `publicKey` member holds the options). We hand that to the standard
//! `PublicKeyCredential.parseCreationOptionsFromJSON()` /
//! `parseRequestOptionsFromJSON()` helpers (which decode the base64url buffers),
//! call `navigator.credentials.create()` / `.get()`, then serialize the result
//! with `PublicKeyCredential.toJSON()` and return that JSON string for the matching
//! `#[server]` fn to verify. No JavaScript file — pure `web-sys` + `js-sys`, gated
//! to the browser build; the SSR build links inert stubs so the call sites compile.
//!
//! Refs: MDN `PublicKeyCredential.parseCreationOptionsFromJSON` / `toJSON`;
//! W3C WebAuthn-3 JSON serialization.

/// Run the registration ceremony for the given `CreationChallengeResponse` JSON,
/// returning the credential as `RegistrationResponseJSON`.
///
/// # Errors
///
/// A human-readable message if WebAuthn is unavailable or the user cancels.
#[cfg(feature = "hydrate")]
pub async fn create(options_json: String) -> Result<String, String> {
    imp::ceremony(&options_json, "parseCreationOptionsFromJSON", "create").await
}

/// Run the authentication ceremony for the given `RequestChallengeResponse` JSON,
/// returning the assertion as `AuthenticationResponseJSON`.
///
/// # Errors
///
/// A human-readable message if WebAuthn is unavailable or the user cancels.
#[cfg(feature = "hydrate")]
pub async fn get(options_json: String) -> Result<String, String> {
    imp::ceremony(&options_json, "parseRequestOptionsFromJSON", "get").await
}

/// SSR stub: the ceremony only runs in the browser. The call sites (built for both
/// targets) compile against this; it is never invoked during a server render.
#[cfg(not(feature = "hydrate"))]
pub async fn create(_options_json: String) -> Result<String, String> {
    Err("passkeys are only available in the browser".to_owned())
}

/// SSR stub for [`get`]; see [`create`].
#[cfg(not(feature = "hydrate"))]
pub async fn get(_options_json: String) -> Result<String, String> {
    Err("passkeys are only available in the browser".to_owned())
}

#[cfg(feature = "hydrate")]
mod imp {
    use js_sys::{Function, JSON, Object, Promise, Reflect};
    use wasm_bindgen::{JsCast as _, JsValue};
    use wasm_bindgen_futures::JsFuture;

    fn key(name: &str) -> JsValue {
        JsValue::from_str(name)
    }

    /// `window.PublicKeyCredential` — the constructor object that also carries the
    /// static `parse*FromJSON` helpers.
    fn pkc_ctor() -> Result<JsValue, String> {
        let win = web_sys::window().ok_or("no browser window")?;
        Reflect::get(&win, &key("PublicKeyCredential"))
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
            .ok_or_else(|| "this browser does not support passkeys".to_owned())
    }

    /// `navigator.credentials`.
    fn navigator_credentials() -> Result<JsValue, String> {
        let win = web_sys::window().ok_or("no browser window")?;
        let navigator =
            Reflect::get(&win, &key("navigator")).map_err(|_| "no navigator".to_owned())?;
        Reflect::get(&navigator, &key("credentials"))
            .map_err(|_| "the credentials API is unavailable".to_owned())
    }

    /// Call a function-valued property `name` on `target` with one argument.
    fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Result<JsValue, String> {
        let f: Function = Reflect::get(target, &key(name))
            .map_err(|_| format!("{name} is missing"))?
            .dyn_into()
            .map_err(|_| format!("{name} is not callable"))?;
        f.call1(target, arg).map_err(|_| format!("{name} failed"))
    }

    /// Shared registration/authentication ceremony: parse the server options,
    /// invoke `navigator.credentials.<method>`, and serialize the result.
    pub(super) async fn ceremony(
        options_json: &str,
        parse_method: &str,
        credentials_method: &str,
    ) -> Result<String, String> {
        let ctor = pkc_ctor()?;
        let server = JSON::parse(options_json).map_err(|_| "malformed options".to_owned())?;
        let inner = Reflect::get(&server, &key("publicKey"))
            .map_err(|_| "options missing publicKey".to_owned())?;
        let parsed = call1(&ctor, parse_method, &inner)?;

        let options = Object::new();
        Reflect::set(&options, &key("publicKey"), &parsed)
            .map_err(|_| "could not build ceremony options".to_owned())?;

        let creds = navigator_credentials()?;
        let promise: Promise = call1(&creds, credentials_method, &options.into())?
            .dyn_into()
            .map_err(|_| "the ceremony did not return a promise".to_owned())?;
        let credential = JsFuture::from(promise)
            .await
            .map_err(|_| "the passkey request was cancelled or failed".to_owned())?;

        let json = {
            let to_json: Function = Reflect::get(&credential, &key("toJSON"))
                .map_err(|_| "toJSON is missing".to_owned())?
                .dyn_into()
                .map_err(|_| "toJSON is not callable".to_owned())?;
            to_json
                .call0(&credential)
                .map_err(|_| "toJSON failed".to_owned())?
        };
        JSON::stringify(&json)
            .ok()
            .and_then(|s| s.as_string())
            .ok_or_else(|| "could not serialize the credential".to_owned())
    }
}
