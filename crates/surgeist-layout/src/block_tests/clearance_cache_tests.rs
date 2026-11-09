//! Real host caching, request rebinding and immutable-batch qualification for
//! #2125. Geometry follows the independently retained CSS2/policy oracles in
//! in_flow_margins_tests and owning comments6728652731/6728787611/6733332381.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Clone)]
struct Published<S: LayoutScalar> {
    caches: HashMap<u32, CacheOf<S>>,
    unrounded: HashMap<u32, LayoutOutputEntryOf<u32, S>>,
    visible: Option<CompletedLayoutBatchOf<u32, S>>,
}

struct Host<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    published: Published<S>,
    targets: LayoutPublicationTargets<u32>,
    hits: RefCell<Vec<(u32, RunMode)>>,
}

impl<S: LayoutScalar> Host<S> {
    fn new(clear: Clear) -> Self {
        Self {
            tree: multioffer_tree(clear),
            published: Published {
                caches: HashMap::new(),
                unrounded: HashMap::new(),
                visible: None,
            },
            targets: LayoutPublicationTargets::new(),
            hits: RefCell::new(Vec::new()),
        }
    }

    fn install(&mut self, request: &LayoutBoundRequestOf<S>) {
        self.targets.install(0, request.target_basis()).unwrap();
    }

    fn set_block_clear(&mut self, clear: Clear) {
        let mut style = self.tree.node_input(5).clone();
        style.clear = clear;
        self.tree.insert_input(5, LayoutInputOf::box_input(style));
    }

    fn compute(
        &self,
        request: &LayoutBoundRequestOf<S>,
        changed: &[u32],
    ) -> CompletedLayoutBatchOf<u32, S> {
        let capture = LayoutFlowInput::try_new_bound(
            self,
            0,
            request.clone(),
            vec![],
            WrappingInput::default(),
            vec![],
        )
        .unwrap();
        compute_layout_with_flow_invalidated(&capture, 0, request.root_request(), changed).unwrap()
    }

    fn compute_ordinary(&self, request: LayoutRootRequestOf<S>) -> CompletedLayoutBatchOf<u32, S> {
        compute_layout(self, 0, request).unwrap()
    }

    fn performed_warm_after(
        &mut self,
        cold: &CompletedLayoutBatchOf<u32, S>,
        request: LayoutRootRequestOf<S>,
    ) -> CompletedLayoutBatchOf<u32, S> {
        // Bound continuous requests currently bypass every cache operation.
        // Ordinary shared-BFC children also intentionally bypass performed
        // reuse. The independent following leaf supplies the supported performed
        // cache path while the adjoining component recomputes coherently.
        cold.apply_to(self).unwrap();
        self.hits.borrow_mut().clear();
        let warm = self.compute_ordinary(request);
        let hits = self.hits.borrow();
        eprintln!(
            "actual matching cache hits {}: {hits:?}",
            std::any::type_name::<S>()
        );
        assert!(
            hits.iter()
                .any(|(node, mode)| *node == 6 && *mode == RunMode::PerformLayout),
            "the actual independent following leaf must return a matching performed CacheOf answer, not only a float-size query: {hits:?}"
        );
        warm
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
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let answer = self
            .published
            .caches
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        if answer.is_some() {
            self.hits.borrow_mut().push((node, input.run_mode()));
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<LayoutOutputEntryOf<u32, S>> {
        self.published.unrounded.get(&node).copied()
    }
}

impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Host<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn publication_targets(&self) -> &LayoutPublicationTargets<u32> {
        &self.targets
    }
    fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<u32> {
        &mut self.targets
    }
    fn prepare_layout_batch(
        &self,
        preparation: LayoutBatchPreparation<'_, u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let batch = preparation.batch();
        let mut next = self.published.clone();
        // Whole-root replacement retires performed reuse for previous and new
        // members, while preserving unaffected numeric measurements. Explicit
        // invalidations clear both cache classes before installing actual stores.
        for &node in preparation.previous_members() {
            if batch.replaces_whole_root() || batch.node_layout(node).is_none() {
                if let Some(cache) = next.caches.get_mut(&node) {
                    cache.clear_performed();
                }
                next.unrounded.remove(&node);
            }
        }
        if batch.replaces_whole_root() {
            for (node, _) in batch.node_layouts() {
                if let Some(cache) = next.caches.get_mut(&node) {
                    cache.clear_performed();
                }
            }
        }
        for &node in batch.invalidated_nodes() {
            next.caches.remove(&node);
            next.unrounded.remove(&node);
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
        for (node, view) in batch.node_layouts() {
            next.unrounded.insert(
                node,
                view.single_box()
                    .expect("fixture contains real continuous boxes")
                    .unrounded_entry(),
            );
        }
        next.visible = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch(&mut self, commit: LayoutBatchCommit<'_, u32, S, Self::Prepared>) {
        self.published = commit.into_prepared();
    }
}

fn block<S: LayoutScalar>(height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::AUTO,
            height.map_or(PreferredSizeOf::AUTO, |h| {
                PreferredSizeOf::px(S::from_f64(h))
            }),
        ),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..NodeInputOf::default()
    }
}

fn margins<S: LayoutScalar>(top: f64, bottom: f64) -> Edges<LengthAutoOf<S>> {
    Edges {
        top: LengthAutoOf::px(S::from_f64(top)),
        bottom: LengthAutoOf::px(S::from_f64(bottom)),
        ..Edges::all(LengthAutoOf::ZERO)
    }
}

fn multioffer_tree<S: LayoutScalar>(clear: Clear) -> PublicLayoutTreeOf<S> {
    let float = |side, height| NodeInputOf {
        float: side,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(50.0)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..block(None)
    };
    PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 6])
        .children(1, [])
        .children(2, [])
        .children(3, [4, 5])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(0, block(None))
        .style(1, float(Float::Left, 15.0))
        .style(2, float(Float::Right, 25.0))
        .style(3, block(None))
        .style(
            4,
            NodeInputOf {
                clear: Clear::Left,
                margin: margins(8.0, 12.0),
                ..block(None)
            },
        )
        .style(
            5,
            NodeInputOf {
                clear,
                margin: margins(20.0, 0.0),
                ..block(Some(10.0))
            },
        )
        .style(
            6,
            NodeInputOf {
                // This zero-margin H2 sibling starts below both floats in every
                // oracle. Its independent BFC enables legitimate performed reuse
                // while its final placement still depends on the clearing parent.
                block_flow_context: BlockFlowContext::Independent,
                ..block(Some(2.0))
            },
        )
}

