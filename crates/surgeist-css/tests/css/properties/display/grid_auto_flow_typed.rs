//! Current authored grid-auto-flow model. Grid 2 (2025-03-26) §7.7 supplies
//! axis/dense ordering; Grid 3 (2026-01-21) §2.3 and the recorded WebKit choice
//! supply symbolic normal and bare dense without choosing a layout direction.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::assert_checked_value_rejected;
use surgeist_css::*;

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("grid-auto-flow: {value}");
    let report = CssParser::default().parse_block_contents(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn direct_construction_covers_six_distinct_states_with_canonical_text() {
    let cases = [
        (CssGridAutoFlow::Normal, "normal"),
        (CssGridAutoFlow::Dense, "dense"),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, false),
            "row",
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
            "row dense",
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, false),
            "column",
        ),
        (
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
            "column dense",
        ),
    ];
    for (index, (value, css)) in cases.into_iter().enumerate() {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), css);
        for (other_index, (other, _)) in cases.into_iter().enumerate() {
            if index != other_index {
                assert_ne!(value, other, "distinct authored states");
            }
        }
    }
    assert_ne!(
        CssGridAutoFlow::Dense,
        CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true)
    );
}

#[test]
fn parsed_keyword_orders_canonicalize_without_losing_authored_slice() {
    for (authored, expected, state) in [
        ("NoRmAl", "normal", CssGridAutoFlow::Normal),
        ("DENSE", "dense", CssGridAutoFlow::Dense),
        (
            "DENSE rOw",
            "row dense",
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Row, true),
        ),
        (
            "d\\65 nse c\\6f lumn",
            "column dense",
            CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true),
        ),
    ] {
        let source = parsed(authored);
        let CssKnownPropertyValueRef::GridAutoFlow(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed grid-auto-flow")
        };
        assert_eq!(wrapper.as_css(), authored);
        assert_eq!(*wrapper.value(), state);
        assert_eq!(
            wrapper
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn authored_axes_and_initial_are_directly_represented() {
    for (text, expected) in [
        ("normal", None),
        ("dense", None),
        (
            "row dense",
            Some(CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Row, true)),
        ),
        (
            "dense column",
            Some(CssGridAutoFlowMode::new(CssGridAutoFlowAxis::Column, true)),
        ),
    ] {
        let source = parsed(text);
        let CssKnownPropertyValueRef::GridAutoFlow(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed grid-auto-flow")
        };
        match expected {
            Some(flow) => assert_eq!(*wrapper.value(), CssGridAutoFlow::ExplicitAxis(flow)),
            None => assert!(matches!(
                wrapper.value(),
                CssGridAutoFlow::Normal | CssGridAutoFlow::Dense
            )),
        }
    }
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::GridAutoFlow.metadata().unwrap().kind()
    else {
        panic!("grid-auto-flow longhand")
    };
    assert!(!metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::GridAutoFlow(initial) = initial.view() else {
        panic!("typed grid-auto-flow initial")
    };
    assert_eq!(*initial, CssGridAutoFlow::Normal);
}

#[test]
fn canonical_serializer_charges_each_keyword_and_exact_output_bytes() {
    let value = CssGridAutoFlow::explicit_axis(CssGridAutoFlowAxis::Column, true);
    for (limits, error) in [
        (
            CssSerializationLimits::new(1, 2, 12),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 1, 12),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 2, 11),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            error
        );
    }
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(2, 2, 12))
            .unwrap(),
        "column dense"
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "column dense"
    );
}

#[test]
fn invalid_keyword_combinations_are_rejected_by_the_public_grammar() {
    // Grid 3 §2.3: normal stands alone; one axis and one dense, each once.
    for (value, token) in [
        ("normal dense", "dense"),
        ("dense normal", "normal"),
        ("row column", "column"),
        ("column row", "row"),
        ("row row", "row"),
        ("dense dense", "dense"),
        ("dense row column", "column"),
    ] {
        assert_checked_value_rejected(CssKnownProperty::GridAutoFlow, value, Some(token));
    }
}

#[test]
fn normalization_charges_one_terminal_contribution_for_dense() {
    let report = CssParser::default().parse_sheet(".p{grid-auto-flow:dense}");
    assert!(report.is_clean());
    let original = report.syntax().clone();
    let error = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(1, 1, 1, 0).unwrap())
        .unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 0))
    );
    assert_eq!(report.syntax(), &original);
    assert!(
        report
            .syntax()
            .normalize(CssNormalizationLimits::try_new(1, 1, 1, 1).unwrap())
            .is_ok()
    );
}
