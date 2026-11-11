use super::*;
use crate::parser::control::{HtmlParseInvariant, HtmlParseStop};
use surgeist_html::{
    bounded::construction::{
        HtmlConstructionActionDescription, HtmlConstructionCommand, HtmlConstructionLeaf,
    },
    construction::{HtmlBuilderNode, HtmlConstructionResource},
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};
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
    assert_eq!(owner.test_storage().position(&root), Ok(0));
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
    assert_eq!(owner.test_storage().position(&created), Ok(1));
    let before = control.usage().actual_capacity_bytes;
    assert_eq!(
        control.require_construction_retirement(&stale, current),
        Err(HtmlParseStop::InternalInvariant(
            HtmlParseInvariant::CapacityMismatch
        ))
    );
    assert_eq!(control.usage().actual_capacity_bytes, before);
    assert_eq!(owner.test_storage().position(&root), Ok(0));
    assert_eq!(owner.test_storage().position(&created), Ok(1));
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
                leaf: HtmlConstructionLeaf::Comment(DocumentString::default()),
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
            node: child,
            parent: root,
            before: None,
        },
        &[(HtmlParseResource::ManagedEdges, 1)],
    );
    let stop = control.stop().expect("competing insertion limit");
    assert!(
        matches!(stop, HtmlParseStop::LimitExceeded { resource: actual, .. } if actual == resource),
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
        HtmlParseWorkCategory::InputUnitRead,
        HtmlParseWorkCategory::InspectByte,
        HtmlParseWorkCategory::CopyByte,
        HtmlParseWorkCategory::StateStep,
        HtmlParseWorkCategory::RecordVisit,
        HtmlParseWorkCategory::RecordWrite,
        HtmlParseWorkCategory::ReserveByte,
        HtmlParseWorkCategory::ProviderCall,
    ]
    .map(|category| control.usage().work(category));
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), Some(stop));
    assert_eq!(
        [
            HtmlParseWorkCategory::InputUnitRead,
            HtmlParseWorkCategory::InspectByte,
            HtmlParseWorkCategory::CopyByte,
            HtmlParseWorkCategory::StateStep,
            HtmlParseWorkCategory::RecordVisit,
            HtmlParseWorkCategory::RecordWrite,
            HtmlParseWorkCategory::ReserveByte,
            HtmlParseWorkCategory::ProviderCall
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
            PreexistingElement::new(
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
            matches!(stop, HtmlParseStop::LimitExceeded { resource: actual, .. } if actual == resource),
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

// Impossible custody states are reported as the controller's first stop with
// the owner's storage handed back, never as a panic. Each test empties one
// custody slot through crate-private access, asserts the exact typed error,
// that the stop is latched first, and that the owner's own retirement drains
// every resource afterwards.
fn registry_row() -> HtmlParseStop {
    HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow)
}

fn assert_registry_row<T>(result: Result<T, SessionError>, control: &ParseControl) {
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(registry_row())),
        "typed registry-row error"
    );
    assert_eq!(
        control.stop(),
        Some(registry_row()),
        "latched as the first stop"
    );
}

fn drained(owner: ConstructionOwner, control: &mut ParseControl) {
    owner.retire(control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
    assert_eq!(control.stop(), Some(registry_row()));
}

fn fresh() -> (ParseControl, ConstructionOwner) {
    let mut control = ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    );
    let owner = ConstructionOwner::new(&mut control).unwrap();
    (control, owner)
}

/// An owner with one element record at position 0, the target of attribute edits.
fn with_element() -> (ParseControl, ConstructionOwner) {
    let (mut control, mut owner) = fresh();
    owner
        .apply(
            &mut control,
            HtmlConstructionCommand::CreateElement {
                name: QualifiedName::new(Namespace::none(), None, "x"),
                attributes: OrderedAttributes::default(),
                link_flags: None,
            },
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap();
    (control, owner)
}

#[test]
fn starting_an_operation_without_the_storage_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let storage = owner.owner.take().unwrap();
    let result = owner.start(&mut control, HtmlConstructionCommand::CreateFragment);
    assert_registry_row(result, &control);
    owner.owner = Some(storage);
    drained(owner, &mut control);
}

