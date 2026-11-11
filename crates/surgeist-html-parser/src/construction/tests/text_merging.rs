use super::*;
use crate::HtmlParseCancellation;
use crate::construction::provenance::{GroupLeafCursor, LeafValue};

/// The units of each leaf of `group`, in order, read through the history leaf readers.
fn leaves(
    c: &mut ParseControl,
    session: &ParserArenaSession,
    group: &OriginGroupRef,
) -> Vec<Vec<u16>> {
    let storage = session.active().unwrap();
    let evidence = storage.evidence.as_ref().unwrap();
    let strings = storage.leaf_strings();
    let mut cursor = GroupLeafCursor::new(evidence, group, c).unwrap();
    let mut leaves = Vec::new();
    while let Some(leaf) = cursor.next(evidence, c).unwrap() {
        leaves.push(strings.units(c, leaf.value, leaf.units).unwrap().to_vec());
    }
    cursor.release(c).unwrap();
    leaves
}

/// A copy of the group a text's field holds, which the caller reads leaves through.
fn text_group(
    c: &mut ParseControl,
    session: &mut ParserArenaSession,
    node: &ParserNode,
) -> OriginGroupRef {
    let storage = session.active_mut().unwrap();
    let group = storage
        .payloads
        .detail_origins
        .get(c, FieldRef::Text(node.index))
        .unwrap()
        .expect("a group of the text");
    storage
        .evidence
        .as_ref()
        .unwrap()
        .copy_group_reference(c, group)
        .unwrap()
}

/// Whether each origin group of the evidence is retired, in group order.
fn retired_groups(session: &mut ParserArenaSession) -> Vec<bool> {
    session
        .active_mut()
        .unwrap()
        .evidence
        .as_mut()
        .unwrap()
        .groups_for_test()
        .iter()
        .map(|group| group.retirement.is_some())
        .collect()
}

/// The index of the group the table of origins holds for the text of `node`.
fn text_group_index(session: &ParserArenaSession, node: &ParserNode) -> Option<usize> {
    session
        .active()
        .unwrap()
        .payloads
        .detail_origins
        .peek(FieldRef::Text(node.index))
        .map(|group| group.index)
}

fn insert_all(
    c: &mut ParseControl,
    session: &mut ParserArenaSession,
    root: &ParserNode,
    pieces: &[&[u16]],
) -> ParserNode {
    let mut node = None;
    for piece in pieces {
        let mut input = Some(owned_text(c, piece));
        let inserted = session
            .insert_text(c, root, None, &mut input, None)
            .unwrap()
            .unwrap();
        assert!(input.is_none());
        node = Some(inserted._node);
    }
    node.unwrap()
}

#[test]
fn appended_text_copies_the_same_work_whatever_the_length_of_the_text_before_it() {
    let mut copied = Vec::new();
    for prefix in [8usize, 16, 32, 64, 128] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut expected = vec![0x61; prefix];
        expected[0] = 0xd800;
        expected[1] = 0;
        // A first append makes the room the next ones use: the capacity doubles.
        let first = insert_all(&mut c, &mut session, &root, &[&expected, &[0x62]]);
        expected.push(0x62);
        let mut work = 0;
        for unit in [0xdc00, 0, 0xd800, 0x63] {
            let mut input = Some(owned_text(&mut c, &[unit]));
            let before = c.usage().work(HtmlParseWorkCategory::CopyByte);
            let next = session
                .insert_text(&mut c, &root, None, &mut input, None)
                .unwrap()
                .unwrap();
            work += c.usage().work(HtmlParseWorkCategory::CopyByte) - before;
            assert_eq!(next._node.index, first.index);
            assert_eq!(next._offset, expected.len());
            expected.push(unit);
            assert_eq!(native_text(&session, &first), expected);
        }
        assert_eq!(
            session.active().unwrap().construction.usage().utf16_units(),
            prefix + 5
        );
        // The text holds its own units and nothing more.
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            (prefix + 5) as u64
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
        copied.push((prefix, work));
    }
    // The same four incoming units on a text with room, so the length before them adds no work.
    assert!(
        copied.iter().all(|(_, work)| *work == copied[0].1),
        "appended text copy work: {copied:?}"
    );
}

