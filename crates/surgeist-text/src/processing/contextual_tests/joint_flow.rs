//! Functional qualification of explicit joint-flow capture authority. These
//! tests observe existing raw records, not a union analysis or used presentation.

use crate::*;
use std::sync::Arc;

fn paragraph() -> SourceProcessingParagraph {
    SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    }
}

fn connection(
    before: usize,
    after: usize,
    kind: ParagraphConnectionKind,
) -> SourceProcessingConnection {
    SourceProcessingConnection {
        before_origin: before,
        after_origin: after,
        kind,
    }
}

fn event_origin(origin: usize) -> SourceProcessingEvent {
    SourceProcessingEvent::Origin {
        origin,
        affinity: Affinity::After,
    }
}

fn three_members() -> ContextualSourceInput {
    let mut input = super::input("AB", WhiteSpace::Preserve);
    input.origins = [Range::new(0, 1), Range::new(1, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    // Input paragraph IDs are deliberately different from actual event order.
    input.paragraphs = vec![paragraph(), paragraph(), paragraph()];
    input.connections = vec![
        connection(0, 1, ParagraphConnectionKind::Continuous),
        connection(1, 2, ParagraphConnectionKind::Continuous),
    ];
    input.boundary_policies = (0..2)
        .map(|ordinal| super::policy(ParagraphBoundaryDefinition::Connection(ordinal)))
        .collect();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(1),
        event_origin(0),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(2),
        event_origin(1),
        SourceProcessingEvent::Connection(1),
        SourceProcessingEvent::ExitParagraph(2),
        SourceProcessingEvent::EnterParagraph(0),
        event_origin(2),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    input.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![1, 2, 0],
        facts_owner: 2,
    }];
    input
}

fn two_members(kind: ParagraphConnectionKind) -> ContextualSourceInput {
    let mut input = super::input("A  B", WhiteSpace::Collapse);
    input.origins = [Range::new(0, 2), Range::new(2, 4)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    input.paragraphs = vec![paragraph(), paragraph()];
    input.connections = vec![connection(0, 1, kind)];
    input.boundary_policies = vec![super::policy(ParagraphBoundaryDefinition::Connection(0))];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(1),
        event_origin(0),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(0),
        event_origin(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    input.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![1, 0],
        facts_owner: 1,
    }];
    input
}

fn with_child_hole(kind: InlineBoxKind) -> ContextualSourceInput {
    let mut input = super::hole(kind);
    input.paragraphs.push(paragraph());
    input
        .origins
        .push(super::origin(&input.source, Range::new(9, 9)));
    input
        .connections
        .push(connection(2, 3, ParagraphConnectionKind::Continuous));
    input
        .boundary_policies
        .push(super::policy(ParagraphBoundaryDefinition::Connection(1)));
    // Preserve all genuine atomic/child-domain events and both original raw
    // records. A real empty final parent member follows the first parent record.
    input
        .events
        .insert(11, SourceProcessingEvent::Connection(1));
    let outer_exit = input.events.len() - 1;
    input.events.splice(
        outer_exit..outer_exit,
        [
            SourceProcessingEvent::EnterParagraph(2),
            event_origin(3),
            SourceProcessingEvent::ExitParagraph(2),
        ],
    );
    input.joint_flows = vec![SourceProcessingJointFlow {
        scope: 0,
        paragraphs: vec![0, 2],
        facts_owner: 0,
    }];
    input
}

fn reason(error: &Error) -> &SourceProcessingError {
    match error.detail() {
        Some(ErrorDetail::SourceProcessing { reason }) => reason,
        _ => panic!("expected the typed contextual source error: {error:?}"),
    }
}

fn refuses(input: ContextualSourceInput, expected: SourceProcessingJointFlowError) {
    let error = process_contextual_source(input).unwrap_err();
    assert_eq!(
        reason(&error),
        &SourceProcessingError::JointFlow { reason: expected }
    );
}

fn refuses_source(input: ContextualSourceInput, expected: SourceProcessingConnectionError) {
    let error = process_contextual_source(input).unwrap_err();
    assert_eq!(
        reason(&error),
        &SourceProcessingError::SourceConnection { reason: expected }
    );
}

