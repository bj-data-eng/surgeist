//! Later physical float-band refusal retains an applied positioned packet.
use super::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}

struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    mode: l::WritingMode,
    refuse: Cell<bool>,
    queries: RefCell<Vec<(l::FloatExclusionQueryOf<S>, bool)>>,
}

impl<S: l::LayoutScalar> l::Traverse for ShapeHost<S> {
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

impl<S: l::LayoutScalar> l::LayoutTree for ShapeHost<S> {
    type MeasureError = Refusal;

    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(&self.inner, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(&self.inner, node)
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
    ) -> Option<Result<Option<l::FloatExclusionIntervalOf<S>>, Refusal>> {
        if node != self.float {
            return None;
        }
        assert_eq!(
            query.flow_axes(),
            l::FlowAxes::new(self.mode, l::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::Point::ZERO);
        assert_eq!(query.margin_box().size(), l::Size::new(n(40.0), n(4.0)));
        let nonempty = query.band_minimum() < query.band_maximum();
        let later = nonempty
            && match self.mode {
                l::WritingMode::VerticalRl => query.band_maximum() <= n(30.0),
                l::WritingMode::VerticalLr => query.band_minimum() >= n(10.0),
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

impl<S: l::LayoutScalar> ShapeHost<S> {
    fn compute(
        &self,
        bindings: &[l::TextParagraphBinding<u32>],
        changed: &[u32],
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S, Refusal>> {
        let flow =
            l::LayoutFlowInput::try_new(self, 0, bindings.to_vec(), l::WrappingInput::default())
                .expect("the incoming float is outside the actual captured source window");
        l::compute_layout_with_flow_invalidated(
            &flow,
            0,
            l::LayoutRootRequestOf::viewport(available(40.0, 4.0)).unwrap(),
            changed,
        )
    }
}

fn same_text_geometry<S: l::LayoutScalar>(
    left: &l::CompletedLayoutBatchOf<u32, S>,
    right: &l::CompletedLayoutBatchOf<u32, S>,
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
        let positions = |text: &t::Layout| {
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

fn late_scene<S: l::LayoutScalar>(
    parent: &Parent,
    child: &Fixture,
    mode: l::WritingMode,
) -> (ShapeHost<S>, Vec<l::TextParagraphBinding<u32>>) {
    let mut input = root(mode, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(4.0));
    let mut inner = Host::new(input);
    let float = inner.push(
        Some(0),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: mode,
            float: l::Float::Left,
            float_exclusion: l::FloatExclusion::Shape,
            size: l::Size::new(px(40.0), px(4.0)),
            ..l::NodeInputOf::default()
        }),
    );
    assert_eq!(float, 1);
    let bindings = attach(&mut inner, 0, parent, child);
    assert_eq!(bindings[0].nodes, [2, 3, 4]);
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

fn late_refusal<S: l::LayoutScalar>() {
    let child = fixture();
    qualify_child::<S>(&child);
    let parent = captured_parent(
        &child.bytes,
        t::Size::new(6.0, 14.0),
        false,
        t::Id::from_u64(3),
    );
    qualify_parent(&parent);
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let (mut host, bindings) = late_scene::<S>(&parent, &child, mode);
        let old = host.compute(&bindings, &[]).unwrap();
        let text = placed(&old, 0);
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [t::Range::new(0, 1), t::Range::new(1, 2)]
        );
        assert_eq!(text.source(), &parent.source);
        assert_eq!(text.inline_boxes().len(), 1);
        assert_eq!(text.inline_boxes()[0].kind(), t::InlineBoxKind::OutOfFlow);
        assert_eq!(text.inline_boxes()[0].id(), t::Id::from_u64(3));
        assert_eq!(text.inline_boxes()[0].rect().size, t::Size::new(6.0, 14.0));
        let mut lines = std::collections::BTreeMap::new();
        for row in old.unrounded_inline_fragments() {
            if matches!(row.owner(), l::InlineFragmentOwner::Node(2 | 4)) {
                let fragment = row.fragment();
                lines
                    .entry(fragment.line_index())
                    .or_insert_with(|| fragment.finalized_fragment().finalized_line().clone());
            }
        }
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines
                .values()
                .map(|line| line.atomic_slots().count())
                .sum::<usize>(),
            1
        );
        assert!(lines.values().all(|line| line.feedback().advance == 4.0));
        old.apply_to(&mut host.inner).unwrap();
        let caches = host.inner.published.caches.clone();
        let nodes = host.inner.published.unrounded.clone();
        let published = host.inner.published.batch.as_ref().unwrap().clone();
        host.queries.borrow_mut().clear();
        host.refuse.set(true);
        let error = host.compute(&bindings, &[0, host.float]).unwrap_err();
        assert_eq!(error.operation(), l::LayoutOperation::FloatExclusionQuery);
        assert_eq!(
            error.site(),
            l::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: host.float
            }
        );
        assert!(matches!(
            error.kind(),
            l::LayoutErrorKindOf::Measurement(Refusal::LaterBand)
        ));
        let queries = host.queries.borrow();
        let failed = queries.iter().position(|(_, refused)| *refused).unwrap();
        let later = &queries[failed].0;
        assert!(later.band_minimum() < later.band_maximum());
        assert!(if mode == l::WritingMode::VerticalRl {
            later.band_maximum() <= n(30.0)
        } else {
            later.band_minimum() >= n(10.0)
        });
        assert!(
            queries[..failed].iter().any(|(q, refused)| !*refused
                && q.band_minimum() < q.band_maximum()
                && if mode == l::WritingMode::VerticalRl {
                    q.band_maximum() > n(30.0)
                } else {
                    q.band_minimum() < n(10.0)
                }),
            "an earlier actual nonempty band succeeds before the later provider refusal"
        );
        drop(queries);
        assert_eq!(host.inner.published.caches, caches);
        assert_eq!(host.inner.published.unrounded, nodes);
        let retained = host.inner.published.batch.as_ref().unwrap();
        assert_eq!(retained.unrounded_entries(), published.unrounded_entries());
        assert_eq!(retained.final_entries(), published.final_entries());
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
                actual.positions().collect::<Vec<_>>(),
                row.layout().positions().collect::<Vec<_>>()
            );
        }
        same_text_geometry(retained, &published);
        host.refuse.set(false);
        let restored = host.compute(&bindings, &[0, host.float]).unwrap();
        let (fresh, bindings) = late_scene::<S>(&parent, &child, mode);
        let fresh = fresh.compute(&bindings, &[]).unwrap();
        for other in [&old, &fresh] {
            assert_eq!(semantic(&restored), semantic(other));
            assert_eq!(restored.final_entries(), other.final_entries());
            same_text_geometry(&restored, other);
        }
    }
}

#[test]
fn native_upright_positioned_later_provider_refusal_preserves_applied_packet() {
    late_refusal::<f32>();
    late_refusal::<f64>();
}
