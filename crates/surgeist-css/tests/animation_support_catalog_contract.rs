#![forbid(unsafe_code)]
//! Animations 1 WD 2023-03-02 §§3.2–3.10 defines the selected authored
//! property/list productions. Easing 1 CRD 2023-02-13 owns easing syntax.
//! Catalog completeness concerns authored grammar, without contextual animation
//! execution.

use surgeist_css::*;

fn complete(property: CssKnownProperty, value: &str, expected_entries: usize) {
    let css = format!("{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok(), "{css}");
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained ordinary declaration")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), property);
    assert_eq!(known.grammar(), property.grammar());
    let entries = match known.property_value().unwrap() {
        CssKnownPropertyValueRef::AnimationName(value) => value.names().names().len(),
        CssKnownPropertyValueRef::AnimationDuration(value) => value.durations().values().len(),
        CssKnownPropertyValueRef::AnimationDelay(value) => value.delays().values().len(),
        CssKnownPropertyValueRef::AnimationTimingFunction(value) => {
            value.timing_functions().values().len()
        }
        CssKnownPropertyValueRef::AnimationIterationCount(value) => {
            value.iteration_counts().values().len()
        }
        CssKnownPropertyValueRef::AnimationDirection(value) => {
            value.directions().directions().len()
        }
        CssKnownPropertyValueRef::AnimationFillMode(value) => value.fill_modes().modes().len(),
        CssKnownPropertyValueRef::AnimationPlayState(value) => value.play_states().states().len(),
        CssKnownPropertyValueRef::Animation(value) => {
            let animations = value.animations().values();
            assert_eq!(animations[1].fill_mode(), Some(CssAnimationFillMode::None));
            let Some(CssAnimationName::Custom(name)) = animations[1].name() else {
                panic!("keyword priority leaves backwards as the second item's name")
            };
            assert_eq!(name.as_str(), "backwards");
            animations.len()
        }
        other => panic!("animation ordinary list: {other:?}"),
    };
    assert_eq!(entries, expected_entries, "retained entries for {css}");

    let id = format!("baseline.property.{}", property.canonical_name());
    let feature = feature_metadata(&id).expect("recognized animation property feature");
    assert_eq!(feature.id().as_str(), id);
    assert_eq!(feature.spelling(), property.canonical_name());
    assert_eq!(known.grammar().feature_id(), feature.id());
    assert_eq!(feature.kind(), CssFeatureKind::Property);
    assert_eq!(feature.source().id().as_str(), "I-ANIMATIONS1");
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.supported_subset(), None);
    assert_eq!(feature.unsupported_remainder(), None);
    assert_eq!(feature.recognized_unsupported_code(), None);
}

#[test]
fn name_catalog_covers_selected_identifier_string_none_lists() {
    complete(
        CssKnownProperty::AnimationName,
        "none, fade, \"slide\", MiXeD",
        4,
    );
}

#[test]
fn duration_catalog_covers_selected_nonnegative_time_lists() {
    complete(
        CssKnownProperty::AnimationDuration,
        "0s, 125ms, calc(1s + 250ms)",
        3,
    );
}

#[test]
fn delay_catalog_covers_selected_signed_time_lists() {
    complete(
        CssKnownProperty::AnimationDelay,
        "-250ms, 0s, calc(-1s + 250ms)",
        3,
    );
}

#[test]
fn easing_catalog_covers_selected_keyword_bezier_and_step_lists() {
    complete(
        CssKnownProperty::AnimationTimingFunction,
        "linear, ease-in, cubic-bezier(.2,-1,.8,2), steps(2,jump-none)",
        4,
    );
}

#[test]
fn count_catalog_covers_selected_zero_fraction_infinite_and_math_lists() {
    complete(
        CssKnownProperty::AnimationIterationCount,
        "0, 2.5, infinite, calc(1 + 2)",
        4,
    );
}

#[test]
fn direction_catalog_covers_every_selected_direction_keyword_in_a_list() {
    complete(
        CssKnownProperty::AnimationDirection,
        "normal, reverse, alternate, alternate-reverse",
        4,
    );
}

#[test]
fn fill_catalog_covers_every_selected_fill_keyword_in_a_list() {
    complete(
        CssKnownProperty::AnimationFillMode,
        "none, forwards, backwards, both",
        4,
    );
}

#[test]
fn play_state_catalog_covers_both_selected_state_keywords_in_a_list() {
    complete(CssKnownProperty::AnimationPlayState, "running, paused", 2);
}

#[test]
fn shorthand_catalog_covers_all_slots_and_name_keyword_priority() {
    complete(
        CssKnownProperty::Animation,
        "fade 1s ease-in -250ms 2.5 reverse both paused, 3s none backwards",
        2,
    );
}
