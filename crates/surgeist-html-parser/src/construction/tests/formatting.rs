use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    values::{DocumentString, Namespace},
};

fn formatting_lexical_tag(
    tokenizer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> (Option<Tag>, crate::HtmlSourceRange) {
    loop {
        match tokenizer.next(c, false).unwrap().expect("actual token") {
            crate::parser::tokenizer::token::Event::Token(event) => {
                let token = event.into_token(c).unwrap();
                let TokenKind::Tag(tag) = token.kind else {
                    panic!("expected real tag")
                };
                return (Some(tag), token.source);
            }
            event => event.release(c).unwrap(),
        }
    }
}
fn formatting_parent(c: &mut ParseControl, session: &mut ParserArenaSession) -> ParserNode {
    let root = session.root(c).unwrap();
    let html = session.allocate_implicit_html(c).unwrap();
    session.insert(c, &root, &html, None, None).unwrap();
    let body = session.allocate_implicit_body(c, None).unwrap();
    session.insert(c, &html, &body, None, None).unwrap();
    body
}
#[test]
fn template_owned_html_wrapper_pre_registration_stops_reclaim_lexical_fields() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // require: 8 visits (including the retained Radio and Range phases); kind: 1; name owner+alias: 2; lowercase
    // template: 16; indexed attribute: 1; its two owner+alias checks: 4;
    // lowercase a: 2. This is the actual borrowed preflight, before take.
    const PREFLIGHT: u64 = 34;
    // The newly owned wrapper reads length (1), eight units (3 each),
    // then root kind (1). Cut each credit before document preparation.
    for wrapper_credit in 0..26 {
        let source = crate::HtmlSource::utf8("<template a='V&#65;'>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut input, _) = formatting_lexical_tag(&mut lexer, &mut c);
        lexer.release(&mut c).unwrap();
        let tag = input.as_ref().unwrap();
        assert_eq!(
            tag.name.value.text().code_units(),
            [116, 101, 109, 112, 108, 97, 116, 101]
        );
        assert_eq!(
            tag.attributes.as_slice()[0].value.value.text().code_units(),
            [86, 65]
        );
        assert!(!tag.name.origins.as_slice().is_empty());
        assert_eq!(tag.attributes.as_slice()[0].value.origins.len(), 2);
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - PREFLIGHT - wrapper_credit,
        )
        .unwrap();
        let result = session.allocate_html_tag(&mut c, &mut input);
        let first = c.stop().expect("wrapper work stop");
        assert!(matches!(
            first,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert_eq!(result.err(), Some(SessionError::Stop(first)));
        assert!(
            input.is_none(),
            "lexical transfer precedes wrapper, credit={wrapper_credit}"
        );
        assert_eq!(session.active().unwrap().construction.len(), 1);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(first));
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        storage_zero(&c);
    }
}

#[test]
fn template_owned_html_wrapper_cancellation_reclaims_transferred_lexical_fields() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<template a='V&#65;'>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, _) = formatting_lexical_tag(&mut lexer, &mut c);
    lexer.release(&mut c).unwrap();
    // Exercise the real owned wrapper with fields from a validated actual
    // lexical tag. Cancellation is observable at its first admission.
    SessionStorage::check_html_tag(&mut c, input.as_mut().unwrap()).unwrap();
    let Tag {
        name, attributes, ..
    } = input.take().unwrap();
    cancellation.cancel();
    assert_eq!(
        session
            .active_mut()
            .unwrap()
            .allocate_html_payload(&mut c, name.value, Some(name.origins), attributes,)
            .err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

fn formatting_complete_copy_prefix(
    session: &mut ParserArenaSession,
    c: &mut ParseControl,
    creation: &FormattingCreation,
    parent: &ParserNode,
) -> u64 {
    // A separate real prefix locates the admission window; it is not the
    // cleanup oracle. Independently check all copied fields/storage below.
    let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.validate_standalone_document(c).unwrap();
    let storage = session.active().unwrap();
    let parent = storage.index(c, parent).unwrap();
    storage.container(c, parent).unwrap();
    assert!(
        !session
            .active_mut()
            .unwrap()
            .associated_document_parent(c, parent)
            .unwrap()
    );
    session.require_formatting_creation(c, creation).unwrap();
    let storage = session.active().unwrap();
    let document = storage.node_document_ref(c, parent).unwrap();
    session
        .active_mut()
        .unwrap()
        .prepare_document_allocation(c, Some(document), false)
        .unwrap();
    let original = creation.token.as_ref().unwrap();
    let mut copy = Some(copy_formatting_tag(c, original).unwrap());
    let complete = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
    let copied = copy.as_ref().unwrap();
    assert_eq!(copied.name.value.text().code_units(), [98]);
    assert_eq!(
        copied.attributes.as_slice()[0]
            .name
            .value
            .text()
            .code_units(),
        [97]
    );
    assert_eq!(
        copied.attributes.as_slice()[0]
            .value
            .value
            .text()
            .code_units(),
        [86, 65]
    );
    assert_ne!(
        copied.name.value.text().code_units().as_ptr(),
        original.name.value.text().code_units().as_ptr()
    );
    let origins = |text: &Text| {
        text.origins
            .as_slice()
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        origins(&copied.attributes.as_slice()[0].value),
        origins(&original.attributes.as_slice()[0].value)
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::TokenAttributes)
            .current,
        2
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        8
    );
    discard_formatting_tag(&mut copy, c).unwrap();
    complete
}

