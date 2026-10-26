use super::*;
use std::cell::Cell;

#[derive(Clone)]
struct Published<S: l::LayoutScalar> {
    caches: Vec<l::CacheOf<S>>,
    unrounded: Vec<Option<l::NodeOutputOf<S>>>,
    final_output: Vec<Option<l::NodeOutputOf<S>>>,
    batch: Option<l::CompletedLayoutBatchOf<u32, S>>,
}
struct Host<S: l::LayoutScalar> {
    tree: Tree<S>,
    published: Published<S>,
    hits: Vec<Cell<usize>>,
}
impl<S: l::LayoutScalar> l::Traverse for Host<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.tree, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.tree, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for Host<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.tree, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.tree, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        let answer = self.published.caches[node as usize].get_with_context(input, context);
        if answer.is_some() {
            self.hits[node as usize].set(self.hits[node as usize].get() + 1);
        }
        answer
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        self.published.unrounded[node as usize]
    }
}
impl<S: l::LayoutScalar> l::LayoutBatchSink<u32, S> for Host<S> {
    type Error = Infallible;
    type Prepared = Published<S>;
    fn prepare_layout_batch(
        &self,
        batch: &l::CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut next = self.published.clone();
        for &id in batch.invalidated_nodes() {
            next.caches[id as usize].clear();
            next.unrounded[id as usize] = None;
            next.final_output[id as usize] = None;
        }
        for row in batch.cache_clear_entries() {
            next.caches[row.node() as usize].clear();
        }
        for row in batch.unrounded_entries() {
            next.unrounded[row.node() as usize] = Some(row.output());
        }
        for row in batch.final_entries() {
            next.final_output[row.node() as usize] = Some(row.output());
        }
        for row in batch.cache_store_entries() {
            next.caches[row.node() as usize].store_with_context(
                row.input(),
                row.context(),
                row.output(),
            );
        }
        next.batch = Some(batch.clone());
        Ok(next)
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.published = prepared;
    }
}
impl<S: l::LayoutScalar> Host<S> {
    fn new(direction: l::Direction) -> Self {
        let mode = l::WritingMode::HorizontalTb;
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::TextGroupAlign::Center;
        let tree = Tree::new(
            vec![
                l::LayoutInputOf::box_input(root),
                l::LayoutInputOf::box_input(block(mode, direction, 100.0)),
                atomic(mode, direction, 60.0),
                l::LayoutInputOf::box_input(block(mode, direction, 100.0)),
                atomic(mode, direction, 40.0),
            ],
            &[(0, &[1, 3]), (1, &[2]), (3, &[4])],
        );
        Self {
            tree,
            published: Published {
                caches: vec![l::CacheOf::new(); 5],
                unrounded: vec![None; 5],
                final_output: vec![None; 5],
                batch: None,
            },
            hits: (0..5).map(|_| Cell::new(0)).collect(),
        }
    }
    fn compute(&self, changed: &[u32]) -> l::CompletedLayoutBatchOf<u32, S> {
        let request = l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(n(100.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = l::compute_layout_invalidated(self, 0, request, changed).unwrap();
        numeric(&batch);
        batch
    }
    fn leaf_hits(&self) -> usize {
        self.hits[2].get() + self.hits[4].get()
    }
}
fn phase<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    width: f64,
    starts: [f64; 2],
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
        assert_eq!(node(entries, 1).location, l::Point::ZERO);
        assert_eq!(node(entries, 3).location, l::Point::new(S::ZERO, n(10.0)));
        assert_eq!(
            node(entries, 2).location,
            l::Point::new(n(starts[0]), S::ZERO)
        );
        assert_eq!(
            node(entries, 4).location,
            l::Point::new(n(starts[1]), S::ZERO)
        );
        assert_eq!(node(entries, 2).size, l::Size::new(n(width), n(10.0)));
        assert_eq!(node(entries, 4).size, l::Size::new(n(40.0), n(10.0)));
        assert_eq!(node(entries, 2).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 4).source_index, l::SourceIndex::new(0));
        assert_eq!(node(entries, 3).source_index, l::SourceIndex::new(1));
    }
}
fn same<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
) {
    for (left, right) in [
        (a.unrounded_entries(), b.unrounded_entries()),
        (a.final_entries(), b.final_entries()),
    ] {
        for id in 0..5 {
            assert_eq!(node(left, id), node(right, id));
        }
    }
}
fn cache_lane<S: l::LayoutScalar>(direction: l::Direction) {
    let mut host = Host::<S>::new(direction);
    let rtl = direction == l::Direction::Rtl;
    let old_starts = if rtl { [20.0, 40.0] } else { [20.0, 20.0] };
    let new_starts = if rtl { [10.0, 50.0] } else { [10.0, 10.0] };
    let cold = host.compute(&[]);
    assert_eq!(host.leaf_hits(), 0);
    phase(&cold, 60.0, old_starts);
    assert!(
        cold.cache_store_entries()
            .iter()
            .any(|row| row.node() == 2 || row.node() == 4),
        "actual eligible atomic sizing must seed the host cache"
    );
    cold.apply_to(&mut host).unwrap();
    let before = host.leaf_hits();
    let warm = host.compute(&[]);
    assert!(
        host.leaf_hits() > before,
        "warm proof is a real eligible CacheOf answer, never a repeated uncached root"
    );
    phase(&warm, 60.0, old_starts);
    same(&cold, &warm);
    warm.apply_to(&mut host).unwrap();
    // Only the limiting sibling changes; the peer's own inputs remain identical.
    host.tree.update(2, |item| item.size.width = px(80.0));
    let changed = host.compute(&[2]);
    for id in [0, 1, 2] {
        assert!(changed.invalidated_nodes().contains(&id));
    }
    phase(&changed, 80.0, new_starts);
    phase(host.published.batch.as_ref().unwrap(), 60.0, old_starts);
    let fresh = host.tree.compute(l::WritingMode::HorizontalTb, &[]);
    phase(&fresh, 80.0, new_starts);
    same(&changed, &fresh);
    changed.apply_to(&mut host).unwrap();
    let before = host.leaf_hits();
    let changed_warm = host.compute(&[]);
    assert!(host.leaf_hits() > before);
    same(&fresh, &changed_warm);
    changed_warm.apply_to(&mut host).unwrap();
    host.tree
        .update(0, |root| root.text_group_align = l::TextGroupAlign::None);
    let none = host.compute(&[0]);
    phase(&none, 80.0, if rtl { [20.0, 60.0] } else { [0.0, 0.0] });
    none.apply_to(&mut host).unwrap();
    host.tree
        .update(0, |root| root.text_group_align = l::TextGroupAlign::Right);
    let right = host.compute(&[0]);
    phase(&right, 80.0, if rtl { [20.0, 60.0] } else { [20.0, 20.0] });
    right.apply_to(&mut host).unwrap();
    host.tree
        .update(0, |root| root.text_group_align = l::TextGroupAlign::Center);
    let restored = host.compute(&[0]);
    phase(&restored, 80.0, new_starts);
    same(&changed, &restored);
    for old in [&cold, &warm] {
        phase(old, 60.0, old_starts);
    }
    phase(&changed, 80.0, new_starts);
    same(&cold, &warm);
}
#[test]
fn changed_limiting_sibling_repositions_warm_peers_without_mutating_old_batches() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        cache_lane::<f32>(direction);
        cache_lane::<f64>(direction);
    }
}
