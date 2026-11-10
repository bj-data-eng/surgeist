use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    elements::form::HtmlNativeFormAssociationKind,
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

fn assert_listed_owner(
    session: &ParserArenaSession,
    c: &mut ParseControl,
    node: &ParserNode,
    kind: HtmlNativeFormAssociationKind,
    owner: Option<&ParserNode>,
    parser_inserted: bool,
) {
    let state = session.form_association_state(c, node).unwrap();
    assert_eq!(state.kind, kind);
    assert_eq!(state.parser_inserted, parser_inserted);
    match (session.form_association_owner(c, node).unwrap(), owner) {
        (Some(actual), Some(expected)) => {
            assert!(session.same_node(c, &actual, expected).unwrap())
        }
        (None, None) => {}
        _ => panic!("wrong actual listed form owner"),
    }
}

#[test]
fn ordinary_creation_does_not_issue_input_select_or_details_profile_metadata() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><input><select><details>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    for _ in 0..3 {
        let node = form_creation(&mut session, &mut lexer, &mut c, &form, Some(&form));
        assert_eq!(
            session.form_association_state(&mut c, &node),
            Err(SessionError::FormAssociationMetadataUnavailable)
        );
        assert_eq!(
            session.input_state(&mut c, &node),
            Err(SessionError::InputMetadataUnavailable)
        );
        assert!(matches!(
            session.select_state(&mut c, &node),
            Err(SessionError::SelectMetadataUnavailable)
        ));
        assert!(matches!(
            session.active().unwrap().details_domain(&mut c, node.index),
            Ok(details::NotificationDomain::Unavailable)
        ));
    }
    assert!(c.stop().is_none());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_creation_suppresses_pointer_for_present_empty_form_and_retains_qualified_kinds() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form id=f><button><fieldset form=''><object form=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        true,
    );
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let fieldset = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
    assert_listed_owner(
        &session,
        &mut c,
        &fieldset,
        HtmlNativeFormAssociationKind::Fieldset,
        None,
        false,
    );
    session
        .insert(&mut c, &body, &fieldset, None, None)
        .unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &fieldset,
        HtmlNativeFormAssociationKind::Fieldset,
        None,
        false,
    );
    let object = form_creation(&mut session, &mut lexer, &mut c, &body, Some(&form));
    assert_listed_owner(
        &session,
        &mut c,
        &object,
        HtmlNativeFormAssociationKind::Object,
        None,
        false,
    );
    session.insert(&mut c, &body, &object, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &object,
        HtmlNativeFormAssociationKind::Object,
        Some(&form),
        false,
    );
    assert_eq!(
        session.image_form_state(&mut c, &button),
        Err(SessionError::ImageFormMetadataUnavailable)
    );
    lexer.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().next().unwrap();
    let id = document.node(&body).unwrap().children().nth(3).unwrap();
    assert_eq!(
        frozen.form_association_state(&mut c, &id).unwrap().kind,
        HtmlNativeFormAssociationKind::Object
    );
    assert!(
        frozen
            .form_association_owner(&mut c, &id)
            .unwrap()
            .is_some()
    );
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_first_id_tree_order_blocks_later_form_and_id_only_removal_reassociates() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button form=f><div id=f><form id=f><form id=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let blocker = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &blocker, None, None).unwrap();
    let first = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &first, None, None).unwrap();
    let second = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &second, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    session.remove(&mut c, &blocker, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&first),
        false,
    );
    session.remove(&mut c, &first, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&second),
        false,
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_removal_id_event_resets_detached_same_document_recipient_after_local_preservation() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<div><button form=f><form id=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let group = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &group, None, None).unwrap();
    let button = form_creation(&mut session, &mut lexer, &mut c, &group, None);
    session.insert(&mut c, &group, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, &mut c, &group, None);
    session.insert(&mut c, &group, &form, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    session.remove(&mut c, &group, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(button.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(group.index)
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(form.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(group.index)
    );
    // An actual hook inside an already detached tree still runs the global
    // reset. A parentless removal afterwards has no hook and no phase.
    let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.remove(&mut c, &form, None).unwrap();
    let hook = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
    let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.remove(&mut c, &form, None).unwrap();
    let noop = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
    assert!(hook > noop);
    assert!(session.active().unwrap().effects.form.is_none());
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_connected_explicit_form_has_no_ancestor_fallback_but_detached_reset_does() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form id=f><fieldset form=missing><object form=''>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    let fieldset = form_creation(&mut session, &mut lexer, &mut c, &form, Some(&form));
    session
        .insert(&mut c, &form, &fieldset, None, None)
        .unwrap();
    let object = form_creation(&mut session, &mut lexer, &mut c, &form, Some(&form));
    session.insert(&mut c, &form, &object, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &fieldset,
        HtmlNativeFormAssociationKind::Fieldset,
        None,
        false,
    );
    assert_listed_owner(
        &session,
        &mut c,
        &object,
        HtmlNativeFormAssociationKind::Object,
        None,
        false,
    );
    session.remove(&mut c, &form, None).unwrap();
    // ID removal globally resets every explicit-form recipient in this
    // originating document, each against its real detached ordinary tree.
    assert_listed_owner(
        &session,
        &mut c,
        &fieldset,
        HtmlNativeFormAssociationKind::Fieldset,
        Some(&form),
        false,
    );
    assert_listed_owner(
        &session,
        &mut c,
        &object,
        HtmlNativeFormAssociationKind::Object,
        Some(&form),
        false,
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_late_body_id_merge_resets_exact_target_before_next_attribute_and_freezes() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=f title=after>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
        panic!("body")
    };
    let mut input = Some(tag);
    session
        .add_body_attributes_if_missing(&mut c, &body, &mut input)
        .unwrap();
    assert!(input.is_none());
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    assert!(session.active().unwrap().effects.form.is_none());
    let ConstructionNodeKind::Element(body_record) = &session
        .active()
        .unwrap()
        .construction
        .node_at(body.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("body")
    };
    assert_eq!(
        body_record
            .attributes()
            .iter()
            .map(|a| a.name().local_name().code_units().to_vec())
            .collect::<Vec<_>>(),
        vec![vec![105, 100], vec![116, 105, 116, 108, 101]]
    );
    lexer.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().next().unwrap();
    let id = document.node(&body).unwrap().children().next().unwrap();
    assert_eq!(frozen.form_association_owner(&mut c, &id).unwrap(), None);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_metadata_cap_rejects_all_three_kinds_before_any_lexical_take() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for text in ["<button>", "<fieldset>", "<object>"] {
        let source = crate::HtmlSource::utf8(text);
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .auxiliary_entries(22)
                .active_formatting_entries(22)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("tag")
        };
        // Lexical pending events precede the final retained allocation.
        // Body stays detached: no insertion, so no child-order growth is admitted.
        let _body = session.allocate_implicit_body(&mut c, None).unwrap();
        let mut input = Some(tag);
        let original = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .to_vec();
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &html,
            None,
            crate::construction::CreationContext::Ordinary,
        );
        let stop = c.stop().unwrap();
        assert_eq!(result.err(), Some(SessionError::Stop(stop)));
        assert!(matches!(
            stop,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                current: 22,
                requested_delta: 3,
                ..
            }
        ));
        assert_eq!(
            input.as_ref().unwrap().name.value.text().code_units(),
            original
        );
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            3
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), 16);
        assert!(session.active().unwrap().effects.form.is_none());
        assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
    }
}

