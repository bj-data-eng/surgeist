//! Box model, spacing, borders, radii and shadows keep authored syntax, and
//! their checked constructors reject invalid states.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn assert_percentage_literal(component: &CssComponentValue, expected_number: &str) {
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view() else {
        panic!("expected exact percentage token");
    };
    assert_eq!(number.representation(), expected_number);
}

#[test]
fn rejects_padding_auto() {
    let error = CssParser::default()
        .parse_sheet(".panel { padding: auto; }")
        .expect_err("rejected parse report");
    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
}

#[test]
fn rejects_border_width_percent() {
    let error = CssParser::default()
        .parse_sheet(".panel { border-width: 10%; }")
        .expect_err("rejected parse report");
    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
}

#[test]
fn rejects_gap_auto() {
    let error = CssParser::default()
        .parse_sheet(".panel { gap: auto; }")
        .expect_err("rejected parse report");
    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
}

#[test]
fn accepts_margin_auto() {
    let margin = property_payload!(
        declaration(".panel { margin: auto; }", CssKnownProperty::Margin),
        Margin,
        value
    );
    assert!(
        margin
            .assigned()
            .iter()
            .all(|value| matches!(value, CssMarginValue::Auto))
    );
}

#[test]
fn parses_spacing_inset_and_z_index_values() {
    let inset = property_payload!(
        declaration(".panel { inset: auto 10px 5%; }", CssKnownProperty::Inset),
        Inset,
        value
    );
    assert_eq!(inset.authored().len(), 3);
    let [top, right, bottom, left] = inset.assigned();
    assert_eq!(top, &CssInsetValue::Auto);
    assert_length_literal(
        right
            .length_percentage()
            .unwrap()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "10",
        "px",
    );
    assert_percentage_literal(
        bottom
            .length_percentage()
            .unwrap()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "5",
    );
    assert_length_literal(
        left.length_percentage()
            .unwrap()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "10",
        "px",
    );
    let top = property_payload!(
        declaration(".panel { top: calc(10px + 5%); }", CssKnownProperty::Top),
        Top,
        value
    );
    let top = top.length_percentage().unwrap().calculation().unwrap();
    assert_eq!(top.result_type(), CssCalculationType::LengthPercentage);
    assert_eq!(
        top.serialize_tokens(Default::default()).unwrap().as_css(),
        "calc(10px + 5%)"
    );
    let CssZIndexValue::Integer(value) = property_payload!(
        declaration(".panel { z-index: -2; }", CssKnownProperty::ZIndex),
        ZIndex,
        value
    ) else {
        panic!("integer z-index")
    };
    assert_integer_literal(&value, "-2");
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { box-decoration-break: clone; }",
                CssKnownProperty::BoxDecorationBreak
            ),
            BoxDecorationBreak,
            value
        ),
        CssBoxDecorationBreak::Clone
    );
}

#[test]
fn parses_spacing_longhands_with_existing_component_rules() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { margin-left: auto; }",
                CssKnownProperty::MarginLeft
            ),
            MarginLeft,
            value
        ),
        CssMarginValue::Auto
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { padding-top: 12px; }",
                CssKnownProperty::PaddingTop
            ),
            PaddingTop,
            value
        )
        .serialize_specified(Default::default())
        .unwrap(),
        "12px"
    );
    let width = property_payload!(
        declaration(
            ".panel { border-right-width: 2px; }",
            CssKnownProperty::BorderRightWidth
        ),
        BorderRightWidth,
        value
    );
    let CssLineWidth::Length(length) = width else {
        panic!("exact border width")
    };
    assert_length_literal(
        length.literal().map(|literal| literal.component()).unwrap(),
        "2",
        "px",
    );
}

