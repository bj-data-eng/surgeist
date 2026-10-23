//! Name- and property-keyed forms of the shared admission, rejection and
//! expansion helpers, for property-family suites whose cases are keyed by a
//! canonical name or by one fixed known property.
//!
//! Each helper delegates to the single body in [`super`] or
//! [`super::authored_property`] and only adapts the key, so a suite keeps the
//! checks of the shared helper instead of a local copy of them.

use super::authored_property::{self, ParserFront};
use super::{declaration, known_property, selected_grammar};
use surgeist_css::*;

/// The sole clean style-attribute declaration of `property`, written with
/// `importance`, that keeps the property identity.
#[track_caller]
pub fn property_declaration(
    property: CssKnownProperty,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    let source = declaration(property.canonical_name(), value, importance);
    assert_eq!(source.known().unwrap().property(), property, "{value}");
    assert_eq!(source.importance(), importance, "{value}");
    source
}

/// [`property_declaration`] with `!important`.
#[track_caller]
pub fn important_property_declaration(property: CssKnownProperty, value: &str) -> CssDeclaration {
    property_declaration(property, value, CssImportance::Important)
}

/// [`property_declaration`] with normal importance.
#[track_caller]
pub fn normal_property_declaration(property: CssKnownProperty, value: &str) -> CssDeclaration {
    property_declaration(property, value, CssImportance::Normal)
}

/// [`super::validated_important_declaration`] of `property`, which keeps the
/// property identity.
#[track_caller]
pub fn validated_important_property_declaration(
    property: CssKnownProperty,
    value: &str,
) -> CssDeclaration {
    let source = super::validated_important_declaration(property.canonical_name(), value);
    assert_eq!(source.known().unwrap().property(), property, "{value}");
    source
}

/// Normal-importance component values checked by the grammar of the
/// property named `name`.
pub fn normal_grammar_components(
    name: &str,
    components: CssComponentValues,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    parse_property_value_for_grammar(selected_grammar(name), components, CssImportance::Normal)
}

/// The important text front of `property`, by name or, when `grammar` is
/// set, by its grammar.
#[track_caller]
pub fn property_text_front(
    property: CssKnownProperty,
    text: &str,
    grammar: bool,
) -> CssDeclaration {
    let front = if grammar {
        ParserFront::TextGrammar
    } else {
        ParserFront::TextName
    };
    front.parse(property, text)
}

/// [`property_text_front`] of the property named `name`.
#[track_caller]
pub fn named_text_front(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    property_text_front(known_property(name), text, grammar)
}

/// Every authored admission front of the property named `name`.
#[track_caller]
pub fn named_fronts(name: &str, text: &str) -> [CssDeclaration; 5] {
    authored_property::fronts(known_property(name), text)
}

/// The property named `name` admits `value` as an important style attribute
/// and through its checked grammar, keeping the target property, importance
/// and authored component text.
#[track_caller]
pub fn accepted_by_attribute_and_grammar(name: &str, value: &str) {
    for source in [
        super::important_declaration(name, value),
        super::checked_grammar_declaration(name, value),
    ] {
        assert_eq!(
            source.known().unwrap().property(),
            selected_grammar(name).target_property()
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            value
        );
    }
}

/// [`authored_property::invalid`] for the property named `name`.
#[track_caller]
pub fn invalid_named_value(name: &str, value: &str) {
    authored_property::invalid(known_property(name), value);
}

/// `error` is the grammar of `property` rejecting its value.
#[track_caller]
pub fn assert_property_grammar_error(
    error: &CssPropertyValueParseError,
    property: CssKnownProperty,
) {
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("{property:?}: expected a property grammar rejection, got {error:?}")
    };
    assert_eq!(detail.property(), property, "{error:?}");
}

/// Byte offset in `value` of the token that first violates the grammar,
/// given `rest`, the authored input that remains from that token; an empty
/// `rest` names the end of the value.
#[track_caller]
pub fn responsible_offset(value: &str, rest: &str) -> usize {
    assert!(
        value.ends_with(rest),
        "{value:?}: responsible input {rest:?} is not a suffix"
    );
    value.len() - rest.len()
}

