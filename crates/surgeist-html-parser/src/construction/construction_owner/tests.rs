use super::*;
use crate::parser::control::{Invariant, ParseStop};
#[test]
fn aborted_child_bank_ticket_cannot_retire_later_recreated_record_bank() {
    let mut control = ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    );
    let mut owner = ConstructionOwner::new(&mut control).unwrap();
    let root = owner
        .apply(
            &mut control,
            HtmlConstructionCommand::CreateFragment,
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap()
        .unwrap();
    let mut pending = owner
        .start(&mut control, HtmlConstructionCommand::CreateFragment)
        .unwrap();
    loop {
        let edges = pending.before_edges();
        let action = pending.action(&mut control).unwrap().unwrap();
        let child = matches!(
            action.description(),
            HtmlConstructionActionDescription::InitializeChildren { .. }
        );
        assert!(
            !owner
                .perform_action(&mut control, action, &[], edges)
                .unwrap()
        );
        if child {
            break;
        }
    }
    // Retain the actual issued ticket rather than manufacturing an ID/capacity.
    let stale = owner.children[1].take().unwrap();
    let old = stale.snapshot().clone();
    owner.abort_pending(&mut control, pending).unwrap();
    assert!(owner.node_at(1).is_none());
    assert_eq!(owner.storage().position(&root), Ok(0));
    let created = owner
        .apply(
            &mut control,
            HtmlConstructionCommand::CreateFragment,
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap()
        .unwrap();
    let current = owner.children[1].as_ref().unwrap().snapshot();
    assert_ne!(old.id(), current.id());
    assert_eq!(owner.storage().position(&created), Ok(1));
    let before = control.usage().actual_capacity_bytes;
    assert_eq!(
        control.require_construction_retirement(&stale, current),
        Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
    );
    assert_eq!(control.usage().actual_capacity_bytes, before);
    assert_eq!(owner.storage().position(&root), Ok(0));
    assert_eq!(owner.storage().position(&created), Ok(1));
    let stop = control.stop();
    drop(stale);
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), stop);
}

fn insertion_pair(
    limits: crate::HtmlParseLimits,
) -> (
    ParseControl,
    ConstructionOwner,
    HtmlBuilderNode,
    HtmlBuilderNode,
) {
    let mut control = ParseControl::new(limits, crate::HtmlParseCancellation::new());
    let mut owner = ConstructionOwner::new(&mut control).unwrap();
    let root = owner
        .apply(
            &mut control,
            HtmlConstructionCommand::CreateFragment,
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap()
        .unwrap();
    let child = owner
        .apply(
            &mut control,
            HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Comment(DocumentString::default()),
            },
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap()
        .unwrap();
    (control, owner, root, child)
}

fn insertion_preflight_precedes_scratch_capacity(resource: HtmlParseResource) {
    let limits = crate::construction::tests::limits();
    let (mut probe, owner, _, _) = insertion_pair(limits);
    // The observed setup footprint only places the competing capacity boundary.
    // The required first error is independently the original logical preflight.
    let capacity = probe.usage().actual_capacity_bytes;
    owner.retire(&mut probe).unwrap();
    assert_eq!(probe.usage().actual_capacity_bytes, 0);
    let limits = limits.into_builder().managed_capacity_bytes(capacity);
    let limits = match resource {
        HtmlParseResource::ManagedEdges => limits.managed_edges(0),
        HtmlParseResource::TreeDepth => limits.tree_depth(0),
        _ => unreachable!(),
    }
    .finish()
    .unwrap();
    let (mut control, mut owner, root, child) = insertion_pair(limits);
    assert_eq!(control.usage().actual_capacity_bytes, capacity);
    let old_bank = owner.children[0].as_ref().unwrap().snapshot().clone();
    let result = owner.apply(
        &mut control,
        HtmlConstructionCommand::Insert {
            node: child.clone(),
            parent: root.clone(),
            before: None,
        },
        &[(HtmlParseResource::ManagedEdges, 1)],
    );
    let stop = control.stop().expect("competing insertion limit");
    assert!(
        matches!(stop, ParseStop::LimitExceeded { resource: actual, .. } if actual == resource),
        "{stop:?}"
    );
    assert!(matches!(result, Err(SessionError::Stop(actual)) if actual == stop));
    assert_eq!(owner.node_at(0).unwrap().child_count(), 0);
    assert!(owner.node_at(1).unwrap().parent().is_none());
    assert_eq!(
        owner.children[0].as_ref().unwrap().snapshot().id(),
        old_bank.id()
    );
    assert_eq!(control.usage().actual_capacity_bytes, capacity);
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        0
    );
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        0
    );
    let work = [
        WorkCategory::InputUnitRead,
        WorkCategory::InspectByte,
        WorkCategory::CopyByte,
        WorkCategory::StateStep,
        WorkCategory::RecordVisit,
        WorkCategory::RecordWrite,
        WorkCategory::ReserveByte,
        WorkCategory::ProviderCall,
    ]
    .map(|category| control.usage().work(category));
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), Some(stop));
    assert_eq!(
        [
            WorkCategory::InputUnitRead,
            WorkCategory::InspectByte,
            WorkCategory::CopyByte,
            WorkCategory::StateStep,
            WorkCategory::RecordVisit,
            WorkCategory::RecordWrite,
            WorkCategory::ReserveByte,
            WorkCategory::ProviderCall
        ]
        .map(|category| control.usage().work(category)),
        work
    );
}

