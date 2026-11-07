//! Grid selects a resolved reference in its own alignment-axis context.
//! All geometry cases execute font-free Native boxes/paragraphs.
use super::*;
use crate::geometry::LogicalEdgesOf;
use crate::output::{NamedBaselineDemand, NamedBaselineSet};
use crate::test_support::readback::{self, Phase};

fn n<S: LayoutScalar>(v: f64) -> S {
    S::from_f64(v)
}
fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}
fn observed<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    let mut entries = readback::continuous_custody(batch, Phase::Unrounded)
        .into_iter()
        .filter(|entry| entry.node() == node);
    let value = entries.next().expect("one actual Native box").output();
    assert!(entries.next().is_none());
    value
}

fn fixture<S: LayoutScalar>(
    reference: GridBaselineReference,
    last: bool,
    column: bool,
    nested: bool,
) -> PublicLayoutTreeOf<S> {
    let preference = if last {
        AlignItems::LastBaseline
    } else {
        AlignItems::Baseline
    };
    let root = NodeInputOf {
        display: Display::Grid,
        box_sizing: BoxSizing::BorderBox,
        size: Size::splat(n(200.0)).map(PreferredSizeOf::px),
        grid_template_columns: vec![
            TrackComponentOf::px(n(if column { 200.0 } else { 100.0 }));
            if column { 1 } else { 2 }
        ],
        grid_template_rows: vec![
            TrackComponentOf::px(n(if column { 100.0 } else { 200.0 }));
            if column { 2 } else { 1 }
        ],
        grid_baseline_references: GridBaselineReferences {
            columns: if column {
                reference
            } else {
                GridBaselineReference::Physical
            },
            rows: if column {
                GridBaselineReference::Physical
            } else {
                reference
            },
        },
        justify_content: Some(AlignContent::Start),
        align_content: Some(AlignContent::Start),
        ..NodeInputOf::default()
    };
    let mut tree = PublicLayoutTreeOf::new().children(0, [1, 2]).style(0, root);
    for (node, extent, start, end) in [(1, 50.0, 3.0, 5.0), (2, 100.0, 7.0, 11.0)] {
        let mode = if column {
            WritingMode::VerticalRl
        } else {
            WritingMode::HorizontalTb
        };
        let axes = FlowAxes::new(mode, Direction::Ltr);
        let border = axes.physical_size(LogicalSizeOf::new(n(20.0), n(extent)));
        let margin = axes.physical_edges(LogicalEdgesOf::new(
            LengthAutoOf::ZERO,
            LengthAutoOf::ZERO,
            LengthAutoOf::px(n(start)),
            LengthAutoOf::px(n(end)),
        ));
        let style = NodeInputOf {
            display: Display::Flex,
            writing_mode: mode,
            size: border.map(PreferredSizeOf::px),
            grid_column: GridPlacement::try_lines(
                if column { 1 } else { node as isize },
                if column { 2 } else { node as isize + 1 },
            )
            .unwrap(),
            grid_row: GridPlacement::try_lines(
                if column { node as isize } else { 1 },
                if column { node as isize + 1 } else { 2 },
            )
            .unwrap(),
            align_self: Some(if column {
                AlignItems::Start
            } else {
                preference
            }),
            justify_self: Some(if column {
                preference
            } else {
                AlignItems::Start
            }),
            margin,
            ..NodeInputOf::default()
        };
        tree = tree.children(node, []).style(node, style);
    }
    if nested {
        assert!(!column, "this fixture's inherited shared axis is Row");
        let child = tree.node_input(1).clone();
        tree = tree
            .children(1, [11])
            .style(
                11,
                NodeInputOf {
                    grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                    ..child
                },
            )
            .children(11, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                    grid_template_columns: vec![TrackComponentOf::px(n(100.0))],
                    grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))],
                    // The actual inherited shared Row must retain owner0's reference,
                    // including the first pass before any target exists.
                    grid_baseline_references: GridBaselineReferences {
                        columns: GridBaselineReference::Named(BaselineName::Alphabetic),
                        rows: GridBaselineReference::Named(BaselineName::Alphabetic),
                    },
                    ..NodeInputOf::default()
                },
            );
    }
    tree
}