#[test]
fn character_insertion_merges_preceding_text_keeps_identity_and_the_units_of_its_data() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let mut first = Some(owned_text(&mut c, &[0xd800, 0x20]));
    let first = session
        .insert_text(&mut c, &root, None, &mut first, None)
        .unwrap()
        .unwrap();
    assert_eq!(first._offset, 0);
    let mut next = Some(owned_text(&mut c, &[0xd83d, 0xde00, 0]));
    let next = session
        .insert_text(&mut c, &root, None, &mut next, None)
        .unwrap()
        .unwrap();
    assert_eq!(next._node.index, first._node.index);
    assert_eq!(next._offset, 2);
    assert_eq!(
        native_text(&session, &first._node),
        [0xd800, 0x20, 0xd83d, 0xde00, 0]
    );
    assert_eq!(
        session.active().unwrap().construction.usage().utf16_units(),
        5
    );
    // The merged text holds five units; the two tokens it absorbed are not stored strings.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        5
    );
    // Four origin records are the two unique leaf atoms; three groups, two field causes,
    // merge and retirement give nine actual source-map entries.
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        9
    );
    let mut frozen = session.freeze(&mut c).unwrap();
    assert_eq!(frozen.fragment().unwrap().usage().utf16_units(), 5);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn the_leaves_of_a_text_merged_from_many_tokens_are_the_tokens_in_order() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let pieces: [&[u16]; 5] = [
        &[0x41],
        &[0xd800, 0],
        &[0x42, 0x43, 0x44],
        &[0xdc00],
        &[0xd83d, 0xde42],
    ];
    let node = insert_all(&mut c, &mut session, &root, &pieces);
    let group = text_group(&mut c, &mut session, &node);
    let merged: Vec<u16> = pieces
        .iter()
        .flat_map(|piece| piece.iter().copied())
        .collect();
    assert_eq!(native_text(&session, &node), merged);
    assert_eq!(
        leaves(&mut c, &session, &group),
        pieces
            .iter()
            .map(|piece| piece.to_vec())
            .collect::<Vec<_>>()
    );
    let storage = session.active().unwrap();
    let evidence = storage.evidence.as_ref().unwrap();
    let whole = evidence
        .materialize_group(&mut c, storage.leaf_strings(), &group)
        .unwrap();
    assert_eq!(whole.text().code_units(), merged);
    ParseEvidence::release_materialized_group(&mut c, &mut Some(whole)).unwrap();
    drop(group);
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn the_leaf_of_a_comment_a_processing_instruction_and_a_doctype_is_its_string() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source =
        crate::HtmlSource::utf8("<!--c--><?marker data?><!DOCTYPE html PUBLIC \"p\" \"s\">");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.initialize_parse_evidence(&mut c, &source).unwrap();
    let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let units = |text: &str| -> Vec<u16> { text.encode_utf16().collect() };
    let TokenKind::Comment(comment) = lexical_kind(&mut lexer, &mut c) else {
        panic!("comment");
    };
    let comment = session
        .allocate_comment(&mut c, &mut Some(comment), None)
        .unwrap();
    let TokenKind::ProcessingInstruction { target, data } = lexical_kind(&mut lexer, &mut c) else {
        panic!("processing instruction");
    };
    let instruction = session
        .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
        .unwrap();
    let TokenKind::Doctype(doctype) = lexical_kind(&mut lexer, &mut c) else {
        panic!("doctype");
    };
    let doctype = session
        .allocate_doctype(&mut c, &mut Some(doctype))
        .unwrap();
    let expected = [
        (FieldRef::Comment(comment.index), units("c")),
        (
            FieldRef::ProcessingInstructionTarget(instruction.index),
            units("marker"),
        ),
        (
            FieldRef::ProcessingInstructionData(instruction.index),
            units("data"),
        ),
        (FieldRef::DoctypeName(doctype.index), units("html")),
        (FieldRef::DoctypePublic(doctype.index), units("p")),
        (FieldRef::DoctypeSystem(doctype.index), units("s")),
    ];
    for (field, string) in expected {
        let storage = session.active_mut().unwrap();
        let group = storage
            .payloads
            .detail_origins
            .get(&mut c, field)
            .unwrap()
            .expect("a group of the field");
        let group = storage
            .evidence
            .as_ref()
            .unwrap()
            .copy_group_reference(&mut c, group)
            .unwrap();
        assert_eq!(leaves(&mut c, &session, &group), vec![string], "{field:?}");
        drop(group);
    }
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_refused_append_leaves_the_text_the_usage_and_the_origins_and_release_leaves_nothing() {
    // The cap holds the two units of the text and refuses the third.
    let selected = limits()
        .into_builder()
        .managed_utf16_units(2)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let node = insert_all(&mut c, &mut session, &root, &[&[0x61, 0x62]]);
    let origins = session.active().unwrap().payloads.detail_origins.len();
    // The text has the one group of its creation, and it is not retired.
    assert_eq!(origins, 1);
    assert_eq!(retired_groups(&mut session), [false]);
    assert_eq!(text_group_index(&session, &node), Some(0));
    let before = c.usage().clone();
    let mut input = Some(owned_text(&mut c, &[0x63]));
    // The token's own units count as buffered, below the managed cap.
    let result = session.insert_text(&mut c, &root, None, &mut input, None);
    let Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
        resource: HtmlParseResource::ManagedUtf16Units,
        limit: 2,
        current: 2,
        requested_delta: 1,
    })) = result
    else {
        panic!(
            "the append names the managed units it would exceed: {:?}",
            result.map(|_| ())
        );
    };
    assert!(input.is_none(), "the token is released with the refusal");
    assert_eq!(native_text(&session, &node), [0x61, 0x62]);
    assert_eq!(
        session.active().unwrap().construction.usage().utf16_units(),
        2
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        before
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current
    );
    assert_eq!(
        session.active().unwrap().payloads.detail_origins.len(),
        origins
    );
    // The merge made the group of the token and the merged group before the append was
    // refused, and retired the group of the text. The field names the group it had.
    assert_eq!(retired_groups(&mut session), [true, false, false]);
    assert_eq!(text_group_index(&session, &node), Some(0));
    // The token's leaf names the third unit, which the text does not hold. The parse has
    // stopped, so a control of another parse charges the reads.
    let mut reader = control();
    let storage = session.active().unwrap();
    let strings = storage.leaf_strings();
    let string = RecordString::of(FieldRef::Text(node.index)).unwrap();
    assert!(matches!(
        strings.units(&mut reader, LeafValue::Field { string, offset: 2 }, 1),
        Err(SessionError::WrongToken)
    ));
    assert_eq!(
        strings
            .units(&mut reader, LeafValue::Field { string, offset: 0 }, 2)
            .unwrap(),
        [0x61, 0x62]
    );
    if session.release(&mut c).is_err() {
        session.release(&mut c).unwrap();
    }
    storage_zero(&c);
}

