//! Actual endpoint geometry through contextual capture, Ahem and native Layout.

use super::*;
use crate::test_support::inline_endpoint::Fixture;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn finalized_lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<text::FinalizedLine> {
    let mut lines = Vec::new();
    for fragment in batch.unrounded_inline_fragments() {
        let line = fragment.fragment().finalized_fragment().finalized_line();
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    lines
}

fn qualify(line: &text::FinalizedLine) {
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), 10.0);
    }
}

#[test]
fn end_edge_stops_local_hanging_and_moves_the_actual_line_below_a_float_both_scalars() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new("X \nY", (0, 2), text::WhiteSpace::Preserve, false, false);
        for (edge, raw, fit, alignment, y) in [
            (InlineBoundaryEdgeOf::ZERO, 20.0, 10.0, 15.0, 0.0),
            (
                InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap(),
                25.0,
                25.0,
                25.0,
                20.0,
            ),
        ] {
            let batch = fixture.compute(
                InlineBoundaryEdgeOf::ZERO,
                edge,
                false,
                Some(40.0),
                true,
                AvailableOf::definite(S::from_f64(40.0)),
            );
            let lines = finalized_lines(&batch);
            assert_eq!(lines.len(), 2);
            let first = &lines[0];
            qualify(first);
            assert_eq!(
                (
                    first.feedback().advance,
                    first.measurements().fit().advance(),
                    first.measurements().alignment().advance()
                ),
                (raw, fit, alignment),
                "raw / fit / alignment"
            );
            assert_eq!(
                first.measurements().min_content_contribution().advance(),
                fit
            );
            assert_eq!(
                first.measurements().max_content_contribution().advance(),
                raw
            );
            assert_eq!(output(&batch, 20).location.y, S::from_f64(y + 8.0));
            let endpoint = output(&batch, 21);
            assert_eq!(endpoint.size, Size::new(edge.advance(), S::ZERO));
            assert_eq!(endpoint.content_size, Size::ZERO);
            assert_eq!(endpoint.border.right, edge.border());
            assert_eq!(endpoint.padding.right, edge.padding());
            assert!(
                batch
                    .unrounded_inline_fragments()
                    .iter()
                    .all(|entry| !matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            );
            let slot = first
                .control_slots()
                .find(|slot| {
                    slot.control()
                        == fixture
                            .binding
                            .controls
                            .iter()
                            .find(|row| row.node == 21)
                            .unwrap()
                            .occurrence
                })
                .unwrap();
            assert_eq!(slot.inline_bounds(), (20.0, raw));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn actual_tab_sees_endpoint_total_and_f32_threshold_before_output_narrowing() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new("A\tB", (1, 3), text::WhiteSpace::Preserve, false, true);
        for (edge, total, advance) in [
            (InlineBoundaryEdgeOf::ZERO, 0.0, 30.0),
            (
                InlineBoundaryEdgeOf::try_new(S::from_f64(3.0), S::from_f64(5.0)).unwrap(),
                8.0,
                50.0,
            ),
        ] {
            let batch = fixture.compute(
                edge,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(100.0),
                false,
                AvailableOf::definite(S::from_f64(100.0)),
            );
            let lines = finalized_lines(&batch);
            assert_eq!(lines.len(), 1);
            qualify(&lines[0]);
            assert_eq!(lines[0].feedback().advance, advance);
            let request = lines[0].selected_line().request().clone();
            let allocation = request
                .controls
                .iter()
                .find(|row| {
                    row.occurrence
                        == fixture
                            .binding
                            .controls
                            .iter()
                            .find(|row| row.node == 20)
                            .unwrap()
                            .occurrence
                })
                .unwrap();
            assert_eq!(allocation.boundary.unwrap().advance(), total);
            assert_eq!(output(&batch, 20).size.width, edge.advance());
            assert_eq!(output(&batch, 20).border.left, edge.border());
            assert_eq!(output(&batch, 20).padding.left, edge.padding());
            let b = lines[0]
                .run_views()
                .flat_map(|run| {
                    let origin = run.origin().x;
                    run.glyphs()
                        .map(move |glyph| (glyph.id(), origin + glyph.pen()))
                        .collect::<Vec<_>>()
                })
                .find(|row| row.0 == 36)
                .unwrap();
            assert_eq!(b.1 + 10.0, advance);
            let intrinsic = fixture.compute(
                edge,
                InlineBoundaryEdgeOf::ZERO,
                false,
                None,
                false,
                AvailableOf::MAX_CONTENT,
            );
            assert_eq!(output(&intrinsic, 0).size.width, S::from_f64(advance));
            assert_eq!(finalized_lines(&intrinsic)[0].feedback().advance, advance);
        }
    }
    lane::<f32>();
    lane::<f64>();
    let fixture = Fixture::new("A\tB", (1, 3), text::WhiteSpace::Preserve, false, true);
    let edge = InlineBoundaryEdgeOf::try_new(4.8f32, 0.2f32).unwrap();
    assert_eq!(edge.advance(), 5.0);
    assert_eq!(
        f64::from(edge.border()) + f64::from(edge.padding()),
        5.000_000_193_715_095_5
    );
    let batch = fixture.compute(
        edge,
        InlineBoundaryEdgeOf::ZERO,
        false,
        Some(100.0),
        false,
        AvailableOf::definite(100.0),
    );
    let line = finalized_lines(&batch).remove(0);
    assert_eq!(line.feedback().advance, 30.0);
    assert_eq!(
        line.selected_line()
            .request()
            .controls
            .iter()
            .find_map(|row| row.boundary.filter(|edge| edge.advance() > 0.0))
            .unwrap()
            .advance(),
        5.0
    );
    assert_eq!(output(&batch, 20).border.left, 4.8f32);
    assert_eq!(output(&batch, 20).padding.left, 0.2f32);
}

