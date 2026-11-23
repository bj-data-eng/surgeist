//! One inline owner continues through an empty block into retained paint.

use super::*;
use std::sync::Arc;

const JAPANESE: &str = "今日は天気です。";

struct ContinuationTree {
    inputs: Vec<layout::node::LayoutInput>,
    non_box: layout::node::NodeInput,
}

impl layout::tree::Traverse for ContinuationTree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 { 1..6 } else { 0..0 }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 5 } else { 0 }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::tree::LayoutTree for ContinuationTree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::node::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: usize) -> layout::node::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

fn capture() -> (
    layout::batch::CompletedLayoutBatch<usize>,
    Vec<text::prepared::PreparedText>,
    Vec<u8>,
) {
    let authored = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        locale: Some("ja".into()),
        word_space_transform: text::style::WordSpaceTransform::SpaceAutoPhrase,
        ..text::style::Style::default()
    };
    // The generated blank uses the owner's 20px font. In phrase_ahem,
    // SPACE is half an em: its advance is 10px, while originals remain 10px.
    let owner = text::style::Style {
        size: 20.0,
        line_height: text::style::LineHeight::Absolute(20.0),
        word_space_transform: text::style::WordSpaceTransform::Space,
        ..authored.clone()
    };
    let base = text::style::Style {
        word_space_transform: text::style::WordSpaceTransform::None,
        ..authored.clone()
    };
    let mut source = text::source::Source::identified(
        format!("{JAPANESE}{JAPANESE}"),
        text::source::Id::from_u64(6311),
        1,
    );
    let n = JAPANESE.encode_utf16().count();
    let end = 2 * n;
    source.span(text::source::Range::new(0, end), authored.clone());
    let origin = |start, end| text::paragraph::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::source::Range::new(start, end),
        aggregate: text::source::Range::new(start, end),
        kind: text::paragraph::ParagraphOriginKind::Authored,
    };
    let block = || text::capture::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: text::capture::SourceProcessingTopology::Complete,
        trim: text::capture::WhiteSpaceTrim::default(),
        base_style: base.clone(),
        used_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
    };
    use text::capture::SourceProcessingEvent as E;
    let control = |control| E::Control {
        control,
        affinity: text::layout::Affinity::After,
    };
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::layout::Affinity::After,
    };
    let processed = Arc::new(
        text::capture::process_contextual_source(text::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::capture::SourceProcessingEnvelope::Complete,
            origins: vec![origin(0, n), origin(n, end)],
            connections: vec![],
            controls: vec![
                text::paragraph::ParagraphControl {
                    at: 0,
                    kind: text::paragraph::ParagraphControlKind::InlineBoundaryStart,
                },
                text::paragraph::ParagraphControl {
                    at: end,
                    kind: text::paragraph::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                text::capture::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: text::capture::WhiteSpaceTrim::default(),
                    style: Some(owner),
                    boundaries: Some(text::capture::SourceProcessingScopeBoundaries {
                        start: 0,
                        end: 1,
                    }),
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| text::capture::SourceProcessingParagraph {
                    base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // Neither paragraph-local preparation owns a counterfeit endpoint.
            // The empty child is an actual paragraph in its own plain Block domain.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                occurrence(0),
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                occurrence(1),
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let mut system = text::System::default();
    let bytes = super::phrase_transform::phrase_ahem();
    system
        .register_fonts(bytes.clone(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let prepared: Vec<_> = processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::analyzed::AnalyzedSource::from_processed_paragraph(
                paragraph,
                text::analyzed::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let policies = analyzed
                .wrapping_relations()
                .filter(|row| row.ancestor_wrap().is_none())
                .map(|row| text::analyzed::WrappingPolicy {
                    relation: row.handle(),
                    ancestor_wrap: authored.wrap,
                })
                .collect();
            let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
            system
                .prepare(&analyzed, text::prepared::PreparationOptions::default())
                .unwrap()
        })
        .collect();
    assert_eq!(prepared.len(), 3);
    assert_ne!(prepared[0], prepared[2]);
    let first = prepared[0].scopes().next().unwrap();
    let last = prepared[2].scopes().next().unwrap();
    assert_ne!(first, last);
    assert_eq!(first.prepared_text(), prepared[0]);
    assert_eq!(last.prepared_text(), prepared[2]);
    assert_eq!(first.captured_scope(), last.captured_scope());
    for (paragraph, cut) in [(0, 3), (2, 11)] {
        assert_eq!(prepared[paragraph].scopes().count(), 1);
        assert_eq!(prepared[paragraph].virtual_separators().count(), 1);
        let occurrence = prepared[paragraph].virtual_separators().next().unwrap();
        assert_eq!(occurrence.prepared_text(), prepared[paragraph]);
        assert_eq!(occurrence.view().source_position().get(), cut);
        assert_eq!(
            occurrence.view().scope(),
            prepared[paragraph].scopes().next().unwrap()
        );
    }
    let canonical = first.captured_scope();
    assert_eq!(
        prepared[0]
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [canonical.entry_control().unwrap().paragraph()]
    );
    assert_eq!(
        prepared[2]
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [canonical.exit_control().unwrap().paragraph()]
    );
    assert!(first.view().entry_control().is_some());
    assert!(first.view().exit_control().is_none());
    assert!(last.view().entry_control().is_none());
    assert!(last.view().exit_control().is_some());
    let child = prepared[1].captured_owner().unwrap();
    assert_ne!(child.domain(), canonical.domain());
    assert_eq!(child.domain().parent(), Some(canonical));
    assert_eq!(
        child.domain().domain_definition(),
        Some(text::capture::CapturedDomainDefinition::Block)
    );
    assert_eq!(prepared[1].origin_views().count(), 0);
    assert_eq!(prepared[1].control_views().count(), 0);
    assert_eq!(prepared[1].atomic_views().count(), 0);
    assert_eq!(prepared[1].participants().count(), 0);
    assert_eq!(prepared[1].scopes().count(), 0);
    assert_eq!(prepared[1].virtual_separators().count(), 0);

    let boundary = |kind| {
        layout::node::LayoutInput::InlineBoundary(
            layout::inline::InlineBoundaryInput::contribution(
                kind,
                layout::inline::InlineMetrics::try_new(
                    surgeist_layout::baseline::InlineBaselineContext::Physical,
                    16.0,
                    20.0,
                )
                .unwrap(),
                layout::inline::InlineBoundaryEdge::ZERO,
            )
            .unwrap(),
        )
    };
    let tree = ContinuationTree {
        inputs: vec![
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                ..layout::node::NodeInput::DEFAULT
            }),
            boundary(layout::inline::InlineBoundaryKind::Start),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                prepared[0].origin_views().next().unwrap().handle(),
            )),
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                prepared[2].origin_views().next().unwrap().handle(),
            )),
            boundary(layout::inline::InlineBoundaryKind::End),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let bindings = prepared
        .iter()
        .enumerate()
        .map(|(index, prepared)| {
            let (container, nodes, origin_node, control_node) = match index {
                0 => (0, vec![1, 2], 2, 1),
                1 => (3, vec![], 0, 0),
                2 => (0, vec![4, 5], 4, 5),
                _ => unreachable!(),
            };
            layout::text_layout::TextParagraphBinding {
                inline_box_fragments:
                    layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
                prepared: prepared.clone(),
                container,
                nodes,
                origins: prepared
                    .origin_views()
                    .map(|view| layout::text_layout::TextOriginBinding {
                        origin: view.handle(),
                        node: origin_node,
                    })
                    .collect(),
                controls: prepared
                    .control_views()
                    .map(|view| layout::text_layout::TextControlBinding {
                        occurrence: view.handle(),
                        node: control_node,
                    })
                    .collect(),
                atomics: vec![],
                line_strut: None,
            }
        })
        .collect();
    let captured = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        bindings,
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &captured,
        0,
        NativeViewport::new(
            layout::geometry::Size::new(Some(200.0), None),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
                layout::geometry::Direction::Ltr,
            ),
        )
        .request(layout::geometry::Size::new(
            layout::constraint::SizingOperation::Normal,
            layout::constraint::SizingOperation::Normal,
        )),
    )
    .unwrap();
    drop(system);
    (batch, prepared, bytes)
}

