//! Recovery diagnostics, parse errors and their stable codes and fact views.

pub(crate) mod error;
pub(crate) mod recovery;

#[doc(inline)]
pub use error::{
    CssDeclarationContextRef, CssError, CssErrorCode, CssEscapeFailure, CssFeatureId,
    CssGrammarExpectation, CssProductionId, CssTokenKind, CssTokenSummary,
};
#[doc(inline)]
pub use recovery::{CssRecoveryAction, CssRecoveryDiagnostic};