#[test]
fn endpoint_node_uses_its_own_direction_and_control_fragment_without_a_scope_envelope() {
    fn lane<S: LayoutScalar>() {
        // The endpoint owns its physical side independently of its resolved
        // even/odd Text control level and paragraph progression.
        for (value, paragraph_rtl, own_rtl, level) in
            [("AB", false, true, 0), ("אב", true, false, 1)]
        {
            let fixture = Fixture::new(
                value,
                (0, 2),
                text::WhiteSpace::Preserve,
                paragraph_rtl,
                false,
            );
            let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(3.0), S::from_f64(5.0)).unwrap();
            let batch = fixture.compute(
                edge,
                InlineBoundaryEdgeOf::ZERO,
                own_rtl,
                Some(100.0),
                false,
                AvailableOf::definite(S::from_f64(100.0)),
            );
            let strip = output(&batch, 20);
            assert_eq!(strip.size, Size::new(S::from_f64(8.0), S::ZERO));
            assert_eq!(strip.content_size, Size::ZERO);
            assert_eq!(
                strip.border,
                if own_rtl {
                    Edges::new(S::ZERO, S::from_f64(3.0), S::ZERO, S::ZERO)
                } else {
                    Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(3.0))
                }
            );
            assert_eq!(
                strip.padding,
                if own_rtl {
                    Edges::new(S::ZERO, S::from_f64(5.0), S::ZERO, S::ZERO)
                } else {
                    Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(5.0))
                }
            );
            let fragment = batch
                .unrounded_inline_fragments()
                .iter()
                .find(|entry| matches!(entry.owner(), InlineFragmentOwner::Node(20)))
                .unwrap();
            let view = fragment.fragment().finalized_fragment().view();
            assert!(matches!(
                view.owner(),
                text::FinalizedFragmentOwner::Control(_)
            ));
            assert!(view.origin().is_none());
            assert!(view.local_range().is_none());
            let rect = fragment.fragment().rect().unwrap();
            assert_eq!(rect.origin().x, strip.location.x);
            assert_eq!(rect.size(), Size::new(S::from_f64(8.0), S::from_f64(10.0)));
            let line = finalized_lines(&batch).remove(0);
            assert_eq!(line.feedback().advance, 28.0);
            assert_eq!(
                line.control_slots()
                    .find(|slot| slot.control() == fixture.binding.controls[0].occurrence)
                    .unwrap()
                    .level()
                    .get(),
                level
            );
            assert_eq!(
                batch.placed_text()[0].layout().source().text().code_units(),
                value.encode_utf16().collect::<Vec<_>>()
            );
            assert!(
                batch
                    .unrounded_inline_fragments()
                    .iter()
                    .all(|entry| !matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn generated_separator_scope_rectangles_keep_their_own_geometry_when_endpoints_widen() {
    let fixture = Fixture::with_generated_separators(
        "今日は天気です。今日は天気です。今日は天気です。",
        (0, 24),
        text::WhiteSpace::Collapse,
        false,
        false,
        true,
    );
    let zero = fixture.compute(
        InlineBoundaryEdgeOf::ZERO,
        InlineBoundaryEdgeOf::ZERO,
        false,
        Some(500.0),
        false,
        AvailableOf::definite(500.0),
    );
    let widened = fixture.compute(
        InlineBoundaryEdgeOf::try_new(3.0, 5.0).unwrap(),
        InlineBoundaryEdgeOf::try_new(2.0, 3.0).unwrap(),
        false,
        Some(500.0),
        false,
        AvailableOf::definite(500.0),
    );
    let scopes = |batch: &CompletedLayoutBatchOf<u32, f64>| {
        batch
            .unrounded_inline_fragments()
            .iter()
            .filter_map(|entry| {
                if !matches!(entry.owner(), InlineFragmentOwner::Scope(_)) {
                    return None;
                }
                let text::FinalizedFragmentOwner::VirtualSeparator(separator) =
                    entry.fragment().finalized_fragment().view().owner()
                else {
                    panic!("scope geometry belongs to its actual generated separator")
                };
                Some((separator, entry.fragment().rect().unwrap()))
            })
            .collect::<Vec<_>>()
    };
    let before = scopes(&zero);
    let after = scopes(&widened);
    assert!(before.len() > 1);
    assert_eq!(before.len(), after.len());
    for ((first, before), (second, after)) in before.iter().zip(&after) {
        assert_eq!(first, second);
        assert_eq!(after.origin().x, before.origin().x + 8.0);
        assert_eq!(after.origin().y, before.origin().y);
        assert_eq!(after.size(), before.size());
        assert_ne!(after.size().width, output(&widened, 20).size.width);
    }
    assert_eq!(output(&widened, 20).size, Size::new(8.0, 0.0));
    assert_eq!(output(&widened, 21).size, Size::new(5.0, 0.0));
}

#[test]
fn justification_feedback_preserves_one_selected_endpoint_total() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::new("A B", (2, 3), text::WhiteSpace::Collapse, false, false);
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(3.0), S::from_f64(5.0)).unwrap();
        let tree = fixture
            .tree(edge, InlineBoundaryEdgeOf::ZERO, false, Some(48.0), false)
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(48.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    text_align: TextAlign::Justify,
                    text_align_last: TextAlignLast::Justify,
                    text_justification: TextJustification::Spaces,
                    ..NodeInputOf::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![fixture.binding.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 0, request(48.0)).unwrap();
        let line = finalized_lines(&batch).remove(0);
        qualify(&line);
        assert_eq!(line.feedback().advance, 48.0);
        assert_eq!(line.measurements().fit().advance(), 48.0);
        let start = fixture
            .binding
            .controls
            .iter()
            .find(|row| row.node == 20)
            .unwrap();
        assert_eq!(
            line.selected_line()
                .request()
                .controls
                .iter()
                .find(|row| row.occurrence == start.occurrence)
                .unwrap()
                .boundary
                .unwrap()
                .advance(),
            8.0
        );
        assert_eq!(
            line.control_slots()
                .find(|row| row.control() == start.occurrence)
                .unwrap()
                .inline_bounds(),
            (30.0, 38.0)
        );
        assert_eq!(output(&batch, 20).location.x, S::from_f64(30.0));
        assert_eq!(output(&batch, 20).size.width, S::from_f64(8.0));
    }
    lane::<f32>();
    lane::<f64>();
}
