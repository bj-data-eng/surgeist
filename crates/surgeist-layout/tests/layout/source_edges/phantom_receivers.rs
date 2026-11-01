//! Real atomic and individual inline-edge receiver controls.
use super::*;

#[derive(Clone, Copy)]
enum ReceiverKind {
    Atomic {
        cancel_margins: bool,
    },
    Endpoints {
        before: (f64, f64),
        after: (f64, f64),
    },
}
impl ReceiverKind {
    fn real(self) -> bool {
        match self {
            Self::Atomic { .. } => true,
            Self::Endpoints { before, after } => [before.0, before.1, after.0, after.1]
                .into_iter()
                .any(|x| x != 0.0),
        }
    }
    fn advance(self) -> f64 {
        match self {
            Self::Atomic { .. } => 0.0,
            Self::Endpoints { before, after } => before.0 + before.1 + after.0 + after.1,
        }
    }
}
struct ReceiverTree<S: layout::LayoutScalar> {
    inputs: Vec<layout::LayoutInputOf<S>>,
    non_box: layout::NodeInputOf<S>,
    children: Vec<u32>,
}
impl<S: layout::LayoutScalar> layout::Traverse for ReceiverTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        let children: &[u32] = if node == 0 { &self.children } else { &[] };
        children.iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        if node == 0 { self.children.len() } else { 0 }
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        assert_eq!(node, 0);
        self.children[index]
    }
}
impl<S: layout::LayoutScalar> layout::LayoutTree for ReceiverTree<S> {
    type MeasureError = Infallible;
    fn node_input(&self, node: u32) -> &layout::NodeInputOf<S> {
        self.inputs[node as usize].as_box().unwrap_or(&self.non_box)
    }
    fn layout_input(&self, node: u32) -> layout::LayoutInputViewOf<'_, S> {
        self.inputs[node as usize].as_view()
    }
}
fn receiver_metric() -> text::LineBoxStrut {
    // Actual host-supplied used S geometry, not an inferred descendant/font strut.
    text::LineBoxStrut {
        scope: None,
        metrics: text::LineStrut {
            baseline: 16.0,
            line_extent: 20.0,
        },
        alignment: text::LineBoxAlignment::Baseline,
    }
}
fn receiver_input(kind: ReceiverKind, direction: layout::Direction) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let mut raw = source(true, direction);
    raw.computed_line_boundaries = vec![text::ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: text::SourceBoxSide::Before,
    }];
    raw.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: text::Affinity::After,
        },
        E::EnterScope(1),
    ];
    match kind {
        ReceiverKind::Atomic { .. } => {
            raw.source.inline_box(text::InlineBox::new(
                text::Id::from_u64(659800),
                text::InlineBoxKind::InFlow,
                2,
                text::Size::new(0.0, 0.0),
            ));
            raw.events.push(E::Atomic {
                atomic: 0,
                affinity: text::Affinity::After,
            });
            for definition in [
                text::ParagraphBoundaryDefinition::AtomicBefore(0),
                text::ParagraphBoundaryDefinition::AtomicAfter(0),
            ] {
                raw.boundary_policies.push(text::ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: text::Wrap::None,
                    following_wrap: text::Wrap::None,
                    ancestor_wrap: text::Wrap::None,
                });
            }
        }
        ReceiverKind::Endpoints { .. } => {
            raw.controls = vec![
                text::ParagraphControl {
                    at: 2,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: 2,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ];
            let text::SourceProcessingScope::Inline { boundaries, .. } = &mut raw.scopes[1] else {
                unreachable!()
            };
            *boundaries = Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 });
            raw.events.extend([
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::Before,
                },
            ]);
        }
    }
    raw.events
        .extend([E::ExitScope(1), E::ExitParagraph(0), E::ExitScope(0)]);
    raw
}
fn signed_atomic_margin<S: layout::LayoutScalar>(
    direction: layout::Direction,
    cancel: bool,
) -> layout::Edges<S> {
    if !cancel {
        return layout::Edges::ZERO;
    }
    match direction {
        layout::Direction::Ltr => {
            layout::Edges::new(S::ZERO, S::from_f64(-5.0), S::ZERO, S::from_f64(5.0))
        }
        layout::Direction::Rtl => {
            layout::Edges::new(S::ZERO, S::from_f64(5.0), S::ZERO, S::from_f64(-5.0))
        }
    }
}
fn nonphantom_receiver<S: layout::LayoutScalar>(kind: ReceiverKind, direction: layout::Direction) {
    let processed =
        Arc::new(text::process_contextual_source(receiver_input(kind, direction)).unwrap());
    assert_eq!(processed.source().text(), &text::Text::from("AB"));
    assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
    let scope = processed.captured_scope(1).unwrap();
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        text::CapturedComputedLineBoundaryTarget::Scope {
            scope: scope.clone(),
            side: text::SourceBoxSide::Before,
        }
    );
    assert_eq!(cause.source_position().get(), 2);
    assert!(processed.captured_computed_line_boundary(1).is_none());
    assert_eq!(scope.parent(), processed.captured_scope(0));
    assert!(scope.exit_position() < processed.captured_paragraph(0).unwrap().exit_position());
    let prepared = normal_prepare(&processed);
    let forced = prepared
        .opportunity_views()
        .find(|o| o.kind() == text::PreparedBreakKind::Forced)
        .unwrap();
    let forced_boundary = forced.boundary();
    let pin = prepared
        .boundary_incidences(
            &prepared.source_segments().next().unwrap(),
            &forced_boundary,
        )
        .unwrap();
    assert!(!pin.is_empty());
    let px =
        |v| layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(S::from_f64(v)).unwrap());
    let mut inputs = vec![layout::LayoutInputOf::box_input(layout::NodeInputOf {
        display: layout::Display::Block,
        direction,
        size: layout::Size::new(px(40.0), layout::PreferredSizeOf::AUTO),
        ..layout::NodeInputOf::default()
    })];
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(i, view)| {
            assert_eq!(
                view.source().id(),
                Some(text::Id::from_u64(656001 + i as u64))
            );
            assert_eq!(view.aggregate_range(), text::Range::new(i, i + 1));
            inputs.push(layout::LayoutInputOf::inline_text(
                layout::TextParticipation::new(view.handle()),
            ));
            layout::TextOriginBinding {
                origin: view.handle(),
                node: i as u32 + 1,
            }
        })
        .collect();
    let mut atomics = vec![];
    let mut controls = vec![];
    match kind {
        ReceiverKind::Atomic { cancel_margins } => {
            assert_eq!(prepared.atomic_views().count(), 1);
            let view = prepared.atomic_views().next().unwrap();
            assert_eq!(view.original_box().size(), text::Size::new(0.0, 0.0));
            assert_eq!(view.capture_position(), processed.captured_atomic(0));
            inputs.push(layout::LayoutInputOf::box_input(layout::NodeInputOf {
                display: layout::Display::InlineBlock,
                direction,
                size: layout::Size::new(px(0.0), px(0.0)),
                margin: signed_atomic_margin::<S>(direction, cancel_margins).map(|value| {
                    layout::LengthAutoOf::value(layout::LengthPercentageOf::px(value).unwrap())
                }),
                ..layout::NodeInputOf::default()
            }));
            atomics.push(layout::TextAtomicBinding {
                occurrence: view.handle(),
                node: 3,
            });
        }
        ReceiverKind::Endpoints { before, after } => {
            assert_eq!(prepared.control_views().count(), 2);
            for (i, view) in prepared.control_views().enumerate() {
                let handle = view.handle();
                assert_eq!(handle.captured_control(), processed.captured_control(i));
                let (kind, edge) = if i == 0 {
                    (layout::InlineBoundaryKind::Start, before)
                } else {
                    (layout::InlineBoundaryKind::End, after)
                };
                inputs.push(layout::LayoutInputOf::inline_boundary(
                    layout::InlineBoundaryInputOf::contribution(
                        kind,
                        layout::InlineMetricsOf::try_new(
                            surgeist_layout::InlineBaselineContext::Physical,
                            S::from_f64(16.0),
                            S::from_f64(20.0),
                        )
                        .unwrap(),
                        layout::InlineBoundaryEdgeOf::try_new(
                            S::from_f64(edge.0),
                            S::from_f64(edge.1),
                        )
                        .unwrap(),
                    )
                    .with_direction(direction),
                ));
                controls.push(layout::TextControlBinding {
                    occurrence: handle,
                    node: 3 + i as u32,
                });
            }
        }
    }
    let nodes = (1..inputs.len() as u32).collect::<Vec<_>>();
    let binding = layout::TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: nodes.clone(),
        origins,
        controls,
        atomics,
        line_strut: None,
    };
    let tree = ReceiverTree {
        inputs,
        non_box: layout::NodeInputOf::non_box(),
        children: nodes,
    };
    let child = |box_| layout::GeneratedChildBoxInput {
        box_,
        participation: layout::GeneratedBoxParticipation::InFlow,
    };
    let inline = layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(scope));
    let wrapping = layout::WrappingInput {
        scopes: vec![layout::WrappingScopeInput::Captured {
            scope: processed.captured_scope(1).unwrap(),
            wrap_inside: layout::WrapInside::Auto,
            edges: layout::WrapEdges::AUTO,
            metric_contribution: Some(
                layout::InlineBoxMetricContribution::Strut(receiver_metric()),
            ),
        }],
        child_order: vec![
            layout::InlineChildOrderInput {
                parent: layout::GeneratedBoxRef::Node(0),
                children: vec![
                    child(layout::GeneratedBoxRef::Node(1)),
                    child(layout::GeneratedBoxRef::Node(2)),
                    child(inline.clone()),
                ],
                prefix: layout::ChildOrderCompleteness::Complete,
                suffix: layout::ChildOrderCompleteness::Complete,
                formatting_context: 0,
            },
            layout::InlineChildOrderInput {
                parent: inline,
                children: if matches!(kind, ReceiverKind::Atomic { .. }) {
                    vec![child(layout::GeneratedBoxRef::Node(3))]
                } else {
                    vec![]
                },
                prefix: layout::ChildOrderCompleteness::Complete,
                suffix: layout::ChildOrderCompleteness::Complete,
                formatting_context: 0,
            },
        ],
    };
    let flow =
        layout::LayoutFlowInput::try_new(&tree, 0, vec![binding], wrapping, Vec::new()).unwrap();
    let batch = layout::compute_layout_with_flow(&flow, 0, request()).unwrap();
    assert_eq!(
        prepared
            .boundary_incidences(
                &prepared.source_segments().next().unwrap(),
                &forced_boundary
            )
            .unwrap(),
        pin
    );
    let placed = batch.placed_text()[0].layout();
    assert!(placed.analyzed_source().identity() == prepared.analyzed_source().identity());
    let real = kind.real();
    assert_eq!(
        placed.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        if real {
            vec![text::Range::new(0, 2), text::Range::new(2, 2)]
        } else {
            vec![text::Range::new(0, 2)]
        }
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.bounds().size.height)
            .collect::<Vec<_>>(),
        if real { vec![10.0, 20.0] } else { vec![10.0] }
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|l| l.baseline())
            .collect::<Vec<_>>(),
        if real { vec![8.0, 26.0] } else { vec![8.0] }
    );
    let total = if real { 30.0 } else { 10.0 };
    assert_eq!(placed.metrics().line_count(), if real { 2 } else { 1 });
    assert_eq!(placed.metrics().size().height, total);
    let root = batch
        .final_entries()
        .iter()
        .find(|e| e.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        layout::Size::new(S::from_f64(40.0), S::from_f64(total))
    );
    let glyphs = placed
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| {
                    (
                        run.line(),
                        g.id(),
                        g.advance(),
                        run.origin().x + g.pen() + f64::from(g.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let x = if direction == layout::Direction::Ltr {
        0.0
    } else {
        20.0
    };
    assert_eq!(glyphs, [(0, 35, 10.0, x), (0, 36, 10.0, x + 10.0)]);
    if real {
        match kind {
            ReceiverKind::Atomic { cancel_margins } => {
                // A zero-size atomic has no source fragment exposing a
                // FinalizedLine. Use its real committed atomic observation.
                let boxes = placed.inline_boxes();
                assert_eq!(boxes.len(), 1);
                assert_eq!(boxes[0].id(), text::Id::from_u64(659800));
                assert_eq!(boxes[0].kind(), text::InlineBoxKind::InFlow);
                assert_eq!(boxes[0].index(), 2);
                assert_eq!(boxes[0].line(), 1);
                assert_eq!(boxes[0].rect().size, text::Size::new(0.0, 0.0));
                let atomic = batch
                    .final_entries()
                    .iter()
                    .find(|e| e.node() == 3)
                    .unwrap()
                    .output();
                assert_eq!(atomic.size, layout::Size::new(S::ZERO, S::ZERO));
                assert_eq!(
                    atomic.margin,
                    signed_atomic_margin::<S>(direction, cancel_margins)
                );
            }
            ReceiverKind::Endpoints { .. } => {
                let receiving = batch
                    .final_inline_fragments()
                    .iter()
                    .find(|e| e.fragment().line_index() == 1)
                    .expect("a nonzero inline edge owns its real receiving fragment");
                let line = receiving.fragment().finalized_fragment().finalized_line();
                let selection = line.selected_line();
                let request = selection.request();
                assert_eq!(request.context.end, text::LineEnd::Paragraph);
                assert_eq!(line.measurements().fit().advance(), kind.advance());
                assert_eq!(request.controls.len(), 2);
                assert!(request.atomics.is_empty());
                assert_eq!(
                    request
                        .controls
                        .iter()
                        .map(|c| c.boundary.as_ref().unwrap().advance())
                        .sum::<f64>(),
                    kind.advance()
                );
            }
        }
    } else {
        assert!(
            batch
                .final_inline_fragments()
                .iter()
                .all(|e| e.fragment().line_index() == 0)
        );
    }
}
#[test]
fn zero_size_inflow_atomic_keeps_a_zero_advance_paragraph_receiver_real() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for cancel_margins in [false, true] {
            nonphantom_receiver::<f32>(ReceiverKind::Atomic { cancel_margins }, direction);
            nonphantom_receiver::<f64>(ReceiverKind::Atomic { cancel_margins }, direction);
        }
    }
}
#[test]
fn each_individual_nonzero_inline_border_or_padding_prevents_phantom_completion() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for (before, after) in [
            ((5.0, 0.0), (0.0, 0.0)),
            ((0.0, 5.0), (0.0, 0.0)),
            ((0.0, 0.0), (5.0, 0.0)),
            ((0.0, 0.0), (0.0, 5.0)),
        ] {
            nonphantom_receiver::<f32>(ReceiverKind::Endpoints { before, after }, direction);
            nonphantom_receiver::<f64>(ReceiverKind::Endpoints { before, after }, direction);
        }
    }
}

