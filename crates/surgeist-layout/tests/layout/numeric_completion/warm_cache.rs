//! Eligible leaf caching alongside uncached numeric phantom-root execution.
use super::*;
use std::cell::Cell;

#[derive(Clone)]
struct Published<S: l::LayoutScalar> {
    caches: Vec<l::CacheOf<S>>,
    unrounded: Vec<Option<l::NodeOutputOf<S>>>,
    final_output: Vec<Option<l::NodeOutputOf<S>>>,
}
struct CachedNumeric<S: l::LayoutScalar> {
    actual: NumericTree<S>,
    published: Published<S>,
    cache_calls: Vec<Cell<usize>>,
    cache_hits: Vec<Cell<usize>>,
    last_answer: Vec<Cell<Option<l::ComputeOutputOf<S>>>>,
    with_leaf: bool,
}
impl<S: l::LayoutScalar> l::Traverse for CachedNumeric<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.actual, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.actual, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.actual, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for CachedNumeric<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.actual, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.actual, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        let index = node as usize;
        self.cache_calls[index].set(self.cache_calls[index].get() + 1);
        let answer = self.published.caches[index].get_with_context(input, context);
        if let Some(output) = answer {
            self.cache_hits[index].set(self.cache_hits[index].get() + 1);
            self.last_answer[index].set(Some(output));
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::LayoutScalar> l::LayoutBatchSink<u32, S> for CachedNumeric<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn prepare_layout_batch(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut next = self.published.clone();
        for &node in batch.invalidated_nodes() {
            let index = node as usize;
            next.caches[index].clear();
            next.unrounded[index] = None;
            next.final_output[index] = None;
        }
        for entry in batch.cache_clear_entries() {
            next.caches[entry.node() as usize].clear();
        }
        for entry in batch.unrounded_entries() {
            next.unrounded[entry.node() as usize] = Some(entry.output());
        }
        for entry in batch.final_entries() {
            next.final_output[entry.node() as usize] = Some(entry.output());
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
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.published = prepared;
    }
}
impl<S: l::LayoutScalar> CachedNumeric<S> {
    fn new(direction: l::Direction, with_leaf: bool) -> Self {
        let mut participants = vec![
            boundary(direction, l::InlineBoundaryKind::Start),
            boundary(direction, l::InlineBoundaryKind::End),
        ];
        if with_leaf {
            // A genuine Block separator outside the phantom inline run, NOT an
            // atomic participant that would itself make that line formatted.
            participants.push(l::LayoutInputOf::box_input(l::NodeInputOf {
                display: l::Display::Block,
                direction,
                size: l::Size::new(
                    l::PreferredSizeOf::value(
                        l::LengthPercentageOf::px(S::from_f64(60.0)).unwrap(),
                    ),
                    l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(5.0)).unwrap()),
                ),
                ..l::NodeInputOf::default()
            }));
        }
        let actual = NumericTree::<S>::new(direction, participants);
        let count = actual.inputs.len();
        Self {
            actual,
            published: Published {
                caches: vec![l::CacheOf::new(); count],
                unrounded: vec![None; count],
                final_output: vec![None; count],
            },
            cache_calls: (0..count).map(|_| Cell::new(0)).collect(),
            cache_hits: (0..count).map(|_| Cell::new(0)).collect(),
            last_answer: (0..count).map(|_| Cell::new(None)).collect(),
            with_leaf,
        }
    }
    fn compute(&self, changed: &[u32]) -> l::CompletedLayoutBatchOf<u32, S> {
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(60.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        l::compute_layout_invalidated(self, 0, request, changed).unwrap()
    }
    fn replace_pair(
        &mut self,
        direction: l::Direction,
        baseline: f64,
        extent: f64,
        positive_start: bool,
    ) {
        for (node, kind) in [
            (1, l::InlineBoundaryKind::Start),
            (2, l::InlineBoundaryKind::End),
        ] {
            let edge = if positive_start && node == 1 {
                l::InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::ZERO).unwrap()
            } else {
                l::InlineBoundaryEdgeOf::ZERO
            };
            self.actual.inputs[node] = l::LayoutInputOf::inline_boundary(
                l::InlineBoundaryInputOf::new(
                    kind,
                    l::InlineMetricsOf::try_new(
                        surgeist_layout::InlineBaselineContext::Physical,
                        S::from_f64(baseline),
                        S::from_f64(extent),
                    )
                    .unwrap(),
                    edge,
                )
                .with_direction(direction),
            );
        }
    }
    fn assert_metrics(&self, baseline: f64, extent: f64) {
        for node in [1, 2] {
            let l::LayoutInputViewOf::InlineBoundary(input) =
                l::LayoutTree::layout_input(&self.actual, node)
            else {
                panic!("actual matched endpoints required");
            };
            assert_eq!(
                input.metrics(),
                l::InlineMetricsOf::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    S::from_f64(baseline),
                    S::from_f64(extent)
                )
                .unwrap()
            );
        }
    }
    fn assert_root_bypass(&self, batch: &l::CompletedLayoutBatchOf<u32, S>) {
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
fn assert_numeric_state<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    direction: l::Direction,
    positive: bool,
    with_leaf: bool,
) {
    assert_numeric_only(batch);
    let line_height = if positive { 30.0 } else { 0.0 };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(
            node(entries, 0).size,
            l::Size::new(
                S::from_f64(60.0),
                S::from_f64(line_height + if with_leaf { 5.0 } else { 0.0 })
            )
        );
        for (id, index) in [(1, 0), (2, 1)] {
            let actual = node(entries, id);
            assert_eq!(actual.source_index, l::SourceIndex::new(index));
            assert_eq!(
                actual.location.y,
                if positive { S::from_f64(16.0) } else { S::ZERO }
            );
            assert_eq!(actual.padding, l::Edges::all(S::ZERO));
            if positive && id == 1 {
                assert_eq!(actual.size, l::Size::new(S::from_f64(2.0), S::ZERO));
                assert_eq!(
                    actual.border,
                    if direction == l::Direction::Ltr {
                        l::Edges {
                            left: S::from_f64(2.0),
                            ..l::Edges::all(S::ZERO)
                        }
                    } else {
                        l::Edges {
                            right: S::from_f64(2.0),
                            ..l::Edges::all(S::ZERO)
                        }
                    }
                );
            } else {
                assert_eq!(actual.size, l::Size::ZERO);
                assert_eq!(actual.border, l::Edges::all(S::ZERO));
            }
        }
        if with_leaf {
            let leaf = node(entries, 3);
            assert_eq!(leaf.source_index, l::SourceIndex::new(2));
            assert_eq!(leaf.size, l::Size::new(S::from_f64(60.0), S::from_f64(5.0)));
            assert_eq!(
                leaf.location,
                l::Point::new(S::ZERO, S::from_f64(line_height))
            );
        }
    }
}
fn assert_same_numeric_outputs<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
    count: usize,
) {
    for (left, right) in [
        (a.unrounded_entries(), b.unrounded_entries()),
        (a.final_entries(), b.final_entries()),
    ] {
        for id in 0..count {
            assert_eq!(node(left, id as u32), node(right, id as u32));
        }
    }
}
fn assert_endpoint_invalidation<S: l::LayoutScalar>(batch: &l::CompletedLayoutBatchOf<u32, S>) {
    for id in [0, 1, 2] {
        assert!(batch.invalidated_nodes().contains(&id));
    }
}
fn repeat_after_publication<S: l::LayoutScalar>(
    host: &mut CachedNumeric<S>,
    direction: l::Direction,
    positive: bool,
) -> l::CompletedLayoutBatchOf<u32, S> {
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
            l::Size::new(S::from_f64(60.0), S::from_f64(5.0))
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
fn numeric_cache_policy<S: l::LayoutScalar>(direction: l::Direction, with_leaf: bool) {
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
            "genuine eligible leaf producer stores seed the host cache"
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
    // differ. A repeated SAME phase after publication must genuinely hit it.
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
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_cache_policy::<f32>(direction, false);
        numeric_cache_policy::<f64>(direction, false);
    }
}
#[test]
fn actual_cached_block_leaf_keeps_numeric_phantom_and_positive_line_positions() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        numeric_cache_policy::<f32>(direction, true);
        numeric_cache_policy::<f64>(direction, true);
    }
}
