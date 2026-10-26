//! Functional coverage alongside the new aggregate output APIs. Grid2 §7.4
//! requires the row/column slash; §7.8 determines auto-flow orientation and
//! canonical auto-flow/dense order. Existing child numeric/identity contracts
//! determine operand text. There was no existing aggregate output API for RED.
use crate::css::support::authored_property::{checked, parsed};
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::*;

type Limits = CssSerializationLimits;
type Kind = SerializationFailure;

fn serialize(value: &CssDeclaration, limits: Limits) -> Result<String, CssSerializationError> {
    match value.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplate(value) => value.value().serialize_specified(limits),
        CssKnownPropertyValueRef::Grid(value) => value.value().serialize_specified(limits),
        _ => panic!("aggregate"),
    }
}

fn output(property: CssKnownProperty, text: &str, expected: &str) {
    let values = [
        parsed(property, text),
        checked(property, text, false),
        checked(property, text, true),
    ];
    for value in values {
        assert_eq!(value.importance(), CssImportance::Important);
        let before = value.clone();
        assert_eq!(
            serialize(&value, Limits::default()).unwrap(),
            expected,
            "{text}"
        );
        assert_eq!(value, before);
        // Both actual checked boundaries re-admit the independently expected text.
        checked(property, expected, false);
        checked(property, expected, true);
        match value.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::GridTemplate(value) => {
                assert_eq!(
                    value
                        .value()
                        .serialize_specified(Default::default())
                        .unwrap(),
                    expected
                )
            }
            CssKnownPropertyValueRef::Grid(value) => {
                assert_eq!(
                    value
                        .value()
                        .serialize_specified(Default::default())
                        .unwrap(),
                    expected
                )
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn represented_template_forms_emit_required_slash_and_typed_children() {
    for property in [CssKnownProperty::GridTemplate, CssKnownProperty::Grid] {
        for (text, expected) in [
            ("NONE", "none"),
            ("auto / MIN-CONTENT", "auto / min-content"),
            ("10px / 20px", "10px / 20px"),
            (
                "[a\\ b] minmax(10px, 1fr) / repeat(2, fit-content(20px))",
                "[a\\ b] minmax(10px, 1fr) / repeat(2, fit-content(20px))",
            ),
        ] {
            output(property, text, expected);
        }
    }
}

#[test]
fn row_and_column_auto_flow_preserve_optional_sizes_and_dense_order() {
    for (text, expected) in [
        ("auto-flow / 10px", "auto-flow / 10px"),
        ("dense auto-flow / 10px", "auto-flow dense / 10px"),
        ("auto-flow 20px / 10px", "auto-flow 20px / 10px"),
        ("dense auto-flow 20px / 10px", "auto-flow dense 20px / 10px"),
        ("10px / auto-flow", "10px / auto-flow"),
        ("10px / dense auto-flow", "10px / auto-flow dense"),
        ("10px / auto-flow 20px", "10px / auto-flow 20px"),
        ("10px / dense auto-flow 20px", "10px / auto-flow dense 20px"),
        (
            "auto-flow auto min-content / 10px",
            "auto-flow auto min-content / 10px",
        ),
        (
            "10px / auto-flow auto min-content",
            "10px / auto-flow auto min-content",
        ),
    ] {
        output(CssKnownProperty::Grid, text, expected);
    }
}

#[test]
fn aggregate_node_and_byte_thresholds_have_no_enum_carrier_cost() {
    for (property, text, nodes) in [
        (CssKnownProperty::GridTemplate, "none", 1),
        (CssKnownProperty::Grid, "none", 1),
        (CssKnownProperty::GridTemplate, "10px / 20px", 5),
        (CssKnownProperty::Grid, "10px / 20px", 5),
        (CssKnownProperty::Grid, "auto-flow / 10px", 4),
        (CssKnownProperty::Grid, "10px / auto-flow", 4),
        (CssKnownProperty::Grid, "auto-flow dense / 10px", 5),
        (CssKnownProperty::Grid, "10px / auto-flow dense", 5),
        (CssKnownProperty::Grid, "auto-flow dense 20px / 10px", 7),
        (CssKnownProperty::Grid, "10px / auto-flow dense 20px", 7),
    ] {
        let value = parsed(property, text);
        assert_eq!(
            serialize(&value, Limits::new(nodes, nodes, text.len())).unwrap(),
            text
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, text.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, text.len()),
                Kind::ProjectionNodeLimit,
            ),
            (Limits::new(nodes, nodes, text.len() - 1), Kind::ByteLimit),
        ] {
            assert_eq!(
                serialization_failure(&serialize(&value, limits).unwrap_err()),
                kind,
                "{text}"
            );
        }
    }
}

#[test]
fn symbolic_and_exact_numeric_children_use_the_projection_contract() {
    output(
        CssKnownProperty::GridTemplate,
        "calc(10px + 5%) / 1e-50fr",
        "calc(5% + 10px) / 0fr",
    );
    output(
        CssKnownProperty::Grid,
        "calc(10px + 5%) / dense auto-flow fit-content(1e-50px)",
        "calc(5% + 10px) / auto-flow dense fit-content(0px)",
    );
    let huge = format!("auto-flow 1{}fr / 10px", "0".repeat(50));
    output(CssKnownProperty::Grid, "auto-flow 1e50fr / 10px", &huge);
}

#[test]
fn output_and_atomic_error_preserve_authored_numeric_origins() {
    let source = "/*😀*/grid:10px / dense auto-flow 1e-50px!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Grid(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("grid");
    };
    let value = wrapper.value();
    let before = value.clone();
    let operand = value.auto_tracks().unwrap().sizes()[0]
        .breadth()
        .unwrap()
        .length_percentage()
        .unwrap();
    let CssValueOrigin::Parsed(origin) = operand.origin() else {
        panic!("parsed origin");
    };
    let start = source.find("1e-50px").unwrap();
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + "1e-50px".len()
    );
    assert_eq!(origin.source().as_str(), source);
    assert!(
        matches!(operand.literal().map(|literal| literal.component()).unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
        if number.representation() == "1e-50" && unit == "px")
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "10px / auto-flow dense 0px"
    );
    assert!(
        value
            .serialize_specified(Limits::new(100, 100, 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(*value, before);
    assert!(
        matches!(operand.literal().map(|literal| literal.component()).unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
        if number.representation() == "1e-50" && unit == "px")
    );
    assert_eq!(wrapper.as_css(), "10px / dense auto-flow 1e-50px");
}
