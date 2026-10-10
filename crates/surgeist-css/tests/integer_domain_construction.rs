#![forbid(unsafe_code)]

//! Checked integer owners preserve exact authored tokens. Independent ordering
//! expectations follow mathematical signed integers; positive repeat/step counts,
//! inclusive counter bounds and descending additive weights follow their selected
//! CSS contracts. Constructors do not impose downstream implementation ranges.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use std::cmp::Ordering;
use surgeist_css::*;

fn integer(text: &str) -> CssIntegerLiteral {
    CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}

fn positive(text: &str) -> CssPositiveIntegerLiteral {
    CssPositiveIntegerLiteral::try_new(integer(text)).unwrap()
}

fn symbol(text: &str) -> CssCounterSymbol {
    CssCounterSymbol::String(CssContentString::try_new(text).unwrap())
}

fn bound(text: &str) -> CssCounterStyleRangeBound {
    CssCounterStyleRangeBound::Integer(CssIntegerValue::Literal(integer(text)))
}

fn parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed origin");
    };
    let start = source.find(token).unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + token.len()
    );
}

fn auto_repeat(count: CssPositiveIntegerLiteral) -> CssGridIntegerTrackRepeat {
    let content = CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(
        CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto()),
    )])
    .unwrap();
    CssGridIntegerTrackRepeat::try_new(CssPositiveIntegerValue::Literal(count), content).unwrap()
}

#[test]
fn mathematical_comparison_handles_signs_zeroes_digit_lengths_and_huge_adjacent_values() {
    for (left, right, expected) in [
        ("+0007", "7", Ordering::Equal),
        ("-0000", "+0", Ordering::Equal),
        (
            "-999999999999999999999999999999",
            "-999999999999999999999999999998",
            Ordering::Less,
        ),
        ("2147483648", "2147483649", Ordering::Less),
        ("-2147483649", "-2147483648", Ordering::Less),
        (
            "999999999999999999999999999999",
            "1000000000000000000000000000000",
            Ordering::Less,
        ),
        ("-10", "-9", Ordering::Less),
        ("-1", "-0", Ordering::Less),
        ("-0", "1", Ordering::Less),
        ("+0010", "9", Ordering::Greater),
        ("-00010", "-010", Ordering::Equal),
    ] {
        let left = integer(left);
        let right = integer(right);
        assert_eq!(left.compare_value(&right), expected);
        assert_eq!(right.compare_value(&left), expected.reverse());
    }
}

#[test]
fn mathematical_comparison_preserves_distinct_lexical_and_provenance_equality_contracts() {
    let parsed_component = parse_component_values("+0007").unwrap().items()[0].clone();
    let parsed = CssIntegerLiteral::try_from_component(parsed_component.clone()).unwrap();
    let programmatic = integer("+0007");
    assert_eq!(parsed.compare_value(&programmatic), Ordering::Equal);
    assert_ne!(parsed, programmatic);
    assert_ne!(integer("+0007"), integer("7"));
    assert_ne!(integer("-0"), integer("+0"));
    assert_eq!(parsed.component(), &parsed_component);
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        CssPositiveIntegerLiteral::try_new(parsed).unwrap(),
        CssPositiveIntegerLiteral::try_new(programmatic).unwrap(),
    );
    assert_ne!(positive("+0007"), positive("7"));
}

#[test]
fn positive_literal_admission_rejects_noninteger_kinds_zero_and_negative_values() {
    for text in ["0", "+0000", "-0", "-1", "-2147483649"] {
        assert!(
            CssPositiveIntegerLiteral::try_new(integer(text)).is_none(),
            "{text}"
        );
    }
    for text in ["2.0", "2e0", "2px", "2%", "calc(2)"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        assert!(
            CssIntegerLiteral::try_from_component(component).is_err(),
            "{text}"
        );
    }
    for text in ["+0007", "2147483648", "999999999999999999999999999999"] {
        assert_eq!(positive(text).integer().numeric().representation(), text);
    }
}

