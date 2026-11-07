use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{self, Phase};
use crate::{LayoutInputViewOf, LayoutTree, LeafMeasureInputOf, NodeInputOf};

#[derive(Clone, Copy, Debug)]
enum Measurement {
    RefuseResolvedWidth,
    SmallerAfterRows,
    MaximumOnlyChanges,
    RevisitMaximumRefuses { minimum_changes: bool },
    LastRowsChange,
    HiddenMinimumChange,
    OppositeExtent,
    FixedIntrinsicWidths,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedWidth;

struct StageTree<S: LayoutScalar> {
    source: PublicLayoutTreeOf<S>,
    measurement: Measurement,
    leaf_start: u32,
    refuse_formal_height_at_width: std::cell::Cell<Option<S>>,
    caches: Vec<crate::LayoutCacheStoreEntryOf<u32, S>>,
    hits: std::cell::Cell<usize>,
    requests: std::cell::RefCell<Vec<(u32, ComputeInputOf<S>)>>,
}

impl<S: LayoutScalar> StageTree<S> {
    fn new(measurement: Measurement) -> Self {
        let mut source = PublicLayoutTreeOf::new()
            .children(
                0,
                if matches!(measurement, Measurement::HiddenMinimumChange) {
                    vec![1, 2]
                } else {
                    vec![1]
                },
            )
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MAX_CONTENT),
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![TrackComponentOf::AUTO],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            );
        for node in [1, 2] {
            source = source.children(node, []).style(
                node,
                NodeInputOf {
                    item_is_replaced: true,
                    grid_column: GridPlacement::try_line(1).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    justify_self: Some(AlignItems::Start),
                    align_self: Some(AlignItems::Start),
                    ..NodeInputOf::default()
                },
            );
        }
        Self {
            source,
            measurement,
            leaf_start: 1,
            refuse_formal_height_at_width: std::cell::Cell::new(None),
            caches: Vec::new(),
            hits: std::cell::Cell::new(0),
            requests: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn nested() -> Self {
        let mut tree = Self::new(Measurement::RefuseResolvedWidth);
        let leaf = tree.source.node_input(1).clone();
        let grid = tree.source.node_input(0).clone();
        let mut columns_subgrid = grid.clone();
        columns_subgrid.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
        columns_subgrid.grid_template_columns =
            vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(vec![]))];
        let mut rows_subgrid = grid;
        rows_subgrid.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
        rows_subgrid.grid_template_rows =
            vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(vec![]))];
        tree.source = tree
            .source
            .children(1, [2])
            .children(2, [3])
            .children(3, [])
            .style(1, columns_subgrid)
            .style(2, rows_subgrid)
            .style(3, leaf);
        tree.leaf_start = 3;
        tree
    }

    fn observe(
        &self,
        through: GridSizingPhase,
    ) -> LayoutResultOf<u32, GridIntrinsicObservation<S>, S, ResolvedWidth> {
        self.observe_at(GridAxisKind::Column, through)
    }

    fn observe_at(
        &self,
        first_axis: GridAxisKind,
        through: GridSizingPhase,
    ) -> LayoutResultOf<u32, GridIntrinsicObservation<S>, S, ResolvedWidth> {
        let input = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            crate::ContainingLayoutContext::new(
                crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr),
                crate::ParentFormattingContext::BlockFlow,
            ),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MaxContent,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        );
        match crate::engine::observe_grid_probe(
            self,
            0,
            GridOverflowProbe {
                layout: GridOverflowLayout::RootPrefix {
                    input,
                    stage: GridSizingStage {
                        first_axis,
                        through,
                    },
                },
            },
        )? {
            GridOverflowObservation::Intrinsic(observation) => Ok(observation),
            GridOverflowObservation::Children(_) => {
                unreachable!("root prefix returns its actual intrinsic observation")
            }
        }
    }
}

impl<S: LayoutScalar> Traverse for StageTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.source, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.source.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.source.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for StageTree<S> {
    type MeasureError = ResolvedWidth;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.source.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.source.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node >= self.leaf_start
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: crate::CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        if self.has_leaf_measurement(node) {
            self.requests.borrow_mut().push((node, *input));
        }
        let output = self
            .caches
            .iter()
            .find(|entry| {
                entry.node() == node && entry.input() == input && entry.context() == context
            })
            .map(crate::LayoutCacheStoreEntryOf::output);
        if output.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        output
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, ResolvedWidth>> {
        let available = input.content_sizing_request().axes();
        let width = available
            .width
            .space()
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get);
        if width.is_some()
            && width == self.refuse_formal_height_at_width.get()
            && available.height.operation() == SizingOperation::MaxContent
            && input.known_content_size().width.is_none()
        {
            return Some(Err(ResolvedWidth));
        }
        let rows_known = available.height.space().definite_size().is_some();
        // A used-minimum query can legitimately supply its selected width.
        // Refuse the later dependency on a finite width constraint while the
        // provider still has to determine that width itself.
        if matches!(self.measurement, Measurement::RefuseResolvedWidth)
            && width.is_some()
            && input.known_content_size().width.is_none()
        {
            return Some(Err(ResolvedWidth));
        }
        if matches!(self.measurement, Measurement::RevisitMaximumRefuses { .. })
            && rows_known
            && available.width.operation() == SizingOperation::MaxContent
        {
            return Some(Err(ResolvedWidth));
        }
        let minimum = available.width.operation() == SizingOperation::MinContent;
        // These are declared host measurements, dependent only on the actual
        // content-space request. They model the opposite-axis dependencies in
        // Grid 2's four sizing stages; no callback receives a phase label.
        let measured_width = match self.measurement {
            Measurement::FixedIntrinsicWidths => {
                if minimum {
                    40.0
                } else {
                    80.0
                }
            }
            Measurement::OppositeExtent => {
                if minimum {
                    10.0
                } else {
                    return Some(Ok(Size::new(
                        available
                            .height
                            .space()
                            .definite_size()
                            .map_or(S::from_f64(80.0), crate::NonNegativeFiniteOf::get),
                        S::from_f64(20.0),
                    )));
                }
            }
            Measurement::RefuseResolvedWidth => 40.0,
            Measurement::SmallerAfterRows => match (rows_known, minimum) {
                (false, true) => 20.0,
                (false, false) => 40.0,
                (true, true) => 10.0,
                (true, false) => 30.0,
            },
            Measurement::RevisitMaximumRefuses { minimum_changes } => {
                if minimum {
                    if rows_known && minimum_changes {
                        15.0
                    } else {
                        10.0
                    }
                } else {
                    40.0
                }
            }
            Measurement::MaximumOnlyChanges => {
                if minimum {
                    10.0
                } else if rows_known {
                    35.0
                } else {
                    40.0
                }
            }
            Measurement::LastRowsChange => match (
                available
                    .height
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get),
                minimum,
            ) {
                (Some(height), _) if height == S::from_f64(50.0) => {
                    if minimum {
                        20.0
                    } else {
                        70.0
                    }
                }
                (Some(_), _) => {
                    if minimum {
                        15.0
                    } else {
                        35.0
                    }
                }
                (None, _) => {
                    if minimum {
                        10.0
                    } else {
                        40.0
                    }
                }
            },
            Measurement::HiddenMinimumChange if node == 1 => {
                if minimum {
                    40.0
                } else {
                    50.0
                }
            }
            Measurement::HiddenMinimumChange => {
                if minimum {
                    if rows_known { 15.0 } else { 10.0 }
                } else if rows_known {
                    60.0
                } else {
                    40.0
                }
            }
        };
        let height = if matches!(self.measurement, Measurement::LastRowsChange)
            && width == Some(S::from_f64(35.0))
        {
            50.0
        } else {
            30.0
        };
        Some(Ok(Size::new(
            S::from_f64(measured_width),
            S::from_f64(height),
        )))
    }
}

