#![forbid(unsafe_code)]
//! Existing-boundary regressions for signed authored thickness and unordered indent.
//!
//! Selected Text Decoration 4 WD 2022-05-04 §2.4 floors the actual thickness
//! at one device pixel; that downstream floor does not reject or clamp authored
//! negative lengths or percentages. Values 3 §8.1.2 and Values 4 §10.9 admit
//! mixed math with a dimensional percentage basis. Selected Text 4
//! WD 2026-08-14 §9.1 uses unordered `&&` for indent and its two flags.
//!
//! Independent browser witness: WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d,
//! Source/WebCore/css/CSSProperties.json (unrestricted thickness grammar),
//! Source/WebCore/style/values/text-decoration/StyleTextDecorationThickness.h,
//! and LayoutTests/imported/w3c/web-platform-tests/css/css-text-decor/
//! text-decoration-thickness-valid.html and its expected.txt admit -10px,
//! -49em, -27%, and mixed calculations. No new serializer API is used here.

use surgeist_css::*;

fn ordinary(component: &CssComponentValue, coefficient: &str, suffix: Option<&str>) {
    match (component.view(), suffix) {
        (CssComponentValueRef::Token(CssValueTokenRef::Number(number)), None) => {
            assert_eq!(number.representation(), coefficient);
        }
        (CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)), Some("%")) => {
            assert_eq!(number.representation(), coefficient);
        }
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }),
            Some(suffix),
        ) => {
            assert_eq!(number.representation(), coefficient);
            assert_eq!(unit, suffix);
        }
        _ => panic!("expected the retained ordinary scalar and its exact unit"),
    }
}

fn parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("expected original parsed scalar provenance")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find(token).unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + token.len()
    );
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
}

fn thickness(declaration: &CssDeclaration) -> &CssTextDecorationThickness {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TextDecorationThickness(value) => value.value(),
        CssKnownPropertyValueRef::TextDecoration(value) => value.value().thickness().unwrap(),
        _ => panic!("expected decoration thickness or decoration shorthand"),
    }
}

fn literal_thickness(declaration: &CssDeclaration, coefficient: &str, suffix: Option<&str>) {
    let CssTextDecorationThickness::Length(value) = thickness(declaration) else {
        panic!("expected authored numeric thickness")
    };
    ordinary(value.literal_component().unwrap(), coefficient, suffix);
}

fn programmatic(tokens: &[&str]) -> CssComponentValues {
    CssComponentValues::try_new(
        tokens
            .iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap()
}

fn checked_property(property: CssKnownProperty, components: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

fn checked_grammar(name: &str, components: CssComponentValues) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).unwrap(),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

fn assert_constructed(
    declaration: &CssDeclaration,
    property: CssKnownProperty,
    components: &CssComponentValues,
) {
    assert_eq!(declaration.known().unwrap().property(), property);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), components);
}

