//! Fixtures shared by the color and selector modules.
//!
//! Color helpers build authored color declarations and check bounded color
//! serialization. Selector helpers build namespace contexts and destructure
//! selector graphs, and give typed evidence for selector construction and
//! whole-selector rejection.

use std::fmt::Debug;

use super::serialization_limits::{LimitKind, LimitLadder, Limits};
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::*;

/// The sole declaration of the clean style attribute
/// `{property}:{text}!important`. Its value keeps the authored text, and its
/// first component keeps a parsed origin in that attribute starting at the
/// value.
#[track_caller]
pub fn parsed_important_declaration(property: &str, text: &str) -> CssDeclaration {
    let source = format!("{property}:{text}!important");
    let declaration = super::sole_declaration(&source);
    assert_eq!(
        declaration.importance(),
        CssImportance::Important,
        "{source}"
    );
    assert_eq!(
        declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        text,
        "{source}"
    );
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("parsed value origin: {source}")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        property.len() + 1,
        "{source}"
    );
    declaration
}

/// A named color that must exist.
pub fn named_color(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}

/// The images of the clean `background-image:{css}` style attribute.
#[track_caller]
pub fn background_images(css: &str) -> CssImageValueList {
    let source = super::sole_declaration(&format!("background-image:{css}"));
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed images")
    };
    value.value().clone()
}

/// The completed longhand expansion of `source`, each contribution keeping the
/// source occurrence and its importance.
#[track_caller]
pub fn expanded_longhands(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("completed expansion")
    };
    for value in values.items() {
        assert!(value.source().same_occurrence(source));
        assert_eq!(value.source().importance(), source.importance());
    }
    values
}

/// Typed evidence that checking `text` as `color` failed in the color
/// grammar, for an input whose violation names no single responsible token
/// (a truncated argument list or a calculation's resolved type).
#[track_caller]
pub fn assert_color_grammar_failure(
    result: Result<CssDeclaration, CssPropertyValueError>,
    text: &str,
) {
    match result {
        Ok(declaration) => panic!("{text}: admitted {declaration:?}"),
        Err(error) => assert!(
            error.code() == Some(CssErrorCode::InvalidColorSyntax),
            "{text}: not a color syntax rejection: {error:?}"
        ),
    }
}

/// The authored token that a color-valued `property` rejection names: an
/// `InvalidColorSyntax` error from the `<color>` grammar that `color` and each
/// color longhand use, or an `InvalidPropertyValue` error naming another
/// `property` whose own grammar rejected the value.
#[track_caller]
fn color_rejection_token<'a>(
    code: Option<CssErrorCode>,
    known_property: Option<CssKnownProperty>,
    encountered: Option<&'a CssTokenSummary>,
    property: CssKnownProperty,
    text: &str,
    error: &dyn std::fmt::Debug,
) -> &'a str {
    match code {
        Some(CssErrorCode::InvalidColorSyntax) => {}
        Some(CssErrorCode::InvalidPropertyValue) if property != CssKnownProperty::Color => {
            assert_eq!(known_property, Some(property), "{text}");
        }
        _ => panic!("{property:?}:{text}: not a color value rejection: {error:?}"),
    }
    encountered
        .unwrap_or_else(|| panic!("{property:?}:{text}: no responsible token: {error:?}"))
        .authored()
}

/// Typed evidence that checking the component values of `text` as the
/// color-valued `property` failed in its grammar, naming the authored token
/// `responsible`. A parsed responsible token anchors the error at its first
/// occurrence in `text`; a programmatic token has no source position to
/// compare.
#[track_caller]
pub fn assert_color_value_rejected(
    result: Result<CssDeclaration, CssPropertyValueError>,
    property: CssKnownProperty,
    text: &str,
    responsible: &str,
) {
    let at = text.find(responsible).expect("responsible token in text");
    assert_color_value_rejected_at(result, property, text, responsible, at);
}

/// [`assert_color_value_rejected`] whose responsible token starts at byte
/// `at` of `text`.
#[track_caller]
pub fn assert_color_value_rejected_at(
    result: Result<CssDeclaration, CssPropertyValueError>,
    property: CssKnownProperty,
    text: &str,
    responsible: &str,
    at: usize,
) {
    assert_eq!(&text[at..at + responsible.len()], responsible, "{text}");
    let error = match result {
        Ok(declaration) => panic!("{text}: admitted {declaration:?}"),
        Err(error) => error,
    };
    assert!(
        error.code().is_some(),
        "{text}: not a grammar rejection: {error:?}"
    );
    assert_eq!(
        color_rejection_token(
            error.code(),
            error.known_property(),
            error.encountered(),
            property,
            text,
            &error
        ),
        responsible,
        "{text}"
    );
    match error.origin() {
        CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) => {
            assert_eq!(origin.source().as_str(), text);
            assert_eq!(origin.span().start().byte_offset().value(), at, "{text}");
            assert_eq!(
                origin.span().end().byte_offset().value(),
                at + responsible.len(),
                "{text}"
            );
        }
        CssSerializedOrigin::Token(CssValueOrigin::Programmatic) => {}
        origin => panic!("{text}: error not anchored to the responsible token: {origin:?}"),
    }
}

