//! TC55 Fetch API: `fetch`, `Request`, `Response`, `Headers`.
//!
//! Spec: <https://fetch.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! The Fetch API is required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `fetch`, `Request`, `Response`, and `Headers` from `surgeist_js_test_host::fetch`.

/// Register `fetch`, `Request`, `Response`, and `Headers` into the given context.
///
/// # Errors
///
/// Returns a [`surgeist_js::JsError`] if registration fails.
#[cfg(feature = "fetch")]
pub fn register(
    _realm: Option<surgeist_js::realm::Realm>,
    _ctx: &mut surgeist_js::Context,
) -> surgeist_js::JsResult<()> {
    Ok(())
}