#[test]
fn zero_inline_edges_with_only_vertical_struts_leave_receiver_phantom() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        let kind = ReceiverKind::Endpoints {
            before: (0.0, 0.0),
            after: (0.0, 0.0),
        };
        nonphantom_receiver::<f32>(kind, direction);
        nonphantom_receiver::<f64>(kind, direction);
    }
}

// Missing public controls: actual selected source text and consecutive mandatory
// endings, using the same real capture/normal preparation/Layout front door.
struct SemanticReceiverFixture<S: layout::LayoutScalar> {
    processed: Arc<text::ProcessedSource>,
    prepared: text::PreparedText,
    tree: ReceiverTree<S>,
    binding: layout::TextParagraphBinding<u32>,
    wrapping: layout::WrappingInput<u32>,
}
fn semantic_receiver_fixture<S: layout::LayoutScalar>(
    input: text::ContextualSourceInput,
    direction: layout::Direction,
) -> SemanticReceiverFixture<S> {
    let processed = Arc::new(text::process_contextual_source(input).unwrap());
    let prepared = normal_prepare(&processed);
    let px = |value| {
        layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(S::from_f64(value)).unwrap())
    };
    let mut inputs = vec![layout::LayoutInputOf::box_input(layout::NodeInputOf {
        display: layout::Display::Block,
        direction,
        size: layout::Size::new(px(40.0), layout::PreferredSizeOf::AUTO),
        ..layout::NodeInputOf::default()
    })];
    let origins = prepared
        .origin_views()
        .map(|view| {
            let node = inputs.len() as u32;
            inputs.push(layout::LayoutInputOf::inline_text(
                layout::TextParticipation::new(view.handle()),
            ));
            layout::TextOriginBinding {
                origin: view.handle(),
                node,
            }
        })
        .collect::<Vec<_>>();
    let controls = prepared
        .control_views()
        .enumerate()
        .map(|(index, view)| {
            let node = inputs.len() as u32;
            assert_eq!(
                view.handle().captured_control(),
                processed.captured_control(index)
            );
            let kind = match view.kind() {
                text::ParagraphControlKind::InlineBoundaryStart => {
                    layout::InlineBoundaryKind::Start
                }
                text::ParagraphControlKind::InlineBoundaryEnd => layout::InlineBoundaryKind::End,
                _ => panic!("this actual fixture supplies only its genuine inline endpoints"),
            };
            inputs.push(layout::LayoutInputOf::inline_boundary(
                layout::InlineBoundaryInputOf::contribution(
                    kind,
                    layout::InlineMetricsOf::try_new(
                        surgeist_layout::InlineBaselineContext::Physical,
                        S::from_f64(16.0),
                        S::from_f64(20.0),
                    )
                    .unwrap(),
                    layout::InlineBoundaryEdgeOf::ZERO,
                )
                .with_direction(direction),
            ));
            layout::TextControlBinding {
                occurrence: view.handle(),
                node,
            }
        })
        .collect::<Vec<_>>();
    let nodes = (1..inputs.len() as u32).collect::<Vec<_>>();
    let binding = layout::TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: nodes.clone(),
        origins,
        controls,
        atomics: vec![],
        line_strut: None,
    };
    let scope = processed.captured_scope(1).unwrap();
    let inline = layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(scope.clone()));
    let child = |box_| layout::GeneratedChildBoxInput {
        box_,
        participation: layout::GeneratedBoxParticipation::InFlow,
    };
    let row = |parent, children| layout::InlineChildOrderInput {
        parent,
        children,
        prefix: layout::ChildOrderCompleteness::Complete,
        suffix: layout::ChildOrderCompleteness::Complete,
        formatting_context: 0,
    };
    let wrapping = layout::WrappingInput {
        scopes: vec![layout::WrappingScopeInput::Captured {
            scope,
            wrap_inside: layout::WrapInside::Auto,
            edges: layout::WrapEdges::AUTO,
            metric_contribution: Some(
                layout::InlineBoxMetricContribution::Strut(receiver_metric()),
            ),
        }],
        child_order: vec![
            row(
                layout::GeneratedBoxRef::Node(0),
                vec![
                    child(layout::GeneratedBoxRef::Node(1)),
                    child(layout::GeneratedBoxRef::Node(2)),
                    child(inline.clone()),
                ],
            ),
            row(
                inline,
                binding
                    .origins
                    .iter()
                    .skip(2)
                    .map(|origin| child(layout::GeneratedBoxRef::Node(origin.node)))
                    .collect(),
            ),
        ],
    };
    SemanticReceiverFixture {
        processed,
        prepared,
        binding,
        wrapping,
        tree: ReceiverTree {
            inputs,
            non_box: layout::NodeInputOf::non_box(),
            children: nodes,
        },
    }
}

