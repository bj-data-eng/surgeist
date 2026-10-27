//! New API functional evidence; no executable preimplementation RED exists.
//! Expected order and disambiguation follow the selected Transitions 1 §2.5
//! and Animations 1 §3.10 references. Limits follow the provider node policy.

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::property_admission::assert_numeric_rejected;
use crate::css::support::serialization_limits::assert_limits as budget;
use crate::css::support::sole_declaration as declaration;
use crate::css::support::value_assertions::checked_range;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssValueOrigin};
use surgeist_css::numeric::{
    CssNonNegative, CssNumber, CssNumeric, CssNumericLiteral, CssTime, CssTimeUnit,
};
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::rule::keyframes::{CssKeyframesIdent, CssKeyframesString};
use surgeist_css::serialization::CssSerializationLimits as L;
use surgeist_css::values::animation::{
    CssAnimation, CssAnimationComponents, CssAnimationDirection, CssAnimationDirectionList,
    CssAnimationFillMode, CssAnimationFillModeList, CssAnimationIterationCount,
    CssAnimationIterationCountList, CssAnimationList, CssAnimationName, CssAnimationNameList,
    CssAnimationPlayState, CssAnimationPlayStateList, CssDelayList, CssDurationList, CssEasing,
    CssEasingKeyword, CssTransition, CssTransitionList, CssTransitionProperty,
    CssTransitionPropertyList, CssTransitionPropertyName,
};
use surgeist_css::values::common::CssCustomIdent;

fn time(text: &str) -> CssNumeric<CssTime> {
    CssNumeric::<CssTime>::try_from_literal(
        CssNumericLiteral::<CssTime>::try_new(text, CssTimeUnit::Seconds).unwrap(),
    )
    .unwrap()
}

fn duration(text: &str) -> CssNumeric<CssTime, CssNonNegative> {
    checked_range::<CssTime, CssNonNegative>(&time(text)).unwrap()
}

fn name(text: &str) -> CssAnimationName {
    CssAnimationName::Custom(
        CssKeyframesIdent::try_new(CssCustomIdent::try_new(text).unwrap()).unwrap(),
    )
}

fn property(text: &str) -> CssTransitionProperty {
    CssTransitionProperty::Custom(
        CssTransitionPropertyName::try_new(CssCustomIdent::try_new(text).unwrap()).unwrap(),
    )
}

fn count(text: &str) -> CssAnimationIterationCount {
    CssAnimationIterationCount::Number(
        CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap(),
    )
}

fn animations(css: &str) -> CssAnimationList {
    let declaration = declaration(&format!("animation:{css}"));
    let CssKnownPropertyValueRef::Animation(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("animation")
    };
    value.value().clone()
}

fn transitions(css: &str) -> CssTransitionList {
    let declaration = declaration(&format!("transition:{css}"));
    let CssKnownPropertyValueRef::Transition(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    value.value().clone()
}

macro_rules! check {
    ($value:expr, $input:expr, $projection:expr, $css:expr) => {{
        let value = $value;
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), $css);
        budget($css, $input, $projection, |limits| {
            value.serialize_specified(limits)
        });
    }};
}