#[test]
fn reading_the_record_count_without_the_storage_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let storage = owner.owner.take().unwrap();
    assert_registry_row(owner.reserve_records(&mut control, 1), &control);
    owner.owner = Some(storage);
    drained(owner, &mut control);
}

#[test]
fn a_pending_operation_whose_owner_was_taken_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let mut pending = owner
        .start(&mut control, HtmlConstructionCommand::CreateFragment)
        .unwrap();
    let operation = pending.operation.take();
    assert_registry_row(pending.action(&mut control).map(|_| ()), &control);
    pending.operation = operation;
    owner.abort_pending(&mut control, pending).unwrap();
    drained(owner, &mut control);
}

#[test]
fn aborting_a_pending_operation_whose_owner_was_taken_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let mut pending = owner
        .start(&mut control, HtmlConstructionCommand::CreateFragment)
        .unwrap();
    let operation = pending.operation.take().unwrap();
    assert_registry_row(owner.abort_pending(&mut control, pending), &control);
    // The operation was never consumed, so aborting it returns the storage.
    owner.abort_operation(&mut control, operation).unwrap();
    drained(owner, &mut control);
}

#[test]
fn recovering_a_frozen_arena_without_its_wrapper_ticket_reports_the_registry_row() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let mut frozen = session.freeze(&mut c).unwrap();
    let wrapper = frozen.storage.as_mut().unwrap().construction.wrapper.take();
    assert_registry_row(frozen.release(&mut c), &c);
    // The refusal consumed nothing: restoring the ticket lets the owner finish.
    frozen.storage.as_mut().unwrap().construction.wrapper = wrapper;
    frozen.release(&mut c).unwrap();
    crate::construction::tests::storage_zero(&c);
}

#[test]
fn a_refused_retirement_still_releases_everything_the_session_holds() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    c.refuse_next_construction_retire();
    assert_registry_row(session.release(&mut c), &c);
    crate::construction::tests::storage_zero(&c);
}

#[test]
fn a_refused_recovery_still_releases_everything_the_frozen_arena_holds() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let mut frozen = session.freeze(&mut c).unwrap();
    c.refuse_next_construction_recovery();
    assert_registry_row(frozen.release(&mut c), &c);
    crate::construction::tests::storage_zero(&c);
}

// Characterization: a cleanup walk that meets a refusal keeps going, so every
// other owner is still released or credited, and it reports the first refusal.
// A test hook makes the first step refuse after its work (reported as a wrong
// token) and the second refuse as the registry row, which latches as the first
// stop. The walk must return the first refusal and leave nothing behind.
use crate::parser::control::WalkSite;

