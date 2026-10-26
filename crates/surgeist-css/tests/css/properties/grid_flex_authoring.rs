//! Grid Layout 2 and Flexbox 1 properties keep authored track, line and flex
//! syntax, and reject invalid grids.

use crate::css::support::value_assertions::positive_literal;
use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn parses_grid_track_lists_and_template_areas() {
    let columns = declaration(
        ".panel { grid-template-columns: [main] repeat(2, minmax(10px, 1fr)) fit-content(20%); }",
        CssKnownProperty::GridTemplateColumns,
    );
    let Some(CssKnownPropertyValueRef::GridTemplateColumns(value)) =
        columns.known().unwrap().property_value()
    else {
        panic!("grid columns")
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "[main] repeat(2, minmax(10px, 1fr)) fit-content(20%)"
    );
    let areas = declaration(
        ".panel { grid-template-areas: \"header header\" \"nav main\"; }",
        CssKnownProperty::GridTemplateAreas,
    );
    let Some(CssKnownPropertyValueRef::GridTemplateAreas(value)) =
        areas.known().unwrap().property_value()
    else {
        panic!("grid areas")
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "\"header header\" \"nav main\""
    );
    let none = declaration(
        ".panel { grid-template-areas: none; }",
        CssKnownProperty::GridTemplateAreas,
    );
    let Some(CssKnownPropertyValueRef::GridTemplateAreas(value)) =
        none.known().unwrap().property_value()
    else {
        panic!("grid areas none")
    };
    assert_eq!(value.value(), &CssGridTemplateAreas::None);
}

#[test]
fn parses_grid_flow_lines_and_shorthands() {
    let flow = declaration(
        ".panel { grid-auto-flow: column dense; }",
        CssKnownProperty::GridAutoFlow,
    );
    let Some(CssKnownPropertyValueRef::GridAutoFlow(value)) =
        flow.known().unwrap().property_value()
    else {
        panic!("grid flow")
    };
    assert_eq!(
        *value.value(),
        CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true)
    );
    for (property, css, expected) in [
        (
            CssKnownProperty::GridRowStart,
            ".panel { grid-row-start: span 2 main; }",
            "span 2 main",
        ),
        (
            CssKnownProperty::GridColumn,
            ".panel { grid-column: nav / span 3; }",
            "nav / span 3",
        ),
        (
            CssKnownProperty::GridArea,
            ".panel { grid-area: header / 1 / span 2 / main; }",
            "header / 1 / span 2 / main",
        ),
    ] {
        let declaration = declaration(css, property);
        let value = declaration.known().unwrap().property_value().unwrap();
        let serialized = match value {
            CssKnownPropertyValueRef::GridRowStart(value) => value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            CssKnownPropertyValueRef::GridColumn(value) => value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            CssKnownPropertyValueRef::GridArea(value) => value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            _ => panic!("grid placement"),
        };
        assert_eq!(serialized, expected);
    }
}

#[test]
fn parses_grid_template_and_grid_shorthands() {
    let template = declaration(
        ".panel { grid-template: 100px 1fr / repeat(2, minmax(10px, 1fr)); }",
        CssKnownProperty::GridTemplate,
    );
    let Some(CssKnownPropertyValueRef::GridTemplate(value)) =
        template.known().unwrap().property_value()
    else {
        panic!("grid template")
    };
    assert_eq!(
        value
            .value()
            .rows()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "100px 1fr"
    );
    assert_eq!(
        value
            .value()
            .columns()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "repeat(2, minmax(10px, 1fr))"
    );
    let grid = declaration(
        ".panel { grid: auto-flow dense 12px / repeat(auto-fit, 10px); }",
        CssKnownProperty::Grid,
    );
    let Some(CssKnownPropertyValueRef::Grid(value)) = grid.known().unwrap().property_value() else {
        panic!("grid")
    };
    assert_eq!(
        value.value().auto_flow(),
        Some(CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Row, true))
    );
    assert_eq!(
        value
            .value()
            .auto_tracks()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "12px"
    );
    assert_eq!(
        value
            .value()
            .explicit_tracks()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "repeat(auto-fit, 10px)"
    );
}

#[test]
fn parses_order_flex_and_content_alignment() {
    let order = property_payload!(
        declaration(".panel { order: -2; }", CssKnownProperty::Order),
        Order,
        value
    );
    let Some(integer) = order.literal() else {
        panic!("exact order integer")
    };
    assert_eq!(integer.numeric().representation(), "-2");
    let flex = property_payload!(
        declaration(".panel { flex: 2 0 10rem; }", CssKnownProperty::Flex),
        Flex,
        value
    );
    let CssFlexValue::Components(components) = flex else {
        panic!("authored flex components")
    };
    assert_eq!(
        number_spelling(
            components
                .grow()
                .unwrap()
                .literal()
                .map(|literal| literal.component())
                .unwrap()
        ),
        "2"
    );
    assert_eq!(
        number_spelling(
            components
                .shrink()
                .unwrap()
                .literal()
                .map(|literal| literal.component())
                .unwrap()
        ),
        "0"
    );
    let CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length))) =
        components.basis().unwrap().view()
    else {
        panic!("length flex basis")
    };
    assert_length_literal(
        length.literal().map(|literal| literal.component()).unwrap(),
        "10",
        "rem",
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { flex: none; }", CssKnownProperty::Flex),
            Flex,
            value
        ),
        CssFlexValue::None
    );
    assert_eq!(
        property_payload!(
            declaration(".panel { flex: auto; }", CssKnownProperty::Flex),
            Flex,
            value
        ),
        CssFlexValue::Auto
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { align-content: space-evenly; }",
                CssKnownProperty::AlignContent
            ),
            AlignContent,
            value
        )
        .value(),
        CssAlignmentValue::SpaceEvenly
    );
    assert_eq!(
        property_payload!(
            declaration(
                ".panel { justify-content: center; }",
                CssKnownProperty::JustifyContent
            ),
            JustifyContent,
            value
        )
        .value(),
        CssAlignmentValue::Position {
            overflow: None,
            position: CssAlignmentPosition::Center
        }
    );
}

