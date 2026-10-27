//! Actual encounter geometry for indivisible Text units and provisional floats.
use super::*;
use surgeist_text as text;

fn float_tree<S: LayoutScalar>(
    binding: &TextParagraphBinding<u32>,
    width: f64,
    side: Float,
    count: usize,
) -> PublicLayoutTreeOf<S> {
    let mut children = vec![10];
    children.extend((0..count).map(|i| 30 + u32::try_from(i).unwrap()));
    children.push(11);
    let mut tree = PublicLayoutTreeOf::<S>::new().children(2, children).style(
        2,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        },
    );
    for row in &binding.origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for i in 0..count {
        let node = 30 + u32::try_from(i).unwrap();
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: side,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(8.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    tree
}
fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|e| e.node() == node)
        .unwrap()
        .output()
}
fn assert_shared<S: LayoutScalar>(mark: bool, count: usize) {
    let mut binding = super::canonical::paragraph(
        if mark {
            super::canonical::fonts::mark_ahem()
        } else {
            super::canonical::fonts::ligature_ahem()
        },
        if mark { &["A", "\u{301}"] } else { &["A", "A"] },
    );
    binding.nodes = vec![10];
    binding
        .nodes
        .extend((0..count).map(|i| 30 + u32::try_from(i).unwrap()));
    binding.nodes.push(11);
    let tree = float_tree::<S>(&binding, 20.0, Float::Left, count);
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(20.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(
        layout.source().text().code_units(),
        if mark { &[65, 769][..] } else { &[65, 65][..] }
    );
    assert_eq!(
        layout.glyph_runs().iter().map(|r| r.advance()).sum::<f64>(),
        12.5
    );
    // Whole12.5+float8 exceeds20. Extraction-half6.25 would incorrectly fit.
    // The real16px line therefore owns the entire first row; both floats
    // encounter that same shared text prefix, then accumulate below it.
    for i in 0..count {
        let actual = output(&batch, 30 + u32::try_from(i).unwrap());
        assert_eq!(
            actual.location,
            Point::new(S::from_f64(i as f64 * 8.0), S::from_f64(16.0))
        );
    }
    assert_eq!(output(&batch, 2).size.height, S::from_f64(20.0));
    let warm = compute_layout_with_flow(&capture, 2, request(20.0)).unwrap();
    assert_eq!(warm.final_entries(), batch.final_entries());
    let facts = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch
            .final_inline_fragments()
            .iter()
            .map(|entry| {
                let fragment = entry.fragment();
                assert_eq!(
                    fragment
                        .finalized_fragment()
                        .finalized_line()
                        .prepared_text(),
                    binding.prepared
                );
                (
                    entry.owner().clone(),
                    fragment.rect(),
                    fragment.baseline(),
                    fragment.line_index(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(facts(&warm), facts(&batch));
}
#[test]
fn float_encounter_counts_actual_whole_ligature_and_mark_f32() {
    assert_shared::<f32>(false, 1);
    assert_shared::<f32>(true, 1);
}
#[test]
fn float_encounter_counts_actual_whole_ligature_and_mark_f64() {
    assert_shared::<f64>(false, 1);
    assert_shared::<f64>(true, 1);
}
#[test]
fn two_float_encounters_share_one_text_prefix_and_accumulate_placements() {
    assert_shared::<f32>(false, 2);
    assert_shared::<f64>(true, 2);
}

fn assert_ordinary<S: LayoutScalar>(rtl: bool, side: Float) {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        if rtl {
            text::ParagraphBaseDirection::RightToLeft
        } else {
            text::ParagraphBaseDirection::LeftToRight
        },
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let mut tree = float_tree::<S>(&binding, 30.0, side, 1);
    tree = tree.style(
        2,
        NodeInputOf {
            display: Display::Block,
            direction: if rtl { Direction::Rtl } else { Direction::Ltr },
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        },
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(30.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(output(&batch, 2).size.height, S::from_f64(10.0));
    assert_eq!(output(&batch, 30).location.y, S::ZERO);
    // Preserve the existing normalized logical-side mapping through FlowAxes.
    assert_eq!(
        output(&batch, 30).location.x,
        S::from_f64(if (side == Float::Left) != rtl {
            0.0
        } else {
            22.0
        })
    );
}
#[test]
fn ordinary_float_encounter_preserves_one_paragraph_in_both_directions_and_sides() {
    for rtl in [false, true] {
        for side in [Float::Left, Float::Right] {
            assert_ordinary::<f32>(rtl, side);
            assert_ordinary::<f64>(rtl, side);
        }
    }
}

/// An actual committed host cache and batch sink. The tree remains immutable
/// throughout each computation; only a complete accepted batch can replace it.
pub(super) struct CachedHost<S: LayoutScalar> {
    pub(super) tree: PublicLayoutTreeOf<S>,
    caches: std::collections::HashMap<u32, CacheOf<S>>,
    published: Option<CompletedLayoutBatchOf<u32, S>>,
    hits: std::cell::RefCell<Vec<u32>>,
    fail_shape: bool,
    reject_batch: bool,
}
impl<S: LayoutScalar> CachedHost<S> {
    pub(super) fn new(tree: PublicLayoutTreeOf<S>) -> Self {
        Self {
            tree,
            caches: std::collections::HashMap::new(),
            published: None,
            hits: Default::default(),
            fail_shape: false,
            reject_batch: false,
        }
    }
    fn compute(
        &self,
        binding: &TextParagraphBinding<u32>,
        dirty: &[u32],
    ) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S, &'static str> {
        let capture = LayoutFlowInput::try_new(
            self,
            2,
            vec![binding.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )?;
        compute_layout_with_flow_invalidated(&capture, 2, request(30.0), dirty)
    }
}
impl<S: LayoutScalar> Traverse for CachedHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;
    fn children(&self, n: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, n)
    }
    fn child_count(&self, n: u32) -> usize {
        self.tree.child_count(n)
    }
    fn child(&self, n: u32, i: usize) -> u32 {
        self.tree.child(n, i)
    }
}
impl<S: LayoutScalar> LayoutTree for CachedHost<S> {
    type MeasureError = &'static str;
    fn node_input(&self, n: u32) -> &NodeInputOf<S> {
        self.tree.node_input(n)
    }
    fn layout_input(&self, n: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(n)
    }
    fn cache_get(
        &self,
        n: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        let value = self
            .caches
            .get(&n)
            .and_then(|cache| cache.get_with_context(input, context));
        if value.is_some() {
            self.hits.borrow_mut().push(n);
        }
        value
    }
    fn unrounded_layout(&self, n: u32) -> Option<NodeOutputOf<S>> {
        self.published
            .as_ref()?
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == n)
            .map(|row| row.output())
    }
    fn float_exclusion_interval(
        &self,
        _: u32,
        _: FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<FloatExclusionIntervalOf<S>>, &'static str>> {
        Some(if self.fail_shape {
            Err("actual float shape refused")
        } else {
            Ok(None)
        })
    }
}
impl<S: LayoutScalar> LayoutBatchSink<u32, S> for CachedHost<S> {
    type Error = &'static str;
    type Prepared = (
        std::collections::HashMap<u32, CacheOf<S>>,
        CompletedLayoutBatchOf<u32, S>,
    );
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, S>,
    ) -> Result<Self::Prepared, Self::Error> {
        if self.reject_batch {
            return Err("reject complete replacement");
        }
        let mut caches = self.caches.clone();
        for node in batch.invalidated_nodes() {
            caches.remove(node);
        }
        for row in batch.cache_clear_entries() {
            caches.remove(&row.node());
        }
        for row in batch.cache_store_entries() {
            caches.entry(row.node()).or_default().store_with_context(
                row.input(),
                row.context(),
                row.output(),
            );
        }
        Ok((caches, batch.clone()))
    }
    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.caches = prepared.0;
        self.published = Some(prepared.1);
    }
}
fn assert_cache_and_failure<S: LayoutScalar>(side: Float, width: f64, expected: Point<S>) {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<S>(&binding, 30.0, side, 1);
    let mut host = CachedHost::new(tree);
    let cold = host.compute(&binding, &[]).unwrap();
    cold.apply_to(&mut host).unwrap();
    assert!(host.caches.contains_key(&30));
    host.hits.borrow_mut().clear();
    let warm = host.compute(&binding, &[]).unwrap();
    assert!(
        host.hits.borrow().contains(&30),
        "actual float child request must hit committed cache"
    );
    assert_eq!(warm.final_entries(), cold.final_entries());
    warm.apply_to(&mut host).unwrap();
    host.tree.insert_input(
        2,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        }),
    );
    let changed = host.compute(&binding, &[2]).unwrap();
    assert_eq!(output(&changed, 30).location, expected);
    let fresh = CachedHost::new(host.tree.clone())
        .compute(&binding, &[])
        .unwrap();
    assert_eq!(changed.final_entries(), fresh.final_entries());
    changed.apply_to(&mut host).unwrap();
    let old_cache = host.caches.clone();
    let old = host.published.clone().unwrap();
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            float: side,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(8.0)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            ),
            ..NodeInputOf::default()
        }),
    );
    host.tree.insert_input(
        2,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        }),
    );
    host.fail_shape = true;
    let error = host.compute(&binding, &[30]).unwrap_err();
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::Measurement("actual float shape refused")
    ));
    assert_eq!(host.caches, old_cache);
    assert_eq!(
        host.published.as_ref().unwrap().final_entries(),
        old.final_entries()
    );
    assert_eq!(
        host.published.as_ref().unwrap().final_inline_fragments(),
        old.final_inline_fragments()
    );
    assert_eq!(
        host.published.as_ref().unwrap().placed_text()[0]
            .layout()
            .positions()
            .collect::<Vec<_>>(),
        old.placed_text()[0]
            .layout()
            .positions()
            .collect::<Vec<_>>()
    );
    host.fail_shape = false;
    let replacement = host.compute(&binding, &[30]).unwrap();
    host.reject_batch = true;
    assert_eq!(
        replacement.apply_to(&mut host),
        Err("reject complete replacement")
    );
    assert_eq!(host.caches, old_cache);
    assert_eq!(
        host.published.as_ref().unwrap().final_entries(),
        old.final_entries()
    );
}
#[test]
fn float_encounter_cached_child_changed_frame_and_late_failure_are_atomic_f32() {
    assert_cache_and_failure::<f32>(Float::Left, 20.0, Point::new(0.0, 10.0));
}
#[test]
fn float_encounter_cached_child_changed_frame_and_late_failure_are_atomic_f64() {
    assert_cache_and_failure::<f64>(Float::Left, 20.0, Point::new(0.0, 10.0));
}

