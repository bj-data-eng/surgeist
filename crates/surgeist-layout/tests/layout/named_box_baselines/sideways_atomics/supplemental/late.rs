//! Real later float-band refusal retains every applied atomic channel.
use super::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    LaterBand,
}

struct ShapeHost<S: l::LayoutScalar> {
    inner: Host<S>,
    float: u32,
    capacity: f64,
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
            l::FlowAxes::new(l::WritingMode::SidewaysRl, l::Direction::Ltr)
        );
        assert_eq!(query.margin_box().origin(), l::Point::ZERO);
        assert_eq!(
            query.margin_box().size(),
            l::Size::new(n(40.0), n(self.capacity))
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
            l::LayoutRootRequestOf::viewport(available(40.0, self.capacity)).unwrap(),
            changed,
        )
    }
}

fn late_scene<S: l::LayoutScalar>(
    parent: &Captured,
    child: &Captured,
    kind: t::InlineBoxKind,
) -> (ShapeHost<S>, Vec<l::TextParagraphBinding<u32>>) {
    let capacity = if kind == t::InlineBoxKind::InFlow {
        24.0
    } else {
        10.0
    };
    let mut input = root(
        l::WritingMode::SidewaysRl,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.size = l::Size::new(px(40.0), px(capacity));
    let mut inner = Host::new(input);
    let float = inner.push(
        Some(0),
        l::LayoutInputOf::box_input(l::NodeInputOf {
            display: l::Display::Block,
            writing_mode: l::WritingMode::SidewaysRl,
            float: l::Float::Left,
            float_exclusion: l::FloatExclusion::Shape,
            size: l::Size::new(px(40.0), px(capacity)),
            ..l::NodeInputOf::default()
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

fn same_text<S: l::LayoutScalar>(
    a: &l::CompletedLayoutBatchOf<u32, S>,
    b: &l::CompletedLayoutBatchOf<u32, S>,
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
        // Independently placed results issue foreign handles. Compare source
        // observations and physical geometry; retained snapshots below keep
        // their exact original handles as well.
        assert_eq!(positions(row.layout()), positions(other.layout()));
    }
}

fn refusal<S: l::LayoutScalar>() {
    let child = capture(
        AHEM,
        "A",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Horizontal,
        5.0,
    );
    qualify_child::<S>(&child);
    for kind in [t::InlineBoxKind::InFlow, t::InlineBoxKind::OutOfFlow] {
        let parent = capture(
            AHEM,
            "AB",
            Some((kind, t::Id::from_u64(3))),
            t::ParagraphBaseDirection::LeftToRight,
            t::LineOrientation::Sideways,
            10.0,
        );
        qualify_parent(&parent, kind);
        let (mut host, bindings) = late_scene::<S>(&parent, &child, kind);
        let old = host.compute(&bindings, &[]).unwrap();
        let text = placed(&old, 0);
        assert_eq!(text.lines().len(), 2);
        assert_eq!(text.inline_boxes().len(), 1);
        assert_eq!(text.inline_boxes()[0].kind(), kind);
        assert_eq!(text.inline_boxes()[0].id(), t::Id::from_u64(3));
        assert_eq!(text.inline_boxes()[0].rect().size, t::Size::new(6.0, 14.0));
        assert_eq!(text.source(), &parent.source);
        assert!(
            text.run_views()
                .all(|run| run.glyph_basis() == t::GlyphBasis::Clockwise)
        );
        if kind == t::InlineBoxKind::InFlow {
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
        let q = &queries[failed].0;
        assert!(q.band_minimum() < q.band_maximum() && q.band_maximum() <= n(30.0));
        assert!(queries[..failed].iter().any(|(q, refused)| !*refused
            && q.band_minimum() < q.band_maximum()
            && q.band_maximum() > n(30.0)));
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
            assert_eq!(semantic(&restored), semantic(other));
            assert_eq!(restored.final_entries(), other.final_entries());
            same_text(&restored, other);
        }
    }
}

#[test]
fn physical_clockwise_atoms_later_provider_refusal_preserves_all_applied_channels() {
    refusal::<f32>();
    refusal::<f64>();
}
