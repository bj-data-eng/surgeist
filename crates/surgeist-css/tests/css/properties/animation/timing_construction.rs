use crate::css::support::value_assertions::positive_literal;
use surgeist_css::{
    CssAnimation, CssAnimationComponents, CssAnimationDirection, CssAnimationFillMode,
    CssAnimationIterationCount, CssAnimationList, CssAnimationName, CssAnimationPlayState,
    CssCubicBezier, CssEasing, CssEasingKeyword, CssInteger, CssKnownPropertyValueRef,
    CssNonNegative, CssNumeric, CssNumericLiteral, CssParser, CssPositive, CssStepPosition,
    CssSteps, CssTime, CssTimeUnit, CssTransition, CssTransitionList, CssTransitionProperty,
};

#[test]
fn checked_aggregates_reject_empty_items_and_lists() {
    assert!(CssTransition::try_new(None, None, None, None).is_none());
    assert!(CssAnimation::try_new(CssAnimationComponents::default()).is_none());
    assert!(CssTransitionList::try_new(Vec::new()).is_none());
    assert!(CssAnimationList::try_new(Vec::new()).is_none());
}

#[test]
fn checked_transition_preserves_distinct_time_domains_and_omissions() {
    let duration = CssNumeric::<CssTime, CssNonNegative>::try_from_literal(
        CssNumericLiteral::<CssTime>::try_new("1", CssTimeUnit::Seconds).unwrap(),
    )
    .unwrap();
    let delay = CssNumeric::<CssTime>::try_from_literal(
        CssNumericLiteral::<CssTime>::try_new("-250", CssTimeUnit::Milliseconds).unwrap(),
    )
    .unwrap();
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
        matches!(transition.duration().and_then(|value| value.literal()), Some(value)
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
    let animation =
        CssAnimation::try_new(CssAnimationComponents {
            name: Some(CssAnimationName::None),
            duration: Some(
                CssNumeric::<CssTime, CssNonNegative>::try_from_literal(
                    CssNumericLiteral::<CssTime>::try_new("2", CssTimeUnit::Seconds).unwrap(),
                )
                .unwrap(),
            ),
            delay: Some(
                CssNumeric::<CssTime>::try_from_literal(
                    CssNumericLiteral::<CssTime>::try_new("-1", CssTimeUnit::Seconds).unwrap(),
                )
                .unwrap(),
            ),
            timing_function: Some(CssEasing::Keyword(CssEasingKeyword::EaseOut)),
            iteration_count:
                Some(
                    CssAnimationIterationCount::Number(
                        surgeist_css::CssNumeric::<
                            surgeist_css::CssNumber,
                            surgeist_css::CssNonNegative,
                        >::try_from_component(
                            surgeist_css::CssComponentValue::try_number("3").unwrap(),
                        )
                        .unwrap(),
                    ),
                ),
            direction: Some(CssAnimationDirection::Alternate),
            fill_mode: Some(CssAnimationFillMode::Both),
            play_state: Some(CssAnimationPlayState::Paused),
        })
        .unwrap();
    assert!(matches!(animation.name(), Some(CssAnimationName::None)));
    assert!(
        matches!(animation.duration().and_then(|value| value.literal()), Some(value) if value.numeric().representation() == "2")
    );
    assert!(
        matches!(animation.delay().and_then(|value| value.literal()), Some(value) if value.numeric().representation() == "-1")
    );
    assert!(matches!(
        animation.timing_function(),
        Some(CssEasing::Keyword(CssEasingKeyword::EaseOut))
    ));
    assert!(
        matches!(animation.iteration_count(), Some(CssAnimationIterationCount::Number(value)) if value.serialize_specified(Default::default()).unwrap() == "3")
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
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(
                positive_literal(CssNumericLiteral::<CssInteger>::from_i32(1)).unwrap()
            )
            .unwrap(),
            Some(CssStepPosition::JumpNone)
        )
        .is_none()
    );
    assert!(
        CssSteps::try_new(
            CssNumeric::<CssInteger, CssPositive>::try_from_literal(
                positive_literal(CssNumericLiteral::<CssInteger>::from_i32(2)).unwrap()
            )
            .unwrap(),
            Some(CssStepPosition::JumpNone)
        )
        .is_some()
    );
}

#[test]
fn parsed_symbolic_easing_remains_typed_through_checked_aggregates() {
    let report = CssParser::default()
        .parse_block_contents("transition-timing-function: steps(calc(1 + 1), jump-none)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::TransitionTimingFunction(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("timing function wrapper");
    };
    let [easing @ CssEasing::Steps(steps)] = wrapper.value().values() else {
        panic!("one steps function");
    };
    assert!(steps.count().calculation().is_some());
    assert_eq!(steps.position(), Some(CssStepPosition::JumpNone));
    let transition = CssTransition::try_new(None, None, None, Some(easing.clone())).unwrap();
    let animation = CssAnimation::try_new(CssAnimationComponents {
        timing_function: Some(easing.clone()),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    assert!(
        matches!(transition.timing_function(), Some(CssEasing::Steps(value)) if value.count().calculation().is_some())
    );
    assert!(
        matches!(animation.timing_function(), Some(CssEasing::Steps(value)) if value.count().calculation().is_some())
    );
    assert!(animation.name().is_none());
    assert!(animation.duration().is_none());
}

fn checked_number(representation: &str) -> surgeist_css::CssNumeric<surgeist_css::CssNumber> {
    surgeist_css::CssNumeric::<surgeist_css::CssNumber>::try_from_component(
        surgeist_css::CssComponentValue::try_number(representation).unwrap(),
    )
    .unwrap()
}
