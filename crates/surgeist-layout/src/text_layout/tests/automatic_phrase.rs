//! Layout-owned association checks for Text-issued virtual phrase separators.

use super::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const PHRASE: &str = "今日はとても良い天気です。";

struct Fixture {
    binding: TextParagraphBinding<u32>,
    tree: PublicLayoutTreeOf<f64>,
    scope: TextScopeBinding<u32>,
}

fn fixture(owner_size: f32, complete: bool, boundaries: [u32; 4]) -> Fixture {
    fixture_in_direction(
        owner_size,
        complete,
        boundaries,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn fixture_in_direction(
    owner_size: f32,
    complete: bool,
    boundaries: [u32; 4],
    direction: ParagraphBaseDirection,
) -> Fixture {
    let mut source_style = ahem_style(10.0, 20.0);
    source_style.locale = Some("ja".into());
    source_style.word_space_transform = text::WordSpaceTransform::SpaceAutoPhrase;
    let mut owner_style = source_style.clone();
    owner_style.size = owner_size;
    let mut base = source_style.clone();
    base.word_space_transform = text::WordSpaceTransform::None;
    let mut source = Source::new(PHRASE);
    let whole = text::Range::new(0, source.text().len());
    source.span(whole, source_style);
    let control = |kind, at| text::ParagraphControl { kind, at };
    let inline = |style, start, end| text::SourceProcessingScope::Inline {
        trim: text::WhiteSpaceTrim::default(),
        style: Some(style),
        boundaries: complete.then_some(text::SourceProcessingScopeBoundaries { start, end }),
    };
    use text::SourceProcessingEvent as E;
    let event_control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: whole,
                aggregate: whole,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![
                control(text::ParagraphControlKind::InlineBoundaryStart, 0),
                control(text::ParagraphControlKind::InlineBoundaryStart, 0),
                control(text::ParagraphControlKind::InlineBoundaryEnd, whole.end),
                control(text::ParagraphControlKind::InlineBoundaryEnd, whole.end),
            ],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    topology: text::SourceProcessingTopology::Complete,
                    trim: text::WhiteSpaceTrim::default(),
                    base_style: base.clone(),
                    used_direction: direction,
                },
                inline(base, 0, 3),
                inline(owner_style, 1, 2),
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: direction,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                event_control(0),
                E::EnterScope(2),
                event_control(1),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                event_control(2),
                E::ExitScope(2),
                event_control(3),
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed =
        text::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
            .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    // The published upstream sentence has exactly three internal phrase cuts.
    assert_eq!(
        prepared
            .virtual_separators()
            .map(|v| v.view().source_position().get())
            .collect::<Vec<_>>(),
        [3, 6, 8]
    );
    assert_eq!(prepared.scopes().len(), 1);
    let origin = prepared.origin_views().next().unwrap().handle();
    let controls: Vec<_> = prepared
        .control_views()
        .zip(boundaries)
        .map(|(view, node)| TextControlBinding {
            occurrence: view.handle(),
            node,
        })
        .collect();
    let nodes = vec![
        boundaries[0],
        boundaries[1],
        1,
        boundaries[2],
        boundaries[3],
    ];
    let scope = prepared.scopes().next().unwrap();
    let expected_scope = TextScopeBinding {
        scope,
        association: TextScopeAssociation::Inline {
            start: boundaries[1],
            end: boundaries[2],
        },
    };
    let binding = TextParagraphBinding {
        prepared,
        container: 0,
        nodes: nodes.clone(),
        origins: vec![TextOriginBinding {
            origin: origin.clone(),
            node: 1,
        }],
        atomics: vec![],
        controls,
        line_strut: None,
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, nodes)
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if direction == ParagraphBaseDirection::RightToLeft {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                },
                ..NodeInputOf::default()
            },
        )
        .children(1, [])
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(origin)),
        );
    for (index, node) in boundaries.into_iter().enumerate() {
        let kind = if index < 2 {
            InlineBoundaryKind::Start
        } else {
            InlineBoundaryKind::End
        };
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::new(kind, InlineMetricsOf::try_new(8.0, 10.0).unwrap())
                    .with_direction(if direction == ParagraphBaseDirection::RightToLeft {
                        Direction::Rtl
                    } else {
                        Direction::Ltr
                    }),
            ),
        );
    }
    Fixture {
        binding,
        tree,
        scope: expected_scope,
    }
}

fn rejects(
    tree: &PublicLayoutTreeOf<f64>,
    binding: TextParagraphBinding<u32>,
    expected: TextBindingError,
) {
    let Err(error) = TextLayoutInput::try_new(tree, 0, vec![binding]) else {
        panic!("invalid scope association must refuse")
    };
    assert!(
        matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(actual)) if *actual == expected),
        "{error:?}"
    );
}