#[test]
fn listed_getters_require_actual_active_and_frozen_arena_and_control_owners() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let node = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &node, None, None).unwrap();
    let mut other = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let foreign = other.root(&mut c).unwrap();
    assert_eq!(
        session.form_association_state(&mut c, &foreign),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(
        session.form_association_owner(&mut c, &foreign).err(),
        Some(SessionError::ForeignNode)
    );
    assert_eq!(
        session.form_association_state(&mut c, &body),
        Err(SessionError::FormAssociationMetadataUnavailable)
    );
    let mut foreign_c = control();
    foreign_c.admit_source(&source).unwrap();
    let before = c.usage();
    assert!(matches!(
        session.form_association_state(&mut foreign_c, &node),
        Err(SessionError::Stop(_))
    ));
    assert_eq!(c.usage(), before);
    lexer.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let html = document.root().children().next().unwrap();
    let body = document.node(&html).unwrap().children().next().unwrap();
    let id = document.node(&body).unwrap().children().next().unwrap();
    let mut foreign_frozen = other.freeze(&mut c).unwrap();
    let foreign_id = foreign_frozen.fragment().unwrap().root().id();
    assert_eq!(
        frozen.form_association_state(&mut c, &foreign_id),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(
        frozen.form_association_owner(&mut c, &foreign_id).err(),
        Some(SessionError::ForeignNode)
    );
    assert_eq!(
        frozen.form_association_state(&mut c, &id).unwrap().kind,
        HtmlNativeFormAssociationKind::Button
    );
    assert_eq!(
        frozen.image_form_state(&mut c, &id),
        Err(SessionError::ImageFormMetadataUnavailable)
    );
    frozen.release(&mut c).unwrap();
    foreign_frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_id_matching_is_exact_untrimmed_utf16_and_empty_id_never_matches_empty_form() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut units: Vec<u16> = "<button form='X ".encode_utf16().collect();
    units.extend([0xd800, 0xd83d, 0xde00]);
    units.extend("'><form id='x ".encode_utf16());
    units.extend([0xd800, 0xd83d, 0xde00]);
    units.extend("'><form id='X ".encode_utf16());
    units.extend([0xd800, 0xd83d, 0xde00]);
    units.extend("'><fieldset form=''><form id=''>".encode_utf16());
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(units));
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let lower = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &lower, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    let exact = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &exact, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&exact),
        false,
    );
    let empty = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &empty, None, None).unwrap();
    let empty_id = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session
        .insert(&mut c, &body, &empty_id, None, None)
        .unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &empty,
        HtmlNativeFormAssociationKind::Fieldset,
        None,
        false,
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_detached_id_change_resets_only_same_ordinary_tree() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><button form=f><button form=f><body id=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    let in_form = form_creation(&mut session, &mut lexer, &mut c, &form, None);
    session.insert(&mut c, &form, &in_form, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &in_form,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    let outside = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &outside, None, None).unwrap();
    session.remove(&mut c, &body, None).unwrap();
    let before = session.form_association_state(&mut c, &in_form).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
        panic!("body")
    };
    session
        .add_body_attributes_if_missing(&mut c, &body, &mut Some(tag))
        .unwrap();
    assert_eq!(
        session.form_association_state(&mut c, &in_form).unwrap(),
        before
    );
    assert_listed_owner(
        &session,
        &mut c,
        &in_form,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    assert_listed_owner(
        &session,
        &mut c,
        &outside,
        HtmlNativeFormAssociationKind::Button,
        None,
        false,
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

fn listed_move_fixture(
    c: &mut ParseControl,
) -> (ParserArenaSession, ParserNode, ParserNode, ParserNode) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<div><button form=f><form id=f><div>");
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, c).unwrap();
    let (mut session, body) = form_session(c);
    let group = form_creation(&mut session, &mut lexer, c, &body, None);
    session.insert(c, &body, &group, None, None).unwrap();
    let button = form_creation(&mut session, &mut lexer, c, &group, None);
    session.insert(c, &group, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, c, &group, None);
    session.insert(c, &group, &form, None, None).unwrap();
    let target = form_creation(&mut session, &mut lexer, c, &body, None);
    session.insert(c, &body, &target, None, None).unwrap();
    lexer.release(c).unwrap();
    (session, group, button, target)
}

