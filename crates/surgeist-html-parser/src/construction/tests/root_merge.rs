use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    values::DocumentString,
};

#[test]
fn implied_head_and_both_root_attribute_paths_keep_order_exact_backing_and_origins() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for implied in [false, true] {
        let source = crate::HtmlSource::utf16(DocumentString::from_utf16(
            "<html id=old><html id=ignored x:id=p a='"
                .encode_utf16()
                .chain([0xd800, 0xd83d, 0xde00])
                .chain("' a=last z=end>".encode_utf16())
                .collect::<Vec<_>>(),
        ));
        let mut c = control();
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("first tag")
        };
        let root = if implied {
            let root = session.allocate_implicit_html(&mut c).unwrap();
            session
                .add_attributes_if_missing(&mut c, &root, &mut Some(first))
                .unwrap();
            root
        } else {
            session.allocate_html_tag(&mut c, &mut Some(first)).unwrap()
        };
        let doc = session.root(&mut c).unwrap();
        session.insert(&mut c, &doc, &root, None, None).unwrap();
        let head = session.allocate_implicit_head(&mut c, None).unwrap();
        session.insert(&mut c, &root, &head, None, None).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("second tag")
        };
        assert!(second.had_duplicate_attributes);
        let expected_origin = second.attributes.as_slice()[2].value.origins.as_slice()[0]
            .source
            .clone();
        let mut second = Some(second);
        session
            .add_attributes_if_missing(&mut c, &root, &mut second)
            .unwrap();
        assert!(second.is_none());
        let storage = session.active().unwrap();
        let ConstructionNodeKind::Element(element) = &storage
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        assert_eq!(
            element
                .attributes()
                .iter()
                .map(|a| a.name().local_name().to_utf8().unwrap())
                .collect::<Vec<_>>(),
            ["id", "x:id", "a", "z"]
        );
        assert_eq!(
            element
                .attributes()
                .get(0)
                .unwrap()
                .value()
                .to_utf8()
                .unwrap(),
            "old"
        );
        assert_eq!(
            element.attributes().get(2).unwrap().value().code_units(),
            [0xd800, 0xd83d, 0xde00]
        );
        assert!(
            element
                .attributes()
                .iter()
                .all(|a| a.name().namespace().uri().is_none())
        );
        assert!(storage.payloads.values.iter().any(|v| matches!(v, RetainedPayload::String { origins: Some(origins), location: StringLocation::AttributeValue(_, 2), .. } if storage.observed_origins(origins, &mut c).unwrap()[0].source == expected_origin)));
        assert_eq!(storage.construction.test_storage().usage().attributes(), 4);
        t.release(&mut c).unwrap();
        let mut frozen = session.freeze(&mut c).unwrap();
        let document = frozen.document().unwrap();
        let root_id = document.root().children().next().unwrap();
        let surgeist_html::construction::HtmlNodeKind::Element(element) =
            document.node(&root_id).unwrap().kind()
        else {
            panic!("root")
        };
        assert_eq!(element.attributes().len(), 4);
        let head_id = document.node(&root_id).unwrap().children().next().unwrap();
        let surgeist_html::construction::HtmlNodeKind::Element(head) =
            document.node(&head_id).unwrap().kind()
        else {
            panic!("head")
        };
        assert_eq!(head.name().local_name().to_utf8().unwrap(), "head");
        assert!(head.attributes().is_empty());
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

fn observed_root_merge(new_attributes: bool) -> crate::parser::control::OwnedSpanObservation {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // A real lexer produces every field and original source association.
    // Sixteen short attributes fit the existing finite fixture capacities.
    let attributes = (0..16).map(|i| format!(" a{i:02}=v")).collect::<String>();
    let input = if new_attributes {
        format!("<html><html{attributes}>")
    } else {
        format!("<html{attributes}><html{attributes}>")
    };
    let source = crate::HtmlSource::utf8(input);
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("actual first tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("actual second tag")
    };
    let mut second = Some(second);
    c.clear_owned_span_observation();
    session
        .add_attributes_if_missing(&mut c, &root, &mut second)
        .unwrap();
    let observed = c.owned_span_observation();
    assert!(second.is_none());
    assert_eq!(c.stop(), None);
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("actual root")
    };
    assert_eq!(element.attributes().len(), 16);
    for (i, attribute) in element.attributes().iter().enumerate() {
        assert_eq!(
            attribute.name().local_name().to_utf8().unwrap(),
            format!("a{i:02}")
        );
        assert_eq!(attribute.value().to_utf8().unwrap(), "v");
    }
    assert_eq!(attribute_holders(element.attributes()), 2);
    assert_eq!(
        Arc::weak_count(element.attributes().to_ordered().shared_attributes()),
        1
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    observed
}

