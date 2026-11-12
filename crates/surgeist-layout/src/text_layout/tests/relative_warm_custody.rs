use super::*;
use crate::test_support::readback::{self, Phase};

#[derive(Clone, Copy)]
enum ReturnedCustody {
    Unrounded,
    Final,
}

// The real CachedHost still owns cache admission and transactional publication.
// This adapter changes only which ACTUAL committed entry the warm getter returns.
struct CustodyHost<S: LayoutScalar> {
    inner: CachedHost<S>,
    returned: std::cell::RefCell<Vec<LayoutOutputEntryOf<u32, S>>>,
    phase: ReturnedCustody,
}
impl<S: LayoutScalar> CustodyHost<S> {
    fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            inner: CachedHost::new(tree),
            returned: Default::default(),
            phase: ReturnedCustody::Unrounded,
        }
    }
}
impl<S: LayoutScalar> Traverse for CustodyHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <CachedHost<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, n: u32) -> Self::Children<'_> {
        self.inner.children(n)
    }
    fn child_count(&self, n: u32) -> usize {
        self.inner.child_count(n)
    }
    fn child(&self, n: u32, i: usize) -> u32 {
        self.inner.child(n, i)
    }
}
impl<S: LayoutScalar> LayoutTree for CustodyHost<S> {
    type MeasureError = &'static str;
    fn node_input(&self, n: u32) -> &NodeInputOf<S> {
        self.inner.node_input(n)
    }
    fn layout_input(&self, n: u32) -> LayoutInputViewOf<'_, S> {
        self.inner.layout_input(n)
    }
    fn cache_get(
        &self,
        n: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.inner.cache_get(n, input, context)
    }
    fn unrounded_layout(&self, n: u32) -> Option<LayoutOutputEntryOf<u32, S>> {
        let published = self.inner.published.as_ref()?;
        let view = published.node_layout(n)?;
        let entry = match (self.phase, view) {
            (ReturnedCustody::Unrounded, crate::NodeLayoutRefOf::SingleBox(value)) => {
                value.unrounded_entry()
            }
            (ReturnedCustody::Final, crate::NodeLayoutRefOf::SingleBox(value)) => {
                value.final_entry()
            }
            (ReturnedCustody::Unrounded, crate::NodeLayoutRefOf::NonBox(value)) => {
                value.unrounded_entry()
            }
            (ReturnedCustody::Final, crate::NodeLayoutRefOf::NonBox(value)) => value.final_entry(),
            (_, crate::NodeLayoutRefOf::Fragments(_)) => {
                panic!("continuous custody fixture unexpectedly fragmented")
            }
        };
        self.returned.borrow_mut().push(entry);
        Some(entry)
    }
    fn float_exclusion_interval(
        &self,
        n: u32,
        q: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, &'static str>> {
        self.inner.float_exclusion_interval(n, q)
    }
}
impl<S: LayoutScalar> LayoutBatchSink<u32, S> for CustodyHost<S> {
    fn publication_targets(&self) -> &crate::LayoutPublicationTargets<u32> {
        self.inner.publication_targets()
    }
    fn publication_targets_mut(&mut self) -> &mut crate::LayoutPublicationTargets<u32> {
        self.inner.publication_targets_mut()
    }
    type Error = &'static str;
    type Prepared = <CachedHost<S> as LayoutBatchSink<u32, S>>::Prepared;
    fn prepare_layout_batch(
        &self,
        preparation: crate::LayoutBatchPreparation<'_, u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        self.inner.prepare_layout_batch(preparation)
    }
    fn commit_layout_batch(
        &mut self,
        commit: crate::LayoutBatchCommit<'_, u32, S, Self::Prepared>,
    ) {
        self.inner.commit_layout_batch(commit)
    }
}

