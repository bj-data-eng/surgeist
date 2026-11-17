//! Checked identities, revisions and stamps that qualify every read and change.
//!
//! `NodeId`, `DocumentId` and `DomAttrId` name exact nodes of one `Dom`; they are
//! generation-safe, and their fields cannot be fabricated. `DomOwnerId` names the
//! exclusive store, `DomRevision` counts its published revisions and `DomStamp` and
//! `DomDocumentStamp` pair an owner (and a document) with the revision a value was read
//! at.
//!
//! Use these when a value must stay tied to the snapshot it came from: a revision
//! advances once per observably changed batch, so a stamp shows whether a read is still
//! current. A snapshot retains its exact identities and data after later edits. The
//! canonical-tree contract (`docs/canonical-tree.md`) owns the rules.

pub use crate::internal::identity::{
    DocumentId, DomAttrId, DomOwnerId, DomRevision, DomStamp, NodeId,
};
pub use crate::internal::snapshot::DomDocumentStamp;
