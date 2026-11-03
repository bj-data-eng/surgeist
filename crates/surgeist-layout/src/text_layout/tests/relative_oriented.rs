//! Relative translation preserves normal inline metrics across actual Text orientations.
//! Normal ancestor padding still moves the exported operational reference.
use super::*;

const ORIENTED_MODES: [WritingMode; 4] = [
    WritingMode::HorizontalTb,
    WritingMode::SidewaysRl,
    WritingMode::VerticalRl,
    WritingMode::VerticalLr,
];

fn oriented_atomic<S: LayoutScalar>(mode: WritingMode, inline: f64, cross: f64) -> NodeInputOf<S> {
    let mut input = child(mode, cross, BaselineSource::Auto, VerticalAlignOf::Baseline);
    let physical = size::<f64>(mode, inline, cross);
    input.size = Size::new(px(physical.width), px(physical.height));
    input.box_sizing = BoxSizing::BorderBox;
    assert!(input.atomic_inline_participation.is_none());
    input
}

fn oriented_translate<S: LayoutScalar>(input: &mut NodeInputOf<S>, mode: WritingMode, amount: f64) {
    input.position = Position::Relative;
    input.inset = Edges::all(LengthAutoOf::AUTO);
    match mode {
        WritingMode::HorizontalTb => input.inset.top = LengthAutoOf::px(n(amount)),
        WritingMode::SidewaysRl | WritingMode::VerticalRl => {
            input.inset.right = LengthAutoOf::px(n(amount));
        }
        WritingMode::VerticalLr => input.inset.left = LengthAutoOf::px(n(amount)),
        _ => unreachable!(),
    }
}

fn oriented_pad_start<S: LayoutScalar>(input: &mut NodeInputOf<S>, mode: WritingMode) {
    let amount = LengthOf::px(n(3.0));
    match mode {
        WritingMode::HorizontalTb => input.padding.top = amount,
        WritingMode::SidewaysRl | WritingMode::VerticalRl => input.padding.right = amount,
        WritingMode::VerticalLr => input.padding.left = amount,
        _ => unreachable!(),
    }
}

fn oriented_source(layout: &text::Layout, native: &Native) -> text::Rect {
    custody(layout, &native.original);
    raw(layout, 10.0, 10.0);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 1));
    let clusters = layout.clusters();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].range(), text::Range::new(0, 1));
    clusters[0].bounds()
}

// Ordinary cluster bounds cover the complete normal line interaction band,
// while its inline extent remains this actual X cluster's10px contribution.
fn oriented_interaction_rect(mode: WritingMode, inline: f64, cross: f64, band: f64) -> text::Rect {
    if vertical(mode) {
        text::Rect::new(cross, inline, band, 10.0)
    } else {
        text::Rect::new(inline, cross, 10.0, band)
    }
}

// Actual paint origin of the same retained real X packet, independently of
// the cluster's whole-line interaction band. No synthetic font rectangle.
fn oriented_glyph_origin(layout: &text::Layout) -> text::Point {
    let mut runs = layout.run_views();
    let run = runs.next().expect("actual X glyph run");
    assert!(runs.next().is_none());
    assert_eq!(run.range(), text::Range::new(0, 1));
    let origin = run.origin();
    assert!(origin.x.is_finite() && origin.y.is_finite());
    origin
}

fn oriented_node_text_agree<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) {
    let layout = placed(batch, 0);
    let boxes = layout.inline_boxes();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].id(), text::Id::from_u64(u64::from(node)));
    let packet = output(batch, node);
    assert_eq!(
        boxes[0].rect(),
        text::Rect::new(
            packet.location.x.to_f64(),
            packet.location.y.to_f64(),
            packet.size.width.to_f64(),
            packet.size.height.to_f64(),
        )
    );
}