fn root_request<S: LayoutScalar>(width: f64) -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(width)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ))
}

fn bound<S: LayoutScalar>(source: LayoutSourceBasis, width: f64) -> LayoutBoundRequestOf<S> {
    LayoutBoundRequestOf::continuous(source, root_request(width))
}

fn assert_horizontal<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, width: f64) {
    let px = S::from_f64;
    for (node, view) in batch.node_layouts() {
        let single = view.single_box().unwrap();
        for output in [single.unrounded_output(), single.final_output()] {
            let (x, expected_width) = match node {
                1 => (0.0, 50.0),
                2 => (width - 50.0, 50.0),
                _ => (0.0, width),
            };
            assert_eq!(
                (output.location.x, output.size.width),
                (px(x), px(expected_width)),
                "node{node} horizontal geometry"
            );
        }
    }
}

fn assert_geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    width: f64,
    cleared: bool,
) {
    assert_horizontal(batch, width);
    // AC6733332381: E's qualified clearance7 is retained for B's contextual
    // hypothetical, H_B=7+max(8,12,20)=27. With right25, P0/E15/B27/H37.
    // Actual authored B:none instead freshly qualifies E at H20>left15:
    // P20/E=Bglobal20/H10. All expected values precede cache observations.
    let (parent_y, parent_height, empty_y, block_y, following_y) = if cleared {
        (0.0, 37.0, 15.0, 27.0, 37.0)
    } else {
        (20.0, 10.0, 0.0, 0.0, 30.0)
    };
    for (node, y, height) in [
        (1, 0.0, 15.0),
        (2, 0.0, 25.0),
        (3, parent_y, parent_height),
        (4, empty_y, 0.0),
        (5, block_y, 10.0),
        (6, following_y, 2.0),
        (0, 0.0, following_y + 2.0),
    ] {
        let single = batch.node_layout(node).unwrap().single_box().unwrap();
        for output in [single.unrounded_output(), single.final_output()] {
            assert_eq!(
                (output.location.y, output.size.height),
                (S::from_f64(y), S::from_f64(height)),
                "node{node} clearance geometry"
            );
        }
    }
}

