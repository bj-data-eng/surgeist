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
    let report = parse_style_attribute(css);
    assert_eq!(
        validate_style_attribute(css).as_ref(),
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
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("{property:?}: expected an invalid property value, got {diagnostic:?}")
    };
    assert_eq!(detail.property(), property, "rejecting property");
    let expected = CssPropertyValueErrorKind::Grammar(diagnostic.error().kind().clone());
    let components = parse_component_values(value).unwrap();
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal,
        ),
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Normal),
    ] {
        let error = result.expect_err(value);
        assert_eq!(error.kind(), &expected, "{property:?}: {value}");
    }
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
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        panic!("{property:?}: expected an invalid replacement, got {error:?}")
    };
    assert_checked_rejected(Err(error.clone()), property, token);
}

/// Asserts that `kind` is a property-value grammar rejection by `property`
/// at the responsible authored `token`.
#[track_caller]
pub fn assert_invalid_value<'a>(
    kind: &ErrorKind,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) {
    let encountered = match kind {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), property, "rejecting property");
            detail.encountered()
        }
        // A `<color>` failure keeps its color-syntax kind, which names the
        // responsible token but no property.
        ErrorKind::InvalidColorSyntax(detail) => detail.encountered(),
        other => panic!("{property:?}: expected an invalid property value, got {other:?}"),
    };
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
    result: Result<CssDeclaration, CssPropertyValueParseError>,
    property: CssKnownProperty,
    token: impl Into<Responsible<'a>>,
) -> CssPropertyValueParseError {
    let error = match result {
        Ok(declaration) => panic!("{property:?}: checked construction accepted {declaration:?}"),
        Err(error) => error,
    };
    let CssPropertyValueErrorKind::Grammar(kind) = error.kind() else {
        panic!("{property:?}: expected a grammar rejection, got {error:?}")
    };
    assert_invalid_value(kind, property, token);
    error
}

/// Asserts that checked construction rejected components whose original
/// syntax was repaired by an implicit EOF closure, at that closure.
#[track_caller]
pub fn assert_checked_closure_rejected(
    result: Result<CssDeclaration, CssPropertyValueParseError>,
) -> CssPropertyValueParseError {
    let error = match result {
        Ok(declaration) => panic!("checked construction accepted a closure: {declaration:?}"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ),
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
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    for importance in [CssImportance::Normal, CssImportance::Important] {
        assert_checked_rejected(
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                importance,
            ),
            property,
            token,
        );
        assert_checked_rejected(
            parse_property_value_for_grammar(property.grammar(), components.clone(), importance),
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
    let report = parse_style_attribute(css);
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
    let code = match diagnostic.error().kind() {
        ErrorKind::InvalidColorSyntax(_) => CssErrorCode::InvalidColorSyntax,
        _ => CssErrorCode::InvalidPropertyValue,
    };
    assert_eq!(diagnostic.error().code(), code, "{css}");
    assert_invalid_value(diagnostic.error().kind(), property, token);
    assert_eq!(
        validate_style_attribute(css).unwrap_err().diagnostics(),
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
            matches!(diagnostic.error().kind(), ErrorKind::UnexpectedEnd(_)),
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
    Component(CssComponentValueErrorKind),
}

/// Asserts that checked value text for `property` failed as `expected`.
#[track_caller]
pub fn assert_value_text_rejected(
    result: &Result<CssDeclaration, CssPropertyValueSyntaxError>,
    property: CssKnownProperty,
    expected: ValueTextRejection<'_>,
) {
    match (result, expected) {
        (Err(CssPropertyValueSyntaxError::Property(error)), ValueTextRejection::Grammar(token)) => {
            assert_checked_rejected(Err(error.clone()), property, token);
        }
        (Err(CssPropertyValueSyntaxError::Property(error)), ValueTextRejection::Syntax(token)) => {
            let CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedToken(detail)) =
                error.kind()
            else {
                panic!("{property:?}: expected an unexpected token, got {error:?}")
            };
            assert_eq!(detail.encountered().authored(), token, "responsible token");
        }
        (Err(CssPropertyValueSyntaxError::Property(error)), ValueTextRejection::Closure) => {
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{property:?}: expected an unexpected end, got {error:?}"
            );
        }
        (
            Err(CssPropertyValueSyntaxError::Component(error)),
            ValueTextRejection::Component(kind),
        ) => {
            assert_eq!(error.kind(), kind, "{error:?}");
        }
        (actual, expected) => panic!("{property:?}: expected {expected:?}, got {actual:?}"),
    }
}

/// Asserts that checked numeric construction failed with `kind`, and
/// returns the failure for further origin checks.
#[track_caller]
pub fn assert_numeric_rejected<T: std::fmt::Debug>(
    result: Result<T, CssNumericConstructionError>,
    kind: CssNumericConstructionErrorKind,
) -> CssNumericConstructionError {
    let error = match result {
        Ok(value) => panic!("{kind:?}: numeric construction accepted {value:?}"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), &kind, "{error:?}");
    error
}

/// Asserts that checked construction of the single component `component`
/// failed with `kind` at that component's origin.
#[track_caller]
pub fn assert_component_rejected<T: std::fmt::Debug>(
    construct: impl FnOnce(CssComponentValue) -> Result<T, CssNumericConstructionError>,
    component: CssComponentValue,
    kind: CssNumericConstructionErrorKind,
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
        parse_property_value_for_grammar(
            grammar,
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        ),
        property,
        token,
    );
}
