//! Real host cache seeded by an ordinary completed layout before Text attachment.
use super::*;

struct WarmRoot<'a, S: l::scalar::LayoutScalar> {
    actual: &'a Root<S>,
    cache: l::cache::CacheOf<S>,
    unrounded: Option<l::batch::LayoutOutputEntryOf<u32, S>>,
}
impl<S: l::scalar::LayoutScalar> l::tree::Traverse for WarmRoot<'_, S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::tree::Traverse::children(self.actual, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::tree::Traverse::child_count(self.actual, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::tree::Traverse::child(self.actual, node, index)
    }
}
impl<S: l::scalar::LayoutScalar> l::tree::LayoutTree for WarmRoot<'_, S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::node::NodeInputOf<S> {
        l::tree::LayoutTree::node_input(self.actual, node)
    }
    fn layout_input(&self, node: u32) -> l::node::LayoutInputViewOf<'_, S> {
        l::tree::LayoutTree::layout_input(self.actual, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::cache::ComputeInputOf<S>,
        context: l::cache::CacheKeyContext,
    ) -> Option<l::cache::ComputeOutputOf<S>> {
        assert_eq!(node, 0);
        self.cache.get_with_context(input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::batch::LayoutOutputEntryOf<u32, S>> {
        assert_eq!(node, 0);
        self.unrounded
    }
}
impl<S: l::scalar::LayoutScalar> WarmRoot<'_, S> {
    fn request(&self) -> l::request::LayoutRootRequestOf<S> {
        {
            let sizing_request = crate::layout::native_request::sizing(
                surgeist_layout::geometry::Size::new(Some(S::from_f64(30.0)), None),
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
        }
    }
    fn attach(
        &self,
        f: &Fixture<S>,
        inventory: l::inline_wrapping::WrappingInput<u32>,
    ) -> Result<l::batch::CompletedLayoutBatchOf<u32, S>, l::error::LayoutErrorOf<u32, S>> {
        let capture = l::text_layout::LayoutFlowInput::try_new(
            self,
            0,
            vec![f.binding()],
            inventory,
            Vec::new(),
        )?;
        l::compute_layout_with_flow(&capture, 0, self.request())
    }
}

fn root_output<S: l::scalar::LayoutScalar>(
    batch: &l::batch::CompletedLayoutBatchOf<u32, S>,
) -> l::node::NodeOutputOf<S> {
    batch
        .node_layout(0)
        .unwrap()
        .single_box()
        .expect("fixture node produces one box")
        .final_output()
}
fn warm_semantic_custody<S: l::scalar::LayoutScalar>(direction: l::geometry::Direction) {
    let f = Fixture::<S>::new(direction, true);
    let pin = f.inventory_facts(true);
    let mut host = WarmRoot {
        actual: &f.tree,
        cache: l::cache::CacheOf::new(),
        unrounded: None,
    };
    // This is an actual previous publication using the same unchanged native
    // childless tree before its captured Text paragraph is attached.
    let native = l::compute_layout(&host, 0, host.request()).unwrap();
    assert_eq!(
        root_output(&native).size,
        l::geometry::Size::new(S::from_f64(30.0), S::ZERO)
    );
    assert!(native.placed_text().is_empty());
    assert!(!native.cache_store_entries().is_empty());
    for row in native.cache_store_entries() {
        assert_eq!(row.node(), 0);
        host.cache
            .store_with_context(row.input(), row.context(), row.output());
    }
    host.unrounded = Some(
        native
            .node_layout(0)
            .unwrap()
            .single_box()
            .expect("root produces one box")
            .unrounded_entry(),
    );
    assert!(!host.cache.is_empty());
    assert!(
        native
            .cache_store_entries()
            .iter()
            .any(
                |row| host.cache.get_with_context(row.input(), row.context()) == Some(row.output())
            ),
        "at least one retained host answer must remain queryable"
    );
    // The host's retained answers are producer output, not a fake
    // positive line or an invented zero-size semantic paragraph allocation.
    let cache_before = host.cache.clone();
    let unrounded_before = host.unrounded;
    let first = host.attach(&f, f.inventory()).unwrap();
    assert_semantic_output(&f, &first);
    let replay = host.attach(&f, f.inventory()).unwrap();
    assert_semantic_output(&f, &replay);
    assert_eq!(
        first.placed_text()[0].layout().key(),
        replay.placed_text()[0].layout().key()
    );

    // Actual S changes from baseline16/extent20 to24/40. Force is now40 high;
    // the nominal receiver is max(24,12)+max(40-24,30-12)=42, still phantom.
    let mut changed_input = f.inventory();
    match &mut changed_input.scopes[0] {
        l::inline_wrapping::WrappingScopeInput::Captured {
            scope,
            metric_contribution,
            ..
        } => {
            assert_eq!(&*scope, &f.scopes[0]);
            *metric_contribution = Some(l::inline_wrapping::InlineBoxMetricContribution::Strut(
                metric(24.0, 40.0),
            ));
        }
        _ => panic!("the actual S metric owner is required"),
    }
    let changed = host.attach(&f, changed_input).unwrap();
    assert_eq!(
        root_output(&changed).size,
        l::geometry::Size::new(S::from_f64(30.0), S::from_f64(40.0))
    );
    let layout = changed.placed_text()[0].layout();
    assert_eq!(layout.source(), f.prepared.analyzed_source().source());
    assert_eq!(
        layout.analyzed_source().identity(),
        f.prepared.analyzed_source().identity()
    );
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].baseline(), 24.0);
    assert_eq!(layout.lines()[0].bounds().size.height, 40.0);
    assert!(changed.final_inline_fragments().is_empty());
    let retained = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].origin.y, 40.0);
    let receipt = retained[0].line.finalized_line();
    assert_eq!(receipt.prepared_text(), f.prepared);
    assert_eq!(receipt.feedback().baseline, 24.0);
    assert_eq!(receipt.feedback().line_height, 42.0);
    assert!(!receipt.has_in_flow_content());
    assert_eq!(
        receipt.selected_line().request().context.end,
        t::selection::LineEnd::Paragraph
    );
    assert_struts(
        &receipt.selected_line().request().line_box_struts,
        &[
            expected_metric(&f.scopes[0], 24.0, 40.0),
            expected_metric(&f.scopes[1], 12.0, 30.0),
        ],
    );
    assert_ne!(first.placed_text()[0].layout().key(), layout.key());
    assert_semantic_output(&f, &first);
    assert_eq!(f.inventory_facts(true), pin);

    let mut missing = f.inventory();
    match &mut missing.scopes[1] {
        l::inline_wrapping::WrappingScopeInput::Captured {
            metric_contribution,
            ..
        } => *metric_contribution = None,
        _ => panic!("the actual T metric owner is required"),
    }
    let refused = host.attach(&f, missing).unwrap_err();
    assert!(matches!(
        refused.kind(),
        l::error::LayoutErrorKindOf::InvalidInput(l::error::LayoutInvalidInputOf::WrappingScope(
            l::inline_wrapping::WrappingScopeError::MissingBoxMetrics
        ))
    ));
    assert_eq!(host.cache, cache_before);
    assert_eq!(host.unrounded, unrounded_before);
    assert_semantic_output(&f, &first);
    assert_eq!(f.inventory_facts(true), pin);
}
#[test]
fn real_warm_native_cache_cannot_hide_node_free_text_or_metric_changes() {
    for direction in [l::geometry::Direction::Ltr, l::geometry::Direction::Rtl] {
        warm_semantic_custody::<f32>(direction);
        warm_semantic_custody::<f64>(direction);
    }
}