fn border_synthesis<S: LayoutScalar>() {
    for column in [false, true] {
        for last in [false, true] {
            for reference in [
                GridBaselineReference::Physical,
                GridBaselineReference::Named(BaselineName::Alphabetic),
                GridBaselineReference::Named(BaselineName::Central),
            ] {
                let tree = fixture::<S>(reference, last, column, false);
                assert!(!tree.has_leaf_measurement(1) && !tree.has_leaf_measurement(2));
                assert_eq!((tree.child_count(1), tree.child_count(2)), (0, 0));
                let batch = compute_layout(&tree, 0, request()).unwrap();
                assert_eq!(observed(&batch, 0).size, Size::splat(n(200.0)));
                let central = reference == GridBaselineReference::Named(BaselineName::Central);
                // Row/Htb: First target max(3+25,7+50)=57; Last distance
                // max(5+25,11+50)=61. Column/VRL First normalizes to the
                // owner's Last cohort; selected physical ports remain25/50.
                let (one, two, point) = match (column, last, central) {
                    (false, false, true) => (32.0, 7.0, 57.0),
                    (false, true, true) => (114.0, 89.0, 139.0),
                    (false, false, false) => (57.0, 7.0, 107.0),
                    (false, true, false) => (139.0, 89.0, 189.0),
                    (true, false, true) => (118.0, 93.0, 143.0),
                    (true, true, true) => (36.0, 11.0, 61.0),
                    (true, false, false) => (93.0, 93.0, 93.0),
                    (true, true, false) => (11.0, 11.0, 11.0),
                };
                for (node, origin, extent) in [(1, one, 50.0), (2, two, 100.0)] {
                    let output = observed(&batch, node);
                    let actual = if column {
                        output.location.x
                    } else {
                        output.location.y
                    };
                    assert_eq!(
                        actual,
                        n(origin),
                        "{reference:?} column={column} last={last} node={node}"
                    );
                    assert_eq!(
                        if column {
                            output.size.width
                        } else {
                            output.size.height
                        },
                        n(extent)
                    );
                    let port = if central {
                        extent / 2.0
                    } else if column {
                        0.0
                    } else {
                        extent
                    };
                    assert_eq!(actual + n(port), n(point));
                    let axes = FlowAxes::new(
                        if column {
                            WritingMode::VerticalRl
                        } else {
                            WritingMode::HorizontalTb
                        },
                        Direction::Ltr,
                    );
                    assert_eq!(
                        output.margin,
                        axes.physical_edges(LogicalEdgesOf::new(
                            S::ZERO,
                            S::ZERO,
                            n(if node == 1 { 3.0 } else { 7.0 }),
                            n(if node == 1 { 5.0 } else { 11.0 })
                        ))
                    );
                }
            }
        }
    }
}

fn inherited_context<S: LayoutScalar>() {
    for last in [false, true] {
        let tree = fixture::<S>(
            GridBaselineReference::Named(BaselineName::Central),
            last,
            false,
            true,
        );
        let batch = compute_layout(&tree, 0, request()).unwrap();
        let inner = observed(&batch, 1).location.y + observed(&batch, 11).location.y;
        let peer = observed(&batch, 2).location.y;
        assert_eq!(inner, n(if last { 114.0 } else { 32.0 }));
        assert_eq!(peer, n(if last { 89.0 } else { 7.0 }));
        assert_eq!(inner + n(25.0), peer + n(50.0));
        let again = compute_layout(&tree, 0, request()).unwrap();
        assert_eq!(observed(&again, 11), observed(&batch, 11));
    }
}

