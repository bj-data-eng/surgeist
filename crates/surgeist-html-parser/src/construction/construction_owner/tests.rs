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
        Err(ParseStop::InternalInvariant(Invariant::CapacityMismatch))
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

// Impossible custody states are reported as the controller's first stop with
// the owner's storage handed back, never as a panic. Each test empties one
// custody slot through crate-private access, asserts the exact typed error,
// that the stop is latched first, and that the owner's own retirement drains
// every resource afterwards.
fn registry_row() -> ParseStop {
    ParseStop::InternalInvariant(Invariant::RegistryRow)
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
            HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Element(HtmlElement::new(
                    QualifiedName::new(Namespace::none(), None, "x"),
                    OrderedAttributes::default(),
                )),
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
    owner
        .restore_aborted(&mut control, operation.abort())
        .unwrap();
    drained(owner, &mut control);
}

#[test]
fn beginning_an_attribute_edit_without_the_storage_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let storage = owner.owner.take().unwrap();
    let result = owner.begin_attributes(&mut control, 0, HtmlConstructionAttributeEditMode::Slots);
    assert_registry_row(result, &control);
    owner.owner = Some(storage);
    drained(owner, &mut control);
}

#[test]
fn beginning_a_second_attribute_edit_reports_the_registry_row_and_keeps_the_first() {
    let (mut control, mut owner) = with_element();
    owner
        .begin_attributes(&mut control, 0, HtmlConstructionAttributeEditMode::Slots)
        .unwrap();
    let result = owner.begin_attributes(&mut control, 0, HtmlConstructionAttributeEditMode::Slots);
    assert_registry_row(result, &control);
    assert!(owner.has_attribute_edit());
    owner.abort_attributes(&mut control).unwrap();
    drained(owner, &mut control);
}

#[test]
fn planning_a_candidate_without_an_attribute_edit_reports_the_registry_row() {
    let (mut control, mut owner) = fresh();
    let result = owner.plan_attribute_candidate(&mut control, 0, &[]);
    assert_registry_row(result, &control);
    drained(owner, &mut control);
}

#[test]
fn losing_the_attribute_edit_during_a_replacement_reports_the_registry_row() {
    use crate::construction::effects::tests::{graph, named, request};
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let div = named(&nodes, "div", 0).index;
    let storage = session.active_mut().unwrap();
    let container = storage.element_attribute_container(&mut c, div).unwrap();
    storage.construction.hide_next_attribute_edit();
    let result = storage.replace_attribute_storage(&mut c, div, container, 0, &mut None, &[]);
    assert_registry_row(result, &c);
    // The edit was never lost for good: giving it back lets the owner end it.
    storage.construction.restore_hidden_attribute_edit();
    storage.construction.abort_attributes(&mut c).unwrap();
    session.release(&mut c).unwrap();
    crate::construction::tests::storage_zero(&c);
}

#[test]
fn losing_the_attribute_edit_during_a_removal_reports_the_registry_row() {
    use crate::construction::effects::tests::{graph, named, request};
    use crate::construction::mutation::ManagedFamily;
    let request = request("<body><dialog open id=e>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let dialog = named(&nodes, "dialog", 0);
    session
        .active_mut()
        .unwrap()
        .construction
        .hide_next_attribute_edit();
    let result = session.remove_managed_attribute(&mut c, &dialog, "open", ManagedFamily::Dialog);
    assert_registry_row(result, &c);
    let storage = session.active_mut().unwrap();
    storage.construction.restore_hidden_attribute_edit();
    storage.construction.abort_attributes(&mut c).unwrap();
    // The element's shared attributes lived in the lost edit's storage, so the
    // row that held them cannot be put back; its release reports the emptied
    // row instead of dropping it silently, and the first stop is unchanged.
    assert_eq!(session.release(&mut c), Err(SessionError::Consumed));
    assert_eq!(c.stop(), Some(registry_row()));
}

#[test]
fn ending_an_attribute_edit_without_its_plan_ticket_reports_the_registry_row() {
    let (mut control, mut owner) = with_element();
    owner
        .begin_attributes(
            &mut control,
            0,
            HtmlConstructionAttributeEditMode::NamedMerge { candidate_count: 1 },
        )
        .unwrap();
    // The plan bank is real and empty; only its ticket goes missing.
    let ticket = owner.attribute_plan.take().unwrap();
    assert_eq!(ticket.snapshot().payload_bytes(), 0);
    assert_registry_row(owner.abort_attributes(&mut control), &control);
    drop(ticket);
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
fn a_refused_witness_retirement_still_retires_the_rest_of_the_edit_and_reports_the_first_refusal() {
    use crate::construction::tests::{control, lexical_kind, storage_zero};
    use crate::parser::tokenizer::{states::State, tests::LexerFixture, token::TokenKind};
    let source = crate::HtmlSource::utf8("<html id=x><html a=y b=z>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, State::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut lexer, &mut c) else {
        panic!("tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut lexer, &mut c) else {
        panic!("tag")
    };
    let mut incoming = Some(second);
    // Two new names give the edit two witnesses to retire.
    c.refuse_walk_steps(WalkSite::AttributeWitness, 2);
    let result = session.add_attributes_if_missing(&mut c, &root, &mut incoming);
    assert_eq!(result, Err(SessionError::WrongToken));
    assert_eq!(c.stop(), Some(registry_row()));
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
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
fn putting_an_attribute_edit_back_into_an_occupied_slot_reports_the_registry_row() {
    let (mut control, mut owner) = with_element();
    owner
        .begin_attributes(&mut control, 0, HtmlConstructionAttributeEditMode::Slots)
        .unwrap();
    let lent = owner.take_attribute_edit(&mut control).unwrap();
    // A second edit occupies the slot while the first is lent out. Beginning
    // one through the owner is refused, so another owner supplies it.
    let mut other = ConstructionOwner::new(&mut control).unwrap();
    other
        .apply(
            &mut control,
            HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Element(HtmlElement::new(
                    QualifiedName::new(Namespace::none(), None, "y"),
                    OrderedAttributes::default(),
                )),
            },
            &[(HtmlParseResource::ManagedNodes, 1)],
        )
        .unwrap();
    other
        .begin_attributes(&mut control, 0, HtmlConstructionAttributeEditMode::Slots)
        .unwrap();
    let occupant = other.take_attribute_edit(&mut control).unwrap();
    owner.attribute_edit = Some(occupant);
    owner.put_attribute_edit(lent);
    assert_eq!(
        control.stop(),
        Some(registry_row()),
        "latched as the first stop"
    );
    // The occupant was not displaced; giving it back lets both owners finish.
    let occupant = owner.attribute_edit.take().unwrap();
    other.attribute_edit = Some(occupant);
    other.abort_attributes(&mut control).unwrap();
    other.retire(&mut control).unwrap();
    drained(owner, &mut control);
}
