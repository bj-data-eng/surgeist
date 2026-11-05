// Intended private child of grid_tests/scroll_composition_tests.rs.
// Real measured leaf: natural80x90, AUTO preferred sizes, maxima20x30.
use super::*;
use crate::LayoutTree;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn border_size<S: LayoutScalar>() -> Size<S> {
    Size::new(scalar(20.0), scalar(30.0))
}
fn natural_size<S: LayoutScalar>() -> Size<S> {
    Size::new(scalar(80.0), scalar(90.0))
}

fn tree<S: LayoutScalar>(cyclic: bool, overflow: Overflow) -> PublicLayoutTreeOf<S> {
    let track = if cyclic {
        TrackComponentOf::percent(S::from_f64(1.0))
    } else {
        TrackComponentOf::AUTO
    };
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                grid_template_columns: vec![track.clone()],
                grid_template_rows: vec![track],
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO),
                max_size: Size::new(MaxSizeOf::px(scalar(20.0)), MaxSizeOf::px(scalar(30.0))),
                overflow: computed_overflow(overflow, overflow),
                ..NodeInputOf::default()
            },
        )
        // Actual LayoutTree::measure_leaf provider, not a stored ComputeOutput.
        .measure(1, natural_size())
}
fn leaf_query<S: LayoutScalar>(mode: RunMode) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::Grid,
        ),
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}
fn qualify_actual_leaf<S: LayoutScalar>(source: &PublicLayoutTreeOf<S>, overflow: Overflow) {
    let style = source.node_input(1);
    assert!(style.size.width.is_auto());
    assert!(style.size.height.is_auto());
    assert!(source.has_leaf_measurement(1));
    let query = leaf_query(RunMode::ComputeSize);
    assert_eq!(query.known(), Size::NONE);
    // Same actual tree/style/provider used by compute_layout below. AUTO
    // avoids the fully-definite ComputeSize shortcut that can omit content.
    let measured = crate::measurement::compute_tree_leaf(source, 1, query, style)
        .expect("real constrained natural leaf query succeeds");
    assert_eq!(measured.size, border_size());
    assert_eq!(measured.content_size, natural_size());
    assert_eq!(measured.baselines(), BaselinesOf::NONE);
    assert_eq!(measured.named_baselines, NamedBaselinesOf::unknown());
    if overflow == Overflow::Visible {
        let performed = crate::measurement::compute_tree_leaf(
            source,
            1,
            leaf_query(RunMode::PerformLayout),
            style,
        )
        .expect("real performed leaf retains its visible natural overflow");
        assert_eq!(performed.size, border_size());
        assert_eq!(performed.content_size, natural_size());
        let geometry = performed
            .scroll_geometry
            .expect("actual performed leaf scroll geometry");
        assert_eq!(
            geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, border_size()).unwrap()
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            ScrollRectOf::try_new(Point::ZERO, natural_size()).unwrap()
        );
    }
}
fn assert_actual_batch<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    overflow: Overflow,
    cyclic: bool,
) {
    let root = fri08_c01_placement_output(batch, 0);
    let child = fri08_c01_placement_output(batch, 1);
    // Qualify the actual performed child before testing the root's normal
    // intrinsic size: both leaf channels remain and independent.
    assert_eq!(
        child.location,
        Point::ZERO,
        "cyclic={cyclic} {overflow:?} actual leaf origin"
    );
    assert_eq!(
        child.size,
        border_size(),
        "cyclic={cyclic} {overflow:?} actual constrained leaf border"
    );
    assert_eq!(
        (child.border, child.padding, child.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    if overflow == Overflow::Visible {
        assert_eq!(child.content_size, natural_size());
        let child_geometry = child.scroll_geometry.expect("actual visible leaf geometry");
        assert_eq!(
            child_geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, border_size()).unwrap()
        );
        assert_eq!(
            child_geometry.scrollable_overflow(),
            ScrollRectOf::try_new(Point::ZERO, natural_size()).unwrap()
        );
    }
    // Cyclic100% tracks are treated as AUTO for intrinsic sizing. The sole
    // child's queried constrained20x30 border supplies both contributions.
    // Then100% resolves against that established20x30 normal content box.
    assert_eq!(
        root.size,
        border_size(),
        "cyclic={cyclic} {overflow:?}: normal Grid size must not borrow native visual overflow80x90"
    );
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(
        (root.border, root.padding, root.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    if overflow == Overflow::Visible {
        assert_eq!(root.content_size, natural_size());
        let geometry = root
            .scroll_geometry
            .expect("Grid retains actual visible descendant overflow");
        assert_eq!(
            geometry.border_box(),
            ScrollRectOf::try_new(Point::ZERO, border_size()).unwrap()
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            ScrollRectOf::try_new(Point::ZERO, natural_size()).unwrap()
        );
    }
}
fn cyclic_border<S: LayoutScalar>(overflow: Overflow) {
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
    // Compare AUTO and percentage tracks for non-scrollable overflow. Hidden
    // is qualified directly through the actual cyclic-track operation below.
    if matches!(overflow, Overflow::Visible | Overflow::Clip) {
        let control = tree::<S>(false, overflow);
        qualify_actual_leaf(&control, overflow);
        let prior =
            compute_layout(&control, 0, request).expect("actual AUTO-track control completes");
        assert_actual_batch(&prior, overflow, false);
    }
    let cyclic = tree::<S>(true, overflow);
    qualify_actual_leaf(&cyclic, overflow);
    let actual =
        compute_layout(&cyclic, 0, request).expect("actual cyclic-percentage Grid completes");
    assert_actual_batch(&actual, overflow, true);
}

#[test]
fn cyclic_percent_uses_queried_border_with_visible_overflow_f32() {
    cyclic_border::<f32>(Overflow::Visible);
}
#[test]
fn cyclic_percent_uses_queried_border_with_visible_overflow_f64() {
    cyclic_border::<f64>(Overflow::Visible);
}
#[test]
fn cyclic_percent_clip_control_uses_queried_border_f32() {
    cyclic_border::<f32>(Overflow::Clip);
}
#[test]
fn cyclic_percent_clip_control_uses_queried_border_f64() {
    cyclic_border::<f64>(Overflow::Clip);
}
#[test]
fn cyclic_percent_hidden_control_uses_queried_border_f32() {
    cyclic_border::<f32>(Overflow::Hidden);
}
#[test]
fn cyclic_percent_hidden_control_uses_queried_border_f64() {
    cyclic_border::<f64>(Overflow::Hidden);
}
