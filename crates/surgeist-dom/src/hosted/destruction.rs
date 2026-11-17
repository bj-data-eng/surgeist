//! Hosted document destruction and its proofs.
//!
//! The `DomHostedDestruction*` types carry a destruction request, its lease and its
//! publication, and `DomHostedDocumentDestructionProof` is a sealed observation that
//! cleanup of one exact Document and Global generation succeeded.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedDestructionInvocation, DomHostedDestructionInvocationRequest,
    DomHostedDestructionLease, DomHostedDestructionPublication, DomHostedDestructionRequest,
    DomHostedDestructionStep, DomHostedDocumentDestructionProof,
};
