use surgeist_css::{
    CssAnimationDirection, CssAnimationFillMode, CssAnimationIterationNumber,
    CssAnimationIterationValue, CssAnimationIterationValueList, CssAnimationName,
    CssAnimationPlayState, CssCalculationExpressionRef, CssCalculationType, CssDelay, CssDelayList,
    CssDelayLiteral, CssDuration, CssDurationList, CssDurationLiteral, CssEasing, CssEasingValue,
    CssErrorCode, CssKnownProperty, CssKnownPropertyValueRef, CssRecoveryAction, CssSourcePosition,
    CssStepPosition, CssTimeUnit, CssTokenKind, CssTransitionProperty, ErrorKind,
    parse_style_attribute,
};

fn for_each_permutation(
    components: &mut [&'static str],
    index: usize,
    visit: &mut impl FnMut(&[&'static str]),
) {
    if index == components.len() {
        visit(components);
        return;
    }

    for swap_index in index..components.len() {
        components.swap(index, swap_index);
        for_each_permutation(components, index + 1, visit);
        components.swap(index, swap_index);
    }
}

fn assert_ascii_position(position: CssSourcePosition, byte_offset: usize) {
    assert_eq!(position.byte_offset().value(), byte_offset);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), byte_offset as u32);
}

fn assert_retained_color_sibling(
    source: &str,
    report: &surgeist_css::CssParseReport<surgeist_css::CssDeclarationList>,
) {
    let [declaration] = report.syntax().as_slice() else {
        panic!("{source}: valid color sibling must be retained exactly once");
    };
    assert_eq!(
        declaration
            .known()
            .expect("retained known color")
            .property(),
        CssKnownProperty::Color,
        "{source}",
    );
}

struct InvalidTimingCase {
    source: &'static str,
    property: CssKnownProperty,
    position: usize,
    span_end: usize,
    encountered: Option<(CssTokenKind, &'static str)>,
}

fn assert_invalid_timing_case(case: &InvalidTimingCase) {
    let report = parse_style_attribute(case.source);
    assert_retained_color_sibling(case.source, &report);
    let [diagnostic] = report.diagnostics() else {
        panic!("{}: expected one exact timing diagnostic", case.source);
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue,
        "{}",
        case.source,
    );
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{}",
        case.source,
    );
    assert_ascii_position(diagnostic.error().position(), case.position);
    assert_ascii_position(diagnostic.span().start(), 0);
    assert_ascii_position(diagnostic.span().end(), case.span_end);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("{}: expected typed property-value detail", case.source);
    };
    assert_eq!(detail.property(), case.property, "{}", case.source);
    assert_eq!(
        detail.expectation().as_str(),
        "a value accepted by the property's grammar",
        "{}",
        case.source,
    );
    match (detail.encountered(), case.encountered) {
        (Some(actual), Some((kind, authored))) => {
            assert_eq!(actual.kind(), kind, "{}", case.source);
            assert_eq!(actual.authored(), authored, "{}", case.source);
        }
        (None, None) => {}
        (actual, expected) => panic!(
            "{}: encountered token mismatch: actual={actual:?}, expected={expected:?}",
            case.source,
        ),
    }

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(case.source)
            .expect_err("strict validation must reject the recovered timing declaration");
        assert_eq!(
            failure.diagnostics(),
            report.diagnostics(),
            "{}",
            case.source
        );
    }
}

#[test]
fn duration_and_delay_literals_enforce_distinct_finite_sign_domains() {
    let duration = CssDurationLiteral::try_new(1.5, CssTimeUnit::Seconds)
        .expect("finite non-negative duration");
    assert_eq!(duration.value(), 1.5);
    assert_eq!(duration.unit(), CssTimeUnit::Seconds);
    assert!(CssDurationLiteral::try_new(-0.25, CssTimeUnit::Seconds).is_none());

    let delay =
        CssDelayLiteral::try_new(-250.0, CssTimeUnit::Milliseconds).expect("finite signed delay");
    assert_eq!(delay.value(), -250.0);
    assert_eq!(delay.unit(), CssTimeUnit::Milliseconds);

    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssDurationLiteral::try_new(value, CssTimeUnit::Seconds).is_none());
        assert!(CssDelayLiteral::try_new(value, CssTimeUnit::Seconds).is_none());
    }

    assert!(CssDurationList::try_new(Vec::new()).is_none());
    assert!(CssDelayList::try_new(Vec::new()).is_none());
    assert_eq!(
        CssDurationList::try_new(vec![CssDuration::Literal(duration)])
            .expect("non-empty duration list")
            .values()
            .len(),
        1,
    );
    assert_eq!(
        CssDelayList::try_new(vec![CssDelay::Literal(delay)])
            .expect("non-empty delay list")
            .values()
            .len(),
        1,
    );
}

