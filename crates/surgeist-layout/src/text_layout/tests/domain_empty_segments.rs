//! Empty structural segments, rather than empty raw members inside one run.
//! Every empty interval has genuine domain/child-Block frontiers, no authored
//! origin and no Layout node. Completion must come from native NonFormatted
//! receipts; new segment APIs are functional qualification, not historical RED.
use super::*;
use std::sync::Arc;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const ROOT: u32 = 0;
const A: u32 = 2;
const BLOCK: u32 = 3;
const C: u32 = 4;
const B: u32 = 6;

#[derive(Clone, Copy)]
enum Empty {
    Before,
    After,
    Both,
}
impl Empty {
    fn nodes(self) -> Vec<u32> {
        match self {
            Self::Before => vec![B, C],
            Self::After => vec![A, B],
            Self::Both => vec![B],
        }
    }
    fn source(self) -> &'static str {
        match self {
            Self::Before => "BC",
            Self::After => "AB",
            Self::Both => "B",
        }
    }
    fn empty_segments(self) -> &'static [usize] {
        match self {
            Self::Before => &[0],
            Self::After => &[1],
            Self::Both => &[0, 1],
        }
    }
}

fn block(size: f32) -> text::SourceProcessingScope {
    text::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: text::SourceProcessingTopology::Complete,
        trim: text::WhiteSpaceTrim::default(),
        base_style: text::Style {
            white_space: text::WhiteSpace::Preserve,
            ..ahem_style(size, f64::from(size))
        },
        used_direction: ParagraphBaseDirection::LeftToRight,
    }
}

struct Fixture {
    source: Source,
    raws: Vec<text::CapturedParagraph>,
    prepared: Vec<text::PreparedText>,
    origin_nodes: Vec<u32>,
}

fn fixture(empty: Empty) -> Fixture {
    use text::SourceProcessingEvent as E;
    let source = Source::identified(empty.source(), text::Id::from_u64(784_902), 73);
    let origin_nodes = empty.nodes();
    let authored = |node| {
        origin_nodes
            .iter()
            .position(|&candidate| candidate == node)
            .map(|origin| E::Origin {
                origin,
                affinity: text::Affinity::After,
            })
    };
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if let Some(origin) = authored(A) {
        events.push(origin);
    }
    events.extend([
        E::ExitParagraph(0),
        E::EnterScope(1),
        E::EnterParagraph(1),
        authored(B).unwrap(),
        E::ExitParagraph(1),
        E::ExitScope(1),
        E::EnterParagraph(2),
    ]);
    if let Some(origin) = authored(C) {
        events.push(origin);
    }
    events.extend([E::ExitParagraph(2), E::ExitScope(0)]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: (0..origin_nodes.len())
                .map(|i| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(i, i + 1),
                    aggregate: text::Range::new(i, i + 1),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            // At most one parent origin exists. No invented connection through B
            // is needed: the admitted ordinary Block supplies the real frontiers.
            connections: vec![],
            boundary_policies: vec![],
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scope_alignment_limits: None,
            scopes: vec![block(10.0), block(20.0)],
            paragraphs: (0..3)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            joint_flows: vec![text::SourceProcessingJointFlow {
                scope: 0,
                paragraphs: vec![0, 2],
                facts_owner: 0,
            }],
            events,
        })
        .unwrap(),
    );
    let raws: Vec<_> = processed
        .paragraphs()
        .map(|raw| raw.captured_paragraph())
        .collect();
    assert_eq!(raws.len(), 3);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let mut prepared = Vec::new();
    for member in [0, 1] {
        let analyzed = text::AnalyzedDomain::from_processed_paragraph(
            processed.paragraphs().nth(member).unwrap(),
            text::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert!(Arc::ptr_eq(analyzed.processed_source(), &processed));
        assert_eq!(
            analyzed.outputs().count(),
            1,
            "one complete output per actual domain, including metadata-only parent"
        );
        let output = crate::test_support::text::in_flat_wrapping_context(
            analyzed.outputs().next().unwrap().clone(),
            Wrap::Word,
        );
        prepared.push(
            system
                .prepare(&output, text::PreparationOptions::default())
                .unwrap(),
        );
    }
    let owner = prepared[0].captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [raws[0].clone(), raws[2].clone()]
    );
    assert_eq!(owner.facts_owner(), raws[0]);
    assert_eq!(owner.joint_flow(), raws[0].joint_flow());
    assert!(owner.joint_flow().is_some());
    assert_eq!(
        prepared[1]
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [raws[1].clone()]
    );
    assert_eq!(prepared[0].origin_views().count(), origin_nodes.len() - 1);
    Fixture {
        source,
        raws,
        prepared,
        origin_nodes,
    }
}

