//! TC55 encoding APIs: `TextEncoder`, `TextDecoder`, `TextEncoderStream`, `TextDecoderStream`.
//!
//! Spec: <https://encoding.spec.whatwg.org/>
//!
//! # TC55 Status
//!
//! `TextEncoder`, `TextDecoder`, `TextEncoderStream`, and `TextDecoderStream` are required
//! in the `WinterTC` TC55 Minimum Common Web API.
//!
//! # TODO
//!
//! - Migrate `TextEncoder` and `TextDecoder` from `surgeist_js_test_host::text`.
//! - Implement `TextEncoderStream`.
//! - Implement `TextDecoderStream`.

/// Register encoding globals (`TextEncoder`, `TextDecoder`, `TextEncoderStream`,
/// `TextDecoderStream`) into the given context.
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
