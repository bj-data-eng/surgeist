//! Source barriers and signed immediate closes use real captured scope ends.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;
use std::sync::Arc;
use surgeist_text as text;

#[derive(Clone, Copy)]
enum Scene {
    OpeningBarrier,
    ZeroAdvanceContent,
    NestedCloses,
}

fn binding(scene: Scene) -> TextParagraphBinding<u32> {
    use text::SourceProcessingEvent as E;
    let nested = !matches!(scene, Scene::ZeroAdvanceContent);
    let source = text::Source::new(if nested { "" } else { "AB" });
    let end = source.text().len();
    let mut style = crate::test_support::text::ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::Collapse;
    if !nested {
        // Real Ahem AB is20px with one internal tracking gap. This produces
        // emitted source content of advance0, not a transformed/deleted origin.
        style.letter_spacing = -20.0;
    }
    let mut controls = vec![
        text::ParagraphControl {
            at: 0,
            kind: text::ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: end,
            kind: text::ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    let inline_scope = |start, end| text::SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        alignment_boundary: None,
        font_reference: None,
        trim: Default::default(),
        style: Some(style.clone()),
        boundaries: Some(text::SourceProcessingScopeBoundaries { start, end }),
    };
    let mut scopes = vec![
        text::SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: text::SourceProcessingTopology::Complete,
            trim: Default::default(),
            base_style: style.clone(),
            used_direction: text::ParagraphBaseDirection::LeftToRight,
        },
        inline_scope(0, 1),
    ];
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        control(0),
    ];
    if nested {
        controls.extend([
            text::ParagraphControl {
                at: 0,
                kind: text::ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: end,
                kind: text::ParagraphControlKind::InlineBoundaryEnd,
            },
        ]);
        scopes.push(inline_scope(2, 3));
        events.extend([E::EnterScope(2), control(2)]);
    }
    events.push(E::Origin {
        origin: 0,
        affinity: text::Affinity::After,
    });
    if nested {
        events.extend([control(3), E::ExitScope(2)]);
    }
    events.extend([
        control(1),
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, end),
                aggregate: text::Range::new(0, end),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls,
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            scopes,
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events,
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
    let control_count = if nested { 4 } else { 2 };
    let controls = prepared
        .control_views()
        .map(|view| {
            let captured = view.handle().captured_control().unwrap();
            let index = (0..control_count)
                .find(|&index| processed.captured_control(index).as_ref() == Some(&captured))
                .unwrap();
            TextControlBinding {
                node: 20 + u32::try_from(index).unwrap(),
                occurrence: view.handle(),
            }
        })
        .collect();
    let mut nodes = if nested {
        vec![20, 22, 10, 23, 21]
    } else {
        vec![20, 10, 21]
    };
    nodes.insert(
        if matches!(scene, Scene::NestedCloses) {
            2
        } else {
            1
        },
        2,
    );
    TextParagraphBinding {
        prepared,
        container: 0,
        nodes,
        origins: vec![TextOriginBinding { node: 10, origin }],
        controls,
        atomics: Vec::new(),
        inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
        line_strut: None,
    }
}

fn execute<S: LayoutScalar>(
    scene: Scene,
    inner_end: (f64, f64),
    outer_padding: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let binding = binding(scene);
    let endpoint = |node| {
        binding
            .controls
            .iter()
            .find(|row| row.node == node)
            .unwrap()
            .occurrence
            .captured_control()
            .unwrap()
    };
    let outer_scope = endpoint(20).scope();
    assert_eq!(outer_scope.exit_control(), Some(endpoint(21)));
    let inner_scope = (!matches!(scene, Scene::ZeroAdvanceContent)).then(|| endpoint(22).scope());
    if let Some(inner) = &inner_scope {
        assert_eq!(inner.parent(), Some(outer_scope.clone()));
        assert_eq!(inner.exit_control(), Some(endpoint(23)));
    }
    let origin = &binding.origins[0].origin;
    if matches!(scene, Scene::ZeroAdvanceContent) {
        assert_eq!(origin.view().source().text(), &text::Text::from("AB"));
        assert!(binding.prepared.unit_views().any(|unit| {
            unit.empty_origin().is_none()
                && !unit.processed_range().is_empty()
                && unit.origins().any(|row| row.origin() == *origin)
        }));
    } else {
        assert!(origin.view().source().text().is_empty());
        assert!(
            binding
                .prepared
                .unit_views()
                .any(|unit| { unit.empty_origin().as_ref() == Some(origin) })
        );
    }
    let ordinary = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(px(30.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    };
    let mut children = vec![52];
    children.extend(binding.nodes.iter().copied());
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, children)
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
            LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
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
    for row in &binding.controls {
        let (kind, margin, padding) = match row.node {
            20 | 22 => (InlineBoundaryKind::Start, 0.0, 0.0),
            21 => (InlineBoundaryKind::End, 0.0, outer_padding),
            23 => (InlineBoundaryKind::End, inner_end.0, inner_end.1),
            _ => unreachable!("only the actual paired scope endpoints are bound"),
        };
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, px(8.0), px(10.0))
                    .unwrap(),
                InlineBoundaryEdgeOf::try_new(px(margin), S::ZERO, px(padding)).unwrap(),
                axes,
            )),
        );
    }
    let outer = GeneratedBoxRef::Inline(InlineBoxRef::Scope(outer_scope.clone()));
    let child = |box_, participation| GeneratedChildBoxInput {
        box_,
        participation,
    };
    let flow = |box_| child(box_, GeneratedBoxParticipation::InFlow);
    let float = || {
        child(
            GeneratedBoxRef::Node(2),
            GeneratedBoxParticipation::OutOfFlow,
        )
    };
    let order = |parent, children| InlineChildOrderInput {
        parent,
        children,
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let mut orders = vec![order(
        GeneratedBoxRef::Node(0),
        vec![flow(GeneratedBoxRef::Node(52)), flow(outer.clone())],
    )];
    let mut scopes = vec![outer_scope];
    if let Some(inner_scope) = inner_scope {
        let inner = GeneratedBoxRef::Inline(InlineBoxRef::Scope(inner_scope.clone()));
        let (outer_children, inner_children) = if matches!(scene, Scene::OpeningBarrier) {
            (
                vec![float(), flow(inner.clone())],
                vec![flow(GeneratedBoxRef::Node(10))],
            )
        } else {
            (
                vec![flow(inner.clone())],
                vec![float(), flow(GeneratedBoxRef::Node(10))],
            )
        };
        orders.extend([order(outer, outer_children), order(inner, inner_children)]);
        scopes.push(inner_scope);
    } else {
        orders.push(order(outer, vec![float(), flow(GeneratedBoxRef::Node(10))]));
    }
    let wrapping = WrappingInput {
        scopes: scopes
            .into_iter()
            .map(|scope| WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            })
            .collect(),
        child_order: orders,
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
    compute_layout_with_flow(&capture, 0, request).unwrap()
}

