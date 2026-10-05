#![forbid(unsafe_code)]
//! Functional coverage for the new specified-value API; no missing-symbol RED.
//! Selected Images 3 CRD 2023-12-18 §5.1 admits none and angle || flip, with
//! per-grammar canonical order. Quarter-turn rounding belongs to computed values.
//! Ordinary numeric formatting and math projection retain the shared Values/CSSOM owner.

use CssSpecifiedValueSerializationErrorKind as Kind;
use CssSpecifiedValueSerializationLimits as Limits;
use surgeist_css::*;

fn angle(coefficient: &str, unit: CssAngleUnit) -> CssAngleValue {
    CssAngleValue::from_literal(CssAngleLiteral::try_new(coefficient, unit).unwrap())
}

fn parsed(value: &str) -> (CssDeclaration, CssImageOrientation) {
    let source = format!("/* 🦀 */ image-orientation:{value} !important; color:blue");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration, sibling] = report.syntax().as_slice() else {
        panic!("orientation and sibling")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(sibling.importance(), CssImportance::Normal);
    let CssKnownPropertyValueRef::ImageOrientation(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed orientation")
    };
    (declaration.clone(), value.orientation().clone())
}

fn child(value: &CssImageOrientation) -> &CssAngleValue {
    match value {
        CssImageOrientation::Angle(angle) | CssImageOrientation::Flip(Some(angle)) => angle,
        _ => panic!("orientation with retained angle"),
    }
}

fn assert_parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed angle origin")
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

#[test]
fn explicit_keywords_omitted_angle_and_explicit_zero_keep_distinct_output_and_identity() {
    for (value, expected) in [
        (CssImageOrientation::None, "none"),
        (CssImageOrientation::FromImage, "from-image"),
        (CssImageOrientation::Flip(None), "flip"),
        (
            CssImageOrientation::Angle(angle("0", CssAngleUnit::Degrees)),
            "0deg",
        ),
        (
            CssImageOrientation::Flip(Some(angle("0", CssAngleUnit::Degrees))),
            "0deg flip",
        ),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let (_, reparsed) = parsed(expected);
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
        assert_eq!(value, reparsed);
    }
    assert_ne!(CssImageOrientation::None, CssImageOrientation::FromImage);
    assert_ne!(
        CssImageOrientation::None,
        CssImageOrientation::Angle(angle("0", CssAngleUnit::Degrees))
    );
    assert_ne!(
        CssImageOrientation::Flip(None),
        CssImageOrientation::Flip(Some(angle("0", CssAngleUnit::Degrees)))
    );
}

