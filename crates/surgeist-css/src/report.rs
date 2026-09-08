use crate::{CssSourceSpan, Error};

/// A parser-produced authored syntax value and its ordered recovery diagnostics.
///
/// This authored/diagnostic-phase report always contains valid retained syntax.
/// Its private fields prevent public callers from attaching diagnostics to an
/// invalid retained node, and [`Self::is_clean`] is exactly equivalent to an
/// empty diagnostic slice. A report does not perform application-strict
/// validation, cascade, substitution, contextual resolution, selector matching,
/// or resource loading.
///
/// ```compile_fail
/// use surgeist_css::CssParseReport;
///
/// fn forge<T>(syntax: T) -> CssParseReport<T> {
///     CssParseReport { syntax, diagnostics: Vec::new() }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssParseReport<T> {
    syntax: T,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}

impl<T> CssParseReport<T> {
    pub(crate) fn new(syntax: T, mut diagnostics: Vec<CssRecoveryDiagnostic>) -> Self {
        diagnostics.sort_by_key(|diagnostic| diagnostic.error.position().byte_offset());
        Self {
            syntax,
            diagnostics,
        }
    }

    /// Returns the valid retained authored syntax.
    ///
    /// This authored-phase accessor does not imply that the source was clean and
    /// does not resolve or evaluate the retained syntax.
    #[must_use]
    pub const fn syntax(&self) -> &T {
        &self.syntax
    }

    /// Returns diagnostics ordered by first responsible authored byte offset.
    ///
    /// Equal offsets preserve parser discovery order, including child-before-parent
    /// recovery. This diagnostic-phase accessor preserves recovery provenance; it
    /// does not log diagnostics or apply recovery actions.
    #[must_use]
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }

    /// Reports whether the diagnostic phase produced no recovery diagnostics.
    ///
    /// Cleanliness is exactly an empty diagnostic vector. It is not a separate
    /// validity predicate: retained authored syntax is valid by construction,
    /// and this method performs no application-strict validation or resolution.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Decomposes the report into valid authored syntax and ordered diagnostics.
    ///
    /// This authored/diagnostic-phase operation preserves both owned values and
    /// their invariants. It does not validate, resolve, or otherwise reinterpret
    /// the syntax or diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (T, Vec<CssRecoveryDiagnostic>) {
        (self.syntax, self.diagnostics)
    }

    #[cfg(feature = "app-strict")]
    pub(crate) fn into_validation_result(self) -> Result<T, CssValidationFailure> {
        let (syntax, diagnostics) = self.into_parts();
        match CssValidationFailure::new(diagnostics) {
            Some(failure) => Err(failure),
            None => Ok(syntax),
        }
    }
}

/// A diagnostic-phase description of one CSS-owned recovery decision.
///
/// The variants are the complete I01 action vocabulary and the enum is
/// non-exhaustive so later initiatives can add actions without invalidating
/// wildcard-compatible consumers. An action records what recovery did; it does
/// not itself parse, mutate syntax, log, validate, cascade, or resolve values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssRecoveryAction {
    /// The diagnostic phase discarded one invalid authored declaration.
    DropDeclaration,
    /// The diagnostic phase discarded one invalid authored at-rule descriptor.
    DropDescriptor,
    /// The diagnostic phase discarded one invalid authored qualified rule.
    DropQualifiedRule,
    /// The diagnostic phase discarded one invalid authored at-rule.
    DropAtRule,
    /// The diagnostic phase discarded one invalid authored keyframe block.
    DropKeyframeBlock,
    /// The diagnostic phase discarded one invalid member of a forgiving selector list.
    DropSelectorListItem,
    /// The diagnostic phase retained a guaranteed-false sentinel for an invalid media query.
    ReplaceMediaQueryWithNever,
    /// The diagnostic phase retained valid authored syntax after an implicit EOF closure.
    RetainWithImplicitClosure,
    /// The diagnostic phase ignored one authored top-level legacy CDO or CDC token.
    IgnoreLegacyToken,
    /// The diagnostic phase stopped retaining the smallest unit at the nesting limit.
    StopAtNestingLimit,
}

/// A structured parse error, its recovery-unit span, and the action taken.
///
/// This diagnostic-phase value has private fields. Crate-owned checked
/// construction requires `span.start() <= error.position() <= span.end()` by
/// byte offset, even though the already-valid span is
/// inclusive-start/exclusive-end. Equality at the end and zero-width spans are
/// permitted for missing-token and EOF provenance. The value reports a
/// completed recovery decision; it does not execute recovery, retain invalid
/// authored nodes, log, validate, cascade, or resolve syntax.
///
/// ```compile_fail
/// use surgeist_css::{CssRecoveryAction, CssRecoveryDiagnostic, CssSourceSpan, Error};
///
/// fn forge(error: Error, span: CssSourceSpan) -> CssRecoveryDiagnostic {
///     CssRecoveryDiagnostic {
///         error,
///         span,
///         action: CssRecoveryAction::DropDeclaration,
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssRecoveryDiagnostic {
    error: Error,
    span: CssSourceSpan,
    action: CssRecoveryAction,
}

