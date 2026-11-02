// Intended child of grid_tests/scroll_composition_tests.rs.
// Actual native measured leaf under genuine flattened-subgrid/Lanes owners.
use super::*;
use crate::LayoutTree;

#[derive(Clone, Copy, Debug)]
enum Form {
    FlattenedMin,
    FlattenedMax,
    DirectLanes,
    NestedLanes,
}
impl Form {
    fn nested(self) -> bool {
        !matches!(self, Self::DirectLanes)
    }
    fn display(self) -> Display {
        match self {
            Self::FlattenedMin | Self::FlattenedMax => Display::Grid,
            Self::DirectLanes | Self::NestedLanes => Display::GridLanes,
        }
    }
    fn preferred_width<S: LayoutScalar>(self) -> PreferredSizeOf<S> {
        match self {
            Self::FlattenedMin => PreferredSizeOf::MIN_CONTENT,
            _ => PreferredSizeOf::MAX_CONTENT,
        }
    }
    fn column<S: LayoutScalar>(self) -> TrackComponentOf<S> {
        // The min-content root has a real MAX_CONTENT track: the test does
        // not demand positive AUTO minimum merely because content exists.
        match self {
            Self::FlattenedMax => TrackComponentOf::AUTO,
            _ => TrackComponentOf::MAX_CONTENT,
        }
    }
}
fn s<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn border<S: LayoutScalar>() -> Size<S> {
    Size::new(s(20.0), s(30.0))
}
fn natural<S: LayoutScalar>() -> Size<S> {
    Size::new(s(80.0), s(90.0))
}
fn source<S: LayoutScalar>(form: Form, overflow: Overflow) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [if form.nested() { 1 } else { 2 }])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: form.display(),
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(form.preferred_width(), PreferredSizeOf::px(s(30.0))),
                grid_template_columns: vec![form.column()],
                grid_template_rows: vec![TrackComponentOf::px(s(30.0))],
                grid_auto_flow: GridAutoFlow::Row,
                grid_flow_tolerance: GridFlowToleranceOf::Length(LengthOf::ZERO),
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Start),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
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
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .measure(2, natural());
    if form.nested() {
        tree = tree.children(1, [2]).style(
            1,
            NodeInputOf {
                display: Display::Grid,
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                    Vec::new(),
                ))],
                grid_template_rows: vec![TrackComponentOf::px(s(30.0))],
                grid_column: GridPlacement::try_lines(1, -1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                gap: Size::new(LengthOf::ZERO, LengthOf::ZERO),
                justify_items: Some(AlignItems::Stretch),
                align_items: Some(AlignItems::Start),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}
