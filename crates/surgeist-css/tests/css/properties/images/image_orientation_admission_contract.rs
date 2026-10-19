#![forbid(unsafe_code)]
//! Existing public admission boundaries for Images 3 image-orientation.
//! Independent authority: selected Images 3 CRD 2023-12-18 §5.1,
//! references/css-images-3--CRD-css-images-3-20231218--74eb4b475aa0.md:
//! `from-image | none | [ <angle> || flip ]` admits either component order.
//! Quarter-turn rounding is a computed-value operation; authored angles remain exact.
//! These tests require no future keyword variant or serialization API.

use surgeist_css::*;

const FLIP_FIRST: &[(&str, &str, &str, CssAngleUnit)] = &[
    (
        "flip 3.000e1deg",
        "3.000e1deg",
        "3.000e1",
        CssAngleUnit::Degrees,
    ),
    ("FLIP -0.25turn", "-0.25turn", "-0.25", CssAngleUnit::Turns),
    (r"f\6c ip 1e2grad", "1e2grad", "1e2", CssAngleUnit::Gradians),
];
const NONE: &[&str] = &["none", "NONE", r"n\6f ne"];
const INVALID: &[&str] = &[
    "flip flip",
    "flip 30deg flip",
    "30deg flip flip",
    "30deg 60deg",
    "flip 30deg 60deg",
    "none flip",
    "none 30deg",
    "from-image flip",
    "from-image 30deg",
    "30px",
    "30%",
];

fn position(position: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(position.byte_offset().value(), offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}

fn parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("expected the angle's original parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find(token).unwrap();
    position(origin.span().start(), source, start);
    position(origin.span().end(), source, start + token.len());
}

fn orientation(declaration: &CssDeclaration) -> &CssImageOrientation {
    let known = declaration
        .known()
        .expect("known image-orientation occurrence");
    assert_eq!(known.property(), CssKnownProperty::ImageOrientation);
    assert_eq!(
        known.grammar(),
        CssKnownProperty::ImageOrientation.grammar()
    );
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let CssKnownPropertyValueRef::ImageOrientation(value) = known.property_value().unwrap() else {
        panic!("typed image-orientation value")
    };
    value.orientation()
}

fn flip_angle<'a>(
    declaration: &'a CssDeclaration,
    coefficient: &str,
    unit: CssAngleUnit,
) -> &'a CssAngleValue {
    let CssImageOrientation::Flip(Some(angle)) = orientation(declaration) else {
        panic!("flip and its authored angle must both be retained")
    };
    let literal = angle.literal().expect("exact ordinary angle branch");
    assert_eq!(literal.numeric().representation(), coefficient);
    assert_eq!(literal.unit(), unit);
    angle
}

fn parsed_accepts(value: &str) -> CssDeclaration {
    let source =
        format!("/* 🦀 */ color:red !important; image-orientation:{value} !important; width:1px");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("all three declarations must survive in source order")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    assert_eq!(declaration.importance(), CssImportance::Important);
    orientation(declaration);
    position(
        declaration.position().unwrap(),
        &source,
        source.find("image-orientation").unwrap(),
    );
    let region = declaration.parsed_value().unwrap();
    assert_eq!(region.source().as_str(), source);
    let span = region.span();
    assert_eq!(
        source[span.start().byte_offset().value()..span.end().byte_offset().value()].trim(),
        value
    );
    declaration.clone()
}

fn checked_accepts(value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).expect("well-formed supplied components");
    let declaration = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::ImageOrientation.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ImageOrientation),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{value}: {error:?}"));
    orientation(&declaration);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &components);
    declaration
}

#[test]
fn flip_before_angle_retains_exact_angle_importance_provenance_and_siblings() {
    for &(value, token, coefficient, unit) in FLIP_FIRST {
        let declaration = parsed_accepts(value);
        let angle = flip_angle(&declaration, coefficient, unit);
        let source = declaration.parsed_name().unwrap().source().as_str();
        parsed_origin(angle.origin(), source, token);
        assert_eq!(
            angle.literal().unwrap().component().origin(),
            angle.origin()
        );
    }
}

#[test]
fn validator_admits_flip_before_angle() {
    for &(value, _, _, _) in FLIP_FIRST {
        assert!(
            validate_style_attribute(&format!("image-orientation:{value} !important; color:blue"))
                .is_ok(),
            "{value}"
        );
    }
}

#[test]
fn checked_property_admits_flip_before_angle_and_preserves_supplied_angle_origin() {
    for &(value, token, coefficient, unit) in FLIP_FIRST {
        let declaration = checked_accepts(value, false);
        parsed_origin(
            flip_angle(&declaration, coefficient, unit).origin(),
            value,
            token,
        );
    }
}

