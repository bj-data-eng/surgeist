//! Actual host geometry caches cannot substitute for demanded native publication.
use super::*;
use crate::{
    CacheKeyContext, CacheOf, ComputeInputOf, ComputeOutputOf, FloatExclusionIntervalOf,
    FloatExclusionQueryOf, LayoutBatchSink, LayoutInputViewOf, LayoutTree, LeafMeasureInputOf,
    RunMode, Traverse,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LeafFailure(u32);
#[derive(Clone)]
struct Committed<S: LayoutScalar> {
    caches: HashMap<u32, CacheOf<S>>,
    unrounded: HashMap<u32, NodeOutputOf<S>>,
    batch: Option<CompletedLayoutBatchOf<u32, S>>,
}
struct Host<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    committed: Rc<RefCell<Committed<S>>>,
    hits: RefCell<Vec<(u32, RunMode)>>,
    measured: RefCell<Vec<u32>>,
    shapes: RefCell<Vec<(u32, FloatExclusionQueryOf<S>, bool)>>,
    fail: Cell<Option<u32>>,
}
struct Sink<S: LayoutScalar> {
    committed: Rc<RefCell<Committed<S>>>,
}
impl<S: LayoutScalar> Host<S> {
    fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            committed: Rc::new(RefCell::new(Committed {
                caches: HashMap::new(),
                unrounded: HashMap::new(),
                batch: None,
            })),
            hits: RefCell::new(Vec::new()),
            measured: RefCell::new(Vec::new()),
            shapes: RefCell::new(Vec::new()),
            fail: Cell::new(None),
        }
    }
    fn sink(&self) -> Sink<S> {
        Sink {
            committed: Rc::clone(&self.committed),
        }
    }
}
impl<S: LayoutScalar> Traverse for Host<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl<S: LayoutScalar> LayoutTree for Host<S> {
    type MeasureError = LeafFailure;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        self.tree.has_leaf_measurement(node)
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<std::result::Result<Size<S>, LeafFailure>> {
        self.measured.borrow_mut().push(node);
        if self.fail.get() == Some(node) {
            return Some(Err(LeafFailure(node)));
        }
        self.tree
            .measure_leaf(node, input)
            .map(|result| result.map_err(|never| match never {}))
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .committed
            .borrow()
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        if output.is_some() {
            self.hits.borrow_mut().push((node, input.run_mode()));
        }
        output
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<S>> {
        self.committed.borrow().unrounded.get(&node).copied()
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: FloatExclusionQueryOf<S>,
    ) -> Option<std::result::Result<Option<FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        assert_eq!(node, 30);
        assert_eq!(
            query.flow_axes(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        );
        let bounds = query.margin_box();
        assert_eq!(bounds.size(), Size::new(n(1.0), n(10.0)));
        // The actual rectangle fills the real float margin box. Its provider
        // receives production band/placement authority; no cache output or
        // guessed line geometry is manufactured here.
        let interval = if query.band_minimum() < bounds.origin().y + bounds.size().height
            && query.band_maximum() > bounds.origin().y
        {
            FloatExclusionIntervalOf::try_new(
                query,
                bounds.origin().x,
                bounds.origin().x + bounds.size().width,
            )
            .unwrap()
        } else {
            None
        };
        self.shapes
            .borrow_mut()
            .push((node, query, interval.is_some()));
        Some(Ok(interval))
    }
}
impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Sink<S> {
    type Error = core::convert::Infallible;
    type Prepared = Committed<S>;
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> std::result::Result<Self::Prepared, Self::Error> {
        let mut next = self.committed.borrow().clone();
        for node in batch.invalidated_nodes() {
            next.caches.remove(node);
            next.unrounded.remove(node);
        }
        for entry in batch.cache_clear_entries() {
            next.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            next.caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        for entry in batch.unrounded_entries() {
            next.unrounded.insert(entry.node(), entry.output());
        }
        next.batch = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch(&mut self, next: Self::Prepared) {
        *self.committed.borrow_mut() = next;
    }
}

fn admit<'a, S: LayoutScalar>(
    host: &'a Host<S>,
    root: u32,
    wrapping: WrappingInput<u32>,
) -> LayoutFlowInput<'a, Host<S>> {
    LayoutFlowInput::try_new(host, root, vec![], wrapping, Vec::new()).unwrap()
}

fn seed_and_observe_root<S: LayoutScalar>(host: &Host<S>, root: u32, shape: Option<u32>) {
    // No scope/box operation or family demand is smuggled into this ordinary
    // control. The tree has genuine neutral Contribution endpoints, not
    // physical BoxEdges that would already make geometry caching ineligible.
    let ordinary = admit(host, root, WrappingInput::default());
    let cold = compute_layout_with_flow(&ordinary, root, request(n(100.0))).unwrap();
    assert_eq!(cold.native_inline_box_families().count(), 0);
    assert_eq!(cold.native_inline_box_pieces().count(), 0);
    if let Some(node) = shape {
        assert_eq!(node, 30);
        assert!(
            host.shapes
                .borrow()
                .iter()
                .any(|(actual, _, used)| *actual == node && *used),
            "the actual positive rectangle must participate before a cache premise is claimed"
        );
        let actual = output(&cold, node);
        assert_eq!(actual.location, Point::new(S::ZERO, S::ZERO));
        assert_eq!(actual.size, Size::new(n(1.0), n(10.0)));
        assert_eq!(actual.source_index.get(), 0);
        assert_eq!(
            atom_positions(&cold),
            [
                (1.0, 0.0),
                (11.0, 0.0),
                (21.0, 0.0),
                (31.0, 0.0),
                (51.0, 0.0),
                (41.0, 0.0)
            ]
        );
    }
    let seed = cold
        .cache_store_entries()
        .iter()
        .find(|entry| {
            entry.node() == root && entry.input().run_mode() == RunMode::PerformRootLayout
        })
        .expect("the real ordinary viewport root must offer a geometry cache entry");
    let mut sink = host.sink();
    cold.apply_to(&mut sink).unwrap();
    assert_eq!(
        host.committed.borrow().caches[&root].get_with_context(seed.input(), seed.context()),
        Some(seed.output())
    );
    host.hits.borrow_mut().clear();
    let warm = compute_layout_with_flow(&ordinary, root, request(n(100.0))).unwrap();
    assert!(
        host.hits
            .borrow()
            .contains(&(root, RunMode::PerformRootLayout)),
        "the ordinary root cache must actually hit, not just contain an unusable entry"
    );
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(warm.native_inline_box_families().count(), 0);
    warm.apply_to(&mut sink).unwrap();
}

fn check_family<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    assert!(batch.placed_text().is_empty());
    let families = batch.native_inline_box_families().collect::<Vec<_>>();
    assert_eq!(families.len(), 1);
    assert_eq!(families[0].formatter(), 0);
    assert_eq!(
        atom_positions(batch),
        [
            (1.0, 0.0),
            (11.0, 0.0),
            (21.0, 0.0),
            (51.0, 0.0),
            (41.0, 0.0),
            (31.0, 0.0)
        ]
    );
    let c = box_piece(batch, 12);
    let a = box_piece(batch, 13);
    assert_eq!(member_ids(c), [(12, 4, 2)]);
    assert_eq!(member_ids(a), [(13, 5, 3)]);
    assert_ne!(c.fragment(), a.fragment());
    assert_eq!(c.fragment().parent(), a.fragment().parent());
    assert_eq!(
        c.fragment().parent().unwrap().owner(),
        GeneratedBoxRef::Node(0)
    );
    assert_eq!(c.executed_edges().count(), 0);
    assert_eq!(a.executed_edges().count(), 0);
    // These actual zero Contributions are source aliases. The odd member's
    // End alias is51, not its physical border end61 and not the old fence41.
    boundary_position(batch, 20, 21.0, 0.0, 0.0);
    boundary_position(batch, 21, 51.0, 0.0, 0.0);
    let actual_float = output(batch, 30);
    assert_eq!(actual_float.location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(actual_float.size, Size::new(n(1.0), n(10.0)));
    assert_eq!(actual_float.source_index.get(), 0);
}

fn warm_native<S: LayoutScalar>(opaque_descendant: bool) {
    let mut f = prepared_fixture::<S>([0, 0, 0, 1, 1, 1], false);
    contribution_edges(&mut f, 0.0, 0.0);
    // A material leading rectangle creates the real shape-backed container
    // profile eligible for complete subtree geometry-cache lookup/replay.
    f.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            float_exclusion: crate::FloatExclusion::Shape,
            size: Size::new(PreferredSizeOf::px(n(1.0)), PreferredSizeOf::px(n(10.0))),
            ..NodeInputOf::default()
        }),
    );
    f.nodes.insert(0, 30);
    f.finish();
    f.wrapping
        .child_order
        .iter_mut()
        .find(|row| row.parent == GeneratedBoxRef::Node(0))
        .unwrap()
        .children
        .insert(
            0,
            GeneratedChildBoxInput {
                box_: GeneratedBoxRef::Node(30),
                participation: GeneratedBoxParticipation::OutOfFlow,
            },
        );
    assert_eq!(f.nodes, [30, 10, 11, 20, 12, 13, 21, 14, 15]);
    assert_eq!(f.tree.node_input(30).atomic_inline_participation, None);
    // Keep the same real ten-pixel atomic, but obtain its dimensions through
    // the maintained numeric leaf provider. This creates a genuine fallible
    // owner operation for invalidation/retry, not a fabricated geometry receipt.
    let mut measured = f.tree.node_input(14).clone();
    measured.size = Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO);
    f.tree.insert_input(14, LayoutInputOf::box_input(measured));
    f.tree = f.tree.measure(14, Size::splat(n(10.0)));
    let root = if opaque_descendant {
        // Actual opaque InlineBlock0 owns its own formatter and real children.
        // Ancestor100 sees that atomic, never expands its six inner operands.
        let mut inner = f.tree.node_input(0).clone();
        inner.display = crate::Display::InlineBlock;
        // The actual child owns a100px IFC. This excludes an unrelated
        // shrink-to-fit decision from the leading-float cache qualifier.
        inner.size.width = PreferredSizeOf::px(n(100.0));
        inner.atomic_inline_participation = Some(AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            InlineBreakOpportunity::Prohibited,
        ));
        f.tree.insert_input(0, LayoutInputOf::box_input(inner));
        f.tree.insert_children(100, [0]);
        f.tree.insert_input(
            100,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::Block,
                direction: Direction::Ltr,
                ..NodeInputOf::default()
            }),
        );
        100
    } else {
        0
    };
    let mut wrapping = f.wrapping;
    wrapping
        .native_inline_boxes
        .push(NativeInlineBoxFamilyInput { formatter: 0 });
    let host = Host::new(f.tree);
    seed_and_observe_root(&host, root, Some(30));
    assert!(
        host.measured.borrow().contains(&14),
        "the measured-leaf fixture premise must execute"
    );
    let good = admit(&host, root, wrapping);
    let first = compute_layout_with_flow(&good, root, request(n(100.0))).unwrap();
    check_family(&first);
    let c = box_piece(&first, 12).fragment();
    let a = box_piece(&first, 13).fragment();
    let mut sink = host.sink();
    first.apply_to(&mut sink).unwrap();
    let repeat = compute_layout_with_flow(&good, root, request(n(100.0))).unwrap();
    check_family(&repeat);
    assert_eq!(box_piece(&repeat, 12).fragment(), c);
    assert_eq!(box_piece(&repeat, 13).fragment(), a);
    assert_eq!(
        repeat.native_inline_box_families().collect::<Vec<_>>(),
        first.native_inline_box_families().collect::<Vec<_>>()
    );
    repeat.apply_to(&mut sink).unwrap();
    let before = host.committed.borrow().clone();
    host.fail.set(Some(14));
    host.measured.borrow_mut().clear();
    let error = crate::compute_layout_with_flow_invalidated(&good, root, request(n(100.0)), &[14])
        .unwrap_err();
    assert_eq!(error.site(), crate::LayoutErrorSiteOf::Node(14));
    assert_eq!(error.operation(), crate::LayoutOperation::LeafMeasurement);
    assert!(
        matches!(
            error.kind(),
            crate::LayoutErrorKindOf::Measurement(LeafFailure(14))
        ),
        "{error:?}"
    );
    assert!(host.measured.borrow().contains(&14));
    assert_eq!(host.committed.borrow().caches, before.caches);
    assert_eq!(host.committed.borrow().unrounded, before.unrounded);
    {
        let state = host.committed.borrow();
        let accepted = state.batch.as_ref().unwrap();
        assert_eq!(
            accepted.final_entries(),
            before.batch.as_ref().unwrap().final_entries()
        );
        assert_eq!(
            accepted.native_inline_box_families().collect::<Vec<_>>(),
            before
                .batch
                .as_ref()
                .unwrap()
                .native_inline_box_families()
                .collect::<Vec<_>>()
        );
        check_family(accepted);
    }
    // A failed computation publishes no batch. Reusing the SAME original good
    // admitted input after recovery must retain its family and global IDs.
    host.fail.set(None);
    host.measured.borrow_mut().clear();
    let retry =
        crate::compute_layout_with_flow_invalidated(&good, root, request(n(100.0)), &[14]).unwrap();
    assert!(host.measured.borrow().contains(&14));
    check_family(&retry);
    assert_eq!(box_piece(&retry, 12).fragment(), c);
    assert_eq!(box_piece(&retry, 13).fragment(), a);
    retry.apply_to(&mut sink).unwrap();
    check_family(&first);
    assert_eq!(box_piece(&first, 12).fragment(), c);
    assert_eq!(box_piece(&first, 13).fragment(), a);
}

