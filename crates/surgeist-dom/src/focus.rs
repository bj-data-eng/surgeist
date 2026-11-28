//! Focus requests and observations, autofocus history and the focus-delegate protocol.
//!
//! `DomFocusRequest` is the single bridge for remembered-focus and allow-focus
//! observations, `DomFocusTarget` names the producer role a request needs, and
//! `DomFocusPreparation`, `DomFocusPublication` and `DomFocusObservation` carry the
//! prepared and published observation. Observing grants no authority to move focus.
//! `DomDocumentAutofocus` and `DomFocusedAreaDesignation` hold retained history; neither
//! resolves the currently focused area. The `DomSource*Focus*` and autofocus request
//! types are the Source continuation for operations that reach focusing.
//! `DomFocusChain` records the ordered Document and navigable-container roles at one
//! immutable canonical snapshot.
//!
//! `docs/dialogs-and-focus.md` owns the contract.

pub use crate::internal::autofocus::DomDocumentAutofocus;
pub use crate::internal::document::DomFocusedAreaDesignation;
pub use crate::internal::source::{
    DomFocusObservation, DomFocusPreparation, DomFocusPublication, DomFocusRequest, DomFocusTarget,
    DomSourceAutofocusInsertionRequest, DomSourceDialogFocusingRequest,
    DomSourceFocusDelegateRequest, DomSourceFocusDelegateStep,
};
pub use crate::internal::store::focus::chain::{
    DomFocusChain, DomFocusChainEntry, DomFocusChainError, DomFocusChainLimits,
};