#[test]
fn both_grid_repeat_owners_retain_checked_count_components_and_origins() {
    for text in ["+0007", "2147483648"] {
        for component in [
            CssComponentValue::try_number(text).unwrap(),
            parse_component_values(text).unwrap().items()[0].clone(),
        ] {
            let count = CssPositiveIntegerLiteral::try_new(
                CssIntegerLiteral::try_from_component(component.clone()).unwrap(),
            )
            .unwrap();
            let track = auto_repeat(count.clone());
            let size = CssGridTrackSize::from_breadth(CssGridTrackBreadth::from_length_percentage(
                CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                    CssComponentValue::try_dimension("1", "px").unwrap(),
                )
                .unwrap(),
            ));
            let content =
                CssGridFixedRepeatContent::try_new(vec![CssGridFixedRepeatComponent::FixedSize(
                    CssGridFixedSize::try_new(size).unwrap(),
                )])
                .unwrap();
            let fixed = CssGridIntegerFixedRepeat::try_new(
                CssPositiveIntegerValue::Literal(count),
                content,
            )
            .unwrap();
            for retained in [track.count(), fixed.count()] {
                let CssPositiveIntegerValue::Literal(retained) = retained else {
                    panic!("ordinary exact positive count");
                };
                assert_eq!(retained.integer().component(), &component);
                assert_eq!(retained.integer().numeric().representation(), text);
                assert_eq!(retained.integer().origin(), component.origin());
            }
        }
    }
}

#[test]
fn parsed_repeat_count_keeps_original_snapshot_and_precise_token_span() {
    let source = "grid-template-columns: repeat(+0007, 1fr)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let Some(CssKnownPropertyValueRef::GridTemplateColumns(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("checked columns");
    };
    let [CssGridGeneralTrackComponent::Repeat(repeat)] =
        value.value().general_list().unwrap().components()
    else {
        panic!("one integer repeat");
    };
    let CssPositiveIntegerValue::Literal(count) = repeat.count() else {
        panic!("ordinary exact positive count");
    };
    let count = count.integer();
    assert_eq!(count.numeric().representation(), "+0007");
    parsed_origin(count.origin(), source, "+0007");
    let CssValueOrigin::Parsed(origin) = count.origin() else {
        panic!("parsed count");
    };
    assert!(
        origin
            .source()
            .same_snapshot(declaration.parsed_value().unwrap().source())
    );
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "repeat(7, 1fr)"
    );
}

#[test]
fn repeat_counts_share_cumulative_input_projection_and_canonical_byte_budgets() {
    let list = CssGridTrackList::general(
        CssGridGeneralTrackList::try_new(vec![
            CssGridGeneralTrackComponent::Repeat(auto_repeat(positive("+0007"))),
            CssGridGeneralTrackComponent::Repeat(auto_repeat(positive("2147483648"))),
        ])
        .unwrap(),
    );
    let expected = "repeat(7, auto) repeat(2147483648, auto)";
    assert_eq!(list.serialize_specified().unwrap(), expected);
    // One list plus two repetitions, each with a count and keyword track: seven nodes.
    let exact = CssSpecifiedValueSerializationLimits::new(7, 7, expected.len());
    assert_eq!(
        list.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    for (limits, expected_kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 7, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            list.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected_kind
        );
    }
    assert_eq!(list.serialize_specified().unwrap(), expected);
}

#[test]
fn fixed_system_keeps_omission_distinct_from_explicit_default_and_huge_signed_starts() {
    let omitted = CssCounterStyleFixedSystem::new(None);
    assert!(omitted.first_symbol_value().is_none());
    for text in [
        "1",
        "+0001",
        "-2147483649",
        "999999999999999999999999999999",
    ] {
        let start = CssIntegerValue::Literal(integer(text));
        let fixed = CssCounterStyleFixedSystem::new(Some(start.clone()));
        assert_eq!(fixed.first_symbol_value(), Some(&start));
        assert_eq!(
            fixed
                .first_symbol_value()
                .unwrap()
                .literal()
                .numeric()
                .representation(),
            text
        );
        assert_ne!(fixed, omitted);
    }
    assert_ne!(
        CssCounterStyleFixedSystem::new(Some(CssIntegerValue::Literal(integer("1")))),
        CssCounterStyleFixedSystem::new(Some(CssIntegerValue::Literal(integer("+0001")))),
    );
}

