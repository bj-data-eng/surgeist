#![forbid(unsafe_code)]
//! Easing 1 §2.4 defines keyword and step-position output; Values 4
//! §§10.12–10.13 defer math range checks and integer rounding beyond specified values.
//! https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/#serialization

use surgeist_css::*;

fn parsed_for(property: CssKnownProperty, source: &str) -> CssEasingList {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(source).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::TransitionTimingFunction(value) => {
            value.timing_functions().clone()
        }
        CssKnownPropertyValueRef::AnimationTimingFunction(value) => {
            value.timing_functions().clone()
        }
        _ => panic!("easing list"),
    }
}

fn parsed(source: &str) -> CssEasingList {
    parsed_for(CssKnownProperty::TransitionTimingFunction, source)
}

fn number(source: &str) -> CssSpecifiedNumber {
    let components = parse_component_values(source).unwrap();
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(components).unwrap(),
    )
    .unwrap()
}

fn count(source: &str) -> CssPositiveIntegerValue {
    CssPositiveIntegerValue::Literal(
        CssPositiveIntegerLiteral::try_new(
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(source).unwrap())
                .unwrap(),
        )
        .unwrap(),
    )
}

fn assert_list(source: &str, expected: &str) {
    for property in [
        CssKnownProperty::TransitionTimingFunction,
        CssKnownProperty::AnimationTimingFunction,
    ] {
        let value = parsed_for(property, source);
        let original = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{source}");
        assert_eq!(
            parsed_for(property, expected)
                .serialize_specified()
                .unwrap(),
            expected
        );
        assert_eq!(value, original);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    65_536,
                    262_144,
                    expected.len(),
                ))
                .unwrap(),
            expected,
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    65_536,
                    262_144,
                    expected.len() - 1,
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        );
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, original);
    }
}

#[test]
fn keywords_keep_five_identities_and_canonicalize_only_step_aliases() {
    for (source, expected, keyword) in [
        ("LINEAR", "linear", CssEasingKeyword::Linear),
        ("ease", "ease", CssEasingKeyword::Ease),
        ("Ease-In", "ease-in", CssEasingKeyword::EaseIn),
        ("ease-out", "ease-out", CssEasingKeyword::EaseOut),
        ("ease-in-out", "ease-in-out", CssEasingKeyword::EaseInOut),
        ("step-start", "steps(1, start)", CssEasingKeyword::StepStart),
        ("step-end", "steps(1)", CssEasingKeyword::StepEnd),
    ] {
        assert_list(source, expected);
        let value = CssEasing::Keyword(keyword);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, CssEasing::Keyword(keyword));
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len(),
                ))
                .unwrap(),
            expected,
        );
    }
}

#[test]
fn positions_omit_end_aliases_and_preserve_start_spelling() {
    for (position, input, expected) in [
        (None, "steps(2)", "steps(2)"),
        (Some(CssStepPosition::End), "steps(2, end)", "steps(2)"),
        (
            Some(CssStepPosition::JumpEnd),
            "steps(2, jump-end)",
            "steps(2)",
        ),
        (
            Some(CssStepPosition::Start),
            "steps(2, start)",
            "steps(2, start)",
        ),
        (
            Some(CssStepPosition::JumpStart),
            "steps(2, jump-start)",
            "steps(2, jump-start)",
        ),
        (
            Some(CssStepPosition::JumpBoth),
            "steps(2, jump-both)",
            "steps(2, jump-both)",
        ),
        (
            Some(CssStepPosition::JumpNone),
            "steps(2, jump-none)",
            "steps(2, jump-none)",
        ),
    ] {
        assert_list(input, expected);
        let value = CssSteps::try_new(count("2"), position).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value.position(), position);
    }
}

#[test]
fn cubic_coordinates_share_exact_cssom_formatting_and_preserve_order() {
    // CSSOM ties away from zero, applied to exact authored decimals before
    // six-place formatting; Y coordinates are unrestricted signed Numbers.
    assert_list(
        "cubic-bezier(+000.0000005, -0.0000005, 1e0, +0002.5000000)",
        "cubic-bezier(0.000001, -0.000001, 1, 2.5)",
    );
    assert_list(
        "cubic-bezier(-0, -20, 1, 20)",
        "cubic-bezier(0, -20, 1, 20)",
    );
    // Equivalent curves retain function spelling rather than converting to a keyword.
    assert_list(
        "cubic-bezier(.25, .1, .25, 1)",
        "cubic-bezier(0.25, 0.1, 0.25, 1)",
    );
}

