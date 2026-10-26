//! Assertions that a parse report is clean, shared by report-driven tests.

use surgeist_css::{CssError, CssParseReport};

pub trait CssParseReportTestExt<T> {
    fn expect(self, message: &str) -> T;
    fn expect_err(self, message: &str) -> CssError;
    fn unwrap_or_else<F>(self, operation: F) -> T
    where
        F: FnOnce(CssError) -> T;
}

impl<T> CssParseReportTestExt<T> for CssParseReport<T> {
    fn expect(self, message: &str) -> T {
        let (syntax, diagnostics) = self.into_parts();
        assert!(diagnostics.is_empty(), "{message}: {diagnostics:?}");
        syntax
    }

    fn expect_err(self, message: &str) -> CssError {
        let (_, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{message}"))
            .error()
            .clone()
    }

    fn unwrap_or_else<F>(self, operation: F) -> T
    where
        F: FnOnce(CssError) -> T,
    {
        let (syntax, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .map_or(syntax, |diagnostic| operation(diagnostic.error().clone()))
    }
}

/// Asserts the whole-contents refusal of block contents (Syntax 3 (2026 CRD)
/// §5.5.5: the contents are one unit charged against the component limits).
/// One diagnostic spans the complete source with a component resource-limit
/// cause.
pub fn assert_contents_refused<T: std::fmt::Debug>(
    report: &CssParseReport<T>,
    source: &str,
    context: &str,
) {
    let [diagnostic] = report.diagnostics() else {
        panic!("{context}: one diagnostic: {:?}", report.diagnostics());
    };
    assert!(
        diagnostic.error().is_resource_refusal(),
        "{context}: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .error()
            .component_error()
            .is_some_and(surgeist_css::CssComponentValueError::is_resource_limit),
        "{context}: {diagnostic:?}"
    );
    assert_eq!(
        (
            diagnostic.span().start().byte_offset().value(),
            diagnostic.span().end().byte_offset().value()
        ),
        (0, source.len()),
        "{context}"
    );
}