/// Typed evidence that a text front rejected `text` as the color-valued
/// `property`: no declaration, and one diagnostic that names the authored
/// token `responsible` at its first offset in `text`.
#[track_caller]
pub fn assert_color_text_rejected(
    report: CssParseReport<Option<CssDeclaration>>,
    property: CssKnownProperty,
    text: &str,
    responsible: &str,
) {
    assert!(report.syntax().is_none(), "{text}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("{text}: one diagnostic: {report:?}")
    };
    let error = diagnostic.error();
    assert_eq!(
        color_rejection_token(
            Some(error.code()),
            error.known_property(),
            error.encountered(),
            property,
            text,
            error
        ),
        responsible,
        "{text}"
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        text.find(responsible).expect("responsible token in text"),
        "{text}"
    );
}

/// A bounded color serialization that fails with `kind`, with the same
/// result on a second attempt and without changing the color.
#[track_caller]
pub fn assert_color_limit_error(value: &CssColor, limits: Limits, kind: LimitKind) {
    let before = value.clone();
    let result = value.serialize_specified(limits);
    assert_eq!(result, value.serialize_specified(limits));
    assert_eq!(value, &before);
    assert_eq!(
        serialization_failure(&result.unwrap_err()),
        kind,
        "{limits:?}"
    );
}

/// The [`LimitLadder`] of `serialize(value, limits)`: `expected` from exactly
/// `input` and `projection` nodes, with no attempt changing `value`. Every
/// failure repeats identically, and each node rung also fails with its own
/// kind when the other two limits are unbounded.
#[track_caller]
pub fn assert_bounded_serialization<T>(
    value: &T,
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(&T, Limits) -> Result<String, CssSerializationError>,
) where
    T: Clone + Debug + PartialEq,
{
    LimitLadder::new(expected, input, projection).assert_unchanged_with(
        value,
        &serialize,
        |limits, kind, error| {
            assert_eq!(serialize(value, limits).as_ref(), Err(error), "{limits:?}");
            let isolated = match kind {
                LimitKind::InputNodeLimit => {
                    Limits::new(limits.max_input_nodes(), usize::MAX, usize::MAX)
                }
                LimitKind::ProjectionNodeLimit => {
                    Limits::new(usize::MAX, limits.max_projection_nodes(), usize::MAX)
                }
                _ => return,
            };
            assert_eq!(
                serialization_failure(&serialize(value, isolated).unwrap_err()),
                kind,
                "{isolated:?}"
            );
        },
    );
}

/// [`assert_bounded_serialization`] of a color's specified text.
#[track_caller]
pub fn assert_color_limits(value: &CssColor, expected: &str, input: usize, projection: usize) {
    assert_bounded_serialization(
        value,
        expected,
        input,
        projection,
        CssColor::serialize_specified,
    );
}

/// The rungs of a serialization of `value` that needs `bytes` of scratch
/// output, more than its final text `expected`, so its byte limit is not the
/// length of that text: each node limit one below `input` or `projection`
/// fails with its own kind while the other limits are unbounded, `bytes - 1`
/// fails with `ByteLimit` at the exact node limits, and the exact limits
/// serialize `expected`. Every attempt repeats identically and leaves `value`
/// unchanged.
#[track_caller]
pub fn assert_scratch_serialization<T>(
    value: &T,
    expected: &str,
    input: usize,
    projection: usize,
    bytes: usize,
    serialize: impl Fn(&T, Limits) -> Result<String, CssSerializationError>,
) where
    T: Clone + Debug + PartialEq,
{
    assert!(bytes > expected.len(), "scratch exceeds the final text");
    let before = value.clone();
    let attempt = |limits: Limits| {
        let result = serialize(value, limits);
        assert_eq!(result, serialize(value, limits), "{limits:?}");
        assert_eq!(value, &before, "{limits:?} changed the value");
        result
    };
    for (limits, kind) in [
        (
            Limits::new(input - 1, usize::MAX, usize::MAX),
            LimitKind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, projection - 1, usize::MAX),
            LimitKind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection, bytes - 1),
            LimitKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&attempt(limits).unwrap_err()),
            kind,
            "{limits:?}"
        );
    }
    assert_eq!(
        attempt(Limits::new(input, projection, bytes)).unwrap(),
        expected
    );
}

/// The default and `svg` namespace bindings used by namespaced selectors.
pub fn namespace_context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ])
}

/// The `(a, b)` coefficients of an An+B pattern.
pub fn nth_coefficients(pattern: CssNthPattern) -> (i32, i32) {
    match pattern {
        CssNthPattern::Odd => (2, 1),
        CssNthPattern::Even => (2, 0),
        CssNthPattern::Integer(b) => (0, b),
        CssNthPattern::AnPlusB(value) => (value.a(), value.b()),
        _ => panic!("unexpected nth pattern: {pattern:?}"),
    }
}

/// The compound that a selector must be.
#[track_caller]
pub fn originating_compound(value: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(value) = value else {
        panic!("originating compound")
    };
    value
}