#[test]
fn wide_real_attribute_merge_respects_declared_poll_span() {
    let observed = observed_root_merge(true);
    assert!(observed.polls > 0);
    assert_eq!(observed.append_slots, 16);
    assert!(
        observed.largest_append_copy_bytes <= 256,
        "actual owned attribute copy exceeds the selected 256-credit poll span: {observed:?}"
    );
    assert!(
        observed.largest_registry_credits_between_polls <= 256,
        "actual owned registry iterations exceed the selected 256-credit poll span: {observed:?}"
    );
}

#[test]
fn duplicate_only_root_merge_respects_declared_poll_span() {
    let observed = observed_root_merge(false);
    assert!(observed.polls > 0);
    assert_eq!(observed.append_slots, 0);
    assert!(
        observed.largest_registry_credits_between_polls <= 256,
        "actual owned registry iterations exceed the selected 256-credit poll span: {observed:?}"
    );
}

fn prepare_real_merge<'source>(
    source: &'source crate::HtmlSource,
    c: &mut ParseControl,
) -> (
    ParserArenaSession,
    ParserNode,
    PreparedAttributeMerge,
    crate::parser::tokenizer::tests::LexerFixture<'source>,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    c.admit_source(source).unwrap();
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, c).unwrap();
    let mut session = ParserArenaSession::new(c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut tokenizer, c) else {
        panic!("actual first tag")
    };
    let root = session.allocate_html_tag(c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut tokenizer, c) else {
        panic!("actual second tag")
    };
    let mut input = Some(second);
    let prepared = session
        .active_mut()
        .unwrap()
        .prepare_attribute_merge(c, &root, &mut input)
        .unwrap()
        .unwrap();
    assert!(input.is_none());
    (session, root, prepared, tokenizer)
}

fn assert_partial_merge_storage(
    session: &ParserArenaSession,
    c: &mut ParseControl,
    root: &ParserNode,
    prepared: &PreparedAttributeMerge,
    moved: usize,
) {
    let storage = session.active().unwrap();
    let RetainedPayload::Attributes(Some(AttributeStorage::Transferring {
        values,
        moved: actual,
        ..
    })) = &storage.payloads.values[prepared.stage]
    else {
        panic!("actual retained transfer phase")
    };
    assert_eq!(*actual, moved);
    assert_eq!(values.len(), 2 - moved);
    let ConstructionNodeKind::Element(element) = &storage
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("actual root")
    };
    let expected = [("id", "old"), ("a", "one"), ("b", "two")];
    assert_eq!(element.attributes().len(), 1 + moved);
    for (attribute, (name, value)) in element.attributes().iter().zip(expected) {
        assert_eq!(attribute.name().local_name().to_utf8().unwrap(), name);
        assert_eq!(attribute.value().to_utf8().unwrap(), value);
    }
    assert_eq!(attribute_holders(element.attributes()), 2);
    assert_eq!(
        Arc::weak_count(element.attributes().to_ordered().shared_attributes()),
        1
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        3
    );
}

fn dispose_stopped_merge(
    mut session: ParserArenaSession,
    tokenizer: crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) {
    let stop = c.stop().expect("real terminal boundary");
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    let capacity_peak = c.usage().actual_capacity_high_water;
    let nodes = c.usage().resource(HtmlParseResource::ManagedNodes).current;
    let attribute_peak = c
        .usage()
        .resource(HtmlParseResource::ManagedAttributes)
        .high_water;
    assert!(matches!(session.freeze(c), Err(SessionError::Stop(actual)) if actual == stop));
    tokenizer.release(c).unwrap();
    session.release(c).unwrap();
    storage_zero(c);
    assert_eq!(c.stop(), Some(stop));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(c.usage().actual_capacity_high_water, capacity_peak);
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        nodes
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .high_water,
        attribute_peak
    );
}

