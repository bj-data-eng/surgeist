use super::*;
use surgeist_html::{construction::HtmlNodeKind, values::DocumentString};

#[test]
fn recovered_end_br_discards_actual_attributes_and_keeps_name_origin_and_native_empty_attrs() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("</BR x=kept x=ignored>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("tag")
    };
    assert_eq!(tag.kind, TagKind::End);
    assert!(tag.had_duplicate_attributes);
    let name = tag.name.value.text().code_units().as_ptr();
    let origins = tag
        .name
        .origins
        .as_slice()
        .iter()
        .map(|o| (o.start, o.end, o.source.clone(), o.kind))
        .collect::<Vec<_>>();
    let mut input = Some(tag);
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    session.recover_br_end_tag(&mut c, &mut input).unwrap();
    let tag = input.as_ref().unwrap();
    assert_eq!(tag.kind, TagKind::Start);
    assert_eq!(tag.name.value.text().code_units().as_ptr(), name);
    assert_eq!(
        tag.name
            .origins
            .as_slice()
            .iter()
            .map(|o| (o.start, o.end, o.source.clone(), o.kind))
            .collect::<Vec<_>>(),
        origins
    );
    assert_eq!(tag.attributes.len(), 0);
    assert!(tag.had_duplicate_attributes);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::TokenAttributes)
            .current,
        0
    );
    let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
    let root = session.root(&mut c).unwrap();
    session.insert(&mut c, &root, &node, None, None).unwrap();
    tokenizer.release(&mut c).unwrap();
    let mut arena = session.freeze(&mut c).unwrap();
    assert_eq!(locals(arena.fragment().unwrap()), ["br"]);
    let fragment = arena.fragment().unwrap();
    let id = fragment.root().children().next().unwrap();
    let HtmlNodeKind::Element(element) = fragment.node(&id).unwrap().kind() else {
        panic!("br")
    };
    assert!(element.attributes().is_empty());
    arena.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn end_br_alias_foreign_and_cancel_reject_before_discarding_any_actual_field() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("</br x=kept>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(tag);
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let alias = input.as_ref().unwrap().attributes.as_slice()[0]
        .value
        .value
        .text()
        .clone();
    let capacity = c.usage().actual_capacity_bytes;
    assert_eq!(
        session.recover_br_end_tag(&mut c, &mut input),
        Err(SessionError::AliasedPayload)
    );
    assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
    assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
    assert_eq!(c.usage().actual_capacity_bytes, capacity);
    drop(alias);
    let mut foreign = control();
    foreign.admit_source(&source).unwrap();
    assert!(matches!(
        session.recover_br_end_tag(&mut foreign, &mut input),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
    cancellation.cancel();
    assert_eq!(
        session.recover_br_end_tag(&mut c, &mut input),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
    assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
    release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}
#[test]
fn end_br_work_stops_retain_preflight_input_or_finish_all_disposal_without_start_reprocessing() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("</br x=one y=two>");
    let mut required = None;
    for remaining in [
        None,
        Some(0),
        Some(1),
        Some(100),
        Some(u64::MAX - 1),
        Some(u64::MAX),
    ] {
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let name = tag.name.value.text().code_units().as_ptr();
        let mut input = Some(tag);
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let remaining = remaining.map(|value| {
            if value >= u64::MAX - 1 {
                required.unwrap() - (u64::MAX - value)
            } else {
                value
            }
        });
        if let Some(remaining) = remaining {
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                limits().work_credits() - used - remaining,
            )
            .unwrap();
        }
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let result = session.recover_br_end_tag(&mut c, &mut input);
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .name
                .value
                .text()
                .code_units()
                .as_ptr(),
            name
        );
        if remaining.is_none() {
            result.unwrap();
            required = Some(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
        } else if remaining == required {
            result.unwrap();
            assert_eq!(input.as_ref().unwrap().kind, TagKind::Start);
            assert_eq!(input.as_ref().unwrap().attributes.len(), 0);
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            assert_eq!(input.as_ref().unwrap().kind, TagKind::End);
            if remaining == required.map(|work| work - 1) {
                // Final empty pop observes the first stop after both attrs
                // were disposed. Empty lexical capacity still gets released.
                assert_eq!(input.as_ref().unwrap().attributes.len(), 0);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::TokenAttributes)
                        .current,
                    0
                );
            } else if remaining.is_some_and(|credits| credits <= 1) {
                assert_eq!(input.as_ref().unwrap().attributes.len(), 2);
            } else {
                // An intermediate stop may precede the slot transfer or
                // occur in its complete terminal disposal. It cannot leave
                // one partially disposed attribute in the lexical owner.
                let attributes = input.as_ref().unwrap().attributes.len();
                assert!(matches!(attributes, 0 | 2));
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::TokenAttributes)
                        .current,
                    attributes as u64
                );
            }
            let stop = c.stop().unwrap();
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        }
        let stop = c.stop();
        let mut kind = input.take().map(TokenKind::Tag);
        if release_token_kind(&mut kind, &mut c).is_err() {
            release_token_kind(&mut kind, &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if stop.is_some() {
            assert_eq!(c.stop(), stop);
        }
    }
}
#[test]
fn body_merge_qualifies_both_actual_local_names_and_preserves_import_refusal() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<body a=x>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(tag);
    let original = input
        .as_ref()
        .unwrap()
        .name
        .value
        .text()
        .code_units()
        .as_ptr();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let html = session.allocate_implicit_html(&mut c).unwrap();
    let body = session.allocate_implicit_body(&mut c, None).unwrap();
    assert_eq!(
        session.add_body_attributes_if_missing(&mut c, &html, &mut input),
        Err(SessionError::WrongToken)
    );
    assert_eq!(
        session.add_attributes_if_missing(&mut c, &body, &mut input),
        Err(SessionError::WrongToken)
    );
    assert_eq!(
        input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr(),
        original
    );
    let imported = session
        .allocate_element(&mut c, super::tests::html("body"))
        .unwrap();
    assert_eq!(
        session.active().unwrap().standalone_ineligibility,
        Some(HtmlParseUnsupportedState::Provenance)
    );
    assert_eq!(
        session.add_body_attributes_if_missing(&mut c, &imported, &mut input),
        Err(SessionError::AttributeStorageUnavailable)
    );
    assert_eq!(
        input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr(),
        original
    );
    session.allocate_implicit_p(&mut c, None).unwrap();
    session
        .add_body_attributes_if_missing(&mut c, &body, &mut input)
        .unwrap();
    assert_eq!(
        session.active().unwrap().standalone_ineligibility,
        Some(HtmlParseUnsupportedState::Provenance)
    );
    assert!(input.is_none());
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn body_merge_actual_attribute_cap_preserves_old_order_and_reclaims_staged_payloads() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<body id=old><body a=one b=two>");
    for cap in [2, 3] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .managed_attributes(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let body = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(second);
        let result = session.add_body_attributes_if_missing(&mut c, &body, &mut input);
        let HtmlNodeKind::Element(element) = &session
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
            element.attributes().as_slice()[0]
                .name()
                .local_name()
                .code_units(),
            [105, 100]
        );
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            [111, 108, 100]
        );
        if cap == 3 {
            result.unwrap();
            assert_eq!(element.attributes().len(), 3);
            assert!(input.is_none());
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedAttributes,
                    ..
                }))
            ));
            assert_eq!(element.attributes().len(), 1);
            let stop = c.stop().unwrap();
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
        }
        if let Some(tag) = input.take() {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn body_partition_wide_alternation_classifies_each_actual_unit_once() {
    let source = crate::HtmlSource::utf8(format!("<a v='{}'>", "A ".repeat(256)));
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .buffered_utf16_units(2000)
            .source_map_entries(2000)
            .work_credits(100_000_000)
            .finish()
            .unwrap(),
        HtmlParseCancellation::new(),
    );
    let mut input = Some(body_lexical_value(&source, &mut c));
    let reads = c.usage().work(HtmlParseWorkCategory::InspectByte);
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::InspectByte) - reads,
        1024
    );
    for index in 0..512 {
        let run = runs.take_next(&mut c).unwrap().unwrap();
        assert_eq!(
            run.kind,
            if index % 2 == 0 {
                BodyTextKind::Other
            } else {
                BodyTextKind::Whitespace
            }
        );
        assert_eq!(
            run.text.value.text().code_units(),
            [if index % 2 == 0 { 65 } else { 32 }]
        );
        assert_eq!(
            (
                run.text.origins.as_slice()[0].source.start(),
                run.text.origins.as_slice()[0].source.end()
            ),
            (index + 6, index + 7)
        );
        run.text.release(&mut c).unwrap();
    }
    assert!(runs.take_next(&mut c).unwrap().is_none());
    runs.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn body_partition_actual_origin_and_run_slots_require_their_coexisting_caps() {
    let source = crate::HtmlSource::utf8("<a v='A B C D'>");
    // Root child-storage plus six slots each for real html/body/p owners
    // coexist with seven output runs. These caps also admit the actual
    // lexer's seventeen-node attribute trie and its returned token record.
    // Retaining the lexer across the operation adds 22 real lexical rows:
    // two occurrence/cause pairs and 18 syntax/name/value parts. Seven
    // input origins coexist with seven partition origins (22+7+7=36).
    for (maps, auxiliary, succeeds) in [(35, 30, false), (36, 29, false), (36, 30, true)] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .source_map_entries(maps)
                .auxiliary_entries(auxiliary)
                .active_formatting_entries(auxiliary)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let (value, tag, attribute) = lexical_attribute_text(&mut tokenizer, &mut c);
        tag.release(&mut c).unwrap();
        attribute.release(&mut c).unwrap();
        let mut input = Some(value);
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        let body = session.allocate_implicit_body(&mut c, None).unwrap();
        let p = session.allocate_implicit_p(&mut c, None).unwrap();
        let root = session.root(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        session.insert(&mut c, &html, &body, None, None).unwrap();
        session.insert(&mut c, &body, &p, None, None).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            23
        );
        let capacity = c.usage().actual_capacity_bytes;
        let result = partition_body_text(&mut c, &mut input);
        assert_eq!(result.is_ok(), succeeds);
        if let Ok(mut runs) = result {
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .high_water,
                36
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .high_water,
                30
            );
            runs.release(&mut c).unwrap();
        } else {
            assert_eq!(
                input.as_ref().unwrap().value.text().code_units(),
                [65, 32, 66, 32, 67, 32, 68]
            );
            assert_eq!(c.usage().actual_capacity_bytes, capacity);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                29
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                23
            );
            release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn body_runs_last_cleanup_stop_reclaims_all_storage_and_exposes_completed_ownership() {
    let source = crate::HtmlSource::utf8("<a v='A B'>");
    let mut probe = control();
    let mut input = Some(body_lexical_value(&source, &mut probe));
    let mut runs = partition_body_text(&mut probe, &mut input).unwrap();
    let before = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    runs.release(&mut probe).unwrap();
    let release_work = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    let mut c = control();
    let mut input = Some(body_lexical_value(&source, &mut c));
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
    c.charge(
        HtmlParseWorkCategory::StateStep,
        limits().work_credits() - used - release_work + 1,
    )
    .unwrap();
    assert!(matches!(
        runs.release(&mut c),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        }))
    ));
    assert!(runs.entries.is_none());
    runs.release(&mut c).unwrap();
    storage_zero(&c);
    assert!(c.stop().is_some());
}
#[test]
fn first_lf_work_and_unit_stops_keep_original_input_and_release_every_staged_owner() {
    let source = crate::HtmlSource::utf8("<a v='\nA B'>");
    let mut probe = control();
    let mut input = Some(body_lexical_value(&source, &mut probe));
    let before = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let prefix = split_first_lf(&mut probe, &mut input).unwrap().unwrap();
    let required = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    prefix.release(&mut probe).unwrap();
    input.take().unwrap().release(&mut probe).unwrap();
    storage_zero(&probe);
    for remaining in [0, 1, required / 2, required - 1, required] {
        let mut c = control();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let original = input.as_ref().unwrap().value.text().code_units().as_ptr();
        let capacity = c.usage().actual_capacity_bytes;
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - remaining,
        )
        .unwrap();
        let result = split_first_lf(&mut c, &mut input);
        assert_eq!(result.is_ok(), remaining == required);
        if let Ok(Some(prefix)) = result {
            prefix.release(&mut c).unwrap();
            input.take().unwrap().release(&mut c).unwrap();
        } else {
            assert_eq!(
                input.as_ref().unwrap().value.text().code_units().as_ptr(),
                original
            );
            assert_eq!(c.usage().actual_capacity_bytes, capacity);
            release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
        }
        storage_zero(&c);
    }
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .buffered_utf16_units(7)
            .finish()
            .unwrap(),
        HtmlParseCancellation::new(),
    );
    let mut input = Some(body_lexical_value(&source, &mut c));
    assert!(matches!(
        split_first_lf(&mut c, &mut input),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::BufferedUtf16Units,
            ..
        }))
    ));
    assert_eq!(
        input.as_ref().unwrap().value.text().code_units(),
        [10, 65, 32, 66]
    );
    release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn recovered_image_keeps_actual_attributes_and_exact_original_name_span() {
    use crate::parser::tokenizer::{
        states::HtmlTokenizerState, tests::LexerFixture, token::HtmlParseOriginKind,
    };
    let source = crate::HtmlSource::utf8("<ImAgE id=x />");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(tag);
    let value = input.as_ref().unwrap().attributes.as_slice()[0]
        .value
        .value
        .text()
        .code_units()
        .as_ptr();
    let self_closing = input.as_ref().unwrap().self_closing;
    let duplicate = input.as_ref().unwrap().had_duplicate_attributes;
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let reads = c.usage().work(HtmlParseWorkCategory::InputUnitRead);
    session
        .recover_image_tag(&mut c, &source, &mut input)
        .unwrap();
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::InputUnitRead) - reads,
        2
    );
    let tag = input.as_ref().unwrap();
    assert_eq!(tag.name.value.text().code_units(), [105, 109, 103]);
    assert_eq!(
        (tag.kind, tag.self_closing, tag.had_duplicate_attributes),
        (TagKind::Start, self_closing, duplicate)
    );
    assert_eq!(
        tag.attributes.as_slice()[0]
            .value
            .value
            .text()
            .code_units()
            .as_ptr(),
        value
    );
    let origin = &tag.name.origins.as_slice()[0];
    assert_eq!(
        (
            origin.start,
            origin.end,
            origin.source.start(),
            origin.source.end(),
            origin.kind
        ),
        (0, 3, 1, 6, HtmlParseOriginKind::RecoverySyntax)
    );
    let node = session.allocate_html_tag(&mut c, &mut input).unwrap();
    assert!(input.is_none());
    let root = session.root(&mut c).unwrap();
    session.insert(&mut c, &root, &node, None, None).unwrap();
    assert!(session.active().unwrap().standalone_ineligibility.is_none());
    tokenizer.release(&mut c).unwrap();
    let mut arena = session.freeze(&mut c).unwrap();
    assert_eq!(locals(arena.fragment().unwrap()), ["img"]);
    arena.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn recovered_image_literal_coexistence_cap_preserves_old_backing_before_commit() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<image id=x>");
    // Actual incoming image(5), id(2), x(1), and tokenizer last-start(5)
    // occupy thirteen units; the replacement requires three more live.
    for cap in [15, 16] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let original = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let bytes = c.usage().actual_capacity_bytes;
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let before = c.usage().actual_capacity_bytes;
        let result = session.recover_image_tag(&mut c, &source, &mut input);
        if cap == 16 {
            result.unwrap();
            assert_eq!(
                input.as_ref().unwrap().name.value.text().code_units(),
                [105, 109, 103]
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .high_water,
                16
            );
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::BufferedUtf16Units,
                    ..
                }))
            ));
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .name
                    .value
                    .text()
                    .code_units()
                    .as_ptr(),
                original
            );
            assert_eq!(c.usage().actual_capacity_bytes, before);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                13
            );
        }
        assert!(before >= bytes);
        release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
