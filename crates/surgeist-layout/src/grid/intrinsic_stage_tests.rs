use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    LayoutInputViewOf, LayoutTree, LeafMeasureInputOf, MeasurementAvailableOf, NodeInputOf,
};

#[derive(Clone, Copy, Debug)]
enum Measurement {
    RefuseResolvedWidth,
    SmallerAfterRows,
    MaximumOnlyChanges,
    LastRowsChange,
    HiddenMinimumChange,
    OppositeExtent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedWidth;

struct StageTree<S: LayoutScalar> {
    source: PublicLayoutTreeOf<S>,
    measurement: Measurement,
    leaf_start: u32,
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
            Size::splat(AvailableOf::MAX_CONTENT),
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
        let available = input.available_content_size();
        let width = available
            .width
            .definite_value()
            .map(crate::NonNegativeFiniteOf::get);
        let rows_known = available.height.definite_value().is_some();
        if matches!(self.measurement, Measurement::RefuseResolvedWidth) && width.is_some() {
            return Some(Err(ResolvedWidth));
        }
        let minimum = available.width == MeasurementAvailableOf::MIN_CONTENT;
        // These are declared host measurements, dependent only on the actual
        // content-space request. They model the opposite-axis dependencies in
        // Grid 2's four sizing stages; no callback receives a phase label.
        let measured_width = match self.measurement {
            Measurement::OppositeExtent => {
                if minimum {
                    10.0
                } else {
                    return Some(Ok(Size::new(
                        available
                            .height
                            .definite_value()
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
                    .definite_value()
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
        let error = tree
            .observe(GridSizingPhase::InitialSecond)
            .err()
            .expect("row sizing reaches the actual width-dependent provider refusal");
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement(ResolvedWidth)
        ));
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

fn assert_retained_prefix<S: LayoutScalar>() {
    let mut tree = StageTree::<S>::nested();
    tree.measurement = Measurement::MaximumOnlyChanges;
    let cold = tree.observe(GridSizingPhase::Complete).unwrap().output;
    // A complete public layout correctly discards these isolated keys. Seed
    // the actual observed keys through a real owning Session instead.
    let requests = tree.requests.take();
    assert!(!requests.is_empty());
    let seed = crate::engine::seed_grid_leaf_requests(&tree, &requests).unwrap();
    tree.caches = seed.cache_store_entries().to_vec();
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
    let second = tree.observe(GridSizingPhase::InitialSecond);
    assert_eq!(first.unwrap().output.size.width, S::from_f64(40.0));
    assert!(matches!(
        second.err().expect("later stage refuses").kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
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
                root.grid_auto_flow = GridAutoFlow::Row;
                (GridAxisKind::Column, GridAxisKind::Row)
            } else {
                root.grid_auto_flow = GridAutoFlow::Column;
                (GridAxisKind::Row, GridAxisKind::Column)
            };
            tree.source = tree.source.style(0, root);
            let prefix = tree
                .observe_at(grid_axis, GridSizingPhase::InitialFirst)
                .unwrap();
            assert_eq!(prefix.output.size.width, S::from_f64(40.0));
            assert!(prefix.effective_gutter.is_none());
            for (first_axis, through) in [
                (grid_axis, GridSizingPhase::InitialSecond),
                (stacking_axis, GridSizingPhase::InitialFirst),
                (grid_axis, GridSizingPhase::Complete),
            ] {
                let error = tree.observe_at(first_axis, through).err().expect(
                    "actual stacking-size and completed operations reach the host dependency",
                );
                assert!(matches!(
                    error.kind(),
                    LayoutErrorKindOf::Measurement(ResolvedWidth)
                ));
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
    root.grid_template_rows = vec![TrackComponentOf::px(S::from_f64(20.0))];
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
            .any(|(_, input)| input.available().width == AvailableOf::Definite(S::from_f64(40.0)))
    );

    let refusing = lanes_cutoff_tree::<S>(false);
    let request =
        crate::LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap();
    let error = crate::compute_layout(&refusing, 0, request).expect_err(
        "public full layout must still propagate the actual stacking measurement refusal",
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement(ResolvedWidth)
    ));
}

#[test]
fn lanes_cutoff_preserves_qualified_auto_stacking_size_and_full_layout_work() {
    assert_lanes_required_work::<f32>();
    assert_lanes_required_work::<f64>();
}
