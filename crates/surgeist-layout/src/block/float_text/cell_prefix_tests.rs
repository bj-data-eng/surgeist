//! A combined original before a native float remains actual prefix content.

use super::*;
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use surgeist_text as text;

fn cell_binding() -> TextParagraphBinding<u32> {
    cell_binding_for("A", 1)
}

fn cell_binding_for(value: &str, original_count: usize) -> TextParagraphBinding<u32> {
    assert_eq!(value.encode_utf16().count(), original_count);
    let mut style = crate::test_support::text::ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Preserve;
    style.text_combine_upright = text::TextCombineUpright::All;
    let mut source = text::Source::new(value);
    source.span(text::Range::new(0, original_count), style.clone());
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
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Control {
            control: 0,
            affinity: text::Affinity::After,
        },
    ];
    for origin in 0..original_count {
        if origin > 0 {
            events.push(E::Connection(origin - 1));
        }
        events.push(E::Origin {
            origin,
            affinity: text::Affinity::After,
        });
    }
    events.extend([
        E::Control {
            control: 1,
            affinity: text::Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: (0..original_count)
                .map(|index| text::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: text::Range::new(index, index + 1),
                    aggregate: text::Range::new(index, index + 1),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: (0..original_count - 1)
                .map(|index| text::SourceProcessingConnection {
                    before_origin: index,
                    after_origin: index + 1,
                    kind: text::ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: original_count,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: (0..original_count - 1)
                .map(|index| text::ParagraphBoundaryPolicy {
                    definition: text::ParagraphBoundaryDefinition::Connection(index),
                    preceding_wrap: text::Wrap::Word,
                    following_wrap: text::Wrap::Word,
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect(),
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
            events,
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
    let origins: Vec<_> = prepared
        .origin_views()
        .enumerate()
        .map(|(index, view)| TextOriginBinding {
            node: 10 + u32::try_from(index).unwrap(),
            origin: view.handle(),
        })
        .collect();
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(prepared.cell_views().len(), 1);
    for (index, row) in origins.iter().enumerate() {
        assert!(cell.source_cell().source_pieces().any(|piece| {
            Some(piece.origin_position()) == row.origin.view().capture_position()
                && piece.local() == text::Range::new(index, index + 1)
        }));
    }
    let controls = prepared
        .control_views()
        .zip([20, 21])
        .map(|(view, node)| TextControlBinding {
            node,
            occurrence: view.handle(),
        })
        .collect();
    let nodes = std::iter::once(20)
        .chain(origins.iter().map(|row| row.node))
        .chain(std::iter::once(21))
        .collect();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes,
        origins,
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
    cell_prefix_for::<S>(cell_binding());
}

fn cell_prefix_for<S: LayoutScalar>(binding: TextParagraphBinding<u32>) {
    let prefix_error = |error| crate::block::inline_run::text_run_error::<u32, S, ()>(0, error);
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
    let mut before_nodes = binding.nodes.clone();
    before_nodes.insert(1, 2);
    let mut after_nodes = binding.nodes.clone();
    let after_offset = after_nodes.len() - 1;
    after_nodes.insert(after_offset, 2);
    let mut cases = vec![
        (before_nodes, 1, (0.0, false)),
        (after_nodes, after_offset, (10.0, true)),
    ];
    if binding.origins.len() > 1 {
        let mut interior_nodes = binding.nodes.clone();
        interior_nodes.insert(2, 2);
        cases.push((interior_nodes, 2, (10.0, true)));
    }
    for (nodes, offset, expected) in cases {
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
        for row in &binding.origins {
            tree = tree.children(row.node, []).input(
                row.node,
                LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
            );
        }
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
            None,
        )
        .unwrap_or_else(|_| panic!("real Cell float event"));
        assert_eq!(
            AcceptedFloatPrefix::Seam(event.seam.clone())
                .observe(&line)
                .map_err(prefix_error)
                .unwrap(),
            expected,
            "native Cell prefix must retain its actual source extent and content"
        );
        let base = FloatExclusions::new(
            axes,
            Size::splat(S::from_f64(100.0)),
            S::from_f64(30.0),
            crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
        );
        let mut encounters = Encounters::new(
            vec![event],
            base,
            S::ZERO,
            FloatInlineSpace::Finite,
            false,
            false,
        )
        .map_err(prefix_error)
        .unwrap();
        let basis = encounters
            .replay::<()>(
                0,
                &line,
                EncounterReplay {
                    boxes: None,
                    preceding: None,
                    frame: crate::text_layout::fit::CandidateFrame {
                        entry_phase: crate::text_layout::fit::EntryPhase::Consumed,
                        phase: crate::text_layout::fit::EntryPhase::Consumed,
                        offset: 0.0,
                        float_source_floor: 0.0,
                        logical_start: 0.0,
                        minimum_block: None,
                        deferred_from: None,
                    },
                    placed: 0.0,
                    defer_preceding: false,
                },
                None,
            )
            .unwrap();
        assert_eq!(basis.len(), 1);
        assert_eq!(
            (basis[0].prefix, basis[0].preceding),
            expected,
            "real replay must recognize the same original Cell seam"
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

#[test]
fn multi_original_cell_interior_float_keeps_actual_prefix_f32() {
    cell_prefix_for::<f32>(cell_binding_for("AA", 2));
}

#[test]
fn multi_original_cell_interior_float_keeps_actual_prefix_f64() {
    cell_prefix_for::<f64>(cell_binding_for("AA", 2));
}