// The located helpers below take the responsible input as
// `impl Into<Option<&str>>`: a `&str` is the input that remains from the
// responsible token, and `None` asserts the rejection kind and property
// without a location, for the few values whose implemented location differs
// from the grammar-derived one (each such value is reported for review).

/// Checking `value` as `property`, by name and by grammar and at either
/// importance, is a grammar rejection of `property` located at the token
/// from which `rest` remains (an empty `rest` locates the end of input).
#[track_caller]
pub fn assert_checked_value_rejected_at<'a>(
    property: CssKnownProperty,
    value: &str,
    rest: impl Into<Option<&'a str>>,
) {
    let rest = rest.into();
    let components = parse_component_values(value).unwrap();
    let serialized = components.serialize_tokens(Default::default()).unwrap();
    assert_eq!(serialized.as_css(), value, "authored component text");
    let expected = rest.map(|rest| {
        serialized
            .origin_at(responsible_offset(value, rest))
            .unwrap()
    });
    for importance in [CssImportance::Normal, CssImportance::Important] {
        for grammar in [false, true] {
            let error = authored_property::checked_components(
                property,
                components.clone(),
                grammar,
                importance,
            )
            .unwrap_err();
            assert_property_grammar_error(&error, property);
            if let Some(expected) = expected {
                assert_eq!(
                    error.origin(),
                    expected,
                    "{property:?}: {value:?} (grammar: {grammar}): responsible token {rest:?}"
                );
            }
        }
    }
}

/// A style-attribute diagnostic of `property`, for a value authored at byte
/// `value_start`, is a property grammar failure located at the token of
/// `value` from which `rest` remains.
#[track_caller]
pub fn assert_value_diagnostic_at<'a>(
    error: &Error,
    property: CssKnownProperty,
    value_start: usize,
    value: &str,
    rest: impl Into<Option<&'a str>>,
) {
    let ErrorKind::InvalidPropertyValue(detail) = error.kind() else {
        panic!("{value:?}: expected a property grammar diagnostic, got {error:?}")
    };
    assert_eq!(detail.property(), property, "{value:?}");
    if let Some(rest) = rest.into() {
        assert_eq!(
            error.position().byte_offset().value(),
            value_start + responsible_offset(value, rest),
            "{value:?}: responsible token {rest:?}"
        );
    }
}

/// `origin` locates the token of `text` from which `rest` remains, or the
/// end of `text` when `rest` is empty.
#[track_caller]
pub fn assert_origin_at(origin: &CssSerializedOrigin, text: &str, rest: &str) {
    let offset = responsible_offset(text, rest);
    if rest.is_empty() {
        assert!(
            matches!(origin, CssSerializedOrigin::End(_)),
            "{text:?}: expected the end of input, got {origin:?}"
        );
        return;
    }
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(token)) = origin else {
        panic!("{text:?}: expected the parsed token {rest:?}, got {origin:?}")
    };
    assert_eq!(token.source().as_str(), text);
    assert_eq!(
        token.span().start().byte_offset().value(),
        offset,
        "{text:?}: responsible token {rest:?}"
    );
}

/// `result` is a grammar rejection of `property` whose responsible token is
/// programmatic, so it carries no authored position.
#[track_caller]
pub fn assert_programmatic_rejection(
    result: Result<CssDeclaration, CssPropertyValueParseError>,
    property: CssKnownProperty,
) {
    let error = result.unwrap_err();
    assert_property_grammar_error(&error, property);
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

/// Strict reentry rejected the replacement `text` as an invalid value of
/// `property` located at the token from which `rest` remains.
#[track_caller]
pub fn assert_invalid_replacement_at<'a>(
    error: &CssExpansionError,
    property: CssKnownProperty,
    text: &str,
    rest: impl Into<Option<&'a str>>,
) {
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        panic!("{text:?}: expected an invalid replacement, got {error:?}")
    };
    assert_property_grammar_error(error, property);
    if let Some(rest) = rest.into() {
        assert_origin_at(error.origin(), text, rest);
    }
}