#[test]
fn grid_and_flex_property_families_accept_supported_values() {
    let sheet = parse_sheet(
        ".panel {
            grid-template-rows: [top] 100px 1fr;
            grid-template-columns: repeat(2, minmax(10px, 1fr));
            grid-template-areas: \"header header\" \"nav main\";
            grid-template: 100px / 1fr 2fr;
            grid-auto-rows: minmax(10px, auto);
            grid-auto-columns: fit-content(20%);
            grid-auto-flow: row dense;
            grid-row-start: 1;
            grid-row-end: span 2;
            grid-column-start: nav;
            grid-column-end: auto;
            grid-row: 1 / span 2;
            grid-column: nav / main;
            grid-area: header / nav / main / 4;
            grid: auto-flow 12px / repeat(auto-fill, 10px);
            order: 2;
            flex: 1 1 auto;
            justify-content: space-between;
            align-content: stretch;
        }",
    )
    .expect("clean parse report");
    let declarations = style_rule(&sheet.rules()[0]).declarations();

    for property in [
        CssKnownProperty::GridTemplateRows,
        CssKnownProperty::GridTemplateColumns,
        CssKnownProperty::GridTemplateAreas,
        CssKnownProperty::GridTemplate,
        CssKnownProperty::GridAutoRows,
        CssKnownProperty::GridAutoColumns,
        CssKnownProperty::GridAutoFlow,
        CssKnownProperty::GridRowStart,
        CssKnownProperty::GridRowEnd,
        CssKnownProperty::GridColumnStart,
        CssKnownProperty::GridColumnEnd,
        CssKnownProperty::GridRow,
        CssKnownProperty::GridColumn,
        CssKnownProperty::GridArea,
        CssKnownProperty::Grid,
        CssKnownProperty::Order,
        CssKnownProperty::Flex,
        CssKnownProperty::JustifyContent,
        CssKnownProperty::AlignContent,
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
fn grid_and_flex_properties_reject_cross_family_values() {
    for input in [
        ".panel { order: 1.2; }",
        ".panel { grid-auto-flow: left; }",
        ".panel { grid-template-areas: \"a a\" \"a .\"; }",
        ".panel { grid-row: 1 / / 2; }",
        ".panel { flex: solid; }",
        ".panel { justify-content: auto; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}

#[test]
fn checked_grid_constructors_reject_parser_invalid_states() {
    assert_eq!(CssCustomIdent::try_new(""), None);
    assert_eq!(
        CssGridLineName::try_new(CssIdent::try_new("auto").unwrap()),
        None
    );
    assert_eq!(CssGridLineNames::new(Vec::new()).names(), &[]);
    assert_eq!(CssGridGeneralTrackList::try_new(Vec::new()), None);
    assert_eq!(
        positive_literal(CssNumericLiteral::<CssInteger>::from_i32(0)),
        None
    );
    assert_eq!(CssGridTrackSizeList::try_new(Vec::new()), None);
    assert!(CssGridTemplateAreas::try_rows(Vec::new()).is_err());
    assert_eq!(
        CssGridLine::try_indexed(
            CssNumeric::<CssInteger>::try_from_literal(CssNumericLiteral::<CssInteger>::from_i32(
                0
            ))
            .unwrap(),
            None
        ),
        None
    );
    assert_eq!(CssGridLine::try_span(None, None), None);
}

#[test]
fn rejects_grid_auto_flow_shorthand_without_explicit_tracks() {
    for input in [
        ".panel { grid: auto-flow; }",
        ".panel { grid: auto-flow dense; }",
        ".panel { grid: auto-flow 12px; }",
    ] {
        assert_sheet_rejected(
            input,
            &ExpectedErrorKind::InvalidPropertyValue {
                property: declared_property(input),
            },
        );
    }
}

#[test]
fn invalid_parser_custom_ident_errors_keep_source_location() {
    let error = parse_sheet(".panel {\n  grid-template-columns: [auto] 1fr;\n}")
        .expect_err("rejected parse report");

    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
    assert_eq!(error.position().line().value(), 1);
    assert_ne!(error.position().column().value(), 0);
}

#[test]
fn rejects_inconsistent_grid_template_area_row_widths() {
    let error = parse_sheet(".panel { grid-template-areas: \"a a\" \"b\"; }")
        .expect_err("rejected parse report");

    assert!(error.code() == CssErrorCode::InvalidPropertyValue);
}
