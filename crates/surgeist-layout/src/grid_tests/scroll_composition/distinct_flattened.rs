//! Distinct native leaf purposes through a real inherited-column subgrid.
use super::*;
use crate::test_support::readback::{self, Phase};

fn flattened_source<S: LayoutScalar>(
    space: f64,
    cap: GrowthCap,
    overflow: Overflow,
) -> DistinctLeaf<S> {
    let n = S::from_f64;
    let mut host = source::<S>(space + 7.0, cap, overflow);
    host.source = host
        .source
        .children(0, [2])
        .children(2, [1])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(n(space + 7.0)),
                    PreferredSizeOf::px(n(10.0)),
                ),
                grid_template_columns: vec![TrackComponentOf::px(n(7.0)), cap.track()],
                grid_template_rows: vec![TrackComponentOf::px(n(10.0))],
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Grid,
                min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::AUTO),
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                    Vec::new(),
                ))],
                grid_template_rows: vec![TrackComponentOf::px(n(10.0))],
                grid_column: GridPlacement::try_lines(2, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Stretch),
                ..NodeInputOf::default()
            },
        );
    host
}

fn native_output<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
) -> NodeOutputOf<S> {
    let mut matching = entries.iter().filter(|entry| entry.node() == node);
    let result = matching
        .next()
        .expect("real flattened node has output")
        .output();
    assert!(matching.next().is_none());
    result
}

fn scroll_payload<S: LayoutScalar>(output: NodeOutputOf<S>, overflow: Overflow, extent: Size<S>) {
    let rect = |size| ScrollRectOf::try_new(Point::ZERO, size).unwrap();
    let scroll = output
        .scroll_geometry
        .expect("accepted node owns scroll geometry");
    assert_eq!(
        scroll.flow_axes(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
    );
    assert_eq!(scroll.border_box(), rect(output.size));
    assert_eq!(scroll.padding_box(), rect(output.size));
    assert_eq!(scroll.content_box(), rect(output.size));
    assert_eq!(scroll.scrollport(), rect(output.size));
    assert_eq!(scroll.scrollable_overflow(), rect(extent));
    assert_eq!(scroll.used_overflow_x(), overflow);
    assert_eq!(scroll.used_overflow_y(), overflow);
    let clip = scroll.overflow_clip();
    if overflow == Overflow::Visible {
        assert_eq!(clip.x(), None);
        assert_eq!(clip.y(), None);
    } else {
        let x = clip.x().expect("Hidden/Clip has an actual clip");
        let y = clip.y().expect("Hidden/Clip has an actual clip");
        assert_eq!((x.minimum(), x.maximum()), (S::ZERO, output.size.width));
        assert_eq!((y.minimum(), y.maximum()), (S::ZERO, output.size.height));
    }
    // Only Hidden exposes a scrolling range in this closed fixture. The
    // increasing LTR axes retain actual own content beyond the viewport;
    // Visible and Clip remain non-scrolling even if their raw extent is larger.
    let range_max = if overflow == Overflow::Hidden {
        Size::new(
            (extent.width - output.size.width).max(S::ZERO),
            (extent.height - output.size.height).max(S::ZERO),
        )
    } else {
        Size::ZERO
    };
    let range = scroll.physical_range();
    assert_eq!(
        (range.x().minimum(), range.x().maximum()),
        (S::ZERO, range_max.width)
    );
    assert_eq!(
        (range.y().minimum(), range.y().maximum()),
        (S::ZERO, range_max.height)
    );
    assert_eq!(scroll.scrollbar_size(), Size::ZERO);
    let gutters = scroll.gutters();
    assert_eq!(
        (
            gutters.top(),
            gutters.right(),
            gutters.bottom(),
            gutters.left()
        ),
        (None, None, None, None)
    );
    assert_eq!(scroll.resolved_scroll_padding(), Edges::ZERO);
    assert_eq!(scroll.optimal_viewing_region(), rect(output.size));
    assert_eq!(scroll.target().border_box(), rect(output.size));
}

fn flattened<S: LayoutScalar>(space: f64, cap: GrowthCap, overflow: Overflow, track: f64) {
    let n = S::from_f64;
    let host = flattened_source::<S>(space, cap, overflow);
    // Reuse the SAME faithful native size provider and independently run the
    // actual leaf owner. It returns MinContent40/MaxContent80; only
    // leaf1 is measured. Neither Grid nor subgrid output is supplied.
    qualify_actual_native_queries(&host);
    assert_eq!(Traverse::children(&host, 0).collect::<Vec<_>>(), [2]);
    assert_eq!(Traverse::children(&host, 2).collect::<Vec<_>>(), [1]);
    assert_eq!(Traverse::child_count(&host, 1), 0);
    assert_eq!(
        host.node_input(0).grid_template_columns,
        vec![TrackComponentOf::px(n(7.0)), cap.track()]
    );
    let wrapper = host.node_input(2);
    let inherited = subgrid_axis_report(host.node_input(0), wrapper, GridAxisKind::Column);
    let standalone = subgrid_axis_report(host.node_input(0), wrapper, GridAxisKind::Row);
    assert!(inherited.requested() && inherited.can_inherit());
    assert!(!standalone.requested() && !standalone.can_inherit());
    assert_eq!(wrapper.grid_column, GridPlacement::try_lines(2, 3).unwrap());
    assert_eq!(
        wrapper.grid_template_rows,
        vec![TrackComponentOf::px(n(10.0))]
    );
    // The wrapper's explicit zero minimum excludes an additional automatic
    // wrapper floor. The descendant still owns Hidden0 versus Clip40.
    assert_eq!(wrapper.min_size.width, MinSizeOf::px(S::ZERO));
    for node in [0, 2, 1] {
        let style = host.node_input(node);
        assert_eq!(style.margin, Edges::all(LengthAutoOf::px(S::ZERO)));
        assert_eq!(style.border, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(style.padding, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(host.has_leaf_measurement(node), node == 1);
    }
    let request = {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::NonNegativeFiniteOf::new(n(space + 7.0)).unwrap(),
                ),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(n(10.0)).unwrap()),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            definite,
        )
    };
    let batch = compute_layout(&host, 0, request).expect("actual flattened Grid completes");
    assert!(batch.placed_text().is_empty());
    for entries in [
        readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
        readback::continuous_custody(&batch, Phase::Final).as_slice(),
    ] {
        assert_eq!(entries.len(), 3);
        let root = native_output(entries, 0);
        let wrapper = native_output(entries, 2);
        let leaf = native_output(entries, 1);
        for (output, location, size) in [
            (root, Point::ZERO, Size::new(n(space + 7.0), n(10.0))),
            (
                wrapper,
                Point::new(n(7.0), S::ZERO),
                Size::new(n(track), n(10.0)),
            ),
            (leaf, Point::ZERO, Size::new(n(track), n(10.0))),
        ] {
            assert_eq!(output.source_index, SourceIndex::new(0));
            assert_eq!(output.location, location);
            assert_eq!(output.size, size);
            assert_eq!(
                (output.margin, output.border, output.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO)
            );
            assert_eq!(output.content_box_size(), size);
        }
        let root_extent = Size::new(n((space + 7.0).max(track + 7.0)), n(10.0));
        assert_eq!(root.content_size, root_extent);
        assert_eq!(wrapper.content_size, wrapper.size);
        // PerformLayout deliberately clears provider known_content_size.
        // The actual allocated border is still track, but the same provider's
        // Normal/Definite(track) content is min(max(track,40),80). In the
        // Hidden20 shortage this preserves40 of own content inside border20.
        let measured_extent = Size::new(n(track.clamp(40.0, 80.0)), n(10.0));
        assert_eq!(leaf.content_size, measured_extent);
        // Visible root keeps a real40-wide overflow at shortage; it exposes
        // no scrolling range. Wrapper and leaf local coordinates stay distinct
        // from the actual7px ancestor track prefix.
        scroll_payload(root, Overflow::Visible, root_extent);
        scroll_payload(wrapper, Overflow::Visible, wrapper.size);
        scroll_payload(leaf, overflow, measured_extent);
    }
}

