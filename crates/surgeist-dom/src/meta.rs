//! Meta element state, document policy and CSP, and the native Meta composition.
//!
//! `DomMetaDocumentState` reads the established Meta processing history of a Document,
//! captured with its revision; it is not a summary of the currently attached elements.
//! `DomMetaColorSchemeState` and `DomMetaThemeColorState` record the last completed
//! selections, `DomDocumentCspState` is source-qualified knowledge of the CSP policy
//! container, and `DomMetaSubject` and `DomMetaPreferredOccurrence` identify the
//! historical subject of a Meta operation and a preferred-name invocation. The
//! `DomMetaOrdinary*` and `DomMetaSource*` types stage an ordinary or a Source Meta
//! change, and their published custody keeps the actual locks until the runtime has
//! installed its names. `DomSourceMetaCspRequest` observes one Meta CSP insertion.
//!
//! `docs/urls-and-metadata.md` and `docs/source-and-style.md` (native Meta composition)
//! own the contracts.

pub use crate::internal::meta::{
    DomDocumentCspState, DomDocumentPolicyContainer, DomMetaColorSchemeState, DomMetaDocumentState,
    DomMetaError, DomMetaThemeColorState,
};
pub use crate::internal::meta_atomic::{
    DomMetaAtomicError, DomMetaAtomicLimits, DomMetaInlineBatch, DomMetaInlineResolver,
    DomMetaOrdinaryCommit, DomMetaPreferredOccurrence, DomMetaRetirement, DomMetaRootCommit,
    DomMetaRuntimePrefix, DomMetaRuntimeUsage, DomMetaSourceCommit, DomMetaSubject,
};
pub use crate::internal::source::DomSourceMetaCspRequest;
pub use crate::internal::store::meta_atomic::{
    DomMetaOrdinaryPreparation, DomMetaOrdinaryPublished,
};
pub use crate::internal::store::source::operation::meta::{
    DomMetaSourcePreparation, DomMetaSourcePublished, DomMetaSourceRuntimeHold, DomMetaSourceStep,
    DomSourceMetaCspError, DomSourceMetaCspProcessorFailure,
};
