//! The one owner of large-stack execution for deep recursive parsing.
//!
//! Recursive descent up to a profile's direct depth runs on the caller's stack.
//! Deeper input runs on one scoped parser thread with a fixed larger stack, so
//! the 256-level structural contract never depends on the caller's stack size.
//! When that thread cannot be started (resource pressure, or a target without
//! threads), deep recursion never falls back to the caller's stack and never
//! panics: the input is refused with a typed `ParserUnavailable` outcome whose
//! limit is the depth trusted on the caller's stack, located at the first
//! opening beyond it (#1277). Refused syntax is empty, `None` or the front door's documented
//! replacement. The refusal is a resource failure for every consumer; a front
//! door may still append its tokenizer EOF and escape diagnostics after it.
//! Media construction and serialization, whose depth is a model walk rather
//! than source nesting, share the same thread policy through [`media`].

use super::recovery;
use crate::{
    CssComponentValue, CssComponentValueError, CssComponentValueErrorKind, CssParseReport,
    CssRecoveryAction, CssRecoveryDiagnostic, CssSourcePosition, CssSourceSpan, CssValueOrigin,
};

/// The parser thread's stack, sized for 256 levels of the largest recursive frames.
const PARSER_STACK_BYTES: usize = 16 * 1024 * 1024;

/// The production named by a refusal: the nested block or function whose
/// recursion needs the parser thread.
const REFUSED_PRODUCTION: &str = "css.block";

/// Recursion frame classes with their own trusted caller-stack depth.
#[derive(Clone, Copy)]
pub(super) enum StackProfile {
    /// Stylesheet rule and declaration recursion.
    Structural,
    /// Fragment front doors and checked component grammars. Recursive value
    /// and selector grammar can have larger frames than structural rule
    /// parsing, so the caller's stack is trusted for half the structural depth.
    Fragment,
}

impl StackProfile {
    /// The deepest nesting parsed directly on the caller's stack. This is a
    /// stack admission threshold, not a lower nesting contract.
    const fn direct_depth(self) -> u32 {
        match self {
            Self::Structural => 128,
            Self::Fragment => 63,
        }
    }

    const fn thread_name(self) -> &'static str {
        match self {
            Self::Structural => "surgeist-css-bounded-parser",
            Self::Fragment => "surgeist-css-fragment-parser",
        }
    }
}

/// The larger-stack parser thread could not be started for deep input.
pub(crate) struct ParserThreadUnavailable {
    direct_depth: u32,
}

fn execute<T: Send>(
    profile: StackProfile,
    depth: u32,
    parse: impl FnOnce() -> T + Send,
) -> Result<T, ParserThreadUnavailable> {
    if depth <= profile.direct_depth() {
        return Ok(parse());
    }
    std::thread::scope(|scope| {
        let builder = std::thread::Builder::new()
            .name(profile.thread_name().to_owned())
            .stack_size(PARSER_STACK_BYTES);
        match spawn_parser(scope, builder, parse) {
            Ok(thread) => match thread.join() {
                Ok(value) => Ok(value),
                Err(panic) => std::panic::resume_unwind(panic),
            },
            Err(_) => Err(ParserThreadUnavailable {
                direct_depth: profile.direct_depth(),
            }),
        }
    })
}

/// Parses complete `source` under `profile`, refusing deep input with
/// `refused` syntax when the parser thread is unavailable.
pub(super) fn report<T: Send>(
    profile: StackProfile,
    source: &str,
    refused: impl FnOnce() -> T,
    parse: impl FnOnce() -> CssParseReport<T> + Send,
) -> CssParseReport<T> {
    match execute(profile, recovery::maximum_nested_depth(source), parse) {
        Ok(report) => report,
        Err(unavailable) => CssParseReport::new(refused(), vec![unavailable.diagnostic(source)]),
    }
}

/// Parses checked components nested `depth` levels deep, refusing deep input
/// with a component `ParserUnavailable` error when the parser thread is unavailable.
pub(super) fn components<T: Send, E: Send>(
    components: &[CssComponentValue],
    depth: u32,
    refused: impl FnOnce(CssComponentValueError) -> E,
    parse: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    execute(StackProfile::Fragment, depth, parse).unwrap_or_else(|unavailable| {
        Err(refused(CssComponentValueError::new(
            CssComponentValueErrorKind::ParserUnavailable,
            unavailable.responsible_component(components),
        )))
    })
}

