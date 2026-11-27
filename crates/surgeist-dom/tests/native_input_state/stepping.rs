//! Literal native Input stepping expectations from the pinned HTML algorithm.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError,
    identity::NodeId,
    limits::{DomLimits, DomNameLimits},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
};
use surgeist_html::{elements::input::HtmlInputStepDirection as Direction, values::Namespace};

fn setup(kind: &str, attributes: &[(&str, &str)]) -> (Dom, NodeId) {
    let limits = DomLimits::new(
        64,
        8192,
        256,
        0,
        0,
        8,
        100_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    // Range fixtures traverse the existing bounded sanitization cursor during
    // setup as well as the separate stepping operation. The refusal test
    // controls the Source allowance independently.
    .with_input_processing(16_000_000, 100_000_000)
    .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let input = batch.create_element_ns(&document, Namespace::html(), "input");
    batch.set_attribute(&input, "type", kind);
    for &(name, value) in attributes {
        batch.set_attribute(&input, name, value);
    }
    batch.insert_before(&input, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let input = commit.created(&input).unwrap().unwrap();
    (dom, input)
}

fn step(
    dom: &mut Dom,
    input: NodeId,
    direction: Direction,
    count: i32,
    work: u64,
) -> Result<DomSourceProgress, DomError> {
    let cancel = AtomicBool::new(false);
    let limits = DomSourceLimits::new(16_000_000, 64_000, work, 64, 512, 512, 1);
    dom.begin_source_edit(
        DomSourceEdit::step_input(input, direction, count),
        limits,
        &cancel,
    )?
    .advance(&cancel)
}

fn value(dom: &Dom, input: NodeId) -> String {
    let snapshot = dom.snapshot();
    let input = snapshot.node(input).unwrap().as_input().unwrap();
    assert!(!input.state().flags().dirty_value());
    String::from_utf16(input.value()).unwrap()
}

#[test]
fn unaligned_number_moves_one_directional_grid_point_before_considering_count() {
    for (direction, count, expected) in [(Direction::Up, -5, "6"), (Direction::Down, 0, "3")] {
        let (mut dom, input) = setup("number", &[("min", "0"), ("step", "3"), ("value", "4")]);
        assert!(matches!(
            step(&mut dom, input, direction, count, 10_000_000).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(value(&dom, input), expected);
    }
}

#[test]
fn aligned_signed_step_vetoes_reversal_and_reaches_zero_count_serialization() {
    let (mut dom, input) = setup("number", &[("min", "0"), ("step", "3"), ("value", "6.0")]);
    let original = dom.snapshot();
    assert!(matches!(
        step(&mut dom, input, Direction::Up, -1, 10_000_000).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert_eq!(value(&dom, input), "6.0");
    assert!(matches!(
        step(&mut dom, input, Direction::Up, 0, 10_000_000).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(value(&dom, input), "6");
    assert_eq!(
        original.node(input).unwrap().as_input().unwrap().value(),
        &[54, 46, 48]
    );
}

#[test]
fn zero_fallback_and_temporal_steps_keep_exact_epoch_coordinates() {
    for (kind, attributes, direction, expected) in [
        ("week", &[][..], Direction::Up, "1970-W02"),
        ("week", &[][..], Direction::Down, "1970-W01"),
        (
            "datetime-local",
            &[("step", "0.125"), ("value", "287396-10-12T08:59:01.125")][..],
            Direction::Up,
            "287396-10-12T08:59:01.25",
        ),
    ] {
        let (mut dom, input) = setup(kind, attributes);
        assert!(matches!(
            step(&mut dom, input, direction, 1, 10_000_000).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(value(&dom, input), expected);
    }
}

#[test]
fn range_and_calendar_serialize_through_the_live_type_at_invocation() {
    for (kind, attributes, direction, count, expected) in [
        (
            "range",
            &[("min", "0"), ("max", "8"), ("step", "3"), ("value", "3")][..],
            Direction::Up,
            9,
            "6",
        ),
        (
            "date",
            &[("value", "2000-03-01")][..],
            Direction::Down,
            1,
            "2000-02-29",
        ),
        (
            "month",
            &[("value", "1969-12")][..],
            Direction::Up,
            1,
            "1970-01",
        ),
        ("time", &[("value", "12:59")][..], Direction::Up, 1, "13:00"),
    ] {
        let (mut dom, input) = setup(kind, attributes);
        assert!(matches!(
            step(&mut dom, input, direction, count, 10_000_000).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(value(&dom, input), expected);
    }
    let (mut dom, input) = setup("text", &[("value", "2")]);
    assert!(matches!(
        step(&mut dom, input, Direction::Up, 1, 10_000_000),
        Err(DomError::InputStepInvalidState)
    ));
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "number");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    step(&mut dom, input, Direction::Up, 1, 10_000_000).unwrap();
    assert_eq!(value(&dom, input), "3");
}

#[test]
fn range_chooses_the_larger_finite_lattice_point_at_an_exact_tie() {
    let (mut dom, input) = setup(
        "range",
        &[
            ("min", "9007199254740992"),
            ("max", "9007199254740994"),
            ("step", "1"),
            ("value", "9007199254740992"),
        ],
    );
    step(&mut dom, input, Direction::Up, 1, 10_000_000).unwrap();
    assert_eq!(value(&dom, input), "9007199254740994");
}

#[test]
fn any_precedes_the_interval_and_a_reversed_time_interval_is_a_noop() {
    let edges = [("min", "23:00"), ("max", "02:00"), ("value", "00:30")];
    let (mut dom, input) = setup("time", &edges);
    let before = dom.snapshot().stamp();
    step(&mut dom, input, Direction::Up, 1, 10_000_000).unwrap();
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(value(&dom, input), "00:30");
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "step", "AnY");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let before = dom.snapshot().stamp();
    assert!(matches!(
        step(&mut dom, input, Direction::Down, 1, 10_000_000),
        Err(DomError::InputStepInvalidState)
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(value(&dom, input), "00:30");
}

#[test]
fn out_of_range_numbers_clamp_in_the_invoked_direction() {
    let (mut dom, input) = setup(
        "number",
        &[("value", "13"), ("min", "0"), ("max", "10"), ("step", "3")],
    );
    let before = dom.snapshot().stamp();
    step(&mut dom, input, Direction::Up, 1, 10_000_000).unwrap();
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(value(&dom, input), "13");
    step(&mut dom, input, Direction::Down, 1, 10_000_000).unwrap();
    assert_eq!(value(&dom, input), "9");
}

#[test]
fn stepping_reads_the_dirty_current_value_and_preserves_default_value() {
    let (mut dom, input) = setup("number", &[("value", "2"), ("step", "3")]);
    let mut batch = DomBatch::new();
    batch.set_input_value(input, "4");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    step(&mut dom, input, Direction::Up, 0, 10_000_000).unwrap();
    let after = dom.snapshot();
    let state = after.node(input).unwrap().as_input().unwrap();
    assert!(state.state().flags().dirty_value());
    assert_eq!(state.value(), &[u16::from(b'5')]);
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "value", "8");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let after = dom.snapshot();
    let state = after.node(input).unwrap().as_input().unwrap();
    assert_eq!(state.value(), &[u16::from(b'5')]);
}

#[test]
fn work_refusal_and_cancellation_preserve_the_snapshot_and_allow_retry() {
    let (mut dom, input) = setup("date", &[("value", "2000-02-28")]);
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        step(&mut dom, input, Direction::Up, 1, 1),
        Err(DomError::SourceWorkLimit { limit: 1 })
    ));
    let cancelled = AtomicBool::new(false);
    let mut edit = dom
        .begin_source_edit(
            DomSourceEdit::step_input(input, Direction::Up, 1),
            DomSourceLimits::new(16_000_000, 64_000, 10_000_000, 64, 512, 512, 1),
            &cancelled,
        )
        .unwrap();
    cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(edit.advance(&cancelled), Err(DomError::Cancelled)));
    drop(edit);
    super::unchanged(&dom, &before, usage);
    assert_eq!(value(&dom, input), "2000-02-28");
    step(&mut dom, input, Direction::Up, 1, 10_000_000).unwrap();
    assert_eq!(value(&dom, input), "2000-02-29");
}
