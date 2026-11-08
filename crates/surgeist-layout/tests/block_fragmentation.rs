#![forbid(unsafe_code)]

#[path = "support/native_request.rs"]
mod native_request;

use std::cell::{Cell, RefCell};
use surgeist_layout::*;

type Batch<S> = CompletedLayoutBatchOf<usize, S>;
type Computed<S> = LayoutResultOf<usize, Batch<S>, S, &'static str>;

// Existing geometry oracles exercise principal payloads explicitly.
trait PrincipalGeometry<S: LayoutScalar> {
    fn unrounded(&self) -> BoxFragmentGeometryOf<S>;
    fn final_geometry(&self) -> BoxFragmentGeometryOf<S>;
}
impl<S: LayoutScalar> PrincipalGeometry<S> for FragmentOutputOf<usize, S> {
    fn unrounded(&self) -> BoxFragmentGeometryOf<S> {
        self.principal_box()
            .expect("principal fragment")
            .unrounded()
    }
    fn final_geometry(&self) -> BoxFragmentGeometryOf<S> {
        self.principal_box()
            .expect("principal fragment")
            .final_geometry()
    }
}

fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn block<S: LayoutScalar>(width: Option<f64>, height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            width.map_or(PreferredSizeOf::AUTO, px),
            height.map_or(PreferredSizeOf::AUTO, px),
        ),
        ..NodeInputOf::default()
    }
}

#[derive(Clone, Copy)]
enum Measurement {
    Responsive,
    ResponsiveRejectNormalWidth(f64),
    Fixed(f64, f64),
    IntrinsicRange,
    IntrinsicIcb,
}

struct Tree<S: LayoutScalar> {
    inputs: Vec<NodeInputOf<S>>,
    children: Vec<Vec<usize>>,
    measurements: Vec<Option<Measurement>>,
    observed: RefCell<Vec<(usize, LeafMeasureInputOf<S>)>>,
    fail_narrow: Cell<bool>,
    cache_calls: Cell<usize>,
    custody_calls: Cell<usize>,
    caches: Vec<CacheOf<S>>,
    custody: Vec<Option<LayoutOutputEntryOf<usize, S>>>,
}

impl<S: LayoutScalar> Tree<S> {
    fn new(inputs: Vec<NodeInputOf<S>>, children: Vec<Vec<usize>>) -> Self {
        let count = inputs.len();
        Self {
            inputs,
            children,
            measurements: vec![None; count],
            observed: RefCell::new(Vec::new()),
            fail_narrow: Cell::new(false),
            cache_calls: Cell::new(0),
            custody_calls: Cell::new(0),
            caches: vec![CacheOf::new(); count],
            custody: vec![None; count],
        }
    }
    fn responsive() -> Self {
        let mut tree = Self::new(
            vec![
                block(None, None),
                block(None, Some(30.0)),
                block(None, None),
            ],
            vec![vec![1, 2], vec![], vec![]],
        );
        tree.measurements[2] = Some(Measurement::Responsive);
        tree
    }
    fn reset_observations(&self) {
        self.observed.borrow_mut().clear();
        self.cache_calls.set(0);
        self.custody_calls.set(0);
    }
}
impl<S: LayoutScalar> Traverse for Tree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;
    fn children(&self, node: usize) -> Self::Children<'_> {
        self.children[node].iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children[node].len()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children[node][index]
    }
}
impl<S: LayoutScalar> LayoutTree for Tree<S> {
    type MeasureError = &'static str;
    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        &self.inputs[node]
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        LayoutInputViewOf::Box(&self.inputs[node])
    }
    fn has_leaf_measurement(&self, node: usize) -> bool {
        self.measurements[node].is_some()
    }
    fn measure_leaf(
        &self,
        node: usize,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        let measurement = self.measurements[node]?;
        self.observed.borrow_mut().push((node, input));
        let available = input
            .content_sizing_request()
            .axes()
            .width
            .space()
            .definite_size()
            .map(NonNegativeFiniteOf::get);
        let known = input.known_content_size();
        Some(match measurement {
            Measurement::Responsive | Measurement::ResponsiveRejectNormalWidth(_) => {
                // A real host-owned content function over actual content-space
                // constraints. A definite imposed size is honored when present.
                let width = known.width.or(available).unwrap_or(S::from_f64(100.0));
                if matches!(measurement,Measurement::ResponsiveRejectNormalWidth(rejected) if normal_content_width(input,rejected))
                {
                    Err("normal content width rejected")
                } else if self.fail_narrow.get() && width == S::from_f64(60.0) {
                    Err("narrow measurement failed")
                } else {
                    Ok(Size::new(
                        width,
                        known.height.unwrap_or(if width <= S::from_f64(60.0) {
                            S::from_f64(40.0)
                        } else {
                            S::from_f64(30.0)
                        }),
                    ))
                }
            }
            Measurement::IntrinsicRange => {
                let width = known.width.unwrap_or_else(|| {
                    match input.content_sizing_request().axes().width.operation() {
                        SizingOperation::MinContent => S::from_f64(10.0),
                        SizingOperation::MaxContent => S::from_f64(80.0),
                        SizingOperation::Normal => available.unwrap_or(S::from_f64(80.0)),
                    }
                });
                Ok(Size::new(width, known.height.unwrap_or(S::from_f64(30.0))))
            }
            Measurement::IntrinsicIcb => {
                // This opaque content's intrinsic inline extent depends on
                // its content fallback block extent. Normal layout honors the offer.
                let width = known.width.unwrap_or_else(|| {
                    match input.content_sizing_request().axes().width.operation() {
                        SizingOperation::MinContent | SizingOperation::MaxContent => input
                            .content_sizing_request()
                            .orthogonal_fallback()
                            .height
                            .expect("intrinsic content needs a content fallback height")
                            .get(),
                        SizingOperation::Normal => available.expect("Normal content has an offer"),
                    }
                });
                Ok(Size::new(width, known.height.unwrap_or(S::from_f64(30.0))))
            }
            Measurement::Fixed(width, height) => Ok(Size::new(
                known.width.unwrap_or(S::from_f64(width)),
                known.height.unwrap_or(S::from_f64(height)),
            )),
        })
    }
    fn cache_get(
        &self,
        node: usize,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.cache_calls.set(self.cache_calls.get() + 1);
        self.caches[node].get_with_context(input, context)
    }
    fn unrounded_layout(&self, node: usize) -> Option<LayoutOutputEntryOf<usize, S>> {
        self.custody_calls.set(self.custody_calls.get() + 1);
        self.custody[node]
    }
}

#[derive(Clone, Debug, PartialEq)]
struct VisibleFragment<S: LayoutScalar> {
    id: LayoutFragmentId,
    parent: Option<FragmentCoordinateSpace>,
    payload: FragmentPayloadOf<usize, S>,
}
#[derive(Clone, Debug, PartialEq)]
enum Visible<S: LayoutScalar> {
    Single(
        LayoutFragmentId,
        Option<FragmentCoordinateSpace>,
        LayoutOutputEntryOf<usize, S>,
        LayoutOutputEntryOf<usize, S>,
    ),
    Fragments(Vec<VisibleFragment<S>>),
    NonBox(NonBoxKind, NonBoxGeometryOf<S>, NonBoxGeometryOf<S>),
}
#[derive(Clone, Debug, PartialEq)]
struct Published<S: LayoutScalar> {
    geometry: Vec<Option<Visible<S>>>,
    caches: Vec<CacheOf<S>>,
    custody: Vec<Option<LayoutOutputEntryOf<usize, S>>>,
}
struct Sink<S: LayoutScalar> {
    targets: LayoutPublicationTargets<usize>,
    state: Published<S>,
    preparations: Cell<usize>,
    commits: usize,
    reject: Cell<bool>,
    previous: RefCell<Vec<usize>>,
}
impl<S: LayoutScalar> Sink<S> {
    fn new(count: usize) -> Self {
        Self {
            targets: LayoutPublicationTargets::new(),
            state: Published {
                geometry: vec![None; count],
                caches: vec![CacheOf::new(); count],
                custody: vec![None; count],
            },
            preparations: Cell::new(0),
            commits: 0,
            reject: Cell::new(false),
            previous: RefCell::new(Vec::new()),
        }
    }
    fn authorize(&mut self, root: usize, request: &LayoutBoundRequestOf<S>) {
        self.targets.install(root, request.target_basis()).unwrap();
    }
    fn copy_cache_to(&self, tree: &mut Tree<S>) {
        tree.caches.clone_from(&self.state.caches);
        tree.custody.clone_from(&self.state.custody);
    }
}
impl<S: LayoutScalar> LayoutBatchSink<usize, S> for Sink<S> {
    type Error = &'static str;
    type Prepared = Published<S>;
    fn publication_targets(&self) -> &LayoutPublicationTargets<usize> {
        &self.targets
    }
    fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<usize> {
        &mut self.targets
    }
    fn prepare_layout_batch<'call>(
        &self,
        preparation: LayoutBatchPreparation<'call, usize, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        self.preparations.set(self.preparations.get() + 1);
        *self.previous.borrow_mut() = preparation.previous_members().to_vec();
        let batch = preparation.batch();
        assert_eq!(preparation.target_basis(), batch.target_basis());
        let mut next = self.state.clone();
        if batch.replaces_whole_root() {
            for &node in preparation.previous_members() {
                next.geometry[node] = None;
                next.custody[node] = None;
                next.caches[node].clear_performed();
            }
            for (node, _) in batch.node_layouts() {
                next.caches[node].clear_performed();
            }
        }
        for &node in batch.invalidated_nodes() {
            next.geometry[node] = None;
            next.custody[node] = None;
            next.caches[node].clear();
        }
        for clear in batch.cache_clear_entries() {
            next.caches[clear.node()].clear();
        }
        for (node, layout) in batch.node_layouts() {
            next.geometry[node] = Some(match layout {
                NodeLayoutRefOf::SingleBox(value) => {
                    next.custody[node] = Some(value.unrounded_entry());
                    Visible::Single(
                        value.id().clone(),
                        value.coordinate_parent().cloned(),
                        value.unrounded_entry(),
                        value.final_entry(),
                    )
                }
                NodeLayoutRefOf::Fragments(fragments) => {
                    next.custody[node] = None;
                    Visible::Fragments(
                        fragments
                            .iter()
                            .map(|value| VisibleFragment {
                                id: value.id().clone(),
                                parent: value.coordinate_parent().cloned(),
                                payload: *value.payload(),
                            })
                            .collect(),
                    )
                }
                NodeLayoutRefOf::NonBox(value) => {
                    next.custody[node] = None;
                    Visible::NonBox(
                        value.kind(),
                        value.unrounded_geometry(),
                        value.final_geometry(),
                    )
                }
            });
        }
        for entry in batch.cache_store_entries() {
            next.caches[entry.node()].store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        if self.reject.get() {
            return Err("preparation rejected");
        }
        Ok(next)
    }
    fn commit_layout_batch<'call>(
        &mut self,
        commit: LayoutBatchCommit<'call, usize, S, Self::Prepared>,
    ) {
        assert_eq!(commit.target_basis(), commit.batch().target_basis());
        assert_eq!(commit.previous_members(), self.previous.borrow().as_slice());
        self.state = commit.into_prepared();
        self.commits += 1;
    }
}

fn axes() -> FlowAxes {
    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
}
fn root_request<S: LayoutScalar>(width: f64) -> LayoutRootRequestOf<S> {
    let sizing = native_request::sizing(
        Size::new(Some(S::from_f64(width)), None),
        Size::splat(SizingOperation::Normal),
    );
    LayoutRootRequestOf::standalone(
        sizing,
        axes(),
        sizing.orthogonal_fallback().initial_containing_block(),
    )
}
fn frame<S: LayoutScalar>(width: f64, height: f64, initial: bool) -> BlockFragmentainerOf<S> {
    BlockFragmentainerOf {
        origin: Point::ZERO,
        size: Size::new(S::from_f64(width), S::from_f64(height)),
        breaks: if initial {
            FragmentationBreakKinds::NONE
        } else {
            FragmentationBreakKinds::PAGE
        },
        page_side: None,
    }
}
fn fragmented<S: LayoutScalar>(
    source: LayoutSourceBasis,
    frames: Vec<BlockFragmentainerOf<S>>,
) -> LayoutBoundRequestOf<S> {
    LayoutBoundRequestOf::block_fragmentation(
        source,
        root_request(100.0),
        FragmentationKind::Page,
        axes(),
        frames,
    )
    .unwrap()
}
fn unequal<S: LayoutScalar>(source: LayoutSourceBasis) -> LayoutBoundRequestOf<S> {
    fragmented(
        source,
        vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
    )
}
fn compute<S: LayoutScalar>(
    tree: &Tree<S>,
    root: usize,
    request: &LayoutBoundRequestOf<S>,
) -> Computed<S> {
    let capture = LayoutFlowInput::try_new_bound(
        tree,
        root,
        request.clone(),
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )?;
    assert_eq!(
        capture.publication_basis().as_ref(),
        Some(&request.target_basis())
    );
    compute_layout_with_flow(&capture, root, request.root_request())
}
fn single<S: LayoutScalar>(batch: &Batch<S>, node: usize) -> SingleBoxOutputRefOf<'_, usize, S> {
    batch
        .node_layout(node)
        .unwrap()
        .single_box()
        .expect("actual single box")
}
fn fragments<S: LayoutScalar>(batch: &Batch<S>, node: usize) -> &[FragmentOutputOf<usize, S>] {
    batch
        .node_layout(node)
        .unwrap()
        .fragments()
        .expect("actual fragmented box")
}
fn assert_packet<S: LayoutScalar>(packet: NodeOutputOf<S>, location: (f64, f64), size: (f64, f64)) {
    assert_eq!(
        packet.location,
        Point::new(S::from_f64(location.0), S::from_f64(location.1))
    );
    assert_eq!(
        packet.size,
        Size::new(S::from_f64(size.0), S::from_f64(size.1))
    );
    assert_eq!(packet.padding, Edges::all(S::ZERO));
    assert_eq!(packet.border, Edges::all(S::ZERO));
    assert_eq!(packet.margin, Edges::all(S::ZERO));
}
fn assert_fragment<S: LayoutScalar>(value: &FragmentOutputOf<usize, S>, size: (f64, f64)) {
    for packet in [value.unrounded(), value.final_geometry()] {
        assert_eq!(
            packet.size,
            Size::new(S::from_f64(size.0), S::from_f64(size.1))
        );
        assert_eq!(packet.padding, Edges::all(S::ZERO));
        assert_eq!(packet.border, Edges::all(S::ZERO));
        assert_eq!(packet.margin, Edges::all(S::ZERO));
    }
}
macro_rules! scalars {
    ($helper:ident,$f32:ident,$f64:ident) => {
        #[test]
        fn $f32() {
            $helper::<f32>();
        }
        #[test]
        fn $f64() {
            $helper::<f64>();
        }
    };
}

fn unequal_frames_remeasure_deferred_native_child<S: LayoutScalar>() {
    let tree = Tree::<S>::responsive();
    let request = unequal(LayoutSourceBasis::fresh());
    let mut sink = Sink::new(3);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request).unwrap();
    assert_eq!(batch.root(), 0);
    assert_eq!(batch.target_basis(), Some(&request.target_basis()));
    assert!(batch.replaces_whole_root());
    assert!(batch.node_layout(0).unwrap().single_box().is_none());
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    // Breaking extends the first parent content box through the unused20 units;
    // the final parent is its actual natural40-unit continuation.
    assert_fragment(&parents[0], (100.0, 50.0));
    assert_fragment(&parents[1], (60.0, 40.0));
    assert_eq!(parents[0].source_children(), [(SourceIndex::new(0), 1)]);
    assert_eq!(parents[1].source_children(), [(SourceIndex::new(1), 2)]);
    assert_eq!(parents[0].previous(), None);
    assert_eq!(parents[0].next(), Some(parents[1].id()));
    assert_eq!(parents[1].previous(), Some(parents[0].id()));
    assert_eq!(parents[1].next(), None);
    assert_eq!(
        (
            parents[0].continuation_order(),
            parents[1].continuation_order()
        ),
        (0, 1)
    );
    assert_ne!(parents[0].fragmentainer(), parents[1].fragmentainer());
    for (node, index, size) in [(1, 0, (100.0, 30.0)), (2, 1, (60.0, 40.0))] {
        let child = single(&batch, node);
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                parents[index].id().clone()
            ))
        );
        assert_packet(child.unrounded_output(), (0.0, 0.0), size);
        assert_packet(child.final_output(), (0.0, 0.0), size);
    }
    let widths = tree
        .observed
        .borrow()
        .iter()
        .filter(|(node, _)| *node == 2)
        .filter_map(|(_, input)| {
            input
                .content_sizing_request()
                .axes()
                .width
                .space()
                .definite_size()
                .map(NonNegativeFiniteOf::get)
        })
        .collect::<Vec<_>>();
    assert!(widths.contains(&S::from_f64(100.0)));
    assert!(widths.contains(&S::from_f64(60.0)));
    assert_eq!(tree.cache_calls.get(), 0);
    assert_eq!(tree.custody_calls.get(), 0);
    assert!(batch.cache_store_entries().is_empty());
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(sink.commits, 1);
}
scalars!(
    unequal_frames_remeasure_deferred_native_child,
    unequal_frames_remeasure_deferred_native_child_f32,
    unequal_frames_remeasure_deferred_native_child_f64
);

fn oversized_native_box_and_forced_sibling_preserve_geometry<S: LayoutScalar>() {
    let mut second = block::<S>(None, Some(50.0));
    second.break_before = FragmentationBreak::Page;
    let mut tree = Tree::<S>::new(
        vec![block(None, None), block(None, None), second],
        vec![vec![1, 2], vec![], vec![]],
    );
    tree.measurements[1] = Some(Measurement::Fixed(100.0, 150.0));
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(100.0, 100.0, false)],
    );
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    assert_fragment(&parents[0], (100.0, 150.0));
    assert_fragment(&parents[1], (100.0, 50.0));
    assert_eq!(parents[0].source_children(), [(SourceIndex::new(0), 1)]);
    assert_eq!(parents[1].source_children(), [(SourceIndex::new(1), 2)]);
    for (node, index, height) in [(1, 0, 150.0), (2, 1, 50.0)] {
        let child = single(&batch, node);
        assert_packet(child.unrounded_output(), (0.0, 0.0), (100.0, height));
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                parents[index].id().clone()
            ))
        );
    }
}
scalars!(
    oversized_native_box_and_forced_sibling_preserve_geometry,
    oversized_native_box_and_forced_sibling_preserve_geometry_f32,
    oversized_native_box_and_forced_sibling_preserve_geometry_f64
);

fn native_zero_boxes_follow_the_actual_break_side<S: LayoutScalar>() {
    for overflowing in [false, true] {
        let first = if overflowing { 150.0 } else { 30.0 };
        let mut tree = Tree::<S>::new(
            vec![
                block(None, None),
                block(None, Some(first)),
                block(None, Some(0.0)),
                block(None, Some(0.0)),
                block(None, Some(30.0)),
            ],
            vec![vec![1, 2, 3, 4], vec![], vec![], vec![], vec![]],
        );
        if overflowing {
            tree.inputs[1].size.height = PreferredSizeOf::AUTO;
            tree.measurements[1] = Some(Measurement::Fixed(100.0, 150.0));
        }
        let request = fragmented(
            LayoutSourceBasis::fresh(),
            vec![frame(100.0, 50.0, true), frame(100.0, 80.0, false)],
        );
        let batch = compute(&tree, 0, &request).unwrap();
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        assert_fragment(&parents[0], (100.0, if overflowing { 150.0 } else { 50.0 }));
        assert_fragment(&parents[1], (100.0, 30.0));
        for node in [2, 3] {
            let child = single(&batch, node);
            let index = usize::from(overflowing);
            assert_eq!(
                child.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    parents[index].id().clone()
                ))
            );
            assert_packet(
                child.unrounded_output(),
                (0.0, if overflowing { 0.0 } else { 30.0 }),
                (100.0, 0.0),
            );
        }
        let earlier = if overflowing {
            vec![(SourceIndex::new(0), 1)]
        } else {
            vec![
                (SourceIndex::new(0), 1),
                (SourceIndex::new(1), 2),
                (SourceIndex::new(2), 3),
            ]
        };
        assert_eq!(parents[0].source_children(), earlier);
        assert_packet(
            single(&batch, 4).unrounded_output(),
            (0.0, 0.0),
            (100.0, 30.0),
        );
    }
}
scalars!(
    native_zero_boxes_follow_the_actual_break_side,
    native_zero_boxes_follow_the_actual_break_side_f32,
    native_zero_boxes_follow_the_actual_break_side_f64
);

fn rtl_and_orthogonal_children_keep_actual_parent_spaces<S: LayoutScalar>() {
    let mut tree = Tree::<S>::new(
        vec![
            block(None, None),
            block(Some(20.0), Some(30.0)),
            block(Some(20.0), Some(40.0)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    for input in &mut tree.inputs {
        input.direction = Direction::Rtl;
    }
    let rtl_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Rtl);
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        root_request(100.0),
        FragmentationKind::Page,
        rtl_axes,
        vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_fragment(&parents[0], (100.0, 50.0));
    assert_fragment(&parents[1], (60.0, 40.0));
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (80.0, 0.0),
        (20.0, 30.0),
    );
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (40.0, 0.0),
        (20.0, 40.0),
    );

    let mut wrapper = block::<S>(Some(10.0), Some(30.0));
    wrapper.writing_mode = WritingMode::VerticalRl;
    let mut nested = block::<S>(Some(4.0), Some(6.0));
    nested.writing_mode = WritingMode::VerticalRl;
    let tree = Tree::<S>::new(
        vec![block(None, None), block(None, Some(30.0)), wrapper, nested],
        vec![vec![1, 2], vec![], vec![3], vec![]],
    );
    let request = unequal(LayoutSourceBasis::fresh());
    let batch = compute(&tree, 0, &request).unwrap();
    let parent = fragments(&batch, 0);
    assert_fragment(&parent[1], (60.0, 30.0));
    let wrapper = single(&batch, 2);
    assert_packet(wrapper.unrounded_output(), (0.0, 0.0), (10.0, 30.0));
    assert_eq!(
        wrapper.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(parent[1].id().clone()))
    );
    let nested = single(&batch, 3);
    assert_packet(nested.unrounded_output(), (6.0, 0.0), (4.0, 6.0));
    assert_eq!(
        nested.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(wrapper.id().clone()))
    );
}
scalars!(
    rtl_and_orthogonal_children_keep_actual_parent_spaces,
    rtl_and_orthogonal_children_keep_actual_parent_spaces_f32,
    rtl_and_orthogonal_children_keep_actual_parent_spaces_f64
);

fn result_handles_are_distinct_even_for_cloned_targets<S: LayoutScalar>() {
    let tree = Tree::<S>::responsive();
    let request = unequal(LayoutSourceBasis::fresh());
    let first = compute(&tree, 0, &request).unwrap();
    let second = compute(&tree, 0, &request.clone()).unwrap();
    assert_eq!(first.target_basis(), second.target_basis());
    let old = fragments(&first, 0)[0].id().clone();
    let new = fragments(&second, 0)[0].id().clone();
    assert_ne!(old, new);
    assert_eq!(first.fragment(&old).unwrap().node(), 0);
    assert_eq!(second.fragment(&new).unwrap().node(), 0);
    assert!(matches!(
        second.fragment(&old),
        Err(LayoutHandleError::WrongResult)
    ));
    assert!(matches!(
        first.fragment(&new),
        Err(LayoutHandleError::WrongResult)
    ));
    let retained = first.clone();
    assert_eq!(retained.fragment(&old).unwrap().id(), &old);
    let single_id = single(&first, 1).id().clone();
    assert_eq!(first.fragment(&single_id).unwrap().id(), &single_id);
}
scalars!(
    result_handles_are_distinct_even_for_cloned_targets,
    result_handles_are_distinct_even_for_cloned_targets_f32,
    result_handles_are_distinct_even_for_cloned_targets_f64
);

fn target_authorization_rejects_stale_or_unbound_publication<S: LayoutScalar>() {
    let tree = Tree::<S>::responsive();
    let source = LayoutSourceBasis::fresh();
    let original = unequal(source.clone());
    let mut sink = Sink::new(3);
    let batch = compute(&tree, 0, &original).unwrap();
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::MissingTarget)
    );
    assert_eq!(sink.preparations.get(), 0);
    sink.authorize(0, &original);
    batch.apply_to(&mut sink).unwrap();
    let before = sink.state.clone();
    let commits = sink.commits;
    let unbound = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    assert_eq!(
        unbound.apply_to(&mut sink),
        Err(LayoutBatchApplyError::TrackedRootRequiresBoundRequest)
    );
    let same_source_new_constraints = fragmented(
        source,
        vec![frame(100.0, 60.0, true), frame(60.0, 80.0, false)],
    );
    sink.authorize(0, &same_source_new_constraints);
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::StaleConstraints)
    );
    let changed_source = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &changed_source);
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::StaleSource)
    );
    assert_eq!(sink.state, before);
    assert_eq!(sink.commits, commits);
    assert_eq!(sink.preparations.get(), 1);
    assert_eq!(sink.targets.get(0), Some(&changed_source.target_basis()));
}
scalars!(
    target_authorization_rejects_stale_or_unbound_publication,
    target_authorization_rejects_stale_or_unbound_publication_f32,
    target_authorization_rejects_stale_or_unbound_publication_f64
);

fn second_frame_failure_and_prepare_failure_are_atomic<S: LayoutScalar>() {
    let tree = Tree::<S>::responsive();
    let old = LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
    let mut sink = Sink::new(3);
    sink.authorize(0, &old);
    compute(&tree, 0, &old)
        .unwrap()
        .apply_to(&mut sink)
        .unwrap();
    let before = sink.state.clone();
    let commits = sink.commits;
    tree.fail_narrow.set(true);
    tree.reset_observations();
    let failing = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &failing);
    let error = compute(&tree, 0, &failing).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement("narrow measurement failed")
    ));
    assert!(tree.observed.borrow().iter().any(|(_, input)| {
        input
            .content_sizing_request()
            .axes()
            .width
            .space()
            .definite_size()
            .is_some_and(|width| width.get() == S::from_f64(60.0))
    }));
    assert_eq!(sink.state, before);
    assert_eq!(sink.commits, commits);
    tree.fail_narrow.set(false);
    let succeeding = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &succeeding);
    let batch = compute(&tree, 0, &succeeding).unwrap();
    sink.reject.set(true);
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::Preparation("preparation rejected"))
    );
    assert_eq!(sink.state, before);
    assert_eq!(sink.commits, commits);
    sink.reject.set(false);
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(sink.commits, commits + 1);
    assert!(matches!(
        sink.state.geometry[0],
        Some(Visible::Fragments(_))
    ));
}
scalars!(
    second_frame_failure_and_prepare_failure_are_atomic,
    second_frame_failure_and_prepare_failure_are_atomic_f32,
    second_frame_failure_and_prepare_failure_are_atomic_f64
);

fn bound_replacement_removes_obsolete_and_detached_geometry<S: LayoutScalar>() {
    let mut tree = Tree::<S>::responsive();
    let source = LayoutSourceBasis::fresh();
    let fragmented = unequal(source.clone());
    let continuous = LayoutBoundRequestOf::continuous(source, root_request(100.0));
    let mut sink = Sink::new(3);
    sink.authorize(0, &continuous);
    let first = compute(&tree, 0, &continuous).unwrap();
    assert_packet(
        single(&first, 0).unrounded_output(),
        (0.0, 0.0),
        (100.0, 60.0),
    );
    first.apply_to(&mut sink).unwrap();
    sink.authorize(0, &fragmented);
    let divided = compute(&tree, 0, &fragmented).unwrap();
    divided.apply_to(&mut sink).unwrap();
    assert!(matches!(
        sink.state.geometry[0],
        Some(Visible::Fragments(_))
    ));
    sink.authorize(0, &continuous);
    compute(&tree, 0, &continuous)
        .unwrap()
        .apply_to(&mut sink)
        .unwrap();
    assert!(matches!(sink.state.geometry[0], Some(Visible::Single(..))));
    tree.children[0] = vec![1];
    let detached =
        LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
    sink.authorize(0, &detached);
    let next = compute(&tree, 0, &detached).unwrap();
    assert!(next.node_layout(2).is_none());
    next.apply_to(&mut sink).unwrap();
    assert!(sink.previous.borrow().contains(&2));
    assert!(sink.state.geometry[2].is_none());
    assert!(sink.state.custody[2].is_none());
    assert_eq!(sink.targets.published_members(0), [0, 1]);
    assert_packet(
        single(&next, 0).unrounded_output(),
        (0.0, 0.0),
        (100.0, 30.0),
    );
    // Retaining an old completed result retains its own valid fragment graph.
    assert_eq!(fragments(&divided, 0).len(), 2);
}
scalars!(
    bound_replacement_removes_obsolete_and_detached_geometry,
    bound_replacement_removes_obsolete_and_detached_geometry_f32,
    bound_replacement_removes_obsolete_and_detached_geometry_f64
);

