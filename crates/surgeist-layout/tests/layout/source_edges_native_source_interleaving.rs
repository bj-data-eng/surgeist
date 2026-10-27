//! A declared physical native outer
//! box has no canonical InlineScope, but still contains a genuine source box.
use std::slice;
use surgeist_layout as layout;
use surgeist_text as text;

struct MixedHost<S: layout::LayoutScalar> {
    root: layout::NodeInputOf<S>,
    boundaries: [layout::InlineBoundaryInputOf<S>; 4],
    children: [Vec<u32>; 8],
    declarations: Vec<layout::SourceWrappingScopeInput<u32>>,
    associations: Vec<layout::SourceScopeAssociation<u32>>,
    orders: Vec<layout::InlineChildOrderInput<u32, usize>>,
}
impl<S: layout::LayoutScalar> MixedHost<S> {
    fn new(
        side: text::SourceBoxSide,
        empty: bool,
        direction: layout::Direction,
        reverse: bool,
    ) -> Self {
        let outer = layout::InlineBoxRef::NativePair { start: 2, end: 6 };
        let inner = layout::InlineBoxRef::Scope(1);
        let e = layout::InlineBoxRef::Scope(2);
        let child = |box_| layout::GeneratedChildBoxInput {
            box_,
            participation: layout::GeneratedBoxParticipation::InFlow,
        };
        let node = |n| layout::GeneratedBoxRef::Node(n);
        let inline = |b| layout::GeneratedBoxRef::Inline(b);
        let row = |parent, children| layout::InlineChildOrderInput {
            parent,
            children,
            prefix: layout::ChildOrderCompleteness::Complete,
            suffix: layout::ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let mut edge = layout::WrapEdges::AUTO;
        match side {
            text::SourceBoxSide::Before => edge.before = layout::WrapEdge::Line,
            text::SourceBoxSide::After => edge.after = layout::WrapEdge::Line,
        }
        let mut declarations = vec![
            layout::SourceWrappingScopeInput {
                box_: outer.clone(),
                wrap_inside: layout::WrapInside::Auto,
                edges: layout::WrapEdges::AUTO,
            },
            layout::SourceWrappingScopeInput {
                box_: inner.clone(),
                wrap_inside: layout::WrapInside::Auto,
                edges: edge,
            },
        ];
        if reverse {
            declarations.reverse();
        }
        let mut inside = vec![child(inline(inner.clone()))];
        if empty {
            match side {
                text::SourceBoxSide::Before => inside.insert(0, child(inline(e.clone()))),
                text::SourceBoxSide::After => inside.push(child(inline(e.clone()))),
            }
        }
        let mut orders = vec![
            row(
                node(0),
                vec![child(node(1)), child(inline(outer.clone())), child(node(7))],
            ),
            row(inline(outer), inside),
            row(inline(inner), vec![child(node(4))]),
        ];
        let mut associations = vec![layout::SourceScopeAssociation {
            scope: 1,
            flow: 0,
            pair: Some((3, 5)),
        }];
        if empty {
            orders.push(row(inline(e), vec![]));
            associations.push(layout::SourceScopeAssociation {
                scope: 2,
                flow: 0,
                pair: None,
            });
        }
        let metrics = layout::InlineMetricsOf::try_new(
            surgeist_layout::InlineBaselineContext::Physical,
            S::ZERO,
            S::ZERO,
        )
        .unwrap();
        let boundary = |kind| {
            layout::InlineBoundaryInputOf::new(kind, metrics, layout::InlineBoundaryEdgeOf::ZERO)
                .with_direction(direction)
        };
        Self {
            root: layout::NodeInputOf {
                display: layout::Display::Block,
                direction,
                ..layout::NodeInputOf::default()
            },
            boundaries: [
                boundary(layout::InlineBoundaryKind::Start),
                boundary(layout::InlineBoundaryKind::Start),
                boundary(layout::InlineBoundaryKind::End),
                boundary(layout::InlineBoundaryKind::End),
            ],
            children: [
                vec![1, 2, 3, 4, 5, 6, 7],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            declarations,
            associations,
            orders,
        }
    }
    fn occurrences(&self) -> layout::SourceOccurrenceAssociations<u32> {
        layout::SourceOccurrenceAssociations {
            origins: vec![(0, 1), (1, 4), (2, 7)],
            controls: vec![(0, 2), (1, 3), (2, 5), (3, 6)],
            atomics: vec![],
        }
    }
}
impl<S: layout::LayoutScalar> layout::Traverse for MixedHost<S> {
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
impl<S: layout::LayoutScalar> layout::WrapEdgeSource for MixedHost<S> {
    fn wrap_edge_source_node(&self, node: u32) -> layout::WrapEdgeSourceNodeView<'_, S> {
        match node {
            0 => layout::WrapEdgeSourceNodeView::Box(&self.root),
            1 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 0 },
            2 => layout::WrapEdgeSourceNodeView::Boundary(self.boundaries[0]),
            3 => layout::WrapEdgeSourceNodeView::Boundary(self.boundaries[1]),
            4 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 1 },
            5 => layout::WrapEdgeSourceNodeView::Boundary(self.boundaries[2]),
            6 => layout::WrapEdgeSourceNodeView::Boundary(self.boundaries[3]),
            7 => layout::WrapEdgeSourceNodeView::SourceOrigin { origin: 2 },
            _ => unreachable!("the fixture has eight actual nodes"),
        }
    }
    fn source_wrapping_scopes(&self) -> &[layout::SourceWrappingScopeInput<u32>] {
        &self.declarations
    }
    fn source_scope_associations(&self) -> &[layout::SourceScopeAssociation<u32>] {
        &self.associations
    }
    fn generated_child_order(&self) -> &[layout::InlineChildOrderInput<u32, usize>] {
        &self.orders
    }
}
fn mixed_input(
    side: text::SourceBoxSide,
    empty: bool,
    direction: layout::Direction,
) -> text::ContextualSourceInput {
    use text::SourceProcessingEvent as E;
    let base = if direction == layout::Direction::Ltr {
        text::ParagraphBaseDirection::LeftToRight
    } else {
        text::ParagraphBaseDirection::RightToLeft
    };
    let origin = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let start = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let end = |control| E::Control {
        control,
        affinity: text::Affinity::Before,
    };
    let mut events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        origin(0),
        E::Connection(0),
        start(0),
    ];
    if empty && side == text::SourceBoxSide::Before {
        events.extend([E::EnterScope(2), E::ExitScope(2)]);
    }
    events.extend([
        E::EnterScope(1),
        start(1),
        origin(1),
        end(2),
        E::ExitScope(1),
    ]);
    if empty && side == text::SourceBoxSide::After {
        events.extend([E::EnterScope(2), E::ExitScope(2)]);
    }
    events.extend([
        end(3),
        E::Connection(1),
        origin(2),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ]);
    let mut scopes = vec![
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
            boundaries: Some(text::SourceProcessingScopeBoundaries { start: 1, end: 2 }),
        },
    ];
    if empty {
        scopes.push(text::SourceProcessingScope::Inline {
            trim: Default::default(),
            style: None,
            boundaries: None,
        });
    }
    text::ContextualSourceInput {
        source: text::Source::identified("PAQ", text::Id::from_u64(659100), 23),
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        envelope: text::SourceProcessingEnvelope::Complete,
        origins: ["P", "A", "Q"]
            .into_iter()
            .enumerate()
            .map(|(i, s)| text::ParagraphOrigin {
                text_node: None,
                source: text::Source::identified(s, text::Id::from_u64(659101 + i as u64), 23),
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(i, i + 1),
                kind: text::ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: (0..2)
            .map(|i| text::SourceProcessingConnection {
                before_origin: i,
                after_origin: i + 1,
                kind: text::ParagraphConnectionKind::Continuous,
            })
            .collect(),
        boundary_policies: (0..2)
            .map(|i| text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(i),
                preceding_wrap: text::Wrap::None,
                following_wrap: text::Wrap::None,
                ancestor_wrap: text::Wrap::None,
            })
            .collect(),
        controls: vec![
            text::ParagraphControl {
                at: 1,
                kind: text::ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: 1,
                kind: text::ParagraphControlKind::InlineBoundaryStart,
            },
            text::ParagraphControl {
                at: 2,
                kind: text::ParagraphControlKind::InlineBoundaryEnd,
            },
            text::ParagraphControl {
                at: 2,
                kind: text::ParagraphControlKind::InlineBoundaryEnd,
            },
        ],
        computed_line_boundaries: vec![],
        replacements: vec![],
        scopes,
        paragraphs: vec![text::SourceProcessingParagraph {
            base_direction: base,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
        }],
        events,
    }
}
fn qualify_capture(p: &text::ProcessedSource, side: text::SourceBoxSide, empty: bool) {
    let root = p.captured_scope(0).unwrap();
    let s = p.captured_scope(1).unwrap();
    let outer_start = p.captured_control(0).unwrap();
    let inner_start = p.captured_control(1).unwrap();
    let inner_end = p.captured_control(2).unwrap();
    let outer_end = p.captured_control(3).unwrap();
    for (control, expected_kind, expected_at) in [
        (
            &outer_start,
            text::ParagraphControlKind::InlineBoundaryStart,
            1,
        ),
        (
            &inner_start,
            text::ParagraphControlKind::InlineBoundaryStart,
            1,
        ),
        (&inner_end, text::ParagraphControlKind::InlineBoundaryEnd, 2),
        (&outer_end, text::ParagraphControlKind::InlineBoundaryEnd, 2),
    ] {
        assert_eq!(control.kind(), &expected_kind);
        assert_eq!(control.source_position().get(), expected_at);
    }
    assert_eq!(s.parent(), Some(root.clone()));
    assert_eq!(s.entry_control(), Some(inner_start.clone()));
    assert_eq!(s.exit_control(), Some(inner_end.clone()));
    assert_ne!(s.entry_control(), Some(outer_start.clone()));
    assert_ne!(s.exit_control(), Some(outer_end.clone()));
    assert!(p.captured_scope(if empty { 3 } else { 2 }).is_none());
    assert!(p.captured_control(4).is_none());
    assert!(outer_start.event_position() < s.enter_position());
    assert!(s.enter_position() < inner_start.event_position());
    assert!(inner_start.event_position() < p.captured_origin(1).unwrap());
    assert!(p.captured_origin(1).unwrap() < inner_end.event_position());
    assert!(inner_end.event_position() < s.exit_position());
    assert!(s.exit_position() < outer_end.event_position());
    if empty {
        let e = p.captured_scope(2).unwrap();
        assert_eq!(e.parent(), Some(root));
        assert!(e.entry_control().is_none());
        assert!(e.exit_control().is_none());
        match side {
            text::SourceBoxSide::Before => {
                assert!(outer_start.event_position() < e.enter_position());
                assert!(e.exit_position() < s.enter_position());
            }
            text::SourceBoxSide::After => {
                assert!(s.exit_position() < e.enter_position());
                assert!(e.exit_position() < outer_end.event_position());
            }
        }
    }
    assert_eq!(p.source().id(), Some(text::Id::from_u64(659100)));
    assert_eq!(p.source().revision(), 23);
    assert_eq!(p.source().text(), &text::Text::from("PAQ"));
    assert_eq!(p.processed_text().code_units(), [0x50, 0x41, 0x51]);
}
fn mixed_force<S: layout::LayoutScalar>(empty: bool) {
    for direction in [layout::Direction::Ltr, layout::Direction::Rtl] {
        for side in [text::SourceBoxSide::Before, text::SourceBoxSide::After] {
            // Text independently accepts both the original nested controls and
            // the exact effective semantic target, without a fabricated scope N.
            let mut direct = mixed_input(side, empty, direction);
            direct.computed_line_boundaries = vec![if empty {
                text::ComputedLineBoundaryInput::Scope { scope: 1, side }
            } else {
                text::ComputedLineBoundaryInput::Endpoint {
                    control: if side == text::SourceBoxSide::Before {
                        0
                    } else {
                        3
                    },
                }
            }];
            let direct = text::process_contextual_source(direct).unwrap();
            qualify_capture(&direct, side, empty);
            let direct_expected = if empty {
                text::CapturedComputedLineBoundaryTarget::Scope {
                    scope: direct.captured_scope(1).unwrap(),
                    side,
                }
            } else {
                text::CapturedComputedLineBoundaryTarget::Endpoint(
                    direct
                        .captured_control(if side == text::SourceBoxSide::Before {
                            0
                        } else {
                            3
                        })
                        .unwrap(),
                )
            };
            assert_eq!(
                direct.captured_computed_line_boundary(0).unwrap().target(),
                direct_expected
            );
            assert!(direct.captured_computed_line_boundary(1).is_none());
            for reverse in [false, true] {
                let host = MixedHost::<S>::new(side, empty, direction, reverse);
                let (processed, _receipt) = layout::process_wrapping_source(
                    &host,
                    0,
                    mixed_input(side, empty, direction),
                    host.occurrences(),
                )
                .expect("complete real mixed native/source box order must admit")
                .into_parts();
                qualify_capture(&processed, side, empty);
                let cause = processed.captured_computed_line_boundary(0).unwrap();
                assert!(processed.captured_computed_line_boundary(1).is_none());
                let expected = if empty {
                    text::CapturedComputedLineBoundaryTarget::Scope {
                        scope: processed.captured_scope(1).unwrap(),
                        side,
                    }
                } else {
                    text::CapturedComputedLineBoundaryTarget::Endpoint(
                        processed
                            .captured_control(if side == text::SourceBoxSide::Before {
                                0
                            } else {
                                3
                            })
                            .unwrap(),
                    )
                };
                assert_eq!(cause.target(), expected);
                assert_eq!(
                    cause.source_position().get(),
                    if side == text::SourceBoxSide::Before {
                        1
                    } else {
                        2
                    }
                );
                assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
                assert_eq!(cause.paragraph(), processed.captured_paragraph(0).unwrap());
            }
        }
    }
}
#[test]
fn captured_inner_force_propagates_to_real_unscoped_native_outer_endpoint() {
    mixed_force::<f32>(false);
    mixed_force::<f64>(false);
}
#[test]
fn real_empty_source_sibling_stops_inner_force_inside_unscoped_native_outer() {
    mixed_force::<f32>(true);
    mixed_force::<f64>(true);
}