#[test]
fn parses_border_style_and_border_shorthand_values() {
    let styles = property_payload!(
        declaration(
            ".panel { border-style: solid dashed; }",
            CssKnownProperty::BorderStyle
        ),
        BorderStyle,
        value
    );
    assert_eq!(
        styles.authored(),
        &[CssLineStyle::Solid, CssLineStyle::Dashed]
    );
    assert_eq!(
        styles.assigned(),
        [
            &CssLineStyle::Solid,
            &CssLineStyle::Dashed,
            &CssLineStyle::Solid,
            &CssLineStyle::Dashed
        ]
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { border-left-style: groove; }",
                CssKnownProperty::BorderLeftStyle
            ),
            BorderLeftStyle,
            value
        ),
        CssLineStyle::Groove
    );
    let border = property_payload!(
        declaration(
            ".panel { border: solid 2px #fff; }",
            CssKnownProperty::Border
        ),
        Border,
        value
    );
    assert_eq!(border.style(), Some(CssLineStyle::Solid));
    assert_eq!(
        border
            .width()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "2px"
    );
    assert_eq!(border.color().unwrap().hex_value().unwrap().digits(), "fff");
    assert_eq!(
        border.serialize_specified(Default::default()).unwrap(),
        "2px solid rgb(255, 255, 255)"
    );

    let top = property_payload!(
        declaration(
            ".panel { border-top: black dotted; }",
            CssKnownProperty::BorderTop
        ),
        BorderTop,
        value
    );
    assert!(top.width().is_none());
    assert_eq!(top.style(), Some(CssLineStyle::Dotted));
    assert_eq!(top.color().unwrap().named().unwrap().name(), "black");
}

#[test]
fn parses_border_radius_shorthand_and_longhands() {
    let corner = property_payload!(
        declaration(
            ".panel { border-top-left-radius: 4px 10%; }",
            CssKnownProperty::BorderTopLeftRadius
        ),
        BorderTopLeftRadius,
        value
    );
    assert_length_literal(
        corner
            .first()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "4",
        "px",
    );
    assert_percentage_literal(
        corner
            .authored_second()
            .unwrap()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "10",
    );
    assert_percentage_literal(
        corner
            .second()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        "10",
    );
    let radii = property_payload!(
        declaration(
            ".panel { border-radius: 1px 2px 3px / 4px 5px; }",
            CssKnownProperty::BorderRadius
        ),
        BorderRadius,
        value
    );
    assert_eq!(radii.horizontal_values().len(), 3);
    assert_eq!(radii.authored_vertical_values().unwrap().len(), 2);
    for (corner, horizontal, vertical) in [
        (radii.top_left(), "1", "4"),
        (radii.top_right(), "2", "5"),
        (radii.bottom_right(), "3", "4"),
        (radii.bottom_left(), "2", "5"),
    ] {
        assert_length_literal(
            corner
                .first()
                .literal()
                .map(|literal| literal.component())
                .unwrap(),
            horizontal,
            "px",
        );
        assert_length_literal(
            corner
                .authored_second()
                .unwrap()
                .literal()
                .map(|literal| literal.component())
                .unwrap(),
            vertical,
            "px",
        );
        assert_length_literal(
            corner
                .second()
                .literal()
                .map(|literal| literal.component())
                .unwrap(),
            vertical,
            "px",
        );
    }
}

#[test]
fn parses_box_shadow_none_and_shadow_lists() {
    assert_eq!(
        property_payload!(
            declaration(".panel { box-shadow: none; }", CssKnownProperty::BoxShadow),
            BoxShadow,
            value
        ),
        CssBoxShadow::None
    );

    let value = property_payload!(
        declaration(
            ".panel { box-shadow: inset 1px 2px 3px 4px black, 0 1px #fff; }",
            CssKnownProperty::BoxShadow
        ),
        BoxShadow,
        value
    );

    let CssBoxShadow::Shadows(shadows) = value else {
        panic!("expected box-shadow list");
    };
    assert_eq!(shadows.shadows().len(), 2);
    assert_eq!(
        shadows.shadows()[0],
        CssShadow::try_new(
            true,
            signed_length("1px"),
            signed_length("2px"),
            Some(nonnegative_length("3px")),
            Some(signed_length("4px")),
            Some(CssColor::from_named(
                CssNamedColor::try_new("black").unwrap()
            )),
        )
        .unwrap()
    );
    assert_eq!(
        shadows.shadows()[1],
        CssShadow::try_new(
            false,
            signed_length("0"),
            signed_length("1px"),
            None,
            None,
            Some(CssColor::from_hex(CssHexColor::try_new("fff").unwrap())),
        )
        .unwrap()
    );
}