fn bound_cache_bypass_preserves_actual_measurements<S: LayoutScalar>() {
    let mut tree = Tree::<S>::responsive();
    let mut sink = Sink::new(3);
    // The real atomic leaf owns its intrinsic width over its complete content.
    // Its intrinsic queries exercise bound probes without requiring unsupported
    // whole-source intrinsic custody for the continuing ordinary root.
    tree.inputs[2].size.width = PreferredSizeOf::MIN_CONTENT;
    tree.measurements[2] = Some(Measurement::Fixed(80.0, 30.0));
    let seed = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    let measurement_input = ComputeInputOf::leaf_content_size(
        Size::NONE,
        Size::new(Some(S::from_f64(100.0)), None),
        ContainingLayoutContext::new(axes(), ParentFormattingContext::BlockFlow),
        native_request::sizing(
            Size::new(Some(S::from_f64(100.0)), None),
            Size::splat(SizingOperation::Normal),
        ),
    )
    .unwrap();
    let measurement_output = compute_leaf(measurement_input, &tree.inputs[2], |input| {
        tree.measure_leaf(2, input).unwrap()
    })
    .unwrap();
    assert_eq!(
        measurement_output.size,
        Size::new(S::from_f64(80.0), S::from_f64(30.0))
    );
    assert!(measurement_output.scroll_geometry.is_none());
    let measurement_context = CacheKeyContext::new();
    let measurements = [(
        2,
        measurement_input,
        measurement_context,
        measurement_output,
    )];
    let performed = seed
        .cache_store_entries()
        .iter()
        // The actual leaf producer attaches canonical scroll geometry only
        // during performed layout.
        .filter(|entry| entry.node() == 2 && entry.output().scroll_geometry.is_some())
        .map(|entry| (entry.node(), *entry.input(), entry.context()))
        .collect::<Vec<_>>();
    assert!(!performed.is_empty());
    seed.apply_to(&mut sink).unwrap();
    sink.state.caches[2].store_with_context(
        &measurement_input,
        measurement_context,
        measurement_output,
    );
    sink.copy_cache_to(&mut tree);
    for (node, input, context, output) in &measurements {
        assert_eq!(
            tree.caches[*node].get_with_context(input, *context),
            Some(*output)
        );
    }
    for (node, input, context) in &performed {
        assert!(
            tree.caches[*node]
                .get_with_context(input, *context)
                .is_some()
        );
    }
    for fragmented_mode in [true, false] {
        tree.reset_observations();
        let source = LayoutSourceBasis::fresh();
        let request = if fragmented_mode {
            unequal(source)
        } else {
            LayoutBoundRequestOf::continuous(source, root_request(100.0))
        };
        sink.authorize(0, &request);
        let batch = compute(&tree, 0, &request).unwrap();
        assert_eq!(tree.cache_calls.get(), 0);
        assert_eq!(tree.custody_calls.get(), 0);
        assert!(batch.cache_store_entries().is_empty());
        assert_eq!(
            single(&batch, 2).unrounded_output().size,
            Size::new(S::from_f64(80.0), S::from_f64(30.0))
        );
        assert!(
            tree.observed.borrow().iter().any(|(node, input)| *node == 2
                && input.content_sizing_request().axes().width.operation()
                    == SizingOperation::MinContent),
            "bound execution performs the real atomic intrinsic operation"
        );
        batch.apply_to(&mut sink).unwrap();
        for (node, input, context, output) in &measurements {
            assert_eq!(
                sink.state.caches[*node].get_with_context(input, *context),
                Some(*output)
            );
        }
        for (node, input, context) in &performed {
            assert_eq!(
                sink.state.caches[*node].get_with_context(input, *context),
                None
            );
        }
        sink.copy_cache_to(&mut tree);
    }
}
scalars!(
    bound_cache_bypass_preserves_actual_measurements,
    bound_cache_bypass_preserves_actual_measurements_f32,
    bound_cache_bypass_preserves_actual_measurements_f64
);

fn unsupported_inline_box_refuses_complete_native_result<S: LayoutScalar>() {
    let mut inline = block::<S>(Some(10.0), Some(10.0));
    inline.display = Display::InlineBlock;
    inline.atomic_inline_participation = Some(AtomicInlineParticipation::new(
        surgeist_text::BidiLevel::try_new(0).unwrap(),
        InlineBreakOpportunity::Prohibited,
    ));
    let tree = Tree::<S>::new(vec![block(None, None), inline], vec![vec![1], vec![]]);
    let request = unequal(LayoutSourceBasis::fresh());
    let error = compute(&tree, 0, &request).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile
        )
    ));
}
scalars!(
    unsupported_inline_box_refuses_complete_native_result,
    unsupported_inline_box_refuses_complete_native_result_f32,
    unsupported_inline_box_refuses_complete_native_result_f64
);

fn performed_host_leaf_keeps_measured_content_under_fixed_box<S: LayoutScalar>() {
    let mut tree = Tree::<S>::new(
        vec![block(None, None), block(Some(20.0), Some(10.0))],
        vec![vec![1], vec![]],
    );
    tree.measurements[1] = Some(Measurement::Fixed(40.0, 30.0));
    let request = unequal(LayoutSourceBasis::fresh());
    let batch = compute(&tree, 0, &request).unwrap();
    let leaf = single(&batch, 1).unrounded_output();
    assert_packet(leaf, (0.0, 0.0), (20.0, 10.0));
    assert_eq!(
        leaf.content_size,
        Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
    assert_eq!(
        leaf.scroll_geometry.unwrap().scrollable_overflow().size(),
        Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
}
scalars!(
    performed_host_leaf_keeps_measured_content_under_fixed_box,
    performed_host_leaf_keeps_measured_content_under_fixed_box_f32,
    performed_host_leaf_keeps_measured_content_under_fixed_box_f64
);

// A defensive control for an unstable target provider. Exclusive publication
// does not authorize switching owners during preparation; this safe adversary
// selects two preinstalled empty registries to verify the final admission check.
struct UnstableTargets<S: LayoutScalar> {
    sink: Sink<S>,
    alternate: LayoutPublicationTargets<usize>,
    switched: Cell<bool>,
}
impl<S: LayoutScalar> LayoutBatchSink<usize, S> for UnstableTargets<S> {
    type Error = &'static str;
    type Prepared = Published<S>;
    fn publication_targets(&self) -> &LayoutPublicationTargets<usize> {
        if self.switched.get() {
            &self.alternate
        } else {
            &self.sink.targets
        }
    }
    fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<usize> {
        if self.switched.get() {
            &mut self.alternate
        } else {
            &mut self.sink.targets
        }
    }
    fn prepare_layout_batch<'call>(
        &self,
        preparation: LayoutBatchPreparation<'call, usize, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        assert!(preparation.previous_members().is_empty());
        let staged = self.sink.prepare_layout_batch(preparation)?;
        self.switched.set(true);
        Ok(staged)
    }
    fn commit_layout_batch<'call>(
        &mut self,
        commit: LayoutBatchCommit<'call, usize, S, Self::Prepared>,
    ) {
        self.sink.commit_layout_batch(commit);
    }
}
fn unstable_target_provider_is_rechecked_after_preparation<S: LayoutScalar>() {
    for change_source in [false, true] {
        let tree = Tree::<S>::responsive();
        let source = LayoutSourceBasis::fresh();
        let candidate = unequal(source.clone());
        let alternate = unequal::<S>(if change_source {
            LayoutSourceBasis::fresh()
        } else {
            source
        });
        let mut adversary = UnstableTargets {
            sink: Sink::new(3),
            alternate: LayoutPublicationTargets::new(),
            switched: Cell::new(false),
        };
        adversary.sink.authorize(0, &candidate);
        adversary
            .alternate
            .install(0, alternate.target_basis())
            .unwrap();
        let before = adversary.sink.state.clone();
        let batch = compute(&tree, 0, &candidate).unwrap();
        let expected = if change_source {
            LayoutBatchApplyError::StaleSource
        } else {
            LayoutBatchApplyError::StaleConstraints
        };
        assert_eq!(batch.apply_to(&mut adversary), Err(expected));
        assert_eq!(adversary.sink.preparations.get(), 1);
        assert_eq!(adversary.sink.commits, 0);
        assert_eq!(adversary.sink.state, before);
        assert!(adversary.switched.get());
        assert_eq!(
            adversary.publication_targets().get(0),
            Some(&alternate.target_basis())
        );
        assert!(
            adversary
                .publication_targets()
                .published_members(0)
                .is_empty()
        );
    }
}
scalars!(
    unstable_target_provider_is_rechecked_after_preparation,
    unstable_target_provider_is_rechecked_after_preparation_f32,
    unstable_target_provider_is_rechecked_after_preparation_f64
);

fn tracked_root_ownership_rejects_overlapping_publication<S: LayoutScalar>() {
    let tree = Tree::<S>::responsive();
    let outer = unequal(LayoutSourceBasis::fresh());
    let mut sink = Sink::new(3);
    sink.authorize(0, &outer);
    compute(&tree, 0, &outer)
        .unwrap()
        .apply_to(&mut sink)
        .unwrap();
    let before = sink.state.clone();
    let preparations = sink.preparations.get();
    let nested = LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
    sink.authorize(1, &nested);
    let batch = compute(&tree, 1, &nested).unwrap();
    assert_eq!(
        batch.apply_to(&mut sink),
        Err(LayoutBatchApplyError::OverlappingRoot)
    );
    assert_eq!(sink.preparations.get(), preparations);
    assert_eq!(sink.state, before);
}
scalars!(
    tracked_root_ownership_rejects_overlapping_publication,
    tracked_root_ownership_rejects_overlapping_publication_f32,
    tracked_root_ownership_rejects_overlapping_publication_f64
);

fn decreasing_block_axis_uses_each_fragment_parent_extent<S: LayoutScalar>() {
    let mut tree = Tree::<S>::new(
        vec![
            block(None, None),
            block(Some(30.0), None),
            block(Some(40.0), None),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    for input in &mut tree.inputs {
        input.writing_mode = WritingMode::VerticalRl;
    }
    let vertical = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let sizing = native_request::sizing(
        Size::new(Some(S::from_f64(50.0)), Some(S::from_f64(100.0))),
        Size::splat(SizingOperation::Normal),
    );
    let root = LayoutRootRequestOf::standalone(
        sizing,
        vertical,
        sizing.orthogonal_fallback().initial_containing_block(),
    );
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        root,
        FragmentationKind::Page,
        vertical,
        vec![frame(50.0, 100.0, true), frame(80.0, 60.0, false)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    assert_fragment(&parents[0], (50.0, 100.0));
    assert_fragment(&parents[1], (40.0, 60.0));
    // Block-start is the physical right edge: first child begins50-30=20;
    // the final40-wide parent aligns to the second80-wide frame at x40.
    assert_eq!(parents[0].unrounded().location, Point::ZERO);
    assert_eq!(
        parents[1].unrounded().location,
        Point::new(S::from_f64(40.0), S::ZERO)
    );
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (20.0, 0.0),
        (30.0, 100.0),
    );
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 0.0),
        (40.0, 60.0),
    );
    for (node, index) in [(1, 0), (2, 1)] {
        assert_eq!(
            single(&batch, node).coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                parents[index].id().clone()
            ))
        );
    }
}
scalars!(
    decreasing_block_axis_uses_each_fragment_parent_extent,
    decreasing_block_axis_uses_each_fragment_parent_extent_f32,
    decreasing_block_axis_uses_each_fragment_parent_extent_f64
);

fn external_origins_drive_fragment_local_endpoint_rounding<S: LayoutScalar>() {
    let tree = Tree::<S>::new(
        vec![
            block(None, None),
            block(None, Some(30.5)),
            block(None, Some(30.5)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    let mut first = frame::<S>(100.0, 50.0, true);
    first.origin = Point::new(S::from_f64(10.25), S::from_f64(20.25));
    let mut second = frame::<S>(60.0, 80.0, false);
    second.origin = Point::new(S::from_f64(200.75), S::from_f64(100.75));
    let request = fragmented(LayoutSourceBasis::fresh(), vec![first, second]);
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    for (index, expected, rounded) in [(0, first, (10.0, 20.0)), (1, second, (201.0, 101.0))] {
        let parent = &parents[index];
        let external = batch
            .fragmentainer(parent.fragmentainer().expect("external frame"))
            .unwrap();
        assert_eq!(
            parent.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragmentainer(
                external.id().clone()
            ))
        );
        assert_eq!(parent.unrounded().location, Point::ZERO);
        assert_eq!(parent.final_geometry().location, Point::ZERO);
        assert_eq!(external.origin(), expected.origin);
        assert_eq!(external.size(), expected.size);
        assert_eq!(external.boundary(), expected.breaks);
        assert_eq!(external.flow_axes(), axes());
        assert_eq!(external.kind(), FragmentationKind::Page);
        assert_eq!(
            external.final_origin(),
            Point::new(S::from_f64(rounded.0), S::from_f64(rounded.1))
        );
    }
    // R(20.25+30.5)-R(20.25)=51-20=31, whereas
    // R(100.75+30.5)-R(100.75)=131-101=30. Local origins stay zero.
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (100.0, 30.5),
    );
    assert_packet(single(&batch, 1).final_output(), (0.0, 0.0), (100.0, 31.0));
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 0.0),
        (60.0, 30.5),
    );
    assert_packet(single(&batch, 2).final_output(), (0.0, 0.0), (60.0, 30.0));
    assert_eq!(parents[1].unrounded().size.height, S::from_f64(30.5));
    assert_eq!(parents[1].final_geometry().size.height, S::from_f64(30.0));
    let repeated = compute(&tree, 0, &request).unwrap();
    assert!(matches!(
        repeated.fragmentainer(parents[0].fragmentainer().expect("external frame")),
        Err(LayoutHandleError::WrongResult)
    ));
    let value = batch.fragment(single(&batch, 2).id()).unwrap();
    assert_eq!(value.id(), single(&batch, 2).id());
    assert!(value.principal_box().is_some());
}
scalars!(
    external_origins_drive_fragment_local_endpoint_rounding,
    external_origins_drive_fragment_local_endpoint_rounding_f32,
    external_origins_drive_fragment_local_endpoint_rounding_f64
);

fn hidden_native_participant_has_explicit_nonbox_state<S: LayoutScalar>() {
    let mut hidden = block::<S>(Some(90.0), Some(90.0));
    hidden.display = Display::None;
    let tree = Tree::<S>::new(vec![block(None, None), hidden], vec![vec![1], vec![]]);
    let request = LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
    let batch = compute(&tree, 0, &request).unwrap();
    let node = batch.node_layout(1).unwrap();
    assert!(node.single_box().is_none());
    assert!(node.fragments().is_none());
    let nonbox = node.non_box().unwrap();
    assert_eq!(nonbox.kind(), NonBoxKind::Hidden);
    assert_eq!(nonbox.unrounded_geometry().size, Size::ZERO);
    assert_eq!(nonbox.final_geometry().size, Size::ZERO);
}
scalars!(
    hidden_native_participant_has_explicit_nonbox_state,
    hidden_native_participant_has_explicit_nonbox_state_f32,
    hidden_native_participant_has_explicit_nonbox_state_f64
);

#[derive(Clone, Copy, Debug)]
enum BlockConstraint {
    Minimum,
    Maximum,
    Ratio,
}

fn constrain_block<S: LayoutScalar>(
    input: &mut NodeInputOf<S>,
    constraint: BlockConstraint,
    minimum: f64,
    maximum: f64,
) {
    match constraint {
        BlockConstraint::Minimum => {
            input.min_size.height =
                MinSizeOf::value(LengthPercentageOf::px(S::from_f64(minimum)).unwrap())
        }
        BlockConstraint::Maximum => {
            input.max_size.height =
                MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(maximum)).unwrap())
        }
        BlockConstraint::Ratio => input.aspect_ratio = AspectRatioOf::new(S::from_f64(2.0)),
    }
}

fn ordinary_ancestor_tree<S: LayoutScalar>(nested: bool, child_height: f64) -> (Tree<S>, usize) {
    if nested {
        (
            Tree::new(
                vec![
                    block(None, None),
                    block(None, None),
                    block(None, Some(child_height)),
                    block(None, Some(child_height)),
                ],
                vec![vec![1], vec![2, 3], vec![], vec![]],
            ),
            1,
        )
    } else {
        (
            Tree::new(
                vec![
                    block(None, None),
                    block(None, Some(child_height)),
                    block(None, Some(child_height)),
                ],
                vec![vec![1, 2], vec![], vec![]],
            ),
            0,
        )
    }
}

fn continuing_absolute_maxima_have_geometry_while_ratios_refuse<S: LayoutScalar>() {
    for nested in [false, true] {
        for constraint in [BlockConstraint::Maximum, BlockConstraint::Ratio] {
            let (mut tree, subject) = ordinary_ancestor_tree::<S>(nested, 30.0);
            // This maximum does not prevent the two 30-unit children
            // continuing across capacity 50. The ratio 2 prefix occupies 50,
            // so it also has an actual continuation, not merely a trial.
            constrain_block(&mut tree.inputs[subject], constraint, 1.0, 1000.0);
            let request = unequal(LayoutSourceBasis::fresh());
            if matches!(constraint, BlockConstraint::Maximum) {
                let batch =
                    compute(&tree, 0, &request).expect("absolute maximum supports continuation");
                for node in [0, subject] {
                    let chain = fragments(&batch, node);
                    assert_eq!(chain.len(), 2);
                    for (index, (width, height)) in
                        [(100.0, 50.0), (60.0, 30.0)].into_iter().enumerate()
                    {
                        assert_fragment(&chain[index], (width, height));
                        assert_eq!(
                            chain[index].fragmentainer(),
                            Some(batch.fragmentainers()[index].id())
                        );
                        assert_eq!(chain[index].unrounded().location, Point::ZERO);
                        assert_eq!(chain[index].final_geometry().location, Point::ZERO);
                    }
                }
                let owners = fragments(&batch, subject);
                for (index, node) in if nested { [2, 3] } else { [1, 2] }.into_iter().enumerate() {
                    minimum_progress_child(
                        &batch,
                        node,
                        index,
                        owners[index].id(),
                        (0.0, 0.0),
                        (if index == 0 { 100.0 } else { 60.0 }, 30.0),
                    );
                }
                continue;
            }
            let error = compute(&tree, 0, &request).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    LayoutErrorKindOf::UnsupportedCapability(
                        LayoutUnsupportedCapability::BlockFragmentationProfile
                    )
                ),
                "nested={nested}, constraint={constraint:?}, error={error:?}"
            );
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(subject));
        }
    }
}
scalars!(
    continuing_absolute_maxima_have_geometry_while_ratios_refuse,
    continuing_absolute_maxima_have_geometry_while_ratios_refuse_f32,
    continuing_absolute_maxima_have_geometry_while_ratios_refuse_f64
);

fn continuing_ordinary_ancestors_refuse_nonvisible_overflow<S: LayoutScalar>() {
    for nested in [false, true] {
        for overflow in [Overflow::Hidden, Overflow::Clip, Overflow::Auto] {
            let (mut tree, subject) = ordinary_ancestor_tree::<S>(nested, 30.0);
            tree.inputs[subject].overflow = ComputedOverflow::try_new(overflow, overflow).unwrap();
            let request = unequal(LayoutSourceBasis::fresh());
            let error = compute(&tree, 0, &request).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    LayoutErrorKindOf::UnsupportedCapability(
                        LayoutUnsupportedCapability::BlockFragmentationProfile
                    )
                ),
                "nested={nested}, overflow={overflow:?}, error={error:?}"
            );
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(subject));
        }
    }
}
scalars!(
    continuing_ordinary_ancestors_refuse_nonvisible_overflow,
    continuing_ordinary_ancestors_refuse_nonvisible_overflow_f32,
    continuing_ordinary_ancestors_refuse_nonvisible_overflow_f64
);

fn assert_whole_box_size<S: LayoutScalar>(batch: &Batch<S>, node: usize, size: (f64, f64)) {
    match batch.node_layout(node).unwrap() {
        NodeLayoutRefOf::SingleBox(value) => {
            assert_packet(value.unrounded_output(), (0.0, 0.0), size);
            assert_packet(value.final_output(), (0.0, 0.0), size);
        }
        NodeLayoutRefOf::Fragments(pieces) => {
            assert_eq!(pieces.len(), 1);
            assert_fragment(&pieces[0], size);
            assert_eq!(pieces[0].unrounded().location, Point::ZERO);
        }
        NodeLayoutRefOf::NonBox(_) => panic!("ordinary whole container generates a box"),
    }
}

fn whole_ordinary_ancestors_keep_existing_constraints<S: LayoutScalar>() {
    for nested in [false, true] {
        for (constraint, height) in [
            (BlockConstraint::Minimum, 45.0),
            (BlockConstraint::Maximum, 35.0),
            (BlockConstraint::Ratio, 50.0),
        ] {
            let (mut tree, subject) = ordinary_ancestor_tree::<S>(nested, 20.0);
            constrain_block(&mut tree.inputs[subject], constraint, 45.0, 35.0);
            let request = unequal(LayoutSourceBasis::fresh());
            let batch = compute(&tree, 0, &request).unwrap();
            // The complete 40-unit child flow fits this one accepted frame.
            // Existing sizing gives max(40,45)=45, min(40,35)=35, or 100/2=50.
            assert_whole_box_size(&batch, subject, (100.0, height));
            if nested {
                assert_whole_box_size(&batch, 0, (100.0, height));
            }
            let first_child = if nested { 2 } else { 1 };
            assert_packet(
                single(&batch, first_child).unrounded_output(),
                (0.0, 0.0),
                (100.0, 20.0),
            );
            assert_packet(
                single(&batch, first_child + 1).unrounded_output(),
                (0.0, 20.0),
                (100.0, 20.0),
            );
        }
        for overflow in [Overflow::Hidden, Overflow::Clip, Overflow::Auto] {
            let (mut tree, subject) = ordinary_ancestor_tree::<S>(nested, 20.0);
            tree.inputs[subject].overflow = ComputedOverflow::try_new(overflow, overflow).unwrap();
            let request = unequal(LayoutSourceBasis::fresh());
            let batch = compute(&tree, 0, &request).unwrap();
            assert_whole_box_size(&batch, subject, (100.0, 40.0));
            if nested {
                assert_whole_box_size(&batch, 0, (100.0, 40.0));
            }
        }
    }
}
scalars!(
    whole_ordinary_ancestors_keep_existing_constraints,
    whole_ordinary_ancestors_keep_existing_constraints_f32,
    whole_ordinary_ancestors_keep_existing_constraints_f64
);

fn constrained_atomic_boxes_keep_real_sizing_and_overflow<S: LayoutScalar>() {
    for measured in [false, true] {
        for constraint in [
            BlockConstraint::Minimum,
            BlockConstraint::Maximum,
            BlockConstraint::Ratio,
        ] {
            for overflow in [Overflow::Visible, Overflow::Hidden] {
                let intrinsic_height = if matches!(constraint, BlockConstraint::Ratio) {
                    10.0
                } else {
                    30.0
                };
                let authored_height = match constraint {
                    BlockConstraint::Minimum => Some(10.0),
                    BlockConstraint::Maximum => Some(30.0),
                    BlockConstraint::Ratio => None,
                };
                let mut atomic =
                    block::<S>(Some(20.0), if measured { None } else { authored_height });
                constrain_block(&mut atomic, constraint, 40.0, 20.0);
                if matches!(constraint, BlockConstraint::Ratio) {
                    atomic.aspect_ratio = AspectRatioOf::new(S::ONE);
                }
                atomic.overflow = ComputedOverflow::try_new(overflow, overflow).unwrap();
                let mut tree = Tree::<S>::new(
                    vec![block(None, None), block(None, Some(40.0)), atomic],
                    vec![vec![1, 2], vec![], vec![]],
                );
                if measured {
                    tree.measurements[2] = Some(Measurement::Fixed(20.0, intrinsic_height));
                }
                let request = unequal(LayoutSourceBasis::fresh());
                let batch = compute(&tree, 0, &request).unwrap();
                let height = if matches!(constraint, BlockConstraint::Minimum) {
                    40.0
                } else {
                    20.0
                };
                let parents = fragments(&batch, 0);
                assert_eq!(parents.len(), 2);
                assert_fragment(&parents[0], (100.0, 50.0));
                assert_fragment(&parents[1], (60.0, height));
                let child = single(&batch, 2);
                assert_packet(child.unrounded_output(), (0.0, 0.0), (20.0, height));
                assert_eq!(
                    child.coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(parents[1].id().clone()))
                );
                if measured {
                    let packet = child.unrounded_output();
                    let geometry = packet.scroll_geometry.unwrap();
                    assert_eq!(geometry.used_overflow_x(), overflow);
                    assert_eq!(geometry.used_overflow_y(), overflow);
                    // Canonical content includes the used padding-box seed as
                    // well as actual host content; max-height 20 retains content 30.
                    assert_eq!(
                        packet.content_size,
                        Size::new(S::from_f64(20.0), S::from_f64(height.max(intrinsic_height)))
                    );
                }
            }
        }
    }
}
scalars!(
    constrained_atomic_boxes_keep_real_sizing_and_overflow,
    constrained_atomic_boxes_keep_real_sizing_and_overflow_f32,
    constrained_atomic_boxes_keep_real_sizing_and_overflow_f64
);

fn vertical_bound_request<S: LayoutScalar>(source: LayoutSourceBasis) -> LayoutBoundRequestOf<S> {
    let vertical = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let sizing = native_request::sizing(
        Size::new(Some(S::from_f64(50.0)), Some(S::from_f64(100.0))),
        Size::splat(SizingOperation::Normal),
    );
    let root = LayoutRootRequestOf::standalone(
        sizing,
        vertical,
        sizing.orthogonal_fallback().initial_containing_block(),
    );
    LayoutBoundRequestOf::block_fragmentation(
        source,
        root,
        FragmentationKind::Page,
        vertical,
        vec![frame(50.0, 100.0, true), frame(80.0, 60.0, false)],
    )
    .unwrap()
}

fn opposite_block_progression_refuses_only_actual_continuation<S: LayoutScalar>() {
    let mut tree = Tree::<S>::new(
        vec![
            block(None, None),
            block(None, None),
            block(Some(30.0), None),
            block(Some(30.0), None),
        ],
        vec![vec![1], vec![2, 3], vec![], vec![]],
    );
    tree.inputs[0].writing_mode = WritingMode::VerticalRl;
    for input in &mut tree.inputs[1..] {
        input.writing_mode = WritingMode::VerticalLr;
    }
    // The nested container's two 30-unit children cross the 50-unit block
    // capacity. Its physical block axis matches the context, but its block
    // progression starts from Left rather than Right.
    let request = vertical_bound_request(LayoutSourceBasis::fresh());
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile
        )
    ));
}
scalars!(
    opposite_block_progression_refuses_only_actual_continuation,
    opposite_block_progression_refuses_only_actual_continuation_f32,
    opposite_block_progression_refuses_only_actual_continuation_f64
);

fn whole_opposite_and_atomic_orthogonal_boxes_keep_real_production<S: LayoutScalar>() {
    let mut tree = Tree::<S>::new(
        vec![
            block(None, None),
            block(None, None),
            block(Some(20.0), None),
            block(Some(20.0), None),
        ],
        vec![vec![1], vec![2, 3], vec![], vec![]],
    );
    tree.inputs[0].writing_mode = WritingMode::VerticalRl;
    for input in &mut tree.inputs[1..] {
        input.writing_mode = WritingMode::VerticalLr;
    }
    let request = vertical_bound_request(LayoutSourceBasis::fresh());
    let batch = compute(&tree, 0, &request).unwrap();
    let parent = single(&batch, 0);
    assert_packet(parent.unrounded_output(), (10.0, 0.0), (40.0, 100.0));
    assert_packet(parent.final_output(), (10.0, 0.0), (40.0, 100.0));
    let opposite = single(&batch, 1);
    assert_packet(opposite.unrounded_output(), (0.0, 0.0), (40.0, 100.0));
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 0.0),
        (20.0, 100.0),
    );
    assert_packet(
        single(&batch, 3).unrounded_output(),
        (20.0, 0.0),
        (20.0, 100.0),
    );
    assert_eq!(
        opposite.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(parent.id().clone()))
    );

    for orthogonal in [false, true] {
        let mut root = block::<S>(None, None);
        root.writing_mode = WritingMode::VerticalRl;
        let mut before = block::<S>(Some(40.0), None);
        before.writing_mode = WritingMode::VerticalRl;
        let mut atomic = block::<S>(Some(30.0), Some(20.0));
        atomic.writing_mode = if orthogonal {
            WritingMode::HorizontalTb
        } else {
            WritingMode::VerticalLr
        };
        let mut tree = Tree::new(
            vec![root, before, atomic, block(Some(4.0), Some(6.0))],
            vec![
                vec![1, 2],
                vec![],
                if orthogonal { vec![3] } else { vec![] },
                vec![],
            ],
        );
        // Empty opposite-flow boxes are atomic. The nonempty Htb box has an
        // orthogonal physical block axis, so its real child remains inside it.
        if !orthogonal {
            tree.inputs[3].display = Display::None;
        }
        let request = vertical_bound_request(LayoutSourceBasis::fresh());
        let batch = compute(&tree, 0, &request).unwrap();
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        assert_fragment(&parents[0], (50.0, 100.0));
        assert_fragment(&parents[1], (30.0, 60.0));
        assert_eq!(
            parents[1].unrounded().location,
            Point::new(S::from_f64(50.0), S::ZERO)
        );
        let child = single(&batch, 2);
        assert_packet(child.unrounded_output(), (0.0, 0.0), (30.0, 20.0));
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[1].id().clone()))
        );
        if orthogonal {
            let inside = single(&batch, 3);
            assert_packet(inside.unrounded_output(), (0.0, 0.0), (4.0, 6.0));
            assert_eq!(
                inside.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(child.id().clone()))
            );
        }
    }
}
scalars!(
    whole_opposite_and_atomic_orthogonal_boxes_keep_real_production,
    whole_opposite_and_atomic_orthogonal_boxes_keep_real_production_f32,
    whole_opposite_and_atomic_orthogonal_boxes_keep_real_production_f64
);

fn explicit_block_percentage_parent<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let request = root_request::<S>(100.0);
    LayoutRootRequestOf::standalone(
        request.sizing_request(),
        axes(),
        Size::new(
            Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
            Some(NonNegativeFiniteOf::new(S::from_f64(200.0)).unwrap()),
        ),
    )
}