#[test]
fn explicit_flow_keeps_empty_member_facts_owner_and_event_ordered_raw_records() {
    let input = three_members();
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let joint = processed.captured_joint_flow(0).unwrap();
    let members = [1, 2, 0].map(|ordinal| processed.captured_paragraph(ordinal).unwrap());
    assert_eq!(joint.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(joint.domain(), joint.scope());
    assert_eq!(joint.members().len(), 3);
    assert_eq!(joint.members().collect::<Vec<_>>(), members);
    assert_eq!(joint.facts_owner(), members[1]);
    assert_eq!(joint.base_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(joint.paragraph_strut(), ParagraphStrut::DefaultFont);
    for member in &members {
        assert_eq!(member.joint_flow(), Some(joint.clone()));
        assert_eq!(member.base_direction(), ParagraphBaseDirection::LeftToRight);
        assert_eq!(member.paragraph_strut(), ParagraphStrut::DefaultFont);
    }
    let edges = joint.connections().collect::<Vec<_>>();
    assert_eq!(edges.len(), 2);
    for (ordinal, edge) in edges.iter().enumerate() {
        assert_eq!(
            edge.before_origin(),
            processed.captured_origin(ordinal).unwrap()
        );
        assert_eq!(
            edge.after_origin(),
            processed.captured_origin(ordinal + 1).unwrap()
        );
        assert_eq!(edge.kind(), ParagraphConnectionKind::Continuous);
        assert!(edge.before_origin() < edge.event_position());
        assert!(edge.event_position() < edge.after_origin());
        assert_eq!(
            edge.boundary_policy(),
            Some(super::policy(ParagraphBoundaryDefinition::Connection(
                ordinal
            )))
        );
    }
    assert!(edges[0].event_position() < edges[1].event_position());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x41, 0x42]
    );
    processed.admit_joint_flow(&joint).unwrap();
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let views = processed
        .joint_flow_paragraphs(&joint)
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(views.len(), 3);
    for (index, (range, expected)) in [
        (Range::new(0, 1), "A"),
        (Range::new(1, 1), ""),
        (Range::new(1, 2), "B"),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(Arc::ptr_eq(views[index].processed_source(), &processed));
        assert_eq!(views[index].source_range(), range);
        assert_eq!(views[index].source_pieces().collect::<Vec<_>>(), [range]);
        assert_eq!(
            super::text(views[index].plaintext_items(Affinity::After)),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        // Private observation of the actual already-issued row reference:
        // successful readback cannot manufacture a synthetic merged record.
        assert!(std::ptr::eq(views[index].data(), raw[index].data()));
    }
    assert!(members[0].exit_position() < members[1].enter_position());
    assert!(members[1].exit_position() < members[2].enter_position());
}

#[test]
fn equal_facts_and_continuous_connections_do_not_supply_joint_authority() {
    let mut undeclared = three_members();
    undeclared.joint_flows.clear();
    let independent = Arc::new(process_contextual_source(undeclared.clone()).unwrap());
    assert!(independent.captured_joint_flow(0).is_none());
    for ordinal in 0..3 {
        assert!(
            independent
                .captured_paragraph(ordinal)
                .unwrap()
                .joint_flow()
                .is_none()
        );
    }

    // Independent old raw records remain valid without cross-view joins.
    undeclared.connections.clear();
    undeclared.boundary_policies.clear();
    undeclared
        .events
        .retain(|event| !matches!(event, SourceProcessingEvent::Connection(_)));
    let processed = Arc::new(process_contextual_source(undeclared).unwrap());
    assert!(processed.captured_joint_flow(0).is_none());
    assert_eq!(processed.paragraphs().count(), 3);
    for ordinal in 0..3 {
        let paragraph = processed.captured_paragraph(ordinal).unwrap();
        assert_eq!(paragraph.joint_flow(), None);
        assert_eq!(
            paragraph.base_direction(),
            ParagraphBaseDirection::LeftToRight
        );
        assert_eq!(paragraph.paragraph_strut(), ParagraphStrut::DefaultFont);
    }
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn joint_receipts_preserve_atomic_domain_holes_and_exact_existing_parent_views() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let input = with_child_hole(kind);
        let source = input.source.clone();
        let processed = Arc::new(process_contextual_source(input).unwrap());
        let joint = processed.captured_joint_flow(0).unwrap();
        assert_eq!(
            joint.members().collect::<Vec<_>>(),
            [
                processed.captured_paragraph(0).unwrap(),
                processed.captured_paragraph(2).unwrap()
            ]
        );
        assert_eq!(processed.captured_paragraph(1).unwrap().joint_flow(), None);
        assert_eq!(
            processed.captured_paragraph(1).unwrap().base_direction(),
            ParagraphBaseDirection::RightToLeft
        );
        assert_eq!(joint.base_direction(), ParagraphBaseDirection::LeftToRight);
        let edges = joint.connections().collect::<Vec<_>>();
        assert_eq!(edges.len(), 2);
        assert_eq!(
            edges[0].before_origin(),
            processed.captured_origin(0).unwrap()
        );
        assert_eq!(
            edges[0].after_origin(),
            processed.captured_origin(2).unwrap()
        );
        assert_eq!(
            edges[0].boundary_policy(),
            None,
            "local policy stays in its actual raw projection"
        );
        assert_eq!(
            edges[1].before_origin(),
            processed.captured_origin(2).unwrap()
        );
        assert_eq!(
            edges[1].after_origin(),
            processed.captured_origin(3).unwrap()
        );
        assert_eq!(
            edges[1].boundary_policy(),
            Some(super::policy(ParagraphBoundaryDefinition::Connection(1)))
        );
        assert_eq!(processed.source(), &source);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let joined = processed
            .joint_flow_paragraphs(&joint)
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(raw.len(), 3);
        assert_eq!(joined.len(), 2);
        assert!(std::ptr::eq(joined[0].data(), raw[0].data()));
        assert!(std::ptr::eq(joined[1].data(), raw[2].data()));
        assert_eq!(
            joined[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 3), Range::new(6, 9)]
        );
        assert_eq!(
            joined[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(9, 9)]
        );
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(3, 6)]
        );
        assert_eq!(
            super::text(joined[0].plaintext_items(Affinity::After)),
            if kind == InlineBoxKind::InFlow {
                "A  B"
            } else {
                "A B"
            }
            .encode_utf16()
            .collect::<Vec<_>>()
        );
        assert!(super::text(joined[1].plaintext_items(Affinity::After)).is_empty());
        assert_eq!(
            super::text(raw[1].plaintext_items(Affinity::After)),
            " I ".encode_utf16().collect::<Vec<_>>()
        );
        let actual_atomics = joint
            .scope()
            .flow_events()
            .unwrap()
            .filter_map(|event| match event {
                CapturedFlowEvent::Atomic { position, .. } => Some(position),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_atomics, [processed.captured_atomic(0).unwrap()]);
    }
}