/// Runs fragment-profile work that the caller has already measured as deep.
pub(crate) fn run_deep<T: Send>(
    deep: bool,
    action: impl FnOnce() -> T + Send,
) -> Result<T, ParserThreadUnavailable> {
    execute(
        StackProfile::Fragment,
        if deep { u32::MAX } else { 0 },
        action,
    )
}

/// Media construction or serialization whose caller measured its model as
/// deep. Without the parser thread the operation fails with a component
/// `ParserUnavailable` error at `origin`.
pub(crate) fn media<T: Send, E: Send + From<CssComponentValueError>>(
    deep: bool,
    origin: impl FnOnce() -> CssValueOrigin,
    action: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    run_deep(deep, action).unwrap_or_else(|_| {
        Err(
            CssComponentValueError::new(CssComponentValueErrorKind::ParserUnavailable, origin())
                .into(),
        )
    })
}

/// The first function or block with as many enclosing levels as checked
/// fragment components may use directly on the caller's stack.
pub(crate) fn deep_component_origin(components: &[CssComponentValue]) -> CssValueOrigin {
    ParserThreadUnavailable {
        direct_depth: StackProfile::Fragment.direct_depth(),
    }
    .responsible_component(components)
}

impl ParserThreadUnavailable {
    fn diagnostic(&self, source: &str) -> CssRecoveryDiagnostic {
        let opening =
            recovery::first_opening_at_depth(source, self.direct_depth).unwrap_or(source.len());
        self.diagnostic_at(source, opening)
    }

    /// The refusal diagnostic located at the byte offset of the deep construct.
    pub(crate) fn diagnostic_at(&self, source: &str, opening: usize) -> CssRecoveryDiagnostic {
        let error = crate::error::from_parse_error(
            source,
            crate::error::parser_unavailable(
                source,
                opening,
                self.direct_depth,
                REFUSED_PRODUCTION,
            ),
        );
        let span = CssSourceSpan::new(
            CssSourcePosition::from_byte_offset_in(source, 0),
            CssSourcePosition::from_byte_offset_in(source, source.len()),
        )
        .expect("complete source span");
        CssRecoveryDiagnostic::new(error, span, CssRecoveryAction::StopAtNestingLimit)
            .expect("refusal opening lies within the complete source")
    }

    /// The first function or block with `direct_depth` enclosing levels.
    fn responsible_component(&self, components: &[CssComponentValue]) -> CssValueOrigin {
        let mut items = components;
        for level in 0..=self.direct_depth {
            let remaining = self.direct_depth - level;
            let Some(component) = items.iter().find(|component| {
                component
                    .child_values()
                    .is_some_and(|children| children.nesting_depth() >= remaining)
            }) else {
                break;
            };
            if remaining == 0 {
                return component.origin().clone();
            }
            items = component
                .child_values()
                .expect("selected nested component")
                .items();
        }
        components
            .first()
            .map_or(CssValueOrigin::Programmatic, |component| {
                component.origin().clone()
            })
    }
}

#[cfg(test)]
thread_local! {
    static PARSER_THREAD_REFUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Starts one scoped parser thread. Tests can refuse the start through the
/// private seam below to reach the platform's thread-creation failure path.
fn spawn_parser<'scope, 'env, T: Send + 'scope>(
    scope: &'scope std::thread::Scope<'scope, 'env>,
    builder: std::thread::Builder,
    parse: impl FnOnce() -> T + Send + 'scope,
) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, T>> {
    #[cfg(test)]
    if PARSER_THREAD_REFUSED.get() {
        return Err(std::io::Error::other("parser thread refused by test seam"));
    }
    builder.spawn_scoped(scope, parse)
}

/// Runs `action` while this thread's parser-thread starts fail.
#[cfg(test)]
fn with_parser_thread_unavailable<R>(action: impl FnOnce() -> R) -> R {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            PARSER_THREAD_REFUSED.set(false);
        }
    }
    PARSER_THREAD_REFUSED.set(true);
    let _restore = Restore;
    action()
}

