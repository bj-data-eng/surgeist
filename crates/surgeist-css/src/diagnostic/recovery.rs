use crate::diagnostic::CssError;
use crate::source::CssSourceSpan;

/// A diagnostic-phase description of one CSS-owned recovery decision.
///
/// The variants describe the parser's recovery vocabulary and the enum is
/// non-exhaustive so later recovery support can add actions without invalidating
/// wildcard-compatible consumers. An action records what recovery did; it does
/// not itself parse, mutate syntax, log, validate, cascade, or resolve values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssRecoveryAction {
    /// The requested complete fragment was rejected.
    RejectInput,
    /// The diagnostic phase discarded one invalid authored declaration.
    DropDeclaration,
    /// The diagnostic phase discarded one invalid authored at-rule descriptor.
    DropDescriptor,
    /// The diagnostic phase discarded one invalid source while retaining valid `src` fallbacks.
    DropFontSourceListItem,
    /// The diagnostic phase discarded one invalid authored qualified rule.
    DropQualifiedRule,
    /// The diagnostic phase discarded one invalid authored at-rule.
    DropAtRule,
    /// The diagnostic phase discarded one invalid authored keyframe block.
    DropKeyframeBlock,
    /// The diagnostic phase discarded one invalid member of a forgiving selector list.
    DropSelectorListItem,
    /// Retained a match-nothing invalid nesting-containing member of Is/Where.
    PreserveInvalidSelectorListItem,
    /// The diagnostic phase retained a guaranteed-false sentinel for an invalid media query.
    ReplaceMediaQueryWithNever,
    /// The diagnostic phase retained valid authored syntax after an implicit EOF closure.
    RetainWithImplicitClosure,
    /// The diagnostic phase retained a grammatical rule that made its stylesheet nonconforming.
    RetainNonconformingRule,
    /// Preliminary generic rule syntax retained a recoverable fault before
    /// semantic grammar admission. This does not assert a retained semantic rule.
    RetainSyntaxCandidate,
    /// Retained an underscore-leading target as symbolic legacy navigation syntax.
    RetainLegacyNavigationTarget,
    /// The tokenizer ignored an authored comment terminated by EOF instead of `*/`.
    /// This recovery is independent of retention or rejection of surrounding syntax.
    IgnoreUnterminatedComment,
    /// The tokenizer recovered an authored escape as a backslash delimiter or U+FFFD.
    /// This lexical decision is independent of retention or rejection of surrounding syntax.
    RecoverEscape,
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
/// use surgeist_css::diagnostic::{CssRecoveryAction, CssRecoveryDiagnostic, CssError};
/// use surgeist_css::source::CssSourceSpan;
///
/// fn forge(error: CssError, span: CssSourceSpan) -> CssRecoveryDiagnostic {
///     CssRecoveryDiagnostic {
///         error,
///         span,
///         action: CssRecoveryAction::DropDeclaration,
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CssRecoveryDiagnostic {
    pub(crate) error: CssError,
    span: CssSourceSpan,
    action: CssRecoveryAction,
    opening_owner: Option<usize>,
    source: Option<crate::source::CssSourceSnapshot>,
}

impl PartialEq for CssRecoveryDiagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error && self.span == other.span && self.action == other.action
    }
}
impl Eq for CssRecoveryDiagnostic {}
impl CssRecoveryDiagnostic {
    /// Actual diagnostic-owned out-of-line backing, excluding this diagnostic's
    /// inline layout and its separately shared source graph. String/Vec spare
    /// capacity and boxed payloads are included. Recomputed after clones.
    /// Each reached observation is admitted before its operands are read.
    pub fn retained_backing_bytes_with<E>(
        &self,
        mut admit: impl FnMut() -> Result<(), E>,
    ) -> Result<usize, crate::source::CssRetainedPayloadError<E>> {
        self.error.retained_backing_bytes_with(&mut admit)
    }

    /// Actual immutable parser input, when established by the owning parser.
    #[must_use]
    pub const fn source(&self) -> Option<&crate::source::CssSourceSnapshot> {
        self.source.as_ref()
    }

    pub(crate) fn bind_source(&mut self, source: &crate::source::CssSourceSnapshot) {
        self.source = Some(source.clone());
    }
    pub(crate) fn resolve_original_coordinates(&mut self, source: &str) {
        self.error.resolve_original_coordinates(source);
        self.span = CssSourceSpan::new(
            crate::source::CssSourcePosition::from_byte_offset_in(
                source,
                self.span.start().byte_offset().value(),
            ),
            crate::source::CssSourcePosition::from_byte_offset_in(
                source,
                self.span.end().byte_offset().value(),
            ),
        )
        .expect("same-length replay preserves ordered original byte boundaries");
    }

    pub(crate) fn with_opening_owner(mut self, owner: usize) -> Self {
        self.opening_owner = Some(owner);
        self
    }
    pub(crate) fn opening_owner(&self) -> Option<usize> {
        self.opening_owner
    }
    pub(crate) fn clear_opening_owner(&mut self) {
        self.opening_owner = None;
    }

    #[must_use]
    pub(crate) fn new(
        error: CssError,
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
            opening_owner: None,
            source: None,
        })
    }

    /// Returns the structured error at the first responsible source position.
    ///
    /// This diagnostic-phase accessor preserves typed error provenance. Display
    /// prose is not control flow, and the accessor performs no logging or parsing.
    #[must_use]
    pub const fn error(&self) -> &CssError {
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
