#![forbid(unsafe_code)]
//! Existing callable admission boundaries for selected UI4 WD 2026-01-20
//! §3.1 and §3.4: outline-color admits auto, and outline combines unordered
//! width, style and color, including separate auto style/color components.
//! The current expansion schema does not expose Outline contributions. Lone
//! auto controls therefore assert admission and provenance only, not the future
//! model's required both-style-and-color-auto interpretation or initial value.

use surgeist_css::*;

const AUTO_SPELLINGS: &[&str] = &["auto", "AuTo", r"\61 uto"];
const DISTINCT_AUTO_COMPONENTS: &[&str] = &[
    "auto auto",
    "auto solid",
    "solid auto",
    "auto none",
    "none auto",
    "2px auto auto",
    "auto 2px auto",
    "auto auto 2px",
    "2px solid auto",
    "2px auto solid",
    "solid 2px auto",
    "solid auto 2px",
    "auto 2px solid",
    "auto solid 2px",
];

fn name(property: CssKnownProperty) -> &'static str {
    match property {
        CssKnownProperty::Outline => "outline",
        CssKnownProperty::OutlineColor => "outline-color",
        _ => panic!("selected outline admission boundary"),
    }
}

fn position(actual: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(actual.byte_offset().value(), offset);
    assert_eq!(actual.line().value(), 0);
    assert_eq!(
        actual.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}

fn source(property: CssKnownProperty, value: &str) -> String {
    format!(
        "/* 🦀 */ color:red!important; {}:{value}!important; width:1px",
        name(property)
    )
}

fn assert_ordinary(declaration: &CssDeclaration, property: CssKnownProperty) {
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), property);
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    assert!(known.property_value().is_some());
    assert_eq!(declaration.importance(), CssImportance::Important);
}

fn rejection_code(kind: &ErrorKind, property: CssKnownProperty) -> CssErrorCode {
    match kind {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), property);
            CssErrorCode::InvalidPropertyValue
        }
        ErrorKind::InvalidColorSyntax(_) if property == CssKnownProperty::OutlineColor => {
            CssErrorCode::InvalidColorSyntax
        }
        _ => panic!("expected owning outline grammar rejection: {kind:?}"),
    }
}

fn parsed_admits(property: CssKnownProperty, value: &str) {
    let source = source(property, value);
    let report = parse_style_attribute(&source);
    assert!(
        report.is_clean(),
        "valid {}:{value}: {:?}",
        name(property),
        report.diagnostics()
    );
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("valid outline declaration and both neighbors must survive: {source}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    assert_ordinary(declaration, property);
    let authored = match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Outline(value) => value.as_css(),
        CssKnownPropertyValueRef::OutlineColor(value) => value.as_css(),
        _ => panic!("selected typed property wrapper"),
    };
    assert_eq!(authored, value);
    for item in [before, declaration, after] {
        let property = item.known().unwrap().property();
        let start = source
            .find(&format!("{}:", property.canonical_name()))
            .unwrap();
        position(item.position().unwrap(), &source, start);
        let name_origin = item.parsed_name().unwrap();
        let value_origin = item.parsed_value().unwrap();
        assert_eq!(name_origin.source().as_str(), source);
        assert_eq!(value_origin.source().as_str(), source);
        position(name_origin.span().start(), &source, start);
        position(
            name_origin.span().end(),
            &source,
            start + property.canonical_name().len(),
        );
    }
    let value_start =
        source.find(&format!("{}:", name(property))).unwrap() + name(property).len() + 1;
    let origin = declaration.parsed_value().unwrap();
    position(origin.span().start(), &source, value_start);
    position(origin.span().end(), &source, value_start + value.len());
}

fn validator_admits(property: CssKnownProperty, value: &str) {
    let source = source(property, value);
    let validated = validate_style_attribute(&source)
        .unwrap_or_else(|error| panic!("valid {}:{value}: {error:?}", name(property)));
    let report = parse_style_attribute(&source);
    assert!(report.is_clean());
    assert_eq!(&validated, report.syntax());
    let [_, declaration, _] = validated.as_slice() else {
        panic!("three valid declarations")
    };
    assert_ordinary(declaration, property);
}

fn checked_components_admit(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) {
    let declaration = if grammar {
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("valid checked {} value: {error:?}", name(property)));
    assert_ordinary(&declaration, property);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &components);
    for (retained, supplied) in declaration
        .value_components()
        .items()
        .iter()
        .zip(components.items())
    {
        assert_eq!(retained.origin(), supplied.origin());
    }
}

fn checked_admits(property: CssKnownProperty, value: &str, grammar: bool) {
    checked_components_admit(property, parse_component_values(value).unwrap(), grammar);
}

macro_rules! admission_contract {
    ($parsed:ident, $validated:ident, $property:ident, $grammar:ident, $known:expr, $values:expr) => {
        #[test]
        fn $parsed() {
            for value in $values {
                parsed_admits($known, value);
            }
        }
        #[test]
        fn $validated() {
            for value in $values {
                validator_admits($known, value);
            }
        }
        #[test]
        fn $property() {
            for value in $values {
                checked_admits($known, value, false);
            }
        }
        #[test]
        fn $grammar() {
            for value in $values {
                checked_admits($known, value, true);
            }
        }
    };
}