#[test]
fn checked_border_constructor_rejects_empty_shorthands() {
    assert_eq!(CssBorder::try_new(None, None, None), None);
    let border = CssBorder::try_new(None, Some(CssLineStyle::Solid), None).unwrap();
    assert!(border.width().is_none());
    assert_eq!(border.style(), Some(CssLineStyle::Solid));
    assert!(border.color().is_none());
}

#[test]
fn checked_border_width_rejects_negative_and_percentage_values() {
    for invalid in ["-1px", "10%", "auto", "min-content"] {
        assert!(
            CssNumeric::<CssLength, CssNonNegative>::try_from_component(
                CssComponentValue::try_token(invalid).unwrap()
            )
            .is_err(),
            "{invalid}"
        );
    }
    let width = CssNumeric::<CssLength, CssNonNegative>::try_from_component(
        CssComponentValue::try_token("1px").unwrap(),
    )
    .unwrap();
    let border = CssBorder::try_new(
        Some(CssLineWidth::Length(width)),
        Some(CssLineStyle::Solid),
        None,
    )
    .unwrap();
    assert_eq!(
        border
            .width()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "1px"
    );
}

#[test]
fn checked_corner_radius_constructor_rejects_parser_invalid_values() {
    for css in [
        "auto",
        "min-content",
        "max-content",
        "fit-content",
        "normal",
        "-1px",
        "-1%",
    ] {
        assert!(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
                CssComponentValue::try_token(css).unwrap()
            )
            .is_err(),
            "{css}"
        );
    }
    for css in ["-1px", "-1%"] {
        let calculation = CssCalculation::<CssLengthPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), css).unwrap(),
            Default::default(),
        )
        .unwrap();
        assert!(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(calculation)
                .is_err()
        );
    }
    let radius = CssSidePair::new(
        nonnegative_length_percentage("1px"),
        Some(nonnegative_length_percentage("25%")),
    );
    assert!(exact_dimension(
        radius.first().literal().map(|literal| literal.component()),
        "1",
        "px"
    ));
    assert!(exact_percentage(
        radius.second().literal().map(|literal| literal.component()),
        "25"
    ));
}

#[test]
fn checked_shadow_constructor_rejects_invalid_pairings_and_lengths() {
    assert!(
        CssNumeric::<CssLength>::try_from_component(CssComponentValue::try_token("auto").unwrap())
            .is_err()
    );
    assert!(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension("-3", "px").unwrap()
        )
        .is_err()
    );
    assert!(
        CssShadow::try_new(
            false,
            signed_length("1px"),
            signed_length("2px"),
            None,
            Some(signed_length("4px")),
            None
        )
        .is_none()
    );
    let shadow = CssShadow::try_new(
        false,
        signed_length("-1px"),
        signed_length("2px"),
        Some(nonnegative_length("3px")),
        Some(signed_length("-4px")),
        None,
    )
    .unwrap();
    assert!(exact_dimension(
        shadow
            .offset_x()
            .literal()
            .map(|literal| literal.component()),
        "-1",
        "px"
    ));
    assert!(exact_dimension(
        shadow
            .offset_y()
            .literal()
            .map(|literal| literal.component()),
        "2",
        "px"
    ));
    assert!(exact_dimension(
        shadow
            .blur_radius()
            .unwrap()
            .literal()
            .map(|literal| literal.component()),
        "3",
        "px"
    ));
    assert!(exact_dimension(
        shadow
            .spread_radius()
            .unwrap()
            .literal()
            .map(|literal| literal.component()),
        "-4",
        "px"
    ));
}