fn bind<S: LayoutScalar>(
    fixture: &Fixture,
) -> (PublicLayoutTreeOf<S>, Vec<TextParagraphBinding<u32>>) {
    let mut children = Vec::new();
    if fixture.origin_nodes.contains(&A) {
        children.push(A);
    }
    children.push(BLOCK);
    if fixture.origin_nodes.contains(&C) {
        children.push(C);
    }
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(ROOT, children)
        .children(BLOCK, [B])
        .style(
            ROOT,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        )
        .style(
            BLOCK,
            NodeInputOf {
                display: Display::Block,
                ..NodeInputOf::default()
            },
        );
    let bindings = fixture
        .prepared
        .iter()
        .enumerate()
        .map(|(index, prepared)| {
            let origins = prepared
                .origin_views()
                .map(|origin| {
                    assert_eq!(origin.source(), &fixture.source);
                    let range = origin.aggregate_range();
                    assert_eq!(range.end, range.start + 1);
                    let node = fixture.origin_nodes[range.start];
                    assert_eq!(node == B, index == 1);
                    let handle = origin.handle();
                    tree.insert_children(node, []);
                    tree.insert_input(
                        node,
                        LayoutInputOf::inline_text(TextParticipation::new(handle.clone())),
                    );
                    TextOriginBinding {
                        origin: handle,
                        node,
                    }
                })
                .collect::<Vec<_>>();
            TextParagraphBinding {
                inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
                prepared: prepared.clone(),
                container: if index == 0 { ROOT } else { BLOCK },
                nodes: origins.iter().map(|origin| origin.node).collect(),
                origins,
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            }
        })
        .collect();
    (tree, bindings)
}

