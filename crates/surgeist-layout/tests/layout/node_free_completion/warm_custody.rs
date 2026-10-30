//! Real host cache seeded by an ordinary completed layout before Text attachment.
use super::*;

struct WarmRoot<'a, S: l::LayoutScalar> {
    actual: &'a Root<S>,
    cache: l::CacheOf<S>,
    unrounded: Option<l::NodeOutputOf<S>>,
}
impl<S: l::LayoutScalar> l::Traverse for WarmRoot<'_, S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        l::Traverse::children(self.actual, node)
    }
    fn child_count(&self, node: u32) -> usize {
        l::Traverse::child_count(self.actual, node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        l::Traverse::child(self.actual, node, index)
    }
}
impl<S: l::LayoutScalar> l::LayoutTree for WarmRoot<'_, S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &l::NodeInputOf<S> {
        l::LayoutTree::node_input(self.actual, node)
    }
    fn layout_input(&self, node: u32) -> l::LayoutInputViewOf<'_, S> {
        l::LayoutTree::layout_input(self.actual, node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &l::ComputeInputOf<S>,
        context: l::CacheKeyContext,
    ) -> Option<l::ComputeOutputOf<S>> {
        assert_eq!(node, 0);
        self.cache.get_with_context(input, context)
    }
    fn unrounded_layout(&self, node: u32) -> Option<l::NodeOutputOf<S>> {
        assert_eq!(node, 0);
        self.unrounded
    }
}
impl<S: l::LayoutScalar> WarmRoot<'_, S> {
    fn request() -> l::LayoutRootRequestOf<S> {
        l::LayoutRootRequestOf::viewport(l::Size::new(
            l::AvailableOf::definite(S::from_f64(30.0)),
            l::AvailableOf::MAX_CONTENT,
        ))
        .unwrap()
    }
    fn attach(
        &self,
        f: &Fixture<S>,
        inventory: l::WrappingInput<u32>,
    ) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
        let capture =
            l::LayoutFlowInput::try_new(self, 0, vec![f.binding()], inventory, Vec::new())?;
        l::compute_layout_with_flow(&capture, 0, Self::request())
    }
}

fn root_output<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
) -> l::NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|row| row.node() == 0)
        .unwrap()
        .output()
}
fn warm_semantic_custody<S: l::LayoutScalar>(direction: l::Direction) {
    let f = Fixture::<S>::new(direction, true);
    let pin = f.inventory_facts(true);
    let mut host = WarmRoot {
        actual: &f.tree,
        cache: l::CacheOf::new(),
        unrounded: None,
    };
    // This is an actual previous publication using the same unchanged native
    // childless tree before its captured Text paragraph is attached.
    let native = l::compute_layout(&host, 0, WarmRoot::<S>::request()).unwrap();
    assert_eq!(
        root_output(&native).size,
        l::Size::new(S::from_f64(30.0), S::ZERO)
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
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap()
            .output(),
    );
    assert!(!host.cache.is_empty());
    assert!(
        native
            .cache_store_entries()
            .iter()
            .any(
                |row| host.cache.get_with_context(row.input(), row.context()) == Some(row.output())
            ),
        "at least one genuine retained host answer must remain queryable"
    );
    // The host's retained answers are genuine producer output, not a fake
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
        l::WrappingScopeInput::Captured {
            scope,
            metric_contribution,
            ..
        } => {
            assert_eq!(&*scope, &f.scopes[0]);
            *metric_contribution = Some(l::InlineBoxMetricContribution::Strut(metric(24.0, 40.0)));
        }
        _ => panic!("the actual S metric owner is required"),
    }
    let changed = host.attach(&f, changed_input).unwrap();
    assert_eq!(
        root_output(&changed).size,
        l::Size::new(S::from_f64(30.0), S::from_f64(40.0))
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
        t::LineEnd::Paragraph
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
        l::WrappingScopeInput::Captured {
            metric_contribution,
            ..
        } => *metric_contribution = None,
        _ => panic!("the actual T metric owner is required"),
    }
    let refused = host.attach(&f, missing).unwrap_err();
    assert!(matches!(
        refused.kind(),
        l::LayoutErrorKindOf::InvalidInput(l::LayoutInvalidInputOf::WrappingScope(
            l::WrappingScopeError::MissingBoxMetrics
        ))
    ));
    assert_eq!(host.cache, cache_before);
    assert_eq!(host.unrounded, unrounded_before);
    assert_semantic_output(&f, &first);
    assert_eq!(f.inventory_facts(true), pin);
}
#[test]
fn real_warm_native_cache_cannot_hide_node_free_text_or_metric_changes() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        warm_semantic_custody::<f32>(direction);
        warm_semantic_custody::<f64>(direction);
    }
}
