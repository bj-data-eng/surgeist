//! Qualified carried metrics determine the accepted real float band.
use super::super::{Host, absent, atomic, n, node, px, request, root};
use std::{convert::Infallible, slice};
use surgeist_layout as l;
use surgeist_text::BidiLevel;

struct TriangleHost<S: l::LayoutScalar> {
    inner: Host<S>,
}

impl<S: l::LayoutScalar> l::Traverse for TriangleHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(&self.inner, node)
    }

    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(&self.inner, node)
    }

    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(&self.inner, node, index)
    }
}

impl<S: l::LayoutScalar> l::LayoutTree for TriangleHost<S> {
    type MeasureError = Infallible;

    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.inner, node)
    }

    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.inner, node)
    }

    fn has_leaf_measurement(&self, node: u32) -> bool {
        l::LayoutTree::has_leaf_measurement(&self.inner, node)
    }

    fn measure_leaf(
        &self,
        node: u32,
        input: l::LeafMeasureInputOf<S>,
    ) -> Option<Result<l::Size<S>, Self::MeasureError>> {
        l::LayoutTree::measure_leaf(&self.inner, node, input)
    }

    fn cache_context(&self) -> l::CacheKeyContext {
        l::LayoutTree::cache_context(&self.inner)
    }

    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        l::LayoutTree::cache_get(&self.inner, node, input, context)
    }

    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        l::LayoutTree::unrounded_layout(&self.inner, node)
    }

    fn float_exclusion_interval(
        &self,
        node: u32,
        query: l::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<l::FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        let margin_box = query.margin_box();
        assert_eq!(margin_box.origin(), l::Point::ZERO);
        assert_eq!(margin_box.size(), l::Size::new(n(30.0), n(30.0)));
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(l::WritingMode::HorizontalTb, l::Direction::Ltr)
        );

        // One immutable filled triangle has local vertices (0,0), (0,30),
        // (30,30). Its slice at y occupies x0..y. The union over ANY band
        // therefore ends at its clipped maximum y, independently of a line's
        // identity, baseline name, trial, query ordering or invocation count.
        let origin = margin_box.origin();
        let lo = query.band_minimum() - origin.y;
        let hi = query.band_maximum() - origin.y;
        if hi <= S::ZERO || lo >= n(30.0) {
            return Some(Ok(None));
        }
        let right = origin.x + hi.max(S::ZERO).min(n(30.0));
        Some(Ok(l::FloatExclusionIntervalOf::try_new(
            query, origin.x, right,
        )
        .expect(
            "finite triangle section belongs to the actual queried float",
        )))
    }
}

impl<S: l::LayoutScalar> TriangleHost<S> {
    fn new(tail: bool) -> Self {
        let mode = l::WritingMode::HorizontalTb;
        let direction = l::Direction::Ltr;
        let mut inner = Host::new(root(
            mode,
            direction,
            l::InlineBaselineContext::Named(l::BaselineName::Central),
        ));
        let float = l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: mode,
            direction,
            float: l::Float::Left,
            float_exclusion: l::FloatExclusion::Shape,
            size: l::Size::new(px(30.0), px(30.0)),
            ..l::NodeInputOf::default()
        };
        assert_eq!(inner.push(Some(0), l::LayoutInputOf::box_input(float)), 1);

        let mut first = atomic(mode, direction, 10.0, (0.0, 0.0));
        first.vertical_align = l::VerticalAlign::Top;
        first.atomic_inline_participation = Some(l::AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            l::InlineBreakOpportunity::Mandatory,
        ));
        assert_eq!(inner.push(Some(0), l::LayoutInputOf::box_input(first)), 2);
        if tail {
            let mut last = atomic(mode, direction, 6.0, (0.0, 0.0));
            last.size.width = px(80.0);
            assert_eq!(inner.push(Some(0), l::LayoutInputOf::box_input(last)), 3);
        }
        Self { inner }
    }

    fn compute(&self) -> l::CompletedLayoutBatchOf<u32, S> {
        // Compute through this wrapper so real float queries reach the shape
        // provider; inner.compute would omit the provider entirely.
        l::compute_layout_invalidated(self, 0, request(l::WritingMode::HorizontalTb), &[]).unwrap()
    }
}