#[test]
fn formatting_completed_copy_take_stops_keep_recipe_and_reclaim_independent_fields() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    fn fixture(
        source: &crate::HtmlSource,
        c: &mut ParseControl,
    ) -> (ParserArenaSession, ParserNode, FormattingCreation) {
        c.admit_source(source).unwrap();
        let mut session =
            ParserArenaSession::new(c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
        let parent = formatting_parent(c, &mut session);
        let mut lexer = LexerFixture::new(source, HtmlTokenizerState::Data, None, c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut lexer, c);
        lexer.release(c).unwrap();
        let creation = session
            .retain_formatting_creation(c, &mut input, range)
            .unwrap();
        assert!(input.is_none());
        (session, parent, creation)
    }
    let source = crate::HtmlSource::utf8("<b a='V&#65;'>");
    let mut measured = control();
    let (mut session, parent, mut creation) = fixture(&source, &mut measured);
    let prefix = formatting_complete_copy_prefix(&mut session, &mut measured, &creation, &parent);
    creation.release(&mut measured).unwrap();
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    // take_payload: one actual Option visit, then the fixed Option<Tag>
    // copy, then two writes. Cut each admitted action after complete copy.
    for take_credit in [
        0,
        1,
        1 + mem::size_of::<Option<Tag>>() as u64,
        2 + mem::size_of::<Option<Tag>>() as u64,
    ] {
        let mut c = control();
        let (mut session, parent, mut creation) = fixture(&source, &mut c);
        let pointer = creation
            .token
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - prefix - take_credit,
        )
        .unwrap();
        let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
        let first = c.stop().expect("copied token take work stop");
        assert!(matches!(
            first,
            HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert_eq!(result.err(), Some(SessionError::Stop(first)));
        assert_eq!(session.active().unwrap().construction.len(), 3);
        assert_eq!(
            creation
                .token
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr(),
            pointer
        );
        assert_eq!(creation.token.as_ref().unwrap().attributes.len(), 1);
        assert_eq!(
            session.freeze(&mut c).err(),
            Some(SessionError::Stop(first))
        );
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        creation.release(&mut c).unwrap();
        assert!(creation.token.is_none());
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(first));
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        storage_zero(&c);
    }
}
#[test]
fn formatting_replay_copies_exact_utf16_attributes_and_original_origins_into_distinct_nodes() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
        "<b z='A&#x1F600;\r\n"
            .encode_utf16()
            .chain([0xd800])
            .chain("' a=x>".encode_utf16())
            .collect::<Vec<_>>(),
    ));
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let parent = formatting_parent(&mut c, &mut session);
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let original_pointer = input
        .as_ref()
        .unwrap()
        .name
        .value
        .text()
        .code_units()
        .as_ptr();
    let original_origins = input.as_ref().unwrap().attributes.as_slice()[0]
        .value
        .origins
        .as_slice()
        .iter()
        .map(|o| (o.start, o.end, o.source.clone(), o.kind))
        .collect::<Vec<_>>();
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    assert!(input.is_none());
    let first = session
        .create_formatting_from_creation(&mut c, &creation, &parent)
        .unwrap();
    let second = session
        .create_formatting_from_creation(&mut c, &creation, &parent)
        .unwrap();
    assert!(!session.same_node(&mut c, &first, &second).unwrap());
    session.insert(&mut c, &parent, &first, None, None).unwrap();
    session
        .insert(&mut c, &parent, &second, None, None)
        .unwrap();
    for node in [&first, &second] {
        let storage = session.active().unwrap();
        let ConstructionNodeKind::Element(element) = &storage
            .construction
            .node_at(node.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("element")
        };
        assert_ne!(
            element.name().local_name().code_units().as_ptr(),
            original_pointer
        );
        assert_eq!(element.name().namespace(), &Namespace::html());
        let attributes = test_support::attributes_of(element);
        assert_eq!(attributes[0].name().local_name().code_units(), [122]);
        assert_eq!(attributes[0].name().namespace(), &Namespace::none());
        assert_eq!(
            test_support::value_units(attributes[0]),
            [65, 0xd83d, 0xde00, 10, 0xd800]
        );
        assert_eq!(attributes[1].name().local_name().code_units(), [97]);
        let group = storage
            .payloads
            .detail_origins
            .get(&mut c, FieldRef::AttributeValue(node.index, 0))
            .unwrap()
            .expect("actual attribute origins");
        let origins = storage.observed_origins(group, &mut c).unwrap();
        let observed = origins
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>();
        assert_eq!(observed, original_origins);
    }
    creation.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    let mut arena = session.freeze(&mut c).unwrap();
    assert!(arena.document().is_some());
    arena.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        5
    );
}

