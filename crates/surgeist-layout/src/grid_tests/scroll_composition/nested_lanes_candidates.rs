// Intended private child of the existing scroll_composition/subgrid_lanes.rs.
// Uses its scalar/border/actual native leaf-query helpers; no old test changes.
use super::*;

fn two_column_lane_subgrid<S: LayoutScalar>(
    overflow: Overflow,
    indefinite: bool,
) -> PublicLayoutTreeOf<S> {
    let local = if indefinite {
        GridPlacement::try_span(1).unwrap()
    } else {
        GridPlacement::try_line(1).unwrap()
    };
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::GridLanes,
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::px(s(30.0))),
                grid_template_columns: vec![
                    TrackComponentOf::MAX_CONTENT,
                    TrackComponentOf::MAX_CONTENT,
                ],
                grid_template_rows: vec![TrackComponentOf::px(s(30.0))],
                grid_auto_flow: GridAutoFlow::Row,
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                // A real GRID-LANES subgrid: its automatic descendants have the
                // all-possible-parent-track rule in retained Grid3§3.4.1.
                display: Display::GridLanes,
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(s(30.0))),
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                    Vec::new(),
                ))],
                grid_template_rows: vec![TrackComponentOf::px(s(30.0))],
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_auto_flow: GridAutoFlow::Row,
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                box_sizing: BoxSizing::ContentBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                max_size: Size::new(MaxSizeOf::px(s(20.0)), MaxSizeOf::px(s(30.0))),
                overflow: computed_overflow(overflow, overflow),
                grid_column: local,
                ..NodeInputOf::default()
            },
        )
        .measure(2, natural())
}
fn qualify_two_column_source<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>, indefinite: bool) {
    qualify_leaf(tree); // actual min/max ComputeSize results20x30 /80x90
    assert_eq!(tree.children(0).collect::<Vec<_>>(), vec![1]);
    assert_eq!(tree.children(1).collect::<Vec<_>>(), vec![2]);
    let root = tree.node_input(0);
    let wrapper = tree.node_input(1);
    let leaf = tree.node_input(2);
    assert_eq!(root.display, Display::GridLanes);
    assert_eq!(wrapper.display, Display::GridLanes);
    assert_eq!(
        root.grid_template_columns,
        vec![TrackComponentOf::MAX_CONTENT, TrackComponentOf::MAX_CONTENT]
    );
    assert_eq!(wrapper.grid_column, GridPlacement::try_lines(1, 3).unwrap());
    let column = subgrid_axis_report(root, wrapper, GridAxisKind::Column);
    let row = subgrid_axis_report(root, wrapper, GridAxisKind::Row);
    assert!(column.requested() && column.can_inherit());
    assert!(!row.requested() && !row.can_inherit());
    assert_eq!(
        leaf.grid_column,
        if indefinite {
            GridPlacement::try_span(1).unwrap()
        } else {
            GridPlacement::try_line(1).unwrap()
        }
    );
    if indefinite {
        assert!(leaf.grid_column.start().is_none() && leaf.grid_column.end().is_none());
    }
}
fn assert_candidate_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    overflow: Overflow,
    indefinite: bool,
) {
    let width = if indefinite { 40.0 } else { 20.0 };
    let root = fri08_c01_placement_output(batch, 0);
    let wrapper = fri08_c01_placement_output(batch, 1);
    let leaf = fri08_c01_placement_output(batch, 2);
    // One20px contribution at EACH of two active one-track starts gives
    // 20+20=40. The definite control contributes only to its first track.
    assert_eq!(
        root.size,
        Size::new(s(width), s(30.0)),
        "{overflow:?} indefinite={indefinite}: Lane subgrid candidate contributions must survive automatic-minimum zero"
    );
    assert_eq!(wrapper.size, Size::new(s(width), s(30.0)));
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(wrapper.location, Point::ZERO);
    assert_eq!(leaf.location, Point::ZERO);
    assert_eq!(leaf.size, border());
    assert_eq!(
        (leaf.padding, leaf.border, leaf.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    if overflow == Overflow::Visible {
        assert_eq!(leaf.content_size, natural());
        let geometry = leaf
            .scroll_geometry
            .expect("actual performed natural leaf geometry");
        assert_eq!(
            geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, border()).unwrap()
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            ScrollRectOf::try_new(Point::ZERO, natural()).unwrap()
        );
    }
}
fn nested_candidates_contribute<S: LayoutScalar>(overflow: Overflow) {
    // Same nodes/axes/span/provider. An explicitly placed descendant
    // is a separate legitimate20px control, not a fabricated candidate list.
    let definite = two_column_lane_subgrid::<S>(overflow, false);
    qualify_two_column_source(&definite, false);
    let request = {
        let axes = Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
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
    let prior = compute_layout(&definite, 0, request)
        .expect("real definite Lane-subgrid control completes");
    assert_candidate_geometry(&prior, overflow, false);

    let indefinite = two_column_lane_subgrid::<S>(overflow, true);
    qualify_two_column_source(&indefinite, true);
    let actual =
        compute_layout(&indefinite, 0, request).expect("real indefinite Lane-subgrid completes");
    assert_candidate_geometry(&actual, overflow, true);
    assert_candidate_geometry(&prior, overflow, false);
}
#[test]
fn nested_indefinite_hidden_contributes_to_both_columns_f32() {
    nested_candidates_contribute::<f32>(Overflow::Hidden);
}
#[test]
fn nested_indefinite_hidden_contributes_to_both_columns_f64() {
    nested_candidates_contribute::<f64>(Overflow::Hidden);
}
#[test]
fn nested_indefinite_visible_contributes_to_both_columns_f32() {
    nested_candidates_contribute::<f32>(Overflow::Visible);
}
#[test]
fn nested_indefinite_visible_contributes_to_both_columns_f64() {
    nested_candidates_contribute::<f64>(Overflow::Visible);
}
#[test]
fn nested_indefinite_clip_contributes_to_both_columns_f32() {
    nested_candidates_contribute::<f32>(Overflow::Clip);
}
#[test]
fn nested_indefinite_clip_contributes_to_both_columns_f64() {
    nested_candidates_contribute::<f64>(Overflow::Clip);
}
