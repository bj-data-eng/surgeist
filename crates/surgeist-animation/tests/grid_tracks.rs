#![forbid(unsafe_code)]

use surgeist_animation::computed::{
    ComputedDimension, ComputedNumeric, DimensionKind, DimensionPercentage,
};
use surgeist_animation::easing::Easing;
use surgeist_animation::grid::{
    FixedRepeat, GridErrorKind, GridValue, ImplicitTrackList, LineNames, TemplateTrackList,
    TrackBreadth, TrackSection, TrackSize,
};
use surgeist_animation::interpolation::{
    DiscreteValue, InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationPair,
    InterpolationProgress, PropertyKey,
};
use surgeist_animation::keyframe::{
    KeyframeAnimationId, KeyframeSegment, KeyframeTiming, KeyframeTrack,
};
use surgeist_animation::progress::UnitRatio;
use surgeist_animation::sample::{SampleClassification, SampleValue};
use surgeist_animation::time::{AnimationDelay, AnimationDuration, ElapsedTime};
use surgeist_animation::timing::{
    AnimationDirection, AnimationPlayState, FillMode, IterationCount,
};

#[test]
fn lists_and_repeats_reject_empty_or_zero_shapes() {
    assert_eq!(
        TemplateTrackList::new(names(&[]), vec![])
            .unwrap_err()
            .kind(),
        GridErrorKind::EmptyList
    );
    assert_eq!(
        ImplicitTrackList::new(vec![]).unwrap_err().kind(),
        GridErrorKind::EmptyList
    );
    assert_eq!(
        FixedRepeat::new(0, names(&[]), vec![(length(10.0), names(&[]))])
            .unwrap_err()
            .kind(),
        GridErrorKind::ZeroRepeatCount
    );
    assert_eq!(
        FixedRepeat::new(2, names(&[]), vec![]).unwrap_err().kind(),
        GridErrorKind::EmptyList
    );
}

#[test]
fn numeric_track_roles_reject_wrong_units_and_flexible_minima() {
    let angle =
        ComputedNumeric::dimension(ComputedDimension::new(DimensionKind::Angle, 30.0).unwrap());
    assert_eq!(
        TrackBreadth::length_percentage(angle.clone())
            .unwrap_err()
            .kind(),
        GridErrorKind::InvalidNumericType
    );
    assert_eq!(
        TrackSize::fit_content(angle).unwrap_err().kind(),
        GridErrorKind::InvalidNumericType
    );
    assert_eq!(
        TrackSize::minmax(flex_breadth(1.0), flex_breadth(2.0))
            .unwrap_err()
            .kind(),
        GridErrorKind::FlexibleMinimum
    );
    assert_eq!(
        TrackBreadth::flex(f64::INFINITY).unwrap_err().kind(),
        GridErrorKind::Numeric
    );
    assert_eq!(
        TrackBreadth::flex(f64::NAN).unwrap_err().kind(),
        GridErrorKind::Numeric
    );
}

#[test]
fn template_rows_and_columns_keep_line_names_and_sample_each_track_locally() {
    let from = template(
        &["first"],
        vec![
            (single(length(8.0)), names(&["a", "b"])),
            (single(flex(1.0)), names(&["end"])),
        ],
    );
    let to = template(
        &["first"],
        vec![
            (single(length(24.0)), names(&["c"])),
            (single(plain(TrackBreadth::auto())), names(&[])),
        ],
    );
    let early = template(
        &["first"],
        vec![
            (single(length(12.0)), names(&["a", "b"])),
            (single(flex(1.0)), names(&["end"])),
        ],
    );
    let tie = template(
        &["first"],
        vec![
            (single(length(16.0)), names(&["c"])),
            (single(plain(TrackBreadth::auto())), names(&[])),
        ],
    );
    for property in ["grid-template-rows", "grid-template-columns"] {
        let track = track(property, from.clone(), to.clone());
        assert_sample(&track, 0.25, early.clone());
        assert_sample(&track, 0.5, tie.clone());
    }
}

#[test]
fn implicit_rows_and_columns_keep_length_percentage_terms_without_a_basis() {
    let from = implicit(vec![length(12.0), flex(0.5)]);
    let to = implicit(vec![percentage(0.5), flex(1.5)]);
    for property in ["grid-auto-rows", "grid-auto-columns"] {
        let track = track(property, from.clone(), to.clone());
        assert_sample(&track, 0.25, implicit(vec![mixed(9.0, 0.125), flex(0.75)]));
    }
}

#[test]
fn explicit_minmax_bounds_switch_independently_and_preserve_function_tags() {
    let from = implicit(vec![
        minmax(10.0, flex_breadth(1.0)),
        length(10.0),
        fit(20.0),
    ]);
    let to = implicit(vec![
        minmax(30.0, TrackBreadth::auto()),
        minmax(50.0, flex_breadth(2.0)),
        plain(TrackBreadth::max_content()),
    ]);
    let track = track("grid-auto-rows", from, to);
    assert_sample(
        &track,
        0.25,
        implicit(vec![
            minmax(15.0, flex_breadth(1.0)),
            length(10.0),
            fit(20.0),
        ]),
    );
    assert_sample(
        &track,
        0.5,
        implicit(vec![
            minmax(20.0, TrackBreadth::auto()),
            minmax(50.0, flex_breadth(2.0)),
            plain(TrackBreadth::max_content()),
        ]),
    );
}

