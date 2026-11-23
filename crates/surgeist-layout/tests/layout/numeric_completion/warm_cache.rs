//! Eligible leaf caching alongside uncached numeric phantom-root execution.
use super::*;
use std::cell::Cell;

#[derive(Clone)]
struct Published<S: l::scalar::LayoutScalar> {
    caches: Vec<l::cache::CacheOf<S>>,
    unrounded: Vec<Option<l::batch::LayoutOutputEntryOf<u32, S>>>,
    final_output: Vec<Option<crate::layout::observation::Observation<S>>>,
}
struct CachedNumeric<S: l::scalar::LayoutScalar> {
    actual: NumericTree<S>,
    published: Published<S>,
    publication_targets: l::batch::LayoutPublicationTargets<u32>,
    cache_calls: Vec<Cell<usize>>,
    cache_hits: Vec<Cell<usize>>,
    last_answer: Vec<Cell<Option<l::cache::ComputeOutputOf<S>>>>,
    with_leaf: bool,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for CachedNumeric<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::tree::Traverse::children(&self.actual, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::tree::Traverse::child_count(&self.actual, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::tree::Traverse::child(&self.actual, node, index)
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for CachedNumeric<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        l::tree::LayoutTree::node_input(&self.actual, node)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        l::tree::LayoutTree::layout_input(&self.actual, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::cache::ComputeInputOf<S>,
        context: l::cache::CacheKeyContext,
    ) -> Option<l::cache::ComputeOutputOf<S>> {
        let index = node as usize;
        self.cache_calls[index].set(self.cache_calls[index].get() + 1);
        let answer = self.published.caches[index].get_with_context(input, context);
        if let Some(output) = answer {
            self.cache_hits[index].set(self.cache_hits[index].get() + 1);
            self.last_answer[index].set(Some(output));
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::batch::LayoutOutputEntryOf<u32, S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutBatchSink<u32, S> for CachedNumeric<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn publication_targets(&self) -> &l::batch::LayoutPublicationTargets<u32> {
        &self.publication_targets
    }
    fn publication_targets_mut(&mut self) -> &mut l::batch::LayoutPublicationTargets<u32> {
        &mut self.publication_targets
    }
    fn prepare_layout_batch<'call>(
        &self,
        preparation: l::batch::LayoutBatchPreparation<'call, u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let batch = preparation.batch();
        let mut next = self.published.clone();
        for &node in preparation
            .previous_members()
            .iter()
            .filter(|&&node| batch.node_layout(node).is_none())
        {
            let index = node as usize;
            next.caches[index].clear_performed();
            next.unrounded[index] = None;
            next.final_output[index] = None;
        }
        for &node in batch.invalidated_nodes() {
            let index = node as usize;
            next.caches[index].clear();
            next.unrounded[index] = None;
            next.final_output[index] = None;
        }
        for entry in batch.cache_clear_entries() {
            next.caches[entry.node() as usize].clear();
        }
        for (node, view) in batch.node_layouts() {
            let entry = match view {
                l::fragmentation::NodeLayoutRefOf::SingleBox(view) => view.unrounded_entry(),
                l::fragmentation::NodeLayoutRefOf::NonBox(view) => view.unrounded_entry(),
                l::fragmentation::NodeLayoutRefOf::Fragments(_) => {
                    panic!("continuous fixture unexpectedly fragmented")
                }
            };
            next.unrounded[node as usize] = Some(entry);
        }
        for (node, _) in batch.node_layouts() {
            next.final_output[node as usize] = Some(crate::layout::observation::observe(
                (batch, true),
                node,
                crate::layout::observation::expected_kind(
                    l::tree::LayoutTree::layout_input(&self.actual, node),
                    false,
                ),
            ));
        }
        for entry in batch.cache_store_entries() {
            next.caches[entry.node() as usize].store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        Ok(next)
    }
    fn commit_layout_batch<'call>(
        &mut self,
        commit: l::batch::LayoutBatchCommit<'call, u32, S, Self::Prepared>,
    ) {
        let prepared = commit.into_prepared();
        self.published = prepared;
    }
}
impl<S: l::scalar::LayoutScalar> CachedNumeric<S> {
    fn new(direction: l::geometry::Direction, with_leaf: bool) -> Self {
        let mut participants = vec![
            boundary(direction, l::inline::InlineBoundaryKind::Start),
            boundary(direction, l::inline::InlineBoundaryKind::End),
        ];
        if with_leaf {
            // A Block separator outside the phantom inline run, NOT an
            // atomic participant that would itself make that line formatted.
            participants.push(l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
                display: l::box_model::Display::Block,
                direction,
                size: l::geometry::Size::new(
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::sizing::PreferredSizeOf::value(
                        l::length::LengthPercentageOf::px(S::from_f64(5.0)).unwrap(),
                    ),
                ),
                ..l::node::NodeInputOf::default()
            }));
        }
        let actual = NumericTree::<S>::new(direction, participants);
        let count = actual.inputs.len();
        Self {
            actual,
            publication_targets: l::batch::LayoutPublicationTargets::new(),
            published: Published {
                caches: vec![l::cache::CacheOf::new(); count],
                unrounded: vec![None; count],
                final_output: vec![None; count],
            },
            cache_calls: (0..count).map(|_| Cell::new(0)).collect(),
            cache_hits: (0..count).map(|_| Cell::new(0)).collect(),
            last_answer: (0..count).map(|_| Cell::new(None)).collect(),
            with_leaf,
        }
    }
    fn compute(&self, changed: &[u32]) -> l::batch::CompletedLayoutBatchOf<u32, S> {
        let request = {
            let sizing_request = crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(Some(S::from_f64(60.0)), None),
                surgeist_layout::geometry::Size::new(
                    surgeist_layout::constraint::SizingOperation::Normal,
                    surgeist_layout::constraint::SizingOperation::Normal,
                ),
            );
            l::request::LayoutRootRequestOf::standalone(
                sizing_request,
                surgeist_layout::geometry::FlowAxes::new(
                    surgeist_layout::tree::LayoutTree::node_input(self, 0).writing_mode,
                    surgeist_layout::tree::LayoutTree::node_input(self, 0).direction,
                ),
                sizing_request
                    .orthogonal_fallback()
                    .initial_containing_block(),
            )
        };
        l::compute_layout_invalidated(self, 0, request, changed).unwrap()
    }
    fn replace_pair(
        &mut self,
        direction: l::geometry::Direction,
        baseline: f64,
        extent: f64,
        positive_start: bool,
    ) {
        for (node, kind) in [
            (1, l::inline::InlineBoundaryKind::Start),
            (2, l::inline::InlineBoundaryKind::End),
        ] {
            let edge = if positive_start && node == 1 {
                l::inline::InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(2.0), S::ZERO)
                    .unwrap()
            } else {
                l::inline::InlineBoundaryEdgeOf::ZERO
            };
            self.actual.inputs[node] = l::node::LayoutInputOf::inline_boundary(
                l::inline::InlineBoundaryInputOf::contribution(
                    kind,
                    l::inline::InlineMetricsOf::try_new(
                        surgeist_layout::baseline::InlineBaselineContext::Physical,
                        S::from_f64(baseline),
                        S::from_f64(extent),
                    )
                    .unwrap(),
                    edge,
                )
                .unwrap()
                .with_direction(direction),
            );
        }
    }
    fn assert_metrics(&self, baseline: f64, extent: f64) {
        for node in [1, 2] {
            let l::node::LayoutInputViewOf::InlineBoundary(input) =
                l::tree::LayoutTree::layout_input(&self.actual, node)
            else {
                panic!("actual matched endpoints required");
            };
            assert_eq!(
                input.metrics(),
                l::inline::InlineMetricsOf::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    S::from_f64(baseline),
                    S::from_f64(extent)
                )
                .unwrap()
            );
        }
    }
    fn assert_root_bypass(&self, batch: &l::batch::CompletedLayoutBatchOf<u32, S>) {
        // The ordinary nonleaf root has no shape-provider dependency. Its
        // intentional current cache policy reexecutes it; this is not a hit.
        assert_eq!(self.cache_calls[0].get(), 0);
        assert_eq!(self.cache_hits[0].get(), 0);
        assert!(
            !batch
                .cache_store_entries()
                .iter()
                .any(|entry| entry.node() == 0)
        );
        assert!(self.published.caches[0].is_empty());
    }
}
fn assert_numeric_state<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
    direction: l::geometry::Direction,
    positive: bool,
    with_leaf: bool,
) {
    assert_numeric_only(batch);
    let line_height = if positive { 30.0 } else { 0.0 };
    for entries in [
        crate::layout::observation::phase(batch, false),
        crate::layout::observation::phase(batch, true),
    ] {
        assert_eq!(
            node(entries, 0).size,
            l::geometry::Size::new(
                S::from_f64(60.0),
                S::from_f64(line_height + if with_leaf { 5.0 } else { 0.0 })
            )
        );
        for (id, index) in [(1, 0), (2, 1)] {
            let actual = node_of_kind(
                entries,
                id,
                crate::layout::observation::Kind::NonBox(
                    l::fragmentation::NonBoxKind::InlineBoundary,
                ),
            );
            assert_eq!(actual.source_index, l::node::SourceIndex::new(index));
            assert_eq!(
                actual.location.y,
                if positive { S::from_f64(16.0) } else { S::ZERO }
            );
            assert_eq!(actual.padding, l::geometry::Edges::all(S::ZERO));
            if positive && id == 1 {
                assert_eq!(
                    actual.size,
                    l::geometry::Size::new(S::from_f64(2.0), S::ZERO)
                );
                assert_eq!(
                    actual.border,
                    if direction == l::geometry::Direction::Ltr {
                        l::geometry::Edges {
                            left: S::from_f64(2.0),
                            ..l::geometry::Edges::all(S::ZERO)
                        }
                    } else {
                        l::geometry::Edges {
                            right: S::from_f64(2.0),
                            ..l::geometry::Edges::all(S::ZERO)
                        }
                    }
                );
            } else {
                assert_eq!(actual.size, l::geometry::Size::ZERO);
                assert_eq!(actual.border, l::geometry::Edges::all(S::ZERO));
            }
        }
        if with_leaf {
            let leaf = node(entries, 3);
            assert_eq!(leaf.source_index, l::node::SourceIndex::new(2));
            assert_eq!(
                leaf.size,
                l::geometry::Size::new(S::from_f64(60.0), S::from_f64(5.0))
            );
            assert_eq!(
                leaf.location,
                l::geometry::Point::new(S::ZERO, S::from_f64(line_height))
            );
        }
    }
}
fn assert_same_numeric_outputs<S: l::scalar::LayoutScalar>(
    a: &l::batch::CompletedLayoutBatchOf<u32, S>,
    b: &l::batch::CompletedLayoutBatchOf<u32, S>,
    count: usize,
) {
    for (left, right) in [
        (
            crate::layout::observation::phase(a, false),
            crate::layout::observation::phase(b, false),
        ),
        (
            crate::layout::observation::phase(a, true),
            crate::layout::observation::phase(b, true),
        ),
    ] {
        for id in 0..count {
            let kind = if id == 0 || id == 3 {
                crate::layout::observation::Kind::Box
            } else {
                crate::layout::observation::Kind::NonBox(
                    l::fragmentation::NonBoxKind::InlineBoundary,
                )
            };
            assert_eq!(
                node_of_kind(left, id as u32, kind),
                node_of_kind(right, id as u32, kind)
            );
        }
    }
}
fn assert_endpoint_invalidation<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
) {
    for id in [0, 1, 2] {
        assert!(batch.invalidated_nodes().contains(&id));
    }
}
fn repeat_after_publication<S: l::scalar::LayoutScalar>(
    host: &mut CachedNumeric<S>,
    direction: l::geometry::Direction,
    positive: bool,
) -> l::batch::CompletedLayoutBatchOf<u32, S> {
    for id in 0..host.actual.inputs.len() {
        assert!(host.published.unrounded[id].is_some());
        assert!(host.published.final_output[id].is_some());
    }
    let previous_leaf_hits = host.with_leaf.then(|| host.cache_hits[3].get());
    let repeated = host.compute(&[]);
    host.assert_root_bypass(&repeated);
    assert_numeric_state(&repeated, direction, positive, host.with_leaf);
    if let Some(before) = previous_leaf_hits {
        assert!(
            host.cache_hits[3].get() > before,
            "eligible real Block leaf returns an actual retained CacheOf answer during repeated compute"
        );
        assert_eq!(
            host.last_answer[3].get().unwrap().size,
            l::geometry::Size::new(S::from_f64(60.0), S::from_f64(5.0))
        );
    } else {
        assert!(
            host.cache_hits.iter().all(|hits| hits.get() == 0),
            "pure endpoint root repeats are explicitly uncached reexecution, not warm proof"
        );
    }
    repeated.apply_to(host).unwrap();
    repeated
}
fn numeric_cache_policy<S: l::scalar::LayoutScalar>(
    direction: l::geometry::Direction,
    with_leaf: bool,
) {
    let mut host = CachedNumeric::<S>::new(direction, with_leaf);
    host.assert_metrics(8.0, 10.0);
    let cold = host.compute(&[]);
    assert!(host.cache_hits.iter().all(|hits| hits.get() == 0));
    host.assert_root_bypass(&cold);
    assert_numeric_state(&cold, direction, false, with_leaf);
    if with_leaf {
        assert!(
            cold.cache_store_entries()
                .iter()
                .any(|entry| entry.node() == 3),
            "eligible leaf producer stores seed the host cache"
        );
    }
    cold.apply_to(&mut host).unwrap();
    let repeated = repeat_after_publication(&mut host, direction, false);
    assert_same_numeric_outputs(&cold, &repeated, host.actual.inputs.len());

    host.replace_pair(direction, 16.0, 30.0, false);
    host.assert_metrics(16.0, 30.0);
    let nominal = host.compute(&[1, 2]);
    assert_endpoint_invalidation(&nominal);
    host.assert_root_bypass(&nominal);
    assert_numeric_state(&nominal, direction, false, with_leaf);
    let nominal_fresh = host.actual.compute();
    assert_numeric_state(&nominal_fresh, direction, false, with_leaf);
    assert_same_numeric_outputs(&nominal, &nominal_fresh, host.actual.inputs.len());
    nominal.apply_to(&mut host).unwrap();
    // The refresh itself need not hit the sibling cache: keys/contexts may
    // differ. A repeated SAME phase after publication must hit it.
    let nominal_repeated = repeat_after_publication(&mut host, direction, false);
    assert_same_numeric_outputs(&nominal_fresh, &nominal_repeated, host.actual.inputs.len());

    host.replace_pair(direction, 16.0, 30.0, true);
    host.assert_metrics(16.0, 30.0);
    let positive = host.compute(&[1, 2]);
    assert_endpoint_invalidation(&positive);
    host.assert_root_bypass(&positive);
    assert_numeric_state(&positive, direction, true, with_leaf);
    let positive_fresh = host.actual.compute();
    assert_numeric_state(&positive_fresh, direction, true, with_leaf);
    assert_same_numeric_outputs(&positive, &positive_fresh, host.actual.inputs.len());
    positive.apply_to(&mut host).unwrap();
    let positive_repeated = repeat_after_publication(&mut host, direction, true);
    assert_same_numeric_outputs(
        &positive_fresh,
        &positive_repeated,
        host.actual.inputs.len(),
    );

    for old in [
        &cold,
        &repeated,
        &nominal,
        &nominal_fresh,
        &nominal_repeated,
    ] {
        assert_numeric_state(old, direction, false, with_leaf);
    }
    assert_same_numeric_outputs(&cold, &repeated, host.actual.inputs.len());
    assert_same_numeric_outputs(&nominal, &nominal_repeated, host.actual.inputs.len());
    assert_numeric_state(&positive, direction, true, with_leaf);
}
#[test]
fn pure_numeric_endpoint_root_reexecutes_uncached_after_explicit_metric_and_bp_changes() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_cache_policy::<f32>(direction, false);
        numeric_cache_policy::<f64>(direction, false);
    }
}
#[test]
fn actual_cached_block_leaf_keeps_numeric_phantom_and_positive_line_positions() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        numeric_cache_policy::<f32>(direction, true);
        numeric_cache_policy::<f64>(direction, true);
    }
}
