//! Positioning, visibility and Box Alignment 3 keyword families keep their
//! authored values and reject family leakage.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef};
use surgeist_css::values::box_model::{
    CssClear, CssContentVisibility, CssFloat, CssLayoutPosition, CssVisibility,
};
use surgeist_css::values::layout::{
    CssAlignmentPosition, CssAlignmentValue, CssBaselinePosition, CssOverflowPosition,
};

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn invalid_display_keyword_is_typed_with_property_context() {
    let error = CssParser::default()
        .parse_sheet(".panel { display: solid; }")
        .expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {:?}",
        error
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Display);
}

#[test]
fn unsupported_overflow_keyword_is_typed_with_property_context() {
    let error = CssParser::default()
        .parse_sheet(".panel { overflow: none; }")
        .expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {:?}",
        error
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Overflow);
}

#[test]
fn unsupported_position_keyword_is_typed_with_property_context() {
    let error = CssParser::default()
        .parse_sheet(".panel { position: running; }")
        .expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {:?}",
        error
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Position);
}

#[test]
fn unsupported_alignment_keyword_is_typed_with_property_context() {
    let error = CssParser::default()
        .parse_sheet(".panel { align-items: unsafe stretch; }")
        .expect_err("rejected parse report");

    let detail = &error;
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "unexpected error kind: {:?}",
        error
    );
    assert_eq!(
        detail.known_property().unwrap(),
        CssKnownProperty::AlignItems
    );
}

#[test]
fn parses_position_float_clear_visibility_values() {
    assert_eq!(
        property_payload!(
            declaration(".panel { position: static; }", CssKnownProperty::Position),
            Position,
            value
        ),
        CssLayoutPosition::Static
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { position: fixed; }", CssKnownProperty::Position),
            Position,
            value
        ),
        CssLayoutPosition::Fixed
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { position: sticky; }", CssKnownProperty::Position),
            Position,
            value
        ),
        CssLayoutPosition::Sticky
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { float: left; }", CssKnownProperty::Float),
            Float,
            value
        ),
        CssFloat::Left
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { clear: both; }", CssKnownProperty::Clear),
            Clear,
            value
        ),
        CssClear::Both
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { visibility: collapse; }",
                CssKnownProperty::Visibility
            ),
            Visibility,
            value
        ),
        CssVisibility::Collapse
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { content-visibility: auto; }",
                CssKnownProperty::ContentVisibility
            ),
            ContentVisibility,
            value
        ),
        CssContentVisibility::Auto
    );
}

#[test]
fn parses_content_alignment_and_place_shorthands() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { align-content: space-between; }",
                CssKnownProperty::AlignContent
            ),
            AlignContent,
            value
        )
        .value(),
        CssAlignmentValue::SpaceBetween
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { justify-content: safe center; }",
                CssKnownProperty::JustifyContent
            ),
            JustifyContent,
            value
        )
        .value(),
        CssAlignmentValue::Position {
            overflow: Some(CssOverflowPosition::Safe),
            position: CssAlignmentPosition::Center
        }
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { align-items: first baseline; }",
                CssKnownProperty::AlignItems
            ),
            AlignItems,
            value
        )
        .value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::First)
    );
    let content = property_payload!(
        declaration(
            ".panel { place-content: center end; }",
            CssKnownProperty::PlaceContent
        ),
        PlaceContent,
        value
    );
    assert_eq!(
        content.align().value(),
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::Center
        }
    );
    assert_eq!(
        content.justify().value(),
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::End
        }
    );
    let items = property_payload!(
        declaration(
            ".panel { place-items: stretch; }",
            CssKnownProperty::PlaceItems
        ),
        PlaceItems,
        value
    );
    assert_eq!(items.align().value(), CssAlignmentValue::Stretch);
    assert_eq!(items.justify().value(), CssAlignmentValue::Stretch);
    let own = property_payload!(
        declaration(
            ".panel { place-self: end center; }",
            CssKnownProperty::PlaceSelf
        ),
        PlaceSelf,
        value
    );
    assert_eq!(
        own.align().value(),
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::End
        }
    );
    assert_eq!(
        own.justify().value(),
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::Center
        }
    );
}

#[test]
fn preserves_explicit_safe_alignment_values() {
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { align-items: safe end; }",
                CssKnownProperty::AlignItems
            ),
            AlignItems,
            value
        )
        .value(),
        CssAlignmentValue::Position {
            overflow: Some(CssOverflowPosition::Safe),
            position: CssAlignmentPosition::End
        }
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { align-self: safe flex-end; }",
                CssKnownProperty::AlignSelf
            ),
            AlignSelf,
            value
        )
        .value(),
        CssAlignmentValue::Position {
            overflow: Some(CssOverflowPosition::Safe),
            position: CssAlignmentPosition::FlexEnd
        }
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { justify-content: safe center; }",
                CssKnownProperty::JustifyContent
            ),
            JustifyContent,
            value
        )
        .value(),
        CssAlignmentValue::Position {
            overflow: Some(CssOverflowPosition::Safe),
            position: CssAlignmentPosition::Center
        }
    );
}

#[test]
fn rejects_positioning_alignment_and_visibility_leakage_values() {
    let cases = [
        (
            ".panel { float: center; }",
            Rejection::property("float", Some("center")),
        ),
        (
            ".panel { clear: start; }",
            Rejection::property("clear", Some("start")),
        ),
        (
            ".panel { align-content: left; }",
            Rejection::property("align-content", Some("left")),
        ),
        (
            ".panel { justify-content: auto; }",
            Rejection::property("justify-content", Some("auto")),
        ),
        (
            ".panel { place-items: auto; }",
            Rejection::property("place-items", Some("auto")),
        ),
        (
            ".panel { place-items: space-between; }",
            Rejection::property("place-items", Some("space-between")),
        ),
        (
            ".panel { visibility: auto; }",
            Rejection::property("visibility", Some("auto")),
        ),
        (
            ".panel { content-visibility: collapse; }",
            Rejection::property("content-visibility", Some("collapse")),
        ),
    ];

    for (case, rejection) in cases {
        assert_rejection(case, rejection);
    }
}

#[test]
fn safe_prefixed_alignment_positions_are_admitted_but_stretch_is_rejected() {
    let accepted = [
        ".panel { align-items: safe start; }",
        ".panel { align-items: safe flex-start; }",
        ".panel { align-content: safe start; }",
        ".panel { align-content: safe flex-start; }",
        ".panel { place-content: safe start; }",
        ".panel { place-content: safe flex-start; }",
    ];
    for case in accepted {
        assert!(
            CssParser::default().parse_sheet(case).is_clean(),
            "{case} should be admitted"
        );
    }
    let rejected = [
        (
            ".panel { align-items: safe stretch; }",
            Rejection::property("align-items", Some("stretch")),
        ),
        (
            ".panel { align-content: safe stretch; }",
            Rejection::property("align-content", Some("stretch")),
        ),
        (
            ".panel { place-content: safe stretch; }",
            Rejection::property("place-content", Some("stretch")),
        ),
    ];

    for (case, rejection) in rejected {
        assert_rejection(case, rejection);
    }
}