#[test]
fn automatic_phrase_scope_inventory_requires_the_exact_issued_scope_and_own_pair() {
    let f = fixture(20.0, true, [10, 11, 12, 13]);
    TextLayoutInput::try_new(&f.tree, 0, vec![f.binding.clone()]).unwrap();
    let mut missing = f.binding.clone();
    missing.controls.pop();
    rejects(&f.tree, missing, TextBindingError::ControlCoverage);
    let mut duplicate = f.binding.clone();
    duplicate.controls.push(duplicate.controls[0].clone());
    rejects(&f.tree, duplicate, TextBindingError::ControlCoverage);
    let foreign = fixture(20.0, true, [10, 11, 12, 13]);
    let mut wrong = f.binding.clone();
    wrong.controls[1] = foreign.binding.controls[1].clone();
    rejects(&f.tree, wrong, TextBindingError::ControlCoverage);
    let incomplete = fixture(20.0, false, [10, 11, 12, 13]);
    rejects(
        &incomplete.tree,
        incomplete.binding,
        TextBindingError::ScopeBoundaryCapture,
    );
}

#[test]
fn automatic_phrase_equal_anchor_boundaries_cannot_swap_or_cross_the_issued_scope() {
    let f = fixture(20.0, true, [10, 11, 12, 13]);
    for order in [[11, 10, 1, 12, 13], [10, 11, 1, 13, 12]] {
        let tree = f.tree.clone().children(0, order);
        let mut wrong = f.binding.clone();
        wrong.nodes = order.to_vec();
        rejects(&tree, wrong, TextBindingError::ParticipantOrder);
    }
    let mut reused = f.binding.clone();
    reused.controls[1] = reused.controls[0].clone();
    rejects(&f.tree, reused, TextBindingError::ControlCoverage);
    let tree = f.tree.clone().input(
        11,
        LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
            InlineBoundaryKind::End,
            InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
        )),
    );
    rejects(&tree, f.binding, TextBindingError::ParticipantRole);
}

type ScopeFragmentFact = (
    TextScopeBinding<u32>,
    Option<ScrollRectOf<f64>>,
    Point<f64>,
    usize,
);

fn scope_facts(batch: &CompletedLayoutBatchOf<u32, f64>) -> Vec<ScopeFragmentFact> {
    batch
        .final_inline_fragments()
        .iter()
        .filter_map(|entry| match entry.owner() {
            InlineFragmentOwner::Scope(owner) => Some((
                owner.clone(),
                entry.fragment().rect(),
                entry.fragment().baseline(),
                entry.fragment().line_index(),
            )),
            InlineFragmentOwner::Node(_) => None,
        })
        .collect()
}

#[test]
fn automatic_phrase_multiple_occurrences_publish_one_scope_owner_at_each_placed_position() {
    let f = fixture(20.0, true, [10, 11, 12, 13]);
    let capture = TextLayoutInput::try_new(&f.tree, 0, vec![f.binding.clone()]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(500.0)).unwrap();
    let facts = scope_facts(&batch);
    assert_eq!(facts.len(), 3);
    assert!(facts.iter().all(|fact| fact.0 == f.scope && fact.3 == 0));
    assert_eq!(
        facts
            .iter()
            .map(|fact| fact.1.unwrap().origin().x)
            .collect::<Vec<_>>(),
        [30.0, 80.0, 120.0]
    );
    assert!(
        facts
            .iter()
            .all(|fact| fact.1.unwrap().size().width == 20.0)
    );
    assert_eq!(
        batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            .count(),
        3
    );
    // Boundary nodes keep only their genuine normalized control allocation.
    assert!(
        batch
            .final_entries()
            .iter()
            .filter(|entry| [10, 11, 12, 13].contains(&entry.node()))
            .all(|entry| entry.output().size.width == 0.0)
    );
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        PHRASE.encode_utf16().collect::<Vec<_>>()
    );
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Retained {
    caches: HashMap<u32, CacheOf<f64>>,
    unrounded: HashMap<u32, NodeOutputOf<f64>>,
    fragments: Vec<InlineFragmentOutputEntryOf<u32, f64>>,
}

struct CachedTree {
    tree: PublicLayoutTreeOf<f64>,
    retained: Retained,
    reject_publication: bool,
    cache_hits: Cell<usize>,
}