#[test]
fn the_leaves_of_a_text_stay_valid_when_a_later_append_is_refused() {
    // The cap holds the three units of the first two tokens and refuses the fourth.
    let selected = limits()
        .into_builder()
        .managed_utf16_units(3)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let pieces: [&[u16]; 2] = [&[0x61, 0xd800], &[0xdc00]];
    let node = insert_all(&mut c, &mut session, &root, &pieces);
    let group = text_group(&mut c, &mut session, &node);
    let before = group.index;
    assert_eq!(
        leaves(&mut c, &session, &group),
        pieces
            .iter()
            .map(|piece| piece.to_vec())
            .collect::<Vec<_>>()
    );
    drop(group);
    let mut input = Some(owned_text(&mut c, &[0x62]));
    assert!(
        session
            .insert_text(&mut c, &root, None, &mut input, None)
            .is_err()
    );
    // The text keeps its units, and the field keeps the group whose leaves are ranges of them.
    assert_eq!(native_text(&session, &node), [0x61, 0xd800, 0xdc00]);
    let kept = session
        .active()
        .unwrap()
        .payloads
        .detail_origins
        .peek(FieldRef::Text(node.index))
        .expect("the group of the text");
    assert_eq!(kept.index, before);
    // Groups: the text's own (0), the first token's (1) and the first merged group (2, the
    // field's). The refused merge made the token's (3) and the merged group (4) and retired the
    // field's group (2), as the first merge retired the text's own (0).
    assert_eq!(before, 2);
    assert_eq!(
        retired_groups(&mut session),
        [true, false, true, false, false]
    );
    if session.release(&mut c).is_err() {
        session.release(&mut c).unwrap();
    }
    storage_zero(&c);
}

