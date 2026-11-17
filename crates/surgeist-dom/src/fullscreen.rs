//! Fullscreen model and exit protocol, and top-layer state.
//!
//! `DomFullscreenModelStep` names the first reached model action that has no
//! implementation. The `DomFullscreenExit*` types carry the synchronous exit and the
//! runtime-owned parallel completion, which have separate qualifications;
//! `DomFullscreenEvent`, `DomFullscreenEventBatch` and `DomFullscreenEventKind` are the
//! queued event records. `DomDocumentTopLayer` is the borrowed producer input for the top
//! layer that Fullscreen shares with Dialog, and `DomTopLayerUaRule` is a generated
//! declaration for `overlay` that Style collects.
//!
//! `docs/fullscreen.md` owns the contract.

pub use crate::internal::fullscreen::{
    DomFullscreenEvent, DomFullscreenEventBatch, DomFullscreenEventKind, DomFullscreenExitFact,
    DomFullscreenExitId, DomFullscreenExitReturn, DomFullscreenExitStatus, DomFullscreenExitStep,
    DomFullscreenModelStep,
};
pub use crate::internal::store::source::operation::fullscreen::{
    DomFullscreenExitHandoffCommit, DomFullscreenExitJob, DomSourceFullscreenExitHandoffRequest,
};
pub use crate::internal::top_layer::{DomDocumentTopLayer, DomTopLayerUaRule};