fn custody_n<S: LayoutScalar>(x: f64) -> S {
    S::from_f64(x)
}
fn custody_compute<S: LayoutScalar>(
    host: &CustodyHost<S>,
    bindings: Vec<TextParagraphBinding<u32>>,
    dirty: &[u32],
) -> CompletedLayoutBatchOf<u32, S> {
    let flow = LayoutFlowInput::try_new(
        host,
        0,
        bindings,
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow_invalidated(
        &flow,
        0,
        {
            let containing_flow_axes = {
                let root_input = crate::LayoutTree::node_input((flow).tree(), 0);
                crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
            };
            let requested_axes = Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(custody_n::<S>(50.0)).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::MaxContent,
                ),
            );
            let mut logical_requests = containing_flow_axes.logical_size(requested_axes);
            // These fixtures perform ordinary block-size layout independently of
            // the explicitly requested inline contribution operation.
            logical_requests.block = logical_requests
                .block
                .with_operation(crate::SizingOperation::Normal);
            logical_requests.inline = logical_requests
                .inline
                .with_operation(crate::SizingOperation::Normal);
            let axes = containing_flow_axes.physical_size(logical_requests);
            // The old Native NoParent fixture explicitly supplied these percentage
            // bases. It supplied no independent ICB dimensions or ancestor scrollport.
            let percentage_parent = requested_axes.map(|axis| axis.space().definite_size());
            let fallback = crate::OrthogonalFallbackOf::new(
                crate::Size::NONE,
                crate::NearestScrollportOf::Absent,
                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            );
            crate::LayoutRootRequestOf::standalone(
                crate::SizingRequestOf::new(axes, fallback),
                containing_flow_axes,
                percentage_parent,
            )
        },
        dirty,
    )
    .unwrap()
}
fn custody_root<S: LayoutScalar>() -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(custody_n::<S>(50.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}
fn custody_atom<S: LayoutScalar>(left_axis: bool, offset: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        position: Position::Relative,
        vertical_align: VerticalAlignOf::Baseline,
        size: Size::new(
            PreferredSizeOf::px(custody_n::<S>(if left_axis { 10.4 } else { 10.0 })),
            PreferredSizeOf::px(custody_n::<S>(if left_axis { 10.0 } else { 10.4 })),
        ),
        padding: Edges::all(LengthOf::px(custody_n::<S>(0.25))),
        inset: if left_axis {
            Edges {
                left: LengthAutoOf::px(custody_n::<S>(offset)),
                ..Edges::all(LengthAutoOf::AUTO)
            }
        } else {
            Edges {
                top: LengthAutoOf::px(custody_n::<S>(offset)),
                ..Edges::all(LengthAutoOf::AUTO)
            }
        },
        ..NodeInputOf::default()
    }
}
fn custody_paragraph(left_axis: bool) -> PreparedParagraph {
    let style = ahem_style(10.0, 10.0);
    let mut parts = vec![
        ParagraphPart::text(1, Source::new("X"), style.clone()),
        ParagraphPart::Atomic {
            node: 2,
            size: text::Size::new(
                if left_axis { 10.4 } else { 10.0 },
                if left_axis { 10.0 } else { 10.4 },
            ),
        },
    ];
    if left_axis {
        parts.push(ParagraphPart::text(3, Source::new("X"), style.clone()));
    }
    PreparedParagraph::new(parts, style, text::ParagraphBaseDirection::LeftToRight)
}
fn custody_fixture<S: LayoutScalar>(
    p: &PreparedParagraph,
    left_axis: bool,
    offset: f64,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, if left_axis { vec![1, 2, 3] } else { vec![1, 2] })
        .children(1, [])
        .children(2, [])
        .style(0, custody_root())
        .input(1, p.text_input(1))
        .style(2, custody_atom(left_axis, offset));
    if left_axis {
        tree = tree.children(3, []).input(3, p.text_input(3));
    }
    tree
}
// Real nonempty shape-dependent subtree required by the existing performed
// restoration path. Its sole child is an out-of-flow float, so it introduces
// no in-flow line/baseline. The real1px border has an interior margin and stays within actual content.
fn custody_restorable_fixture<S: LayoutScalar>(
    p: &PreparedParagraph,
    left_axis: bool,
    offset: f64,
) -> PublicLayoutTreeOf<S> {
    custody_fixture(p, left_axis, offset)
        .children(2, [4])
        .children(4, [])
        .style(
            4,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                float_exclusion: FloatExclusion::Shape,
                // Keep the real restore-route stimulus well inside both rounded
                // content frames; padding-edge quantization is not this control.
                margin: Edges::all(LengthAutoOf::px(custody_n::<S>(2.0))),
                size: Size::new(
                    PreferredSizeOf::px(custody_n::<S>(1.0)),
                    PreferredSizeOf::px(custody_n::<S>(1.0)),
                ),
                ..NodeInputOf::default()
            },
        )
}

