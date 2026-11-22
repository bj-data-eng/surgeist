//! Document class, interface, creation cause, environment and metadata.
//!
//! `DomDocumentMetadata` is the canonical metadata of a Document, captured together with
//! its tree and revision. `DomDocumentClass` and `DomDocumentInterface` record the
//! Document's type and its actual implemented interface, `DomDocumentCreationCause`
//! records how it came to exist, and `DomDocumentEnvironment` selects the closed offline
//! environment that native creation and completed offline installation share.
//! `DomOpaqueOrigin` is a real opaque-origin identity, and `DomNativeInputPlatform` holds
//! the native selection capabilities of an environment.
//!
//! Each native creation has its own opaque origin, distinct across documents and stores
//! even when their URLs match; cloning preserves the identity.
//! `docs/document-creation.md` owns the contract.

pub use crate::internal::document::{
    DomDocumentClass, DomDocumentCreationCause, DomDocumentEnvironment, DomDocumentInterface,
    DomDocumentMetadata, DomDocumentReadiness, DomNativeInputPlatform, DomOpaqueOrigin,
};