#[test]
fn counter_ranges_validate_mathematical_order_and_preserve_interval_order_and_infinity() {
    let first =
        CssCounterStyleRangeInterval::try_new(bound("2147483648"), bound("2147483649")).unwrap();
    let equal = CssCounterStyleRangeInterval::try_new(bound("+0007"), bound("7")).unwrap();
    assert!(
        CssCounterStyleRangeInterval::try_new(bound("2147483649"), bound("2147483648")).is_none()
    );
    assert!(CssCounterStyleRangeInterval::try_new(bound("-9"), bound("-10")).is_none());
    let infinite = CssCounterStyleRangeInterval::try_new(
        CssCounterStyleRangeBound::Infinite,
        CssCounterStyleRangeBound::Infinite,
    )
    .unwrap();
    let list = CssCounterStyleRanges::try_new(vec![equal.clone(), first.clone(), infinite.clone()])
        .unwrap();
    assert_eq!(list.ranges(), [equal, first, infinite]);
    assert!(
        matches!(list.ranges()[0].lower(), CssCounterStyleRangeBound::Integer(value) if value.literal().numeric().representation() == "+0007")
    );
    assert_eq!(
        list.ranges()[2].lower(),
        &CssCounterStyleRangeBound::Infinite
    );
    assert_eq!(
        list.ranges()[2].upper(),
        &CssCounterStyleRangeBound::Infinite
    );
    assert!(CssCounterStyleRanges::try_new(Vec::new()).is_none());
}

#[test]
fn pad_and_additive_tuples_keep_nonnegative_exact_tokens_including_signed_zero() {
    for text in ["-0000", "+0", "+0007", "2147483648"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        let literal = CssIntegerLiteral::try_from_component(component.clone()).unwrap();
        let pad =
            CssCounterStylePad::try_new(CssIntegerValue::Literal(literal.clone()), symbol("_"))
                .unwrap();
        let tuple =
            CssCounterAdditiveTuple::try_new(CssIntegerValue::Literal(literal), symbol("x"))
                .unwrap();
        for value in [pad.minimum_length(), tuple.weight()] {
            assert_eq!(value.literal().component(), &component);
            assert_eq!(value.literal().numeric().representation(), text);
            assert_eq!(value.literal().origin(), component.origin());
        }
        assert!(matches!(pad.symbol(), CssCounterSymbol::String(value) if value.as_str() == "_"));
        assert!(matches!(tuple.symbol(), CssCounterSymbol::String(value) if value.as_str() == "x"));
    }
    for text in ["-1", "-2147483649"] {
        assert!(
            CssCounterStylePad::try_new(CssIntegerValue::Literal(integer(text)), symbol("_"))
                .is_none()
        );
        assert!(
            CssCounterAdditiveTuple::try_new(CssIntegerValue::Literal(integer(text)), symbol("x"))
                .is_none()
        );
    }
}

#[test]
fn additive_lists_require_strict_mathematical_descent_without_sorting_or_deduplication() {
    let tuple = |weight, text| {
        CssCounterAdditiveTuple::try_new(CssIntegerValue::Literal(integer(weight)), symbol(text))
            .unwrap()
    };
    let input = vec![
        tuple("2147483649", "a"),
        tuple("2147483648", "b"),
        tuple("-0000", "c"),
    ];
    let list = CssCounterAdditiveSymbols::try_new(input.clone()).unwrap();
    assert_eq!(list.tuples(), input);
    for weights in [["7", "+0007"], ["-0", "+0"], ["2147483648", "2147483649"]] {
        assert!(
            CssCounterAdditiveSymbols::try_new(vec![
                tuple(weights[0], "a"),
                tuple(weights[1], "b")
            ])
            .is_none()
        );
    }
    assert!(CssCounterAdditiveSymbols::try_new(Vec::new()).is_none());
}

