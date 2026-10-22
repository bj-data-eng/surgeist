//! Authored-syntax helpers for the syntax, supports, container, media and
//! serialization modules: parsed-origin and source-position checks, checked
//! construction from source text, declaration projections, block-fragment
//! reports and typed rejection evidence.

use std::fmt::Debug;
use std::ops::Range;

use surgeist_css::*;

/// The parsed origin of an authored occurrence.
#[track_caller]
pub fn parsed_origin(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(parsed) = origin else {
        panic!("parsed origin: {origin:?}")
    };
    parsed
}

/// Asserts that two origins are parsed and share one source snapshot and span.
#[track_caller]
pub fn assert_same_parsed_origin(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (actual, expected) = (parsed_origin(actual), parsed_origin(expected));
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

/// Asserts a source position's UTF-8 byte offset, zero-based line and
/// zero-based UTF-16 column.
#[track_caller]
pub fn assert_position(actual: CssSourcePosition, byte: usize, line: u32, column: u32) {
    assert_eq!(actual.byte_offset().value(), byte, "original UTF-8 byte");
    assert_eq!(actual.line().value(), line, "zero-based line");
    assert_eq!(actual.column().value(), column, "zero-based UTF-16 column");
}

/// [`assert_position`] with the expected byte, line and column as one tuple.
#[track_caller]
pub fn assert_position_at(actual: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_position(actual, expected.0, expected.1, expected.2);
}

/// Asserts a parsed origin's source text and the coordinates of both span
/// ends.
#[track_caller]
pub fn assert_origin_region(
    actual: &CssParsedOrigin,
    source: &str,
    start: (usize, u32, u32),
    end: (usize, u32, u32),
) {
    assert_eq!(actual.source().as_str(), source);
    assert_position_at(actual.span().start(), start);
    assert_position_at(actual.span().end(), end);
}

/// Asserts a parsed origin's source text and byte span.
#[track_caller]
pub fn assert_origin_bytes(actual: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(actual.source().as_str(), source);
    assert_eq!(actual.span().start().byte_offset().value(), start);
    assert_eq!(actual.span().end().byte_offset().value(), end);
}

/// Asserts a span's byte range.
#[track_caller]
pub fn assert_span_bytes(span: CssSourceSpan, expected: Range<usize>) {
    assert_eq!(span.start().byte_offset().value(), expected.start);
    assert_eq!(span.end().byte_offset().value(), expected.end);
}

/// One programmatic token component.
#[track_caller]
pub fn component_token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).unwrap()
}

/// Supplied components as one checked sequence.
#[track_caller]
pub fn component_list(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items).unwrap()
}

/// A container condition checked from the components of `source`.
#[track_caller]
pub fn checked_container_condition(source: &str) -> CssContainerCondition {
    CssContainerCondition::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

/// A supports condition checked from the components of `source` with the
/// default namespace context.
#[track_caller]
pub fn checked_supports_condition(source: &str) -> CssSupportsCondition {
    CssSupportsCondition::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

/// A supports declaration checked from the components of `source`.
#[track_caller]
pub fn checked_supports_declaration(source: &str) -> CssSupportsDeclaration {
    CssSupportsDeclaration::try_from_components(parse_component_values(source).unwrap()).unwrap()
}

/// An import rule checked from the components of `source` with the default
/// namespace context.
#[track_caller]
pub fn checked_import_rule(source: &str) -> CssImportRule {
    CssImportRule::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

/// The declaration of a declaration-kind supports condition.
#[track_caller]
pub fn supports_declaration(condition: &CssSupportsCondition) -> &CssSupportsDeclaration {
    let CssSupportsConditionKind::Declaration(value) = condition.kind() else {
        panic!("declaration condition: {condition:?}")
    };
    value
}

/// A `container-name` value of decoded names.
#[track_caller]
pub fn container_names(values: &[&str]) -> CssContainerNames {
    CssContainerNames::Names(
        CssContainerNameList::try_new(
            values
                .iter()
                .map(|name| CssContainerName::try_from_decoded(*name).unwrap())
                .collect(),
        )
        .unwrap(),
    )
}

/// The canonical or custom property names of a declaration list, in order.
pub fn property_names(declarations: &CssDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("unexpected property-name kind"),
        })
        .collect()
}

/// The known properties of a declaration list, in order.
#[track_caller]
pub fn known_properties(declarations: &CssDeclarationList) -> Vec<CssKnownProperty> {
    declarations
        .iter()
        .map(|declaration| declaration.known().unwrap().property())
        .collect()
}

/// The declarations of a normalized sheet's top-level items, in order.
pub fn normalized_declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

/// The declaration retained by a clean `parse_declaration` of `source`.
#[track_caller]
pub fn parsed_declaration(source: &str) -> CssDeclaration {
    let report = parse_declaration(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report
        .syntax()
        .as_ref()
        .expect("retained declaration")
        .clone()
}

/// The clean `name:value!important` declaration, whose known property has
/// the canonical name `name`.
#[track_caller]
pub fn important_known_declaration(name: &str, value: &str) -> CssDeclaration {
    let declaration = crate::css::support::important_declaration(name, value);
    assert_eq!(
        declaration.known().unwrap().property().canonical_name(),
        name
    );
    assert_eq!(declaration.importance(), CssImportance::Important);
    declaration
}

/// A declaration checked from the components of `value` against the grammar
/// selected by `name`, with `importance`.
#[track_caller]
pub fn grammar_checked_declaration(
    name: &str,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).expect("known grammar"),
        parse_component_values(value).unwrap(),
        importance,
    )
    .unwrap()
}