fn assert_triangle_packet<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    tail: bool,
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), if tail { 4 } else { 3 });
        let root = node(entries, 0);
        assert_eq!(root.location, l::Point::ZERO);
        assert_eq!(root.size, l::Size::new(n(100.0), n(30.0)));
        assert_eq!(root.source_index, l::SourceIndex::new(0));
        let pair = root.named_baselines.get(l::BaselineName::Central);
        assert_eq!(pair.first(), l::BaselineFactOf::Unknown);
        if tail {
            match pair.last() {
                l::BaselineFactOf::Available(coordinate) => {
                    assert_eq!(coordinate.axis(), l::PhysicalAxis::Vertical);
                    assert_eq!(coordinate.offset(), n(15.0));
                }
                other => panic!("real receiving line supplies CentralY15, got {other:?}"),
            }
        } else {
            assert_eq!(pair.last(), l::BaselineFactOf::Unknown);
        }

        let float = node(entries, 1);
        assert_eq!(float.location, l::Point::ZERO);
        assert_eq!(float.size, l::Size::new(n(30.0), n(30.0)));
        assert_eq!(float.source_index, l::SourceIndex::new(0));
        absent(float.named_baselines);

        let first = node(entries, 2);
        assert_eq!(first.location, l::Point::new(n(10.0), S::ZERO));
        assert_eq!(first.size, l::Size::new(n(20.0), n(10.0)));
        assert_eq!(first.source_index, l::SourceIndex::new(1));
        absent(first.named_baselines);
        if tail {
            let last = node(entries, 3);
            assert_eq!(last.location, l::Point::new(n(20.0), n(12.0)));
            assert_eq!(last.size, l::Size::new(n(80.0), n(6.0)));
            assert_eq!(last.source_index, l::SourceIndex::new(2));
            absent(last.named_baselines);
        }
    }
    assert!(batch.placed_text().is_empty());
    assert!(batch.unrounded_inline_fragments().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
}

fn node_rows<S: l::LayoutScalar>(
    entries: &[l::LayoutOutputEntryOf<u32, S>],
) -> Vec<(u32, l::NodeOutputOf<S>)> {
    let mut rows = entries
        .iter()
        .map(|entry| (entry.node(), entry.output()))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0);
    rows
}

fn actual_receiver_uses_qualified_triangle_band<S: l::LayoutScalar>() {
    // Qualify the actual producer and its source geometry without a real
    // receiver. The float keeps root height30; phantom completion adds no line
    // or named authority. This separate tree has no source/output fabrication.
    let no_tail = TriangleHost::<S>::new(false).compute();
    assert_triangle_packet(&no_tail, false);
    let prior_no_tail = (
        node_rows(no_tail.unrounded_entries()),
        node_rows(no_tail.final_entries()),
    );

    // WM4 midpoint synthesis gives A5/5 and T3/3. The first Top-only band0..10
    // places A at x10. The receiver is10 high at y10..20, with CentralY15,
    // hence T(20,12): the contour leaves exactly80 units beside the float.
    // Relabelled raw10/0 would query10..23, leave77 and defer the80-wide T.
    // Final geometry detects a stale accepted band/deferral. It cannot prove
    // ordering of speculative queries followed by complete correct feedback.
    let mut host = TriangleHost::<S>::new(true);
    let cold = host.compute();
    assert_triangle_packet(&cold, true);
    let prior_cold = (
        node_rows(cold.unrounded_entries()),
        node_rows(cold.final_entries()),
    );
    cold.apply_to(&mut host.inner).unwrap();

    // The wrapper delegates the real CacheOf and committed unrounded packets.
    // A warm whole-subtree hit may need no provider call; callback counts and
    // cache bookkeeping rows are not the geometric contract.
    let warm = host.compute();
    assert_triangle_packet(&warm, true);
    assert_eq!(node_rows(warm.unrounded_entries()), prior_cold.0);
    assert_eq!(node_rows(warm.final_entries()), prior_cold.1);
    warm.apply_to(&mut host.inner).unwrap();

    // Later computation/publication cannot mutate previously completed values.
    assert_triangle_packet(&no_tail, false);
    assert_triangle_packet(&cold, true);
    assert_eq!(node_rows(no_tail.unrounded_entries()), prior_no_tail.0);
    assert_eq!(node_rows(no_tail.final_entries()), prior_no_tail.1);
    assert_eq!(node_rows(cold.unrounded_entries()), prior_cold.0);
    assert_eq!(node_rows(cold.final_entries()), prior_cold.1);
}

#[test]
fn actual_atomic_receiver_fits_the_qualified_triangle_band_f32() {
    actual_receiver_uses_qualified_triangle_band::<f32>();
}

#[test]
fn actual_atomic_receiver_fits_the_qualified_triangle_band_f64() {
    actual_receiver_uses_qualified_triangle_band::<f64>();
}