#[test]
fn a_refused_bank_credit_still_retires_every_other_bank_and_reports_the_first_refusal() {
    let (mut control, owner, _root, _child) = insertion_pair(crate::construction::tests::limits());
    // One bank could not tell a continued walk from a stopped one.
    let banks = owner.children.iter().flatten().count()
        + usize::from(owner.records.is_some())
        + usize::from(owner.identity.is_some());
    assert!(banks >= 3, "{banks} banks");
    control.refuse_walk_steps(WalkSite::ConstructionBank, 2);
    let result = owner.retire(&mut control);
    assert_eq!(result.err(), Some(SessionError::WrongToken));
    assert_eq!(control.stop(), Some(registry_row()));
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn a_refused_frozen_owner_still_releases_every_other_owner_and_reports_the_first_refusal() {
    use crate::construction::effects::tests::{graph, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let mut frozen = session.freeze(&mut c).unwrap();
    c.refuse_walk_steps(WalkSite::FrozenOwner, 2);
    let result = frozen.release(&mut c);
    assert_eq!(result.err(), Some(SessionError::WrongToken));
    assert_eq!(c.stop(), Some(registry_row()));
    crate::construction::tests::storage_zero(&c);
}

#[test]
fn an_attribute_edit_without_the_storage_reports_the_registry_row() {
    for edit in 0..5 {
        let (mut control, mut owner) = with_element();
        let storage = owner.owner.take().unwrap();
        let mut kept = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        let result = match edit {
            0 => owner.reserve_attributes(&mut control, 0, 1),
            1 => owner
                .append_attribute(&mut control, 0, &mut None, None)
                .map(drop),
            2 => owner
                .replace_attribute(&mut control, 0, 0, &mut None)
                .map(drop),
            3 => owner.remove_attribute(&mut control, 0, 0).map(drop),
            _ => owner
                .plan_attribute_merge(&mut control, 0, &[], &mut kept)
                .map(drop),
        };
        assert_registry_row(result, &control);
        kept.release(&mut control).unwrap();
        owner.owner = Some(storage);
        drained(owner, &mut control);
    }
}

#[test]
fn an_edit_of_an_absent_record_is_a_foreign_node() {
    let (mut control, mut owner) = with_element();
    let result = owner.reserve_attributes(&mut control, 7, 1);
    assert_eq!(result, Err(SessionError::ForeignNode));
    assert_eq!(control.stop(), None);
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

fn attribute(name: &str, value: &str) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
}

fn held(owner: &ConstructionOwner, control: &mut ParseControl) -> Vec<String> {
    match owner.node_at(0).unwrap().kind() {
        surgeist_html::construction::ConstructionNodeKind::Element(element) => element
            .attributes()
            .iter()
            .map(|held| held.name().local_name().to_utf8().unwrap())
            .collect(),
        _ => {
            let _ = control;
            panic!("an element")
        }
    }
}

#[test]
fn an_append_that_the_capacity_limit_refuses_leaves_the_list_and_every_figure_as_it_was() {
    let (mut control, mut owner) = with_element();
    let before = control.usage().clone();
    let held_bytes = before
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current();
    control.test_limit(HtmlParseResource::ManagedCapacityBytes, held_bytes);
    let mut incoming = Some(attribute("a", "v"));
    let result = owner.append_attribute(&mut control, 0, &mut incoming, None);
    assert!(matches!(
        result,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    assert!(incoming.is_some(), "the attribute stays with the caller");
    assert!(held(&owner, &mut control).is_empty());
    assert_eq!(
        control
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current(),
        held_bytes,
        "no capacity was charged for the refused growth"
    );
    assert_eq!(owner.attribute_bytes, 0);
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

#[test]
fn appended_attributes_grow_the_list_geometrically_and_are_released_with_the_owner() {
    let (mut control, mut owner) = with_element();
    for name in ["a", "b", "c", "d", "e"] {
        owner
            .append_attribute(&mut control, 0, &mut Some(attribute(name, "v")), None)
            .unwrap();
    }
    assert_eq!(held(&owner, &mut control), ["a", "b", "c", "d", "e"]);
    let capacity = match owner.node_at(0).unwrap().kind() {
        surgeist_html::construction::ConstructionNodeKind::Element(element) => element
            .attributes()
            .to_ordered()
            .shared_attributes()
            .capacity(),
        _ => unreachable!(),
    };
    // One slot, then doubling: 1, 2, 4, 8 for five attributes.
    assert_eq!(capacity, 8);
    assert_eq!(
        owner.attribute_bytes,
        (capacity * mem::size_of::<Attribute>()) as u64,
        "the owner holds exactly the capacity its lists reserved"
    );
    owner.retire(&mut control).unwrap();
    assert_eq!(control.usage().actual_capacity_bytes, 0);
}

/// The cause values of HTML's construction owner's error, one per variant. A cause added to
/// the owner's enum does not compile in [`disposition`] until it has a row there, and a row
/// here is what this test file then pins.
fn every_cause() -> [HtmlConstructionCause; 30] {
    use HtmlConstructionCause as C;
    [
        C::ForeignBuilder,
        C::ParentCannotHaveChildren,
        C::TemplateContentsRequired,
        C::ShadowHostMustBeElement,
        C::ShadowAlreadyAttached,
        C::UnknownNode,
        C::WrongNodeKind,
        C::RootMove,
        C::AssociationMove,
        C::WrongSibling,
        C::Cycle,
        C::AssociationAlreadyOwned,
        C::InvalidTemplateAssociation,
        C::DuplicateAttribute {
            first: 0,
            duplicate: 1,
        },
        C::AttributeNameMismatch,
        C::AttributeOrderMismatch,
        C::InvalidShadowAssociation,
        C::InvalidContentsReference,
        C::ParentChildConflict,
        C::InvalidOutputRoot,
        C::OperationNotReady,
        C::ForeignAllocation,
        C::CapacityMismatch,
        C::IndexUnavailable,
        C::SharedAllocationAliased,
        C::ReleaseUnderflow,
        C::LimitExceeded {
            resource: HtmlConstructionResource::Nodes,
            limit: 1,
            attempted: 2,
        },
        C::RepresentationExhausted,
        C::AllocationFailed,
        C::Accounting(HtmlAccountingFault::ForeignScope),
    ]
}

/// Every cause of HTML's construction owner's error, with the error the parser reports for it
/// and whether that error latches a stop.
///
/// - Handle, placement and cycle refusals are document-shaped errors the
///   callers classify themselves and latch nothing.
/// - A shared payload that was aliased and a count that cannot be represented
///   keep their own stops.
/// - A ticket, layout or release that disagrees with what was charged is the
///   charged-accounting fault, and an allocation that failed is a reached
///   environment stop, not an invariant.
/// - A failure of the ledger scope stops with its accounting fault.
/// - Every other refusal is the owner declining a request that the parser's
///   sequencing guarantees valid.
#[test]
fn every_owner_cause_reports_what_it_violates() {
    use HtmlConstructionCause as C;
    let invariant = |invariant| SessionError::Stop(HtmlParseStop::InternalInvariant(invariant));
    let mismatch = invariant(HtmlParseInvariant::CapacityMismatch);
    let refusal = invariant(HtmlParseInvariant::ConstructionOwnerRefusal);
    let allocation_failed = SessionError::Stop(HtmlParseStop::AllocationFailed {
        resource: HtmlParseResource::ManagedCapacityBytes,
        layout_bytes: 0,
    });
    let exhausted = SessionError::Stop(HtmlParseStop::RepresentationExhausted {
        resource: HtmlParseResource::ManagedCapacityBytes,
    });
    let limit = C::LimitExceeded {
        resource: HtmlConstructionResource::Nodes,
        limit: 1,
        attempted: 2,
    };
    let duplicate = C::DuplicateAttribute {
        first: 0,
        duplicate: 1,
    };
    let scope_fault = HtmlAccountingFault::ForeignScope;
    let rows = [
        (C::ForeignBuilder, SessionError::ForeignNode, false),
        (C::UnknownNode, SessionError::ForeignNode, false),
        (C::RootMove, SessionError::RootMove, false),
        (C::AssociationMove, SessionError::AssociationMove, false),
        (
            C::ParentCannotHaveChildren,
            SessionError::NotContainer,
            false,
        ),
        (
            C::TemplateContentsRequired,
            SessionError::TemplateContentsRequired,
            false,
        ),
        (C::WrongSibling, SessionError::WrongSibling, false),
        (C::ParentChildConflict, SessionError::WrongSibling, false),
        (C::Cycle, SessionError::Cycle, false),
        (
            C::SharedAllocationAliased,
            invariant(HtmlParseInvariant::SharedAllocationAliased),
            true,
        ),
        (C::RepresentationExhausted, exhausted, true),
        (C::CapacityMismatch, mismatch, true),
        (C::IndexUnavailable, refusal, true),
        (C::ForeignAllocation, mismatch, true),
        (limit, mismatch, true),
        (
            C::ReleaseUnderflow,
            invariant(HtmlParseInvariant::ReleaseUnderflow),
            true,
        ),
        (C::AllocationFailed, allocation_failed, true),
        (
            C::Accounting(scope_fault),
            SessionError::Stop(HtmlParseStop::Accounting(scope_fault)),
            true,
        ),
        (C::ShadowHostMustBeElement, refusal, true),
        (C::ShadowAlreadyAttached, refusal, true),
        (C::WrongNodeKind, refusal, true),
        (C::AssociationAlreadyOwned, refusal, true),
        (C::InvalidTemplateAssociation, refusal, true),
        (duplicate, refusal, true),
        (C::AttributeNameMismatch, refusal, true),
        (C::AttributeOrderMismatch, refusal, true),
        (C::InvalidShadowAssociation, refusal, true),
        (C::InvalidContentsReference, refusal, true),
        (C::InvalidOutputRoot, refusal, true),
        (C::OperationNotReady, refusal, true),
    ];
    assert_eq!(rows.len(), every_cause().len());
    for cause in every_cause() {
        assert_eq!(
            rows.iter().filter(|(row, _, _)| *row == cause).count(),
            1,
            "{cause:?} has one row"
        );
    }
    for (cause, expected, latched) in rows {
        let mut c = crate::construction::tests::control();
        assert_eq!(report(&mut c, disposition(cause)), expected, "{cause:?}");
        let stop = match expected {
            SessionError::Stop(stop) => Some(stop),
            _ => None,
        };
        assert_eq!(c.stop(), stop.filter(|_| latched), "{cause:?}");
    }
}

/// A failure of the ledger scope that the owner reports stops the parse with that accounting
/// fault, as every other ledger fault of the parser does. The parser's bounded storage cannot
/// produce this cause, so the test reports a constructed one.
#[test]
fn an_owner_accounting_cause_stops_with_its_accounting_fault() {
    for fault in [
        HtmlAccountingFault::ForeignScope,
        HtmlAccountingFault::Allocation,
        HtmlAccountingFault::ReleaseUnderflow,
        HtmlAccountingFault::ScopeLimit { limit: 2 },
    ] {
        let mut c = crate::construction::tests::control();
        let reported = report(
            &mut c,
            disposition(HtmlConstructionCause::Accounting(fault)),
        );
        assert_eq!(
            reported,
            SessionError::Stop(HtmlParseStop::Accounting(fault))
        );
        assert_eq!(c.stop(), Some(HtmlParseStop::Accounting(fault)));
    }
}

/// The construction path has no classification of its own: a document-shaped
/// cause is a session error that `session_class` classifies as a fault of the
/// parser, an accounting fault is the class a stage error would carry, and the
/// error returned for a latched stop is the outcome `session_class` assigns to
/// that stop.
#[test]
fn owner_dispositions_agree_with_the_session_classification() {
    for cause in every_cause() {
        let mut c = crate::construction::tests::control();
        let reported = report(&mut c, disposition(cause));
        match disposition(cause) {
            Disposition::Session(error) => {
                assert_eq!(reported, error, "{cause:?}");
                assert!(
                    matches!(session_class(&error), SessionClass::Fault(_)),
                    "{cause:?}"
                );
                assert_eq!(c.stop(), None, "{cause:?}");
            }
            Disposition::Class(class) => {
                assert!(matches!(class, SessionClass::Fault(_)), "{cause:?}");
                let SessionError::Stop(stop) = reported else {
                    panic!("{cause:?} did not latch a stop");
                };
                assert_eq!(session_class(&reported), SessionClass::Outcome(stop));
                assert_eq!(c.stop(), Some(stop), "{cause:?}");
            }
            Disposition::Representation
            | Disposition::AllocationFailed
            | Disposition::Accounting(_) => {
                let SessionError::Stop(stop) = reported else {
                    panic!("{cause:?} did not latch a stop");
                };
                assert_eq!(session_class(&reported), SessionClass::Outcome(stop));
                assert_eq!(c.stop(), Some(stop), "{cause:?}");
            }
        }
    }
}

#[test]
fn a_refused_record_after_a_bank_reserve_leaves_the_bank_retirable_for_what_was_admitted() {
    let mut control = ParseControl::new(
        crate::construction::tests::limits(),
        crate::HtmlParseCancellation::new(),
    );
    let mut owner = ConstructionOwner::new(&mut control).unwrap();
    let held = control
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current;
    control.test_limit(HtmlParseResource::ManagedCapacityBytes, held + 16_000);
    control.overshoot_next_capacity_record(1_000_000);
    let result = owner.apply(
        &mut control,
        HtmlConstructionCommand::CreateFragment,
        &[(HtmlParseResource::ManagedNodes, 1)],
    );
    assert!(matches!(
        result,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedCapacityBytes,
            ..
        }))
    ));
    owner.retire(&mut control).unwrap();
    crate::parser::tree_builder::tests::assert_released(&control);
}
