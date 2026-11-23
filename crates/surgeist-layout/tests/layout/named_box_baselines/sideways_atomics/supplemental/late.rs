//! Real later float-band refusal retains every applied atomic channel.
use super::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}

struct ShapeHost<S: l::scalar::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    capacity: f64,
    refuse: Cell<bool>,
    queries: RefCell<Vec<(l::box_model::FloatExclusionQueryOf<S>, bool)>>,
}

impl<S: l::scalar::LayoutScalar> l::tree::Traverse for ShapeHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        l::tree::Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::tree::Traverse::child_count(&self.inner, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::tree::Traverse::child(&self.inner, node, index)
    }
}

impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for ShapeHost<S> {
    type MeasureError = Refusal;

    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        l::tree::LayoutTree::node_input(&self.inner, node)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        l::tree::LayoutTree::layout_input(&self.inner, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::cache::ComputeInputOf<S>,
        context: l::cache::CacheKeyContext,
    ) -> Option<l::cache::ComputeOutputOf<S>> {
        l::tree::LayoutTree::cache_get(&self.inner, node, input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::batch::LayoutOutputEntryOf<u32, S>> {
        l::tree::LayoutTree::unrounded_layout(&self.inner, node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: l::box_model::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<l::box_model::FloatExclusionIntervalOf<S>>, Refusal>> {
        if node != self.float {
            return None;
        }
        assert_eq!(
            query.flow_axes(),
            l::geometry::FlowAxes::new(
                l::geometry::WritingMode::SidewaysRl,
                l::geometry::Direction::Ltr
            )
        );
        assert_eq!(query.margin_box().origin(), l::geometry::Point::ZERO);
        assert_eq!(
            query.margin_box().size(),
            l::geometry::Size::new(n(40.0), n(self.capacity))
        );
        let nonempty = query.band_minimum() < query.band_maximum();
        let later = nonempty && query.band_maximum() <= n(30.0);
        let refused = self.refuse.get() && later;
        self.queries.borrow_mut().push((query, refused));
        // Empty shape exclusion is a real successful provider result. Refusal
        // depends on the actual physical band, never on source or call count.
        Some(if refused {
            Err(Refusal::LaterBand)
        } else {
            Ok(None)
        })
    }
}

impl<S: l::scalar::LayoutScalar> ShapeHost<S> {
    fn compute(
        &self,
        bindings: &[l::text_layout::TextParagraphBinding<u32>],
        changed: &[u32],
    ) -> Result<l::batch::CompletedLayoutBatchOf<u32, S>, l::error::LayoutErrorOf<u32, S, Refusal>>
    {
        let flow = l::text_layout::LayoutFlowInput::try_new(
            self,
            0,
            bindings.to_vec(),
            l::inline_wrapping::WrappingInput::default(),
            Vec::new(),
        )
        .expect("the incoming float is outside the actual captured source window");
        l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            {
                let sizing_request = available(40.0, self.capacity);
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
            },
            changed,
        )
    }
}

fn late_scene<S: l::scalar::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::source::InlineBoxKind,
) -> (ShapeHost<S>, Vec<l::text_layout::TextParagraphBinding<u32>>) {
    let capacity = if kind == t::source::InlineBoxKind::InFlow {
        24.0
    } else {
        10.0
    };
    let mut input = root(
        l::geometry::WritingMode::SidewaysRl,
        l::geometry::Direction::Ltr,
        l::baseline::InlineBaselineContext::Physical,
    );
    input.size = l::geometry::Size::new(px(40.0), px(capacity));
    let mut inner = Host::new(input);
    let float = inner.push(
        Some(0),
        l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
            display: l::box_model::Display::Block,
            writing_mode: l::geometry::WritingMode::SidewaysRl,
            float: l::box_model::Float::Left,
            float_exclusion: l::box_model::FloatExclusion::Shape,
            size: l::geometry::Size::new(px(40.0), px(capacity)),
            ..l::node::NodeInputOf::default()
        }),
    );
    assert_eq!(float, 1);
    let bindings = attach(&mut inner, 0, parent, child, kind);
    assert_eq!(bindings[0].nodes, [2, 3, 4]);
    (
        ShapeHost {
            inner,
            float,
            capacity,
            refuse: Cell::new(false),
            queries: RefCell::new(Vec::new()),
        },
        bindings,
    )
}

fn same_text<S: l::scalar::LayoutScalar>(
    a: &l::batch::CompletedLayoutBatchOf<u32, S>,
    b: &l::batch::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(a.placed_text().len(), b.placed_text().len());
    for row in a.placed_text() {
        let other = b
            .placed_text()
            .iter()
            .find(|other| row.container() == other.container())
            .unwrap();
        assert_eq!(row.nodes(), other.nodes());
        assert_eq!(row.layout().source(), other.layout().source());
        assert_eq!(row.layout().key(), other.layout().key());
        let positions = |text: &t::layout::Layout| {
            text.positions()
                .map(|position| {
                    (
                        position.source_cursor(),
                        text.position_geometry(&position).unwrap().rect(),
                    )
                })
                .collect::<Vec<_>>()
        };
        // Independently placed results issue foreign handles. Compare source
        // observations and physical geometry; retained snapshots below keep
        // their exact original handles as well.
        assert_eq!(positions(row.layout()), positions(other.layout()));
    }
}

