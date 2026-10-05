#![forbid(unsafe_code)]
//! Existing public admission boundaries for outline's exclusion of hidden.
//! Primary authority: selected UI 4 WD 2026-01-20 §3.3,
//! references/css-ui-4--WD-css-ui-4-20260120--42d9c51404fb.md:
//! <outline-line-style> excludes hidden while outline-style also admits auto.
//! UI 3 REC 2026-04-07 §4 corroborates the same prohibition.
//! Outline's shorthand includes its style; ordinary border-style still admits hidden.
//! No future outline payload or output-order API is assumed.

use surgeist_css::*;

const HIDDEN_SPELLINGS: &[&str] = &["hidden", "HIDDEN", r"h\69 dden"];

fn position(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}

fn property_name(property: CssKnownProperty) -> &'static str {
    match property {
        CssKnownProperty::Outline => "outline",
        CssKnownProperty::OutlineStyle => "outline-style",
        CssKnownProperty::BorderStyle => "border-style",
        _ => panic!("test property boundary"),
    }
}

fn parsed_rejects(property: CssKnownProperty, value: &str) {
    let name = property_name(property);
    let source = format!("/* 🦀 */ color:red !important; {name}:{value} !important; width:1px");
    let report = parse_style_attribute(&source);
    assert!(
        !report.is_clean(),
        "hidden is forbidden for {name}: {source}"
    );
    let [before, after] = report.syntax().as_slice() else {
        panic!("invalid outline must drop while both neighbors survive: {source}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    position(
        before.position().unwrap(),
        &source,
        source.find("color").unwrap(),
    );
    position(
        after.position().unwrap(),
        &source,
        source.find("width").unwrap(),
    );
    for sibling in [before, after] {
        assert_eq!(sibling.parsed_name().unwrap().source().as_str(), source);
        assert_eq!(sibling.parsed_value().unwrap().source().as_str(), source);
    }
    let [diagnostic] = report.diagnostics() else {
        panic!("one typed outline grammar diagnostic: {source}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("known-property grammar failure")
    };
    assert_eq!(detail.property(), property);
    let start = source.find(name).unwrap();
    let end = start + source[start..].find(';').unwrap() + 1;
    position(diagnostic.span().start(), &source, start);
    position(diagnostic.span().end(), &source, end);
    let responsible = diagnostic.error().position();
    let offset = responsible.byte_offset().value();
    assert!((start..=end).contains(&offset));
    position(responsible, &source, offset);
    let failure =
        validate_style_attribute(&source).expect_err("strict validator rejects recovered outline");
    assert_eq!(failure.diagnostics(), report.diagnostics());
}

fn checked_rejects(
    property: CssKnownProperty,
    value: &str,
    grammar: bool,
) -> CssPropertyValueParseError {
    let components = parse_component_values(value).expect("well-formed components");
    checked_components_rejects(property, components, grammar)
}

fn checked_components_rejects(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) -> CssPropertyValueParseError {
    let result = if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    };
    let error = result.expect_err("hidden is not an outline style");
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("typed outline grammar failure: {error:?}")
    };
    assert_eq!(detail.property(), property);
    error
}

macro_rules! hidden_contract {
    ($parsed:ident, $validated:ident, $property:ident, $grammar:ident, $known:expr, $value:literal) => {
        #[test]
        fn $parsed() {
            parsed_rejects($known, $value);
        }
        #[test]
        fn $validated() {
            let source = format!(
                "{}:{} !important; color:blue",
                property_name($known),
                $value
            );
            assert!(validate_style_attribute(&source).is_err(), "{source}");
        }
        #[test]
        fn $property() {
            checked_rejects($known, $value, false);
        }
        #[test]
        fn $grammar() {
            checked_rejects($known, $value, true);
        }
    };
}

hidden_contract!(
    hidden_outline_style_drops_with_typed_diagnostic_and_preserved_neighbors,
    validator_rejects_hidden_outline_style,
    checked_property_rejects_hidden_outline_style,
    checked_grammar_rejects_hidden_outline_style,
    CssKnownProperty::OutlineStyle,
    "hidden"
);
hidden_contract!(
    hidden_outline_shorthand_drops_as_one_declaration_with_preserved_neighbors,
    validator_rejects_hidden_outline_shorthand,
    checked_property_rejects_hidden_outline_shorthand,
    checked_grammar_rejects_hidden_outline_shorthand,
    CssKnownProperty::Outline,
    "2px hidden red"
);

#[test]
fn hidden_case_and_escape_spellings_reject_in_outline_longhand_and_shorthand() {
    for value in HIDDEN_SPELLINGS {
        for property in [CssKnownProperty::OutlineStyle, CssKnownProperty::Outline] {
            parsed_rejects(property, value);
            checked_rejects(property, value, false);
            checked_rejects(property, value, true);
        }
    }
    for value in ["hidden red 2px", "red hidden 2px", "2px red hidden"] {
        parsed_rejects(CssKnownProperty::Outline, value);
        checked_rejects(CssKnownProperty::Outline, value, false);
        checked_rejects(CssKnownProperty::Outline, value, true);
    }
}

