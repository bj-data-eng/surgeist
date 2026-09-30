#![forbid(unsafe_code)]

//! Checked authored Grid3 flow-tolerance and intrinsic longhand contributions.
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#placement-tolerance
//! The grammar has no nonnegative range annotation. `normal` remains symbolic:
//! its 1em/0 used value requires the downstream layout context.
//! Checked sum assembly requires a first operand and preserves signed math,
//! later subtraction, and trusted typed calculation children.
//! Values, component origins, contribution ordering, and strict retry behavior
//! below are asserted independently of serialization followed by reparsing.

use surgeist_css::{CssKnownProperty as Property, *};

fn value(declaration: &CssDeclaration) -> &CssFlowTolerance {
    let Some(CssKnownPropertyValueRef::FlowTolerance(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("an ordinary flow-tolerance property wrapper");
    };
    value.value()
}

fn length(value: &CssFlowTolerance) -> &CssSpecifiedLengthPercentage {
    match value.as_ref() {
        CssFlowToleranceRef::LengthPercentage(length) => length,
        other => panic!("a symbolic length-percentage: {other:?}"),
    }
}

fn contribution(value: &CssLonghandContribution) -> &CssFlowTolerance {
    assert_eq!(value.property(), Property::FlowTolerance);
    match value.value() {
        CssContributionValueRef::Ordinary(CssLonghandValueRef::FlowTolerance(value)) => value,
        other => panic!("one ordinary flow-tolerance contribution: {other:?}"),
    }
}

fn checked(components: CssComponentValues, importance: CssImportance) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(Property::FlowTolerance),
        components,
        importance,
    )
    .expect("valid authored flow-tolerance components")
}

fn constructors_preserve_symbolic_keywords_and_signed_numeric_payloads() {
    assert!(matches!(
        CssFlowTolerance::normal().as_ref(),
        CssFlowToleranceRef::Normal
    ));
    assert!(matches!(
        CssFlowTolerance::default().as_ref(),
        CssFlowToleranceRef::Normal
    ));
    assert!(matches!(
        CssFlowTolerance::infinite().as_ref(),
        CssFlowToleranceRef::Infinite
    ));
    assert_eq!(CssFlowTolerance::default(), CssFlowTolerance::normal());
    assert_ne!(CssFlowTolerance::normal(), CssFlowTolerance::infinite());

    for css in [
        "0",
        "-2px",
        "0px",
        "-0.5em",
        "2rem",
        "-25%",
        "125%",
        "calc(-3px)",
        "calc(-4%)",
        "calc(-5em)",
        "calc(-6rem)",
        "calc(-7%)",
    ] {
        let expected = signed_length_percentage(css);
        let tolerance = CssFlowTolerance::length_percentage(expected.clone());
        assert_eq!(length(&tolerance), &expected);
        assert_eq!(tolerance.clone(), tolerance);
    }
    for keyword in [
        "auto",
        "min-content",
        "max-content",
        "fit-content",
        "normal",
        "thin",
        "medium",
        "thick",
    ] {
        assert!(
            CssSpecifiedLengthPercentage::try_from_component(
                CssComponentValue::try_ident(keyword).unwrap()
            )
            .is_err(),
            "{keyword}"
        );
    }
    println!("checked symbolic and signed values: ok");
}

fn checked_sum_construction_preserves_signed_math() {
    let signed = CssLengthPercentageCalculation::try_sum(
        CssLengthPercentageCalculation::try_dimension(-2.0, CssLengthUnit::Px).unwrap(),
        [(
            CssCalculationSumOperator::Subtract,
            CssLengthPercentageCalculation::try_percentage(3.0).unwrap(),
        )],
    )
    .unwrap();
    let nested = CssLengthPercentageCalculation::try_sum(
        signed.clone(),
        [(
            CssCalculationSumOperator::Subtract,
            CssLengthPercentageCalculation::try_dimension(-4.0, CssLengthUnit::Em).unwrap(),
        )],
    )
    .unwrap();
    for valid in [signed, nested] {
        let expected = CssSpecifiedLengthPercentage::try_from_calculation(valid).unwrap();
        let tolerance = CssFlowTolerance::length_percentage(expected.clone());
        assert_eq!(length(&tolerance), &expected);
    }
    println!("checked calculation construction: ok");
}

