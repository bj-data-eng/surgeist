//! Animations 1 WD 2023-03-02 §§3.2–3.10 supplies list admission, initials,
//! omission defaults and keyword/time-slot priority; Easing 1 CRD 2023-02-13
//! supplies easing admission. Cascade 5 CR 2022-01-13 §3.1 supplies shorthand
//! propagation. Exact child provenance, ordered intrinsic contributions and
//! bounded atomic failures are Surgeist contracts, before contextual execution.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::Responsible::{At, End};
use crate::css::support::property_admission::eof_closure_origin as implicit_origin;
use crate::css::support::property_admission::{
    assert_checked_rejected, assert_implicit_closure, assert_style_attribute_rejected,
};
use crate::css::support::{
    checked_property_grammar_declaration as checked, sole_declaration as parsed,
};
use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const MEMBERS: [P; 8] = [
    P::AnimationDuration,
    P::AnimationTimingFunction,
    P::AnimationDelay,
    P::AnimationIterationCount,
    P::AnimationDirection,
    P::AnimationFillMode,
    P::AnimationPlayState,
    P::AnimationName,
];
const INITIALS: [&str; 8] = ["0s", "ease", "0s", "1", "normal", "none", "running", "none"];

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary longhands")
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
struct Lists<'a> {
    durations: &'a CssDurationList,
    easings: &'a CssEasingList,
    delays: &'a CssDelayList,
    counts: &'a CssAnimationIterationCountList,
    directions: &'a CssAnimationDirectionList,
    fills: &'a CssAnimationFillModeList,
    plays: &'a CssAnimationPlayStateList,
    names: &'a CssAnimationNameList,
}
fn lists(values: &CssLonghandContributions) -> Lists<'_> {
    assert_eq!(
        values
            .items()
            .iter()
            .map(CssLonghandContribution::property)
            .collect::<Vec<_>>(),
        MEMBERS
    );
    let [duration, easing, delay, count, direction, fill, play, name] = values.items() else {
        panic!("eight terminals")
    };
    let CssLonghandValueRef::AnimationDuration(durations) =
        duration.ordinary_value().unwrap().view()
    else {
        panic!("duration")
    };
    let CssLonghandValueRef::AnimationTimingFunction(easings) =
        easing.ordinary_value().unwrap().view()
    else {
        panic!("easing")
    };
    let CssLonghandValueRef::AnimationDelay(delays) = delay.ordinary_value().unwrap().view() else {
        panic!("delay")
    };
    let CssLonghandValueRef::AnimationIterationCount(counts) =
        count.ordinary_value().unwrap().view()
    else {
        panic!("count")
    };
    let CssLonghandValueRef::AnimationDirection(directions) =
        direction.ordinary_value().unwrap().view()
    else {
        panic!("direction")
    };
    let CssLonghandValueRef::AnimationFillMode(fills) = fill.ordinary_value().unwrap().view()
    else {
        panic!("fill")
    };
    let CssLonghandValueRef::AnimationPlayState(plays) = play.ordinary_value().unwrap().view()
    else {
        panic!("play")
    };
    let CssLonghandValueRef::AnimationName(names) = name.ordinary_value().unwrap().view() else {
        panic!("name")
    };
    Lists {
        durations,
        easings,
        delays,
        counts,
        directions,
        fills,
        plays,
        names,
    }
}
fn output(
    value: CssLonghandValueRef<'_>,
    limits: CssSerializationLimits,
) -> Result<String, CssSerializationError> {
    match value {
        CssLonghandValueRef::AnimationDuration(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationTimingFunction(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationDelay(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationIterationCount(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationDirection(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationFillMode(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationPlayState(v) => v.serialize_specified(limits),
        CssLonghandValueRef::AnimationName(v) => v.serialize_specified(limits),
        other => panic!("animation payload: {other:?}"),
    }
}
fn length(value: CssLonghandValueRef<'_>) -> usize {
    match value {
        CssLonghandValueRef::AnimationDuration(v) => v.values().len(),
        CssLonghandValueRef::AnimationTimingFunction(v) => v.values().len(),
        CssLonghandValueRef::AnimationDelay(v) => v.values().len(),
        CssLonghandValueRef::AnimationIterationCount(v) => v.values().len(),
        CssLonghandValueRef::AnimationDirection(v) => v.directions().len(),
        CssLonghandValueRef::AnimationFillMode(v) => v.modes().len(),
        CssLonghandValueRef::AnimationPlayState(v) => v.states().len(),
        CssLonghandValueRef::AnimationName(v) => v.names().len(),
        other => panic!("animation payload: {other:?}"),
    }
}
fn origin_clone(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    assert_eq!(actual, expected);
    if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected) {
        assert!(actual.source().same_snapshot(expected.source()));
        assert_eq!(actual.span(), expected.span());
    }
}
fn time_clone<R: CssRange<CssTime>>(
    actual: &CssNumeric<CssTime, R>,
    expected: &CssNumeric<CssTime, R>,
) {
    assert_eq!(actual, expected); // Value equality ignores origins; checked below.
    origin_clone(actual.origin(), expected.origin());
    if let Some(actual) = actual.calculation() {
        assert_eq!(
            actual.components(),
            expected.calculation().unwrap().components()
        );
    }
}
fn count_clone(actual: &CssAnimationIterationCount, expected: &CssAnimationIterationCount) {
    assert_eq!(actual, expected); // Count's semantic equality alone omits origins.
    if let (
        CssAnimationIterationCount::Number(actual),
        CssAnimationIterationCount::Number(expected),
    ) = (actual, expected)
    {
        assert_eq!(actual, expected); // Numeric value equality ignores origins.
        origin_clone(actual.origin(), expected.origin());
        assert_eq!(
            actual.literal().map(|literal| literal.component()),
            expected.literal().map(|literal| literal.component())
        );
        if let Some(actual) = actual.calculation() {
            assert_eq!(
                actual.components(),
                expected.calculation().unwrap().components()
            );
        }
    }
}
fn easing_clone(actual: &CssEasing, expected: &CssEasing) {
    assert_eq!(actual, expected);
    match (actual, expected) {
        (CssEasing::CubicBezier(a), CssEasing::CubicBezier(b)) => {
            for (a, b) in [
                (a.x1().value(), b.x1().value()),
                (a.y1(), b.y1()),
                (a.x2().value(), b.x2().value()),
                (a.y2(), b.y2()),
            ] {
                assert_eq!(a, b);
                origin_clone(a.origin(), b.origin());
                assert_eq!(
                    a.literal().map(|literal| literal.component()),
                    b.literal().map(|literal| literal.component())
                );
                if let Some(a) = a.calculation() {
                    assert_eq!(a.components(), b.calculation().unwrap().components());
                }
            }
        }
        (CssEasing::Steps(a), CssEasing::Steps(b)) => match (
            (a.count().literal(), a.count().calculation()),
            (b.count().literal(), b.count().calculation()),
        ) {
            ((Some(a), _), (Some(b), _)) => {
                assert_eq!(a.component(), b.component());
                origin_clone(a.component().origin(), b.component().origin());
            }
            ((_, Some(a)), (_, Some(b))) => {
                assert_eq!(a.components(), b.components());
                origin_clone(a.origin(), b.origin());
            }
            other => panic!("same easing count branch: {other:?}"),
        },
        (CssEasing::Keyword(_), CssEasing::Keyword(_)) => {}
        other => panic!("same easing branch: {other:?}"),
    }
}
fn animation(source: &CssDeclaration) -> &CssAnimationList {
    let CssKnownPropertyValueRef::Animation(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("animation")
    };
    value.animations()
}
fn explicit_children(actual: &Lists<'_>, original: &CssAnimationList) {
    for (index, item) in original.values().iter().enumerate() {
        if let Some(value) = item.duration() {
            time_clone(&actual.durations.values()[index], value);
        }
        if let Some(value) = item.delay() {
            time_clone(&actual.delays.values()[index], value);
        }
        if let Some(value) = item.timing_function() {
            easing_clone(&actual.easings.values()[index], value);
        }
        if let Some(value) = item.iteration_count() {
            count_clone(&actual.counts.values()[index], value);
        }
        if let Some(value) = item.direction() {
            assert_eq!(actual.directions.directions()[index], value);
        }
        if let Some(value) = item.fill_mode() {
            assert_eq!(actual.fills.modes()[index], value);
        }
        if let Some(value) = item.play_state() {
            assert_eq!(actual.plays.states()[index], value);
        }
        if let Some(value) = item.name() {
            assert_eq!(&actual.names.names()[index], value);
        }
    }
}

#[test]
fn eight_borrowed_initial_variants_have_exact_programmatic_scalar_contents() {
    for (property, expected) in MEMBERS.into_iter().zip(INITIALS) {
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
        assert_eq!(length(value.view()), 1);
        assert_eq!(
            output(value.view(), CssSerializationLimits::default()).unwrap(),
            expected
        );
        match value.view() {
            CssLonghandValueRef::AnimationDuration(v) => {
                let time = &v.values()[0];
                assert_eq!(time.origin(), &CssValueOrigin::Programmatic);
                assert_eq!(time.literal().unwrap().numeric().representation(), "0");
                assert_eq!(time.literal().unwrap().unit(), CssTimeUnit::Seconds);
            }
            CssLonghandValueRef::AnimationDelay(v) => {
                assert_eq!(v.values()[0].origin(), &CssValueOrigin::Programmatic);
                assert_eq!(
                    v.values()[0].literal().unwrap().unit(),
                    CssTimeUnit::Seconds
                );
            }
            CssLonghandValueRef::AnimationIterationCount(v) => {
                let CssAnimationIterationCount::Number(v) = &v.values()[0] else {
                    panic!("ordinary one")
                };
                assert_eq!(v.origin(), &CssValueOrigin::Programmatic);
                assert_eq!(
                    v.literal().map(|literal| literal.component()),
                    Some(&CssComponentValue::try_number("1").unwrap())
                );
            }
            CssLonghandValueRef::AnimationTimingFunction(v) => {
                assert_eq!(v.values(), &[CssEasing::Keyword(CssEasingKeyword::Ease)])
            }
            CssLonghandValueRef::AnimationDirection(v) => {
                assert_eq!(v.directions(), &[CssAnimationDirection::Normal])
            }
            CssLonghandValueRef::AnimationFillMode(v) => {
                assert_eq!(v.modes(), &[CssAnimationFillMode::None])
            }
            CssLonghandValueRef::AnimationPlayState(v) => {
                assert_eq!(v.states(), &[CssAnimationPlayState::Running])
            }
            CssLonghandValueRef::AnimationName(v) => {
                assert_eq!(v.names(), &[CssAnimationName::None])
            }
            other => panic!("animation initial: {other:?}"),
        }
    }
}

fn entry(index: usize) -> [String; 8] {
    [
        format!("{}ms", index + 1),
        if index.is_multiple_of(2) {
            "ease-in"
        } else {
            "linear"
        }
        .into(),
        format!("-{}ms", index + 1),
        format!("{}.5", index + 1),
        ["normal", "reverse", "alternate", "alternate-reverse"][index % 4].into(),
        ["none", "forwards", "backwards", "both"][index % 4].into(),
        if index.is_multiple_of(2) {
            "running"
        } else {
            "paused"
        }
        .into(),
        format!("fade{index}"),
    ]
}
fn canonical_entries(slot: usize) -> String {
    (0..20)
        .map(|index| match slot {
            0 => format!("{}s", format!("0.{:03}", index + 1).trim_end_matches('0')),
            2 => format!("-{}s", format!("0.{:03}", index + 1).trim_end_matches('0')),
            _ => entry(index)[slot].clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
fn direct_children(values: &CssLonghandContributions, source: &CssDeclaration) {
    let view = values.items()[0].ordinary_value().unwrap().view();
    match (view, source.known().unwrap().property_value().unwrap()) {
        (
            CssLonghandValueRef::AnimationDuration(a),
            CssKnownPropertyValueRef::AnimationDuration(b),
        ) => {
            for (a, b) in a.values().iter().zip(b.durations().values()) {
                time_clone(a, b);
            }
        }
        (CssLonghandValueRef::AnimationDelay(a), CssKnownPropertyValueRef::AnimationDelay(b)) => {
            for (a, b) in a.values().iter().zip(b.delays().values()) {
                time_clone(a, b);
            }
        }
        (
            CssLonghandValueRef::AnimationTimingFunction(a),
            CssKnownPropertyValueRef::AnimationTimingFunction(b),
        ) => {
            for (a, b) in a.values().iter().zip(b.timing_functions().values()) {
                easing_clone(a, b);
            }
        }
        (
            CssLonghandValueRef::AnimationIterationCount(a),
            CssKnownPropertyValueRef::AnimationIterationCount(b),
        ) => {
            for (a, b) in a.values().iter().zip(b.iteration_counts().values()) {
                count_clone(a, b);
            }
        }
        (
            CssLonghandValueRef::AnimationDirection(a),
            CssKnownPropertyValueRef::AnimationDirection(b),
        ) => assert_eq!(a.directions(), b.directions().directions()),
        (
            CssLonghandValueRef::AnimationFillMode(a),
            CssKnownPropertyValueRef::AnimationFillMode(b),
        ) => assert_eq!(a.modes(), b.fill_modes().modes()),
        (
            CssLonghandValueRef::AnimationPlayState(a),
            CssKnownPropertyValueRef::AnimationPlayState(b),
        ) => assert_eq!(a.states(), b.play_states().states()),
        (CssLonghandValueRef::AnimationName(a), CssKnownPropertyValueRef::AnimationName(b)) => {
            assert_eq!(a.names(), b.names().names())
        }
        other => panic!("matching original longhand: {other:?}"),
    }
}
#[test]
fn all_nine_properties_preserve_every_entry_of_twenty_item_lists_across_three_fronts() {
    for (slot, property) in MEMBERS.into_iter().chain([P::Animation]).enumerate() {
        let css = (0..20)
            .map(|index| {
                if property == P::Animation {
                    entry(index).join(" ")
                } else {
                    entry(index)[slot].clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let authored = parsed(&format!(
            "/*😀*/{}:{css}!important",
            property.canonical_name()
        ));
        let grammar = checked(property, &css);
        let named = parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(&css).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        assert!(authored.position().is_some());
        assert!(grammar.position().is_none());
        assert!(named.position().is_none());
        for source in [&authored, &grammar, &named] {
            let values = completed(source);
            if property == P::Animation {
                let actual = lists(&values);
                assert_eq!(animation(source).values().len(), 20);
                explicit_children(&actual, animation(source));
                for (slot, item) in values.items().iter().enumerate() {
                    let view = item.ordinary_value().unwrap().view();
                    assert_eq!(length(view), 20);
                    assert_eq!(
                        output(view, CssSerializationLimits::default()).unwrap(),
                        canonical_entries(slot)
                    );
                }
            } else {
                assert_eq!(values.items().len(), 1);
                direct_children(&values, source);
                let view = values.items()[0].ordinary_value().unwrap().view();
                assert_eq!(length(view), 20);
                assert_eq!(
                    output(view, CssSerializationLimits::default()).unwrap(),
                    canonical_entries(slot)
                );
            }
        }
    }
}

#[test]
fn sparse_items_use_scalar_defaults_and_preserve_explicit_child_origins() {
    let source = parsed(
        "animation:fade 125ms, ease-in reverse both paused, slide 2s linear -250ms 2.5!important",
    );
    let values = completed(&source);
    let actual = lists(&values);
    explicit_children(&actual, animation(&source));
    for (item, expected) in values.items().iter().zip([
        "0.125s, 0s, 2s",
        "ease, ease-in, linear",
        "0s, 0s, -0.25s",
        "1, 1, 2.5",
        "normal, reverse, normal",
        "none, both, none",
        "running, paused, running",
        "fade, none, slide",
    ]) {
        let view = item.ordinary_value().unwrap().view();
        assert_eq!(length(view), 3);
        assert_eq!(
            output(view, CssSerializationLimits::default()).unwrap(),
            expected
        );
    }
    assert_eq!(
        actual.durations.values()[1].origin(),
        &CssValueOrigin::Programmatic
    );
    for delay in &actual.delays.values()[..2] {
        assert_eq!(delay.origin(), &CssValueOrigin::Programmatic);
    }
    for count in &actual.counts.values()[..2] {
        let CssAnimationIterationCount::Number(count) = count else {
            panic!("default one")
        };
        assert_eq!(count.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            count.literal().map(|literal| literal.component()),
            Some(&CssComponentValue::try_number("1").unwrap())
        );
    }
    assert!(matches!(
        actual.durations.values()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert!(matches!(
        actual.delays.values()[2].origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn explicit_initial_slots_retain_parsed_origins_while_omitted_defaults_are_programmatic() {
    let source = parsed("animation:fade 0s ease 0s 1 normal none running, slide");
    let values = completed(&source);
    let actual = lists(&values);
    explicit_children(&actual, animation(&source));
    for (list, expected) in values.items().iter().zip([
        "0s, 0s",
        "ease, ease",
        "0s, 0s",
        "1, 1",
        "normal, normal",
        "none, none",
        "running, running",
        "fade, slide",
    ]) {
        assert_eq!(
            output(
                list.ordinary_value().unwrap().view(),
                CssSerializationLimits::default()
            )
            .unwrap(),
            expected
        );
    }
    for times in [
        [
            actual.durations.values()[0].clone().into_any_range(),
            actual.durations.values()[1].clone().into_any_range(),
        ],
        [
            actual.delays.values()[0].clone(),
            actual.delays.values()[1].clone(),
        ],
    ] {
        assert!(matches!(times[0].origin(), CssValueOrigin::Parsed(_)));
        assert_eq!(times[1].origin(), &CssValueOrigin::Programmatic);
        assert_eq!(times[0], times[1]);
    }
    let [
        CssAnimationIterationCount::Number(explicit),
        CssAnimationIterationCount::Number(default),
    ] = actual.counts.values()
    else {
        panic!("two ordinary counts")
    };
    assert!(matches!(explicit.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(default.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(explicit, default);
    assert_eq!(actual.counts.values()[0], actual.counts.values()[1]);
}

#[test]
fn symbolic_count_and_easing_children_retain_original_graphs_and_numeric_precision() {
    for source in [
        parsed(
            "/*😀*/animation:fade calc(1s + 2s) cubic-bezier(calc(.25),-2,.75,3) calc(-250ms) calc(1 + 1.5)!important",
        ),
        checked(
            P::Animation,
            "fade calc(1s + 2s) steps(calc(2 + 1),jump-none) calc(-250ms) calc(1 + 1.5)",
        ),
    ] {
        let values = completed(&source);
        let actual = lists(&values);
        explicit_children(&actual, animation(&source));
        assert_eq!(
            actual
                .durations
                .serialize_specified(Default::default())
                .unwrap(),
            "calc(3s)"
        );
        assert_eq!(
            actual
                .delays
                .serialize_specified(Default::default())
                .unwrap(),
            "calc(-0.25s)"
        );
        assert_eq!(
            actual
                .counts
                .serialize_specified(Default::default())
                .unwrap(),
            "calc(2.5)"
        );
        assert!(actual.durations.values()[0].calculation().is_some());
        let CssAnimationIterationCount::Number(count) = &actual.counts.values()[0] else {
            panic!("count")
        };
        assert!(count.calculation().is_some());
        assert!(matches!(count.origin(), CssValueOrigin::Parsed(_)));
        let expected = if source.position().is_some() {
            "cubic-bezier(calc(0.25), -2, 0.75, 3)"
        } else {
            "steps(calc(3), jump-none)"
        };
        assert_eq!(
            actual
                .easings
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn mixed_programmatic_and_parsed_children_survive_checked_projection_and_reentry() {
    let components =
        parse_component_values("fade 1s ease-in -250ms 2.5 reverse both paused").unwrap();
    let dimension = CssComponentValue::try_dimension("125", "ms").unwrap();
    let number = CssComponentValue::try_number("3.5").unwrap();
    let mut items = components.items().to_vec();
    let duration = items
        .iter()
        .position(|v| {
            matches!(
                v.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
            )
        })
        .unwrap();
    let count = items
        .iter()
        .position(|v| {
            matches!(
                v.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Number(_))
            )
        })
        .unwrap();
    items[duration] = dimension.clone();
    items[count] = number.clone();
    let replacement = CssComponentValues::try_new(items, Default::default()).unwrap();
    let source = parse_property_value_for_grammar(
        P::Animation.grammar(),
        replacement.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let values = completed(&source);
    let actual = lists(&values);
    explicit_children(&actual, animation(&source));
    assert_eq!(
        actual.durations.values()[0].literal().unwrap().component(),
        &dimension
    );
    assert_eq!(
        actual.durations.values()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    let CssAnimationIterationCount::Number(count) = &actual.counts.values()[0] else {
        panic!("count")
    };
    assert_eq!(
        count.literal().map(|literal| literal.component()),
        Some(&number)
    );
    assert_eq!(count.origin(), &CssValueOrigin::Programmatic);
    assert!(matches!(
        actual.delays.values()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let pending = parsed("animation:var(--animation)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending")
    };
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("longhands")
    };
    let reentered = lists(&values);
    explicit_children(&reentered, animation(&source));
    for item in values.items() {
        assert!(item.source().same_occurrence(&pending));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(
            item.source().known().unwrap().grammar(),
            P::Animation.grammar()
        );
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn late_recovered_time_projections_keep_implicit_graphs_through_output_and_normalization() {
    for (css, duration) in [
        (".a{animation:fade 1s,slide calc((2s", true),
        (".a{animation:fade 1s,slide 2s calc((-250ms", false),
    ] {
        let report = parse_sheet(css);
        assert_implicit_closure(css, &report);
        let limits = CssNormalizationLimits::try_new(0, 1, 1, 8).unwrap();
        let normalized = normalize_report_with_limits(&report, limits).unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declaration = normalized
            .syntax()
            .items()
            .iter()
            .find_map(|item| {
                if let CssNormalizedItem::Declaration(v) = item {
                    Some(v)
                } else {
                    None
                }
            })
            .unwrap();
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("eight retained terminals")
        };
        let actual = lists(values);
        let original = animation(declaration.source());
        assert!(CssAnimationList::try_new(original.values().to_vec()).is_none());
        explicit_children(&actual, original);
        let time = if duration {
            assert!(CssDurationList::try_new(actual.durations.values().to_vec()).is_none());
            actual.durations.values()[1].clone().into_any_range()
        } else {
            assert!(CssDelayList::try_new(actual.delays.values().to_vec()).is_none());
            actual.delays.values()[1].clone()
        };
        let raw = time.calculation().unwrap().components().clone();
        implicit_origin(&raw, css);
        let CssComponentValueRef::Function(function) = raw.items()[0].view() else {
            panic!("original calc graph")
        };
        let CssValueOrigin::ImplicitClosure { at, .. } = function.closing_origin() else {
            panic!("original calc function remains implicitly closed")
        };
        assert_eq!(at.source().as_str(), css);
        assert_eq!(at.span().start().byte_offset().value(), css.len());
        assert_eq!(at.span().start(), at.span().end());
        assert_eq!(
            time.serialize_specified(Default::default()).unwrap(),
            if duration { "calc(2s)" } else { "calc(-0.25s)" }
        );
        assert_eq!(time.calculation().unwrap().components(), &raw);
        assert_eq!(
            validate_sheet(css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let before = report.clone();
        assert_eq!(
            normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 7).unwrap()
            )
            .unwrap_err()
            .exhausted_limit(),
            Some((CssNormalizationResource::Contributions, 7))
        );
        assert_eq!(report.diagnostics(), before.diagnostics());
        assert!(normalize_report_with_limits(&report, limits).is_ok());
    }
}

#[test]
fn pending_handles_retry_original_eof_grammar_residual_and_output_resource_failures() {
    for property in MEMBERS.into_iter().chain([P::Animation]) {
        let source = parsed(&format!(
            "{}:var(--animation)!important",
            property.canonical_name()
        ));
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let good = if property == P::Animation {
            "fade 1ms linear -1ms 2.5 reverse both paused"
        } else {
            INITIALS[MEMBERS.iter().position(|p| *p == property).unwrap()]
        };
        let recovered_source = format!("{good}/*");
        let recovered = parse_component_values(&recovered_source).unwrap();
        let origin = implicit_origin(&recovered, &recovered_source);
        let error = handle.reenter(recovered).unwrap_err();
        let Some(error) = error.replacement_error() else {
            panic!("strict recovered replacement")
        };
        assert!(error.code() == Some(CssErrorCode::UnexpectedEnd));
        assert_eq!(error.origin(), CssSerializedOrigin::End(Some(origin)));
        assert!(
            handle
                .reenter(parse_component_values("1px").unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
        assert!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
        let replacement = parse_component_values(good).unwrap();
        let CssContributions::Longhands(first) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed replacement")
        };
        for item in first.items() {
            let view = item.ordinary_value().unwrap().view();
            assert!(
                output(view, CssSerializationLimits::new(100, 100, 0))
                    .unwrap_err()
                    .is_byte_limit()
            );
        }
        let CssContributions::Longhands(retry) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("reusable pending handle")
        };
        for (a, b) in first.items().iter().zip(retry.items()) {
            assert_eq!(a.ordinary_value(), b.ordinary_value());
            assert!(b.source().same_occurrence(&source));
            assert_eq!(b.source().importance(), CssImportance::Important);
            assert_eq!(b.source().known().unwrap().grammar(), property.grammar());
            assert_eq!(b.replacement_components(), Some(&replacement));
            assert!(
                output(
                    b.ordinary_value().unwrap().view(),
                    CssSerializationLimits::default()
                )
                .is_ok()
            );
        }
    }
}

#[test]
fn projected_eight_lists_use_exact_cumulative_provider_budgets_and_fail_atomically() {
    let source = checked(
        P::Animation,
        "fade 1ms linear -1ms 2.5 reverse both paused, slide 2ms ease -2ms 1 normal none running",
    );
    let values = completed(&source);
    let actual = lists(&values);
    explicit_children(&actual, animation(&source));
    for (item, (nodes, expected)) in values.items().iter().zip([
        (3, "0.001s, 0.002s"),
        (3, "linear, ease"),
        (3, "-0.001s, -0.002s"),
        (5, "2.5, 1"),
        (3, "reverse, normal"),
        (3, "both, none"),
        (3, "paused, running"),
        (3, "fade, slide"),
    ]) {
        let view = item.ordinary_value().unwrap().view();
        let limits = CssSerializationLimits::new;
        assert_eq!(
            output(view, limits(nodes, nodes, expected.len())).unwrap(),
            expected
        );
        for (limit, kind) in [
            (
                limits(nodes - 1, nodes, expected.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                limits(nodes, nodes - 1, expected.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                limits(nodes, nodes, expected.len() - 1),
                SerializationFailure::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&output(view, limit).unwrap_err()),
                kind
            );
            assert_eq!(
                output(view, limits(nodes, nodes, expected.len())).unwrap(),
                expected
            );
        }
    }
    explicit_children(&actual, animation(&source));
}

#[test]
fn each_animation_role_rejects_intrinsic_grammar_negatives_through_all_fronts() {
    for (property, invalid) in [
        (
            P::AnimationName,
            vec![
                ("", End),
                ("1s", At("1s")),
                ("fade slide", At("slide")),
                ("fade,", At(",")),
                ("fade,,slide", At(",")),
                ("initial, fade", At(",")),
                ("default", At("default")),
            ],
        ),
        (
            P::AnimationDuration,
            vec![
                ("", End),
                ("-1s", At("-1s")),
                ("-1e-999s", At("-1e-999s")),
                ("0", At("0")),
                ("1px", At("1px")),
                ("1s,", At(",")),
                ("inherit, 1s", At(",")),
                ("calc(1px)", At("calc(")),
                ("1s 2s", At("2s")),
            ],
        ),
        (
            P::AnimationDelay,
            vec![
                ("", End),
                ("0", At("0")),
                ("1px", At("1px")),
                ("1s,", At(",")),
                ("1s,,2s", At(",")),
                ("unset, 1s", At(",")),
                ("calc(1px)", At("calc(")),
                ("1s 2s", At("2s")),
            ],
        ),
        (
            P::AnimationTimingFunction,
            vec![
                ("", End),
                ("1s", At("1s")),
                ("ease,", At(",")),
                ("ease ease-in", At("ease-in")),
                ("cubic-bezier(-1,0,1,1)", At("-1")),
                ("steps(1,jump-none)", At("jump-none")),
                ("revert, ease", At(",")),
            ],
        ),
        (
            P::AnimationIterationCount,
            vec![
                ("", End),
                ("-1", At("-1")),
                ("-1e-999", At("-1e-999")),
                ("1px", At("1px")),
                ("1 2", At("2")),
                ("1,", At(",")),
                ("infinite,,2", At(",")),
                ("initial, 1", At(",")),
                ("calc(1s)", At("calc(")),
            ],
        ),
        (
            P::AnimationDirection,
            vec![
                ("", End),
                ("sideways", At("sideways")),
                ("normal reverse", At("reverse")),
                ("normal,", At(",")),
                ("normal,,reverse", At(",")),
                ("inherit, normal", At(",")),
            ],
        ),
        (
            P::AnimationFillMode,
            vec![
                ("", End),
                ("filled", At("filled")),
                ("none both", At("both")),
                ("both,", At(",")),
                ("both,,none", At(",")),
                ("unset, none", At(",")),
            ],
        ),
        (
            P::AnimationPlayState,
            vec![
                ("", End),
                ("stopped", At("stopped")),
                ("paused running", At("running")),
                ("paused,", At(",")),
                ("running,,paused", At(",")),
                ("revert-layer, running", At(",")),
            ],
        ),
        (
            P::Animation,
            vec![
                ("", End),
                ("fade -1s", At("-1s")),
                ("fade 1s 2s 3s", At("3s")),
                ("fade slide 1s", At("slide")),
                ("fade 1s ease linear", At("linear")),
                ("fade 1s 2 3", At("3")),
                ("fade normal reverse", At("reverse")),
                ("fade both forwards", At("forwards")),
                ("fade running paused", At("paused")),
                ("fade 1s,", At(",")),
                ("fade 1s,,slide 2s", At(",")),
                ("inherit, fade 1s", At(",")),
                ("fade calc(1px)", At("calc(")),
            ],
        ),
    ] {
        for (body, token) in invalid {
            let css = format!("{}:{body}", property.canonical_name());
            let report = assert_style_attribute_rejected(&css, property, token);
            assert!(
                report.syntax().is_empty(),
                "invalid declaration dropped: {css}"
            );
            assert_eq!(
                validate_style_attribute(&css).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            let components = parse_component_values(body).unwrap();
            let grammar = parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            let named = parse_property_value(
                CssPropertyNameRef::Known(property),
                components,
                CssImportance::Normal,
            )
            .unwrap_err();
            assert_eq!(grammar, named, "same original grammar: {css}");
            assert_checked_rejected(Err(grammar), property, token);
        }
    }
}

#[test]
fn quoted_names_and_keyword_priority_remain_independent_of_projection_defaults() {
    let source = parsed(
        "animation:\"\" 1s, \" \" 2s, none 3s, backwards 4s, normal 5s reverse, 2 6s infinite, paused 7s running, \"none\" 8s",
    );
    let values = completed(&source);
    let actual = lists(&values);
    explicit_children(&actual, animation(&source));
    assert_eq!(
        actual
            .names
            .serialize_specified(Default::default())
            .unwrap(),
        "\"\", \" \", none, none, reverse, infinite, running, \"none\""
    );
    assert_eq!(actual.fills.modes()[2], CssAnimationFillMode::None);
    assert_eq!(actual.fills.modes()[3], CssAnimationFillMode::Backwards);
    assert_eq!(
        actual.directions.directions()[4],
        CssAnimationDirection::Normal
    );
    let CssAnimationIterationCount::Number(count) = &actual.counts.values()[5] else {
        panic!("explicit count before colliding name")
    };
    assert_eq!(count.serialize_specified(Default::default()).unwrap(), "2");
    assert_eq!(actual.plays.states()[6], CssAnimationPlayState::Paused);
    let CssKnownPropertyValueRef::Animation(original) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("animation")
    };
    assert_eq!(
        original
            .animations()
            .serialize_specified(Default::default())
            .unwrap(),
        "1s \"\", 2s \" \", 3s none, 4s backwards, 5s normal reverse, 6s 2 infinite, 7s paused running, 8s \"none\""
    );
    let name = CssAnimationNameList::try_new(vec![CssAnimationName::String(
        CssKeyframesString::new("\0"),
    )])
    .unwrap();
    assert!(
        name.serialize_specified(Default::default())
            .unwrap_err()
            .is_unrepresentable_value()
    );
}