#[test]
fn formatting_noah_ignores_attribute_order_duplicate_recovery_and_self_closing_but_preserves_exact_values()
 {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source =
        crate::HtmlSource::utf8("<b a=x z=y><b z=y a=x a=ignored /><b a=x z=Y><i z=y a=x>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut first, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    let mut a = session
        .retain_formatting_creation(&mut c, &mut first, range)
        .unwrap();
    let (mut second, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    assert!(second.as_ref().unwrap().had_duplicate_attributes);
    assert!(second.as_ref().unwrap().self_closing);
    let mut b = session
        .retain_formatting_creation(&mut c, &mut second, range)
        .unwrap();
    let (mut third, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    let mut different_value = session
        .retain_formatting_creation(&mut c, &mut third, range)
        .unwrap();
    let (mut fourth, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    let mut different_name = session
        .retain_formatting_creation(&mut c, &mut fourth, range)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    assert!(session.same_formatting_creation(&mut c, &a, &b).unwrap());
    assert!(
        !session
            .same_formatting_creation(&mut c, &a, &different_value)
            .unwrap()
    );
    assert!(
        !session
            .same_formatting_creation(&mut c, &a, &different_name)
            .unwrap()
    );
    a.release(&mut c).unwrap();
    b.release(&mut c).unwrap();
    different_value.release(&mut c).unwrap();
    different_name.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_capture_rejects_alias_and_wrong_family_without_consuming_original() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<b a=x><applet>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    let alias = input.as_ref().unwrap().name.value.text().clone();
    assert!(matches!(
        session.retain_formatting_creation(&mut c, &mut input, range.clone()),
        Err(SessionError::AliasedPayload)
    ));
    assert!(input.is_some());
    assert_eq!(c.stop(), None);
    drop(alias);
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    let (mut wrapper, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    assert!(matches!(
        session.retain_formatting_creation(&mut c, &mut wrapper, range),
        Err(SessionError::WrongToken)
    ));
    assert!(wrapper.is_some());
    discard_formatting_tag(&mut wrapper, &mut c).unwrap();
    creation.release(&mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_replay_rejects_foreign_arena_and_parent_before_new_payload_admission() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<b a=x>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let parent = formatting_parent(&mut c, &mut session);
    let mut other =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let other_parent = other.root(&mut c).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    let before = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    assert!(matches!(
        other.create_formatting_from_creation(&mut c, &creation, &other_parent),
        Err(SessionError::ForeignNode)
    ));
    assert!(matches!(
        session.create_formatting_from_creation(&mut c, &creation, &other_parent),
        Err(SessionError::ForeignNode)
    ));
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        before
    );
    assert!(creation.token.is_some());
    let node = session
        .create_formatting_from_creation(&mut c, &creation, &parent)
        .unwrap();
    session.insert(&mut c, &parent, &node, None, None).unwrap();
    creation.release(&mut c).unwrap();
    other.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_replay_token_attribute_coexistence_exact_and_one_short_reclaims_only_copies() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for cap in [3, 4] {
        let source = crate::HtmlSource::utf8("<b a=x z=y>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .token_attributes(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
        if cap == 3 {
            // One attribute beyond the exact cap of three is the first stop.
            let first = HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::TokenAttributes,
                limit: 3,
                current: 3,
                requested_delta: 1,
            };
            assert_eq!(result.err(), Some(SessionError::Stop(first)));
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                5
            );
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                3
            );
        } else {
            let node = result.unwrap();
            session.insert(&mut c, &parent, &node, None, None).unwrap();
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::TokenAttributes)
                    .high_water,
                4
            );
        }
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::TokenAttributes)
                .current,
            2
        );
        let stop = c.stop();
        creation.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}

