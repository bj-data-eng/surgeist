//! TC55 event APIs: `EventTarget`, `Event`, `CustomEvent`, `ErrorEvent`, `MessageEvent`.
//!
//! Spec: <https://dom.spec.whatwg.org/#interface-eventtarget>
//!
//! # TC55 Status
//!
//! `EventTarget` and associated event interfaces are required in the `WinterTC` TC55
//! Minimum Common Web API.
//!
//! # TODO
//!
//! - Implement `EventTarget`.
//! - Implement `Event`.
//! - Implement `CustomEvent`.
//! - Implement `ErrorEvent`.
//! - Implement `MessageEvent`.

/// Register event globals (`EventTarget`, `Event`, `CustomEvent`, `ErrorEvent`,
/// `MessageEvent`) into the given context.
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
