use surgeist_css::{
    CssCalculation, CssCalculationExpressionRef, CssCalculationProductOperator,
    CssCalculationSumOperator, CssComponentValue, CssComponentValues, CssLength, CssNumber,
    CssNumeric, CssValueOrigin, parse_component_values,
};

#[test]
fn shallow_clones_preserve_exact_leaves_function_strategy_and_absent_bounds() {
    for text in [
        "calc(123456789012345678901234567890 + pi)",
        "round(up, 1.2500, .25)",
        "clamp(none, 2, none)",
    ] {
        let original = CssCalculation::<CssNumber>::try_from_components(
            parse_component_values(text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let cloned = original.clone();
        assert_eq!(cloned, original);
        drop(original);
        assert_eq!(
            cloned
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            text
        );
    }
}

#[test]
fn mixed_origin_clone_keeps_owned_structure_and_provenance_after_original_drop() {
    let children = parse_component_values("(1px + 2px) * 3 / 4 - min(5px, 6px)").unwrap();
    let CssValueOrigin::Parsed(parsed) = children.items()[0].origin().clone() else {
        panic!("parsed group");
    };
    let original = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::try_new(
            vec![CssComponentValue::try_function("CALC", children).unwrap()],
            Default::default(),
        )
        .unwrap(),
        Default::default(),
    )
    .unwrap();
    let numeric_type = original.numeric_type();
    let cloned = original.clone();
    assert_eq!(cloned, original);
    drop(original);

    assert_eq!(cloned.numeric_type(), numeric_type);
    assert!(matches!(cloned.origin(), CssValueOrigin::Programmatic));
    let CssCalculationExpressionRef::NestedCalc(root) = cloned.expression() else {
        panic!("outer calc");
    };
    let CssCalculationExpressionRef::Sum(sum) = root.operand() else {
        panic!("subtraction");
    };
    assert_eq!(sum.len(), 2);
    assert_eq!(
        sum.term(1).unwrap().operator(),
        Some(CssCalculationSumOperator::Subtract)
    );
    let CssCalculationExpressionRef::Product(product) = sum.term(0).unwrap().expression() else {
        panic!("product");
    };
    assert_eq!(product.len(), 3);
    assert_eq!(
        product.factor(1).unwrap().operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    assert_eq!(
        product.factor(2).unwrap().operator(),
        Some(CssCalculationProductOperator::Divide)
    );
    let CssCalculationExpressionRef::Group(group) = product.factor(0).unwrap().expression() else {
        panic!("group");
    };
    let CssCalculationExpressionRef::Sum(inner) = group.operand() else {
        panic!("inner sum");
    };
    let CssCalculationExpressionRef::Value(leaf) = inner.term(0).unwrap().expression() else {
        panic!("first length");
    };
    assert_eq!(leaf.literal().representation(), "1");
    assert_eq!(leaf.literal().unit(), Some("px"));
    let CssValueOrigin::Parsed(actual) = leaf.literal().origin() else {
        panic!("parsed leaf");
    };
    assert!(actual.source().same_snapshot(parsed.source()));
    assert_eq!(actual.span().start().byte_offset().value(), 1);
    assert_eq!(
        cloned
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc((1px + 2px) * 3 / 4 - min(5px, 6px))",
    );
    assert_eq!(
        CssNumeric::<CssLength>::try_from_calculation(cloned)
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "calc(-2.75px)",
    );
}