#[test]
fn formatting_recipe_terminal_cancel_and_foreign_disposal_preserve_owner_then_reclaim_once() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<b a=x>");
    let cancel = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancel.clone());
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let parent = formatting_parent(&mut c, &mut session);
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    let mut foreign = control();
    foreign.admit_source(&source).unwrap();
    assert!(creation.release(&mut foreign).is_err());
    assert!(creation.token.is_some());
    cancel.cancel();
    assert!(matches!(
        session.create_formatting_from_creation(&mut c, &creation, &parent),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    let alias = creation.token.as_ref().unwrap().name.value.text().clone();
    assert_eq!(creation.release(&mut c), Err(SessionError::AliasedPayload));
    assert!(creation.token.is_some());
    drop(alias);
    creation.release(&mut c).unwrap();
    creation.release(&mut c).unwrap();
    assert_freeze_reports_first_stop(&mut session, &mut c, HtmlParseStop::Cancelled);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}

#[test]
fn formatting_same_source_foreign_control_replay_preserves_both_arenas_and_original_recipe() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<b a=x>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    let mut foreign = control();
    foreign.admit_source(&source).unwrap();
    let mut other =
        ParserArenaSession::new(&mut foreign, RootKind::Document(HtmlDocumentMode::NoQuirks))
            .unwrap();
    let other_root = other.root(&mut foreign).unwrap();
    assert!(matches!(
        other.create_formatting_from_creation(&mut foreign, &creation, &other_root),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert_eq!(
        foreign
            .usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    assert_eq!(
        foreign
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current,
        1
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        3
    );
    assert_eq!(c.stop(), None);
    assert!(creation.token.is_some());
    other.release(&mut foreign).unwrap();
    storage_zero(&foreign);
    session
        .create_formatting_from_creation(&mut c, &creation, &root)
        .unwrap();
    creation.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_child_transfer_preserves_existing_destination_order_and_backlinks() {
    let mut c = control();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let parent = formatting_parent(&mut c, &mut session);
    let from = session.allocate_implicit_p(&mut c, None).unwrap();
    let to = session.allocate_implicit_p(&mut c, None).unwrap();
    session.insert(&mut c, &parent, &from, None, None).unwrap();
    session.insert(&mut c, &parent, &to, None, None).unwrap();
    let existing = session.allocate_implicit_p(&mut c, None).unwrap();
    session.insert(&mut c, &to, &existing, None, None).unwrap();
    let one = session.allocate_implicit_p(&mut c, None).unwrap();
    let two = session.allocate_implicit_p(&mut c, None).unwrap();
    session.insert(&mut c, &from, &one, None, None).unwrap();
    session.insert(&mut c, &from, &two, None, None).unwrap();
    let edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
    session.reparent_children(&mut c, &from, &to, None).unwrap();
    assert!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(from.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>()
            .is_empty()
    );
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(to.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [existing.index, one.index, two.index]
    );
    for child in [&existing, &one, &two] {
        let actual_parent = session.parent(&mut c, child).unwrap().unwrap();
        assert!(session.same_node(&mut c, &actual_parent, &to).unwrap());
    }
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedEdges).current,
        edges
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        session.active().unwrap().construction.len() as u64
            + session.active().unwrap().payloads.values.len() as u64
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_namespace_copy_exact_and_one_short_buffer_limit_retains_original_and_staged_name() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Original b needs one unit. Replay's copied name moves to Managed
    // before the defining 28-unit HTML namespace is buffered: peak 1+28.
    for cap in [28, 29] {
        let source = crate::HtmlSource::utf8("<b>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = session.root(&mut c).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
        if cap == 28 {
            // Peak 1 + 28 against the cap of 28: one unit is refused.
            let first = HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::BufferedUtf16Units,
                limit: 28,
                current: 28,
                requested_delta: 1,
            };
            assert_eq!(result.err(), Some(SessionError::Stop(first)));
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                1
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                1
            );
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        } else {
            assert!(result.is_ok());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                29
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .high_water,
                29
            );
        }
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current,
            1
        );
        let stop = c.stop();
        creation.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}

#[test]
fn formatting_capture_one_short_ordinary_document_qualification_preserves_original_before_take() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Readiness first inspects the real form phase plus seven intrinsic readiness fields including Radio and Range (eight visits).
    // Qualification then admits eligibility/root-kind together (two).
    // One short leaves only readiness admitted; exactly ten completes
    // qualification but cannot admit the next source-range visit.
    for remaining in [9, 10] {
        let source = crate::HtmlSource::utf8("<b>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        tokenizer.release(&mut c).unwrap();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - work - remaining,
        )
        .unwrap();
        let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
        assert!(matches!(
            session.retain_formatting_creation(&mut c, &mut input, range),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        assert!(input.is_some());
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordVisit) - before,
            if remaining == 10 { 10 } else { 8 }
        );
        let stop = c.stop();
        discard_formatting_tag(&mut input, &mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}

#[test]
fn formatting_wide_replay_source_map_coexistence_exact_and_one_short_reclaims_partial_copies() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Real b/v/512 literal x values have 514 original codepoint origins.
    // Independent replay retains them and needs 514 new origin records.
    // Native retention additionally owns four groups and four explicit
    // source-independent field causes (name, URI, attribute name/value).
    // Keep the real lexical owner across replay: two occurrence/cause pairs
    // and 523 actual delimiter/field parts add 527 retained source rows.
    // The retained lexer last-start b adds one buffered unit (514 -> 515).
    for cap in [1564, 1565] {
        let source = crate::HtmlSource::utf8(format!("<b v='{}'>", "x".repeat(512)));
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(2000)
                .source_map_entries(cap)
                .work_credits(100_000_000)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = session.root(&mut c).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut input, range)
            .unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            1041
        );
        let result = session.create_formatting_from_creation(&mut c, &creation, &parent);
        if cap == 1564 {
            // 1041 retained entries plus the copy leave one short of 1565.
            let first = HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                limit: 1564,
                current: 1564,
                requested_delta: 1,
            };
            assert_eq!(result.err(), Some(SessionError::Stop(first)));
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                1564
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                515
            );
            // The copy of the element was made before its attributes were appended.
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                2
            );
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        } else {
            assert!(result.is_ok());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                1565
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .high_water,
                1565
            );
        }
        let stop = c.stop();
        creation.release(&mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }
}

