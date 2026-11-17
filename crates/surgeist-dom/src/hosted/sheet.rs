//! Constructed stylesheets and style-root withdrawal for a hosted document.
//!
//! `DomHostedSheetConstruction` stages a CSSOM batch and an optional canonical
//! attachment, and `DomHostedStyleRootWithdrawal` is evidence that canonical custody of a
//! style root was withdrawn; each has a preparation or a published result and, for
//! construction, a typed error.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedSheetConstruction, DomHostedSheetConstructionError,
    DomHostedSheetConstructionPublished, DomHostedStyleRootWithdrawal,
    DomHostedStyleRootWithdrawalPreparation, DomHostedStyleRootWithdrawalPublished,
};