fn unknown_refuses_without_changing_prior_output<S: LayoutScalar>() {
    let good = fixture::<S>(
        GridBaselineReference::Named(BaselineName::Central),
        false,
        false,
        false,
    );
    let prior = compute_layout(&good, 0, request()).unwrap();
    let prior_one = observed(&prior, 1);
    // An actual in-flow child makes Flex nonempty. Generic physical ports
    // are not an authenticated named baseline or proof of named absence.
    let bad = good.clone().children(2, [21]).children(21, []).style(
        21,
        NodeInputOf {
            size: Size::splat(n(10.0)).map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        },
    );
    let error = compute_layout(&bad, 0, request()).unwrap_err();
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 0,
            subject: 2
        }
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::NamedGridBaseline)
    ));
    assert_eq!(observed(&prior, 1), prior_one);
    let retry = compute_layout(&good, 0, request()).unwrap();
    assert_eq!(observed(&retry, 1), prior_one);
}

fn zero_line_fixture<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
    fixture::<S>(
        GridBaselineReference::Named(BaselineName::Central),
        true,
        false,
        false,
    )
    .children(1, [11])
    .children(11, [])
    .style(
        1,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(n(20.0)), PreferredSizeOf::px(n(50.0))),
            inline_baseline_context: InlineBaselineContext::Named(BaselineName::Central),
            align_self: Some(AlignItems::LastBaseline),
            grid_column: GridPlacement::try_lines(1, 2).unwrap(),
            grid_row: GridPlacement::try_lines(1, 2).unwrap(),
            ..NodeInputOf::default()
        },
    )
    .style(
        2,
        NodeInputOf {
            display: Display::Flex,
            size: Size::new(PreferredSizeOf::px(n(20.0)), PreferredSizeOf::px(n(100.0))),
            align_self: Some(AlignItems::LastBaseline),
            grid_column: GridPlacement::try_lines(2, 3).unwrap(),
            grid_row: GridPlacement::try_lines(1, 2).unwrap(),
            ..NodeInputOf::default()
        },
    )
    .input(
        11,
        LayoutInputOf::line_break(
            LineBreakInputOf::new().with_metrics(
                InlineMetricsOf::try_new(
                    InlineBaselineContext::Named(BaselineName::Central),
                    S::ZERO,
                    S::ZERO,
                )
                .unwrap(),
            ),
        ),
    )
}

fn actual_named_last_zero<S: LayoutScalar>() {
    let tree = zero_line_fixture::<S>();
    let batch = compute_layout(&tree, 0, request()).unwrap();
    let one = observed(&batch, 1);
    let last = one.named_baselines.get(BaselineName::Central).last();
    assert_eq!(
        last,
        BaselineFactOf::Available(
            BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, S::ZERO).unwrap()
        )
    );
    // Both last distances are50; selected point is200-50=150.
    assert_eq!(one.location.y, n(150.0));
    assert_eq!(observed(&batch, 2).location.y, n(100.0));
}