fn assert_grid_axis_prefix_requests<S: LayoutScalar>(tree: &StageTree<S>) {
    let requests = tree.requests.borrow();
    assert!(
        requests
            .iter()
            .any(|(_, input)| input.known().width.is_some()),
        "the prefix includes the selected-axis used-minimum query"
    );
    assert!(
        requests
            .iter()
            .all(|(_, input)| input.known().width.is_some()
                || !input
                    .sizing_request()
                    .axes()
                    .width
                    .space()
                    .definite_size()
                    .is_some()),
        "the grid-axis prefix must not borrow a later opposite width constraint"
    );
}

fn assert_width_dependency<S: LayoutScalar>(tree: &StageTree<S>, start: usize) {
    assert!(
        tree.requests.borrow().iter().skip(start).any(|(_, input)| {
            input.known().width.is_none()
                && input
                    .sizing_request()
                    .axes()
                    .width
                    .space()
                    .definite_size()
                    .is_some()
        }),
        "the actual operation must reach its unresolved-width dependency"
    );
}

fn assert_cutoff<S: LayoutScalar>() {
    for tree in [
        StageTree::<S>::new(Measurement::RefuseResolvedWidth),
        StageTree::<S>::nested(),
    ] {
        let first = tree.observe(GridSizingPhase::InitialFirst).unwrap();
        assert_eq!(first.output.size.width, S::from_f64(40.0));
        assert!(
            first.effective_gutter.is_none(),
            "an unresolved opposite axis is not an allocated scrollport"
        );
        assert_grid_axis_prefix_requests(&tree);
        let prefix_requests = tree.requests.borrow().len();
        let error = tree
            .observe(GridSizingPhase::InitialSecond)
            .err()
            .expect("row sizing reaches the actual width-dependent provider refusal");
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(ResolvedWidth)
        ));
        assert_width_dependency(&tree, prefix_requests);
    }
}

#[test]
fn bounded_column_preparation_does_not_execute_the_later_fallible_row_stage() {
    assert_cutoff::<f32>();
    assert_cutoff::<f64>();
}

fn assert_revisit<S: LayoutScalar>() {
    for (measurement, width, height) in [
        (Measurement::SmallerAfterRows, 30.0, 30.0),
        (Measurement::MaximumOnlyChanges, 40.0, 30.0),
        (Measurement::HiddenMinimumChange, 60.0, 30.0),
        (Measurement::LastRowsChange, 35.0, 50.0),
    ] {
        let output = StageTree::<S>::new(measurement)
            .observe(GridSizingPhase::Complete)
            .unwrap()
            .output;
        assert_eq!(
            output.size,
            Size::new(S::from_f64(width), S::from_f64(height)),
            "{measurement:?}"
        );
    }
}

#[test]
fn individual_minimum_changes_replace_both_roles_and_final_rows_do_not_reopen_columns() {
    assert_revisit::<f32>();
    assert_revisit::<f64>();
}

fn assert_revisit_maximum_query_permission<S: LayoutScalar>() {
    let changed = StageTree::<S>::new(Measurement::RevisitMaximumRefuses {
        minimum_changes: true,
    });
    let error = changed
        .observe(GridSizingPhase::Complete)
        .err()
        .expect("changed min-content must query max-content for the permitted revisit");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));

    let stable = StageTree::<S>::new(Measurement::RevisitMaximumRefuses {
        minimum_changes: false,
    });
    let observed = stable
        .observe(GridSizingPhase::Complete)
        .expect("unchanged min-content must not query max-content for a skipped revisit");
    assert_eq!(
        observed.output.size,
        Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
}

#[test]
fn grid_revisit_queries_max_content_only_when_min_content_changes_f32() {
    assert_revisit_maximum_query_permission::<f32>();
}

#[test]
fn grid_revisit_queries_max_content_only_when_min_content_changes_f64() {
    assert_revisit_maximum_query_permission::<f64>();
}

