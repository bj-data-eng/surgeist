//! A real native atomic line precedes a physical later-band provider refusal.
use super::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}

struct ShapeHost<S: l::scalar::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    mode: l::geometry::WritingMode,
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
            l::geometry::FlowAxes::new(self.mode, l::geometry::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::geometry::Point::ZERO);
        assert_eq!(
            query.margin_box().size(),
            l::geometry::Size::new(n(40.0), n(18.0))
        );
        let nonempty = query.band_minimum() < query.band_maximum();
        let later = nonempty
            && match self.mode {
                l::geometry::WritingMode::VerticalRl => query.band_maximum() <= n(30.0),
                l::geometry::WritingMode::VerticalLr => query.band_minimum() >= n(10.0),
                _ => panic!("this provider owns native vertical fixtures"),
            };
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
                let sizing_request = available(40.0, 18.0);
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

fn scene<S: l::scalar::LayoutScalar>(
    parent: &Parent,
    child: &Fixture,
    mode: l::geometry::WritingMode,
) -> (ShapeHost<S>, Vec<l::text_layout::TextParagraphBinding<u32>>) {
    let mut input = root(mode, l::geometry::Direction::Ltr, central());
    input.size = l::geometry::Size::new(px(40.0), px(18.0));
    let mut inner = Host::new(input);
    let float = inner.push(
        Some(0),
        l::node::LayoutInputOf::box_input(l::node::NodeInputOf {
            display: l::box_model::Display::Block,
            writing_mode: mode,
            float: l::box_model::Float::Left,
            float_exclusion: l::box_model::FloatExclusion::Shape,
            size: l::geometry::Size::new(px(40.0), px(18.0)),
            ..l::node::NodeInputOf::default()
        }),
    );
    assert_eq!(float, 1);
    let bindings = attach(&mut inner, 0, parent, child_input(), Some(child));
    assert_eq!(bindings[0].nodes, [2, 3, 4]);
    assert_eq!(bindings[0].atomics[0].node, 3);
    assert_eq!(bindings[1].nodes, [5]);
    (
        ShapeHost {
            inner,
            float,
            mode,
            refuse: Cell::new(false),
            queries: RefCell::new(Vec::new()),
        },
        bindings,
    )
}

fn same_text_geometry<S: l::scalar::LayoutScalar>(
    left: &l::batch::CompletedLayoutBatchOf<u32, S>,
    right: &l::batch::CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(left.placed_text().len(), right.placed_text().len());
    for row in left.placed_text() {
        let other = right
            .placed_text()
            .iter()
            .find(|other| other.container() == row.container())
            .unwrap();
        assert_eq!(row.nodes(), other.nodes());
        let a = row.layout();
        let b = other.layout();
        assert_eq!(a.source(), b.source());
        assert_eq!(
            a.analyzed_source().identity(),
            b.analyzed_source().identity()
        );
        assert_eq!(a.lines(), b.lines());
        assert_eq!(a.inline_boxes(), b.inline_boxes());
        // Fresh placements issue distinct handles. Compare the public source
        // coordinates and physical geometry rather than foreign identities.
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
        assert_eq!(positions(a), positions(b));
    }
}

fn late_refusal<S: l::scalar::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::geometry::Size::new(6.0, 14.0),
        false,
        t::source::Id::from_u64(3),
    );
    for mode in [
        l::geometry::WritingMode::VerticalRl,
        l::geometry::WritingMode::VerticalLr,
    ] {
        let (mut host, bindings) = scene::<S>(&parent, &child, mode);
        let old = host.compute(&bindings, &[]).unwrap();
        let text = placed(&old, 0);
        assert_eq!(text.source(), &parent.source);
        assert_eq!(text.orientation(), t::selection::LineOrientation::Vertical);
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::source::Range::new(0, 1), t::source::Range::new(1, 2)]
        );
        let lines = receipts(&old, &bindings[0]);
        assert_eq!(lines.len(), 2);
        let slots = lines[&0].atomic_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].occurrence, bindings[0].atomics[0].occurrence);
        assert_eq!(lines[&1].atomic_slots().count(), 0);
        assert_eq!(
            (
                lines[&0].feedback().advance,
                lines[&0].feedback().baseline,
                lines[&0].feedback().line_height
            ),
            (18.0, 8.0, 10.0)
        );
        assert_eq!(lines[&1].feedback().advance, 4.0);
        let c = if mode == l::geometry::WritingMode::VerticalRl {
            32.0
        } else {
            2.0
        };
        atomic_rect(
            &old,
            &bindings[0],
            t::geometry::Rect::new(c - 2.0, 4.0, 6.0, 14.0),
        );
        assert_eq!(placed(&old, 3).lines()[0].baseline(), c);
        old.apply_to(&mut host.inner).unwrap();
        let caches = host.inner.published.caches.clone();
        let nodes = host.inner.published.unrounded.clone();
        let published = host.inner.published.batch.as_ref().unwrap().clone();

        host.queries.borrow_mut().clear();
        host.refuse.set(true);
        // Err proves that no completed batch is returned. Immutable compute
        // has no sink/application step whose chronology could be observed.
        let error = host.compute(&bindings, &[0, host.float]).unwrap_err();
        assert_eq!(
            error.operation(),
            l::error::LayoutOperation::FloatExclusionQuery
        );
        assert_eq!(
            error.site(),
            l::error::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: host.float,
            }
        );
        assert!(matches!(
            error.kind(),
            l::error::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
        ));
        let queries = host.queries.borrow();
        let failed = queries.iter().position(|(_, refused)| *refused).unwrap();
        let later = &queries[failed].0;
        assert!(later.band_minimum() < later.band_maximum());
        assert!(if mode == l::geometry::WritingMode::VerticalRl {
            later.band_maximum() <= n(30.0)
        } else {
            later.band_minimum() >= n(10.0)
        });
        assert!(
            queries[..failed].iter().any(|(query, refused)| !*refused
                && query.band_minimum() < query.band_maximum()
                && if mode == l::geometry::WritingMode::VerticalRl {
                    query.band_maximum() > n(30.0)
                } else {
                    query.band_minimum() < n(10.0)
                }),
            "an earlier nonempty physical band must succeed before the later refusal"
        );
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
            let actual = placed(retained, row.container());
            assert_eq!(actual.source(), row.layout().source());
            assert_eq!(
                actual.analyzed_source().identity(),
                row.layout().analyzed_source().identity()
            );
            assert_eq!(
                actual.positions().collect::<Vec<_>>(),
                row.layout().positions().collect::<Vec<_>>()
            );
        }
        same_text_geometry(retained, &published);

        host.refuse.set(false);
        let restored = host.compute(&bindings, &[0, host.float]).unwrap();
        let (fresh, fresh_bindings) = scene::<S>(&parent, &child, mode);
        let fresh = fresh.compute(&fresh_bindings, &[]).unwrap();
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
            same_text_geometry(&restored, other);
        }
    }
}

#[test]
fn native_atomic_later_float_provider_refusal_preserves_applied_packet_f32() {
    late_refusal::<f32>();
}
#[test]
fn native_atomic_later_float_provider_refusal_preserves_applied_packet_f64() {
    late_refusal::<f64>();
}
