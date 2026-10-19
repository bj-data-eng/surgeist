#![forbid(unsafe_code)]
//! Functional evidence for the new typed item-flow APIs. Selected Grid 3
//! supplies the finite longhands; the adopted product policy supplies full
//! D/W/P/T output, authored wrap facet preservation and fixed-child costs.

use surgeist_css::*;

const DIRECTIONS: [(CssItemDirection, &str); 5] = [
    (CssItemDirection::Auto, "auto"),
    (CssItemDirection::Row, "row"),
    (CssItemDirection::Column, "column"),
    (CssItemDirection::RowReverse, "row-reverse"),
    (CssItemDirection::ColumnReverse, "column-reverse"),
];
const WRAPS: [(CssItemWrap, &str, usize); 12] = [
    (CssItemWrap::Mode(CssItemWrapMode::Auto), "auto", 1),
    (CssItemWrap::Mode(CssItemWrapMode::NoWrap), "nowrap", 1),
    (CssItemWrap::Mode(CssItemWrapMode::Wrap), "wrap", 1),
    (CssItemWrap::Order(CssItemWrapOrder::Normal), "normal", 1),
    (CssItemWrap::Order(CssItemWrapOrder::Reverse), "reverse", 1),
    (
        CssItemWrap::Both(CssItemWrapMode::Auto, CssItemWrapOrder::Normal),
        "auto normal",
        2,
    ),
    (
        CssItemWrap::Both(CssItemWrapMode::Auto, CssItemWrapOrder::Reverse),
        "auto reverse",
        2,
    ),
    (
        CssItemWrap::Both(CssItemWrapMode::NoWrap, CssItemWrapOrder::Normal),
        "nowrap normal",
        2,
    ),
    (
        CssItemWrap::Both(CssItemWrapMode::NoWrap, CssItemWrapOrder::Reverse),
        "nowrap reverse",
        2,
    ),
    (
        CssItemWrap::Both(CssItemWrapMode::Wrap, CssItemWrapOrder::Normal),
        "wrap normal",
        2,
    ),
    (
        CssItemWrap::Both(CssItemWrapMode::Wrap, CssItemWrapOrder::Reverse),
        "wrap reverse",
        2,
    ),
    (CssItemWrap::WrapReverse, "wrap-reverse", 1),
];
const PACKS: [(CssItemPack, &str, usize); 4] = [
    (CssItemPack::Normal, "normal", 1),
    (CssItemPack::Dense, "dense", 1),
    (CssItemPack::Balance, "balance", 1),
    (CssItemPack::DenseBalance, "dense balance", 2),
];

