//! Checked integer owners preserve exact authored tokens. Independent ordering
//! expectations follow mathematical signed integers; positive repeat/step counts,
//! inclusive counter bounds and descending additive weights follow their selected
//! CSS contracts. Constructors do not impose downstream implementation ranges.

use crate::css::support::descriptor_phases;
use descriptor_phases::*;

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use std::cmp::Ordering;
use surgeist_css::*;

fn integer(text: &str) -> CssNumericLiteral<CssInteger> {
    CssNumericLiteral::<CssInteger>::try_from_component(
        CssComponentValue::try_number(text).unwrap(),
    )
    .unwrap()
}

/// An exact integer literal that the `[1,∞]` range admits.
fn positive(text: &str) -> CssNumericLiteral<CssInteger> {
    let literal = integer(text);
    assert!(CssNumeric::<CssInteger, CssPositive>::try_from_literal(literal.clone()).is_ok());
    literal
}

fn symbol(text: &str) -> CssCounterSymbol {
    CssCounterSymbol::String(CssContentString::try_new(text).unwrap())
}

fn bound(text: &str) -> CssCounterStyleRangeBound {
    CssCounterStyleRangeBound::Integer(
        CssNumeric::<CssInteger>::try_from_literal(integer(text)).unwrap(),
    )
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

fn auto_repeat(count: CssNumericLiteral<CssInteger>) -> CssGridIntegerTrackRepeat {
    let content = CssGridTrackRepeatContent::try_new(vec![CssGridTrackRepeatComponent::TrackSize(
        CssGridTrackSize::from_breadth(CssGridTrackBreadth::auto()),
    )])
    .unwrap();
    CssGridIntegerTrackRepeat::new(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(count).unwrap(),
        content,
    )
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
fn mathematical_comparison_and_equality_ignore_lexical_spelling_and_provenance() {
    let parsed_component = CssComponentValues::parse(&CssParser::default(), "+0007")
        .unwrap()
        .items()[0]
        .clone();
    let parsed =
        CssNumericLiteral::<CssInteger>::try_from_component(parsed_component.clone()).unwrap();
    let programmatic = integer("+0007");
    assert_eq!(parsed.compare_value(&programmatic), Ordering::Equal);
    assert_eq!(parsed, programmatic);
    assert_eq!(integer("+0007"), integer("7"));
    assert_eq!(integer("-0"), integer("+0"));
    assert_eq!(parsed.component(), &parsed_component);
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    // A semantic owner compares the exact lexical integer without provenance.
    let count = |literal| {
        CssColumnCount::Count(
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(literal).unwrap(),
        )
    };
    assert_eq!(count(parsed), count(programmatic));
    assert_eq!(count(positive("+0007")), count(positive("7")));
}

#[test]
fn positive_literal_admission_rejects_noninteger_kinds_zero_and_negative_values() {
    for text in ["0", "+0000", "-0", "-1", "-2147483649"] {
        let error =
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(integer(text)).unwrap_err();
        assert!(error.is_out_of_range(), "{text}");
    }
    for text in ["2.0", "2e0", "2px", "2%", "calc(2)"] {
        let component = CssComponentValues::parse(&CssParser::default(), text)
            .unwrap()
            .items()[0]
            .clone();
        let origin = component.origin().clone();
        let error = CssNumericLiteral::<CssInteger>::try_from_component(component).unwrap_err();
        assert!(error.is_root_domain_mismatch(), "{text}");
        assert_eq!(error.origin(), Some(&origin), "{text}");
    }
    for text in ["+0007", "2147483648", "999999999999999999999999999999"] {
        assert_eq!(positive(text).numeric().representation(), text);
    }
}

#[test]
fn both_grid_repeat_owners_retain_checked_count_components_and_origins() {
    for text in ["+0007", "2147483648"] {
        for component in [
            CssComponentValue::try_number(text).unwrap(),
            CssComponentValues::parse(&CssParser::default(), text)
                .unwrap()
                .items()[0]
                .clone(),
        ] {
            let count =
                CssNumericLiteral::<CssInteger>::try_from_component(component.clone()).unwrap();
            let track = auto_repeat(count.clone());
            let size = CssGridTrackSize::from_breadth(CssGridTrackBreadth::from_length_percentage(
                CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
                    CssComponentValue::try_dimension("1", "px").unwrap(),
                )
                .unwrap(),
            ));
            let content =
                CssGridFixedRepeatContent::try_new(vec![CssGridFixedRepeatComponent::FixedSize(
                    CssGridFixedSize::try_new(size).unwrap(),
                )])
                .unwrap();
            let fixed = CssGridIntegerFixedRepeat::new(
                CssNumeric::<CssInteger, CssPositive>::try_from_literal(count).unwrap(),
                content,
            );
            for retained in [track.count(), fixed.count()] {
                let Some(retained) = retained.literal() else {
                    panic!("ordinary exact positive count");
                };
                assert_eq!(retained.component(), &component);
                assert_eq!(retained.numeric().representation(), text);
                assert_eq!(retained.origin(), component.origin());
            }
        }
    }
}

#[test]
fn parsed_repeat_count_keeps_original_snapshot_and_precise_token_span() {
    let source = "grid-template-columns: repeat(+0007, 1fr)";
    let report = CssParser::default().parse_block_contents(source);
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
    let Some(count) = repeat.count().literal() else {
        panic!("ordinary exact positive count");
    };
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
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
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
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        expected
    );
    // One list plus two repetitions, each with a count and keyword track: seven nodes.
    let exact = CssSerializationLimits::new(7, 7, expected.len());
    assert_eq!(list.serialize_specified(exact).unwrap(), expected);
    for (limits, expected_kind) in [
        (
            CssSerializationLimits::new(6, 7, expected.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 6, expected.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 7, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&list.serialize_specified(limits).unwrap_err()),
            expected_kind
        );
    }
    assert_eq!(
        list.serialize_specified(Default::default()).unwrap(),
        expected
    );
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
        let start = CssNumeric::<CssInteger>::try_from_literal(integer(text)).unwrap();
        let fixed = CssCounterStyleFixedSystem::new(Some(start.clone()));
        assert_eq!(fixed.first_symbol_value(), Some(&start));
        assert_eq!(
            fixed
                .first_symbol_value()
                .unwrap()
                .expect_literal()
                .numeric()
                .representation(),
            text
        );
        assert_ne!(fixed, omitted);
    }
    assert_eq!(
        CssCounterStyleFixedSystem::new(Some(
            CssNumeric::<CssInteger>::try_from_literal(integer("1")).unwrap()
        )),
        CssCounterStyleFixedSystem::new(Some(
            CssNumeric::<CssInteger>::try_from_literal(integer("+0001")).unwrap()
        )),
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
        matches!(list.ranges()[0].lower(), CssCounterStyleRangeBound::Integer(value) if value.expect_literal().numeric().representation() == "+0007")
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
        let component = CssComponentValues::parse(&CssParser::default(), text)
            .unwrap()
            .items()[0]
            .clone();
        let literal =
            CssNumericLiteral::<CssInteger>::try_from_component(component.clone()).unwrap();
        let pad = CssCounterStylePad::try_new(
            CssNumeric::<CssInteger>::try_from_literal(literal.clone()).unwrap(),
            symbol("_"),
        )
        .unwrap();
        let tuple = CssCounterAdditiveTuple::try_new(
            CssNumeric::<CssInteger>::try_from_literal(literal).unwrap(),
            symbol("x"),
        )
        .unwrap();
        for value in [pad.minimum_length(), tuple.weight()] {
            assert_eq!(value.expect_literal().component(), &component);
            assert_eq!(value.expect_literal().numeric().representation(), text);
            assert_eq!(value.expect_literal().origin(), component.origin());
        }
        assert!(matches!(pad.symbol(), CssCounterSymbol::String(value) if value.as_str() == "_"));
        assert!(matches!(tuple.symbol(), CssCounterSymbol::String(value) if value.as_str() == "x"));
    }
    for text in ["-1", "-2147483649"] {
        assert!(
            CssCounterStylePad::try_new(
                CssNumeric::<CssInteger>::try_from_literal(integer(text)).unwrap(),
                symbol("_")
            )
            .is_none()
        );
        assert!(
            CssCounterAdditiveTuple::try_new(
                CssNumeric::<CssInteger>::try_from_literal(integer(text)).unwrap(),
                symbol("x")
            )
            .is_none()
        );
    }
}

