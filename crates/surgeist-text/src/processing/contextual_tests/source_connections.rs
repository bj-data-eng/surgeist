//! Existing-callable source continuity is independent of joint-analysis
//! permission. Expectations come from the actual source, events and raw facts.

use crate::*;
use std::sync::Arc;

fn input(kind: ParagraphConnectionKind) -> ContextualSourceInput {
    let mut input = super::input("A  B", WhiteSpace::Collapse);
    input.source = Source::identified("A  B", Id::from_u64(78420), 1);
    let before = Source::identified("A ", Id::from_u64(78421), 2);
    let after = Source::identified(" B", Id::from_u64(78422), 3);
    input.origins = vec![
        ParagraphOrigin {
            source: before,
            local: Range::new(0, 2),
            aggregate: Range::new(0, 2),
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        },
        ParagraphOrigin {
            source: after,
            local: Range::new(0, 2),
            aggregate: Range::new(2, 4),
            kind: ParagraphOriginKind::Generated,
            text_node: None,
        },
    ];
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
    ];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind,
    }];
    input.boundary_policies = if kind == ParagraphConnectionKind::Continuous {
        vec![super::policy(ParagraphBoundaryDefinition::Connection(0))]
    } else {
        vec![]
    };
    input.joint_flows.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(0),
    ];
    input
}

fn assert_independent_capture(input: ContextualSourceInput, expected: &str) {
    let source = input.source.clone();
    let original = input.origins.clone();
    let facts = input.paragraphs.clone();
    let kind = input.connections[0].kind;
    let policy = input.boundary_policies.first().copied();
    assert!(input.joint_flows.is_empty());
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(expected));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        expected.encode_utf16().collect::<Vec<_>>()
    );
    assert!(processed.captured_joint_flow(0).is_none());
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    for (ordinal, range) in [Range::new(0, 2), Range::new(2, 4)].into_iter().enumerate() {
        let paragraph = processed.captured_paragraph(ordinal).unwrap();
        assert_eq!(paragraph.joint_flow(), None);
        assert_eq!(paragraph.base_direction(), facts[ordinal].base_direction);
        assert_eq!(paragraph.paragraph_strut(), facts[ordinal].paragraph_strut);
        assert!(Arc::ptr_eq(raw[ordinal].processed_source(), &processed));
        assert_eq!(raw[ordinal].source_range(), range);
        assert_eq!(raw[ordinal].source_pieces().collect::<Vec<_>>(), [range]);
        let data = raw[ordinal].data();
        assert_eq!(data.origins.len(), 1);
        assert_eq!(
            data.origins[0].original(&processed),
            &original[ordinal].source
        );
        assert_eq!(data.origins[0].local, Range::new(0, 2));
        assert_eq!(data.origins[0].aggregate, range);
        assert_eq!(data.origins[0].kind, original[ordinal].kind);
        assert!(
            data.connections.is_empty(),
            "a global cross edge is not a fabricated raw-local edge"
        );
        assert!(
            data.boundary_policies.is_empty(),
            "the cross policy remains with the actual global row"
        );
    }
    // Current production metadata is sufficient to observe the global source
    // row without inventing a joint handle or adding a public test API.
    let capture = &raw[0].data().scope_topology.as_ref().unwrap().capture;
    assert!(Arc::ptr_eq(
        capture,
        &raw[1].data().scope_topology.as_ref().unwrap().capture
    ));
    assert_eq!(capture.connections.len(), 1);
    let edge = &capture.connections[0];
    assert_eq!(edge.fact.before_origin, 0);
    assert_eq!(edge.fact.after_origin, 1);
    assert_eq!(edge.fact.kind, kind);
    assert_eq!(edge.order, 3);
    assert_eq!(edge.boundary_policy, policy);
    assert!(matches!(
        capture.events[edge.order].event,
        SourceProcessingEvent::Connection(0)
    ));
    assert_eq!(capture.events[edge.order].at, 2);
}

#[test]
fn continuous_source_edge_preserves_independent_raw_records_without_joint_analysis() {
    assert_independent_capture(input(ParagraphConnectionKind::Continuous), "A B");
}