fn oriented_relative_atomic<S: LayoutScalar>() {
    premises();
    for mode in ORIENTED_MODES {
        let parent = native("X", 10.0, 10.0, mode, &[(2, 10.0, 10.4)]);
        let binding = parent.binding(0, 1);
        assert_eq!(binding.nodes, vec![1, 2]);
        assert_eq!(binding.atomics.len(), 1);
        assert_eq!(binding.atomics[0].node, 2);
        assert_eq!(
            binding.atomics[0].occurrence,
            parent.prepared.atomic_views().next().unwrap().handle()
        );
        let upright = matches!(mode, WritingMode::VerticalRl | WritingMode::VerticalLr);
        // H/CW use the real hhea8/2; upright uses the qualified em5/5.
        // The unshifted canonical10px empty atomic contributes10/0 or5/5.
        let cross = if upright { 10.0 } else { 12.0 };
        let before = if upright { 5.0 } else { 10.0 };
        let reference = match mode {
            WritingMode::HorizontalTb => 10.0,
            WritingMode::SidewaysRl => 2.0,
            _ => 5.0,
        };
        let mut published = Vec::new();
        for shifted in [false, true] {
            let mut atomic = oriented_atomic::<S>(mode, 10.0, 10.4);
            let physical = size::<f64>(mode, 10.0, 10.4);
            assert_eq!(
                atomic.size,
                Size::new(px(physical.width), px(physical.height))
            );
            if shifted {
                if upright {
                    atomic.position = Position::Relative;
                    atomic.inset.left = LengthAutoOf::px(n(0.32));
                } else {
                    oriented_translate(&mut atomic, mode, 0.72);
                }
            }
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2])
                .children(1, [])
                .children(2, [])
                .style(0, root(mode))
                .input(1, parent.input())
                .style(2, atomic);
            let batch = solve(&tree, mode, vec![binding.clone()]).unwrap();
            // Normal sizing/ref facts are asserted before visual corner facts.
            assert_eq!(output(&batch, 0).size, size(mode, 100.0, cross));
            let line = finalized(&batch, 0);
            let feedback = line.feedback();
            assert_eq!(line.measurements().fit().advance(), 20.0);
            assert_eq!(
                line.measurements().margin_edge_fit().required_inline_size(),
                20.0
            );
            assert_eq!((feedback.baseline, feedback.line_height), (before, cross));
            assert_eq!(placed(&batch, 0).lines()[0].baseline(), reference);
            assert_eq!(
                oriented_source(placed(&batch, 0), &parent),
                oriented_interaction_rect(mode, 0.0, 0.0, cross)
            );
            oriented_node_text_agree(&batch, 2);
            let packet = output(&batch, 2);
            let visual_cross = if shifted { 11.0 } else { 10.0 };
            assert_eq!(packet.size, size(mode, 10.0, visual_cross));
            let cross_origin = if mode == WritingMode::SidewaysRl {
                if shifted { 1.0 } else { 2.0 }
            } else {
                0.0
            };
            let expected_location = if vertical(mode) {
                Point::new(n(cross_origin), n(10.0))
            } else {
                Point::new(n(10.0), n(cross_origin))
            };
            assert_eq!(packet.location, expected_location);
            published.push(batch);
        }
        // The original publication remains unchanged after the separate request.
        assert_eq!(output(&published[0], 0).size, size(mode, 100.0, cross));
        assert_eq!(
            placed(&published[0], 0).clusters(),
            placed(&published[1], 0).clusters()
        );
        assert_eq!(
            oriented_glyph_origin(placed(&published[0], 0)),
            oriented_glyph_origin(placed(&published[1], 0))
        );
    }
}

