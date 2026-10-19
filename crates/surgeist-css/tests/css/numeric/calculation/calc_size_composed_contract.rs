#![forbid(unsafe_code)]
//! Values5 WD2024-11-11 §10.1 requires specified simplification of each
//! calc-sum independently. Interpolation basis rewriting is a later operation.
//! Shared strict reentry retains the original occurrence and replacement origins.

use surgeist_css::*;

fn parsed_origin(origin: &CssValueOrigin, source: &str, start: usize, end: usize) {
    let CssValueOrigin::Parsed(original) = origin else {
        panic!("expected original parsed origin: {origin:?}")
    };
    assert_eq!(original.source().as_str(), source);
    assert_eq!(original.span().start().byte_offset().value(), start);
    assert_eq!(original.span().end().byte_offset().value(), end);
}

// Function origins identify actual opener tokens. Closing origins separately
// anchor the far end; only parsed_value identifies a whole declaration region.
fn function_boundaries(
    component: &CssComponentValue,
    source: &str,
    opening_start: usize,
    opening_end: usize,
    closing_start: usize,
    closing_end: usize,
) {
    parsed_origin(component.origin(), source, opening_start, opening_end);
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("original function component")
    };
    parsed_origin(
        function.closing_origin(),
        source,
        closing_start,
        closing_end,
    );
    let (CssValueOrigin::Parsed(opening), CssValueOrigin::Parsed(closing)) =
        (component.origin(), function.closing_origin())
    else {
        panic!("both delimiters were actually authored")
    };
    assert!(opening.source().same_snapshot(closing.source()));
}

fn parsed_value_region(declaration: &CssDeclaration, source: &str, start: usize, end: usize) {
    let original = declaration
        .parsed_value()
        .expect("original complete value region");
    assert_eq!(original.source().as_str(), source);
    assert_eq!(original.span().start().byte_offset().value(), start);
    assert_eq!(original.span().end().byte_offset().value(), end);
}

fn width_calc(declaration: &CssDeclaration) -> &CssCalcSize {
    let CssKnownPropertyValueRef::Width(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary width")
    };
    let CssSizeValue::BoxSize(CssBoxSize::CalcSize(value)) = wrapper.value() else {
        panic!("intrinsic calc-size width")
    };
    value.as_calc_size()
}

fn numeric_sum(expression: CssCalculationExpressionRef<'_>, expected: &[&str], percentage: bool) {
    let CssCalculationExpressionRef::Sum(sum) = expression else {
        panic!("original multi-term sum")
    };
    assert_eq!(sum.len(), expected.len());
    for (index, representation) in expected.iter().enumerate() {
        let CssCalculationExpressionRef::Value(value) = sum.term(index).unwrap().expression()
        else {
            panic!("original numeric leaf")
        };
        if percentage {
            assert!(matches!(value, CssCalculationValueRef::Percentage(_)));
        } else {
            assert!(matches!(value, CssCalculationValueRef::Length(_)));
            assert_eq!(value.literal().unit(), Some("px"));
        }
        assert_eq!(value.literal().representation(), *representation);
    }
}

fn size_then_lengths(expression: CssCalculationExpressionRef<'_>, expected: &[&str]) {
    let CssCalculationExpressionRef::Sum(sum) = expression else {
        panic!("original size-containing sum")
    };
    assert_eq!(sum.len(), expected.len() + 1);
    assert!(matches!(
        sum.term(0).unwrap().expression(),
        CssCalculationExpressionRef::Size(_)
    ));
    for (index, representation) in expected.iter().enumerate() {
        let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(value)) =
            sum.term(index + 1).unwrap().expression()
        else {
            panic!("original length leaf after symbolic size")
        };
        assert_eq!(value.unit(), Some("px"));
        assert_eq!(value.representation(), *representation);
    }
}