#[test]
fn every_bounded_slot_work_cut_preserves_literal_prefix_backings_and_cleanup() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let mut c = control();
    let (mut session, _, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
    let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session
        .active_mut()
        .unwrap()
        .edit_attributes(
            &mut c,
            prepared.target,
            prepared.container,
            prepared.stage,
            1,
            true,
        )
        .unwrap();
    let envelope = c.usage().resource(HtmlParseResource::WorkCredits).current - setup;
    eprintln!("controlled named first-slot coverage envelope: {envelope}");
    assert!(
        envelope <= 4096,
        "finite test campaign bound, not a correctness oracle"
    );
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    let mut prefixes = [false; 2];
    // Include the next slot's first refused action after the first commit.
    for cut in 0..=envelope + 1 {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .work_credits(setup + cut)
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
        let first = session.active_mut().unwrap().edit_attributes(
            &mut c,
            prepared.target,
            prepared.container,
            prepared.stage,
            1,
            true,
        );
        let refused = if first.is_ok() {
            session.active_mut().unwrap().edit_attributes(
                &mut c,
                prepared.target,
                prepared.container,
                prepared.stage,
                1,
                true,
            )
        } else {
            first
        };
        assert!(matches!(
            refused,
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }))
        ));
        let RetainedPayload::Attributes(Some(AttributeStorage::Transferring { moved, .. })) =
            &session.active().unwrap().payloads.values[prepared.stage]
        else {
            panic!("real transfer progress")
        };
        let moved = *moved;
        assert!(moved <= 1);
        prefixes[moved] = true;
        // Expected source values are literal, independently paired with all
        // actual original registry backings and the retained redirect.
        assert_partial_merge_storage(&session, &mut c, &root, &prepared, moved);
        dispose_stopped_merge(session, tokenizer, &mut c);
    }
    assert_eq!(prefixes, [true, true]);
}

#[test]
fn exact_and_one_short_rebind_work_preserve_redirect_and_terminal_cleanup() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let mut c = control();
    let (mut session, _, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
    let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
    for _ in 0..2 {
        session
            .active_mut()
            .unwrap()
            .edit_attributes(
                &mut c,
                prepared.target,
                prepared.container,
                prepared.stage,
                1,
                true,
            )
            .unwrap();
    }
    // A measured completed prefix only reaches the unchanged rebind action;
    // the rebind's expected byte/write cost below remains literal.
    let prefix = c.usage().resource(HtmlParseResource::WorkCredits).current - setup;
    assert!(prefix <= 8192);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    let rebind_credits = (1 + 2 * mem::size_of::<StringLocation>() + 1) as u64;
    for short in [false, true] {
        let mut c = ParseControl::new(
            limits()
                .into_builder()
                .work_credits(setup + prefix + rebind_credits - u64::from(short))
                .finish()
                .unwrap(),
            HtmlParseCancellation::new(),
        );
        let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
        for _ in 0..2 {
            session
                .active_mut()
                .unwrap()
                .edit_attributes(
                    &mut c,
                    prepared.target,
                    prepared.container,
                    prepared.stage,
                    1,
                    true,
                )
                .unwrap();
        }
        let index = prepared.stage + 1; // real first registered incoming name
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let result = session.active_mut().unwrap().rebind_attribute_location(
            &mut c,
            index,
            prepared.stage,
            prepared.container,
            prepared.start,
        );
        if short {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
        } else {
            result.unwrap();
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current - before,
                rebind_credits
            );
            assert!(
                session
                    .active_mut()
                    .unwrap()
                    .rebind_attribute_location(
                        &mut c,
                        index + 1,
                        prepared.stage,
                        prepared.container,
                        prepared.start
                    )
                    .is_err()
            );
        }
        let RetainedPayload::String {
            location,
            origins: Some(origins),
            ..
        } = &session.active().unwrap().payloads.values[index]
        else {
            panic!("actual original name")
        };
        assert!(
            matches!(location, StringLocation::AttributeName(owner, offset)
            if *owner == if short { prepared.stage } else { prepared.container }
                && *offset == if short { 1 } else { 2 })
        );
        let evidence = session.active().unwrap().evidence.as_ref().unwrap();
        let provenance::GroupBody::Leaf { start, length, .. } =
            evidence.require_group(origins).unwrap().body
        else {
            panic!("original retained leaf");
        };
        assert_eq!(length, 1);
        // Fixed admitted evidence is inspected by the test harness, without
        // attempting a new successful read through the stopped controller.
        assert_eq!(
            evidence
                .fixed_original_origin(start)
                .unwrap()
                .source
                .start(),
            25
        );
        assert_eq!(
            evidence.fixed_original_origin(start).unwrap().source.end(),
            26
        );
        assert_partial_merge_storage(&session, &mut c, &root, &prepared, 2);
        dispose_stopped_merge(session, tokenizer, &mut c);
    }
}