impl CssRecoveryDiagnostic {
    #[must_use]
    pub(crate) fn new(
        error: Error,
        span: CssSourceSpan,
        action: CssRecoveryAction,
    ) -> Option<Self> {
        let error_offset = error.position().byte_offset();
        if error_offset < span.start().byte_offset() || span.end().byte_offset() < error_offset {
            return None;
        }

        Some(Self {
            error,
            span,
            action,
        })
    }

    /// Returns the structured error at the first responsible source position.
    ///
    /// This diagnostic-phase accessor preserves typed error provenance. Display
    /// prose is not control flow, and the accessor performs no logging or parsing.
    #[must_use]
    pub const fn error(&self) -> &Error {
        &self.error
    }

    /// Returns the authored-source recovery-unit span.
    ///
    /// This diagnostic-phase metadata is ordered. By construction, the error
    /// position lies at or after the inclusive start and at or before the
    /// exclusive end. Equality at the end is permitted for missing-token, EOF,
    /// and zero-width provenance; it does not imply slice containment. The span
    /// does not imply that any invalid authored node was retained and does not
    /// perform source loading or syntax resolution.
    #[must_use]
    pub const fn span(&self) -> CssSourceSpan {
        self.span
    }

    /// Returns the CSS-owned action associated with this diagnostic.
    ///
    /// This diagnostic-phase choice is related to, but distinct from, the error
    /// and span. Reading it does not execute recovery, validation, or downstream
    /// cascade and resolution work.
    #[must_use]
    pub const fn action(&self) -> CssRecoveryAction {
        self.action
    }
}

/// A validation-phase rejection containing one or more recovery diagnostics.
///
/// The private diagnostic vector is non-empty by checked construction, so
/// [`Self::first`] is always defined and all diagnostics remain available in
/// source order. This staged value does not expose validators, parse source,
/// alter ordinary parser behavior, or perform cascade, substitution, contextual
/// resolution, selector matching, or resource loading.
///
/// ```compile_fail
/// use surgeist_css::CssValidationFailure;
///
/// let _ = CssValidationFailure { diagnostics: Vec::new() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssValidationFailure {
    diagnostics: Vec<CssRecoveryDiagnostic>,
}

