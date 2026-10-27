//! Single-rule stylesheet fixtures, rule and payload accessors, and typed
//! acceptance and rejection evidence shared by authored-syntax tests.
//!
//! Fixtures are `.panel { name: value; }` style rules parsed by the public
//! stylesheet parser. A rejection names the exact diagnostic kind and the
//! authored identity it reports, so an unrelated earlier failure cannot
//! satisfy it.

use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssValueTokenRef,
};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword};
use surgeist_css::diagnostic::{CssError, CssErrorCode, CssRecoveryAction};
use surgeist_css::numeric::{
    CssCalculation, CssCalculationExpressionRef, CssCalculationValueRef, CssInteger, CssLength,
    CssLengthPercentage, CssNonNegative, CssNumber, CssNumeric,
};
use surgeist_css::property::{CssKnownDeclaration, CssKnownProperty};
use surgeist_css::query::media::CssMediaLength;
use surgeist_css::rule::{
    CssContainerRule, CssLayerBlockRule, CssMediaRule, CssRule, CssStyleRule,
};
use surgeist_css::selector::{CssPseudoElement, CssPseudoElementSegment, CssPseudoElementSequence};
use surgeist_css::source::CssSourcePosition;
use surgeist_css::values::font::CssFontWeightRange;

use crate::css::support::parse_report::CssParseReportTestExt;

/// The typed payload of a declaration's `$variant` property wrapper, read
/// through that wrapper's `$accessor`.
macro_rules! property_payload {
    ($declaration:expr, $variant:ident, $accessor:ident) => {{
        let declaration = &$declaration;
        let Some(CssKnownPropertyValueRef::$variant(value)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            panic!(
                "expected an ordinary {} wrapper: {declaration:?}",
                stringify!($variant)
            );
        };
        value.$accessor().clone()
    }};
}

/// The ordinary value of one effective font-face descriptor.
macro_rules! face_value {
    ($descriptors:expr, $kind:ident) => {{
        let descriptor = $descriptors
            .effective(CssFontFaceDescriptorKind::$kind)
            .expect("effective font-face descriptor");
        let CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::$kind(value)) =
            descriptor.value()
        else {
            panic!("expected ordinary font-face descriptor");
        };
        value
    }};
}

/// The parsed position `line` newlines and `column` spaces into a source:
/// the position of an unknown at-rule authored there, which CSS Syntax 3
/// (CRD 2026-10-01) §9 reports at its at-keyword.
pub fn source_position(line: u32, column: u32) -> CssSourcePosition {
    let source = format!(
        "{}{}@unknown;",
        "\n".repeat(line as usize),
        " ".repeat(column as usize)
    );
    let error = CssParser::default()
        .parse_sheet(&source)
        .expect_err("an unknown at-rule is diagnosed");
    let position = error.position();
    assert_eq!(
        (
            position.line().value(),
            position.column().value(),
            position.byte_offset().value()
        ),
        (line, column, (line + column) as usize),
        "the at-keyword sits at the requested position"
    );
    position
}

