#![forbid(unsafe_code)]

use surgeist_animation::easing::Easing;
use surgeist_animation::interpolation::PropertyKey;
use surgeist_animation::keyframe_input::{
    CompositeOperation, KeyframeInput, KeyframeInputErrorKind, KeyframeInputLocation,
    KeyframesInput, NormalizedKeyframes, PropertyIndexedInput,
};

#[derive(Debug, PartialEq)]
struct Value {
    text: &'static str,
    source: u32,
}

type Input = KeyframesInput<Value, &'static str>;
type Frame = KeyframeInput<Value, &'static str>;
type Indexed = PropertyIndexedInput<Value, &'static str>;

#[test]
fn absence_and_empty_inputs_return_empty_normalized_sets() {
    for input in [
        Input::Absent,
        Input::Sequence(vec![]),
        Input::PropertyIndexed(Indexed::default()),
    ] {
        assert!(input.normalize().unwrap().frames().is_empty());
    }
}

#[test]
fn singleton_missing_offset_is_one_without_changing_specified_offset() {
    let result = Input::Sequence(vec![Frame::default()]).normalize().unwrap();
    assert_eq!(offsets(&result), vec![(None, 1.0)]);
    assert_eq!(result.frames()[0].easing(), &Easing::linear());
    assert_eq!(result.frames()[0].composite(), None);
    assert_eq!(
        result.frames()[0].effective_composite(CompositeOperation::Add),
        CompositeOperation::Add
    );
}

#[test]
fn explicit_anchor_offsets_remain_exact_and_equal_anchor_runs_stay_equal() {
    let result = Input::Sequence(vec![
        frame(Some(0.3)),
        frame(Some(0.9)),
        frame(None),
        frame(Some(0.9)),
    ])
    .normalize()
    .unwrap();
    assert_eq!(
        offsets(&result),
        vec![
            (Some(0.3), 0.3),
            (Some(0.9), 0.9),
            (None, 0.9),
            (Some(0.9), 0.9)
        ]
    );
}

#[test]
fn missing_offsets_are_distributed_between_explicit_neighbors() {
    let result = Input::Sequence(
        [None, None, Some(0.4), None, None, Some(0.7), None]
            .into_iter()
            .map(frame)
            .collect(),
    )
    .normalize()
    .unwrap();
    let expected = [
        (None, 0.0),
        (None, 0.2),
        (Some(0.4), 0.4),
        (None, 0.5),
        (None, 0.6),
        (Some(0.7), 0.7),
        (None, 1.0),
    ];
    for ((specified, computed), (expected_specified, expected_computed)) in
        offsets(&result).into_iter().zip(expected)
    {
        assert_eq!(specified, expected_specified);
        assert!((computed - expected_computed).abs() < 1e-15);
    }
}

#[test]
fn equal_offsets_preserve_separate_frames_values_and_metadata() {
    let mut first = frame(Some(0.5));
    first
        .properties
        .push((key("--Color"), Some(value("red", 4))));
    first.easing = Ok(Easing::step_start());
    let mut second = frame(Some(0.5));
    second
        .properties
        .push((key("--Color"), Some(value("blue", 9))));
    second.composite = Some(CompositeOperation::Accumulate);
    let result = Input::Sequence(vec![first, second]).normalize().unwrap();
    assert_eq!(offsets(&result), vec![(Some(0.5), 0.5), (Some(0.5), 0.5)]);
    assert_eq!(result.frames()[0].properties()[0].1, value("red", 4));
    assert_eq!(result.frames()[1].properties()[0].1, value("blue", 9));
    assert_eq!(result.frames()[0].easing(), &Easing::step_start());
    assert_eq!(
        result.frames()[1].effective_composite(CompositeOperation::Add),
        CompositeOperation::Accumulate
    );
}