fn original_hidden_origin(property: CssKnownProperty) {
    for spelling in HIDDEN_SPELLINGS {
        let source = format!("/* 🦀 */ {spelling}");
        let components = parse_component_values(&source).unwrap();
        let hidden = components.items().iter().find(|component| matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(ident)) if ident.eq_ignore_ascii_case("hidden"))).expect("decoded hidden identifier");
        let supplied = CssComponentValues::try_new(vec![hidden.clone()]).unwrap();
        for grammar in [false, true] {
            let error = checked_components_rejects(property, supplied.clone(), grammar);
            let origin = match error.origin() {
                CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => {
                    origin
                }
                _ => panic!("singleton rejection must map to its original hidden token"),
            };
            assert_eq!(origin, hidden.origin());
            let CssValueOrigin::Parsed(origin) = origin else {
                panic!("original component provenance")
            };
            assert_eq!(origin.source().as_str(), source);
            let start = source.find(spelling).unwrap();
            position(origin.span().start(), &source, start);
            position(origin.span().end(), &source, start + spelling.len());
        }
    }
}

#[test]
fn checked_outline_style_rejection_maps_to_supplied_hidden_token_source() {
    original_hidden_origin(CssKnownProperty::OutlineStyle);
}

#[test]
fn checked_outline_shorthand_rejection_maps_to_supplied_hidden_token_source() {
    original_hidden_origin(CssKnownProperty::Outline);
}

fn admitted(property: CssKnownProperty, value: &str) -> Vec<CssDeclaration> {
    let source = format!(
        "/* 🦀 */ color:red !important; {}:{value} !important; width:1px",
        property_name(property)
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("all valid occurrences survive")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    assert!(validate_style_attribute(&source).is_ok());
    let components = parse_component_values(value).unwrap();
    let by_property = parse_property_value(
        CssPropertyNameRef::Known(property),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let by_grammar = parse_property_value_for_grammar(
        property.grammar(),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    for checked in [&by_property, &by_grammar] {
        assert_eq!(checked.value_components(), &components);
        assert!(checked.position().is_none());
        assert!(checked.parsed_name().is_none());
        assert!(checked.parsed_value().is_none());
    }
    let declarations = vec![declaration.clone(), by_property, by_grammar];
    for declaration in &declarations {
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), property);
        assert!(known.global().is_none());
        assert!(known.substitution_dependent().is_none());
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
    declarations
}

#[test]
fn auto_none_and_solid_outline_controls_remain_admitted() {
    for (value, expected) in [
        ("auto", CssOutlineStyle::Auto),
        ("none", CssOutlineStyle::Border(CssBorderStyle::None)),
        ("solid", CssOutlineStyle::Border(CssBorderStyle::Solid)),
    ] {
        for property in [CssKnownProperty::OutlineStyle, CssKnownProperty::Outline] {
            for declaration in admitted(property, value) {
                match declaration.known().unwrap().property_value().unwrap() {
                    CssKnownPropertyValueRef::OutlineStyle(value) => {
                        assert_eq!(value.value(), &expected)
                    }
                    CssKnownPropertyValueRef::Outline(value) => {
                        assert_eq!(value.value().style(), Some(expected))
                    }
                    _ => panic!("typed outline value"),
                }
            }
        }
    }
    for declaration in admitted(CssKnownProperty::Outline, "2px solid red") {
        let CssKnownPropertyValueRef::Outline(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            value.value().style(),
            Some(CssOutlineStyle::Border(CssBorderStyle::Solid))
        );
        assert!(value.value().width().is_some());
        assert_eq!(
            value.value().color().unwrap().keyword_srgba8(),
            Some([255, 0, 0, 255])
        );
    }
}

#[test]
fn border_style_hidden_remains_accepted_through_all_front_doors() {
    for value in HIDDEN_SPELLINGS {
        for declaration in admitted(CssKnownProperty::BorderStyle, value) {
            let CssKnownPropertyValueRef::BorderStyle(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("typed border style")
            };
            assert_eq!(
                value.value().assigned_values(),
                [&CssBorderStyle::Hidden; 4]
            );
        }
    }
}

#[test]
fn duplicate_outline_components_and_unknown_styles_remain_invalid() {
    for (property, value) in [
        (CssKnownProperty::OutlineStyle, "solid solid"),
        (CssKnownProperty::OutlineStyle, "mystery"),
        (CssKnownProperty::Outline, "solid dashed"),
        (CssKnownProperty::Outline, "2px 3px solid"),
        (CssKnownProperty::Outline, "solid red blue"),
    ] {
        parsed_rejects(property, value);
        checked_rejects(property, value, false);
        checked_rejects(property, value, true);
    }
}
