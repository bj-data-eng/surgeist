#![expect(
    dead_code,
    reason = "each integration target uses a different assertion subset"
)]

use surgeist_css::{CssParseReport, Error};

pub trait CssParseReportTestExt<T> {
    fn expect(self, message: &str) -> T;
    fn expect_err(self, message: &str) -> Error;
    fn unwrap_or_else<F>(self, operation: F) -> T
    where
        F: FnOnce(Error) -> T;
}

impl<T> CssParseReportTestExt<T> for CssParseReport<T> {
    fn expect(self, message: &str) -> T {
        let (syntax, diagnostics) = self.into_parts();
        assert!(diagnostics.is_empty(), "{message}: {diagnostics:?}");
        syntax
    }

    fn expect_err(self, message: &str) -> Error {
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
        F: FnOnce(Error) -> T,
    {
        let (syntax, diagnostics) = self.into_parts();
        diagnostics
            .into_iter()
            .next()
            .map_or(syntax, |diagnostic| operation(diagnostic.error().clone()))
    }
}
