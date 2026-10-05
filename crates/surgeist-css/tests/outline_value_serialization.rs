#![forbid(unsafe_code)]
//! New-model functional expectations from selected UI4 WD 2026-01-20 §3.1–3.4.
//! Canonical order is width/style/color; ambiguous lone auto sets both slots.
//! These tests cover the represented Auto/shared-Color subset, not image-1D.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn exact_limits(
    emit: impl Fn(Limits) -> Result<String, CssSpecifiedValueSerializationError>,
    expected: &str,
    input: usize,
    projection: usize,
) {
    assert_eq!(
        emit(Limits::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(input - 1, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input, projection - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
        assert_eq!(
            emit(Limits::new(input, projection, expected.len())).unwrap(),
            expected
        );
    }
}

fn declaration(property: CssKnownProperty, source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(source).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{property:?}:{source}: {error:?}"))
}

fn outline(source: &str) -> CssOutline {
    let declaration = declaration(CssKnownProperty::Outline, source);
    let CssKnownPropertyValueRef::Outline(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline")
    };
    value.value().clone()
}

fn color(source: &str) -> CssOutlineColor {
    let declaration = declaration(CssKnownProperty::OutlineColor, source);
    let CssKnownPropertyValueRef::OutlineColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline-color")
    };
    value.value().clone()
}

fn width(source: &str) -> CssOutlineWidth {
    let declaration = declaration(CssKnownProperty::OutlineWidth, source);
    let CssKnownPropertyValueRef::OutlineWidth(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline-width")
    };
    value.value().clone()
}

#[test]
fn every_semantic_outline_style_emits_one_checked_keyword() {
    for (value, expected) in [
        (CssOutlineStyle::Auto, "auto"),
        (CssOutlineStyle::None, "none"),
        (CssOutlineStyle::Dotted, "dotted"),
        (CssOutlineStyle::Dashed, "dashed"),
        (CssOutlineStyle::Solid, "solid"),
        (CssOutlineStyle::Double, "double"),
        (CssOutlineStyle::Groove, "groove"),
        (CssOutlineStyle::Ridge, "ridge"),
        (CssOutlineStyle::Inset, "inset"),
        (CssOutlineStyle::Outset, "outset"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
            1,
        );
        let declaration = declaration(
            CssKnownProperty::OutlineStyle,
            &expected.to_ascii_uppercase(),
        );
        let CssKnownPropertyValueRef::OutlineStyle(parsed) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("style")
        };
        assert_eq!(parsed.value(), &value);
    }
}

#[test]
fn outline_width_keywords_and_literal_lengths_share_numeric_limits() {
    for (value, expected) in [
        (CssOutlineWidth::Thin, "thin"),
        (CssOutlineWidth::Medium, "medium"),
        (CssOutlineWidth::Thick, "thick"),
        (width("2.0000000PX"), "2px"),
        (width("-0em"), "0em"),
    ] {
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
            1,
        );
        assert_eq!(value, before);
    }
}

#[test]
fn outline_color_auto_is_symbolic_initial_and_shared_color_is_transparent() {
    assert_eq!(CssOutlineColor::default(), CssOutlineColor::Auto);
    assert!(CssOutlineColor::Auto.color().is_none());
    for (value, expected) in [
        (CssOutlineColor::Auto, "auto"),
        (
            CssOutlineColor::Color(Box::new(CssColor::transparent())),
            "transparent",
        ),
        (color("CURRENTcolor"), "currentcolor"),
        (color("pUrPlE"), "purple"),
    ] {
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
            1,
        );
        assert_eq!(value, before);
    }
}

#[test]
fn all_optional_outline_field_subsets_keep_width_style_color_order() {
    let red = color("red");
    for (value, expected, nodes) in [
        (
            CssOutline::try_new(Some(CssOutlineWidth::Thin), None, None).unwrap(),
            "thin",
            2,
        ),
        (
            CssOutline::try_new(None, Some(CssOutlineStyle::Solid), None).unwrap(),
            "solid",
            2,
        ),
        (
            CssOutline::try_new(None, None, Some(red.clone())).unwrap(),
            "red",
            2,
        ),
        (
            CssOutline::try_new(
                Some(CssOutlineWidth::Thin),
                Some(CssOutlineStyle::Solid),
                None,
            )
            .unwrap(),
            "thin solid",
            3,
        ),
        (
            CssOutline::try_new(Some(CssOutlineWidth::Thin), None, Some(red.clone())).unwrap(),
            "thin red",
            3,
        ),
        (
            CssOutline::try_new(None, Some(CssOutlineStyle::Solid), Some(red.clone())).unwrap(),
            "solid red",
            3,
        ),
        (
            CssOutline::try_new(
                Some(CssOutlineWidth::Thin),
                Some(CssOutlineStyle::Solid),
                Some(red),
            )
            .unwrap(),
            "thin solid red",
            4,
        ),
    ] {
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            nodes,
            nodes,
        );
        assert_eq!(value, before);
    }
    assert!(CssOutline::try_new(None, None, None).is_none());
}

#[test]
fn constructed_color_only_auto_synthesizes_initial_none_style() {
    for (value, expected, input, projection) in [
        (
            CssOutline::try_new(None, None, Some(CssOutlineColor::Auto)).unwrap(),
            "none auto",
            2,
            3,
        ),
        (
            CssOutline::try_new(
                Some(CssOutlineWidth::Thin),
                None,
                Some(CssOutlineColor::Auto),
            )
            .unwrap(),
            "thin none auto",
            3,
            4,
        ),
    ] {
        let before = value.clone();
        assert!(value.style().is_none());
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            input,
            projection,
        );
        assert_eq!(value, before);
        let reparsed = outline(expected);
        assert_eq!(reparsed.style(), Some(CssOutlineStyle::None));
        assert_eq!(reparsed.color(), Some(&CssOutlineColor::Auto));
        assert_eq!(reparsed.width(), value.width());
    }
}