#[test]
fn disconnected_cross_view_edge_is_a_real_lexical_barrier_once() {
    use crate::processing::lexical_inventory::{
        LexicalBarrierOrigin, LexicalRecordKind, LexicalState,
    };
    for (kind, expected, barriers) in [
        (ParagraphConnectionKind::Continuous, "A B", 0),
        (ParagraphConnectionKind::Disconnected, "A  B", 1),
    ] {
        let processed = Arc::new(process_contextual_source(two_members(kind)).unwrap());
        let joint = processed.captured_joint_flow(0).unwrap();
        assert_eq!(joint.members().len(), 2);
        assert_eq!(joint.connections().next().unwrap().kind(), kind);
        assert_eq!(processed.processed_text(), &Text::from(expected));
        assert_eq!(
            super::text(processed.plaintext_items(Affinity::After)),
            expected.encode_utf16().collect::<Vec<_>>()
        );
        let LexicalState::Executed { records, .. } = &processed.lexical else {
            panic!("actual executed source")
        };
        assert_eq!(
            records
                .iter()
                .filter(|record| matches!(
                    record.kind,
                    LexicalRecordKind::Barrier {
                        origin: LexicalBarrierOrigin::CapturedEvent(3)
                    }
                ))
                .count(),
            barriers
        );
        assert_eq!(
            processed
                .joint_flow_paragraphs(&joint)
                .unwrap()
                .map(|view| view.source_range())
                .collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 4)]
        );
    }
}