fn warm_same_input<S: LayoutScalar>() {
    let mut host = Host::<S>::new(Clear::Right);
    let request = root_request(100.0);
    let cold = host.compute_ordinary(request);
    assert!(
        host.hits.borrow().is_empty(),
        "cold host has no retained entries"
    );
    // Seed before checking the known-wrong cold geometry, so RED reaches
    // actual matching performed cache hits and the warm geometry oracle.
    let warm = host.performed_warm_after(&cold, request);
    assert_geometry(&warm, 100.0, true);
    assert_geometry(&cold, 100.0, true);
    assert_eq!(
        warm.node_layout(3)
            .unwrap()
            .single_box()
            .unwrap()
            .unrounded_output(),
        cold.node_layout(3)
            .unwrap()
            .single_box()
            .unwrap()
            .unrounded_output()
    );
}

fn authored_style_rebind<S: LayoutScalar>() {
    let mut host = Host::<S>::new(Clear::Right);
    let old_request = bound(LayoutSourceBasis::fresh(), 100.0);
    host.install(&old_request);
    let old = host.compute(&old_request, &[]);
    old.apply_to(&mut host).unwrap();
    host.set_block_clear(Clear::None);
    let new_request = bound(LayoutSourceBasis::fresh(), 100.0);
    host.install(&new_request);
    // B's authored clear affects E's hypothetical and the whole adjoining P
    // component. Declare P/E/B explicitly; CacheKeyContext remains unchanged.
    let rebound = host.compute(&new_request, &[3, 4, 5]);
    assert_eq!(rebound.invalidated_nodes(), &[0, 3, 4, 5]);
    rebound.apply_to(&mut host).unwrap();
    let repeated = host.compute(&new_request, &[]);
    let fresh = Host::<S>::new(Clear::None).compute(&new_request, &[]);
    // Bound repeated computation is intentionally not claimed as a cache hit.
    assert_geometry(&rebound, 100.0, false);
    assert_geometry(&repeated, 100.0, false);
    assert_geometry(&fresh, 100.0, false);
}

fn changed_request<S: LayoutScalar>() {
    let mut host = Host::<S>::new(Clear::Right);
    let source = LayoutSourceBasis::fresh();
    let old_request = bound(source.clone(), 100.0);
    host.install(&old_request);
    let old = host.compute(&old_request, &[]);
    old.apply_to(&mut host).unwrap();
    let new_request = bound(source, 120.0);
    assert_ne!(old_request.target_basis(), new_request.target_basis());
    host.install(&new_request);
    let rebound = host.compute(&new_request, &[]);
    rebound.apply_to(&mut host).unwrap();
    let repeated = host.compute(&new_request, &[]);
    assert_horizontal(&repeated, 120.0);
    assert_geometry(&repeated, 120.0, true);
    assert_geometry(&rebound, 120.0, true);
    assert_geometry(&old, 100.0, true);
}

fn prior_batch_stays_immutable<S: LayoutScalar>() {
    let mut host = Host::<S>::new(Clear::None);
    let original_request = bound(LayoutSourceBasis::fresh(), 100.0);
    host.install(&original_request);
    let original = host.compute(&original_request, &[]);
    assert_geometry(&original, 100.0, false);
    original.apply_to(&mut host).unwrap();
    host.set_block_clear(Clear::Right);
    let changed_source = LayoutSourceBasis::fresh();
    let changed = bound(changed_source.clone(), 100.0);
    host.install(&changed);
    let cleared = host.compute(&changed, &[3, 4, 5]);
    cleared.apply_to(&mut host).unwrap();
    // Inspect the independent old oracle immediately after the later cleared
    // batch is applied, before any assertion on that later batch's geometry.
    assert_geometry(&original, 100.0, false);
    let wider = bound(changed_source, 120.0);
    host.install(&wider);
    let rebound = host.compute(&wider, &[]);
    rebound.apply_to(&mut host).unwrap();
    // Recheck after source/constraint/cache/visible replacements; no equality
    // to a captured actual snapshot substitutes for these exact old numbers.
    assert_geometry(&original, 100.0, false);
    assert_horizontal(host.published.visible.as_ref().unwrap(), 120.0);
}

#[test]
fn warm_same_input_f32() {
    warm_same_input::<f32>();
}

#[test]
fn warm_same_input_f64() {
    warm_same_input::<f64>();
}

#[test]
fn authored_style_rebind_f32() {
    authored_style_rebind::<f32>();
}

#[test]
fn authored_style_rebind_f64() {
    authored_style_rebind::<f64>();
}

#[test]
fn changed_request_f32() {
    changed_request::<f32>();
}

#[test]
fn changed_request_f64() {
    changed_request::<f64>();
}

#[test]
fn prior_batch_stays_immutable_f32() {
    prior_batch_stays_immutable::<f32>();
}

#[test]
fn prior_batch_stays_immutable_f64() {
    prior_batch_stays_immutable::<f64>();
}