#[test]
fn box_model_property_families_accept_supported_values() {
    let sheet = CssParser::default()
        .parse_sheet(
            ".panel {
            inset: auto 1px 2%;
            top: auto;
            right: 1px;
            bottom: 2%;
            left: calc(3px + 4%);
            z-index: 7;
            box-decoration-break: slice;
            margin-top: auto;
            margin-right: 1px;
            margin-bottom: 2%;
            margin-left: calc(3px + 4%);
            padding-top: 1px;
            padding-right: 2%;
            padding-bottom: calc(3px + 4%);
            padding-left: 0;
            border: 1px solid black;
            border-top: solid;
            border-right: 1px;
            border-bottom: #fff;
            border-left: dashed black;
            border-top-width: 1px;
            border-right-width: 2px;
            border-bottom-width: 3px;
            border-left-width: 4px;
            border-top-color: black;
            border-right-color: white;
            border-bottom-color: transparent;
            border-left-color: #fff;
            border-style: none hidden dotted dashed;
            border-top-style: solid;
            border-right-style: double;
            border-bottom-style: ridge;
            border-left-style: outset;
            border-radius: 1px 2px / 3px 4px;
            border-top-left-radius: 1px;
            border-top-right-radius: 1px 2px;
            border-bottom-right-radius: 10%;
            border-bottom-left-radius: calc(1px + 2%);
            box-shadow: 1px 2px;
        }",
        )
        .expect("clean parse report");
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssKnownProperty::Inset,
        CssKnownProperty::Top,
        CssKnownProperty::Right,
        CssKnownProperty::Bottom,
        CssKnownProperty::Left,
        CssKnownProperty::ZIndex,
        CssKnownProperty::BoxDecorationBreak,
        CssKnownProperty::MarginTop,
        CssKnownProperty::MarginRight,
        CssKnownProperty::MarginBottom,
        CssKnownProperty::MarginLeft,
        CssKnownProperty::PaddingTop,
        CssKnownProperty::PaddingRight,
        CssKnownProperty::PaddingBottom,
        CssKnownProperty::PaddingLeft,
        CssKnownProperty::Border,
        CssKnownProperty::BorderTop,
        CssKnownProperty::BorderRight,
        CssKnownProperty::BorderBottom,
        CssKnownProperty::BorderLeft,
        CssKnownProperty::BorderTopWidth,
        CssKnownProperty::BorderRightWidth,
        CssKnownProperty::BorderBottomWidth,
        CssKnownProperty::BorderLeftWidth,
        CssKnownProperty::BorderTopColor,
        CssKnownProperty::BorderRightColor,
        CssKnownProperty::BorderBottomColor,
        CssKnownProperty::BorderLeftColor,
        CssKnownProperty::BorderStyle,
        CssKnownProperty::BorderTopStyle,
        CssKnownProperty::BorderRightStyle,
        CssKnownProperty::BorderBottomStyle,
        CssKnownProperty::BorderLeftStyle,
        CssKnownProperty::BorderRadius,
        CssKnownProperty::BorderTopLeftRadius,
        CssKnownProperty::BorderTopRightRadius,
        CssKnownProperty::BorderBottomRightRadius,
        CssKnownProperty::BorderBottomLeftRadius,
        CssKnownProperty::BoxShadow,
    ] {
        assert!(
            declarations.iter().any(|declaration| declaration
                .known()
                .map(|known| known.property())
                == Some(property)),
            "missing parsed declaration for {property:?}",
        );
    }
}

#[test]
fn non_negative_box_model_properties_reject_negative_lengths() {
    for input in [
        ".panel { border-radius: -1px; }",
        ".panel { padding-top: -1px; }",
        ".panel { border-width: -1px; }",
        ".panel { box-shadow: 1px 2px -3px; }",
    ] {
        let error = CssParser::default().parse_sheet(input).expect_err(input);
        assert!(error.code() == CssErrorCode::InvalidPropertyValue);
    }
}

#[test]
fn box_model_properties_reject_cross_family_values() {
    for input in [
        ".panel { padding-top: auto; }",
        ".panel { border-width: 10%; }",
        ".panel { border-style: 10px; }",
        ".panel { border-radius: auto; }",
        ".panel { box-shadow: auto; }",
        ".panel { z-index: 1.5; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
    // A <color> value is rejected by the color grammar itself.
    assert_sheet_rejected(
        ".panel { border-color: solid; }",
        &ExpectedErrorKind::InvalidColorSyntax {
            encountered: "solid",
        },
    );
}
