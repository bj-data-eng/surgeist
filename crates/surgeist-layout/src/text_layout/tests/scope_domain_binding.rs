//! An atomic content domain belongs to its nominated occurrence, even at a shared anchor.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const A0: u32 = 10;
const A1: u32 = 20;
const INLINE_NODES: [u32; 3] = [30, 31, 32];

fn atomic_domain() -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    let mut base = ahem_style(10.0, 10.0);
    base.locale = Some("ja".into());
    let owner = text::Style {
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..base.clone()
    };
    let mut source = Source::new(JAPANESE);
    let whole = text::Range::new(0, JAPANESE.encode_utf16().count());
    source.span(whole, owner.clone());
    // Distinct genuine occurrences have the same anchor and real box metrics.
    for node in [A0, A1] {
        source.inline_box(text::InlineBox::new(
            text::Id::from_u64(u64::from(node)),
            text::InlineBoxKind::InFlow,
            0,
            text::Size::new(100.0, 10.0),
        ));
    }
    use text::SourceProcessingEvent as E;
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
                text::ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: whole.end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: (0..2)
                .flat_map(|atomic| {
                    [
                        text::ParagraphBoundaryDefinition::AtomicBefore(atomic),
                        text::ParagraphBoundaryDefinition::AtomicAfter(atomic),
                    ]
                })
                .map(|definition| text::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            scopes: vec![
                text::SourceProcessingScope::Block {
                    topology: text::SourceProcessingTopology::Complete,
                    trim: text::WhiteSpaceTrim::default(),
                    base_style: base.clone(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                text::SourceProcessingScope::AtomicBlock {
                    topology: text::SourceProcessingTopology::Complete,
                    trim: text::WhiteSpaceTrim::default(),
                    base_style: base,
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    atomic: 0,
                },
                text::SourceProcessingScope::Inline {
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(owner),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: (0..2)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // The domain explicitly names A0 after BOTH occurrences. Event order
            // cannot substitute "the latest atomic" (A1) for that custody fact.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Atomic {
                    atomic: 0,
                    affinity: text::Affinity::After,
                },
                E::Atomic {
                    atomic: 1,
                    affinity: text::Affinity::After,
                },
                E::EnterScope(1),
                E::EnterParagraph(1),
                E::EnterScope(2),
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitScope(2),
                E::ExitParagraph(1),
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let mut prepared: Vec<_> = processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(paragraph).unwrap();
            let analyzed =
                crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    assert_eq!(prepared.len(), 2);
    let child = prepared.pop().unwrap();
    let parent = prepared.pop().unwrap();
    assert_eq!(parent.scopes().len(), 0);
    assert_eq!(child.scopes().len(), 1);
    assert_eq!(
        child
            .virtual_separators()
            .map(|v| v.view().source_position().get())
            .collect::<Vec<_>>(),
        [3]
    );
    let atomics: Vec<_> = parent.atomic_views().map(|v| v.handle()).collect();
    assert_eq!(atomics.len(), 2);
    for (atomic, node) in atomics.iter().zip([A0, A1]) {
        assert_eq!(
            atomic.view().original_box().id(),
            text::Id::from_u64(u64::from(node))
        );
        assert_eq!(atomic.view().original_box().index(), 0);
    }
    let scope = child.scopes().next().unwrap();
    let origin = child.origin_views().next().unwrap().handle();
    let controls = child
        .control_views()
        .zip([30, 32])
        .map(|(v, node)| TextControlBinding {
            occurrence: v.handle(),
            node,
        })
        .collect();
    let bindings = vec![
        TextParagraphBinding {
            prepared: parent,
            container: 0,
            nodes: vec![A0, A1],
            origins: vec![],
            atomics: atomics
                .into_iter()
                .zip([A0, A1])
                .map(|(occurrence, node)| TextAtomicBinding { occurrence, node })
                .collect(),
            controls: vec![],
            scopes: vec![],
            line_strut: None,
        },
        TextParagraphBinding {
            prepared: child,
            container: A0,
            nodes: INLINE_NODES.to_vec(),
            origins: vec![TextOriginBinding {
                origin: origin.clone(),
                node: 31,
            }],
            atomics: vec![],
            controls,
            scopes: vec![TextScopeBinding {
                scope,
                association: TextScopeAssociation::Inline { start: 30, end: 32 },
            }],
            line_strut: None,
        },
    ];
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [A0, A1])
        .children(A0, INLINE_NODES)
        .children(A1, [])
        .children(31, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .input(
            31,
            LayoutInputOf::inline_text(TextParticipation::new(origin)),
        );
    for node in [A0, A1] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::InlineBlock,
                size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::px(10.0)),
                ..NodeInputOf::default()
            },
        );
    }
    for (node, kind) in [
        (30, InlineBoundaryKind::Start),
        (32, InlineBoundaryKind::End),
    ] {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
            )),
        );
    }
    (tree, bindings)
}

#[test]
fn earlier_same_anchor_atomic_accepts_its_own_content_domain_and_computes_phrase_geometry() {
    let (tree, bindings) = atomic_domain();
    let capture = TextLayoutInput::try_new(&tree, 0, bindings.clone()).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(500.0)).unwrap();
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 1);
    let InlineFragmentOwner::Scope(owner) = generated[0].owner() else {
        unreachable!()
    };
    assert_eq!(owner, &bindings[1].scopes[0]);
    let rect = generated[0].fragment().rect().unwrap();
    // Real Ahem advances: three 10px original units precede the generated space.
    assert_eq!(rect.origin().x, 30.0);
    assert_eq!(rect.size().width, 10.0);
    assert!(batch.placed_text().iter().any(|placed| {
        placed.layout().source().text().code_units() == JAPANESE.encode_utf16().collect::<Vec<_>>()
    }));
}

#[test]
fn later_same_anchor_atomic_rejects_the_earlier_atomics_content_domain() {
    let (tree, mut bindings) = atomic_domain();
    // Preserve the exact source capture, preparations, occurrence bindings,
    // endpoint controls and parent run. Move only the actual child container.
    let wrong_tree = tree.children(A0, []).children(A1, INLINE_NODES);
    bindings[1].container = A1;
    let Err(error) = TextLayoutInput::try_new(&wrong_tree, 0, bindings) else {
        panic!("A0's captured atomic content domain must not be accepted under A1")
    };
    assert!(
        matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
                TextBindingError::ScopeAssociation
            ))
        ),
        "{error:?}"
    );
}
