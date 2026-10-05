#![forbid(unsafe_code)]
//! New API functional evidence; no executable preimplementation RED exists.
//! Expected order and disambiguation follow the selected Transitions 1 §2.5
//! and Animations 1 §3.10 references. Limits follow the provider node policy.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn time(text: &str) -> CssTimeValue {
    CssTimeValue::from_literal(CssTimeLiteral::try_new(text, CssTimeUnit::Seconds).unwrap())
}

fn duration(text: &str) -> CssDuration {
    CssDuration::try_new(time(text)).unwrap()
}

fn name(text: &str) -> CssAnimationName {
    CssAnimationName::Custom(CssCustomIdent::try_new(text).unwrap())
}

fn property(text: &str) -> CssTransitionProperty {
    CssTransitionProperty::Custom(CssCustomIdent::try_new(text).unwrap())
}

fn count(text: &str) -> CssAnimationIterationCount {
    CssAnimationIterationCount::Number(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap(),
    )
}

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}

fn animations(css: &str) -> CssAnimationList {
    let declaration = declaration(&format!("animation:{css}"));
    let CssKnownPropertyValueRef::Animation(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("animation")
    };
    value.animations().clone()
}

fn transitions(css: &str) -> CssTransitionList {
    let declaration = declaration(&format!("transition:{css}"));
    let CssKnownPropertyValueRef::Transition(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("transition")
    };
    value.transitions().clone()
}

fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
}

