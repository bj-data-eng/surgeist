//! Intrinsic width tags remain distinct when an orthogonal host supplies height.
use super::*;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

struct NativeCache<S: LayoutScalar> {
    native: PublicLayoutTreeOf<S>,
    caches: RefCell<HashMap<u32, CacheOf<S>>>,
    unrounded: RefCell<HashMap<u32, LayoutOutputEntryOf<u32, S>>>,
    hits: Cell<usize>,
}

impl<S: LayoutScalar> NativeCache<S> {
    fn accept(&self, batch: &CompletedLayoutBatchOf<u32, S>) {
        let mut caches = self.caches.borrow_mut();
        for entry in batch.cache_clear_entries() {
            caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            caches.entry(entry.node()).or_default().store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        self.unrounded.borrow_mut().extend(
            batch
                .unrounded_entries()
                .iter()
                .map(|entry| (entry.node(), *entry)),
        );
    }
}

impl<S: LayoutScalar> Traverse for NativeCache<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.native, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.native.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.native.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for NativeCache<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.native.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.native.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        self.native.has_leaf_measurement(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let output = self
            .caches
            .borrow()
            .get(&node)
            .and_then(|cache| cache.get_with_context(input, context));
        if output.is_some() {
            self.hits.set(self.hits.get() + 1);
        }
        output
    }
    fn unrounded_layout(&self, node: u32) -> Option<LayoutOutputEntryOf<u32, S>> {
        self.unrounded.borrow().get(&node).copied()
    }
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut entries = entries.iter().filter(|entry| entry.node() == node);
    let value = entries
        .next()
        .expect("the native box is published")
        .output();
    assert!(entries.next().is_none(), "one published box per node");
    value
}

fn assert_geometry<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, minimum: bool) {
    let root_width = if minimum { 20.0 } else { 40.0 };
    let second = if minimum {
        Point::new(n(0.0), n(10.0))
    } else {
        Point::new(n(20.0), n(0.0))
    };
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        for (node, location, size, source) in [
            (0, Point::ZERO, Size::new(n(root_width), n(20.0)), 0),
            (1, Point::ZERO, Size::new(n(20.0), n(10.0)), 0),
            (2, second, Size::new(n(20.0), n(10.0)), 1),
        ] {
            let actual = output(entries, node);
            assert_eq!(actual.location, location, "node{node} position");
            assert_eq!(actual.size, size, "node{node} size");
            assert_eq!(actual.source_index, SourceIndex::new(source));
            assert_eq!(
                (actual.margin, actual.border, actual.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO)
            );
            let scroll = actual
                .scroll_geometry
                .expect("native performed box retains scroll geometry");
            assert_eq!(
                scroll.flow_axes(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
            );
            for rect in [
                scroll.border_box(),
                scroll.padding_box(),
                scroll.content_box(),
                scroll.scrollport(),
                scroll.scrollable_overflow(),
            ] {
                assert_eq!(rect.origin(), Point::ZERO);
                assert_eq!(rect.size(), size);
            }
            assert_eq!(scroll.scrollbar_size(), Size::ZERO);
            let range = scroll.physical_range();
            assert_eq!(
                (
                    range.x().minimum(),
                    range.x().maximum(),
                    range.y().minimum(),
                    range.y().maximum()
                ),
                (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
            );
        }
    }
}

fn characterize<S: LayoutScalar>() {
    let native = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::HorizontalTb,
                direction: Direction::Ltr,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(n(20.0)), PreferredSizeOf::px(n(10.0))),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(n(20.0)), PreferredSizeOf::px(n(10.0))),
                ..NodeInputOf::default()
            },
        );
    assert_eq!(
        native.node_input(0).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    assert_eq!(Traverse::children(&native, 0).collect::<Vec<_>>(), [1, 2]);
    for node in [0, 1, 2] {
        assert!(!native.has_leaf_measurement(node));
    }
    let parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let viewport = Size::splat(AvailableOf::Definite(n(400.0)));
    let context = FlexItemRootContextOf::under_viewport(viewport, parent).unwrap();
    assert_eq!(context.viewport_available(), viewport);
    assert_eq!(context.parent_flow_axes(), parent);
    let host = NativeCache {
        native,
        caches: RefCell::new(HashMap::new()),
        unrounded: RefCell::new(HashMap::new()),
        hits: Cell::new(0),
    };
    for minimum in [true, false, true] {
        let width = if minimum {
            AvailableOf::MIN_CONTENT
        } else {
            AvailableOf::MAX_CONTENT
        };
        let available = Size::new(width, AvailableOf::Definite(n(20.0)));
        let request = LayoutRootRequestOf::flex_item_under_viewport(available, context).unwrap();
        assert_eq!(request.available(), available);
        assert_eq!(
            request.context(),
            LayoutRootContextOf::FlexItemUnderViewport(context)
        );
        // Parent-inline is physical height. The still-unknown physical width
        // retains its intrinsic tag: wrapping MIN uses20, MAX uses40.
        let cold = compute_layout(&host.native, 0, request)
            .expect("cold native contextual root completes");
        assert_geometry(&cold, minimum);
        // Ordinary nonleaf roots intentionally bypass the performed-layout
        // cache. Warm reuse here exercises actual cache-eligible descendants;
        // it does not establish a root-operation cache-key regression.
        let cached = compute_layout(&host, 0, request)
            .expect("cache-enabled native contextual root completes");
        assert_geometry(&cached, minimum);
        assert_eq!(cached.unrounded_entries(), cold.unrounded_entries());
        assert_eq!(cached.final_entries(), cold.final_entries());
        host.accept(&cached);
        let hits = host.hits.get();
        let warm =
            compute_layout(&host, 0, request).expect("warm native contextual root completes");
        assert_geometry(&warm, minimum);
        assert_eq!(warm.unrounded_entries(), cold.unrounded_entries());
        assert_eq!(warm.final_entries(), cold.final_entries());
        assert!(
            host.hits.get() > hits,
            "the warm request reaches the real populated cache"
        );
        host.accept(&warm);
    }
}

#[test]
fn orthogonal_host_inline_size_preserves_intrinsic_width_wrap_f32() {
    characterize::<f32>();
}
#[test]
fn orthogonal_host_inline_size_preserves_intrinsic_width_wrap_f64() {
    characterize::<f64>();
}