#[cfg(test)]
mod tests {
    //! The crate's structural contract retains up to 256 nested levels. Input
    //! deeper than the caller-stack depth needs the larger parser stack; when
    //! that thread cannot start, the parse is refused with a typed
    //! `ParserUnavailable` outcome at the first opening the caller's stack is
    //! not trusted with (#1277).
    use super::with_parser_thread_unavailable;
    use crate::{
        CssComponentValueErrorKind, CssErrorCode, CssImportance, CssKnownProperty, CssMediaQuery,
        CssPropertyNameRef, CssRecoveryAction, CssRecoveryDiagnostic, CssSerializedOrigin,
        CssValueOrigin,
    };

    /// Structural sheet recursion runs directly to 128 levels.
    const STRUCTURAL_DIRECT_DEPTH: u32 = 128;
    /// Fragment value/selector recursion runs directly below 64 levels.
    const FRAGMENT_DIRECT_DEPTH: u32 = 63;

    fn assert_refusal(
        diagnostics: &[CssRecoveryDiagnostic],
        source: &str,
        limit: u32,
        opening: usize,
    ) {
        let [diagnostic] = diagnostics else {
            panic!("one refusal diagnostic: {diagnostics:?}")
        };
        let error = diagnostic.error();
        assert_eq!(
            error.code(),
            CssErrorCode::ParserUnavailable,
            "{diagnostic:?}"
        );
        assert_eq!(error.nesting_limit(), Some(limit));
        assert_eq!(
            error.production().map(|value| value.as_str()),
            Some("css.block")
        );
        assert!(error.is_resource_refusal());
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(error.position().byte_offset().value(), opening);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }

    fn nested_rules(depth: usize) -> String {
        format!("{}{}", ".a{".repeat(depth), "}".repeat(depth))
    }

    #[test]
    fn deep_sheet_is_refused_typed_when_the_parser_thread_is_unavailable() {
        let source = nested_rules(150);
        let report =
            with_parser_thread_unavailable(|| crate::CssParser::default().parse_sheet(&source));
        assert!(report.syntax().rules().is_empty());
        // `.a{` repeats every three bytes; the brace with 128 enclosing blocks
        // is the 129th, at 3 * 128 + 2.
        assert_refusal(report.diagnostics(), &source, STRUCTURAL_DIRECT_DEPTH, 386);

        let available = crate::CssParser::default().parse_sheet(&source);
        assert!(available.is_clean(), "{:?}", available.diagnostics());
        assert_eq!(available.syntax().rules().len(), 1);
    }