fn assert_retained_prefix<S: LayoutScalar>() {
    let mut tree = StageTree::<S>::nested();
    tree.measurement = Measurement::MaximumOnlyChanges;
    let cold = tree.observe(GridSizingPhase::Complete).unwrap().output;
    // A complete public layout correctly discards these isolated keys. Seed
    // the actual observed keys through a real owning Session instead.
    let requests = tree.requests.take();
    assert!(!requests.is_empty());
    let seed = crate::engine::seed_grid_leaf_requests(&tree, &requests).unwrap();
    tree.caches = seed;
    let retained = tree.caches.clone();
    let warm = tree.observe(GridSizingPhase::Complete).unwrap().output;
    assert_eq!(warm, cold);
    assert!(
        tree.hits.get() > 0,
        "the actual bounded nested operation must reach retained leaf keys"
    );
    assert_eq!(
        tree.caches, retained,
        "observation cannot publish or replace the host's retained cache"
    );
    // The same production Session helper asserts no node, fragment, warm-text,
    // cache-store or invalidation staging escapes on either success or failure.
}

#[test]
fn nested_stage_observation_reuses_only_retained_leaf_keys_without_publication() {
    assert_retained_prefix::<f32>();
    assert_retained_prefix::<f64>();
}

fn assert_initial_estimate<S: LayoutScalar>() {
    let mut mixed = StageTree::<S>::new(Measurement::OppositeExtent);
    let mut root = mixed.source.node_input(0).clone();
    root.grid_template_rows = vec![
        TrackComponentOf::px(S::from_f64(20.0)),
        TrackComponentOf::AUTO,
    ];
    mixed.source = mixed.source.style(0, root);
    let estimated = mixed.observe(GridSizingPhase::InitialFirst).unwrap();
    assert_eq!(
        estimated.output.size.width,
        S::from_f64(20.0),
        "only the item's spanned maximum tracks determine its opposite estimate"
    );
    assert!(
        estimated.effective_gutter.is_none(),
        "the other unbounded row is not an allocated zero"
    );

    for (alignment, expected) in [
        (AlignContent::Start, 40.0),
        (AlignContent::SpaceBetween, 100.0),
    ] {
        let mut tree = StageTree::<S>::new(Measurement::OppositeExtent);
        let mut root = tree.source.node_input(0).clone();
        root.size.height = PreferredSizeOf::px(S::from_f64(100.0));
        root.grid_template_rows = vec![TrackComponentOf::px(S::from_f64(20.0)); 2];
        root.align_content = Some(alignment);
        let mut child = tree.source.node_input(1).clone();
        child.grid_row = GridPlacement::try_lines(1, 3).unwrap();
        tree.source = tree.source.style(0, root).style(1, child);
        assert_eq!(
            tree.observe(GridSizingPhase::InitialFirst)
                .unwrap()
                .output
                .size
                .width,
            S::from_f64(expected),
            "definite container and tracks qualify the spanned alignment gap"
        );
    }
}

#[test]
fn initial_maximum_track_estimate_is_span_specific_and_alignment_is_qualified() {
    assert_initial_estimate::<f32>();
    assert_initial_estimate::<f64>();
}

