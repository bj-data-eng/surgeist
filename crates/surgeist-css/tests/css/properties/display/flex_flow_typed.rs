#![forbid(unsafe_code)]

//! Public construction and bounded specified serialization for Flexbox 1
//! §§5.1–5.3. The direction and wrap keywords stay symbolic until layout.

use surgeist_css::*;

const PROGRAMMATIC_FLOW: CssFlexFlow =
    CssFlexFlow::new(CssFlexDirection::ColumnReverse, CssFlexWrap::Wrap);

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

#[test]
fn all_direction_keywords_serialize_as_their_specified_identifiers() {
    for (direction, keyword) in [
        (CssFlexDirection::Row, "row"),
        (CssFlexDirection::RowReverse, "row-reverse"),
        (CssFlexDirection::Column, "column"),
        (CssFlexDirection::ColumnReverse, "column-reverse"),
    ] {
        assert_eq!(direction.serialize_specified().unwrap(), keyword);
        let source = declaration(&format!("flex-direction:{keyword}"));
        let CssKnownPropertyValueRef::FlexDirection(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed direction: {keyword}")
        };
        assert_eq!(value.value(), &direction);
    }
}

#[test]
fn all_wrap_keywords_serialize_as_their_specified_identifiers() {
    for (wrap, keyword) in [
        (CssFlexWrap::NoWrap, "nowrap"),
        (CssFlexWrap::Wrap, "wrap"),
        (CssFlexWrap::WrapReverse, "wrap-reverse"),
    ] {
        assert_eq!(wrap.serialize_specified().unwrap(), keyword);
        let source = declaration(&format!("flex-wrap:{keyword}"));
        let CssKnownPropertyValueRef::FlexWrap(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed wrap: {keyword}")
        };
        assert_eq!(value.value(), &wrap);
    }
}

#[test]
fn every_typed_flow_pair_is_constructible_and_serializes_direction_first() {
    for (direction, direction_text) in [
        (CssFlexDirection::Row, "row"),
        (CssFlexDirection::RowReverse, "row-reverse"),
        (CssFlexDirection::Column, "column"),
        (CssFlexDirection::ColumnReverse, "column-reverse"),
    ] {
        for (wrap, wrap_text) in [
            (CssFlexWrap::NoWrap, "nowrap"),
            (CssFlexWrap::Wrap, "wrap"),
            (CssFlexWrap::WrapReverse, "wrap-reverse"),
        ] {
            let flow = CssFlexFlow::new(direction, wrap);
            assert_eq!(flow.direction(), direction);
            assert_eq!(flow.wrap(), wrap);
            let canonical = format!("{direction_text} {wrap_text}");
            assert_eq!(flow.serialize_specified().unwrap(), canonical);

            let source = declaration(&format!("flex-flow:{canonical}"));
            let CssKnownPropertyValueRef::FlexFlow(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("parsed flow: {canonical}")
            };
            assert_eq!(value.flow(), &flow);
            assert_eq!(value.flow().serialize_specified().unwrap(), canonical);
        }
    }
}

#[test]
fn const_constructed_flow_is_usable_without_parsing_or_serialization() {
    assert_eq!(
        PROGRAMMATIC_FLOW.direction(),
        CssFlexDirection::ColumnReverse
    );
    assert_eq!(PROGRAMMATIC_FLOW.wrap(), CssFlexWrap::Wrap);
    assert_eq!(
        PROGRAMMATIC_FLOW,
        CssFlexFlow::new(CssFlexDirection::ColumnReverse, CssFlexWrap::Wrap)
    );
}

#[test]
fn parsed_flow_keeps_authored_spelling_and_serializes_omitted_initials() {
    for (authored, expected) in [
        ("WRAP-REVERSE   column", "column wrap-reverse"),
        ("wrap-reverse", "row wrap-reverse"),
        ("column", "column nowrap"),
        ("nowrap", "row nowrap"),
    ] {
        let source = declaration(&format!("flex-flow:  {authored}  !important"));
        let CssKnownPropertyValueRef::FlexFlow(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("ordinary flow: {authored}")
        };
        assert_eq!(value.as_css(), authored);
        assert_eq!(value.flow().serialize_specified().unwrap(), expected);
        let reparsed = declaration(&format!("flex-flow:{expected}"));
        let CssKnownPropertyValueRef::FlexFlow(reparsed) =
            reparsed.known().unwrap().property_value().unwrap()
        else {
            panic!("reparsed canonical flow: {expected}")
        };
        assert_eq!(reparsed.flow(), value.flow());
        assert_eq!(reparsed.flow().serialize_specified().unwrap(), expected);
    }
}

#[test]
fn parsed_longhand_wrappers_keep_exact_authored_css_and_semantic_values() {
    let direction = declaration("flex-direction:  ROW-REVERSE  !important");
    let CssKnownPropertyValueRef::FlexDirection(direction) =
        direction.known().unwrap().property_value().unwrap()
    else {
        panic!("direction wrapper")
    };
    assert_eq!(direction.as_css(), "ROW-REVERSE");
    assert_eq!(direction.value(), &CssFlexDirection::RowReverse);
    assert_eq!(direction.value(), &CssFlexDirection::RowReverse);

    let wrap = declaration("flex-wrap:  WRAP-REVERSE  !important");
    let CssKnownPropertyValueRef::FlexWrap(wrap) = wrap.known().unwrap().property_value().unwrap()
    else {
        panic!("wrap wrapper")
    };
    assert_eq!(wrap.as_css(), "WRAP-REVERSE");
    assert_eq!(wrap.value(), &CssFlexWrap::WrapReverse);
    assert_eq!(wrap.value(), &CssFlexWrap::WrapReverse);
}

#[test]
fn individual_keywords_respect_exact_input_projection_and_byte_limits() {
    let value = CssFlexDirection::Row;
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "row"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 3),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 3),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 2),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
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
    assert_eq!(
        CssFlexWrap::WrapReverse
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 12))
            .unwrap(),
        "wrap-reverse"
    );
}

#[test]
fn flow_uses_one_cumulative_budget_for_both_component_keywords() {
    let flow = CssFlexFlow::new(CssFlexDirection::Row, CssFlexWrap::Wrap);
    assert_eq!(
        flow.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 8))
            .unwrap(),
        "row wrap"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 2, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 1, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 2, 7),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            flow.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(flow.serialize_specified().unwrap(), "row wrap");
}