macro_rules! check {
    ($value:expr, $input:expr, $projection:expr, $css:expr) => {{
        let value = $value;
        assert_eq!(value.serialize_specified().unwrap(), $css);
        budget($css, $input, $projection, |limits| {
            value.serialize_specified_with_limits(limits)
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
            CssAnimationName::String(CssKeyframesString::try_new("ease").unwrap())
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
fn full_shorthands_follow_grammar_order_and_keep_explicit_initials() {
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
    check!(&transition, 6, 6, "opacity 0s ease -1s");
    assert_eq!(
        transitions("0s ease opacity -1s")
            .serialize_specified()
            .unwrap(),
        "opacity 0s ease -1s"
    );
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
    check!(
        &animation,
        11,
        11,
        "2s ease-out -0.5s 1 normal none running slide"
    );
    let reparsed = animations(&animation.serialize_specified().unwrap());
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
                value.properties().serialize_specified()
            }
            CssKnownPropertyValueRef::TransitionDuration(value) => {
                value.durations().serialize_specified()
            }
            CssKnownPropertyValueRef::AnimationDuration(value) => {
                value.durations().serialize_specified()
            }
            CssKnownPropertyValueRef::TransitionDelay(value) => {
                value.delays().serialize_specified()
            }
            CssKnownPropertyValueRef::AnimationDelay(value) => value.delays().serialize_specified(),
            CssKnownPropertyValueRef::AnimationName(value) => value.names().serialize_specified(),
            CssKnownPropertyValueRef::AnimationIterationCount(value) => {
                value.iteration_counts().serialize_specified()
            }
            CssKnownPropertyValueRef::AnimationDirection(value) => {
                value.directions().serialize_specified()
            }
            CssKnownPropertyValueRef::AnimationFillMode(value) => {
                value.fill_modes().serialize_specified()
            }
            CssKnownPropertyValueRef::AnimationPlayState(value) => {
                value.play_states().serialize_specified()
            }
            _ => panic!("timed property"),
        }
        .unwrap();
        assert_eq!(text, expected, "{property}");
        assert!(parse_style_attribute(&format!("{property}:{text}")).is_clean());
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
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(animations(expected), value);
    }
}

#[test]
fn delay_only_items_insert_duration_without_mutating_omissions_or_exact_input() {
    let delay = CssTimeValue::from_literal(
        CssTimeLiteral::try_new("-250", CssTimeUnit::Milliseconds).unwrap(),
    );
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
            .serialize_specified()
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
        assert_eq!(parsed.serialize_specified().unwrap(), expected);
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
            .serialize_specified()
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
            let text = value.serialize_specified().unwrap();
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
            assert_eq!(parsed.serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn escaped_identifiers_and_quoted_names_keep_their_distinct_forms_and_list_order() {
    let value = CssAnimationNameList::try_new(vec![
        name("a b"),
        CssAnimationName::String(CssKeyframesString::try_new("a b").unwrap()),
        name("9spin"),
    ])
    .unwrap();
    check!(value, 4, 4, "a\\ b, \"a b\", \\39 spin");
    let parsed = animations("\"ease\", a\\ b, \\39 spin");
    assert_eq!(
        parsed.serialize_specified().unwrap(),
        "\"ease\", a\\ b, \\39 spin"
    );
    let reparsed = animations(&parsed.serialize_specified().unwrap());
    assert_eq!(parsed, reparsed);
}

#[test]
fn unrecoverable_keyword_variants_and_none_lists_fail_closed() {
    for reserved in ["all", "ALL", "none", "NoNe"] {
        let value = CssTransitionPropertyList::try_new(vec![property(reserved)]).unwrap();
        assert_eq!(
            value.serialize_specified().unwrap_err().kind(),
            K::UnrepresentableValue
        );
    }
    let names = CssAnimationNameList::try_new(vec![name("none")]).unwrap();
    assert_eq!(
        names.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
    let properties = CssTransitionPropertyList::try_new(vec![
        CssTransitionProperty::None,
        CssTransitionProperty::All,
    ])
    .unwrap();
    assert_eq!(
        properties.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
    let transitions = CssTransitionList::try_new(vec![
        CssTransition::try_new(Some(CssTransitionProperty::None), None, None, None).unwrap(),
        CssTransition::try_new(Some(CssTransitionProperty::All), None, None, None).unwrap(),
    ])
    .unwrap();
    assert_eq!(
        transitions.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
}

#[test]
fn late_byte_failure_returns_no_partial_css_and_leaves_large_lists_unchanged() {
    let value = CssDelayList::try_new(vec![time("-1"); 8192]).unwrap();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(4, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(usize::MAX, 4, usize::MAX))
            .unwrap_err()
            .kind(),
        K::ProjectionNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(usize::MAX, usize::MAX, 8))
            .unwrap_err()
            .kind(),
        K::ByteLimit
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
    assert!(CssDuration::try_new(tiny_negative.clone()).is_err());
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
fn nul_transition_property_identity_cannot_be_replaced_during_output() {
    let values = CssTransitionPropertyList::try_new(vec![property("a\0b")]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
    assert_eq!(values.properties()[0], property("a\0b"));
}

#[test]
fn nul_animation_identifier_identity_cannot_be_replaced_during_output() {
    let values = CssAnimationNameList::try_new(vec![name("a\0b")]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
    assert_eq!(values.names()[0], name("a\0b"));
}

#[test]
fn nul_quoted_animation_identity_cannot_be_replaced_during_output() {
    let original = CssAnimationName::String(CssKeyframesString::try_new("a\0b").unwrap());
    let values = CssAnimationNameList::try_new(vec![original.clone()]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
    assert_eq!(values.names()[0], original);
}

#[test]
fn nul_transition_shorthand_identity_cannot_be_replaced_during_output() {
    let item = CssTransition::try_new(Some(property("a\0b")), None, None, None).unwrap();
    let values = CssTransitionList::try_new(vec![item]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
}

#[test]
fn nul_animation_shorthand_identifier_cannot_be_replaced_during_output() {
    let item = CssAnimation::try_new(CssAnimationComponents {
        name: Some(name("a\0b")),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    let values = CssAnimationList::try_new(vec![item]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
}

#[test]
fn nul_animation_shorthand_string_cannot_be_replaced_during_output() {
    let item = CssAnimation::try_new(CssAnimationComponents {
        name: Some(CssAnimationName::String(
            CssKeyframesString::try_new("a\0b").unwrap(),
        )),
        ..CssAnimationComponents::default()
    })
    .unwrap();
    let values = CssAnimationList::try_new(vec![item]).unwrap();
    assert_eq!(
        values.serialize_specified().unwrap_err().kind(),
        K::UnrepresentableValue
    );
}