admission_contract!(
    outline_color_auto_parses_with_exact_source_and_neighbors,
    outline_color_auto_passes_strict_validation,
    checked_property_admits_outline_color_auto,
    checked_grammar_admits_outline_color_auto,
    CssKnownProperty::OutlineColor,
    AUTO_SPELLINGS
);
admission_contract!(
    separate_outline_auto_components_parse_in_every_represented_order,
    separate_outline_auto_components_pass_strict_validation,
    checked_property_admits_separate_outline_auto_components,
    checked_grammar_admits_separate_outline_auto_components,
    CssKnownProperty::Outline,
    DISTINCT_AUTO_COMPONENTS
);

#[test]
fn checked_outline_color_auto_preserves_supplied_escaped_token_origin() {
    let source = "/* 🦀 */ \\61 uto";
    let values = parse_component_values(source).unwrap();
    let auto = values.items().iter().find(|component| {
        matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == "auto")
    }).unwrap();
    let CssValueOrigin::Parsed(origin) = auto.origin() else {
        panic!("parsed auto token")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find('\\').unwrap();
    position(origin.span().start(), source, start);
    position(origin.span().end(), source, source.len());
    let supplied = CssComponentValues::try_new(vec![auto.clone()]).unwrap();
    for grammar in [false, true] {
        checked_components_admit(CssKnownProperty::OutlineColor, supplied.clone(), grammar);
    }
}

fn all_admit(property: CssKnownProperty, value: &str) {
    parsed_admits(property, value);
    validator_admits(property, value);
    checked_admits(property, value, false);
    checked_admits(property, value, true);
}

#[test]
fn lone_outline_auto_and_width_auto_controls_preserve_admission_and_lexical_order() {
    // Both style and color must ultimately be auto per §3.1. Admission alone
    // cannot prove that interpretation with the current Color-only payload.
    for value in ["auto", "AuTo", r"\61 uto", "auto 2px", "2px auto"] {
        all_admit(CssKnownProperty::Outline, value);
    }
}

#[test]
fn explicit_outline_color_style_width_controls_remain_unordered() {
    for value in [
        "2px solid red",
        "2px red solid",
        "solid 2px red",
        "solid red 2px",
        "red 2px solid",
        "red solid 2px",
        "auto red",
        "red auto",
    ] {
        all_admit(CssKnownProperty::Outline, value);
    }
}

#[test]
fn explicit_outline_color_alternatives_remain_admitted() {
    for value in ["red", "CURRENTcolor", "rgb(1 2 3)"] {
        all_admit(CssKnownProperty::OutlineColor, value);
    }
}

#[test]
fn duplicate_and_exclusive_outline_components_drop_only_the_invalid_declaration() {
    for (property, value) in [
        (CssKnownProperty::OutlineColor, "auto auto"),
        (CssKnownProperty::OutlineColor, "auto red"),
        (CssKnownProperty::Outline, "auto auto auto"),
        (CssKnownProperty::Outline, "auto auto red"),
        (CssKnownProperty::Outline, "auto auto solid"),
        (CssKnownProperty::Outline, "auto red blue"),
        (CssKnownProperty::Outline, "auto solid dashed"),
        (CssKnownProperty::Outline, "2px 3px auto"),
    ] {
        let source = source(property, value);
        let report = parse_style_attribute(&source);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only invalid outline drops: {source}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(before.importance(), CssImportance::Important);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
        assert_eq!(after.importance(), CssImportance::Normal);
        for declaration in [before, after] {
            let start = source
                .find(&format!(
                    "{}:",
                    declaration.known().unwrap().property().canonical_name()
                ))
                .unwrap();
            position(declaration.position().unwrap(), &source, start);
            assert_eq!(declaration.parsed_name().unwrap().source().as_str(), source);
            assert_eq!(
                declaration.parsed_value().unwrap().source().as_str(),
                source
            );
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid-property diagnostic")
        };
        assert_eq!(
            diagnostic.error().code(),
            rejection_code(diagnostic.error().kind(), property)
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = source.find(&format!("{}:", name(property))).unwrap();
        let end = start + source[start..].find(';').unwrap() + 1;
        position(diagnostic.span().start(), &source, start);
        position(diagnostic.span().end(), &source, end);
        let responsible = diagnostic.error().position();
        let offset = responsible.byte_offset().value();
        assert!((start..=end).contains(&offset));
        position(responsible, &source, offset);
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        for grammar in [false, true] {
            let components = parse_component_values(value).unwrap();
            let error = if grammar {
                parse_property_value_for_grammar(
                    property.grammar(),
                    components,
                    CssImportance::Important,
                )
            } else {
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components,
                    CssImportance::Important,
                )
            }
            .expect_err("exclusive or duplicate outline components");
            let CssPropertyValueErrorKind::Grammar(kind) = error.kind() else {
                panic!("typed known-property grammar failure: {error:?}")
            };
            rejection_code(kind, property);
        }
    }
}