fn retained_receiver_input(
    space: bool,
    direction: layout::Direction,
) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let mut raw = source(true, direction);
    let text::SourceProcessingScope::Block { base_style, .. } = &raw.scopes[0] else {
        unreachable!()
    };
    let base_style = base_style.clone();
    let mut selected_style = base_style.clone();
    selected_style.white_space = text::WhiteSpace::Preserve;
    // A real internal boundary, rather than a suppressed outer half, cancels
    // the two one-em glyphs in the retained zero-advance text receiver.
    selected_style.letter_spacing = if space { 0.0 } else { -20.0 };
    let tail = if space { " " } else { "XX" };
    let tail_len = tail.encode_utf16().count();
    let mut aggregate =
        text::Source::identified(format!("AB{tail}"), text::Id::from_u64(659900), 1);
    aggregate.span(text::Range::new(0, 2), base_style);
    aggregate.span(text::Range::new(2, 2 + tail_len), selected_style.clone());
    raw.source = aggregate;
    raw.origins.push(text::ParagraphOrigin {
        text_node: None,
        source: text::Source::identified(tail, text::Id::from_u64(659903), 1),
        local: text::Range::new(0, tail_len),
        aggregate: text::Range::new(2, 2 + tail_len),
        kind: text::ParagraphOriginKind::Authored,
    });
    raw.connections.push(text::SourceProcessingConnection {
        before_origin: 1,
        after_origin: 2,
        kind: text::ParagraphConnectionKind::Continuous,
    });
    raw.boundary_policies.push(text::ParagraphBoundaryPolicy {
        definition: text::ParagraphBoundaryDefinition::Connection(1),
        preceding_wrap: text::Wrap::None,
        following_wrap: text::Wrap::None,
        ancestor_wrap: text::Wrap::None,
    });
    let text::SourceProcessingScope::Inline { style, .. } = &mut raw.scopes[1] else {
        unreachable!()
    };
    *style = Some(selected_style);
    raw.computed_line_boundaries = vec![text::ComputedLineBoundaryInput::Scope {
        scope: 1,
        side: text::SourceBoxSide::Before,
    }];
    raw.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: text::Affinity::After,
        },
        E::Connection(0),
        E::Origin {
            origin: 1,
            affinity: text::Affinity::After,
        },
        E::EnterScope(1),
        E::Connection(1),
        E::Origin {
            origin: 2,
            affinity: text::Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    raw
}
fn retained_text_receiver<S: layout::LayoutScalar>(space: bool, direction: layout::Direction) {
    let f = semantic_receiver_fixture::<S>(retained_receiver_input(space, direction), direction);
    let tail_len = if space { 1 } else { 2 };
    assert_eq!(
        f.processed.source().text(),
        &text::Text::from(if space { "AB " } else { "ABXX" })
    );
    assert_eq!(f.prepared.origin_views().count(), 3);
    assert_eq!(f.prepared.control_views().count(), 0);
    assert_eq!(f.prepared.atomic_views().count(), 0);
    let origin = f.binding.origins[2].origin.clone();
    assert_eq!(
        origin.view().source().id(),
        Some(text::Id::from_u64(659903))
    );
    assert_eq!(origin.view().source().revision(), 1);
    let offers = f
        .prepared
        .opportunity_views()
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(
        offers
            .iter()
            .map(|o| (
                o.view().kind(),
                o.view().boundary().view().participant_offset()
            ))
            .collect::<Vec<_>>(),
        [
            (text::PreparedBreakKind::Forced, 3),
            (text::PreparedBreakKind::ParagraphEnd, 3 + tail_len)
        ]
    );
    let pin = f
        .prepared
        .boundary_incidences(
            &f.prepared.source_segments().next().unwrap(),
            &offers[0].view().boundary(),
        )
        .unwrap();
    let flow = layout::LayoutFlowInput::try_new(
        &f.tree,
        0,
        vec![f.binding.clone()],
        f.wrapping,
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&flow, 0, request()).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), f.processed.source().text());
    assert_eq!(
        placed.analyzed_source().identity(),
        f.prepared.analyzed_source().identity()
    );
    assert_eq!(placed.nonformatted_lines().count(), 0);
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 2), text::Range::new(2, 2 + tail_len)]
    );
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| (line.baseline(), line.bounds().size.height))
            .collect::<Vec<_>>(),
        [(8.0, 10.0), (26.0, 20.0)]
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        layout::Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
    let fragment = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().finalized_fragment().view().origin() == Some(origin.clone()))
        .expect("the real receiving occurrence keeps a published source fragment");
    assert_eq!(fragment.fragment().line_index(), 1);
    let fragment = fragment.fragment().finalized_fragment();
    let line = fragment.finalized_line();
    for entries in [
        batch.unrounded_inline_fragments(),
        batch.final_inline_fragments(),
    ] {
        let mut fragments = entries
            .iter()
            .filter(|entry| {
                entry.fragment().finalized_fragment().view().origin() == Some(origin.clone())
            })
            .collect::<Vec<_>>();
        fragments.sort_by_key(|entry| {
            entry
                .fragment()
                .finalized_fragment()
                .view()
                .local_range()
                .unwrap()
                .start
        });
        // The selected units retain separate slices of the same occurrence.
        // Their exact local and aggregate ranges jointly cover the whole tail.
        let mut coverage = Vec::new();
        for entry in fragments {
            assert_eq!(
                entry.owner(),
                &layout::InlineFragmentOwner::Node(f.binding.origins[2].node)
            );
            let fragment = entry.fragment();
            assert_eq!(fragment.line_index(), 1);
            assert!(fragment.rect().is_some());
            let issued = fragment.finalized_fragment();
            assert_eq!(issued.finalized_line(), line);
            assert_eq!(issued.finalized_line().prepared_text(), f.prepared);
            coverage.push((
                issued.view().local_range().unwrap(),
                issued.view().aggregate_range().unwrap(),
            ));
        }
        let expected = if space {
            vec![(text::Range::new(0, 1), text::Range::new(2, 3))]
        } else {
            vec![
                (text::Range::new(0, 1), text::Range::new(2, 3)),
                (text::Range::new(1, 2), text::Range::new(3, 4)),
            ]
        };
        assert_eq!(coverage, expected);
    }
    let mut cells = placed
        .clusters()
        .into_iter()
        .filter(|cluster| cluster.range().start >= 2)
        .map(|cluster| (cluster.range(), cluster.bounds().size.width))
        .collect::<Vec<_>>();
    cells.sort_by_key(|cell| cell.0.start);
    let expected_cells = if space {
        vec![(text::Range::new(2, 3), 10.0)]
    } else {
        vec![
            (text::Range::new(2, 3), -10.0),
            (text::Range::new(3, 4), 10.0),
        ]
    };
    assert_eq!(cells, expected_cells);
    assert!(line.has_in_flow_content());
    assert_eq!(
        line.measurements().fit().advance(),
        if space { 10.0 } else { 0.0 }
    );
    let selected = line.selected_line();
    assert_eq!(selected.request().context.end, text::LineEnd::Paragraph);
    assert_eq!(
        selected.request().termination,
        text::PreparedTermination::Ordinary(offers[1].clone())
    );
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // Every retained glyph has its real one-em provider advance. The two X
    // units have one actual internal gap(-20), so their logical total is0.
    let expected = if space {
        vec![(text::Range::new(2, 3), 10.0)]
    } else {
        vec![
            (text::Range::new(2, 3), 10.0),
            (text::Range::new(3, 4), 10.0),
        ]
    };
    assert_eq!(glyphs, expected);
    assert_eq!(
        f.prepared
            .boundary_incidences(
                &f.prepared.source_segments().next().unwrap(),
                &offers[0].view().boundary()
            )
            .unwrap(),
        pin
    );
}
#[test]
fn actual_retained_text_with_cancelled_advance_keeps_paragraph_receiver_real() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        retained_text_receiver::<f32>(false, direction);
        retained_text_receiver::<f64>(false, direction);
    }
}
#[test]
fn preserved_space_keeps_paragraph_receiver_real_at_the_line_edge() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        retained_text_receiver::<f32>(true, direction);
        retained_text_receiver::<f64>(true, direction);
    }
}

