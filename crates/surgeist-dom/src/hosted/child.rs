//! Child navigables created and removed inside a hosted document.
//!
//! The `DomHostedChild*` types prepare, publish and remove a child,
//! `DomHostedEmbedderKind` and `DomHostedEmbedderPolicyInputs` describe the embedding
//! element (the latter holds raw facts, not a final permissions-policy answer), and
//! `DomHostedPreparedChild` is the producer's acknowledgement of a prepared child.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedChildPreparation, DomHostedChildRemovalHandoff, DomHostedChildRemovalPublication,
    DomHostedChildRemovalRequest, DomHostedChildRequest, DomHostedChildStep, DomHostedEmbedderKind,
    DomHostedEmbedderPolicyInputs, DomHostedPreparedChild,
};
