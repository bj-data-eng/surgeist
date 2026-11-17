//! Attribute write verification hooks of the Source continuation.
//!
//! `DomSourceAttributeVerificationRequest` observes one verification of an attribute
//! write, `DomSourceAttributeVerificationOutcome` is the qualified verifier's result,
//! `DomSourceAttributeSink` is the sink selected for a standard write, and the remaining
//! types scope the callback, the policy read and the retained payload.
//!
//! `docs/source-and-style.md` owns the contract.

pub use crate::internal::source::{
    DomSourceAttributeAttachmentOutput, DomSourceAttributeCallbackScope,
    DomSourceAttributePayloadRetention, DomSourceAttributePolicyRead, DomSourceAttributeSink,
    DomSourceAttributeVerificationOutcome, DomSourceAttributeVerificationRequest,
};