fn refusal<S: l::scalar::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::paragraph::ParagraphBaseDirection::LeftToRight,
        t::selection::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [
        t::source::InlineBoxKind::InFlow,
        t::source::InlineBoxKind::OutOfFlow,
    ] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::source::Id::from_u64(3))),
            t::paragraph::ParagraphBaseDirection::LeftToRight,
            t::selection::LineOrientation::SidewaysClockwise,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (mut host, bindings) = late_scene::<S>(&parent, &child, kind);
        let old = host.compute(&bindings, &[]).unwrap();
        let text = placed(&old, 0);
        assert_eq!(text.lines().len(), 2);
        assert_eq!(text.inline_boxes().len(), 1);
        assert_eq!(text.inline_boxes()[0].kind(), kind);
        assert_eq!(text.inline_boxes()[0].id(), t::source::Id::from_u64(3));
        assert_eq!(
            text.inline_boxes()[0].rect().size,
            t::geometry::Size::new(6.0, 14.0)
        );
        assert_eq!(text.source(), &parent.source);
        assert!(
            text.run_views()
                .all(|run| run.glyph_basis() == t::geometry::GlyphBasis::Clockwise)
        );
        if kind == t::source::InlineBoxKind::InFlow {
            assert_eq!(text.inline_boxes()[0].line(), 0);
            assert_eq!(receipt(&old, 2).feedback().advance, 24.0);
        }
        old.apply_to(&mut host.inner).unwrap();
        let caches = host.inner.published.caches.clone();
        let nodes = host.inner.published.unrounded.clone();
        let published = host.inner.published.batch.as_ref().unwrap().clone();
        host.queries.borrow_mut().clear();
        host.refuse.set(true);
        let error = host.compute(&bindings, &[0, host.float]).unwrap_err();
        assert_eq!(
            error.operation(),
            l::error::LayoutOperation::FloatExclusionQuery
        );
        assert_eq!(
            error.site(),
            l::error::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: host.float
            }
        );
        assert!(matches!(
            error.kind(),
            l::error::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
        ));
        let queries = host.queries.borrow();
        let failed = queries.iter().position(|(_, refused)| *refused).unwrap();
        let q = &queries[failed].0;
        assert!(q.band_minimum() < q.band_maximum() && q.band_maximum() <= n(30.0));
        assert!(queries[..failed].iter().any(|(q, refused)| !*refused
            && q.band_minimum() < q.band_maximum()
            && q.band_maximum() > n(30.0)));
        drop(queries);
        assert_eq!(host.inner.published.caches, caches);
        assert_eq!(host.inner.published.unrounded, nodes);
        let retained = host.inner.published.batch.as_ref().unwrap();
        assert_eq!(
            retained
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.unrounded_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.unrounded_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>(),
            published
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.unrounded_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.unrounded_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            retained
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>(),
            published
                .node_layouts()
                .map(|(node, view)| (
                    node,
                    match view {
                        surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                            view.final_entry(),
                        surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                            panic!("continuous fixture unexpectedly fragmented"),
                    }
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            retained.unrounded_inline_fragments(),
            published.unrounded_inline_fragments()
        );
        assert_eq!(
            retained.final_inline_fragments(),
            published.final_inline_fragments()
        );
        for row in published.placed_text() {
            assert_eq!(
                placed(retained, row.container())
                    .positions()
                    .collect::<Vec<_>>(),
                row.layout().positions().collect::<Vec<_>>()
            );
        }
        same_text(retained, &published);
        host.refuse.set(false);
        let restored = host.compute(&bindings, &[0, host.float]).unwrap();
        let (fresh, bindings) = late_scene::<S>(&parent, &child, kind);
        let fresh = fresh.compute(&bindings, &[]).unwrap();
        for other in [&old, &fresh] {
            assert_eq!(semantic(&restored, &host), semantic(other, &host));
            assert_eq!(
                restored
                    .node_layouts()
                    .map(|(node, view)| (
                        node,
                        match view {
                            surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                                view.final_entry(),
                            surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                view.final_entry(),
                            surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                panic!("continuous fixture unexpectedly fragmented"),
                        }
                    ))
                    .collect::<Vec<_>>(),
                other
                    .node_layouts()
                    .map(|(node, view)| (
                        node,
                        match view {
                            surgeist_layout::fragmentation::NodeLayoutRefOf::SingleBox(view) =>
                                view.final_entry(),
                            surgeist_layout::fragmentation::NodeLayoutRefOf::NonBox(view) =>
                                view.final_entry(),
                            surgeist_layout::fragmentation::NodeLayoutRefOf::Fragments(_) =>
                                panic!("continuous fixture unexpectedly fragmented"),
                        }
                    ))
                    .collect::<Vec<_>>()
            );
            same_text(&restored, other);
        }
    }
}

#[test]
fn physical_clockwise_atoms_later_provider_refusal_preserves_all_applied_channels() {
    refusal::<f32>();
    refusal::<f64>();
}