macro_rules! cohort {
    ($name:ident, $scalar:ty, $space:expr, $cap:expr, $overflow:expr, $track:expr) => {
        #[test]
        fn $name() {
            flattened::<$scalar>($space, $cap, $overflow, $track);
        }
    };
}
// Grid1 minimum0 leaves room20; non-scroll Clip's content minimum is40.
cohort!(
    hidden_minimum_zero_shortage_f32,
    f32,
    20.,
    GrowthCap::MinContent,
    Overflow::Hidden,
    20.
);
cohort!(
    hidden_minimum_zero_shortage_f64,
    f64,
    20.,
    GrowthCap::MinContent,
    Overflow::Hidden,
    20.
);
cohort!(
    clip_content_minimum_shortage_f32,
    f32,
    20.,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.
);
cohort!(
    clip_content_minimum_shortage_f64,
    f64,
    20.,
    GrowthCap::MinContent,
    Overflow::Clip,
    40.
);
// At room60, the distinct MinContent and MaxContent limits give40 and60.
cohort!(
    hidden_min_content_cap_f32,
    f32,
    60.,
    GrowthCap::MinContent,
    Overflow::Hidden,
    40.
);
cohort!(
    hidden_min_content_cap_f64,
    f64,
    60.,
    GrowthCap::MinContent,
    Overflow::Hidden,
    40.
);
cohort!(
    hidden_max_content_growth_f32,
    f32,
    60.,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    60.
);
cohort!(
    hidden_max_content_growth_f64,
    f64,
    60.,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    60.
);
cohort!(
    hidden_max_content_cap_f32,
    f32,
    100.,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    80.
);
cohort!(
    hidden_max_content_cap_f64,
    f64,
    100.,
    GrowthCap::MaxContent,
    Overflow::Hidden,
    80.
);