fn parsed_and_constructed_payloads_match_independent_signed_expectations() {
    for source in ["0", "-2px", "-0.5em", "-25%", "+125%"] {
        let expected = signed_length_percentage(source);
        let report = parse_style_attribute(&format!("flow-tolerance:{source}"));
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let constructed = CssFlowTolerance::length_percentage(expected.clone());
        assert!(exact_literal(
            length(value(&report.syntax()[0])).literal_component(),
            source
        ));
        assert_eq!(length(&constructed), &expected);
        assert_eq!(value(&report.syntax()[0]), &constructed);
    }
    for (source, expected) in [
        ("NoRmAl", CssFlowTolerance::normal()),
        (r"\69 nfinite", CssFlowTolerance::infinite()),
    ] {
        let report = parse_style_attribute(&format!("flow-tolerance: {source} !important"));
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(value(&report.syntax()[0]), &expected);
        let Some(CssKnownPropertyValueRef::FlowTolerance(wrapper)) =
            report.syntax()[0].known().unwrap().property_value()
        else {
            unreachable!()
        };
        assert_eq!(wrapper.as_css(), source);
    }

    let report = parse_style_attribute("flow-tolerance:calc(-2px - 3%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let calc = length(value(&report.syntax()[0]))
        .calculation()
        .expect("retained exact calculation");
    let CssCalculationExpressionRef::NestedCalc(root) = calc.expression() else {
        panic!("expected calc root")
    };
    let CssCalculationExpressionRef::Sum(terms) = root.operand() else {
        panic!("expected signed sum")
    };
    assert_eq!(terms.len(), 2);
    assert!(
        matches!(terms.term(0).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "-2" && v.unit() == Some("px"))
    );
    assert_eq!(
        terms.term(1).unwrap().operator(),
        Some(surgeist_css::CssCalculationSumOperator::Subtract)
    );
    assert!(
        matches!(terms.term(1).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(v)) if v.representation() == "3")
    );

    let report = parse_style_attribute("flow-tolerance:calc(-2 * 3px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let calculation = length(value(&report.syntax()[0]))
        .calculation()
        .expect("retained exact calculation");
    assert_eq!(calculation.result_type(), CssCalculationType::Length);
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("expected calc root")
    };
    let CssCalculationExpressionRef::Product(product) = root.operand() else {
        panic!("the authored multiplication is retained");
    };
    assert_eq!(product.len(), 2);
    let first = product.factor(0).unwrap();
    assert_eq!(first.operator(), None);
    assert!(matches!(
        first.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(v)) if v.representation() == "-2"
    ));
    let second = product.factor(1).unwrap();
    assert_eq!(
        second.operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    assert!(matches!(second.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(length))
            if length.representation() == "3" && length.unit() == Some("px")));
    println!("independent parsed and constructed semantics: ok");
}

fn component_construction_preserves_value_importance_and_origins() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_dimension("-0.5", "em").unwrap(),
    ])
    .unwrap();
    let declaration = checked(components, CssImportance::Important);
    assert!(exact_literal(
        length(value(&declaration)).literal_component(),
        "-0.5em"
    ));
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.position(), None);
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert!(
        declaration
            .value_components()
            .items()
            .iter()
            .all(|item| { matches!(item.origin(), CssValueOrigin::Programmatic) })
    );
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&declaration).unwrap()
    else {
        panic!("completed longhand from programmatic components");
    };
    let [item] = values.items() else {
        panic!("one longhand");
    };
    assert_eq!(contribution(item), value(&declaration));
    assert!(item.source().same_occurrence(&declaration));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
    println!("component construction and provenance: ok");
}

