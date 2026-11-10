//! Immediate closing-box sizes participate in captured float fitting.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;
use std::sync::Arc;
use surgeist_text as text;

fn empty_binding() -> TextParagraphBinding<u32> {
    use text::SourceProcessingEvent as E;
    let source = text::Source::new("");
    let mut style = crate::test_support::text::ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 0),
                aggregate: text::Range::new(0, 0),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            scopes: vec![
                text::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: text::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: style.clone(),
                    used_direction: text::ParagraphBaseDirection::LeftToRight,
                },
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: Some(style),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
            ],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
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
            scope_alignment_limits: None,
            computed_line_boundaries: Vec::new(),
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
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

fn immediate_end<S: LayoutScalar>(end_padding: f64) {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut binding = empty_binding();
    let opening = binding.controls.iter().find(|row| row.node == 20).unwrap();
    let scope = opening.occurrence.captured_control().unwrap().scope();
    assert_eq!(scope.entry_control(), opening.occurrence.captured_control());
    let closing = binding.controls.iter().find(|row| row.node == 21).unwrap();
    assert_eq!(scope.exit_control(), closing.occurrence.captured_control());
    let origin = &binding.origins[0].origin;
    assert!(origin.view().source().text().is_empty());
    assert!(binding.prepared.participants().any(|participant| {
        matches!(participant, text::PreparedParticipant::Text(unit)
            if unit.view().empty_origin().as_ref() == Some(origin))
    }));
    binding.nodes.insert(1, 2);
    assert_eq!(binding.nodes, [20, 2, 10, 21]);
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [52, 20, 2, 10, 21])
        .style(0, ordinary.clone())
        .children(2, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(px(25.0)), PreferredSizeOf::px(px(20.0))),
                ..Default::default()
            },
        )
        .children(10, [])
        .input(
            10,
            LayoutInputOf::inline_text(TextParticipation::new(binding.origins[0].origin.clone())),
        )
        .children(20, [])
        .children(21, [])
        .children(52, [])
        .style(
            52,
            NodeInputOf {
                size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::px(px(10.0))),
                margin: Edges {
                    bottom: LengthAutoOf::px(px(12.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..ordinary
            },
        );
    for (node, kind, padding) in [
        (20, InlineBoundaryKind::Start, 0.0),
        (21, InlineBoundaryKind::End, end_padding),
    ] {
        tree = tree.input(
            node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, px(8.0), px(10.0))
                    .unwrap(),
                InlineBoundaryEdgeOf::try_new(S::ZERO, S::ZERO, px(padding)).unwrap(),
                axes,
            )),
        );
    }
    let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope.clone()));
    let in_flow = |box_| GeneratedChildBoxInput {
        box_,
        participation: GeneratedBoxParticipation::InFlow,
    };
    let order = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let wrapping = WrappingInput {
        scopes: vec![WrappingScopeInput::Captured {
            scope,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            order(
                GeneratedBoxRef::Node(0),
                vec![in_flow(GeneratedBoxRef::Node(52)), in_flow(inline.clone())],
            ),
            order(
                inline,
                vec![
                    GeneratedChildBoxInput {
                        box_: GeneratedBoxRef::Node(2),
                        participation: GeneratedBoxParticipation::OutOfFlow,
                    },
                    in_flow(GeneratedBoxRef::Node(10)),
                ],
            ),
        ],
        ..Default::default()
    };
    let capture = LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        Size::new(Some(NonNegativeFiniteOf::new(px(30.0)).unwrap()), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(30.0)).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
    let [placed] = batch.placed_text() else {
        panic!("one authentic empty-source paragraph");
    };
    assert!(placed.layout().source().text().is_empty());
    let before = single_box_output(&batch, 52, Phase::Final);
    assert_eq!((before.location.y, before.size.height), (S::ZERO, px(10.0)));
    let float = single_box_output(&batch, 2, Phase::Final);
    assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
    // Native fitting includes immediate closing padding. Source22 plus the
    // actual10px line gives the nonfitting floor32. Exact25+5=30 fits;25+6
    // does not. The literal empty origin introduces no new opening/content.
    let top = if end_padding <= 5.0 { 22.0 } else { 32.0 };
    assert_eq!(
        (
            float.location.y,
            float.location.y + float.size.height,
            single_box_output(&batch, 0, Phase::Final).size.height,
        ),
        (px(top), px(top + 20.0), px(top + 20.0)),
        "immediate closing padding {end_padding}",
    );
    if end_padding > 0.0 {
        // These placements describe the participating edge's real line, not
        // an inference from nominal metrics or the Text line-vector length.
        assert!(
            placed.layout().lines().iter().any(|line| {
                line.bounds().origin.y == 22.0 && line.bounds().size.height == 10.0
            })
        );
    } else {
        let receipts = placed.layout().nonformatted_lines().collect::<Vec<_>>();
        assert!(!receipts.is_empty());
        assert!(
            receipts
                .iter()
                .all(|row| !row.line.finalized_line().has_in_flow_content())
        );
    }
}

#[test]
fn empty_origin_closing_zero_keeps_float_at_source_f32() {
    immediate_end::<f32>(0.0);
}
#[test]
fn empty_origin_closing_zero_keeps_float_at_source_f64() {
    immediate_end::<f64>(0.0);
}
#[test]
fn empty_origin_closing_exact_fit_keeps_float_at_source_f32() {
    immediate_end::<f32>(5.0);
}
#[test]
fn empty_origin_closing_exact_fit_keeps_float_at_source_f64() {
    immediate_end::<f64>(5.0);
}
#[test]
fn empty_origin_closing_nonfit_defers_float_after_line_f32() {
    immediate_end::<f32>(6.0);
}
#[test]
fn empty_origin_closing_nonfit_defers_float_after_line_f64() {
    immediate_end::<f64>(6.0);
}