#[test]
fn float_encounter_cached_child_wider_frame_and_failure_keep_all_channels() {
    assert_cache_and_failure::<f32>(Float::Right, 40.0, Point::new(32.0, 0.0));
    assert_cache_and_failure::<f64>(Float::Right, 40.0, Point::new(32.0, 0.0));
}

/// Candidate-local missing case, deliberately not dispatched until the replay
/// floor review: the genuine empty origin is a zero anchor, not a preceding
/// generated line box. X cannot fit the two pixels beside the eight-pixel
/// float. The float remains at its original encounter and X starts below it.
#[test]
fn leading_empty_origin_float_keeps_its_encounter_while_text_avoids_its_band() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new(""), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<f64>(&binding, 10.0, Float::Left, 1);
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(10.0)).unwrap();
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(
        batch.placed_text()[0].layout().lines()[0].bounds().origin.y,
        4.0
    );
    assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 12.0);
    assert_eq!(output(&batch, 2).size.height, 14.0);
}

fn assert_soft_replay<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A "), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<S>(&binding, 25.0, Float::Left, 1);
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(25.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), 2);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(layout.lines()[1].range(), text::Range::new(2, 3));
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(layout.lines()[1].baseline(), 18.0);
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    let first = batch
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let origin = paragraph.origin(10);
    let at = text::TextSeam::Origin {
        local: text::SourcePosition::try_new(origin.view().source().text(), 2).unwrap(),
        origin,
    };
    assert_eq!(first.feedback().advance, 10.0);
    assert_eq!(first.prefix_at(&at).unwrap().advance(), 10.0);
    // The longer candidate's interior SPACE contributes10; the actually chosen
    // soft line discards it. Float placement must come from this candidate.
    assert_eq!(output(&batch, 2).size.height, S::from_f64(20.0));
}
#[test]
fn float_encounter_replays_the_actual_soft_line_edge_discard() {
    assert_soft_replay::<f32>();
    assert_soft_replay::<f64>();
}

fn assert_clear<S: LayoutScalar>() {
    let mut binding =
        super::canonical::paragraph(super::canonical::fonts::ligature_ahem(), &["A", "A"]);
    binding.nodes = vec![10, 30, 31, 11];
    let tree = float_tree::<S>(&binding, 20.0, Float::Left, 2).style(
        31,
        NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            clear: Clear::Both,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(8.0)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            ),
            ..NodeInputOf::default()
        },
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(20.0)).unwrap();
    assert_eq!(
        output(&batch, 30).location,
        Point::new(S::ZERO, S::from_f64(16.0))
    );
    assert_eq!(
        output(&batch, 31).location,
        Point::new(S::ZERO, S::from_f64(20.0))
    );
    assert_eq!(output(&batch, 2).size.height, S::from_f64(24.0));
}
#[test]
fn float_encounter_retains_clearance_across_two_same_unit_events() {
    assert_clear::<f32>();
    assert_clear::<f64>();
}