impl Traverse for CachedTree {
    type Node = u32;
    type Scalar = f64;
    type Children<'a> = <PublicLayoutTreeOf<f64> as Traverse>::Children<'a>;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl LayoutTree for CachedTree {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<f64> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, f64> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<f64>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<f64>> {
        let result = self
            .retained
            .caches
            .get(&node)?
            .get_with_context(input, context);
        if result.is_some() {
            self.cache_hits.set(self.cache_hits.get() + 1);
        }
        result
    }
    fn unrounded_layout(&self, node: u32) -> Option<NodeOutputOf<f64>> {
        self.retained.unrounded.get(&node).copied()
    }
}
impl LayoutBatchSink<u32, f64> for CachedTree {
    type Error = &'static str;
    type Prepared = Retained;
    fn prepare_layout_batch(
        &self,
        batch: &CompletedLayoutBatchOf<u32, f64>,
    ) -> Result<Retained, Self::Error> {
        if self.reject_publication {
            return Err("scope publication refused");
        }
        let mut next = self.retained.clone();
        for entry in batch.cache_clear_entries() {
            next.caches.remove(&entry.node());
        }
        for entry in batch.cache_store_entries() {
            next.caches
                .entry(entry.node())
                .or_default()
                .store_with_context(entry.input(), entry.context(), entry.output());
        }
        for entry in batch.unrounded_entries() {
            next.unrounded.insert(entry.node(), entry.output());
        }
        next.fragments = batch.final_inline_fragments().to_vec();
        Ok(next)
    }
    fn commit_layout_batch(&mut self, prepared: Retained) {
        self.retained = prepared;
    }
}

fn compute(
    tree: &CachedTree,
    binding: &TextParagraphBinding<u32>,
) -> CompletedLayoutBatchOf<u32, f64> {
    let capture = TextLayoutInput::try_new(tree, 100, vec![binding.clone()]).unwrap();
    compute_layout_with_text(&capture, 100, request(500.0)).unwrap()
}

fn with_cached_sibling(tree: PublicLayoutTreeOf<f64>) -> PublicLayoutTreeOf<f64> {
    tree.children(100, [0, 90])
        .children(90, [])
        .style(
            100,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            90,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::px(20.0), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::default()
            },
        )
}

#[test]
fn automatic_phrase_changed_scope_style_pair_and_preparation_replace_warm_output_atomically() {
    let original = fixture(20.0, true, [10, 11, 12, 13]);
    let mut tree = CachedTree {
        tree: with_cached_sibling(original.tree),
        retained: Retained::default(),
        reject_publication: false,
        cache_hits: Cell::new(0),
    };
    let old = compute(&tree, &original.binding);
    old.apply_to(&mut tree).unwrap();
    assert!(!tree.retained.caches.is_empty());
    let before = tree.retained.clone();
    let changed = fixture(30.0, true, [20, 21, 22, 23]);
    tree.tree = with_cached_sibling(changed.tree.clone());
    let warm = compute(&tree, &changed.binding);
    let cold_tree = CachedTree {
        tree: with_cached_sibling(changed.tree),
        retained: Retained::default(),
        reject_publication: false,
        cache_hits: Cell::new(0),
    };
    let cold = compute(&cold_tree, &changed.binding);
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(scope_facts(&warm), scope_facts(&cold));
    assert_ne!(original.scope.scope, changed.scope.scope);
    let facts = scope_facts(&warm);
    assert_eq!(
        facts
            .iter()
            .map(|fact| fact.1.unwrap().origin().x)
            .collect::<Vec<_>>(),
        [30.0, 90.0, 140.0]
    );
    assert!(
        facts
            .iter()
            .all(|fact| fact.0 == changed.scope && fact.1.unwrap().size().width == 30.0)
    );
    assert_eq!(warm.invalidated_nodes(), &[100, 0, 20, 21, 1, 22, 23]);
    assert!(tree.cache_hits.get() > 0);
    assert_eq!(cold_tree.cache_hits.get(), 0);
    tree.reject_publication = true;
    assert_eq!(warm.apply_to(&mut tree), Err("scope publication refused"));
    assert_eq!(tree.retained, before);
    tree.reject_publication = false;
    warm.apply_to(&mut tree).unwrap();
    assert!(
        tree.retained
            .fragments
            .iter()
            .filter_map(|entry| match entry.owner() {
                InlineFragmentOwner::Scope(owner) => Some(owner),
                InlineFragmentOwner::Node(_) => None,
            })
            .all(|owner| owner == &changed.scope)
    );
    let mut wrong = changed.binding;
    wrong.controls[1] = original.binding.controls[1].clone();
    rejects(&tree.tree, wrong, TextBindingError::ControlCoverage);
}