fn fragment_fill_does_not_create_descendant_percentage_authority<S: LayoutScalar>() {
    let mut sink = Sink::new(4);
    for fixed_control in [true, false] {
        let mut percentage_leaf = block::<S>(None, None);
        percentage_leaf.size.height = if fixed_control {
            px(25.0)
        } else {
            PreferredSizeOf::value(
                LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap(),
            )
        };
        let mut tree = Tree::new(
            vec![
                block(None, None),
                block(None, Some(30.0)),
                percentage_leaf,
                block(None, Some(30.0)),
            ],
            vec![vec![1, 2, 3], vec![], vec![], vec![]],
        );
        tree.measurements[2] = Some(Measurement::Fixed(20.0, 10.0));
        let request = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            explicit_block_percentage_parent(),
            FragmentationKind::Page,
            axes(),
            vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
        )
        .unwrap();
        sink.authorize(0, &request);
        if !fixed_control {
            let before = sink.state.clone();
            let commits = sink.commits;
            let error = compute(&tree, 0, &request).unwrap_err();
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
            ));
            // The ordinary measured-leaf producer likewise requires an
            // explicit basis during performed layout. Fragment fill must not
            // turn this refusal into a guessed used height of 25 or 40.
            let continuous = LayoutBoundRequestOf::continuous(
                LayoutSourceBasis::fresh(),
                explicit_block_percentage_parent(),
            );
            sink.authorize(0, &continuous);
            let error = compute(&tree, 0, &continuous).unwrap_err();
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
            ));
            assert_eq!(sink.state, before);
            assert_eq!(sink.commits, commits);
            continue;
        }
        let batch = compute(&tree, 0, &request).unwrap();
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        assert_fragment(&parents[0], (100.0, 50.0));
        assert_fragment(&parents[1], (60.0, 55.0));
        let owner = single(&batch, 2);
        assert_packet(owner.unrounded_output(), (0.0, 0.0), (60.0, 25.0));
        assert_eq!(
            owner.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[1].id().clone()))
        );
        assert_packet(
            single(&batch, 3).unrounded_output(),
            (0.0, 25.0),
            (60.0, 30.0),
        );
        batch.apply_to(&mut sink).unwrap();
        // Authored height 25 is definite and defers this real measured box.
        // The root's Auto block size supplies no descendant percentage basis,
        // including after its first fragment fills the 50-unit frame.
    }

    // A distinct, whole-fitting root DOES own the explicit standalone basis:
    // min-height 50% of parent 200 is 100, regardless of frame capacity 150.
    let mut root = block::<S>(None, None);
    root.min_size.height =
        MinSizeOf::value(LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap());
    let tree = Tree::new(vec![root, block(None, Some(10.0))], vec![vec![1], vec![]]);
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        explicit_block_percentage_parent(),
        FragmentationKind::Page,
        axes(),
        vec![frame(100.0, 150.0, true)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &request).unwrap();
    assert_whole_box_size(&batch, 0, (100.0, 100.0));
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (100.0, 10.0),
    );
}
scalars!(
    fragment_fill_does_not_create_descendant_percentage_authority,
    fragment_fill_does_not_create_descendant_percentage_authority_f32,
    fragment_fill_does_not_create_descendant_percentage_authority_f64
);

fn margin_boundary_case<S: LayoutScalar>(separator: u8) {
    margin_boundary_cap_case::<S>(separator, false);
}
fn margin_boundary_cap_case<S: LayoutScalar>(separator: u8, capped: bool) {
    // Both actual break kinds are computed before checking either result.
    let batches = [false, true].map(|forced| {
        let mut wrapper = block::<S>(None, None);
        if capped {
            wrapper.max_size.height =
                MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(1000.0)).unwrap());
        }

        let edge = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
        match separator {
            1 => wrapper.padding.bottom = edge,
            2 => wrapper.border.bottom = edge,
            3 => wrapper.block_flow_context = BlockFlowContext::Independent,
            _ => {}
        }
        let mut child = block::<S>(None, Some(10.0));
        child.margin.bottom =
            LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
        let mut next = block::<S>(None, Some(50.0));
        if forced {
            next.break_before = FragmentationBreak::Page;
        }
        let tree = Tree::new(
            vec![block(None, None), wrapper, child, next],
            vec![vec![1, 3], vec![2], vec![], vec![]],
        );
        let request = fragmented(
            LayoutSourceBasis::fresh(),
            vec![frame(100.0, 50.0, true), frame(100.0, 100.0, false)],
        );
        compute(&tree, 0, &request).unwrap()
    });
    for batch in batches {
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        assert_fragment(&parents[0], (100.0, 50.0));
        assert_fragment(&parents[1], (100.0, 50.0));
        let wrapper = single(&batch, 1);
        let child = single(&batch, 2);
        let (height, margin) = match separator {
            0 => (10.0, 0.0),
            3 => (30.0, 20.0),
            _ => (40.0, 20.0),
        };
        // Padding/border separates the child's margin from the outer break:
        // wrapper height=child10 + internal margin20 + separator10 =40.
        // Without that separator the margin collapses through and adjoins
        // the break, so §5.2 truncates it and the wrapper remains10 high.
        // An independent formatting context also contains that margin, giving
        // child10 + internal margin20 =30 even with no padding or border.
        for packet in [wrapper.unrounded_output(), wrapper.final_output()] {
            assert_eq!(packet.location, Point::ZERO);
            assert_eq!(
                packet.size,
                Size::new(S::from_f64(100.0), S::from_f64(height))
            );
            let mut padding = Edges::all(S::ZERO);
            if separator == 1 {
                padding.bottom = S::from_f64(10.0);
            }
            let mut border = Edges::all(S::ZERO);
            if separator == 2 {
                border.bottom = S::from_f64(10.0);
            }
            assert_eq!(packet.padding, padding);
            assert_eq!(packet.border, border);
            assert_eq!(packet.margin, Edges::all(S::ZERO));
        }
        for packet in [child.unrounded_output(), child.final_output()] {
            assert_eq!(packet.location, Point::ZERO);
            assert_eq!(
                packet.size,
                Size::new(S::from_f64(100.0), S::from_f64(10.0))
            );
            let mut expected = Edges::all(S::ZERO);
            expected.bottom = S::from_f64(margin);
            assert_eq!(packet.margin, expected);
            assert_eq!(packet.padding, Edges::all(S::ZERO));
            assert_eq!(packet.border, Edges::all(S::ZERO));
        }
        assert_eq!(
            wrapper.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[0].id().clone()))
        );
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(wrapper.id().clone()))
        );
        let next = single(&batch, 3);
        assert_packet(next.unrounded_output(), (0.0, 0.0), (100.0, 50.0));
        assert_eq!(
            next.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[1].id().clone()))
        );
    }
}
fn padding_separated_descendant_margin_is_internal<S: LayoutScalar>() {
    margin_boundary_case::<S>(1);
}
scalars!(
    padding_separated_descendant_margin_is_internal,
    padding_separated_descendant_margin_is_internal_f32,
    padding_separated_descendant_margin_is_internal_f64
);
fn border_separated_descendant_margin_is_internal<S: LayoutScalar>() {
    margin_boundary_case::<S>(2);
}
scalars!(
    border_separated_descendant_margin_is_internal,
    border_separated_descendant_margin_is_internal_f32,
    border_separated_descendant_margin_is_internal_f64
);
fn collapsed_descendant_margin_adjoins_the_break<S: LayoutScalar>() {
    margin_boundary_case::<S>(0);
}
scalars!(
    collapsed_descendant_margin_adjoins_the_break,
    collapsed_descendant_margin_adjoins_the_break_f32,
    collapsed_descendant_margin_adjoins_the_break_f64
);
fn independent_block_context_keeps_descendant_margin_internal<S: LayoutScalar>() {
    margin_boundary_case::<S>(3);
}
scalars!(
    independent_block_context_keeps_descendant_margin_internal,
    independent_block_context_keeps_descendant_margin_internal_f32,
    independent_block_context_keeps_descendant_margin_internal_f64
);

fn continuing_intrinsic_inline_container_retains_whole_demand<S: LayoutScalar>() {
    let mut root = block::<S>(None, None);
    root.size.width = PreferredSizeOf::MAX_CONTENT;
    let tree = Tree::new(
        vec![
            root,
            block(Some(10.0), Some(30.0)),
            block(Some(80.0), Some(30.0)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(100.0, 100.0, false)],
    );
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    // The source prefix is10 wide, but the whole-element max-content size is80.
    assert_fragment(&parents[0], (80.0, 50.0));
    assert_fragment(&parents[1], (80.0, 30.0));
    assert_eq!(parents[0].source_children(), [(SourceIndex::new(0), 1)]);
    assert_eq!(parents[1].source_children(), [(SourceIndex::new(1), 2)]);
}
scalars!(
    continuing_intrinsic_inline_container_retains_whole_demand,
    continuing_intrinsic_inline_container_retains_whole_demand_f32,
    continuing_intrinsic_inline_container_retains_whole_demand_f64
);

fn whole_intrinsic_and_atomic_intrinsic_sizing_stay_supported<S: LayoutScalar>() {
    let mut root = block::<S>(None, None);
    root.size.width = PreferredSizeOf::MAX_CONTENT;
    let tree = Tree::new(
        vec![
            root,
            block(Some(10.0), Some(20.0)),
            block(Some(80.0), Some(20.0)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(100.0, 100.0, false)],
    );
    let batch = compute(&tree, 0, &request).unwrap();
    assert_whole_box_size(&batch, 0, (80.0, 40.0));
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (10.0, 20.0),
    );
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 20.0),
        (80.0, 20.0),
    );

    let tree = Tree::<S>::new(
        vec![
            block(Some(80.0), None),
            block(Some(10.0), Some(30.0)),
            block(Some(80.0), Some(30.0)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(100.0, 100.0, false)],
    );
    let batch = compute(&tree, 0, &request).unwrap();
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    assert_fragment(&parents[0], (80.0, 50.0));
    assert_fragment(&parents[1], (80.0, 30.0));
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (10.0, 30.0),
    );
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 0.0),
        (80.0, 30.0),
    );

    let mut atomic = block::<S>(None, None);
    atomic.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut tree = Tree::new(
        vec![block(None, None), block(None, Some(30.0)), atomic],
        vec![vec![1, 2], vec![], vec![]],
    );
    tree.measurements[2] = Some(Measurement::Fixed(80.0, 30.0));
    let request = unequal(LayoutSourceBasis::fresh());
    let batch = compute(&tree, 0, &request).unwrap();
    assert_packet(
        single(&batch, 2).unrounded_output(),
        (0.0, 0.0),
        (80.0, 30.0),
    );
    assert_fragment(&fragments(&batch, 0)[1], (60.0, 30.0));
}
scalars!(
    whole_intrinsic_and_atomic_intrinsic_sizing_stay_supported,
    whole_intrinsic_and_atomic_intrinsic_sizing_stay_supported_f32,
    whole_intrinsic_and_atomic_intrinsic_sizing_stay_supported_f64
);

fn minimum_progress_nested_terminal_keeps_parent_ancestry<S: LayoutScalar>() {
    for nested in [false, true] {
        let (mut tree, subject) = if nested {
            (
                Tree::<S>::new(
                    vec![
                        block(None, None),
                        block(None, None),
                        block(None, Some(10.0)),
                    ],
                    vec![vec![1], vec![2], vec![]],
                ),
                1,
            )
        } else {
            (
                Tree::<S>::new(
                    vec![block(None, None), block(None, Some(10.0))],
                    vec![vec![1], vec![]],
                ),
                0,
            )
        };
        tree.inputs[subject].min_size.height =
            MinSizeOf::value(LengthPercentageOf::px(S::from_f64(150.0)).unwrap());
        let request = fragmented(
            LayoutSourceBasis::fresh(),
            vec![frame(100.0, 100.0, true), frame(100.0, 100.0, false)],
        );
        let batch = minimum_progress_batch(&tree, &request);
        let root_children: &[&[(usize, usize)]] = if nested {
            &[&[(0, 1)], &[(0, 1)]]
        } else {
            &[&[(0, 1)], &[]]
        };
        let roots = minimum_progress_chain(&batch, 0, root_children);
        assert_fragment(&roots[0], (100.0, 100.0));
        assert_fragment(&roots[1], (100.0, 50.0));
        let subject_fragments = if nested {
            let values = minimum_progress_chain(&batch, subject, &[&[(0, 2)], &[]]);
            for index in [0, 1] {
                assert_eq!(
                    values[index].coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        roots[index].id().clone()
                    ))
                );
                assert_eq!(values[index].unrounded().location, Point::ZERO);
                assert_eq!(values[index].final_geometry().location, Point::ZERO);
            }
            values
        } else {
            roots
        };
        assert_fragment(&subject_fragments[0], (100.0, 100.0));
        assert_fragment(&subject_fragments[1], (100.0, 50.0));
        minimum_progress_child(
            &batch,
            if nested { 2 } else { 1 },
            0,
            subject_fragments[0].id(),
            (0.0, 0.0),
            (100.0, 10.0),
        );
    }
}
scalars!(
    minimum_progress_nested_terminal_keeps_parent_ancestry,
    minimum_progress_nested_terminal_keeps_parent_ancestry_f32,
    minimum_progress_nested_terminal_keeps_parent_ancestry_f64
);

fn actual_atomic_minimum_size_may_overflow_whole_once<S: LayoutScalar>() {
    let mut atomic = block::<S>(Some(20.0), None);
    atomic.min_size.height = MinSizeOf::value(LengthPercentageOf::px(S::from_f64(150.0)).unwrap());
    let mut tree = Tree::new(vec![block(None, None), atomic], vec![vec![1], vec![]]);
    tree.measurements[1] = Some(Measurement::Fixed(20.0, 10.0));
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(100.0, 100.0, false)],
    );
    let batch = compute(&tree, 0, &request).unwrap();
    assert_whole_box_size(&batch, 0, (100.0, 150.0));
    let parent = single(&batch, 0);
    let child = single(&batch, 1);
    assert_packet(child.unrounded_output(), (0.0, 0.0), (20.0, 150.0));
    assert_packet(child.final_output(), (0.0, 0.0), (20.0, 150.0));
    assert_eq!(
        child.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(parent.id().clone()))
    );
}
scalars!(
    actual_atomic_minimum_size_may_overflow_whole_once,
    actual_atomic_minimum_size_may_overflow_whole_once_f32,
    actual_atomic_minimum_size_may_overflow_whole_once_f64
);

fn normal_content_width<S: LayoutScalar>(input: LeafMeasureInputOf<S>, width: f64) -> bool {
    let axis = input.content_sizing_request().axes().width;
    axis.operation() == SizingOperation::Normal
        && axis
            .space()
            .definite_size()
            .is_some_and(|value| value.get() == S::from_f64(width))
}

fn forced_boundary_skips_later_normal_measurement_in_prior_frame<S: LayoutScalar>() {
    let mut tree = Tree::<S>::responsive();
    tree.inputs[2].break_before = FragmentationBreak::Page;
    tree.measurements[2] = Some(Measurement::ResponsiveRejectNormalWidth(100.0));
    let request = unequal(LayoutSourceBasis::fresh());
    let batch = match compute(&tree, 0, &request) {
        Ok(batch) => batch,
        Err(error) => {
            panic!("forced boundary must precede the refused Normal width100 query: {error:?}")
        }
    };
    let parents = fragments(&batch, 0);
    assert_eq!(parents.len(), 2);
    assert_fragment(&parents[0], (100.0, 50.0));
    assert_fragment(&parents[1], (60.0, 40.0));
    for (node, index, size) in [(1, 0, (100.0, 30.0)), (2, 1, (60.0, 40.0))] {
        let child = single(&batch, node);
        assert_packet(child.unrounded_output(), (0.0, 0.0), size);
        assert_packet(child.final_output(), (0.0, 0.0), size);
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                parents[index].id().clone()
            ))
        );
    }
    assert_eq!(parents[0].source_children(), [(SourceIndex::new(0), 1)]);
    assert_eq!(parents[1].source_children(), [(SourceIndex::new(1), 2)]);
    let observed = tree.observed.borrow();
    // Intrinsic queries are permitted at either width. Only actual Normal
    // width100 is refused; B's forced page starts in the60-wide frame.
    assert!(
        !observed
            .iter()
            .any(|(node, input)| *node == 2 && normal_content_width(*input, 100.0))
    );
    assert!(
        observed
            .iter()
            .any(|(node, input)| *node == 2 && normal_content_width(*input, 60.0))
    );
}
scalars!(
    forced_boundary_skips_later_normal_measurement_in_prior_frame,
    forced_boundary_skips_later_normal_measurement_in_prior_frame_f32,
    forced_boundary_skips_later_normal_measurement_in_prior_frame_f64
);

fn applicable_normal_measurement_refusals_remain_errors<S: LayoutScalar>() {
    for (forced, rejected_width) in [(false, 100.0), (true, 60.0)] {
        let mut tree = Tree::<S>::responsive();
        if forced {
            tree.inputs[2].break_before = FragmentationBreak::Page;
        }
        tree.measurements[2] = Some(Measurement::ResponsiveRejectNormalWidth(rejected_width));
        let request = unequal(LayoutSourceBasis::fresh());
        let error = match compute(&tree, 0, &request) {
            Ok(_) => panic!(
                "applicable Normal width{rejected_width} refusal must fail the complete result"
            ),
            Err(error) => error,
        };
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
        assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement("normal content width rejected")
        ));
        assert!(
            tree.observed
                .borrow()
                .iter()
                .any(|(node, input)| *node == 2 && normal_content_width(*input, rejected_width))
        );
    }
}
scalars!(
    applicable_normal_measurement_refusals_remain_errors,
    applicable_normal_measurement_refusals_remain_errors_f32,
    applicable_normal_measurement_refusals_remain_errors_f64
);

fn actual_break_provenance_controls_adjoining_start_margin<S: LayoutScalar>() {
    let batches = [false, true].map(|forced| {
        let mut before = block::<S>(None, Some(30.0));
        before.margin.bottom =
            LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
        let mut after = block::<S>(None, Some(30.0));
        after.margin.top = LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
        if forced {
            after.break_before = FragmentationBreak::Page;
        }
        let tree = Tree::new(
            vec![block(None, None), before, after],
            vec![vec![1, 2], vec![], vec![]],
        );
        let request = unequal(LayoutSourceBasis::fresh());
        compute(&tree, 0, &request).unwrap()
    });
    for (forced, batch) in [false, true].into_iter().zip(batches) {
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        assert_fragment(&parents[0], (100.0, 50.0));
        assert_fragment(&parents[1], (60.0, if forced { 40.0 } else { 30.0 }));
        let before = single(&batch, 1);
        assert_packet(before.unrounded_output(), (0.0, 0.0), (100.0, 30.0));
        assert_packet(before.final_output(), (0.0, 0.0), (100.0, 30.0));
        assert_eq!(
            before.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[0].id().clone()))
        );
        let after = single(&batch, 2);
        let start = if forced { 10.0 } else { 0.0 };
        // §5.2 truncates the adjoining end margin for either break. Only an
        // actual forced entry preserves the following start margin. The root
        // is the real Block formatting root, so that10 remains inside it.
        for packet in [after.unrounded_output(), after.final_output()] {
            assert_eq!(packet.location, Point::new(S::ZERO, S::from_f64(start)));
            assert_eq!(packet.size, Size::new(S::from_f64(60.0), S::from_f64(30.0)));
            let mut margin = Edges::all(S::ZERO);
            margin.top = S::from_f64(start);
            assert_eq!(packet.margin, margin);
            assert_eq!(packet.padding, Edges::all(S::ZERO));
            assert_eq!(packet.border, Edges::all(S::ZERO));
        }
        assert_eq!(
            after.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parents[1].id().clone()))
        );
    }
}
scalars!(
    actual_break_provenance_controls_adjoining_start_margin,
    actual_break_provenance_controls_adjoining_start_margin_f32,
    actual_break_provenance_controls_adjoining_start_margin_f64
);

fn bound_source_invalidation_discards_changed_measurement_only<S: LayoutScalar>() {
    let mut retained_changed = Vec::new();
    for fragmented_mode in [true, false] {
        let mut tree = Tree::<S>::responsive();
        tree.measurements[1] = Some(Measurement::Fixed(20.0, 30.0));
        tree.measurements[2] = Some(Measurement::Fixed(80.0, 30.0));
        tree.inputs[2].size.width = PreferredSizeOf::MIN_CONTENT;
        let mut sink = Sink::new(3);
        compute_layout(&tree, 0, root_request(100.0))
            .unwrap()
            .apply_to(&mut sink)
            .unwrap();

        let measurement_input = ComputeInputOf::leaf_content_size(
            Size::NONE,
            Size::new(Some(S::from_f64(100.0)), None),
            ContainingLayoutContext::new(axes(), ParentFormattingContext::BlockFlow),
            native_request::sizing(
                Size::new(Some(S::from_f64(100.0)), None),
                Size::splat(SizingOperation::Normal),
            ),
        )
        .unwrap();
        let context = CacheKeyContext::new();
        let unchanged = compute_leaf(measurement_input, &tree.inputs[1], |input| {
            tree.measure_leaf(1, input).unwrap()
        })
        .unwrap();
        let old = compute_leaf(measurement_input, &tree.inputs[2], |input| {
            tree.measure_leaf(2, input).unwrap()
        })
        .unwrap();
        assert_eq!(
            unchanged.size,
            Size::new(S::from_f64(20.0), S::from_f64(30.0))
        );
        assert_eq!(old.size, Size::new(S::from_f64(80.0), S::from_f64(30.0)));
        assert!(unchanged.scroll_geometry.is_none());
        assert!(old.scroll_geometry.is_none());
        sink.state.caches[1].store_with_context(&measurement_input, context, unchanged);
        sink.state.caches[2].store_with_context(&measurement_input, context, old);
        sink.copy_cache_to(&mut tree);

        let source = LayoutSourceBasis::fresh();
        let initial = if fragmented_mode {
            unequal(source)
        } else {
            LayoutBoundRequestOf::continuous(source, root_request(100.0))
        };
        sink.authorize(0, &initial);
        let initial_batch = compute(&tree, 0, &initial).unwrap();
        assert!(initial_batch.invalidated_nodes().is_empty());
        initial_batch.apply_to(&mut sink).unwrap();
        // Entering bound mode with unchanged source preserves both independent
        // measurement keys, even though performed geometry is wholly replaced.
        assert_eq!(
            sink.state.caches[1].get_with_context(&measurement_input, context),
            Some(unchanged)
        );
        assert_eq!(
            sink.state.caches[2].get_with_context(&measurement_input, context),
            Some(old)
        );

        // One authored input changes. ContentSize keeps actual box-edge
        // lowering: content 80 + new right padding 40 gives a real 120-wide result
        // under the identical public input key. A's source stays unchanged.
        tree.inputs[2].padding.right =
            LengthOf::value(LengthPercentageOf::px(S::from_f64(40.0)).unwrap());
        let fresh = compute_leaf(measurement_input, &tree.inputs[2], |input| {
            tree.measure_leaf(2, input).unwrap()
        })
        .unwrap();
        assert_eq!(fresh.size, Size::new(S::from_f64(120.0), S::from_f64(30.0)));
        assert!(fresh.scroll_geometry.is_none());
        sink.copy_cache_to(&mut tree);
        tree.reset_observations();
        let source = LayoutSourceBasis::fresh();
        let changed_request = if fragmented_mode {
            unequal(source)
        } else {
            LayoutBoundRequestOf::continuous(source, root_request(100.0))
        };
        sink.authorize(0, &changed_request);
        let capture = LayoutFlowInput::try_new_bound(
            &tree,
            0,
            changed_request.clone(),
            Vec::new(),
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let changed =
            compute_layout_with_flow_invalidated(&capture, 0, changed_request.root_request(), &[2])
                .unwrap();
        assert!(changed.replaces_whole_root());
        assert!(changed.invalidated_nodes().contains(&0));
        assert!(changed.invalidated_nodes().contains(&2));
        assert!(!changed.invalidated_nodes().contains(&1));
        assert!(
            changed
                .cache_clear_entries()
                .iter()
                .any(|entry| entry.node() == 2)
        );
        assert!(
            !changed
                .cache_clear_entries()
                .iter()
                .any(|entry| entry.node() == 1)
        );
        assert!(changed.cache_store_entries().is_empty());
        assert_eq!(tree.cache_calls.get(), 0);
        assert_eq!(tree.custody_calls.get(), 0);
        if fragmented_mode {
            let parents = fragments(&changed, 0);
            assert_eq!(parents.len(), 2);
            assert_fragment(&parents[0], (100.0, 50.0));
            assert_fragment(&parents[1], (60.0, 30.0));
        } else {
            assert_packet(
                single(&changed, 0).unrounded_output(),
                (0.0, 0.0),
                (100.0, 60.0),
            );
        }
        let child = single(&changed, 2);
        let y = if fragmented_mode { 0.0 } else { 30.0 };
        for packet in [child.unrounded_output(), child.final_output()] {
            assert_eq!(packet.location, Point::new(S::ZERO, S::from_f64(y)));
            assert_eq!(
                packet.size,
                Size::new(S::from_f64(120.0), S::from_f64(30.0))
            );
            let mut padding = Edges::all(S::ZERO);
            padding.right = S::from_f64(40.0);
            assert_eq!(packet.padding, padding);
            assert_eq!(packet.border, Edges::all(S::ZERO));
            assert_eq!(packet.margin, Edges::all(S::ZERO));
        }
        changed.apply_to(&mut sink).unwrap();
        assert_eq!(
            sink.state.caches[1].get_with_context(&measurement_input, context),
            Some(unchanged)
        );
        retained_changed.push((
            fragmented_mode,
            sink.state.caches[2]
                .get_with_context(&measurement_input, context)
                .map(|output| output.size),
        ));
    }
    // Both actual modes execute before this assertion, exposing source-clear
    // effects separately from unchanged-member performed-only replacement.
    assert!(
        retained_changed.iter().all(|(_, output)| output.is_none()),
        "explicit source invalidation retained old measurement sizes: {retained_changed:?}"
    );
}
scalars!(
    bound_source_invalidation_discards_changed_measurement_only,
    bound_source_invalidation_discards_changed_measurement_only_f32,
    bound_source_invalidation_discards_changed_measurement_only_f64
);

// Break3 varying-size fragmentainers retain each element's whole-source
// intrinsic size; a narrower frame does not replace it with a prefix demand.
fn whole_max_content_width_survives_narrower_continuation<S: LayoutScalar>() {
    for intrinsic in [false, true] {
        let mut root = block::<S>(None, None);
        if intrinsic {
            root.size.width = PreferredSizeOf::MAX_CONTENT;
        }
        let tree = Tree::new(
            vec![
                root,
                block(Some(10.0), Some(30.0)),
                block(Some(80.0), Some(30.0)),
            ],
            vec![vec![1, 2], vec![], vec![]],
        );
        let request = unequal(LayoutSourceBasis::fresh());
        let batch = compute(&tree, 0, &request)
            .expect("whole-element intrinsic width is retained across fragmentainers");
        let parents = fragments(&batch, 0);
        assert_eq!(parents.len(), 2);
        let widths = if intrinsic {
            [80.0, 80.0]
        } else {
            [100.0, 60.0]
        };
        assert_fragment(&parents[0], (widths[0], 50.0));
        assert_fragment(&parents[1], (widths[1], 30.0));
        assert_eq!(parents[0].previous(), None);
        assert_eq!(parents[0].next(), Some(parents[1].id()));
        assert_eq!(parents[1].previous(), Some(parents[0].id()));
        assert_eq!(parents[1].next(), None);
        assert_eq!(parents[0].continuation_order(), 0);
        assert_eq!(parents[1].continuation_order(), 1);
        assert_ne!(parents[0].fragmentainer(), parents[1].fragmentainer());
        for parent in parents {
            assert_eq!(parent.unrounded().location, Point::ZERO);
            assert_eq!(parent.final_geometry().location, Point::ZERO);
        }
        assert_eq!(parents[0].source_children(), [(SourceIndex::new(0), 1)]);
        assert_eq!(parents[1].source_children(), [(SourceIndex::new(1), 2)]);
        for (node, frame, size) in [(1, 0, (10.0, 30.0)), (2, 1, (80.0, 30.0))] {
            let child = single(&batch, node);
            assert_eq!(
                child.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    parents[frame].id().clone()
                ))
            );
            assert_packet(child.unrounded_output(), (0.0, 0.0), size);
            assert_packet(child.final_output(), (0.0, 0.0), size);
        }
    }
}
scalars!(
    whole_max_content_width_survives_narrower_continuation,
    whole_max_content_width_survives_narrower_continuation_f32,
    whole_max_content_width_survives_narrower_continuation_f64
);

fn preferred_intrinsic_root_selects_its_own_component<S: LayoutScalar>() {
    for (preferred, width) in [
        (PreferredSizeOf::MAX_CONTENT, 80.0),
        (PreferredSizeOf::MIN_CONTENT, 10.0),
    ] {
        let mut root = block::<S>(None, None);
        root.size.width = preferred;
        let mut tree = Tree::new(vec![root, block(None, None)], vec![vec![1], vec![]]);
        tree.measurements[1] = Some(Measurement::IntrinsicRange);
        let request =
            LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
        let batch = compute(&tree, 0, &request).unwrap();
        assert_packet(
            single(&batch, 0).unrounded_output(),
            (0.0, 0.0),
            (width, 30.0),
        );
        assert_packet(single(&batch, 0).final_output(), (0.0, 0.0), (width, 30.0));
        assert_packet(
            single(&batch, 1).unrounded_output(),
            (0.0, 0.0),
            (width, 30.0),
        );
        assert_packet(single(&batch, 1).final_output(), (0.0, 0.0), (width, 30.0));
    }
}
scalars!(
    preferred_intrinsic_root_selects_its_own_component,
    preferred_intrinsic_root_selects_its_own_component_f32,
    preferred_intrinsic_root_selects_its_own_component_f64
);