#[test]
fn offsets_reject_nonfinite_and_out_of_range_inputs() {
    for offset in [-0.01, 1.01, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
        let error = Input::Sequence(vec![frame(Some(offset))])
            .normalize()
            .unwrap_err();
        assert_eq!(error.kind(), KeyframeInputErrorKind::InvalidOffset);
        assert_eq!(error.location(), KeyframeInputLocation::Frame(0));
    }
}

#[test]
fn offset_order_precedes_range_and_easing_validation() {
    let mut first = frame(Some(2.0));
    first.easing = Err("invalid easing");
    let error = Input::Sequence(vec![first, frame(None), frame(Some(1.0))])
        .normalize()
        .unwrap_err();
    assert_eq!(error.kind(), KeyframeInputErrorKind::OffsetOrder);
    assert_eq!(error.location(), KeyframeInputLocation::Frame(2));
    assert_eq!(error.offset(), Some(1.0));
    assert_eq!(error.previous_offset(), Some(2.0));

    let mut first = frame(Some(1.5));
    first.easing = Err("invalid easing");
    let error = Input::Sequence(vec![first]).normalize().unwrap_err();
    assert_eq!(error.kind(), KeyframeInputErrorKind::InvalidOffset);

    let error = Input::PropertyIndexed(Indexed {
        properties: vec![(key("a"), vec![None, None])],
        offsets: vec![Some(2.0), Some(1.0)],
        easings: vec![Err("invalid easing")],
        ..Indexed::default()
    })
    .normalize()
    .unwrap_err();
    assert_eq!(error.kind(), KeyframeInputErrorKind::OffsetOrder);
    assert_eq!(error.location(), KeyframeInputLocation::Frame(1));

    let error = Input::PropertyIndexed(Indexed {
        properties: vec![(key("a"), vec![None])],
        offsets: vec![Some(2.0)],
        easings: vec![Err("invalid easing")],
        ..Indexed::default()
    })
    .normalize()
    .unwrap_err();
    assert_eq!(error.kind(), KeyframeInputErrorKind::InvalidOffset);
}

#[test]
fn invalid_values_are_discarded_without_removing_their_frames() {
    let mut first = frame(None);
    first.properties = vec![
        (key("--Color"), Some(value("red", 1))),
        (key("--color"), None),
    ];
    let mut second = frame(None);
    second.properties = vec![(key("--color"), Some(value("blue", 2)))];
    let result = Input::Sequence(vec![first, Frame::default(), second])
        .normalize()
        .unwrap();
    assert_eq!(
        offsets(&result),
        vec![(None, 0.0), (None, 0.5), (None, 1.0)]
    );
    assert_eq!(result.frames()[0].properties()[0].0.as_str(), "--Color");
    assert!(result.frames()[1].properties().is_empty());
    assert_eq!(result.frames()[2].properties()[0].0.as_str(), "--color");
}

#[test]
fn property_tracks_merge_intermediate_offsets_before_positional_assignments() {
    let result = Input::PropertyIndexed(Indexed {
        properties: vec![
            (
                key("a"),
                vec![
                    Some(value("a0", 0)),
                    Some(value("a1", 1)),
                    Some(value("a2", 2)),
                ],
            ),
            (key("b"), vec![Some(value("b0", 3)), Some(value("b1", 4))]),
            (key("c"), vec![Some(value("c0", 5))]),
        ],
        offsets: vec![Some(0.2), None, Some(0.8), Some(f64::INFINITY)],
        ..Indexed::default()
    })
    .normalize()
    .unwrap();
    assert_eq!(
        offsets(&result),
        vec![(Some(0.2), 0.2), (None, 0.5), (Some(0.8), 0.8)]
    );
    assert_eq!(
        result.frames()[0]
            .properties()
            .iter()
            .map(|(_, v)| v.text)
            .collect::<Vec<_>>(),
        vec!["a0", "b0"]
    );
    assert_eq!(result.frames()[1].properties()[0].1, value("a1", 1));
    assert_eq!(
        result.frames()[2]
            .properties()
            .iter()
            .map(|(_, v)| v.text)
            .collect::<Vec<_>>(),
        vec!["a2", "b1", "c0"]
    );
}

