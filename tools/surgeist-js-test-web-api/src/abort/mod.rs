//! TC55 `AbortController` and `AbortSignal` implementation.
//!
//! Spec: <https://dom.spec.whatwg.org/#interface-abortcontroller>
//!
//! # TC55 Status
//!
//! `AbortController` and `AbortSignal` are required in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `AbortController` and `AbortSignal` from `surgeist_js_test_host::abort`.

/// Register `AbortController` and `AbortSignal` into the given context.
///
/// # Errors
///
/// Returns a [`surgeist_js::JsError`] if registration fails.
pub fn register(
    _realm: Option<surgeist_js::realm::Realm>,
    _ctx: &mut surgeist_js::Context,
) -> surgeist_js::JsResult<()> {
    Ok(())
}