fn custody_contained_float<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    let parent = custody_entry(batch, 2, true).output();
    let float = custody_entry(batch, 4, true).output();
    assert_eq!(
        float.size,
        Size::new(custody_n::<S>(1.0), custody_n::<S>(1.0))
    );
    let content = parent.scroll_geometry.unwrap().content_box();
    assert!(float.location.x >= content.origin().x);
    assert!(float.location.y >= content.origin().y);
    assert!(float.location.x + float.size.width <= content.origin().x + content.size().width);
    assert!(float.location.y + float.size.height <= content.origin().y + content.size().height);
    assert!(batch.placed_text().iter().all(|row| row.container() != 2));
}

fn custody_entry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    finalized: bool,
) -> LayoutOutputEntryOf<u32, S> {
    let rows = if finalized {
        readback::continuous_custody(batch, Phase::Final)
    } else {
        readback::continuous_custody(batch, Phase::Unrounded)
    };
    *rows.iter().find(|row| row.node() == node).unwrap()
}
fn custody_visual_agreement<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    let node = custody_entry(batch, 2, true).output();
    let visual = batch.placed_text()[0].layout().inline_boxes()[0].rect();
    assert_eq!(visual.origin.x, node.location.x.to_f64());
    assert_eq!(visual.origin.y, node.location.y.to_f64());
    assert_eq!(visual.size.width, node.size.width.to_f64());
    assert_eq!(visual.size.height, node.size.height.to_f64());
}
fn custody_assert_simple<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    left_axis: bool,
    offset: f64,
) {
    for finalized in [false, true] {
        assert_eq!(
            custody_entry(batch, 0, finalized).output().size,
            Size::new(custody_n::<S>(50.0), custody_n::<S>(12.0))
        );
    }
    let placed = batch.placed_text();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].container(), 0);
    assert_eq!(
        placed[0].nodes(),
        if left_axis {
            &[1, 2, 3][..]
        } else {
            &[1, 2][..]
        }
    );
    let layout = placed[0].layout();
    assert_eq!(
        layout.source().text().code_units(),
        if left_axis {
            &[0x58, 0x58][..]
        } else {
            &[0x58][..]
        }
    );
    let lines = layout.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].baseline(), 10.0);
    for run in layout.glyph_runs() {
        assert_eq!(run.font().index(), 0);
        assert_eq!(
            run.font().bytes(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .as_slice()
        );
        assert_eq!(run.font_size(), 10.0);
    }
    let first = batch
        .unrounded_inline_fragments()
        .iter()
        .find(|row| *row.owner() == crate::InlineFragmentOwner::Node(1))
        .unwrap()
        .fragment();
    assert_eq!(first.baseline().y, custody_n::<S>(10.0));
    assert_eq!(first.rect().unwrap().origin().y, custody_n::<S>(2.0));
    if left_axis {
        let following = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|row| *row.owner() == crate::InlineFragmentOwner::Node(3))
            .unwrap()
            .fragment();
        assert_eq!(following.rect().unwrap().origin().x, custody_n::<S>(20.0));
    }
    custody_visual_agreement(batch);
    custody_contained_float(batch);
    let node = custody_entry(batch, 2, true).output();
    let geometry = node.scroll_geometry.unwrap();
    assert_eq!(geometry.border_box().origin(), Point::ZERO);
    assert_eq!(geometry.border_box().size(), node.size);
    assert_eq!(geometry.padding_box(), geometry.border_box());
    assert_eq!(geometry.scrollable_overflow(), geometry.padding_box());
    let content = geometry.content_box();
    // Independent canonical-corner arithmetic. Normal raw10.4 snaps to10,
    // while translated endpoints can span11. The .25 padding corners are
    // evaluated from measured geometry, not from either snapped width.
    if left_axis && offset == 0.32 {
        // Border10.32..20.72 ->10..21; content10.57..20.47 ->11..20.
        assert_eq!(node.location.x, custody_n::<S>(10.0));
        assert_eq!(node.size.width, custody_n::<S>(11.0));
        assert_eq!(content.origin().x, custody_n::<S>(1.0));
        assert_eq!(content.size().width, custody_n::<S>(9.0));
    } else if left_axis {
        // Border10.72..21.12 ->11..21; content10.97..20.87 ->11..21.
        assert_eq!(node.location.x, custody_n::<S>(11.0));
        assert_eq!(node.size.width, custody_n::<S>(10.0));
        assert_eq!(content.origin().x, S::ZERO);
        assert_eq!(content.size().width, custody_n::<S>(10.0));
    } else if offset == 0.72 {
        // Normal reference10 minus measured before10.4 gives rawtop−.4;
        // translation.72 gives.32..10.72 ->0..11, content.57..10.47 ->1..10.
        assert_eq!(node.location.y, S::ZERO);
        assert_eq!(node.size.height, custody_n::<S>(11.0));
        assert_eq!(content.origin().y, custody_n::<S>(1.0));
        assert_eq!(content.size().height, custody_n::<S>(9.0));
    } else {
        assert_eq!(node.location.y, S::ZERO);
        assert_eq!(node.size.height, custody_n::<S>(10.0));
        assert_eq!(content.origin().y, S::ZERO);
        assert_eq!(content.size().height, custody_n::<S>(10.0));
    }
}
fn custody_same_scene<S: LayoutScalar>(
    actual: &CompletedLayoutBatchOf<u32, S>,
    expected: &CompletedLayoutBatchOf<u32, S>,
) {
    assert_eq!(
        readback::continuous_custody(actual, Phase::Unrounded).as_slice(),
        readback::continuous_custody(expected, Phase::Unrounded).as_slice()
    );
    assert_eq!(
        readback::continuous_custody(actual, Phase::Final).as_slice(),
        readback::continuous_custody(expected, Phase::Final).as_slice()
    );
    // A new operation may issue a different finalized-line owner. Compare
    // its actual prepared occurrence and ordinal/source/geometry instead.
    let fragments = |rows: &[crate::InlineFragmentOutputEntryOf<u32, S>]| {
        rows.iter()
            .map(|row| {
                let fragment = row.fragment();
                let issued = fragment.finalized_fragment();
                let ordinal = issued
                    .finalized_line()
                    .fragments()
                    .position(|candidate| &candidate == issued)
                    .expect("issued fragment belongs to its actual finalized owner");
                let view = issued.view();
                (
                    row.owner().clone(),
                    ordinal,
                    view.owner(),
                    view.origin(),
                    view.local_range(),
                    view.aggregate_range(),
                    view.processed_range(),
                    view.bounds(),
                    fragment.rect(),
                    fragment.baseline(),
                    fragment.line_index(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        fragments(actual.unrounded_inline_fragments()),
        fragments(expected.unrounded_inline_fragments())
    );
    assert_eq!(
        fragments(actual.final_inline_fragments()),
        fragments(expected.final_inline_fragments())
    );
    assert_eq!(
        actual.placed_text()[0].layout().inline_metric_facts(),
        expected.placed_text()[0].layout().inline_metric_facts()
    );
}
fn custody_roundtrip<S: LayoutScalar>(left_axis: bool) {
    let initial = if left_axis { 0.32 } else { 0.72 };
    let changed_offset = if left_axis { 0.72 } else { 0.32 };
    let p = custody_paragraph(left_axis);
    let mut host = CustodyHost::new(custody_restorable_fixture(&p, left_axis, initial));
    assert_eq!(host.child_count(2), 1);
    assert_eq!(host.child(2, 0), 4);
    assert_eq!(host.child_count(4), 0);
    assert_eq!(host.node_input(4).float, Float::Left);
    assert_eq!(host.node_input(4).float_exclusion, FloatExclusion::Shape);
    assert_eq!(
        host.node_input(4).size,
        Size::new(
            PreferredSizeOf::px(custody_n::<S>(1.0)),
            PreferredSizeOf::px(custody_n::<S>(1.0))
        )
    );
    assert!(host.layout_input(4).as_inline_text().is_none());
    assert!(host.layout_input(4).as_text_control().is_none());
    // Exact source occurrence and nominal input remain through all runs.
    let issued = p.prepared.atomic_views().next().unwrap();
    assert_eq!(p.binding(0).atomics[0].occurrence, issued.handle());
    assert!(host.node_input(2).atomic_inline_participation.is_none());
    assert_eq!(
        host.node_input(2).size,
        custody_atom::<S>(left_axis, initial).size
    );
    assert_eq!(
        host.node_input(2).padding,
        Edges::all(LengthOf::px(custody_n::<S>(0.25)))
    );
    let cold = custody_compute(&host, vec![p.binding(0)], &[]);
    custody_assert_simple(&cold, left_axis, initial);
    let cold_fragment_owners = cold
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().clone())
        .collect::<Vec<_>>();
    cold.apply_to(&mut host).unwrap();
    assert!(host.inner.caches.contains_key(&2));
    let committed_pending = custody_entry(&cold, 2, false);
    host.inner.hits.borrow_mut().clear();
    host.returned.borrow_mut().clear();
    let warm = custody_compute(&host, vec![p.binding(0)], &[]);
    assert!(
        host.inner.hits.borrow().contains(&2),
        "actual cache lookup must hit this non-Text atomic child"
    );
    assert!(
        host.returned.borrow().contains(&committed_pending),
        "warm host must return the actual committed unrounded entry"
    );
    // Reaching the actual leaf proves parent admission continued through the
    // complete subtree, rather than rejecting its receipt and recomputing.
    let committed_float = custody_entry(&cold, 4, false);
    assert!(
        host.returned.borrow().contains(&committed_float),
        "warm restoration must traverse the committed float child"
    );
    custody_assert_simple(&warm, left_axis, initial);
    custody_same_scene(&warm, &cold);
    warm.apply_to(&mut host).unwrap();

    // A real final entry cannot serve as unrounded/Pending custody. Existing
    // contract treats this as a cache miss and recomputes, not a second round.
    host.phase = ReturnedCustody::Final;
    host.inner.hits.borrow_mut().clear();
    host.returned.borrow_mut().clear();
    let committed_final = custody_entry(&warm, 2, true);
    let wrong_phase = custody_compute(&host, vec![p.binding(0)], &[]);
    assert!(host.inner.hits.borrow().contains(&2));
    assert!(host.returned.borrow().contains(&committed_final));
    custody_assert_simple(&wrong_phase, left_axis, initial);
    // Opaque unrounded entries, not just their visual output(), must again
    // equal actual Pending custody; accepting a Finalized entry cannot hide.
    custody_same_scene(&wrong_phase, &cold);
    host.phase = ReturnedCustody::Unrounded;
    wrong_phase.apply_to(&mut host).unwrap();

    host.inner.tree.insert_input(
        2,
        LayoutInputOf::box_input(custody_atom(left_axis, changed_offset)),
    );
    let changed = custody_compute(&host, vec![p.binding(0)], &[2]);
    custody_assert_simple(&changed, left_axis, changed_offset);
    let fresh = CustodyHost::new(host.inner.tree.clone());
    let fresh_batch = custody_compute(&fresh, vec![p.binding(0)], &[]);
    custody_same_scene(&changed, &fresh_batch);
    assert_ne!(
        custody_entry(&changed, 2, true).output().size,
        custody_entry(&cold, 2, true).output().size
    );
    // Neither a staged recompute nor later accepted replacement mutates old publication.
    custody_assert_simple(&cold, left_axis, initial);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &wrong_phase);
    changed.apply_to(&mut host).unwrap();
    custody_assert_simple(&cold, left_axis, initial);
    // Within this retained original publication, exact finalized owners remain immutable.
    assert_eq!(
        cold.final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        cold_fragment_owners
    );
}

fn custody_inner_named_recompute<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let outer = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X"), style.clone()),
            ParagraphPart::Atomic {
                node: 2,
                size: text::Size::new(14.4, 14.4),
            },
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let inner = PreparedParagraph::plain(3, Source::new("X"), ahem_style(10.0, 10.0));
    let atom = NodeInputOf {
        display: Display::InlineBlock,
        inline_baseline_context: InlineBaselineContext::Named(BaselineName::Alphabetic),
        position: Position::Relative,
        vertical_align: VerticalAlignOf::Baseline,
        size: Size::new(
            PreferredSizeOf::px(custody_n::<S>(14.4)),
            PreferredSizeOf::px(custody_n::<S>(14.4)),
        ),
        padding: Edges::all(LengthOf::px(custody_n::<S>(0.25))),
        inset: Edges {
            left: LengthAutoOf::px(custody_n::<S>(0.32)),
            top: LengthAutoOf::px(custody_n::<S>(0.72)),
            ..Edges::all(LengthAutoOf::AUTO)
        },
        ..NodeInputOf::default()
    };
    // Reprepare the SAME admitted analyzed owners with actual Alphabetic
    // authority, then reissue all preparation-owned handles used below.
    // PreparedParagraph caches old handles privately; mutating only .prepared
    // would leave foreign-preparation origin/atomic inputs and is not lawful.
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
            ))
            .to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let mut outer_binding = outer.binding(0);
    let mut inner_binding = inner.binding(2);
    for binding in [&mut outer_binding, &mut inner_binding] {
        let analyzed = binding.prepared.analyzed_source();
        let options = binding
            .prepared
            .preparation_options()
            .with_baseline_reference(text::FontBaseline::Alphabetic);
        let prepared = system.prepare(&analyzed, options).unwrap();
        assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
        assert_eq!(prepared.orientation(), text::LineOrientation::Horizontal);
        assert_eq!(
            prepared.baseline_reference(),
            Some(text::FontBaseline::Alphabetic)
        );
        assert_eq!(
            prepared.analyzed_source().typography(),
            text::AnalysisTypography::HorizontalComposition
        );
        assert_eq!(binding.origins.len(), 1);
        let origins = prepared.origin_views().collect::<Vec<_>>();
        assert_eq!(origins.len(), 1);
        binding.origins[0].origin = origins[0].handle();
        let atomics = prepared.atomic_views().collect::<Vec<_>>();
        assert_eq!(atomics.len(), binding.atomics.len());
        for (association, occurrence) in binding.atomics.iter_mut().zip(atomics) {
            assert_eq!(
                occurrence.original_box().id().as_u64(),
                u64::from(association.node)
            );
            association.occurrence = occurrence.handle();
        }
        assert!(binding.controls.is_empty());
        binding.prepared = prepared;
    }
    let mut root = custody_root();
    root.inline_baseline_context = InlineBaselineContext::Named(BaselineName::Alphabetic);
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [3])
        .children(3, [])
        .style(0, root)
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(
                outer_binding.origins[0].origin.clone(),
            )),
        )
        .style(2, atom)
        .input(
            3,
            LayoutInputOf::inline_text(TextParticipation::new(
                inner_binding.origins[0].origin.clone(),
            )),
        );
    let mut host = CustodyHost::new(tree);
    let cold = custody_compute(
        &host,
        vec![outer_binding.clone(), inner_binding.clone()],
        &[],
    );
    let check = |batch: &CompletedLayoutBatchOf<u32, S>| {
        let placed = batch.placed_text();
        let actual = placed.iter().find(|row| row.container() == 2).unwrap();
        assert_eq!(actual.nodes(), &[3]);
        assert_eq!(actual.layout().source().text().code_units(), &[0x58]);
        let lines = actual.layout().lines();
        assert_eq!(lines.len(), 1);
        for run in actual.layout().glyph_runs() {
            assert_eq!(
                run.font().bytes(),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf"
                ))
                .as_slice()
            );
            assert_eq!(run.font_size(), 10.0);
        }
        let node = custody_entry(batch, 2, true).output();
        let named = node.named_baselines.get(BaselineName::Alphabetic);
        // Named readback is relative to the published border, while Text
        // line baselines are global: actual Alphabetic8 + borderTop1 =9.
        assert_eq!(node.location.y, custody_n::<S>(1.0));
        assert_eq!(lines[0].baseline(), 9.0);
        let expected = BaselineFactOf::Available(
            BaselineCoordinateOf::try_new(PhysicalAxis::Vertical, custody_n::<S>(8.0)).unwrap(),
        );
        assert_eq!(named.first(), expected);
        assert_eq!(named.last(), expected);
        let geometry = node.scroll_geometry.unwrap();
        assert_eq!(geometry.border_box().size(), node.size);
        assert_eq!(geometry.padding_box(), geometry.border_box());
        let inner_fragment = batch
            .unrounded_inline_fragments()
            .iter()
            .find(|entry| *entry.owner() == crate::InlineFragmentOwner::Node(3))
            .unwrap()
            .fragment();
        let inner_line = inner_fragment.finalized_fragment().finalized_line();
        // Normal allocation14 starts at visual content inset1 and ends15.
        // Its directed terminal padding1 extends real overflow to16, even
        // though the ten-unit glyph fits the visual content box.
        assert_eq!(
            inner_line
                .selected_line()
                .request()
                .context
                .available_inline,
            Some(14.0)
        );
        assert_eq!(
            inner_line.metric_bounds(),
            text::Rect::new(0.0, 0.0, 14.0, 10.0)
        );
        assert_eq!(
            geometry.padding_box().size(),
            Size::new(custody_n::<S>(15.0), custody_n::<S>(14.0))
        );
        assert_eq!(
            geometry.content_box(),
            ScrollRectOf::try_new(
                Point::new(custody_n::<S>(1.0), S::ZERO),
                Size::new(custody_n::<S>(13.0), custody_n::<S>(14.0))
            )
            .unwrap()
        );
        assert_eq!(
            geometry.scrollable_overflow(),
            ScrollRectOf::try_new(
                Point::ZERO,
                Size::new(custody_n::<S>(16.0), custody_n::<S>(14.0))
            )
            .unwrap()
        );
        let outer_placed = placed.iter().find(|row| row.container() == 0).unwrap();
        let visual = outer_placed.layout().inline_boxes()[0].rect();
        assert_eq!(visual.origin.x, node.location.x.to_f64());
        assert_eq!(visual.origin.y, node.location.y.to_f64());
        assert_eq!(visual.size.width, node.size.width.to_f64());
        assert_eq!(visual.size.height, node.size.height.to_f64());
    };
    check(&cold);
    cold.apply_to(&mut host).unwrap();
    let warm = custody_compute(
        &host,
        vec![outer_binding.clone(), inner_binding.clone()],
        &[],
    );
    check(&warm);
    custody_same_scene(&warm, &cold);
    // Inner native Text makes this atomic a bound ancestor. Do NOT claim an
    // atomic host-cache hit: this control qualifies real named recomputation
    // and coherent readback, separately from the hit-proven float-only atomic lanes.
    assert_eq!(
        custody_entry(&warm, 2, true).output().named_baselines,
        custody_entry(&cold, 2, true).output().named_baselines
    );
}

