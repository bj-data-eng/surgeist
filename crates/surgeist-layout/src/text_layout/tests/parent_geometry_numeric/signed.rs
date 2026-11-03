use super::*;

fn signed_endpoint<S: LayoutScalar>(
    frame: FrameCase,
    kind: InlineBoundaryKind,
    parent: FlowAxes,
    margin: f64,
    border: f64,
    padding: f64,
) -> InlineBoundaryInputOf<S> {
    InlineBoundaryInputOf::box_edge(
        kind,
        InlineMetricsOf::try_new(InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap(),
        InlineBoundaryEdgeOf::try_new(
            S::from_f64(margin),
            S::from_f64(border),
            S::from_f64(padding),
        )
        .unwrap(),
        parent,
    )
    .with_writing_mode(frame.mode)
    .with_direction(frame.direction)
}

fn signed_tree<S: LayoutScalar>(frame: FrameCase, nested: bool) -> PublicLayoutTreeOf<S> {
    let mut tree = fixture::<S>(frame, nested);
    let parent = FlowAxes::new(
        frame.mode,
        if nested {
            other(frame.direction)
        } else {
            frame.direction
        },
    );
    let (start, end) = if nested { (2, 4) } else { (1, 5) };
    tree.insert_input(
        start,
        LayoutInputOf::inline_boundary(signed_endpoint(
            frame,
            InlineBoundaryKind::Start,
            parent,
            3.0,
            2.0,
            1.0,
        )),
    );
    tree.insert_input(
        end,
        LayoutInputOf::inline_boundary(signed_endpoint(
            frame,
            InlineBoundaryKind::End,
            parent,
            -1.0,
            1.0,
            1.0,
        )),
    );
    tree
}

fn geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    frame: FrameCase,
    nested: bool,
) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    let positive_side = matches!(frame.start, Side::Left | Side::Top);
    let inline_extent = if nested { 22.0 } else { 17.0 };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let output = |node| {
            entries
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        assert_eq!(
            output(0).size,
            if frame.mode == WritingMode::HorizontalTb {
                Size::new(S::from_f64(inline_extent), S::from_f64(10.0))
            } else {
                Size::new(S::from_f64(10.0), S::from_f64(inline_extent))
            }
        );
        let body_inline = if nested {
            if positive_side { 2.0 } else { 10.0 }
        } else if positive_side {
            6.0
        } else {
            1.0
        };
        assert_eq!(
            output(3).location,
            if frame.mode == WritingMode::HorizontalTb {
                Point::new(S::from_f64(body_inline), S::ZERO)
            } else {
                Point::new(S::ZERO, S::from_f64(body_inline))
            }
        );
        assert_eq!(output(3).size, Size::splat(S::from_f64(10.0)));
        if nested {
            // Pstart1 + child(Start6, Native10, End1) + Pend4 =22.
            // The child uses P's opposite actual direction, never the formatter.
            let positions = if positive_side {
                [0.0, 12.0, 0.0, 18.0]
            } else {
                [21.0, 7.0, 20.0, 0.0]
            };
            for (index, (node, extent, side, margin, border, padding)) in [
                (1, 1.0, frame.start, 0.0, 1.0, 0.0),
                (2, 3.0, frame.start.opposite(), 3.0, 2.0, 1.0),
                (4, 2.0, frame.start, -1.0, 1.0, 1.0),
                (5, 4.0, frame.start.opposite(), 0.0, 1.0, 3.0),
            ]
            .into_iter()
            .enumerate()
            {
                check_strip(
                    output(node),
                    frame,
                    positions[index],
                    extent,
                    side,
                    border,
                    padding,
                );
                assert_eq!(output(node).margin, side.edges(margin));
            }
            for (rank, node) in [1, 2, 3, 4, 5].into_iter().enumerate() {
                assert_eq!(output(node).source_index.get(), rank);
            }
        } else {
            // Start6 + actual Native10 + End1 =17. Its Native occupied box
            // stays nonnegative in either direction, independently of BP overhang.
            let positions = if positive_side {
                [3.0, 16.0]
            } else {
                [11.0, -1.0]
            };
            for (index, (node, extent, side, margin, border, padding)) in [
                (1, 3.0, frame.start, 3.0, 2.0, 1.0),
                (5, 2.0, frame.start.opposite(), -1.0, 1.0, 1.0),
            ]
            .into_iter()
            .enumerate()
            {
                check_strip(
                    output(node),
                    frame,
                    positions[index],
                    extent,
                    side,
                    border,
                    padding,
                );
                assert_eq!(output(node).margin, side.edges(margin));
            }
            for (rank, node) in [1, 3, 5].into_iter().enumerate() {
                assert_eq!(output(node).source_index.get(), rank);
            }
        }
    }
}

fn frames<S: LayoutScalar>(nested: bool) {
    for frame in FRAMES {
        let tree = signed_tree::<S>(frame, nested);
        let prior = run(&tree, nested);
        geometry(&prior, frame, nested);
        let retry = run(&tree, nested);
        geometry(&retry, frame, nested);
        geometry(&prior, frame, nested);
    }
}

#[test]
fn signed_numeric_edges_keep_physical_strip_components_and_actual_flow_in_both_lanes() {
    frames::<f32>(false);
    frames::<f64>(false);
}

#[test]
fn signed_numeric_child_edges_use_their_real_opposing_parent_in_both_lanes() {
    frames::<f32>(true);
    frames::<f64>(true);
}
