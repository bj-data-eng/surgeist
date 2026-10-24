//! A genuine inline scope continues around a nested in-flow block.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn continuation(
    enabled: bool,
    captured_boundaries: bool,
) -> (PublicLayoutTreeOf<f64>, Vec<TextParagraphBinding<u32>>) {
    let mut base = ahem_style(10.0, 10.0);
    base.locale = Some("ja".into());
    let authored = text::Style {
        word_space_transform: if enabled {
            text::WordSpaceTransform::SpaceAutoPhrase
        } else {
            text::WordSpaceTransform::None
        },
        ..base.clone()
    };
    let owner = text::Style {
        word_space_transform: if enabled {
            text::WordSpaceTransform::Space
        } else {
            text::WordSpaceTransform::None
        },
        ..base.clone()
    };
    let mut source = Source::new(format!("{JAPANESE}X{JAPANESE}"));
    let n = JAPANESE.encode_utf16().count();
    let end = 2 * n + 1;
    source.span(text::Range::new(0, end), authored);
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    let block = || text::SourceProcessingScope::Block {
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: base.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
    };
    use text::SourceProcessingEvent as E;
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![origin(0, n), origin(n, n + 1), origin(n + 1, end)],
            connections: vec![],
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: end,
                    kind: ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                text::SourceProcessingScope::Inline {
                    trim: text::WhiteSpaceTrim::default(),
                    style: Some(owner),
                    boundaries: captured_boundaries
                        .then_some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // The sole genuine Start is in p0 and the sole genuine End in p2.
            // The child block creates its own p1 without closing the inline.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                occurrence(0),
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                occurrence(1),
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                occurrence(2),
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(2),
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
    let prepared: Vec<_> = processed
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
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 4, 5])
        .children(3, [6])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for (node, kind) in [(1, InlineBoundaryKind::Start), (5, InlineBoundaryKind::End)] {
        tree = tree.children(node, []).input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                kind,
                InlineMetricsOf::try_new(8.0, 10.0).unwrap(),
            )),
        );
    }
    let bindings = prepared
        .into_iter()
        .enumerate()
        .map(|(index, prepared)| {
            let (container, nodes, origin_node, control_node) = match index {
                0 => (0, vec![1, 2], 2, 1),
                1 => (3, vec![6], 6, 0),
                2 => (0, vec![4, 5], 4, 5),
                _ => unreachable!(),
            };
            let origin = prepared.origin_views().next().unwrap().handle();
            tree.insert_children(origin_node, []);
            tree.insert_input(
                origin_node,
                LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
            );
            // Published Japanese literal: 今日は | 天気です。; X has no proposal.
            let expected = if enabled && index != 1 {
                vec![if index == 0 { 3 } else { n + 4 }]
            } else {
                vec![]
            };
            assert_eq!(
                prepared
                    .virtual_separators()
                    .map(|v| v.view().source_position().get())
                    .collect::<Vec<_>>(),
                expected
            );
            let scopes = prepared
                .scopes()
                .map(|scope| TextScopeBinding {
                    scope,
                    association: TextScopeAssociation::Inline { start: 1, end: 5 },
                })
                .collect();
            let controls = prepared
                .control_views()
                .map(|view| TextControlBinding {
                    occurrence: view.handle(),
                    node: control_node,
                })
                .collect();
            TextParagraphBinding {
                prepared,
                container,
                nodes,
                origins: vec![TextOriginBinding {
                    origin,
                    node: origin_node,
                }],
                atomics: vec![],
                controls,
                scopes,
                line_strut: None,
            }
        })
        .collect();
    (tree, bindings)
}

#[test]
fn complete_inline_scope_around_real_block_without_generated_output_is_accepted() {
    let (tree, bindings) = continuation(false, true);
    let capture = TextLayoutInput::try_new(&tree, 0, bindings).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(500.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 3);
    assert!(
        batch
            .final_inline_fragments()
            .iter()
            .all(|entry| matches!(entry.owner(), InlineFragmentOwner::Node(_)))
    );
    assert_eq!(
        batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 3)
            .unwrap()
            .output()
            .size
            .height,
        10.0
    );
}

#[test]
fn complete_inline_scope_keeps_generated_output_before_and_after_real_block() {
    let (tree, bindings) = continuation(true, true);
    let capture = TextLayoutInput::try_new(&tree, 0, bindings.clone()).unwrap_or_else(|error| {
        panic!("the complete global inline pair [1,5] must support its actual paragraph projections: {error:?}")
    });
    let batch = compute_layout_with_text(&capture, 0, request(500.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 3);
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(), InlineFragmentOwner::Scope(_)))
        .collect();
    assert_eq!(generated.len(), 2);
    let mut positions = Vec::new();
    for entry in generated {
        let InlineFragmentOwner::Scope(owner) = entry.owner() else {
            unreachable!()
        };
        assert_eq!(
            owner.association,
            TextScopeAssociation::Inline { start: 1, end: 5 }
        );
        let fragment = entry.fragment();
        let view = fragment.finalized_fragment().view();
        let text::FinalizedFragmentOwner::VirtualSeparator(occurrence) = view.owner() else {
            panic!("real virtual separator authority")
        };
        let at = occurrence.view().source_position().get();
        let paragraph = if at == 3 {
            &bindings[0]
        } else {
            assert_eq!(at, 12);
            &bindings[2]
        };
        assert_eq!(occurrence.prepared_text(), paragraph.prepared);
        assert_eq!(view.origin(), None);
        assert_eq!(view.local_range(), None);
        assert_eq!(view.aggregate_range(), None);
        positions.push((at, fragment.rect().unwrap()));
    }
    positions.sort_by_key(|(at, _)| *at);
    // Each source glyph and genuine generated SPACE advances10px. The child
    // block contributes its real10px line between the two parent10px lines.
    assert_eq!(
        positions,
        vec![
            (
                3,
                ScrollRectOf::try_new(Point::new(30.0, 0.0), Size::new(10.0, 10.0)).unwrap()
            ),
            (
                12,
                ScrollRectOf::try_new(Point::new(30.0, 20.0), Size::new(10.0, 10.0)).unwrap()
            ),
        ]
    );
    assert!(
        batch
            .final_entries()
            .iter()
            .filter(|entry| [1, 5].contains(&entry.node()))
            .all(|entry| entry.output().size.width == 0.0)
    );
}

#[test]
fn missing_global_inline_boundary_capture_remains_a_refusal() {
    let (tree, bindings) = continuation(true, false);
    let Err(error) = TextLayoutInput::try_new(&tree, 0, bindings) else {
        panic!("an uncaptured global pair must not be inferred from nearby controls")
    };
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
            TextBindingError::ScopeBoundaryCapture
        ))
    ));
}
