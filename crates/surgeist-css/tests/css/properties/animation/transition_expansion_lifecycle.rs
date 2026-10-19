#![forbid(unsafe_code)]
//! Transitions 1 WD 2026-01-08 §§2.1–2.5 supplies noninherited initials,
//! list grammar, first-time duration admission, and shorthand omission defaults.
//! Cascade 5 CR 2022-01-13 §3.1 supplies shorthand propagation semantics.
//! Exact raw time-child provenance and contribution order are Surgeist contracts.
//! These functional tests exercise the four new borrowed longhand variants.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const MEMBERS: [P; 4] = [
    P::TransitionProperty,
    P::TransitionDuration,
    P::TransitionTimingFunction,
    P::TransitionDelay,
];

fn parsed(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}

fn checked(property: P, css: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        property.grammar(),
        parse_component_values(css).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed ordinary longhands")
    };
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        assert_eq!(
            item.source().known().unwrap().grammar(),
            source.known().unwrap().grammar()
        );
        assert_eq!(item.source().value_components(), source.value_components());
    }
    values
}

fn lists(
    values: &CssLonghandContributions,
) -> (
    &CssTransitionPropertyList,
    &CssDurationList,
    &CssEasingList,
    &CssDelayList,
) {
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        MEMBERS
    );
    let [property, duration, easing, delay] = values.items() else {
        panic!("four ordered terminals")
    };
    let CssLonghandValueRef::TransitionProperty(property) =
        property.ordinary_value().unwrap().view()
    else {
        panic!("property list")
    };
    let CssLonghandValueRef::TransitionDuration(duration) =
        duration.ordinary_value().unwrap().view()
    else {
        panic!("duration list")
    };
    let CssLonghandValueRef::TransitionTimingFunction(easing) =
        easing.ordinary_value().unwrap().view()
    else {
        panic!("easing list")
    };
    let CssLonghandValueRef::TransitionDelay(delay) = delay.ordinary_value().unwrap().view() else {
        panic!("delay list")
    };
    (property, duration, easing, delay)
}

fn assert_literal(time: &CssTimeValue, coefficient: &str, unit: CssTimeUnit) {
    let literal = time.literal().unwrap();
    assert_eq!(literal.numeric().representation(), coefficient);
    assert_eq!(literal.unit(), unit);
}

fn assert_time_clone(actual: &CssTimeValue, expected: &CssTimeValue) {
    // Raw time equality includes component spans and delimiter origins, unlike
    // duration/list structural equality. Also check parsed snapshot identity.
    assert_eq!(actual, expected);
    assert_eq!(actual.origin(), expected.origin());
    if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
        (actual.origin(), expected.origin())
    {
        assert!(actual.source().same_snapshot(expected.source()));
    }
    if let Some(calculation) = actual.calculation() {
        assert_eq!(
            calculation.components(),
            expected.calculation().unwrap().components()
        );
    }
}

#[test]
fn all_four_initial_variants_contain_one_intrinsic_entry() {
    for property in MEMBERS {
        let metadata = property.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("longhand")
        };
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("ordinary initial")
        };
        assert_eq!(value.property().known_property(), property);
        match value.view() {
            CssLonghandValueRef::TransitionProperty(list) => {
                assert_eq!(list.properties(), &[CssTransitionProperty::All])
            }
            CssLonghandValueRef::TransitionDuration(list) => {
                assert_eq!(list.values().len(), 1);
                assert_literal(list.values()[0].time(), "0", CssTimeUnit::Seconds);
                assert_eq!(list.values()[0].origin(), &CssValueOrigin::Programmatic);
            }
            CssLonghandValueRef::TransitionTimingFunction(list) => {
                assert_eq!(list.values(), &[CssEasing::Keyword(CssEasingKeyword::Ease)])
            }
            CssLonghandValueRef::TransitionDelay(list) => {
                assert_eq!(list.values().len(), 1);
                assert_literal(&list.values()[0], "0", CssTimeUnit::Seconds);
                assert_eq!(list.values()[0].origin(), &CssValueOrigin::Programmatic);
            }
            other => panic!("transition initial: {other:?}"),
        }
    }
}