/// Typed evidence that selector construction failed because the specified
/// output of the supplied graph failed with `kind`, and that the construction
/// error keeps that cause as its source.
#[track_caller]
pub fn assert_specified_construction_cause(error: &CssSelectorConstructionError, kind: LimitKind) {
    let Some(cause) = error.serialization_error() else {
        panic!("specified construction cause: {error:?}")
    };
    assert_eq!(serialization_failure(cause), kind, "{error:?}");
    let source = std::error::Error::source(error).expect("original specified source");
    assert_eq!(source.downcast_ref::<CssSerializationError>(), Some(cause));
}

/// Typed evidence that construction of an invalid selector graph failed
/// because its specified output is unrepresentable.
#[track_caller]
pub fn assert_unrepresentable_construction<T: Debug>(
    result: Result<T, CssSelectorConstructionError>,
) {
    match result {
        Ok(value) => panic!("invalid selector graph admitted: {value:?}"),
        Err(error) => assert_specified_construction_cause(&error, LimitKind::UnrepresentableValue),
    }
}

/// Typed evidence that a strict list factory rejected an empty vector before
/// any output admission.
#[track_caller]
pub fn assert_empty_list_construction<T: Debug>(result: Result<T, CssSelectorConstructionError>) {
    let error = result.expect_err("empty strict list admitted");
    assert!(error.is_empty_list(), "{error:?}");
    assert!(std::error::Error::source(&error).is_none());
}

/// Typed evidence that `parse_selector` rejected the whole `source` in the
/// default namespace context: no selector, one `InvalidSelector` diagnostic
/// that rejects the input and spans all of it, and strict validation that
/// fails with the same diagnostic.
#[track_caller]
pub fn assert_selector_rejected(source: &str) {
    assert_selector_rejected_in(source, &CssNamespaceContext::default());
}

/// [`assert_selector_rejected`] in a namespace context.
#[track_caller]
pub fn assert_selector_rejected_in(source: &str, context: &CssNamespaceContext) {
    assert_selector_report_rejected(
        source,
        CssSelector::parse(&CssParser::default(), source, context),
    );
}

/// Typed evidence that a selector fragment `report` of `source` rejected the
/// whole input: no syntax, one `InvalidSelector` diagnostic that rejects the
/// input and spans all of it, and strict validation that fails with the same
/// diagnostic.
#[track_caller]
pub fn assert_selector_report_rejected<T: Debug>(source: &str, report: CssParseReport<Option<T>>) {
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete rejection: {source}: {report:?}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidSelector,
        "{source}"
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RejectInput,
        "{source}"
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        0,
        "{source}"
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.len(),
        "{source}"
    );
    let diagnostics = report.diagnostics().to_vec();
    let failure = report
        .into_validation_result()
        .expect_err("strict validation rejects the selector");
    assert_eq!(failure.diagnostics(), diagnostics, "{source}");
}

/// Typed evidence that forgiving selector recovery in `source` dropped exactly
/// the list member `item`: one `InvalidSelector` diagnostic that drops a list
/// item and spans the first occurrence of `item`, and strict validation that
/// fails with the same diagnostic.
#[track_caller]
pub fn assert_selector_item_dropped<T: Debug>(source: &str, report: CssParseReport<T>, item: &str) {
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped member: {source}: {report:?}")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropSelectorListItem,
        "{source}"
    );
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidSelector,
        "{source}"
    );
    let start = source.find(item).expect("dropped member in source");
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        start,
        "{source}"
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + item.len(),
        "{source}"
    );
    assert_strict_validation_rejects(report);
}

/// Strict validation of a recovered `report` fails with exactly the
/// diagnostics that recovery reported.
#[track_caller]
pub fn assert_strict_validation_rejects<T: Debug>(report: CssParseReport<T>) {
    let diagnostics = report.diagnostics().to_vec();
    assert!(!diagnostics.is_empty(), "recovery reported no diagnostic");
    let failure = report
        .into_validation_result()
        .expect_err("strict validation rejects recovered input");
    assert_eq!(failure.diagnostics(), diagnostics);
}

/// Typed evidence that a selector fragment `report` of `source` ended by
/// rejecting all of it: no selector syntax survives, the final diagnostic
/// rejects the whole input as an invalid selector, and strict validation
/// fails with the reported diagnostics.
#[track_caller]
pub fn assert_selector_input_rejected<T: Debug>(source: &str, report: CssParseReport<Option<T>>) {
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    let rejection = report.diagnostics().last().expect("a rejection");
    assert_eq!(
        rejection.action(),
        CssRecoveryAction::RejectInput,
        "{source:?}"
    );
    assert_eq!(
        rejection.error().code(),
        CssErrorCode::InvalidSelector,
        "{source:?}"
    );
    assert_eq!(
        rejection.span().start().byte_offset().value(),
        0,
        "{source:?}"
    );
    assert_eq!(
        rejection.span().end().byte_offset().value(),
        source.len(),
        "{source:?}"
    );
    assert_strict_validation_rejects(report);
}