fn assert_lanes_cutoff<S: LayoutScalar>(tree: StageTree<S>) {
    let first = tree.observe(GridSizingPhase::InitialFirst);
    assert_eq!(first.unwrap().output.size.width, S::from_f64(40.0));
    assert_grid_axis_prefix_requests(&tree);
    let prefix_requests = tree.requests.borrow().len();
    let second = tree.observe(GridSizingPhase::InitialSecond);
    assert!(matches!(
        second.err().expect("later stage refuses").kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
    assert_width_dependency(&tree, prefix_requests);
}

#[test]
fn lanes_cutoff_ordinary_grid_control() {
    assert_lanes_cutoff(StageTree::<f32>::new(Measurement::RefuseResolvedWidth));
    assert_lanes_cutoff(StageTree::<f64>::new(Measurement::RefuseResolvedWidth));
}

fn lanes_cutoff_tree<S: LayoutScalar>(inherited: bool) -> StageTree<S> {
    let mut tree = StageTree::<S>::new(Measurement::RefuseResolvedWidth);
    if inherited {
        let leaf = tree.source.node_input(1).clone();
        let mut lanes = tree.source.node_input(0).clone();
        lanes.display = Display::GridLanes;
        lanes.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
        lanes.grid_template_columns =
            vec![TrackComponentOf::Subgrid(crate::SubgridTrack::new(vec![]))];
        lanes.overflow = crate::ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
        lanes.scrollbar_width = crate::ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap();
        tree.source = tree
            .source
            .children(1, [2])
            .children(2, [])
            .style(1, lanes)
            .style(2, leaf);
        tree.leaf_start = 2;
    } else {
        let mut lanes = tree.source.node_input(0).clone();
        lanes.display = Display::GridLanes;
        tree.source = tree.source.style(0, lanes);
    }
    tree
}

#[test]
fn lanes_cutoff_standalone_f32() {
    assert_lanes_cutoff(lanes_cutoff_tree::<f32>(false));
}

#[test]
fn lanes_cutoff_standalone_f64() {
    assert_lanes_cutoff(lanes_cutoff_tree::<f64>(false));
}

#[test]
fn lanes_cutoff_inherited_auto_f32() {
    assert_lanes_cutoff(lanes_cutoff_tree::<f32>(true));
}

#[test]
fn lanes_cutoff_inherited_auto_f64() {
    assert_lanes_cutoff(lanes_cutoff_tree::<f64>(true));
}

fn assert_inherited_grid_cutoff<S: LayoutScalar>() {
    let mut tree = lanes_cutoff_tree::<S>(true);
    let mut child = tree.source.node_input(1).clone();
    child.display = Display::Grid;
    tree.source = tree.source.style(1, child);
    assert_lanes_cutoff(tree);
}

#[test]
fn lanes_cutoff_inherited_ordinary_grid_control() {
    assert_inherited_grid_cutoff::<f32>();
    assert_inherited_grid_cutoff::<f64>();
}

fn assert_lanes_oriented_dependency<S: LayoutScalar>() {
    for mode in [
        crate::WritingMode::HorizontalTb,
        crate::WritingMode::VerticalRl,
        crate::WritingMode::VerticalLr,
        crate::WritingMode::SidewaysRl,
        crate::WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let mut tree = lanes_cutoff_tree::<S>(false);
            let mut root = tree.source.node_input(0).clone();
            root.writing_mode = mode;
            root.direction = direction;
            let flow = crate::FlowAxes::new(mode, direction);
            // Keep the actual grid axis physically horizontal. In orthogonal
            // writing modes it is the local row axis, so the mapped prefix is
            // row-first; no raw InitialFirst rule can identify this dependency.
            let (grid_axis, stacking_axis) = if flow.inline_axis() == PhysicalAxis::Horizontal {
                root.grid_lanes_flow =
                    crate::GridLanesFlow::new(GridAxisKind::Column, false, false);
                (GridAxisKind::Column, GridAxisKind::Row)
            } else {
                root.grid_lanes_flow = crate::GridLanesFlow::new(GridAxisKind::Row, false, false);
                (GridAxisKind::Row, GridAxisKind::Column)
            };
            tree.source = tree.source.style(0, root);
            let prefix = tree
                .observe_at(grid_axis, GridSizingPhase::InitialFirst)
                .unwrap();
            assert_eq!(prefix.output.size.width, S::from_f64(40.0));
            assert!(prefix.effective_gutter.is_none());
            assert_grid_axis_prefix_requests(&tree);
            for (first_axis, through) in [
                (grid_axis, GridSizingPhase::InitialSecond),
                (stacking_axis, GridSizingPhase::InitialFirst),
                (grid_axis, GridSizingPhase::Complete),
            ] {
                let prefix_requests = tree.requests.borrow().len();
                let error = tree.observe_at(first_axis, through).err().expect(
                    "actual stacking-size and completed operations reach the host dependency",
                );
                assert!(matches!(
                    error.kind(),
                    LayoutErrorKindOf::Measurement(ResolvedWidth)
                ));
                assert_width_dependency(&tree, prefix_requests);
            }
        }
    }
}

#[test]
fn lanes_cutoff_follows_resolved_physical_grid_and_stacking_dependencies() {
    assert_lanes_oriented_dependency::<f32>();
    assert_lanes_oriented_dependency::<f64>();
}

fn assert_lanes_required_work<S: LayoutScalar>() {
    let mut tree = lanes_cutoff_tree::<S>(false);
    let mut root = tree.source.node_input(0).clone();
    root.size.height = PreferredSizeOf::px(S::from_f64(20.0));
    root.overflow = crate::ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
    root.scrollbar_width = crate::ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap();
    tree.source = tree.source.style(0, root);
    let error = tree.observe(GridSizingPhase::InitialFirst).err().expect(
        "a qualified initial Auto frame needs real stacking work before its gutter observation",
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
    assert_width_dependency(&tree, 0);

    let mut tree = lanes_cutoff_tree::<S>(false);
    tree.measurement = Measurement::MaximumOnlyChanges;
    let completed = tree.observe(GridSizingPhase::Complete).unwrap();
    assert_eq!(
        completed.output.size,
        Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
    assert!(
        tree.requests
            .borrow()
            .iter()
            .any(|(_, input)| input.sizing_request().axes().width
                == AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(S::from_f64(40.0)).unwrap()
                    ),
                    SizingOperation::Normal
                ))
    );

    let refusing = lanes_cutoff_tree::<S>(false);
    let request = {
        let axes = Size::splat(AxisSizingRequestOf::new(
            AvailableSpaceOf::Indefinite,
            SizingOperation::MaxContent,
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
    let error = crate::compute_layout(&refusing, 0, request).expect_err(
        "public full layout must still propagate the actual stacking measurement refusal",
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
    assert_width_dependency(&refusing, 0);
}

#[test]
fn lanes_cutoff_preserves_qualified_auto_stacking_size_and_full_layout_work() {
    assert_lanes_required_work::<f32>();
    assert_lanes_required_work::<f64>();
}

fn assert_scrollbar_maximum_track_space<S: LayoutScalar>() {
    let n = S::from_f64;
    // Requested scrollbars15 exceed the unallocated max outer2x4. The raw
    // provisional inset is preserved, but actual track space must saturate0.
    // The size-authored control already has an allocated saturated scroll box.
    for explicit_size in [false, true] {
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: if explicit_size {
                        Size::new(PreferredSizeOf::px(n(2.0)), PreferredSizeOf::px(n(4.0)))
                    } else {
                        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
                    },
                    max_size: Size::new(crate::MaxSizeOf::px(n(2.0)), crate::MaxSizeOf::px(n(4.0))),
                    overflow: crate::ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll)
                        .unwrap(),
                    scrollbar_width: crate::ScrollbarWidthOf::try_new(n(15.0)).unwrap(),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    ..NodeInputOf::default()
                },
            );
        assert!(!tree.has_leaf_measurement(0) && !tree.has_leaf_measurement(1));
        let batch = crate::compute_layout(&tree, 0, {
            let axes = Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
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
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                definite,
            )
        })
        .expect("finite max-size/scrollbar geometry completes");
        for entries in [
            readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
            readback::continuous_custody(&batch, Phase::Final).as_slice(),
        ] {
            let root = entries
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            let child = entries
                .iter()
                .find(|entry| entry.node() == 1)
                .unwrap()
                .output();
            assert_eq!(root.size, Size::new(n(2.0), n(4.0)));
            assert_eq!(child.size, Size::ZERO);
        }
    }
    // Saturation of finite provisional residuals must not weaken the checked
    // consumer's nonfinite contract, even for a zero-item track.
    let tracks = [TrackSizingOf::<S>::AUTO];
    let rows = [GridIntrinsicContributionRow {
        span: GridTrackSpan::new(1, 2),
        contributions: GridTrackContributions {
            minimum: S::ZERO,
            min_content: S::ZERO,
            max_content: S::ZERO,
        },
    }];
    let phases = resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks: &tracks,
            basis: None,
            available: GridTrackSizingConstraint {
                operation: SizingOperation::Normal,
                available_extent: Some(S::ZERO),
            },
            gap: S::ZERO,
            gutters: None,
            base_floors: &[],
        },
        &rows,
    )
    .unwrap();
    for value in [S::INFINITY, -S::INFINITY, S::NAN] {
        assert!(matches!(
            resolve_tracks_from_phases(ResolvedAxisTrackInput {
                tracks: &tracks,
                phases: &phases,
                items: &rows,
                available: GridTrackSizingConstraint {
                    operation: SizingOperation::Normal,
                    available_extent: Some(value)
                },
                flex_bounds: GridTrackFinishingBounds::NONE,
                stretch_size: None,
                gap: S::ZERO,
                alignment: AlignContent::Start,
                gutters: None,
            }),
            Err(SpanningContributionError::InvalidNumeric { .. })
        ));
    }
}

