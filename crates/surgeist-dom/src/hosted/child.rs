//! Child navigables created and removed inside a hosted document.
//!
//! `DomHostedChildPreparation`, `DomHostedChildRequest` and `DomHostedChildStep` create a
//! child, and `DomHostedChildRemovalRequest`, `DomHostedChildRemovalHandoff` and
//! `DomHostedChildRemovalPublication` remove one. `DomHostedEmbedderKind` and
//! `DomHostedEmbedderPolicyInputs` describe the embedding element (the latter holds raw
//! facts, not a final permissions-policy answer), and `DomHostedPreparedChild` is the
//! producer's acknowledgement of the retained original prefix. The queued history of a
//! child is in `hosted::history`.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedChildPreparation, DomHostedChildRemovalHandoff, DomHostedChildRemovalPublication,
    DomHostedChildRemovalRequest, DomHostedChildRequest, DomHostedChildStep, DomHostedEmbedderKind,
    DomHostedEmbedderPolicyInputs, DomHostedPreparedChild,
};