#[test]
fn additive_lists_require_strict_mathematical_descent_without_sorting_or_deduplication() {
    let tuple = |weight, text| {
        CssCounterAdditiveTuple::try_new(
            CssNumeric::<CssInteger>::try_from_literal(integer(weight)).unwrap(),
            symbol(text),
        )
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
    let report = CssParser::default().parse_sheet(source);
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
    assert_eq!(
        start.expect_literal().numeric().representation(),
        "-0002147483649"
    );
    parsed_origin(start.expect_literal().origin(), source, "-0002147483649");
    let CssCounterStyleRange::Ranges(ranges) = descriptors.range().unwrap().ordinary_range() else {
        panic!("finite range");
    };
    let CssCounterStyleRangeBound::Integer(lower) = ranges.ranges()[0].lower() else {
        panic!("finite lower bound");
    };
    parsed_origin(lower.expect_literal().origin(), source, "+0002147483648");
    assert_eq!(
        lower.expect_literal().numeric().representation(),
        "+0002147483648"
    );
    assert_eq!(
        descriptors
            .pad()
            .unwrap()
            .ordinary_pad()
            .minimum_length()
            .expect_literal()
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
            .map(|value| value.weight().expect_literal().numeric().representation())
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
        let report = CssParser::default().parse_sheet(&source);
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
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean());
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::CounterStyle(_), CssRule::Style(_)]
        ));
    }
    let source = "@counter-style valid { symbols: \"x\"; range: 2147483649 2147483648; pad: -1 \"_\"; suffix: \".\"; } .after { color: red; }";
    let report = CssParser::default().parse_sheet(source);
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
                CssNumeric::<CssInteger, CssPositive>::try_from_literal(positive(text)).unwrap(),
                Some(CssStepPosition::JumpNone)
            )
            .is_none()
        );
        assert!(
            CssSteps::try_new(
                CssNumeric::<CssInteger, CssPositive>::try_from_literal(positive(text)).unwrap(),
                None
            )
            .is_some()
        );
    }
    for text in ["2", "2147483648"] {
        let steps = CssSteps::try_new(
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(positive(text)).unwrap(),
            Some(CssStepPosition::JumpNone),
        )
        .unwrap();
        assert!(
            steps
                .count()
                .literal()
                .is_some_and(|value| value.numeric().representation() == text)
        );
    }
    let components = CssComponentValues::parse(&CssParser::default(), "calc(1)").unwrap();
    let calculation =
        CssCalculation::<CssInteger>::try_from_components(components.clone(), Default::default())
            .unwrap();
    let steps = CssSteps::try_new(
        CssNumeric::<CssInteger, CssPositive>::try_from_calculation(calculation).unwrap(),
        Some(CssStepPosition::JumpNone),
    )
    .unwrap();
    let Some(calculation) = steps.count().calculation() else {
        panic!("symbolic count");
    };
    assert_eq!(calculation.components(), &components);
    let omitted = CssSteps::try_new(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(positive("2")).unwrap(),
        None,
    )
    .unwrap();
    let explicit = CssSteps::try_new(
        CssNumeric::<CssInteger, CssPositive>::try_from_literal(positive("2")).unwrap(),
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
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
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
        CssNumeric::<CssInteger, CssPositive>::try_from_calculation(
            CssCalculation::<CssInteger>::try_from_components(components, Default::default())
                .unwrap(),
        )
        .unwrap(),
        Some(CssStepPosition::JumpNone),
    )
    .unwrap();
    let Some(literal) = steps.count().literal() else {
        panic!("ordinary bare integer root");
    };
    assert_eq!(literal.component(), &original);
    assert_eq!(literal.numeric().representation(), "+0002");
    parsed_origin(literal.origin(), source, "+0002");
    assert_eq!(steps.position(), Some(CssStepPosition::JumpNone));

    let bare = CssSteps::try_new(
        CssNumeric::<CssInteger, CssPositive>::try_from_calculation(
            CssCalculation::<CssInteger>::literal(2),
        )
        .unwrap(),
        None,
    )
    .unwrap();
    let Some(literal) = bare.count().literal() else {
        panic!("programmatic bare integer root");
    };
    assert_eq!(literal.component(), integer("2").component());
    assert_eq!(literal.origin(), &CssValueOrigin::Programmatic);

    for source in ["calc(0)", "calc(-1)", "calc(1)"] {
        let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
        let steps = CssSteps::try_new(
            CssNumeric::<CssInteger, CssPositive>::try_from_calculation(
                CssCalculation::<CssInteger>::try_from_components(
                    components.clone(),
                    Default::default(),
                )
                .unwrap(),
            )
            .unwrap(),
            Some(CssStepPosition::JumpNone),
        )
        .unwrap();
        let Some(calculation) = steps.count().calculation() else {
            panic!("genuine symbolic function");
        };
        assert_eq!(calculation.components(), &components);
        parsed_origin(calculation.origin(), source, "calc(");
    }
}

