//! Atomic edge declarations must not
//! bypass a genuine captured inline parent. No fonts or preparation are needed.
use std::slice;
use surgeist_layout as layout;
use surgeist_text as text;

#[derive(Clone, Copy)]
enum Proof {
    Absent,
    ParentWithoutOrder,
    Complete,
    IncompleteSide,
}
struct AtomicHost<S: layout::LayoutScalar> {
    root: layout::NodeInputOf<S>,
    atomic: layout::NodeInputOf<S>,
    children: [Vec<u32>; 4],
    associations: Vec<layout::SourceScopeAssociation<u32>>,
    orders: Vec<layout::InlineChildOrderInput<u32, usize>>,
}
impl<S: layout::LayoutScalar> AtomicHost<S> {
    fn new(
        side: text::SourceBoxSide,
        proof: Proof,
        siblings_inside: bool,
        direction: layout::Direction,
        forced: bool,
    ) -> Self {
        let child = |box_| layout::GeneratedChildBoxInput {
            box_,
            participation: layout::GeneratedBoxParticipation::InFlow,
        };
        let node = |n| layout::GeneratedBoxRef::Node(n);
        let scope = layout::GeneratedBoxRef::Inline(layout::InlineBoxRef::Scope(1));
        let row = |parent, children| layout::InlineChildOrderInput {
            parent,
            children,
            prefix: layout::ChildOrderCompleteness::Complete,
            suffix: layout::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let mut orders = if matches!(proof, Proof::Complete | Proof::IncompleteSide) {
            if siblings_inside {
                vec![
                    row(node(0), vec![child(scope.clone())]),
                    row(scope, vec![child(node(1)), child(node(2)), child(node(3))]),
                ]
            } else {
                vec![
                    row(
                        node(0),
                        vec![child(node(1)), child(scope.clone()), child(node(3))],
                    ),
                    row(scope, vec![child(node(2))]),
                ]
            }
        } else {
            vec![]
        };
        if matches!(proof, Proof::IncompleteSide) {
            match side {
                text::SourceBoxSide::Before => {
                    orders[1].prefix = layout::ChildOrderCompleteness::Unknown
                }
                text::SourceBoxSide::After => {
                    orders[1].suffix = layout::ChildOrderCompleteness::Unknown
                }
            }
        }
        let mut edges = layout::WrapEdges::AUTO;
        if forced {
            match side {
                text::SourceBoxSide::Before => edges.before = layout::WrapEdge::Line,
                text::SourceBoxSide::After => edges.after = layout::WrapEdge::Line,
            }
        }
        let px = || {
            layout::PreferredSizeOf::value(
                layout::LengthPercentageOf::px(S::from_f64(10.0)).unwrap(),
            )
        };
        Self {
            root: layout::NodeInputOf {
                display: layout::Display::Block,
                direction,
                ..layout::NodeInputOf::default()
            },
            atomic: layout::NodeInputOf {
                display: layout::Display::InlineBlock,
                direction,
                size: layout::Size::new(px(), px()),
                wrap_edges: edges,
                ..layout::NodeInputOf::default()
            },
            children: [vec![1, 2, 3], vec![], vec![], vec![]],
            associations: if matches!(proof, Proof::Absent) {
                vec![]
            } else {
                vec![layout::SourceScopeAssociation {
                    scope: 1,
                    flow: 0,
                    pair: None,
                }]
            },
            orders,
        }
    }
    fn occurrences(&self) -> layout::SourceOccurrenceAssociations<u32> {
        layout::SourceOccurrenceAssociations {
            origins: vec![(0, 1), (1, 3)],
            controls: vec![],
            atomics: vec![(0, 2)],
        }
    }
}
impl<S: layout::LayoutScalar> layout::Traverse for AtomicHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        self.children[node as usize].iter().copied()
    }
    fn child_count(&self, node: u32) -> usize {
        self.children[node as usize].len()
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.children[node as usize][index]
    }
}
impl<S: layout::LayoutScalar> layout::WrapEdgeSource for AtomicHost<S> {
    fn wrap_edge_source_node(&self, node: u32) -> layout::WrapEdgeSourceNodeView<'_, S> {
        match node {
            0 => layout::WrapEdgeSourceNodeView::Box(&self.root),
            1 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 0 },
            2 => layout::WrapEdgeSourceNodeView::Box(&self.atomic),
            3 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 1 },
            _ => unreachable!("four genuine nodes"),
        }
    }
    fn source_wrapping_scopes(&self) -> &[layout::SourceWrappingScopeInput<u32>] {
        &[]
    }
    fn source_scope_associations(&self) -> &[layout::SourceScopeAssociation<u32>] {
        &self.associations
    }
    fn generated_child_order(&self) -> &[layout::InlineChildOrderInput<u32, usize>] {
        &self.orders
    }
}
fn source(siblings_inside: bool, direction: layout::Direction) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let base = if direction == layout::Direction::Ltr {
        text::ParagraphBaseDirection::LeftToRight
    } else {
        text::ParagraphBaseDirection::RightToLeft
    };
    let mut source = text::Source::identified("AB", text::Id::from_u64(659000), 21);
    source.inline_box(text::InlineBox::new(
        text::Id::from_u64(659003),
        text::InlineBoxKind::InFlow,
        1,
        text::Size::new(10.0, 10.0),
    ));
    let origin = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let atomic = E::Atomic {
        atomic: 0,
        affinity: text::Affinity::After,
    };
    let mut events = vec![E::EnterScope(0), E::EnterParagraph(0)];
    if siblings_inside {
        events.extend([
            E::EnterScope(1),
            origin(0),
            atomic,
            E::Connection(0),
            origin(1),
            E::ExitScope(1),
        ]);
    } else {
        events.extend([
            origin(0),
            E::EnterScope(1),
            atomic,
            E::ExitScope(1),
            E::Connection(0),
            origin(1),
        ]);
    }
    events.extend([E::ExitParagraph(0), E::ExitScope(0)]);
    let policy = |definition| text::ParagraphBoundaryPolicy {
        definition,
        preceding_wrap: text::Wrap::None,
        following_wrap: text::Wrap::None,
        ancestor_wrap: text::Wrap::None,
    };
    text::ContextualSourceInput {
        source,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: ["A", "B"]
            .into_iter()
            .enumerate()
            .map(|(i, s)| text::ParagraphOrigin {
                text_node: None,
                source: text::Source::identified(s, text::Id::from_u64(659001 + i as u64), 21),
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(i, i + 1),
                kind: text::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![text::SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: text::ParagraphConnectionKind::Continuous,
        }],
        controls: vec![],
        computed_line_boundaries: vec![],
        replacements: vec![],
        boundary_policies: vec![
            policy(text::ParagraphBoundaryDefinition::Connection(0)),
            policy(text::ParagraphBoundaryDefinition::AtomicBefore(0)),
            policy(text::ParagraphBoundaryDefinition::AtomicAfter(0)),
        ],
        scopes: vec![
            text::SourceProcessingScope::Block {
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: text::Style {
                    wrap: text::Wrap::None,
                    hyphens: text::Hyphens::None,
                    ..text::Style::default()
                },
                used_direction: base,
            },
            text::SourceProcessingScope::Inline {
                trim: Default::default(),
                style: None,
                boundaries: None,
            },
        ],
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: base,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        }],
        events,
    }
}
fn qualify_raw(input: text::ContextualSourceInput, siblings_inside: bool) {
    let processed = text::process_contextual_source(input).unwrap();
    let s = processed.captured_scope(1).unwrap();
    let atomic = processed.captured_atomic(0).unwrap();
    let a = processed.captured_origin(0).unwrap();
    let b = processed.captured_origin(1).unwrap();
    assert_eq!(s.parent(), processed.captured_scope(0));
    assert_eq!(
        s.domain_topology(),
        text::SourceProcessingTopology::Complete
    );
    assert!(s.enter_position() < atomic);
    assert!(atomic < s.exit_position());
    if siblings_inside {
        assert!(s.enter_position() < a);
        assert!(b < s.exit_position());
    } else {
        assert!(a < s.enter_position());
        assert!(s.exit_position() < b);
    }
    assert!(processed.captured_control(0).is_none());
    assert!(processed.captured_computed_line_boundary(0).is_none());
}
fn unproved_parent<S: layout::LayoutScalar>(proof: Proof) {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for side in [text::SourceBoxSide::Before, text::SourceBoxSide::After] {
            qualify_raw(source(false, direction), false);
            // Neutral declarations need no propagation proof, and establish
            // that these real source/occurrence bindings are admissible.
            let neutral = AtomicHost::<S>::new(side, proof, false, direction, false);
            let (processed, _) = layout::process_wrapping_source(
                &neutral,
                0,
                source(false, direction),
                neutral.occurrences(),
            )
            .unwrap()
            .into_parts();
            assert!(processed.captured_computed_line_boundary(0).is_none());
            let host = AtomicHost::<S>::new(side, proof, false, direction, true);
            let result = layout::process_wrapping_source(
                &host,
                0,
                source(false, direction),
                host.occurrences(),
            );
            let error = match result {
                Ok(_) => panic!(
                    "a forced atomic edge cannot publish a receipt without its real inline-parent proof"
                ),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error.kind(),
                    layout::LayoutErrorKindOf::InvalidInput(
                        layout::LayoutInvalidInputOf::WrappingScope(
                            layout::WrappingScopeError::MissingChildOrder
                                | layout::WrappingScopeError::SourceAssociation
                        )
                    )
                ),
                "{error:?}"
            );
        }
    }
}
#[test]
fn associated_atomic_parent_without_generated_order_refuses_receipt_before_and_after() {
    unproved_parent::<f32>(Proof::ParentWithoutOrder);
    unproved_parent::<f64>(Proof::ParentWithoutOrder);
}
#[test]
fn omitted_atomic_parent_association_cannot_publish_an_unproved_interior_force() {
    unproved_parent::<f32>(Proof::Absent);
    unproved_parent::<f64>(Proof::Absent);
}
#[test]
fn incomplete_relevant_parent_side_refuses_even_when_parent_identity_is_supplied() {
    unproved_parent::<f32>(Proof::IncompleteSide);
    unproved_parent::<f64>(Proof::IncompleteSide);
}
fn complete_parent<S: layout::LayoutScalar>() {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for side in [text::SourceBoxSide::Before, text::SourceBoxSide::After] {
            for siblings_inside in [false, true] {
                qualify_raw(source(siblings_inside, direction), siblings_inside);
                let host =
                    AtomicHost::<S>::new(side, Proof::Complete, siblings_inside, direction, true);
                let (processed, _receipt) = layout::process_wrapping_source(
                    &host,
                    0,
                    source(siblings_inside, direction),
                    host.occurrences(),
                )
                .unwrap()
                .into_parts();
                let cause = processed.captured_computed_line_boundary(0).unwrap();
                assert!(processed.captured_computed_line_boundary(1).is_none());
                let expected = if siblings_inside {
                    text::CapturedComputedLineBoundaryTarget::Atomic {
                        position: processed.captured_atomic(0).unwrap(),
                        side,
                    }
                } else {
                    text::CapturedComputedLineBoundaryTarget::Scope {
                        scope: processed.captured_scope(1).unwrap(),
                        side,
                    }
                };
                assert_eq!(cause.target(), expected);
                assert_eq!(cause.source_position().get(), 1);
                assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
                assert_eq!(cause.paragraph(), processed.captured_paragraph(0).unwrap());
                assert_eq!(processed.source().text(), &text::Text::from("AB"));
                assert_eq!(processed.processed_text().code_units(), [0x41, 0x42]);
                assert!(processed.captured_control(0).is_none());
            }
        }
    }
}
#[test]
fn proved_first_last_atomic_propagates_to_parent_but_real_siblings_stop_at_atomic_edge() {
    complete_parent::<f32>();
    complete_parent::<f64>();
}
