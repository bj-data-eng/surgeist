//! TC55 URL APIs: `URL`, `URLSearchParams`.
//!
//! Spec: <https://url.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! `URL` and `URLSearchParams` are required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `URL` from `surgeist_js_test_host::url`.
//! - Implement `URLSearchParams`.

/// Register `URL` and `URLSearchParams` into the given context.
///
/// # Errors
///
/// Returns a [`surgeist_js::JsError`] if registration fails.
#[cfg(feature = "url")]
pub fn register(
    _realm: Option<surgeist_js::realm::Realm>,
    _ctx: &mut surgeist_js::Context,
) -> surgeist_js::JsResult<()> {
    Ok(())
}