fn assert_geometry<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, top: f64, root: f64) {
    let px = S::from_f64;
    let before = single_box_output(batch, 52, Phase::Final);
    assert_eq!((before.location.y, before.size.height), (S::ZERO, px(10.0)));
    let float = single_box_output(batch, 2, Phase::Final);
    assert_eq!(float.size, Size::new(px(25.0), px(20.0)));
    assert_eq!(float.location.y, px(top));
    assert_eq!(
        single_box_output(batch, 0, Phase::Final).size.height,
        px(root)
    );
}

fn opening_stops<S: LayoutScalar>() {
    let batch = execute::<S>(Scene::OpeningBarrier, (0.0, 0.0), 6.0);
    // F sees a new opening, so End6 is not reserved:25 fits30 at source22.
    // The actual6px closing content cannot fit the remaining5 and uses42..52.
    assert_geometry(&batch, 22.0, 52.0);
    assert!(
        batch.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .any(|line| { line.bounds().origin.y == 42.0 && line.bounds().size.height == 10.0 })
    );
}

fn emitted_zero_advance_stops<S: LayoutScalar>() {
    let batch = execute::<S>(Scene::ZeroAdvanceContent, (0.0, 0.0), 6.0);
    assert_geometry(&batch, 22.0, 52.0);
    let placed = &batch.placed_text()[0];
    assert_eq!(placed.layout().source().text(), &text::Text::from("AB"));
    assert!(
        placed
            .layout()
            .lines()
            .iter()
            .any(|line| { line.bounds().origin.y == 42.0 && line.bounds().size.height == 10.0 })
    );
    let actual = batch
        .final_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| {
            line.selected_line().request().controls.iter().any(|row| {
                matches!(
                    row.occurrence.view().kind(),
                    text::ParagraphControlKind::InlineBoundaryEnd
                )
            })
        })
        .expect("the actual closing allocation retains its selected-line receipt");
    let end = actual
        .selected_line()
        .request()
        .controls
        .iter()
        .find(|row| {
            matches!(
                row.occurrence.view().kind(),
                text::ParagraphControlKind::InlineBoundaryEnd
            )
        })
        .unwrap()
        .occurrence
        .clone();
    let prefix = actual
        .prefix_at(&text::TextSeam::BeforeParticipant(
            text::PreparedParticipant::Control(end),
        ))
        .unwrap();
    assert_eq!(prefix.advance(), 0.0);
    assert!(prefix.has_in_flow_content());
}

fn nested_signed<S: LayoutScalar>() {
    // Inner End's -6 margin and +6 padding cancel numerically. Both authentic
    // ends are immediate and open at F; outer6 still makes25+0+6 nonfitting.
    let nonfit = execute::<S>(Scene::NestedCloses, (-6.0, 6.0), 6.0);
    assert_geometry(&nonfit, 32.0, 52.0);
    // Dropping only outer padding leaves signed demand0, so F stays22. Inner
    // component participation still establishes a real22..32 line, not phantom.
    let cancellation = execute::<S>(Scene::NestedCloses, (-6.0, 6.0), 0.0);
    assert_geometry(&cancellation, 22.0, 42.0);
    assert!(
        cancellation.placed_text()[0]
            .layout()
            .lines()
            .iter()
            .any(|line| { line.bounds().origin.y == 22.0 && line.bounds().size.height == 10.0 })
    );
}

#[test]
fn new_opening_stops_ancestor_end_reservation_f32() {
    opening_stops::<f32>();
}
#[test]
fn new_opening_stops_ancestor_end_reservation_f64() {
    opening_stops::<f64>();
}
#[test]
fn emitted_zero_advance_stops_ancestor_end_reservation_f32() {
    emitted_zero_advance_stops::<f32>();
}
#[test]
fn emitted_zero_advance_stops_ancestor_end_reservation_f64() {
    emitted_zero_advance_stops::<f64>();
}
#[test]
fn nested_immediate_ends_keep_signed_cancellation_and_participation_f32() {
    nested_signed::<f32>();
}
#[test]
fn nested_immediate_ends_keep_signed_cancellation_and_participation_f64() {
    nested_signed::<f64>();
}