#[test]
fn parsed_counter_fields_retain_huge_lexemes_exact_origins_and_descriptor_order() {
    let source = concat!(
        "@counter-style exact { system: fixed -0002147483649; symbols: \"x\"; ",
        "range: +0002147483648 2147483649; pad: 2147483648 \"_\"; ",
        "additive-symbols: 2147483649 \"a\", 2147483648 \"b\"; }",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("counter rule");
    };
    let descriptors = rule.descriptors();
    let Some(CssCounterStyleSystem::Fixed(fixed)) =
        descriptors.system().map(|value| value.ordinary_system())
    else {
        panic!("fixed system");
    };
    let start = fixed.first_symbol_value().unwrap();
    assert_eq!(start.literal().numeric().representation(), "-0002147483649");
    parsed_origin(start.literal().origin(), source, "-0002147483649");
    let CssCounterStyleRange::Ranges(ranges) = descriptors.range().unwrap().ordinary_range() else {
        panic!("finite range");
    };
    let CssCounterStyleRangeBound::Integer(lower) = ranges.ranges()[0].lower() else {
        panic!("finite lower bound");
    };
    parsed_origin(lower.literal().origin(), source, "+0002147483648");
    assert_eq!(lower.literal().numeric().representation(), "+0002147483648");
    assert_eq!(
        descriptors
            .pad()
            .unwrap()
            .ordinary_pad()
            .minimum_length()
            .literal()
            .numeric()
            .representation(),
        "2147483648"
    );
    assert_eq!(
        descriptors
            .additive_symbols()
            .unwrap()
            .ordinary_additive_symbols()
            .tuples()
            .iter()
            .map(|value| value.weight().literal().numeric().representation())
            .collect::<Vec<_>>(),
        ["2147483649", "2147483648"],
    );
    let positions = descriptors
        .occurrences()
        .map(|value| {
            let position = match value {
                CssCounterStyleDescriptorRef::System(value) => value.position(),
                CssCounterStyleDescriptorRef::Symbols(value) => value.position(),
                CssCounterStyleDescriptorRef::Range(value) => value.position(),
                CssCounterStyleDescriptorRef::Pad(value) => value.position(),
                CssCounterStyleDescriptorRef::AdditiveSymbols(value) => value.position(),
                _ => panic!("unexpected descriptor"),
            };
            position.byte_offset().value()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        positions,
        ["system:", "symbols:", "range:", "pad:", "additive-symbols:"]
            .map(|name| source.find(name).unwrap())
    );
}

#[test]
fn counter_effective_system_coupling_and_neighbor_recovery_remain_checked() {
    for body in [
        "system: numeric; symbols: \"x\";",
        "system: alphabetic; symbols: \"x\";",
        "system: additive;",
    ] {
        // Counter Styles 3 §§3.1/3.8 retain syntactically valid rules that
        // do not define a usable counter style; the integer model is unchanged.
        let source = format!("@counter-style incomplete {{ {body} }} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(
            matches!(
                report.syntax().rules(),
                [CssRule::CounterStyle(_), CssRule::Style(_)]
            ),
            "{source}"
        );
    }
    for body in [
        "system: extends base; symbols: \"x\";",
        "system: extends base; additive-symbols: 1 \"x\";",
    ] {
        let source = format!("@counter-style bad {{ {body} }} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean());
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::CounterStyle(_), CssRule::Style(_)]
        ));
    }
    let source = "@counter-style valid { symbols: \"x\"; range: 2147483649 2147483648; pad: -1 \"_\"; suffix: \".\"; } .after { color: red; }";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("neighbors survive");
    };
    assert!(rule.descriptors().system().is_none());
    assert!(rule.descriptors().range().is_none());
    assert!(rule.descriptors().pad().is_none());
    assert!(rule.descriptors().suffix().is_some());
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|value| value.action() == CssRecoveryAction::DropDescriptor)
    );
}

#[test]
fn steps_share_positive_literals_and_defer_calculation_range_with_authored_omission() {
    for text in ["1", "+0001"] {
        assert!(
            CssSteps::try_new(
                CssPositiveIntegerValue::Literal(positive(text)),
                Some(CssStepPosition::JumpNone)
            )
            .is_none()
        );
        assert!(
            CssSteps::try_new(CssPositiveIntegerValue::Literal(positive(text)), None).is_some()
        );
    }
    for text in ["2", "2147483648"] {
        let steps = CssSteps::try_new(
            CssPositiveIntegerValue::Literal(positive(text)),
            Some(CssStepPosition::JumpNone),
        )
        .unwrap();
        assert!(
            matches!(steps.count(), CssPositiveIntegerValue::Literal(value) if value.integer().numeric().representation() == text)
        );
    }
    let components = parse_component_values("calc(1)").unwrap();
    let calculation = CssIntegerCalculation::try_from_components(components.clone()).unwrap();
    let steps = CssSteps::try_new(
        CssPositiveIntegerValue::Calculation(calculation),
        Some(CssStepPosition::JumpNone),
    )
    .unwrap();
    let CssPositiveIntegerValue::Calculation(calculation) = steps.count() else {
        panic!("symbolic count");
    };
    assert_eq!(calculation.components(), &components);
    let omitted = CssSteps::try_new(CssPositiveIntegerValue::Literal(positive("2")), None).unwrap();
    let explicit = CssSteps::try_new(
        CssPositiveIntegerValue::Literal(positive("2")),
        Some(CssStepPosition::End),
    )
    .unwrap();
    assert_eq!(omitted.position(), None);
    assert_eq!(explicit.position(), Some(CssStepPosition::End));
    assert_ne!(omitted, explicit);
}

