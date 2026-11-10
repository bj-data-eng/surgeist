//! A directional source control absorbed by a Cell remains a valid close-scan step.
use super::*;
use crate::text_layout::scope_alignment::{ScopeAlignmentResources, ScopeAlignmentSnapshot};

fn absorbed_directional_control<S: LayoutScalar>() {
    let px = S::from_f64;
    let prefix_error = |error| crate::block::inline_run::text_run_error::<u32, S, ()>(0, error);
    let mut binding = cell_binding_with_inner_override("AA", 2, true);
    binding.nodes.insert(2, 2);
    assert_eq!(binding.nodes, [20, 10, 2, 22, 11, 21]);
    let directional = binding.controls.iter().find(|row| row.node == 22).unwrap();
    assert_eq!(
        directional.occurrence.view().kind(),
        &text::ParagraphControlKind::RightToLeftOverride
    );
    assert_eq!(directional.occurrence.view().participant(), None);
    let cell = binding.prepared.cell_views().next().unwrap();
    assert_eq!(binding.prepared.cell_views().len(), 1);
    for (index, origin) in binding.origins.iter().enumerate() {
        assert!(cell.source_cell().source_pieces().any(|piece| {
            Some(piece.origin_position()) == origin.origin.view().capture_position()
                && piece.local() == text::Range::new(index, index + 1)
        }));
    }
    let scope = binding
        .controls
        .iter()
        .find(|row| row.node == 20)
        .unwrap()
        .occurrence
        .captured_control()
        .unwrap()
        .scope();
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(30.0))),
                ..Default::default()
            },
        )
        .children(2, [])
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(10.0))),
                ..Default::default()
            },
        )
        .children(22, [])
        .input(
            22,
            LayoutInputOf::text_control(TextControlParticipation::new(
                directional.occurrence.clone(),
            )),
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
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                kind,
                InlineMetricsOf::try_new(InlineBaselineContext::Physical, px(8.0), px(10.0))
                    .unwrap(),
                InlineBoundaryEdgeOf::ZERO,
                axes,
            )),
        );
    }
    let flow = |box_| GeneratedChildBoxInput {
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
    let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope.clone()));
    let wrapping = WrappingInput {
        scopes: vec![WrappingScopeInput::Captured {
            scope,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        }],
        child_order: vec![
            order(GeneratedBoxRef::Node(0), vec![flow(inline.clone())]),
            order(
                inline,
                vec![
                    flow(GeneratedBoxRef::Node(10)),
                    GeneratedChildBoxInput {
                        box_: GeneratedBoxRef::Node(2),
                        participation: GeneratedBoxParticipation::OutOfFlow,
                    },
                    flow(GeneratedBoxRef::Node(11)),
                ],
            ),
        ],
        ..Default::default()
    };
    let capture = LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let [native] = capture.associations.runs() else {
        panic!("one actual combined Cell run");
    };
    let admitted = capture
        .associations
        .admitted_run(&capture.paragraphs, native);
    let request = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(px(30.0))),
        ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(px(30.0)).unwrap()),
                SizingOperation::Normal,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Unknown,
                Size::splat(OrthogonalFallbackLimitOf::Unknown),
            ),
        ),
    );
    let mut computation = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    let packet = computation
        .probe_float_child(2, request, crate::engine::FloatChildTrial::Detached)
        .unwrap();
    assert_eq!(packet.output.size, Size::splat(px(10.0)));
    let binding = &capture.paragraphs[0];
    let resources = ScopeAlignmentResources::<S>::new(
        ScopeAlignmentSnapshot::admit(vec![], &capture.paragraphs, false)
            .unwrap()
            .freeze(),
        Default::default(),
    );
    let allocator = crate::block::inline_run::BoundaryAllocationContext {
        container: 0,
        prepared: &binding.prepared,
        associations: &capture.associations,
        resources: &resources,
        orientation: binding.prepared.orientation(),
        axes,
    };
    let controls: Vec<_> = binding
        .controls
        .iter()
        .filter(|row| row.node != 22)
        .map(|row| {
            let LayoutInputViewOf::InlineBoundary(input) = computation.layout_input(row.node)
            else {
                panic!("actual paired physical endpoint");
            };
            let boundary = allocator
                .allocate(
                    &computation,
                    row.node,
                    &row.occurrence,
                    crate::inline::InlineEndpointOf::from_input(input),
                )
                .unwrap();
            assert!(boundary.box_context().is_some());
            assert_eq!(boundary.advance(), 0.0);
            text::ControlAllocation {
                occurrence: row.occurrence.clone(),
                boundary: Some(boundary),
                strut: Some(text::ControlStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                    alignment: text::LineBoxAlignment::Baseline,
                }),
            }
        })
        .collect();
    assert_eq!(controls.len(), 2);
    let segment = binding.prepared.source_segments().next().unwrap();
    let line = binding
        .prepared
        .select(text::LineSelectionRequest {
            inline_boxes: None,
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: binding.prepared.participants().collect(),
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: Vec::new(),
            controls,
            boundary_context: Vec::new(),
            line_strut: None,
            line_box_struts: Vec::new(),
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: binding.prepared.orientation(),
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
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    let origin = &binding.origins[0].origin;
    let direct = AcceptedFloatPrefix::Seam(text::TextSeam::Origin {
        origin: origin.clone(),
        local: text::SourcePosition::try_new(origin.view().source().text(), 1).unwrap(),
    });
    assert_eq!(
        direct.observe(&line).map_err(prefix_error).unwrap(),
        (10.0, true)
    );
    let pending = PendingFloat {
        placement: super::super::super::floats::PendingPlacement::Unresolved,
        node: 2,
        source_index: 2,
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
        2,
        request,
        None,
    )
    .map_err(prefix_error)
    .unwrap();
    assert!(
        event.inside,
        "the native float is between actual originals inside the opaque Cell"
    );
    let observation = AcceptedFloatPrefix::Seam(event.seam.clone());
    assert_eq!(
        observation.observe(&line).map_err(prefix_error).unwrap(),
        (10.0, true)
    );
    let mut events = FloatEvents::new();
    events.push(event, None).map_err(prefix_error).unwrap();
    let bound = events
        .bind(
            &admitted,
            &capture.associations,
            line.selected_line().request().controls.iter(),
            |owner| {
                source_interval(
                    &admitted,
                    &capture.associations,
                    &capture.wrapping,
                    &capture.graph,
                    owner,
                    None,
                )
            },
            None,
        )
        .map_err(prefix_error)
        .expect("absorbed directional source control remains admissible during closing projection");
    assert_eq!(bound.len(), 1);
    let demand = bound[0]
        .closing_ends
        .demand(&line, &observation, None)
        .map_err(prefix_error)
        .unwrap();
    // The next original is actual Cell content, so the outer End is not an
    // immediate closing edge even though RLO itself is transparent.
    assert_eq!((demand.extra, demand.participates), (0.0, false));
}

#[test]
fn absorbed_directional_control_keeps_cell_float_projection_f32() {
    absorbed_directional_control::<f32>();
}

#[test]
fn absorbed_directional_control_keeps_cell_float_projection_f64() {
    absorbed_directional_control::<f64>();
}