#[test]
fn listed_exact_and_one_short_effect_work_retains_cleanup_only_phase_and_first_stop() {
    let mut measured = control();
    let (mut session, group, _, target) = listed_move_fixture(&mut measured);
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .insert(&mut measured, &target, &group, None, None)
        .unwrap();
    let whole = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - start;
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    let tail = 2 + mem::size_of::<Option<effects::EffectFrame>>() as u64 + 1 + 1;
    for short in [false, true] {
        let mut c = control();
        let (mut session, group, button, target) = listed_move_fixture(&mut c);
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            unused - whole + if short { tail + 1 } else { 0 },
        )
        .unwrap();
        let result = session.insert(&mut c, &target, &group, None, None);
        if short {
            let stop = c.stop().unwrap();
            assert_eq!(result, Err(SessionError::Stop(stop)));
            let storage = session.active().unwrap();
            assert!(storage.effects.form.is_none());
            let (list, cursor) = storage
                .effects
                .retained_post_connection()
                .expect("specific unfinished final postconnection clear");
            assert_eq!(cursor, 3);
            assert!(matches!(
                storage.payloads.values[list],
                RetainedPayload::Retired
            ));
            assert_eq!(
                session.form_association_state(&mut c, &button),
                Err(SessionError::Stop(stop))
            );
            assert_eq!(
                session.remove(&mut c, &group, None),
                Err(SessionError::Stop(stop))
            );
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        } else {
            result.unwrap();
            assert_eq!(
                session
                    .active()
                    .unwrap()
                    .construction
                    .node_at(group.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                Some(target.index)
            );
            assert!(session.active().unwrap().effects.form.is_none());
            // Successful operation exhausts its exact finite budget. Latch
            // the next real attempted credit before terminal disposal.
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
}

#[test]
fn listed_real_cancel_between_removal_and_insertion_retains_completed_global_reset() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, group, button, target) = listed_move_fixture(&mut c);
    session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
    assert_eq!(
        session.insert(&mut c, &target, &group, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(group.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(target.index)
    );
    let phase = storage.effects.form.unwrap();
    assert_eq!(phase.stage, FormEffectStage::Adoption);
    assert_eq!(phase.next, Some(group.index));
    let state = storage
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            } if *node == button.index => Some(*state),
            _ => None,
        })
        .unwrap();
    assert_eq!(state.owner, None);
    assert!(!state.parser_inserted);
    assert_eq!(state.evidence, HtmlParseFormEvidence::Reset);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