fn root_and_nested_intrinsic_components_keep_whole_demand<S: LayoutScalar>() {
    for nested in [false, true] {
        for (preferred, width) in [
            (PreferredSizeOf::MAX_CONTENT, 80.0),
            (PreferredSizeOf::MIN_CONTENT, 10.0),
        ] {
            let mut subject = block::<S>(None, None);
            subject.size.width = preferred;
            let (inputs, children, owner, leaves) = if nested {
                (
                    vec![
                        block(None, None),
                        subject,
                        block(None, None),
                        block(None, None),
                    ],
                    vec![vec![1], vec![2, 3], vec![], vec![]],
                    1,
                    [2, 3],
                )
            } else {
                (
                    vec![subject, block(None, None), block(None, None)],
                    vec![vec![1, 2], vec![], vec![]],
                    0,
                    [1, 2],
                )
            };
            let mut tree = Tree::new(inputs, children);
            for leaf in leaves {
                tree.measurements[leaf] = Some(Measurement::IntrinsicRange);
            }
            let request = unequal(LayoutSourceBasis::fresh());
            let batch = compute(&tree, 0, &request).unwrap();
            let owners = fragments(&batch, owner);
            assert_eq!(owners.len(), 2);
            assert_fragment(&owners[0], (width, 50.0));
            assert_fragment(&owners[1], (width, 30.0));
            if nested {
                let roots = fragments(&batch, 0);
                assert_fragment(&roots[0], (100.0, 50.0));
                assert_fragment(&roots[1], (60.0, 30.0));
                for index in [0, 1] {
                    assert_eq!(
                        owners[index].coordinate_parent(),
                        Some(&FragmentCoordinateSpace::Fragment(
                            roots[index].id().clone()
                        ))
                    );
                }
            }
            for (index, leaf) in leaves.into_iter().enumerate() {
                let child = single(&batch, leaf);
                assert_eq!(
                    child.coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        owners[index].id().clone()
                    ))
                );
                assert_packet(child.unrounded_output(), (0.0, 0.0), (width, 30.0));
                assert_packet(child.final_output(), (0.0, 0.0), (width, 30.0));
            }
        }
    }
}
scalars!(
    root_and_nested_intrinsic_components_keep_whole_demand,
    root_and_nested_intrinsic_components_keep_whole_demand_f32,
    root_and_nested_intrinsic_components_keep_whole_demand_f64
);

fn whole_intrinsic_preparation_preserves_forced_suffix_measurement<S: LayoutScalar>() {
    let mut intrinsic = block::<S>(None, None);
    intrinsic.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut later = block::<S>(None, None);
    later.break_before = FragmentationBreak::Page;
    let mut tree = Tree::new(
        vec![
            block(None, None),
            intrinsic,
            block(Some(10.0), Some(20.0)),
            block(Some(80.0), Some(20.0)),
            later,
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    tree.measurements[4] = Some(Measurement::ResponsiveRejectNormalWidth(100.0));
    let request = unequal(LayoutSourceBasis::fresh());
    let batch = compute(&tree, 0, &request).unwrap();
    let roots = fragments(&batch, 0);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 40.0));
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (80.0, 40.0),
    );
    assert_packet(
        single(&batch, 4).unrounded_output(),
        (0.0, 0.0),
        (60.0, 40.0),
    );
    let observations = tree.observed.borrow();
    let later = observations
        .iter()
        .filter(|(node, _)| *node == 4)
        .collect::<Vec<_>>();
    assert!(!later.is_empty());
    assert!(
        later
            .iter()
            .all(|(_, input)| !normal_content_width(*input, 100.0))
    );
    assert!(
        later
            .iter()
            .any(|(_, input)| normal_content_width(*input, 60.0))
    );
}
scalars!(
    whole_intrinsic_preparation_preserves_forced_suffix_measurement,
    whole_intrinsic_preparation_preserves_forced_suffix_measurement_f32,
    whole_intrinsic_preparation_preserves_forced_suffix_measurement_f64
);

fn intrinsic_icb_template<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let original = root_request::<S>(100.0);
    let dimension = |value| Some(NonNegativeFiniteOf::new(S::from_f64(value)).unwrap());
    let fallback = OrthogonalFallbackOf::new(
        Size::new(dimension(100.0), dimension(999.0)),
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    );
    LayoutRootRequestOf::standalone(
        original.sizing_request().with_orthogonal_fallback(fallback),
        axes(),
        Size::new(dimension(100.0), None),
    )
}

fn whole_intrinsic_components_bind_first_frame_icb<S: LayoutScalar>() {
    let mut root = block::<S>(None, None);
    root.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut tree = Tree::new(
        vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
        vec![vec![1, 2], vec![], vec![]],
    );
    tree.measurements[1] = Some(Measurement::IntrinsicIcb);
    tree.measurements[2] = Some(Measurement::IntrinsicIcb);
    let continuous =
        LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), intrinsic_icb_template());
    let ordinary = compute(&tree, 0, &continuous).unwrap();
    assert_packet(
        single(&ordinary, 0).unrounded_output(),
        (0.0, 0.0),
        (999.0, 60.0),
    );
    tree.reset_observations();
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        intrinsic_icb_template(),
        FragmentationKind::Page,
        axes(),
        vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &request).unwrap();
    let roots = fragments(&batch, 0);
    assert_fragment(&roots[0], (50.0, 50.0));
    assert_fragment(&roots[1], (50.0, 30.0));
    for leaf in [1, 2] {
        assert_packet(
            single(&batch, leaf).unrounded_output(),
            (0.0, 0.0),
            (50.0, 30.0),
        );
    }
    let observations = tree.observed.borrow();
    let intrinsic = observations
        .iter()
        .filter(|(_, input)| {
            input.content_sizing_request().axes().width.operation() != SizingOperation::Normal
        })
        .collect::<Vec<_>>();
    assert!(!intrinsic.is_empty());
    for (_, input) in intrinsic {
        assert_eq!(
            input
                .content_sizing_request()
                .orthogonal_fallback()
                .height
                .unwrap()
                .get(),
            S::from_f64(50.0)
        );
    }
    let normal = observations
        .iter()
        .filter(|(_, input)| {
            input.content_sizing_request().axes().width.operation() == SizingOperation::Normal
        })
        .collect::<Vec<_>>();
    assert!(!normal.is_empty());
    for (_, input) in normal {
        assert_eq!(
            input
                .content_sizing_request()
                .orthogonal_fallback()
                .height
                .unwrap()
                .get(),
            S::from_f64(999.0)
        );
    }
}
scalars!(
    whole_intrinsic_components_bind_first_frame_icb,
    whole_intrinsic_components_bind_first_frame_icb_f32,
    whole_intrinsic_components_bind_first_frame_icb_f64
);

fn atomic_intrinsic_requests_bind_first_frame_icb<S: LayoutScalar>() {
    let mut atomic = block::<S>(None, Some(30.0));
    atomic.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut tree = Tree::new(
        vec![block(None, None), atomic.clone(), atomic],
        vec![vec![1, 2], vec![], vec![]],
    );
    tree.measurements[1] = Some(Measurement::IntrinsicIcb);
    tree.measurements[2] = Some(Measurement::IntrinsicIcb);
    let continuous =
        LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), intrinsic_icb_template());
    let ordinary = compute(&tree, 0, &continuous).unwrap();
    assert_packet(
        single(&ordinary, 0).unrounded_output(),
        (0.0, 0.0),
        (100.0, 60.0),
    );
    assert_packet(
        single(&ordinary, 1).unrounded_output(),
        (0.0, 0.0),
        (999.0, 30.0),
    );
    assert_packet(
        single(&ordinary, 2).unrounded_output(),
        (0.0, 30.0),
        (999.0, 30.0),
    );
    tree.reset_observations();
    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        intrinsic_icb_template(),
        FragmentationKind::Page,
        axes(),
        vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &request).unwrap();
    let roots = fragments(&batch, 0);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 30.0));
    for (index, leaf) in [1, 2].into_iter().enumerate() {
        let value = single(&batch, leaf);
        assert_eq!(
            value.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        assert_packet(value.unrounded_output(), (0.0, 0.0), (50.0, 30.0));
        assert_packet(value.final_output(), (0.0, 0.0), (50.0, 30.0));
    }
    let observations = tree.observed.borrow();
    assert!(!observations.is_empty());
    for (_, input) in observations.iter() {
        assert_eq!(
            input.content_sizing_request().axes().width.operation(),
            SizingOperation::MaxContent
        );
        assert_eq!(
            input
                .content_sizing_request()
                .orthogonal_fallback()
                .height
                .unwrap()
                .get(),
            S::from_f64(50.0)
        );
    }
}
scalars!(
    atomic_intrinsic_requests_bind_first_frame_icb,
    atomic_intrinsic_requests_bind_first_frame_icb_f32,
    atomic_intrinsic_requests_bind_first_frame_icb_f64
);

fn atomic_intrinsic_requests_keep_nearest_scrollport<S: LayoutScalar>() {
    let mut atomic = block::<S>(None, Some(30.0));
    atomic.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut tree = Tree::new(
        vec![block(None, None), atomic.clone(), atomic],
        vec![vec![1, 2], vec![], vec![]],
    );
    tree.measurements[1] = Some(Measurement::IntrinsicIcb);
    tree.measurements[2] = Some(Measurement::IntrinsicIcb);
    let original = intrinsic_icb_template::<S>();
    let fallback = original.sizing_request().orthogonal_fallback();
    let nearest = NearestScrollportOf::Present(Size::new(
        OrthogonalFallbackLimitOf::Unbounded,
        OrthogonalFallbackLimitOf::Finite(NonNegativeFiniteOf::new(S::from_f64(30.0)).unwrap()),
    ));
    let request = LayoutRootRequestOf::standalone(
        original
            .sizing_request()
            .with_orthogonal_fallback(OrthogonalFallbackOf::new(
                fallback.initial_containing_block(),
                nearest,
                fallback.containing_limits(),
            )),
        axes(),
        Size::new(
            Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
            None,
        ),
    );
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request,
        FragmentationKind::Page,
        axes(),
        vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &bound).unwrap();
    let roots = fragments(&batch, 0);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 30.0));
    for node in [1, 2] {
        assert_packet(
            single(&batch, node).unrounded_output(),
            (0.0, 0.0),
            (30.0, 30.0),
        );
        assert_packet(
            single(&batch, node).final_output(),
            (0.0, 0.0),
            (30.0, 30.0),
        );
    }
}
scalars!(
    atomic_intrinsic_requests_keep_nearest_scrollport,
    atomic_intrinsic_requests_keep_nearest_scrollport_f32,
    atomic_intrinsic_requests_keep_nearest_scrollport_f64
);

fn atomic_intrinsic_requests_keep_containing_limits<S: LayoutScalar>() {
    let mut root = block::<S>(None, None);
    root.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(40.0)).unwrap());
    let mut atomic = block::<S>(None, Some(30.0));
    atomic.size.width = PreferredSizeOf::MAX_CONTENT;
    let mut tree = Tree::new(vec![root, atomic], vec![vec![1], vec![]]);
    tree.measurements[1] = Some(Measurement::IntrinsicIcb);
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        intrinsic_icb_template(),
        FragmentationKind::Page,
        axes(),
        vec![frame(100.0, 50.0, true)],
    )
    .unwrap();
    let batch = compute(&tree, 0, &bound).unwrap();
    assert_packet(
        single(&batch, 0).unrounded_output(),
        (0.0, 0.0),
        (100.0, 30.0),
    );
    assert_packet(
        single(&batch, 1).unrounded_output(),
        (0.0, 0.0),
        (40.0, 30.0),
    );
    assert_packet(single(&batch, 1).final_output(), (0.0, 0.0), (40.0, 30.0));
}
scalars!(
    atomic_intrinsic_requests_keep_containing_limits,
    atomic_intrinsic_requests_keep_containing_limits_f32,
    atomic_intrinsic_requests_keep_containing_limits_f64
);

fn intrinsic_custody_does_not_create_block_percentage_basis<S: LayoutScalar>() {
    for fixed in [true, false] {
        let mut root = block::<S>(None, None);
        root.size.width = PreferredSizeOf::MAX_CONTENT;
        let mut measured = block::<S>(None, None);
        measured.size.height = if fixed {
            px(25.0)
        } else {
            PreferredSizeOf::value(
                LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap(),
            )
        };
        let mut tree = Tree::new(
            vec![
                root,
                block(Some(10.0), Some(30.0)),
                measured,
                block(Some(10.0), Some(30.0)),
            ],
            vec![vec![1, 2, 3], vec![], vec![], vec![]],
        );
        tree.measurements[2] = Some(Measurement::Fixed(20.0, 10.0));
        let bound = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            explicit_block_percentage_parent(),
            FragmentationKind::Page,
            axes(),
            vec![frame(100.0, 50.0, true), frame(60.0, 80.0, false)],
        )
        .unwrap();
        if fixed {
            let batch = compute(&tree, 0, &bound).unwrap();
            let roots = fragments(&batch, 0);
            assert_fragment(&roots[0], (20.0, 50.0));
            assert_fragment(&roots[1], (20.0, 55.0));
            assert_packet(
                single(&batch, 2).unrounded_output(),
                (0.0, 0.0),
                (20.0, 25.0),
            );
            assert_packet(
                single(&batch, 3).unrounded_output(),
                (0.0, 25.0),
                (10.0, 30.0),
            );
        } else {
            let error = compute(&tree, 0, &bound).unwrap_err();
            assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
            ));
        }
    }
}
scalars!(
    intrinsic_custody_does_not_create_block_percentage_basis,
    intrinsic_custody_does_not_create_block_percentage_basis_f32,
    intrinsic_custody_does_not_create_block_percentage_basis_f64
);

fn whole_intrinsic_components_respect_size_containment<S: LayoutScalar>() {
    for (containment, width) in [
        (SizeContainment::None, 80.0),
        (SizeContainment::InlineAxis, 0.0),
    ] {
        let mut root = block::<S>(None, None);
        root.size.width = PreferredSizeOf::MAX_CONTENT;
        let mut child = block::<S>(None, None);
        child.size_containment = containment;
        let tree = Tree::new(
            vec![root, child, block(Some(80.0), Some(20.0))],
            vec![vec![1], vec![2], vec![]],
        );
        let continuous =
            LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
        let ordinary = compute(&tree, 0, &continuous).unwrap();
        assert_packet(
            single(&ordinary, 0).unrounded_output(),
            (0.0, 0.0),
            (width, 20.0),
        );
        assert_packet(
            single(&ordinary, 1).unrounded_output(),
            (0.0, 0.0),
            (width, 20.0),
        );
        assert_packet(
            single(&ordinary, 2).unrounded_output(),
            (0.0, 0.0),
            (80.0, 20.0),
        );
        let bound = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
        let batch = compute(&tree, 0, &bound).unwrap();
        for phase in [false, true] {
            let root = single(&batch, 0);
            let child = single(&batch, 1);
            let leaf = single(&batch, 2);
            let output = |value: SingleBoxOutputRefOf<'_, usize, S>| {
                if phase {
                    value.final_output()
                } else {
                    value.unrounded_output()
                }
            };
            assert_packet(output(root), (0.0, 0.0), (width, 20.0));
            assert_packet(output(child), (0.0, 0.0), (width, 20.0));
            assert_packet(output(leaf), (0.0, 0.0), (80.0, 20.0));
            assert_eq!(
                child.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(root.id().clone()))
            );
            assert_eq!(
                leaf.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(child.id().clone()))
            );
        }
    }
}
scalars!(
    whole_intrinsic_components_respect_size_containment,
    whole_intrinsic_components_respect_size_containment_f32,
    whole_intrinsic_components_respect_size_containment_f64
);

fn contained_intrinsic_sizing_keeps_both_axes_and_authored_width<S: LayoutScalar>() {
    for (containment, authored_width, width, height) in [
        (SizeContainment::BothAxes, None, 0.0, 0.0),
        (SizeContainment::InlineAxis, Some(30.0), 30.0, 20.0),
    ] {
        let mut root = block::<S>(None, None);
        root.size.width = PreferredSizeOf::MAX_CONTENT;
        let mut child = block::<S>(authored_width, None);
        child.size_containment = containment;
        let tree = Tree::new(
            vec![root, child, block(Some(80.0), Some(20.0))],
            vec![vec![1], vec![2], vec![]],
        );
        for request in [
            LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0)),
            fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]),
        ] {
            let batch = compute(&tree, 0, &request).unwrap();
            let root = single(&batch, 0);
            let child = single(&batch, 1);
            let leaf = single(&batch, 2);
            assert_packet(root.unrounded_output(), (0.0, 0.0), (width, height));
            assert_packet(root.final_output(), (0.0, 0.0), (width, height));
            assert_packet(child.unrounded_output(), (0.0, 0.0), (width, height));
            assert_packet(child.final_output(), (0.0, 0.0), (width, height));
            assert_packet(leaf.unrounded_output(), (0.0, 0.0), (80.0, 20.0));
            assert_packet(leaf.final_output(), (0.0, 0.0), (80.0, 20.0));
            assert_eq!(
                child.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(root.id().clone()))
            );
            assert_eq!(
                leaf.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(child.id().clone()))
            );
        }
    }
}
scalars!(
    contained_intrinsic_sizing_keeps_both_axes_and_authored_width,
    contained_intrinsic_sizing_keeps_both_axes_and_authored_width_f32,
    contained_intrinsic_sizing_keeps_both_axes_and_authored_width_f64
);

fn intrinsic_preparation_preserves_anonymous_generating_context<S: LayoutScalar>() {
    for intrinsic in [false, true] {
        for preceding in [false, true] {
            let mut root = block::<S>(None, None);
            if intrinsic {
                root.size.width = PreferredSizeOf::MAX_CONTENT;
            }
            let mut anonymous = block::<S>(None, None);
            anonymous.block_generation = BlockGeneration::Anonymous;
            anonymous.text_indent = root.text_indent;
            let tree = Tree::new(
                vec![
                    root,
                    anonymous,
                    block(Some(10.0), Some(20.0)),
                    block(Some(10.0), Some(5.0)),
                ],
                vec![
                    if preceding { vec![3, 1] } else { vec![1] },
                    vec![2],
                    vec![],
                    vec![],
                ],
            );
            let width = if intrinsic { 10.0 } else { 100.0 };
            let offset = if preceding { 5.0 } else { 0.0 };
            for request in [
                LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0)),
                fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]),
            ] {
                let batch = compute(&tree, 0, &request)
                    .expect("anonymous Block retains its generating context");
                let root = single(&batch, 0);
                let anonymous = single(&batch, 1);
                let leaf = single(&batch, 2);
                assert_packet(root.unrounded_output(), (0.0, 0.0), (width, offset + 20.0));
                assert_packet(root.final_output(), (0.0, 0.0), (width, offset + 20.0));
                assert_packet(anonymous.unrounded_output(), (0.0, offset), (width, 20.0));
                assert_packet(anonymous.final_output(), (0.0, offset), (width, 20.0));
                assert_eq!(anonymous.generated_role(), BlockGeneration::Anonymous);
                assert_eq!(
                    anonymous.unrounded_output().source_index,
                    SourceIndex::new(usize::from(preceding))
                );
                assert_eq!(
                    anonymous.coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(root.id().clone()))
                );
                assert_packet(leaf.unrounded_output(), (0.0, 0.0), (10.0, 20.0));
                assert_packet(leaf.final_output(), (0.0, 0.0), (10.0, 20.0));
                assert_eq!(
                    leaf.coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(anonymous.id().clone()))
                );
            }
        }
    }
}
scalars!(
    intrinsic_preparation_preserves_anonymous_generating_context,
    intrinsic_preparation_preserves_anonymous_generating_context_f32,
    intrinsic_preparation_preserves_anonymous_generating_context_f64
);

fn minimum_progress_input<S: LayoutScalar>(minimum: f64) -> NodeInputOf<S> {
    let mut input = block(None, None);
    input.min_size.height = MinSizeOf::value(LengthPercentageOf::px(S::from_f64(minimum)).unwrap());
    input
}

fn minimum_progress_small_minimum_keeps_content_geometry<S: LayoutScalar>() {
    for nested in [false, true] {
        let (mut tree, subject) = ordinary_ancestor_tree::<S>(nested, 30.0);
        constrain_block(
            &mut tree.inputs[subject],
            BlockConstraint::Minimum,
            1.0,
            1000.0,
        );
        let request = unequal(LayoutSourceBasis::fresh());
        let batch = minimum_progress_batch(&tree, &request);
        let children: &[&[(usize, usize)]] = if nested {
            &[&[(0, 1)], &[(0, 1)]]
        } else {
            &[&[(0, 1)], &[(1, 2)]]
        };
        let roots = minimum_progress_chain(&batch, 0, children);
        assert_fragment(&roots[0], (100.0, 50.0));
        assert_fragment(&roots[1], (60.0, 30.0));
        let parents = if nested {
            let values = minimum_progress_chain(&batch, subject, &[&[(0, 2)], &[(1, 3)]]);
            for index in [0, 1] {
                assert_eq!(
                    values[index].coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        roots[index].id().clone()
                    ))
                );
                assert_eq!(values[index].unrounded().location, Point::ZERO);
                assert_eq!(values[index].final_geometry().location, Point::ZERO);
            }
            values
        } else {
            roots
        };
        assert_fragment(&parents[0], (100.0, 50.0));
        assert_fragment(&parents[1], (60.0, 30.0));
        minimum_progress_child(
            &batch,
            if nested { 2 } else { 1 },
            0,
            parents[0].id(),
            (0.0, 0.0),
            (100.0, 30.0),
        );
        minimum_progress_child(
            &batch,
            if nested { 3 } else { 2 },
            1,
            parents[1].id(),
            (0.0, 0.0),
            (60.0, 30.0),
        );
    }
}
scalars!(
    minimum_progress_small_minimum_keeps_content_geometry,
    minimum_progress_small_minimum_keeps_content_geometry_f32,
    minimum_progress_small_minimum_keeps_content_geometry_f64
);

fn minimum_progress_batch<S: LayoutScalar>(
    tree: &Tree<S>,
    request: &LayoutBoundRequestOf<S>,
) -> Batch<S> {
    let mut sink = Sink::new(tree.inputs.len());
    sink.authorize(0, request);
    let batch = compute(tree, 0, request)
        .expect("absolute block minimum continues until its required progress completes");
    assert_eq!(batch.root(), 0);
    assert_eq!(batch.target_basis(), Some(&request.target_basis()));
    assert!(batch.replaces_whole_root());
    let mut nodes = batch
        .node_layouts()
        .map(|(node, _)| node)
        .collect::<Vec<_>>();
    nodes.sort_unstable();
    assert_eq!(nodes, (0..tree.inputs.len()).collect::<Vec<_>>());
    let mut identities = Vec::new();
    for (_, value) in batch.node_layouts() {
        match value {
            NodeLayoutRefOf::SingleBox(value) => identities.push(value.id().clone()),
            NodeLayoutRefOf::Fragments(values) => {
                identities.extend(values.iter().map(|value| value.id().clone()));
            }
            NodeLayoutRefOf::NonBox(_) => panic!("fixture contains only visible source boxes"),
        }
    }
    for (index, identity) in identities.iter().enumerate() {
        assert!(!identities[..index].contains(identity));
    }
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(sink.commits, 1);
    assert_eq!(sink.targets.published_members(0).len(), tree.inputs.len());
    batch
}

fn minimum_progress_chain<'a, S: LayoutScalar>(
    batch: &'a Batch<S>,
    node: usize,
    children: &[&[(usize, usize)]],
) -> &'a [FragmentOutputOf<usize, S>] {
    let values = fragments(batch, node);
    assert_eq!(values.len(), children.len());
    for (index, (value, children)) in values.iter().zip(children).enumerate() {
        assert_eq!(value.node(), node);
        assert_eq!(value.continuation_order(), index);
        assert_eq!(
            value.previous(),
            index.checked_sub(1).map(|prior| values[prior].id())
        );
        assert_eq!(
            value.next(),
            values.get(index + 1).map(FragmentOutputOf::id)
        );
        assert_eq!(
            value.fragmentainer(),
            Some(batch.fragmentainers()[index].id())
        );
        assert_eq!(batch.fragment(value.id()).unwrap().id(), value.id());
        assert_eq!(
            value.source_children(),
            children
                .iter()
                .map(|&(rank, child)| (SourceIndex::new(rank), child))
                .collect::<Vec<_>>()
        );
        if node == batch.root() {
            assert_eq!(
                value.coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragmentainer(
                    batch.fragmentainers()[index].id().clone()
                ))
            );
        }
    }
    values
}

fn minimum_progress_child<S: LayoutScalar>(
    batch: &Batch<S>,
    node: usize,
    source_index: usize,
    parent: &LayoutFragmentId,
    location: (f64, f64),
    size: (f64, f64),
) {
    let value = single(batch, node);
    assert_eq!(
        value.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(parent.clone()))
    );
    for output in [value.unrounded_output(), value.final_output()] {
        assert_eq!(output.source_index, SourceIndex::new(source_index));
        assert_packet(output, location, size);
    }
}

fn minimum_progress_break_fill_counts_unused_extent<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            minimum_progress_input::<S>(90.0),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
        ],
        vec![vec![1, 2], vec![], vec![]],
    );
    let batch = minimum_progress_batch(&tree, &unequal(LayoutSourceBasis::fresh()));
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    // Break3 §5.3 counts the first fragment's unused20 as minimum progress:
    // 50 +40 =90, even though the two source children occupy only30 +30.
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 40.0));
    for value in roots {
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
}
scalars!(
    minimum_progress_break_fill_counts_unused_extent,
    minimum_progress_break_fill_counts_unused_extent_f32,
    minimum_progress_break_fill_counts_unused_extent_f64
);

fn minimum_progress_terminal_fragment_has_no_source_child<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![minimum_progress_input::<S>(150.0), block(None, Some(10.0))],
        vec![vec![1], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[]]);
    assert_fragment(&roots[0], (100.0, 100.0));
    assert_fragment(&roots[1], (60.0, 50.0));
    for value in roots {
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 10.0));
}
scalars!(
    minimum_progress_terminal_fragment_has_no_source_child,
    minimum_progress_terminal_fragment_has_no_source_child_f32,
    minimum_progress_terminal_fragment_has_no_source_child_f64
);

fn minimum_progress_nested_residual_precedes_following_sibling<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            block(None, None),
            block(None, Some(20.0)),
            minimum_progress_input::<S>(100.0),
            block(None, Some(10.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 2, 4], vec![], vec![3], vec![], vec![]],
    );
    let batch = minimum_progress_batch(&tree, &unequal(LayoutSourceBasis::fresh()));
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1), (1, 2)], &[(1, 2), (2, 4)]]);
    let wrappers = minimum_progress_chain(&batch, 2, &[&[(0, 3)], &[]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 80.0));
    assert_fragment(&wrappers[0], (100.0, 30.0));
    assert_fragment(&wrappers[1], (60.0, 70.0));
    for (index, y) in [(0, 20.0), (1, 0.0)] {
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        for geometry in [
            wrappers[index].unrounded(),
            wrappers[index].final_geometry(),
        ] {
            assert_eq!(geometry.location, Point::new(S::ZERO, S::from_f64(y)));
        }
        assert_eq!(roots[index].unrounded().location, Point::ZERO);
        assert_eq!(roots[index].final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 20.0));
    minimum_progress_child(&batch, 3, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 10.0));
    minimum_progress_child(&batch, 4, 2, roots[1].id(), (0.0, 70.0), (60.0, 10.0));
}
scalars!(
    minimum_progress_nested_residual_precedes_following_sibling,
    minimum_progress_nested_residual_precedes_following_sibling_f32,
    minimum_progress_nested_residual_precedes_following_sibling_f64
);

fn minimum_progress_padding_case<S: LayoutScalar>(sizing: BoxSizing, final_height: f64) {
    let mut root = minimum_progress_input::<S>(100.0);
    root.box_sizing = sizing;
    let padding = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
    root.padding.top = padding;
    root.padding.bottom = padding;
    let tree = Tree::new(
        vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
        vec![vec![1, 2], vec![], vec![]],
    );
    let batch = minimum_progress_batch(&tree, &unequal(LayoutSourceBasis::fresh()));
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    // Content-box minimum100 plus the two10-unit sliced edges occupies120;
    // border-box minimum100 includes those edges. Both fill the first50.
    for (index, height, top, bottom) in [(0, 50.0, 10.0, 0.0), (1, final_height, 0.0, 10.0)] {
        for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
            assert_eq!(geometry.location, Point::ZERO);
            assert_eq!(
                geometry.size,
                Size::new(
                    S::from_f64(if index == 0 { 100.0 } else { 60.0 }),
                    S::from_f64(height)
                )
            );
            assert_eq!(
                geometry.padding,
                Edges {
                    top: S::from_f64(top),
                    bottom: S::from_f64(bottom),
                    left: S::ZERO,
                    right: S::ZERO
                }
            );
            assert_eq!(geometry.border, Edges::all(S::ZERO));
            assert_eq!(geometry.margin, Edges::all(S::ZERO));
        }
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 10.0), (100.0, 30.0));
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
}
fn minimum_progress_content_box_minimum_keeps_sliced_padding<S: LayoutScalar>() {
    minimum_progress_padding_case::<S>(BoxSizing::ContentBox, 70.0);
}
scalars!(
    minimum_progress_content_box_minimum_keeps_sliced_padding,
    minimum_progress_content_box_minimum_keeps_sliced_padding_f32,
    minimum_progress_content_box_minimum_keeps_sliced_padding_f64
);
fn minimum_progress_border_box_minimum_includes_sliced_padding<S: LayoutScalar>() {
    minimum_progress_padding_case::<S>(BoxSizing::BorderBox, 50.0);
}
scalars!(
    minimum_progress_border_box_minimum_includes_sliced_padding,
    minimum_progress_border_box_minimum_includes_sliced_padding_f32,
    minimum_progress_border_box_minimum_includes_sliced_padding_f64
);

fn minimum_progress_percentage_inline_size_uses_each_frame<S: LayoutScalar>() {
    let mut wrapper = minimum_progress_input::<S>(90.0);
    wrapper.size.width = PreferredSizeOf::value(
        LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap(),
    );
    let tree = Tree::new(
        vec![
            block(None, None),
            wrapper,
            block(None, Some(30.0)),
            block(None, Some(30.0)),
        ],
        vec![vec![1], vec![2, 3], vec![], vec![]],
    );
    let batch = minimum_progress_batch(&tree, &unequal(LayoutSourceBasis::fresh()));
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[(1, 3)]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 40.0));
    assert_fragment(&wrappers[0], (50.0, 50.0));
    assert_fragment(&wrappers[1], (30.0, 40.0));
    for index in [0, 1] {
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        for geometry in [
            roots[index].unrounded(),
            roots[index].final_geometry(),
            wrappers[index].unrounded(),
            wrappers[index].final_geometry(),
        ] {
            assert_eq!(geometry.location, Point::ZERO);
        }
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (50.0, 30.0));
    minimum_progress_child(&batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (30.0, 30.0));
}
scalars!(
    minimum_progress_percentage_inline_size_uses_each_frame,
    minimum_progress_percentage_inline_size_uses_each_frame_f32,
    minimum_progress_percentage_inline_size_uses_each_frame_f64
);