impl CssValidationFailure {
    #[must_use]
    #[cfg_attr(
        not(any(test, feature = "app-strict")),
        expect(
            dead_code,
            reason = "validation failure construction is feature-gated with app-strict"
        )
    )]
    pub(crate) fn new(diagnostics: Vec<CssRecoveryDiagnostic>) -> Option<Self> {
        if diagnostics.is_empty() {
            None
        } else {
            Some(Self { diagnostics })
        }
    }

    /// Returns every diagnostic that caused validation-phase rejection.
    ///
    /// The slice is non-empty and remains in parser-produced source order. This
    /// accessor does not reparse, revalidate, log, or resolve the authored syntax.
    #[must_use]
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }

    /// Returns the first diagnostic that caused validation-phase rejection.
    ///
    /// Checked private construction guarantees that this diagnostic exists. The
    /// accessor does not assign priority beyond source order and performs no
    /// parsing, logging, cascade, or resolution.
    #[must_use]
    pub fn first(&self) -> &CssRecoveryDiagnostic {
        &self.diagnostics[0]
    }

    /// Consumes the validation failure and returns its non-empty diagnostics.
    ///
    /// This validation-phase operation preserves source order and diagnostic
    /// invariants. It does not retry parsing, execute recovery, or resolve syntax.
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<CssRecoveryDiagnostic> {
        self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::{CssParseReport, CssRecoveryAction, CssRecoveryDiagnostic, CssValidationFailure};
    use crate::test_support::CssParseReportTestExt;
    use crate::{CssSourcePosition, CssSourceSpan, Error, parse_sheet};

    const INVALID_SOURCE: &str = ".x { mystery: 1; }";

    fn invalid_property_error() -> Error {
        parse_sheet(INVALID_SOURCE).expect_err("unknown property must remain a strict error")
    }

    fn diagnostic(action: CssRecoveryAction) -> CssRecoveryDiagnostic {
        diagnostic_for(INVALID_SOURCE, action)
    }

    fn diagnostic_for(source: &str, action: CssRecoveryAction) -> CssRecoveryDiagnostic {
        let error = parse_sheet(source).expect_err("diagnostic source must remain a strict error");
        let span = CssSourceSpan::new(
            error.position(),
            CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("ordered recovery span");

        CssRecoveryDiagnostic::new(error, span, action)
            .expect("error position lies inside recovery span")
    }

    #[test]
    fn report_clean_value_has_no_diagnostics_and_decomposes() {
        let report = CssParseReport::new(vec![1_u8, 2], Vec::new());

        assert_eq!(report.syntax(), &[1, 2]);
        assert!(report.diagnostics().is_empty());
        assert!(report.is_clean());

        let (syntax, diagnostics) = report.into_parts();
        assert_eq!(syntax, vec![1, 2]);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn report_recovered_value_is_not_clean_and_decomposes_without_loss() {
        let diagnostic = diagnostic(CssRecoveryAction::DropDeclaration);
        let report = CssParseReport::new("retained syntax", vec![diagnostic.clone()]);

        assert_eq!(report.syntax(), &"retained syntax");
        assert_eq!(report.diagnostics(), std::slice::from_ref(&diagnostic));
        assert!(!report.is_clean());

        let (syntax, diagnostics) = report.into_parts();
        assert_eq!(syntax, "retained syntax");
        assert_eq!(diagnostics, vec![diagnostic]);
    }

    #[test]
    fn structural_recovery_report_orders_offsets_and_preserves_child_parent_ties() {
        let child = diagnostic(CssRecoveryAction::DropDeclaration);
        let parent = CssRecoveryDiagnostic::new(
            child.error().clone(),
            child.span(),
            CssRecoveryAction::DropAtRule,
        )
        .expect("the parent tie shares the child's responsible position");
        let earlier = diagnostic_for("@bad;", CssRecoveryAction::DropAtRule);

        let report = CssParseReport::new((), vec![child.clone(), parent.clone(), earlier.clone()]);

        assert_eq!(report.diagnostics(), &[earlier, child, parent]);
    }

    #[test]
    fn report_diagnostic_exposes_related_error_span_and_action() {
        let error = invalid_property_error();
        let error_position = error.position();
        let span = CssSourceSpan::new(
            error_position,
            CssSourcePosition::from_byte_offset_in(INVALID_SOURCE, INVALID_SOURCE.len()),
        )
        .expect("ordered recovery span");
        let diagnostic =
            CssRecoveryDiagnostic::new(error.clone(), span, CssRecoveryAction::DropDeclaration)
                .expect("related diagnostic");

        assert_eq!(diagnostic.error(), &error);
        assert_eq!(diagnostic.span(), span);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    }

    #[test]
    fn report_diagnostic_accepts_zero_width_missing_token_span_at_exclusive_end() {
        const SOURCE: &str = "@media screen;";

        let error = parse_sheet(SOURCE).expect_err("media requires a missing block at EOF");
        let end = CssSourcePosition::from_byte_offset_in(SOURCE, SOURCE.len());
        assert_eq!(error.position(), end);
        let span = CssSourceSpan::new(end, end).expect("zero-width missing-token span");

        let diagnostic = CssRecoveryDiagnostic::new(error, span, CssRecoveryAction::DropAtRule)
            .expect("error position may equal the exclusive span end");

        assert_eq!(diagnostic.span().start(), diagnostic.span().end());
        assert_eq!(diagnostic.error().position(), diagnostic.span().end());
    }

    #[test]
    fn report_diagnostic_rejects_error_position_outside_recovery_span() {
        let error = invalid_property_error();
        let before_error = CssSourceSpan::new(
            CssSourcePosition::from_byte_offset_in(INVALID_SOURCE, 0),
            CssSourcePosition::from_byte_offset_in(
                INVALID_SOURCE,
                error.position().byte_offset().value() - 1,
            ),
        )
        .expect("ordered span before error");
        let after_error = CssSourceSpan::new(
            CssSourcePosition::from_byte_offset_in(
                INVALID_SOURCE,
                error.position().byte_offset().value() + 1,
            ),
            CssSourcePosition::from_byte_offset_in(INVALID_SOURCE, INVALID_SOURCE.len()),
        )
        .expect("ordered span after error");

        assert!(
            CssRecoveryDiagnostic::new(
                error.clone(),
                before_error,
                CssRecoveryAction::DropDeclaration,
            )
            .is_none()
        );
        assert!(
            CssRecoveryDiagnostic::new(error, after_error, CssRecoveryAction::DropDeclaration,)
                .is_none()
        );
    }

    #[test]
    fn report_validation_failure_requires_diagnostics_and_preserves_order() {
        assert!(CssValidationFailure::new(Vec::new()).is_none());

        let first = diagnostic(CssRecoveryAction::DropDeclaration);
        let second = diagnostic_for("@not-a-css-rule;", CssRecoveryAction::DropAtRule);
        let failure = CssValidationFailure::new(vec![first.clone(), second.clone()])
            .expect("non-empty diagnostics must form a validation failure");

        assert_eq!(failure.first(), &first);
        assert_eq!(failure.diagnostics(), &[first.clone(), second.clone()]);
        assert_eq!(failure.into_diagnostics(), vec![first, second]);
    }
}