#[test]
fn joint_capture_retains_a_real_forced_control_occurrence_without_text_invention() {
    let mut input = three_members();
    input.controls.push(ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    });
    input.events.insert(
        3,
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::Before,
        },
    );
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let joint = processed.captured_joint_flow(0).unwrap();
    let control = processed.captured_control(0).unwrap();
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x41, 0x42]
    );
    assert_eq!(
        joint
            .scope()
            .flow_events()
            .unwrap()
            .filter_map(|event| match event {
                CapturedFlowEvent::Control(control) => Some(control),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [control]
    );
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let joined = processed
        .joint_flow_paragraphs(&joint)
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(joined.len(), 3);
    assert_eq!(joined[0].data().controls.len(), 1);
    assert!(matches!(
        joined[0].data().controls[0].kind,
        ParagraphControlKind::ForcedBreak
    ));
    assert!(joined[1].data().controls.is_empty());
    assert!(joined[2].data().controls.is_empty());
    assert!(std::ptr::eq(joined[0].data(), raw[0].data()));
}

#[test]
fn aliases_admit_but_a_fresh_capture_with_equal_source_and_facts_is_foreign() {
    let input = three_members();
    let processed = Arc::new(process_contextual_source(input.clone()).unwrap());
    let retained = Arc::clone(&processed);
    let foreign = Arc::new(process_contextual_source(input).unwrap());
    let joint = processed.captured_joint_flow(0).unwrap();
    let alias = retained.captured_joint_flow(0).unwrap();
    let other = foreign.captured_joint_flow(0).unwrap();
    assert_eq!(joint, joint.clone());
    assert_eq!(joint, alias);
    assert_ne!(joint, other);
    assert_ne!(
        joint.connections().next().unwrap(),
        other.connections().next().unwrap()
    );
    assert_eq!(processed.source(), foreign.source());
    assert_eq!(joint.base_direction(), other.base_direction());
    assert_eq!(joint.paragraph_strut(), other.paragraph_strut());
    processed.admit_joint_flow(&alias).unwrap();
    assert!(matches!(
        reason(&processed.admit_joint_flow(&other).unwrap_err()),
        SourceProcessingError::ForeignJointFlow
    ));
    let error = match processed.joint_flow_paragraphs(&other) {
        Err(error) => error,
        Ok(_) => panic!("equal source facts do not admit a foreign joint owner"),
    };
    assert!(matches!(
        reason(&error),
        SourceProcessingError::ForeignJointFlow
    ));
    assert_eq!(
        processed
            .joint_flow_paragraphs(&joint)
            .unwrap()
            .map(|view| view.source_range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 1), Range::new(1, 2)]
    );
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn explicit_nondefault_paragraph_facts_do_not_borrow_domain_defaults() {
    let mut input = three_members();
    for facts in &mut input.paragraphs {
        facts.base_direction = ParagraphBaseDirection::RightToLeft;
        facts.paragraph_strut = ParagraphStrut::AllocatedContributions;
    }
    // The genuine fixture's outer domain remains LTR and its text is ASCII.
    // Neither fact substitutes for explicitly supplied raw paragraph facts.
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let joint = processed.captured_joint_flow(0).unwrap();
    assert_eq!(
        joint.facts_owner(),
        processed.captured_paragraph(2).unwrap()
    );
    assert_eq!(joint.base_direction(), ParagraphBaseDirection::RightToLeft);
    assert_eq!(
        joint.paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    for member in joint.members() {
        assert_eq!(member.base_direction(), ParagraphBaseDirection::RightToLeft);
        assert_eq!(
            member.paragraph_strut(),
            ParagraphStrut::AllocatedContributions
        );
    }
    assert_eq!(processed.processed_text(), &Text::from("AB"));
}

#[test]
fn declarations_refuse_empty_bounds_duplicate_overlap_order_and_skipped_empty_member() {
    use SourceProcessingJointFlowError as J;
    let mut empty = two_members(ParagraphConnectionKind::Continuous);
    empty.joint_flows[0].paragraphs.clear();
    refuses(empty, J::EmptyMembers { declaration: 0 });

    let mut scope = two_members(ParagraphConnectionKind::Continuous);
    scope.joint_flows[0].scope = 9;
    refuses(
        scope,
        J::InvalidScope {
            declaration: 0,
            scope: 9,
        },
    );

    let mut member = two_members(ParagraphConnectionKind::Continuous);
    member.joint_flows[0].paragraphs = vec![9];
    refuses(
        member,
        J::InvalidMember {
            declaration: 0,
            paragraph: 9,
        },
    );

    let mut duplicate = two_members(ParagraphConnectionKind::Continuous);
    duplicate.joint_flows[0].paragraphs = vec![1, 1, 0];
    refuses(
        duplicate,
        J::DuplicateMember {
            declaration: 0,
            paragraph: 1,
        },
    );

    let mut overlap = two_members(ParagraphConnectionKind::Continuous);
    overlap.joint_flows.push(overlap.joint_flows[0].clone());
    refuses(
        overlap,
        J::MemberAlreadyAssigned {
            declaration: 1,
            paragraph: 1,
            first_declaration: 0,
        },
    );

    let mut order = two_members(ParagraphConnectionKind::Continuous);
    order.joint_flows[0].paragraphs = vec![0, 1];
    refuses(
        order,
        J::MemberOrder {
            declaration: 0,
            previous: 0,
            paragraph: 1,
        },
    );

    let mut skipped = three_members();
    skipped.joint_flows[0].paragraphs = vec![1, 0];
    skipped.joint_flows[0].facts_owner = 1;
    refuses(
        skipped,
        J::SkippedMember {
            declaration: 0,
            paragraph: 2,
        },
    );
}

#[test]
fn owning_scope_must_be_a_domain_and_every_member_must_belong_to_it() {
    use SourceProcessingJointFlowError as J;
    let mut inline = two_members(ParagraphConnectionKind::Continuous);
    inline.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
        horizontal_inner_direction: None,
    });
    inline
        .events
        .insert(1, SourceProcessingEvent::EnterScope(1));
    inline
        .events
        .insert(inline.events.len() - 1, SourceProcessingEvent::ExitScope(1));
    inline.joint_flows[0].scope = 1;
    refuses(
        inline,
        J::NotDomainRoot {
            declaration: 0,
            scope: 1,
        },
    );

    let mut foreign_domain = with_child_hole(InlineBoxKind::InFlow);
    foreign_domain.joint_flows[0].paragraphs = vec![0, 1, 2];
    refuses(
        foreign_domain,
        J::WrongDomain {
            declaration: 0,
            paragraph: 1,
        },
    );
}