#[test]
fn global_phrase_owner_keeps_each_local_occurrence_through_empty_block_paint_and_copy() {
    let (batch, prepared, bytes) = capture();
    assert_eq!(batch.placed_text().len(), 2);
    assert!(
        batch
            .placed_text()
            .iter()
            .all(|entry| entry.container() == 0)
    );
    assert_eq!(batch.placed_text()[0].nodes(), [1, 2]);
    assert_eq!(batch.placed_text()[1].nodes(), [4, 5]);
    let empty_box = batch
        .node_layout(3)
        .unwrap()
        .single_box()
        .expect("fixture node produces one box")
        .final_output();
    assert_eq!(empty_box.location, layout::geometry::Point::new(0.0, 20.0));
    assert_eq!(empty_box.size, layout::geometry::Size::new(200.0, 0.0));
    assert!(
        !batch
            .final_inline_fragments()
            .iter()
            .any(|entry| entry.owner() == &layout::text_layout::InlineFragmentOwner::Node(3))
    );
    let generated: Vec<_> = batch
        .final_inline_fragments()
        .iter()
        .filter(|entry| {
            matches!(
                entry.owner(),
                layout::text_layout::InlineFragmentOwner::Scope(_)
            )
        })
        .collect();
    assert_eq!(generated.len(), 2);
    let mut positions = Vec::new();
    for entry in generated {
        let layout::text_layout::InlineFragmentOwner::Scope(owner) = entry.owner() else {
            unreachable!()
        };
        let fragment = entry.fragment();
        let finalized = fragment.finalized_fragment();
        let view = finalized.view();
        let text::finalized::FinalizedFragmentOwner::VirtualSeparator(occurrence) = view.owner()
        else {
            panic!("generated fragment retains its real virtual occurrence")
        };
        let at = occurrence.view().source_position().get();
        let (paragraph, y) = match at {
            3 => (0, 0.0),
            11 => (2, 20.0),
            _ => panic!("unexpected cut {at}"),
        };
        assert_eq!(owner.scope, prepared[paragraph].scopes().next().unwrap());
        assert_eq!(owner.scope.prepared_text(), prepared[paragraph]);
        assert_eq!(occurrence.prepared_text(), prepared[paragraph]);
        assert_eq!(occurrence.view().scope(), owner.scope);
        assert_eq!(
            owner.scope.captured_scope(),
            prepared[0].scopes().next().unwrap().captured_scope()
        );
        assert_eq!(
            owner.association,
            layout::text_layout::TextScopeAssociation::Inline { start: 1, end: 5 }
        );
        assert_eq!(view.origin(), None);
        assert_eq!(view.aggregate_range(), None);
        assert_eq!(fragment.line_index(), 0);
        let rect = fragment.rect().unwrap();
        // Three 10px original cells precede a half-em SPACE at owner size20.
        // Each parent line allocates 20px; the empty block adds zero.
        assert_eq!(rect.origin(), layout::geometry::Point::new(30.0, y));
        assert_eq!(rect.size(), layout::geometry::Size::new(10.0, 20.0));
        assert_eq!(
            view.bounds(),
            Some(text::geometry::Rect::new(30.0, 0.0, 10.0, 20.0))
        );
        let line = finalized.finalized_line();
        let mappings: Vec<_> = line
            .mapping_views()
            .filter(|mapping| mapping.separator_virtual().is_some())
            .collect();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].separator_virtual(), Some(occurrence));
        assert_eq!(mappings[0].source_range(), text::source::Range::new(at, at));
        positions.push(at);
    }
    positions.sort_unstable();
    assert_eq!(positions, [3, 11]);
    let mut copied = Vec::new();
    for (entry, top) in batch.placed_text().iter().zip([0.0, 20.0]) {
        let placed = entry.layout();
        assert_eq!(placed.lines().len(), 1);
        // The real 20px owner contributes Ahem ascent16/descent4. The empty
        // block adds no line: the second retained paint input starts at20.
        assert_eq!(placed.lines()[0].bounds().origin.y, top);
        assert_eq!(placed.lines()[0].baseline(), top + 16.0);
        assert_eq!(
            placed.source().text(),
            &text::source::Text::from(format!("{JAPANESE}{JAPANESE}"))
        );
        for run in placed.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.origin().y, top + 16.0);
        }
        let at = |x| {
            placed
                .positions()
                .find(|position| placed.position_geometry(position).unwrap().rect().origin.x == x)
                .unwrap()
        };
        let blank = placed.position_selection(&at(30.0), &at(40.0)).unwrap();
        assert!(!blank.is_collapsed());
        assert!(blank.source_text().is_empty());
        assert!(blank.plaintext().is_empty());
        assert!(blank.source_pieces().is_empty());
        // Eight actual original cells plus the one generated10px blank =90px.
        let whole = placed.position_selection(&at(0.0), &at(90.0)).unwrap();
        assert_eq!(whole.source_text(), &text::source::Text::from(JAPANESE));
        assert_eq!(whole.plaintext(), &text::source::Text::from(JAPANESE));
        copied.extend_from_slice(whole.source_text().code_units());
        // System has already been dropped by capture; Render consumes retained
        // exact finalized lines and font bytes without asking Text to rewrap.
        let scene = render::text_layout::prepare_text_layout(
            placed,
            render::geometry::Transform::identity(),
            render::text_layout::TextLayoutLimits::new(
                render::text::GlyphRunLimits::new(64, 4096, 8 << 20, 8 << 20),
                16,
                64,
                8,
            ),
            &mut surgeist_render::accounting::RenderAccountingContext::default(),
        )
        .unwrap();
        assert!(std::ptr::eq(scene.layout(), placed));
        assert!(!scene.scene().is_empty());
    }
    assert_eq!(
        copied,
        format!("{JAPANESE}{JAPANESE}")
            .encode_utf16()
            .collect::<Vec<_>>()
    );
}