#[test]
fn formatting_capture_foreign_source_and_replay_import_refusal_leave_original_recipe_owned() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<b a=x>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let parent = session.root(&mut c).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let mut creation = session
        .retain_formatting_creation(&mut c, &mut input, range)
        .unwrap();
    session.allocate_element(&mut c, html("div")).unwrap();
    assert!(matches!(
        session.create_formatting_from_creation(&mut c, &creation, &parent),
        Err(SessionError::Stop(
            HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance)
        ))
    ));
    assert!(creation.token.is_some());
    creation.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);

    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (mut input, _range) = formatting_lexical_tag(&mut tokenizer, &mut c);
    tokenizer.release(&mut c).unwrap();
    let foreign = crate::HtmlSource::utf8("<b a=x>").range(0, 7).unwrap();
    assert!(
        session
            .retain_formatting_creation(&mut c, &mut input, foreign)
            .is_err()
    );
    assert!(input.is_some());
    discard_formatting_tag(&mut input, &mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn formatting_child_transfer_stops_with_an_ordered_coherent_prefix_and_reclaims_all_scratch() {
    let mut observed_partial = false;
    // Seven nodes/38 retained records, no attributes: the complete caller
    // and normal release cost less than5820 on this host layout. Sample
    // actual commit prefixes within the independently bounded8192 envelope.
    for remaining in (0..=8192).step_by(32) {
        let mut c = control();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let parent = formatting_parent(&mut c, &mut session);
        let from = session.allocate_implicit_p(&mut c, None).unwrap();
        let to = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &parent, &from, None, None).unwrap();
        session.insert(&mut c, &parent, &to, None, None).unwrap();
        let one = session.allocate_implicit_p(&mut c, None).unwrap();
        let two = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &from, &one, None, None).unwrap();
        session.insert(&mut c, &from, &two, None, None).unwrap();
        let before_nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
        let before_edges = c.usage().resource(HtmlParseResource::ManagedEdges).current;
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - work - remaining,
        )
        .unwrap();
        let result = session.reparent_children(&mut c, &from, &to, None);
        let storage = session.active().unwrap();
        let moved = storage
            .construction
            .node_at(to.index)
            .expect("retained construction position")
            .children()
            .count();
        let ordered = [one.index, two.index];
        assert_eq!(
            storage
                .construction
                .node_at(to.index)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>(),
            &ordered[..moved]
        );
        assert_eq!(
            storage
                .construction
                .node_at(from.index)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>(),
            &ordered[moved..]
        );
        for (index, child) in ordered.iter().enumerate() {
            assert_eq!(
                storage
                    .construction
                    .node_at(*child)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                Some(if index < moved { to.index } else { from.index })
            );
        }
        observed_partial |= moved == 1;
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            before_nodes
        );
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedEdges).current,
            before_edges
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            storage.construction.len() as u64 + storage.payloads.values.len() as u64
        );
        if result.is_err() {
            // The sweep leaves `remaining` work credits, so the first stop
            // is the exhausted work-credit limit of the configured limits.
            let first = limit_stop_of(
                &result,
                HtmlParseResource::WorkCredits,
                limits().work_credits(),
            );
            assert_freeze_reports_first_stop(&mut session, &mut c, first);
        }
        let stop = c.stop();
        if session.release(&mut c).is_err() {
            assert!(c.stop().is_some());
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if let Some(stop) = stop {
            assert_eq!(c.stop(), Some(stop));
        }
    }
    assert!(
        observed_partial,
        "a real first-child commit must survive a later work stop"
    );
}