#[test]
fn image_alias_foreign_and_cancellation_leave_actual_name_and_attributes_in_input() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<image id=x>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(tag);
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let alias = input.as_ref().unwrap().attributes.as_slice()[0]
        .value
        .value
        .text()
        .clone();
    assert_eq!(
        session.recover_image_tag(&mut c, &source, &mut input),
        Err(SessionError::AliasedPayload)
    );
    assert_eq!(
        input.as_ref().unwrap().name.value.text().code_units(),
        [105, 109, 97, 103, 101]
    );
    drop(alias);
    let mut foreign = control();
    foreign.admit_source(&source).unwrap();
    assert!(matches!(
        session.recover_image_tag(&mut foreign, &source, &mut input),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert!(session.active().unwrap().standalone_ineligibility.is_none());
    cancellation.cancel();
    assert_eq!(
        session.recover_image_tag(&mut c, &source, &mut input),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    assert_eq!(input.as_ref().unwrap().attributes.len(), 1);
    release_token_kind(&mut input.take().map(TokenKind::Tag), &mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}
#[test]
fn image_work_exact_and_partial_stops_preserve_or_commit_one_actual_name_owner() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<image id=x>");
    let mut required = None;
    for remaining in [
        None,
        Some(0),
        Some(1),
        Some(100),
        Some(u64::MAX - 1),
        Some(u64::MAX),
    ] {
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        let original = input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .code_units()
            .as_ptr();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let bytes = c.usage().actual_capacity_bytes;
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let remaining = remaining.map(|value| {
            if value >= u64::MAX - 1 {
                required.unwrap() - (u64::MAX - value)
            } else {
                value
            }
        });
        if let Some(remaining) = remaining {
            c.charge(
                HtmlParseWorkCategory::StateStep,
                limits().work_credits() - before - remaining,
            )
            .unwrap();
        }
        let start = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let result = session.recover_image_tag(&mut c, &source, &mut input);
        if remaining.is_none() {
            result.unwrap();
            required = Some(c.usage().resource(HtmlParseResource::WorkCredits).current - start);
        } else if remaining == required {
            result.unwrap();
            assert_eq!(
                input.as_ref().unwrap().name.value.text().code_units(),
                [105, 109, 103]
            );
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            assert_eq!(
                input
                    .as_ref()
                    .unwrap()
                    .name
                    .value
                    .text()
                    .code_units()
                    .as_ptr(),
                original
            );
            assert_eq!(c.usage().actual_capacity_bytes, bytes);
        }
        let stop = c.stop();
        let mut token = input.take().map(TokenKind::Tag);
        if release_token_kind(&mut token, &mut c).is_err() {
            release_token_kind(&mut token, &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if stop.is_some() {
            assert_eq!(c.stop(), stop);
        }
    }
}
#[test]
fn body_runs_keep_exact_utf16_kinds_order_and_newline_reference_origins() {
    use crate::parser::tokenizer::token::HtmlParseOriginKind;
    let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
        "<a v='A \r\n&#x20;"
            .encode_utf16()
            .chain([0xd83d, 0xde00, 0xd800, 0xfeff, 0x0b])
            .chain(" B'>".encode_utf16())
            .collect::<Vec<_>>(),
    ));
    let mut c = control();
    let mut input = Some(body_lexical_value(&source, &mut c));
    let len = input.as_ref().unwrap().value.text().len();
    let reads = c.usage().work(HtmlParseWorkCategory::InspectByte);
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    assert!(input.is_none());
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::InspectByte) - reads,
        2 * len as u64
    );
    let expected = [
        (BodyTextKind::Other, vec![65]),
        (BodyTextKind::Whitespace, vec![32, 10, 32]),
        (
            BodyTextKind::Other,
            vec![0xd83d, 0xde00, 0xd800, 0xfeff, 0x0b],
        ),
        (BodyTextKind::Whitespace, vec![32]),
        (BodyTextKind::Other, vec![66]),
    ];
    for (kind, units) in expected {
        let run = runs.take_next(&mut c).unwrap().unwrap();
        assert_eq!(run.kind, kind);
        assert_eq!(run.text.value.text().code_units(), units);
        for origin in run.text.origins.as_slice() {
            assert_eq!(origin.source.source(), source.identity());
        }
        if kind == BodyTextKind::Whitespace && units.len() == 3 {
            let origins = run.text.origins.as_slice();
            assert_eq!(
                (
                    origins[1].start,
                    origins[1].end,
                    origins[1].source.start(),
                    origins[1].source.end(),
                    origins[1].kind
                ),
                (1, 2, 8, 10, HtmlParseOriginKind::Newline)
            );
            assert_eq!(
                (
                    origins[2].start,
                    origins[2].end,
                    origins[2].source.start(),
                    origins[2].source.end(),
                    origins[2].kind
                ),
                (2, 3, 10, 16, HtmlParseOriginKind::CharacterReference)
            );
        }
        run.text.release(&mut c).unwrap();
    }
    assert!(runs.take_next(&mut c).unwrap().is_none());
    runs.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn body_null_run_retains_the_actual_data_null_source() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("\0");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Text(text) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("text")
    };
    let mut input = Some(text);
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    let run = runs.take_next(&mut c).unwrap().unwrap();
    assert_eq!(run.kind, BodyTextKind::Null);
    assert_eq!(run.text.value.text().code_units(), [0]);
    assert_eq!(
        (
            run.text.origins.as_slice()[0].source.start(),
            run.text.origins.as_slice()[0].source.end()
        ),
        (0, 1)
    );
    run.text.release(&mut c).unwrap();
    runs.release(&mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn body_partition_full_coexisting_units_exact_and_one_short_preserve_input_on_failure() {
    let source = crate::HtmlSource::utf8("<a v='A B '>");
    for cap in [7, 8] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .buffered_utf16_units(cap)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let mut input = Some(body_lexical_value(&source, &mut c));
        let bytes = c.usage().actual_capacity_bytes;
        let result = partition_body_text(&mut c, &mut input);
        if cap == 8 {
            let mut runs = result.unwrap();
            assert!(input.is_none());
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .high_water,
                8
            );
            runs.release(&mut c).unwrap();
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::BufferedUtf16Units,
                    ..
                }))
            ));
            assert_eq!(
                input.as_ref().unwrap().value.text().code_units(),
                [65, 32, 66, 32]
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current,
                4
            );
            assert_eq!(c.usage().actual_capacity_bytes, bytes);
            release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
        }
        storage_zero(&c);
    }
}
#[test]
fn body_partition_alias_foreign_and_cancel_preflight_preserve_original_owner() {
    let source = crate::HtmlSource::utf8("<a v='A B'>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let mut input = Some(body_lexical_value(&source, &mut c));
    let alias = input.as_ref().unwrap().value.text().clone();
    assert!(matches!(
        partition_body_text(&mut c, &mut input),
        Err(SessionError::AliasedPayload)
    ));
    assert!(input.is_some());
    drop(alias);
    let mut foreign = control();
    foreign.admit_source(&source).unwrap();
    assert!(matches!(
        partition_body_text(&mut foreign, &mut input),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignControl
        )))
    ));
    assert!(input.is_some());
    cancellation.cancel();
    assert!(matches!(
        partition_body_text(&mut c, &mut input),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    assert_eq!(
        input.as_ref().unwrap().value.text().code_units(),
        [65, 32, 66]
    );
    release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}
#[test]
fn body_partition_work_boundaries_cleanup_all_staged_real_owners() {
    let source = crate::HtmlSource::utf8("<a v='A B C'>");
    let mut probe = control();
    let mut input = Some(body_lexical_value(&source, &mut probe));
    let before = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let mut runs = partition_body_text(&mut probe, &mut input).unwrap();
    let required = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    runs.release(&mut probe).unwrap();
    storage_zero(&probe);
    for remaining in [0, 1, required / 3, required * 2 / 3, required - 1, required] {
        let mut c = control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let bytes = c.usage().actual_capacity_bytes;
        let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - used - remaining,
        )
        .unwrap();
        let result = partition_body_text(&mut c, &mut input);
        assert_eq!(result.is_ok(), remaining == required);
        if let Ok(mut runs) = result {
            assert!(input.is_none());
            discard_runs(&mut runs, &mut c).unwrap();
        } else {
            assert!(input.is_some());
            assert_eq!(c.usage().actual_capacity_bytes, bytes);
            let stop = c.stop().unwrap();
            assert_eq!(session.freeze(&mut c).err(), Some(SessionError::Stop(stop)));
            release_token_kind(&mut input.take().map(TokenKind::Text), &mut c).unwrap();
            assert_eq!(c.stop(), Some(stop));
        }
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
    }
}
#[test]
fn body_runs_partial_take_cancel_and_alias_disposal_retain_actual_slots() {
    let source = crate::HtmlSource::utf8("<a v='A B'>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    let mut input = Some(body_lexical_value(&source, &mut c));
    let mut runs = partition_body_text(&mut c, &mut input).unwrap();
    let first = runs.take_next(&mut c).unwrap().unwrap();
    let alias = runs.entries.as_ref().unwrap().as_slice()[1]
        .as_ref()
        .unwrap()
        .text
        .value
        .text()
        .clone();
    assert_eq!(runs.release(&mut c), Err(SessionError::AliasedPayload));
    assert!(runs.entries.is_some());
    drop(alias);
    cancellation.cancel();
    assert!(matches!(
        runs.take_next(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    first.text.release(&mut c).unwrap();
    runs.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
}
#[test]
fn first_lf_retains_crlf_and_character_reference_sources_and_exact_suffix() {
    use crate::parser::tokenizer::token::HtmlParseOriginKind;
    for (literal, end, kind) in [
        ("\r\nA B", 8, HtmlParseOriginKind::Newline),
        ("&#10;A B", 11, HtmlParseOriginKind::CharacterReference),
    ] {
        let source = crate::HtmlSource::utf8(format!("<a v='{literal}'>"));
        let mut c = control();
        let mut input = Some(body_lexical_value(&source, &mut c));
        let prefix = split_first_lf(&mut c, &mut input).unwrap().unwrap();
        assert_eq!(prefix.value.text().code_units(), [10]);
        let origin = &prefix.origins.as_slice()[0];
        assert_eq!(
            (
                origin.start,
                origin.end,
                origin.source.start(),
                origin.source.end(),
                origin.kind
            ),
            (0, 1, 6, end, kind)
        );
        assert_eq!(
            input.as_ref().unwrap().value.text().code_units(),
            [65, 32, 66]
        );
        assert_eq!(
            input.as_ref().unwrap().origins.as_slice()[0].source.start(),
            end
        );
        prefix.release(&mut c).unwrap();
        input.take().unwrap().release(&mut c).unwrap();
        storage_zero(&c);
    }
    let source = crate::HtmlSource::utf8("<a v='\n'>");
    let mut c = control();
    let mut input = Some(body_lexical_value(&source, &mut c));
    let original = input.as_ref().unwrap().value.text().code_units().as_ptr();
    let prefix = split_first_lf(&mut c, &mut input).unwrap().unwrap();
    assert!(input.is_none());
    assert_eq!(prefix.value.text().code_units().as_ptr(), original);
    prefix.release(&mut c).unwrap();
    storage_zero(&c);
}
#[test]
fn body_and_p_literals_and_both_body_merge_paths_freeze_same_native_arena() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for implied in [false, true] {
        let source =
            crate::HtmlSource::utf8("<body id=old><body id=ignored x:id=p a=one a=last z=end>");
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let body = if implied {
            let body = session.allocate_implicit_body(&mut c, None).unwrap();
            session
                .add_body_attributes_if_missing(&mut c, &body, &mut Some(first))
                .unwrap();
            body
        } else {
            session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
        };
        let root = session.root(&mut c).unwrap();
        session.insert(&mut c, &root, &body, None, None).unwrap();
        let p = session.allocate_implicit_p(&mut c, None).unwrap();
        session.insert(&mut c, &body, &p, None, None).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        session
            .add_body_attributes_if_missing(&mut c, &body, &mut Some(second))
            .unwrap();
        let storage = session.active().unwrap();
        assert!(storage.standalone_ineligibility.is_none());
        let HtmlNodeKind::Element(element) = &storage
            .construction
            .node_at(body.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("body")
        };
        let attributes = element.attributes().as_slice();
        assert_eq!(
            attributes
                .iter()
                .map(|a| (
                    a.name().local_name().to_utf8().unwrap(),
                    a.value().to_utf8().unwrap()
                ))
                .collect::<Vec<_>>(),
            [("id", "old"), ("x:id", "p"), ("a", "one"), ("z", "end")]
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
        );
        assert!(
            attributes
                .iter()
                .all(|a| a.name().namespace().uri().is_none())
        );
        t.release(&mut c).unwrap();
        let mut arena = session.freeze(&mut c).unwrap();
        let fragment = arena.fragment().unwrap();
        assert_eq!(locals(fragment), ["body"]);
        let body_id = fragment.root().children().next().unwrap();
        let p_id = fragment.node(&body_id).unwrap().children().next().unwrap();
        let HtmlNodeKind::Element(p) = fragment.node(&p_id).unwrap().kind() else {
            panic!("p")
        };
        assert_eq!(p.name().local_name().code_units(), [112]);
        arena.release(&mut c).unwrap();
        storage_zero(&c);
    }
}
