//! Typed admission evidence for property values.
//!
//! An accepted value is clean and validates to the same declarations. A
//! rejection names its error kind, the rejecting property and the
//! responsible authored token. Each suite derives the expected token from the
//! grammar it tests by the crate's location rule (reference "Source
//! coordinates and diagnostics"): a token that cannot continue the
//! production is responsible, including a closing `)`; when input ends
//! before the value or function is complete, the last authored token is;
//! `None` only for an input without any authored token. A math function
//! whose contents parse but whose type does not check is responsible as a
//! whole (CSS Values 4 §10.9), except that a sum names its incompatible
//! operand.
//!
//! No specification defines the responsible token, so it is an
//! implementation-defined part of the diagnostic; every case asserts the
//! token this rule derives from its grammar.

use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::{component_failure, numeric_failure};
use surgeist_css::*;

/// The authored token a property-value rejection is expected to name.
#[derive(Clone, Copy, Debug)]
pub enum Responsible<'a> {
    /// The rejection names this authored token.
    At(&'a str),
    /// The value is empty, so no authored token is responsible.
    End,
}

impl<'a> From<Option<&'a str>> for Responsible<'a> {
    fn from(token: Option<&'a str>) -> Self {
        token.map_or(Self::End, Self::At)
    }
}

/// The sole declaration of the clean style attribute `css`, which validation
/// admits as the same declaration list.
#[track_caller]
pub fn validated_sole_declaration(css: &str) -> CssDeclaration {
    let report = CssParser::default().parse_block_contents(css);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(css)
            .into_validation_result()
            .as_ref(),
        Ok(report.syntax()),
        "{css}: validation admits the clean report"
    );
    crate::css::support::sole_declaration(css)
}

/// [`validated_sole_declaration`] of `name:value`, written with `importance`.
#[track_caller]
pub fn validated_declaration(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    let priority = match importance {
        CssImportance::Important => "!important",
        CssImportance::Normal => "",
    };
    validated_sole_declaration(&format!("{name}:{value}{priority}"))
}

/// [`validated_declaration`] of a known property.
#[track_caller]
pub fn validated_known_declaration(
    property: CssKnownProperty,
    value: &str,
    importance: CssImportance,
) -> CssDeclaration {
    validated_declaration(property.canonical_name(), value, importance)
}

/// The first original implicit-closure origin in the serialization of
/// `components`, which must retain one.
#[track_caller]
pub fn implicit_closure_origin(components: &CssComponentValues) -> CssValueOrigin {
    let serialized = components.serialize_tokens(Default::default()).unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original implicit closure")
}

/// [`implicit_closure_origin`] of `components`, which must be an original
/// EOF closure of `original`: an empty span at its end.
#[track_caller]
pub fn eof_closure_origin(components: &CssComponentValues, original: &str) -> CssValueOrigin {
    // Emitted mappings retain original EOF; repaired comment text does not
    // establish an origin by its length or require a closing suffix.
    let origin = implicit_closure_origin(components);
    let CssValueOrigin::ImplicitClosure { at, .. } = &origin else {
        unreachable!()
    };
    assert_eq!(at.source().as_str(), original);
    assert_eq!(at.span().start().byte_offset().value(), original.len());
    assert_eq!(at.span().start(), at.span().end());
    origin
}

/// The first implicit closing origin of a top-level function or block in
/// `components`, else the first implicit-closure mapping of their
/// serialization (which covers comments ended by EOF).
#[track_caller]
pub fn implicit_closing_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    implicit_closure_origin(components)
}

/// Asserts that `diagnostic` dropped a declaration of `property` for its
/// value, and that checked construction of `value` by name and by grammar
/// fails with the same property-grammar error.
#[track_caller]
pub fn assert_checked_matches_diagnostic(
    property: CssKnownProperty,
    value: &str,
    diagnostic: &CssRecoveryDiagnostic,
) {
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "{property:?}: expected an invalid property value, got {diagnostic:?}"
    );
    assert_eq!(
        detail.known_property().unwrap(),
        property,
        "rejecting property"
    );
    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
    for result in [
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            components.clone(),
            CssImportance::Normal,
        ),
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(property.grammar()),
            components,
            CssImportance::Normal,
        ),
    ] {
        let error = result.expect_err(value);
        assert_same_grammar_facts(
            &error,
            diagnostic.error(),
            &format!("{property:?}: {value}"),
        );
    }
}