#[test]
fn longhand_lists_keep_item_order_and_charge_exact_nodes_and_bytes() {
    check!(
        CssTransitionPropertyList::try_new(vec![property("opacity"), CssTransitionProperty::All])
            .unwrap(),
        3,
        3,
        "opacity, all"
    );
    check!(
        CssTransitionPropertyList::try_new(vec![CssTransitionProperty::None]).unwrap(),
        2,
        2,
        "none"
    );
    check!(
        CssDurationList::try_new(vec![duration("1"), duration("0.25")]).unwrap(),
        3,
        3,
        "1s, 0.25s"
    );
    check!(
        CssDelayList::try_new(vec![time("-1"), time("0")]).unwrap(),
        3,
        3,
        "-1s, 0s"
    );
    check!(
        CssAnimationNameList::try_new(vec![
            CssAnimationName::None,
            name("ease"),
            CssAnimationName::String(CssKeyframesString::new("ease"))
        ])
        .unwrap(),
        4,
        4,
        "none, ease, \"ease\""
    );
    check!(
        CssAnimationIterationCountList::try_new(vec![
            CssAnimationIterationCount::Infinite,
            count("2.5")
        ])
        .unwrap(),
        4,
        4,
        "infinite, 2.5"
    );
    check!(
        CssAnimationDirectionList::try_new(vec![
            CssAnimationDirection::Normal,
            CssAnimationDirection::Reverse,
            CssAnimationDirection::Alternate,
            CssAnimationDirection::AlternateReverse
        ])
        .unwrap(),
        5,
        5,
        "normal, reverse, alternate, alternate-reverse"
    );
    check!(
        CssAnimationFillModeList::try_new(vec![
            CssAnimationFillMode::None,
            CssAnimationFillMode::Forwards,
            CssAnimationFillMode::Backwards,
            CssAnimationFillMode::Both
        ])
        .unwrap(),
        5,
        5,
        "none, forwards, backwards, both"
    );
    check!(
        CssAnimationPlayStateList::try_new(vec![
            CssAnimationPlayState::Running,
            CssAnimationPlayState::Paused
        ])
        .unwrap(),
        3,
        3,
        "running, paused"
    );
}