    /// #1277: an unavailable parser thread is a refusal of its own, so its
    /// diagnostic code differs from the 256-level structural nesting limit
    /// that the same input reaches when it is genuinely too deep.
    #[test]
    fn parser_thread_refusal_is_not_reported_as_the_nesting_limit() {
        let refused = with_parser_thread_unavailable(|| {
            crate::CssParser::default().parse_sheet(nested_rules(150))
        });
        let excessive = crate::CssParser::default().parse_sheet(nested_rules(300));
        let nesting = excessive
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
            .expect("300 levels exceed the structural nesting limit");
        let [refusal] = refused.diagnostics() else {
            panic!("one refusal diagnostic: {:?}", refused.diagnostics())
        };
        assert_eq!(refusal.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_ne!(refusal.error().code(), nesting.error().code());
        // Both remain resource refusals, so publishing consumers abort on both.
        assert_eq!(refusal.error().code(), CssErrorCode::ParserUnavailable);
        assert_eq!(nesting.error().code(), CssErrorCode::NestingLimit);
        assert!(refusal.error().is_resource_refusal());
        assert!(nesting.error().is_resource_refusal());
    }

    /// #1277 at the component level: the refusal of deep checked components
    /// differs from a component nesting limit set by the caller's limits.
    #[test]
    fn component_parser_thread_refusal_is_not_reported_as_the_nesting_limit() {
        let source = format!("{}color{}", "(".repeat(70), ")".repeat(70));
        let components = crate::component_values::parse_component_values(&source).unwrap();
        let refused = with_parser_thread_unavailable(|| {
            CssMediaQuery::try_from_components(components.clone(), Default::default())
        })
        .expect_err("refused without the parser thread");
        let refused = refused
            .component_error()
            .expect("component refusal")
            .clone();
        let limited = crate::component_values::parse_component_values_with_limits(
            &source,
            crate::CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap(),
        )
        .expect_err("the caller's depth limit refuses the nesting");
        assert_eq!(limited.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_ne!(refused.kind(), limited.kind());
        // Both remain resource limits, so publishing consumers abort on both.
        assert!(refused.is_parser_unavailable() && !refused.is_nesting_limit());
        assert!(refused.is_resource_limit());
        assert!(limited.is_resource_limit());
    }

    /// A diagnostic that carries the component refusal is a resource refusal too,
    /// the grouping CSSOM's publication admission reads.
    #[test]
    fn component_parser_thread_refusal_in_a_diagnostic_is_a_resource_refusal() {
        let source = format!("{}color{}", "(".repeat(70), ")".repeat(70));
        let components = crate::component_values::parse_component_values(&source).unwrap();
        let refused = with_parser_thread_unavailable(|| {
            CssMediaQuery::try_from_components(components, Default::default())
        })
        .expect_err("refused without the parser thread");
        let component = refused
            .component_error()
            .expect("component refusal")
            .clone();
        let error = crate::error::from_parse_error(
            &source,
            crate::error::invalid_component_value(
                cssparser::SourceLocation { line: 0, column: 1 },
                component,
            ),
        );
        assert_eq!(error.code(), CssErrorCode::InvalidComponentValue);
        assert!(error.component_error().unwrap().is_parser_unavailable());
        assert!(error.is_resource_refusal());
    }

    #[test]
    fn shallow_sheet_needs_no_parser_thread() {
        let source = nested_rules(STRUCTURAL_DIRECT_DEPTH as usize);
        let report =
            with_parser_thread_unavailable(|| crate::CssParser::default().parse_sheet(&source));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1);
    }

    #[test]
    fn deep_style_attribute_is_refused_typed_when_the_parser_thread_is_unavailable() {
        let prefix = "color: red; width: ";
        let source = format!("{prefix}{}{}", "(".repeat(150), ")".repeat(150));
        let report = with_parser_thread_unavailable(|| {
            crate::CssParser::default().parse_block_contents(&source)
        });
        assert!(report.syntax().is_empty());
        // Block contents is a fragment front, so its direct recursion ends below
        // 64 levels like the other fragments; the 64th opening is refused.
        assert_refusal(
            report.diagnostics(),
            &source,
            FRAGMENT_DIRECT_DEPTH,
            prefix.len() + 63,
        );
    }

    fn nested_calc(depth: usize) -> String {
        format!("{}1px{}", "calc(".repeat(depth), ")".repeat(depth))
    }

    #[test]
    fn deep_declaration_fragment_is_refused_typed_instead_of_panicking() {
        let prefix = "width: ";
        let source = format!("{prefix}{}", nested_calc(70));
        let report = with_parser_thread_unavailable(|| {
            crate::CssDeclaration::parse(&crate::CssParser::default(), &source)
        });
        assert!(report.syntax().is_none());
        // Each `calc(` is five bytes; this function has 63 enclosing levels.
        assert_refusal(
            report.diagnostics(),
            &source,
            FRAGMENT_DIRECT_DEPTH,
            prefix.len() + 5 * 63,
        );

        let available = crate::CssDeclaration::parse(&crate::CssParser::default(), &source);
        assert!(available.is_clean(), "{:?}", available.diagnostics());
    }

    #[test]
    fn deep_media_query_fragment_is_refused_as_a_never_query() {
        let source = format!("{}{}", "(".repeat(70), ")".repeat(70));
        let report = with_parser_thread_unavailable(|| {
            crate::CssMediaQuery::parse(&crate::CssParser::default(), &source)
        });
        assert!(matches!(report.syntax(), CssMediaQuery::Never(_)));
        assert_refusal(report.diagnostics(), &source, FRAGMENT_DIRECT_DEPTH, 63);
    }

    #[test]
    fn deep_checked_components_are_refused_with_the_responsible_component() {
        let components = crate::component_values::parse_component_values(&nested_calc(70)).unwrap();
        let error = with_parser_thread_unavailable(|| {
            crate::CssDeclaration::try_from_components(
                &crate::CssParser::default(),
                crate::CssPropertyTarget::Property(CssPropertyNameRef::Known(
                    CssKnownProperty::Width,
                )),
                components.clone(),
                CssImportance::Normal,
            )
        })
        .expect_err("refused without the parser thread");
        let component = error.component_error().expect("component refusal");
        assert_eq!(
            component.kind(),
            CssComponentValueErrorKind::ParserUnavailable
        );
        assert!(component.is_resource_limit());
        assert_eq!(error.code(), None);
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
            panic!("parsed responsible component: {error:?}")
        };
        assert_eq!(origin.span().start().byte_offset().value(), 5 * 63);

        assert!(
            crate::CssDeclaration::try_from_components(
                &crate::CssParser::default(),
                crate::CssPropertyTarget::Property(CssPropertyNameRef::Known(
                    CssKnownProperty::Width
                )),
                components,
                CssImportance::Normal
            )
            .is_ok()
        );
    }

