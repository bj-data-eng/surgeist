//! Impossible registry and custody states are reported as the controller's
//! first stop with every owner handed back, never as a panic. Each test forces
//! the state through crate-private access, asserts the exact typed error, that
//! the stop is latched first, and that the owner's own release drains every
//! resource afterwards.
use super::effects::tests::{graph, named, request};
use super::tests::{body_lexical_value, control, fresh_script_owner, lexical_kind, storage_zero};
use super::*;
use crate::parser::control::Invariant;

fn registry_row() -> ParseStop {
    ParseStop::InternalInvariant(Invariant::RegistryRow)
}

/// The exact typed error, latched as the controller's first stop.
fn assert_registry_row<T>(result: Result<T, SessionError>, c: &ParseControl) {
    assert_eq!(
        result.err(),
        Some(SessionError::Stop(registry_row())),
        "typed registry-row error"
    );
    assert_eq!(c.stop(), Some(registry_row()), "latched as the first stop");
}

fn input_slot(storage: &SessionStorage, node: usize) -> usize {
    storage
        .payloads
        .values
        .iter()
        .position(
            |row| matches!(row, RetainedPayload::Input { node: Some(owner), .. } if *owner == node),
        )
        .expect("input metadata row")
}

fn association_slot(storage: &SessionStorage, node: usize) -> usize {
    storage
        .payloads
        .values
        .iter()
        .position(|row| {
            matches!(row, RetainedPayload::FormAssociation { node: Some(owner), .. } if *owner == node)
        })
        .expect("form association row")
}

/// An input graph; its document record (position 0) is never an element.
fn input_request() -> crate::HtmlDocumentParseRequest {
    request("<body><input type=text value=a>")
}

#[test]
fn installing_an_input_attribute_on_a_record_that_is_not_an_element_reports_the_registry_row() {
    let request = input_request();
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    let slot = input_slot(storage, input);
    let container = storage.element_attribute_container(&mut c, input).unwrap();
    let result = storage.initial_input_attribute(
        &mut c,
        0,
        slot,
        association_slot(storage, input),
        (container, 0),
        InputAttributeInitialization::Installed,
    );
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn resetting_an_input_on_a_record_that_is_not_an_element_reports_the_registry_row() {
    let request = input_request();
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    let slot = input_slot(storage, input);
    assert_registry_row(storage.reset_input(&mut c, 0, slot), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn changing_an_input_type_on_a_record_that_is_not_an_element_reports_the_registry_row() {
    let request = input_request();
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    let slot = input_slot(storage, input);
    let container = storage.element_attribute_container(&mut c, input).unwrap();
    let state = storage.read_input_state(&mut c, slot).unwrap();
    let result = storage.change_input_type(
        &mut c,
        0,
        slot,
        container,
        state,
        crate::HtmlInputType::Hidden,
    );
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn an_input_attribute_hook_past_the_attribute_list_reports_the_registry_row() {
    let request = request("<body><input type=text value=a>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    let slot = input_slot(storage, input);
    let association = association_slot(storage, input);
    let container = storage.element_attribute_container(&mut c, input).unwrap();
    let result = storage.initial_input_attribute(
        &mut c,
        input,
        slot,
        association,
        (container, usize::MAX / 2),
        InputAttributeInitialization::Installed,
    );
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_transfer_from_a_stage_that_was_already_reversed_hands_the_stage_back() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let div = named(&nodes, "div", 0).index;
    let storage = session.active_mut().unwrap();
    let container = storage.element_attribute_container(&mut c, div).unwrap();
    storage.prepare_payload_slot(&mut c).unwrap();
    let (values, allocation) = c.new_vector().unwrap();
    let stage = storage.payloads.values.len();
    storage
        .payloads
        .values
        .push(RetainedPayload::Attributes(Some(
            AttributeStorage::Building {
                values,
                allocation,
                original_len: 0,
                reversed_pairs: 1,
            },
        )));
    // The stage is not the unchanged reverse stage a transfer consumes.
    assert_registry_row(
        storage.begin_attribute_transfer(&mut c, stage, container, 0),
        &c,
    );
    // The stage was handed back, so the owner's release reclaims it.
    assert!(matches!(
        storage.payloads.values.get(stage),
        Some(RetainedPayload::Attributes(Some(
            AttributeStorage::Building { .. }
        )))
    ));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn disposing_a_string_past_the_registry_reports_the_registry_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let past = storage.payloads.values.len();
    assert_registry_row(storage.dispose_aggregate_string(&mut c, past), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn disposing_a_row_that_is_not_a_string_keeps_the_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let div = named(&nodes, "div", 0).index;
    let storage = session.active_mut().unwrap();
    let container = storage.element_attribute_container(&mut c, div).unwrap();
    assert_eq!(
        storage.dispose_aggregate_string(&mut c, container),
        Err(SessionError::WrongToken)
    );
    // The refused row is still the attribute container, not a retired husk.
    assert!(matches!(
        storage.payloads.values.get(container),
        Some(RetainedPayload::Attributes(Some(
            AttributeStorage::Frozen { .. }
        )))
    ));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_text_run_slot_that_was_already_transferred_reports_the_registry_row() {
    let source = crate::HtmlSource::utf8("<a v='A B'>");
    let mut c = control();
    let mut input = Some(body_lexical_value(&source, &mut c));
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    // Take the first run out of its slot behind the collection's back.
    let stolen = runs
        .entries
        .as_mut()
        .unwrap()
        .get_mut(0, &mut c)
        .unwrap()
        .unwrap()
        .take()
        .unwrap();
    stolen.text.release(&mut c).unwrap();
    assert_registry_row(runs.take_next(&mut c), &c);
    runs.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn releasing_a_frozen_arena_without_its_publication_lease_keeps_the_owner() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let mut frozen = session.freeze(&mut c).unwrap();
    let lease = frozen.storage.as_mut().unwrap().lease.take();
    assert_registry_row(frozen.release(&mut c), &c);
    // The refusal consumed nothing: restoring the lease lets the owner finish.
    frozen.storage.as_mut().unwrap().lease = lease;
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_refused_release_after_failed_attribute_validation_keeps_the_shared_attributes() {
    use crate::parser::tokenizer::{states::State, tests::LexerFixture};
    let request = request("<div a=1 b=2>");
    let (mut c, mut session, _proof) = fresh_script_owner(&request);
    let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
    let TokenKind::Tag(mut tag) = lexical_kind(&mut lexer, &mut c) else {
        panic!("start tag")
    };
    // Rename the second attribute to the first's name so validation refuses it.
    let first = copy_formatting_text(&mut c, &tag.attributes.as_slice()[0].name).unwrap();
    let second = tag.attributes.get_mut(1, &mut c).unwrap().unwrap();
    mem::replace(&mut second.name, first)
        .release(&mut c)
        .unwrap();
    // Validation refuses, and the release of the shared attributes refuses too.
    c.refuse_next_shared_release();
    let aliased = ParseStop::InternalInvariant(Invariant::SharedAllocationAliased);
    let result = session.allocate_html_tag(&mut c, &mut Some(tag));
    assert_eq!(result.err(), Some(SessionError::Stop(aliased)));
    assert_eq!(c.stop(), Some(aliased));
    // The refused Arc and ticket stayed with the registry, so release drains.
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