/// The property name authored in a `.panel { name: value; }` fixture.
pub fn declared_property(input: &'static str) -> &'static str {
    let (_, declaration) = input.split_once("{ ").expect("one style rule fixture");
    let (name, _) = declaration
        .split_once(':')
        .expect("one declaration fixture");
    name
}

pub fn style_rule(rule: &CssRule) -> &CssStyleRule {
    match rule {
        CssRule::Style(rule) => rule,
        unexpected => panic!("expected style rule, got {unexpected:?}"),
    }
}

pub fn media_rule(rule: &CssRule) -> &CssMediaRule {
    match rule {
        CssRule::Media(rule) => rule,
        unexpected => panic!("expected media rule, got {unexpected:?}"),
    }
}

pub fn container_rule(rule: &CssRule) -> &CssContainerRule {
    match rule {
        CssRule::Container(rule) => rule,
        unexpected => panic!("expected container rule, got {unexpected:?}"),
    }
}

pub fn layer_block_rule(rule: &CssRule) -> &CssLayerBlockRule {
    match rule {
        CssRule::LayerBlock(rule) => rule,
        unexpected => panic!("expected layer block rule, got {unexpected:?}"),
    }
}

/// The known property of an ordinary declaration, or `None` for a custom or
/// independent SVG declaration.
pub fn declaration_property(declaration: &CssDeclaration) -> Option<CssKnownProperty> {
    declaration.known().map(CssKnownDeclaration::property)
}

/// The first declaration of `property` in the first style rule of a clean sheet.
pub fn declaration(input: &str, property: CssKnownProperty) -> CssDeclaration {
    let sheet = CssParser::default()
        .parse_sheet(input)
        .expect("clean stylesheet");
    style_rule(&sheet.rules()[0])
        .declarations()
        .iter()
        .find(|declaration| declaration_property(declaration) == Some(property))
        .unwrap_or_else(|| panic!("{input}: no {property:?} declaration"))
        .clone()
}

/// The sole declaration of the sole style rule of a clean sheet.
pub fn single_declaration(input: &str) -> CssDeclaration {
    let sheet = CssParser::default()
        .parse_sheet(input)
        .expect("clean stylesheet");
    let [rule] = sheet.rules() else {
        panic!("{input} should parse exactly one rule");
    };
    let rule = style_rule(rule);
    let [declaration] = rule.declarations().as_slice() else {
        panic!("{input} should parse exactly one declaration");
    };
    declaration.clone()
}

fn declaration_sheet(property_name: &str, authored_value: &str) -> String {
    format!(".test {{ {property_name}: {authored_value}; }}")
}

/// The sole declaration of a clean `.test { name: value; }` sheet.
pub fn parse_single_declaration(property_name: &str, authored_value: &str) -> CssDeclaration {
    single_declaration(&declaration_sheet(property_name, authored_value))
}

pub fn weight_number(spelling: &str) -> CssNumeric<CssNumber, CssFontWeightRange> {
    CssNumeric::<CssNumber, CssFontWeightRange>::try_from_component(
        CssComponentValue::try_number(spelling).unwrap(),
    )
    .expect("valid exact font weight")
}

pub fn nonnegative_number(spelling: &str) -> CssNumeric<CssNumber, CssNonNegative> {
    CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
        CssComponentValue::try_number(spelling).unwrap(),
    )
    .unwrap()
}

pub fn number_spelling(component: &CssComponentValue) -> &str {
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        panic!("expected exact number token");
    };
    number.representation()
}

pub fn assert_length_literal(
    component: &CssComponentValue,
    expected_number: &str,
    expected_unit: &str,
) {
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        component.view()
    else {
        panic!("expected exact dimension token");
    };
    assert_eq!(number.representation(), expected_number);
    assert_eq!(unit, expected_unit);
}

pub fn assert_integer_literal(value: &CssNumeric<CssInteger>, expected: &str) {
    let Some(integer) = value.literal() else {
        panic!("expected exact integer literal")
    };
    assert_eq!(integer.numeric().representation(), expected);
}

pub fn assert_media_length_literal(value: &CssMediaLength, number: &str, unit: &str) {
    let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(literal)) =
        value.calculation().expression()
    else {
        panic!("expected exact media length literal");
    };
    assert_eq!(literal.representation(), number);
    assert_eq!(literal.unit(), Some(unit));
}

pub fn assert_pseudo_elements(sequence: &CssPseudoElementSequence, expected: &[CssPseudoElement]) {
    let expected: Vec<_> = expected
        .iter()
        .cloned()
        .map(CssPseudoElementSegment::PseudoElement)
        .collect();
    assert_eq!(sequence.segments(), expected);
}

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

pub fn exact_percentage(component: Option<&CssComponentValue>, representation: &str) -> bool {
    matches!(
        component.map(CssComponentValue::view),
        Some(CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)))
            if number.representation() == representation
    )
}