fn successive_forced_receivers<S: layout::LayoutScalar>(direction: layout::Direction) {
    let mut input = receiver_input(
        ReceiverKind::Endpoints {
            before: (0.0, 0.0),
            after: (0.0, 0.0),
        },
        direction,
    );
    input.computed_line_boundaries = vec![
        text::ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: text::SourceBoxSide::Before,
        },
        text::ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: text::SourceBoxSide::After,
        },
    ];
    let f = semantic_receiver_fixture::<S>(input, direction);
    let scope = f.processed.captured_scope(1).unwrap();
    let before = f.processed.captured_computed_line_boundary(0).unwrap();
    let after = f.processed.captured_computed_line_boundary(1).unwrap();
    assert_ne!(before, after);
    assert_eq!(before.source_position().get(), 2);
    assert_eq!(after.source_position().get(), 2);
    assert_eq!(f.prepared.origin_views().count(), 2);
    assert_eq!(f.prepared.control_views().count(), 2);
    assert_eq!(f.prepared.participants().count(), 6);
    let offers = f
        .prepared
        .opportunity_views()
        .map(|o| o.handle())
        .collect::<Vec<_>>();
    assert_eq!(
        offers
            .iter()
            .map(|offer| {
                let view = offer.view();
                (
                    view.kind(),
                    view.boundary().view().participant_offset(),
                    view.boundary().view().source_before().get(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (text::PreparedBreakKind::Forced, 3, 2),
            (text::PreparedBreakKind::Forced, 6, 2),
            (text::PreparedBreakKind::ParagraphEnd, 6, 2),
        ]
    );
    let pins = offers
        .iter()
        .take(2)
        .map(|offer| {
            f.prepared
                .boundary_incidences(
                    &f.prepared.source_segments().next().unwrap(),
                    &offer.view().boundary(),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    for (pin, expected) in pins.iter().zip([
        text::CapturedScopeIncidence::AtBefore,
        text::CapturedScopeIncidence::AtAfter,
    ]) {
        assert_eq!(pin.len(), 1);
        assert_eq!(
            f.prepared
                .incidence_scope_relation(&pin[0], &scope)
                .unwrap(),
            expected
        );
    }
    let flow = layout::LayoutFlowInput::try_new(
        &f.tree,
        0,
        vec![f.binding.clone()],
        f.wrapping,
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(&flow, 0, request()).unwrap();
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text(), &text::Text::from("AB"));
    assert_eq!(
        placed
            .lines()
            .iter()
            .map(|line| (line.range(), line.baseline(), line.bounds().size.height))
            .collect::<Vec<_>>(),
        [
            (text::Range::new(0, 2), 8.0, 10.0),
            (text::Range::new(2, 2), 26.0, 20.0),
        ]
    );
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(
        root.size,
        layout::Size::new(S::from_f64(40.0), S::from_f64(30.0))
    );
    let fragment = batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.fragment().line_index() == 1)
        .expect("actual selected endpoint controls identify the real second Forced line");
    let line = fragment.fragment().finalized_fragment().finalized_line();
    assert!(!line.has_in_flow_content());
    assert_eq!(line.measurements().fit().advance(), 0.0);
    let selected = line.selected_line();
    assert_eq!(selected.request().context.end, text::LineEnd::Forced);
    assert_eq!(
        selected.request().termination,
        text::PreparedTermination::Ordinary(offers[1].clone())
    );
    assert_eq!(selected.request().controls.len(), 2);
    assert!(
        line.nonformatted().is_err(),
        "a genuine Forced ending cannot be a phantom receipt"
    );
    let tail = placed.nonformatted_lines().collect::<Vec<_>>();
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].origin.y, 30.0);
    let final_line = tail[0].line.finalized_line();
    let selected = final_line.selected_line();
    assert_eq!(selected.request().context.end, text::LineEnd::Paragraph);
    assert_eq!(
        selected.request().termination,
        text::PreparedTermination::Ordinary(offers[2].clone())
    );
    assert_eq!(selected.request().start, offers[1].view().boundary());
    assert_eq!(selected.request().end, offers[2].view().boundary());
    assert!(selected.request().participants.is_empty());
    assert_eq!(final_line.feedback().line_height, 10.0);
    for (offer, pin) in offers.iter().take(2).zip(pins) {
        assert_eq!(
            f.prepared
                .boundary_incidences(
                    &f.prepared.source_segments().next().unwrap(),
                    &offer.view().boundary()
                )
                .unwrap(),
            pin
        );
    }
}
#[test]
fn consecutive_genuine_forces_at_one_source_offset_each_keep_a_real_line() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        successive_forced_receivers::<f32>(direction);
        successive_forced_receivers::<f64>(direction);
    }
}