#[test]
fn specified_math_keeps_range_rounding_and_missing_context_deferred() {
    for (source, expected) in [
        (
            "cubic-bezier(calc(2), calc(-3), calc(-1), calc(4))",
            "cubic-bezier(calc(2), calc(-3), calc(-1), calc(4))",
        ),
        (
            "cubic-bezier(calc(0 / 0), sign(-2em), 1, calc(infinity))",
            "cubic-bezier(calc(NaN), sign(-2em), 1, calc(infinity))",
        ),
        ("steps(calc(1 + 1), jump-none)", "steps(calc(2), jump-none)"),
        ("steps(calc(1), jump-none)", "steps(calc(1), jump-none)"),
        ("steps(calc(0))", "steps(calc(0))"),
        ("steps(calc(-1), start)", "steps(calc(-1), start)"),
        ("steps(calc(1.5))", "steps(calc(1.5))"),
        (
            "steps(sign(-2em), jump-both)",
            "steps(sign(-2em), jump-both)",
        ),
        ("steps(calc(0 / 0), end)", "steps(calc(NaN))"),
    ] {
        assert_list(source, expected);
    }
}

#[test]
fn counts_retain_exact_large_integers_without_machine_rounding() {
    assert_list(
        "steps(+0009007199254740993, end)",
        "steps(9007199254740993)",
    );
    let huge = "9999999999999999999999999999999999999999999999999999999999999999999";
    assert_list(
        &format!("steps({huge}, jump-none)"),
        &format!("steps({huge}, jump-none)"),
    );
}

#[test]
fn checked_composition_retains_original_components_and_origins() {
    let x1 = number("+000.2500");
    let y1 = number("sign(-2em)");
    let x2 = number("1e0");
    let y2 = number("-2.0000005");
    let value = CssCubicBezier::try_new(x1.clone(), y1.clone(), x2.clone(), y2.clone()).unwrap();
    let originals = [x1, y1, x2, y2];
    let before: Vec<_> = originals
        .iter()
        .map(|value| value.origin().clone())
        .collect();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "cubic-bezier(0.25, sign(-2em), 1, -2.000001)"
    );
    for (index, actual) in [
        value.x1().value(),
        value.y1(),
        value.x2().value(),
        value.y2(),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(actual.origin(), &before[index]);
        if let Some(component) = originals[index].literal_component() {
            assert_eq!(actual.literal_component(), Some(component));
        } else {
            assert_eq!(
                actual.calculation().unwrap().components(),
                originals[index].calculation().unwrap().components()
            );
        }
    }
    let list = CssEasingList::try_new(vec![CssEasing::CubicBezier(value.clone())]).unwrap();
    assert_eq!(
        list.serialize_specified().unwrap(),
        value.serialize_specified().unwrap()
    );
    assert!(CssEasingList::try_new(Vec::new()).is_none());
}