#[test]
fn cancellation_between_real_slot_and_rebind_operations_retains_exact_owner_state() {
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    for during_rebind in [false, true] {
        let cancellation = HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        let (mut session, root, prepared, tokenizer) = prepare_real_merge(&source, &mut c);
        let moved = if during_rebind { 2 } else { 1 };
        for _ in 0..moved {
            session
                .active_mut()
                .unwrap()
                .edit_attributes(
                    &mut c,
                    prepared.target,
                    prepared.container,
                    prepared.stage,
                    1,
                    true,
                )
                .unwrap();
        }
        if during_rebind {
            session
                .active_mut()
                .unwrap()
                .rebind_attribute_location(
                    &mut c,
                    prepared.stage + 1,
                    prepared.stage,
                    prepared.container,
                    prepared.start,
                )
                .unwrap();
        }
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        cancellation.cancel();
        let result = if during_rebind {
            session.active_mut().unwrap().rebind_attribute_location(
                &mut c,
                prepared.stage + 2,
                prepared.stage,
                prepared.container,
                prepared.start,
            )
        } else {
            session.active_mut().unwrap().edit_attributes(
                &mut c,
                prepared.target,
                prepared.container,
                prepared.stage,
                1,
                true,
            )
        };
        assert_eq!(result, Err(SessionError::Stop(HtmlParseStop::Cancelled)));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        assert_partial_merge_storage(&session, &mut c, &root, &prepared, moved);
        dispose_stopped_merge(session, tokenizer, &mut c);
    }
}

#[test]
fn merge_rejects_weak_holder_and_foreign_origin_before_consuming_input() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=x><html a=y>");
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let mut incoming = Some(second);
    let weak = match &session
        .active()
        .unwrap()
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    {
        ConstructionNodeKind::Element(e) => {
            Arc::downgrade(e.attributes().to_ordered().shared_attributes())
        }
        _ => panic!("root"),
    };
    assert_eq!(
        session.add_attributes_if_missing(&mut c, &root, &mut incoming),
        Err(SessionError::AliasedPayload)
    );
    assert!(incoming.is_some());
    drop(weak);
    // Correct execution tickets alone cannot qualify an origin from another
    // immutable source. The actual original slot remains owned by caller.
    let foreign_source = crate::HtmlSource::utf8("other");
    let original = mem::replace(
        &mut incoming
            .as_mut()
            .unwrap()
            .attributes
            .get_mut(0, &mut c)
            .unwrap()
            .unwrap()
            .source,
        foreign_source.range(0, 5).unwrap(),
    );
    assert!(matches!(
        session.add_attributes_if_missing(&mut c, &root, &mut incoming),
        Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::ForeignSource
        )))
    ));
    assert!(incoming.is_some());
    // Terminal disposal validates real ownership rather than source content.
    let _retained_original = original;
    let mut kind = incoming.take().map(TokenKind::Tag);
    release_token_kind(&mut kind, &mut c).unwrap();
    t.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn imported_attribute_backing_is_explicitly_unavailable_and_retained() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.allocate_element(&mut c, html("html")).unwrap();
    let mut incoming = Some(owned_tag(&mut c, "html", &[("x", &[121])]));
    assert_eq!(
        session.add_attributes_if_missing(&mut c, &root, &mut incoming),
        Err(SessionError::AttributeStorageUnavailable)
    );
    assert!(incoming.is_some());
    let mut kind = incoming.map(TokenKind::Tag);
    release_token_kind(&mut kind, &mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn staged_attribute_cap_failure_preserves_root_and_reclaims_exact_real_slots() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=old><html a=one z=two>");
    let mut c = ParseControl::new(
        limits()
            .into_builder()
            .managed_attributes(2)
            .finish()
            .unwrap(),
        crate::HtmlParseCancellation::new(),
    );
    c.admit_source(&source).unwrap();
    let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
    let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
        panic!("tag")
    };
    let mut input = Some(second);
    assert!(matches!(
        session.add_attributes_if_missing(&mut c, &root, &mut input),
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedAttributes,
            ..
        }))
    ));
    assert!(input.is_none());
    let ConstructionNodeKind::Element(element) = &session
        .active()
        .unwrap()
        .construction
        .node_at(root.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("root")
    };
    assert_eq!(element.attributes().len(), 1);
    assert_eq!(
        element
            .attributes()
            .get(0)
            .unwrap()
            .value()
            .to_utf8()
            .unwrap(),
        "old"
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .current,
        2
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::TokenAttributes)
            .current,
        0
    );
    let stop = c.stop();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    t.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), stop);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedAttributes)
            .high_water,
        2
    );
}