#[test]
fn ordinary_angles_use_shared_precision_and_units_without_quarter_turn_rounding_or_modulo() {
    for (coefficient, unit, expected) in [
        ("3.000e1", CssAngleUnit::Degrees, "30deg"),
        ("22.5000004", CssAngleUnit::Degrees, "22.5deg"),
        ("450", CssAngleUnit::Degrees, "450deg"),
        ("-0.25", CssAngleUnit::Turns, "-0.25turn"),
        ("1e2", CssAngleUnit::Gradians, "100grad"),
        ("1", CssAngleUnit::Radians, "1rad"),
    ] {
        let value = CssImageOrientation::Angle(angle(coefficient, unit));
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let literal = child(&value).literal().unwrap();
        assert_eq!(literal.numeric().representation(), coefficient);
        assert_eq!(literal.unit(), unit);
        assert_eq!(literal.origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn both_authored_flip_orders_emit_angle_then_flip_without_changing_source_provenance() {
    for value in ["flip 3.000e1deg", "3.000e1deg flip", r"f\6c ip 3.000e1deg"] {
        let (declaration, value) = parsed(value);
        let original = value.clone();
        let origin = child(&value).origin().clone();
        let components = declaration.value_components().clone();
        let source = declaration.parsed_name().unwrap().source().as_str();
        assert_parsed_origin(&origin, source, "3.000e1deg");
        assert_eq!(value.serialize_specified().unwrap(), "30deg flip");
        assert_eq!(value, original);
        assert_eq!(child(&value).origin(), &origin);
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(
            child(&value).literal().unwrap().numeric().representation(),
            "3.000e1"
        );
    }
}

#[test]
fn checked_property_and_grammar_reentry_preserve_supplied_angle_origins_during_serialization() {
    let source = "/* 🦀 */ flip -0.25turn";
    let components = parse_component_values(source).unwrap();
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
        assert!(declaration.position().is_none());
        assert!(declaration.parsed_name().is_none());
        assert!(declaration.parsed_value().is_none());
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert_eq!(declaration.value_components(), &components);
        let CssKnownPropertyValueRef::ImageOrientation(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("typed orientation")
        };
        let value = value.orientation();
        assert_parsed_origin(child(value).origin(), source, "-0.25turn");
        assert_eq!(value.serialize_specified().unwrap(), "-0.25turn flip");
        assert_parsed_origin(child(value).origin(), source, "-0.25turn");
    }
}

#[test]
fn angle_calculations_use_existing_numeric_projection_without_orientation_rounding() {
    for (authored, expected) in [
        ("calc(15deg + 15deg)", "calc(30deg)"),
        ("flip calc(15deg + 15deg)", "calc(30deg) flip"),
        ("calc(15deg + 15deg) flip", "calc(30deg) flip"),
        ("flip calc(1turn + 90deg)", "calc(450deg) flip"),
    ] {
        let (_, value) = parsed(authored);
        let calculation = child(&value).calculation().unwrap();
        let components = calculation.components().clone();
        let origin = calculation.origin().clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(100, 100, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(100, 100, expected.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(
            child(&value).calculation().unwrap().components(),
            &components
        );
        assert_eq!(child(&value).origin(), &origin);
    }
}

#[test]
fn carrier_transparent_keyword_and_literal_node_budgets_have_exact_boundaries() {
    for (value, expected, nodes) in [
        (CssImageOrientation::None, "none", 1),
        (CssImageOrientation::FromImage, "from-image", 1),
        (CssImageOrientation::Flip(None), "flip", 1),
        (
            CssImageOrientation::Angle(angle("3.000e1", CssAngleUnit::Degrees)),
            "30deg",
            1,
        ),
        (
            CssImageOrientation::Flip(Some(angle("3.000e1", CssAngleUnit::Degrees))),
            "30deg flip",
            2,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
}

#[test]
fn failure_after_angle_or_separator_is_atomic_and_preserves_exact_input_and_origin() {
    let (declaration, value) = parsed("flip 3.000e1deg");
    let before = value.clone();
    let origin = child(&value).origin().clone();
    let components = declaration.value_components().clone();
    for (limits, kind) in [
        (Limits::new(1, 2, 10), Kind::InputNodeLimit),
        (Limits::new(2, 1, 10), Kind::ProjectionNodeLimit),
        (Limits::new(2, 2, 6), Kind::ByteLimit),
        (Limits::new(2, 2, 9), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
        assert_eq!(child(&value).origin(), &origin);
        assert_eq!(declaration.value_components(), &components);
        assert_eq!(
            child(&value).literal().unwrap().numeric().representation(),
            "3.000e1"
        );
    }
    assert_eq!(value.serialize_specified().unwrap(), "30deg flip");
}

#[test]
fn tiny_nonzero_and_signed_zero_angles_remain_explicit_while_authored_coefficients_stay_exact() {
    for coefficient in ["-0", "1e-999", "-1e-999"] {
        let value = CssImageOrientation::Flip(Some(angle(coefficient, CssAngleUnit::Degrees)));
        assert_eq!(value.serialize_specified().unwrap(), "0deg flip");
        assert_eq!(
            child(&value).literal().unwrap().numeric().representation(),
            coefficient
        );
    }
}