fn fact_selection_and_cache<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let fact = |axis, offset| {
        BaselineFactOf::Available(BaselineCoordinateOf::try_new(axis, n(offset)).unwrap())
    };
    let output = ComputeOutputOf::from_outer_size(Size::new(n(20.0), n(50.0)))
        .with_named_baselines(NamedBaselinesOf::unknown().with(
            BaselineName::Central,
            BaselinePairOf::new(
                fact(PhysicalAxis::Vertical, 7.0),
                fact(PhysicalAxis::Vertical, 0.0),
            ),
        ));
    let selected = select_grid_baseline(
        GridBaselineReference::Named(BaselineName::Central),
        AlignItems::LastBaseline,
        flow,
        PhysicalAxis::Vertical,
        output,
    )
    .unwrap()
    .unwrap();
    assert_eq!(selected.coordinate, S::ZERO);
    assert!(!selected.synthesized);
    let input = |output, auto_margins| AncestorBaselineMemberInput {
        reference: GridBaselineReference::Named(BaselineName::Central),
        source: 1_u32,
        axis: GridAxisKind::Row,
        ancestor_span: GridTrackSpan::new(1, 3),
        alignment_context_reversed: false,
        alignment: AlignItems::LastBaseline,
        block_auto_margins: auto_margins,
        synthesized_baseline_cycle: true,
        output,
        margin: Edges::ZERO,
        child_flow_axes: flow,
        containing_flow_axes: flow,
        start_adjustment: S::ZERO,
        end_adjustment: S::ZERO,
    };
    assert!(
        ancestor_baseline_member(input(output, false))
            .unwrap()
            .is_some(),
        "a real Available Last0 does not acquire a synthesized-cycle exclusion"
    );
    assert!(
        ancestor_baseline_member(input(
            output.with_named_baselines(NamedBaselinesOf::absent()),
            false
        ))
        .unwrap()
        .is_none()
    );
    let wrong = output.with_named_baselines(NamedBaselinesOf::unknown().with(
        BaselineName::Central,
        BaselinePairOf::new(
            fact(PhysicalAxis::Horizontal, 7.0),
            fact(PhysicalAxis::Horizontal, 0.0),
        ),
    ));
    assert_eq!(
        select_grid_baseline(
            GridBaselineReference::Named(BaselineName::Central),
            AlignItems::LastBaseline,
            flow,
            PhysicalAxis::Vertical,
            wrong
        ),
        Err(GridBaselineSelectionError::Invalid(
            GridBaselineError::AxisMismatch {
                expected: PhysicalAxis::Vertical,
                actual: PhysicalAxis::Horizontal
            }
        ))
    );
    assert_eq!(
        select_grid_baseline(
            GridBaselineReference::Named(BaselineName::Alphabetic),
            AlignItems::Baseline,
            flow,
            PhysicalAxis::Vertical,
            output
        ),
        Err(GridBaselineSelectionError::Unknown)
    );
    assert!(
        ancestor_baseline_member(input(
            output.with_named_baselines(NamedBaselinesOf::unknown()),
            true
        ))
        .unwrap()
        .is_none()
    );
    let base = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::new(Some(n(20.0)), Some(n(50.0))),
        Size::NONE,
        ContainingLayoutContext::new(flow, ParentFormattingContext::Grid),
        request::<S>().sizing_request(),
    );
    let parent = NodeInputOf::<S> {
        display: Display::Grid,
        grid_baseline_references: GridBaselineReferences {
            columns: GridBaselineReference::Physical,
            rows: GridBaselineReference::Named(BaselineName::Central),
        },
        ..NodeInputOf::default()
    };
    for (alignment, source, set) in [
        (
            AlignItems::Baseline,
            BaselineSource::Last,
            NamedBaselineSet::First,
        ),
        (
            AlignItems::LastBaseline,
            BaselineSource::First,
            NamedBaselineSet::Last,
        ),
    ] {
        let child = NodeInputOf::<S> {
            align_self: Some(alignment),
            baseline_source: source,
            ..NodeInputOf::default()
        };
        let projected = GridChildInput::from_node(&child);
        let demanded = grid_item_baseline_demand(
            &GridContainerProjection::from_node(&parent),
            projected.item(),
            None,
            base,
        )
        .unwrap();
        assert_eq!(
            demanded.named_baseline_demand(),
            NamedBaselineDemand::Fact {
                name: BaselineName::Central,
                set
            }
        );
        let unrelated = grid_item_baseline_demand(
            &GridContainerProjection::from_node(&parent),
            projected.item(),
            Some(GridAxisKind::Column),
            base,
        )
        .unwrap();
        assert_eq!(unrelated.named_baseline_demand(), NamedBaselineDemand::None);
    }
    let asked = base.with_named_baseline_demand(NamedBaselineDemand::Fact {
        name: BaselineName::Central,
        set: NamedBaselineSet::Last,
    });
    let mut cache = CacheOf::new();
    cache.store_with_context(&asked, CacheKeyContext::new(), output);
    assert_eq!(
        cache.get_with_context(&asked, CacheKeyContext::new()),
        Some(output)
    );
    for other in [
        base,
        base.with_named_baseline_demand(NamedBaselineDemand::Fact {
            name: BaselineName::Central,
            set: NamedBaselineSet::First,
        }),
        base.with_named_baseline_demand(NamedBaselineDemand::Fact {
            name: BaselineName::Alphabetic,
            set: NamedBaselineSet::Last,
        }),
    ] {
        assert_eq!(cache.get_with_context(&other, CacheKeyContext::new()), None);
    }
}

