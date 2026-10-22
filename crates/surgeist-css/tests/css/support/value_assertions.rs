//! Typed rejection evidence and value extractors shared by the numeric,
//! values and CSSOM suites.
//!
//! Each rejection helper names the error kind, the rejecting property or
//! code, and the responsible token or offset, so a test cannot pass on an
//! unrelated earlier failure. Expected kinds and tokens stay with the
//! calling tests, which derive them from the grammar they exercise.

use std::fmt::Debug;

use surgeist_css::*;

/// Asserts that checked numeric construction rejected its input with `kind`
/// at `origin` (the responsible component's origin, or `None` when the
/// failure has no responsible component).
#[track_caller]
pub fn assert_numeric_rejected<T: Debug>(
    result: Result<T, CssNumericConstructionError>,
    kind: CssNumericConstructionErrorKind,
    origin: Option<&CssValueOrigin>,
) {
    let error = result.expect_err("checked numeric construction must reject");
    assert_eq!(error.kind(), &kind, "{error:?}");
    assert_eq!(error.origin(), origin, "{error:?}");
}

/// Asserts that checked numeric construction rejected its input with `kind`.
#[track_caller]
pub fn assert_numeric_kind<T: Debug>(
    result: Result<T, CssNumericConstructionError>,
    kind: CssNumericConstructionErrorKind,
) {
    let error = result.expect_err("checked numeric construction must reject");
    assert_eq!(error.kind(), &kind, "{error:?}");
}

/// Asserts that checked numeric construction exceeded a resource limit, and
/// that the exceeded component-value limit is `limit`.
#[track_caller]
pub fn assert_numeric_resource<T: Debug>(
    result: Result<T, CssNumericConstructionError>,
    limit: CssComponentValueErrorKind,
) {
    let error = result.expect_err("checked numeric construction must exceed a limit");
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::ResourceLimit,
        "{error:?}"
    );
    let source = std::error::Error::source(&error)
        .and_then(|source| source.downcast_ref::<CssComponentValueError>())
        .unwrap_or_else(|| panic!("component limit source: {error:?}"));
    assert_eq!(source.kind(), limit, "{error:?}");
}

/// Asserts that `construct` rejects the single `component` with `kind` at
/// that component's origin.
#[track_caller]
pub fn assert_component_rejected<T: Debug>(
    construct: fn(CssComponentValue) -> Result<T, CssNumericConstructionError>,
    component: CssComponentValue,
    kind: CssNumericConstructionErrorKind,
) {
    let origin = component.origin().clone();
    assert_numeric_rejected(construct(component), kind, Some(&origin));
}

/// Asserts that the calculation grammar or type checker rejected the input:
/// with `kind` when the violated rule is unambiguous, else with one of the
/// grammar or typing kinds (an empty argument included) rather than a
/// recovered, component or resource failure.
#[track_caller]
pub fn assert_calculation_rejected<T: Debug>(
    result: Result<T, CssNumericConstructionError>,
    kind: Option<CssNumericConstructionErrorKind>,
) {
    use CssNumericConstructionErrorKind as K;
    let error = result.expect_err("calculation construction must reject");
    match kind {
        Some(kind) => assert_eq!(error.kind(), &kind, "{error:?}"),
        None => assert!(
            matches!(
                error.kind(),
                K::EmptyValue
                    | K::MalformedExpression
                    | K::UnknownFunction
                    | K::Arity
                    | K::InvalidArgumentType
                    | K::IncompatibleTypes
                    | K::RootDomainMismatch
            ),
            "{error:?}"
        ),
    }
}

/// The property-grammar detail of a checked property-value rejection.
#[track_caller]
pub fn property_grammar_error(error: &CssPropertyValueParseError) -> &CssPropertyValueError {
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("property grammar rejection: {error:?}")
    };
    detail
}

/// The grammar detail of a checked property value that `property`'s grammar
/// rejected.
#[track_caller]
pub fn property_rejection(
    result: Result<CssDeclaration, CssPropertyValueParseError>,
    property: CssKnownProperty,
) -> CssPropertyValueError {
    let error = result.expect_err("checked property value must reject");
    let detail = property_grammar_error(&error);
    assert_eq!(detail.property(), property, "{error:?}");
    detail.clone()
}