fn assert_twenty(source: &CssDeclaration, property: P) {
    let values = completed(source);
    let known = source.known().unwrap().property_value().unwrap();
    if property != P::Transition {
        assert_eq!(values.items().len(), 1);
    }
    match known {
        CssKnownPropertyValueRef::TransitionProperty(original) => {
            let CssLonghandValueRef::TransitionProperty(actual) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("property")
            };
            assert_eq!(actual.properties().len(), 20);
            assert_eq!(
                CssTransitionPropertyList::try_new(original.properties().properties().to_vec())
                    .as_ref(),
                Some(actual)
            );
            for (index, property) in actual.properties().iter().enumerate() {
                let CssTransitionProperty::Custom(name) = property else {
                    panic!("custom property name")
                };
                assert_eq!(name.as_str(), format!("unknown{index}"));
            }
        }
        CssKnownPropertyValueRef::TransitionDuration(original) => {
            let CssLonghandValueRef::TransitionDuration(actual) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("duration")
            };
            assert_eq!(actual.values().len(), 20);
            assert!(CssDurationList::try_new(actual.values().to_vec()).is_some());
            for (index, (actual, original)) in actual
                .values()
                .iter()
                .zip(original.durations().values())
                .enumerate()
            {
                assert_literal(
                    actual.time(),
                    &(index + 1).to_string(),
                    CssTimeUnit::Milliseconds,
                );
                assert_time_clone(actual.time(), original.time());
            }
        }
        CssKnownPropertyValueRef::TransitionDelay(original) => {
            let CssLonghandValueRef::TransitionDelay(actual) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("delay")
            };
            assert_eq!(actual.values().len(), 20);
            assert!(CssDelayList::try_new(actual.values().to_vec()).is_some());
            for (index, (actual, original)) in actual
                .values()
                .iter()
                .zip(original.delays().values())
                .enumerate()
            {
                assert_literal(
                    actual,
                    &format!("-{}", index + 1),
                    CssTimeUnit::Milliseconds,
                );
                assert_time_clone(actual, original);
            }
        }
        CssKnownPropertyValueRef::TransitionTimingFunction(original) => {
            let CssLonghandValueRef::TransitionTimingFunction(actual) =
                values.items()[0].ordinary_value().unwrap().view()
            else {
                panic!("easing")
            };
            assert_eq!(actual.values().len(), 20);
            assert_eq!(
                CssEasingList::try_new(original.timing_functions().values().to_vec()).as_ref(),
                Some(actual)
            );
            for (index, actual) in actual.values().iter().enumerate() {
                assert_eq!(
                    *actual,
                    CssEasing::Keyword(if index % 2 == 0 {
                        CssEasingKeyword::EaseIn
                    } else {
                        CssEasingKeyword::Linear
                    })
                );
            }
        }
        CssKnownPropertyValueRef::Transition(original) => {
            let (properties, durations, easings, delays) = lists(&values);
            assert_eq!(original.transitions().values().len(), 20);
            assert!(CssTransitionList::try_new(original.transitions().values().to_vec()).is_some());
            assert_eq!(properties.properties().len(), 20);
            assert_eq!(durations.values().len(), 20);
            assert_eq!(easings.values().len(), 20);
            assert_eq!(delays.values().len(), 20);
            for (index, original) in original.transitions().values().iter().enumerate() {
                let CssTransitionProperty::Custom(name) = &properties.properties()[index] else {
                    panic!("custom property name")
                };
                assert_eq!(name.as_str(), format!("unknown{index}"));
                assert_eq!(Some(&properties.properties()[index]), original.property());
                assert_literal(
                    durations.values()[index].time(),
                    &(index + 1).to_string(),
                    CssTimeUnit::Milliseconds,
                );
                assert_time_clone(
                    durations.values()[index].time(),
                    original.duration().unwrap().time(),
                );
                assert_literal(
                    &delays.values()[index],
                    &format!("-{}", index + 1),
                    CssTimeUnit::Milliseconds,
                );
                assert_time_clone(&delays.values()[index], original.delay().unwrap());
                assert_eq!(
                    easings.values()[index],
                    CssEasing::Keyword(if index % 2 == 0 {
                        CssEasingKeyword::EaseIn
                    } else {
                        CssEasingKeyword::Linear
                    })
                );
                assert_eq!(Some(&easings.values()[index]), original.timing_function());
            }
        }
        other => panic!("twenty transition entries: {other:?}"),
    }
}

