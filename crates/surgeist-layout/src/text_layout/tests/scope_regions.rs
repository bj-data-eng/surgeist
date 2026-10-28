//! Many separate authentic owners and a dormant shared occurrence after one owner.

use super::*;
use std::sync::Arc;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";
const FONT: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn styles() -> (text::Style, text::Style, text::Style) {
    let mut plain = ahem_style(10.0, 10.0);
    plain.locale = Some("ja".into());
    let authored = text::Style {
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..plain.clone()
    };
    let owner = text::Style {
        word_space_transform: text::WordSpaceTransform::Space,
        ..plain.clone()
    };
    (plain, authored, owner)
}
fn block(style: &text::Style) -> text::SourceProcessingScope {
    text::SourceProcessingScope::Block {
        topology: text::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: style.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
    }
}
fn prepare(input: text::ContextualSourceInput) -> Vec<text::PreparedText> {
    let processed = Arc::new(text::process_contextual_source(input).unwrap());
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(FONT.to_vec()).unwrap();
    processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(
                paragraph,
                text::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let analyzed =
                crate::test_support::text::in_flat_wrapping_context(analyzed, Wrap::Word);
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect()
}
fn definitions(count: usize) -> Vec<text::SourceProcessingParagraph> {
    (0..count)
        .map(|_| text::SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        })
        .collect()
}
fn bind(
    prepared: text::PreparedText,
    nodes: Vec<u32>,
    source_node: u32,
    control_nodes: &[u32],
    tree: &mut PublicLayoutTreeOf<f64>,
) -> TextParagraphBinding<u32> {
    let origin = prepared.origin_views().next().unwrap().handle();
    tree.insert_children(source_node, []);
    tree.insert_input(
        source_node,
        LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
    );
    let controls = prepared
        .control_views()
        .zip(control_nodes.iter().copied())
        .map(|(view, node)| {
            let kind = match view.kind() {
                text::ParagraphControlKind::InlineBoundaryStart => InlineBoundaryKind::Start,
                text::ParagraphControlKind::InlineBoundaryEnd => InlineBoundaryKind::End,
                _ => panic!("actual inline control"),
            };
            tree.insert_children(node, []);
            tree.insert_input(
                node,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    kind,
                    InlineMetricsOf::try_new(crate::InlineBaselineContext::Physical, 8.0, 10.0)
                        .unwrap(),
                    crate::InlineBoundaryEdgeOf::ZERO,
                )),
            );
            TextControlBinding {
                occurrence: view.handle(),
                node,
            }
        })
        .collect();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes,
        origins: vec![TextOriginBinding {
            origin,
            node: source_node,
        }],
        atomics: vec![],
        controls,
        line_strut: None,
    }
}

#[test]
fn many_disjoint_owners_keep_their_own_geometry_without_requiring_empty_gaps() {
    const COUNT: usize = 64;
    let (plain, authored, owner) = styles();
    let mut source = Source::new(JAPANESE.repeat(COUNT));
    source.span(text::Range::new(0, source.text().len()), authored);
    let n = JAPANESE.encode_utf16().count();
    let mut scopes = vec![block(&plain)];
    let mut controls = Vec::new();
    let mut origins = Vec::new();
    let mut events = vec![text::SourceProcessingEvent::EnterScope(0)];
    let mut children = Vec::new();
    use text::SourceProcessingEvent as E;
    for index in 0..COUNT {
        let scope = scopes.len();
        scopes.push(text::SourceProcessingScope::Inline {
            trim: Default::default(),
            style: Some(owner.clone()),
            boundaries: Some(text::SourceProcessingScopeBoundaries {
                start: 2 * index,
                end: 2 * index + 1,
            }),
        });
        controls.extend([
            text::ParagraphControl {
                at: index * n,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: (index + 1) * n,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ]);
        origins.push(text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: text::Range::new(index * n, (index + 1) * n),
            aggregate: text::Range::new(index * n, (index + 1) * n),
            kind: text::ParagraphOriginKind::Authored,
        });
        events.extend([
            E::EnterParagraph(2 * index),
            E::EnterScope(scope),
            E::Control {
                control: 2 * index,
                affinity: text::Affinity::After,
            },
            E::Origin {
                origin: index,
                affinity: text::Affinity::After,
            },
            E::Control {
                control: 2 * index + 1,
                affinity: text::Affinity::After,
            },
            E::ExitScope(scope),
            E::ExitParagraph(2 * index),
        ]);
        let first = 1000 + 4 * index as u32;
        children.extend([first, first + 1, first + 2]);
        if index + 1 < COUNT {
            let scope = scopes.len();
            scopes.push(block(&plain));
            events.extend([
                E::EnterScope(scope),
                E::EnterParagraph(2 * index + 1),
                E::ExitParagraph(2 * index + 1),
                E::ExitScope(scope),
            ]);
            children.push(first + 3);
        }
    }
    events.push(E::ExitScope(0));
    let all = prepare(text::ContextualSourceInput {
        computed_line_boundaries: vec![],
        source,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins,
        connections: vec![],
        controls,
        boundary_policies: vec![],
        replacements: vec![],
        scopes,
        paragraphs: definitions(2 * COUNT - 1),
        events,
    });
    let mut tree = PublicLayoutTreeOf::new().children(0, children).style(
        0,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    );
    let mut bindings = Vec::new();
    for (paragraph, prepared) in all.into_iter().enumerate() {
        if paragraph % 2 != 0 {
            continue;
        }
        let index = paragraph / 2;
        let first = 1000 + 4 * index as u32;
        bindings.push(bind(
            prepared,
            vec![first, first + 1, first + 2],
            first + 1,
            &[first, first + 2],
            &mut tree,
        ));
        if index + 1 < COUNT {
            tree = tree.children(first + 3, []).style(
                first + 3,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            );
        }
    }
    bindings.reverse();
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request(500.0)).unwrap();
    let mut generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter_map(|entry| {
            let InlineFragmentOwner::Scope(_) = entry.owner() else {
                return None;
            };
            let text::FinalizedFragmentOwner::VirtualSeparator(occurrence) =
                entry.fragment().finalized_fragment().view().owner()
            else {
                panic!("virtual occurrence");
            };
            Some((
                occurrence.view().source_position().get(),
                entry.fragment().rect().unwrap(),
            ))
        })
        .collect();
    generated.sort_by_key(|(at, _)| *at);
    assert_eq!(generated.len(), COUNT);
    for (index, (at, rect)) in generated.iter().enumerate() {
        assert_eq!(*at, index * n + 3);
        assert_eq!(rect.origin(), Point::new(30.0, 10.0 * index as f64));
        assert_eq!(rect.size(), Size::new(10.0, 10.0));
    }
    assert_eq!(batch.placed_text().len(), COUNT);
}

