//! Recovery diagnostics, parse errors and their stable codes and fact views.
//!
//! # Diagnostics and coordinates
//!
//! Each [`CssRecoveryDiagnostic`] exposes a typed [`CssError`] and stable
//! [`CssErrorCode`], the first responsible [`CssSourcePosition`](crate::source::CssSourcePosition), the complete
//! [`CssSourceSpan`](crate::source::CssSourceSpan) of the recovery unit, and the [`CssRecoveryAction`] taken.
//! Byte offsets index the original UTF-8 input. Lines and columns are zero-based,
//! and columns count UTF-16 code units. Display and debug prose are for people;
//! control flow should match typed variants and include a wildcard for every
//! non-exhaustive enum. Closed enums such as [`CssImportance`](crate::declaration::CssImportance) and
//! [`CssSupportStatus`](crate::conformance::CssSupportStatus) remain exhaustively matchable.

pub(crate) mod error;
pub(crate) mod recovery;

#[doc(inline)]
pub use error::{
    CssDeclarationContextRef, CssError, CssErrorCode, CssEscapeFailure, CssFeatureId,
    CssGrammarExpectation, CssProductionId, CssTokenKind, CssTokenSummary,
};
#[doc(inline)]
pub use recovery::{CssRecoveryAction, CssRecoveryDiagnostic};