/// The grammar facts that a stylesheet diagnostic and a checked property-value
/// failure both report, so that one assertion covers every front.
pub trait GrammarFacts: std::fmt::Debug {
    /// The diagnostic code, or `None` when no grammar rejected the value.
    fn grammar_code(&self) -> Option<CssErrorCode>;
    /// The property whose grammar rejected the value.
    fn rejecting_property(&self) -> Option<CssKnownProperty>;
    /// The responsible authored token, or `None` at the end of input.
    fn responsible_token(&self) -> Option<&CssTokenSummary>;
}

impl GrammarFacts for CssError {
    fn grammar_code(&self) -> Option<CssErrorCode> {
        Some(self.code())
    }
    fn rejecting_property(&self) -> Option<CssKnownProperty> {
        self.known_property()
    }
    fn responsible_token(&self) -> Option<&CssTokenSummary> {
        self.encountered()
    }
}

impl GrammarFacts for CssPropertyValueError {
    fn grammar_code(&self) -> Option<CssErrorCode> {
        self.code()
    }
    fn rejecting_property(&self) -> Option<CssKnownProperty> {
        self.known_property()
    }
    fn responsible_token(&self) -> Option<&CssTokenSummary> {
        self.encountered()
    }
}

/// The facts [`property_value_facts`] reads, in accessor order.
pub type PropertyValueFacts<'a> = (
    Option<CssErrorCode>,
    Option<CssGrammarExpectation>,
    Option<&'a CssTokenSummary>,
    Option<CssKnownProperty>,
    Option<&'a CssColorComponentName>,
    Option<CssProductionId>,
    Option<u32>,
    Option<CssDeclarationContextRef<'a>>,
    Option<Option<&'a CssComponentValueError>>,
    Option<ComponentFailure>,
);

/// The grammar and component facts of a checked property-value failure,
/// without its origin, for comparing two fronts' failures: a grammar
/// failure's complete component error, and otherwise only the class of the
/// component failure.
pub fn property_value_facts(error: &CssPropertyValueError) -> PropertyValueFacts<'_> {
    (
        error.code(),
        error.expectation(),
        error.encountered(),
        error.known_property(),
        error.color_component(),
        error.production(),
        error.nesting_limit(),
        error.declaration_context(),
        error.code().map(|_| error.component_error()),
        error.component_error().map(component_failure),
    )
}

/// Asserts that a checked property-value failure reports the same grammar facts as a
/// stylesheet diagnostic; only the diagnostic carries an authored position.
#[track_caller]
pub fn assert_same_grammar_facts(
    checked: &CssPropertyValueError,
    diagnostic: &CssError,
    label: &str,
) {
    assert_eq!(
        (
            checked.code(),
            checked.expectation(),
            checked.encountered(),
            checked.known_property(),
            checked.color_component(),
            checked.production(),
            checked.nesting_limit(),
            checked.declaration_context(),
            checked.component_error(),
        ),
        (
            Some(diagnostic.code()),
            diagnostic.expectation(),
            diagnostic.encountered(),
            diagnostic.known_property(),
            diagnostic.color_component(),
            diagnostic.production(),
            diagnostic.nesting_limit(),
            diagnostic.declaration_context(),
            diagnostic.component_error(),
        ),
        "{label}"
    );
}

/// Asserts that strict reentry rejected its replacement by the grammar of
/// `property` at `token`.
#[track_caller]
pub fn assert_reentry_rejected<'a>(
    result: Result<CssContributions, CssExpansionError>,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) {
    let error = result.expect_err("strict reentry rejects the replacement");
    let Some(error) = error.replacement_error() else {
        panic!("{property:?}: expected an invalid replacement, got {error:?}")
    };
    assert_checked_rejected(Err(error.clone()), property, token);
}

/// Asserts that `error` is a property-value grammar rejection by `property`
/// at the responsible authored `token`.
#[track_caller]
pub fn assert_invalid_value<'a>(
    error: &CssError,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) {
    assert_invalid_value_facts(
        Some(error.code()),
        error.known_property(),
        error.encountered(),
        property,
        token,
        error,
    );
}