fn assert_signed<S: LayoutScalar>(spacing: f64) {
    let mut first = ahem_style(10.0, 10.0);
    first.letter_spacing = spacing;
    let mut second = ahem_style(10.0, 10.0);
    second.letter_spacing = -10.0;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), first.clone()),
            ParagraphPart::text(11, Source::new("X"), second),
        ],
        first,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 31, 11];
    let mut tree = float_tree::<S>(&binding, 12.0, Float::Left, 2);
    for node in [30, 31] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(4.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(12.0)).unwrap();
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    assert_eq!(
        output(&batch, 31).location,
        Point::new(S::from_f64(4.0), S::ZERO)
    );
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        &[65, 88]
    );
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 1);
    let line = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let origin = paragraph.origin(10);
    let at = text::TextSeam::Origin {
        local: text::SourcePosition::try_new(origin.view().source().text(), 1).unwrap(),
        origin,
    };
    assert_eq!(line.prefix_at(&at).unwrap().advance(), 10.0 + spacing);
    assert_eq!(output(&batch, 2).size.height, S::from_f64(10.0));
}
#[test]
fn float_encounter_source_progress_is_independent_of_zero_or_negative_prefix() {
    for spacing in [-10.0, -12.5] {
        assert_signed::<f32>(spacing);
        assert_signed::<f64>(spacing);
    }
}

/// The prefix has a real earlier-content line, unlike the genuine leading
/// EmptyOrigin case. CSS2's preceding-line floor prevents moving A below its
/// own encountered float. A/X remain one unbreakable word; defer the float.
#[test]
fn preceding_unbreakable_content_keeps_its_line_and_defers_the_encountered_float() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    assert!(
        paragraph
            .prepared
            .opportunity_views()
            .all(|row| row.kind() == text::PreparedBreakKind::ParagraphEnd)
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30, 11];
    let tree = float_tree::<f64>(&binding, 20.0, Float::Left, 1);
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(20.0)).unwrap();
    assert_eq!(output(&batch, 30).location, Point::new(0.0, 10.0));
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 1);
    assert_eq!(
        batch.placed_text()[0].layout().lines()[0].bounds().origin.y,
        0.0
    );
    assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 8.0);
    assert_eq!(output(&batch, 2).size.height, 14.0);
}

fn assert_discarded_prefix<S: LayoutScalar>() {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new(" "), style.clone()),
            ParagraphPart::text(11, Source::new(""), style.clone()),
            ParagraphPart::text(12, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 11, 30, 12];
    let tree = float_tree::<S>(&binding, 10.0, Float::Left, 1).children(2, binding.nodes.clone());
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(10.0)).unwrap();
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), [32, 88]);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].bounds().origin.y, 4.0);
    assert_eq!(layout.lines()[0].baseline(), 12.0);
    assert_eq!(output(&batch, 2).size.height, S::from_f64(14.0));
}
#[test]
fn discarded_leading_space_and_empty_origin_do_not_defer_the_encountered_float() {
    assert_discarded_prefix::<f32>();
    assert_discarded_prefix::<f64>();
}

#[test]
fn leading_float_uses_the_actual_legal_break_reshaped_association() {
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 12.5,
        line_height: text::LineHeight::Absolute(16.0),
        letter_spacing: 5.0,
        word_break: text::WordBreak::BreakAll,
        ..text::Style::default()
    };
    let analyzed = text::AnalyzedSource::new(Source::new("AA"), style).unwrap();
    let mut fonts = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    fonts
        .register_fonts(super::canonical::fonts::ligature_ahem())
        .unwrap();
    let prepared = fonts
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let binding = TextParagraphBinding {
        prepared,
        container: 2,
        nodes: vec![30, 10],
        origins: vec![TextOriginBinding {
            node: 10,
            origin: origin.clone(),
        }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    let whole =
        crate::test_support::text_fragment::fragment_for(&binding.prepared).finalized_line();
    let seam = text::TextSeam::Origin {
        origin: origin.clone(),
        local: text::SourcePosition::try_new(origin.view().source().text(), 1).unwrap(),
    };
    assert!(whole.prefix_at(&seam).unwrap().interior_unit().is_some());
    assert_eq!(whole.feedback().advance, 22.5);
    let tree = float_tree::<f64>(&binding, 26.0, Float::Left, 1).children(2, binding.nodes.clone());
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(26.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.lines().len(), 2);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 1));
    assert_eq!(layout.lines()[1].range(), text::Range::new(1, 2));
    assert_eq!(output(&batch, 30).location, Point::ZERO);
    let first = batch
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert!(first.prefix_at(&seam).unwrap().interior_unit().is_none());
    assert_eq!(first.feedback().advance, 17.5);
    assert_eq!(
        first
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35]
    );
    assert_eq!(
        whole
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [58]
    );
}

#[test]
fn discretionary_control_float_seam_lends_the_actual_selected_replacement_bundle() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::with_replacements(
        vec![
            ParagraphPart::text(10, Source::new("AX"), style.clone()),
            ParagraphPart::Control {
                node: 20,
                kind: ParagraphControlKind::DiscretionaryBreak {
                    style: Box::new(style.clone()),
                },
            },
            ParagraphPart::text(11, Source::new("XXX"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
        vec![
            text::ParagraphReplacement::resolved_native_logical_before(
                text::ParagraphReplacementDefinition::Control {
                    control: 0,
                    origin: None,
                },
                2,
                vec!["X".into()],
            )
            .unwrap(),
        ],
    );
    let mut binding = paragraph.binding(2);
    let alternative = binding
        .prepared
        .alternative_views()
        .next()
        .unwrap()
        .handle();
    let control = binding.controls[0].occurrence.clone();
    binding.nodes = vec![10, 20, 30, 11];
    let tree = float_tree::<f64>(&binding, 35.0, Float::Left, 1)
        .children(2, binding.nodes.clone())
        .children(20, [])
        .input(
            20,
            LayoutInputOf::text_control(TextControlParticipation::new(control.clone())),
        );
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(35.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), [65, 88, 88, 88, 88]);
    assert_eq!(layout.lines().len(), 2);
    let first = batch
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let observation = first
        .prefix_at(&text::TextSeam::AfterParticipant(
            text::PreparedParticipant::Control(control),
        ))
        .unwrap();
    assert_eq!(observation.advance(), 30.0);
    assert!(observation.has_retained_flow_participation());
    assert_eq!(output(&batch, 30).location, Point::new(0.0, 10.0));
    let replacement = first
        .fragments()
        .find(|fragment| {
            matches!(
                fragment.view().owner(),
                text::FinalizedFragmentOwner::Alternative(_)
            )
        })
        .unwrap();
    assert_eq!(
        replacement.view().owner(),
        text::FinalizedFragmentOwner::Alternative(alternative)
    );
    assert_eq!(replacement.view().aggregate_range(), None);
}

fn assert_retained_zero_prefix<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let mut zero = style.clone();
    zero.letter_spacing = -10.0;
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A"), zero),
            ParagraphPart::text(11, Source::new(""), style.clone()),
            ParagraphPart::text(12, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 11, 30, 12];
    let tree = float_tree::<S>(&binding, 10.0, Float::Left, 1).children(2, binding.nodes.clone());
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(10.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), [65, 88]);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].bounds().origin.y, 0.0);
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(
        output(&batch, 30).location,
        Point::new(S::ZERO, S::from_f64(10.0))
    );
    assert_eq!(output(&batch, 2).size.height, S::from_f64(14.0));
    let first = batch
        .final_inline_fragments()
        .iter()
        .find(|row| row.fragment().line_index() == 0)
        .unwrap()
        .fragment()
        .finalized_fragment()
        .finalized_line();
    let origin = paragraph.origin(11);
    let local = text::SourcePosition::try_new(origin.view().source().text(), 0).unwrap();
    let prefix = first
        .prefix_at(&text::TextSeam::Origin { origin, local })
        .unwrap();
    assert_eq!(prefix.advance(), 0.0);
    assert!(prefix.has_retained_flow_participation());
}
#[test]
fn retained_zero_advance_before_empty_origin_defers_float_instead_of_moving_its_text() {
    assert_retained_zero_prefix::<f32>();
    assert_retained_zero_prefix::<f64>();
}