fn minimum_progress_percentage_edges<S: LayoutScalar>(sizing: BoxSizing, minimum: f64) -> Tree<S> {
    let mut root = minimum_progress_input::<S>(minimum);
    root.box_sizing = sizing;
    let edge =
        LengthOf::value(LengthPercentageOf::from_percent_fraction(S::from_f64(0.1)).unwrap());
    root.padding.top = edge;
    root.padding.bottom = edge;
    Tree::new(vec![root, block(None, Some(10.0))], vec![vec![1], vec![]])
}

fn minimum_progress_percentage_content_box_edges_keep_target<S: LayoutScalar>() {
    let tree = minimum_progress_percentage_edges::<S>(BoxSizing::ContentBox, 150.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[]]);
    // The first border extent100 contains start padding10 and content90.
    // The remaining content60 plus the second frame's end padding6 gives66.
    for (index, top, bottom) in [(0, 10.0, 0.0), (1, 0.0, 6.0)] {
        for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
            assert_eq!(geometry.location, Point::ZERO);
            assert_eq!(
                geometry.size,
                Size::new(
                    S::from_f64(if index == 0 { 100.0 } else { 60.0 }),
                    S::from_f64(if index == 0 { 100.0 } else { 66.0 }),
                )
            );
            assert_eq!(geometry.padding.top, S::from_f64(top));
            assert_eq!(geometry.padding.bottom, S::from_f64(bottom));
            assert_eq!(geometry.border, Edges::all(S::ZERO));
        }
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 10.0), (100.0, 10.0));
}
scalars!(
    minimum_progress_percentage_content_box_edges_keep_target,
    minimum_progress_percentage_content_box_edges_keep_target_f32,
    minimum_progress_percentage_content_box_edges_keep_target_f64
);

fn minimum_progress_percentage_border_box_tail_refuses_varying_target<S: LayoutScalar>() {
    let tree = minimum_progress_percentage_edges::<S>(BoxSizing::BorderBox, 150.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile
        )
    ));
}
scalars!(
    minimum_progress_percentage_border_box_tail_refuses_varying_target,
    minimum_progress_percentage_border_box_tail_refuses_varying_target_f32,
    minimum_progress_percentage_border_box_tail_refuses_varying_target_f64
);

fn minimum_progress_percentage_border_box_whole_stays_supported<S: LayoutScalar>() {
    let tree = minimum_progress_percentage_edges::<S>(BoxSizing::BorderBox, 90.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let expected = (
        Point::ZERO,
        Size::new(S::from_f64(100.0), S::from_f64(90.0)),
        Edges {
            top: S::from_f64(10.0),
            bottom: S::from_f64(10.0),
            left: S::ZERO,
            right: S::ZERO,
        },
    );
    match batch.node_layout(0).unwrap() {
        NodeLayoutRefOf::SingleBox(value) => {
            for geometry in [value.unrounded_output(), value.final_output()] {
                assert_eq!(
                    (geometry.location, geometry.size, geometry.padding),
                    expected
                );
                assert_eq!(geometry.border, Edges::all(S::ZERO));
                assert_eq!(geometry.margin, Edges::all(S::ZERO));
            }
        }
        NodeLayoutRefOf::Fragments(values) => {
            assert_eq!(values.len(), 1);
            for geometry in [values[0].unrounded(), values[0].final_geometry()] {
                assert_eq!(
                    (geometry.location, geometry.size, geometry.padding),
                    expected
                );
                assert_eq!(geometry.border, Edges::all(S::ZERO));
                assert_eq!(geometry.margin, Edges::all(S::ZERO));
            }
        }
        NodeLayoutRefOf::NonBox(_) => panic!("visible whole box retains its geometry"),
    }
    let child = single(&batch, 1);
    for output in [child.unrounded_output(), child.final_output()] {
        assert_packet(output, (0.0, 10.0), (100.0, 10.0));
    }
    assert_eq!(batch.node_layouts().count(), 2);
}
scalars!(
    minimum_progress_percentage_border_box_whole_stays_supported,
    minimum_progress_percentage_border_box_whole_stays_supported_f32,
    minimum_progress_percentage_border_box_whole_stays_supported_f64
);

fn minimum_margin_classification_preserves_authored_positive_minimum<S: LayoutScalar>() {
    let wrapper = minimum_progress_input::<S>(10.0);
    let mut child = block::<S>(None, Some(0.0));
    let margin = LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(5.0)).unwrap());
    child.margin.top = margin;
    child.margin.bottom = margin;
    let tree = Tree::new(
        vec![block(None, None), wrapper, child],
        vec![vec![1], vec![2], vec![]],
    );

    // CSS2 §8.3.1 and its retained errata use the computed minimum when
    // classifying collapse-through. A temporary used-size floor cannot erase
    // the wrapper's positive minimum. The public continuous owner is an
    // independent control over exactly these same immutable authored inputs.
    let continuous = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    for final_phase in [false, true] {
        let output = |node| {
            let value = single(&continuous, node);
            if final_phase {
                value.final_output()
            } else {
                value.unrounded_output()
            }
        };
        assert_eq!(
            output(0).size,
            Size::new(S::from_f64(100.0), S::from_f64(20.0)),
            "ordinary continuous root control"
        );
        assert_eq!(output(0).location, Point::ZERO);
        assert_eq!(
            output(1).location,
            Point::new(S::ZERO, S::from_f64(5.0)),
            "ordinary continuous wrapper control"
        );
        assert_eq!(
            output(1).size,
            Size::new(S::from_f64(100.0), S::from_f64(10.0))
        );
        assert_eq!(output(2).size, Size::new(S::from_f64(100.0), S::ZERO));
        assert_eq!(
            output(2).margin,
            Edges {
                top: S::from_f64(5.0),
                bottom: S::from_f64(5.0),
                left: S::ZERO,
                right: S::ZERO
            }
        );
        for node in [0, 1, 2] {
            assert_eq!(output(node).source_index, SourceIndex::ZERO);
        }
    }

    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let mut sink = Sink::new(3);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request).unwrap();
    assert_eq!(batch.target_basis(), Some(&request.target_basis()));
    assert_eq!(batch.fragmentainers().len(), 1);
    for final_phase in [false, true] {
        for node in [0, 1, 2] {
            let actual = single(&batch, node);
            let expected = single(&continuous, node);
            let (actual, expected) = if final_phase {
                (actual.final_output(), expected.final_output())
            } else {
                (actual.unrounded_output(), expected.unrounded_output())
            };
            assert_eq!(
                actual, expected,
                "node{node}, final={final_phase}: whole-fit fragmentation preserves authored margin classification"
            );
        }
    }
    for result in [&continuous, &batch] {
        assert_eq!(result.node_layouts().count(), 3);
        for (owner, child) in [(0, 1), (1, 2)] {
            let owner = result.fragment(single(result, owner).id()).unwrap();
            assert_eq!(owner.source_children(), &[(SourceIndex::ZERO, child)]);
            assert_eq!(
                result
                    .child_fragments(owner.id())
                    .unwrap()
                    .map(FragmentOutputOf::node)
                    .collect::<Vec<_>>(),
                vec![child]
            );
        }
        let identities = [
            single(result, 0).id().clone(),
            single(result, 1).id().clone(),
            single(result, 2).id().clone(),
        ];
        for (index, identity) in identities.iter().enumerate() {
            assert!(!identities[..index].contains(identity));
        }
    }
    assert_eq!(
        single(&continuous, 0).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Root)
    );
    assert_eq!(
        single(&batch, 0).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            batch.fragmentainers()[0].id().clone()
        ))
    );
    assert_eq!(
        single(&batch, 1).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(
            single(&batch, 0).id().clone()
        ))
    );
    assert_eq!(
        single(&batch, 2).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(
            single(&batch, 1).id().clone()
        ))
    );
}
scalars!(
    minimum_margin_classification_preserves_authored_positive_minimum,
    minimum_margin_classification_preserves_authored_positive_minimum_f32,
    minimum_margin_classification_preserves_authored_positive_minimum_f64
);

fn minimum_calc_size_any_absolute_keeps_minimum_progress<S: LayoutScalar>() {
    for nested in [false, true] {
        let (mut tree, owner) = ordinary_ancestor_tree::<S>(nested, 30.0);
        // Values5's Any basis needs no additional information: this authored
        // calculation is the definite90px minimum, not a frame percentage.
        tree.inputs[owner].min_size.height = MinSizeOf::calc_size(
            MinSizeCalcBasis::Any,
            CalcSizeCalculationOf::value(LengthPercentageOf::px(S::from_f64(90.0)).unwrap()),
        )
        .unwrap();
        let continuous = compute_layout(&tree, 0, root_request(100.0)).unwrap();
        for final_phase in [false, true] {
            for node in [0, owner] {
                let value = single(&continuous, node);
                let output = if final_phase {
                    value.final_output()
                } else {
                    value.unrounded_output()
                };
                assert_packet(output, (0.0, 0.0), (100.0, 90.0));
                assert_eq!(output.source_index, SourceIndex::ZERO);
            }
        }

        let request = unequal(LayoutSourceBasis::fresh());
        let batch = minimum_progress_batch(&tree, &request);
        let root_children: &[&[(usize, usize)]] = if nested {
            &[&[(0, 1)], &[(0, 1)]]
        } else {
            &[&[(0, 1)], &[(1, 2)]]
        };
        let roots = minimum_progress_chain(&batch, 0, root_children);
        assert_fragment(&roots[0], (100.0, 50.0));
        assert_fragment(&roots[1], (60.0, 40.0));
        for value in roots {
            assert_eq!(value.unrounded().location, Point::ZERO);
            assert_eq!(value.final_geometry().location, Point::ZERO);
        }
        let parents = if nested {
            let values = minimum_progress_chain(&batch, owner, &[&[(0, 2)], &[(1, 3)]]);
            assert_fragment(&values[0], (100.0, 50.0));
            assert_fragment(&values[1], (60.0, 40.0));
            for index in [0, 1] {
                assert_eq!(
                    values[index].coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        roots[index].id().clone()
                    ))
                );
                assert_eq!(values[index].unrounded().location, Point::ZERO);
                assert_eq!(values[index].final_geometry().location, Point::ZERO);
            }
            values
        } else {
            roots
        };
        minimum_progress_child(
            &batch,
            if nested { 2 } else { 1 },
            0,
            parents[0].id(),
            (0.0, 0.0),
            (100.0, 30.0),
        );
        minimum_progress_child(
            &batch,
            if nested { 3 } else { 2 },
            1,
            parents[1].id(),
            (0.0, 0.0),
            (60.0, 30.0),
        );
    }
}
scalars!(
    minimum_calc_size_any_absolute_keeps_minimum_progress,
    minimum_calc_size_any_absolute_keeps_minimum_progress_f32,
    minimum_calc_size_any_absolute_keeps_minimum_progress_f64
);

fn minimum_whole_bounds_input<S: LayoutScalar>() -> NodeInputOf<S> {
    let mut root = minimum_progress_input::<S>(90.0);
    root.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
    root
}

fn minimum_whole_bounds_control<S: LayoutScalar>(
    tree: &Tree<S>,
    child_size: (f64, f64),
) -> Batch<S> {
    let continuous = compute_layout(tree, 0, root_request(100.0)).unwrap();
    // CSS2 §10.7 applies the minimum after the maximum. These authored bounds
    // therefore retain90, independently of any per-fragment used-size floor.
    for final_phase in [false, true] {
        let root = single(&continuous, 0);
        let child = single(&continuous, 1);
        let (root, child) = if final_phase {
            (root.final_output(), child.final_output())
        } else {
            (root.unrounded_output(), child.unrounded_output())
        };
        assert_packet(root, (0.0, 0.0), (100.0, 90.0));
        assert_packet(child, (0.0, 0.0), child_size);
        assert_eq!(root.source_index, SourceIndex::ZERO);
        assert_eq!(child.source_index, SourceIndex::ZERO);
    }
    assert_eq!(continuous.node_layouts().count(), 2);
    assert_ne!(single(&continuous, 0).id(), single(&continuous, 1).id());
    assert_eq!(
        single(&continuous, 0).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Root)
    );
    continuous
}

fn minimum_whole_bounds_preserved<S: LayoutScalar>(tree: &Tree<S>, continuous: &Batch<S>) {
    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let batch = minimum_progress_batch(tree, &request);
    for final_phase in [false, true] {
        for node in [0, 1] {
            let actual = single(&batch, node);
            let expected = single(continuous, node);
            let (actual, expected) = if final_phase {
                (actual.final_output(), expected.final_output())
            } else {
                (actual.unrounded_output(), expected.unrounded_output())
            };
            assert_eq!(
                actual, expected,
                "node{node}, final={final_phase}: whole-fit fragmentation retains authored minimum/maximum authority"
            );
        }
    }
    assert_eq!(batch.fragmentainers().len(), 1);
    assert_eq!(batch.node_layouts().count(), 2);
    assert_ne!(single(&batch, 0).id(), single(&batch, 1).id());
    assert_eq!(
        single(&batch, 0).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            batch.fragmentainers()[0].id().clone()
        ))
    );
    assert_eq!(
        single(&batch, 1).coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(
            single(&batch, 0).id().clone()
        ))
    );
}

fn minimum_whole_bounds_minimum_wins_smaller_maximum<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![minimum_whole_bounds_input::<S>(), block(None, Some(10.0))],
        vec![vec![1], vec![]],
    );
    let continuous = minimum_whole_bounds_control(&tree, (100.0, 10.0));
    minimum_whole_bounds_preserved(&tree, &continuous);
}
scalars!(
    minimum_whole_bounds_minimum_wins_smaller_maximum,
    minimum_whole_bounds_minimum_wins_smaller_maximum_f32,
    minimum_whole_bounds_minimum_wins_smaller_maximum_f64
);

fn minimum_whole_bounds_keep_authored_percentage_parent<S: LayoutScalar>() {
    let mut child = block::<S>(Some(20.0), None);
    child.size.height = PreferredSizeOf::value(
        LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap(),
    );
    let mut tree = Tree::new(
        vec![minimum_whole_bounds_input::<S>(), child],
        vec![vec![1], vec![]],
    );
    tree.measurements[1] = Some(Measurement::Fixed(20.0, 10.0));
    // The already-owned definite parent basis is max(min90,max20)=90.
    // Its child's50% is45; frame100 must neither replace nor create that basis.
    let continuous = minimum_whole_bounds_control(&tree, (20.0, 45.0));
    minimum_whole_bounds_preserved(&tree, &continuous);
}
scalars!(
    minimum_whole_bounds_keep_authored_percentage_parent,
    minimum_whole_bounds_keep_authored_percentage_parent_f32,
    minimum_whole_bounds_keep_authored_percentage_parent_f64
);

fn minimum_extra_qualification_three_frame_root_tail<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![minimum_progress_input::<S>(250.0), block(None, Some(10.0))],
        vec![vec![1], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 100.0, true),
            frame(60.0, 80.0, false),
            frame(90.0, 100.0, false),
        ],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[], &[]]);
    // Break3 §5.3 counts100 +80 of frame fill toward the250 minimum.
    for (value, size) in roots
        .iter()
        .zip([(100.0, 100.0), (60.0, 80.0), (90.0, 70.0)])
    {
        assert_fragment(value, size);
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 10.0));
}
scalars!(
    minimum_extra_qualification_three_frame_root_tail,
    minimum_extra_qualification_three_frame_root_tail_f32,
    minimum_extra_qualification_three_frame_root_tail_f64
);

fn minimum_extra_qualification_overlapping_ancestor_minima<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            minimum_progress_input::<S>(250.0),
            minimum_progress_input(150.0),
            block(None, Some(10.0)),
        ],
        vec![vec![1], vec![2], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 100.0, true),
            frame(60.0, 80.0, false),
            frame(90.0, 100.0, false),
        ],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1)], &[]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[]]);
    // The same first100 advances both ancestors; their minima are not added.
    for (value, size) in roots
        .iter()
        .zip([(100.0, 100.0), (60.0, 80.0), (90.0, 70.0)])
    {
        assert_fragment(value, size);
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    for (index, size) in [(100.0, 100.0), (60.0, 50.0)].into_iter().enumerate() {
        assert_fragment(&wrappers[index], size);
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        assert_eq!(wrappers[index].unrounded().location, Point::ZERO);
        assert_eq!(wrappers[index].final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 10.0));
}
scalars!(
    minimum_extra_qualification_overlapping_ancestor_minima,
    minimum_extra_qualification_overlapping_ancestor_minima_f32,
    minimum_extra_qualification_overlapping_ancestor_minima_f64
);

fn minimum_extra_qualification_last_child_force_follows_minimum_tail<S: LayoutScalar>() {
    let mut child = block::<S>(None, Some(10.0));
    child.break_after = FragmentationBreak::Page;
    let tree = Tree::new(
        vec![
            block(None, None),
            minimum_progress_input(150.0),
            child,
            block(None, Some(10.0)),
        ],
        vec![vec![1, 3], vec![2], vec![], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 100.0, true),
            frame(60.0, 80.0, false),
            frame(90.0, 100.0, false),
        ],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1)], &[(1, 3)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[]]);
    // The last child's force propagates after its container (§3.1.1), including
    // that container's minimum tail. The departure fills frame1 (§5.3).
    for (value, size) in roots
        .iter()
        .zip([(100.0, 100.0), (60.0, 80.0), (90.0, 10.0)])
    {
        assert_fragment(value, size);
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    for (index, size) in [(100.0, 100.0), (60.0, 50.0)].into_iter().enumerate() {
        assert_fragment(&wrappers[index], size);
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        assert_eq!(wrappers[index].unrounded().location, Point::ZERO);
        assert_eq!(wrappers[index].final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 10.0));
    minimum_progress_child(&batch, 3, 1, roots[2].id(), (0.0, 0.0), (90.0, 10.0));
}
scalars!(
    minimum_extra_qualification_last_child_force_follows_minimum_tail,
    minimum_extra_qualification_last_child_force_follows_minimum_tail_f32,
    minimum_extra_qualification_last_child_force_follows_minimum_tail_f64
);

fn minimum_extra_qualification_missing_tail_frame_preserves_host_receipts<S: LayoutScalar>() {
    let mut tree = Tree::new(
        vec![minimum_progress_input::<S>(250.0), block(None, Some(10.0))],
        vec![vec![1], vec![]],
    );
    let mut sink = Sink::new(2);
    let seed = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    for output in [
        single(&seed, 0).unrounded_output(),
        single(&seed, 0).final_output(),
    ] {
        assert_packet(output, (0.0, 0.0), (100.0, 250.0));
    }
    seed.apply_to(&mut sink).unwrap();
    sink.copy_cache_to(&mut tree);
    assert!(tree.caches.iter().any(|cache| !cache.is_empty()));
    assert!(tree.custody.iter().all(Option::is_some));
    let caches = tree.caches.clone();
    let custody = tree.custody.clone();
    tree.reset_observations();
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    sink.authorize(0, &request);
    // Only180 of the authored250 is available; source completion cannot end
    // the request while70 of minimum progress still needs a third frame.
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::MissingFragmentainer {
            index: 2
        })
    ));
    assert_eq!(tree.caches, caches);
    assert_eq!(tree.custody, custody);
    assert_eq!(tree.cache_calls.get(), 0);
    assert_eq!(tree.custody_calls.get(), 0);
}
scalars!(
    minimum_extra_qualification_missing_tail_frame_preserves_host_receipts,
    minimum_extra_qualification_missing_tail_frame_preserves_host_receipts_f32,
    minimum_extra_qualification_missing_tail_frame_preserves_host_receipts_f64
);

fn minimum_extra_qualification_vertical_minimum_uses_physical_block_axis<S: LayoutScalar>() {
    for mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let mut root_input = block::<S>(None, None);
        root_input.min_size.width =
            MinSizeOf::value(LengthPercentageOf::px(S::from_f64(90.0)).unwrap());
        let mut tree = Tree::new(
            vec![root_input, block(Some(10.0), None)],
            vec![vec![1], vec![]],
        );
        for input in &mut tree.inputs {
            input.writing_mode = mode;
        }
        let vertical = FlowAxes::new(mode, Direction::Ltr);
        let sizing = native_request::sizing(
            Size::new(Some(S::from_f64(50.0)), Some(S::from_f64(100.0))),
            Size::splat(SizingOperation::Normal),
        );
        let root = LayoutRootRequestOf::standalone(
            sizing,
            vertical,
            sizing.orthogonal_fallback().initial_containing_block(),
        );
        let request = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            root,
            FragmentationKind::Page,
            vertical,
            vec![frame(50.0, 100.0, true), frame(80.0, 60.0, false)],
        )
        .unwrap();
        let batch = minimum_progress_batch(&tree, &request);
        let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[]]);
        assert_fragment(&roots[0], (50.0, 100.0));
        assert_fragment(&roots[1], (40.0, 60.0));
        let decreasing_x = if mode == WritingMode::VerticalRl {
            40.0
        } else {
            0.0
        };
        for (value, x) in roots.iter().zip([0.0, decreasing_x]) {
            assert_eq!(
                value.unrounded().location,
                Point::new(S::from_f64(x), S::ZERO)
            );
            assert_eq!(
                value.final_geometry().location,
                Point::new(S::from_f64(x), S::ZERO)
            );
        }
        minimum_progress_child(
            &batch,
            1,
            0,
            roots[0].id(),
            (decreasing_x, 0.0),
            (10.0, 100.0),
        );
    }
}
scalars!(
    minimum_extra_qualification_vertical_minimum_uses_physical_block_axis,
    minimum_extra_qualification_vertical_minimum_uses_physical_block_axis_f32,
    minimum_extra_qualification_vertical_minimum_uses_physical_block_axis_f64
);

fn minimum_border_box_admission_equal_width_continuation_refuses<S: LayoutScalar>() {
    let tree = minimum_progress_percentage_edges::<S>(BoxSizing::BorderBox, 150.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(100.0, 100.0, false)],
    );
    // The bounded profile excludes continued BorderBox minima with original
    // percentage block insets, including equal-width fragmentainers.
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile
        )
    ));
}
scalars!(
    minimum_border_box_admission_equal_width_continuation_refuses,
    minimum_border_box_admission_equal_width_continuation_refuses_f32,
    minimum_border_box_admission_equal_width_continuation_refuses_f64
);

fn minimum_border_box_admission_deferred_tree<S: LayoutScalar>(minimum: f64) -> Tree<S> {
    let mut wrapper = minimum_progress_input::<S>(minimum);
    wrapper.box_sizing = BoxSizing::BorderBox;
    let edge =
        LengthOf::value(LengthPercentageOf::from_percent_fraction(S::from_f64(0.1)).unwrap());
    wrapper.padding.top = edge;
    wrapper.padding.bottom = edge;
    Tree::new(
        vec![
            block(None, None),
            block(None, Some(50.0)),
            wrapper,
            block(None, Some(10.0)),
        ],
        vec![vec![1, 2], vec![], vec![3], vec![]],
    )
}

fn minimum_border_box_admission_deferred_continuation_refuses_at_owner<S: LayoutScalar>() {
    let tree = minimum_border_box_admission_deferred_tree::<S>(150.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 50.0, true),
            frame(60.0, 100.0, false),
            frame(100.0, 100.0, false),
        ],
    );
    // The wrapper first enters frame1. Its actual continuation must be checked
    // there, rather than admitted by a whole-box snapshot from frame0.
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile
        )
    ));
}
scalars!(
    minimum_border_box_admission_deferred_continuation_refuses_at_owner,
    minimum_border_box_admission_deferred_continuation_refuses_at_owner_f32,
    minimum_border_box_admission_deferred_continuation_refuses_at_owner_f64
);

fn minimum_border_box_admission_deferred_whole_box_keeps_percentage_padding<S: LayoutScalar>() {
    let tree = minimum_border_box_admission_deferred_tree::<S>(90.0);
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(60.0, 100.0, false)],
    );
    let batch = minimum_progress_batch(&tree, &request);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 90.0));
    for root in roots {
        assert_eq!(root.unrounded().location, Point::ZERO);
        assert_eq!(root.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 50.0));
    let wrapper = single(&batch, 2);
    assert_eq!(
        wrapper.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(roots[1].id().clone()))
    );
    for output in [wrapper.unrounded_output(), wrapper.final_output()] {
        assert_eq!(output.source_index, SourceIndex::new(1));
        assert_eq!(output.location, Point::ZERO);
        assert_eq!(output.size, Size::new(S::from_f64(60.0), S::from_f64(90.0)));
        assert_eq!(
            output.padding,
            Edges {
                top: S::from_f64(6.0),
                bottom: S::from_f64(6.0),
                left: S::ZERO,
                right: S::ZERO,
            }
        );
        assert_eq!(output.border, Edges::all(S::ZERO));
        assert_eq!(output.margin, Edges::all(S::ZERO));
    }
    minimum_progress_child(&batch, 3, 0, wrapper.id(), (0.0, 6.0), (60.0, 10.0));
}
scalars!(
    minimum_border_box_admission_deferred_whole_box_keeps_percentage_padding,
    minimum_border_box_admission_deferred_whole_box_keeps_percentage_padding_f32,
    minimum_border_box_admission_deferred_whole_box_keeps_percentage_padding_f64
);

fn minimum_satisfied_closure_whole_geometry<S: LayoutScalar>(
    batch: &Batch<S>,
    bottom: f64,
    native: bool,
) {
    assert_eq!(batch.node_layouts().count(), 2);
    let root = single(batch, 0);
    let child = single(batch, 1);
    assert_ne!(root.id(), child.id());
    for output in [root.unrounded_output(), root.final_output()] {
        assert_eq!(output.source_index, SourceIndex::new(0));
        assert_eq!(output.location, Point::ZERO);
        assert_eq!(
            output.size,
            Size::new(S::from_f64(100.0), S::from_f64(150.0 + bottom))
        );
        let mut padding = Edges::all(S::ZERO);
        padding.bottom = S::from_f64(bottom);
        assert_eq!(output.padding, padding);
        assert_eq!(output.border, Edges::all(S::ZERO));
        assert_eq!(output.margin, Edges::all(S::ZERO));
    }
    for output in [child.unrounded_output(), child.final_output()] {
        assert_eq!(output.source_index, SourceIndex::new(0));
        assert_packet(output, (0.0, 0.0), (100.0, 150.0));
    }
    if native {
        assert_eq!(batch.fragmentainers().len(), 1);
        assert_eq!(
            root.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragmentainer(
                batch.fragmentainers()[0].id().clone()
            ))
        );
        assert_eq!(
            child.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(root.id().clone()))
        );
    } else {
        assert_eq!(
            root.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Root)
        );
    }
}

fn minimum_satisfied_closure_oversized_child_case<S: LayoutScalar>(bottom: f64) {
    let mut root = minimum_progress_input::<S>(1.0);
    root.padding.bottom = LengthOf::value(LengthPercentageOf::px(S::from_f64(bottom)).unwrap());
    let mut tree = Tree::new(vec![root, block(None, Some(150.0))], vec![vec![1], vec![]]);
    // The complete150-unit atomic child already exceeds the one-unit minimum.
    // Closing padding belongs to that whole box, not to another source item.
    let continuous =
        compute_layout(&tree, 0, root_request(100.0)).expect("continuous minimum control");
    minimum_satisfied_closure_whole_geometry(&continuous, bottom, false);
    tree.inputs[0].min_size.height = MinSizeOf::AUTO;
    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let automatic = compute(&tree, 0, &request).expect("native min-auto control");
    minimum_satisfied_closure_whole_geometry(&automatic, bottom, true);
    tree.inputs[0].min_size.height = MinSizeOf::value(LengthPercentageOf::px(S::ONE).unwrap());
    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let minimum = compute(&tree, 0, &request)
        .expect("satisfied minimum closes without another fragmentainer");
    minimum_satisfied_closure_whole_geometry(&minimum, bottom, true);
}

fn minimum_satisfied_closure_oversized_sole_child_stays_whole<S: LayoutScalar>() {
    minimum_satisfied_closure_oversized_child_case::<S>(0.0);
}
scalars!(
    minimum_satisfied_closure_oversized_sole_child_stays_whole,
    minimum_satisfied_closure_oversized_sole_child_stays_whole_f32,
    minimum_satisfied_closure_oversized_sole_child_stays_whole_f64
);

fn minimum_satisfied_closure_oversized_sole_child_retains_closing_padding<S: LayoutScalar>() {
    minimum_satisfied_closure_oversized_child_case::<S>(10.0);
}
scalars!(
    minimum_satisfied_closure_oversized_sole_child_retains_closing_padding,
    minimum_satisfied_closure_oversized_sole_child_retains_closing_padding_f32,
    minimum_satisfied_closure_oversized_sole_child_retains_closing_padding_f64
);

fn minimum_satisfied_closure_real_following_atomic_still_advances<S: LayoutScalar>() {
    for (height, forced) in [(10.0, false), (0.0, true), (0.0, false)] {
        // A real zero-height source child retains its source/forced break side;
        // it is distinct from a container's zero-source closing marker.
        for minimum in [false, true] {
            let mut root = block::<S>(None, None);
            if minimum {
                root.min_size.height = MinSizeOf::value(LengthPercentageOf::px(S::ONE).unwrap());
            }
            let mut following = block(None, Some(height));
            if forced {
                following.break_before = FragmentationBreak::Page;
            }
            let tree = Tree::new(
                vec![root, block(None, Some(150.0)), following],
                vec![vec![1, 2], vec![], vec![]],
            );
            let request = fragmented(
                LayoutSourceBasis::fresh(),
                vec![frame(100.0, 100.0, true), frame(60.0, 100.0, false)],
            );
            let batch = compute(&tree, 0, &request).expect(if minimum {
                "satisfied minimum retains actual following child"
            } else {
                "native min-auto following-child control"
            });
            assert_eq!(batch.node_layouts().count(), 3);
            assert_eq!(batch.fragmentainers().len(), 2);
            let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
            assert_fragment(&roots[0], (100.0, 150.0));
            assert_fragment(&roots[1], (60.0, height));
            for root in roots {
                assert_eq!(root.unrounded().location, Point::ZERO);
                assert_eq!(root.final_geometry().location, Point::ZERO);
            }
            minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 150.0));
            minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, height));
            assert_ne!(single(&batch, 1).id(), single(&batch, 2).id());
        }
    }
}
scalars!(
    minimum_satisfied_closure_real_following_atomic_still_advances,
    minimum_satisfied_closure_real_following_atomic_still_advances_f32,
    minimum_satisfied_closure_real_following_atomic_still_advances_f64
);