#[test]
fn full_shorthands_follow_grammar_order_and_omit_meaningless_transition_initials() {
    let transition = CssTransitionList::try_new(vec![
        CssTransition::try_new(
            Some(property("opacity")),
            Some(duration("0")),
            Some(time("-1")),
            Some(CssEasing::Keyword(CssEasingKeyword::Ease)),
        )
        .unwrap(),
    ])
    .unwrap();
    // CSSOM (editor capture 2026-10-09) "serialize a CSS value" omits component
    // values whose omission keeps the meaning. Transitions 1 §2.5: initials are
    // `all`, `0s`, `ease` and `0s`, and the first <time> is always the duration,
    // so an initial duration stays whenever a delay follows. Omitted initials
    // still cost their nodes. Chromium 152 (2026-10-21) also drops `ease` but
    // emits the ambiguous `opacity -1s` here; the grammar governs.
    check!(&transition, 6, 6, "opacity 0s -1s");
    assert_eq!(
        transitions("0s ease opacity -1s")
            .serialize_specified(Default::default())
            .unwrap(),
        "opacity 0s -1s"
    );
    for (authored, expected) in [
        ("opacity 1s ease", "opacity 1s"),
        ("all 1s", "1s"),
        ("opacity 0s", "opacity"),
        ("opacity 0ms ease 0ms", "opacity"),
        ("opacity 1s -0s", "opacity 1s"),
        ("opacity 1s ease-in 0s", "opacity 1s ease-in"),
        ("none 1s", "none 1s"),
        ("opacity calc(0s)", "opacity calc(0s)"),
        ("opacity 1s ease, all 0s", "opacity 1s, all"),
        // Every component initial: the first present one in canonical order stays.
        ("all 0s ease 0s", "all"),
        ("ease", "ease"),
        ("0s 0s", "0s"),
        // A property spelled like an easing keyword needs an easing before it.
        ("ease-out linear", "ease-out linear"),
        ("linear ease", "linear ease"),
    ] {
        assert_eq!(
            transitions(authored)
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{authored}"
        );
        // The shortest form reenters the grammar and is already shortest.
        assert_eq!(
            transitions(expected)
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
    let animation = CssAnimationList::try_new(vec![
        CssAnimation::try_new(CssAnimationComponents {
            name: Some(name("slide")),
            duration: Some(duration("2")),
            delay: Some(time("-0.5")),
            timing_function: Some(CssEasing::Keyword(CssEasingKeyword::EaseOut)),
            iteration_count: Some(count("1")),
            direction: Some(CssAnimationDirection::Normal),
            fill_mode: Some(CssAnimationFillMode::None),
            play_state: Some(CssAnimationPlayState::Running),
        })
        .unwrap(),
    ])
    .unwrap();
    // Recorded exception: the animation shorthand keeps explicit initials.
    // Animations 1 §3.10 lists its components with no omission rule, and CSSOM's
    // note leaves property-specific legacy serialization to implementations;
    // Chromium 152 (2026-10-21) emits `1s ease 0s 1 normal none running fade`
    // for an all-explicit animation, keeping every initial.
    check!(
        &animation,
        11,
        11,
        "2s ease-out -0.5s 1 normal none running slide"
    );
    let reparsed = animations(&animation.serialize_specified(Default::default()).unwrap());
    assert_eq!(reparsed, animation);
    assert_eq!(
        animations("slide running none normal 1 2s ease-out -0.5s"),
        animation
    );
}

#[test]
fn parsed_longhands_use_the_same_canonical_providers_as_constructed_lists() {
    for (property, authored, expected) in [
        ("transition-property", "opacity, ALL", "opacity, all"),
        ("transition-duration", "1000ms, 250ms", "1s, 0.25s"),
        ("animation-duration", "1000ms, 250ms", "1s, 0.25s"),
        ("transition-delay", "-1000ms, 0ms", "-1s, 0s"),
        ("animation-delay", "-1000ms, 0ms", "-1s, 0s"),
        (
            "animation-name",
            "none, ease, 'ease'",
            "none, ease, \"ease\"",
        ),
        (
            "animation-iteration-count",
            "INFINITE, 2.50",
            "infinite, 2.5",
        ),
        (
            "animation-direction",
            "NORMAL, reverse, alternate, alternate-reverse",
            "normal, reverse, alternate, alternate-reverse",
        ),
        (
            "animation-fill-mode",
            "NONE, forwards, backwards, both",
            "none, forwards, backwards, both",
        ),
        ("animation-play-state", "RUNNING, paused", "running, paused"),
    ] {
        let parsed = declaration(&format!("{property}:{authored}"));
        let text = match parsed.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::TransitionProperty(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::TransitionDuration(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationDuration(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::TransitionDelay(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationDelay(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationName(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationIterationCount(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationDirection(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationFillMode(value) => {
                value.value().serialize_specified(Default::default())
            }
            CssKnownPropertyValueRef::AnimationPlayState(value) => {
                value.value().serialize_specified(Default::default())
            }
            _ => panic!("timed property"),
        }
        .unwrap();
        assert_eq!(text, expected, "{property}");
        assert!(
            CssParser::default()
                .parse_block_contents(format!("{property}:{text}"))
                .is_clean()
        );
    }
}

#[test]
fn explicit_non_name_slots_disambiguate_keyword_names_without_extra_defaults() {
    for (components, expected) in [
        (
            CssAnimationComponents {
                name: Some(name("ease")),
                timing_function: Some(CssEasing::Keyword(CssEasingKeyword::EaseOut)),
                ..Default::default()
            },
            "ease-out ease",
        ),
        (
            CssAnimationComponents {
                name: Some(name("infinite")),
                iteration_count: Some(count("2")),
                ..Default::default()
            },
            "2 infinite",
        ),
        (
            CssAnimationComponents {
                name: Some(name("normal")),
                direction: Some(CssAnimationDirection::Reverse),
                ..Default::default()
            },
            "reverse normal",
        ),
        (
            CssAnimationComponents {
                name: Some(name("backwards")),
                fill_mode: Some(CssAnimationFillMode::Forwards),
                ..Default::default()
            },
            "forwards backwards",
        ),
        (
            CssAnimationComponents {
                name: Some(name("running")),
                play_state: Some(CssAnimationPlayState::Paused),
                ..Default::default()
            },
            "paused running",
        ),
        (
            CssAnimationComponents {
                name: Some(CssAnimationName::None),
                fill_mode: Some(CssAnimationFillMode::Both),
                ..Default::default()
            },
            "both none",
        ),
    ] {
        let value =
            CssAnimationList::try_new(vec![CssAnimation::try_new(components).unwrap()]).unwrap();
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(animations(expected), value);
    }
}

#[test]
fn delay_only_items_insert_duration_without_mutating_omissions_or_exact_input() {
    let delay = CssNumeric::<CssTime>::try_from_literal(
        CssNumericLiteral::<CssTime>::try_new("-250", CssTimeUnit::Milliseconds).unwrap(),
    )
    .unwrap();
    let transition = CssTransitionList::try_new(vec![
        CssTransition::try_new(None, None, Some(delay.clone()), None).unwrap(),
    ])
    .unwrap();
    check!(&transition, 3, 4, "0s -0.25s");
    let animation = CssAnimationList::try_new(vec![
        CssAnimation::try_new(CssAnimationComponents {
            delay: Some(delay),
            ..Default::default()
        })
        .unwrap(),
    ])
    .unwrap();
    check!(&animation, 3, 4, "0s -0.25s");
    assert!(transition.values()[0].duration().is_none());
    assert!(animation.values()[0].duration().is_none());
    let literal = animation.values()[0].delay().unwrap().literal().unwrap();
    assert_eq!(literal.numeric().representation(), "-250");
    assert_eq!(literal.unit(), CssTimeUnit::Milliseconds);
    assert_eq!(literal.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        animations("0s -0.25s").values()[0]
            .delay()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "-0.25s"
    );
}

#[test]
fn keyword_names_fill_only_the_ambiguous_slot_before_the_final_name() {
    for (name_text, expected) in [
        ("ease", "ease ease"),
        ("linear", "ease linear"),
        ("ease-in", "ease ease-in"),
        ("ease-out", "ease ease-out"),
        ("ease-in-out", "ease ease-in-out"),
        ("step-start", "ease step-start"),
        ("step-end", "ease step-end"),
        ("infinite", "1 infinite"),
        ("normal", "normal normal"),
        ("reverse", "normal reverse"),
        ("alternate", "normal alternate"),
        ("alternate-reverse", "normal alternate-reverse"),
        ("forwards", "none forwards"),
        ("backwards", "none backwards"),
        ("both", "none both"),
        ("running", "running running"),
        ("paused", "running paused"),
        ("EaSe", "ease EaSe"),
    ] {
        let original_name = name(name_text);
        let value = CssAnimationList::try_new(vec![
            CssAnimation::try_new(CssAnimationComponents {
                name: Some(original_name.clone()),
                ..Default::default()
            })
            .unwrap(),
        ])
        .unwrap();
        check!(&value, 3, 4, expected);
        let parsed = animations(expected);
        assert_eq!(
            parsed.values()[0].name(),
            Some(&original_name),
            "{name_text}"
        );
        assert_eq!(
            parsed.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert!(value.values()[0].timing_function().is_none());
        assert!(value.values()[0].iteration_count().is_none());
        assert!(value.values()[0].direction().is_none());
        assert!(value.values()[0].fill_mode().is_none());
        assert!(value.values()[0].play_state().is_none());
    }
    let explicit_none = CssAnimationList::try_new(vec![
        CssAnimation::try_new(CssAnimationComponents {
            name: Some(CssAnimationName::None),
            ..Default::default()
        })
        .unwrap(),
    ])
    .unwrap();
    check!(explicit_none, 3, 4, "none none");
    assert_eq!(
        animations("none none").values()[0].name(),
        Some(&CssAnimationName::None)
    );
    assert_eq!(
        animations("3s none backwards")
            .serialize_specified(Default::default())
            .unwrap(),
        "3s none backwards"
    );
}

#[test]
fn transition_keyword_property_follows_the_actual_or_default_easing_slot() {
    for keyword in [
        "ease",
        "linear",
        "ease-in",
        "ease-out",
        "ease-in-out",
        "step-start",
        "step-end",
    ] {
        let prop = property(keyword);
        for easing in [None, Some(CssEasing::Keyword(CssEasingKeyword::EaseOut))] {
            let value = CssTransitionList::try_new(vec![
                CssTransition::try_new(
                    Some(prop.clone()),
                    Some(duration("1")),
                    None,
                    easing.clone(),
                )
                .unwrap(),
            ])
            .unwrap();
            let expected = format!(
                "1s {} {keyword}",
                if easing.is_some() { "ease-out" } else { "ease" }
            );
            let text = value.serialize_specified(Default::default()).unwrap();
            assert_eq!(text, expected);
            let parsed = transitions(&text);
            assert_eq!(parsed.values()[0].property(), Some(&prop));
            assert_eq!(
                parsed.values()[0].timing_function(),
                Some(
                    easing
                        .as_ref()
                        .unwrap_or(&CssEasing::Keyword(CssEasingKeyword::Ease))
                )
            );
            assert_eq!(
                parsed.serialize_specified(Default::default()).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn escaped_identifiers_and_quoted_names_keep_their_distinct_forms_and_list_order() {
    let value = CssAnimationNameList::try_new(vec![
        name("a b"),
        CssAnimationName::String(CssKeyframesString::new("a b")),
        name("9spin"),
    ])
    .unwrap();
    check!(value, 4, 4, "a\\ b, \"a b\", \\39 spin");
    let parsed = animations("\"ease\", a\\ b, \\39 spin");
    assert_eq!(
        parsed.serialize_specified(Default::default()).unwrap(),
        "\"ease\", a\\ b, \\39 spin"
    );
    let reparsed = animations(&parsed.serialize_specified(Default::default()).unwrap());
    assert_eq!(parsed, reparsed);
}

#[test]
fn checked_keyword_names_and_mixed_none_lists_are_rejected() {
    for reserved in ["all", "ALL", "none", "NoNe"] {
        assert!(
            CssTransitionPropertyName::try_new(CssCustomIdent::try_new(reserved).unwrap())
                .is_none()
        );
    }
    assert!(CssKeyframesIdent::try_new(CssCustomIdent::try_new("none").unwrap()).is_none());
    assert!(
        CssTransitionPropertyList::try_new(vec![
            CssTransitionProperty::None,
            CssTransitionProperty::All,
        ])
        .is_none()
    );
    assert!(
        CssTransitionList::try_new(vec![
            CssTransition::try_new(Some(CssTransitionProperty::None), None, None, None).unwrap(),
            CssTransition::try_new(Some(CssTransitionProperty::All), None, None, None).unwrap(),
        ])
        .is_none()
    );
}

#[test]
fn late_byte_failure_returns_no_partial_css_and_leaves_large_lists_unchanged() {
    let value = CssDelayList::try_new(vec![time("-1"); 8192]).unwrap();
    assert!(
        value
            .serialize_specified(L::new(4, usize::MAX, usize::MAX))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        value
            .serialize_specified(L::new(usize::MAX, 4, usize::MAX))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        value
            .serialize_specified(L::new(usize::MAX, usize::MAX, 8))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(value.values().len(), 8192);
    assert_eq!(
        value.values()[8191]
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "-1"
    );
    check!(
        CssDelayList::try_new(value.values()[..2].to_vec()).unwrap(),
        3,
        3,
        "-1s, -1s"
    );
}

#[test]
fn exact_duration_admission_and_signed_delay_projection_keep_distinct_domains() {
    let tiny_negative = time("-1e-999");
    assert_numeric_rejected(
        checked_range::<CssTime, CssNonNegative>(&tiny_negative.clone()),
        NumericFailure::OutOfRange,
    );
    let delays = CssDelayList::try_new(vec![tiny_negative]).unwrap();
    check!(&delays, 2, 2, "0s");
    assert_eq!(
        delays.values()[0]
            .literal()
            .unwrap()
            .numeric()
            .representation(),
        "-1e-999"
    );
    assert_eq!(delays.values()[0].origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn nul_quoted_animation_identity_cannot_be_replaced_during_output() {
    let original = CssAnimationName::String(CssKeyframesString::new("a\0b"));
    let values = CssAnimationNameList::try_new(vec![original.clone()]).unwrap();
    assert!(
        values
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
    assert_eq!(values.names()[0], original);
}

#[test]
fn nul_animation_shorthand_string_cannot_be_replaced_during_output() {
    let item = CssAnimation::try_new(CssAnimationComponents {
        name: Some(CssAnimationName::String(CssKeyframesString::new("a\0b"))),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    let values = CssAnimationList::try_new(vec![item]).unwrap();
    assert!(
        values
            .serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
}