#[test]
fn ambiguous_parsed_auto_populates_both_semantic_fields_without_computation() {
    for (source, expected, nodes) in [
        ("auto", "auto", 3),
        ("AuTo", "auto", 3),
        (r"\61 uto", "auto", 3),
        ("auto auto", "auto", 3),
        ("auto 2px", "2px auto", 4),
        ("2px auto", "2px auto", 4),
        ("auto auto 2px", "2px auto", 4),
        ("2px auto auto", "2px auto", 4),
        ("auto 2px auto", "2px auto", 4),
    ] {
        let value = outline(source);
        let before = value.clone();
        assert_eq!(value.style(), Some(CssOutlineStyle::Auto));
        assert_eq!(value.color(), Some(&CssOutlineColor::Auto));
        assert_eq!(value.serialize_specified().unwrap(), expected);
        exact_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            nodes,
            nodes,
        );
        assert_eq!(value, before);
    }
}

#[test]
fn explicit_alternatives_assign_auto_to_the_available_semantic_slot() {
    for source in [
        "auto red",
        "red auto",
        "2px auto red",
        "2px red auto",
        "auto 2px red",
        "auto red 2px",
        "red 2px auto",
        "red auto 2px",
    ] {
        let value = outline(source);
        assert_eq!(value.style(), Some(CssOutlineStyle::Auto));
        assert_eq!(
            value.color().unwrap().color().unwrap().keyword_srgba8(),
            Some([255, 0, 0, 255])
        );
        assert_eq!(
            value.serialize_specified().unwrap(),
            if value.width().is_some() {
                "2px auto red"
            } else {
                "auto red"
            }
        );
    }
    for source in [
        "auto solid",
        "solid auto",
        "2px auto solid",
        "2px solid auto",
        "auto 2px solid",
        "auto solid 2px",
        "solid 2px auto",
        "solid auto 2px",
    ] {
        let value = outline(source);
        assert_eq!(value.style(), Some(CssOutlineStyle::Solid));
        assert_eq!(value.color(), Some(&CssOutlineColor::Auto));
        assert_eq!(
            value.serialize_specified().unwrap(),
            if value.width().is_some() {
                "2px solid auto"
            } else {
                "solid auto"
            }
        );
    }
}

#[test]
fn explicit_initial_none_with_auto_color_does_not_gain_an_auto_style() {
    let value = CssOutline::try_new(
        None,
        Some(CssOutlineStyle::None),
        Some(CssOutlineColor::Auto),
    )
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "none auto");
    exact_limits(
        |limits| value.serialize_specified_with_limits(limits),
        "none auto",
        3,
        3,
    );
    for source in ["none auto", "auto none"] {
        let parsed = outline(source);
        assert_eq!(parsed, value);
    }
}

#[test]
fn parsed_shorthand_retains_lexical_components_importance_and_both_numeric_origins() {
    let source = "calc(1px + 2px) solid rgb(calc(2 + 3) 0 0)";
    let components = parse_component_values(source).unwrap();
    let retained_components = components.clone();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Outline),
        components,
        CssImportance::Important,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Outline(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline")
    };
    let value = wrapper.value();
    let before = value.clone();
    let CssOutlineWidth::Length(length) = value.width().unwrap() else {
        panic!("checked length")
    };
    let width_origin = length.origin().clone();
    let CssColorComponent::NumberCalculation(channel) = &value
        .color()
        .unwrap()
        .color()
        .unwrap()
        .rgb_value()
        .unwrap()
        .channels()[0]
    else {
        panic!("color math")
    };
    let color_origin = channel.origin().clone();
    assert!(matches!(&width_origin, CssValueOrigin::Parsed(_)));
    assert!(matches!(&color_origin, CssValueOrigin::Parsed(_)));
    for (limits, kind) in [
        (Limits::new(0, 262_144, 1_048_576), Kind::InputNodeLimit),
        (Limits::new(65_536, 0, 1_048_576), Kind::ProjectionNodeLimit),
        (Limits::new(65_536, 262_144, 0), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            value.serialize_specified().unwrap(),
            "calc(3px) solid rgb(5, 0, 0)"
        );
    }
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.value_components(), &retained_components);
    assert_eq!(length.origin(), &width_origin);
    assert_eq!(channel.origin(), &color_origin);
    assert_eq!(value, &before);
}

#[test]
fn parsed_auto_fields_keep_the_original_escaped_value_and_neighbors() {
    let source = r"/* 🦀 */ color:red; outline:2px \61 uto!important; width:1px";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("all three declarations")
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    let CssKnownPropertyValueRef::Outline(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("outline")
    };
    assert_eq!(wrapper.as_css(), r"2px \61 uto");
    assert_eq!(wrapper.value().style(), Some(CssOutlineStyle::Auto));
    assert_eq!(wrapper.value().color(), Some(&CssOutlineColor::Auto));
    assert_eq!(wrapper.value().serialize_specified().unwrap(), "2px auto");
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.parsed_name().unwrap().source().as_str(), source);
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );
    assert_eq!(&validate_style_attribute(source).unwrap(), report.syntax());
}