#[test]
fn border_box_synthesis_and_reversed_column_use_actual_margins_once() {
    border_synthesis::<f32>();
    border_synthesis::<f64>();
}
#[test]
fn inherited_row_reference_survives_empty_target_and_replay() {
    inherited_context::<f32>();
    inherited_context::<f64>();
}
#[test]
fn nonempty_unknown_refuses_and_preserves_the_prior_batch() {
    unknown_refuses_without_changing_prior_output::<f32>();
    unknown_refuses_without_changing_prior_output::<f64>();
}
#[test]
fn real_zero_height_native_line_supplies_available_last_zero() {
    actual_named_last_zero::<f32>();
    actual_named_last_zero::<f64>();
}
#[test]
fn selected_fact_axis_cycle_and_cache_are_independent_of_generic_ports() {
    fact_selection_and_cache::<f32>();
    fact_selection_and_cache::<f64>();
}

fn lane_capability<S: LayoutScalar>() {
    for column in [false, true] {
        let original = fixture::<S>(GridBaselineReference::Physical, false, column, false);
        let mut lane_style = original.node_input(0).clone();
        lane_style.display = Display::GridLanes;
        let physical = original.clone().style(0, lane_style.clone());
        let prior = compute_layout(&physical, 0, request()).unwrap();
        assert_eq!(
            observed(&prior, 1).size,
            if column {
                Size::new(n(50.0), n(20.0))
            } else {
                Size::new(n(20.0), n(50.0))
            }
        );
        let mut named_style = lane_style.clone();
        if column {
            named_style.grid_baseline_references.columns =
                GridBaselineReference::Named(BaselineName::Central);
        } else {
            named_style.grid_baseline_references.rows =
                GridBaselineReference::Named(BaselineName::Central);
        }
        let named = original.clone().style(0, named_style);
        if column {
            let aligned = compute_layout(&named, 0, request()).unwrap();
            // VerticalRl First uses the owner's Last cohort. Its under-side
            // distance is max(3+25, 7+50)=57, so the shared point is 200-57=143.
            // Column lanes pack the 20-unit stacking boxes at 0 and 20.
            for (node, x, y, width) in [(1, 118.0, 0.0, 50.0), (2, 93.0, 20.0, 100.0)] {
                let output = observed(&aligned, node);
                assert_eq!(output.location, Point::new(n(x), n(y)));
                assert_eq!(output.size, Size::new(n(width), n(20.0)));
                assert_eq!(output.named_baselines, NamedBaselinesOf::absent());
            }
        } else {
            let error = compute_layout(&named, 0, request()).unwrap_err();
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::UnsupportedCapability(
                    LayoutUnsupportedCapability::NamedGridLanesBaseline {
                        axis: GridAxisKind::Row
                    }
                )
            ));
        }
        assert_eq!(
            observed(&prior, 1).size,
            observed(&compute_layout(&physical, 0, request()).unwrap(), 1).size
        );
        // A resolved name without any participating baseline request does not
        // prohibit an otherwise supported physical Lane operation.
        let mut inactive = named;
        for node in [1, 2] {
            let mut style = inactive.node_input(node).clone();
            style.align_self = Some(AlignItems::Start);
            style.justify_self = Some(AlignItems::Start);
            inactive.insert_input(node, LayoutInputOf::box_input(style));
        }
        compute_layout(&inactive, 0, request()).unwrap();
    }
}
#[test]
fn named_lane_grid_axis_aligns_and_stacking_retains_refusal() {
    lane_capability::<f32>();
    lane_capability::<f64>();
}