fn listed_id_slot_fixture(
    c: &mut ParseControl,
) -> (
    ParserArenaSession,
    ParserNode,
    ParserNode,
    PreparedAttributeMerge,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=f title=after>");
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, c).unwrap();
    let (mut session, body) = form_session(c);
    let button = form_creation(&mut session, &mut lexer, c, &body, None);
    session.insert(c, &body, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, c, &body, None);
    session.insert(c, &body, &form, None, None).unwrap();
    assert_listed_owner(
        &session,
        c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    let TokenKind::Tag(tag) = lexical_kind(&mut lexer, c) else {
        panic!("body")
    };
    let prepared = session
        .active_mut()
        .unwrap()
        .prepare_named_attribute_merge(c, &body, &mut Some(tag), AttributeMergeName::Body)
        .unwrap()
        .unwrap();
    lexer.release(c).unwrap();
    session
        .active_mut()
        .unwrap()
        .edit_attributes(
            c,
            prepared.target,
            prepared.container,
            prepared.stage,
            1,
            true,
        )
        .unwrap();
    // This is the actual fixed append producer, not an invented phase. It
    // retains both the installed slot and incoming redirect until drain.
    let storage = session.active().unwrap();
    assert_eq!(
        storage.effects.form.unwrap().stage,
        FormEffectStage::IdChange
    );
    let RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved, values, .. })) =
        &storage.payloads.values[prepared.stage]
    else {
        panic!("transfer")
    };
    assert_eq!(*moved, 1);
    assert_eq!(values.len(), 1);
    let ConstructionNodeKind::Element(element) = &storage
        .construction
        .node_at(body.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("body")
    };
    assert_eq!(element.attributes().len(), 1);
    assert_eq!(
        element
            .attributes()
            .get(0)
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        [105, 100]
    );
    (session, body, button, prepared)
}