fn oriented_ancestor_and_normal_padding<S: LayoutScalar>() {
    premises();
    for mode in ORIENTED_MODES {
        let outer = native("X", 10.0, 10.0, mode, &[(2, 20.0, 20.0)]);
        let inner = native("X", 10.0, 10.0, mode, &[(12, 10.0, 10.4)]);
        let (control_reference, padded_reference, control_before, padded_before, inner_band_cross) =
            match mode {
                WritingMode::HorizontalTb => (10.0, 13.0, 10.0, 13.0, 0.0),
                WritingMode::SidewaysRl => (10.0, 7.0, 10.0, 13.0, 8.0),
                WritingMode::VerticalRl => (15.0, 12.0, 5.0, 8.0, 10.0),
                WritingMode::VerticalLr => (5.0, 8.0, 15.0, 12.0, 0.0),
                _ => unreachable!(),
            };
        let signed_move = if matches!(mode, WritingMode::SidewaysRl | WritingMode::VerticalRl) {
            -3.0
        } else {
            3.0
        };
        let mut published = Vec::new();
        //0=control,1=relative ancestor,2=genuine normal block-start padding.
        for case in 0..3 {
            let mut ancestor = oriented_atomic::<S>(mode, 20.0, 20.0);
            if case == 1 {
                oriented_translate(&mut ancestor, mode, 3.0);
            }
            if case == 2 {
                oriented_pad_start(&mut ancestor, mode);
            }
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2])
                .children(1, [])
                .children(2, [11, 12])
                .children(11, [])
                .children(12, [])
                .style(0, root(mode))
                .input(1, outer.input())
                .style(2, ancestor)
                .input(11, inner.input())
                .style(12, oriented_atomic(mode, 10.0, 10.4));
            let bindings = vec![outer.binding(0, 1), inner.binding(2, 11)];
            assert_eq!(bindings[0].nodes, vec![1, 2]);
            assert_eq!(bindings[1].nodes, vec![11, 12]);
            let batch = solve(&tree, mode, bindings).unwrap();
            let reference = if case == 2 {
                padded_reference
            } else {
                control_reference
            };
            let before = if case == 2 {
                padded_before
            } else {
                control_before
            };
            assert_eq!(output(&batch, 0).size, size(mode, 100.0, 20.0));
            assert_eq!(output(&batch, 2).size, size(mode, 20.0, 20.0));
            let line = finalized(&batch, 0);
            let feedback = line.feedback();
            assert_eq!(line.measurements().fit().advance(), 30.0);
            assert_eq!(
                line.measurements().margin_edge_fit().required_inline_size(),
                30.0
            );
            assert_eq!(finalized(&batch, 2).measurements().fit().advance(), 20.0);
            assert_eq!((feedback.baseline, feedback.line_height), (before, 20.0));
            assert_eq!(placed(&batch, 0).lines()[0].baseline(), reference);
            let axis = if vertical(mode) {
                PhysicalAxis::Horizontal
            } else {
                PhysicalAxis::Vertical
            };
            let pair = output(&batch, 2).named_baselines.get(name(mode));
            fact(pair.first(), axis, reference);
            fact(pair.last(), axis, reference);
            // A normal reference can move within an unchanged20px line band.
            assert_eq!(
                oriented_source(placed(&batch, 0), &outer),
                oriented_interaction_rect(mode, 0.0, 0.0, 20.0)
            );
            // Completed Text views already carry their global placed anchor.
            let global_inner = oriented_source(placed(&batch, 2), &inner);
            let inner_cross = inner_band_cross + if case == 0 { 0.0 } else { signed_move };
            let inner_band = if matches!(mode, WritingMode::VerticalRl | WritingMode::VerticalLr) {
                10.0
            } else {
                12.0
            };
            assert_eq!(
                global_inner,
                oriented_interaction_rect(mode, 10.0, inner_cross, inner_band)
            );
            oriented_node_text_agree(&batch, 2);
            published.push(batch);
        }
        // Equal visible descendant placement has different operational causes:
        // only normal padding changes the outer reference and outside source X.
        assert_eq!(
            placed(&published[0], 0).clusters(),
            placed(&published[1], 0).clusters()
        );
        // The full normal interaction band stays fixed. The actual glyph
        // origin distinguishes normal padding from relative-only ancestry.
        assert_eq!(
            placed(&published[1], 0).clusters(),
            placed(&published[2], 0).clusters()
        );
        let outside_control = oriented_glyph_origin(placed(&published[0], 0));
        let outside_relative = oriented_glyph_origin(placed(&published[1], 0));
        let outside_padded = oriented_glyph_origin(placed(&published[2], 0));
        assert_eq!(outside_control, outside_relative);
        let expected_padded = if vertical(mode) {
            text::Point::new(outside_relative.x + signed_move, outside_relative.y)
        } else {
            text::Point::new(outside_relative.x, outside_relative.y + signed_move)
        };
        assert_eq!(outside_padded, expected_padded);
        let inner_origins = published
            .iter()
            .map(|batch| oriented_glyph_origin(placed(batch, 2)))
            .collect::<Vec<_>>();
        let shifted_inner = if vertical(mode) {
            text::Point::new(inner_origins[0].x + signed_move, inner_origins[0].y)
        } else {
            text::Point::new(inner_origins[0].x, inner_origins[0].y + signed_move)
        };
        assert_eq!(inner_origins[1], shifted_inner);
        assert_eq!(inner_origins[2], shifted_inner);
        assert_eq!(
            output(&published[0], 2).named_baselines,
            output(&published[1], 2).named_baselines
        );
    }
}

#[test]
fn fractional_relative_atomic_preserves_oriented_normal_metrics_f32() {
    oriented_relative_atomic::<f32>();
}
#[test]
fn fractional_relative_atomic_preserves_oriented_normal_metrics_f64() {
    oriented_relative_atomic::<f64>();
}
#[test]
fn relative_ancestor_is_not_normal_padding_across_real_text_orientations_f32() {
    oriented_ancestor_and_normal_padding::<f32>();
}
#[test]
fn relative_ancestor_is_not_normal_padding_across_real_text_orientations_f64() {
    oriented_ancestor_and_normal_padding::<f64>();
}
