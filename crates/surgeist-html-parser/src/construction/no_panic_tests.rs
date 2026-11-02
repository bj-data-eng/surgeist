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

/// The first processing-instruction record of a graph.
fn pi_node(storage: &SessionStorage) -> usize {
    (0..)
        .map_while(|position| storage.construction.node_at(position))
        .find(|record| matches!(record.kind(), HtmlNodeKind::ProcessingInstruction(_)))
        .expect("processing instruction record")
        .position()
}

#[test]
fn a_meta_csp_state_slot_past_the_registry_reports_the_registry_row_for_the_base_query() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    storage.csp_slot = Some(storage.payloads.values.len());
    assert_registry_row(storage.csp_base_current(&mut c, 0), &c);
    storage.csp_slot = None;
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_meta_csp_state_slot_naming_another_row_kind_reports_the_registry_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert!(!matches!(
        storage.payloads.values.first(),
        Some(RetainedPayload::CspState(_))
    ));
    storage.csp_slot = Some(0);
    assert_registry_row(storage.ensure_csp_policy(&mut c, 0), &c);
    storage.csp_slot = None;
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_document_naming_another_row_kind_as_its_meta_csp_policy_reports_the_registry_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let document = storage.document_slot(&mut c, 0).unwrap();
    storage.meta_history_mut(document).unwrap().csp = Some(document);
    assert_registry_row(storage.csp_policy(&mut c, 0), &c);
    storage.meta_history_mut(document).unwrap().csp = None;
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn an_initial_details_attribute_past_the_attribute_list_reports_the_registry_row() {
    let request = request("<body><details open>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let details = named(&nodes, "details", 0).index;
    let storage = session.active_mut().unwrap();
    assert_registry_row(
        storage.initial_details_attribute(&mut c, details, usize::MAX / 2),
        &c,
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn parsing_a_processing_instruction_into_a_map_past_the_registry_reports_the_registry_row() {
    let request = request("<body><?x a=\"b\"?>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let pi = pi_node(storage);
    let past = storage.payloads.values.len();
    assert_registry_row(storage.parse_pi_map(&mut c, pi, past), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn finishing_a_processing_instruction_without_pairs_into_a_map_past_the_registry_reports_the_registry_row()
 {
    let request = request("<body><?x?>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let pi = pi_node(storage);
    let past = storage.payloads.values.len();
    assert_registry_row(storage.parse_pi_map(&mut c, pi, past), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn binding_a_processing_instruction_map_to_a_row_that_is_not_one_reports_the_registry_row() {
    let request = request("<body><?x a=\"b\"?>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let pi = pi_node(storage);
    let past = storage.payloads.values.len();
    assert_registry_row(storage.bind_pi_map(&mut c, past, pi), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn binding_a_processing_instruction_map_to_another_row_kind_reports_the_registry_row() {
    let request = request("<body><?x a=\"b\"?>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let pi = pi_node(storage);
    assert!(!matches!(
        storage.payloads.values.first(),
        Some(RetainedPayload::PiAttributes { .. })
    ));
    assert_registry_row(storage.bind_pi_map(&mut c, 0, pi), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn binding_an_svg_script_slot_to_a_row_that_is_not_one_reports_the_registry_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let div = named(&nodes, "div", 0).index;
    let storage = session.active_mut().unwrap();
    let past = storage.payloads.values.len();
    assert_registry_row(storage.bind_svg_slot(&mut c, past, div), &c);
    assert_registry_row(storage.bind_svg_slot(&mut c, 0, div), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn releasing_a_work_list_past_the_registry_reports_the_registry_row() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert_registry_row(storage.release_work_list(&mut c, usize::MAX / 2), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn releasing_a_work_list_on_another_row_kind_leaves_that_row_in_place() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert!(!matches!(
        storage.payloads.values.first(),
        Some(RetainedPayload::WorkList(_))
    ));
    let before = mem::discriminant(&storage.payloads.values[0]);
    assert_eq!(
        storage.release_work_list(&mut c, 0).err(),
        Some(SessionError::WrongToken)
    );
    assert_eq!(mem::discriminant(&storage.payloads.values[0]), before);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn field_group_lookups_past_the_registry_report_the_registry_row() {
    let past = usize::MAX / 2;
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert_registry_row(storage.copied_slot_group(&mut c, past), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);

    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert_registry_row(storage.reset_group(&mut c, past, None, 0), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);

    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert_registry_row(storage.retire_field_group(&mut c, past), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);

    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    assert_registry_row(storage.merge_field_groups(&mut c, past, 0, 0, 0), &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn an_id_edit_over_a_row_that_is_not_a_staged_attribute_list_reports_the_registry_row() {
    let request = input_request();
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    // The stage names the input's own metadata row, which holds no attributes.
    let stage = input_slot(storage, input);
    let result = storage.prepare_id_attribute_effects(&mut c, input, stage);
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_textarea_without_its_state_row_reports_the_registry_row_when_it_leaves_the_stack() {
    let request = request("<body><textarea>x</textarea>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let textarea = named(&nodes, "textarea", 0).clone();
    let storage = session.active_mut().unwrap();
    // Private-state fault injection: the node's join to its state row is lost.
    storage.payloads.node_joins[textarea.index].textarea = None;
    let result = session.parser_stack_removed(&mut c, &textarea, None);
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_textarea_whose_state_join_names_another_row_kind_reports_the_registry_row() {
    let request = request("<body><textarea>x</textarea><input type=text>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let textarea = named(&nodes, "textarea", 0).clone();
    let input = named(&nodes, "input", 0).index;
    let storage = session.active_mut().unwrap();
    // Private-state fault injection: the join names an input's metadata row.
    let other = input_slot(storage, input);
    storage.payloads.node_joins[textarea.index].textarea = Some(joined_cell(other).unwrap());
    let result = session.parser_stack_removed(&mut c, &textarea, None);
    assert_registry_row(result, &c);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_textarea_refresh_over_an_aliased_raw_value_reports_the_aliased_allocation() {
    let request = request("<body><textarea>x</textarea>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let textarea = named(&nodes, "textarea", 0).clone();
    // A second owner of the raw value makes it impossible to replace in place.
    let held = std::sync::Arc::clone(
        session
            .textarea_raw_value(&mut c, &textarea)
            .unwrap()
            .shared_code_units(),
    );
    let result = session.parser_stack_removed(&mut c, &textarea, None);
    let aliased = ParseStop::InternalInvariant(Invariant::SharedAllocationAliased);
    assert_eq!(result.err(), Some(SessionError::Stop(aliased)));
    assert_eq!(c.stop(), Some(aliased));
    drop(held);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// The mapping every effect-work catch-all shares: an operation that is still
/// owed names pending effects, a uniqueness check that failed names the aliased
/// allocation, a row that is missing, of another kind or unjoined is a registry
/// row, and the outcomes the driver classifies itself pass through unchanged
/// without a stop.
#[test]
fn effect_work_errors_map_to_their_precise_invariant() {
    let pending = ParseStop::InternalInvariant(Invariant::PendingFormEffects);
    let aliased = ParseStop::InternalInvariant(Invariant::SharedAllocationAliased);
    for (error, expected) in [
        (SessionError::PendingIntrinsicWork, pending),
        (SessionError::AliasedPayload, aliased),
        (SessionError::WrongToken, registry_row()),
        (SessionError::ForeignNode, registry_row()),
        (SessionError::Consumed, registry_row()),
        (SessionError::TextareaMetadataUnavailable, registry_row()),
        (SessionError::NodeDocumentUnavailable, registry_row()),
        (SessionError::SelectMetadataUnavailable, registry_row()),
        (SessionError::NonceMetadataUnavailable, registry_row()),
    ] {
        let mut c = control();
        assert_eq!(
            latch_effect_fault(&mut c, error),
            SessionError::Stop(expected),
            "{error:?}"
        );
        assert_eq!(c.stop(), Some(expected), "{error:?}");
    }
    for error in [
        SessionError::CspService,
        SessionError::AssociatedFacts,
        SessionError::ScriptMetadataUnavailable,
        SessionError::SvgScriptMetadataUnavailable,
        SessionError::CompletionMetadataUnavailable,
        SessionError::CompletionIncomplete,
        SessionError::ContextReactionUnavailable,
    ] {
        let mut c = control();
        assert_eq!(latch_effect_fault(&mut c, error), error, "{error:?}");
        assert_eq!(c.stop(), None, "{error:?}");
    }
}