fn qualify_leaf<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) {
    let style = tree.node_input(2);
    assert!(style.size.width.is_auto() && style.size.height.is_auto());
    assert!(tree.has_leaf_measurement(2));
    assert_eq!(tree.child_count(2), 0);
    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
        let output = crate::measurement::compute_tree_leaf(
            tree,
            2,
            ComputeInputOf::for_child(
                RunMode::ComputeSize,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(None, Some(s(30.0))),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::Grid,
                ),
                Size::new(available, AvailableOf::MAX_CONTENT),
            ),
            style,
        )
        .expect("actual native min/max leaf query succeeds");
        assert_eq!(output.size, border());
        assert_eq!(output.content_size, natural());
        assert_eq!(output.baselines(), BaselinesOf::NONE);
        assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
    }
}
fn qualify_nested<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>, form: Form) {
    if !form.nested() {
        return;
    }
    assert_eq!(tree.children(0).collect::<Vec<_>>(), vec![1]);
    assert_eq!(tree.children(1).collect::<Vec<_>>(), vec![2]);
    let root = tree.node_input(0);
    let wrapper = tree.node_input(1);
    // Existing owning helper projects these actual NodeInputs and runs the
    // real eligibility/mapping owner. No synthetic subgrid report is supplied.
    let column = subgrid_axis_report(root, wrapper, GridAxisKind::Column);
    let row = subgrid_axis_report(root, wrapper, GridAxisKind::Row);
    assert!(
        column.requested() && column.can_inherit(),
        "{form:?}: genuine columns subgrid must be eligible"
    );
    assert!(
        !row.requested() && !row.can_inherit(),
        "{form:?}: wrapper owns its ordinary fixed30 row"
    );
    assert_eq!(
        wrapper.grid_column,
        GridPlacement::try_lines(1, -1).unwrap()
    );
}
fn contributes<S: LayoutScalar>(form: Form, overflow: Overflow) {
    let tree = source::<S>(form, overflow);
    qualify_leaf(&tree);
    qualify_nested(&tree, form);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .expect("actual flattened/native Lanes Grid completes");
    let root = fri08_c01_placement_output(&batch, 0);
    assert_eq!(
        root.size,
        border(),
        "{form:?} {overflow:?}: actual content contribution survives zero automatic minimum"
    );
    assert_eq!(root.location, Point::ZERO);
    if form.nested() {
        let wrapper = fri08_c01_placement_output(&batch, 1);
        assert_eq!(
            wrapper.size,
            border(),
            "actual inherited column plus owned fixed30 row"
        );
        assert_eq!(wrapper.location, Point::ZERO);
    }
    let leaf = fri08_c01_placement_output(&batch, 2);
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
            .expect("actual performed measured leaf retains own geometry");
        assert_eq!(
            geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, border()).unwrap()
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            ScrollRectOf::try_new(Point::ZERO, natural()).unwrap()
        );
    }
    // Root/wrapper overflow propagation is intentionally not reconstructed
    // from the normal contribution; actual own leaf overflow is separate.
}
macro_rules! cohort {
    ($name:ident, $scalar:ty, $form:expr, $overflow:expr) => {
        #[test]
        fn $name() {
            contributes::<$scalar>($form, $overflow);
        }
    };
}
cohort!(
    flattened_min_hidden_f32,
    f32,
    Form::FlattenedMin,
    Overflow::Hidden
);
cohort!(
    flattened_min_hidden_f64,
    f64,
    Form::FlattenedMin,
    Overflow::Hidden
);
cohort!(
    flattened_min_visible_f32,
    f32,
    Form::FlattenedMin,
    Overflow::Visible
);
cohort!(
    flattened_min_visible_f64,
    f64,
    Form::FlattenedMin,
    Overflow::Visible
);
cohort!(
    flattened_min_clip_f32,
    f32,
    Form::FlattenedMin,
    Overflow::Clip
);
cohort!(
    flattened_min_clip_f64,
    f64,
    Form::FlattenedMin,
    Overflow::Clip
);
cohort!(
    flattened_max_hidden_f32,
    f32,
    Form::FlattenedMax,
    Overflow::Hidden
);
cohort!(
    flattened_max_hidden_f64,
    f64,
    Form::FlattenedMax,
    Overflow::Hidden
);
cohort!(
    flattened_max_visible_f32,
    f32,
    Form::FlattenedMax,
    Overflow::Visible
);
cohort!(
    flattened_max_visible_f64,
    f64,
    Form::FlattenedMax,
    Overflow::Visible
);
cohort!(
    flattened_max_clip_f32,
    f32,
    Form::FlattenedMax,
    Overflow::Clip
);
cohort!(
    flattened_max_clip_f64,
    f64,
    Form::FlattenedMax,
    Overflow::Clip
);
cohort!(
    direct_lanes_hidden_f32,
    f32,
    Form::DirectLanes,
    Overflow::Hidden
);
cohort!(
    direct_lanes_hidden_f64,
    f64,
    Form::DirectLanes,
    Overflow::Hidden
);
cohort!(
    direct_lanes_visible_f32,
    f32,
    Form::DirectLanes,
    Overflow::Visible
);
cohort!(
    direct_lanes_visible_f64,
    f64,
    Form::DirectLanes,
    Overflow::Visible
);
cohort!(
    direct_lanes_clip_f32,
    f32,
    Form::DirectLanes,
    Overflow::Clip
);
cohort!(
    direct_lanes_clip_f64,
    f64,
    Form::DirectLanes,
    Overflow::Clip
);
cohort!(
    nested_lanes_hidden_f32,
    f32,
    Form::NestedLanes,
    Overflow::Hidden
);
cohort!(
    nested_lanes_hidden_f64,
    f64,
    Form::NestedLanes,
    Overflow::Hidden
);
cohort!(
    nested_lanes_visible_f32,
    f32,
    Form::NestedLanes,
    Overflow::Visible
);
cohort!(
    nested_lanes_visible_f64,
    f64,
    Form::NestedLanes,
    Overflow::Visible
);
cohort!(
    nested_lanes_clip_f32,
    f32,
    Form::NestedLanes,
    Overflow::Clip
);
cohort!(
    nested_lanes_clip_f64,
    f64,
    Form::NestedLanes,
    Overflow::Clip
);

#[path = "nested_lanes_candidates.rs"]
mod nested_candidates;