#[test]
fn iteration_values_check_literals_and_keep_infinite_and_calculation_branches() {
    assert!(CssAnimationIterationNumber::try_new(-0.25).is_none());
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(CssAnimationIterationNumber::try_new(value).is_none());
    }
    let number = CssAnimationIterationNumber::try_new(2.5).expect("finite iteration number");
    assert_eq!(number.value(), 2.5);
    assert!(CssAnimationIterationValueList::try_new(Vec::new()).is_none());
    let values = CssAnimationIterationValueList::try_new(vec![
        CssAnimationIterationValue::Infinite,
        CssAnimationIterationValue::Number(number),
    ])
    .expect("non-empty iteration list");
    assert_eq!(values.values().len(), 2);
}

#[test]
fn timing_longhands_expose_exact_current_values_and_lossy_i01_boundaries() {
    let report = parse_style_attribute(concat!(
        "transition-duration: 1s, calc((2ms + 3ms) * 2); ",
        "transition-delay: -1s, 2ms; ",
        "animation-duration: 3ms, calc(1s + 2s); ",
        "animation-delay: calc(-1s + 2s), -4ms; ",
        "animation-iteration-count: infinite, 2.5, calc((1 + 2) * 3)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 5);

    let CssKnownPropertyValueRef::TransitionDuration(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transition-duration wrapper");
    };
    assert!(matches!(
        value.durations().values()[0],
        CssDuration::Literal(literal)
            if literal.value() == 1.0 && literal.unit() == CssTimeUnit::Seconds
    ));
    let CssDuration::Calculation(calculation) = &value.durations().values()[1] else {
        panic!("expected duration calculation");
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Time);
    assert!(matches!(
        calculation.expression(),
        CssCalculationExpressionRef::Product(_)
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::TransitionDelay(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transition-delay wrapper");
    };
    assert!(matches!(
        value.delays().values()[0],
        CssDelay::Literal(literal)
            if literal.value() == -1.0 && literal.unit() == CssTimeUnit::Seconds
    ));
    assert!(matches!(value.delays().values()[1], CssDelay::Literal(_)));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::AnimationDuration(value) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected animation-duration wrapper");
    };
    assert_eq!(value.durations().values().len(), 2);
    assert!(matches!(
        value.durations().values()[1],
        CssDuration::Calculation(_)
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::AnimationDelay(value) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected animation-delay wrapper");
    };
    assert!(matches!(
        value.delays().values()[0],
        CssDelay::Calculation(_)
    ));
    assert!(matches!(
        value.delays().values()[1],
        CssDelay::Literal(literal) if literal.value() == -4.0
    ));
    assert!(value.i01_subset().is_none());

    let CssKnownPropertyValueRef::AnimationIterationCount(value) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected animation-iteration-count wrapper");
    };
    let values = value.iteration_counts().values();
    assert!(matches!(values[0], CssAnimationIterationValue::Infinite));
    assert!(matches!(
        values[1],
        CssAnimationIterationValue::Number(number) if number.value() == 2.5
    ));
    assert!(matches!(
        values[2],
        CssAnimationIterationValue::Calculation(_)
    ));
    assert!(value.i01_subset().is_none());
}

#[test]
fn positive_i01_timing_inputs_keep_exact_compatibility_debug_observables() {
    let report = parse_style_attribute(concat!(
        "transition-duration: 1s, 2ms; transition-delay: 3s; ",
        "animation-duration: 4ms; animation-delay: 5s; ",
        "animation-iteration-count: infinite, 2; ",
        "transition: opacity 1s ease 2ms; ",
        "animation: fade 3s linear 2 4ms reverse both paused"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let values: Vec<String> = report
        .syntax()
        .iter()
        .map(|declaration| {
            let value = declaration.known().unwrap().property_value().unwrap();
            match value {
                CssKnownPropertyValueRef::TransitionDuration(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::TransitionDelay(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::AnimationDuration(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::AnimationDelay(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::AnimationIterationCount(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::Transition(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                CssKnownPropertyValueRef::Animation(value) => {
                    format!("{:?}", value.i01_subset().unwrap())
                }
                _ => panic!("unexpected timing wrapper"),
            }
        })
        .collect();
    assert_eq!(
        values,
        [
            "CssTimeList { times: [CssTime { value: 1.0, unit: Seconds }, CssTime { value: 2.0, unit: Milliseconds }] }",
            "CssTimeList { times: [CssTime { value: 3.0, unit: Seconds }] }",
            "CssTimeList { times: [CssTime { value: 4.0, unit: Milliseconds }] }",
            "CssTimeList { times: [CssTime { value: 5.0, unit: Seconds }] }",
            "CssAnimationIterationCountList { counts: [Infinite, Number(CssAnimationIterationNumber { value: 2.0 })] }",
            "CssTransitionList { items: [CssTransition { property: Some(Custom(CssCustomIdent { value: \"opacity\" })), duration: Some(CssTime { value: 1.0, unit: Seconds }), delay: Some(CssTime { value: 2.0, unit: Milliseconds }), timing_function: Some(Ease) }] }",
            "CssAnimationList { items: [CssAnimation { name: Some(Custom(CssCustomIdent { value: \"fade\" })), duration: Some(CssTime { value: 3.0, unit: Seconds }), delay: Some(CssTime { value: 4.0, unit: Milliseconds }), timing_function: Some(Linear), iteration_count: Some(Number(CssAnimationIterationNumber { value: 2.0 })), direction: Some(Reverse), fill_mode: Some(Both), play_state: Some(Paused) }] }",
        ]
    );
}

#[test]
fn transition_shorthand_assigns_first_time_to_duration_and_second_to_signed_delay() {
    let report = parse_style_attribute(concat!(
        "transition: opacity 1s ease -2ms, transform calc((3ms + 1ms) * 2) ",
        "calc(-1s + 2s) linear"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transition(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transition wrapper");
    };
    let transitions = value.transitions().values();
    assert_eq!(transitions.len(), 2);
    assert!(matches!(
        transitions[0].property(),
        Some(CssTransitionProperty::Custom(property)) if property.as_str() == "opacity"
    ));
    assert!(matches!(
        transitions[0].duration(),
        Some(CssDuration::Literal(_))
    ));
    assert!(matches!(
        transitions[0].delay(),
        Some(CssDelay::Literal(literal)) if literal.value() == -2.0
    ));
    assert!(matches!(
        transitions[0].timing_function(),
        Some(CssEasing::Ease)
    ));
    assert!(matches!(
        transitions[1].duration(),
        Some(CssDuration::Calculation(_))
    ));
    assert!(matches!(
        transitions[1].delay(),
        Some(CssDelay::Calculation(_))
    ));
    assert!(value.i01_subset().is_none());
}

#[test]
fn animation_shorthand_exposes_all_eight_current_components() {
    let report = parse_style_attribute(
        "animation: fade calc((1s + 2s) * 2) ease calc(-1s + 2s) calc((1 + 2) * 3) reverse both paused",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Animation(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected animation wrapper");
    };
    let animation = &value.animations().values()[0];
    assert!(
        matches!(animation.name(), Some(CssAnimationName::Custom(name)) if name.as_str() == "fade")
    );
    assert!(matches!(
        animation.duration(),
        Some(CssDuration::Calculation(_))
    ));
    assert!(matches!(animation.delay(), Some(CssDelay::Calculation(_))));
    assert!(matches!(animation.timing_function(), Some(CssEasing::Ease)));
    assert!(matches!(
        animation.iteration_count(),
        Some(CssAnimationIterationValue::Calculation(_))
    ));
    assert_eq!(animation.direction(), Some(CssAnimationDirection::Reverse));
    assert_eq!(animation.fill_mode(), Some(CssAnimationFillMode::Both));
    assert_eq!(animation.play_state(), Some(CssAnimationPlayState::Paused));
    assert!(value.i01_subset().is_none());
}

#[test]
fn timing_shorthands_propagate_typed_cubic_and_step_values() {
    let report = parse_style_attribute(concat!(
        "transition: opacity 1s cubic-bezier(0.1, -2, 0.9, 3), ",
        "transform 2s steps(2, jump-none); ",
        "animation: fade 1s steps(3, jump-both)"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::Transition(transition) = report.syntax()[0]
        .known()
        .expect("known transition")
        .property_value()
        .expect("ordinary transition")
    else {
        panic!("expected transition wrapper");
    };
    assert!(matches!(
        transition.transitions().values()[0].current_timing_function(),
        Some(CssEasingValue::CubicBezier(_))
    ));
    assert!(matches!(
        transition.transitions().values()[1].current_timing_function(),
        Some(CssEasingValue::Steps(steps))
            if steps.count().literal() == Some(2)
                && steps.position() == Some(CssStepPosition::JumpNone)
    ));

    let CssKnownPropertyValueRef::Animation(animation) = report.syntax()[1]
        .known()
        .expect("known animation")
        .property_value()
        .expect("ordinary animation")
    else {
        panic!("expected animation wrapper");
    };
    assert!(matches!(
        animation.animations().values()[0].current_timing_function(),
        Some(CssEasingValue::Steps(steps))
            if steps.count().literal() == Some(3)
                && steps.position() == Some(CssStepPosition::JumpBoth)
    ));

    assert!(transition.i01_subset().is_some());
    assert!(animation.i01_subset().is_some());
}

#[test]
fn every_transition_component_order_preserves_first_time_and_second_time_domains() {
    let mut components = ["opacity", "1s", "-2ms", "ease-in"];
    for_each_permutation(&mut components, 0, &mut |order| {
        let duration_index = order
            .iter()
            .position(|component| *component == "1s")
            .unwrap();
        let delay_index = order
            .iter()
            .position(|component| *component == "-2ms")
            .unwrap();
        if duration_index > delay_index {
            return;
        }

        let source = format!("transition: {}", order.join(" "));
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("{source}: expected one transition declaration");
        };
        let CssKnownPropertyValueRef::Transition(value) = declaration
            .known()
            .expect("known transition")
            .property_value()
            .expect("ordinary transition")
        else {
            panic!("{source}: expected transition wrapper");
        };
        let [transition] = value.transitions().values() else {
            panic!("{source}: expected one transition item");
        };
        assert!(
            matches!(transition.property(), Some(CssTransitionProperty::Custom(property)) if property.as_str() == "opacity"),
            "{source}",
        );
        assert!(
            matches!(transition.duration(), Some(CssDuration::Literal(literal)) if literal.value() == 1.0 && literal.unit() == CssTimeUnit::Seconds),
            "{source}",
        );
        assert!(
            matches!(transition.delay(), Some(CssDelay::Literal(literal)) if literal.value() == -2.0 && literal.unit() == CssTimeUnit::Milliseconds),
            "{source}",
        );
        assert!(
            matches!(transition.timing_function(), Some(CssEasing::EaseIn)),
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect("strict validation accepts every valid transition order"),
            *report.syntax(),
            "{source}",
        );
    });
}

#[test]
fn every_animation_component_order_preserves_all_eight_typed_domains() {
    let mut components = [
        "fade", "1s", "-2ms", "ease-in", "2", "reverse", "both", "paused",
    ];
    for_each_permutation(&mut components, 0, &mut |order| {
        let duration_index = order
            .iter()
            .position(|component| *component == "1s")
            .unwrap();
        let delay_index = order
            .iter()
            .position(|component| *component == "-2ms")
            .unwrap();
        if duration_index > delay_index {
            return;
        }

        let source = format!("animation: {}", order.join(" "));
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [declaration] = report.syntax().as_slice() else {
            panic!("{source}: expected one animation declaration");
        };
        let CssKnownPropertyValueRef::Animation(value) = declaration
            .known()
            .expect("known animation")
            .property_value()
            .expect("ordinary animation")
        else {
            panic!("{source}: expected animation wrapper");
        };
        let [animation] = value.animations().values() else {
            panic!("{source}: expected one animation item");
        };
        assert!(
            matches!(animation.name(), Some(CssAnimationName::Custom(name)) if name.as_str() == "fade"),
            "{source}",
        );
        assert!(
            matches!(animation.duration(), Some(CssDuration::Literal(literal)) if literal.value() == 1.0 && literal.unit() == CssTimeUnit::Seconds),
            "{source}",
        );
        assert!(
            matches!(animation.delay(), Some(CssDelay::Literal(literal)) if literal.value() == -2.0 && literal.unit() == CssTimeUnit::Milliseconds),
            "{source}",
        );
        assert!(
            matches!(animation.timing_function(), Some(CssEasing::EaseIn)),
            "{source}",
        );
        assert!(
            matches!(animation.iteration_count(), Some(CssAnimationIterationValue::Number(number)) if number.value() == 2.0),
            "{source}",
        );
        assert_eq!(
            animation.direction(),
            Some(CssAnimationDirection::Reverse),
            "{source}",
        );
        assert_eq!(
            animation.fill_mode(),
            Some(CssAnimationFillMode::Both),
            "{source}",
        );
        assert_eq!(
            animation.play_state(),
            Some(CssAnimationPlayState::Paused),
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect("strict validation accepts every valid animation order"),
            *report.syntax(),
            "{source}",
        );
    });
}

#[test]
fn every_invalid_timing_category_has_exact_public_diagnostics_and_strict_parity() {
    for case in [
        InvalidTimingCase {
            source: "transition: opacity -1s 2s; color: red",
            property: CssKnownProperty::Transition,
            position: 20,
            span_end: 27,
            encountered: Some((CssTokenKind::Dimension, "-1s")),
        },
        InvalidTimingCase {
            source: "animation: fade -1s 2s; color: red",
            property: CssKnownProperty::Animation,
            position: 16,
            span_end: 23,
            encountered: Some((CssTokenKind::Dimension, "-1s")),
        },
        InvalidTimingCase {
            source: "transition: opacity 1s ease linear; color: red",
            property: CssKnownProperty::Transition,
            position: 28,
            span_end: 35,
            encountered: Some((CssTokenKind::Ident, "linear")),
        },
        InvalidTimingCase {
            source: "animation: fade 1s reverse alternate; color: red",
            property: CssKnownProperty::Animation,
            position: 27,
            span_end: 37,
            encountered: Some((CssTokenKind::Ident, "alternate")),
        },
        InvalidTimingCase {
            source: "transition: opacity 1s -2s 3s; color: red",
            property: CssKnownProperty::Transition,
            position: 27,
            span_end: 30,
            encountered: Some((CssTokenKind::Dimension, "3s")),
        },
        InvalidTimingCase {
            source: "animation: fade 1s -2s 3s; color: red",
            property: CssKnownProperty::Animation,
            position: 23,
            span_end: 26,
            encountered: Some((CssTokenKind::Dimension, "3s")),
        },
        InvalidTimingCase {
            source: "transition-duration: ; color: red",
            property: CssKnownProperty::TransitionDuration,
            position: 21,
            span_end: 22,
            encountered: None,
        },
        InvalidTimingCase {
            source: "animation-delay: ; color: red",
            property: CssKnownProperty::AnimationDelay,
            position: 17,
            span_end: 18,
            encountered: None,
        },
        InvalidTimingCase {
            source: "transition-duration: 1e999s; color: red",
            property: CssKnownProperty::TransitionDuration,
            position: 21,
            span_end: 28,
            encountered: Some((CssTokenKind::Dimension, "1e999s")),
        },
        InvalidTimingCase {
            source: "transition-delay: -1e999s; color: red",
            property: CssKnownProperty::TransitionDelay,
            position: 18,
            span_end: 26,
            encountered: Some((CssTokenKind::Dimension, "-1e999s")),
        },
        InvalidTimingCase {
            source: "animation-duration: calc(1e999s); color: red",
            property: CssKnownProperty::AnimationDuration,
            position: 25,
            span_end: 33,
            encountered: Some((CssTokenKind::Dimension, "1e999s")),
        },
        InvalidTimingCase {
            source: "animation-delay: calc(-1e999s); color: red",
            property: CssKnownProperty::AnimationDelay,
            position: 22,
            span_end: 31,
            encountered: Some((CssTokenKind::Dimension, "-1e999s")),
        },
        InvalidTimingCase {
            source: "animation-iteration-count: 1e999; color: red",
            property: CssKnownProperty::AnimationIterationCount,
            position: 27,
            span_end: 33,
            encountered: Some((CssTokenKind::Number, "1e999")),
        },
        InvalidTimingCase {
            source: "animation-iteration-count: calc(1e999); color: red",
            property: CssKnownProperty::AnimationIterationCount,
            position: 32,
            span_end: 39,
            encountered: Some((CssTokenKind::Number, "1e999")),
        },
        InvalidTimingCase {
            source: "transition-duration: calc(1px + 2px); color: red",
            property: CssKnownProperty::TransitionDuration,
            position: 26,
            span_end: 37,
            encountered: Some((CssTokenKind::Dimension, "1px")),
        },
        InvalidTimingCase {
            source: "animation-iteration-count: calc(1s + 2s); color: red",
            property: CssKnownProperty::AnimationIterationCount,
            position: 32,
            span_end: 41,
            encountered: Some((CssTokenKind::Dimension, "1s")),
        },
        InvalidTimingCase {
            source: "transition-duration: 1s,; color: red",
            property: CssKnownProperty::TransitionDuration,
            position: 24,
            span_end: 25,
            encountered: None,
        },
        InvalidTimingCase {
            source: "animation-delay: -1s,,2s; color: red",
            property: CssKnownProperty::AnimationDelay,
            position: 21,
            span_end: 25,
            encountered: Some((CssTokenKind::Comma, ",")),
        },
        InvalidTimingCase {
            source: "transition-delay: 1s trailing; color: red",
            property: CssKnownProperty::TransitionDelay,
            position: 21,
            span_end: 30,
            encountered: Some((CssTokenKind::Ident, "trailing")),
        },
        InvalidTimingCase {
            source: "animation-duration: 1s trailing; color: red",
            property: CssKnownProperty::AnimationDuration,
            position: 23,
            span_end: 32,
            encountered: Some((CssTokenKind::Ident, "trailing")),
        },
    ] {
        assert_invalid_timing_case(&case);
    }
}

#[test]
fn repeated_timing_failures_and_depth_255_256_257_have_exact_recovery_behavior() {
    let mut source = String::new();
    for _ in 0..128 {
        source.push_str("transition: -1s 2s;");
    }
    source.push_str("color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(report.diagnostics().len(), 128);

    for depth in [255_usize, 256] {
        let source = format!(
            "transition-duration: {}1s{}; color: red",
            "calc(".repeat(depth),
            ")".repeat(depth)
        );
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let [duration, color] = report.syntax().as_slice() else {
            panic!("depth {depth}: duration and color must both be retained");
        };
        let CssKnownPropertyValueRef::TransitionDuration(duration) = duration
            .known()
            .expect("known duration")
            .property_value()
            .expect("ordinary duration")
        else {
            panic!("depth {depth}: expected transition-duration wrapper");
        };
        assert!(matches!(
            duration.durations().values(),
            [CssDuration::Calculation(calculation)]
                if calculation.result_type() == CssCalculationType::Time
        ));
        assert_eq!(
            color.known().expect("known color sibling").property(),
            CssKnownProperty::Color,
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect("strict validation accepts supported calculation depth"),
            *report.syntax(),
            "depth {depth}",
        );
    }

    let depth = 257_usize;
    let source = format!(
        "transition-duration: {}1s{}; color: red",
        "calc(".repeat(depth),
        ")".repeat(depth)
    );
    let report = parse_style_attribute(&source);
    assert_retained_color_sibling(&source, &report);
    let [diagnostic] = report.diagnostics() else {
        panic!("over-limit timing calculation must produce one diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_ascii_position(diagnostic.error().position(), 1_301);
    assert_ascii_position(diagnostic.span().start(), 0);
    assert_ascii_position(diagnostic.span().end(), 1_566);
    let ErrorKind::NestingLimit(detail) = diagnostic.error().kind() else {
        panic!("depth 257 must expose typed nesting-limit detail");
    };
    assert_eq!(detail.limit(), 256);
    assert_eq!(detail.enclosing_production().as_str(), "css.declaration");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(&source)
            .expect_err("strict validation rejects depth 257");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}