#[test]
fn corresponding_minmax_and_fit_content_numbers_interpolate() {
    let track = track(
        "grid-template-columns",
        list(vec![minmax(8.0, flex_breadth(1.0)), fit(12.0)]),
        list(vec![minmax(24.0, flex_breadth(3.0)), fit(28.0)]),
    );
    assert_sample(
        &track,
        0.5,
        list(vec![minmax(16.0, flex_breadth(2.0)), fit(20.0)]),
    );
}

#[test]
fn matching_fixed_repeats_preserve_counts_names_and_local_keyword_fallback() {
    let from = list_sections(vec![
        single(flex(1.0)),
        repeated(
            3,
            vec![
                (length(8.0), names(&["old"])),
                (plain(TrackBreadth::auto()), names(&[])),
            ],
        ),
        single(length(20.0)),
    ]);
    let to = list_sections(vec![
        single(flex(3.0)),
        repeated(
            3,
            vec![(length(24.0), names(&["new"])), (length(40.0), names(&[]))],
        ),
        single(length(60.0)),
    ]);
    let track = track("grid-template-columns", from, to);
    assert_sample(
        &track,
        0.25,
        list_sections(vec![
            single(flex(1.5)),
            repeated(
                3,
                vec![
                    (length(12.0), names(&["old"])),
                    (plain(TrackBreadth::auto()), names(&[])),
                ],
            ),
            single(length(30.0)),
        ]),
    );
    assert_sample(
        &track,
        0.5,
        list_sections(vec![
            single(flex(2.0)),
            repeated(
                3,
                vec![(length(16.0), names(&["new"])), (length(40.0), names(&[]))],
            ),
            single(length(40.0)),
        ]),
    );
}

#[test]
fn incompatible_repeat_structure_switches_the_entire_template() {
    let from = list_sections(vec![
        single(length(10.0)),
        repeated(2, vec![(length(20.0), names(&[]))]),
    ]);
    for to in [
        list_sections(vec![
            single(length(30.0)),
            repeated(3, vec![(length(40.0), names(&[]))]),
        ]),
        list_sections(vec![
            single(length(30.0)),
            repeated(
                2,
                vec![(length(40.0), names(&[])), (length(50.0), names(&[]))],
            ),
        ]),
        list(vec![length(30.0), length(40.0)]),
    ] {
        let track = track("grid-template-columns", from.clone(), to.clone());
        assert_sample(&track, 0.5_f64.next_down(), from.clone());
        assert_sample(&track, 0.5, to);
    }
}

#[test]
fn unequal_outer_lists_and_none_switch_without_repeating_or_padding() {
    for (from, to) in [
        (
            list(vec![length(10.0), length(20.0)]),
            list(vec![length(30.0)]),
        ),
        (
            implicit(vec![length(10.0), length(20.0)]),
            implicit(vec![length(30.0)]),
        ),
        (GridValue::None, list(vec![length(30.0)])),
    ] {
        let track = track("grid-template-rows", from.clone(), to.clone());
        assert_sample(&track, 0.5_f64.next_down(), from);
        assert_sample(&track, 0.5, to);
    }
}

#[test]
fn finite_extrapolation_retains_signed_intermediates_for_later_range_handling() {
    let pair = pair(
        list(vec![length(10.0), flex(1.0)]),
        list(vec![length(30.0), flex(3.0)]),
    );
    assert_eq!(
        pair.sample(progress(-1.0)),
        InterpolationOutcome::Value(wrapped(list(vec![length(-10.0), flex(-1.0)])))
    );
    assert_eq!(
        pair.sample(progress(2.0)),
        InterpolationOutcome::Value(wrapped(list(vec![length(50.0), flex(5.0)])))
    );
}

#[test]
fn numeric_failure_retains_complete_endpoints_and_recovers_at_a_valid_progress() {
    let from = list(vec![length(1.0), length(f64::MAX)]);
    let to = list(vec![length(2.0), length(f64::MAX / 2.0)]);
    let pair = pair(from.clone(), to.clone());
    let InterpolationOutcome::GridFailure(error) = pair.sample(progress(-2.0)) else {
        panic!("expected typed Grid failure");
    };
    assert_eq!(error.property().as_str(), "grid-template-columns");
    assert_eq!(error.progress(), progress(-2.0));
    assert_eq!(error.from(), &wrapped(from.clone()));
    assert_eq!(error.to(), &wrapped(to));
    assert_eq!(error.error().kind(), GridErrorKind::Numeric);
    assert_eq!(error.error().section_index(), Some(1));
    assert!(std::error::Error::source(error.error()).is_some());
    assert_eq!(
        pair.sample(progress(0.0)),
        InterpolationOutcome::Value(wrapped(from))
    );
}