fn minimum_satisfied_closure_nested_graphical_tail_retains_padding_once<S: LayoutScalar>() {
    let mut root = minimum_progress_input::<S>(1.0);
    root.padding.bottom = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
    let tree = Tree::new(
        vec![root, minimum_progress_input(150.0), block(None, Some(10.0))],
        vec![vec![1], vec![2], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 60.0, false)],
    );
    let batch = minimum_progress_batch(&tree, &request);
    assert_eq!(batch.fragmentainers().len(), 2);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[]]);
    // The wrapper's100 +50 progress satisfies the outer minimum. Only its
    // closing fragment retains the outer10 padding, giving100 then60.
    for (index, width, height, bottom) in [(0, 100.0, 100.0, 0.0), (1, 60.0, 60.0, 10.0)] {
        for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
            assert_eq!(geometry.location, Point::ZERO);
            assert_eq!(
                geometry.size,
                Size::new(S::from_f64(width), S::from_f64(height))
            );
            let mut padding = Edges::all(S::ZERO);
            padding.bottom = S::from_f64(bottom);
            assert_eq!(geometry.padding, padding);
            assert_eq!(geometry.border, Edges::all(S::ZERO));
            assert_eq!(geometry.margin, Edges::all(S::ZERO));
        }
        assert_fragment(
            &wrappers[index],
            (width, if index == 0 { 100.0 } else { 50.0 }),
        );
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        assert_eq!(wrappers[index].unrounded().location, Point::ZERO);
        assert_eq!(wrappers[index].final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 10.0));
}
scalars!(
    minimum_satisfied_closure_nested_graphical_tail_retains_padding_once,
    minimum_satisfied_closure_nested_graphical_tail_retains_padding_once_f32,
    minimum_satisfied_closure_nested_graphical_tail_retains_padding_once_f64
);

fn maximum_progress_case<S: LayoutScalar>(maximum: f64, terminal_height: f64) {
    let mut root = block::<S>(None, None);
    root.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(maximum)).unwrap());
    let tree = Tree::new(
        vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
        vec![vec![1, 2], vec![], vec![]],
    );
    // The unbroken 60-unit flow fits both authored maxima. The ordinary owner
    // and a whole-fit native frame retain that 60 before testing continuation.
    let continuous =
        compute_layout(&tree, 0, root_request(100.0)).expect("ordinary absolute maximum control");
    for (node, y, height) in [(0, 0.0, 60.0), (1, 0.0, 30.0), (2, 30.0, 30.0)] {
        let value = single(&continuous, node);
        for output in [value.unrounded_output(), value.final_output()] {
            assert_packet(output, (0.0, y), (100.0, height));
        }
    }
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let mut whole_sink = Sink::new(3);
    whole_sink.authorize(0, &whole_request);
    let whole = compute(&tree, 0, &whole_request).expect("whole-fit absolute maximum control");
    assert_eq!(whole.fragmentainers().len(), 1);
    assert_eq!(whole.node_layouts().count(), 3);
    let whole_root = single(&whole, 0);
    for output in [whole_root.unrounded_output(), whole_root.final_output()] {
        assert_packet(output, (0.0, 0.0), (100.0, 60.0));
    }
    assert_eq!(
        whole_root.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            whole.fragmentainers()[0].id().clone()
        ))
    );
    minimum_progress_child(&whole, 1, 0, whole_root.id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&whole, 2, 1, whole_root.id(), (0.0, 30.0), (100.0, 30.0));
    whole.apply_to(&mut whole_sink).unwrap();

    let request = unequal(LayoutSourceBasis::fresh());
    let mut sink = Sink::new(3);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("absolute block maximum progresses across actual fragmentainers");
    assert_eq!(batch.root(), 0);
    assert_eq!(batch.target_basis(), Some(&request.target_basis()));
    assert!(batch.replaces_whole_root());
    assert_eq!(batch.fragmentainers().len(), 2);
    assert_eq!(batch.node_layouts().count(), 3);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    // Break3 §5.3 counts the first fragment's unused 20 toward the original
    // content maximum. Thus max 70 leaves 20, while max 1000 leaves the natural 30.
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, terminal_height));
    for value in roots {
        assert_eq!(value.unrounded().location, Point::ZERO);
        assert_eq!(value.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 30.0));
    // Exhausting the parent's content cap never truncates this source child:
    // its actual 30-unit box may overflow the terminal parent's 20-unit box.
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
    assert_ne!(single(&batch, 1).id(), single(&batch, 2).id());
    for node in [1, 2] {
        for root in roots {
            assert_ne!(single(&batch, node).id(), root.id());
        }
    }
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(sink.commits, 1);
    assert_eq!(sink.targets.published_members(0), &[0, 1, 2]);
    assert_eq!(sink.state.custody[0], None);
    for node in [1, 2] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}

fn maximum_progress_large_cap_keeps_natural_terminal_fragment<S: LayoutScalar>() {
    maximum_progress_case::<S>(1000.0, 30.0);
}
scalars!(
    maximum_progress_large_cap_keeps_natural_terminal_fragment,
    maximum_progress_large_cap_keeps_natural_terminal_fragment_f32,
    maximum_progress_large_cap_keeps_natural_terminal_fragment_f64
);

fn maximum_progress_consumes_break_fill_before_terminal_cap<S: LayoutScalar>() {
    maximum_progress_case::<S>(70.0, 20.0);
}
scalars!(
    maximum_progress_consumes_break_fill_before_terminal_cap,
    maximum_progress_consumes_break_fill_before_terminal_cap_f32,
    maximum_progress_consumes_break_fill_before_terminal_cap_f64
);

fn maximum_progress_nested_cap_positions_following_sibling_before_overflow<S: LayoutScalar>() {
    let mut wrapper = block::<S>(None, None);
    wrapper.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(70.0)).unwrap());
    let tree = Tree::new(
        vec![
            block(None, None),
            wrapper,
            block(None, Some(30.0)),
            block(None, Some(30.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let mut whole_sink = Sink::new(5);
    whole_sink.authorize(0, &whole_request);
    let controls = [
        compute_layout(&tree, 0, root_request(100.0)).expect("ordinary nested maximum control"),
        compute(&tree, 0, &whole_request).expect("whole-fit nested maximum control"),
    ];
    for control in controls {
        for (node, y, height) in [
            (0, 0.0, 70.0),
            (1, 0.0, 60.0),
            (2, 0.0, 30.0),
            (3, 30.0, 30.0),
            (4, 60.0, 10.0),
        ] {
            let value = single(&control, node);
            for output in [value.unrounded_output(), value.final_output()] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
    }
    let request = unequal(LayoutSourceBasis::fresh());
    let mut sink = Sink::new(5);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request).expect("nested maximum counts prior break fill");
    assert_eq!(batch.node_layouts().count(), 5);
    assert_eq!(batch.fragmentainers().len(), 2);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1), (1, 4)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[(1, 3)]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 30.0));
    for (index, size) in [(100.0, 50.0), (60.0, 20.0)].into_iter().enumerate() {
        assert_fragment(&wrappers[index], size);
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        for geometry in [
            roots[index].unrounded(),
            roots[index].final_geometry(),
            wrappers[index].unrounded(),
            wrappers[index].final_geometry(),
        ] {
            assert_eq!(geometry.location, Point::ZERO);
        }
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (60.0, 30.0));
    // The following sibling uses the wrapper's remaining box extent 20, not
    // the 30-unit overflow of its last child; the outer terminal box is 30 high.
    minimum_progress_child(&batch, 4, 1, roots[1].id(), (0.0, 20.0), (60.0, 10.0));
    batch.apply_to(&mut sink).unwrap();
    for node in [2, 3, 4] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_nested_cap_positions_following_sibling_before_overflow,
    maximum_progress_nested_cap_positions_following_sibling_before_overflow_f32,
    maximum_progress_nested_cap_positions_following_sibling_before_overflow_f64
);

fn maximum_progress_sliced_padding_converts_original_box_cap_once<S: LayoutScalar>() {
    // Execute both box-sizing lanes before asserting continuing results, so a
    // missing profile in one lane does not hide the other's actual response.
    let results = [BoxSizing::ContentBox, BoxSizing::BorderBox].map(|sizing| {
        let mut root = block::<S>(None, None);
        root.box_sizing = sizing;
        root.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(70.0)).unwrap());
        let edge = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
        root.padding.top = edge;
        root.padding.bottom = edge;
        let tree = Tree::new(
            vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
            vec![vec![1, 2], vec![], vec![]],
        );
        let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
        let mut sink = Sink::new(3);
        sink.authorize(0, &whole_request);
        let controls = [
            compute_layout(&tree, 0, root_request(100.0))
                .expect("ordinary maximum padding control"),
            compute(&tree, 0, &whole_request).expect("whole-fit maximum padding control"),
        ];
        let whole_height = if sizing == BoxSizing::ContentBox {
            80.0
        } else {
            70.0
        };
        for control in controls {
            let parent = single(&control, 0);
            for output in [parent.unrounded_output(), parent.final_output()] {
                assert_eq!(output.location, Point::ZERO);
                assert_eq!(
                    output.size,
                    Size::new(S::from_f64(100.0), S::from_f64(whole_height))
                );
                assert_eq!(
                    output.padding,
                    Edges {
                        top: S::from_f64(10.0),
                        bottom: S::from_f64(10.0),
                        left: S::ZERO,
                        right: S::ZERO
                    }
                );
                assert_eq!(output.border, Edges::all(S::ZERO));
                assert_eq!(output.margin, Edges::all(S::ZERO));
            }
            for (node, y) in [(1, 10.0), (2, 40.0)] {
                let child = single(&control, node);
                for output in [child.unrounded_output(), child.final_output()] {
                    assert_eq!(output.source_index, SourceIndex::new(node - 1));
                    assert_packet(output, (0.0, y), (100.0, 30.0));
                }
            }
        }
        let request = unequal(LayoutSourceBasis::fresh());
        sink.authorize(0, &request);
        (sizing, compute(&tree, 0, &request))
    });
    assert!(
        results.iter().all(|(_, result)| result.is_ok()),
        "sliced padding outcomes: {:?}",
        results
            .each_ref()
            .map(|(sizing, result)| (*sizing, result.as_ref().map(|_| ())))
    );
    for (sizing, result) in results {
        let batch = result.unwrap();
        assert_eq!(batch.node_layouts().count(), 3);
        assert_eq!(batch.fragmentainers().len(), 2);
        let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
        // Break3 §5.3 debits the first content fill 40. The original content
        // cap is 70 (ContentBox) or 50 (BorderBox), leaving 30 or 10 plus end 10.
        let terminal = if sizing == BoxSizing::ContentBox {
            40.0
        } else {
            20.0
        };
        for (index, width, height, top, bottom) in
            [(0, 100.0, 50.0, 10.0, 0.0), (1, 60.0, terminal, 0.0, 10.0)]
        {
            for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
                assert_eq!(geometry.location, Point::ZERO);
                assert_eq!(
                    geometry.size,
                    Size::new(S::from_f64(width), S::from_f64(height))
                );
                assert_eq!(
                    geometry.padding,
                    Edges {
                        top: S::from_f64(top),
                        bottom: S::from_f64(bottom),
                        left: S::ZERO,
                        right: S::ZERO
                    }
                );
                assert_eq!(geometry.border, Edges::all(S::ZERO));
                assert_eq!(geometry.margin, Edges::all(S::ZERO));
            }
        }
        minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 10.0), (100.0, 30.0));
        minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
    }
}
scalars!(
    maximum_progress_sliced_padding_converts_original_box_cap_once,
    maximum_progress_sliced_padding_converts_original_box_cap_once_f32,
    maximum_progress_sliced_padding_converts_original_box_cap_once_f64
);

fn maximum_progress_smaller_maximum_keeps_minimum_precedence<S: LayoutScalar>() {
    let mut root = minimum_progress_input::<S>(90.0);
    root.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(70.0)).unwrap());
    let tree = Tree::new(
        vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
        vec![vec![1, 2], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    let mut sink = Sink::new(3);
    sink.authorize(0, &whole_request);
    let controls = [
        compute_layout(&tree, 0, root_request(100.0))
            .expect("ordinary minimum wins maximum control"),
        compute(&tree, 0, &whole_request).expect("whole-fit minimum wins maximum control"),
    ];
    for control in controls {
        for output in [
            single(&control, 0).unrounded_output(),
            single(&control, 0).final_output(),
        ] {
            assert_packet(output, (0.0, 0.0), (100.0, 90.0));
        }
        for (node, y) in [(1, 0.0), (2, 30.0)] {
            for output in [
                single(&control, node).unrounded_output(),
                single(&control, node).final_output(),
            ] {
                assert_packet(output, (0.0, y), (100.0, 30.0));
            }
        }
    }
    let request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &request);
    let batch =
        compute(&tree, 0, &request).expect("minimum precedence survives maximum continuation");
    assert_eq!(batch.fragmentainers().len(), 2);
    assert_eq!(batch.node_layouts().count(), 3);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    // The effective minimum 90 wins max 70. The first 50 of break fill leaves 40.
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 40.0));
    for root in roots {
        assert_eq!(root.unrounded().location, Point::ZERO);
        assert_eq!(root.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_smaller_maximum_keeps_minimum_precedence,
    maximum_progress_smaller_maximum_keeps_minimum_precedence_f32,
    maximum_progress_smaller_maximum_keeps_minimum_precedence_f64
);

fn maximum_progress_cap<S: LayoutScalar>(cap: f64) -> NodeInputOf<S> {
    let mut value = block(None, None);
    value.max_size.height = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(cap)).unwrap());
    value
}

fn maximum_progress_overflow_request<S: LayoutScalar>(
    nonzero_origins: bool,
) -> LayoutBoundRequestOf<S> {
    let mut frames = vec![
        frame(100.0, 50.0, true),
        frame(60.0, 40.0, false),
        frame(80.0, 60.0, false),
    ];
    if nonzero_origins {
        for (value, (x, y)) in frames
            .iter_mut()
            .zip([(7.0, 11.0), (13.0, 17.0), (19.0, 23.0)])
        {
            value.origin = Point::new(S::from_f64(x), S::from_f64(y));
        }
    }
    fragmented(LayoutSourceBasis::fresh(), frames)
}

fn maximum_progress_overflow_geometry<S: LayoutScalar>(
    value: &FragmentOutputOf<usize, S>,
    width: f64,
    height: f64,
) {
    assert!(matches!(
        value.payload(),
        FragmentPayloadOf::OverflowOnly(_)
    ));
    assert!(value.principal_box().is_none());
    let pair = value
        .overflow_only()
        .expect("overflow carrier has no principal box");
    for geometry in [pair.unrounded(), pair.final_geometry()] {
        assert_eq!(geometry.origin, Point::ZERO);
        assert_eq!(geometry.content_bounds.origin(), Point::ZERO);
        assert_eq!(
            geometry.content_bounds.size(),
            Size::new(S::from_f64(width), S::from_f64(height))
        );
    }
}

fn maximum_progress_children<S: LayoutScalar>(
    batch: &Batch<S>,
    parent: &LayoutFragmentId,
    expected: &[&LayoutFragmentId],
) {
    let actual = batch
        .child_fragments(parent)
        .unwrap()
        .map(|value| value.id())
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), expected.len());
    for id in expected {
        assert!(actual.contains(id));
    }
    for value in batch.child_fragments(parent).unwrap() {
        assert_eq!(
            value.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(parent.clone()))
        );
        assert_eq!(
            value.fragmentainer(),
            batch.fragment(parent).unwrap().fragmentainer()
        );
    }
}

fn maximum_progress_unique_fragments<S: LayoutScalar>(batch: &Batch<S>) {
    let ids = batch
        .fragments()
        .map(|value| value.id())
        .collect::<Vec<_>>();
    for (index, id) in ids.iter().enumerate() {
        assert!(!ids[..index].contains(id));
    }
    for value in batch.fragments() {
        assert_eq!(batch.fragment(value.id()).unwrap().id(), value.id());
        if let Some(frame) = value.fragmentainer() {
            assert_eq!(batch.fragmentainer(frame).unwrap().id(), frame);
        }
    }
}

fn maximum_progress_exhausted_root_keeps_carriers_and_replaces_them<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            maximum_progress_cap::<S>(20.0),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
        ],
        vec![vec![1, 2, 3], vec![], vec![], vec![]],
    );
    let request = maximum_progress_overflow_request(false);
    let mut sink = Sink::new(4);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("exhausted root continues descendant overflow without another principal box");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)], &[(2, 3)]]);
    assert_fragment(&roots[0], (100.0, 20.0));
    assert!(roots[0].overflow_only().is_none());
    for (index, width) in [(0, 100.0), (1, 60.0), (2, 80.0)] {
        if index > 0 {
            maximum_progress_overflow_geometry(&roots[index], width, 30.0);
        }
        minimum_progress_child(
            &batch,
            index + 1,
            index,
            roots[index].id(),
            (0.0, 0.0),
            (width, 30.0),
        );
        maximum_progress_children(&batch, roots[index].id(), &[single(&batch, index + 1).id()]);
    }
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
    assert!(matches!(
        sink.state.geometry[0],
        Some(Visible::Fragments(_))
    ));
    let other = compute(&tree, 0, &request).unwrap();
    assert!(matches!(
        other.fragment(roots[1].id()),
        Err(LayoutHandleError::WrongResult)
    ));
    assert!(matches!(
        batch.fragment(fragments(&other, 0)[1].id()),
        Err(LayoutHandleError::WrongResult)
    ));
    assert!(matches!(
        other.child_fragments(roots[1].id()),
        Err(LayoutHandleError::WrongResult)
    ));
    assert_eq!(batch.fragment(roots[1].id()).unwrap().id(), roots[1].id());
    let continuous =
        LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), root_request(100.0));
    sink.authorize(0, &continuous);
    let replacement = compute(&tree, 0, &continuous).unwrap();
    replacement.apply_to(&mut sink).unwrap();
    assert!(matches!(sink.state.geometry[0], Some(Visible::Single(..))));
    assert!(matches!(
        replacement.fragment(roots[1].id()),
        Err(LayoutHandleError::WrongResult)
    ));
    assert_eq!(batch.fragment(roots[2].id()).unwrap().id(), roots[2].id());
    for node in 0..4 {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&replacement, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_exhausted_root_keeps_carriers_and_replaces_them,
    maximum_progress_exhausted_root_keeps_carriers_and_replaces_them_f32,
    maximum_progress_exhausted_root_keeps_carriers_and_replaces_them_f64
);

fn maximum_progress_two_exhausted_ancestors_keep_local_carrier_ancestry<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            maximum_progress_cap::<S>(5.0),
            maximum_progress_cap(10.0),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
        ],
        vec![vec![1], vec![2, 3, 4], vec![], vec![], vec![]],
    );
    let request = maximum_progress_overflow_request(true);
    let mut sink = Sink::new(5);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("both exhausted ancestors retain target-frame overflow custody");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1)], &[(0, 1)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[(1, 3)], &[(2, 4)]]);
    assert_fragment(&roots[0], (100.0, 5.0));
    assert_fragment(&wrappers[0], (100.0, 10.0));
    for (index, width, x, y) in [
        (0, 100.0, 7.0, 11.0),
        (1, 60.0, 13.0, 17.0),
        (2, 80.0, 19.0, 23.0),
    ] {
        assert_eq!(
            batch.fragmentainers()[index].origin(),
            Point::new(S::from_f64(x), S::from_f64(y))
        );
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        if index == 0 {
            assert_eq!(roots[index].unrounded().location, Point::ZERO);
            assert_eq!(roots[index].final_geometry().location, Point::ZERO);
            assert_eq!(wrappers[index].unrounded().location, Point::ZERO);
            assert_eq!(wrappers[index].final_geometry().location, Point::ZERO);
        } else {
            maximum_progress_overflow_geometry(&roots[index], width, 30.0);
            maximum_progress_overflow_geometry(&wrappers[index], width, 30.0);
        }
        minimum_progress_child(
            &batch,
            index + 2,
            index,
            wrappers[index].id(),
            (0.0, 0.0),
            (width, 30.0),
        );
        maximum_progress_children(&batch, roots[index].id(), &[wrappers[index].id()]);
        maximum_progress_children(
            &batch,
            wrappers[index].id(),
            &[single(&batch, index + 2).id()],
        );
    }
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_two_exhausted_ancestors_keep_local_carrier_ancestry,
    maximum_progress_two_exhausted_ancestors_keep_local_carrier_ancestry_f32,
    maximum_progress_two_exhausted_ancestors_keep_local_carrier_ancestry_f64
);

fn maximum_progress_parallel_overflow_force_keeps_following_sibling_endpoint<S: LayoutScalar>() {
    let results = [false, true].map(|forced| {
        let mut middle = block::<S>(None, Some(30.0));
        if forced {
            middle.break_after = FragmentationBreak::Page;
        }
        let tree = Tree::new(
            vec![
                block(None, None),
                maximum_progress_cap(20.0),
                block(None, Some(30.0)),
                middle,
                block(None, Some(30.0)),
                block(None, Some(10.0)),
            ],
            vec![vec![1, 5], vec![2, 3, 4], vec![], vec![], vec![], vec![]],
        );
        let control = compute_layout(&tree, 0, root_request(100.0)).unwrap();
        for (node, y, height) in [(0, 0.0, 30.0), (1, 0.0, 20.0), (5, 20.0, 10.0)] {
            for output in [
                single(&control, node).unrounded_output(),
                single(&control, node).final_output(),
            ] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
        let request = fragmented(
            LayoutSourceBasis::fresh(),
            vec![
                frame(100.0, 50.0, true),
                frame(60.0, 80.0, false),
                frame(80.0, 60.0, false),
            ],
        );
        let mut sink = Sink::new(6);
        sink.authorize(0, &request);
        (forced, compute(&tree, 0, &request), sink)
    });
    assert!(
        results.iter().all(|(_, result, _)| result.is_ok()),
        "parallel flow outcomes: {:?}",
        results
            .each_ref()
            .map(|(forced, result, _)| (*forced, result.as_ref().map(|_| ())))
    );
    for (forced, result, mut sink) in results {
        let batch = result.unwrap();
        let root_children: &[&[(usize, usize)]] = if forced {
            &[&[(0, 1), (1, 5)], &[(0, 1)], &[(0, 1)]]
        } else {
            &[&[(0, 1), (1, 5)], &[(0, 1)]]
        };
        let wrapper_children: &[&[(usize, usize)]] = if forced {
            &[&[(0, 2)], &[(1, 3)], &[(2, 4)]]
        } else {
            &[&[(0, 2)], &[(1, 3), (2, 4)]]
        };
        let roots = minimum_progress_chain(&batch, 0, root_children);
        let wrappers = minimum_progress_chain(&batch, 1, wrapper_children);
        assert_fragment(&roots[0], (100.0, 30.0));
        assert_fragment(&wrappers[0], (100.0, 20.0));
        for index in 0..roots.len() {
            assert_eq!(
                wrappers[index].coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    roots[index].id().clone()
                ))
            );
            if index > 0 {
                let width = if index == 1 { 60.0 } else { 80.0 };
                let height = if forced { 30.0 } else { 60.0 };
                maximum_progress_overflow_geometry(&roots[index], width, height);
                maximum_progress_overflow_geometry(&wrappers[index], width, height);
            }
            let sibling = single(&batch, 5);
            let expected = if index == 0 {
                vec![wrappers[index].id(), sibling.id()]
            } else {
                vec![wrappers[index].id()]
            };
            maximum_progress_children(&batch, roots[index].id(), &expected);
        }
        minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 30.0));
        minimum_progress_child(&batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (60.0, 30.0));
        minimum_progress_child(
            &batch,
            4,
            2,
            wrappers[if forced { 2 } else { 1 }].id(),
            (0.0, if forced { 0.0 } else { 30.0 }),
            (if forced { 80.0 } else { 60.0 }, 30.0),
        );
        minimum_progress_child(&batch, 5, 1, roots[0].id(), (0.0, 20.0), (100.0, 10.0));
        maximum_progress_children(&batch, wrappers[0].id(), &[single(&batch, 2).id()]);
        if forced {
            maximum_progress_children(&batch, wrappers[1].id(), &[single(&batch, 3).id()]);
            maximum_progress_children(&batch, wrappers[2].id(), &[single(&batch, 4).id()]);
        } else {
            maximum_progress_children(
                &batch,
                wrappers[1].id(),
                &[single(&batch, 3).id(), single(&batch, 4).id()],
            );
        }
        maximum_progress_unique_fragments(&batch);
        batch.apply_to(&mut sink).unwrap();
    }
}
scalars!(
    maximum_progress_parallel_overflow_force_keeps_following_sibling_endpoint,
    maximum_progress_parallel_overflow_force_keeps_following_sibling_endpoint_f32,
    maximum_progress_parallel_overflow_force_keeps_following_sibling_endpoint_f64
);

fn maximum_progress_zero_principal_is_distinct_from_later_overflow<S: LayoutScalar>() {
    let tree = Tree::<S>::new(
        vec![
            block(None, None),
            maximum_progress_cap(0.0),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(60.0, 40.0, false)],
    );
    let mut sink = Sink::new(5);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("actual zero-height owner still carries independent descendant overflow");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1), (1, 4)], &[(0, 1)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[(1, 3)]]);
    assert_fragment(&roots[0], (100.0, 10.0));
    assert_fragment(&wrappers[0], (100.0, 0.0));
    assert!(matches!(
        wrappers[0].payload(),
        FragmentPayloadOf::ActualBox(_)
    ));
    assert!(wrappers[0].overflow_only().is_none());
    for index in [0, 1] {
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        if index == 1 {
            maximum_progress_overflow_geometry(&roots[index], 60.0, 30.0);
            maximum_progress_overflow_geometry(&wrappers[index], 60.0, 30.0);
        }
        minimum_progress_child(
            &batch,
            index + 2,
            index,
            wrappers[index].id(),
            (0.0, 0.0),
            (if index == 0 { 100.0 } else { 60.0 }, 30.0),
        );
        maximum_progress_children(
            &batch,
            wrappers[index].id(),
            &[single(&batch, index + 2).id()],
        );
    }
    minimum_progress_child(&batch, 4, 1, roots[0].id(), (0.0, 0.0), (100.0, 10.0));
    maximum_progress_children(
        &batch,
        roots[0].id(),
        &[wrappers[0].id(), single(&batch, 4).id()],
    );
    maximum_progress_children(&batch, roots[1].id(), &[wrappers[1].id()]);
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_zero_principal_is_distinct_from_later_overflow,
    maximum_progress_zero_principal_is_distinct_from_later_overflow_f32,
    maximum_progress_zero_principal_is_distinct_from_later_overflow_f64
);

fn maximum_progress_late_missing_frame_retains_real_host_custody<S: LayoutScalar>() {
    let mut tree = Tree::new(
        vec![
            maximum_progress_cap::<S>(20.0),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
            block(None, Some(30.0)),
        ],
        vec![vec![1, 2, 3], vec![], vec![], vec![]],
    );
    let mut sink = Sink::new(4);
    let old = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    old.apply_to(&mut sink).unwrap();
    sink.copy_cache_to(&mut tree);
    assert!(tree.caches.iter().any(|cache| !cache.is_empty()));
    assert!(tree.custody.iter().all(Option::is_some));
    let caches = tree.caches.clone();
    let custody = tree.custody.clone();
    tree.reset_observations();
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 50.0, true), frame(60.0, 40.0, false)],
    );
    sink.authorize(0, &request);
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::BlockFragmentation(
                BlockFragmentationFailure::MissingFragmentainer { index: 2 }
            )
        ),
        "actual late failure: {error:?}"
    );
    assert_eq!(tree.caches, caches);
    assert_eq!(tree.custody, custody);
    assert_eq!(tree.cache_calls.get(), 0);
    assert_eq!(tree.custody_calls.get(), 0);
    for node in 0..4 {
        let value = single(&old, node);
        assert_eq!(old.fragment(value.id()).unwrap().id(), value.id());
        assert_eq!(tree.custody[node], Some(value.unrounded_entry()));
    }
}
scalars!(
    maximum_progress_late_missing_frame_retains_real_host_custody,
    maximum_progress_late_missing_frame_retains_real_host_custody_f32,
    maximum_progress_late_missing_frame_retains_real_host_custody_f64
);

fn maximum_progress_axis_request<S: LayoutScalar>(
    vertical: bool,
    whole: bool,
) -> LayoutBoundRequestOf<S> {
    maximum_progress_axis_request_origin(vertical, whole, Point::ZERO)
}
fn maximum_progress_axis_request_origin<S: LayoutScalar>(
    vertical: bool,
    whole: bool,
    origin: Point<S>,
) -> LayoutBoundRequestOf<S> {
    let axes = FlowAxes::new(
        if vertical {
            WritingMode::VerticalRl
        } else {
            WritingMode::HorizontalTb
        },
        if vertical {
            Direction::Ltr
        } else {
            Direction::Rtl
        },
    );
    let physical = |inline, block| {
        if vertical {
            (block, inline)
        } else {
            (inline, block)
        }
    };
    let logical_frames = if whole {
        vec![(100.0, 200.0)]
    } else {
        vec![(100.0, 50.0), (60.0, 40.0), (80.0, 60.0)]
    };
    let (width, height) = physical(logical_frames[0].0, logical_frames[0].1);
    let sizing = native_request::sizing(
        Size::new(Some(S::from_f64(width)), Some(S::from_f64(height))),
        Size::splat(SizingOperation::Normal),
    );
    let root = LayoutRootRequestOf::standalone(
        sizing,
        axes,
        sizing.orthogonal_fallback().initial_containing_block(),
    );
    let frames = logical_frames
        .into_iter()
        .enumerate()
        .map(|(index, (inline, block))| {
            let (width, height) = physical(inline, block);
            let mut result = frame(width, height, index == 0);
            result.origin = origin;
            result
        })
        .collect();
    LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        root,
        FragmentationKind::Page,
        axes,
        frames,
    )
    .unwrap()
}

