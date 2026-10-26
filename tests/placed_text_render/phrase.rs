//! Real phrase occurrence ownership through Layout and retained paint.

use super::*;

const JAPANESE: &str = "今日は天気です。";

// Consumer-owned Ahem adaptation. The eight Japanese characters use the real
// A glyph's 1000-unit cell. SPACE retains the width fixture's blank 500-unit
// cell, making the distinct owner font size observable without new outlines.
pub(super) fn phrase_ahem() -> Vec<u8> {
    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
    }
    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn checksum(bytes: &[u8]) -> u32 {
        bytes.chunks(4).fold(0_u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    }
    fn record(bytes: &[u8], tag: &[u8; 4]) -> usize {
        (0..usize::from(u16_at(bytes, 4)))
            .map(|index| 12 + index * 16)
            .find(|at| &bytes[*at..*at + 4] == tag)
            .unwrap()
    }
    let mut bytes = super::width_transform::width_ahem();
    let mut mappings = vec![(0x20_u32, 3_u32), (0x3000, 1)];
    mappings.extend(JAPANESE.chars().map(|c| (u32::from(c), 35)));
    mappings.sort_unstable();
    mappings.dedup();
    let mut cmap = Vec::from(&[0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12][..]);
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(
        u32::try_from(16 + mappings.len() * 12)
            .unwrap()
            .to_be_bytes(),
    );
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in mappings {
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(scalar.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let entry = record(&bytes, b"cmap");
    bytes[entry + 4..entry + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    let offset = u32::try_from(bytes.len()).unwrap();
    bytes[entry + 8..entry + 12].copy_from_slice(&offset.to_be_bytes());
    bytes[entry + 12..entry + 16]
        .copy_from_slice(&u32::try_from(cmap.len()).unwrap().to_be_bytes());
    bytes.extend(cmap);
    let entry = record(&bytes, b"head");
    let at = usize::try_from(u32_at(&bytes, entry + 8)).unwrap();
    let length = usize::try_from(u32_at(&bytes, entry + 12)).unwrap();
    bytes[at + 8..at + 12].fill(0);
    let sum = checksum(&bytes[at..at + length]);
    bytes[entry + 4..entry + 8].copy_from_slice(&sum.to_be_bytes());
    let adjustment = 0xb1b0_afba_u32.wrapping_sub(checksum(&bytes));
    bytes[at + 8..at + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    bytes
}

struct PhraseTree {
    inputs: Vec<layout::LayoutInput>,
    non_box: layout::NodeInput,
}
impl layout::Traverse for PhraseTree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            1..self.inputs.len()
        } else {
            0..0
        }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.inputs.len() - 1 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}
impl layout::LayoutTree for PhraseTree {
    type MeasureError = Infallible;
    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn capture(
    system: &mut text::System,
    inline: bool,
) -> (
    layout::CompletedLayoutBatch<usize>,
    text::PreparedScope,
    text::PreparedVirtualSeparator,
) {
    let authored = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        locale: Some("ja".into()),
        word_space_transform: text::WordSpaceTransform::SpaceAutoPhrase,
        ..text::Style::default()
    };
    // No authored row uses the owner's font size. It must be retained from the
    // actual governing scope, not donated by the neighboring original row.
    let owner = text::Style {
        size: 20.0,
        line_height: text::LineHeight::Absolute(20.0),
        word_space_transform: text::WordSpaceTransform::Space,
        ..authored.clone()
    };
    let mut source = text::Source::identified(JAPANESE, text::Id::from_u64(6310), 1);
    let range = text::Range::new(0, source.text().len());
    source.span(range, authored.clone());
    let controls = if inline {
        vec![
            text::ParagraphControl {
                at: 0,
                kind: text::ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: range.end,
                kind: text::ParagraphControlKind::InlineBoundaryEnd,
            },
        ]
    } else {
        vec![]
    };
    let mut scopes = vec![text::SourceProcessingScope::Block {
        trim: text::WhiteSpaceTrim::default(),
        base_style: owner.clone(),
        used_direction: text::ParagraphBaseDirection::LeftToRight,
        topology: text::SourceProcessingTopology::Complete,
    }];
    let mut events = vec![
        text::SourceProcessingEvent::EnterScope(0),
        text::SourceProcessingEvent::EnterParagraph(0),
    ];
    if inline {
        scopes.push(text::SourceProcessingScope::Inline {
            trim: text::WhiteSpaceTrim::default(),
            style: Some(owner),
            boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
        });
        events.extend([
            text::SourceProcessingEvent::EnterScope(1),
            text::SourceProcessingEvent::Control {
                control: 0,
                affinity: text::Affinity::After,
            },
        ]);
    }
    events.push(text::SourceProcessingEvent::Origin {
        origin: 0,
        affinity: text::Affinity::After,
    });
    if inline {
        events.extend([
            text::SourceProcessingEvent::Control {
                control: 1,
                affinity: text::Affinity::Before,
            },
            text::SourceProcessingEvent::ExitScope(1),
        ]);
    }
    events.extend([
        text::SourceProcessingEvent::ExitParagraph(0),
        text::SourceProcessingEvent::ExitScope(0),
    ]);
    let processed = std::sync::Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls,
            boundary_policies: vec![],
            replacements: vec![],
            scopes,
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events,
        })
        .unwrap(),
    );
    let analyzed =
        text::AnalyzedSource::from_processed_paragraph(processed.paragraphs().next().unwrap())
            .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap: authored.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let scope = prepared.scopes().next().unwrap();
    let virtual_separator = prepared.virtual_separators().next().unwrap();
    assert_eq!(prepared.scopes().count(), 1);
    assert_eq!(prepared.virtual_separators().count(), 1);
    assert_eq!(virtual_separator.view().scope(), scope);
    assert_eq!(virtual_separator.view().source_position().get(), 3);
    assert_eq!(scope.view().style().size, 20.0);
    let origin = prepared.origin_views().next().unwrap().handle();
    let mut inputs = vec![layout::LayoutInput::box_input(layout::NodeInput {
        display: layout::Display::Block,
        ..layout::NodeInput::DEFAULT
    })];
    let mut bindings = vec![];
    if inline {
        for (kind, control) in [
            (
                layout::InlineBoundaryKind::Start,
                scope.view().entry_control().unwrap(),
            ),
            (
                layout::InlineBoundaryKind::End,
                scope.view().exit_control().unwrap(),
            ),
        ] {
            if kind == layout::InlineBoundaryKind::End {
                inputs.push(layout::LayoutInput::InlineText(
                    layout::TextParticipation::new(origin.clone()),
                ));
            }
            bindings.push(layout::TextControlBinding {
                occurrence: control,
                node: inputs.len(),
            });
            inputs.push(layout::LayoutInput::InlineBoundary(
                layout::InlineBoundaryInput::new(
                    kind,
                    layout::InlineMetrics::try_new(16.0, 20.0).unwrap(),
                    layout::InlineBoundaryEdge::ZERO,
                ),
            ));
        }
    } else {
        inputs.push(layout::LayoutInput::InlineText(
            layout::TextParticipation::new(origin.clone()),
        ));
    }
    let tree = PhraseTree {
        inputs,
        non_box: layout::NodeInput::non_box(),
    };
    let captured = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            nodes: (1..tree.inputs.len()).collect(),
            origins: vec![layout::TextOriginBinding {
                origin,
                node: if inline { 2 } else { 1 },
            }],
            controls: bindings,
            atomics: vec![],
            prepared,
            container: 0,
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &captured,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(200.0),
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    (batch, scope, virtual_separator)
}