#[test]
fn scrollbar_maximum_track_space_f32() {
    assert_scrollbar_maximum_track_space::<f32>();
}
#[test]
fn scrollbar_maximum_track_space_f64() {
    assert_scrollbar_maximum_track_space::<f64>();
}

fn mixed_root_request<S: LayoutScalar>(
    width: SizingOperation,
    height: SizingOperation,
) -> crate::LayoutRootRequestOf<S> {
    crate::LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, width),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, height),
            ),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}
fn mixed_root_output<S: LayoutScalar>(
    batch: &crate::CompletedLayoutBatchOf<u32, S>,
) -> crate::NodeOutputOf<S> {
    let mut entries = readback::continuous_custody(batch, Phase::Unrounded)
        .into_iter()
        .filter(|entry| entry.node() == 0);
    let result = entries.next().expect("actual root output").output();
    assert!(entries.next().is_none());
    result
}

fn assert_normal_columns_recollect_formal_rows<S: LayoutScalar>() {
    let n = S::from_f64;
    let mut tree = StageTree::<S>::new(Measurement::HiddenMinimumChange);
    let mut root = tree.source.node_input(0).clone();
    root.max_size.width = crate::MaxSizeOf::px(n(20.0));
    tree.source = tree.source.style(0, root);
    for node in [1, 2] {
        let mut child = tree.source.node_input(node).clone();
        child.overflow =
            crate::ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
        tree.source = tree.source.style(node, child);
    }
    assert_eq!(
        Traverse::children(&tree.source, 0).collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(tree.source.node_input(0).size.width.is_max_content());
    assert_eq!(
        tree.source.node_input(0).max_size.width,
        crate::MaxSizeOf::px(n(20.0))
    );
    // This maintained host declares MIN/MAX widths40/50 for node1 and
    //10/40 ->15/60 for node2 when actual row space becomes finite. Height30
    // is constant. Hidden makes actual used minima0, while formal Auto
    // minima retain the min-content floor40. The maximum clamps the box20.
    let prior = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::MaxContent, SizingOperation::MaxContent),
    )
    .unwrap();
    assert_eq!(mixed_root_output(&prior).size, Size::new(n(20.0), n(30.0)));
    let prior_entries = readback::continuous_custody(&prior, Phase::Final);
    let formal_row_at = |input: &ComputeInputOf<S>, width| {
        let axes = input.sizing_request().axes();
        input.known().width.is_none()
            && axes.height.operation() == SizingOperation::MaxContent
            && axes
                .width
                .space()
                .definite_size()
                .is_some_and(|value| value.get() == width)
    };
    assert!(
        tree.requests
            .borrow()
            .iter()
            .any(|(_, input)| formal_row_at(input, n(40.0)))
    );
    assert!(
        !tree
            .requests
            .borrow()
            .iter()
            .any(|(_, input)| formal_row_at(input, n(20.0)))
    );
    tree.requests.borrow_mut().clear();
    let mixed = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::Normal, SizingOperation::MaxContent),
    )
    .unwrap();
    // Normal allocation rebuilds the column minimum0 at the accepted20;
    // the formal Row operation must be collected again in that actual frame.
    assert_eq!(mixed_root_output(&mixed).size, Size::new(n(20.0), n(30.0)));
    assert!(
        tree.requests
            .borrow()
            .iter()
            .any(|(_, input)| formal_row_at(input, n(20.0)))
    );
    let accepted_entries = readback::continuous_custody(&mixed, Phase::Final);
    // The host can refuse precisely this new opposite-frame query;
    // an unchanged old formal packet cannot silently bypass the dependency.
    assert!(
        tree.caches.is_empty(),
        "no retained host receipt masks the real refusal"
    );
    tree.refuse_formal_height_at_width.set(Some(n(20.0)));
    let error = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::Normal, SizingOperation::MaxContent),
    )
    .expect_err("the demanded formal height query at physical width20 refuses");
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
    assert_eq!(
        readback::continuous_custody(&mixed, Phase::Final).as_slice(),
        accepted_entries.as_slice()
    );
    assert_eq!(
        readback::continuous_custody(&prior, Phase::Final).as_slice(),
        prior_entries.as_slice()
    );
    tree.refuse_formal_height_at_width.set(None);
    let retry = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::Normal, SizingOperation::MaxContent),
    )
    .unwrap();
    assert_eq!(
        readback::continuous_custody(&retry, Phase::Final).as_slice(),
        accepted_entries.as_slice()
    );
    assert_eq!(
        readback::continuous_custody(&prior, Phase::Final).as_slice(),
        prior_entries.as_slice()
    );
}
#[test]
fn normal_columns_recollect_formal_rows_in_the_active_maximum_frame_f32() {
    assert_normal_columns_recollect_formal_rows::<f32>();
}
#[test]
fn normal_columns_recollect_formal_rows_in_the_active_maximum_frame_f64() {
    assert_normal_columns_recollect_formal_rows::<f64>();
}