    /// `not` and grouping levels count toward media condition depth, so this
    /// source is only about 41 levels deep yet its query needs the thread.
    fn deep_not_condition() -> String {
        format!("{}(color){}", "(not ".repeat(40), ")".repeat(40))
    }

    #[test]
    fn deep_cssom_media_query_is_refused_typed_instead_of_panicking() {
        let source = deep_not_condition();
        assert!(super::recovery::maximum_nested_depth(&source) <= FRAGMENT_DIRECT_DEPTH);
        let report = with_parser_thread_unavailable(|| {
            crate::CssMediaQuery::parse_medium(&crate::CssParser::default(), &source)
        });
        assert!(report.syntax().is_none());
        let refusals = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::ParserUnavailable)
            .collect::<Vec<_>>();
        let [refusal] = refusals.as_slice() else {
            panic!("one refusal: {:?}", report.diagnostics())
        };
        assert_eq!(refusal.error().nesting_limit(), Some(FRAGMENT_DIRECT_DEPTH));
        assert!(refusal.error().is_resource_refusal());
        assert_eq!(refusal.action(), CssRecoveryAction::StopAtNestingLimit);

        let available = crate::CssMediaQuery::parse_medium(&crate::CssParser::default(), &source);
        assert!(available.is_clean(), "{:?}", available.diagnostics());
        assert!(available.syntax().is_some());
    }

    fn assert_media_component_refusal(error: &crate::CssComponentValueError, offset: usize) {
        assert_eq!(error.kind(), CssComponentValueErrorKind::ParserUnavailable);
        assert!(error.is_resource_limit());
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("parsed responsible component: {error:?}")
        };
        assert_eq!(origin.span().start().byte_offset().value(), offset);
    }

    #[test]
    fn deep_media_construction_is_refused_with_its_component_error() {
        let source = format!("{}color{}", "(".repeat(70), ")".repeat(70));
        let components = crate::component_values::parse_component_values(&source).unwrap();
        let error = with_parser_thread_unavailable(|| {
            CssMediaQuery::try_from_components(components.clone(), Default::default())
        })
        .expect_err("refused without the parser thread");
        let Some(error) = error.component_error() else {
            panic!("component refusal: {error:?}")
        };
        // The block with 63 enclosing blocks opens at byte 63.
        assert_media_component_refusal(error, 63);
        assert!(CssMediaQuery::try_from_components(components, Default::default()).is_ok());
    }

    #[test]
    fn deep_media_serialization_is_refused_with_its_component_error() {
        let source = deep_not_condition();
        let report = crate::CssMediaQuery::parse(&crate::CssParser::default(), &source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let query = report.syntax();
        let error = with_parser_thread_unavailable(|| query.serialize_tokens(Default::default()))
            .expect_err("refused without the parser thread");
        let Some(component) = error.component_error() else {
            panic!("component refusal: {error:?}")
        };
        assert!(error.is_unserializable_boundary(), "{error:?}");
        assert_media_component_refusal(component, 0);
        assert!(query.serialize_tokens(Default::default()).is_ok());
    }
}
