//! Source-qualified declarations and installation plans, without an installer.
//!
//! `bindings/dom-core.json` is authored. Its generator validates exact retained
//! IDL blocks and emits `dom_core`. Access records distinguish native adapter
//! contracts, derived projections, guest callback/forwarding protocols and
//! external host requirements. They supply no callable handlers or successful
//! native support. This internal boundary is intentionally not wired to the
//! crate until an actual consumer is ready.

pub(super) mod dom_core;
pub(super) mod model;