#[test]
fn designated_owner_and_exact_used_direction_and_strut_are_required() {
    use SourceProcessingJointFlowError as J;
    let mut owner = two_members(ParagraphConnectionKind::Continuous);
    owner.joint_flows[0].facts_owner = 9;
    refuses(
        owner,
        J::FactsOwnerNotMember {
            declaration: 0,
            paragraph: 9,
        },
    );

    let mut direction = two_members(ParagraphConnectionKind::Continuous);
    direction.paragraphs[0].base_direction = ParagraphBaseDirection::RightToLeft;
    refuses(
        direction,
        J::DirectionMismatch {
            declaration: 0,
            paragraph: 0,
            owner: 1,
        },
    );

    let mut strut = two_members(ParagraphConnectionKind::Continuous);
    strut.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    refuses(
        strut,
        J::StrutMismatch {
            declaration: 0,
            paragraph: 0,
            owner: 1,
        },
    );
}

#[test]
fn cross_view_connections_require_actual_successive_endpoints_and_position() {
    use SourceProcessingConnectionError as C;
    use SourceProcessingJointFlowError as J;
    let mut missing = two_members(ParagraphConnectionKind::Continuous);
    missing.connections.clear();
    missing.boundary_policies.clear();
    missing
        .events
        .retain(|event| !matches!(event, SourceProcessingEvent::Connection(_)));
    refuses(
        missing,
        J::MissingConnection {
            declaration: 0,
            before_origin: 0,
            after_origin: 1,
        },
    );

    let mut duplicate = two_members(ParagraphConnectionKind::Continuous);
    duplicate.connections.push(duplicate.connections[0]);
    duplicate
        .boundary_policies
        .push(super::policy(ParagraphBoundaryDefinition::Connection(1)));
    duplicate
        .events
        .insert(4, SourceProcessingEvent::Connection(1));
    refuses_source(
        duplicate,
        C::Duplicate {
            connection: 1,
            first_connection: 0,
        },
    );

    // All these endpoints share cut 1, but the genuine empty origin is the
    // next occurrence. Numeric adjacency cannot authorize skipping it.
    let mut endpoints = three_members();
    endpoints.connections[0].after_origin = 2;
    refuses_source(endpoints, C::Endpoints { connection: 0 });

    let mut position = two_members(ParagraphConnectionKind::Continuous);
    position.events.remove(3);
    // The after-origin has now executed: this is source cursor 4, not cut 2.
    position
        .events
        .insert(6, SourceProcessingEvent::Connection(0));
    refuses_source(position, C::Position { connection: 0 });

    let mut undeclared_edge = two_members(ParagraphConnectionKind::Continuous);
    undeclared_edge.joint_flows[0].paragraphs = vec![1];
    let independent_edge = Arc::new(process_contextual_source(undeclared_edge).unwrap());
    let declared = independent_edge.captured_joint_flow(0).unwrap();
    assert_eq!(
        declared.members().collect::<Vec<_>>(),
        [independent_edge.captured_paragraph(1).unwrap()]
    );
    assert!(
        independent_edge
            .captured_paragraph(0)
            .unwrap()
            .joint_flow()
            .is_none()
    );
}

