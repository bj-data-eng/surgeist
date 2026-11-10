use super::*;
use surgeist_html::{
    construction::{HtmlElement, HtmlNodeKind},
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

fn radio_pair(
    request: &crate::HtmlDocumentParseRequest,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    ParserNode,
    ParserNode,
) {
    let (mut c, mut session, proof, mut lexer, parent, mut input, source) = input_owner(request);
    let first = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &source,
            &proof,
        )
        .unwrap();
    session
        .insert_parser(&mut c, &parent, &first, None, &proof)
        .unwrap();
    let (mut second, range) = loop {
        match lexer.next(&mut c, false).unwrap().unwrap() {
            crate::parser::tokenizer::token::Event::Token(event) => {
                let token = event.into_token(&mut c).unwrap();
                if let TokenKind::Tag(tag) = token.kind {
                    break (Some(tag), token.source);
                }
                release_token_kind(&mut Some(token.kind), &mut c).unwrap();
            }
            event => event.release(&mut c).unwrap(),
        }
    };
    let second = session
        .allocate_input_for_creation(
            &mut c,
            &mut second,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &proof,
        )
        .unwrap();
    session
        .insert_parser(&mut c, &parent, &second, None, &proof)
        .unwrap();
    lexer.release(&mut c).unwrap();
    (c, session, parent, first, second)
}
#[test]
fn radio_exact_group_names_and_ordered_attributes_preserve_checked_defaults() {
    for (left, right, expected) in [
        (
            "type=radio name=g checked",
            "checked name=g type=radio",
            [false, true],
        ),
        (
            "checked type=radio name=g",
            "name=g type=radio checked",
            [false, true],
        ),
        (
            "type=radio name=g checked",
            "type=radio name=G checked",
            [true, true],
        ),
        (
            "type=radio name='' checked",
            "type=radio name='' checked",
            [true, true],
        ),
        ("type=radio checked", "type=radio checked", [true, true]),
        (
            "type=radio name=' g' checked",
            "type=radio name=g checked",
            [true, true],
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head></head><input {left}><input {right}>")),
            limits(),
        );
        let (mut c, mut session, _, a, b) = radio_pair(&request);
        let states = [
            session.input_state(&mut c, &a).unwrap(),
            session.input_state(&mut c, &b).unwrap(),
        ];
        assert_eq!(states.map(|s| s.flags.checkedness()), expected);
        assert!(
            states
                .iter()
                .all(|s| !s.flags.dirty_checkedness() && !s.flags.dirty_value())
        );
        for node in [&a, &b] {
            assert!(
                session
                    .active()
                    .unwrap()
                    .form_attribute(&mut c, node.index, "checked")
                    .unwrap()
                    .is_some()
            );
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn radio_checked_setter_dirties_only_subject_and_does_not_recheck_false_subject() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked><input type=radio name=g checked>",
        ),
        limits(),
    );
    let (mut c, mut session, _, a, b) = radio_pair(&request);
    session.set_input_checked(&mut c, &a, true).unwrap();
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    let peer = session.input_state(&mut c, &b).unwrap().flags;
    assert!(!peer.checkedness() && !peer.dirty_checkedness());
    session.set_input_checked(&mut c, &a, false).unwrap();
    assert!(!session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    assert!(
        session
            .input_state(&mut c, &a)
            .unwrap()
            .flags
            .dirty_checkedness()
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_clone_propagates_current_dirty_state_then_connection_normalizes_actual_group() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked value=x><input type=radio name=g checked>",
        ),
        limits(),
    );
    let (mut c, mut session, parent, a, b) = radio_pair(&request);
    session.set_input_checked(&mut c, &a, true).unwrap();
    let copy = session
        .active_mut()
        .unwrap()
        .clone_node(&mut c, a.index, DocumentRef::Owned(0))
        .unwrap();
    let copy = session.active().unwrap().node(&mut c, copy).unwrap();
    let copied = session.input_state(&mut c, &copy).unwrap().flags;
    assert!(copied.checkedness() && copied.dirty_checkedness());
    session.insert(&mut c, &parent, &copy, None, None).unwrap();
    assert!(!session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    assert!(
        session
            .input_state(&mut c, &copy)
            .unwrap()
            .flags
            .checkedness()
    );
    assert!(
        session
            .input_state(&mut c, &a)
            .unwrap()
            .flags
            .dirty_checkedness()
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_work_cuts_retain_progress_first_stop_and_reclaim_every_owned_field() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked><input type=radio name=g checked>",
        ),
        limits(),
    );
    let (mut baseline, mut session, _, a, _) = radio_pair(&request);
    let before = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session.set_input_checked(&mut baseline, &a, true).unwrap();
    let work = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    session.release(&mut baseline).unwrap();
    storage_zero(&baseline);
    for short in [0, 1, work / 2] {
        let (mut c, mut session, _, a, b) = radio_pair(&request);
        let limit = c.usage().resource(HtmlParseResource::WorkCredits).current + work - short;
        let remaining = request.limits().work_credits() - limit;
        c.charge(HtmlParseWorkCategory::StateStep, remaining)
            .unwrap();
        let result = session.set_input_checked(&mut c, &a, true);
        if short == 0 {
            result.unwrap();
            assert!(session.active().unwrap().payloads.values.iter().any(|p|
                matches!(p,RetainedPayload::Input {node:Some(n),state,..} if *n==a.index && state.flags.checkedness())));
            assert!(session.active().unwrap().effects.radio.is_none());
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        } else {
            assert!(matches!(result, Err(SessionError::Stop(_))));
            let stop = c.stop();
            assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(_))));
            assert_eq!(c.stop(), stop);
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(b.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(a.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position())
            );
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn radio_cancelled_and_foreign_checked_setters_preserve_actual_state_and_owner() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked><input type=radio name=g checked>",
        ),
        limits(),
    );
    let (mut c, mut session, _, a, _) = radio_pair(&request);
    let before = session.input_state(&mut c, &a).unwrap();
    let other = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let foreign = other.root(&mut c).unwrap();
    assert_eq!(
        session.set_input_checked(&mut c, &foreign, true),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(session.input_state(&mut c, &a).unwrap(), before);
    let mut other = other;
    other.release(&mut c).unwrap();
    request.cancellation().cancel();
    assert_eq!(
        session.set_input_checked(&mut c, &a, true),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(session.active().unwrap().payloads.values.iter().any(|p|
        matches!(p,RetainedPayload::Input {node:Some(n),state,..} if *n==a.index && *state==before)));
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

fn radio_managed_fixture<'a>(
    request: &'a crate::HtmlDocumentParseRequest,
) -> (
    ParseControl,
    ParserArenaSession,
    crate::parser::tree_builder::ConstructionCapability<'a>,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
    ParserNode,
    ParserNode,
) {
    let (mut c, mut session, proof, mut lexer, parent, mut input, range) = input_owner(request);
    let first = session
        .allocate_input_for_creation(
            &mut c,
            &mut input,
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &proof,
        )
        .unwrap();
    session
        .insert_parser(&mut c, &parent, &first, None, &proof)
        .unwrap();
    let (tag, range) = radio_next_tag(&mut lexer, &mut c);
    let second = session
        .allocate_input_for_creation(
            &mut c,
            &mut Some(tag),
            CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: CreationContext::Ordinary,
            },
            &range,
            &proof,
        )
        .unwrap();
    session
        .insert_parser(&mut c, &parent, &second, None, &proof)
        .unwrap();
    (c, session, proof, lexer, first, second)
}
#[test]
fn radio_name_change_uses_actual_managed_fields_and_retirement_then_removal_early_return() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=a checked><input type=radio name=b checked><i name=b>",
        ),
        limits(),
    );
    let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
    let mut field = radio_next_attribute(&mut lexer, &mut c);
    session
        .set_input_attribute(&mut c, &a, &mut field, &proof)
        .unwrap();
    assert!(field.is_none());
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    let storage = session.active().unwrap();
    assert!(storage.payloads.values.iter().any(|p|matches!(p,
        RetainedPayload::String {value,location:StringLocation::Detached,origins:Some(group),..}
        if value.code_units()==[97] && storage.evidence.as_ref().unwrap().require_group(group).unwrap().retirement.is_some())));
    session
        .remove_input_attribute(&mut c, &a, radio::InputGroupAttribute::Name)
        .unwrap();
    session.set_input_checked(&mut c, &b, true).unwrap();
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_checked_added_removed_dirty_gate_and_existing_attribute_replacement_are_distinct() {
    for dirty in [false, true] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=radio name=g><input type=radio name=g checked><i checked=new><i checked=again>",
            ),
            limits(),
        );
        let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
        if dirty {
            session.set_input_checked(&mut c, &a, false).unwrap();
        }
        let mut field = radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &a, &mut field, &proof)
            .unwrap();
        assert_eq!(
            session.input_state(&mut c, &a).unwrap().flags.checkedness(),
            !dirty
        );
        assert_eq!(
            session.input_state(&mut c, &b).unwrap().flags.checkedness(),
            dirty
        );
        session.set_input_checked(&mut c, &b, true).unwrap();
        let mut field = radio_next_attribute(&mut lexer, &mut c);
        session
            .set_input_attribute(&mut c, &a, &mut field, &proof)
            .unwrap();
        // Replacement of present checked is neither Added nor Removed.
        assert!(!session.input_state(&mut c, &a).unwrap().flags.checkedness());
        assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
        session
            .remove_input_attribute(&mut c, &a, radio::InputGroupAttribute::Checked)
            .unwrap();
        assert!(!session.input_state(&mut c, &a).unwrap().flags.checkedness());
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn radio_managed_alias_refusal_preserves_original_then_retry_reclaims_all_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=a checked><input type=radio name=b checked><i name=b>",
        ),
        limits(),
    );
    let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
    let mut field = radio_next_attribute(&mut lexer, &mut c);
    let pointer = field
        .as_ref()
        .unwrap()
        .value
        .value
        .text()
        .code_units()
        .as_ptr();
    let HtmlNodeKind::Element(e) = &session
        .active()
        .unwrap()
        .construction
        .node_at(a.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("input")
    };
    let alias = e.attributes().clone();
    assert_eq!(
        session.set_input_attribute(&mut c, &a, &mut field, &proof),
        Err(SessionError::AliasedPayload)
    );
    assert_eq!(
        field
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .code_units()
            .as_ptr(),
        pointer
    );
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
    drop(alias);
    session
        .set_input_attribute(&mut c, &a, &mut field, &proof)
        .unwrap();
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_actual_owner_change_null_then_final_owner_and_disconnected_setter_use_virtual_tree() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked><input type=radio name=g checked><form>",
        ),
        limits(),
    );
    let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
    let root = session.root(&mut c).unwrap();
    let (tag, _) = radio_next_tag(&mut lexer, &mut c);
    let form = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
    session
        .insert(&mut c, &root, &form, None, Some(&proof))
        .unwrap();
    session.set_input_checked(&mut c, &a, true).unwrap();
    session
        .insert(&mut c, &form, &a, None, Some(&proof))
        .unwrap();
    assert!(
        session
            .form_association_owner(&mut c, &a)
            .unwrap()
            .is_some()
    );
    session.set_input_checked(&mut c, &b, true).unwrap();
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
    session
        .insert(&mut c, &root, &a, None, Some(&proof))
        .unwrap();
    assert!(
        session
            .form_association_owner(&mut c, &a)
            .unwrap()
            .is_none()
    );
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    // No connectedness requirement belongs to checked-setter group membership.
    session.remove(&mut c, &form, Some(&proof)).unwrap();
    session
        .insert(&mut c, &form, &a, None, Some(&proof))
        .unwrap();
    session
        .insert(&mut c, &form, &b, None, Some(&proof))
        .unwrap();
    session.set_input_checked(&mut c, &b, true).unwrap();
    session.set_input_checked(&mut c, &a, true).unwrap();
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_committed_peer_cancellation_retains_real_progress_and_canonical_first_stop() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=g checked><input type=radio name=g checked>",
        ),
        limits(),
    );
    let (mut c, mut session, _, a, b) = radio_pair(&request);
    session.active_mut().unwrap().cancel_after_radio_peer = Some(request.cancellation().clone());
    assert_eq!(
        session.set_input_checked(&mut c, &a, true),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert!(storage.effects.radio.is_some());
    let peer = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::Input {
                node: Some(n),
                state,
                ..
            } if *n == b.index => Some(state),
            _ => None,
        })
        .unwrap();
    assert!(!peer.flags.checkedness());
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let usage = c.usage().resource(HtmlParseResource::WorkCredits);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), usage);
}
#[test]
fn radio_actual_type_signal_from_checked_checkbox_normalizes_and_same_radio_state_skips_signal() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=checkbox name=g checked><input type=radio name=g checked><i type=radio><i type=RADIO>",
        ),
        limits(),
    );
    let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
    let mut attr = radio_next_attribute(&mut lexer, &mut c);
    session
        .set_input_attribute(&mut c, &a, &mut attr, &proof)
        .unwrap();
    assert_eq!(
        session.input_state(&mut c, &a).unwrap().flags.input_type(),
        surgeist_html::attributes::HtmlInputType::Radio
    );
    assert!(session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(!session.input_state(&mut c, &b).unwrap().flags.checkedness());
    session.set_input_checked(&mut c, &b, true).unwrap();
    let mut attr = radio_next_attribute(&mut lexer, &mut c);
    session
        .set_input_attribute(&mut c, &a, &mut attr, &proof)
        .unwrap();
    assert!(!session.input_state(&mut c, &a).unwrap().flags.checkedness());
    assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn radio_matching_imported_peer_requires_actual_canonical_state_but_unrelated_name_does_not() {
    for name in ["g", "other"] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(
                "<html><head></head><input type=radio name=g checked><input type=radio name=other>",
            ),
            limits(),
        );
        let (mut c, mut session, parent, a, _) = radio_pair(&request);
        let native = HtmlElement::new(
            QualifiedName::new(Namespace::html(), None, "input"),
            OrderedAttributes::new(vec![
                Attribute::new(QualifiedName::new(Namespace::none(), None, "type"), "radio"),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "name"), name),
            ])
            .unwrap(),
        );
        let imported = session.allocate_element(&mut c, native).unwrap();
        session
            .insert(&mut c, &parent, &imported, None, None)
            .unwrap();
        let result = session.set_input_checked(&mut c, &a, true);
        if name == "g" {
            let stop = c.stop().unwrap();
            assert_eq!(
                stop,
                HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance)
            );
            assert_eq!(result, Err(SessionError::Stop(stop)));
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            assert!(session.active().unwrap().effects.radio.is_some());
        } else {
            result.unwrap();
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn radio_literal_utf16_names_preserve_unpaired_surrogates_and_exact_group_case() {
    for equal in [false, true] {
        let mut units = "<html><head></head><input type=radio name='"
            .encode_utf16()
            .collect::<Vec<_>>();
        units.extend([0xd800, 0xd83d, 0xde00]);
        units.extend("' checked><input type=radio name='".encode_utf16());
        units.extend([if equal { 0xd800 } else { 0xd801 }, 0xd83d, 0xde00]);
        units.extend("' checked>".encode_utf16());
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
            limits(),
        );
        let (mut c, mut session, _, a, b) = radio_pair(&request);
        assert_eq!(
            session.input_state(&mut c, &a).unwrap().flags.checkedness(),
            !equal
        );
        assert!(session.input_state(&mut c, &b).unwrap().flags.checkedness());
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn radio_name_setter_work_boundaries_keep_original_or_committed_registry_owner_and_first_stop() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(
            "<html><head></head><input type=radio name=a checked><input type=radio name=b checked><i name=b>",
        ),
        limits(),
    );
    let (mut base, mut session, proof, mut lexer, a, _) = radio_managed_fixture(&request);
    let mut attr = radio_next_attribute(&mut lexer, &mut base);
    let before = base
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .set_input_attribute(&mut base, &a, &mut attr, &proof)
        .unwrap();
    let work = base
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    lexer.release(&mut base).unwrap();
    session.release(&mut base).unwrap();
    storage_zero(&base);
    for admitted in [0, 1, work / 2, work - 1, work] {
        let (mut c, mut session, proof, mut lexer, a, b) = radio_managed_fixture(&request);
        let mut attr = radio_next_attribute(&mut lexer, &mut c);
        let pointer = attr
            .as_ref()
            .unwrap()
            .value
            .value
            .text()
            .code_units()
            .as_ptr();
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(HtmlParseWorkCategory::StateStep, remaining - admitted)
            .unwrap();
        let result = session.set_input_attribute(&mut c, &a, &mut attr, &proof);
        if admitted == work {
            result.unwrap();
            assert!(session.active().unwrap().payloads.values.iter().any(|p|matches!(p,
                RetainedPayload::Input {node:Some(n),state,..} if *n==b.index && !state.flags.checkedness())));
            c.charge(HtmlParseWorkCategory::StateStep, 1).unwrap_err();
        } else {
            assert!(matches!(result, Err(SessionError::Stop(_))));
            let first = c.stop().unwrap();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            if let Some(attr) = &attr {
                assert_eq!(attr.value.value.text().code_units().as_ptr(), pointer);
            }
        }
        if let Some(attr) = attr.take() {
            attr.name.release(&mut c).unwrap();
            attr.value.release(&mut c).unwrap();
        }
        let first = c.stop();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), first);
    }
}