fn packets(layout: &text::Layout) -> Vec<(text::Range, u32, f64, f64, f32)> {
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
    }
    layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            assert_eq!(run.orientation(), text::LineOrientation::Horizontal);
            assert_eq!(run.glyph_basis(), text::GlyphBasis::Unrotated);
            let origin = run.origin();
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_empty<S: LayoutScalar>(empty: Empty) {
    let fixture = fixture(empty);
    let parent = &fixture.prepared[0];
    let segments = parent.source_segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 2);
    assert_ne!(segments[0], segments[1]);
    assert!(segments[0].exit_position().unwrap() < segments[1].entry_position().unwrap());
    assert!(fixture.raws[0].exit_position() < segments[1].exit_position().unwrap());
    for segment in &segments {
        assert!(
            segment
                .entry_position()
                .unwrap()
                .same_capture(&fixture.raws[0].enter_position())
        );
        assert_eq!(segment.end_opportunity().view().boundary(), segment.end());
    }
    for &index in empty.empty_segments() {
        let segment = &segments[index];
        let cut = segment.start().view().participant_offset();
        assert_eq!(segment.end().view().participant_offset(), cut);
        let adjacent = if index == 0 {
            segments[1].start()
        } else {
            segments[0].end()
        };
        assert_eq!(
            adjacent.view().participant_offset(),
            cut,
            "distinct segment identity survives equal numeric cuts"
        );
    }
    let (tree, bindings) = bind::<S>(&fixture);
    assert_eq!(
        bindings[0].nodes,
        fixture
            .origin_nodes
            .iter()
            .copied()
            .filter(|&node| node != B)
            .collect::<Vec<_>>()
    );
    assert_eq!(bindings[1].nodes, [B]);
    let flow = LayoutFlowInput::try_new(
        &tree,
        ROOT,
        bindings,
        crate::WrappingInput {
            scopes: vec![],
            child_order: vec![],
        },
        vec![],
    )
    .unwrap();
    let batch = compute_layout_with_flow(&flow, ROOT, request(100.0)).unwrap();
    assert_eq!(
        batch.placed_text().len(),
        2,
        "publish the complete parent once and the independent child once"
    );
    let parent_output = batch
        .placed_text()
        .iter()
        .find(|output| output.container() == ROOT)
        .unwrap();
    let child_output = batch
        .placed_text()
        .iter()
        .find(|output| output.container() == BLOCK)
        .unwrap();
    assert!(
        parent_output
            .layout()
            .run_views()
            .all(|run| run.font_size() == 10.0)
    );
    assert!(
        child_output
            .layout()
            .run_views()
            .all(|run| run.font_size() == 20.0)
    );
    assert_eq!(
        parent_output.nodes(),
        fixture
            .origin_nodes
            .iter()
            .copied()
            .filter(|&node| node != B)
            .collect::<Vec<_>>()
    );
    assert_eq!(child_output.nodes(), [B]);
    for (output, prepared) in [
        (parent_output, parent),
        (child_output, &fixture.prepared[1]),
    ] {
        assert_eq!(output.layout().source(), &fixture.source);
        let actual = output.layout().analyzed_source();
        let owner = actual.captured_owner().unwrap();
        let expected = prepared.captured_owner().unwrap();
        assert_eq!(
            owner.members().collect::<Vec<_>>(),
            expected.members().collect::<Vec<_>>()
        );
        assert_eq!(owner.facts_owner(), expected.facts_owner());
        assert_eq!(owner.joint_flow(), expected.joint_flow());
    }
    let completions = parent_output
        .layout()
        .nonformatted_lines()
        .collect::<Vec<_>>();
    assert_eq!(completions.len(), empty.empty_segments().len());
    for &index in empty.empty_segments() {
        let matches = completions
            .iter()
            .filter(|completion| {
                completion
                    .line
                    .finalized_line()
                    .selected_line()
                    .request()
                    .segment
                    == segments[index]
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "one real completion for each actual empty segment"
        );
        let completion = matches[0];
        let receipt = completion.line.finalized_line();
        assert_eq!(receipt.prepared_text(), *parent);
        assert!(!receipt.has_in_flow_content());
        assert_eq!(receipt.feedback().advance, 0.0);
        // The source's DefaultFont strut is genuine nominal Ahem10 geometry;
        // NonFormatted disposition contributes zero USED line height.
        assert_eq!(receipt.feedback().line_height, 10.0);
        assert!(receipt.run_views().all(|run| run.glyphs().len() == 0));
        assert!(completion.atomics.is_empty());
        let selected = receipt.selected_line();
        assert_eq!(selected.request().start, segments[index].start());
        assert_eq!(selected.request().end, segments[index].end());
        let y = if index == 0 {
            0.0
        } else if matches!(empty, Empty::Both) {
            20.0
        } else {
            30.0
        };
        assert_eq!(completion.origin, text::Point::new(0.0, y));
    }
    let (height, child_top, parent_packets, child_packets) = match empty {
        // Size20 Ahem has ascent16; C at y20 has ascent8. No empty-line10.
        Empty::Before => (
            30.0,
            0.0,
            vec![(text::Range::new(1, 2), 37, 0.0, 28.0, 10.0)],
            vec![(text::Range::new(0, 1), 36, 0.0, 16.0, 20.0)],
        ),
        // A occupies0..10, B occupies10..30, then metadata completes at30.
        Empty::After => (
            30.0,
            10.0,
            vec![(text::Range::new(0, 1), 35, 0.0, 8.0, 10.0)],
            vec![(text::Range::new(1, 2), 36, 0.0, 26.0, 20.0)],
        ),
        Empty::Both => (
            20.0,
            0.0,
            vec![],
            vec![(text::Range::new(0, 1), 36, 0.0, 16.0, 20.0)],
        ),
    };
    assert_eq!(packets(parent_output.layout()), parent_packets);
    assert_eq!(packets(child_output.layout()), child_packets);
    assert_eq!(
        parent_output.layout().lines().len(),
        if matches!(empty, Empty::Both) { 0 } else { 1 }
    );
    assert_eq!(child_output.layout().lines().len(), 1);
    assert_eq!(child_output.layout().nonformatted_lines().count(), 0);
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == ROOT)
            .unwrap()
            .output();
        let child = entries
            .iter()
            .find(|entry| entry.node() == BLOCK)
            .unwrap()
            .output();
        assert_eq!(root.size.height, S::from_f64(height));
        assert_eq!(child.location.y, S::from_f64(child_top));
        assert_eq!(child.size.height, S::from_f64(20.0));
    }
}