#[test]
fn local_emitter_does_not_require_unique_paths_for_dormant_same_domain_text() {
    let (plain, authored, owner) = styles();
    let n = JAPANESE.encode_utf16().count();
    let mut source = Source::new(format!("{JAPANESE}XZ"));
    source.span(text::Range::new(0, n), authored);
    source.span(text::Range::new(n, n + 2), plain.clone());
    let origins = [(0, n), (n, n + 1), (n + 1, n + 2)]
        .into_iter()
        .map(|(start, end)| text::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: text::Range::new(start, end),
            aggregate: text::Range::new(start, end),
            kind: text::ParagraphOriginKind::Authored,
        })
        .collect();
    use text::SourceProcessingEvent as E;
    let all = prepare(text::ContextualSourceInput {
        computed_line_boundaries: vec![],
        source,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins,
        connections: vec![],
        controls: vec![
            text::ParagraphControl {
                at: 0,
                kind: ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: n,
                kind: ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![
            block(&plain),
            text::SourceProcessingScope::Inline {
                trim: Default::default(),
                style: Some(owner),
                boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
            },
            block(&plain),
            block(&plain),
            block(&plain),
        ],
        paragraphs: definitions(6),
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::EnterScope(1),
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
            E::ExitScope(1),
            E::ExitParagraph(0),
            E::EnterScope(2),
            E::EnterParagraph(1),
            E::ExitParagraph(1),
            E::ExitScope(2),
            E::EnterParagraph(2),
            E::Origin {
                origin: 1,
                affinity: text::Affinity::After,
            },
            E::ExitParagraph(2),
            E::EnterScope(3),
            E::EnterParagraph(3),
            E::ExitParagraph(3),
            E::ExitScope(3),
            E::EnterScope(4),
            E::EnterParagraph(4),
            E::ExitParagraph(4),
            E::ExitScope(4),
            E::EnterParagraph(5),
            E::Origin {
                origin: 2,
                affinity: text::Affinity::After,
            },
            E::ExitParagraph(5),
            E::ExitScope(0),
        ],
    });
    assert_eq!(all[0].virtual_separators().len(), 1);
    assert_eq!(all[2].virtual_separators().len(), 0);
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [101, 300, 102, 200, 400, 201, 400, 202, 500])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    for node in [200, 201, 202] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    }
    let bindings = vec![
        bind(
            all[0].clone(),
            vec![101, 300, 102],
            300,
            &[101, 102],
            &mut tree,
        ),
        bind(all[2].clone(), vec![400], 400, &[], &mut tree),
        bind(all[5].clone(), vec![500], 500, &[], &mut tree),
    ];
    // Node400 is one dormant captured origin with two actual graph occurrences.
    // This admission proves the emitting scope before it, not either later path.
    LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
}