#[test]
fn parsed_steps_keep_count_origins_symbolic_math_and_position_omission() {
    let source = "transition-timing-function: steps(+0002, jump-none), steps(calc(1), jump-none), steps(2147483648), steps(2, end)";
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssKnownPropertyValueRef::TransitionTimingFunction(value)) =
        report.syntax()[0].known().unwrap().property_value()
    else {
        panic!("easing list");
    };
    let steps = value
        .value()
        .values()
        .iter()
        .map(|value| {
            let CssEasing::Steps(steps) = value else {
                panic!("steps");
            };
            steps
        })
        .collect::<Vec<_>>();
    let Some(first) = steps[0].count().literal() else {
        panic!("ordinary count");
    };
    assert_eq!(first.numeric().representation(), "+0002");
    parsed_origin(first.origin(), source, "+0002");
    let Some(calculation) = steps[1].count().calculation() else {
        panic!("math count");
    };
    parsed_origin(calculation.expression().origin(), source, "calc(");
    assert_eq!(
        calculation
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "calc(1)"
    );
    assert_eq!(steps[1].position(), Some(CssStepPosition::JumpNone));
    assert!(
        steps[2]
            .count()
            .literal()
            .is_some_and(|value| value.numeric().representation() == "2147483648")
    );
    assert_eq!(steps[2].position(), None);
    assert_eq!(steps[3].position(), Some(CssStepPosition::End));
}