fn assert_formal_columns_use_normal_rows<S: LayoutScalar>() {
    let n = S::from_f64;
    let mut tree = StageTree::<S>::new(Measurement::OppositeExtent);
    let mut root = tree.source.node_input(0).clone();
    root.max_size.height = crate::MaxSizeOf::px(n(10.0));
    root.grid_template_rows = vec![TrackComponentOf::minmax(
        MinTrackSizingOf::px(S::ZERO),
        MaxTrackSizingOf::flex(crate::TrackFlexFactorOf::try_new(S::ONE).unwrap()),
    )];
    tree.source = tree.source.style(0, root);
    assert!(tree.source.node_input(0).size.height.is_max_content());
    assert_eq!(
        tree.source.node_input(0).max_size.height,
        crate::MaxSizeOf::px(n(10.0))
    );
    // The actual host's initial unbounded MAX width is80 and its ordinary
    // height20. The one-fr row is bounded to10. Initial MIN width10 is
    // unchanged, so the bounded intrinsic revisit deliberately keeps80.
    let prior = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::MaxContent, SizingOperation::MaxContent),
    )
    .unwrap();
    assert!(
        tree.requests.borrow().iter().any(|(_, input)| {
            let axes = input.sizing_request().axes();
            input.run_mode() == RunMode::ComputeSize
                && axes.height.operation() == SizingOperation::MaxContent
                && axes.width.operation() == SizingOperation::Normal
                && axes
                    .width
                    .space()
                    .definite_size()
                    .is_some_and(|size| size.get() == n(80.0))
        }),
        "the 1fr row requests its max-content contribution in the resolved column frame"
    );
    assert_eq!(mixed_root_output(&prior).size, Size::new(n(80.0), n(10.0)));
    let prior_entries = readback::continuous_custody(&prior, Phase::Final);
    let actual_placement_at = |input: &ComputeInputOf<S>, width| {
        let axes = input.sizing_request().axes();
        input.run_mode() == RunMode::PerformLayout
            && input.known() == Size::NONE
            && axes.width.operation() == SizingOperation::Normal
            && axes.height.operation() == SizingOperation::Normal
            && axes
                .width
                .space()
                .definite_size()
                .is_some_and(|value| value.get() == width)
            && axes
                .height
                .space()
                .definite_size()
                .is_some_and(|value| value.get() == n(10.0))
    };
    assert!(
        tree.requests
            .borrow()
            .iter()
            .any(|(_, input)| actual_placement_at(input, n(80.0)))
    );
    tree.requests.borrow_mut().clear();
    let mixed = crate::compute_layout(
        &tree,
        0,
        mixed_root_request(SizingOperation::MaxContent, SizingOperation::Normal),
    )
    .unwrap();
    // The receiver re-collects formal columns with the actual row10: the
    // same host now declares MAX width10. The accepted container80 is frozen;
    // its normal child therefore receives actual area10, not old area80.
    assert_eq!(mixed_root_output(&mixed).size, Size::new(n(80.0), n(10.0)));
    assert!(
        tree.requests
            .borrow()
            .iter()
            .any(|(_, input)| actual_placement_at(input, n(10.0)))
    );
    assert_eq!(
        readback::continuous_custody(&prior, Phase::Final).as_slice(),
        prior_entries.as_slice()
    );
}
#[test]
fn formal_columns_recollect_in_the_normal_row_frame_without_resizing_the_container_f32() {
    assert_formal_columns_use_normal_rows::<f32>();
}
#[test]
fn formal_columns_recollect_in_the_normal_row_frame_without_resizing_the_container_f64() {
    assert_formal_columns_use_normal_rows::<f64>();
}

fn assert_nested_mapped_mixed_receiver<S: LayoutScalar>() {
    let n = S::from_f64;
    for direction in [Direction::Ltr, Direction::Rtl] {
        let mut tree = StageTree::<S>::nested();
        tree.measurement = Measurement::SmallerAfterRows;
        let mut root = tree.source.node_input(0).clone();
        root.max_size.width = crate::MaxSizeOf::px(n(5.0));
        tree.source = tree.source.style(0, root);
        for node in [1, 2, 3] {
            let mut child = tree.source.node_input(node).clone();
            child.overflow =
                crate::ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap();
            if node == 1 {
                child.writing_mode = crate::WritingMode::VerticalRl;
                child.direction = direction;
            }
            tree.source = tree.source.style(node, child);
        }
        assert_eq!(Traverse::children(&tree.source, 0).collect::<Vec<_>>(), [1]);
        assert_eq!(Traverse::children(&tree.source, 1).collect::<Vec<_>>(), [2]);
        assert_eq!(Traverse::children(&tree.source, 2).collect::<Vec<_>>(), [3]);
        assert!(matches!(
            tree.source.node_input(1).grid_template_columns.as_slice(),
            [TrackComponentOf::Subgrid(_)]
        ));
        assert!(matches!(
            tree.source.node_input(2).grid_template_rows.as_slice(),
            [TrackComponentOf::Subgrid(_)]
        ));
        let root_projection = GridContainerProjection::from_node(tree.source.node_input(0));
        let outer_projection = GridContainerProjection::from_node(tree.source.node_input(1));
        let outer_item = GridItemProjection::from_node(tree.source.node_input(1));
        let inner_item = GridItemProjection::from_node(tree.source.node_input(2));
        let inherited_outer = map_grid_axis(GridAxisMappingInput {
            queried_axis: GridAxisKind::Column,
            parent_style: &root_projection,
            child_style: &outer_item,
        });
        let inherited_inner = map_grid_axis(GridAxisMappingInput {
            queried_axis: GridAxisKind::Row,
            parent_style: &outer_projection,
            child_style: &inner_item,
        });
        assert_eq!(inherited_outer.parent_axis, GridAxisKind::Row);
        assert_eq!(inherited_inner.parent_axis, GridAxisKind::Column);
        assert_eq!(inherited_outer.reversed, direction == Direction::Rtl);
        assert_eq!(inherited_inner.reversed, direction == Direction::Rtl);
        let parent_flow =
            crate::FlowAxes::new(root_projection.writing_mode, root_projection.direction);
        let outer_flow =
            crate::FlowAxes::new(outer_projection.writing_mode, outer_projection.direction);
        let inner_flow = crate::FlowAxes::new(inner_item.writing_mode, inner_item.direction);
        let mapped = GridSizingStage::COMPLETE.mapped(parent_flow, outer_flow);
        assert_eq!(mapped.first_axis, GridAxisKind::Row);
        assert_eq!(
            mapped.mapped(outer_flow, inner_flow).first_axis,
            GridAxisKind::Column
        );
        // The maintained physical host has MIN20/MAX40 before rows, then
        // MIN10/MAX30 after real finite row space; height30 is constant.
        // Formal Auto floors therefore remain >=10 even under maximum5.
        // Hidden makes Normal's used minimum0, allowing actual allocation5.
        let prior = crate::compute_layout(
            &tree,
            0,
            mixed_root_request(SizingOperation::MaxContent, SizingOperation::MaxContent),
        )
        .unwrap();
        assert_eq!(mixed_root_output(&prior).size, Size::new(n(5.0), n(30.0)));
        let prior_entries = readback::continuous_custody(&prior, Phase::Final);
        let formal_row_at_five = |input: &ComputeInputOf<S>| {
            let axes = input.sizing_request().axes();
            input.known().width.is_none()
                && axes.height.operation() == SizingOperation::MaxContent
                && axes
                    .width
                    .space()
                    .definite_size()
                    .is_some_and(|value| value.get() == n(5.0))
        };
        assert!(
            !tree
                .requests
                .borrow()
                .iter()
                .any(|(_, input)| formal_row_at_five(input))
        );
        tree.requests.borrow_mut().clear();
        let mixed = crate::compute_layout(
            &tree,
            0,
            mixed_root_request(SizingOperation::Normal, SizingOperation::MaxContent),
        )
        .unwrap();
        assert_eq!(mixed_root_output(&mixed).size, Size::new(n(5.0), n(30.0)));
        assert!(
            tree.requests.borrow().iter().any(|(node, input)| {
                *node == 3
                    && input.containing_flow_axes() == inner_flow
                    && formal_row_at_five(input)
            }),
            "the actual nested leaf receives the changed physical frame through both authentic inherited mappings"
        );
        assert_eq!(
            readback::continuous_custody(&prior, Phase::Final).as_slice(),
            prior_entries.as_slice()
        );
    }
}
#[test]
fn nested_row_first_and_reversed_mappings_keep_mixed_receiver_authority_f32() {
    assert_nested_mapped_mixed_receiver::<f32>();
}
#[test]
fn nested_row_first_and_reversed_mappings_keep_mixed_receiver_authority_f64() {
    assert_nested_mapped_mixed_receiver::<f64>();
}

