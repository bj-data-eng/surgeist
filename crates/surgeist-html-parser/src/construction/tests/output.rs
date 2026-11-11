use super::*;
use surgeist_html::{
    construction::HtmlDocumentMode, elements::form::HtmlNativeFormAssociationKind,
    values::Namespace,
};

#[test]
fn output_reads_actual_descendants_after_merge_move_and_freeze_without_raw_cache() {
    let source = crate::HtmlSource::utf8(
        "<output>A<span>B&amp;C</span>D<!--ignored--><?ok ignored?><span>E</span>F",
    );
    let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
    let mut a = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &output, None, &mut a, None)
        .unwrap();
    let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
    let span = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut tag,
            &output,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &output, &span, None, None).unwrap();
    // Data emits one code point per Text token, including the reference.
    for _ in 0..3 {
        let mut b = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &span, None, &mut b, None)
            .unwrap();
    }
    release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
    let mut d = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &output, None, &mut d, None)
        .unwrap();
    let TokenKind::Comment(comment) = lexical_kind(&mut lexer, &mut c) else {
        panic!("real comment")
    };
    let comment = session
        .allocate_comment(&mut c, &mut Some(comment), None)
        .unwrap();
    session
        .insert(&mut c, &output, &comment, None, None)
        .unwrap();
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c) else {
        panic!("real PI")
    };
    let pi = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    session.insert(&mut c, &output, &pi, None, None).unwrap();
    let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
    let last = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut tag,
            &output,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &output, &last, None, None).unwrap();
    let mut e = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &last, None, &mut e, None)
        .unwrap();
    release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
    let mut f = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &last, None, &mut f, None)
        .unwrap();
    lexer.release(&mut c).unwrap();

    assert_eq!(
        session
            .output_state(&mut c, &output)
            .unwrap()
            .default_value_override,
        HtmlParseOutputOverride::Null
    );
    let mut current = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_output_value(&mut c, &output, &mut current)
        .unwrap();
    assert_eq!(current.units(), [65, 66, 38, 67, 68, 69, 70]);
    current.clear(&mut c).unwrap();
    session
        .append_output_default_value(&mut c, &output, &mut current)
        .unwrap();
    assert_eq!(current.units(), [65, 66, 38, 67, 68, 69, 70]);
    session.remove(&mut c, &span, None).unwrap();
    current.clear(&mut c).unwrap();
    session
        .append_output_value(&mut c, &output, &mut current)
        .unwrap();
    assert_eq!(current.units(), [65, 68, 69, 70]);
    session.insert(&mut c, &last, &span, None, None).unwrap();
    current.clear(&mut c).unwrap();
    session
        .append_output_value(&mut c, &output, &mut current)
        .unwrap();
    assert_eq!(current.units(), [65, 68, 69, 70, 66, 38, 67]);
    current.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let id = frozen.document().unwrap().root().children().next().unwrap();
    let mut value = Utf16Buffer::new(&mut c).unwrap();
    frozen.append_output_value(&mut c, &id, &mut value).unwrap();
    assert_eq!(value.units(), [65, 68, 69, 70, 66, 38, 67]);
    value.clear(&mut c).unwrap();
    frozen
        .append_output_default_value(&mut c, &id, &mut value)
        .unwrap();
    assert_eq!(value.units(), [65, 68, 69, 70, 66, 38, 67]);
    assert_eq!(
        frozen
            .output_state(&mut c, &id)
            .unwrap()
            .default_value_override,
        HtmlParseOutputOverride::Null
    );
    value.release(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn output_observation_exact_40_unit_cap_or_sticky_20_unit_owned_prefix() {
    for cap in [39, 40] {
        let literal = format!(
            "<output><span>{}</span><span>{}</span>",
            "A".repeat(20),
            "B".repeat(20)
        );
        let source = crate::HtmlSource::utf8(literal);
        let (mut c, mut session, output, mut lexer) = output_owner(
            &source,
            limits()
                .into_builder()
                .buffered_utf16_units(cap)
                // Forty insertion batches retain their merge and
                // retirement histories; this test still isolates the exact
                // forty-unit observation buffer, not a work-credit cap.
                .work_credits(20_000_000)
                .finish()
                .unwrap(),
        );
        for _ in 0..2 {
            let mut tag = Some(next_fixture_tag(&mut lexer, &mut c, &[115, 112, 97, 110]));
            let span = session
                .allocate_html_tag_for_creation(
                    &mut c,
                    &mut tag,
                    &output,
                    None,
                    crate::construction::CreationContext::Ordinary,
                )
                .unwrap();
            session.insert(&mut c, &output, &span, None, None).unwrap();
            for _ in 0..20 {
                let mut text = next_text(&mut lexer, &mut c);
                session
                    .insert_text(&mut c, &span, None, &mut text, None)
                    .unwrap();
            }
            release_token_kind(&mut Some(lexical_kind(&mut lexer, &mut c)), &mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        let mut buffer = Utf16Buffer::new(&mut c).unwrap();
        let result = session.append_output_value(&mut c, &output, &mut buffer);
        if cap == 40 {
            result.unwrap();
            assert_eq!(buffer.units(), [&[65; 20][..], &[66; 20][..]].concat());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                40
            );
        } else {
            let stop = c.stop().unwrap();
            assert_eq!(result, Err(SessionError::Stop(stop)));
            assert_eq!(
                stop,
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::BufferedUtf16Units,
                    limit: 39,
                    current: 20,
                    requested_delta: 20,
                }
            );
            assert_eq!(buffer.units(), &[65; 20]);
            assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==stop));
            assert_eq!(
                session.append_output_default_value(&mut c, &output, &mut buffer),
                Err(SessionError::Stop(stop))
            );
        }
        buffer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn output_observations_reject_foreign_destination_and_unavailable_or_foreign_nodes() {
    let source = crate::HtmlSource::utf8("<output>A");
    let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
    let mut text = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &output, None, &mut text, None)
        .unwrap();
    lexer.release(&mut c).unwrap();
    let mut other = control();
    let mut foreign = Utf16Buffer::new(&mut other).unwrap();
    foreign.push_unit(81, &mut other).unwrap();
    assert!(matches!(
        session.append_output_value(&mut c, &output, &mut foreign),
        Err(SessionError::Stop(_))
    ));
    assert_eq!(foreign.units(), [81]);
    assert!(c.stop().is_some());
    foreign.release(&mut other).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    storage_zero(&other);

    let (mut c, mut session, output, lexer) = output_owner(&source, limits());
    lexer.release(&mut c).unwrap();
    let mut second =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let wrong = second.root(&mut c).unwrap();
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    assert_eq!(
        session.append_output_value(&mut c, &wrong, &mut buffer),
        Err(SessionError::ForeignNode)
    );
    assert!(buffer.units().is_empty());
    let imported = session
        .allocate_element(&mut c, element(Namespace::html(), "output"))
        .unwrap();
    assert_eq!(
        session.output_state(&mut c, &imported),
        Err(SessionError::OutputMetadataUnavailable)
    );
    assert_eq!(
        session
            .output_state(&mut c, &output)
            .unwrap()
            .default_value_override,
        HtmlParseOutputOverride::Null
    );
    buffer.release(&mut c).unwrap();
    second.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn output_metadata_slots_reject_before_lexical_take_with_exact_auxiliary_boundary() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Three existing native records own three append-index entries.
    for cap in [20, 21] {
        let source = crate::HtmlSource::utf8("<output>");
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .auxiliary_entries(cap)
                .active_formatting_entries(cap)
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
        let mut tag = Some(next_fixture_tag(
            &mut lexer,
            &mut c,
            &[111, 117, 116, 112, 117, 116],
        ));
        session.allocate_implicit_body(&mut c, None).unwrap();
        // Root/html/body retain two independent creation rows in addition to their previous 18 entries;
        // Output next requires document/native preflight3 before its own creation, state and association rows.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            20
        );
        let backing = tag.as_ref().unwrap().name.value.text().block().address();
        assert!(matches!(
            session.allocate_html_tag_for_creation(
                &mut c,
                &mut tag,
                &html,
                None,
                crate::construction::CreationContext::Ordinary
            ),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                ..
            }))
        ));
        assert_eq!(
            tag.as_ref().unwrap().name.value.text().block().address(),
            backing
        );
        release_token_kind(&mut Some(TokenKind::Tag(tag.take().unwrap())), &mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn output_cancellation_preserves_owned_scratch_and_blocks_metadata_freeze_without_reset() {
    let source = crate::HtmlSource::utf8("<output>A");
    let cancellation = HtmlParseCancellation::new();
    let (mut c, mut session, output, mut lexer) =
        output_owner_cancellable(&source, limits(), cancellation.clone());
    let mut text = next_text(&mut lexer, &mut c);
    session
        .insert_text(&mut c, &output, None, &mut text, None)
        .unwrap();
    lexer.release(&mut c).unwrap();
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    buffer.push_unit(81, &mut c).unwrap();
    cancellation.cancel();
    assert_eq!(
        session.append_output_value(&mut c, &output, &mut buffer),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(buffer.units(), [81]);
    assert_eq!(
        session.output_state(&mut c, &output),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert!(matches!(
        session.freeze(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    buffer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    storage_zero(&c);
}

#[test]
fn output_initial_empty_reset_precedes_parser_association_and_cancellation_retains_both_slots() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><output>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut form = Some(next_fixture_tag(&mut lexer, &mut c, &[102, 111, 114, 109]));
    let form = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut form,
            &root,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &root, &form, None, None).unwrap();
    let mut tag = Some(next_fixture_tag(
        &mut lexer,
        &mut c,
        &[111, 117, 116, 112, 117, 116],
    ));
    session.active_mut().unwrap().cancel_after_output_reset = Some(cancellation);
    assert!(matches!(
        session.allocate_html_tag_for_creation(
            &mut c,
            &mut tag,
            &form,
            Some(&form),
            crate::construction::CreationContext::Ordinary
        ),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    assert!(tag.is_none());
    let storage = session.active().unwrap();
    let output_slot = storage
        .payloads
        .values
        .iter()
        .find_map(|p| match p {
            RetainedPayload::Output {
                node: Some(node),
                state,
            } => Some((*node, *state)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        output_slot.1.default_value_override,
        HtmlParseOutputOverride::Null
    );
    assert!(
        storage
            .construction
            .node_at(output_slot.0)
            .expect("retained construction position")
            .child_count()
            == 0
    );
    assert!(
        storage
            .construction
            .node_at(output_slot.0)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position())
            .is_none()
    );
    assert!(storage.payloads.values.iter().any(|p| matches!(p,
        RetainedPayload::FormAssociation{node:None,state}
        if state.kind==HtmlNativeFormAssociationKind::Output && state.owner.is_none()
           && !state.parser_inserted && state.evidence==HtmlParseFormEvidence::Created
    )));
    assert!(matches!(
        session.freeze(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    storage_zero(&c);
}

#[test]
fn output_single_text_read_has_exact_physical_work_and_one_short_terminal_cleanup() {
    for remaining in [24, 25] {
        let source = crate::HtmlSource::utf8("<output>A");
        let (mut c, mut session, output, mut lexer) = output_owner(&source, limits());
        let mut text = next_text(&mut lexer, &mut c);
        session
            .insert_text(&mut c, &output, None, &mut text, None)
            .unwrap();
        lexer.release(&mut c).unwrap();
        let mut destination = Utf16Buffer::new(&mut c).unwrap();
        destination.push_unit(81, &mut c).unwrap();
        destination.clear(&mut c).unwrap(); // capacity for1 exists before measured copying.
        // Getter visits: readiness1 + idle7 + index2; root Document/self-binding2 + Output node-binding1 + Output slot1,
        // root+child2, Text1, leaf/parent/child-list/sibling4, and the creation row1 =22.
        // Copies are one finite state byte plus one UTF16 unit =3.
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let maximum = limits().work_credits();
        c.charge(HtmlParseWorkCategory::StateStep, maximum - used - remaining)
            .unwrap();
        let before = c.usage().clone();
        let result = session.append_output_value(&mut c, &output, &mut destination);
        assert_eq!(destination.units(), [65]);
        if remaining == 25 {
            result.unwrap();
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::RecordVisit)
                    - before.work(HtmlParseWorkCategory::RecordVisit),
                22
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::CopyByte)
                    - before.work(HtmlParseWorkCategory::CopyByte),
                3
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::InspectByte)
                    - before.work(HtmlParseWorkCategory::InspectByte),
                0
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::RecordWrite)
                    - before.work(HtmlParseWorkCategory::RecordWrite),
                0
            );
            assert_eq!(
                c.usage().work(HtmlParseWorkCategory::StateStep)
                    - before.work(HtmlParseWorkCategory::StateStep),
                0
            );
            assert!(c.stop().is_none());
            c.charge(HtmlParseWorkCategory::StateStep, 1).unwrap_err(); // terminal cleanup uses no useful work.
        } else {
            assert_eq!(result, Err(SessionError::Stop(c.stop().unwrap())));
        }
        let stop = c.stop().unwrap();
        assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==stop));
        destination.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(stop));
        storage_zero(&c);
    }
}