#[test]
fn invalid_grammar_drops_only_easing_and_strict_validation_agrees() {
    for invalid in [
        "linear(0)",
        "spring(1,1,1,1)",
        "cubic-bezier(0,1,1)",
        "cubic-bezier(0 1 1 1)",
        "cubic-bezier(0,1,1,1,0)",
        "cubic-bezier(-0.0000000001,0,1,1)",
        "cubic-bezier(0,0,1.0000000001,1)",
        "steps(0)",
        "steps(-1)",
        "steps(1.0)",
        "steps(1e0)",
        "steps(1,jump-none)",
        "steps(2 start)",
        "steps(2,unknown)",
        "steps(2,start,end)",
        "ease,",
        "cubic-bezier(calc((1px + 1%) / 1px),0,1,1)",
        "steps(calc((1px + 1%) / 1px))",
    ] {
        let source = format!("transition-timing-function:{invalid};color:red");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{invalid}");
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn checked_ranges_and_pure_roots_share_admission_with_parsing() {
    for x in ["-0.0000000001", "1.0000000001"] {
        assert!(
            CssCubicBezier::try_new(number(x), number("0"), number("1"), number("1")).is_none()
        );
    }
    assert!(CssSteps::try_new(count("1"), Some(CssStepPosition::JumpNone)).is_none());
    for source in ["0", "-1"] {
        let integer =
            CssIntegerLiteral::try_from_component(CssComponentValue::try_number(source).unwrap())
                .unwrap();
        assert!(CssPositiveIntegerLiteral::try_new(integer).is_none());
    }
    let contextual = parse_component_values("calc((1px + 1%) / 1px)").unwrap();
    assert!(CssNumberCalculation::try_from_components(contextual.clone()).is_err());
    assert!(CssIntegerCalculation::try_from_components(contextual).is_err());
    let fraction =
        CssIntegerCalculation::try_from_components(parse_component_values("calc(1.5)").unwrap())
            .unwrap();
    let value = CssSteps::try_new(CssPositiveIntegerValue::Calculation(fraction), None).unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "steps(calc(1.5))");
}

fn assert_limits(value: &CssEasingList, expected: &str, input: usize, projection: usize) {
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input,
                projection,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection, expected.len() - 1),
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
        assert_eq!(value, &before);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn siblings_share_exact_aggregate_and_numeric_budgets() {
    // List 1; keyword 1; bezier 1 + four Number leaves; steps 1 + integer
    // leaf + explicit position. End is visited but omitted from projection.
    assert_limits(
        &parsed("ease, cubic-bezier(0, -2, 1, 3), steps(2, end)"),
        "ease, cubic-bezier(0, -2, 1, 3), steps(2)",
        10,
        9,
    );
    assert_limits(
        &parsed("steps(2, start), steps(2, jump-start)"),
        "steps(2, start), steps(2, jump-start)",
        7,
        7,
    );
    assert_limits(
        &parsed("steps(2), steps(2, jump-end)"),
        "steps(2), steps(2)",
        6,
        5,
    );
}

#[test]
fn standalone_aggregates_apply_the_same_exact_resource_contract() {
    let cubic =
        CssCubicBezier::try_new(number("0"), number("-2"), number("1"), number("3")).unwrap();
    let expected = "cubic-bezier(0, -2, 1, 3)";
    assert_eq!(
        cubic
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                5,
                5,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            cubic
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(cubic.serialize_specified().unwrap(), expected);
    }
    for (position, expected, inputs, projections) in [
        (None, "steps(2)", 2, 2),
        (Some(CssStepPosition::End), "steps(2)", 3, 2),
        (Some(CssStepPosition::JumpEnd), "steps(2)", 3, 2),
        (Some(CssStepPosition::Start), "steps(2, start)", 3, 3),
    ] {
        let steps = CssSteps::try_new(count("2"), position).unwrap();
        assert_eq!(
            steps
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    inputs,
                    projections,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(inputs - 1, projections, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(inputs, projections, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                steps
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(steps.position(), position);
            assert_eq!(steps.serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn folded_counts_keep_cumulative_numeric_work_and_atomic_failures() {
    // Each count visits its calc root, Sum, and two Number leaves (4 inputs).
    // Projection allocates two scalar leaves and their merged scalar (3).
    // Each steps aggregate and emitted start contributes one of each;
    // the enclosing list contributes one of each: inputs 13, projections 11.
    assert_limits(
        &parsed("steps(calc(1 + 1), start), steps(calc(2 + 1), start)"),
        "steps(calc(2), start), steps(calc(3), start)",
        13,
        11,
    );
}

#[test]
fn list_order_and_shorthand_omissions_survive_canonical_output() {
    assert_list(
        "ease-out, step-start, steps(4, jump-start), step-end, linear",
        "ease-out, steps(1, start), steps(4, jump-start), steps(1), linear",
    );
    let report = parse_style_attribute("transition:opacity 1s;animation:fade 2s step-start");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::Transition(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("transition")
    };
    assert!(value.transitions().values()[0].timing_function().is_none());
    let CssKnownPropertyValueRef::Animation(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("animation")
    };
    assert_eq!(
        value.animations().values()[0]
            .timing_function()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "steps(1, start)"
    );
    assert!(matches!(
        value.animations().values()[0].timing_function(),
        Some(CssEasing::Keyword(CssEasingKeyword::StepStart))
    ));
}

#[test]
fn both_timing_families_expand_their_typed_easing_lists() {
    let report =
        parse_style_attribute("transition-timing-function:steps(2,end);animation:fade 1s ease");
    assert!(report.is_clean());
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&report.syntax()[0]).unwrap()
    else {
        panic!("ordinary transition easing")
    };
    let [item] = values.items() else {
        panic!("one easing longhand")
    };
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::TransitionTimingFunction(easing)) =
        item.value()
    else {
        panic!("typed transition easing")
    };
    assert_eq!(
        easing.values()[0].serialize_specified().unwrap(),
        "steps(2)"
    );
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&report.syntax()[1]).unwrap()
    else {
        panic!("ordinary animation")
    };
    assert_eq!(values.items().len(), 8);
    let item = &values.items()[1];
    assert_eq!(item.property(), CssKnownProperty::AnimationTimingFunction);
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::AnimationTimingFunction(easing)) =
        item.value()
    else {
        panic!("typed animation easing")
    };
    assert_eq!(
        easing.values(),
        &[CssEasing::Keyword(CssEasingKeyword::Ease)]
    );
    assert!(item.source().same_occurrence(&report.syntax()[1]));
}