#[track_caller]
fn assert_invalid_value_facts<'a>(
    code: Option<CssErrorCode>,
    known_property: Option<CssKnownProperty>,
    encountered: Option<&CssTokenSummary>,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
    error: &dyn std::fmt::Debug,
) {
    match code {
        Some(CssErrorCode::InvalidPropertyValue) => {
            assert_eq!(known_property, Some(property), "rejecting property");
        }
        // A `<color>` failure keeps its color-syntax code, which names the
        // responsible token but no property.
        Some(CssErrorCode::InvalidColorSyntax) => {}
        _ => panic!("{property:?}: expected an invalid property value, got {error:?}"),
    }
    let actual = encountered.map(CssTokenSummary::authored);
    let token = match token.into() {
        Responsible::At(token) => Some(token),
        Responsible::End => None,
    };
    assert_eq!(actual, token, "{property:?}: responsible token");
}

/// Asserts that checked construction failed the grammar of `property` at
/// `token`, and returns the failure for further origin checks.
#[track_caller]
pub fn assert_checked_rejected<'a>(
    result: Result<CssDeclaration, CssPropertyValueError>,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) -> CssPropertyValueError {
    let error = match result {
        Ok(declaration) => panic!("{property:?}: checked construction accepted {declaration:?}"),
        Err(error) => error,
    };
    assert!(
        error.code().is_some(),
        "{property:?}: expected a grammar rejection, got {error:?}"
    );
    assert_invalid_value_facts(
        error.code(),
        error.known_property(),
        error.encountered(),
        property,
        token,
        &error,
    );
    error
}

/// Asserts that checked construction rejected components whose original
/// syntax was repaired by an implicit EOF closure, at that closure.
#[track_caller]
pub fn assert_checked_closure_rejected(
    result: Result<CssDeclaration, CssPropertyValueError>,
) -> CssPropertyValueError {
    let error = match result {
        Ok(declaration) => panic!("checked construction accepted a closure: {declaration:?}"),
        Err(error) => error,
    };
    assert!(
        error.code() == Some(CssErrorCode::UnexpectedEnd),
        "{error:?}"
    );
    assert!(
        matches!(
            error.origin(),
            CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { .. }))
        ),
        "original implicit closure provenance: {error:?}"
    );
    error
}

/// Asserts that checked construction of `value` by the grammar of `property`
/// fails at `token`, with both importances and by name and grammar.
#[track_caller]
pub fn assert_checked_value_rejected<'a>(
    property: CssKnownProperty,
    value: &str,
    token: impl Into<Responsible<'a>>,
) {
    let token = token.into();
    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
    let before = components.clone();
    for importance in [CssImportance::Normal, CssImportance::Important] {
        assert_checked_rejected(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
                components.clone(),
                importance,
            ),
            property,
            token,
        );
        assert_checked_rejected(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(property.grammar()),
                components.clone(),
                importance,
            ),
            property,
            token,
        );
    }
    assert_eq!(
        components, before,
        "{property:?}: {value:?}: rejected input changed"
    );
}

/// Asserts that the style attribute `css` reports exactly one diagnostic, a
/// dropped declaration of `property` rejected at `token`, and that
/// validation fails with the same diagnostics.
#[track_caller]
pub fn assert_style_attribute_rejected<'a>(
    css: &str,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) -> CssParseReport<CssDeclarationList> {
    let report = CssParser::default().parse_block_contents(css);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{css}: expected one diagnostic, got {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{css}"
    );
    assert!(
        matches!(
            diagnostic.error().code(),
            CssErrorCode::InvalidColorSyntax | CssErrorCode::InvalidPropertyValue
        ),
        "{css}"
    );
    assert_invalid_value(diagnostic.error(), property, token);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(css)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics(),
        "{css}: validation reports the parse diagnostics"
    );
    report
}