#[test]
fn a_refused_growth_leaves_the_text_the_usage_and_the_capacity_accounts() {
    // The bytes one text of one unit holds: a cap of exactly that leaves no room to grow.
    let mut probe = control();
    let mut session = ParserArenaSession::new(&mut probe, RootKind::Fragment).unwrap();
    let root = session.root(&mut probe).unwrap();
    insert_all(&mut probe, &mut session, &root, &[&[0x61]]);
    // The token to append counts as well, with its string.
    let token = owned_text(&mut probe, &[0x62]);
    let live = probe.usage().actual_capacity_bytes;
    token.release(&mut probe).unwrap();
    session.release(&mut probe).unwrap();
    storage_zero(&probe);

    let selected = limits()
        .into_builder()
        .managed_capacity_bytes(live)
        .finish()
        .unwrap();
    let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let node = insert_all(&mut c, &mut session, &root, &[&[0x61]]);
    let before = c.usage().clone();
    let mut input = Some(owned_text(&mut c, &[0x62]));
    assert_eq!(c.usage().actual_capacity_bytes, live);
    let result = session.insert_text(&mut c, &root, None, &mut input, None);
    assert!(
        matches!(
            result,
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }))
        ),
        "{:?}",
        result.map(|_| ())
    );
    assert_eq!(native_text(&session, &node), [0x61]);
    assert_eq!(
        session.active().unwrap().construction.usage().utf16_units(),
        1
    );
    for resource in [
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ManagedCapacityBytes,
    ] {
        assert_eq!(
            c.usage().resource(resource).current,
            before.resource(resource).current,
            "{resource:?}"
        );
    }
    if session.release(&mut c).is_err() {
        session.release(&mut c).unwrap();
    }
    storage_zero(&c);
}

#[test]
fn every_work_cut_of_a_merge_leaves_the_text_or_the_whole_merge_and_releases_to_zero() {
    fn run(cap: Option<u64>) -> (u64, u64) {
        let selected = limits()
            .into_builder()
            .work_credits(cap.unwrap_or(10_000_000))
            .finish()
            .unwrap();
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        let node = insert_all(&mut c, &mut session, &root, &[&[65, 66]]);
        let incoming = [0xd800, 0, 0xdc00].repeat(100);
        let mut input = Some(owned_text(&mut c, &incoming));
        // The text holds the one group of its creation, in one entry of the table of origins.
        assert_eq!(session.active().unwrap().payloads.detail_origins.len(), 1);
        assert_eq!(retired_groups(&mut session), [false]);
        let start = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let result = session.insert_text(&mut c, &root, None, &mut input, None);
        let end = c.usage().resource(HtmlParseResource::WorkCredits).current;
        match result {
            Ok(Some(inserted)) => {
                assert!(input.is_none());
                assert_eq!(inserted._node.index, node.index);
                assert_eq!(inserted._offset, 2);
                let mut expected = vec![65, 66];
                expected.extend_from_slice(&incoming);
                assert_eq!(native_text(&session, &node), expected);
                assert_eq!(
                    session.active().unwrap().construction.usage().utf16_units(),
                    302
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    302
                );
                // The merge made the token's group (1) and the merged group (2), retired the
                // group of the text, and the field names the merged group in its one entry.
                assert_eq!(session.active().unwrap().payloads.detail_origins.len(), 1);
                assert_eq!(retired_groups(&mut session), [true, false, false]);
                assert_eq!(text_group_index(&session, &node), Some(2));
            }
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })) => {
                // A cut before the token is taken leaves it with the caller.
                if let Some(text) = input.take() {
                    text.release(&mut c).unwrap();
                }
                assert_eq!(native_text(&session, &node), [65, 66]);
                assert_eq!(
                    session.active().unwrap().construction.usage().utf16_units(),
                    2
                );
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::ManagedUtf16Units)
                        .current,
                    2
                );
                // A cut leaves the one entry naming the group of the text. The merge made its
                // groups in order (the token's, then the merged group) and retired the group of
                // the text only after the merged group exists.
                assert_eq!(session.active().unwrap().payloads.detail_origins.len(), 1);
                assert_eq!(text_group_index(&session, &node), Some(0));
                let retired = retired_groups(&mut session);
                assert!(retired.len() <= 3, "{retired:?}");
                assert!(!retired[0] || retired.len() == 3, "{retired:?}");
                assert!(retired[1..].iter().all(|retired| !retired), "{retired:?}");
            }
            other => panic!(
                "only exact success or actual work refusal: {:?}",
                other.map(|_| ())
            ),
        }
        if session.release(&mut c).is_err() {
            // Exact parsing work need not include ordinary disposal work.
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        (start, end)
    }
    let (start, complete) = run(None);
    assert!(complete - start <= 32_768);
    for cap in start..complete {
        run(Some(cap));
    }
    run(Some(complete));
}

