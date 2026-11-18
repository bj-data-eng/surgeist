//! Binding a Runtime surface to the canonical DOM.
//!
//! A [`RuntimeDomHost`] is the exclusive association and publisher bound to one Runtime instance and surface. Bind and
//! rebind are charged against [`SurfaceDomBindingLimits`]; a capture is a [`RuntimeDomHostPublication`], and a rejected
//! operation is a [`RuntimeDomHostError`].

pub use crate::internal::dom_host::{
    RuntimeDomHost, RuntimeDomHostError, RuntimeDomHostPublication, SurfaceDomBindingLimits,
};