#[test]
fn property_tracks_with_different_lengths_form_the_union_of_offsets() {
    let result = Input::PropertyIndexed(Indexed {
        properties: vec![
            (key("a"), vec![None, None, Some(value("a2", 2))]),
            (key("b"), vec![None, Some(value("b1", 4)), None, None]),
        ],
        ..Indexed::default()
    })
    .normalize()
    .unwrap();
    assert_eq!(
        offsets(&result),
        vec![
            (None, 0.0),
            (None, 0.25),
            (None, 0.5),
            (None, 0.75),
            (None, 1.0)
        ]
    );
    assert_eq!(result.frames()[1].properties()[0].1, value("b1", 4));
    assert!(result.frames()[0].properties().is_empty());
}

#[test]
fn property_lists_repeat_easing_and_composite_independently() {
    let result = Input::PropertyIndexed(Indexed {
        properties: vec![(key("opacity"), vec![None, None, None, None, None])],
        easings: vec![Ok(Easing::step_start()), Ok(Easing::step_end())],
        composites: vec![
            Some(CompositeOperation::Replace),
            None,
            Some(CompositeOperation::Accumulate),
        ],
        ..Indexed::default()
    })
    .normalize()
    .unwrap();
    assert_eq!(
        result
            .frames()
            .iter()
            .map(|f| f.easing())
            .collect::<Vec<_>>(),
        vec![
            &Easing::step_start(),
            &Easing::step_end(),
            &Easing::step_start(),
            &Easing::step_end(),
            &Easing::step_start()
        ]
    );
    assert_eq!(
        result
            .frames()
            .iter()
            .map(|f| f.effective_composite(CompositeOperation::Add))
            .collect::<Vec<_>>(),
        vec![
            CompositeOperation::Replace,
            CompositeOperation::Add,
            CompositeOperation::Accumulate,
            CompositeOperation::Replace,
            CompositeOperation::Add
        ]
    );
}

#[test]
fn every_supplied_easing_is_validated_even_without_a_matching_frame() {
    for values in [vec![], vec![None]] {
        let error = Input::PropertyIndexed(Indexed {
            properties: vec![(key("opacity"), values)],
            easings: vec![Ok(Easing::linear()), Err("surplus source location")],
            ..Indexed::default()
        })
        .normalize()
        .unwrap_err();
        assert_eq!(error.kind(), KeyframeInputErrorKind::InvalidEasing);
        assert_eq!(error.location(), KeyframeInputLocation::Easing(1));
        assert_eq!(error.easing_error(), Some(&"surplus source location"));
    }
}

#[test]
fn used_easing_errors_precede_unused_errors_and_preserve_caller_provenance() {
    let error = Input::PropertyIndexed(Indexed {
        properties: vec![(key("a"), vec![None])],
        easings: vec![Err("used source"), Err("unused source")],
        ..Indexed::default()
    })
    .normalize()
    .unwrap_err();
    assert_eq!(error.location(), KeyframeInputLocation::Easing(0));
    assert_eq!(error.easing_error(), Some(&"used source"));
    let input = Frame {
        easing: Err("frame source"),
        ..Frame::default()
    };
    let error = Input::Sequence(vec![Frame::default(), input])
        .normalize()
        .unwrap_err();
    assert_eq!(error.location(), KeyframeInputLocation::Frame(1));
    assert_eq!(error.easing_error(), Some(&"frame source"));
}

#[test]
fn duplicate_property_identities_are_rejected_in_typed_dictionary_inputs() {
    let input = Frame {
        properties: vec![(key("a"), None), (key("a"), Some(value("a1", 1)))],
        ..Frame::default()
    };
    let error = Input::Sequence(vec![input]).normalize().unwrap_err();
    assert_eq!(error.kind(), KeyframeInputErrorKind::DuplicateProperty);
    assert_eq!(error.property().unwrap().as_str(), "a");
    let error = Input::PropertyIndexed(Indexed {
        properties: vec![(key("a"), vec![]), (key("a"), vec![])],
        ..Indexed::default()
    })
    .normalize()
    .unwrap_err();
    assert_eq!(error.location(), KeyframeInputLocation::Property(1));
}