#[test]
fn grid_areas_auto_flow_and_expanded_placement_longhands_are_discrete() {
    let entries = [
        ("grid-template-areas", "\"header header\"", "\"main aside\""),
        ("grid-auto-flow", "row", "column dense"),
        ("grid-row-start", "1", "4"),
        ("grid-row-end", "span 2", "span 5"),
        ("grid-column-start", "main", "aside"),
        ("grid-column-end", "auto", "footer"),
    ];
    for (property, from, to) in entries {
        let track = raw_track(property, keyword(from), keyword(to));
        let before = track.sample(
            ElapsedTime::from_secs(0.5_f64.next_down()).unwrap(),
            AnimationPlayState::Running,
        );
        let at = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        assert!(matches!(before[0].value(), SampleValue::Value(v) if v.value() == &keyword(from)));
        assert!(matches!(at[0].value(), SampleValue::Value(v) if v.value() == &keyword(to)));
    }
}

#[test]
fn repeat_errors_identify_both_indices_and_the_bound_without_publishing_a_prefix() {
    use surgeist_animation::grid::GridComponent;
    let from = list_sections(vec![
        single(length(1.0)),
        repeated(
            2,
            vec![
                (length(2.0), names(&[])),
                (minmax(3.0, flex_breadth(f64::MAX)), names(&[])),
            ],
        ),
    ]);
    let to = list_sections(vec![
        single(length(4.0)),
        repeated(
            2,
            vec![
                (length(5.0), names(&[])),
                (minmax(6.0, flex_breadth(f64::MAX / 2.0)), names(&[])),
            ],
        ),
    ]);
    let InterpolationOutcome::GridFailure(error) = pair(from, to).sample(progress(-2.0)) else {
        panic!("expected repeat arithmetic failure");
    };
    assert_eq!(error.error().section_index(), Some(1));
    assert_eq!(error.error().repeat_index(), Some(1));
    assert_eq!(error.error().component(), Some(GridComponent::Maximum));
}

#[test]
fn all_intrinsic_branches_and_length_to_flex_switch_locally() {
    let from = implicit(vec![
        plain(TrackBreadth::min_content()),
        plain(TrackBreadth::auto()),
        length(4.0),
        length(8.0),
    ]);
    let to = implicit(vec![
        plain(TrackBreadth::max_content()),
        plain(TrackBreadth::min_content()),
        flex(2.0),
        length(24.0),
    ]);
    let track = track("grid-auto-columns", from, to);
    assert_sample(
        &track,
        0.25,
        implicit(vec![
            plain(TrackBreadth::min_content()),
            plain(TrackBreadth::auto()),
            length(4.0),
            length(12.0),
        ]),
    );
    assert_sample(
        &track,
        0.5,
        implicit(vec![
            plain(TrackBreadth::max_content()),
            plain(TrackBreadth::min_content()),
            flex(2.0),
            length(16.0),
        ]),
    );
}

#[test]
fn repeat_leading_and_adjacent_line_name_groups_keep_their_positions() {
    let value = |name: &str, px| {
        template(
            &["CaseSensitive"],
            vec![(
                TrackSection::Repeat(
                    FixedRepeat::new(2, names(&[name]), vec![(length(px), names(&["inside"]))])
                        .unwrap(),
                ),
                names(&["outside"]),
            )],
        )
    };
    let track = track(
        "grid-template-rows",
        value("start", 8.0),
        value("end", 24.0),
    );
    assert_sample(&track, 0.25, value("start", 12.0));
    assert_sample(&track, 0.5, value("end", 16.0));
}