fn maximum_progress_axis_carrier_case<S: LayoutScalar>(vertical: bool) {
    let mut root = if vertical {
        block(None, Some(40.0))
    } else {
        block(Some(40.0), None)
    };
    let maximum = MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
    if vertical {
        root.max_size.width = maximum;
    } else {
        root.max_size.height = maximum;
    }
    let child = if vertical {
        block(Some(30.0), None)
    } else {
        block(None, Some(30.0))
    };
    let mut tree = Tree::new(
        vec![root, child.clone(), child.clone(), child],
        vec![vec![1, 2, 3], vec![], vec![], vec![]],
    );
    for input in &mut tree.inputs {
        input.writing_mode = if vertical {
            WritingMode::VerticalRl
        } else {
            WritingMode::HorizontalTb
        };
        input.direction = if vertical {
            Direction::Ltr
        } else {
            Direction::Rtl
        };
    }
    let whole_request = maximum_progress_axis_request(vertical, true);
    let mut sink = Sink::new(4);
    sink.authorize(0, &whole_request);
    let whole =
        compute(&tree, 0, &whole_request).expect("whole-fit source-owned inline-size axis control");
    let whole_root = single(&whole, 0);
    let root_size = if vertical { (20.0, 40.0) } else { (40.0, 20.0) };
    let child_size = if vertical { (30.0, 40.0) } else { (40.0, 30.0) };
    for output in [whole_root.unrounded_output(), whole_root.final_output()] {
        assert_packet(
            output,
            (if vertical { 180.0 } else { 60.0 }, 0.0),
            root_size,
        );
    }
    assert_eq!(whole.fragmentainers().len(), 1);
    assert_eq!(
        whole_root.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            whole.fragmentainers()[0].id().clone()
        ))
    );
    for (index, offset) in [0.0, 30.0, 60.0].into_iter().enumerate() {
        let location = if vertical {
            (-10.0 - offset, 0.0)
        } else {
            (0.0, offset)
        };
        minimum_progress_child(
            &whole,
            index + 1,
            index,
            whole_root.id(),
            location,
            child_size,
        );
    }
    let request = maximum_progress_axis_request(vertical, false);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("overflow carrier origin follows the actual inline and block axes");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)], &[(2, 3)]]);
    assert_fragment(&roots[0], root_size);
    for geometry in [roots[0].unrounded(), roots[0].final_geometry()] {
        assert_eq!(
            geometry.location,
            Point::new(S::from_f64(if vertical { 30.0 } else { 60.0 }), S::ZERO)
        );
    }
    // Break3 §5.5 positions the zero-edge reference at logical frame start.
    // Its source-owned inline context remains 40; it has no principal size.
    for (index, origin_x) in [
        (1, if vertical { 40.0 } else { 20.0 }),
        (2, if vertical { 60.0 } else { 40.0 }),
    ] {
        assert!(roots[index].principal_box().is_none());
        assert!(matches!(
            roots[index].payload(),
            FragmentPayloadOf::OverflowOnly(_)
        ));
        let pair = roots[index].overflow_only().unwrap();
        for geometry in [pair.unrounded(), pair.final_geometry()] {
            assert_eq!(geometry.origin, Point::new(S::from_f64(origin_x), S::ZERO));
            assert_eq!(
                geometry.content_bounds.origin(),
                Point::new(
                    if vertical {
                        S::from_f64(-30.0)
                    } else {
                        S::ZERO
                    },
                    S::ZERO
                )
            );
            assert_eq!(
                geometry.content_bounds.size(),
                Size::new(S::from_f64(child_size.0), S::from_f64(child_size.1))
            );
        }
    }
    for (index, root) in roots.iter().enumerate() {
        let x = if vertical {
            if index == 0 { -10.0 } else { -30.0 }
        } else {
            0.0
        };
        minimum_progress_child(&batch, index + 1, index, root.id(), (x, 0.0), child_size);
        maximum_progress_children(&batch, root.id(), &[single(&batch, index + 1).id()]);
    }
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
}

fn maximum_progress_rtl_carrier_origin_uses_source_inline_context<S: LayoutScalar>() {
    maximum_progress_axis_carrier_case::<S>(false);
}
scalars!(
    maximum_progress_rtl_carrier_origin_uses_source_inline_context,
    maximum_progress_rtl_carrier_origin_uses_source_inline_context_f32,
    maximum_progress_rtl_carrier_origin_uses_source_inline_context_f64
);

fn maximum_progress_vertical_carrier_keeps_signed_child_offsets<S: LayoutScalar>() {
    maximum_progress_axis_carrier_case::<S>(true);
}
scalars!(
    maximum_progress_vertical_carrier_keeps_signed_child_offsets,
    maximum_progress_vertical_carrier_keeps_signed_child_offsets_f32,
    maximum_progress_vertical_carrier_keeps_signed_child_offsets_f64
);

fn maximum_progress_original_slice_box<S: LayoutScalar>(
    batch: &Batch<S>,
    node: usize,
    location: (f64, f64),
    size: (f64, f64),
    top: f64,
    bottom: f64,
) {
    let value = single(batch, node);
    for output in [value.unrounded_output(), value.final_output()] {
        assert_eq!(
            output.location,
            Point::new(S::from_f64(location.0), S::from_f64(location.1))
        );
        assert_eq!(
            output.size,
            Size::new(S::from_f64(size.0), S::from_f64(size.1))
        );
        assert_eq!(
            output.padding,
            Edges {
                top: S::from_f64(top),
                bottom: S::from_f64(bottom),
                left: S::ZERO,
                right: S::ZERO
            }
        );
        assert_eq!(output.border, Edges::all(S::ZERO));
        assert_eq!(output.margin, Edges::all(S::ZERO));
    }
}

fn maximum_progress_original_slice_pending_end_survives_content_cap<S: LayoutScalar>() {
    let mut root = maximum_progress_cap::<S>(40.0);
    root.box_sizing = BoxSizing::ContentBox;
    let edge = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
    root.padding.top = edge;
    root.padding.bottom = edge;
    let tree = Tree::new(
        vec![root, block(None, Some(30.0)), block(None, Some(30.0))],
        vec![vec![1, 2], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(3);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit original Slice edge control"),
    ] {
        // The natural content 60 is capped at 40; original edges add 20 once.
        maximum_progress_original_slice_box(&control, 0, (0.0, 0.0), (100.0, 60.0), 10.0, 10.0);
        for (node, y) in [(1, 10.0), (2, 40.0)] {
            for output in [
                single(&control, node).unrounded_output(),
                single(&control, node).final_output(),
            ] {
                assert_packet(output, (0.0, y), (100.0, 30.0));
            }
        }
    }
    let request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("content exhaustion preserves the pending original closing edge");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    assert_eq!(batch.fragmentainers().len(), 2);
    // First-frame opening 10 plus accepted content fill 40 reaches the cap.
    // Closing 10 is still an owner obligation, independent of child occupancy 30.
    for (index, width, height, top, bottom) in
        [(0, 100.0, 50.0, 10.0, 0.0), (1, 60.0, 10.0, 0.0, 10.0)]
    {
        for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
            assert_eq!(geometry.location, Point::ZERO);
            assert_eq!(
                geometry.size,
                Size::new(S::from_f64(width), S::from_f64(height))
            );
            assert_eq!(
                geometry.padding,
                Edges {
                    top: S::from_f64(top),
                    bottom: S::from_f64(bottom),
                    left: S::ZERO,
                    right: S::ZERO
                }
            );
            assert_eq!(geometry.border, Edges::all(S::ZERO));
            assert_eq!(geometry.margin, Edges::all(S::ZERO));
        }
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 10.0), (100.0, 30.0));
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 30.0));
    maximum_progress_children(&batch, roots[0].id(), &[single(&batch, 1).id()]);
    maximum_progress_children(&batch, roots[1].id(), &[single(&batch, 2).id()]);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_original_slice_pending_end_survives_content_cap,
    maximum_progress_original_slice_pending_end_survives_content_cap_f32,
    maximum_progress_original_slice_pending_end_survives_content_cap_f64
);

fn maximum_progress_original_slice_grouped_end_defers_whole_natural_head<S: LayoutScalar>() {
    let mut wrapper = maximum_progress_cap::<S>(10.0);
    wrapper.box_sizing = BoxSizing::ContentBox;
    wrapper.padding.bottom = LengthOf::value(LengthPercentageOf::px(S::from_f64(40.0)).unwrap());
    let mut tree = Tree::new(
        vec![
            block(None, None),
            block(None, Some(30.0)),
            wrapper,
            block(None, None),
        ],
        vec![vec![1, 2], vec![], vec![3], vec![]],
    );
    tree.measurements[3] = Some(Measurement::Fixed(100.0, 5.0));
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(4);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit natural closing head control"),
    ] {
        maximum_progress_original_slice_box(&control, 0, (0.0, 0.0), (100.0, 75.0), 0.0, 0.0);
        maximum_progress_original_slice_box(&control, 2, (0.0, 30.0), (100.0, 45.0), 0.0, 40.0);
        for output in [
            single(&control, 3).unrounded_output(),
            single(&control, 3).final_output(),
        ] {
            assert_packet(output, (0.0, 0.0), (100.0, 5.0));
        }
    }
    let request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("grouped original end padding defers with its natural five-unit head");
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 45.0));
    maximum_progress_original_slice_box(&batch, 2, (0.0, 0.0), (60.0, 45.0), 0.0, 40.0);
    let wrapper = single(&batch, 2);
    assert_eq!(
        wrapper.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(roots[1].id().clone()))
    );
    for output in [wrapper.unrounded_output(), wrapper.final_output()] {
        assert_eq!(output.source_index, SourceIndex::new(1));
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 30.0));
    // The accepted child belongs to the new 60-wide owner placement, rather than
    // retaining the 100-wide receipt from the abandoned first-frame origin.
    minimum_progress_child(&batch, 3, 0, wrapper.id(), (0.0, 0.0), (60.0, 5.0));
    maximum_progress_children(&batch, wrapper.id(), &[single(&batch, 3).id()]);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_original_slice_grouped_end_defers_whole_natural_head,
    maximum_progress_original_slice_grouped_end_defers_whole_natural_head_f32,
    maximum_progress_original_slice_grouped_end_defers_whole_natural_head_f64
);

fn maximum_progress_original_slice_opening_is_part_of_fresh_emergency_head<S: LayoutScalar>() {
    let mut root = maximum_progress_cap::<S>(1.0);
    root.box_sizing = BoxSizing::ContentBox;
    root.padding.top = LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
    let tree = Tree::new(vec![root, block(None, Some(150.0))], vec![vec![1], vec![]]);
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(2);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit opening padding control"),
    ] {
        maximum_progress_original_slice_box(&control, 0, (0.0, 0.0), (100.0, 11.0), 10.0, 0.0);
        for output in [
            single(&control, 1).unrounded_output(),
            single(&control, 1).final_output(),
        ] {
            assert_packet(output, (0.0, 10.0), (100.0, 150.0));
        }
    }
    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 50.0, true)]);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("a fresh frame admits the complete opening-plus-unsliceable head once");
    assert_eq!(batch.fragmentainers().len(), 1);
    assert_eq!(batch.node_layouts().count(), 2);
    maximum_progress_original_slice_box(&batch, 0, (0.0, 0.0), (100.0, 11.0), 10.0, 0.0);
    let root = single(&batch, 0);
    assert_eq!(
        root.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            batch.fragmentainers()[0].id().clone()
        ))
    );
    // Actual child occupancy 150 begins after opening 10; neither is pre-existing
    // outside occupancy. Principal content remains capped at 1, giving box 11.
    minimum_progress_child(&batch, 1, 0, root.id(), (0.0, 10.0), (100.0, 150.0));
    maximum_progress_children(&batch, root.id(), &[single(&batch, 1).id()]);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_original_slice_opening_is_part_of_fresh_emergency_head,
    maximum_progress_original_slice_opening_is_part_of_fresh_emergency_head_f32,
    maximum_progress_original_slice_opening_is_part_of_fresh_emergency_head_f64
);

fn maximum_progress_original_slice_short_principal_keeps_large_child_occupancy<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            block::<S>(None, None),
            maximum_progress_cap(1.0),
            block(None, Some(150.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 3], vec![2], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(4);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit independent occupancy control"),
    ] {
        for (node, y, height) in [
            (0, 0.0, 11.0),
            (1, 0.0, 1.0),
            (2, 0.0, 150.0),
            (3, 1.0, 10.0),
        ] {
            for output in [
                single(&control, node).unrounded_output(),
                single(&control, node).final_output(),
            ] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
    }
    let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 50.0, true)]);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("principal endpoint and unsliceable descendant occupancy stay independent");
    assert_eq!(batch.fragmentainers().len(), 1);
    assert_eq!(batch.node_layouts().count(), 4);
    maximum_progress_original_slice_box(&batch, 0, (0.0, 0.0), (100.0, 11.0), 0.0, 0.0);
    maximum_progress_original_slice_box(&batch, 1, (0.0, 0.0), (100.0, 1.0), 0.0, 0.0);
    let root = single(&batch, 0);
    let wrapper = single(&batch, 1);
    assert_eq!(
        root.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            batch.fragmentainers()[0].id().clone()
        ))
    );
    assert_eq!(
        wrapper.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragment(root.id().clone()))
    );
    minimum_progress_child(&batch, 2, 0, wrapper.id(), (0.0, 0.0), (100.0, 150.0));
    minimum_progress_child(&batch, 3, 1, root.id(), (0.0, 1.0), (100.0, 10.0));
    maximum_progress_children(&batch, root.id(), &[wrapper.id(), single(&batch, 3).id()]);
    maximum_progress_children(&batch, wrapper.id(), &[single(&batch, 2).id()]);
    batch.apply_to(&mut sink).unwrap();
}
scalars!(
    maximum_progress_original_slice_short_principal_keeps_large_child_occupancy,
    maximum_progress_original_slice_short_principal_keeps_large_child_occupancy_f32,
    maximum_progress_original_slice_short_principal_keeps_large_child_occupancy_f64
);

fn maximum_progress_interior_force_before_cap_advances_principal_flow<S: LayoutScalar>() {
    let mut second = block::<S>(None, Some(30.0));
    second.break_before = FragmentationBreak::Page;
    let tree = Tree::new(
        vec![
            block(None, None),
            maximum_progress_cap(200.0),
            block(None, Some(30.0)),
            second,
            block(None, Some(10.0)),
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    let continuous = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    for (node, y, height) in [
        (0, 0.0, 70.0),
        (1, 0.0, 60.0),
        (2, 0.0, 30.0),
        (3, 30.0, 30.0),
        (4, 60.0, 10.0),
    ] {
        let value = single(&continuous, node);
        for output in [value.unrounded_output(), value.final_output()] {
            assert_packet(output, (0.0, y), (100.0, height));
        }
    }
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    let mut sink = Sink::new(5);
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("interior Page force advances principal flow before its cap is exhausted");
    assert_eq!(batch.fragmentainers().len(), 2);
    assert_eq!(batch.node_layouts().count(), 5);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(0, 1), (1, 4)]]);
    let wrappers = minimum_progress_chain(&batch, 1, &[&[(0, 2)], &[(1, 3)]]);
    // Break3 §4.3 moves the second child at its interior source boundary.
    // Section 5.3 fills the first principal fragment to 100; total 130 remains
    // below max 200, so the sibling follows the terminal principal 30 at y30.
    assert_fragment(&roots[0], (100.0, 100.0));
    assert_fragment(&roots[1], (60.0, 40.0));
    for (index, size) in [(100.0, 100.0), (60.0, 30.0)].into_iter().enumerate() {
        assert!(matches!(
            wrappers[index].payload(),
            FragmentPayloadOf::ActualBox(_)
        ));
        assert!(wrappers[index].overflow_only().is_none());
        assert_fragment(&wrappers[index], size);
        assert_eq!(
            wrappers[index].coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragment(
                roots[index].id().clone()
            ))
        );
        for geometry in [
            roots[index].unrounded(),
            roots[index].final_geometry(),
            wrappers[index].unrounded(),
            wrappers[index].final_geometry(),
        ] {
            assert_eq!(geometry.location, Point::ZERO);
        }
    }
    minimum_progress_child(&batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (60.0, 30.0));
    minimum_progress_child(&batch, 4, 1, roots[1].id(), (0.0, 30.0), (60.0, 10.0));
    maximum_progress_children(&batch, roots[0].id(), &[wrappers[0].id()]);
    maximum_progress_children(
        &batch,
        roots[1].id(),
        &[wrappers[1].id(), single(&batch, 4).id()],
    );
    maximum_progress_children(&batch, wrappers[0].id(), &[single(&batch, 2).id()]);
    maximum_progress_children(&batch, wrappers[1].id(), &[single(&batch, 3).id()]);
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
    for node in [2, 3, 4] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_interior_force_before_cap_advances_principal_flow,
    maximum_progress_interior_force_before_cap_advances_principal_flow_f32,
    maximum_progress_interior_force_before_cap_advances_principal_flow_f64
);

fn maximum_progress_initial_child_deferral_retains_unaccepted_parent_prefix<S: LayoutScalar>() {
    let tree = Tree::new(
        vec![
            block::<S>(None, None),
            block(None, Some(30.0)),
            maximum_progress_cap(1.0),
            block(None, Some(150.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 2, 4], vec![], vec![3], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(5);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit initial child deferral control"),
    ] {
        for (node, y, height) in [
            (0, 0.0, 41.0),
            (1, 0.0, 30.0),
            (2, 30.0, 1.0),
            (3, 0.0, 150.0),
            (4, 31.0, 10.0),
        ] {
            let value = single(&control, node);
            for output in [value.unrounded_output(), value.final_output()] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
    }
    let request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &request);
    // The child cannot start after the preceding 30 in a 50-unit frame. Its
    // initial deferral moves the owner before any owner occurrence is accepted;
    // principal size 1 must not conceal the child's 150-unit unsliceable head.
    let batch = compute(&tree, 0, &request)
        .expect("initial capped-child deferral preserves the unaccepted preceding prefix");
    assert_eq!(batch.fragmentainers().len(), 2);
    assert_eq!(batch.node_layouts().count(), 5);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2), (2, 4)]]);
    assert_fragment(&roots[0], (100.0, 50.0));
    assert_fragment(&roots[1], (60.0, 11.0));
    for root in roots {
        assert_eq!(root.unrounded().location, Point::ZERO);
        assert_eq!(root.final_geometry().location, Point::ZERO);
    }
    minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 30.0));
    minimum_progress_child(&batch, 2, 1, roots[1].id(), (0.0, 0.0), (60.0, 1.0));
    let wrapper = single(&batch, 2);
    minimum_progress_child(&batch, 3, 0, wrapper.id(), (0.0, 0.0), (60.0, 150.0));
    minimum_progress_child(&batch, 4, 2, roots[1].id(), (0.0, 1.0), (60.0, 10.0));
    // Exact occurrence relationships exclude an abandoned frame 0 owner/carrier
    // and retain the preceding box once while the child occupies frame 1 once.
    maximum_progress_children(&batch, roots[0].id(), &[single(&batch, 1).id()]);
    maximum_progress_children(
        &batch,
        roots[1].id(),
        &[wrapper.id(), single(&batch, 4).id()],
    );
    maximum_progress_children(&batch, wrapper.id(), &[single(&batch, 3).id()]);
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
    for node in [1, 2, 3, 4] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_initial_child_deferral_retains_unaccepted_parent_prefix,
    maximum_progress_initial_child_deferral_retains_unaccepted_parent_prefix_f32,
    maximum_progress_initial_child_deferral_retains_unaccepted_parent_prefix_f64
);

fn maximum_progress_later_child_deferral_after_unsliceable_principal_head<S: LayoutScalar>() {
    let mut tree = Tree::new(
        vec![
            block::<S>(None, None),
            maximum_progress_cap(200.0),
            block(None, Some(150.0)),
            block(None, Some(30.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(5);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit capped unsliceable-head control"),
    ] {
        for (node, y, height) in [
            (0, 0.0, 190.0),
            (1, 0.0, 180.0),
            (2, 0.0, 150.0),
            (3, 150.0, 30.0),
            (4, 180.0, 10.0),
        ] {
            let value = single(&control, node);
            for output in [value.unrounded_output(), value.final_output()] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
    }
    let verify = |batch: &Batch<S>| {
        assert_eq!(batch.fragmentainers().len(), 2);
        assert_eq!(batch.node_layouts().count(), 5);
        let roots = minimum_progress_chain(batch, 0, &[&[(0, 1)], &[(0, 1), (1, 4)]]);
        let wrappers = minimum_progress_chain(batch, 1, &[&[(0, 2)], &[(1, 3)]]);
        assert_fragment(&roots[0], (100.0, 150.0));
        assert_fragment(&roots[1], (60.0, 40.0));
        for (index, size) in [(100.0, 150.0), (60.0, 30.0)].into_iter().enumerate() {
            assert_fragment(&wrappers[index], size);
            assert!(wrappers[index].overflow_only().is_none());
            assert_eq!(
                wrappers[index].coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    roots[index].id().clone()
                ))
            );
            for geometry in [
                roots[index].unrounded(),
                roots[index].final_geometry(),
                wrappers[index].unrounded(),
                wrappers[index].final_geometry(),
            ] {
                assert_eq!(geometry.location, Point::ZERO);
            }
        }
        minimum_progress_child(batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 150.0));
        minimum_progress_child(batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (60.0, 30.0));
        minimum_progress_child(batch, 4, 1, roots[1].id(), (0.0, 30.0), (60.0, 10.0));
        maximum_progress_children(batch, roots[0].id(), &[wrappers[0].id()]);
        maximum_progress_children(
            batch,
            roots[1].id(),
            &[wrappers[1].id(), single(batch, 4).id()],
        );
        maximum_progress_children(batch, wrappers[0].id(), &[single(batch, 2).id()]);
        maximum_progress_children(batch, wrappers[1].id(), &[single(batch, 3).id()]);
        maximum_progress_unique_fragments(batch);
    };
    // Establish the ordinary native policy independently: the first atomic head
    // overflows whole 150, and only its later sibling defers to the second frame.
    tree.inputs[1].max_size.height = block::<S>(None, None).max_size.height;
    let uncapped_request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    sink.authorize(0, &uncapped_request);
    let uncapped = compute(&tree, 0, &uncapped_request)
        .expect("uncapped native unsliceable-head policy control");
    verify(&uncapped);

    tree.inputs[1].max_size.height = maximum_progress_cap::<S>(200.0).max_size.height;
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![frame(100.0, 100.0, true), frame(60.0, 80.0, false)],
    );
    sink.authorize(0, &request);
    // Actual principal progress 150 +30 stays below max 200. The child Deferred
    // must preserve that accepted 150 head without splitting it or forcing entry.
    let batch = compute(&tree, 0, &request)
        .expect("later before-cap child deferral retains the accepted unsliceable head");
    verify(&batch);
    batch.apply_to(&mut sink).unwrap();
    for node in [2, 3, 4] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_later_child_deferral_after_unsliceable_principal_head,
    maximum_progress_later_child_deferral_after_unsliceable_principal_head_f32,
    maximum_progress_later_child_deferral_after_unsliceable_principal_head_f64
);

fn maximum_progress_exact_fill_preserves_sibling_class_boundary<S: LayoutScalar>() {
    let mut tree = Tree::new(
        vec![
            block::<S>(None, None),
            maximum_progress_cap(70.0),
            block(None, Some(50.0)),
            block(None, Some(20.0)),
            block(None, Some(10.0)),
        ],
        vec![vec![1, 4], vec![2, 3], vec![], vec![], vec![]],
    );
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
    let mut sink = Sink::new(5);
    sink.authorize(0, &whole_request);
    for control in [
        compute_layout(&tree, 0, root_request(100.0)).unwrap(),
        compute(&tree, 0, &whole_request).expect("whole-fit exact-fill maximum control"),
    ] {
        for (node, y, height) in [
            (0, 0.0, 80.0),
            (1, 0.0, 70.0),
            (2, 0.0, 50.0),
            (3, 50.0, 20.0),
            (4, 70.0, 10.0),
        ] {
            let value = single(&control, node);
            for output in [value.unrounded_output(), value.final_output()] {
                assert_packet(output, (0.0, y), (100.0, height));
            }
        }
    }
    let verify = |batch: &Batch<S>| {
        assert_eq!(batch.fragmentainers().len(), 2);
        assert_eq!(batch.node_layouts().count(), 5);
        let roots = minimum_progress_chain(batch, 0, &[&[(0, 1)], &[(0, 1), (1, 4)]]);
        let wrappers = minimum_progress_chain(batch, 1, &[&[(0, 2)], &[(1, 3)]]);
        assert_fragment(&roots[0], (100.0, 50.0));
        assert_fragment(&roots[1], (60.0, 30.0));
        for (index, size) in [(100.0, 50.0), (60.0, 20.0)].into_iter().enumerate() {
            assert_fragment(&wrappers[index], size);
            assert!(wrappers[index].overflow_only().is_none());
            assert_eq!(
                wrappers[index].coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    roots[index].id().clone()
                ))
            );
            for geometry in [
                roots[index].unrounded(),
                roots[index].final_geometry(),
                wrappers[index].unrounded(),
                wrappers[index].final_geometry(),
            ] {
                assert_eq!(geometry.location, Point::ZERO);
            }
        }
        minimum_progress_child(batch, 2, 0, wrappers[0].id(), (0.0, 0.0), (100.0, 50.0));
        minimum_progress_child(batch, 3, 1, wrappers[1].id(), (0.0, 0.0), (60.0, 20.0));
        minimum_progress_child(batch, 4, 1, roots[1].id(), (0.0, 20.0), (60.0, 10.0));
        maximum_progress_children(batch, roots[0].id(), &[wrappers[0].id()]);
        maximum_progress_children(
            batch,
            roots[1].id(),
            &[wrappers[1].id(), single(batch, 4).id()],
        );
        maximum_progress_children(batch, wrappers[0].id(), &[single(batch, 2).id()]);
        maximum_progress_children(batch, wrappers[1].id(), &[single(batch, 3).id()]);
        maximum_progress_unique_fragments(batch);
    };
    // The first child exactly fills 50; the next sibling supplies a Class A
    // source boundary without any positive fill gap (Break3 §4.1).
    tree.inputs[1].max_size.height = block::<S>(None, None).max_size.height;
    let uncapped_request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &uncapped_request);
    let uncapped = compute(&tree, 0, &uncapped_request)
        .expect("uncapped native exact-fill sibling-boundary control");
    verify(&uncapped);

    tree.inputs[1].max_size.height = maximum_progress_cap::<S>(70.0).max_size.height;
    let request = unequal(LayoutSourceBasis::fresh());
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("capped exact-fill continuation preserves the accepted sibling boundary");
    verify(&batch);
    batch.apply_to(&mut sink).unwrap();
    for node in [2, 3, 4] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_exact_fill_preserves_sibling_class_boundary,
    maximum_progress_exact_fill_preserves_sibling_class_boundary_f32,
    maximum_progress_exact_fill_preserves_sibling_class_boundary_f64
);

fn maximum_progress_cross_progression_request<S: LayoutScalar>(
    extent: f64,
) -> LayoutRootRequestOf<S> {
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let sizing = native_request::sizing(
        Size::new(Some(S::from_f64(extent)), Some(S::from_f64(100.0))),
        Size::splat(SizingOperation::Normal),
    );
    LayoutRootRequestOf::standalone(
        sizing,
        axes,
        sizing.orthogonal_fallback().initial_containing_block(),
    )
}

fn maximum_progress_cross_progression_verify<S: LayoutScalar>(
    batch: &Batch<S>,
    owner: f64,
    child: f64,
    root_x: f64,
    native: bool,
) {
    let width = owner + 10.0;
    assert_eq!(batch.node_layouts().count(), 5);
    assert_eq!(batch.fragments().count(), 5);
    for (node, x, width) in [
        (0, root_x, width),
        (1, 0.0, width),
        (2, 0.0, owner),
        (3, owner - child, child),
        (4, owner, 10.0),
    ] {
        let value = single(batch, node);
        for output in [value.unrounded_output(), value.final_output()] {
            assert_packet(output, (x, 0.0), (width, 100.0));
            assert_eq!(
                output.source_index,
                SourceIndex::new(usize::from(node == 4))
            );
        }
    }
    let root = single(batch, 0);
    if native {
        assert_eq!(batch.fragmentainers().len(), 1);
        assert_eq!(
            root.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragmentainer(
                batch.fragmentainers()[0].id().clone()
            ))
        );
        for (node, parent) in [(1, 0), (2, 1), (3, 2), (4, 1)] {
            assert_eq!(
                single(batch, node).coordinate_parent(),
                Some(&FragmentCoordinateSpace::Fragment(
                    single(batch, parent).id().clone()
                ))
            );
        }
        maximum_progress_children(batch, root.id(), &[single(batch, 1).id()]);
        maximum_progress_children(
            batch,
            single(batch, 1).id(),
            &[single(batch, 2).id(), single(batch, 4).id()],
        );
        maximum_progress_children(batch, single(batch, 2).id(), &[single(batch, 3).id()]);
    } else {
        assert_eq!(
            root.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Root)
        );
    }
    maximum_progress_unique_fragments(batch);
}