fn parsed_literal(token: &str, coefficient: &str, suffix: Option<&str>) {
    let source = format!(
        "/* 🦀 */ text-decoration-thickness:{token} !important; color:blue; text-decoration-thickness:2px"
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [negative, sibling, positive] = report.syntax().as_slice() else {
        panic!("all three source-order declarations must survive")
    };
    assert_eq!(
        negative.known().unwrap().property(),
        CssKnownProperty::TextDecorationThickness
    );
    assert_eq!(negative.importance(), CssImportance::Important);
    literal_thickness(negative, coefficient, suffix);
    let CssTextDecorationThickness::Length(value) = thickness(negative) else {
        unreachable!()
    };
    parsed_origin(value.origin(), &source, token);
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(sibling.importance(), CssImportance::Normal);
    assert_eq!(positive.importance(), CssImportance::Normal);
    literal_thickness(positive, "2", Some("px"));
}

fn validated_literal(token: &str) {
    let source = format!("text-decoration-thickness:{token} !important; color:blue");
    assert!(
        validate_style_attribute(&source).is_ok(),
        "a valid signed scalar must validate: {source}"
    );
}

fn constructed_literal(token: &str, coefficient: &str, suffix: Option<&str>, grammar: bool) {
    let components = programmatic(&[token]);
    let declaration = if grammar {
        checked_grammar("text-decoration-thickness", components.clone())
    } else {
        checked_property(
            CssKnownProperty::TextDecorationThickness,
            components.clone(),
        )
    };
    assert_constructed(
        &declaration,
        CssKnownProperty::TextDecorationThickness,
        &components,
    );
    literal_thickness(&declaration, coefficient, suffix);
    let CssTextDecorationThickness::Length(value) = thickness(&declaration) else {
        unreachable!()
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        value.literal_component().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
}

macro_rules! signed_literal_contract {
    ($parsed:ident, $validated:ident, $property:ident, $grammar:ident, $token:literal, $coefficient:literal, $suffix:expr) => {
        #[test]
        fn $parsed() {
            parsed_literal($token, $coefficient, $suffix);
        }
        #[test]
        fn $validated() {
            validated_literal($token);
        }
        #[test]
        fn $property() {
            constructed_literal($token, $coefficient, $suffix, false);
        }
        #[test]
        fn $grammar() {
            constructed_literal($token, $coefficient, $suffix, true);
        }
    };
}

signed_literal_contract!(
    negative_length_preserves_coefficients_importance_and_siblings,
    negative_length_validates,
    checked_property_admits_negative_length,
    checked_grammar_admits_negative_length,
    "-10px",
    "-10",
    Some("px")
);
signed_literal_contract!(
    negative_percentage_preserves_coefficients_importance_and_siblings,
    negative_percentage_validates,
    checked_property_admits_negative_percentage,
    checked_grammar_admits_negative_percentage,
    "-27%",
    "-27",
    Some("%")
);
signed_literal_contract!(
    tiny_negative_percentage_stays_exact_without_underflow_clamping,
    tiny_negative_percentage_validates,
    checked_property_admits_tiny_negative_percentage,
    checked_grammar_admits_tiny_negative_percentage,
    "-1e-999%",
    "-1e-999",
    Some("%")
);

fn assert_shorthand(declaration: &CssDeclaration) {
    let CssKnownPropertyValueRef::TextDecoration(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("decoration shorthand")
    };
    let value = value.value();
    assert_eq!(
        value.line().unwrap().components(),
        &[CssTextDecorationLineComponent::Underline]
    );
    assert_eq!(value.style(), Some(CssTextDecorationStyle::Wavy));
    assert_eq!(
        value.color().unwrap().keyword_srgba8(),
        Some([255, 0, 0, 255])
    );
    literal_thickness(declaration, "-10", Some("px"));
}

#[test]
fn shorthand_negative_thickness_retains_other_fields_importance_and_sibling() {
    let source = "/* 🦀 */ text-decoration:underline -10px wavy red !important; color:blue";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [decoration, sibling] = report.syntax().as_slice() else {
        panic!("both declarations")
    };
    assert_eq!(decoration.importance(), CssImportance::Important);
    assert_shorthand(decoration);
    let CssTextDecorationThickness::Length(value) = thickness(decoration) else {
        unreachable!()
    };
    parsed_origin(value.origin(), source, "-10px");
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(sibling.importance(), CssImportance::Normal);
}

#[test]
fn shorthand_negative_thickness_validates() {
    assert!(
        validate_style_attribute("text-decoration:underline -10px wavy red !important; color:blue")
            .is_ok()
    );
}

fn constructed_shorthand(grammar: bool) {
    let components = programmatic(&["underline", "-10px", "wavy", "red"]);
    let declaration = if grammar {
        checked_grammar("text-decoration", components.clone())
    } else {
        checked_property(CssKnownProperty::TextDecoration, components.clone())
    };
    assert_constructed(&declaration, CssKnownProperty::TextDecoration, &components);
    assert_shorthand(&declaration);
    let CssTextDecorationThickness::Length(value) = thickness(&declaration) else {
        unreachable!()
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn checked_property_admits_shorthand_negative_thickness() {
    constructed_shorthand(false);
}

#[test]
fn checked_grammar_admits_shorthand_negative_thickness() {
    constructed_shorthand(true);
}

#[test]
fn checked_reentry_preserves_supplied_negative_token_origins() {
    let source = "/* 🦀 */ -10px";
    let components = parse_component_values(source).unwrap();
    for declaration in [
        checked_property(
            CssKnownProperty::TextDecorationThickness,
            components.clone(),
        ),
        checked_grammar("text-decoration-thickness", components.clone()),
    ] {
        assert_constructed(
            &declaration,
            CssKnownProperty::TextDecorationThickness,
            &components,
        );
        let CssTextDecorationThickness::Length(value) = thickness(&declaration) else {
            unreachable!()
        };
        parsed_origin(value.origin(), source, "-10px");
        assert_eq!(value.literal_component().unwrap().origin(), value.origin());
    }
}

#[test]
fn zero_positive_and_intrinsic_keywords_stay_admitted() {
    for (token, coefficient, suffix) in [
        ("0", "0", None),
        ("0px", "0", Some("px")),
        ("-0px", "-0", Some("px")),
        ("0%", "0", Some("%")),
        ("2px", "2", Some("px")),
        ("17%", "17", Some("%")),
    ] {
        parsed_literal(token, coefficient, suffix);
        validated_literal(token);
        constructed_literal(token, coefficient, suffix, false);
        constructed_literal(token, coefficient, suffix, true);
    }
    for (token, expected) in [
        ("auto", CssTextDecorationThickness::Auto),
        ("from-font", CssTextDecorationThickness::FromFont),
    ] {
        let source = format!("text-decoration-thickness:{token}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(thickness(&report.syntax()[0]), &expected);
        assert!(validate_style_attribute(&source).is_ok());
        let components = programmatic(&[token]);
        for declaration in [
            checked_property(
                CssKnownProperty::TextDecorationThickness,
                components.clone(),
            ),
            checked_grammar("text-decoration-thickness", components.clone()),
        ] {
            assert_constructed(
                &declaration,
                CssKnownProperty::TextDecorationThickness,
                &components,
            );
            assert_eq!(thickness(&declaration), &expected);
        }
    }
}

fn admitted_calculation(text: &str) {
    let source = format!("text-decoration-thickness:{text} !important; color:blue");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration, sibling] = report.syntax().as_slice() else {
        panic!("both declarations")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let CssTextDecorationThickness::Length(value) = thickness(declaration) else {
        panic!("numeric thickness")
    };
    assert!(value.literal_component().is_none());
    let calculation = value.calculation().unwrap();
    parsed_origin(calculation.origin(), &source, "calc(");
    assert!(validate_style_attribute(&source).is_ok());
    let components = parse_component_values(text).unwrap();
    for checked in [
        checked_property(
            CssKnownProperty::TextDecorationThickness,
            components.clone(),
        ),
        checked_grammar("text-decoration-thickness", components.clone()),
    ] {
        assert_constructed(
            &checked,
            CssKnownProperty::TextDecorationThickness,
            &components,
        );
        let CssTextDecorationThickness::Length(value) = thickness(&checked) else {
            panic!("numeric thickness")
        };
        assert!(value.literal_component().is_none());
        parsed_origin(value.calculation().unwrap().origin(), text, "calc(");
    }
}

#[test]
fn negative_calculations_stay_symbolic_before_actual_value_flooring() {
    admitted_calculation("calc(-10px)");
    admitted_calculation("calc(-50em + 13px)");
}

#[test]
fn dimensional_percentage_hints_admit_mixed_thickness_calculations() {
    admitted_calculation("calc(40% - 20px)");
    admitted_calculation("calc(100% - 40em)");
}

#[test]
fn nonzero_unitless_thickness_rejects_the_declaration_and_preserves_sibling() {
    for property in ["text-decoration-thickness", "text-decoration"] {
        let source = format!("{property}:-10 !important; color:blue");
        let report = parse_style_attribute(&source);
        let [sibling] = report.syntax().as_slice() else {
            panic!("only valid sibling")
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
        );
        assert!(validate_style_attribute(&source).is_err());
        let components = programmatic(&["-10"]);
        let grammar = CssPropertyGrammar::from_name(property).unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(grammar.target_property()),
                components.clone(),
                CssImportance::Important
            )
            .is_err()
        );
        assert!(
            parse_property_value_for_grammar(grammar, components, CssImportance::Important)
                .is_err()
        );
    }
}

fn assert_indent(declaration: &CssDeclaration) {
    let CssKnownPropertyValueRef::TextIndent(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("indent")
    };
    let value = value.value();
    assert!(value.hanging());
    assert!(value.each_line());
    ordinary(
        value.length().literal_component().unwrap(),
        "-1e-999",
        Some("%"),
    );
}

#[test]
fn indent_prefix_flag_accepts_exact_signed_percentage_and_preserves_sibling() {
    let source = "/* 🦀 */ text-indent:each-line -1e-999% hanging !important; color:blue";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [indent, sibling] = report.syntax().as_slice() else {
        panic!("both declarations")
    };
    assert_eq!(indent.importance(), CssImportance::Important);
    assert_indent(indent);
    let CssKnownPropertyValueRef::TextIndent(value) =
        indent.known().unwrap().property_value().unwrap()
    else {
        unreachable!()
    };
    parsed_origin(value.value().length().origin(), source, "-1e-999%");
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
}

#[test]
fn indent_prefix_flag_validates_under_unordered_grammar() {
    assert!(
        validate_style_attribute("text-indent:each-line -1e-999% hanging !important; color:blue")
            .is_ok()
    );
}

fn constructed_indent(grammar: bool) {
    let components = programmatic(&["each-line", "-1e-999%", "hanging"]);
    let declaration = if grammar {
        checked_grammar("text-indent", components.clone())
    } else {
        checked_property(CssKnownProperty::TextIndent, components.clone())
    };
    assert_constructed(&declaration, CssKnownProperty::TextIndent, &components);
    assert_indent(&declaration);
    let CssKnownPropertyValueRef::TextIndent(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        value.value().length().origin(),
        &CssValueOrigin::Programmatic
    );
}

#[test]
fn checked_property_admits_indent_prefix_flag() {
    constructed_indent(false);
}

#[test]
fn checked_grammar_admits_indent_prefix_flag() {
    constructed_indent(true);
}
