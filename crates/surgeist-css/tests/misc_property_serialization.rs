#![forbid(unsafe_code)]
//! Independent specified-value expectations from UI3 caret-color, Containment 1
//! grammar order and Compositing 1's ordered background-blend-mode list.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn assert_limits(
    emit: impl Fn(Limits) -> Result<String, CssSpecifiedValueSerializationError>,
    expected: &str,
    nodes: usize,
) {
    assert_eq!(
        emit(Limits::new(nodes, nodes, expected.len())).unwrap(),
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
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
        assert_eq!(
            emit(Limits::new(nodes, nodes, expected.len())).unwrap(),
            expected
        );
    }
}

fn parsed_caret(source: &str) -> CssCaretColor {
    let report = parse_style_attribute(&format!("caret-color:{source}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::CaretColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("caret-color")
    };
    value.caret().clone()
}

#[test]
fn caret_keyword_and_color_are_transparent_owners_with_exact_limits() {
    for (value, expected) in [
        (CssCaretColor::Auto, "auto"),
        (
            CssCaretColor::Color(Box::new(CssColor::transparent())),
            "transparent",
        ),
        (parsed_caret("CURRENTcolor"), "currentcolor"),
        (parsed_caret("pUrPlE"), "purple"),
    ] {
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
        );
        assert_eq!(value, before);
    }
}

#[test]
fn caret_math_and_authored_numeric_origins_survive_failure_and_retry() {
    let value = parsed_caret("rgb(calc(100 * 4) 127 calc(20 - 35))");
    let before = value.clone();
    let CssCaretColor::Color(color) = &value else {
        panic!("color")
    };
    let CssColorComponent::NumberCalculation(first) = &color.rgb_value().unwrap().channels()[0]
    else {
        panic!("calculation")
    };
    let origin = first.origin().clone();
    assert!(matches!(&origin, CssValueOrigin::Parsed(_)));
    assert_eq!(value.serialize_specified().unwrap(), "rgb(255, 127, 0)");
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
        assert_eq!(value.serialize_specified().unwrap(), "rgb(255, 127, 0)");
    }
    assert_eq!(value, before);
    assert_eq!(first.origin(), &origin);
}

#[test]
fn containment_keywords_preserve_their_authored_branch_and_exact_limits() {
    for (value, expected) in [
        (CssContain::None, "none"),
        (CssContain::Strict, "strict"),
        (CssContain::Content, "content"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
        );
    }
}

#[test]
fn every_containment_subset_and_order_emits_grammar_order_without_mutation() {
    use CssContainComponent::{Layout, Paint, Size};
    for (components, expected) in [
        (vec![Size], "size"),
        (vec![Layout], "layout"),
        (vec![Paint], "paint"),
        (vec![Size, Layout], "size layout"),
        (vec![Layout, Size], "size layout"),
        (vec![Size, Paint], "size paint"),
        (vec![Paint, Size], "size paint"),
        (vec![Layout, Paint], "layout paint"),
        (vec![Paint, Layout], "layout paint"),
        (vec![Size, Layout, Paint], "size layout paint"),
        (vec![Size, Paint, Layout], "size layout paint"),
        (vec![Layout, Size, Paint], "size layout paint"),
        (vec![Layout, Paint, Size], "size layout paint"),
        (vec![Paint, Size, Layout], "size layout paint"),
        (vec![Paint, Layout, Size], "size layout paint"),
    ] {
        let value =
            CssContain::Components(CssContainComponentList::try_new(components.clone()).unwrap());
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1 + components.len(),
        );
        assert_eq!(value, before);
        let CssContain::Components(retained) = value else {
            panic!("components")
        };
        assert_eq!(retained.components(), components);
    }
}

#[test]
fn containment_checked_construction_and_parsed_order_remain_intact() {
    assert!(CssContainComponentList::try_new(vec![]).is_none());
    assert!(
        CssContainComponentList::try_new(vec![
            CssContainComponent::Size,
            CssContainComponent::Size
        ])
        .is_none()
    );
    let report = parse_style_attribute("contain:paint size layout!important");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::Contain(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("contain")
    };
    let before = value.containment().clone();
    assert_eq!(
        value.containment().serialize_specified().unwrap(),
        "size layout paint"
    );
    assert_eq!(value.containment(), &before);
}

#[test]
fn every_blend_primitive_is_available_in_an_ordered_list_with_exact_limits() {
    for (mode, expected) in [
        (CssBlendMode::Normal, "normal"),
        (CssBlendMode::Darken, "darken"),
        (CssBlendMode::Multiply, "multiply"),
        (CssBlendMode::ColorBurn, "color-burn"),
        (CssBlendMode::Lighten, "lighten"),
        (CssBlendMode::Screen, "screen"),
        (CssBlendMode::ColorDodge, "color-dodge"),
        (CssBlendMode::Overlay, "overlay"),
        (CssBlendMode::SoftLight, "soft-light"),
        (CssBlendMode::HardLight, "hard-light"),
        (CssBlendMode::Difference, "difference"),
        (CssBlendMode::Exclusion, "exclusion"),
        (CssBlendMode::Hue, "hue"),
        (CssBlendMode::Saturation, "saturation"),
        (CssBlendMode::Color, "color"),
        (CssBlendMode::Luminosity, "luminosity"),
    ] {
        let value = CssBlendModeList::try_new(vec![mode]).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_limits(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            2,
        );
        assert_eq!(value.modes(), &[mode]);
    }
}

#[test]
fn blend_lists_preserve_order_duplicates_and_comma_byte_cost() {
    assert!(CssBlendModeList::try_new(vec![]).is_none());
    let value = CssBlendModeList::try_new(vec![
        CssBlendMode::Screen,
        CssBlendMode::Normal,
        CssBlendMode::Screen,
    ])
    .unwrap();
    let before = value.clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "screen, normal, screen"
    );
    assert_limits(
        |limits| value.serialize_specified_with_limits(limits),
        "screen, normal, screen",
        4,
    );
    assert_eq!(value, before);
    let report = parse_style_attribute("background-blend-mode:SCREEN,normal,SCREEN");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BackgroundBlendMode(parsed) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("background-blend-mode")
    };
    assert_eq!(parsed.modes(), &value);
    assert_eq!(
        parsed.modes().serialize_specified().unwrap(),
        "screen, normal, screen"
    );
}