#[test]
fn a_failed_candidate_releases_all_payloads_without_retaining_partial_frames() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    #[derive(Debug)]
    struct Payload(Arc<AtomicUsize>);
    impl Drop for Payload {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let dropped = Arc::new(AtomicUsize::new(0));
    let frames = (0..3)
        .map(|index| KeyframeInput {
            properties: vec![(key("a"), Some(Payload(dropped.clone())))],
            easing: if index == 1 {
                Err("reject replacement")
            } else {
                Ok(Easing::linear())
            },
            ..KeyframeInput::default()
        })
        .collect();
    let error = KeyframesInput::Sequence(frames).normalize().unwrap_err();
    assert_eq!(error.location(), KeyframeInputLocation::Frame(1));
    assert_eq!(dropped.load(Ordering::Relaxed), 3);
}

#[test]
fn easing_validation_keeps_typed_source_provenance() {
    #[derive(Debug, PartialEq)]
    struct InvalidEasing {
        source: u32,
        span: core::ops::Range<usize>,
    }
    let input = KeyframesInput::Sequence(vec![KeyframeInput::<Value, InvalidEasing> {
        easing: Err(InvalidEasing {
            source: 7,
            span: 8..16,
        }),
        ..KeyframeInput::default()
    }]);
    let error = input.normalize().unwrap_err();
    assert_eq!(
        error.easing_error(),
        Some(&InvalidEasing {
            source: 7,
            span: 8..16
        })
    );
}

#[test]
fn property_indexed_defaults_and_single_offsets_do_not_repeat() {
    let result = Input::PropertyIndexed(Indexed {
        properties: vec![(key("a"), vec![None, None, None])],
        offsets: vec![Some(0.4)],
        ..Indexed::default()
    })
    .normalize()
    .unwrap();
    assert_eq!(
        offsets(&result),
        vec![(Some(0.4), 0.4), (None, 0.7), (None, 1.0)]
    );
    for frame in result.frames() {
        assert_eq!(frame.easing(), &Easing::linear());
        assert_eq!(frame.composite(), None);
    }
}

#[test]
fn normalization_errors_expose_numeric_and_caller_causes() {
    use std::error::Error;
    let invalid_offset = KeyframesInput::Sequence(vec![KeyframeInput::<f64> {
        offset: Some(-0.5),
        ..KeyframeInput::default()
    }])
    .normalize()
    .unwrap_err();
    assert!(invalid_offset.source().is_some());
    assert!(invalid_offset.to_string().contains("invalid offset -0.5"));

    let invalid_easing = KeyframesInput::Sequence(vec![KeyframeInput::<f64, std::io::Error> {
        easing: Err(std::io::Error::other("caller easing cause")),
        ..KeyframeInput::default()
    }])
    .normalize()
    .unwrap_err();
    assert_eq!(
        invalid_easing.source().unwrap().to_string(),
        "caller easing cause"
    );
    assert_eq!(invalid_easing.location(), KeyframeInputLocation::Frame(0));
}

fn offsets(frames: &NormalizedKeyframes<Value>) -> Vec<(Option<f64>, f64)> {
    frames
        .frames()
        .iter()
        .map(|f| {
            (
                f.specified_offset().map(|o| o.value()),
                f.computed_offset().value(),
            )
        })
        .collect()
}

fn frame(offset: Option<f64>) -> Frame {
    Frame {
        offset,
        ..Frame::default()
    }
}

fn key(name: &str) -> PropertyKey {
    PropertyKey::new(name).unwrap()
}
fn value(text: &'static str, source: u32) -> Value {
    Value { text, source }
}