fn assert_nested_float_text_packet<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let parent = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("A\n"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let child = PreparedParagraph::plain(40, Source::new("Z"), ahem_style(5.0, 5.0));
    let mut outer = parent.binding(2);
    outer.nodes = vec![10, 30, 11];
    let inner = child.binding(30);
    let tree = float_tree::<S>(&outer, 10.0, Float::Left, 1)
        .children(30, [40])
        .style(
            30,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(S::from_f64(8.0)), PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .children(40, [])
        .input(40, child.text_input(40));
    let compute = |host: &CachedHost<S>| {
        let capture = LayoutFlowInput::try_new(
            host,
            2,
            vec![outer.clone(), inner.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )?;
        compute_layout_with_flow_invalidated(&capture, 2, request(10.0), &[2])
    };
    let mut host = CachedHost::new(tree);
    let cold = compute(&host).unwrap();
    assert_eq!(cold.placed_text().len(), 2);
    let outer_layout = cold
        .placed_text()
        .iter()
        .find(|p| p.container() == 2)
        .unwrap()
        .layout();
    let child_layout = cold
        .placed_text()
        .iter()
        .find(|p| p.container() == 30)
        .unwrap()
        .layout();
    assert_eq!(outer_layout.source().text().code_units(), [65, 10, 88]);
    assert_eq!(child_layout.source().text().code_units(), [90]);
    assert_eq!(outer_layout.lines().len(), 2);
    assert_eq!(outer_layout.lines()[0].baseline(), 8.0);
    assert_eq!(outer_layout.lines()[1].baseline(), 23.0);
    assert_eq!(child_layout.lines().len(), 1);
    assert_eq!(child_layout.lines()[0].bounds().origin.y, 10.0);
    assert_eq!(child_layout.lines()[0].baseline(), 14.0);
    assert_eq!(
        output(&cold, 30).location,
        Point::new(S::ZERO, S::from_f64(10.0))
    );
    assert_eq!(
        output(&cold, 30).size,
        Size::new(S::from_f64(8.0), S::from_f64(5.0))
    );
    assert_eq!(output(&cold, 2).size.height, S::from_f64(25.0));
    for row in cold.final_inline_fragments() {
        let actual = row
            .fragment()
            .finalized_fragment()
            .finalized_line()
            .prepared_text();
        assert_eq!(
            actual,
            if row.owner() == &crate::InlineFragmentOwner::Node(40) {
                child.prepared.clone()
            } else {
                parent.prepared.clone()
            }
        );
    }
    cold.apply_to(&mut host).unwrap();
    let warm = compute(&host).unwrap();
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(warm.placed_text().len(), 2);
    let facts = |batch: &CompletedLayoutBatchOf<u32, S>| {
        let mut facts = batch
            .placed_text()
            .iter()
            .map(|p| {
                (
                    p.container(),
                    p.layout()
                        .lines()
                        .iter()
                        .map(|line| (line.range(), line.bounds(), line.baseline()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        facts.sort_by_key(|p| p.0);
        facts
    };
    assert_eq!(facts(&warm), facts(&cold));
    warm.apply_to(&mut host).unwrap();
    let prior_caches = host.caches.clone();
    let prior = host.published.as_ref().unwrap().clone();
    let positions = |batch: &CompletedLayoutBatchOf<u32, S>| {
        batch
            .placed_text()
            .iter()
            .map(|p| (p.container(), p.layout().positions().collect::<Vec<_>>()))
            .collect::<Vec<_>>()
    };
    let prior_positions = positions(&prior);
    // Qualify the late provider stimulus while preserving the rectangular
    // positive case. Ok(None) for a genuine Shape means no shape exclusion.
    host.tree.insert_input(
        30,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: FloatExclusion::Shape,
            size: Size::new(PreferredSizeOf::px(S::from_f64(8.0)), PreferredSizeOf::AUTO),
            ..NodeInputOf::default()
        }),
    );
    host.fail_shape = true;
    assert!(matches!(
        compute(&host).unwrap_err().kind(),
        LayoutErrorKindOf::Measurement("actual float shape refused")
    ));
    assert_eq!(host.caches, prior_caches);
    assert_eq!(
        host.published.as_ref().unwrap().final_entries(),
        prior.final_entries()
    );
    assert_eq!(
        host.published.as_ref().unwrap().final_inline_fragments(),
        prior.final_inline_fragments()
    );
    assert_eq!(positions(host.published.as_ref().unwrap()), prior_positions);
}
#[test]
fn accepted_float_child_text_packet_publishes_both_paragraphs_and_rolls_back_late_failure() {
    assert_nested_float_text_packet::<f32>();
    assert_nested_float_text_packet::<f64>();
}

fn captured_float_intrinsic<S: LayoutScalar>(side: Float, minimum: bool) -> (f64, f64) {
    let paragraph = PreparedParagraph::plain(10, Source::new("XX"), ahem_style(10.0, 10.0));
    let mut binding = paragraph.binding(2);
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(2, [30, 10])
        .children(30, [])
        .children(10, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            30,
            NodeInputOf {
                display: Display::Block,
                float: side,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .input(10, paragraph.text_input(10));
    let request = LayoutRootRequestOf::viewport(Size::new(
        if minimum {
            AvailableOf::MIN_CONTENT
        } else {
            AvailableOf::MAX_CONTENT
        },
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    // Existing ordinary-float lane is an independent control using the same
    // actual preparation and box geometry, without interleaved-window capture.
    let ordinary = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let ordinary = compute_layout_with_flow(&ordinary, 2, request).unwrap();
    let ordinary_width = output(&ordinary, 2).size.width.to_f64();
    binding.nodes = vec![30, 10];
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let actual = compute_layout_with_flow(&capture, 2, request).unwrap();
    assert_eq!(
        actual.placed_text()[0]
            .layout()
            .source()
            .text()
            .code_units(),
        [88, 88]
    );
    assert_eq!(output(&actual, 30).size.width, S::from_f64(30.0));
    (ordinary_width, output(&actual, 2).size.width.to_f64())
}
#[test]
fn captured_float_max_content_retains_float_and_unbroken_text_width() {
    let actual: Vec<_> = [Float::Left, Float::Right]
        .into_iter()
        .flat_map(|side| {
            [
                (side, "f32", captured_float_intrinsic::<f32>(side, false)),
                (side, "f64", captured_float_intrinsic::<f64>(side, false)),
            ]
        })
        .collect();
    // The ordinary right-float path is a separately reported sizing defect;
    // this captured-window operation owns the independently required 50px.
    for (side, scalar, (ordinary, captured)) in actual {
        if side == Float::Left {
            assert_eq!(ordinary, 50.0, "ordinary left control {scalar}");
        }
        assert_eq!(captured, 50.0, "captured {side:?} {scalar}");
    }
}
#[test]
fn captured_float_min_content_retains_the_larger_separate_width() {
    let actual: Vec<_> = [Float::Left, Float::Right]
        .into_iter()
        .flat_map(|side| {
            [
                (side, "f32", captured_float_intrinsic::<f32>(side, true)),
                (side, "f64", captured_float_intrinsic::<f64>(side, true)),
            ]
        })
        .collect();
    let expected: Vec<_> = [Float::Left, Float::Right]
        .into_iter()
        .flat_map(|side| [(side, "f32", (30.0, 30.0)), (side, "f64", (30.0, 30.0))])
        .collect();
    assert_eq!(
        actual, expected,
        "each row is (side,scalar,(ordinary,captured))"
    );
}
#[test]
fn trailing_float_after_terminal_forced_break_retains_the_real_empty_paragraph_line() {
    let paragraph = PreparedParagraph::plain(10, Source::new("A\n"), ahem_style(10.0, 10.0));
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![10, 30];
    let tree = float_tree::<f64>(&binding, 40.0, Float::Left, 1).children(2, binding.nodes.clone());
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(40.0)).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text().code_units(), [65, 10]);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(layout.lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(layout.lines()[0].baseline(), 8.0);
    assert_eq!(layout.metrics().size().height, 10.0);
    let completions = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].origin.y, 10.0);
    let receipt = completions[0].line.finalized_line();
    assert_eq!(receipt.prepared_text(), paragraph.prepared);
    assert!(!receipt.has_in_flow_content());
    assert_eq!(receipt.feedback().line_height, 10.0);
    let selected = receipt.selected_line();
    assert_eq!(selected.request().context.end, text::LineEnd::Paragraph);
    assert!(matches!(&selected.request().termination,
        text::PreparedTermination::Ordinary(offer)
            if offer.view().kind() == text::PreparedBreakKind::ParagraphEnd));
    assert_eq!(output(&batch, 30).location, Point::new(0.0, 10.0));
    // The phantom contributes no line height. The genuine8x4 float still
    // sets its owner's auto bottom to10+4, independently of Text's10px extent.
    assert_eq!(output(&batch, 30).size, Size::new(8.0, 4.0));
    assert_eq!(output(&batch, 2).size.height, 14.0);
}

fn assert_max_float_cohort<S: LayoutScalar>(
    source: &str,
    floats: &[(Float, f64, f64, Clear)],
    interleaved: bool,
    expected_width: f64,
    expected_floats: &[(f64, f64)],
) {
    let style = ahem_style(10.0, 10.0);
    let paragraph = if interleaved {
        PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("A"), style.clone()),
                ParagraphPart::text(11, Source::new("X"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        )
    } else {
        PreparedParagraph::plain(10, Source::new(source), style)
    };
    let mut binding = paragraph.binding(2);
    let nodes: Vec<_> = if interleaved {
        std::iter::once(10)
            .chain((0..floats.len()).map(|i| 30 + i as u32))
            .chain(std::iter::once(11))
            .collect()
    } else {
        (0..floats.len())
            .map(|i| 30 + i as u32)
            .chain(std::iter::once(10))
            .collect()
    };
    binding.nodes = nodes.clone();
    let mut tree = PublicLayoutTreeOf::<S>::new().children(2, nodes).style(
        2,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    );
    for row in &binding.origins {
        tree = tree
            .children(row.node, [])
            .input(row.node, paragraph.text_input(row.node));
    }
    for (i, &(side, width, height, clear)) in floats.iter().enumerate() {
        let node = 30 + i as u32;
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: side,
                clear,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::px(S::from_f64(height)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MAX_CONTENT,
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request).unwrap();
    assert_eq!(
        output(&batch, 2).size.width,
        S::from_f64(expected_width),
        "source {source:?}, floats {floats:?}"
    );
    for (i, &(x, y)) in expected_floats.iter().enumerate() {
        assert_eq!(
            output(&batch, 30 + i as u32).location,
            Point::new(S::from_f64(x), S::from_f64(y)),
            "float {i} source {source:?}"
        );
    }
    let warm = compute_layout_with_flow(&capture, 2, request).unwrap();
    assert_eq!(warm.final_entries(), batch.final_entries());
}
fn max_float_cohorts<S: LayoutScalar>() {
    // Each side contributes once to the same unwrapped 20px line.
    assert_max_float_cohort::<S>(
        "XX",
        &[
            (Float::Left, 30.0, 4.0, Clear::None),
            (Float::Right, 15.0, 4.0, Clear::None),
        ],
        false,
        65.0,
        &[(0.0, 0.0), (50.0, 0.0)],
    );
    assert_max_float_cohort::<S>(
        "XX",
        &[
            (Float::Left, 10.0, 4.0, Clear::None),
            (Float::Left, 15.0, 4.0, Clear::None),
            (Float::Right, 5.0, 4.0, Clear::None),
        ],
        false,
        50.0,
        &[(0.0, 0.0), (10.0, 0.0), (45.0, 0.0)],
    );
    // The second forced line's 80px only shares occupation with a tall float.
    assert_max_float_cohort::<S>(
        "X\nXXXXXXXX",
        &[(Float::Left, 30.0, 4.0, Clear::None)],
        false,
        80.0,
        &[(0.0, 0.0)],
    );
    assert_max_float_cohort::<S>(
        "X\nXXXXXXXX",
        &[(Float::Left, 30.0, 20.0, Clear::None)],
        false,
        110.0,
        &[(0.0, 0.0)],
    );
    // Clear starts a new cohort; it cannot add both float widths globally.
    assert_max_float_cohort::<S>(
        "XX",
        &[
            (Float::Left, 30.0, 20.0, Clear::None),
            (Float::Right, 15.0, 4.0, Clear::Both),
        ],
        false,
        50.0,
        &[(0.0, 0.0), (35.0, 20.0)],
    );
    // The intrinsic first pass has no finite zero-capacity deferral.
    assert_max_float_cohort::<S>(
        "AX",
        &[(Float::Left, 30.0, 4.0, Clear::None)],
        true,
        50.0,
        &[(0.0, 0.0)],
    );
    assert_max_float_cohort::<S>(
        "AX",
        &[(Float::Right, 30.0, 4.0, Clear::None)],
        true,
        50.0,
        &[(20.0, 0.0)],
    );
}
#[test]
fn captured_max_float_occupation_tracks_actual_line_clear_and_expiry_cohorts() {
    max_float_cohorts::<f32>();
    max_float_cohorts::<f64>();
}

fn expired_captured_max_cohorts<S: LayoutScalar>() -> f64 {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(10, Source::new("X\n"), style.clone()),
            ParagraphPart::text(11, Source::new("X"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(2);
    binding.nodes = vec![30, 10, 31, 11];
    let mut tree = float_tree::<S>(&binding, 0.0, Float::Left, 2)
        .children(2, binding.nodes.clone())
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for (node, width) in [(30, 30.0), (31, 40.0)] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MAX_CONTENT,
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request).unwrap();
    assert_eq!(output(&batch, 30).location, Point::new(S::ZERO, S::ZERO));
    assert_eq!(
        output(&batch, 31).location,
        Point::new(S::ZERO, S::from_f64(10.0))
    );
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 2);
    output(&batch, 2).size.width.to_f64()
}
#[test]
fn captured_review_expired_max_cohorts_do_not_sum_disjoint_float_widths() {
    // Actual first/second rows need 30+10=40 and 40+10=50, respectively.
    assert_eq!(
        [
            expired_captured_max_cohorts::<f32>(),
            expired_captured_max_cohorts::<f64>()
        ],
        [50.0, 50.0]
    );
}
fn source_ordered_clear_tops<S: LayoutScalar>(mirror: bool, capture_all: bool) -> Vec<f64> {
    let paragraph = PreparedParagraph::plain(10, Source::new("XX"), ahem_style(10.0, 10.0));
    let mut binding = paragraph.binding(2);
    let children = vec![30, 31, 32, 10];
    binding.nodes = if capture_all {
        children.clone()
    } else {
        vec![32, 10]
    };
    let mut tree = float_tree::<S>(&binding, 100.0, Float::Left, 3).children(2, children);
    for (node, width, height, clear, left) in [
        (30, 30.0, 20.0, Clear::None, true),
        (31, 15.0, 4.0, Clear::Both, false),
        (32, 10.0, 4.0, Clear::None, true),
    ] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: if left != mirror {
                    Float::Left
                } else {
                    Float::Right
                },
                clear,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::px(S::from_f64(height)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request(100.0)).unwrap();
    [30, 31, 32]
        .into_iter()
        .map(|node| output(&batch, node).location.y.to_f64())
        .collect()
}
#[test]
fn captured_review_later_float_stays_below_earlier_cleared_float_top() {
    // CSS2 float rule5: a later float cannot have a higher top than an earlier
    // float. Clear moved the second to20; the third fits at that same top.
    let actual = [
        source_ordered_clear_tops::<f32>(false, true),
        source_ordered_clear_tops::<f64>(false, true),
        source_ordered_clear_tops::<f32>(true, true),
        source_ordered_clear_tops::<f64>(true, true),
    ];
    assert_eq!(
        actual,
        [
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0]
        ]
    );
}
fn transparent_leading_capture<S: LayoutScalar>(
    absolute: bool,
    with_float: bool,
) -> Result<(), String> {
    let paragraph = PreparedParagraph::plain(10, Source::new("X"), ahem_style(10.0, 10.0));
    let mut binding = paragraph.binding(2);
    binding.nodes = if with_float {
        vec![90, 30, 10]
    } else {
        vec![90, 10]
    };
    let mut tree = float_tree::<S>(&binding, 40.0, Float::Left, usize::from(with_float))
        .children(2, binding.nodes.clone())
        .children(90, [91])
        .children(91, [])
        .style(
            90,
            NodeInputOf {
                display: if absolute {
                    Display::Block
                } else {
                    Display::None
                },
                position: if absolute {
                    Position::Absolute
                } else {
                    Position::Relative
                },
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(3.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    tree = tree.style(
        91,
        NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(7.0)),
                PreferredSizeOf::px(S::from_f64(6.0)),
            ),
            ..NodeInputOf::default()
        },
    );
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .map_err(|e| format!("admission {e:?}"))?;
    let batch = compute_layout_with_flow(&capture, 2, request(40.0))
        .map_err(|e| format!("consumption {e:?}"))?;
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        [88]
    );
    assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 8.0);
    assert_eq!(output(&batch, 2).size.height, S::from_f64(10.0));
    assert_eq!(
        output(&batch, 90).size,
        if absolute {
            Size::new(S::from_f64(3.0), S::from_f64(4.0))
        } else {
            Size::ZERO
        }
    );
    if !absolute {
        assert_eq!(output(&batch, 91).size, Size::ZERO);
    }
    if with_float {
        assert_eq!(output(&batch, 30).location, Point::ZERO);
    }
    let warm =
        compute_layout_with_flow(&capture, 2, request(40.0)).map_err(|e| format!("warm {e:?}"))?;
    assert_eq!(warm.final_entries(), batch.final_entries());
    Ok(())
}
#[test]
fn captured_review_transparent_leading_nodes_are_consumed_with_the_admitted_window() {
    let actual: Vec<_> = [false, true]
        .into_iter()
        .flat_map(|absolute| {
            [false, true].into_iter().flat_map(move |with_float| {
                [
                    (
                        absolute,
                        with_float,
                        "f32",
                        transparent_leading_capture::<f32>(absolute, with_float),
                    ),
                    (
                        absolute,
                        with_float,
                        "f64",
                        transparent_leading_capture::<f64>(absolute, with_float),
                    ),
                ]
            })
        })
        .collect();
    assert!(actual.iter().all(|row| row.3.is_ok()), "{actual:?}");
}

#[test]
fn captured_review_later_float_borrows_earlier_ordinary_ledger_top() {
    // The first two source floats are outside this valid captured window.
    let actual = [
        source_ordered_clear_tops::<f32>(false, false),
        source_ordered_clear_tops::<f64>(false, false),
        source_ordered_clear_tops::<f32>(true, false),
        source_ordered_clear_tops::<f64>(true, false),
    ];
    assert_eq!(
        actual,
        [
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0],
            vec![0.0, 20.0, 20.0]
        ]
    );
}

fn zero_height_captured_max_geometry<S: LayoutScalar>(
    floats: &[(Float, f64, f64, Clear, f64)],
) -> (f64, Vec<Point<f64>>) {
    let paragraph = PreparedParagraph::with_strut(
        vec![
            ParagraphPart::Control {
                node: 10,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphPart::Control {
                node: 11,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        ahem_style(10.0, 10.0),
        text::ParagraphBaseDirection::LeftToRight,
        text::ParagraphStrut::AllocatedContributions,
        Vec::new(),
    );
    let mut binding = paragraph.binding(2);
    assert_eq!(binding.controls.len(), 2);
    binding.nodes = (0..floats.len())
        .map(|i| 30 + i as u32)
        .chain([10, 11])
        .collect();
    let mut tree = float_tree::<S>(&binding, 0.0, Float::Left, floats.len())
        .children(2, binding.nodes.clone())
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    let metrics =
        InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap();
    for (node, kind) in [
        (10, InlineBoundaryKind::Start),
        (11, InlineBoundaryKind::End),
    ] {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                metrics,
                crate::InlineBoundaryEdgeOf::ZERO,
            )),
        );
    }
    for (i, &(side, width, height, clear, horizontal_margin)) in floats.iter().enumerate() {
        tree = tree.style(
            30 + i as u32,
            NodeInputOf {
                display: Display::Block,
                float: side,
                clear,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::px(S::from_f64(height)),
                ),
                margin: Edges::new(
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(S::from_f64(horizontal_margin)),
                    LengthAutoOf::ZERO,
                    LengthAutoOf::px(S::from_f64(horizontal_margin)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture = LayoutFlowInput::try_new(
        &tree,
        2,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::MAX_CONTENT,
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 2, request).unwrap();
    let layout = batch.placed_text()[0].layout();
    assert!(layout.source().text().code_units().is_empty());
    assert!(layout.lines().is_empty());
    assert_eq!(layout.metrics().line_count(), 0);
    assert_eq!(layout.metrics().first_baseline(), None);
    assert_eq!(layout.metrics().last_baseline(), None);
    assert_eq!(layout.metrics().size().height, 0.0);
    let completions = layout.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(completions.len(), 1);
    let receipt = completions[0].line.finalized_line();
    assert_eq!(receipt.prepared_text(), paragraph.prepared);
    assert!(!receipt.has_in_flow_content());
    assert_eq!(receipt.feedback().advance, 0.0);
    assert_eq!(receipt.feedback().line_height, 0.0);
    let selected = receipt.selected_line();
    let terminal = selected.request();
    assert_eq!(terminal.context.end, text::LineEnd::Paragraph);
    assert!(terminal.atomics.is_empty());
    assert_eq!(terminal.controls.len(), 2);
    for control in &terminal.controls {
        assert_eq!(control.boundary.unwrap().advance(), 0.0);
        let strut = control.strut.unwrap();
        assert_eq!(strut.baseline, 0.0);
        assert_eq!(strut.line_extent, 0.0);
    }
    // These zero-size endpoint nodes use their genuine nonformatted placement
    // origin, not the nominal line's baseline or a fabricated visible rect.
    for node in [10, 11] {
        let control = output(&batch, node);
        assert_eq!(control.size, Size::ZERO);
        assert_eq!(control.location.x.to_f64(), completions[0].origin.x);
        assert_eq!(control.location.y.to_f64(), completions[0].origin.y);
    }
    let geometry = (0..floats.len())
        .map(|i| {
            let actual = output(&batch, 30 + i as u32);
            assert_eq!(actual.margin.left.to_f64(), floats[i].4);
            assert_eq!(actual.margin.right.to_f64(), floats[i].4);
            Point::new(actual.location.x.to_f64(), actual.location.y.to_f64())
        })
        .collect();
    let warm = compute_layout_with_flow(&capture, 2, request).unwrap();
    assert_eq!(warm.final_entries(), batch.final_entries());
    assert_eq!(
        warm.placed_text()[0]
            .layout()
            .nonformatted_lines()
            .cloned()
            .collect::<Vec<_>>(),
        layout.nonformatted_lines().cloned().collect::<Vec<_>>()
    );
    (output(&batch, 2).size.width.to_f64(), geometry)
}
fn zero_height_captured_max_cohort<S: LayoutScalar>(right: bool) -> f64 {
    let side = if right { Float::Right } else { Float::Left };
    let (width, positions) = zero_height_captured_max_geometry::<S>(&[
        (side, 30.0, 4.0, Clear::None, 0.0),
        (side, 40.0, 4.0, Clear::None, 0.0),
    ]);
    assert_eq!(
        positions,
        if right {
            vec![Point::new(40.0, 0.0), Point::ZERO]
        } else {
            vec![Point::ZERO, Point::new(30.0, 0.0)]
        }
    );
    width
}

#[test]
fn captured_review_zero_height_control_line_retains_the_actual_float_cohort_width() {
    // Two positive-height floats share a real4px cohort. Structural controls
    // retain a genuine zero-metric receipt, without publishing a Text line.
    let actual = [
        zero_height_captured_max_cohort::<f32>(false),
        zero_height_captured_max_cohort::<f64>(false),
        zero_height_captured_max_cohort::<f32>(true),
        zero_height_captured_max_cohort::<f64>(true),
    ];
    assert_eq!(actual, [70.0, 70.0, 70.0, 70.0]);
}

fn zero_height_float_query_cohorts<S: LayoutScalar>() {
    assert_eq!(
        zero_height_captured_max_geometry::<S>(&[
            (Float::Left, 30.0, 4.0, Clear::None, 0.0),
            (Float::Right, 40.0, 4.0, Clear::None, 0.0)
        ]),
        (70.0, vec![Point::ZERO, Point::new(30.0, 0.0)])
    );
    // The first positive-height float has expired before the cleared cohort.
    assert_eq!(
        zero_height_captured_max_geometry::<S>(&[
            (Float::Left, 30.0, 4.0, Clear::None, 0.0),
            (Float::Right, 40.0, 4.0, Clear::Both, 0.0),
            (Float::Left, 10.0, 4.0, Clear::None, 0.0)
        ]),
        (
            50.0,
            vec![Point::ZERO, Point::new(10.0, 4.0), Point::new(0.0, 4.0)]
        )
    );
    // Two actual -5px horizontal margins make the first30px box contribute20.
    assert_eq!(
        zero_height_captured_max_geometry::<S>(&[
            (Float::Left, 30.0, 4.0, Clear::None, -5.0),
            (Float::Left, 40.0, 4.0, Clear::None, 0.0)
        ]),
        (60.0, vec![Point::new(-5.0, 0.0), Point::new(20.0, 0.0)])
    );
    assert_eq!(
        zero_height_captured_max_geometry::<S>(&[
            (Float::Right, 30.0, 4.0, Clear::None, -5.0),
            (Float::Right, 40.0, 4.0, Clear::None, 0.0)
        ]),
        (60.0, vec![Point::new(35.0, 0.0), Point::ZERO])
    );
}
#[test]
fn captured_review_zero_height_float_queries_keep_sides_clear_expiry_and_signed_margins() {
    zero_height_float_query_cohorts::<f32>();
    zero_height_float_query_cohorts::<f64>();
}

#[path = "conditional_cache.rs"]
mod conditional_cache;

#[path = "after_continuation.rs"]
mod after_continuation;

#[path = "both_sides_feedback.rs"]
mod both_sides_feedback;

#[test]
fn automatic_portion_retains_preceding_float_at_word_start_and_actual_fractional_frame() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = text::Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
        let mut prefix = style.clone();
        prefix.line_height = text::LineHeight::Absolute(20.5);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("XX "), prefix),
                ParagraphPart::text(11, Source::new("as\u{ad}sociate"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let mut binding = paragraph.binding(2);
        binding.nodes = vec![10, 30, 11];
        let tree = float_tree::<S>(&binding, 80.0, Float::Left, 1).style(
            30,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
        let capture = LayoutFlowInput::try_new(
            &tree,
            2,
            vec![binding.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 2, request(80.0)).unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(
            layout.source().text().code_units(),
            "XX as\u{ad}sociate".encode_utf16().collect::<Vec<_>>()
        );
        // The genuine prefix is 30px, so the 60px float cannot share its 80px
        // row. A word-only probe must retain that prior source encounter.
        assert_eq!(output(&batch, 30).location.y.to_f64(), 21.0);
        assert_eq!(layout.lines()[0].range(), text::Range::new(0, 6));
        assert_eq!(layout.lines()[0].baseline(), 13.0);
        let first = batch
            .final_inline_fragments()
            .iter()
            .find(|row| row.fragment().line_index() == 0)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(first.feedback().advance, 60.0);
        assert!(matches!(
            first
                .selected_line()
                .request()
                .alternative
                .as_ref()
                .unwrap()
                .view()
                .definition(),
            text::PreparedAlternativeDefinition::Source { .. }
        ));
        let warm = compute_layout_with_flow(&capture, 2, request(80.0)).unwrap();
        assert_eq!(warm.final_entries(), batch.final_entries());
    }
    check::<f32>();
    check::<f64>();
}
#[test]
fn automatic_initial_manual_portion_retains_float_before_word_without_replaying_it_inside() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = text::Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
        let mut prefix = style.clone();
        prefix.line_height = text::LineHeight::Absolute(20.5);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("XX "), prefix),
                ParagraphPart::text(11, Source::new("assoc\u{ad}iate"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let mut binding = paragraph.binding(2);
        binding.nodes = vec![10, 30, 11];
        let tree = float_tree::<S>(&binding, 80.0, Float::Left, 1).style(
            30,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(60.0)),
                    PreferredSizeOf::px(S::from_f64(4.0)),
                ),
                ..NodeInputOf::default()
            },
        );
        let capture = LayoutFlowInput::try_new(
            &tree,
            2,
            vec![binding.clone()],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 2, request(80.0)).unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(
            layout.source().text().code_units(),
            "XX assoc\u{ad}iate".encode_utf16().collect::<Vec<_>>()
        );
        // The full candidate's prefix30 defers the float, so the otherwise
        // empty manual portion60 fits80 and excludes Auto4. The genuine
        // Preserve SPACE still contributes to the encounter prefix even when
        // its Soft line alignment contribution hangs.
        assert_eq!(output(&batch, 30).location.y.to_f64(), 21.0);
        assert_eq!(layout.lines()[0].range(), text::Range::new(0, 3));
        assert_eq!(layout.lines()[0].baseline(), 13.0);
        let first = batch
            .final_inline_fragments()
            .iter()
            .find(|row| row.fragment().line_index() == 0)
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(first.feedback().advance, 30.0);
        assert!(first.selected_line().request().alternative.is_none());
        assert!(
            layout
                .lines()
                .iter()
                .any(|line| line.range() == text::Range::new(3, 9))
        );
        let warm = compute_layout_with_flow(&capture, 2, request(80.0)).unwrap();
        assert_eq!(warm.final_entries(), batch.final_entries());
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn automatic_internal_float_retries_changed_band_and_preserves_committed_results_on_refusal() {
    fn check<S: LayoutScalar>() {
        let mut style = ahem_style(10.0, 10.0);
        style.hyphens = text::Hyphens::Auto;
        style.locale = Some("en-US".into());
        style.hyphenate_character = text::HyphenateCharacter::String(text::Text::from("A"));
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("as\u{ad}s"), style.clone()),
                ParagraphPart::text(11, Source::new("ociate"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let mut binding = paragraph.binding(2);
        binding.nodes = vec![10, 30, 11];
        let float_input = |width, shape| NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            float_exclusion: if shape {
                FloatExclusion::Shape
            } else {
                FloatExclusion::MarginBox
            },
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            ),
            ..NodeInputOf::default()
        };
        let tree =
            float_tree::<S>(&binding, 80.0, Float::Left, 1).style(30, float_input(20.0, false));
        let mut host = CachedHost::new(tree);
        let cold = host.compute(&binding, &[]).unwrap();
        assert_eq!(output(&cold, 2).size.width.to_f64(), 80.0);
        assert_eq!(
            cold.placed_text()[0].layout().lines()[0].range(),
            text::Range::new(0, 5)
        );
        assert_eq!(output(&cold, 30).location.y, S::ZERO);
        assert_eq!(
            cold.placed_text()[0].layout().source().text().code_units(),
            "as\u{ad}sociate".encode_utf16().collect::<Vec<_>>()
        );
        cold.apply_to(&mut host).unwrap();
        host.hits.borrow_mut().clear();
        let warm = host.compute(&binding, &[]).unwrap();
        assert!(host.hits.borrow().contains(&30));
        assert_eq!(warm.final_entries(), cold.final_entries());
        host.tree
            .insert_input(30, LayoutInputOf::box_input(float_input(40.0, false)));
        let changed = host.compute(&binding, &[30]).unwrap();
        assert_eq!(
            changed.placed_text()[0].layout().lines()[0].range(),
            text::Range::new(0, 3)
        );
        let fresh = CachedHost::new(host.tree.clone())
            .compute(&binding, &[])
            .unwrap();
        assert_eq!(changed.final_entries(), fresh.final_entries());
        changed.apply_to(&mut host).unwrap();
        let old = host.published.clone().unwrap();
        let caches = host.caches.clone();
        host.tree
            .insert_input(30, LayoutInputOf::box_input(float_input(40.0, true)));
        host.fail_shape = true;
        let error = host.compute(&binding, &[30]).unwrap_err();
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::Measurement("actual float shape refused")
        ));
        assert_eq!(host.caches, caches);
        assert_eq!(
            host.published.as_ref().unwrap().final_entries(),
            old.final_entries()
        );
        assert_eq!(
            host.published.as_ref().unwrap().final_inline_fragments(),
            old.final_inline_fragments()
        );
        assert_eq!(
            host.published.as_ref().unwrap().placed_text()[0]
                .layout()
                .positions()
                .collect::<Vec<_>>(),
            old.placed_text()[0]
                .layout()
                .positions()
                .collect::<Vec<_>>()
        );
        host.fail_shape = false;
        let candidate = host.compute(&binding, &[30]).unwrap();
        host.reject_batch = true;
        assert_eq!(
            candidate.apply_to(&mut host),
            Err("reject complete replacement")
        );
        assert_eq!(host.caches, caches);
        assert_eq!(
            host.published.as_ref().unwrap().final_entries(),
            old.final_entries()
        );
    }
    check::<f32>();
    check::<f64>();
}

#[path = "line_limit_retry.rs"]
mod line_limit_retry;

#[path = "zone_cache.rs"]
mod zone_cache;

#[path = "last_tail_retry.rs"]
mod last_tail_retry;