fn normalization_keeps_occurrence_order_and_symbolic_values() {
    let source = concat!(
        ".grid{flow-tolerance:normal!important;",
        "flow-tolerance:-25%;flow-tolerance:infinite}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_report(&report).expect("flow-tolerance expands as one longhand");
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(declaration) => Some(declaration),
            CssNormalizedItem::Rule(_) => None,
            other => panic!("unexpected normalized item: {other:?}"),
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (index, (declaration, expected)) in declarations
        .iter()
        .zip([
            CssFlowTolerance::normal(),
            CssFlowTolerance::length_percentage(signed_length_percentage("-25%")),
            CssFlowTolerance::infinite(),
        ])
        .enumerate()
    {
        assert_eq!(declaration.order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("one completed flow-tolerance longhand");
        };
        let [item] = values.items() else {
            panic!("one longhand per occurrence");
        };
        assert_eq!(contribution(item), &expected);
        assert!(item.source().same_occurrence(declaration.source()));
        assert_eq!(
            item.source().parsed_name().unwrap().source().as_str(),
            source
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(item.replacement_components().is_none());
    }
    assert!(
        !declarations[0]
            .source()
            .same_occurrence(declarations[1].source())
    );
    assert!(
        declarations[0]
            .selector_context()
            .same_context(declarations[1].selector_context())
    );
    println!("ordered symbolic longhand normalization: ok");
}

fn global_keywords_are_not_resolved_to_ordinary_defaults() {
    for (source, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let declaration = checked(
            parse_component_values(source).unwrap(),
            CssImportance::Important,
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&declaration).unwrap()
        else {
            panic!("completed symbolic global longhand");
        };
        let [item] = values.items() else {
            panic!("one global contribution");
        };
        assert_eq!(item.property(), Property::FlowTolerance);
        assert!(
            matches!(item.value(), CssContributionValueRef::Global(keyword) if keyword == expected)
        );
        assert!(item.source().same_occurrence(&declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    println!("global values remain symbolic: ok");
}

fn pending_reentry_preserves_original_and_replacement_origins_and_is_retryable() {
    let source = r"\66 low-tolerance:var(--threshold,-1px)!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let authored = &report.syntax()[0];
    let CssExpansion::Pending(pending) = expand_declaration(authored).unwrap() else {
        panic!("variable-dependent flow-tolerance stays pending");
    };
    assert!(pending.source().same_occurrence(authored));
    assert_eq!(pending.source().importance(), CssImportance::Important);
    for source in ["-25%", "normal", "infinite", "initial", "calc(-2px - 3%)"] {
        let replacement = parse_component_values(source).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("strict reentry produces one complete longhand");
        };
        let [item] = values.items() else {
            panic!("one completed contribution");
        };
        assert_eq!(item.property(), Property::FlowTolerance);
        match source {
            "-25%" => assert!(exact_literal(
                length(contribution(item)).literal_component(),
                "-25%"
            )),
            "normal" => assert_eq!(contribution(item), &CssFlowTolerance::normal()),
            "infinite" => assert_eq!(contribution(item), &CssFlowTolerance::infinite()),
            "initial" => assert!(matches!(
                item.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Initial)
            )),
            "calc(-2px - 3%)" => {
                let calculation = length(contribution(item))
                    .calculation()
                    .expect("retained exact calculation");
                assert_eq!(
                    calculation.result_type(),
                    CssCalculationType::LengthPercentage
                );
                let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
                    panic!("calc root")
                };
                let CssCalculationExpressionRef::Sum(terms) = root.operand() else {
                    panic!("signed sum")
                };
                assert_eq!(terms.len(), 2);
                assert_eq!(terms.term(0).unwrap().operator(), None);
                assert!(
                    matches!(terms.term(0).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(v)) if v.representation() == "-2" && v.unit() == Some("px"))
                );
                assert_eq!(
                    terms.term(1).unwrap().operator(),
                    Some(surgeist_css::CssCalculationSumOperator::Subtract)
                );
                assert!(
                    matches!(terms.term(1).unwrap().expression(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(v)) if v.representation() == "3")
                );
                assert_eq!(calculation.components(), &replacement);
            }
            _ => unreachable!(),
        }
        assert!(item.source().same_occurrence(authored));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.source().parsed_name(), authored.parsed_name());
        assert_eq!(item.source().parsed_value(), authored.parsed_value());
        let retained = item.replacement_components().unwrap();
        assert_eq!(retained.items().len(), replacement.items().len());
        for (actual, supplied) in retained.items().iter().zip(replacement.items()) {
            assert_eq!(actual.origin(), supplied.origin());
            let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                (actual.origin(), supplied.origin())
            else {
                panic!("replacement tokens retain their own parsed sources");
            };
            assert!(actual.source().same_snapshot(supplied.source()));
            assert_eq!(actual.source().as_str(), source);
        }
    }
    for source in [
        "var(--again)",
        "calc(var(--again) - 2px)",
        r"\76 ar(--again)",
    ] {
        let error = pending
            .reenter(parse_component_values(source).unwrap())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::ResidualSubstitution
        ));
    }
    for source in [
        "auto",
        "normal infinite",
        "-1",
        "1px!important",
        "1px;color:red",
        "calc(1px + 1s)",
    ] {
        let error = pending
            .reenter(parse_component_values(source).unwrap())
            .unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
            panic!("invalid replacement must fail strictly: {source}: {error:?}");
        };
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        assert!(pending.source().same_occurrence(authored));
    }
    let replacement = parse_component_values("1px!important").unwrap();
    let error = pending.reenter(replacement).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        unreachable!()
    };
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
        panic!("the illegal annotation identifies the replacement's ! token");
    };
    assert_eq!(origin.source().as_str(), "1px!important");
    assert_eq!(origin.span().start().byte_offset().value(), 3);
    assert_eq!(origin.span().end().byte_offset().value(), 4);

    let CssContributions::Longhands(values) = pending
        .reenter(parse_component_values("-2px").unwrap())
        .unwrap()
    else {
        panic!("a valid retry succeeds after every prior failed replacement");
    };
    let [item] = values.items() else {
        panic!("one contribution after retry");
    };
    assert!(exact_literal(
        length(contribution(item)).literal_component(),
        "-2px"
    ));
    assert!(item.source().same_occurrence(authored));
    assert!(
        pending
            .source()
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    println!("strict pending reentry and preserved origins: ok");
}