#[test]
fn continuous_cross_view_edge_requires_its_policy_but_disconnected_may_omit_it() {
    let mut continuous = two_members(ParagraphConnectionKind::Continuous);
    continuous.boundary_policies.clear();
    let error = process_contextual_source(continuous).unwrap_err();
    assert_eq!(reason(&error), &SourceProcessingError::BoundaryPolicy);

    let mut disconnected = two_members(ParagraphConnectionKind::Disconnected);
    disconnected.boundary_policies.clear();
    let processed = Arc::new(process_contextual_source(disconnected).unwrap());
    let joint = processed.captured_joint_flow(0).unwrap();
    assert_eq!(processed.processed_text(), &Text::from("A  B"));
    let edges = joint.connections().collect::<Vec<_>>();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind(), ParagraphConnectionKind::Disconnected);
    assert_eq!(edges[0].boundary_policy(), None);
    assert_eq!(joint.members().len(), 2);
}

#[test]
fn cross_view_connection_cannot_cut_inside_an_actual_declared_crlf_delimiter() {
    // The native delimiter is original 1..3 as one occurrence, not two LF/CR
    // tokens. Its true edges are legal storage cuts; its interior is not.
    for cut in [1, 2, 3] {
        let mut input = two_members(ParagraphConnectionKind::Continuous);
        input.source = Source::new("A\r\nB");
        input.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]);
        input.origins = [Range::new(0, cut), Range::new(cut, 4)]
            .into_iter()
            .map(|range| super::origin(&input.source, range))
            .collect();
        if cut == 2 {
            let error = process_contextual_source(input).unwrap_err();
            assert_eq!(reason(&error), &SourceProcessingError::DelimiterCrossing);
        } else {
            let source = input.source.clone();
            let processed = Arc::new(process_contextual_source(input).unwrap());
            let joint = processed.captured_joint_flow(0).unwrap();
            assert_eq!(processed.source(), &source);
            assert_eq!(processed.processed_text(), &Text::from("A B"));
            assert_eq!(
                processed
                    .joint_flow_paragraphs(&joint)
                    .unwrap()
                    .map(|view| view.source_range())
                    .collect::<Vec<_>>(),
                [Range::new(0, cut), Range::new(cut, 4)]
            );
        }
    }
}
