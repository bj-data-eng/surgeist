use super::{Error, ErrorCode};
use crate::{RenderAccountingError, RenderQuantity, RenderRefusalReason, RenderWork};
use std::{error::Error as _, fmt};

#[test]
fn accounting_failure_preserves_typed_cause_and_full_human_message() {
    let diagnostic = RenderAccountingError::GenerationRefused {
        quantity: RenderQuantity::Work(RenderWork::ResourceAllocations),
        current: 3,
        amount: 1,
        reason: RenderRefusalReason::Limit { limit: 3 },
    };
    let error = Error::from(diagnostic);
    let message = format!("render accounting: {diagnostic}");
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(error.to_string(), format!("Accounting: {message}"));
    assert_eq!(error.message(), message);
    assert_eq!(error.message(), message);
    assert!(error.source().is_none());
    assert!(error.glyph_preparation_diagnostic().is_none());
    assert!(error.invalid_value_diagnostic().is_none());
    assert!(error.unsupported_primitive().is_none());
    assert!(error.unresolved_resource_diagnostic().is_none());
    assert!(error.degraded_quality_diagnostic().is_none());
    assert!(error.runtime_capability_unavailable_diagnostic().is_none());
    #[cfg(feature = "render-text")]
    assert!(error.text_layout_diagnostic().is_none());
}

#[test]
fn const_accounting_failure_is_ready_for_typed_observation() {
    // Const evaluation requires neither runtime formatting nor an allocated
    // diagnostic context; the initialized error then crosses the real API.
    let error = const { Error::accounting(RenderAccountingError::GenerationRetired) };
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::GenerationRetired)
    );
    assert_eq!(error.message(), "render accounting: GenerationRetired");
}

#[derive(Debug)]
struct Cause;
impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("source marker")
    }
}
impl std::error::Error for Cause {}

#[test]
fn explicit_message_and_source_enrichment_keep_the_accounting_cause() {
    let diagnostic = RenderAccountingError::Invariant;
    let mut error = Error::from(diagnostic);
    error.append_message(format_args!(" at command {}", 7));
    let mut error = error.with_source(Cause);
    assert_eq!(error.code(), ErrorCode::Accounting);
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(error.message(), "render accounting: Invariant at command 7");
    assert_eq!(error.source().unwrap().to_string(), "source marker");
    error.replace_message("replaced diagnostic prose");
    assert_eq!(error.message(), "replaced diagnostic prose");
    assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(error.source().unwrap().to_string(), "source marker");
    assert_eq!(error.to_string(), "Accounting: replaced diagnostic prose");
}