#[test]
fn checked_grammar_admits_flip_before_angle_and_preserves_supplied_angle_origin() {
    for &(value, token, coefficient, unit) in FLIP_FIRST {
        let declaration = checked_accepts(value, true);
        parsed_origin(
            flip_angle(&declaration, coefficient, unit).origin(),
            value,
            token,
        );
    }
}

#[test]
fn none_is_retained_as_an_ordinary_known_value_with_importance_and_siblings() {
    for value in NONE {
        parsed_accepts(value);
    }
}

#[test]
fn validator_admits_none_keyword_case_and_escape_spellings() {
    for value in NONE {
        assert!(
            validate_style_attribute(&format!("image-orientation:{value} !important; color:blue"))
                .is_ok(),
            "{value}"
        );
    }
}

#[test]
fn checked_property_admits_none_without_fabricated_declaration_provenance() {
    for value in NONE {
        checked_accepts(value, false);
    }
}

#[test]
fn checked_grammar_admits_none_without_fabricated_declaration_provenance() {
    for value in NONE {
        checked_accepts(value, true);
    }
}

#[test]
fn programmatic_flip_before_angle_retains_programmatic_scalar_origin() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("flip").unwrap(),
        CssComponentValue::try_token("3.000e1deg").unwrap(),
    ])
    .unwrap();
    for declaration in [
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ImageOrientation),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap(),
        parse_property_value_for_grammar(
            CssKnownProperty::ImageOrientation.grammar(),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap(),
    ] {
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert!(declaration.position().is_none());
        assert_eq!(
            flip_angle(&declaration, "3.000e1", CssAngleUnit::Degrees).origin(),
            &CssValueOrigin::Programmatic
        );
    }
}

#[test]
fn from_image_flip_only_angle_first_and_angle_only_remain_accepted() {
    for value in [
        "from-image",
        "flip",
        "3.000e1deg flip",
        "3.000e1deg",
        "-0.25turn",
        "1rad",
    ] {
        let parsed = parsed_accepts(value);
        let property = checked_accepts(value, false);
        let grammar = checked_accepts(value, true);
        assert!(validate_style_attribute(&format!("image-orientation:{value}")).is_ok());
        for declaration in [&parsed, &property, &grammar] {
            match value {
                "from-image" => assert!(matches!(
                    orientation(declaration),
                    CssImageOrientation::FromImage
                )),
                "flip" => assert!(matches!(
                    orientation(declaration),
                    CssImageOrientation::Flip(None)
                )),
                "3.000e1deg flip" => {
                    flip_angle(declaration, "3.000e1", CssAngleUnit::Degrees);
                }
                _ => assert!(matches!(
                    orientation(declaration),
                    CssImageOrientation::Angle(_)
                )),
            }
        }
    }
}

fn parsed_rejects(value: &str) {
    let source =
        format!("/* 🦀 */ color:red !important; image-orientation:{value} !important; width:1px");
    let report = parse_style_attribute(&source);
    let [before, after] = report.syntax().as_slice() else {
        panic!("only valid neighboring declarations must survive: {source}")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(before.importance(), CssImportance::Important);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert_eq!(after.importance(), CssImportance::Normal);
    let [diagnostic] = report.diagnostics() else {
        panic!("one image-orientation grammar failure: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed property grammar diagnostic")
    };
    assert_eq!(detail.property(), CssKnownProperty::ImageOrientation);
    let start = source.find("image-orientation").unwrap();
    let end = start + source[start..].find(';').unwrap() + 1;
    position(diagnostic.span().start(), &source, start);
    position(diagnostic.span().end(), &source, end);
}

fn checked_rejects(value: &str, grammar: bool) {
    let components = parse_component_values(value).unwrap();
    let result = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::ImageOrientation.grammar(),
            components,
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ImageOrientation),
            components,
            CssImportance::Important,
        )
    };
    let error = result.expect_err("duplicate or non-angle orientation must be rejected");
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("typed checked property grammar failure: {error:?}")
    };
    assert_eq!(detail.property(), CssKnownProperty::ImageOrientation);
}

#[test]
fn duplicate_exclusive_and_wrong_dimension_values_drop_only_the_invalid_declaration() {
    for value in INVALID {
        parsed_rejects(value);
    }
}

#[test]
fn validator_rejects_duplicate_exclusive_and_wrong_dimension_values() {
    for value in INVALID {
        assert!(
            validate_style_attribute(&format!("image-orientation:{value}; color:blue")).is_err(),
            "{value}"
        );
    }
}

#[test]
fn checked_property_rejects_duplicate_exclusive_and_wrong_dimension_values() {
    for value in INVALID {
        checked_rejects(value, false);
    }
}

#[test]
fn checked_grammar_rejects_duplicate_exclusive_and_wrong_dimension_values() {
    for value in INVALID {
        checked_rejects(value, true);
    }
}
