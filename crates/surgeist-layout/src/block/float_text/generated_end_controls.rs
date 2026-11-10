//! A real originless Wbr emission stops immediate-closing-edge reservation.
mod prefix;

use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::test_support::text::ahem_style;
use crate::*;
use std::sync::Arc;
use surgeist_text as text;

fn generated_binding() -> TextParagraphBinding<u32> {
    use text::SourceProcessingEvent as E;
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Preserve;
    style.wrap = text::Wrap::Word;
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: text::Source::new(""),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: Vec::new(),
            connections: Vec::new(),
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::Wbr {
                        style: Box::new(text::Style {
                            word_space_transform: text::WordSpaceTransform::Space,
                            ..style.clone()
                        }),
                    },
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
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 2 }),
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
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 2,
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
    let controls = prepared
        .control_views()
        .map(|view| {
            let captured = view.handle().captured_control().unwrap();
            let index = (0..3)
                .find(|&index| processed.captured_control(index).as_ref() == Some(&captured))
                .unwrap();
            TextControlBinding {
                node: [20, 24, 21][index],
                occurrence: view.handle(),
            }
        })
        .collect();
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![20, 24, 21],
        origins: Vec::new(),
        controls,
        atomics: Vec::new(),
        inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    }
}

fn generated_content_stops<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut binding = generated_binding();
    assert!(binding.origins.is_empty());
    assert!(
        binding
            .prepared
            .analyzed_source()
            .source()
            .text()
            .is_empty()
    );
    let occurrence = |node| {
        binding
            .controls
            .iter()
            .find(|row| row.node == node)
            .unwrap()
            .occurrence
            .clone()
    };
    let generated = occurrence(24);
    // The actual scope is issued from the real endpoint occurrences, rather
    // than inferred from equal zero-length source anchors.
    let start = occurrence(20).captured_control().unwrap();
    let end = occurrence(21).captured_control().unwrap();
    let scope = start.scope();
    assert_eq!(scope.exit_control(), Some(end.clone()));
    assert_eq!(
        binding.prepared.native_inline_box(start, end).unwrap(),
        text::InlineBoxSource::captured(scope.clone()),
    );
    let units = binding
        .prepared
        .unit_views()
        .filter(|unit| unit.controls().any(|control| control == generated))
        .collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("one actual Wbr-owned emitted Text unit");
    };
    assert!(unit.empty_origin().is_none());
    assert!(unit.processed_range().is_empty());
    assert_eq!(unit.origins().count(), 0);
    assert_eq!(
        unit.controls().collect::<Vec<_>>().as_slice(),
        std::slice::from_ref(&generated),
    );
    // Ahem's preserved generated SPACE is one10px em. It is a nonempty
    // canonical emission even though no original donates any processed range.
    assert_eq!(unit.advance(), 10.0);
    binding.nodes.insert(1, 2);
    assert_eq!(binding.nodes, [20, 2, 24, 21]);
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [52, 20, 2, 24, 21])
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
        .children(24, [])
        .input(
            24,
            LayoutInputOf::text_control(TextControlParticipation::new(generated.clone())),
        )
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
        (21, InlineBoundaryKind::End, 6.0),
    ] {
        tree = tree.children(node, []).input(
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
    let child = |box_, participation| GeneratedChildBoxInput {
        box_,
        participation,
    };
    let flow = |box_| child(box_, GeneratedBoxParticipation::InFlow);
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
                vec![flow(GeneratedBoxRef::Node(52)), flow(inline.clone())],
            ),
            order(
                inline,
                vec![child(
                    GeneratedBoxRef::Node(2),
                    GeneratedBoxParticipation::OutOfFlow,
                )],
            ),
        ],
        ..Default::default()
    };
    let capture = LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let width = NonNegativeFiniteOf::new(px(30.0)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    let batch = compute_layout_with_flow(&capture, 0, request).unwrap();
    let before = single_box_output(&batch, 52, Phase::Final);
    assert_eq!((before.location.y, before.size.height), (S::ZERO, px(10.0)));
    let float = single_box_output(&batch, 2, Phase::Final);
    assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
    // The emitted Wbr stops the static scan before End6:0+25 fits30 at22.
    // Its actual preserved SPACE10+End6 cannot fit the remaining5px band.
    assert_eq!(float.location.y, px(22.0));
    assert_eq!(
        single_box_output(&batch, 0, Phase::Final).size.height,
        px(52.0)
    );
    let [placed] = batch.placed_text() else {
        panic!("one generated paragraph")
    };
    assert!(placed.layout().source().text().is_empty());
    assert!(
        placed
            .layout()
            .lines()
            .iter()
            .any(|line| { line.bounds().origin.y == 42.0 && line.bounds().size.height == 10.0 })
    );
    let line = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.mapping_views()
                .any(|mapping| mapping.separator_control() == Some(generated.clone()))
        })
        .expect("the actual generated contribution retains its selected-line receipt");
    let mappings = line
        .mapping_views()
        .filter(|mapping| mapping.separator_control() == Some(generated.clone()))
        .collect::<Vec<_>>();
    let [mapping] = mappings.as_slice() else {
        panic!("one actual generated SPACE mapping")
    };
    assert_eq!(
        mapping.kind(),
        text::PresentationMappingKind::GeneratedSeparator
    );
    assert!(mapping.source_range().is_empty());
    assert!(mapping.processed_range().is_empty());
    assert!(mapping.presentation_range().end() > mapping.presentation_range().start());
    assert_eq!(line.feedback().advance, 16.0);
    assert!(line.has_in_flow_content());
}

#[test]
fn generated_wbr_stops_ancestor_end_reservation_f32() {
    generated_content_stops::<f32>();
}
#[test]
fn generated_wbr_stops_ancestor_end_reservation_f64() {
    generated_content_stops::<f64>();
}
