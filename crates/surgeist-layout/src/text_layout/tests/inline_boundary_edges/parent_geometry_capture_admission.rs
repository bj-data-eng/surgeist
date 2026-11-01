//! Captured endpoint pairs retain detailed parent-mismatch diagnostics.
//! Invalid bases or operations preserve the prior glyph and strip packet.
use super::*;

fn retained_packet<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, fixture: &Fixture) {
    let lines = finalized_lines(batch);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].prepared_text(), fixture.binding.prepared);
    assert_eq!(lines[0].feedback().advance, 20.0);
    assert_eq!(glyphs(&lines[0]), [(35, 4.0, 10.0)]);
    for (node, start, end, right) in [
        (20, 0.0, 1.0, false),
        (23, 1.0, 4.0, false),
        (22, 14.0, 16.0, true),
        (21, 16.0, 20.0, true),
    ] {
        assert_strip(batch, fixture, node, start, end, right);
    }
}
fn captured_pair_mismatch<S: LayoutScalar>() {
    let fixture = Fixture::nested_parent("A", false, false, true);
    source_premises(&fixture, "A", false, true);
    let prior = compute::<S>(&fixture, false, 20.0, true, true);
    retained_packet(&prior, &fixture);
    for contribution_end in [false, true] {
        let mut tree = native_tree::<S>(&fixture, false, 20.0, true);
        let previous = tree.layout_input(23).as_inline_boundary().unwrap();
        let actual = if contribution_end {
            None
        } else {
            Some(axes(false))
        };
        let modified = if contribution_end {
            InlineBoundaryInputOf::contribution(
                previous.kind(),
                previous.metrics(),
                previous.edge(),
            )
            .unwrap()
        } else {
            InlineBoundaryInputOf::box_edge(
                previous.kind(),
                previous.metrics(),
                previous.edge(),
                axes(false),
            )
        }
        .with_writing_mode(previous.writing_mode())
        .with_direction(previous.direction())
        .with_vertical_align(previous.vertical_align());
        tree.insert_input(23, LayoutInputOf::inline_boundary(modified));
        for node in [20, 22, 23, 21] {
            let boundary = tree.layout_input(node).as_inline_boundary().unwrap();
            assert_eq!(
                (boundary.writing_mode(), boundary.direction()),
                (WritingMode::HorizontalTb, Direction::Ltr)
            );
        }
        // The unchanged genuine C Start22 establishes the actual RTL parent
        // declaration; End23 is its authenticated captured peer, never an ID-
        // or equal-source-cut guess. This producer owns the detailed variant.
        assert_eq!(
            tree.layout_input(22)
                .as_inline_boundary()
                .unwrap()
                .parent_edge_axes(),
            Some(axes(true))
        );
        assert_eq!(modified.parent_edge_axes(), actual);
        let error = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![fixture.binding.clone()],
            wrapping(&fixture, true),
            Vec::new(),
        )
        .err()
        .expect("the captured pair cannot disagree about its parent operation/basis");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(23));
        assert_eq!(error.operation(), LayoutOperation::TextCapture);
        assert!(
            matches!(error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineBoundaryParentMismatch {
                expected,actual:reported_actual,
            }) if *expected==Some(axes(true)) && *reported_actual==actual),
            "{error:?}"
        );
        let retry = compute::<S>(&fixture, false, 20.0, true, true);
        retained_packet(&retry, &fixture);
        retained_packet(&prior, &fixture);
    }
}
#[test]
fn actual_captured_pair_retains_detailed_parent_mismatch_f32() {
    captured_pair_mismatch::<f32>();
}
#[test]
fn actual_captured_pair_retains_detailed_parent_mismatch_f64() {
    captured_pair_mismatch::<f64>();
}