/// Asserts that the report for `css` retained its syntax and that its only
/// diagnostics are unexpected ends at EOF: blocks and functions repaired by
/// implicit closures, or a comment ended by EOF (CSS Syntax 3 §4.3.2).
#[track_caller]
pub fn assert_implicit_closure<T>(css: &str, report: &CssParseReport<T>) {
    assert!(
        !report.diagnostics().is_empty(),
        "{css}: expected a closure"
    );
    for diagnostic in report.diagnostics() {
        assert!(
            matches!(
                diagnostic.action(),
                CssRecoveryAction::RetainWithImplicitClosure
                    | CssRecoveryAction::IgnoreUnterminatedComment
            ),
            "{css}: {diagnostic:?}"
        );
        assert!(
            diagnostic.error().code() == CssErrorCode::UnexpectedEnd,
            "{css}: {diagnostic:?}"
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            css.len(),
            "{css}: the closure is reported at EOF"
        );
    }
}

/// The expected failure of checked property-value text.
#[derive(Clone, Copy, Debug)]
pub enum ValueTextRejection<'a> {
    /// The property grammar rejected the value at this responsible token.
    Grammar(Option<&'a str>),
    /// The value text is not a value at the syntax level: this authored token
    /// is unexpected before any property grammar applies.
    Syntax(&'a str),
    /// The value text ended inside an unclosed function or block
    /// (CSS Syntax 3 §5.4.8), which strict construction does not repair.
    Closure,
    /// Component syntax failed with this kind before the grammar ran.
    Component(ComponentFailure),
}

/// Asserts that checked value text for `property` failed as `expected`.
#[track_caller]
pub fn assert_value_text_rejected(
    result: &Result<CssDeclaration, CssPropertyValueError>,
    property: CssKnownProperty,
    expected: ValueTextRejection<'_>,
) {
    let Err(error) = result else {
        panic!("{property:?}: expected {expected:?}, got {result:?}")
    };
    match expected {
        ValueTextRejection::Grammar(token) => {
            assert_checked_rejected(Err(error.clone()), property, token);
        }
        ValueTextRejection::Syntax(token) => {
            assert_eq!(
                error.code(),
                Some(CssErrorCode::UnexpectedToken),
                "{property:?}: expected an unexpected token, got {error:?}"
            );
            assert_eq!(
                error.encountered().unwrap().authored(),
                token,
                "responsible token"
            );
        }
        ValueTextRejection::Closure => {
            assert_eq!(
                error.code(),
                Some(CssErrorCode::UnexpectedEnd),
                "{property:?}: expected an unexpected end, got {error:?}"
            );
        }
        ValueTextRejection::Component(kind) => {
            assert_eq!(error.code(), None, "{property:?}: {error:?}");
            assert_eq!(
                error.component_error().map(component_failure),
                Some(kind),
                "{error:?}"
            );
        }
    }
}

/// Asserts that checked numeric construction failed with `kind`, and
/// returns the failure for further origin checks.
#[track_caller]
pub fn assert_numeric_rejected<T: std::fmt::Debug>(
    result: Result<T, CssNumericConstructionError>,
    kind: NumericFailure,
) -> CssNumericConstructionError {
    let error = match result {
        Ok(value) => panic!("{kind:?}: numeric construction accepted {value:?}"),
        Err(error) => error,
    };
    assert_eq!(numeric_failure(&error), kind, "{error:?}");
    error
}

/// Asserts that checked construction of the single component `component`
/// failed with `kind` at that component's origin.
#[track_caller]
pub fn assert_component_rejected<T: std::fmt::Debug>(
    construct: impl FnOnce(CssComponentValue) -> Result<T, CssNumericConstructionError>,
    component: CssComponentValue,
    kind: NumericFailure,
) {
    let origin = component.origin().clone();
    let error = assert_numeric_rejected(construct(component), kind);
    assert_eq!(error.origin(), Some(&origin), "responsible component");
}

/// Asserts that `name:value` between two `color` neighbors drops only that
/// declaration, rejected by the selected grammar of `name` at `token`, and
/// that checked construction by that grammar rejects the same token.
#[track_caller]
pub fn assert_declaration_rejected<'a>(name: &str, value: &str, token: impl Into<Responsible<'a>>) {
    let token = token.into();
    let grammar =
        CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"));
    let property = grammar.target_property();
    let source = format!("color:red;{name}:{value};color:blue");
    let report = assert_style_attribute_rejected(&source, property, token);
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|neighbor| neighbor.known().unwrap().property() == CssKnownProperty::Color),
        "{source}: neighbor identities"
    );
    assert_checked_rejected(
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(grammar),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        ),
        property,
        token,
    );
}