/// Asserts that checking a property value failed in `property`'s grammar at
/// the authored token `encountered` (`None` when the value ended first).
#[track_caller]
pub fn assert_property_rejected(
    result: Result<CssDeclaration, CssPropertyValueParseError>,
    property: CssKnownProperty,
    encountered: Option<&str>,
) {
    let detail = property_rejection(result, property);
    assert_eq!(
        detail.encountered().map(CssTokenSummary::authored),
        encountered,
        "{detail:?}"
    );
}

/// The grammar detail of a pending declaration's rejected replacement, which
/// `property`'s grammar rejected.
#[track_caller]
pub fn replacement_rejection(
    result: Result<CssContributions, CssExpansionError>,
    property: CssKnownProperty,
) -> CssPropertyValueError {
    let error = result.expect_err("reentry must reject the replacement");
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        panic!("invalid replacement: {error:?}")
    };
    let detail = property_grammar_error(error);
    assert_eq!(detail.property(), property, "{error:?}");
    detail.clone()
}

/// Asserts that a report or validation failure has a first diagnostic, and
/// that it has `code`.
#[track_caller]
pub fn assert_first_code(diagnostics: &[CssRecoveryDiagnostic], code: CssErrorCode) {
    let first = diagnostics.first().expect("a diagnostic");
    assert_eq!(first.error().code(), code, "{first:?}");
}

/// Asserts that the first diagnostic has `code` and that its responsible
/// position is the first occurrence of `responsible` in `source`.
#[track_caller]
pub fn assert_first_diagnostic(
    diagnostics: &[CssRecoveryDiagnostic],
    source: &str,
    code: CssErrorCode,
    responsible: &str,
) {
    let first = diagnostics
        .first()
        .unwrap_or_else(|| panic!("a diagnostic: {source}"));
    assert_eq!(first.error().code(), code, "{source}: {first:?}");
    let offset = source
        .find(responsible)
        .unwrap_or_else(|| panic!("`{responsible}` in {source}"));
    assert_eq!(
        first.error().position().byte_offset().value(),
        offset,
        "{source}: responsible `{responsible}`: {first:?}"
    );
}

/// [`assert_first_diagnostic`] for an invalid value of `property` whose
/// responsible token is the first occurrence of `responsible` in `source`.
#[track_caller]
pub fn assert_first_property_diagnostic(
    diagnostics: &[CssRecoveryDiagnostic],
    source: &str,
    property: CssKnownProperty,
    responsible: &str,
) {
    assert_first_diagnostic(
        diagnostics,
        source,
        CssErrorCode::InvalidPropertyValue,
        responsible,
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostics[0].error().kind() else {
        unreachable!("checked by code")
    };
    assert_eq!(detail.property(), property, "{source}");
}

/// Asserts that a CSSOM rule projection failed atomically on the `kind`
/// resource budget.
#[track_caller]
pub fn assert_rule_resource<T: Debug>(
    result: Result<T, CssRuleCssomSerializationError>,
    kind: CssSpecifiedValueSerializationErrorKind,
) {
    let error = result.expect_err("rule projection must exceed its budget");
    assert_eq!(
        error.kind(),
        CssRuleCssomSerializationErrorKind::Resource(kind),
        "{error:?}"
    );
}

/// Asserts that the first diagnostic reports an invalid value of `property`.
#[track_caller]
pub fn assert_first_property_rejection(
    diagnostics: &[CssRecoveryDiagnostic],
    property: CssKnownProperty,
) {
    let first = diagnostics.first().expect("a diagnostic");
    let ErrorKind::InvalidPropertyValue(detail) = first.error().kind() else {
        panic!("invalid property value: {first:?}")
    };
    assert_eq!(detail.property(), property, "{first:?}");
}

/// Asserts that `origin` is parsed from `source` and spans `start..end`.
#[track_caller]
pub fn assert_parsed_span(origin: &CssValueOrigin, source: &str, start: usize, end: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("an original parsed origin: {origin:?}");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

/// The first non-whitespace component of input that must tokenize.
#[track_caller]
pub fn first_token(text: &str) -> CssComponentValue {
    parse_component_values(text)
        .unwrap()
        .items()
        .iter()
        .find(|component| {
            !matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            )
        })
        .unwrap()
        .clone()
}

/// A specified number built from a checked number calculation.
#[track_caller]
pub fn calculated_number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

/// A specified length built from a checked length calculation.
#[track_caller]
pub fn calculated_length(source: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(parse_component_values(source).unwrap()).unwrap(),
    )
    .unwrap()
}