#[test]
fn all_five_properties_preserve_twenty_parsed_and_checked_entries() {
    for property in MEMBERS.into_iter().chain([P::Transition]) {
        let css = (0..20)
            .map(|index| {
                let easing = if index % 2 == 0 { "ease-in" } else { "linear" };
                match property {
                    P::TransitionProperty => format!("unknown{index}"),
                    P::TransitionDuration => format!("{}ms", index + 1),
                    P::TransitionTimingFunction => easing.to_owned(),
                    P::TransitionDelay => format!("-{}ms", index + 1),
                    P::Transition => {
                        format!("unknown{index} {}ms {easing} -{}ms", index + 1, index + 1)
                    }
                    _ => unreachable!(),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let authored = parsed(&format!(
            "/*😀*/{}:{css}!important",
            property.canonical_name()
        ));
        let by_grammar = checked(property, &css);
        let by_property = parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(&css).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(authored.position().is_some());
        for source in [&by_grammar, &by_property] {
            assert!(source.position().is_none());
        }
        for source in [&authored, &by_grammar, &by_property] {
            assert_twenty(source, property);
        }
    }
}

#[test]
fn sparse_items_clone_explicit_time_children_and_supply_scalar_defaults() {
    let source = parsed("transition:opacity 125ms, transform ease-in, 2s linear -250ms!important");
    let CssKnownPropertyValueRef::Transition(original) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    let values = completed(&source);
    let (properties, durations, easings, delays) = lists(&values);
    assert_eq!(properties.properties().len(), 3);
    assert_eq!(durations.values().len(), 3);
    assert_eq!(easings.values().len(), 3);
    assert_eq!(delays.values().len(), 3);
    assert_eq!(properties.properties()[2], CssTransitionProperty::All);
    assert_time_clone(
        durations.values()[0].time(),
        original.transitions().values()[0]
            .duration()
            .unwrap()
            .time(),
    );
    assert_literal(durations.values()[1].time(), "0", CssTimeUnit::Seconds);
    assert_eq!(
        durations.values()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_time_clone(
        durations.values()[2].time(),
        original.transitions().values()[2]
            .duration()
            .unwrap()
            .time(),
    );
    assert_eq!(
        easings.values(),
        &[
            CssEasing::Keyword(CssEasingKeyword::Ease),
            CssEasing::Keyword(CssEasingKeyword::EaseIn),
            CssEasing::Keyword(CssEasingKeyword::Linear)
        ]
    );
    for delay in &delays.values()[..2] {
        assert_literal(delay, "0", CssTimeUnit::Seconds);
        assert_eq!(delay.origin(), &CssValueOrigin::Programmatic);
    }
    assert_time_clone(
        &delays.values()[2],
        original.transitions().values()[2].delay().unwrap(),
    );
}

#[test]
fn checked_mixed_origin_children_survive_projection_and_reentry() {
    let parsed = parse_component_values("opacity 1s ease-in -250ms").unwrap();
    let programmatic = CssComponentValue::try_dimension("125", "ms").unwrap();
    let mut items = parsed.items().to_vec();
    let duration_index = items
        .iter()
        .position(|item| {
            matches!(
                item.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
            )
        })
        .unwrap();
    items[duration_index] = programmatic.clone();
    let components = CssComponentValues::try_new(items).unwrap();
    let source = parse_property_value_for_grammar(
        P::Transition.grammar(),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let values = completed(&source);
    let (_, durations, _, delays) = lists(&values);
    assert_eq!(
        durations.values()[0].time().literal().unwrap().component(),
        &programmatic
    );
    assert_eq!(
        durations.values()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    assert!(matches!(
        delays.values()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let CssKnownPropertyValueRef::Transition(original) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    assert_time_clone(
        &delays.values()[0],
        original.transitions().values()[0].delay().unwrap(),
    );
    let pending = checked(P::Transition, "var(--timing)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending")
    };
    let CssContributions::Longhands(reentered) = handle.reenter(components.clone()).unwrap() else {
        panic!("replacement")
    };
    let (_, reentered_durations, _, reentered_delays) = lists(&reentered);
    assert_time_clone(
        reentered_durations.values()[0].time(),
        durations.values()[0].time(),
    );
    assert_time_clone(&reentered_delays.values()[0], &delays.values()[0]);
    for item in reentered.items() {
        assert!(item.source().same_occurrence(&pending));
        assert_eq!(
            item.source().known().unwrap().grammar(),
            P::Transition.grammar()
        );
        assert_eq!(item.replacement_components(), Some(&components));
    }
}

#[test]
fn late_recovered_time_children_keep_their_implicit_graph_and_source_snapshot() {
    for (css, duration_recovered) in [
        ("transition:opacity 1s, transform calc((2s", true),
        ("transition:opacity 1s, transform 2s calc((-250ms", false),
    ] {
        let report = parse_style_attribute(css);
        assert!(!report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("one retained shorthand")
        };
        let CssKnownPropertyValueRef::Transition(original) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("retained transition")
        };
        assert_eq!(original.transitions().values().len(), 2);
        assert!(CssTransitionList::try_new(original.transitions().values().to_vec()).is_none());
        let values = completed(source);
        let (_, durations, _, delays) = lists(&values);
        assert_eq!(durations.values().len(), 2);
        assert_eq!(delays.values().len(), 2);
        assert_time_clone(
            durations.values()[0].time(),
            original.transitions().values()[0]
                .duration()
                .unwrap()
                .time(),
        );
        let (actual, expected) = if duration_recovered {
            assert!(CssDurationList::try_new(durations.values().to_vec()).is_none());
            (
                durations.values()[1].time(),
                original.transitions().values()[1]
                    .duration()
                    .unwrap()
                    .time(),
            )
        } else {
            assert!(CssDelayList::try_new(delays.values().to_vec()).is_none());
            (
                &delays.values()[1],
                original.transitions().values()[1].delay().unwrap(),
            )
        };
        assert_time_clone(actual, expected);
        let components = actual.calculation().unwrap().components();
        let CssComponentValueRef::Function(function) = components.items()[0].view() else {
            panic!("calc")
        };
        assert!(matches!(
            function.closing_origin(),
            CssValueOrigin::ImplicitClosure { .. }
        ));
        assert!(validate_style_attribute(css).is_err());
    }
}

#[test]
fn each_property_rejects_intrinsic_negatives_through_all_admission_front_doors() {
    for (property, invalid) in [
        (
            P::TransitionProperty,
            vec![
                "",
                ",opacity",
                "opacity,",
                "opacity,,transform",
                "none, opacity",
                "opacity none",
                "1s",
                "initial, opacity",
                "\"opacity\"",
            ],
        ),
        (
            P::TransitionDuration,
            vec![
                "",
                "-1s",
                "0",
                "1px",
                "1s,",
                "1s,,2s",
                "inherit, 1s",
                "calc(1px)",
                "1s 2s",
            ],
        ),
        (
            P::TransitionDelay,
            vec![
                "",
                "0",
                "1px",
                "1s,",
                "1s,,2s",
                "unset, 1s",
                "calc(1px)",
                "1s 2s",
            ],
        ),
        (
            P::TransitionTimingFunction,
            vec![
                "",
                "1s",
                "ease,",
                "ease,,linear",
                "ease ease-in",
                "cubic-bezier(-1,0,1,1)",
                "steps(1,jump-none)",
                "revert, ease",
            ],
        ),
        (
            P::Transition,
            vec![
                "",
                "opacity -1s",
                "opacity 1s 2s 3s",
                "opacity transform 1s",
                "opacity 1s ease linear",
                "opacity 1s,",
                "opacity 1s,,transform 2s",
                "none 1s, opacity 2s",
                "inherit, opacity 1s",
                "opacity calc(1px)",
            ],
        ),
    ] {
        for invalid in invalid {
            let css = format!("{}:{invalid}", property.canonical_name());
            let report = parse_style_attribute(&css);
            assert!(!report.is_clean(), "{css}");
            assert!(
                report.syntax().is_empty(),
                "invalid declaration dropped: {css}"
            );
            assert_eq!(
                validate_style_attribute(&css).unwrap_err().diagnostics(),
                report.diagnostics(),
                "{css}"
            );
            let components = parse_component_values(invalid).unwrap();
            let by_grammar = parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            let by_property = parse_property_value(
                CssPropertyNameRef::Known(property),
                components,
                CssImportance::Normal,
            )
            .unwrap_err();
            assert_eq!(by_grammar, by_property, "same original grammar: {css}");
        }
    }
}

#[test]
fn projected_time_lists_keep_cumulative_specified_limits_and_atomic_failure() {
    let source = checked(P::Transition, "opacity 1ms -1ms, transform 2ms -2ms");
    let values = completed(&source);
    let (properties, durations, _, delays) = lists(&values);
    // Each list contributes one node, each ordinary time/name one node.
    // Punctuation adds bytes only. Exact time conversion does not change counts.
    let limits = CssSpecifiedValueSerializationLimits::new;
    assert_eq!(
        properties
            .serialize_specified_with_limits(limits(3, 3, 18))
            .unwrap(),
        "opacity, transform"
    );
    assert_eq!(
        durations
            .serialize_specified_with_limits(limits(3, 3, 14))
            .unwrap(),
        "0.001s, 0.002s"
    );
    assert_eq!(
        delays
            .serialize_specified_with_limits(limits(3, 3, 16))
            .unwrap(),
        "-0.001s, -0.002s"
    );
    for (input, projection, byte_deficit, kind) in [
        (
            2,
            3,
            0,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            3,
            2,
            0,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (3, 3, 1, CssSpecifiedValueSerializationErrorKind::ByteLimit),
    ] {
        assert_eq!(
            properties
                .serialize_specified_with_limits(limits(input, projection, 18 - byte_deficit))
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            durations
                .serialize_specified_with_limits(limits(input, projection, 14 - byte_deficit))
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            delays
                .serialize_specified_with_limits(limits(input, projection, 16 - byte_deficit))
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(durations.serialize_specified().unwrap(), "0.001s, 0.002s");
    assert_eq!(delays.serialize_specified().unwrap(), "-0.001s, -0.002s");
    let CssKnownPropertyValueRef::Transition(original) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    for (index, original) in original.transitions().values().iter().enumerate() {
        assert_time_clone(
            durations.values()[index].time(),
            original.duration().unwrap().time(),
        );
        assert_time_clone(&delays.values()[index], original.delay().unwrap());
    }
}

#[test]
fn projected_math_retains_raw_graph_and_uses_existing_numeric_provider_budgets() {
    let source = checked(P::Transition, "opacity calc(1s + 2s) -250ms");
    let values = completed(&source);
    let (_, durations, _, delays) = lists(&values);
    let raw = durations.values()[0]
        .time()
        .calculation()
        .unwrap()
        .components()
        .clone();
    // Duration list + Calc + Sum + two leaves: five inputs, four projections.
    // The delay list + its ordinary dimension: two inputs and two projections.
    let limits = CssSpecifiedValueSerializationLimits::new;
    assert_eq!(
        durations
            .serialize_specified_with_limits(limits(5, 4, 8))
            .unwrap(),
        "calc(3s)"
    );
    assert_eq!(
        delays
            .serialize_specified_with_limits(limits(2, 2, 6))
            .unwrap(),
        "-0.25s"
    );
    for (budget, kind) in [
        (
            limits(4, 4, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            limits(5, 3, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            limits(5, 4, 7),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            durations
                .serialize_specified_with_limits(budget)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            durations.values()[0]
                .time()
                .calculation()
                .unwrap()
                .components(),
            &raw
        );
    }
    assert_eq!(durations.serialize_specified().unwrap(), "calc(3s)");
    let CssKnownPropertyValueRef::Transition(original) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    assert_time_clone(
        durations.values()[0].time(),
        original.transitions().values()[0]
            .duration()
            .unwrap()
            .time(),
    );
}

#[test]
fn pending_names_and_easing_reject_original_closures_and_remain_retryable() {
    for (property, recovered, valid) in [
        (P::TransitionProperty, "opacity/*", "opacity, transform"),
        (P::TransitionProperty, "initial/*", "unset"),
        (
            P::TransitionTimingFunction,
            "steps(2,end",
            "steps(2,end), ease",
        ),
        (
            P::TransitionTimingFunction,
            "cubic-bezier(0,0,1,1",
            "cubic-bezier(0,0,1,1)",
        ),
        (P::TransitionTimingFunction, "ease/*", "ease/**/"),
        (P::TransitionTimingFunction, "initial/*", "unset"),
    ] {
        let source = parsed(&format!(
            "{}:var(--replacement)!important",
            property.canonical_name()
        ));
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending family member")
        };
        let replacement = parse_component_values(recovered).unwrap();
        let before = replacement.clone();
        let error = handle.reenter(replacement.clone()).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("original grammar rejection")
        };
        assert!(matches!(
            actual.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ));
        let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { at, .. })) =
            actual.origin()
        else {
            panic!("original implicit EOF")
        };
        assert_eq!(at.source().as_str(), recovered);
        assert_eq!(at.span().start().byte_offset().value(), recovered.len());
        assert_eq!(
            actual,
            &parse_property_value_for_grammar(
                property.grammar(),
                replacement.clone(),
                CssImportance::Normal
            )
            .unwrap_err()
        );
        assert_eq!(replacement, before);
        assert!(handle.source().same_occurrence(&source));
        let replacement = parse_component_values(valid).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry completes")
        };
        assert_eq!(values.items().len(), 1);
        let item = &values.items()[0];
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        assert!(handle.source().same_occurrence(&source));
    }
}