fn assert_preferred_operation_is_mode_owned<S: LayoutScalar>() {
    let node = NodeInputOf::<S> {
        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MIN_CONTENT),
        ..NodeInputOf::default()
    };
    let style = GridContainerProjection::from_node(&node);
    let flow = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
    let finite =
        AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(S::from_f64(20.0)).unwrap());
    let issued = Size::new(
        AxisSizingRequestOf::new(finite, SizingOperation::MinContent),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
    );
    let inherent =
        intrinsic_container_available(&style, flow, Size::NONE, issued, SizingMode::InherentSize)
            .unwrap();
    assert_eq!(inherent.inline.operation(), SizingOperation::MaxContent);
    assert_eq!(inherent.inline.space(), finite);
    assert_eq!(inherent.block.operation(), SizingOperation::MinContent);
    assert_eq!(inherent.block.space(), AvailableSpaceOf::Indefinite);
    let content =
        intrinsic_container_available(&style, flow, Size::NONE, issued, SizingMode::ContentSize)
            .unwrap();
    assert_eq!(content, flow.logical_size(issued));
}
#[test]
fn preferred_intrinsic_operation_is_suppressed_only_for_content_size_f32() {
    assert_preferred_operation_is_mode_owned::<f32>();
}
#[test]
fn preferred_intrinsic_operation_is_suppressed_only_for_content_size_f64() {
    assert_preferred_operation_is_mode_owned::<f64>();
}

fn assert_formal_percent_track_floor_preserves_operation<S: LayoutScalar>() {
    let n = S::from_f64;
    for (operation, preferred, expected) in [
        (
            SizingOperation::MinContent,
            PreferredSizeOf::MIN_CONTENT,
            n(40.0),
        ),
        (
            SizingOperation::MaxContent,
            PreferredSizeOf::MAX_CONTENT,
            n(80.0),
        ),
    ] {
        let mut tree = StageTree::<S>::new(Measurement::FixedIntrinsicWidths);
        let mut root = tree.source.node_input(0).clone();
        root.size.width = preferred;
        root.grid_template_columns = vec![TrackComponentOf::percent(S::ONE)];
        tree.source = tree.source.style(0, root);
        let physical =
            AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(n(100.0)).unwrap());
        let request = crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(physical, operation),
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Indefinite,
                        SizingOperation::MaxContent,
                    ),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        assert!(
            tree.source.node_input(0).size.width.is_min_content()
                || tree.source.node_input(0).size.width.is_max_content()
        );
        // This real opaque host declares MIN40/MAX80 independently of its
        // finite physical opportunity. The cyclic 100% track is Auto during
        // intrinsic sizing; no definite percentage parent was supplied.
        // A performed formal operation must not donate physical100 as a
        // normal percentage floor after choosing its intrinsic extent.
        let batch = crate::compute_layout(&tree, 0, request).unwrap();
        assert_eq!(mixed_root_output(&batch).size, Size::new(expected, n(30.0)));
        assert!(tree.requests.borrow().iter().any(|(node, input)| {
            *node == 1
                && input.known().width.is_none()
                && input.sizing_request().axes().width.operation() == operation
        }));
    }
}
#[test]
fn formal_percent_track_floor_keeps_finite_physical_space_separate_f32() {
    assert_formal_percent_track_floor_preserves_operation::<f32>();
}
#[test]
fn formal_percent_track_floor_keeps_finite_physical_space_separate_f64() {
    assert_formal_percent_track_floor_preserves_operation::<f64>();
}