#[test]
fn relative_atomic_top_retains_warm_phase_custody_f32() {
    custody_roundtrip::<f32>(false);
}
#[test]
fn relative_atomic_top_retains_warm_phase_custody_f64() {
    custody_roundtrip::<f64>(false);
}
#[test]
fn relative_atomic_left_retains_warm_phase_custody_f32() {
    custody_roundtrip::<f32>(true);
}
#[test]
fn relative_atomic_left_retains_warm_phase_custody_f64() {
    custody_roundtrip::<f64>(true);
}
#[test]
fn relative_atomic_real_inner_named_readback_recomputes_f32() {
    custody_inner_named_recompute::<f32>();
}
#[test]
fn relative_atomic_real_inner_named_readback_recomputes_f64() {
    custody_inner_named_recompute::<f64>();
}

mod captured_percentage_basis;

#[path = "fractional_parent_basis.rs"]
mod fractional_parent_basis;

#[test]
fn resize_observer_uses_the_measured_text_placed_atomic_box() {
    fn check<S: LayoutScalar>() {
        let paragraph = custody_paragraph(true);
        let host = CustodyHost::<S>::new(custody_fixture(&paragraph, true, 0.32));
        let batch = custody_compute(&host, vec![paragraph.binding(0)], &[]);
        let atomic = batch.node_layout(2).unwrap().single_box().unwrap();
        let grid =
            crate::ResizeObserverPixelGrid::try_new(crate::Point::new(-0.5, 0.0), 1.0).unwrap();
        let measurement = atomic
            .resize_observer_measurement(WritingMode::HorizontalTb, grid)
            .unwrap()
            .unwrap();
        assert_eq!(atomic.unrounded_output().size.width, S::from_f64(11.0));
        assert_eq!(measurement.border_box().inline, S::from_f64(10.4));
        assert_eq!(measurement.content_box().inline, S::from_f64(9.9));
        assert_eq!(
            measurement.content_rect().origin,
            Point::new(S::from_f64(0.25), S::from_f64(0.25))
        );
        assert_eq!(measurement.device_pixel_content_box().inline, 9);
    }
    check::<f32>();
    check::<f64>();
}