fn typed(declaration: &CssDeclaration) -> CssItemFlow {
    let Some(CssKnownPropertyValueRef::ItemFlow(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("ordinary item-flow")
    };
    value.value().clone()
}

fn fronts(text: &str) -> [CssDeclaration; 2] {
    let report = parse_style_attribute(&format!("item-flow:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let constructed = parse_property_value_for_grammar(
        CssKnownProperty::ItemFlow.grammar(),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    [report.syntax()[0].clone(), constructed]
}

fn bounded(
    serialize: impl Fn(
        CssSpecifiedValueSerializationLimits,
    ) -> Result<String, CssSpecifiedValueSerializationError>,
    expected: &str,
    inputs: usize,
    projections: usize,
) {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    assert_eq!(
        serialize(Limits::new(inputs, projections, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(inputs - 1, projections, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(inputs, projections - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(
        serialize(Limits::new(inputs, projections, expected.len())).unwrap(),
        expected
    );
}

#[test]
fn every_typed_longhand_state_emits_its_independent_text_and_exact_facet_costs() {
    for (value, expected) in DIRECTIONS {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        bounded(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            1,
            1,
        );
    }
    for (value, expected, nodes) in WRAPS {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        bounded(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            nodes,
            nodes,
        );
    }
    for (value, expected, nodes) in PACKS {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        bounded(
            |limits| value.serialize_specified_with_limits(limits),
            expected,
            nodes,
            nodes,
        );
    }
}

#[test]
fn all_480_complete_typed_keyword_tuples_reparse_without_erasing_wrap_states() {
    for (direction, direction_text) in DIRECTIONS {
        for (wrap, wrap_text, wrap_nodes) in WRAPS {
            for (pack, pack_text, pack_nodes) in PACKS {
                for (tolerance, tolerance_text) in [
                    (CssFlowTolerance::normal(), "normal"),
                    (CssFlowTolerance::infinite(), "infinite"),
                ] {
                    let value = CssItemFlow::new(direction, wrap, pack, tolerance.clone());
                    let expected =
                        format!("{direction_text} {wrap_text} {pack_text} {tolerance_text}");
                    assert_eq!(value.direction(), direction);
                    assert_eq!(value.wrap(), wrap);
                    assert_eq!(value.pack(), pack);
                    assert_eq!(value.tolerance(), &tolerance);
                    assert_eq!(value.serialize_specified().unwrap(), expected);
                    let nodes = 2 + wrap_nodes + pack_nodes;
                    bounded(
                        |limits| value.serialize_specified_with_limits(limits),
                        &expected,
                        nodes,
                        nodes,
                    );
                    for declaration in fronts(&expected) {
                        let parsed = typed(&declaration);
                        assert_eq!(parsed, value, "{expected}");
                        assert_eq!(parsed.serialize_specified().unwrap(), expected);
                        let CssExpansion::Contributions(CssContributions::Longhands(children)) =
                            expand_declaration(&declaration).unwrap()
                        else {
                            panic!("four ordinary constituents")
                        };
                        let [d, w, p, t] = children.items() else {
                            panic!("four terminals")
                        };
                        assert!(
                            matches!(d.value(), CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemDirection(v)) if *v == direction)
                        );
                        assert!(
                            matches!(w.value(), CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemWrap(v)) if *v == wrap)
                        );
                        assert!(
                            matches!(p.value(), CssContributionValueRef::Ordinary(CssLonghandValueRef::ItemPack(v)) if *v == pack)
                        );
                        assert!(
                            matches!(t.value(), CssContributionValueRef::Ordinary(CssLonghandValueRef::FlowTolerance(v)) if v == &tolerance)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn intrinsic_initial_payloads_and_omitted_whole_constituents_match_typed_defaults() {
    let default = CssItemFlow::default();
    assert_eq!(default.direction(), CssItemDirection::default());
    assert_eq!(default.wrap(), CssItemWrap::default());
    assert_eq!(default.pack(), CssItemPack::default());
    assert_eq!(default.tolerance(), &CssFlowTolerance::normal());
    assert_eq!(
        default.serialize_specified().unwrap(),
        "auto auto normal normal"
    );
    bounded(
        |limits| default.serialize_specified_with_limits(limits),
        "auto auto normal normal",
        4,
        4,
    );

    for property in [
        CssKnownProperty::ItemDirection,
        CssKnownProperty::ItemWrap,
        CssKnownProperty::ItemPack,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand metadata")
        };
        assert!(!metadata.inherited_by_default());
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary typed initial")
        };
        match value.view() {
            CssLonghandValueRef::ItemDirection(value) => assert_eq!(*value, CssItemDirection::Auto),
            CssLonghandValueRef::ItemWrap(value) => {
                assert_eq!(*value, CssItemWrap::Mode(CssItemWrapMode::Auto))
            }
            CssLonghandValueRef::ItemPack(value) => assert_eq!(*value, CssItemPack::Normal),
            _ => panic!("selected initial"),
        }
    }
    for declaration in fronts("row") {
        assert_eq!(
            typed(&declaration),
            CssItemFlow::new(
                CssItemDirection::Row,
                CssItemWrap::default(),
                CssItemPack::Normal,
                CssFlowTolerance::normal()
            )
        );
    }
    for declaration in fronts("reverse") {
        assert_eq!(
            typed(&declaration).wrap(),
            CssItemWrap::Order(CssItemWrapOrder::Reverse)
        );
    }
    for declaration in fronts("wrap-reverse") {
        assert_eq!(typed(&declaration).wrap(), CssItemWrap::WrapReverse);
    }
}

#[test]
fn grouped_search_preserves_original_numeric_tokens_and_function_children() {
    for text in [
        "-02.500PX reverse dense row",
        "reverse /* keep */ calc(-2px - 3%) balance dense row",
    ] {
        for declaration in fronts(text) {
            let original = declaration
                .value_components()
                .items()
                .iter()
                .find(|value| {
                    matches!(
                        value.view(),
                        CssComponentValueRef::Function(_)
                            | CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
                    )
                })
                .unwrap();
            let value = typed(&declaration);
            let CssFlowToleranceRef::LengthPercentage(scalar) = value.tolerance().as_ref() else {
                panic!("typed signed tolerance")
            };
            assert_eq!(scalar.origin(), original.origin());
            match original.view() {
                CssComponentValueRef::Function(_) => assert_eq!(
                    scalar.calculation().unwrap().components().items(),
                    std::slice::from_ref(original)
                ),
                _ => assert_eq!(scalar.literal_component().unwrap(), original),
            }
            let before = value.clone();
            let expected = if text.starts_with('-') {
                "row reverse dense -2.5px"
            } else {
                "row reverse dense balance calc(-3% - 2px)"
            };
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_eq!(value, before);
        }
    }
}

#[test]
fn constructed_symbolic_tolerance_composes_without_changing_owning_work_costs() {
    let tolerance = CssFlowTolerance::length_percentage(
        CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values("calc(1px + 2em)").unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    let value = CssItemFlow::new(
        CssItemDirection::Row,
        CssItemWrap::default(),
        CssItemPack::Normal,
        tolerance,
    );
    let expected = "row auto normal calc(2em + 1px)";
    bounded(
        |limits| value.serialize_specified_with_limits(limits),
        expected,
        7,
        8,
    );
    for declaration in fronts(expected) {
        let reparsed = typed(&declaration);
        assert_eq!(reparsed.direction(), value.direction());
        assert_eq!(reparsed.wrap(), value.wrap());
        assert_eq!(reparsed.pack(), value.pack());
        // Owning numeric canonicalization sorts terms. Authored expression
        // structure remains distinct; canonical numeric output is stable.
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
    }
}