#[test]
fn phrase_separator_retains_actual_scope_font_pair_and_source_free_selection() {
    for inline in [false, true] {
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        let bytes = phrase_ahem();
        system.register_fonts(bytes.clone()).unwrap();
        let (batch, scope, occurrence) = capture(&mut system, inline);
        let placed = batch.placed_text()[0].layout();
        assert_eq!(
            placed.source().text().code_units(),
            JAPANESE.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(placed.lines().len(), 1);
        let advances = placed
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), bytes);
                run.glyphs()
                    .map(|glyph| glyph.advance())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(advances, vec![10.0; 9]);
        let scoped = batch
            .final_inline_fragments()
            .iter()
            .filter(|entry| matches!(entry.owner(), layout::InlineFragmentOwner::Scope(_)))
            .collect::<Vec<_>>();
        assert_eq!(scoped.len(), 1);
        let layout::InlineFragmentOwner::Scope(binding) = scoped[0].owner() else {
            unreachable!()
        };
        assert_eq!(binding.scope, scope);
        assert_eq!(
            binding.association,
            if inline {
                layout::TextScopeAssociation::Inline { start: 1, end: 3 }
            } else {
                layout::TextScopeAssociation::ParagraphContainer(0)
            }
        );
        let fragment = scoped[0].fragment().finalized_fragment();
        assert_eq!(
            fragment.view().owner(),
            text::FinalizedFragmentOwner::VirtualSeparator(occurrence.clone())
        );
        assert_eq!(
            fragment.view().bounds(),
            Some(text::Rect::new(30.0, 0.0, 10.0, 20.0))
        );
        let mapping_line = fragment.finalized_line();
        let mappings = mapping_line
            .mapping_views()
            .filter(|mapping| mapping.separator_virtual().is_some())
            .collect::<Vec<_>>();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].separator_virtual(), Some(occurrence));
        assert_eq!(mappings[0].source_range(), text::Range::new(3, 3));
        let at = |x| {
            placed
                .positions()
                .find(|position| placed.position_geometry(position).unwrap().rect().origin.x == x)
                .unwrap()
        };
        let selection = placed.position_selection(&at(30.0), &at(40.0)).unwrap();
        assert!(!selection.is_collapsed());
        assert!(selection.source_text().is_empty());
        assert!(selection.plaintext().is_empty());
        assert!(selection.source_pieces().is_empty());
        drop(system);
        let scene = render::prepare_text_layout(
            placed,
            render::Transform::identity(),
            render::TextLayoutLimits::new(
                render::GlyphRunLimits::new(64, 4096, 8 << 20, 8 << 20),
                16,
                64,
                8,
            ),
        )
        .unwrap();
        assert!(std::ptr::eq(scene.layout(), placed));
        assert!(!scene.scene().is_empty());
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn phrase_geometry_does_not_create_an_accessible_space_character() {
    for inline in [false, true] {
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(phrase_ahem()).unwrap();
        let (batch, _, _) = capture(&mut system, inline);
        let placed = batch.placed_text()[0].layout();
        let mut state = text::Accessibility::new(60);
        let parent = 1000_u64.into();
        let source_update = placed
            .analyzed_source()
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        let values: String = source_update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect();
        assert_eq!(values, JAPANESE);
        assert!(
            source_update
                .nodes
                .iter()
                .all(|(_, node)| node.bounds().is_none())
        );
        drop(system);
        let update = placed
            .accessibility_update(&mut state, parent, parent, text::Point::new(100.0, 200.0))
            .unwrap();
        let values: String = update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect();
        assert_eq!(values, JAPANESE);
        let lengths = update
            .nodes
            .iter()
            .filter(|(_, node)| node.value().is_some())
            .flat_map(|(_, node)| node.character_lengths().iter().copied())
            .collect::<Vec<_>>();
        // AccessKit character lengths count UTF-8 bytes. All eight original
        // Japanese graphemes use three; the generated SPACE adds no descriptor.
        assert_eq!(lengths, vec![3; 8]);
    }
}