fn main() {
    constructors_preserve_symbolic_keywords_and_signed_numeric_payloads();
    checked_sum_construction_preserves_signed_math();
    parsed_and_constructed_payloads_match_independent_signed_expectations();
    component_construction_preserves_value_importance_and_origins();
    normalization_keeps_occurrence_order_and_symbolic_values();
    global_keywords_are_not_resolved_to_ordinary_defaults();
    pending_reentry_preserves_original_and_replacement_origins_and_is_retryable();
    canonical_specified_values_keep_context_dependent_operands_symbolic();
}

fn canonical_specified_values_keep_context_dependent_operands_symbolic() {
    assert_eq!(
        CssFlowTolerance::normal().serialize_specified().unwrap(),
        "normal"
    );
    assert_eq!(
        CssFlowTolerance::infinite().serialize_specified().unwrap(),
        "infinite"
    );
    let tolerance =
        CssFlowTolerance::length_percentage(signed_length_percentage("calc(-2px - 3%)"));
    assert_eq!(tolerance.serialize_specified().unwrap(), "calc(-3% - 2px)");
    let before = tolerance.clone();
    assert_eq!(
        tolerance
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(100, 100, 1))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(tolerance, before);
    println!("canonical symbolic specified serialization: ok");
}

fn signed_length_percentage(css: &str) -> surgeist_css::CssSpecifiedLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

fn exact_literal(component: Option<&surgeist_css::CssComponentValue>, css: &str) -> bool {
    use surgeist_css::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let expected = surgeist_css::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}
