//! The browser-side WebAuthn ceremony — the first-party facade over
//! `navigator.credentials`.
//!
//! The server sends a `CreationChallengeResponse` / `RequestChallengeResponse` as
//! JSON (its `publicKey` member holds the options, with the binary fields as
//! base64url strings). We decode those base64url fields to `ArrayBuffer`s, call
//! `navigator.credentials.create()` / `.get()`, then re-encode the binary parts of
//! the result back to base64url and return the JSON `webauthn-rs` expects. Pure
//! `web-sys` + `js-sys` (no JavaScript file, and no dependency on the WebAuthn-L3
//! `parse*FromJSON`/`toJSON` helpers, which aren't in every engine). Gated to the
//! browser build; the SSR build links inert stubs so the call sites compile.

/// Run the registration ceremony for the given `CreationChallengeResponse` JSON,
/// returning the credential as the JSON `auth::RegisterPublicKeyCredential` expects.
///
/// # Errors
///
/// A human-readable message if WebAuthn is unavailable or the user cancels.
#[cfg(feature = "hydrate")]
pub async fn create(options_json: String) -> Result<String, String> {
    imp::create(&options_json).await
}

/// Run the authentication ceremony for the given `RequestChallengeResponse` JSON,
/// returning the assertion as the JSON `auth::PublicKeyCredential` expects.
///
/// # Errors
///
/// A human-readable message if WebAuthn is unavailable or the user cancels.
#[cfg(feature = "hydrate")]
pub async fn get(options_json: String) -> Result<String, String> {
    imp::get_assertion(&options_json).await
}

/// SSR stub: the ceremony only runs in the browser. The call sites (built for both
/// targets) compile against this; it is never invoked during a server render.
#[cfg(not(feature = "hydrate"))]
#[expect(
    clippy::unused_async,
    reason = "mirrors the async browser ceremony; never invoked server-side"
)]
pub async fn create(_options_json: String) -> Result<String, String> {
    Err("passkeys are only available in the browser".to_owned())
}

/// SSR stub for [`get`]; see [`create`].
#[cfg(not(feature = "hydrate"))]
#[expect(
    clippy::unused_async,
    reason = "mirrors the async browser ceremony; never invoked server-side"
)]
pub async fn get(_options_json: String) -> Result<String, String> {
    Err("passkeys are only available in the browser".to_owned())
}

#[cfg(feature = "hydrate")]
mod imp {
    use base64::Engine as _;
    use js_sys::{Array, JSON, Object, Promise, Reflect, Uint8Array};
    use wasm_bindgen::{JsCast as _, JsValue};
    use wasm_bindgen_futures::JsFuture;

    const B64: base64::engine::general_purpose::GeneralPurpose =
        base64::engine::general_purpose::URL_SAFE_NO_PAD;

    fn key(name: &str) -> JsValue {
        JsValue::from_str(name)
    }

    fn get(obj: &JsValue, name: &str) -> Result<JsValue, String> {
        Reflect::get(obj, &key(name)).map_err(|_| format!("missing field {name}"))
    }

    fn set(obj: &JsValue, name: &str, value: &JsValue) -> Result<(), String> {
        Reflect::set(obj, &key(name), value)
            .map(|_| ())
            .map_err(|_| format!("could not set {name}"))
    }

    /// Set `out[name]` to the base64url encoding of `obj[name]` (an `ArrayBuffer`).
    fn copy_b64(out: &Object, obj: &JsValue, name: &str) -> Result<(), String> {
        let buffer = get(obj, name)?;
        let bytes = Uint8Array::new(&buffer).to_vec();
        set(out, name, &key(&B64.encode(bytes)))
    }

    /// Replace `obj[name]` (a base64url string) in place with its decoded bytes.
    fn decode_field(obj: &JsValue, name: &str) -> Result<(), String> {
        let encoded = get(obj, name)?
            .as_string()
            .ok_or_else(|| format!("{name} is not a string"))?;
        let bytes = B64
            .decode(encoded)
            .map_err(|_| format!("{name} is not valid base64url"))?;
        set(obj, name, &Uint8Array::from(bytes.as_slice()))
    }

