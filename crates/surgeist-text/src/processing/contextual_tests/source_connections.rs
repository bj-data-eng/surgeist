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