#[test]
fn merge_foreign_control_and_cancel_leave_the_real_lexical_owner_untaken() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    for cancel in [false, true] {
        let source = crate::HtmlSource::utf8("<html a=x>");
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(limits(), cancellation.clone());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.allocate_implicit_html(&mut c).unwrap();
        let TokenKind::Tag(tag) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let mut input = Some(tag);
        if cancel {
            cancellation.cancel();
            assert_eq!(
                session.add_attributes_if_missing(&mut c, &root, &mut input),
                Err(SessionError::Stop(HtmlParseStop::Cancelled))
            );
        } else {
            let mut foreign = control();
            foreign.admit_source(&source).unwrap();
            assert!(matches!(
                session.add_attributes_if_missing(&mut foreign, &root, &mut input),
                Err(SessionError::Stop(HtmlParseStop::InternalInvariant(
                    crate::parser::control::HtmlParseInvariant::ForeignControl
                )))
            ));
        }
        assert!(input.is_some());
        let ConstructionNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        assert!(element.attributes().is_empty());
        let mut kind = input.map(TokenKind::Tag);
        release_token_kind(&mut kind, &mut c).unwrap();
        t.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn merge_work_boundaries_retain_record_registry_owners_and_cleanup_all_staged_storage() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<html id=old><html a=one b=two>");
    let run = |cap: Option<u64>| {
        let selected = match cap {
            Some(cap) => limits().into_builder().work_credits(cap).finish().unwrap(),
            None => limits(),
        };
        let mut c = ParseControl::new(selected, crate::HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut t = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let TokenKind::Tag(first) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let root = session.allocate_html_tag(&mut c, &mut Some(first)).unwrap();
        let TokenKind::Tag(second) = lexical_kind(&mut t, &mut c) else {
            panic!("tag")
        };
        let setup = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let lexical_name = second.name.value.text().code_units().as_ptr();
        let mut input = Some(second);
        let result = session.add_attributes_if_missing(&mut c, &root, &mut input);
        let completed = c.usage().resource(HtmlParseResource::WorkCredits).current;
        let ConstructionNodeKind::Element(element) = &session
            .active()
            .unwrap()
            .construction
            .node_at(root.index)
            .expect("retained construction position")
            .kind()
        else {
            panic!("root")
        };
        assert_eq!(
            element
                .attributes()
                .get(0)
                .unwrap()
                .value()
                .to_utf8()
                .unwrap(),
            "old"
        );
        assert!((1..=3).contains(&element.attributes().len()));
        for (actual, (name, value)) in
            element
                .attributes()
                .iter()
                .zip([("id", "old"), ("a", "one"), ("b", "two")])
        {
            assert_eq!(actual.name().local_name().to_utf8().unwrap(), name);
            assert_eq!(actual.value().to_utf8().unwrap(), value);
        }
        if let Some(tag) = &input {
            assert_eq!(tag.name.value.text().code_units().as_ptr(), lexical_name);
            assert_eq!(tag.attributes.len(), 2);
            for (attribute, (name, value)) in tag
                .attributes
                .as_slice()
                .iter()
                .zip([("a", "one"), ("b", "two")])
            {
                assert_eq!(attribute.name.value.text().to_utf8().unwrap(), name);
                assert_eq!(attribute.value.value.text().to_utf8().unwrap(), value);
            }
        }
        assert_eq!(attribute_holders(element.attributes()), 2);
        assert_eq!(
            Arc::weak_count(element.attributes().to_ordered().shared_attributes()),
            1
        );
        match result {
            Err(error) => assert!(matches!(
                error,
                SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                })
            )),
            Ok(()) => {
                assert_eq!(element.attributes().len(), 3);
                assert!(input.is_none());
            }
        }
        let stop = c.stop();
        if let Some(tag) = input {
            let mut kind = Some(TokenKind::Tag(tag));
            if release_token_kind(&mut kind, &mut c).is_err() {
                release_token_kind(&mut kind, &mut c).unwrap();
            }
        }
        t.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if stop.is_some() {
            assert_eq!(c.stop(), stop);
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                completed
            );
        }
        (setup, completed)
    };
    let (setup, complete) = run(None);
    let envelope = complete - setup;
    eprintln!("controlled complete named merge coverage envelope: {envelope}");
    // This bounds harness runtime, not the semantic tariff. Retained
    // representation growth and newly admitted hooks can increase the real
    // complete envelope; every actual setup..complete cut is still tested.
    assert!(
        envelope <= 65536,
        "finite coverage bound; literal fields/backings are the oracle"
    );
    // Every actual work cut spans borrowed planning (Tag still occupied),
    // witnessed decisions, registration, reserve, appends and late rebind.
    for cap in setup..complete {
        run(Some(cap));
    }
}