pub fn signed_length(css: &str) -> CssNumeric<CssLength> {
    let components = CssComponentValues::parse(&CssParser::default(), css).unwrap();
    if css.contains('(') {
        CssNumeric::<CssLength>::try_from_calculation(
            CssCalculation::<CssLength>::try_from_components(components, Default::default())
                .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLength>::try_from_component(CssComponentValue::try_token(css).unwrap())
            .unwrap()
    }
}

pub fn nonnegative_length(css: &str) -> CssNumeric<CssLength, CssNonNegative> {
    let components = CssComponentValues::parse(&CssParser::default(), css).unwrap();
    if css.contains('(') {
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLength>::try_from_components(components, Default::default())
                .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

pub fn signed_length_percentage(css: &str) -> CssNumeric<CssLengthPercentage> {
    let components = CssComponentValues::parse(&CssParser::default(), css).unwrap();
    if css.contains('(') {
        CssNumeric::<CssLengthPercentage>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                components,
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLengthPercentage>::try_from_component(
            CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

pub fn nonnegative_length_percentage(css: &str) -> CssNumeric<CssLengthPercentage, CssNonNegative> {
    let components = CssComponentValues::parse(&CssParser::default(), css).unwrap();
    if css.contains('(') {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                components,
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

/// One exact diagnostic kind, with the authored identity it names.
///
/// Each variant accepts exactly one [`ErrorKind`] variant, so a rejection by
/// an unrelated rule (another property, a tokenizer failure or a different
/// production) fails the expectation.
pub enum ExpectedErrorKind {
    UnexpectedToken,
    InvalidSelector,
    /// A value rejected by the grammar of `property`.
    InvalidPropertyValue {
        property: &'static str,
    },
    /// A `<color>` value rejected by the color grammar; its diagnostic names
    /// no property, so the responsible token identifies the failure.
    InvalidColorSyntax {
        encountered: &'static str,
    },
    UnknownAtRule {
        name: &'static str,
    },
    UnknownProperty {
        name: &'static str,
    },
}

impl ExpectedErrorKind {
    #[track_caller]
    fn assert_matches(&self, actual: &CssError, label: &str) {
        let matched = match self {
            Self::UnexpectedToken => actual.code() == CssErrorCode::UnexpectedToken,
            Self::InvalidSelector => actual.code() == CssErrorCode::InvalidSelector,
            Self::InvalidPropertyValue { property } => {
                actual.code() == CssErrorCode::InvalidPropertyValue
                    && actual.known_property() == expected_property(property, label)
            }
            Self::InvalidColorSyntax { encountered } => {
                actual.code() == CssErrorCode::InvalidColorSyntax
                    && actual
                        .encountered()
                        .is_some_and(|token| token.authored() == *encountered)
            }
            Self::UnknownAtRule { name } => {
                actual.code() == CssErrorCode::UnknownAtRule
                    && actual.at_rule().is_some_and(|rule| rule.as_str() == *name)
            }
            Self::UnknownProperty { name } => {
                actual.code() == CssErrorCode::UnknownProperty
                    && actual
                        .property_name()
                        .is_some_and(|property| property.as_str() == *name)
            }
        };
        assert!(
            matched,
            "{label} rejected with unexpected error kind: {actual:?}"
        );
    }
}

#[track_caller]
fn expected_property(name: &str, label: &str) -> Option<CssKnownProperty> {
    let expected = CssKnownProperty::from_name(name);
    assert!(
        expected.is_some(),
        "{label}: `{name}` is not a known property"
    );
    expected
}

/// The first diagnostic of `input`, which must match `expected_error`.
#[track_caller]
pub fn assert_sheet_rejected(input: &str, expected_error: &ExpectedErrorKind) -> CssError {
    let error = CssParser::default()
        .parse_sheet(input)
        .expect_err("invalid CSS must report a diagnostic");
    expected_error.assert_matches(&error, input);
    error
}

pub struct RejectedSheetCase {
    pub label: &'static str,
    pub input: &'static str,
    pub expected_error: ExpectedErrorKind,
}

impl RejectedSheetCase {
    #[track_caller]
    pub fn assert_rejects(&self) -> CssError {
        let error = CssParser::default()
            .parse_sheet(self.input)
            .expect_err("invalid CSS must report a diagnostic");
        self.expected_error.assert_matches(&error, self.label);
        error
    }
}

pub struct RejectedDeclarationCase {
    pub label: &'static str,
    pub property_name: &'static str,
    pub authored_value: &'static str,
    pub expected_error: ExpectedErrorKind,
    pub property_name_should_be_recognized: bool,
}

impl RejectedDeclarationCase {
    #[track_caller]
    pub fn assert_rejects(&self) -> CssError {
        let css = declaration_sheet(self.property_name, self.authored_value);
        let error = assert_sheet_rejected(&css, &self.expected_error);
        assert_eq!(
            error.code() == CssErrorCode::UnknownProperty,
            !self.property_name_should_be_recognized,
            "{} property-name recognition mismatch",
            self.label,
        );
        error
    }
}

pub struct AcceptedDeclarationCase {
    pub label: &'static str,
    pub property_name: &'static str,
    pub authored_value: &'static str,
    pub expected_property: CssKnownProperty,
    pub expected_global: CssGlobalKeyword,
}

impl AcceptedDeclarationCase {
    #[track_caller]
    pub fn assert_accepts(&self) -> CssDeclaration {
        let declaration = parse_single_declaration(self.property_name, self.authored_value);
        assert_eq!(
            declaration_property(&declaration),
            Some(self.expected_property),
            "{} parsed to the wrong property",
            self.label,
        );
        let known = declaration.known().expect("known declaration");
        assert_eq!(
            known.global(),
            Some(self.expected_global),
            "{} parsed to the wrong global value",
            self.label,
        );
        assert!(
            known.property_value().is_none(),
            "{} global value must have no ordinary property payload",
            self.label,
        );
        declaration
    }
}

#[track_caller]
pub fn assert_accepts_declarations(cases: &[AcceptedDeclarationCase]) {
    for case in cases {
        case.assert_accepts();
    }
}

#[track_caller]
pub fn assert_rejects_declarations(cases: &[RejectedDeclarationCase]) {
    for case in cases {
        case.assert_rejects();
    }
}

#[track_caller]
pub fn assert_rejects_sheets(cases: &[RejectedSheetCase]) {
    for case in cases {
        case.assert_rejects();
    }
}

/// Asserts that the sole declaration of the sole style rule in `input` is
/// dropped with exactly one diagnostic while its rule is retained: CSS Syntax 3
/// (CRD 2026-10-01) §5.5.5-§5.5.6 consumes and discards an invalid declaration
/// inside a block's contents without discarding the enclosing rule. Returns
/// that diagnostic.
#[track_caller]
pub fn assert_drops_sole_declaration(input: &str) -> CssError {
    let report = CssParser::default().parse_sheet(input);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{input}: exactly one diagnostic: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{input}: the invalid declaration alone is dropped"
    );
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("{input}: the enclosing style rule is retained");
    };
    assert!(
        rule.declarations().is_empty(),
        "{input}: the invalid declaration is not retained"
    );
    diagnostic.error().clone()
}

/// Asserts that the sole keyframe block of the sole `@keyframes` rule in
/// `input` is dropped with exactly one diagnostic while the at-rule is kept:
/// CSS Animations 1 §3 admits only `from`, `to` and percentages in
/// `[0%, 100%]` as `<keyframe-selector>`, so another prelude makes that
/// qualified rule invalid, and CSS Syntax 3 (CRD 2026-10-01) §5.5.5 discards
/// only the invalid rule. The diagnostic is an invalid qualified rule at the
/// `selector` token.
#[track_caller]
pub fn assert_drops_invalid_keyframe_block(input: &str, selector: &str) {
    let report = CssParser::default().parse_sheet(input);
    assert_eq!(
        report.diagnostics().len(),
        1,
        "{input}: one diagnostic: {:?}",
        report.diagnostics()
    );
    crate::css::support::rules::ExpectedRejection::new(CssErrorCode::InvalidQualifiedRule)
        .at(selector)
        .assert_first(&report);
    let [CssRule::Keyframes(rule)] = report.syntax().rules() else {
        panic!("{input}: the @keyframes rule is retained");
    };
    assert!(
        rule.blocks().is_empty(),
        "{input}: the invalid keyframe block is not retained"
    );
}

/// Typed rejection evidence: the diagnostic kind, the authored identity that
/// kind names (a property, at-rule or descriptor) and the responsible token.
///
/// The responsible token is the first authored token after which no
/// continuation of the selected grammar is valid; a rule prelude cut short by
/// its block names that `{`. `encountered: None` states that the kind names
/// no token. Where the specifications leave the responsible token open, the
/// expectation records the parser's diagnostic convention: a complex selector
/// that is invalid as a whole (a pseudo-element before another compound)
/// names the end of its prelude, a construct cut short by its function or
/// block end names its dangling last token, a nested `:has()` names its
/// argument, and an undeclared namespace prefix names the qualified name.
#[derive(Clone, Copy, Debug)]
pub struct Rejection {
    pub code: CssErrorCode,
    pub names: Option<&'static str>,
    pub encountered: Option<&'static str>,
}

impl Rejection {
    pub const fn new(
        code: CssErrorCode,
        names: Option<&'static str>,
        encountered: Option<&'static str>,
    ) -> Self {
        Self {
            code,
            names,
            encountered,
        }
    }

    /// A selector rejected at `encountered`.
    pub const fn selector(encountered: Option<&'static str>) -> Self {
        Self::new(CssErrorCode::InvalidSelector, None, encountered)
    }

    /// A value rejected by the grammar of `property` at `encountered`.
    pub const fn property(property: &'static str, encountered: Option<&'static str>) -> Self {
        Self::new(
            CssErrorCode::InvalidPropertyValue,
            Some(property),
            encountered,
        )
    }

    /// An at-rule prelude rejected by the grammar of `@name` at `encountered`.
    pub const fn prelude(name: &'static str, encountered: Option<&'static str>) -> Self {
        Self::new(CssErrorCode::InvalidAtRulePrelude, Some(name), encountered)
    }

    /// An at-rule body rejected by the grammar of `@name` at `encountered`.
    pub const fn body(name: &'static str, encountered: Option<&'static str>) -> Self {
        Self::new(CssErrorCode::InvalidAtRuleBody, Some(name), encountered)
    }

    /// An `@name` rule authored where its context does not admit it.
    pub const fn placement(name: &'static str) -> Self {
        Self::new(CssErrorCode::InvalidAtRulePlacement, Some(name), None)
    }

    /// An unknown at-rule `@name`.
    pub const fn unknown_at_rule(name: &'static str) -> Self {
        Self::new(CssErrorCode::UnknownAtRule, Some(name), None)
    }

    /// An unknown property `name`.
    pub const fn unknown_property(name: &'static str) -> Self {
        Self::new(CssErrorCode::UnknownProperty, Some(name), None)
    }
}

/// The identity a diagnostic names.
fn rejection_names(error: &CssError) -> Option<String> {
    let name = match error.code() {
        CssErrorCode::InvalidPropertyValue => error.known_property()?.canonical_name(),
        CssErrorCode::UnknownProperty | CssErrorCode::UnsupportedProperty => {
            error.property_name()?.as_str()
        }
        CssErrorCode::UnknownAtRule
        | CssErrorCode::UnsupportedAtRule
        | CssErrorCode::InvalidAtRulePlacement
        | CssErrorCode::InvalidAtRulePrelude
        | CssErrorCode::InvalidAtRuleBody => error.at_rule()?.as_str(),
        CssErrorCode::UnknownDescriptor
        | CssErrorCode::UnsupportedDescriptor
        | CssErrorCode::InvalidDescriptorValue => error.descriptor()?.as_str(),
        _ => return None,
    };
    Some(name.to_owned())
}

/// Asserts that `error` is the expected typed rejection of `input`.
#[track_caller]
pub fn assert_rejection_matches(input: &str, error: &CssError, expected: Rejection) {
    assert_eq!(
        (
            error.code(),
            rejection_names(error),
            error.encountered().map(|token| token.authored().to_owned())
        ),
        (
            expected.code,
            expected.names.map(str::to_owned),
            expected.encountered.map(str::to_owned)
        ),
        "{input}: unexpected rejection {error:?}"
    );
}

/// Asserts that the first diagnostic of the stylesheet `input` is the
/// expected typed rejection, and returns it.
#[track_caller]
pub fn assert_rejection(input: &str, expected: Rejection) -> CssError {
    let error = CssParser::default()
        .parse_sheet(input)
        .expect_err("invalid CSS must report a diagnostic");
    assert_rejection_matches(input, &error, expected);
    error
}
