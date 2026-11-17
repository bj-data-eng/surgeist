//! Constructed stylesheets and style-root withdrawal for a hosted document.
//!
//! `DomHostedSheetConstruction` stages a CSSOM batch and an optional canonical attachment
//! (with a published result and a typed error), and `DomHostedStyleRootWithdrawal` is
//! evidence that canonical custody of a style root was withdrawn (with a preparation and
//! a published result).
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedSheetConstruction, DomHostedSheetConstructionError,
    DomHostedSheetConstructionPublished, DomHostedStyleRootWithdrawal,
    DomHostedStyleRootWithdrawalPreparation, DomHostedStyleRootWithdrawalPublished,
};