#[test]
fn empty_structural_segment_before_child_completes_without_a_native_node_or_line() {
    assert_empty::<f32>(Empty::Before);
    assert_empty::<f64>(Empty::Before);
}

#[test]
fn empty_final_segment_preserves_terminal_horizon_after_the_real_child_block() {
    assert_empty::<f32>(Empty::After);
    assert_empty::<f64>(Empty::After);
}

#[test]
fn two_empty_structural_segments_keep_distinct_identities_at_the_same_numeric_cut() {
    assert_empty::<f32>(Empty::Both);
    assert_empty::<f64>(Empty::Both);
}

#[test]
fn an_empty_resumed_section_rejects_an_earlier_frontier_with_different_original_incidents() {
    let fixture = fixture(Empty::Both);
    let parent = &fixture.prepared[0];
    let segments = parent.source_segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 2);
    assert_ne!(segments[0], segments[1]);
    assert_eq!(fixture.source.text(), &text::Text::from("B"));
    assert_eq!(parent.origin_views().count(), 0);
    assert_eq!(parent.participants().len(), 0);
    let first_start = segments[0].start();
    let resumed_start = segments[1].start();
    assert_eq!(first_start.view().participant_offset(), 0);
    assert_eq!(resumed_start.view().participant_offset(), 0);
    assert_eq!(
        (
            first_start.view().source_before().get(),
            first_start.view().source_after().get()
        ),
        (0, 0)
    );
    assert_eq!(
        (
            resumed_start.view().source_before().get(),
            resumed_start.view().source_after().get()
        ),
        (1, 1)
    );
    assert_eq!(first_start.view().processed_position().get(), 0);
    assert_eq!(resumed_start.view().processed_position().get(), 1);
    for segment in &segments {
        assert_eq!(segment.prepared_text(), *parent);
        assert_eq!(segment.end().view().participant_offset(), 0);
        assert_eq!(segment.end_opportunity().view().boundary(), segment.end());
    }
    let request = |segment: &text::PreparedSourceSegment| text::LineSelectionRequest {
        inline_boxes: None,
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        participants: vec![],
        termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: vec![],
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
        context: text::LineContext {
            alignment_insets: text::LineAlignmentInsets::ZERO,
            formatted_line: text::FormattedLineContext::default(),
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            line_box_direction: text::LineBoxDirection::LeftToRight,
            orientation: text::LineOrientation::Horizontal,
            start: text::LineStart::Paragraph,
            end: text::LineEnd::Paragraph,
            available_inline: Some(100.0),
            indent: 0.0,
            alignment: text::Alignment::Left,
            tabs: text::TabContext::NoTabs,
            spacing: text::SpacingPolicy::None,
        },
    };
    let first_request = request(&segments[0]);
    let resumed_request = request(&segments[1]);
    let first = parent.select(first_request.clone()).unwrap();
    let resumed = parent.select(resumed_request.clone()).unwrap();
    assert_eq!(first.request().segment, segments[0]);
    assert_eq!(first.request().start, first_start);
    assert_eq!(resumed.request().segment, segments[1]);
    assert_eq!(resumed.request().start, resumed_start);
    // These native cuts have equal participant/atom extents, but the actual
    // child B lies between their original incidents. Change ONLY the start;
    // shared boundaries whose genuine incidents coincide remain legitimate.
    let mut wrong = resumed_request;
    wrong.start = first_start.clone();
    let error = parent.select(wrong).unwrap_err();
    assert_eq!(error.code, text::ErrorCode::InvalidTextProtocol);
    assert!(matches!(
        error.detail(),
        Some(text::ErrorDetail::InvalidTextProtocol {
            reason: text::ProtocolError::InvalidSourceSegment { segment: 1 }
        })
    ));
    assert_eq!(first.request().start, first_start);
    assert_eq!(resumed.request().start, resumed_start);
    let owner = parent.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [fixture.raws[0].clone(), fixture.raws[2].clone()]
    );
    assert_eq!(owner.facts_owner(), fixture.raws[0]);
    assert_eq!(owner.joint_flow(), fixture.raws[0].joint_flow());
    assert_eq!(parent.analyzed_source().source(), &fixture.source);
}