#[test]
fn insertion_edge_preflight_precedes_scratch_capacity() {
    insertion_preflight_precedes_scratch_capacity(HtmlParseResource::ManagedEdges);
}
#[test]
fn insertion_depth_preflight_precedes_scratch_capacity() {
    insertion_preflight_precedes_scratch_capacity(HtmlParseResource::TreeDepth);
}

fn parser_insertion_pair(
    limits: crate::HtmlParseLimits,
) -> (ParseControl, ParserArenaSession, ParserNode, ParserNode) {
    let mut control = ParseControl::new(limits, crate::HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut control, RootKind::Fragment).unwrap();
    let root = session.root(&mut control).unwrap();
    let child = session
        .allocate_element(
            &mut control,
            HtmlElement::new(
                QualifiedName::new(Namespace::none(), None, "x"),
                OrderedAttributes::default(),
            ),
        )
        .unwrap();
    (control, session, root, child)
}

#[test]
fn parser_insertion_keeps_depth_and_edge_preflight_before_scratch_capacity() {
    for resource in [
        HtmlParseResource::TreeDepth,
        HtmlParseResource::ManagedEdges,
    ] {
        let limits = crate::construction::tests::limits();
        let (mut probe, mut session, _, _) = parser_insertion_pair(limits);
        let capacity = probe.usage().actual_capacity_bytes;
        session.release(&mut probe).unwrap();
        assert_eq!(probe.usage().actual_capacity_bytes, 0);
        let builder = limits.into_builder().managed_capacity_bytes(capacity);
        let limits = match resource {
            HtmlParseResource::TreeDepth => builder.tree_depth(0),
            HtmlParseResource::ManagedEdges => builder.managed_edges(0),
            _ => unreachable!(),
        }
        .finish()
        .unwrap();
        let (mut control, mut session, root, child) = parser_insertion_pair(limits);
        let auxiliary = control
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries);
        let result = session.insert(&mut control, &root, &child, None, None);
        let stop = control.stop().unwrap();
        assert!(
            matches!(stop, ParseStop::LimitExceeded { resource: actual, .. } if actual == resource),
            "{stop:?}"
        );
        assert!(matches!(result, Err(SessionError::Stop(actual)) if actual == stop));
        let storage = session.active().unwrap();
        assert_eq!(
            storage
                .construction
                .node_at(root.index)
                .unwrap()
                .child_count(),
            0
        );
        assert!(
            storage
                .construction
                .node_at(child.index)
                .unwrap()
                .parent()
                .is_none()
        );
        assert!(storage.effects.form.is_none());
        assert_eq!(control.usage().actual_capacity_bytes, capacity);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries),
            auxiliary
        );
        let work = control.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut control).unwrap();
        assert_eq!(control.usage().actual_capacity_bytes, 0);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(control.stop(), Some(stop));
    }
}
