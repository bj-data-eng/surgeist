//! Actual intrinsic container and protected native sub-one flex track geometry.
use super::*;
use crate::test_support::readback::{self, Phase};

fn three_track_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == node);
    let output = matching
        .next()
        .expect("every actual native box is published")
        .output();
    assert!(
        matching.next().is_none(),
        "one geometry entry per actual node"
    );
    output
}

fn assert_native_three_track_flex_span<S: LayoutScalar>() {
    let n = S::from_f64;
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3, 4])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                grid_template_columns: vec![
                    track_component_flex(n(0.2)),
                    track_component_flex(n(0.3)),
                    track_component_flex(n(0.5)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(n(10.0)); 2],
                gap: Size::splat(LengthOf::px(S::ZERO)),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::px(n(60.0)), PreferredSizeOf::AUTO),
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                justify_self: Some(AlignItems::Stretch),
                align_self: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        );
    let mut tree = tree;
    for (node, start) in [(2, 1), (3, 2), (4, 3)] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                justify_self: Some(AlignItems::Stretch),
                align_self: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        );
    }
    assert_eq!(
        Traverse::children(&tree, 0).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        tree.node_input(0).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    assert_eq!(
        tree.node_input(0).grid_template_columns,
        vec![
            track_component_flex(n(0.2)),
            track_component_flex(n(0.3)),
            track_component_flex(n(0.5)),
        ]
    );
    assert_eq!(
        tree.node_input(0).grid_template_rows,
        vec![TrackComponentOf::px(n(10.0)); 2]
    );
    assert_eq!(tree.node_input(0).gap, Size::splat(LengthOf::px(S::ZERO)));
    for node in 0..=4 {
        assert!(
            !tree.has_leaf_measurement(node),
            "native layout has no supplied measurement"
        );
        let input = tree.node_input(node);
        assert_eq!(input.margin, Edges::all(LengthAutoOf::px(S::ZERO)));
        assert_eq!(input.border, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(input.padding, Edges::all(LengthOf::px(S::ZERO)));
        if node != 0 {
            assert_eq!(input.display, Display::Flex);
            assert_eq!(Traverse::child_count(&tree, node), 0);
        }
    }
    assert_eq!(tree.node_input(1).size.width, PreferredSizeOf::px(n(60.0)));
    assert_eq!(
        tree.node_input(1).grid_column,
        GridPlacement::try_lines(1, 3).unwrap()
    );
    assert_eq!(
        tree.node_input(1).grid_row,
        GridPlacement::try_lines(1, 2).unwrap()
    );
    for (node, start) in [(2, 1), (3, 2), (4, 3)] {
        assert_eq!(
            tree.node_input(node).size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(
            tree.node_input(node).grid_column,
            GridPlacement::try_lines(start, start + 1).unwrap()
        );
        assert_eq!(
            tree.node_input(node).grid_row,
            GridPlacement::try_lines(2, 3).unwrap()
        );
    }
    // The item's definite preferred width is not Auto or percentage-dependent:
    // its Grid minimum contribution and max-content contribution are both60
    // with zero margins. Real isolated MIN/MAX requests independently execute
    // this same native Flex box; no supplied measurement answers are involved.
    for available in [
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
    ] {
        let control = compute_layout(&tree, 1, {
            let axes = Size::splat(available);
            let definite = axes.map(|axis| axis.space().definite_size());
            crate::LayoutRootRequestOf::standalone(
                SizingRequestOf::new(
                    axes,
                    OrthogonalFallbackOf::new(
                        definite,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                definite,
            )
        })
        .expect("the real fixed60 native item completes both extremal controls");
        for entries in [
            readback::continuous_custody(&control, Phase::Unrounded).as_slice(),
            readback::continuous_custody(&control, Phase::Final).as_slice(),
        ] {
            assert_eq!(entries.len(), 1);
            let actual = three_track_output(entries, 1);
            assert_eq!(actual.source_index, SourceIndex::new(0));
            assert_eq!(actual.location, Point::ZERO);
            assert_eq!(actual.size.width, n(60.0));
            assert_eq!(actual.margin, Edges::ZERO);
            assert_eq!(actual.border, Edges::ZERO);
            assert_eq!(actual.padding, Edges::ZERO);
        }
    }
    // This is normal layout with unbounded physical space. Its
    // initial intrinsic MAX sizing is followed by normal allocated tracks.
    let request = {
        let axes = Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::Normal,
        ));
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    };
    assert_eq!(
        request.sizing_request().axes(),
        Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::Normal
        ))
    );
    let batch =
        compute_layout(&tree, 0, request).expect("the real three-track native grid completes");
    assert!(
        batch.placed_text().is_empty(),
        "no Text or font producer participates"
    );

    // Grid1 §11.5's below-one spanning rule gives floors27/33/0: the first
    // two factors receive their .2/.3 portions plus equal portions of the
    // unclaimed half. §11.7's indefinite per-track candidates select33,
    // contributing .5*33=16.5 to the intrinsic container size76.5. The
    // completed raw root exposes that accepted container size, not an
    // independent public array of the initial track sizes. The zero third
    // item remains a source incidence, despite intrinsic demand0.
    assert_eq!(n(0.2) + n(0.3), n(0.5));
    assert_eq!(n(0.5) * n(33.0), n(16.5));
    for (published, entries) in [
        (
            false,
            readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
        ),
        (
            true,
            readback::continuous_custody(&batch, Phase::Final).as_slice(),
        ),
    ] {
        assert_eq!(entries.len(), 5);
        for (node, source_index, x, y, raw_width, published_width, height) in [
            (0, 0, 0.0, 0.0, 76.5, 77.0, 20.0),
            (1, 0, 0.0, 0.0, 60.0, 60.0, 10.0),
            (2, 1, 0.0, 10.0, 27.0, 27.0, 10.0),
            (3, 2, 27.0, 10.0, 33.0, 33.0, 10.0),
            (4, 3, 60.0, 10.0, 8.25, 8.0, 10.0),
        ] {
            let output = three_track_output(entries, node);
            assert_eq!(
                output.source_index,
                SourceIndex::new(source_index),
                "node{node} source occurrence"
            );
            assert_eq!(
                output.location,
                Point::new(n(x), n(y)),
                "node{node} actual position"
            );
            assert_eq!(
                output.size,
                Size::new(
                    n(if published {
                        published_width
                    } else {
                        raw_width
                    }),
                    n(height)
                ),
                "node{node} actual size"
            );
            assert_eq!(output.margin, Edges::ZERO);
            assert_eq!(output.border, Edges::ZERO);
            assert_eq!(output.padding, Edges::ZERO);
            assert_eq!(output.content_box_size(), output.size);
        }
        let final_marker = three_track_output(entries, 4);
        assert_eq!(final_marker.source_index, SourceIndex::new(3));
        assert_eq!(final_marker.location, Point::new(n(60.0), n(10.0)));
        assert_eq!(final_marker.size.height, n(10.0));
        assert_eq!(final_marker.margin, Edges::ZERO);
        assert_eq!(final_marker.border, Edges::ZERO);
        assert_eq!(final_marker.padding, Edges::ZERO);
        assert_eq!(final_marker.content_box_size(), final_marker.size);
        // Intrinsic MAX establishes the root76.5 from27/33/16.5. Normal
        // layout reconstructs bases27/33/0 under accepted Definite76.5;
        // freezing the first2 leaves16.5 / max(.5,1), so the third is8.25.
        // No intrinsic third-track allocation is carried as a new base floor.
        let root = three_track_output(entries, 0);
        let end = final_marker.location.x + final_marker.size.width;
        assert_eq!(end, n(if published { 68.0 } else { 68.25 }));
        assert_eq!(root.size.width - end, n(if published { 9.0 } else { 8.25 }));
        // Publication rounds actual corners: round(68.25)-round(60)=8;
        // root round(76.5)=77. Its remaining host space is independently9.
        for node in 0..=4 {
            let output = three_track_output(entries, node);
            let scroll = output
                .scroll_geometry
                .expect("actual native owner publishes its scroll facts");
            assert_eq!(scroll.border_box().origin(), Point::ZERO);
            assert_eq!(scroll.border_box().size(), output.size);
            assert_eq!(scroll.padding_box().origin(), Point::ZERO);
            assert_eq!(scroll.padding_box().size(), output.size);
            assert_eq!(scroll.content_box().origin(), Point::ZERO);
            assert_eq!(scroll.content_box().size(), output.size);
            assert_eq!(scroll.scrollport().origin(), Point::ZERO);
            assert_eq!(scroll.scrollport().size(), output.size);
            assert_eq!(scroll.scrollable_overflow().origin(), Point::ZERO);
            assert_eq!(scroll.scrollable_overflow().size(), output.size);
            assert_eq!(scroll.used_overflow_x(), Overflow::Visible);
            assert_eq!(scroll.used_overflow_y(), Overflow::Visible);
            assert_eq!(scroll.scrollbar_size(), Size::ZERO);
            let range = scroll.physical_range();
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (S::ZERO, S::ZERO)
            );
            assert_eq!(
                (range.y().minimum(), range.y().maximum()),
                (S::ZERO, S::ZERO)
            );
        }
    }
    // This public scroll record is local to the actual unrounded root box;
    // it is not reconstructed from rounded child observations.
    let raw_root = readback::single_box_output(&batch, 0, Phase::Unrounded);
    let scroll = raw_root
        .scroll_geometry
        .expect("the Grid owner publishes its actual scroll geometry");
    let extent = Size::new(n(76.5), n(20.0));
    assert_eq!(scroll.border_box().origin(), Point::ZERO);
    assert_eq!(scroll.border_box().size(), extent);
    assert_eq!(scroll.padding_box().size(), extent);
    assert_eq!(scroll.content_box().size(), extent);
    assert_eq!(scroll.scrollport().size(), extent);
    assert_eq!(scroll.scrollable_overflow().origin(), Point::ZERO);
    assert_eq!(scroll.scrollable_overflow().size(), extent);
    assert_eq!(scroll.used_overflow_x(), Overflow::Visible);
    assert_eq!(scroll.used_overflow_y(), Overflow::Visible);
    let range = scroll.physical_range();
    assert_eq!(
        (range.x().minimum(), range.x().maximum()),
        (S::ZERO, S::ZERO)
    );
    assert_eq!(
        (range.y().minimum(), range.y().maximum()),
        (S::ZERO, S::ZERO)
    );
}

#[test]
fn native_three_flex_tracks_retain_normal_final_space_f32() {
    assert_native_three_track_flex_span::<f32>();
}
#[test]
fn native_three_flex_tracks_retain_normal_final_space_f64() {
    assert_native_three_track_flex_span::<f64>();
}