#[cfg(feature = "text-accessibility")]
#[test]
fn continued_phrase_and_empty_domain_add_no_accessible_space_or_character() {
    let (batch, prepared, _) = capture();
    let parent = 1000_u64.into();
    let mut empty_state = text::accessibility::Accessibility::new(60);
    let empty = prepared[1]
        .analyzed_source()
        .accessibility_update(&mut empty_state, parent, parent)
        .unwrap();
    assert!(
        empty
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>()
            .is_empty()
    );
    assert!(
        empty
            .nodes
            .iter()
            .flat_map(|(_, node)| node.character_lengths().iter())
            .next()
            .is_none()
    );
    let mut copied = String::new();
    for entry in batch.placed_text() {
        let placed = entry.layout();
        let mut state = text::accessibility::Accessibility::new(100);
        let source = placed
            .analyzed_source()
            .accessibility_update(&mut state, parent, parent)
            .unwrap();
        assert_eq!(
            source
                .nodes
                .iter()
                .filter_map(|(_, node)| node.value())
                .collect::<String>(),
            JAPANESE
        );
        assert!(source.nodes.iter().all(|(_, node)| node.bounds().is_none()));
        let update = placed
            .accessibility_update(
                &mut state,
                parent,
                parent,
                text::geometry::Point::new(100.0, 200.0),
            )
            .unwrap();
        let values = update
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>();
        assert_eq!(values, JAPANESE);
        let lengths: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, node)| node.value().is_some())
            .flat_map(|(_, node)| node.character_lengths().iter().copied())
            .collect();
        // AccessKit counts UTF-8 bytes: eight original Japanese graphemes,
        // three bytes each. Neither the virtual SPACE nor empty child adds one.
        assert_eq!(lengths, vec![3; 8]);
        copied.push_str(&values);
    }
    assert_eq!(copied, format!("{JAPANESE}{JAPANESE}"));
}
