use super::*;
use surgeist_html::{
    construction::{
        ConstructionNodeKind, HtmlAuthoredProcessingInstruction, HtmlDocumentMode,
        HtmlProcessingInstructionError,
    },
    values::{DocumentString, Namespace},
};

#[test]
fn actual_comment_recovered_pi_and_doctype_keep_exact_managed_native_payloads() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
        "<!--a"
            .encode_utf16()
            .chain([0xd800])
            .chain("--><?x a??><!DOCTYPE h PUBLIC 'p' 's'>".encode_utf16())
            .collect::<Vec<_>>(),
    ));
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let TokenKind::Comment(text) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("comment");
    };
    let comment = session
        .allocate_comment(&mut c, &mut Some(text), None)
        .unwrap();
    session.insert(&mut c, &root, &comment, None, None).unwrap();
    assert!(session.document_accepts_doctype(&mut c).unwrap());
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut tokenizer, &mut c)
    else {
        panic!("PI");
    };
    let pi = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    session.insert(&mut c, &root, &pi, None, None).unwrap();
    assert!(session.document_accepts_doctype(&mut c).unwrap());
    let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("doctype");
    };
    let doctype = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
    session.insert(&mut c, &root, &doctype, None, None).unwrap();
    assert!(!session.document_accepts_doctype(&mut c).unwrap());
    session
        .set_document_mode(&mut c, HtmlDocumentMode::LimitedQuirks)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        8
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        0
    );
    let mut output = frozen(session, &mut c);
    let doc = output.document().unwrap();
    assert_eq!(doc.mode(), HtmlDocumentMode::LimitedQuirks);
    let children = doc.root().children().collect::<Vec<_>>();
    let surgeist_html::construction::HtmlNodeKind::Comment(value) =
        doc.node(&children[0]).unwrap().kind()
    else {
        panic!("comment");
    };
    assert_eq!(value.code_units(), [0x61, 0xd800]);
    drop(value);
    let surgeist_html::construction::HtmlNodeKind::ProcessingInstruction(value) =
        doc.node(&children[1]).unwrap().kind()
    else {
        panic!("PI");
    };
    assert_eq!(value.target().code_units(), [0x78]);
    assert_eq!(value.data().code_units(), [0x61, 0x3f]);
    assert_eq!(
        HtmlAuthoredProcessingInstruction::try_from(value.clone()),
        Err(HtmlProcessingInstructionError::DataEndsWithQuestionMark)
    );
    drop(value);
    let surgeist_html::construction::HtmlNodeKind::Doctype(value) =
        doc.node(&children[2]).unwrap().kind()
    else {
        panic!("doctype");
    };
    assert_eq!(value.name().code_units(), [0x68]);
    assert_eq!(value.public_id().unwrap().code_units(), [0x70]);
    assert_eq!(value.system_id().unwrap().code_units(), [0x73]);
    drop(value);
    output.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn missing_doctype_fields_lower_to_managed_empty_strings_and_detached_nodes_survive_guard() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<!DOCTYPE><!DOCTYPE html>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("doctype");
    };
    assert!(
        value.name.is_none()
            && value.public_id.is_none()
            && value.system_id.is_none()
            && value.force_quirks
    );
    let first = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
    assert!(session.document_accepts_doctype(&mut c).unwrap());
    session.insert(&mut c, &root, &first, None, None).unwrap();
    let TokenKind::Doctype(value) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("doctype");
    };
    let detached = session.allocate_doctype(&mut c, &mut Some(value)).unwrap();
    assert!(!session.document_accepts_doctype(&mut c).unwrap());
    assert!(session.parent(&mut c, &detached).unwrap().is_none());
    let ConstructionNodeKind::Doctype(value) = &session
        .active()
        .unwrap()
        .construction
        .node_at(first.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("doctype");
    };
    assert!(value.name().is_empty());
    assert!(value.public_id().unwrap().is_empty());
    assert!(value.system_id().unwrap().is_empty());
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn source_implied_html_owns_literal_backing_and_real_empty_attributes_at_exact_unit_cap() {
    // html has four units; http://www.w3.org/1999/xhtml has twenty-eight.
    for cap in [31, 32] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .managed_utf16_units(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let result = session.allocate_implicit_html(&mut c);
        if cap == 31 {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedUtf16Units,
                    ..
                }))
            ));
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                1
            );
        } else {
            let node = result.unwrap();
            let ConstructionNodeKind::Element(value) = &session
                .active()
                .unwrap()
                .construction
                .node_at(node.index)
                .expect("retained construction position")
                .kind()
            else {
                panic!("element");
            };
            assert_eq!(
                value.name().local_name().code_units(),
                [0x68, 0x74, 0x6d, 0x6c]
            );
            assert_eq!(value.name().namespace(), &Namespace::html());
            assert!(value.attributes().is_empty());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                32
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                0
            );
        }
        if session.release(&mut c).is_err() {
            assert!(session.storage.is_some());
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
    }
}
#[test]
fn doctype_partial_work_failures_keep_real_field_owners_reclaimable() {
    let make = |c: &mut ParseControl| {
        Some(Doctype {
            name: Some(owned_text(c, &[0x68])),
            public_id: Some(owned_text(c, &[0xd800])),
            system_id: Some(owned_text(c, &[0x73])),
            force_quirks: false,
        })
    };
    let mut probe = control();
    let mut probe_session = ParserArenaSession::new(&mut probe, RootKind::Fragment).unwrap();
    let mut input = make(&mut probe);
    let before = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    probe_session
        .allocate_doctype(&mut probe, &mut input)
        .unwrap();
    let required = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    probe_session.release(&mut probe).unwrap();
    storage_zero(&probe);
    for remaining in [0, 1, 40, 100, required - 1, required] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = make(&mut c);
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - remaining,
        )
        .unwrap();
        let result = session.allocate_doctype(&mut c, &mut input);
        assert_eq!(result.is_ok(), remaining == required);
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let stop = c.stop();
        if let Some(value) = input.take() {
            release_token_kind(&mut Some(TokenKind::Doctype(value)), &mut c).unwrap();
        }
        if session.release(&mut c).is_err() {
            // Successful production may use the final available credit.
            // Active disposal then retains its owner for terminal retry.
            assert!(session.storage.is_some());
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if stop.is_some() {
            assert_eq!(c.stop(), stop);
            assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        }
    }
}
#[test]
fn mixed_whitespace_split_preserves_utf16_and_each_exact_origin_after_newline_preprocessing() {
    use crate::parser::tokenizer::{
        states::HtmlTokenizerState, tests::LexerFixture, token::HtmlParseOriginKind,
    };
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
        "<a v='"
            .encode_utf16()
            .chain([0x20, 9, 13, 10, 0xd83d, 0xde00, 0xd800, 0xfeff])
            .chain("'>".encode_utf16())
            .collect::<Vec<_>>(),
    ));
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (value, tag_name, attribute_name) = lexical_attribute_text(&mut tokenizer, &mut c);
    tag_name.release(&mut c).unwrap();
    attribute_name.release(&mut c).unwrap();
    let mut input = Some(value);
    let prefix = split_leading_ascii_whitespace(&mut c, &mut input)
        .unwrap()
        .unwrap();
    assert_eq!(prefix.value.text().code_units(), [0x20, 9, 10]);
    assert_eq!(
        input.as_ref().unwrap().value.text().code_units(),
        [0xd83d, 0xde00, 0xd800, 0xfeff]
    );
    let origin = &prefix.origins.as_slice()[2];
    assert_eq!(
        (
            origin.start,
            origin.end,
            origin.source.start(),
            origin.source.end(),
            origin.kind
        ),
        (2, 3, 8, 10, HtmlParseOriginKind::Newline)
    );
    let ranges = input
        .as_ref()
        .unwrap()
        .origins
        .as_slice()
        .iter()
        .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
        .collect::<Vec<_>>();
    assert_eq!(ranges, [(0, 2, 10, 12), (2, 3, 12, 13), (3, 4, 13, 14)]);
    for origin in input.as_ref().unwrap().origins.as_slice() {
        assert_eq!(origin.source.source(), source.identity());
    }
    prefix.release(&mut c).unwrap();
    input.take().unwrap().release(&mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn failed_mixed_split_preserves_original_payload_capacity_and_source_origins() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<a v=' A'>");
    // Tag name + attribute name + two value units + tokenizer's real last
    // start-tag name occupy five units. Prefix fits at six; suffix does not.
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .buffered_utf16_units(6)
            .finish()
            .unwrap(),
        HtmlParseCancellation::new(),
    );
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let (value, tag_name, attribute_name) = lexical_attribute_text(&mut tokenizer, &mut c);
    let mut input = Some(value);
    let capacity = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .current;
    assert!(matches!(
        split_leading_ascii_whitespace(&mut c, &mut input),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::BufferedUtf16Units,
            ..
        }))
    ));
    assert_eq!(
        input.as_ref().unwrap().value.text().code_units(),
        [0x20, 0x41]
    );
    assert_eq!(input.as_ref().unwrap().origins.len(), 2);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current,
        capacity
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current,
        5
    );
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    release_token_kind(&mut input.map(TokenKind::Text), &mut c).unwrap();
    tag_name.release(&mut c).unwrap();
    attribute_name.release(&mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
}
#[test]
fn whitespace_identity_edges_and_unavailable_cut_preserve_the_original_owner() {
    let mut c = control();
    let mut input = Some(owned_text(&mut c, &[0x0b, 0x20, 0xfeff, 0]));
    assert!(
        split_leading_ascii_whitespace(&mut c, &mut input)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        input.as_ref().unwrap().value.text().code_units(),
        [0x0b, 0x20, 0xfeff, 0]
    );
    input.take().unwrap().release(&mut c).unwrap();
    let mut input = Some(owned_text(&mut c, &[9, 10, 12, 13, 32]));
    let prefix = split_leading_ascii_whitespace(&mut c, &mut input)
        .unwrap()
        .unwrap();
    assert!(input.is_none());
    prefix.release(&mut c).unwrap();
    // One aggregate origin contains both sides; no range is fabricated.
    let mut input = Some(owned_text(&mut c, &[32, 65]));
    assert!(matches!(
        split_leading_ascii_whitespace(&mut c, &mut input),
        Err(SessionError::OriginSplitUnavailable)
    ));
    assert_eq!(input.as_ref().unwrap().value.text().code_units(), [32, 65]);
    input.take().unwrap().release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn lexical_disposal_rejects_terminal_foreign_and_alias_owners_and_reclaims_once() {
    let mut c = control();
    let text = owned_text(&mut c, &[0xd800]);
    let alias = text.value.text().clone();
    let mut input = Some(TokenKind::Text(text));
    let mut foreign = control();
    let remaining = limits().work_credits()
        - foreign
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
    foreign
        .charge(HtmlParseWorkCategory::StateStep, remaining)
        .unwrap();
    foreign
        .charge(HtmlParseWorkCategory::RecordVisit, 1)
        .unwrap_err();
    assert!(release_token_kind(&mut input, &mut foreign).is_err());
    assert!(input.is_some());
    let remaining =
        limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
    c.charge(HtmlParseWorkCategory::StateStep, remaining)
        .unwrap();
    c.charge(HtmlParseWorkCategory::RecordVisit, 1).unwrap_err();
    assert_eq!(
        release_token_kind(&mut input, &mut c),
        Err(SessionError::AliasedPayload)
    );
    assert!(input.is_some());
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    let high_water = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water;
    drop(alias);
    release_token_kind(&mut input, &mut c).unwrap();
    assert!(input.is_none());
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water,
        high_water
    );
    assert_eq!(
        release_token_kind(&mut input, &mut c),
        Err(SessionError::Consumed)
    );
}
#[test]
fn newly_latched_disposal_work_or_cancel_keeps_input_for_terminal_retry() {
    for cancel in [false, true] {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let mut input = Some(TokenKind::Tag(owned_tag(&mut c, "a", &[("x", &[0xd800])])));
        if cancel {
            cancellation.cancel();
        } else {
            let remaining = limits().work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(HtmlParseWorkCategory::StateStep, remaining)
                .unwrap();
        }
        assert!(release_token_kind(&mut input, &mut c).is_err());
        assert!(input.is_some());
        let stop = c.stop();
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        release_token_kind(&mut input, &mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), stop);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    }
}
#[test]
fn late_lexical_attribute_disposal_stop_consumes_and_reclaims_the_original_payload() {
    let mut probe = control();
    let mut input = Some(TokenKind::Tag(owned_tag(
        &mut probe,
        "a",
        &[("x", &[0xd800])],
    )));
    release_token_kind(&mut input, &mut probe).unwrap();
    let total = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    storage_zero(&probe);
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .work_credits(total - 1)
            .finish()
            .unwrap(),
        HtmlParseCancellation::new(),
    );
    let mut input = Some(TokenKind::Tag(owned_tag(&mut c, "a", &[("x", &[0xd800])])));
    assert!(matches!(
        release_token_kind(&mut input, &mut c),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 1,
            ..
        }))
    ));
    assert!(input.is_none());
    storage_zero(&c);
    assert!(c.stop().is_some());
}
#[test]
fn active_lexical_validation_checks_actual_owners_and_cannot_resume_after_stop() {
    let mut c = control();
    let mut foreign = control();
    let mut input = Some(TokenKind::Comment(owned_text(&mut foreign, &[0xd800])));
    assert!(validate_token_kind(input.as_ref().unwrap(), &mut c).is_err());
    assert!(input.is_some());
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits);
    assert!(validate_token_kind(input.as_ref().unwrap(), &mut c).is_err());
    assert_eq!(c.stop(), stop);
    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
    release_token_kind(&mut input, &mut foreign).unwrap();
    storage_zero(&foreign);
    storage_zero(&c);
}
#[test]
fn recovered_pi_preflights_both_field_owners_before_consuming_any_input() {
    let mut c = control();
    let mut foreign = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let mut input = Some((
        owned_text(&mut c, &[0x78]),
        owned_text(&mut foreign, &[0xd800]),
    ));
    assert!(
        session
            .allocate_processing_instruction(&mut c, &mut input, None)
            .is_err()
    );
    let (target, data) = input.take().expect("both original fields retained");
    assert_eq!(target.value.text().code_units(), [0x78]);
    assert_eq!(data.value.text().code_units(), [0xd800]);
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        1
    );
    target.release(&mut c).unwrap();
    data.release(&mut foreign).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    storage_zero(&foreign);
}