#[test]
fn public_keyframe_failure_is_sample_local_and_recovers_at_an_endpoint() {
    use surgeist_animation::easing::{LinearControlPointInput, LinearEasing};
    let from = list(vec![length(0.0)]);
    let to = list(vec![length(f64::MAX)]);
    let easing = Easing::linear_function(
        LinearEasing::new(vec![
            LinearControlPointInput {
                input: 0.0,
                output: 0.0,
            },
            LinearControlPointInput {
                input: 0.5,
                output: 2.0,
            },
            LinearControlPointInput {
                input: 1.0,
                output: 1.0,
            },
        ])
        .unwrap(),
    );
    let timing = KeyframeTiming::try_new(
        AnimationDelay::from_secs(0.0).unwrap(),
        AnimationDuration::from_secs(1.0).unwrap(),
        IterationCount::finite(1.0).unwrap(),
        AnimationDirection::Normal,
        FillMode::Both,
    )
    .unwrap();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("overflow").unwrap(),
        timing,
        vec![
            KeyframeSegment::new(
                PropertyKey::new("grid-template-columns").unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                wrapped(from),
                wrapped(to.clone()),
                easing,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let frame = track.sample(
        ElapsedTime::from_secs(0.5).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(frame[0].classification(), SampleClassification::Failed);
    assert!(
        matches!(frame[0].value(), SampleValue::Error(InterpolationError::Grid(error)) if error.error().section_index() == Some(0))
    );
    let endpoint = track.sample(
        ElapsedTime::from_secs(1.0).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(endpoint[0].classification(), SampleClassification::Filling);
    assert!(
        matches!(endpoint[0].value(), SampleValue::Value(value) if value.value() == &wrapped(to))
    );
}

#[test]
fn symbolic_length_sources_keep_their_context_and_need_no_used_basis() {
    use surgeist_animation::computed::{MathContext, MathNode, NumericType};
    let context = MathContext::new();
    let source = context.source(NumericType::DimensionPercentage(DimensionKind::Length));
    let from = implicit(vec![plain(
        TrackBreadth::length_percentage(ComputedNumeric::source(source.clone())).unwrap(),
    )]);
    let result = from
        .interpolate(&implicit(vec![percentage(0.5)]), progress(0.25))
        .unwrap();
    let GridValue::Implicit(list) = result else {
        panic!("expected implicit list")
    };
    let numeric = list.tracks()[0].as_breadth().unwrap().numeric().unwrap();
    assert!(numeric.math_nodes().unwrap().iter().any(|node| matches!(node, MathNode::Source(retained) if retained == &source && retained.context() == &context)));
}

#[test]
fn supplied_shorthand_constituents_sample_together_without_parsing_a_shorthand() {
    // CSS/Style supplies the expanded grid/grid-template and placement longhands.
    let constituents = [
        (
            "grid-template-rows",
            wrapped(list(vec![length(8.0)])),
            wrapped(list(vec![length(24.0)])),
        ),
        (
            "grid-template-columns",
            wrapped(list(vec![flex(1.0)])),
            wrapped(list(vec![flex(3.0)])),
        ),
        (
            "grid-auto-rows",
            wrapped(implicit(vec![length(4.0)])),
            wrapped(implicit(vec![length(12.0)])),
        ),
        (
            "grid-auto-columns",
            wrapped(implicit(vec![flex(2.0)])),
            wrapped(implicit(vec![flex(4.0)])),
        ),
        (
            "grid-template-areas",
            keyword("\"a a\""),
            keyword("\"b c\""),
        ),
        ("grid-auto-flow", keyword("row"), keyword("column dense")),
        ("grid-row-start", keyword("1"), keyword("3")),
        ("grid-row-end", keyword("span 2"), keyword("span 4")),
        ("grid-column-start", keyword("left"), keyword("right")),
        ("grid-column-end", keyword("auto"), keyword("end")),
    ];
    let segments = constituents
        .iter()
        .map(|(name, from, to)| {
            KeyframeSegment::new(
                PropertyKey::new(*name).unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                from.clone(),
                to.clone(),
                Easing::linear(),
            )
            .unwrap()
        })
        .collect();
    let track = KeyframeTrack::new(
        KeyframeAnimationId::new("expanded-grid").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        segments,
    )
    .unwrap();
    let frame = track.sample(
        ElapsedTime::from_secs(0.5).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(frame.len(), 10);
    for sample in &frame {
        assert_eq!(sample.classification(), SampleClassification::Active);
    }
    for (index, expected) in [
        wrapped(list(vec![length(16.0)])),
        wrapped(list(vec![flex(2.0)])),
        wrapped(implicit(vec![length(8.0)])),
        wrapped(implicit(vec![flex(3.0)])),
    ]
    .iter()
    .enumerate()
    {
        assert!(
            matches!(frame[index].value(), SampleValue::Value(value) if value.value() == expected)
        );
    }
    for (sample, (_, _, expected)) in frame.iter().skip(4).zip(constituents.iter().skip(4)) {
        assert!(matches!(sample.value(), SampleValue::Value(value) if value.value() == expected));
    }
}

#[test]
fn native_running_transition_returns_grid_errors_and_retains_its_snapshot() {
    use surgeist_animation::easing::{LinearControlPointInput, LinearEasing};
    use surgeist_animation::time::OriginRelativeTime;
    use surgeist_animation::transition::{
        TransitionChange, TransitionKey, TransitionPropertyPolicy, TransitionSnapshot,
        TransitionState, TransitionTiming, TransitionTrackId, Transitionability,
    };
    // Applicability is a supplied consumer policy for these numeric-only pairs.
    struct NumericGridPolicy;
    impl TransitionPropertyPolicy for NumericGridPolicy {
        fn transitionability(
            &self,
            _: &PropertyKey,
            _: &InterpolableValue,
            _: &InterpolableValue,
        ) -> Transitionability {
            Transitionability::Continuous
        }
        fn interpolate(
            &self,
            property: &PropertyKey,
            from: &InterpolableValue,
            to: &InterpolableValue,
            progress: InterpolationProgress,
        ) -> InterpolationOutcome {
            match InterpolationPair::new(property.clone(), from.clone(), to.clone()) {
                Ok(pair) => pair.sample(progress),
                Err(error) => InterpolationOutcome::Unsupported(error),
            }
        }
    }
    let key = TransitionKey::new(1_u64, PropertyKey::new("grid-template-columns").unwrap());
    let before = wrapped(list(vec![length(0.0)]));
    let change = TransitionChange {
        key: key.clone(),
        before: before.clone(),
        after: wrapped(list(vec![length(f64::MAX)])),
        eligible: true,
        matching: Some(TransitionTiming::new(
            AnimationDuration::from_secs(1.0).unwrap(),
            AnimationDelay::from_secs(0.0).unwrap(),
            Easing::linear_function(
                LinearEasing::new(vec![
                    LinearControlPointInput {
                        input: 0.0,
                        output: 0.0,
                    },
                    LinearControlPointInput {
                        input: 0.5,
                        output: 2.0,
                    },
                    LinearControlPointInput {
                        input: 1.0,
                        output: 1.0,
                    },
                ])
                .unwrap(),
            ),
        )),
        new_id: TransitionTrackId::new(1),
    };
    let update = TransitionSnapshot::default()
        .update(
            Some(OriginRelativeTime::from_millis(0.0).unwrap()),
            &[change],
            &NumericGridPolicy,
        )
        .unwrap()
        .unwrap();
    let Some(TransitionState::Running(run)) = update.state().get(&key) else {
        panic!("expected running transition")
    };
    let error = run
        .sample(
            Some(OriginRelativeTime::from_millis(500.0).unwrap()),
            &NumericGridPolicy,
        )
        .unwrap_err();
    assert!(
        matches!(error.interpolation_error(), Some(InterpolationError::Grid(failure)) if failure.error().section_index() == Some(0))
    );
    assert_eq!(
        run.sample(
            Some(OriginRelativeTime::from_millis(0.0).unwrap()),
            &NumericGridPolicy
        )
        .unwrap()
        .value(),
        Some(&before)
    );
}

#[test]
fn automatic_repeats_switch_complete_templates_even_for_the_same_keyword() {
    use surgeist_animation::grid::AutomaticRepeatKind::{AutoFill, AutoFit};
    for from_kind in [AutoFill, AutoFit] {
        for to_kind in [AutoFill, AutoFit] {
            let from = automatic_template(from_kind, 8.0, 10.0);
            let to = automatic_template(to_kind, 24.0, 30.0);
            let track = track("grid-template-columns", from.clone(), to.clone());
            assert_sample(&track, 0.5_f64.next_down(), from.clone());
            assert_sample(&track, 0.5, to.clone());
            assert_eq!(
                pair(from.clone(), to.clone()).sample(progress(-2.0)),
                InterpolationOutcome::Value(wrapped(from))
            );
            assert_eq!(
                pair(to.clone(), GridValue::None).sample(progress(0.25)),
                InterpolationOutcome::Value(wrapped(to))
            );
        }
    }
}

#[test]
fn automatic_repeat_guards_fixed_size_bodies_and_the_complete_template() {
    use surgeist_animation::grid::{AutomaticRepeat, AutomaticRepeatKind::AutoFill};
    assert_eq!(
        AutomaticRepeat::new(AutoFill, names(&[]), vec![])
            .unwrap_err()
            .kind(),
        GridErrorKind::EmptyList
    );
    for invalid in [flex(1.0), plain(TrackBreadth::auto()), fit(10.0)] {
        assert_eq!(
            AutomaticRepeat::new(AutoFill, names(&[]), vec![(invalid.clone(), names(&[]))])
                .unwrap_err()
                .kind(),
            GridErrorKind::InvalidAutomaticTrack
        );
        let error = TemplateTrackList::new(
            names(&[]),
            vec![
                (single(invalid), names(&[])),
                (automatic(AutoFill, 10.0), names(&[])),
            ],
        )
        .unwrap_err();
        assert_eq!(error.kind(), GridErrorKind::InvalidAutomaticTrack);
        assert_eq!(error.section_index(), Some(0));
    }
    let error = TemplateTrackList::new(
        names(&[]),
        vec![
            (repeated(2, vec![(flex(1.0), names(&[]))]), names(&[])),
            (automatic(AutoFill, 10.0), names(&[])),
        ],
    )
    .unwrap_err();
    assert_eq!(error.kind(), GridErrorKind::InvalidAutomaticTrack);
    assert_eq!(error.repeat_index(), Some(0));
    let error = TemplateTrackList::new(
        names(&[]),
        vec![
            (automatic(AutoFill, 10.0), names(&[])),
            (automatic(AutoFill, 20.0), names(&[])),
        ],
    )
    .unwrap_err();
    assert_eq!(error.kind(), GridErrorKind::MultipleAutomaticRepeats);
}

#[test]
fn automatic_fixed_size_alternatives_retain_names_and_do_not_resolve_counts() {
    use surgeist_animation::grid::{AutomaticRepeat, AutomaticRepeatKind::AutoFit};
    let fixed_max = TrackSize::minmax(
        TrackBreadth::min_content(),
        TrackBreadth::length_percentage(lp(20.0)).unwrap(),
    )
    .unwrap();
    let body = vec![
        (percentage(0.25), names(&["percent"])),
        (minmax(8.0, flex_breadth(1.0)), names(&["minimum"])),
        (fixed_max, names(&["maximum"])),
    ];
    let repeat = AutomaticRepeat::new(AutoFit, names(&["leading"]), body.clone()).unwrap();
    assert_eq!(repeat.kind(), AutoFit);
    assert_eq!(repeat.leading_names(), &names(&["leading"]));
    assert_eq!(repeat.tracks(), body);
    let value = template(
        &["outer"],
        vec![
            (
                repeated(usize::MAX, vec![(length(4.0), names(&[]))]),
                names(&[]),
            ),
            (TrackSection::AutomaticRepeat(repeat), names(&["trailing"])),
        ],
    );
    assert_eq!(
        value.interpolate(&GridValue::None, progress(0.25)).unwrap(),
        value
    );
}

#[test]
fn subgrid_preserves_anchors_fixed_and_automatic_name_repeats_as_a_distinct_form() {
    use surgeist_animation::grid::{NameRepeat, NameRepeatCount, SubgridSection, SubgridTrackList};
    let fixed = NameRepeat::new(
        NameRepeatCount::Fixed(usize::MAX),
        vec![names(&[]), names(&["a", "b"])],
    )
    .unwrap();
    assert_eq!(fixed.count(), NameRepeatCount::Fixed(usize::MAX));
    assert_eq!(fixed.groups(), &[names(&[]), names(&["a", "b"])]);
    let sections = vec![
        SubgridSection::Names(names(&["anchor"])),
        SubgridSection::Repeat(fixed),
        SubgridSection::Repeat(
            NameRepeat::new(NameRepeatCount::AutoFill, vec![names(&["fill"])]).unwrap(),
        ),
        SubgridSection::Names(names(&[])),
    ];
    let subgrid = SubgridTrackList::new(sections.clone()).unwrap();
    assert_eq!(subgrid.sections(), sections);
    let from = GridValue::Subgrid(subgrid);
    let bare = GridValue::Subgrid(SubgridTrackList::new(vec![]).unwrap());
    for to in [bare, list(vec![length(30.0)]), GridValue::None] {
        let track = track("grid-template-rows", from.clone(), to.clone());
        assert_sample(&track, 0.25, from.clone());
        assert_sample(&track, 0.5, to);
    }
}

#[test]
fn name_repeat_guards_positive_counts_nonempty_groups_and_one_auto_fill() {
    use surgeist_animation::grid::{NameRepeat, NameRepeatCount, SubgridSection, SubgridTrackList};
    assert_eq!(
        NameRepeat::new(NameRepeatCount::Fixed(0), vec![names(&[])])
            .unwrap_err()
            .kind(),
        GridErrorKind::ZeroRepeatCount
    );
    assert_eq!(
        NameRepeat::new(NameRepeatCount::AutoFill, vec![])
            .unwrap_err()
            .kind(),
        GridErrorKind::EmptyList
    );
    let automatic = NameRepeat::new(NameRepeatCount::AutoFill, vec![names(&[])]).unwrap();
    assert_eq!(
        SubgridTrackList::new(vec![
            SubgridSection::Repeat(automatic.clone()),
            SubgridSection::Names(names(&["between"])),
            SubgridSection::Repeat(automatic)
        ])
        .unwrap_err()
        .kind(),
        GridErrorKind::MultipleAutomaticRepeats
    );
}

#[test]
fn grid_addition_and_default_accumulation_keep_independent_numeric_terms() {
    let underlying = list(vec![length(40.0), percentage(0.125), flex(0.5)]);
    let effect = list(vec![length(80.0), length(60.0), flex(1.5)]);
    let expected = list(vec![length(120.0), mixed(60.0, 0.125), flex(2.0)]);
    assert_eq!(underlying.add(&effect).unwrap(), expected);
    assert_eq!(underlying.accumulate(&effect).unwrap(), expected);
    assert_eq!(
        underlying,
        list(vec![length(40.0), percentage(0.125), flex(0.5)])
    );
    assert_eq!(effect, list(vec![length(80.0), length(60.0), flex(1.5)]));
}

#[test]
fn grid_composition_is_local_for_ordinary_items_and_explicit_minmax_bounds() {
    let underlying = implicit(vec![
        length(8.0),
        flex(1.0),
        minmax(6.0, flex_breadth(1.0)),
        fit(5.0),
        length(2.0),
    ]);
    let effect = implicit(vec![
        length(24.0),
        plain(TrackBreadth::auto()),
        minmax(10.0, TrackBreadth::auto()),
        fit(15.0),
        minmax(7.0, flex_breadth(2.0)),
    ]);
    let expected = implicit(vec![
        length(32.0),
        plain(TrackBreadth::auto()),
        minmax(16.0, TrackBreadth::auto()),
        fit(20.0),
        minmax(7.0, flex_breadth(2.0)),
    ]);
    assert_eq!(underlying.add(&effect).unwrap(), expected);
    assert_eq!(underlying.accumulate(&effect).unwrap(), expected);
}

#[test]
fn fixed_repeat_composition_keeps_count_and_takes_all_effect_line_groups() {
    let value = |px, factor, name: &str| {
        template(
            &[name],
            vec![(
                TrackSection::Repeat(
                    FixedRepeat::new(
                        3,
                        names(&[name]),
                        vec![(length(px), names(&[name])), (flex(factor), names(&[]))],
                    )
                    .unwrap(),
                ),
                names(&[name]),
            )],
        )
    };
    assert_eq!(
        value(8.0, 1.0, "base")
            .add(&value(24.0, 2.0, "effect"))
            .unwrap(),
        value(32.0, 3.0, "effect")
    );
    assert_eq!(
        value(8.0, 1.0, "base")
            .accumulate(&value(24.0, 2.0, "effect"))
            .unwrap(),
        value(32.0, 3.0, "effect")
    );
}

#[test]
fn incompatible_composition_selects_complete_effect_before_numeric_work() {
    use surgeist_animation::grid::{AutomaticRepeatKind::AutoFill, SubgridTrackList};
    let underlying = list(vec![length(f64::MAX), length(10.0)]);
    for effect in [
        list(vec![length(f64::MAX)]),
        GridValue::None,
        automatic_template(AutoFill, 20.0, 30.0),
        GridValue::Subgrid(SubgridTrackList::new(vec![]).unwrap()),
        list_sections(vec![
            single(length(f64::MAX)),
            repeated(2, vec![(length(30.0), names(&[]))]),
        ]),
    ] {
        assert_eq!(underlying.add(&effect).unwrap(), effect);
        assert_eq!(underlying.accumulate(&effect).unwrap(), effect);
    }
    let auto = automatic_template(AutoFill, 10.0, 20.0);
    let effect = automatic_template(AutoFill, 30.0, 40.0);
    assert_eq!(auto.add(&effect).unwrap(), effect);
}

#[test]
fn grid_composition_errors_retain_operation_and_nested_numeric_position() {
    use surgeist_animation::computed::ComputedOperation;
    use surgeist_animation::grid::GridComponent;
    let value = list_sections(vec![
        single(length(1.0)),
        repeated(
            2,
            vec![
                (length(2.0), names(&[])),
                (minmax(3.0, flex_breadth(f64::MAX)), names(&[])),
            ],
        ),
    ]);
    for (result, operation) in [
        (value.add(&value), ComputedOperation::Add),
        (value.accumulate(&value), ComputedOperation::Accumulate),
    ] {
        let error = result.unwrap_err();
        assert_eq!(error.kind(), GridErrorKind::Numeric);
        assert_eq!(error.section_index(), Some(1));
        assert_eq!(error.repeat_index(), Some(1));
        assert_eq!(error.component(), Some(GridComponent::Maximum));
        assert_eq!(error.numeric_error().unwrap().operation(), Some(operation));
    }
    assert_eq!(value.interpolate(&value, progress(0.0)).unwrap(), value);
}

#[test]
fn grid_composition_retains_symbolic_sources_and_unclamped_intermediates() {
    use surgeist_animation::computed::{ComputedOperation, MathContext, MathNode, NumericType};
    let context = MathContext::new();
    let source = context.source(NumericType::DimensionPercentage(DimensionKind::Length));
    let underlying = implicit(vec![plain(
        TrackBreadth::length_percentage(ComputedNumeric::source(source.clone())).unwrap(),
    )]);
    for (result, expected_operation) in [
        (
            underlying.add(&implicit(vec![percentage(0.25)])),
            ComputedOperation::Add,
        ),
        (
            underlying.accumulate(&implicit(vec![percentage(0.25)])),
            ComputedOperation::Accumulate,
        ),
    ] {
        let GridValue::Implicit(list) = result.unwrap() else {
            panic!("expected implicit tracks")
        };
        let nodes = list.tracks()[0]
            .as_breadth()
            .unwrap()
            .numeric()
            .unwrap()
            .math_nodes()
            .unwrap();
        assert!(nodes.iter().any(|node| matches!(node, MathNode::Source(actual) if actual == &source && actual.context() == &context)));
        assert!(
            matches!(nodes.last(), Some(MathNode::Combine { operation, .. }) if *operation == expected_operation)
        );
    }
    let negative = list(vec![length(-10.0), flex(-1.0)]);
    assert_eq!(
        negative.add(&list(vec![length(6.0), flex(0.5)])).unwrap(),
        list(vec![length(-4.0), flex(-0.5)])
    );
}

#[test]
fn automatic_template_fallback_precedes_overflow_and_composition_selects_effect() {
    use surgeist_animation::grid::AutomaticRepeatKind::AutoFill;
    let from = automatic_template(AutoFill, f64::MAX, 10.0);
    let to = automatic_template(AutoFill, f64::MAX / 2.0, 30.0);
    assert_eq!(from.interpolate(&to, progress(-2.0)).unwrap(), from);
    assert_eq!(from.add(&from).unwrap(), from);
    assert_eq!(from.accumulate(&to).unwrap(), to);
}

#[test]
fn compose_then_sample_uses_real_grid_pairs_for_add_and_replace_endpoints() {
    let underlying = list(vec![length(40.0), percentage(0.125)]);
    let added_from = underlying
        .add(&list(vec![length(80.0), length(60.0)]))
        .unwrap();
    let added_to = underlying
        .accumulate(&list(vec![length(160.0), percentage(0.25)]))
        .unwrap();
    let sampled_track = track("grid-template-columns", added_from, added_to);
    assert_sample(
        &sampled_track,
        0.5,
        list(vec![length(160.0), mixed(30.0, 0.25)]),
    );
    let from = underlying
        .add(&list(vec![length(80.0), percentage(0.125)]))
        .unwrap();
    let replaced = list(vec![length(200.0), percentage(0.5)]);
    assert_sample(
        &track("grid-template-rows", from, replaced),
        0.5,
        list(vec![length(160.0), percentage(0.375)]),
    );
}

fn automatic(kind: surgeist_animation::grid::AutomaticRepeatKind, px: f64) -> TrackSection {
    use surgeist_animation::grid::AutomaticRepeat;
    TrackSection::AutomaticRepeat(
        AutomaticRepeat::new(
            kind,
            names(&["body-start"]),
            vec![(minmax(px, flex_breadth(1.0)), names(&["body-end"]))],
        )
        .unwrap(),
    )
}
fn automatic_template(
    kind: surgeist_animation::grid::AutomaticRepeatKind,
    before: f64,
    px: f64,
) -> GridValue {
    template(
        &["outer-start"],
        vec![
            (single(length(before)), names(&["before-auto"])),
            (automatic(kind, px), names(&["outer-end"])),
        ],
    )
}

fn lp(px: f64) -> ComputedNumeric {
    ComputedNumeric::dimension(ComputedDimension::new(DimensionKind::Length, px).unwrap())
}
fn plain(value: TrackBreadth) -> TrackSize {
    TrackSize::breadth(value)
}
fn length(px: f64) -> TrackSize {
    plain(TrackBreadth::length_percentage(lp(px)).unwrap())
}
fn percentage(ratio: f64) -> TrackSize {
    plain(TrackBreadth::length_percentage(ComputedNumeric::percentage(ratio).unwrap()).unwrap())
}
fn mixed(px: f64, ratio: f64) -> TrackSize {
    plain(
        TrackBreadth::length_percentage(ComputedNumeric::mix(
            DimensionPercentage::new(
                ComputedDimension::new(DimensionKind::Length, px).unwrap(),
                ratio,
            )
            .unwrap(),
        ))
        .unwrap(),
    )
}
fn flex_breadth(value: f64) -> TrackBreadth {
    TrackBreadth::flex(value).unwrap()
}
fn flex(value: f64) -> TrackSize {
    plain(flex_breadth(value))
}
fn fit(px: f64) -> TrackSize {
    TrackSize::fit_content(lp(px)).unwrap()
}
fn minmax(min: f64, max: TrackBreadth) -> TrackSize {
    TrackSize::minmax(TrackBreadth::length_percentage(lp(min)).unwrap(), max).unwrap()
}
fn names(values: &[&str]) -> LineNames {
    LineNames::new(
        values
            .iter()
            .map(|v| DiscreteValue::from_text(v).unwrap())
            .collect(),
    )
}
fn single(value: TrackSize) -> TrackSection {
    TrackSection::Single(value)
}
fn repeated(count: usize, tracks: Vec<(TrackSize, LineNames)>) -> TrackSection {
    TrackSection::Repeat(FixedRepeat::new(count, names(&[]), tracks).unwrap())
}
fn template(leading: &[&str], sections: Vec<(TrackSection, LineNames)>) -> GridValue {
    GridValue::Template(TemplateTrackList::new(names(leading), sections).unwrap())
}
fn list_sections(sections: Vec<TrackSection>) -> GridValue {
    template(&[], sections.into_iter().map(|v| (v, names(&[]))).collect())
}
fn list(tracks: Vec<TrackSize>) -> GridValue {
    list_sections(tracks.into_iter().map(single).collect())
}
fn implicit(tracks: Vec<TrackSize>) -> GridValue {
    GridValue::Implicit(ImplicitTrackList::new(tracks).unwrap())
}
fn wrapped(value: GridValue) -> InterpolableValue {
    InterpolableValue::grid(value)
}
fn progress(value: f64) -> InterpolationProgress {
    InterpolationProgress::new(value).unwrap()
}
fn pair(from: GridValue, to: GridValue) -> InterpolationPair {
    InterpolationPair::new(
        PropertyKey::new("grid-template-columns").unwrap(),
        wrapped(from),
        wrapped(to),
    )
    .unwrap()
}
fn keyword(value: &str) -> InterpolableValue {
    InterpolableValue::discrete(DiscreteValue::from_text(value).unwrap())
}
fn track(property: &str, from: GridValue, to: GridValue) -> KeyframeTrack {
    raw_track(property, wrapped(from), wrapped(to))
}
fn raw_track(property: &str, from: InterpolableValue, to: InterpolableValue) -> KeyframeTrack {
    KeyframeTrack::new(
        KeyframeAnimationId::new("grid-sample").unwrap(),
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap(),
        vec![
            KeyframeSegment::new(
                PropertyKey::new(property).unwrap(),
                UnitRatio::new(0.0).unwrap(),
                UnitRatio::new(1.0).unwrap(),
                from,
                to,
                Easing::linear(),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn assert_sample(track: &KeyframeTrack, time: f64, expected: GridValue) {
    let frame = track.sample(
        ElapsedTime::from_secs(time).unwrap(),
        AnimationPlayState::Running,
    );
    assert_eq!(frame.len(), 1);
    assert_eq!(frame[0].classification(), SampleClassification::Active);
    let SampleValue::Value(value) = frame[0].value() else {
        panic!("expected Grid sample")
    };
    assert_eq!(value.value(), &wrapped(expected));
}