#[test]
fn central_synthesis_refuses_a_lost_nonzero_half_but_preserves_real_zero() {
    fn check<S: LayoutScalar>(tiny: S) {
        let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let output = ComputeOutputOf::from_outer_size(Size::new(S::ONE, tiny))
            .with_named_baselines(NamedBaselinesOf::absent());
        assert_eq!(
            select_grid_baseline(
                GridBaselineReference::Named(BaselineName::Central),
                AlignItems::Baseline,
                flow,
                PhysicalAxis::Vertical,
                output
            ),
            Err(GridBaselineSelectionError::Invalid(
                GridBaselineError::UnrepresentableCoordinate
            ))
        );
        let zero = select_grid_baseline(
            GridBaselineReference::Named(BaselineName::Central),
            AlignItems::Baseline,
            flow,
            PhysicalAxis::Vertical,
            ComputeOutputOf::from_outer_size(Size::new(S::ONE, S::ZERO))
                .with_named_baselines(NamedBaselinesOf::absent()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(zero.coordinate, S::ZERO);
        assert!(zero.synthesized);
    }
    check(f32::from_bits(1));
    check(f64::from_bits(1));
}

fn actual_scroll_last_export<S: LayoutScalar>() {
    let base = zero_line_fixture::<S>();
    let mut actor = base.node_input(1).clone();
    assert_eq!(actor.baseline_source, BaselineSource::Auto);
    actor.overflow = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
    actor.margin.bottom = LengthAutoOf::px(n(-5.0));
    let tree = base.style(1, actor);
    let batch = compute_layout(&tree, 0, request()).unwrap();
    let one = observed(&batch, 1);
    // General Align3 baseline export: H50 plus signed end margin-5 gives45,
    // already inside the scroll border. Grid's margin contribution is then
    // -5+(50-45)=0; the real empty peer contributes100-50=50.
    assert_eq!(
        one.named_baselines.get(BaselineName::Central).last(),
        BaselineFactOf::Available(
            BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, n(45.0)).unwrap()
        )
    );
    assert_eq!(one.location.y, n(105.0));
    assert_eq!(one.margin.bottom, n(-5.0));
    assert_eq!(observed(&batch, 2).location.y, n(100.0));
}
#[test]
fn general_auto_scroll_last_export_is_available_before_grid_margin_distance() {
    actual_scroll_last_export::<f32>();
    actual_scroll_last_export::<f64>();
}

// This is a sizing/census operation, not a claim of local Lane placement.
fn inherited_lane_sizing_fixture<S: LayoutScalar>(
    owner_reference: GridBaselineReference,
) -> PublicLayoutTreeOf<S> {
    let mut tree = fixture::<S>(owner_reference, false, false, true);
    let mut owner = tree.node_input(0).clone();
    owner.size.height = PreferredSizeOf::AUTO;
    owner.grid_template_rows = vec![TrackComponentOf::AUTO];
    tree.insert_input(0, LayoutInputOf::box_input(owner));

    let mut lane = tree.node_input(1).clone();
    lane.display = Display::GridLanes;
    // The normalized input selects Rows as the grid axis; the Columns
    // stacking axis has no synthetic Grid span.
    lane.grid_lanes_flow = crate::GridLanesFlow::new(GridAxisKind::Row, false, false);
    lane.grid_template_columns.clear();
    tree.insert_input(1, LayoutInputOf::box_input(lane));

    let mut descendant = tree.node_input(11).clone();
    descendant.margin = Edges::all(LengthAutoOf::ZERO);
    descendant.margin.bottom = LengthAutoOf::px(n(100.0));
    tree.insert_input(11, LayoutInputOf::box_input(descendant));
    let mut peer = tree.node_input(2).clone();
    peer.margin = Edges::all(LengthAutoOf::ZERO);
    tree.insert_input(2, LayoutInputOf::box_input(peer));
    tree
}

fn inherited_lane_sizing_input<S: LayoutScalar>() -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        request::<S>().sizing_request(),
    )
}