    /// Decode the `id` of each entry in an optional credential-descriptor array.
    fn decode_credential_ids(options: &JsValue, field: &str) -> Result<(), String> {
        let list = get(options, field)?;
        if list.is_undefined() || list.is_null() {
            return Ok(());
        }
        let array: Array = list
            .dyn_into()
            .map_err(|_| format!("{field} is not an array"))?;
        for i in 0..array.length() {
            decode_field(&array.get(i), "id")?;
        }
        Ok(())
    }

    fn navigator_credentials() -> Result<JsValue, String> {
        let window = web_sys::window().ok_or("no browser window")?;
        let navigator = get(&window, "navigator")?;
        get(&navigator, "credentials")
    }

    /// Call `navigator.credentials.<method>({ publicKey })` and await the result.
    async fn invoke(method: &str, public_key: &JsValue) -> Result<JsValue, String> {
        let options = Object::new();
        set(&options, "publicKey", public_key)?;
        let credentials = navigator_credentials()?;
        let func: js_sys::Function = get(&credentials, method)?
            .dyn_into()
            .map_err(|_| format!("navigator.credentials.{method} is not callable"))?;
        let promise: Promise = func
            .call1(&credentials, &options)
            .map_err(|_| format!("navigator.credentials.{method} failed"))?
            .dyn_into()
            .map_err(|_| "the ceremony did not return a promise".to_owned())?;
        JsFuture::from(promise)
            .await
            .map_err(|_| "the passkey request was cancelled or failed".to_owned())
    }

    /// `cred.getClientExtensionResults()` (or `{}` if unavailable).
    fn extensions(cred: &JsValue) -> JsValue {
        Reflect::get(cred, &key("getClientExtensionResults"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
            .and_then(|f| f.call0(cred).ok())
            .unwrap_or_else(|| Object::new().into())
    }

    /// The shared head of every response object: `id`, `rawId`, `type`.
    fn response_head(cred: &JsValue) -> Result<Object, String> {
        let out = Object::new();
        set(&out, "id", &get(cred, "id")?)?;
        copy_b64(&out, cred, "rawId")?;
        set(&out, "type", &get(cred, "type")?)?;
        set(&out, "clientExtensionResults", &extensions(cred))?;
        Ok(out)
    }

    fn stringify(value: &JsValue) -> Result<String, String> {
        JSON::stringify(value)
            .ok()
            .and_then(|s| s.as_string())
            .ok_or_else(|| "could not serialize the credential".to_owned())
    }

    pub(super) async fn create(options_json: &str) -> Result<String, String> {
        let server = JSON::parse(options_json).map_err(|_| "malformed options".to_owned())?;
        let public_key = get(&server, "publicKey")?;
        decode_field(&public_key, "challenge")?;
        decode_field(&get(&public_key, "user")?, "id")?;
        decode_credential_ids(&public_key, "excludeCredentials")?;

        let cred = invoke("create", &public_key).await?;
        let response = get(&cred, "response")?;
        let out = response_head(&cred)?;
        let resp = Object::new();
        copy_b64(&resp, &response, "clientDataJSON")?;
        copy_b64(&resp, &response, "attestationObject")?;
        set(&out, "response", &resp)?;
        stringify(&out)
    }

    pub(super) async fn get_assertion(options_json: &str) -> Result<String, String> {
        let server = JSON::parse(options_json).map_err(|_| "malformed options".to_owned())?;
        let public_key = get(&server, "publicKey")?;
        decode_field(&public_key, "challenge")?;
        decode_credential_ids(&public_key, "allowCredentials")?;

        let cred = invoke("get", &public_key).await?;
        let response = get(&cred, "response")?;
        let out = response_head(&cred)?;
        let resp = Object::new();
        copy_b64(&resp, &response, "clientDataJSON")?;
        copy_b64(&resp, &response, "authenticatorData")?;
        copy_b64(&resp, &response, "signature")?;
        // `userHandle` is nullable in an assertion.
        let user_handle = get(&response, "userHandle")?;
        if !user_handle.is_undefined() && !user_handle.is_null() {
            let bytes = Uint8Array::new(&user_handle).to_vec();
            set(&resp, "userHandle", &key(&B64.encode(bytes)))?;
        }
        set(&out, "response", &resp)?;
        stringify(&out)
    }
}