fn checked_and_raw_width(
    raw: &str,
    expected: &str,
    basis_end: usize,
    closing_start: usize,
) -> CssCalcSize {
    let components = parse_component_values(raw).unwrap();
    let original_components = components.clone();
    let [component] = components.items() else {
        panic!("one original calc-size function")
    };
    let value = CssCalcSize::try_from_component(component.clone()).unwrap();
    let original_value = value.clone();
    let origin = value.origin().clone();
    let basis_origin = value.basis_origin().clone();
    let calculation_origin = value.calculation().origin().clone();
    parsed_origin(value.origin(), raw, 0, 10);
    parsed_origin(value.basis_origin(), raw, 10, basis_end);
    function_boundaries(component, raw, 0, 10, closing_start, closing_start + 1);

    // Literal expected byte length applies to the complete output, including
    // both sums and every nested basis. No budget is reset between children.
    let exact_bytes = CssSpecifiedValueSerializationLimits::new(128, 128, expected.len());
    assert_eq!(
        value.serialize_specified_with_limits(exact_bytes).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 128, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(128, 0, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(128, 128, expected.len() - 1),
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
        assert_eq!(value, original_value);
        assert_eq!(value.origin(), &origin);
        assert_eq!(value.basis_origin(), &basis_origin);
        assert_eq!(value.calculation().origin(), &calculation_origin);
        assert_eq!(
            value.serialize_specified_with_limits(exact_bytes).unwrap(),
            expected
        );
    }
    assert_eq!(components, original_components);
    assert_eq!(components.serialize().unwrap().as_css(), raw);

    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let before_checked = checked.clone();
    assert_eq!(checked.value_components(), &components);
    assert_eq!(width_calc(&checked), &value);
    assert_eq!(width_calc(&checked).origin(), value.origin());
    assert_eq!(
        width_calc(&checked).serialize_specified().unwrap(),
        expected
    );
    assert_eq!(checked, before_checked);
    assert!(checked.same_occurrence(&before_checked));

    let report = parse_property_value_text(
        raw,
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        CssImportance::Important,
    );
    assert!(report.is_clean(), "{raw}: {:?}", report.diagnostics());
    let parsed = report.syntax().as_ref().unwrap();
    let before_parsed = parsed.clone();
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(width_calc(parsed), &value);
    parsed_origin(width_calc(parsed).origin(), raw, 0, 10);
    parsed_value_region(parsed, raw, 0, closing_start + 1);
    function_boundaries(
        &parsed.value_components().items()[0],
        raw,
        0,
        10,
        closing_start,
        closing_start + 1,
    );
    assert_eq!(width_calc(parsed).serialize_specified().unwrap(), expected);
    assert_eq!(parsed, &before_parsed);
    assert!(parsed.same_occurrence(&before_parsed));
    value
}

#[test]
fn both_length_sums_simplify_while_the_authored_sum_basis_remains_intrinsic() {
    let value = checked_and_raw_width(
        "calc-size(1px + 2px, size + 3px + 4px)",
        "calc-size(3px, 7px + size)",
        13,
        37,
    );
    let CssCalcSizeBasisRef::Sum(basis) = value.basis() else {
        panic!("specified projection must not replace the authored basis with any")
    };
    numeric_sum(basis, &["1", "2"], false);
    size_then_lengths(value.calculation(), &["3", "4"]);
}

#[test]
fn percentage_basis_simplifies_without_interpolation_de_percentification() {
    let value = checked_and_raw_width(
        "calc-size(25% + 25%, size + 1px + 2px)",
        "calc-size(50%, 3px + size)",
        13,
        37,
    );
    let CssCalcSizeBasisRef::Sum(basis) = value.basis() else {
        panic!("original percentage basis remains a sum")
    };
    numeric_sum(basis, &["25", "25"], true);
    size_then_lengths(value.calculation(), &["1", "2"]);
}

#[test]
fn nested_basis_and_both_calculations_simplify_without_flattening_the_basis() {
    let value = checked_and_raw_width(
        "calc-size(calc-size(1px + 2px, size + 3px), size + 4px)",
        "calc-size(calc-size(3px, 3px + size), 4px + size)",
        20,
        54,
    );
    let CssCalcSizeBasisRef::Nested(child) = value.basis() else {
        panic!("nested intrinsic identity survives specified output")
    };
    let CssCalcSizeBasisRef::Sum(basis) = child.basis() else {
        panic!("inner authored sum basis")
    };
    numeric_sum(basis, &["1", "2"], false);
    size_then_lengths(child.calculation(), &["3"]);
    size_then_lengths(value.calculation(), &["4"]);
    assert_eq!(value.basis_origin(), child.origin());
}

#[test]
fn pending_function_basis_reentry_preserves_occurrence_origins_and_allows_retry() {
    const SOURCE: &str = "width:calc-size(var(--basis), size + 1px)!important";
    let report = parse_style_attribute(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one authored pending width")
    };
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.known().unwrap().substitution_dependent().is_some());
    let original = source.clone();
    let components = source.value_components().clone();
    parsed_origin(components.items()[0].origin(), SOURCE, 6, 16);
    parsed_value_region(source, SOURCE, 6, 41);
    function_boundaries(&components.items()[0], SOURCE, 6, 16, 40, 41);
    let CssComponentValueRef::Function(original_function) = components.items()[0].view() else {
        panic!("original pending calc-size component")
    };
    function_boundaries(
        &original_function.values().items()[0],
        SOURCE,
        16,
        20,
        27,
        28,
    );
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("the whole property awaits substitution")
    };
    assert!(handle.source().same_occurrence(source));

    // Residual substitution anywhere in either calc-size argument is rejected
    // before ordinary grammar; failure leaves the original handle reusable.
    for residual in [
        "calc-size(var(--again), size + 2px)",
        "calc-size(min-content, size + env(inset))",
    ] {
        assert_eq!(
            handle
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(handle.source().same_occurrence(source));
        assert_eq!(handle.source().value_components(), &components);
    }
    assert!(matches!(
        handle
            .reenter(parse_component_values("calc-size(any, size)").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));

    const REPLACEMENT: &str = "calc-size(min-content, size + 2px + 3px)";
    let replacement = parse_component_values(REPLACEMENT).unwrap();
    let replacement_before = replacement.clone();
    function_boundaries(&replacement.items()[0], REPLACEMENT, 0, 10, 39, 40);
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed intrinsic width")
        };
        let [item] = values.items() else {
            panic!("one width contribution")
        };
        assert_eq!(item.property(), CssKnownProperty::Width);
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.source().value_components(), &components);
        assert_eq!(item.replacement_components(), Some(&replacement));
        let CssLonghandValueRef::Width(CssSizeValue::BoxSize(CssBoxSize::CalcSize(value))) =
            item.ordinary_value().unwrap().view()
        else {
            panic!("checked width calc-size payload")
        };
        let value = value.as_calc_size();
        assert!(matches!(
            value.basis(),
            CssCalcSizeBasisRef::Keyword(CssIntrinsicSizeKeyword::MinContent)
        ));
        parsed_origin(value.origin(), REPLACEMENT, 0, 10);
        parsed_origin(value.basis_origin(), REPLACEMENT, 10, 21);
        let CssCalculationExpressionRef::Sum(sum) = value.calculation() else {
            panic!("original replacement calculation")
        };
        parsed_origin(
            sum.term(0).unwrap().expression().origin(),
            REPLACEMENT,
            23,
            27,
        );
        parsed_origin(
            sum.term(1).unwrap().expression().origin(),
            REPLACEMENT,
            30,
            33,
        );
        parsed_origin(
            sum.term(2).unwrap().expression().origin(),
            REPLACEMENT,
            36,
            39,
        );
        assert_eq!(
            value.serialize_specified().unwrap(),
            "calc-size(min-content, 5px + size)"
        );
        size_then_lengths(value.calculation(), &["2", "3"]);
    }

    // Reuse that same handle for a genuinely nested percentage basis. Reentry
    // checks the complete replacement; it does not implement var substitution.
    const NESTED: &str = "calc-size(calc-size(25% + 25%, size + 2px), size + 3px)";
    let nested = parse_component_values(NESTED).unwrap();
    function_boundaries(&nested.items()[0], NESTED, 0, 10, 54, 55);
    let CssComponentValueRef::Function(nested_function) = nested.items()[0].view() else {
        panic!("original nested calc-size component")
    };
    function_boundaries(&nested_function.values().items()[0], NESTED, 10, 20, 41, 42);
    let CssContributions::Longhands(values) = handle.reenter(nested.clone()).unwrap() else {
        panic!("completed nested intrinsic width")
    };
    let [item] = values.items() else {
        panic!("one width contribution")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&nested));
    let CssLonghandValueRef::Width(CssSizeValue::BoxSize(CssBoxSize::CalcSize(value))) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("nested width payload")
    };
    let value = value.as_calc_size();
    let CssCalcSizeBasisRef::Nested(child) = value.basis() else {
        panic!("reentry retains nested basis")
    };
    let CssCalcSizeBasisRef::Sum(basis) = child.basis() else {
        panic!("nested percentage sum")
    };
    parsed_origin(value.origin(), NESTED, 0, 10);
    parsed_origin(child.origin(), NESTED, 10, 20);
    parsed_origin(child.basis_origin(), NESTED, 20, 23);
    numeric_sum(basis, &["25", "25"], true);
    size_then_lengths(child.calculation(), &["2"]);
    size_then_lengths(value.calculation(), &["3"]);
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc-size(calc-size(50%, 2px + size), 3px + size)"
    );

    assert_eq!(replacement, replacement_before);
    assert_eq!(source, &original);
    assert!(source.same_occurrence(&original));
    assert!(handle.source().same_occurrence(source));
    assert_eq!(handle.source().value_components(), &components);
}