#[test]
fn source_continuity_does_not_require_equal_raw_direction_or_strut() {
    for (direction, strut) in [
        (
            ParagraphBaseDirection::RightToLeft,
            ParagraphStrut::DefaultFont,
        ),
        (
            ParagraphBaseDirection::LeftToRight,
            ParagraphStrut::AllocatedContributions,
        ),
        (
            ParagraphBaseDirection::RightToLeft,
            ParagraphStrut::AllocatedContributions,
        ),
    ] {
        let mut input = input(ParagraphConnectionKind::Continuous);
        input.paragraphs[1].base_direction = direction;
        input.paragraphs[1].paragraph_strut = strut;
        assert_independent_capture(input, "A B");
    }
}

#[test]
fn disconnected_source_edge_without_policy_keeps_its_barrier_and_independent_views() {
    assert_independent_capture(input(ParagraphConnectionKind::Disconnected), "A  B");
}

#[test]
fn explicit_joint_analysis_still_requires_matching_used_paragraph_facts() {
    for direction_conflict in [true, false] {
        let mut input = input(ParagraphConnectionKind::Continuous);
        input.joint_flows = vec![SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![0, 1],
            facts_owner: 0,
        }];
        let expected = if direction_conflict {
            input.paragraphs[1].base_direction = ParagraphBaseDirection::RightToLeft;
            SourceProcessingJointFlowError::DirectionMismatch {
                declaration: 0,
                paragraph: 1,
                owner: 0,
            }
        } else {
            input.paragraphs[1].paragraph_strut = ParagraphStrut::AllocatedContributions;
            SourceProcessingJointFlowError::StrutMismatch {
                declaration: 0,
                paragraph: 1,
                owner: 0,
            }
        };
        let error = process_contextual_source(input).unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::SourceProcessing {
                reason: SourceProcessingError::JointFlow { reason: expected },
            })
        );
    }
}

// Functional qualification of the subsequently added getter/error seam. The
// four existing-callable RED cases above retain their original behavior.

fn source_refuses(input: ContextualSourceInput, reason: SourceProcessingError) {
    let error = process_contextual_source(input).unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SourceProcessing { reason })
    );
}