#[test]
fn listed_id_append_blocks_observation_before_rebind_and_keeps_cleanup_redirect() {
    let mut c = control();
    let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
    let old_count = session.active().unwrap().construction.test_storage().len();
    assert!(matches!(
        session.form_association_owner(&mut c, &button),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::PendingFormEffects
        )))
    ));
    let stop = c.stop().unwrap();
    assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
    let storage = session.active().unwrap();
    assert_eq!(storage.construction.test_storage().len(), old_count);
    assert!(storage.effects.form.is_some());
    let ConstructionNodeKind::Element(element) = &storage
        .construction
        .node_at(body.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("body")
    };
    assert_eq!(element.attributes().len(), 1);
    assert!(matches!(
        &storage.payloads.values[prepared.stage],
        RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
    ));
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
}

#[test]
fn listed_id_global_drain_exact_and_one_short_keeps_real_slot_owners_and_metadata_prefix() {
    let mut measured = control();
    let (mut session, _, _, _) = listed_id_slot_fixture(&mut measured);
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .active_mut()
        .unwrap()
        .drain_form_effects(&mut measured)
        .unwrap();
    let whole = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - start;
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    for short in [false, true] {
        let mut c = control();
        let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
        let unused =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            unused - whole + u64::from(short),
        )
        .unwrap();
        let result = session.active_mut().unwrap().drain_form_effects(&mut c);
        let storage = session.active().unwrap();
        let state = storage
            .payloads
            .values
            .iter()
            .find_map(|entry| match entry {
                RetainedPayload::FormAssociation {
                    node: Some(node),
                    state,
                } if *node == button.index => Some(*state),
                _ => None,
            })
            .unwrap();
        assert_eq!(state.owner, None);
        assert_eq!(state.evidence, HtmlParseFormEvidence::Reset);
        assert!(!state.parser_inserted);
        assert!(matches!(
            &storage.payloads.values[prepared.stage],
            RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
        ));
        let ConstructionNodeKind::Element(element) = &storage
            .construction
            .node_at(body.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("body")
        };
        assert_eq!(element.attributes().len(), 1);
        if short {
            let stop = c.stop().unwrap();
            assert_eq!(result, Err(SessionError::Stop(stop)));
            assert!(session.active().unwrap().effects.form.is_some());
            assert_eq!(
                session.form_association_owner(&mut c, &button).err(),
                Some(SessionError::Stop(stop))
            );
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        } else {
            result.unwrap();
            assert!(session.active().unwrap().effects.form.is_none());
            // No credit remains to complete the next slot/rebind. Terminal
            // disposal must reclaim the existing redirect, not drop it.
            assert!(c.charge(HtmlParseWorkCategory::StateStep, 1).is_err());
        }
        let stop = c.stop().unwrap();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let high = c.usage().actual_capacity_high_water;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(stop));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_eq!(c.usage().actual_capacity_high_water, high);
    }
}