/// Checked construction of the recovered `text`, whose function or block
/// was closed only by the end of input, fails at that original implicit
/// closure.
#[track_caller]
pub fn assert_implicit_closure_rejection(error: &CssPropertyValueParseError, text: &str) {
    assert!(
        matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ),
        "{text:?}: expected an unexpected end, got {error:?}"
    );
    let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
        error.origin()
    else {
        panic!("{text:?}: expected the original implicit closure, got {error:?}")
    };
    assert_eq!(opening.source().as_str(), text);
    assert_eq!(at.span().start().byte_offset().value(), text.len());
}

/// The style attribute `color:red;{name}:{value};color:blue`, with `name`
/// spelled as authored, drops only the declaration with one grammar
/// diagnostic of `property` located at the token from which `rest` remains.
#[track_caller]
fn assert_attribute_rejected_at(
    name: &str,
    property: CssKnownProperty,
    value: &str,
    rest: Option<&str>,
) {
    let prefix = format!("color:red;{name}:");
    let css = format!("{prefix}{value};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|item| item.known().unwrap().property())
            .collect::<Vec<_>>(),
        [CssKnownProperty::Color, CssKnownProperty::Color],
        "{css}: retained siblings"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("{css}: expected one diagnostic")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{css}"
    );
    assert_value_diagnostic_at(diagnostic.error(), property, prefix.len(), value, rest);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

/// [`authored_property::invalid`] that also locates the responsible token:
/// the token from which `rest` remains, or the end of the value when `rest`
/// is empty, in both the style-attribute diagnostic and checked construction.
#[track_caller]
pub fn assert_value_rejected_at<'a>(
    property: CssKnownProperty,
    value: &str,
    rest: impl Into<Option<&'a str>>,
) {
    let rest = rest.into();
    authored_property::invalid(property, value);
    assert_checked_value_rejected_at(property, value, rest);
    assert_attribute_rejected_at(property.canonical_name(), property, value, rest);
}

/// [`assert_value_rejected_at`] for the property named `name`, whose style
/// attribute is also authored with that spelling.
#[track_caller]
pub fn invalid_named_value_at<'a>(name: &str, value: &str, rest: impl Into<Option<&'a str>>) {
    let rest = rest.into();
    let property = known_property(name);
    assert_value_rejected_at(property, value, rest);
    assert_attribute_rejected_at(name, property, value, rest);
}

/// The longhand contributions of a completed expansion of `source`.
#[track_caller]
pub fn longhand_contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap_or_else(|error| panic!("expansion: {error:?}"))
    else {
        panic!("completed longhand contributions: {source:?}")
    };
    values
}

/// The completed contributions of `source`: exactly one, for the property
/// of `source`, which keeps its authored source.
#[track_caller]
pub fn sole_completed_contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let values = longhand_contributions(source);
    assert_eq!(values.items().len(), 1);
    let item = &values.items()[0];
    assert_eq!(item.property(), source.known().unwrap().property());
    authored_property::assert_source(item, source);
    values
}

/// Every authored front of the property named `name` admits `input` as one
/// ordinary, unsubstituted terminal whose specified form is `canonical`, and
/// the checked grammar admits `canonical` itself.
#[track_caller]
pub fn accepted_through_every_front(name: &str, input: &str, canonical: &str) {
    let expected = format!("{name}: {canonical} !important;");
    for source in named_fronts(name, input) {
        let before = source.clone();
        assert_eq!(
            source.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert!(source.known().unwrap().property_value().is_some());
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        let values = sole_completed_contributions(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            known_property(name)
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(source, before);
    }
    assert_eq!(
        super::authored_checked_declaration(name, canonical, true)
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
}

/// The one longhand contribution of `source`, which keeps the occurrence,
/// importance and property of its source.
#[track_caller]
pub fn sole_longhand_contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let values = longhand_contributions(source);
    let [item] = values.items() else {
        panic!("exactly one terminal contribution: {source:?}")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    item.clone()
}
