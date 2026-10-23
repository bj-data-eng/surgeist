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
