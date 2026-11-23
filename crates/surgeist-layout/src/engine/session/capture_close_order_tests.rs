use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

// A capture closes after every capture opened within it. The captures a failed
// computation leaves open belong to different requests, which a failure
// unwinds as one set, so they close by where they began, whichever request
// holds them.
fn assert_captures_close_latest_first<S: LayoutScalar>(order: impl Fn(Vec<usize>) -> Vec<usize>) {
    let tree = PublicLayoutTreeOf::<S>::new().style(0, crate::node_input::NodeInputOf::default());
    let mut session = ComputeSession::unobserved(&tree, Vec::new());
    let outer = session.begin_effect_capture(CaptureKind::Preparation);
    session.effects.record(StagedEffect::Clear(0));
    let middle = session.begin_effect_capture(CaptureKind::Request);
    session.effects.record(StagedEffect::Clear(1));
    let inner = session.begin_effect_capture(CaptureKind::Preparation);
    session.effects.record(StagedEffect::Clear(2));
    assert!(
        outer < middle && middle < inner,
        "each write moves the start"
    );
    session.close_latest_first(order(vec![outer, middle, inner]));
    assert!(session.effects.is_idle(), "every capture is closed");
}

#[test]
fn captures_held_in_opening_order_close_latest_first_f32() {
    assert_captures_close_latest_first::<f32>(|starts| starts);
}

#[test]
fn captures_held_in_opening_order_close_latest_first_f64() {
    assert_captures_close_latest_first::<f64>(|starts| starts);
}

#[test]
fn captures_held_in_reverse_order_close_latest_first() {
    assert_captures_close_latest_first::<f64>(|mut starts| {
        starts.reverse();
        starts
    });
}

#[test]
fn captures_held_interleaved_close_latest_first() {
    assert_captures_close_latest_first::<f64>(|starts| vec![starts[1], starts[2], starts[0]]);
}