fn parent_child(kind: InlineBoxKind) -> ContextualSourceInput {
    let mut input = input(ParagraphConnectionKind::Continuous);
    input.source = Source::identified("ACB", Id::from_u64(78431), 4);
    input.source.inline_box(InlineBox::new(
        Id::from_u64(78432),
        kind,
        1,
        Size::new(0.0, 0.0),
    ));
    input.origins = [
        ("A", Range::new(0, 1)),
        ("C", Range::new(1, 2)),
        ("B", Range::new(2, 3)),
    ]
    .into_iter()
    .map(|(text, aggregate)| ParagraphOrigin {
        source: Source::new(text),
        local: Range::new(0, 1),
        aggregate,
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    })
    .collect();
    input.connections[0].after_origin = 2;
    input.scopes.push(match kind {
        InlineBoxKind::InFlow => SourceProcessingScope::AtomicBlock {
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: Style {
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            used_direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            atomic: 0,
        },
        InlineBoxKind::OutOfFlow => SourceProcessingScope::OutOfFlowBlock {
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: Style {
                white_space: WhiteSpace::Preserve,
                ..Style::default()
            },
            used_direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            atomic: 0,
        },
    });
    if kind == InlineBoxKind::InFlow {
        input.boundary_policies.extend([
            super::policy(ParagraphBoundaryDefinition::AtomicBefore(0)),
            super::policy(ParagraphBoundaryDefinition::AtomicAfter(0)),
        ]);
    }
    input.paragraphs[1].base_direction = ParagraphBaseDirection::RightToLeft;
    input.paragraphs[1].paragraph_strut = ParagraphStrut::AllocatedContributions;
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        SourceProcessingEvent::Origin {
            origin: 2,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    input
}

#[test]
fn global_connection_getter_retains_real_endpoints_policy_and_capture_identity() {
    let input = input(ParagraphConnectionKind::Continuous);
    let supplied_policy = input.boundary_policies[0];
    let processed = Arc::new(process_contextual_source(input.clone()).unwrap());
    let alias = Arc::clone(&processed);
    let foreign = process_contextual_source(input).unwrap();
    let edge = processed.captured_connection(0).unwrap();
    assert_eq!(edge.before_origin(), processed.captured_origin(0).unwrap());
    assert_eq!(edge.after_origin(), processed.captured_origin(1).unwrap());
    assert_eq!(edge.kind(), ParagraphConnectionKind::Continuous);
    assert_eq!(edge.boundary_policy(), Some(supplied_policy));
    assert_eq!(edge.event_position().index, 3);
    assert!(edge.before_origin() < edge.event_position());
    assert!(edge.event_position() < edge.after_origin());
    assert_eq!(edge, edge.clone());
    assert_eq!(edge, alias.captured_connection(0).unwrap());
    let other = foreign.captured_connection(0).unwrap();
    assert_eq!(processed.source(), foreign.source());
    assert_eq!(edge.kind(), other.kind());
    assert_eq!(edge.boundary_policy(), other.boundary_policy());
    assert_ne!(edge, other);
    assert!(!edge.event_position().same_capture(&other.event_position()));
    assert!(processed.captured_connection(1).is_none());
    assert!(processed.captured_joint_flow(0).is_none());
}

#[test]
fn independent_ascii_views_without_a_source_edge_gain_no_connection_or_join() {
    let mut input = input(ParagraphConnectionKind::Continuous);
    input.source = Source::identified("AB", Id::from_u64(78430), 1);
    input.origins = [("A", Range::new(0, 1)), ("B", Range::new(1, 2))]
        .into_iter()
        .map(|(text, aggregate)| ParagraphOrigin {
            source: Source::new(text),
            local: Range::new(0, 1),
            aggregate,
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        })
        .collect();
    input.connections.clear();
    input.boundary_policies.clear();
    input
        .events
        .retain(|event| !matches!(event, SourceProcessingEvent::Connection(_)));
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.processed_text(), &Text::from("AB"));
    assert_eq!(
        super::text(processed.plaintext_items(Affinity::After)),
        [0x41, 0x42]
    );
    assert!(processed.captured_connection(0).is_none());
    assert!(processed.captured_joint_flow(0).is_none());
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    assert_eq!(raw[0].source_range(), Range::new(0, 1));
    assert_eq!(raw[1].source_range(), Range::new(1, 2));
    assert!(
        raw[0]
            .data()
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
            .connections
            .is_empty()
    );
    for (ordinal, view) in raw.iter().enumerate() {
        assert_eq!(
            processed.captured_paragraph(ordinal).unwrap().joint_flow(),
            None
        );
        assert!(view.data().connections.is_empty());
    }
}

#[test]
fn actual_parent_neighbors_skip_child_domain_content_without_join_permission() {
    for kind in [InlineBoxKind::InFlow, InlineBoxKind::OutOfFlow] {
        let processed = Arc::new(process_contextual_source(parent_child(kind)).unwrap());
        let edge = processed.captured_connection(0).unwrap();
        assert_eq!(edge.before_origin(), processed.captured_origin(0).unwrap());
        assert_eq!(edge.after_origin(), processed.captured_origin(2).unwrap());
        assert_ne!(edge.after_origin(), processed.captured_origin(1).unwrap());
        assert_eq!(edge.event_position().index, 9);
        assert_eq!(
            edge.boundary_policy(),
            None,
            "a real local policy stays in its raw projection"
        );
        assert_eq!(processed.processed_text(), &Text::from("ACB"));
        assert!(processed.captured_joint_flow(0).is_none());
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(
            raw[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(2, 3)]
        );
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(1, 2)]
        );
        assert_eq!(raw[0].data().connections.len(), 1);
        assert_eq!(
            super::text(raw[0].plaintext_items(Affinity::After)),
            [0x41, 0x42]
        );
        assert_eq!(super::text(raw[1].plaintext_items(Affinity::After)), [0x43]);
        assert_eq!(
            processed.captured_paragraph(1).unwrap().base_direction(),
            ParagraphBaseDirection::RightToLeft
        );
        assert_eq!(
            processed.captured_paragraph(1).unwrap().paragraph_strut(),
            ParagraphStrut::AllocatedContributions
        );
    }
}