#[test]
fn listed_id_cancel_after_real_slot_append_retains_owner_and_reclaims_all_partial_storage() {
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let (mut session, body, button, prepared) = listed_id_slot_fixture(&mut c);
    cancellation.cancel();
    assert_eq!(
        session.active_mut().unwrap().drain_form_effects(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.form_association_owner(&mut c, &button).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(session.active().unwrap().effects.form.is_some());
    assert!(matches!(
        &session.active().unwrap().payloads.values[prepared.stage],
        RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved: 1, .. }))
    ));
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(body.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("body")
    };
    assert_eq!(element.attributes().len(), 1);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn listed_virtual_removal_excludes_moved_id_from_outside_tree_before_insertion() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let source = crate::HtmlSource::utf8("<button form=f><div><form id=f><div>");
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let group = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &group, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, &mut c, &group, None);
    session.insert(&mut c, &group, &form, None, None).unwrap();
    let target = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &target, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    lexer.release(&mut c).unwrap();
    session.active_mut().unwrap().cancel_after_removal = Some(cancellation);
    assert_eq!(
        session.insert(&mut c, &target, &group, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(group.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(target.index)
    );
    assert_eq!(
        storage
            .construction
            .node_at(form.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(group.index)
    );
    let state = storage
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::FormAssociation {
                node: Some(node),
                state,
            } if *node == button.index => Some(*state),
            _ => None,
        })
        .unwrap();
    // Final links already expose the ID at the new destination. Removal
    // resolution must exclude the moved root from this outside scan.
    assert_eq!(state.owner, None);
    assert_eq!(
        storage.effects.form.unwrap().stage,
        FormEffectStage::Adoption
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn listed_namespaced_id_is_inapplicable_and_imported_control_metadata_is_unavailable() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button form=f><form id=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    // Preexisting immutable native input deliberately exercises namespace
    // identity, not parser-created physical storage or foreign token support.
    let native = PreexistingElement::new(
        QualifiedName::new(Namespace::html(), None, "button"),
        OrderedAttributes::new(vec![Attribute::new(
            QualifiedName::new(
                Namespace::from_uri("urn:test"),
                Some(DocumentString::from("p")),
                "id",
            ),
            "f",
        )])
        .unwrap(),
    );
    let imported = session.allocate_element(&mut c, native).unwrap();
    session
        .insert(&mut c, &body, &imported, None, None)
        .unwrap();
    session.insert(&mut c, &body, &form, None, None).unwrap();
    assert_listed_owner(
        &session,
        &mut c,
        &button,
        HtmlNativeFormAssociationKind::Button,
        Some(&form),
        false,
    );
    assert_eq!(
        session.form_association_state(&mut c, &imported),
        Err(SessionError::FormAssociationMetadataUnavailable)
    );
    assert_eq!(
        session.validate_standalone_document(&mut c),
        Err(SessionError::Stop(
            HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance)
        ))
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn listed_empty_and_ignored_duplicate_late_id_do_not_replace_existing_form_owner() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<button form=f><form id=f><body id=''><body id=f>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut session, body) = form_session(&mut c);
    let button = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &button, None, None).unwrap();
    let form = form_creation(&mut session, &mut lexer, &mut c, &body, None);
    session.insert(&mut c, &body, &form, None, None).unwrap();
    for _ in 0..2 {
        let TokenKind::Tag(tag) = lexical_kind(&mut lexer, &mut c) else {
            panic!("body")
        };
        session
            .add_body_attributes_if_missing(&mut c, &body, &mut Some(tag))
            .unwrap();
        assert_listed_owner(
            &session,
            &mut c,
            &button,
            HtmlNativeFormAssociationKind::Button,
            Some(&form),
            false,
        );
        assert!(session.active().unwrap().effects.form.is_none());
    }
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(body.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("body")
    };
    assert_eq!(element.attributes().len(), 1);
    assert!(element.attributes().get(0).unwrap().value().is_empty());
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// A creation that registered its form-association row and then finds the
/// row gone has lost a retained row. The row it names is what is violated,
/// not the stage the creation had reached.
#[test]
fn a_creation_that_loses_its_form_association_row_reports_the_registry_row() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head></head><input type=text>"),
        limits(),
    );
    let (mut c, mut session, capability, lexer, parent, mut input, range) = input_owner(&request);
    session.active_mut().unwrap().lose_association_row = Some(None);
    let result = session.allocate_input_for_creation(
        &mut c,
        &mut input,
        CreationLocation {
            intended_parent: &parent,
            current_form: None,
            context: CreationContext::Ordinary,
        },
        &range,
        &capability,
    );
    let registry_row =
        HtmlParseStop::InternalInvariant(crate::parser::control::HtmlParseInvariant::RegistryRow);
    assert_eq!(result.err(), Some(SessionError::Stop(registry_row)));
    assert_eq!(c.stop(), Some(registry_row));
    // The row goes back where the registry expects it, so cleanup finds
    // the registry as it left it.
    let storage = session.active_mut().unwrap();
    let (slot, row) = storage.lose_association_row.take().flatten().unwrap();
    storage.payloads.values[slot] = row;
    release_formatting_tag(&mut input, &mut c).unwrap();
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