fn maximum_progress_cross_progression_whole_fit_case<S: LayoutScalar>(cap: f64, child: f64) {
    let mut tree = Tree::new(
        vec![
            block::<S>(None, None),
            block(None, None),
            block(None, None),
            block(Some(child), None),
            block(Some(10.0), None),
        ],
        vec![vec![1], vec![2, 4], vec![3], vec![], vec![]],
    );
    for (node, input) in tree.inputs.iter_mut().enumerate() {
        input.writing_mode = if matches!(node, 1 | 4) {
            WritingMode::VerticalLr
        } else {
            WritingMode::VerticalRl
        };
    }
    tree.inputs[2].max_size.width =
        MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(cap)).unwrap());
    let owner = child.min(cap);
    let continuous =
        compute_layout(&tree, 0, maximum_progress_cross_progression_request(200.0)).unwrap();
    maximum_progress_cross_progression_verify(
        &continuous,
        owner,
        child,
        200.0 - owner - 10.0,
        false,
    );

    tree.inputs[2].max_size.width = block::<S>(None, None).max_size.width;
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let uncapped_request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        maximum_progress_cross_progression_request(200.0),
        FragmentationKind::Page,
        axes,
        vec![frame(200.0, 100.0, true)],
    )
    .unwrap();
    let mut sink = Sink::new(5);
    sink.authorize(0, &uncapped_request);
    let uncapped =
        compute(&tree, 0, &uncapped_request).expect("uncapped whole-fit cross-progression control");
    maximum_progress_cross_progression_verify(&uncapped, child, child, 200.0 - child - 10.0, true);

    tree.inputs[2].max_size.width =
        MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(cap)).unwrap());
    let whole_request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        maximum_progress_cross_progression_request(300.0),
        FragmentationKind::Page,
        axes,
        vec![frame(300.0, 100.0, true)],
    )
    .unwrap();
    sink.authorize(0, &whole_request);
    let whole =
        compute(&tree, 0, &whole_request).expect("capped whole-fit cross-progression control");
    maximum_progress_cross_progression_verify(&whole, owner, child, 300.0 - owner - 10.0, true);

    let request = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        maximum_progress_cross_progression_request(200.0),
        FragmentationKind::Page,
        axes,
        vec![frame(200.0, 100.0, true)],
    )
    .unwrap();
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("whole-fitting owner under opposite ordinary progression retains actual placement");
    // Block's final pass places Owner at wrapper-local physical 0. The wrapper
    // is owner +10 wide, so that point maps to fragmentation block offset 10.
    // For cap 1/child 150, child-local x is 1-150=-149, without a continuation.
    maximum_progress_cross_progression_verify(&batch, owner, child, 200.0 - owner - 10.0, true);
    batch.apply_to(&mut sink).unwrap();
    for node in 0..5 {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}

fn maximum_progress_cross_progression_exact_cap_keeps_whole_owner_origin<S: LayoutScalar>() {
    maximum_progress_cross_progression_whole_fit_case::<S>(20.0, 20.0);
}
scalars!(
    maximum_progress_cross_progression_exact_cap_keeps_whole_owner_origin,
    maximum_progress_cross_progression_exact_cap_keeps_whole_owner_origin_f32,
    maximum_progress_cross_progression_exact_cap_keeps_whole_owner_origin_f64
);

fn maximum_progress_cross_progression_short_owner_keeps_actual_child_overflow<S: LayoutScalar>() {
    maximum_progress_cross_progression_whole_fit_case::<S>(1.0, 150.0);
}
scalars!(
    maximum_progress_cross_progression_short_owner_keeps_actual_child_overflow,
    maximum_progress_cross_progression_short_owner_keeps_actual_child_overflow_f32,
    maximum_progress_cross_progression_short_owner_keeps_actual_child_overflow_f64
);

fn maximum_progress_selected_inline_percentage_settles_overflow_children<S: LayoutScalar>() {
    let mut root = maximum_progress_cap::<S>(20.0);
    root.box_sizing = BoxSizing::ContentBox;
    let mut relative = block::<S>(None, Some(30.0));
    relative.position = Position::Relative;
    relative.inset.left =
        LengthAutoOf::value(LengthPercentageOf::from_percent_fraction(S::from_f64(0.1)).unwrap());
    let mut tree = Tree::new(
        vec![root, relative.clone(), relative.clone(), relative],
        vec![vec![1, 2, 3], vec![], vec![], vec![]],
    );
    let continuous = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    for output in [
        single(&continuous, 0).unrounded_output(),
        single(&continuous, 0).final_output(),
    ] {
        assert_packet(output, (0.0, 0.0), (100.0, 20.0));
    }
    for (node, y) in [(1, 0.0), (2, 30.0), (3, 60.0)] {
        let child = single(&continuous, node);
        for output in [child.unrounded_output(), child.final_output()] {
            assert_eq!(output.source_index, SourceIndex::new(node - 1));
            assert_packet(output, (10.0, y), (100.0, 30.0));
        }
    }
    let mut sink = Sink::new(4);
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    sink.authorize(0, &whole_request);
    let whole =
        compute(&tree, 0, &whole_request).expect("whole-fit selected inline percentage control");
    assert_eq!(whole.fragmentainers().len(), 1);
    let whole_root = single(&whole, 0);
    for output in [whole_root.unrounded_output(), whole_root.final_output()] {
        assert_packet(output, (0.0, 0.0), (100.0, 20.0));
    }
    assert_eq!(
        whole_root.coordinate_parent(),
        Some(&FragmentCoordinateSpace::Fragmentainer(
            whole.fragmentainers()[0].id().clone()
        ))
    );
    for (node, y) in [(1, 0.0), (2, 30.0), (3, 60.0)] {
        minimum_progress_child(
            &whole,
            node,
            node - 1,
            whole_root.id(),
            (10.0, y),
            (100.0, 30.0),
        );
    }
    maximum_progress_children(
        &whole,
        whole_root.id(),
        &[
            single(&whole, 1).id(),
            single(&whole, 2).id(),
            single(&whole, 3).id(),
        ],
    );

    // Position 3 §3.1 selects only the corresponding inline-axis percentage:
    // each accepted width supplies 10%, hence 10,6,8. No block percentage exists.
    tree.inputs[0].max_size.height = block::<S>(None, None).max_size.height;
    let uncapped_request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 30.0, true),
            frame(60.0, 30.0, false),
            frame(80.0, 30.0, false),
        ],
    );
    sink.authorize(0, &uncapped_request);
    let uncapped = compute(&tree, 0, &uncapped_request)
        .expect("uncapped varying-frame selected inline percentage control");
    let controls = minimum_progress_chain(&uncapped, 0, &[&[(0, 1)], &[(1, 2)], &[(2, 3)]]);
    for (index, width, x) in [(0, 100.0, 10.0), (1, 60.0, 6.0), (2, 80.0, 8.0)] {
        assert_fragment(&controls[index], (width, 30.0));
        assert_eq!(controls[index].unrounded().location, Point::ZERO);
        assert_eq!(controls[index].final_geometry().location, Point::ZERO);
        minimum_progress_child(
            &uncapped,
            index + 1,
            index,
            controls[index].id(),
            (x, 0.0),
            (width, 30.0),
        );
        maximum_progress_children(
            &uncapped,
            controls[index].id(),
            &[single(&uncapped, index + 1).id()],
        );
    }
    tree.inputs[0].max_size.height = maximum_progress_cap::<S>(20.0).max_size.height;
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 30.0, true),
            frame(60.0, 30.0, false),
            frame(80.0, 30.0, false),
        ],
    );
    sink.authorize(0, &request);
    let batch = compute(&tree, 0, &request)
        .expect("overflow-only content retains the actual selected inline percentage basis");
    assert_eq!(batch.fragmentainers().len(), 3);
    assert_eq!(batch.node_layouts().count(), 4);
    let roots = minimum_progress_chain(&batch, 0, &[&[(0, 1)], &[(1, 2)], &[(2, 3)]]);
    assert_fragment(&roots[0], (100.0, 20.0));
    assert_eq!(roots[0].unrounded().location, Point::ZERO);
    assert_eq!(roots[0].final_geometry().location, Point::ZERO);
    for (index, width, x) in [(0, 100.0, 10.0), (1, 60.0, 6.0), (2, 80.0, 8.0)] {
        minimum_progress_child(
            &batch,
            index + 1,
            index,
            roots[index].id(),
            (x, 0.0),
            (width, 30.0),
        );
        maximum_progress_children(&batch, roots[index].id(), &[single(&batch, index + 1).id()]);
        if index > 0 {
            assert!(roots[index].principal_box().is_none());
            assert!(matches!(
                roots[index].payload(),
                FragmentPayloadOf::OverflowOnly(_)
            ));
            let overflow = roots[index].overflow_only().unwrap();
            // Descendant visual bounds have no principal-container seed. Their
            // origin includes the settled shift, while their child size stays 30.
            for geometry in [overflow.unrounded(), overflow.final_geometry()] {
                assert_eq!(geometry.origin, Point::ZERO);
                assert_eq!(
                    geometry.content_bounds.origin(),
                    Point::new(S::from_f64(x), S::ZERO)
                );
                assert_eq!(
                    geometry.content_bounds.size(),
                    Size::new(S::from_f64(width), S::from_f64(30.0))
                );
            }
        }
    }
    maximum_progress_unique_fragments(&batch);
    batch.apply_to(&mut sink).unwrap();
    assert_eq!(sink.state.custody[0], None);
    for node in [1, 2, 3] {
        assert_eq!(
            sink.state.custody[node],
            Some(single(&batch, node).unrounded_entry())
        );
    }
}
scalars!(
    maximum_progress_selected_inline_percentage_settles_overflow_children,
    maximum_progress_selected_inline_percentage_settles_overflow_children_f32,
    maximum_progress_selected_inline_percentage_settles_overflow_children_f64
);

fn maximum_progress_selected_block_percentage_requires_actual_basis<S: LayoutScalar>() {
    let mut root = maximum_progress_cap::<S>(20.0);
    root.box_sizing = BoxSizing::ContentBox;
    let mut relative = block::<S>(None, Some(30.0));
    relative.position = Position::Relative;
    relative.inset.top =
        LengthAutoOf::value(LengthPercentageOf::from_percent_fraction(S::from_f64(0.1)).unwrap());
    let mut tree = Tree::new(
        vec![root, relative.clone(), relative.clone(), relative],
        vec![vec![1, 2, 3], vec![], vec![], vec![]],
    );
    let continuous = compute_layout(&tree, 0, root_request(100.0)).unwrap();
    for output in [
        single(&continuous, 0).unrounded_output(),
        single(&continuous, 0).final_output(),
    ] {
        assert_packet(output, (0.0, 0.0), (100.0, 20.0));
    }
    // The ordinary measured content box is 20 high, so each top 10% is 2;
    // relative displacement leaves the normal child positions 0,30,60 intact.
    for (node, y) in [(1, 2.0), (2, 32.0), (3, 62.0)] {
        let child = single(&continuous, node);
        for output in [child.unrounded_output(), child.final_output()] {
            assert_eq!(output.source_index, SourceIndex::new(node - 1));
            assert_packet(output, (0.0, y), (100.0, 30.0));
        }
    }
    let mut sink = Sink::new(4);
    let whole_request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 100.0, true)]);
    sink.authorize(0, &whole_request);
    let whole = compute(&tree, 0, &whole_request)
        .expect("whole-fit selected block percentage has actual measured basis");
    assert_eq!(whole.fragmentainers().len(), 1);
    let whole_root = single(&whole, 0);
    for output in [whole_root.unrounded_output(), whole_root.final_output()] {
        assert_packet(output, (0.0, 0.0), (100.0, 20.0));
    }
    for (node, y) in [(1, 2.0), (2, 32.0), (3, 62.0)] {
        minimum_progress_child(
            &whole,
            node,
            node - 1,
            whole_root.id(),
            (0.0, y),
            (100.0, 30.0),
        );
    }
    maximum_progress_children(
        &whole,
        whole_root.id(),
        &[
            single(&whole, 1).id(),
            single(&whole, 2).id(),
            single(&whole, 3).id(),
        ],
    );
    whole.apply_to(&mut sink).unwrap();

    tree.inputs[0].max_size.height = block::<S>(None, None).max_size.height;
    let uncapped_request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 30.0, true),
            frame(60.0, 30.0, false),
            frame(80.0, 30.0, false),
        ],
    );
    sink.authorize(0, &uncapped_request);
    let uncapped = compute(&tree, 0, &uncapped_request)
        .expect("uncapped varying-frame block percentage has an actual box in each frame");
    let controls = minimum_progress_chain(&uncapped, 0, &[&[(0, 1)], &[(1, 2)], &[(2, 3)]]);
    for (index, width) in [(0, 100.0), (1, 60.0), (2, 80.0)] {
        assert_fragment(&controls[index], (width, 30.0));
        assert_eq!(controls[index].unrounded().location, Point::ZERO);
        assert_eq!(controls[index].final_geometry().location, Point::ZERO);
        minimum_progress_child(
            &uncapped,
            index + 1,
            index,
            controls[index].id(),
            (0.0, 3.0),
            (width, 30.0),
        );
        maximum_progress_children(
            &uncapped,
            controls[index].id(),
            &[single(&uncapped, index + 1).id()],
        );
    }
    tree.inputs[0].max_size.height = maximum_progress_cap::<S>(20.0).max_size.height;
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 30.0, true),
            frame(60.0, 30.0, false),
            frame(80.0, 30.0, false),
        ],
    );
    sink.authorize(0, &request);
    // The first actual principal content box supplies 20 for the first child's
    // offset 2. A later overflow carrier has no block box from which to resolve
    // the selected top percentage; neither frame height nor overflow is a basis.
    let error = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(
        error.operation(),
        LayoutOperation::ValueResolution,
        "{error:?}"
    );
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
    assert_eq!(
        error.site(),
        LayoutErrorSiteOf::FragmentContainingBlock {
            container: 0,
            subject: 2,
            fragmentainer_index: 1,
            axis: PhysicalAxis::Vertical,
        }
    );
    let previous = sink.state.clone();
    let commits = sink.commits;
    for input in &mut tree.inputs[1..] {
        input.inset.top = LengthAutoOf::value(
            LengthPercentageOf::from_coefficients(S::from_f64(4.0), S::from_f64(0.1)).unwrap(),
        );
    }
    let mixed = compute(&tree, 0, &request).unwrap_err();
    assert_eq!(mixed.operation(), LayoutOperation::ValueResolution);
    assert_eq!(mixed.site(), error.site());
    assert!(matches!(
        mixed.kind(),
        LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
    ));
    assert_eq!(sink.state, previous);
    assert_eq!(sink.commits, commits);
}
scalars!(
    maximum_progress_selected_block_percentage_requires_actual_basis,
    maximum_progress_selected_block_percentage_requires_actual_basis_f32,
    maximum_progress_selected_block_percentage_requires_actual_basis_f64
);

fn maximum_progress_nonbinding_scope_transports_actual_margin_contributors<S: LayoutScalar>() {
    for separator in 0..4 {
        margin_boundary_cap_case::<S>(separator, true);
    }
}
scalars!(
    maximum_progress_nonbinding_scope_transports_actual_margin_contributors,
    maximum_progress_nonbinding_scope_transports_actual_margin_contributors_f32,
    maximum_progress_nonbinding_scope_transports_actual_margin_contributors_f64
);

fn maximum_progress_zero_principal_settles_positive_natural_content<S: LayoutScalar>() {
    // The principal's measured content basis is zero. Natural content150 and
    // frame200 deliberately differ. The absolute4 operand also proves that
    // visual settlement executes rather than leaving the normal location.
    for absolute in [0.0, 4.0] {
        let mut child = block::<S>(None, Some(150.0));
        child.inset.top = LengthAutoOf::value(
            LengthPercentageOf::from_coefficients(S::from_f64(absolute), S::from_f64(0.1)).unwrap(),
        );
        let tree = Tree::new(
            vec![maximum_progress_cap(0.0), block(None, Some(0.0)), child],
            vec![vec![1, 2], vec![], vec![]],
        );
        let request = fragmented(LayoutSourceBasis::fresh(), vec![frame(100.0, 200.0, true)]);
        let batch = compute(&tree, 0, &request)
            .expect("actual zero principal has a full measured inset basis");
        assert_eq!(batch.fragmentainers().len(), 1);
        assert_packet(
            single(&batch, 0).unrounded_output(),
            (0.0, 0.0),
            (100.0, 0.0),
        );
        let child = single(&batch, 2);
        assert_packet(child.unrounded_output(), (0.0, absolute), (100.0, 150.0));
        assert_packet(child.final_output(), (0.0, absolute), (100.0, 150.0));
        maximum_progress_children(
            &batch,
            single(&batch, 0).id(),
            &[single(&batch, 1).id(), child.id()],
        );
    }
}
scalars!(
    maximum_progress_zero_principal_settles_positive_natural_content,
    maximum_progress_zero_principal_settles_positive_natural_content_f32,
    maximum_progress_zero_principal_settles_positive_natural_content_f64
);

fn maximum_progress_unowned_ancestor_case<S: LayoutScalar>(case: u8) {
    let mut root_input = block::<S>(None, None);
    let mut wrapper_input = block::<S>(Some(40.0), None);
    wrapper_input.margin.left = LengthAutoOf::AUTO;
    wrapper_input.margin.right = LengthAutoOf::AUTO;
    let edge = LengthOf::value(LengthPercentageOf::px(S::from_f64(5.0)).unwrap());
    wrapper_input.padding.left = edge;
    wrapper_input.padding.right = edge;
    let percentage = |fraction| {
        LengthAutoOf::value(
            LengthPercentageOf::from_percent_fraction(S::from_f64(fraction)).unwrap(),
        )
    };
    let absolute = |value| LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap());
    let mut owner_input = maximum_progress_cap::<S>(20.0);
    let mut expected_u = (10.0, 0.0);
    let mut expected_w = 5.0;
    match case {
        1 => {
            wrapper_input.inset.left = percentage(0.1);
            expected_u.0 += 6.0;
        }
        2 => {
            wrapper_input.inset.left = absolute(4.0);
            expected_u.0 += 4.0;
        }
        3 => wrapper_input.inset.top = percentage(0.0),
        4 => {
            wrapper_input.inset.top = absolute(4.0);
            wrapper_input.inset.bottom = percentage(0.1);
            expected_u.1 = 4.0;
        }
        5 => {
            wrapper_input.inset.left = percentage(0.1);
            wrapper_input.inset.right = absolute(3.0);
            expected_u.0 -= 3.0;
        }
        6 => {
            wrapper_input.inset.left = LengthAutoOf::value(
                LengthPercentageOf::from_coefficients(S::from_f64(4.0), S::from_f64(0.1)).unwrap(),
            );
            expected_u.0 += 10.0;
        }
        7 => {
            root_input.padding.left =
                LengthOf::value(LengthPercentageOf::px(S::from_f64(10.0)).unwrap());
            root_input.padding.right = root_input.padding.left;
            root_input.scrollbar_gutter = ScrollbarGutter::StableBothEdges;
            root_input.scrollbar_width = ScrollbarWidthOf::try_new(S::from_f64(4.0)).unwrap();
            wrapper_input.inset.left = percentage(0.1);
            expected_u.0 += 4.0;
        }
        8 => {
            wrapper_input.inset.left = percentage(0.125);
            expected_u.0 += 7.5;
        }
        9 => {
            wrapper_input.size.width = PreferredSizeOf::AUTO;
            owner_input.size.width = px(30.0);
            owner_input.margin.left = LengthAutoOf::AUTO;
            owner_input.margin.right = LengthAutoOf::AUTO;
            wrapper_input.inset.left = percentage(0.1);
            expected_u.0 = 6.0;
            expected_w = 15.0;
        }
        10 | 11 => {
            wrapper_input.inset.top = percentage(0.1);
            expected_u.1 = 9.0;
        }
        _ => {}
    }
    let mut last = block::<S>(if case == 8 { Some(30.5) } else { None }, Some(30.0));
    last.break_before = FragmentationBreak::Page;
    let mut tree = Tree::new(
        vec![
            root_input,
            wrapper_input,
            owner_input,
            block(None, Some(30.0)),
            last,
            block(None, Some(90.0)),
        ],
        vec![
            if case == 11 { vec![1, 5] } else { vec![1] },
            vec![2],
            vec![3, 4],
            vec![],
            vec![],
            vec![],
        ],
    );
    for input in &mut tree.inputs {
        input.direction = Direction::Rtl;
    }
    if case == 8 {
        for input in &mut tree.inputs[2..] {
            input.direction = Direction::Ltr;
        }
    }
    let origin = if case == 8 {
        Point::new(S::from_f64(0.25), S::ZERO)
    } else {
        Point::ZERO
    };
    let request = maximum_progress_axis_request_origin(false, false, origin);
    if case == 10 || case == 12 || case == 13 {
        tree.inputs[1].inset.top = LengthAutoOf::AUTO;
        let prior_request = maximum_progress_axis_request(false, false);
        let prior = compute(&tree, 0, &prior_request).unwrap();
        let mut sink = Sink::new(6);
        sink.authorize(0, &prior_request);
        prior.apply_to(&mut sink).unwrap();
        let previous = sink.state.clone();
        let commits = sink.commits;
        if case == 10 {
            tree.inputs[1].inset.top = percentage(0.1);
        } else if case == 12 {
            tree.inputs[1].aspect_ratio = AspectRatioOf::new(S::from_f64(2.0));
        } else {
            tree.inputs[0].overflow =
                ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap();
            tree.inputs[0].scrollbar_gutter = ScrollbarGutter::StableBothEdges;
            tree.inputs[0].scrollbar_width = ScrollbarWidthOf::try_new(S::from_f64(4.0)).unwrap();
        }
        sink.authorize(0, &request);
        let error = compute(&tree, 0, &request).unwrap_err();
        if case == 10 {
            assert_eq!(error.operation(), LayoutOperation::ValueResolution);
            assert_eq!(
                error.site(),
                LayoutErrorSiteOf::FragmentContainingBlock {
                    container: 0,
                    subject: 1,
                    fragmentainer_index: 1,
                    axis: PhysicalAxis::Vertical
                }
            );
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis)
            ));
        } else {
            assert_eq!(
                error.site(),
                LayoutErrorSiteOf::Node(if case == 12 { 1 } else { 0 })
            );
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::UnsupportedCapability(
                    LayoutUnsupportedCapability::BlockFragmentationProfile
                )
            ));
        }
        assert_eq!(sink.state, previous);
        assert_eq!(sink.commits, commits);
        return;
    }
    let batch = compute(&tree, 0, &request)
        .unwrap_or_else(|error| panic!("implicit ancestor case{case}: {error:?}"));
    let roots = fragments(&batch, 0);
    let wrappers = fragments(&batch, 1);
    let owners = fragments(&batch, 2);
    for chain in [&roots, &wrappers, &owners] {
        assert_eq!(chain.len(), 2);
    }
    if case == 7 {
        for geometry in [roots[0].unrounded(), roots[0].final_geometry()] {
            assert_eq!(
                geometry.size,
                Size::new(S::from_f64(100.0), S::from_f64(20.0))
            );
            assert_eq!(
                geometry.padding,
                Edges {
                    left: S::from_f64(10.0),
                    right: S::from_f64(10.0),
                    top: S::ZERO,
                    bottom: S::ZERO
                }
            );
            assert_eq!(geometry.border, Edges::all(S::ZERO));
            assert_eq!(geometry.margin, Edges::all(S::ZERO));
        }
    } else {
        assert_fragment(&roots[0], (100.0, if case == 11 { 50.0 } else { 20.0 }));
    }
    assert_eq!(roots[0].unrounded().location, Point::ZERO);
    if case == 11 {
        assert_fragment(&roots[1], (60.0, 90.0));
        assert_packet(
            single(&batch, 5).unrounded_output(),
            (0.0, 0.0),
            (60.0, 90.0),
        );
    } else {
        assert_eq!(
            roots[1].overflow_only().unwrap().unrounded().origin,
            Point::ZERO
        );
    }
    let wrapper = wrappers[1].overflow_only().unwrap();
    assert_eq!(
        wrapper.unrounded().origin,
        Point::new(S::from_f64(expected_u.0), S::from_f64(expected_u.1))
    );
    assert_eq!(
        owners[1].overflow_only().unwrap().unrounded().origin,
        Point::new(S::from_f64(expected_w), S::ZERO)
    );
    if case == 8 {
        assert_eq!(wrapper.final_geometry().origin.x, S::from_f64(18.0));
        let child = single(&batch, 4);
        assert_eq!(child.unrounded_output().location.x, S::ZERO);
        assert_eq!(child.unrounded_output().size.width, S::from_f64(30.5));
        // Visual22.75/end53.25 gives30. The internal composed-path test
        // independently retains normal15.25/end45.75 and canonical width31.
        assert_eq!(child.final_output().size.width, S::from_f64(30.0));
    } else {
        minimum_progress_child(&batch, 4, 1, owners[1].id(), (0.0, 0.0), (30.0, 30.0));
    }
    minimum_progress_child(&batch, 3, 0, owners[0].id(), (0.0, 0.0), (30.0, 30.0));
    if case == 9 {
        assert_eq!(owners[0].unrounded().location.x, S::from_f64(35.0));
    }
    maximum_progress_children(&batch, wrappers[1].id(), &[owners[1].id()]);
    maximum_progress_children(&batch, owners[1].id(), &[single(&batch, 4).id()]);
    if case != 11 {
        maximum_progress_children(&batch, roots[1].id(), &[wrappers[1].id()]);
    }
    maximum_progress_unique_fragments(&batch);
}
fn maximum_progress_unowned_ancestors_keep_source_inline_carrier_context<S: LayoutScalar>() {
    maximum_progress_unowned_ancestor_case::<S>(0);
}
fn maximum_progress_implicit_carrier_inline_receipts_preserve_axis_authority<S: LayoutScalar>() {
    for case in 1..=13 {
        maximum_progress_unowned_ancestor_case::<S>(case);
    }
}

scalars!(
    maximum_progress_unowned_ancestors_keep_source_inline_carrier_context,
    maximum_progress_unowned_ancestors_keep_source_inline_carrier_context_f32,
    maximum_progress_unowned_ancestors_keep_source_inline_carrier_context_f64
);

scalars!(
    maximum_progress_implicit_carrier_inline_receipts_preserve_axis_authority,
    maximum_progress_implicit_carrier_inline_receipts_preserve_axis_authority_f32,
    maximum_progress_implicit_carrier_inline_receipts_preserve_axis_authority_f64
);

fn maximum_progress_opening_margin_case<S: LayoutScalar>(without_prefix: bool) {
    let mut root_input = block::<S>(None, None);
    let margin = if without_prefix { 99.0 } else { 10.0 };
    if without_prefix {
        root_input.padding.top = LengthOf::value(LengthPercentageOf::px(S::from_f64(1.0)).unwrap());
    }
    let mut owner = maximum_progress_cap::<S>(200.0);
    owner.margin.top = LengthAutoOf::value(LengthPercentageOf::px(S::from_f64(margin)).unwrap());
    let tree = Tree::new(
        vec![
            root_input,
            block(None, Some(90.0)),
            owner,
            block(None, Some(0.0)),
            block(None, Some(150.0)),
            block(None, Some(10.0)),
        ],
        vec![
            if without_prefix {
                vec![2, 5]
            } else {
                vec![1, 2, 5]
            },
            vec![],
            vec![3, 4],
            vec![],
            vec![],
            vec![],
        ],
    );
    let request = fragmented(
        LayoutSourceBasis::fresh(),
        vec![
            frame(100.0, 100.0, true),
            frame(60.0, 80.0, false),
            frame(80.0, 60.0, false),
        ],
    );
    let batch = compute(&tree, 0, &request)
        .expect("opening margin survives an interior retained child transition");
    let roots = fragments(&batch, 0);
    let owners = fragments(&batch, 2);
    assert_eq!(roots.len(), 3);
    assert_eq!(owners.len(), 2);
    for (index, (width, height)) in [(100.0, 100.0), (60.0, 150.0), (80.0, 10.0)]
        .into_iter()
        .enumerate()
    {
        for geometry in [roots[index].unrounded(), roots[index].final_geometry()] {
            assert_eq!(geometry.location, Point::ZERO);
            assert_eq!(
                geometry.size,
                Size::new(S::from_f64(width), S::from_f64(height))
            );
            assert_eq!(
                geometry.padding.top,
                if without_prefix && index == 0 {
                    S::ONE
                } else {
                    S::ZERO
                }
            );
        }
    }
    for (index, (width, height, y)) in [(100.0, 0.0, 100.0), (60.0, 150.0, 0.0)]
        .into_iter()
        .enumerate()
    {
        for geometry in [owners[index].unrounded(), owners[index].final_geometry()] {
            assert_eq!(geometry.location, Point::new(S::ZERO, S::from_f64(y)));
            assert_eq!(
                geometry.size,
                Size::new(S::from_f64(width), S::from_f64(height))
            );
            assert_eq!(
                geometry.margin.top,
                if index == 0 {
                    S::from_f64(margin)
                } else {
                    S::ZERO
                }
            );
        }
        assert_eq!(
            owners[index].fragmentainer(),
            Some(batch.fragmentainers()[index].id())
        );
    }
    minimum_progress_child(&batch, 3, 0, owners[0].id(), (0.0, 0.0), (100.0, 0.0));
    minimum_progress_child(&batch, 4, 1, owners[1].id(), (0.0, 0.0), (60.0, 150.0));
    minimum_progress_child(
        &batch,
        5,
        if without_prefix { 1 } else { 2 },
        roots[2].id(),
        (0.0, 0.0),
        (80.0, 10.0),
    );
    if without_prefix {
        assert!(batch.node_layout(1).is_none());
    } else {
        minimum_progress_child(&batch, 1, 0, roots[0].id(), (0.0, 0.0), (100.0, 90.0));
    }
    maximum_progress_unique_fragments(&batch);
}
fn maximum_progress_opening_margin_prefix<S: LayoutScalar>() {
    maximum_progress_opening_margin_case::<S>(false);
}
fn maximum_progress_opening_margin_no_prefix<S: LayoutScalar>() {
    maximum_progress_opening_margin_case::<S>(true);
}
scalars!(
    maximum_progress_opening_margin_prefix,
    maximum_progress_opening_margin_prefix_f32,
    maximum_progress_opening_margin_prefix_f64
);
scalars!(
    maximum_progress_opening_margin_no_prefix,
    maximum_progress_opening_margin_no_prefix_f32,
    maximum_progress_opening_margin_no_prefix_f64
);