fn empty_completion<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new().children(0, []).style(
        0,
        NodeInputOf {
            display: crate::Display::Block,
            direction: Direction::Ltr,
            ..NodeInputOf::default()
        },
    );
    let host = Host::new(tree);
    seed_and_observe_root(&host, 0, None);
    let ordinary = host.committed.borrow().batch.as_ref().unwrap().clone();
    assert_eq!(output(&ordinary, 0).size, Size::new(n(100.0), S::ZERO));
    let wrapping = WrappingInput {
        native_inline_boxes: vec![NativeInlineBoxFamilyInput { formatter: 0 }],
        scopes: vec![],
        gaps: vec![],
        child_order: vec![order(GeneratedBoxRef::Node(0), vec![])],
    };
    let demanded = admit(&host, 0, wrapping);
    let first = compute_layout_with_flow(&demanded, 0, request(n(100.0))).unwrap();
    assert_eq!(first.native_inline_box_families().count(), 1);
    assert_eq!(
        first
            .native_inline_box_families()
            .next()
            .unwrap()
            .formatter(),
        0
    );
    assert_eq!(
        first.native_inline_box_pieces().count(),
        0,
        "an empty completion cannot fabricate a selected/formatted line"
    );
    assert!(first.placed_text().is_empty());
    assert_eq!(output(&first, 0).size, Size::new(n(100.0), S::ZERO));
    let mut sink = host.sink();
    first.apply_to(&mut sink).unwrap();
    let repeated = compute_layout_with_flow(&demanded, 0, request(n(100.0))).unwrap();
    assert_eq!(
        repeated.native_inline_box_families().collect::<Vec<_>>(),
        first.native_inline_box_families().collect::<Vec<_>>()
    );
    assert_eq!(repeated.native_inline_box_pieces().count(), 0);
    repeated.apply_to(&mut sink).unwrap();
    assert_eq!(
        host.committed
            .borrow()
            .batch
            .as_ref()
            .unwrap()
            .native_inline_box_families()
            .count(),
        1
    );
    // The previously completed absence remains absence; no fake empty fragment
    // or source occurrence is retrofitted into that immutable publication.
    assert_eq!(ordinary.native_inline_box_families().count(), 0);
    assert_eq!(ordinary.native_inline_box_pieces().count(), 0);
}

#[test]
fn observed_host_root_cache_cannot_omit_demanded_native_family_and_retry() {
    warm_native::<f32>(false);
    warm_native::<f64>(false);
}
#[test]
fn observed_ancestor_cache_cannot_omit_its_demanded_opaque_descendant_family() {
    warm_native::<f32>(true);
    warm_native::<f64>(true);
}
#[test]
fn genuine_empty_formatter_publishes_demanded_completion_without_selected_pieces() {
    empty_completion::<f32>();
    empty_completion::<f64>();
}
