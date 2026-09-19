#![forbid(unsafe_code)]

//! Color 4 (2026-09-08), section 3.3: opacity initially equals one and is
//! noninherited. Specified values retain numbers, percentages and calculations;
//! computed clamping is downstream. These are typed transport assertions, not
//! canonical opacity serialization assertions.

use surgeist_css::*;

fn parsed(value: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("opacity:{value}!important"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary opacity contribution")
    };
    let [value] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(value.property(), CssKnownProperty::Opacity);
    assert!(value.source().same_occurrence(source));
    assert_eq!(value.source().importance(), CssImportance::Important);
    values
}

fn opacity(contribution: &CssLonghandContribution) -> &CssOpacityValue {
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::Opacity(value)) =
        contribution.value()
    else {
        panic!("exact opacity payload")
    };
    value
}

fn assert_numeric(value: &CssOpacityValue, index: usize) {
    match (index, value) {
        (0, CssOpacityValue::Literal(number)) => assert_eq!(number.value(), 0.5),
        (1, CssOpacityValue::Number(number)) => assert_eq!(number.value(), -0.5),
        (2, CssOpacityValue::Number(number)) => assert_eq!(number.value(), 1.5),
        (3, CssOpacityValue::Percentage(number)) => assert_eq!(number.value(), -25.0),
        (4, CssOpacityValue::Percentage(number)) => assert_eq!(number.value(), 150.0),
        _ => panic!("independently specified numeric branch {index}: {value:?}"),
    }
}

fn assert_calculation(value: &CssOpacityValue, percentage: bool, programmatic: bool) {
    let expression = match (percentage, value) {
        (false, CssOpacityValue::Calculation(value)) => {
            assert_eq!(value.result_type(), CssCalculationType::Number);
            value.expression()
        }
        (true, CssOpacityValue::PercentageCalculation(value)) => {
            assert_eq!(value.result_type(), CssCalculationType::Percentage);
            value.expression()
        }
        _ => panic!("unresolved calculation: {value:?}"),
    };
    assert_eq!(
        matches!(expression.origin(), CssValueOrigin::Programmatic),
        programmatic
    );
    let CssCalculationExpressionRef::NestedCalc(root) = expression else {
        panic!("authored calc root")
    };
    if percentage {
        let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
            panic!("two authored percentage terms")
        };
        assert_eq!(sum.len(), 2);
        assert_eq!(sum.term(0).unwrap().operator(), None);
        assert_eq!(
            sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Add)
        );
        for index in 0..2 {
            let CssCalculationExpressionRef::Value(CssCalculationValueRef::Percentage(number)) =
                sum.term(index).unwrap().expression()
            else {
                panic!("percentage leaf")
            };
            assert_eq!(number.representation(), "25");
            assert_eq!(
                matches!(number.origin(), CssValueOrigin::Programmatic),
                programmatic
            );
        }
    } else {
        let CssCalculationExpressionRef::Product(product) = root.operand() else {
            panic!("authored division")
        };
        assert_eq!(product.len(), 2);
        assert_eq!(product.factor(0).unwrap().operator(), None);
        assert_eq!(
            product.factor(1).unwrap().operator(),
            Some(CssCalculationProductOperator::Divide)
        );
        for (index, expected) in ["1", "2"].into_iter().enumerate() {
            let CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(number)) =
                product.factor(index).unwrap().expression()
            else {
                panic!("integer leaf")
            };
            assert_eq!(number.representation(), expected);
            assert_eq!(
                matches!(number.origin(), CssValueOrigin::Programmatic),
                programmatic
            );
        }
    }
}

#[test]
fn opacity_initial_is_exactly_one_and_all_reset_includes_its_terminal() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::Opacity.metadata().unwrap().kind()
    else {
        panic!("terminal opacity metadata")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::Opacity
    );
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("fixed initial")
    };
    assert_eq!(value.property().known_property(), CssKnownProperty::Opacity);
    assert!(matches!(
        value.view(),
        CssLonghandValueRef::Opacity(CssOpacityValue::Literal(number)) if number.value() == 1.0
    ));

    let CssPropertyKindRef::UniversalReset(metadata) =
        CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all metadata")
    };
    assert!(!metadata.excludes(CssPropertyNameRef::Known(CssKnownProperty::Opacity)));
    let report = parse_style_attribute("all:unset");
    assert!(report.is_clean());
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&report.syntax()[0]).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Opacity)));
    assert_eq!(reset.keyword(), CssGlobalKeyword::Unset);
}

#[test]
fn parsed_and_programmatic_opacity_transport_preserves_exact_numeric_branches() {
    for (index, spelling) in ["0.5", "-0.5", "1.5", "-25%", "150%"]
        .into_iter()
        .enumerate()
    {
        let source = parsed(spelling);
        let values = expanded(&source);
        assert_numeric(opacity(&values.items()[0]), index);
        assert!(
            values.items()[0]
                .source()
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(source.parsed_value().unwrap().source())
        );

        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(spelling).unwrap()])
                .unwrap();
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Opacity),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(constructed.position().is_none());
        assert!(constructed.parsed_name().is_none());
        assert!(constructed.parsed_value().is_none());
        let values = expanded(&constructed);
        assert_numeric(opacity(&values.items()[0]), index);
        assert_eq!(values.items()[0].source().value_components(), &components);
        assert!(matches!(
            values.items()[0].source().value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
    }
}

#[test]
fn parsed_and_programmatic_opacity_calculations_preserve_operations_and_origins() {
    for (percentage, spelling, tokens) in [
        (false, "calc(1 / 2)", ["1", " ", "/", " ", "2"]),
        (true, "calc(25% + 25%)", ["25%", " ", "+", " ", "25%"]),
    ] {
        let source = parsed(spelling);
        let values = expanded(&source);
        assert_calculation(opacity(&values.items()[0]), percentage, false);

        let arguments = CssComponentValues::try_new(
            tokens
                .into_iter()
                .map(|token| CssComponentValue::try_token(token).unwrap())
                .collect(),
        )
        .unwrap();
        let components = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("calc", arguments).unwrap(),
        ])
        .unwrap();
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Opacity),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(constructed.position().is_none());
        assert!(constructed.parsed_value().is_none());
        let values = expanded(&constructed);
        assert_calculation(opacity(&values.items()[0]), percentage, true);
        assert_eq!(values.items()[0].source().value_components(), &components);
    }
}

#[test]
fn opacity_reentry_emits_exact_replacement_value_with_original_occurrence() {
    let source = parsed("var(--opacity)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending opacity")
    };
    for replacement in ["150%", "calc(1 / 2)", "calc(25% + 25%)"] {
        let components = parse_component_values(replacement).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(components.clone()).unwrap()
        else {
            panic!("completed opacity")
        };
        let [value] = values.items() else {
            panic!("one terminal contribution")
        };
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&components));
        match replacement {
            "150%" => assert_numeric(opacity(value), 4),
            "calc(1 / 2)" => assert_calculation(opacity(value), false, false),
            _ => assert_calculation(opacity(value), true, false),
        }
    }
}
