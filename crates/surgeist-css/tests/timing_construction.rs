use surgeist_css::{
    CssAnimation, CssAnimationComponents, CssAnimationDirection, CssAnimationFillMode,
    CssAnimationIterationCount, CssAnimationList, CssAnimationName, CssAnimationPlayState,
    CssCubicBezier, CssDuration, CssEasing, CssEasingKeyword, CssIntegerLiteral,
    CssKnownPropertyValueRef, CssPositiveIntegerLiteral, CssPositiveIntegerValue, CssStepPosition,
    CssSteps, CssTimeUnit, CssTransition, CssTransitionList, CssTransitionProperty,
    parse_style_attribute,
};
use surgeist_css::{CssTimeLiteral, CssTimeValue};

#[test]
fn checked_aggregates_reject_empty_items_and_lists() {
    assert!(CssTransition::try_new(None, None, None, None).is_none());
    assert!(CssAnimation::try_new(CssAnimationComponents::default()).is_none());
    assert!(CssTransitionList::try_new(Vec::new()).is_none());
    assert!(CssAnimationList::try_new(Vec::new()).is_none());
}

#[test]
fn checked_transition_preserves_distinct_time_domains_and_omissions() {
    let duration = CssDuration::try_new(CssTimeValue::from_literal(
        CssTimeLiteral::try_new("1", CssTimeUnit::Seconds).unwrap(),
    ))
    .unwrap();
    let delay = CssTimeValue::from_literal(
        CssTimeLiteral::try_new("-250", CssTimeUnit::Milliseconds).unwrap(),
    );
    let transition = CssTransition::try_new(
        Some(CssTransitionProperty::All),
        Some(duration),
        Some(delay),
        None,
    )
    .unwrap();
    assert!(matches!(
        transition.property(),
        Some(CssTransitionProperty::All)
    ));
    assert!(
        matches!(transition.duration().and_then(|value| value.time().literal()), Some(value)
        if value.numeric().representation() == "1" && value.unit() == CssTimeUnit::Seconds)
    );
    assert!(
        matches!(transition.delay().and_then(|value| value.literal()), Some(value)
        if value.numeric().representation() == "-250" && value.unit() == CssTimeUnit::Milliseconds)
    );
    assert!(transition.timing_function().is_none());
    assert_eq!(
        CssTransitionList::try_new(vec![transition])
            .unwrap()
            .values()
            .len(),
        1
    );
}

#[test]
fn checked_animation_preserves_eight_semantic_slots() {
    let animation = CssAnimation::try_new(CssAnimationComponents {
        name: Some(CssAnimationName::None),
        duration: Some(
            CssDuration::try_new(CssTimeValue::from_literal(
                CssTimeLiteral::try_new("2", CssTimeUnit::Seconds).unwrap(),
            ))
            .unwrap(),
        ),
        delay: Some(CssTimeValue::from_literal(
            CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap(),
        )),
        timing_function: Some(CssEasing::Keyword(CssEasingKeyword::EaseOut)),
        iteration_count: Some(CssAnimationIterationCount::Number(
            surgeist_css::CssSpecifiedNonNegativeNumber::try_from_component(
                surgeist_css::CssComponentValue::try_number("3").unwrap(),
            )
            .unwrap(),
        )),
        direction: Some(CssAnimationDirection::Alternate),
        fill_mode: Some(CssAnimationFillMode::Both),
        play_state: Some(CssAnimationPlayState::Paused),
    })
    .unwrap();
    assert!(matches!(animation.name(), Some(CssAnimationName::None)));
    assert!(
        matches!(animation.duration().and_then(|value| value.time().literal()), Some(value) if value.numeric().representation() == "2")
    );
    assert!(
        matches!(animation.delay().and_then(|value| value.literal()), Some(value) if value.numeric().representation() == "-1")
    );
    assert!(matches!(
        animation.timing_function(),
        Some(CssEasing::Keyword(CssEasingKeyword::EaseOut))
    ));
    assert!(
        matches!(animation.iteration_count(), Some(CssAnimationIterationCount::Number(value)) if value.serialize_specified().unwrap() == "3")
    );
    assert_eq!(
        animation.direction(),
        Some(CssAnimationDirection::Alternate)
    );
    assert_eq!(animation.fill_mode(), Some(CssAnimationFillMode::Both));
    assert_eq!(animation.play_state(), Some(CssAnimationPlayState::Paused));
    assert_eq!(
        CssAnimationList::try_new(vec![animation])
            .unwrap()
            .values()
            .len(),
        1
    );
}

#[test]
fn easing_construction_checks_bezier_and_step_ranges() {
    let literal = checked_number;
    assert!(
        CssCubicBezier::try_new(literal("-0.1"), literal("0"), literal("0.5"), literal("1"))
            .is_none()
    );
    assert!(
        CssCubicBezier::try_new(literal("0.1"), literal("-2"), literal("0.9"), literal("3"))
            .is_some()
    );
    assert!(
        CssSteps::try_new(
            CssPositiveIntegerValue::Literal(
                CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(1)).unwrap()
            ),
            Some(CssStepPosition::JumpNone)
        )
        .is_none()
    );
    assert!(
        CssSteps::try_new(
            CssPositiveIntegerValue::Literal(
                CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(2)).unwrap()
            ),
            Some(CssStepPosition::JumpNone)
        )
        .is_some()
    );
}

#[test]
fn parsed_symbolic_easing_remains_typed_through_checked_aggregates() {
    let report = parse_style_attribute("transition-timing-function: steps(calc(1 + 1), jump-none)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TransitionTimingFunction(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("timing function wrapper");
    };
    let [easing @ CssEasing::Steps(steps)] = wrapper.timing_functions().values() else {
        panic!("one steps function");
    };
    assert!(matches!(
        steps.count(),
        CssPositiveIntegerValue::Calculation(_)
    ));
    assert_eq!(steps.position(), Some(CssStepPosition::JumpNone));
    let transition = CssTransition::try_new(None, None, None, Some(easing.clone())).unwrap();
    let animation = CssAnimation::try_new(CssAnimationComponents {
        timing_function: Some(easing.clone()),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    assert!(
        matches!(transition.timing_function(), Some(CssEasing::Steps(value)) if matches!(value.count(), CssPositiveIntegerValue::Calculation(_)))
    );
    assert!(
        matches!(animation.timing_function(), Some(CssEasing::Steps(value)) if matches!(value.count(), CssPositiveIntegerValue::Calculation(_)))
    );
    assert!(animation.name().is_none());
    assert!(animation.duration().is_none());
}

fn checked_number(representation: &str) -> surgeist_css::CssSpecifiedNumber {
    surgeist_css::CssSpecifiedNumber::try_from_component(
        surgeist_css::CssComponentValue::try_number(representation).unwrap(),
    )
    .unwrap()
}