#[test]
fn steps_classify_bare_calculation_tokens_without_losing_authored_components_or_origins() {
    let source = " /*ordinary*/ +0002 ";
    let components = parse_component_values(source).unwrap();
    let original = components
        .items()
        .iter()
        .find(|component| {
            matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        })
        .unwrap()
        .clone();
    let steps = CssSteps::try_new(
        CssPositiveIntegerValue::Calculation(
            CssIntegerCalculation::try_from_components(components).unwrap(),
        ),
        Some(CssStepPosition::JumpNone),
    )
    .unwrap();
    let CssPositiveIntegerValue::Literal(literal) = steps.count() else {
        panic!("ordinary bare integer root");
    };
    assert_eq!(literal.integer().component(), &original);
    assert_eq!(literal.integer().numeric().representation(), "+0002");
    parsed_origin(literal.integer().origin(), source, "+0002");
    assert_eq!(steps.position(), Some(CssStepPosition::JumpNone));

    let bare = CssSteps::try_new(
        CssPositiveIntegerValue::Calculation(CssIntegerCalculation::literal(2)),
        None,
    )
    .unwrap();
    let CssPositiveIntegerValue::Literal(literal) = bare.count() else {
        panic!("programmatic bare integer root");
    };
    assert_eq!(literal.integer().component(), integer("2").component());
    assert_eq!(literal.integer().origin(), &CssValueOrigin::Programmatic);

    for source in ["calc(0)", "calc(-1)", "calc(1)"] {
        let components = parse_component_values(source).unwrap();
        let steps = CssSteps::try_new(
            CssPositiveIntegerValue::Calculation(
                CssIntegerCalculation::try_from_components(components.clone()).unwrap(),
            ),
            Some(CssStepPosition::JumpNone),
        )
        .unwrap();
        let CssPositiveIntegerValue::Calculation(calculation) = steps.count() else {
            panic!("genuine symbolic function");
        };
        assert_eq!(calculation.components(), &components);
        parsed_origin(calculation.origin(), source, "calc(");
    }
}

#[test]
fn parsed_steps_keep_count_origins_symbolic_math_and_position_omission() {
    let source = "transition-timing-function: steps(+0002, jump-none), steps(calc(1), jump-none), steps(2147483648), steps(2, end)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssKnownPropertyValueRef::TransitionTimingFunction(value)) =
        report.syntax()[0].known().unwrap().property_value()
    else {
        panic!("easing list");
    };
    let steps = value
        .timing_functions()
        .values()
        .iter()
        .map(|value| {
            let CssEasing::Steps(steps) = value else {
                panic!("steps");
            };
            steps
        })
        .collect::<Vec<_>>();
    let CssPositiveIntegerValue::Literal(first) = steps[0].count() else {
        panic!("ordinary count");
    };
    assert_eq!(first.integer().numeric().representation(), "+0002");
    parsed_origin(first.integer().origin(), source, "+0002");
    let CssPositiveIntegerValue::Calculation(calculation) = steps[1].count() else {
        panic!("math count");
    };
    parsed_origin(calculation.expression().origin(), source, "calc(");
    assert_eq!(
        calculation.components().serialize().unwrap().as_css(),
        "calc(1)"
    );
    assert_eq!(steps[1].position(), Some(CssStepPosition::JumpNone));
    assert!(
        matches!(steps[2].count(), CssPositiveIntegerValue::Literal(value) if value.integer().numeric().representation() == "2147483648")
    );
    assert_eq!(steps[2].position(), None);
    assert_eq!(steps[3].position(), Some(CssStepPosition::End));
}