#[test]
fn automatic_phrase_scope_fragments_follow_real_split_lines_and_rtl_line_anchors() {
    let f = fixture_in_direction(
        20.0,
        true,
        [10, 11, 12, 13],
        ParagraphBaseDirection::RightToLeft,
    );
    let capture = TextLayoutInput::try_new(&f.tree, 0, vec![f.binding.clone()]).unwrap();
    let wide = compute_layout_with_text(&capture, 0, request(500.0)).unwrap();
    // Ahem advances are10px for all13 original units and20px for each of
    // three generated spaces: occupied width190, RTL start500-190=310.
    // UAX9: paragraph level1 gives strong L Japanese level2; terminal ON
    // fullstop sees L before/eos R after, so N2 resolves it R at level1.
    // L2 therefore puts that10px fullstop before the level2 Japanese run.
    // At the literal cuts3/6/8, pens are310+10+3*10=350,
    // 310+10+6*10+20=400, and310+10+8*10+2*20=440.
    assert_eq!(
        scope_facts(&wide)
            .iter()
            .map(|fact| fact.1.unwrap().origin().x)
            .collect::<Vec<_>>(),
        [350.0, 400.0, 440.0]
    );
    let split = compute_layout_with_text(&capture, 0, request(70.0)).unwrap();
    let facts = scope_facts(&split);
    assert_eq!(facts.len(), 3);
    assert!(facts.iter().map(|fact| fact.3).max().unwrap() > 0);
    let lines = split.placed_text()[0].layout().lines();
    for entry in split.final_inline_fragments() {
        let InlineFragmentOwner::Scope(owner) = entry.owner() else {
            continue;
        };
        assert_eq!(owner, &f.scope);
        let fragment = entry.fragment();
        let view = fragment.finalized_fragment().view();
        let text::FinalizedFragmentOwner::VirtualSeparator(virtual_separator) = view.owner() else {
            panic!("scope output retains its exact virtual contributor")
        };
        assert_eq!(virtual_separator.view().scope(), owner.scope);
        assert_eq!(view.origin(), None);
        assert_eq!(view.aggregate_range(), None);
        let local = view.bounds().unwrap();
        let line = &lines[fragment.line_index()];
        let rect = fragment.rect().unwrap();
        let feedback = fragment.finalized_fragment().finalized_line().feedback();
        assert_eq!(rect.origin().x, line.bounds().origin.x + local.origin.x);
        assert_eq!(
            rect.origin().y,
            line.baseline() + local.origin.y - feedback.baseline
        );
        assert_eq!(rect.size(), Size::new(local.size.width, local.size.height));
    }
}

#[test]
fn empty_domain_capture_identity_and_remote_endpoint_invalidation_rebuild_warm_output() {
    let (first_tree, first) = super::scope_continuation::empty_block_capture();
    let mut tree = CachedTree {
        tree: with_cached_sibling(first_tree),
        retained: Retained::default(),
        reject_publication: false,
        cache_hits: Cell::new(0),
    };
    let compute_all = |tree: &CachedTree, paragraphs: &[TextParagraphBinding<u32>]| {
        let capture = TextLayoutInput::try_new(tree, 100, paragraphs.to_vec()).unwrap();
        compute_layout_with_text_invalidated(&capture, 100, request(500.0), &[102]).unwrap()
    };
    let initial = compute_all(&tree, &first);
    initial.apply_to(&mut tree).unwrap();
    let before = tree.retained.clone();
    let (next_tree, next) = super::scope_continuation::empty_block_capture();
    assert_ne!(
        first[1].prepared.captured_paragraph(),
        next[1].prepared.captured_paragraph()
    );
    assert!(next[1].nodes.is_empty());
    assert_eq!(next[1].prepared.participants().count(), 0);
    tree.tree = with_cached_sibling(next_tree.clone());
    let warm = compute_all(&tree, &next);
    let cold_tree = CachedTree {
        tree: with_cached_sibling(next_tree),
        retained: Retained::default(),
        reject_publication: false,
        cache_hits: Cell::new(0),
    };
    let cold = compute_all(&cold_tree, &next);
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(scope_facts(&warm), scope_facts(&cold));
    assert_eq!(warm.placed_text().len(), 2);
    assert_eq!(
        warm.placed_text()
            .iter()
            .map(|p| p.layout().lines().len())
            .sum::<usize>(),
        2
    );
    let facts = scope_facts(&warm);
    assert_eq!(facts.len(), 2);
    assert_eq!(
        facts
            .iter()
            .map(|fact| fact.1.unwrap().origin())
            .collect::<Vec<_>>(),
        [Point::new(30.0, 0.0), Point::new(30.0, 10.0)]
    );
    assert!(
        facts
            .iter()
            .all(|fact| fact.1.unwrap().size().width == 10.0)
    );
    for node in [100, 0, 101, 102, 200, 300, 301] {
        assert!(warm.invalidated_nodes().contains(&node));
    }
    assert!(tree.cache_hits.get() > 0);
    assert_eq!(cold_tree.cache_hits.get(), 0);
    tree.reject_publication = true;
    assert_eq!(warm.apply_to(&mut tree), Err("scope publication refused"));
    assert_eq!(tree.retained, before);
    tree.reject_publication = false;
    warm.apply_to(&mut tree).unwrap();
    assert_eq!(
        tree.retained
            .fragments
            .iter()
            .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
            .count(),
        2
    );
}