#[test]
fn a_source_edge_can_connect_two_distinct_joint_analysis_owners() {
    let mut input = input(ParagraphConnectionKind::Continuous);
    input.paragraphs[1].base_direction = ParagraphBaseDirection::RightToLeft;
    input.paragraphs[1].paragraph_strut = ParagraphStrut::AllocatedContributions;
    input.joint_flows = vec![
        SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![0],
            facts_owner: 0,
        },
        SourceProcessingJointFlow {
            scope: 0,
            paragraphs: vec![1],
            facts_owner: 1,
        },
    ];
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let first = processed.captured_joint_flow(0).unwrap();
    let second = processed.captured_joint_flow(1).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        processed.captured_paragraph(0).unwrap().joint_flow(),
        Some(first.clone())
    );
    assert_eq!(
        processed.captured_paragraph(1).unwrap().joint_flow(),
        Some(second.clone())
    );
    assert_eq!(
        first.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert_eq!(
        second.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(1).unwrap()]
    );
    assert_eq!(first.base_direction(), ParagraphBaseDirection::LeftToRight);
    assert_eq!(second.base_direction(), ParagraphBaseDirection::RightToLeft);
    assert_eq!(
        second.paragraph_strut(),
        ParagraphStrut::AllocatedContributions
    );
    assert_eq!(first.connections().count(), 0);
    assert_eq!(second.connections().count(), 0);
    let edge = processed.captured_connection(0).unwrap();
    assert_eq!(edge.before_origin(), processed.captured_origin(0).unwrap());
    assert_eq!(edge.after_origin(), processed.captured_origin(1).unwrap());
    assert_eq!(processed.processed_text(), &Text::from("A B"));
    assert_eq!(processed.paragraphs().count(), 2);
}

#[test]
fn source_edges_refuse_non_neighbor_origins_and_wrong_event_positions_without_joins() {
    let mut skipped = input(ParagraphConnectionKind::Continuous);
    skipped.origins.push(ParagraphOrigin {
        source: Source::new(""),
        local: Range::new(0, 0),
        aggregate: Range::new(2, 2),
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    });
    skipped.paragraphs.push(SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    });
    drop(skipped.events.splice(
        5..5,
        [
            SourceProcessingEvent::EnterParagraph(2),
            SourceProcessingEvent::Origin {
                origin: 2,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(2),
        ],
    ));
    source_refuses(
        skipped,
        SourceProcessingError::SourceConnection {
            reason: SourceProcessingConnectionError::Endpoints { connection: 0 },
        },
    );

    let mut position = input(ParagraphConnectionKind::Continuous);
    position.events.remove(3);
    position
        .events
        .insert(6, SourceProcessingEvent::Connection(0));
    source_refuses(
        position,
        SourceProcessingError::SourceConnection {
            reason: SourceProcessingConnectionError::Position { connection: 0 },
        },
    );
}

#[test]
fn source_edges_refuse_duplicate_descriptors_and_missing_or_duplicate_continuous_policies() {
    let mut duplicate = input(ParagraphConnectionKind::Continuous);
    duplicate.connections.push(duplicate.connections[0]);
    duplicate
        .events
        .insert(4, SourceProcessingEvent::Connection(1));
    duplicate
        .boundary_policies
        .push(super::policy(ParagraphBoundaryDefinition::Connection(1)));
    source_refuses(
        duplicate,
        SourceProcessingError::SourceConnection {
            reason: SourceProcessingConnectionError::Duplicate {
                connection: 1,
                first_connection: 0,
            },
        },
    );

    let mut policy = input(ParagraphConnectionKind::Continuous);
    policy.boundary_policies.push(policy.boundary_policies[0]);
    source_refuses(policy, SourceProcessingError::BoundaryPolicy);

    let mut missing = input(ParagraphConnectionKind::Continuous);
    missing.boundary_policies.clear();
    source_refuses(missing, SourceProcessingError::BoundaryPolicy);
}

#[test]
fn source_edge_occurrence_cannot_borrow_its_endpoints_parent_domain() {
    let mut input = parent_child(InlineBoxKind::OutOfFlow);
    input.events.remove(9);
    // Cursor 2 and both genuine parent endpoints are unchanged. Only the
    // occurrence now belongs to child domain1, after its real C origin.
    input.events.insert(7, SourceProcessingEvent::Connection(0));
    source_refuses(
        input,
        SourceProcessingError::SourceConnection {
            reason: SourceProcessingConnectionError::Domain { connection: 0 },
        },
    );
}