/// Asserts that [`calculated_number`] keeps `source` as its components and
/// serializes to `expected` without changing the value.
#[track_caller]
pub fn assert_calculated_number(source: &str, expected: &str) {
    let value = calculated_number(source);
    let before = value.clone();
    let calculation = value.calculation().unwrap();
    let components = calculation.components().clone();
    let ty = calculation.numeric_type();
    assert_eq!(components.serialize().unwrap().as_css(), source);
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), before.origin());
    assert_eq!(value.calculation().unwrap().components(), &components);
    assert_eq!(value.calculation().unwrap().numeric_type(), ty);
    assert_eq!(result.unwrap(), expected, "{source}");
}

/// Whether `component` is the dimension token `representation` `unit`.
pub fn exact_dimension(
    component: Option<&CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(
        component.map(CssComponentValue::view),
        Some(CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }))
            if number.representation() == representation && unit == expected_unit
    )
}

/// Whether `component` is the percentage token `representation`%.
pub fn exact_percentage(component: Option<&CssComponentValue>, representation: &str) -> bool {
    matches!(
        component.map(CssComponentValue::view),
        Some(CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)))
            if number.representation() == representation
    )
}

/// The filter of a `filter` or `backdrop-filter` declaration.
#[track_caller]
pub fn filter_value(source: &CssDeclaration) -> &CssFilter {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Filter(value) => value.value(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
        _ => panic!("filter"),
    }
}

/// The text a URL consumer property accepts for one URL: `cursor` requires
/// its trailing keyword fallback.
pub fn url_consumer_text(property: CssKnownProperty, url: &str) -> String {
    if property == CssKnownProperty::Cursor {
        format!("{url}, auto")
    } else {
        url.to_owned()
    }
}

/// The specified value a declaration block reports for a known property.
#[track_caller]
pub fn specified_property_value(
    block: &CssSpecifiedDeclarationBlock,
    property: CssKnownProperty,
) -> Option<String> {
    block
        .property_value(CssPropertyNameRef::Known(property))
        .unwrap()
}

/// The one rule of a clean default-namespace rule parse.
#[track_caller]
pub fn clean_rule(source: &str) -> CssRule {
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().expect("one admitted rule")
}

/// The one media query of a clean CSSOM media-query parse.
#[track_caller]
pub fn clean_media_query(source: &str) -> CssMediaQuery {
    let report = parse_cssom_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.into_parts().0.expect("one clean media query")
}

/// The resource-limit kind of a media CSSOM serialization failure.
#[track_caller]
pub fn media_resource_kind(
    error: &CssMediaCssomSerializationError,
) -> CssSpecifiedValueSerializationErrorKind {
    let CssMediaCssomSerializationError::Resource { error, .. } = error else {
        panic!("expected resource failure: {error:?}");
    };
    error.kind()
}

/// The sole declaration of the clean style attribute `css`, which the
/// validator accepts with the same declarations.
#[track_caller]
pub fn validated_sole_declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(css),
        Ok(report.syntax().clone()),
        "{css}"
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {css}: {:?}", report.syntax())
    };
    declaration.clone()
}

/// [`validated_sole_declaration`] of `property:value`, written with
/// `importance`, that keeps the named known property and importance.
#[track_caller]
pub fn validated_declaration(
    property: CssKnownProperty,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    let priority = match importance {
        CssImportance::Important => "!important",
        CssImportance::Normal => "",
    };
    let source =
        validated_sole_declaration(&format!("{}:{value}{priority}", property.canonical_name()));
    assert_eq!(source.known().unwrap().property(), property);
    assert_eq!(source.importance(), importance);
    source
}

/// The one URL image of a clean `background-image:text` style attribute.
#[track_caller]
pub fn background_image_url(text: &str) -> CssUrl {
    let source = super::sole_declaration(&format!("background-image:{text}"));
    let CssKnownPropertyValueRef::BackgroundImage(images) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary background image: {text}")
    };
    let [CssImageValue::Url(url)] = images.images().images() else {
        panic!("one URL: {text}")
    };
    url.clone()
}
