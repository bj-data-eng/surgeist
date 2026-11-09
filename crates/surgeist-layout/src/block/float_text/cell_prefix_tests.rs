//! A combined original before a native float remains actual prefix content.

use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use surgeist_text as text;

fn cell_binding() -> TextParagraphBinding<u32> {
    let mut style = crate::test_support::text::ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Preserve;
    style.text_combine_upright = text::TextCombineUpright::All;
    let mut source = text::Source::new("A");
    source.span(text::Range::new(0, 1), style.clone());
    // Sideways composition uses the real horizontal Alphabetic font reference;
    // Ahem supplies its ordinary 8/-2 metrics and Alphabetic zero fallback.
    let reference = text::ScopeFontReference {
        script: text::FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: text::FontBaselineAxis::HorizontalY,
        baseline: text::FontBaseline::Alphabetic,
    };
    use text::SourceProcessingEvent as E;
    let mut root_style = style.clone();
    root_style.text_combine_upright = text::TextCombineUpright::None;
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(0, 1),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 1,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            computed_line_boundaries: Vec::new(),
            scope_alignment_limits: Some(text::ScopeAlignmentLimits::new(
                10_000, 100, 4_194_304, 16_777_216,
            )),
            scopes: vec![
                text::SourceProcessingScope::Block {
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: root_style,
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                    writing_axis: Some(text::SourceWritingAxis::Vertical),
                    horizontal_inner_direction: Some(text::ParagraphBaseDirection::LeftToRight),
                    font_reference: Some(reference),
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: Some(text::ParagraphBaseDirection::LeftToRight),
                    alignment_boundary: None,
                    font_reference: Some(reference),
                    trim: Default::default(),
                    style: Some(style),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::AllocatedContributions,
            }],
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
                E::ExitScope(0),
            ],
            joint_flows: Vec::new(),
        })
        .unwrap(),
    );
    let domain = text::AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(
        domain.outputs().next().unwrap().clone(),
        text::Wrap::Word,
    );
    assert_eq!(analyzed.combined_cells().len(), 1);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default()
                .with_orientation(text::LineOrientation::SidewaysClockwise),
        )
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(prepared.cell_views().len(), 1);
    assert!(cell.source_cell().source_pieces().any(|piece| {
        Some(piece.origin_position()) == origin.view().capture_position()
            && piece.local() == text::Range::new(0, 1)
    }));
    let controls = prepared
        .control_views()
        .zip([20, 21])
        .map(|(view, node)| TextControlBinding {
            node,
            occurrence: view.handle(),
        })
        .collect();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![20, 10, 21],
        origins: vec![TextOriginBinding { node: 10, origin }],
        controls,
        atomics: Vec::new(),
        inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    }
}

fn finalized(prepared: &text::PreparedText) -> text::FinalizedLine {
    let segment = prepared.source_segments().next().unwrap();
    prepared
        .select(text::LineSelectionRequest {
            inline_boxes: None,
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: Vec::new(),
            controls: prepared
                .control_views()
                .map(|view| text::ControlAllocation {
                    occurrence: view.handle(),
                    boundary: Some(text::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                    strut: Some(text::ControlStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                        alignment: text::LineBoxAlignment::Baseline,
                    }),
                })
                .collect(),
            boundary_context: Vec::new(),
            line_strut: None,
            line_box_struts: Vec::new(),
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: prepared.orientation(),
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(30.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap()
}

fn cell_prefix<S: LayoutScalar>() {
    let prefix_error = |error| crate::block::inline_run::text_run_error::<u32, S, ()>(0, error);
    let binding = cell_binding();
    let line = finalized(&binding.prepared);
    assert_eq!(line.feedback().advance, 10.0);
    let origin = &binding.origins[0].origin;
    let origin_seam = |local| text::TextSeam::Origin {
        origin: origin.clone(),
        local: text::SourcePosition::try_new(origin.view().source().text(), local).unwrap(),
    };
    let after = AcceptedFloatPrefix::Seam(origin_seam(1));
    assert_eq!(
        after.observe(&line).map_err(prefix_error).unwrap(),
        (10.0, true)
    );
    let before = AcceptedFloatPrefix::Seam(origin_seam(0));
    assert_eq!(
        before.observe(&line).map_err(prefix_error).unwrap(),
        (0.0, false)
    );

    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    for (nodes, offset, expected) in [
        (vec![20, 2, 10, 21], 1, (0.0, false)),
        (vec![20, 10, 2, 21], 2, (10.0, true)),
    ] {
        let mut binding = binding.clone();
        binding.nodes = nodes;
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    size: Size::new(
                        PreferredSizeOf::AUTO,
                        PreferredSizeOf::px(S::from_f64(30.0)),
                    ),
                    ..Default::default()
                },
            )
            .children(10, [])
            .input(
                10,
                LayoutInputOf::inline_text(TextParticipation::new(
                    binding.origins[0].origin.clone(),
                )),
            )
            .children(2, [])
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..Default::default()
                },
            );
        for (node, kind) in [
            (20, InlineBoundaryKind::Start),
            (21, InlineBoundaryKind::End),
        ] {
            tree = tree.children(node, []).input(
                node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap()
                    .with_writing_mode(WritingMode::VerticalRl),
                ),
            );
        }
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let [row] = capture.associations.runs() else {
            panic!("one actual Cell run");
        };
        let admitted = capture.associations.admitted_run(&capture.paragraphs, row);
        let request = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::splat(Some(S::from_f64(30.0))),
            ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(30.0)).unwrap(),
                    ),
                    SizingOperation::Normal,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        );
        let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
        let packet = owner
            .probe_float_child(2, request, crate::engine::FloatChildTrial::Detached)
            .unwrap();
        assert_eq!(packet.output.size, Size::splat(S::from_f64(10.0)));
        let pending = PendingFloat {
            placement: super::super::floats::PendingPlacement::Unresolved,
            node: 2,
            source_index: offset,
            side: Float::Left,
            clear: Clear::None,
            block_start: S::ZERO,
            size: packet.output.size,
            named_baselines: packet.output.named_baselines,
            content_size: packet.output.content_size,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            float_exclusion: FloatExclusion::MarginBox,
            child_compute_geometry: packet.output.scroll_geometry,
        };
        let event = FloatEvent::new(
            pending,
            packet,
            &admitted,
            &capture.associations,
            offset,
            request,
        )
        .unwrap_or_else(|_| panic!("real Cell float event"));
        assert_eq!(
            AcceptedFloatPrefix::Seam(event.seam)
                .observe(&line)
                .map_err(prefix_error)
                .unwrap(),
            expected,
            "native Cell prefix must retain its actual source extent and content"
        );
    }
}

#[test]
fn cell_original_before_float_keeps_actual_prefix_f32() {
    cell_prefix::<f32>();
}

#[test]
fn cell_original_before_float_keeps_actual_prefix_f64() {
    cell_prefix::<f64>();
}