fn inherited_lane_named_census<S: LayoutScalar>() {
    use crate::engine::ComputeSession;
    let input = inherited_lane_sizing_input::<S>();
    assert_eq!(input.run_mode(), RunMode::ComputeSize);
    assert_eq!(input.sizing_mode(), SizingMode::InherentSize);
    assert_eq!(input.known(), Size::NONE);
    assert_eq!(input.parent(), Size::NONE);
    for axis in [
        input.sizing_request().axes().width,
        input.sizing_request().axes().height,
    ] {
        assert_eq!(axis.operation(), SizingOperation::MaxContent);
        assert_eq!(axis.space(), AvailableSpaceOf::Indefinite);
    }

    let mut accepted = None;
    for (reference, height) in [
        (GridBaselineReference::Physical, 200.0),
        (GridBaselineReference::Named(BaselineName::Central), 175.0),
    ] {
        let tree = inherited_lane_sizing_fixture::<S>(reference);
        for node in [0, 1] {
            let style = tree.node_input(node);
            assert_eq!(style.overflow, ComputedOverflow::VISIBLE);
            assert_eq!(style.scrollbar_width.get(), S::ZERO);
        }
        assert_eq!(tree.node_input(0).grid_baseline_references.rows, reference);
        assert_eq!(
            tree.node_input(1).grid_baseline_references.rows,
            GridBaselineReference::Named(BaselineName::Alphabetic)
        );
        assert_eq!(
            grid_axis_for_grid_lanes(&GridContainerProjection::from_node(tree.node_input(1))),
            GridAxisKind::Row
        );
        assert_eq!((tree.child_count(0), tree.child_count(1)), (2, 1));
        assert_eq!((tree.child_count(11), tree.child_count(2)), (0, 0));
        assert!(!tree.has_leaf_measurement(11) && !tree.has_leaf_measurement(2));
        assert_eq!(
            tree.node_input(11).size.height,
            PreferredSizeOf::px(n(50.0))
        );
        assert_eq!(
            tree.node_input(2).size.height,
            PreferredSizeOf::px(n(100.0))
        );
        assert_eq!(
            tree.node_input(11).margin.bottom,
            LengthAutoOf::px(n(100.0))
        );

        let mut session = ComputeSession::new(&tree, Vec::new());
        let output = session.compute_child(0, input).unwrap();
        // Both actual children have fixed physical bodies. The descendant's
        // margin-box contribution is50+100=150. Central's shared First
        // distance is max(25,50)=50, hence150+(50-25)=175. Physical uses
        // border-under ports50/100, hence150+(100-50)=200. The real peer
        // contributes100. This cannot pass with an empty/dropped census,
        // the Lane's local Alphabetic selector, or a first-child-only group.
        assert_eq!(output.size, Size::new(n(200.0), n(height)));
        assert_eq!(
            output.named_baselines,
            NamedBaselinesOf::unknown(),
            "sizing does not invent a named container export"
        );
        if reference == GridBaselineReference::Named(BaselineName::Central) {
            accepted = Some(output);
        }
    }

    let good =
        inherited_lane_sizing_fixture::<S>(GridBaselineReference::Named(BaselineName::Central));
    let bad = good.clone().children(11, [111]).children(111, []).style(
        111,
        NodeInputOf {
            display: Display::Block,
            size: Size::splat(S::ONE).map(PreferredSizeOf::px),
            ..NodeInputOf::default()
        },
    );
    assert_eq!(bad.child_count(11), 1);
    assert!(!bad.has_leaf_measurement(11));
    let mut session = ComputeSession::new(&bad, Vec::new());
    let error = session.compute_child(0, input).unwrap_err();
    // The shared reference belongs to owner0, while the actual Lane
    // container1 is executing the unavailable query for its child11.
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::ContainerSubject {
            container: 1,
            subject: 11
        }
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::NamedGridBaseline)
    ));
    // A real nonempty Flex is Unknown, not Absent. Retrying the original
    // supported input must still reproduce the independently qualified size.
    let mut retry = ComputeSession::new(&good, Vec::new());
    assert_eq!(retry.compute_child(0, input).unwrap(), accepted.unwrap());
}

#[test]
fn inherited_lane_grid_axis_named_census_sizes_without_local_lane_placement() {
    inherited_lane_named_census::<f32>();
    inherited_lane_named_census::<f64>();
}