#[test]
fn increasing_whole_unicode_entity_feeds_have_linear_append_copy_and_lookup_work() {
    use crate::parser::tokenizer::{
        Tokenizer, TokenizerPull, states::HtmlTokenizerState, token::Event,
    };
    for repetitions in [8usize, 16, 32, 64, 128] {
        for utf16 in [false, true] {
            let source = if utf16 {
                crate::HtmlSource::utf16(DocumentString::from_utf16(
                    [65, 0xd800, 13, 10, 0xd83d, 0xde42].repeat(repetitions),
                ))
            } else {
                crate::HtmlSource::utf8("A&amp;🙂\r\n".repeat(repetitions))
            };
            let pattern: &[u16] = if utf16 {
                &[65, 0xd800, 10, 0xd83d, 0xde42]
            } else {
                &[65, 38, 0xd83d, 0xde42, 10]
            };
            let expected = pattern.repeat(repetitions);
            let expected_origins: Vec<_> = (0..repetitions)
                .flat_map(|n| {
                    let out = n * 5;
                    let raw = n * if utf16 { 6 } else { 12 };
                    let spans = if utf16 {
                        [(0, 1, 0, 1), (1, 2, 1, 2), (2, 3, 2, 4), (3, 5, 4, 6)]
                    } else {
                        [(0, 1, 0, 1), (1, 2, 1, 6), (2, 4, 6, 10), (4, 5, 10, 12)]
                    };
                    spans.map(|(a, b, c, d)| (out + a, out + b, raw + c, raw + d))
                })
                .collect();
            // Bounds cover the actual lexical atoms, groups and causes for this finite source.
            let n = source.len() as u64;
            let selected = limits()
                .into_builder()
                .source_units(n)
                .auxiliary_entries(32 * n + 64)
                .active_formatting_entries(32 * n + 64)
                .source_map_entries(32 * n + 64)
                .managed_capacity_bytes(32_768 * n + 1024)
                .peak_transient_bytes(65_536 * n + 2048)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            session.initialize_parse_evidence(&mut c, &source).unwrap();
            let root = session.root(&mut c).unwrap();
            let mut lexer =
                Tokenizer::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
            let mut texts = 0u64;
            let (mut copies, mut visits) = (0u64, 0u64);
            let mut node: Option<ParserNode> = None;
            let mut old_address = 0;
            let mut relocations = 0usize;
            // The loop ends only when the whole source is exhausted.
            loop {
                let pull = lexer
                    .next(
                        &mut c,
                        false,
                        session.active_mut().unwrap().evidence.as_mut().unwrap(),
                    )
                    .unwrap();
                match pull {
                    TokenizerPull::Exhausted => break,
                    TokenizerPull::Event(Event::Token(event)) => {
                        match event.into_token(&mut c).unwrap().kind {
                            TokenKind::Text(text) => {
                                let before_copy = c.usage().work(HtmlParseWorkCategory::CopyByte);
                                let before_visits =
                                    c.usage().work(HtmlParseWorkCategory::RecordVisit);
                                let inserted = session
                                    .insert_text(&mut c, &root, None, &mut Some(text), None)
                                    .unwrap()
                                    .unwrap();
                                copies +=
                                    c.usage().work(HtmlParseWorkCategory::CopyByte) - before_copy;
                                visits += c.usage().work(HtmlParseWorkCategory::RecordVisit)
                                    - before_visits;
                                if let Some(old) = &node {
                                    assert_eq!(inserted._node.index, old.index);
                                }
                                node = Some(inserted._node);
                                texts += 1;
                                let node = node.as_ref().unwrap();
                                let storage = session.active().unwrap();
                                let ConstructionNodeKind::Text(value) =
                                    storage.construction.node_at(node.index).unwrap().kind()
                                else {
                                    panic!("text")
                                };
                                let address = value.code_units().as_ptr() as usize;
                                if old_address != 0 && address != old_address {
                                    relocations += 1;
                                }
                                old_address = address;
                            }
                            TokenKind::Eof => {}
                            _ => panic!("text-only source"),
                        }
                    }
                    TokenizerPull::Event(event) => event.release(&mut c).unwrap(),
                }
            }
            let node = node.unwrap();
            assert_eq!(native_text(&session, &node), expected);
            // Fixed protocol and header copies are bounded per actual Text; the incoming
            // units add a linear term.
            assert!(
                copies <= 4096 * texts + 2 * expected.len() as u64,
                "copy={copies}, texts={texts}"
            );
            assert!(visits <= 256 * texts + 64, "visits={visits}, texts={texts}");
            // The capacity at least doubles at each relocation, so the text moves a logarithmic
            // number of times.
            let doublings = (usize::BITS - expected.len().leading_zeros()) as usize;
            assert!(
                relocations <= doublings,
                "relocations={relocations} for {} units",
                expected.len()
            );
            let group = text_group(&mut c, &mut session, &node);
            let storage = session.active().unwrap();
            let evidence = storage.evidence.as_ref().unwrap();
            let origins = evidence.observed_origins(&group, &mut c).unwrap();
            assert_eq!(
                origins
                    .iter()
                    .map(|o| (o.start, o.end, o.source.start(), o.source.end()))
                    .collect::<Vec<_>>(),
                expected_origins
            );
            let historical = evidence
                .materialize_group(&mut c, storage.leaf_strings(), &group)
                .unwrap();
            assert_eq!(historical.text().code_units(), expected);
            ParseEvidence::release_materialized_group(&mut c, &mut Some(historical)).unwrap();
            // The text holds its own units: the tokens it absorbed are not stored strings.
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                expected.len() as u64
            );
            drop(group);
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
}

/// A comment and a text merged from two tokens in a fragment whose parse records `detail`, and
/// the `AuxiliaryEntries` the parse then holds, with the entries of the table of origins.
fn comment_and_merged_text(c: &mut ParseControl) -> (u64, usize, usize) {
    let mut session = ParserArenaSession::new(c, RootKind::Fragment).unwrap();
    let root = session.root(c).unwrap();
    let mut comment = Some(owned_text(c, &[0x63]));
    session.allocate_comment(c, &mut comment, None).unwrap();
    insert_all(c, &mut session, &root, &[&[0x61], &[0x62]]);
    let aux = c
        .usage()
        .resource(HtmlParseResource::AuxiliaryEntries)
        .current;
    let table = &session.active().unwrap().payloads.detail_origins;
    let measured = (aux, table.len(), table.capacity_bytes());
    session.release(c).unwrap();
    storage_zero(c);
    measured
}

#[test]
fn the_groups_of_text_and_comment_fields_are_entries_of_the_table_of_origins() {
    let mut plain = control_without_detail();
    let (without, entries, bytes) = comment_and_merged_text(&mut plain);
    // A parse that records no graph holds no group and allocates no table.
    assert_eq!((entries, bytes), (0, 0));
    let mut detailed = control();
    let (with, entries, bytes) = comment_and_merged_text(&mut detailed);
    // The comment and the merged text each hold one group, and each group is one entry.
    assert_eq!(entries, 2);
    assert_eq!(with, without + 2);
    // A slot for each node position up to the text's, a run for each of the two records and
    // their entries, each at most doubled by the spare room of its vector.
    let slot = std::mem::size_of::<usize>();
    let run = std::mem::size_of::<super::super::detail_origins::Run>();
    let entry = std::mem::size_of::<super::super::detail_origins::DetailOrigin>();
    assert!(bytes <= 2 * (3 * slot + 2 * run + 2 * entry));
}

#[test]
fn a_leaf_names_the_string_of_a_record_in_eight_bytes_and_a_field_round_trips() {
    use super::super::provenance::LeafValue;
    assert_eq!(std::mem::size_of::<RecordString>(), 8);
    // A leaf holds its string in at most three words, so that a history of many leaves is small.
    assert!(std::mem::size_of::<LeafValue>() <= 24);
    for field in [
        FieldRef::Text(7),
        FieldRef::Comment(8),
        FieldRef::ProcessingInstructionTarget(9),
        FieldRef::ProcessingInstructionData(10),
        FieldRef::DoctypeName(11),
        FieldRef::DoctypePublic(12),
        FieldRef::DoctypeSystem(u32::MAX as usize),
    ] {
        assert_eq!(
            RecordString::of(field).map(RecordString::field),
            Some(field)
        );
    }
    // Fields that registry rows hold, and a record position that does not fit 32 bits, name no
    // record string.
    assert_eq!(RecordString::of(FieldRef::ElementName(1)), None);
    assert_eq!(RecordString::of(FieldRef::AttributeValue(1, 0)), None);
    assert_eq!(RecordString::of(FieldRef::Detached), None);
    assert_eq!(
        RecordString::of(FieldRef::Text(u32::MAX as usize + 1)),
        None
    );
}
