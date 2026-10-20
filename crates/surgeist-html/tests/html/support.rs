//! Fixtures shared by every module of the HTML integration binary.
//!
//! Each helper is written once here. Modules import a helper under the
//! local name their tests already use, so a module still states which
//! fixture it depends on without copying its constructor.

use std::convert::Infallible;

use surgeist_html::bounded::native_element_initialization_with;
use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeContextAuthorState,
    HtmlAttributeContextConformanceError, HtmlInputSanitization, HtmlInputValueSanitization,
    HtmlNativeInitialization, HtmlNonNegativeInteger, HtmlNumericAttributeAuthorState,
    HtmlNumericAttributeReport, HtmlNumericAttributeValueOrigin, HtmlValueInterpreter, HtmlYear,
    KnownHtmlAttribute, KnownHtmlElement, Namespace, OrderedAttributes, QualifiedName,
};

/// HTML-namespace element name with the exact local name and no prefix.
pub fn html_element(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

/// Qualified name of a known HTML element.
pub fn known_element(name: KnownHtmlElement) -> QualifiedName {
    name.qualified_name()
}

/// Qualified name of the HTML `input` element.
pub fn input_element() -> QualifiedName {
    KnownHtmlElement::Input.qualified_name()
}

/// Known HTML attribute carrying the exact raw value.
pub fn known_attribute(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}

/// Known HTML attribute whose raw value is borrowed text, for callers that
/// pass borrowed `String` values.
pub fn known_text_attribute(name: KnownHtmlAttribute, value: &str) -> Attribute {
    known_attribute(name, value)
}

/// The `name` attribute carrying the exact raw value.
pub fn name_attribute(value: impl Into<DocumentString>) -> Attribute {
    known_attribute(KnownHtmlAttribute::Name, value)
}

/// Null-namespace, unprefixed attribute with the exact local name and value.
pub fn unqualified_attribute(local: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}

/// Null-namespace, unprefixed attribute whose raw value is borrowed text,
/// for callers that pass borrowed `String` values.
pub fn unqualified_text_attribute(local: &str, value: &str) -> Attribute {
    unqualified_attribute(local, value)
}

/// Validated ordered attributes in the supplied order.
pub fn ordered_attributes(values: impl IntoIterator<Item = Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(values).unwrap()
}

/// Value interpreter admitting 4096 raw input units.
pub fn value_interpreter() -> HtmlValueInterpreter {
    HtmlValueInterpreter::new(4096)
}

/// Native non-negative integer from exact decimal digits.
pub fn non_negative_integer(digits: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits)).unwrap()
}

/// Valid HTML year.
pub fn year(value: u64) -> HtmlYear {
    HtmlYear::new(value).unwrap()
}

/// Native initialization of an element whose name reads cannot fail.
pub fn initialize_native_element(name: &QualifiedName) -> HtmlNativeInitialization {
    native_element_initialization_with(name, |_| Ok::<_, Infallible>(())).unwrap()
}

/// Author state of an attribute that the input element's current state
/// forbids.
pub fn input_state_forbids_attribute() -> HtmlAttributeContextAuthorState {
    HtmlAttributeContextAuthorState::Nonconforming(
        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
    )
}

/// String sanitization of an input value policy; the literal cases that use
/// it never select the range or color operation.
pub fn string_sanitization(policy: HtmlInputValueSanitization) -> HtmlInputSanitization {
    match policy {
        HtmlInputValueSanitization::String(policy) => policy,
        HtmlInputValueSanitization::Range | HtmlInputValueSanitization::Color => {
            panic!("these literal cases require the string operation")
        }
    }
}

/// Report of a table cell or column span attribute.
pub type SpanReport = HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>;

/// The span report applies `value` from the stated origin.
pub fn assert_span_quantity(
    report: &SpanReport,
    value: u16,
    origin: HtmlNumericAttributeValueOrigin,
) {
    assert_eq!(report.intrinsic(), Some(&value));
    assert_eq!(report.origin(), Some(origin));
}

/// The span attribute is not applicable and reports no value.
pub fn assert_span_unapplied(report: &SpanReport) {
    assert_ne!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
    assert!(report.common().is_none());
    assert!(report.intrinsic().is_none());
    assert!(report.origin().is_none());
}