/// The genuine body retained by a block-fragment report.
#[track_caller]
pub fn fragment_body<T>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
) -> &CssBlockFragment<T> {
    report.syntax().as_ref().expect("retained genuine body")
}

/// The diagnostic carrying `action` in an unclean block-fragment report whose
/// strict validation fails with the same diagnostics.
#[track_caller]
pub fn fragment_recovery<T: Clone + Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    action: CssRecoveryAction,
) -> &CssRecoveryDiagnostic {
    assert!(!report.is_clean());
    assert_strict_rejection(
        report.clone().into_validation_result(),
        report.diagnostics(),
    );
    report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.action() == action)
        .expect("the provider's owned recovery action")
}

/// Asserts that a block-fragment report rejected the complete `source`.
#[track_caller]
pub fn assert_fragment_rejected<T: Clone + Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    assert!(
        report.syntax().is_none(),
        "complete fragment rejected: {source}"
    );
    let diagnostic = fragment_recovery(report, CssRecoveryAction::RejectInput);
    assert_span_bytes(diagnostic.span(), 0..source.len());
}

/// Asserts that a block-fragment report retained the whole `source` and
/// closed it implicitly at EOF.
#[track_caller]
pub fn assert_fragment_implicitly_closed<T: Clone + Debug>(
    report: &CssParseReport<Option<CssBlockFragment<T>>>,
    source: &str,
) {
    assert_origin_bytes(fragment_body(report).origin(), source, 0, source.len());
    let diagnostic = fragment_recovery(report, CssRecoveryAction::RetainWithImplicitClosure);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

/// Asserts that strict validation failed with exactly the recovered
/// `diagnostics`, and that there is at least one.
#[track_caller]
pub fn assert_strict_rejection<T: Debug>(
    validation: Result<T, CssValidationFailure>,
    diagnostics: &[CssRecoveryDiagnostic],
) {
    assert!(!diagnostics.is_empty(), "a rejection needs a diagnostic");
    assert_eq!(validation.unwrap_err().diagnostics(), diagnostics);
}

/// Asserts that an unclean sheet report's diagnostics are exactly those of
/// strict validation, through both the validator and the report.
#[track_caller]
pub fn assert_unclean_sheet(source: &str, report: &CssParseReport<CssSheet>) {
    assert!(!report.is_clean());
    assert_strict_rejection(validate_sheet(source), report.diagnostics());
    assert_strict_rejection(
        report.clone().into_validation_result(),
        report.diagnostics(),
    );
}

/// Asserts that `diagnostics` are exactly `expected`, in order, each named by
/// its error code, recovery action and the byte offset of the responsible
/// authored token.
#[track_caller]
pub fn assert_diagnostics(
    diagnostics: &[CssRecoveryDiagnostic],
    expected: &[(CssErrorCode, CssRecoveryAction, usize)],
) {
    let actual: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.error().code(),
                diagnostic.action(),
                diagnostic.error().position().byte_offset().value(),
            )
        })
        .collect();
    assert_eq!(actual, expected, "{diagnostics:?}");
}

/// Asserts that `kind` reports an invalid value of `property` whose grammar
/// first failed at the authored token `token`, or at EOF when it is `None`.
#[track_caller]
pub fn assert_invalid_property_value(
    kind: &ErrorKind,
    property: CssKnownProperty,
    token: Option<&str>,
) {
    let ErrorKind::InvalidPropertyValue(detail) = kind else {
        panic!("invalid property value: {kind:?}")
    };
    assert_eq!(detail.property(), property, "{kind:?}");
    assert_eq!(
        detail.encountered().map(|token| token.authored()),
        token,
        "{kind:?}"
    );
}

/// The byte offset of the first occurrence of `token` in `source`.
#[track_caller]
pub fn offset_of(source: &str, token: &str) -> usize {
    source
        .find(token)
        .unwrap_or_else(|| panic!("{token:?} in {source:?}"))
}