#[test]
fn cyclic_grid_percentage_margin_uses_the_final_track_only_for_placement() {
    let mut differences = Vec::new();
    for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for (column, container_width, track_width, margin) in [
                (TrackComponentOf::px(100.0_f64), 120.0, 100.0, 1.0),
                (TrackComponentOf::AUTO, 50.0, 50.0, 0.5),
            ] {
                let tree = PublicLayoutTreeOf::new()
                    .children(0, [1, 2])
                    .children(1, [])
                    .children(2, [])
                    .style(
                        0,
                        NodeInputOf {
                            display: Display::Grid,
                            direction,
                            box_sizing,
                            size: Size::new(
                                PreferredSizeOf::px(container_width),
                                PreferredSizeOf::px(20.0),
                            ),
                            grid_template_columns: vec![column],
                            grid_template_rows: vec![TrackComponentOf::px(10.0); 2],
                            justify_content: Some(AlignContent::Start),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        1,
                        NodeInputOf {
                            display: Display::Grid,
                            direction,
                            box_sizing,
                            size: Size::new(PreferredSizeOf::px(50.0), PreferredSizeOf::px(10.0)),
                            margin: Edges::all(LengthAutoOf::percent(0.01)),
                            ..NodeInputOf::default()
                        },
                    )
                    .style(
                        2,
                        NodeInputOf {
                            display: Display::Block,
                            direction,
                            box_sizing,
                            ..NodeInputOf::default()
                        },
                    );
                let batch = crate::compute_layout(
                    &tree,
                    0,
                    mixed_root_request(SizingOperation::Normal, SizingOperation::Normal),
                )
                .unwrap();
                let unrounded = |node| readback::single_box_output(&batch, node, Phase::Unrounded);
                let child = unrounded(1);
                // The empty item stretches across the shared column in its own row.
                let track_probe = unrounded(2);
                let x = match direction {
                    Direction::Ltr => margin,
                    Direction::Rtl => container_width - 50.0 - margin,
                };
                let actual = (track_probe.size.width, child.margin, child.location.x);
                let expected = (track_width, Edges::all(margin), x);
                if actual != expected {
                    differences.push(format!(
                        "{box_sizing:?}, {direction:?}, container width {container_width}: \
                         {actual:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn cyclic_grid_margin_does_not_shrink_a_fixed_maximum_track_cap() {
    let actual = [20.0_f64, 40.0].map(|container_width| {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(
                        PreferredSizeOf::px(container_width),
                        PreferredSizeOf::px(20.0),
                    ),
                    grid_template_columns: vec![TrackComponentOf::minmax(
                        MinTrackSizingOf::AUTO,
                        MaxTrackSizingOf::px(100.0),
                    )],
                    grid_template_rows: vec![TrackComponentOf::px(10.0); 2],
                    justify_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    margin: Edges {
                        left: LengthAutoOf::percent(0.5),
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .measure(1, Size::new(200.0, 10.0))
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            );
        let batch = crate::compute_layout(
            &tree,
            0,
            mixed_root_request(SizingOperation::Normal, SizingOperation::Normal),
        )
        .unwrap();
        let track_probe = batch
            .node_layout(2)
            .map(|layout| match layout {
                crate::NodeLayoutRefOf::SingleBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::NonBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::FragmentedBox(_) => {
                    panic!("continuous fixture unexpectedly fragmented")
                }
            })
            .unwrap();
        track_probe.output().size.width
    });
    // The item's 200px content minimum is capped by its area's 100px
    // maximum. The cyclic margin is zero in this intrinsic calculation.
    assert_eq!(actual, [100.0, 100.0]);
}

fn assert_unissued_track_estimate_does_not_allocate_scrollbar<S: LayoutScalar>(
    align_content: Option<AlignContent>,
) {
    let px = S::from_f64;
    let flow = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(flow, crate::ParentFormattingContext::BlockFlow),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    for (row, complete_gutter) in [
        (
            TrackComponentOf::minmax(
                crate::MinTrackSizingOf::px(S::ZERO),
                crate::MaxTrackSizingOf::px(px(100.0)),
            ),
            S::ZERO,
        ),
        (TrackComponentOf::px(px(60.0)), px(10.0)),
    ] {
        let tree = PublicLayoutTreeOf::<S>::new().children(0, []).style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(px(100.0)),
                    PreferredSizeOf::px(px(20.0)),
                ),
                overflow: crate::ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(px(10.0)).unwrap(),
                grid_template_columns: vec![TrackComponentOf::px(px(80.0))],
                grid_template_rows: vec![row],
                align_content,
                ..NodeInputOf::default()
            },
        );
        for (through, expected_gutter) in [
            (GridSizingPhase::InitialFirst, S::ZERO),
            (GridSizingPhase::Complete, complete_gutter),
        ] {
            let observation = crate::engine::observe_grid_probe(
                &tree,
                0,
                GridOverflowProbe {
                    layout: GridOverflowLayout::RootPrefix {
                        input,
                        stage: GridSizingStage {
                            first_axis: GridAxisKind::Column,
                            through,
                        },
                    },
                },
            )
            .unwrap();
            let GridOverflowObservation::Intrinsic(observation) = observation else {
                panic!("root prefix returns an intrinsic observation");
            };
            let gutter = observation.effective_gutter.expect("definite scrollport");
            assert_eq!(gutter.left + gutter.right, expected_gutter, "{through:?}");
        }
    }
}

#[test]
fn unissued_track_estimate_does_not_allocate_scrollbar() {
    assert_unissued_track_estimate_does_not_allocate_scrollbar::<f32>(None);
    assert_unissued_track_estimate_does_not_allocate_scrollbar::<f64>(None);
}

#[test]
fn unissued_centered_track_estimate_does_not_allocate_scrollbar() {
    assert_unissued_track_estimate_does_not_allocate_scrollbar::<f32>(Some(AlignContent::Center));
    assert_unissued_track_estimate_does_not_allocate_scrollbar::<f64>(Some(AlignContent::Center));
}
