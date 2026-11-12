use super::*;
use surgeist_html::{
    construction::HtmlDocumentMode, elements::form::HtmlNativeFormAssociationKind,
};

fn textarea_owner<'a>(
    source: &'a crate::HtmlSource,
    caps: HtmlParseLimits,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
) {
    textarea_owner_cancellable(source, caps, HtmlParseCancellation::new())
}
fn textarea_owner_cancellable<'a>(
    source: &'a crate::HtmlSource,
    caps: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
) {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut c = ParseControl::new(caps, cancellation);
    c.admit_source(source).unwrap();
    let mut tokenizer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let tag = next_fixture_tag(
        &mut tokenizer,
        &mut c,
        &[116, 101, 120, 116, 97, 114, 101, 97],
    );
    let mut input = Some(tag);
    let textarea = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &root,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    assert!(input.is_none());
    session
        .insert(&mut c, &root, &textarea, None, None)
        .unwrap();
    (c, session, textarea, tokenizer)
}

#[test]
fn textarea_raw_is_independent_exact_child_text_and_retired_backing_is_disposed() {
    let source = crate::HtmlSource::utf8("<textarea>AB<span>C");
    let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &node)
            .unwrap()
            .code_units(),
        []
    );
    let state = session.textarea_state(&mut c, &node).unwrap();
    assert!(!state.flags.dirty_value() && !state.flags.user_validity());
    assert_eq!(
        session.form_association_state(&mut c, &node).unwrap().kind,
        HtmlNativeFormAssociationKind::Textarea
    );
    let mut a = next_text(&mut tokenizer, &mut c);
    let text = session
        .insert_text(&mut c, &node, None, &mut a, None)
        .unwrap()
        .unwrap()
        ._node;
    let first_raw = session.textarea_state(&mut c, &node).unwrap().raw;
    let native = native_text(&session, &text);
    let raw = session
        .textarea_raw_value(&mut c, &node)
        .unwrap()
        .code_units()
        .to_vec();
    assert_eq!(native, [65]);
    assert_eq!(raw, [65]);
    let raw = session.textarea_raw_value(&mut c, &node).unwrap();
    let ConstructionNodeKind::Text(native) = session
        .active()
        .unwrap()
        .construction
        .node_at(text.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("text node")
    };
    assert!(!std::ptr::eq(
        native.code_units().unwrap().as_ptr(),
        raw.code_units().as_ptr()
    ));
    let mut b = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut b, None)
        .unwrap();
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &node)
            .unwrap()
            .code_units(),
        [65, 66]
    );
    assert!(matches!(
        session.active().unwrap().payloads.values[first_raw],
        RetainedPayload::HistoricalString { .. }
    ));
    let before_reset = c
        .usage()
        .resource(HtmlParseResource::ManagedUtf16Units)
        .current;
    for _ in 0..4 {
        session.parser_stack_removed(&mut c, &node, None).unwrap();
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            before_reset
        );
    }
    // A descendant Text is excluded: only immediate Text children contribute.
    let span = next_fixture_tag(&mut tokenizer, &mut c, &[115, 112, 97, 110]);
    let child = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut Some(span),
            &node,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &node, &child, None, None).unwrap();
    let mut text_c = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &child, None, &mut text_c, None)
        .unwrap();
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &node)
            .unwrap()
            .code_units(),
        [65, 66]
    );
    tokenizer.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let id = document.root().children().next().unwrap();
    assert_eq!(
        frozen.textarea_raw_value(&mut c, &id).unwrap().code_units(),
        [65, 66]
    );
    let state = frozen.textarea_state(&mut c, &id).unwrap();
    assert!(!state.flags.dirty_value() && !state.flags.user_validity());
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn textarea_same_parent_reference_removal_runs_old_view_before_insertion() {
    let source = crate::HtmlSource::utf8("<textarea>AB");
    let cancel = HtmlParseCancellation::new();
    let (mut c, mut session, node, mut tokenizer) =
        textarea_owner_cancellable(&source, limits(), cancel.clone());
    let mut a = next_text(&mut tokenizer, &mut c);
    let text = session
        .insert_text(&mut c, &node, None, &mut a, None)
        .unwrap()
        .unwrap()
        ._node;
    let mut b = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut b, None)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    // Cancellation between hook phases proves the removal raw prefix was
    // applied, although final before==node topology is deliberately equal.
    session.active_mut().unwrap().cancel_after_removal = Some(cancel.clone());
    let Err(SessionError::Stop(first)) = session.insert(&mut c, &node, &text, Some(&text), None)
    else {
        panic!("actual phase cancellation")
    };
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(text.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(node.index)
    );
    assert_eq!(native_text(&session, &text), [65, 66]);
    let raw = storage
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::Textarea {
                node: Some(owner),
                state,
            } if *owner == node.index => Some(state.raw),
            _ => None,
        })
        .unwrap();
    let RetainedPayload::String { value, .. } = &storage.payloads.values[raw] else {
        panic!("raw owner")
    };
    assert_eq!(value.code_units(), []);
    assert!(storage.effects.form.is_some());
    assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(stop)) if stop==first));
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(c.stop(), Some(first));
    storage_zero(&c);
}

#[test]
fn textarea_raw_extension_has_exact_40_unit_admission_and_retained_terminal_prefix() {
    // textarea name8 + HTML URI28; A(native1,raw1)=38. B appends one unit to the native text
    // and one unit in place to the raw backing, which has no second copy to coexist with:
    // exact peak40, then current40. A cap of 39 admits the text and refuses the raw value.
    for cap in [39, 40] {
        let source = crate::HtmlSource::utf8("<textarea>AB");
        let caps = limits()
            .into_builder()
            .managed_utf16_units(cap)
            .finish()
            .unwrap();
        let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, caps);
        let mut a = next_text(&mut tokenizer, &mut c);
        let text = session
            .insert_text(&mut c, &node, None, &mut a, None)
            .unwrap()
            .unwrap()
            ._node;
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current,
            38
        );
        let old_raw = session.textarea_state(&mut c, &node).unwrap().raw;
        let mut b = next_text(&mut tokenizer, &mut c);
        let result = session.insert_text(&mut c, &node, None, &mut b, None);
        assert!(b.is_none());
        assert_eq!(native_text(&session, &text), [65, 66]);
        if cap == 40 {
            assert!(result.is_ok());
            assert_eq!(
                session
                    .textarea_raw_value(&mut c, &node)
                    .unwrap()
                    .code_units(),
                [65, 66]
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .current,
                40
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedUtf16Units)
                    .high_water,
                40
            );
            assert!(matches!(
                session.active().unwrap().payloads.values[old_raw],
                RetainedPayload::HistoricalString { .. }
            ));
        } else {
            let Err(SessionError::Stop(first)) = result else {
                panic!("actual coexistence stop")
            };
            assert!(session.active().unwrap().effects.form.is_some());
            let RetainedPayload::String { value, .. } =
                &session.active().unwrap().payloads.values[old_raw]
            else {
                panic!("old owner retained")
            };
            assert_eq!(value.code_units(), [65]);
            assert!(
                matches!(session.textarea_state(&mut c,&node),Err(SessionError::Stop(s)) if s==first)
            );
            assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(s)) if s==first));
        }
        tokenizer.release(&mut c).unwrap();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
    }
}

/// The address of the raw value's shared backing, which an in-place extension keeps.
fn raw_backing(session: &mut ParserArenaSession, c: &mut ParseControl, node: &ParserNode) -> usize {
    session
        .textarea_raw_value(c, node)
        .unwrap()
        .block()
        .address()
}

#[test]
fn textarea_text_appended_at_the_end_extends_the_raw_backing_in_place() {
    let source = crate::HtmlSource::utf8("<textarea>ABC");
    let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
    let mut a = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut a, None)
        .unwrap();
    let backing = raw_backing(&mut session, &mut c, &node);
    let mut rows = session.active().unwrap().payloads.values.len();
    for (appended, expected) in [(66, vec![65, 66]), (67, vec![65, 66, 67])] {
        let mut text = next_text(&mut tokenizer, &mut c);
        let before = session.textarea_state(&mut c, &node).unwrap().raw;
        session
            .insert_text(&mut c, &node, None, &mut text, None)
            .unwrap();
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            expected,
            "after {appended}"
        );
        assert_eq!(raw_backing(&mut session, &mut c, &node), backing);
        // The value moved to a new row, and the row it left keeps only its origins.
        let storage = session.active().unwrap();
        assert!(matches!(
            storage.payloads.values[before],
            RetainedPayload::HistoricalString { .. }
        ));
        assert_ne!(session.textarea_state(&mut c, &node).unwrap().raw, before);
        let now = session.active().unwrap().payloads.values.len();
        assert!(now > rows);
        rows = now;
    }
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn textarea_text_appended_before_a_later_child_rebuilds_the_raw_value() {
    let source = crate::HtmlSource::utf8("<textarea>A<span>B");
    let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
    let mut a = next_text(&mut tokenizer, &mut c);
    let text = session
        .insert_text(&mut c, &node, None, &mut a, None)
        .unwrap()
        .unwrap()
        ._node;
    let span = next_fixture_tag(&mut tokenizer, &mut c, &[115, 112, 97, 110]);
    let child = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut Some(span),
            &node,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &node, &child, None, None).unwrap();
    // The text goes to the end of the first child, which is not the last child, so the
    // units it adds are not at the end of the raw value that the last child ends.
    let mut b = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, Some(&child), &mut b, None)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    assert_eq!(native_text(&session, &text), [65, 66]);
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &node)
            .unwrap()
            .code_units(),
        [65, 66]
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

/// A textarea holding the text A, a span and the text B, in that order, with the tokens after
/// them still to come.
fn textarea_with_two_texts<'a>(
    source: &'a crate::HtmlSource,
) -> (
    ParseControl,
    ParserArenaSession,
    ParserNode,
    ParserNode,
    ParserNode,
    crate::parser::tokenizer::tests::LexerFixture<'a>,
) {
    let (mut c, mut session, node, mut tokenizer) = textarea_owner(source, limits());
    let mut a = next_text(&mut tokenizer, &mut c);
    let first = session
        .insert_text(&mut c, &node, None, &mut a, None)
        .unwrap()
        .unwrap()
        ._node;
    let span = next_fixture_tag(&mut tokenizer, &mut c, &[115, 112, 97, 110]);
    let child = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut Some(span),
            &node,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &node, &child, None, None).unwrap();
    // The span is the last child, so the text after it starts a node of its own.
    let mut b = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut b, None)
        .unwrap();
    (c, session, node, first, child, tokenizer)
}

fn raw_units(
    session: &mut ParserArenaSession,
    c: &mut ParseControl,
    node: &ParserNode,
) -> Vec<u16> {
    session
        .textarea_raw_value(c, node)
        .unwrap()
        .code_units()
        .to_vec()
}

#[test]
fn textarea_text_appended_after_two_text_children_extends_the_raw_value_of_both() {
    let source = crate::HtmlSource::utf8("<textarea>A<span>BC");
    let (mut c, mut session, node, _, _, mut tokenizer) = textarea_with_two_texts(&source);
    assert_eq!(raw_units(&mut session, &mut c, &node), [65, 66]);
    let backing = raw_backing(&mut session, &mut c, &node);
    let mut text = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut text, None)
        .unwrap();
    // The raw value is the text of both children, the second now holding B and C.
    assert_eq!(raw_units(&mut session, &mut c, &node), [65, 66, 67]);
    assert_eq!(raw_backing(&mut session, &mut c, &node), backing);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn textarea_text_appended_after_a_child_was_removed_extends_the_rebuilt_raw_value() {
    let source = crate::HtmlSource::utf8("<textarea>A<span>BC");
    let (mut c, mut session, node, first, child, mut tokenizer) = textarea_with_two_texts(&source);
    // Moving the first text into the span leaves the second as the only text child.
    session.insert(&mut c, &child, &first, None, None).unwrap();
    assert_eq!(raw_units(&mut session, &mut c, &node), [66]);
    let mut text = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut text, None)
        .unwrap();
    assert_eq!(raw_units(&mut session, &mut c, &node), [66, 67]);
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn textarea_repeated_reset_retires_backing_but_keeps_initialized_slots_bounded() {
    // Registry slots: initial raw/textarea/form3, name/namespace2,
    // actual native A1 and first children-change raw1 plus two retired static postconnection list slots plus four real document facts and the textarea empty backing/nonce facts =16, including the creation row.
    // The finite auxiliary cap admits the initial lexical/topology staging,
    // measured from an unconstrained run, then exactly five reset slots;
    // the next reset retains the current raw and latches the slot cap.
    let source = crate::HtmlSource::utf8("<textarea>A");
    let staged = {
        let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, limits());
        let mut text = next_text(&mut tokenizer, &mut c);
        session
            .insert_text(&mut c, &node, None, &mut text, None)
            .unwrap();
        tokenizer.release(&mut c).unwrap();
        let staged = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        session.release(&mut c).unwrap();
        storage_zero(&c);
        staged
    };
    let caps = limits()
        .into_builder()
        .auxiliary_entries(staged + 5)
        .active_formatting_entries(staged + 5)
        .finish()
        .unwrap();
    let (mut c, mut session, node, mut tokenizer) = textarea_owner(&source, caps);
    let mut text = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &node, None, &mut text, None)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    // The text has no registry row: the registry holds 15 rows.
    assert_eq!(session.active().unwrap().payloads.values.len(), 15);
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        staged
    );
    let old = session
        .textarea_raw_value(&mut c, &node)
        .unwrap()
        .block()
        .downgrade();
    for _ in 0..5 {
        session.parser_stack_removed(&mut c, &node, None).unwrap();
    }
    assert!(
        old.upgrade().is_none(),
        "retired backing is actually disposed"
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current,
        staged + 5
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current,
        38
    );
    let current = session
        .textarea_raw_value(&mut c, &node)
        .unwrap()
        .block()
        .downgrade();
    let current_slot = session.textarea_state(&mut c, &node).unwrap().raw;
    let result = session.parser_stack_removed(&mut c, &node, None);
    let first = c.stop().unwrap();
    assert_eq!(result, Err(SessionError::Stop(first)));
    assert!(matches!(
        first,
        HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::AuxiliaryEntries,
            ..
        }
    ));
    let storage = session.active().unwrap();
    // The text has no registry row: the registry holds 20 rows.
    assert_eq!(storage.payloads.values.len(), 20);
    let RetainedPayload::String { value, .. } = &storage.payloads.values[current_slot] else {
        panic!("original current raw retained")
    };
    assert_eq!(value.code_units(), [65]);
    assert!(current.upgrade().is_some());
    assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(s)) if s==first));
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    assert!(current.upgrade().is_none());
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(c.stop(), Some(first));
    storage_zero(&c);
}

#[test]
fn textarea_raw_and_metadata_slots_are_admitted_before_lexical_take() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // One creation attempt under an auxiliary cap. Returns whether it
    // succeeded, whether the lexical tag was taken, the entries held before
    // the attempt and, for a success, the entries it needed.
    let attempt = |cap: Option<u64>| {
        let source = crate::HtmlSource::utf8("<textarea>");
        let mut builder = limits().into_builder();
        if let Some(cap) = cap {
            builder = builder
                .auxiliary_entries(cap)
                .active_formatting_entries(cap);
        }
        let mut c = ParseControl::new(builder.finish().unwrap(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut tokenizer =
            LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let root = session.root(&mut c).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        let tag = next_fixture_tag(
            &mut tokenizer,
            &mut c,
            &[116, 101, 120, 116, 97, 114, 101, 97],
        );
        let backing = tag.name.value.text().block().address();
        let mut input = Some(tag);
        session.allocate_implicit_body(&mut c, None).unwrap();
        let held = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        let result = session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &html,
            None,
            crate::construction::CreationContext::Ordinary,
        );
        let taken = input.is_none();
        if !taken {
            assert_eq!(
                input.as_ref().unwrap().name.value.text().block().address(),
                backing,
                "a tag refused before the take keeps its own name backing"
            );
        }
        let stop = c.stop();
        let needed = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        if let Some(tag) = input.take() {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        (result.map(|_| ()), stop, taken, held, needed)
    };
    let (created, stop, _, held, needed) = attempt(None);
    created.unwrap();
    assert_eq!(stop, None);
    // Every cap that admits the document, html and body but not the whole
    // creation latches the first auxiliary refusal. A cap too small for the
    // raw and metadata slots refuses before the tag is taken, a later cap
    // after; the refusal point never moves back as the cap rises.
    let mut taken_before = false;
    let mut refused_before_take = 0;
    for cap in held..needed {
        let (result, stop, taken, _, _) = attempt(Some(cap));
        assert_eq!(result, Err(SessionError::Stop(stop.unwrap())));
        assert!(
            matches!(
                stop,
                Some(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    limit,
                    ..
                }) if limit == cap
            ),
            "cap {cap}: {stop:?}"
        );
        assert!(
            taken || !taken_before,
            "cap {cap} refused earlier than a smaller cap"
        );
        taken_before |= taken;
        refused_before_take += usize::from(!taken);
    }
    assert!(
        refused_before_take > 0,
        "the raw and metadata slots are admitted before the lexical take"
    );
    let (created, stop, _, _, exact) = attempt(Some(needed));
    created.unwrap();
    assert_eq!(stop, None);
    assert_eq!(exact, needed);
}

#[test]
fn textarea_initial_reset_precedes_parser_association_commit_and_stop_owns_every_slot() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = crate::HtmlSource::utf8("<form><textarea>");
    let cancellation = HtmlParseCancellation::new();
    let mut c = ParseControl::new(limits(), cancellation.clone());
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let root = session.root(&mut c).unwrap();
    let form_tag = next_fixture_tag(&mut tokenizer, &mut c, &[102, 111, 114, 109]);
    let form = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut Some(form_tag),
            &root,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &root, &form, None, None).unwrap();
    let tag = next_fixture_tag(
        &mut tokenizer,
        &mut c,
        &[116, 101, 120, 116, 97, 114, 101, 97],
    );
    session.active_mut().unwrap().cancel_after_textarea_reset = Some(cancellation);
    let mut input = Some(tag);
    assert!(matches!(
        session.allocate_html_tag_for_creation(
            &mut c,
            &mut input,
            &form,
            Some(&form),
            crate::construction::CreationContext::Ordinary
        ),
        Err(SessionError::Stop(_))
    ));
    assert!(input.is_none());
    let storage = session.active().unwrap();
    let state = storage
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::Textarea {
                node: Some(_),
                state,
            } => Some(*state),
            _ => None,
        })
        .unwrap();
    assert!(!state.flags.dirty_value() && !state.flags.user_validity());
    let RetainedPayload::String { value, .. } = &storage.payloads.values[state.raw] else {
        panic!("initial raw owner")
    };
    assert_eq!(value.code_units(), []);
    assert!(storage.payloads.values.iter().any(|entry|matches!(entry,
        RetainedPayload::FormAssociation{node:None,state} if state.kind==HtmlNativeFormAssociationKind::Textarea && state.owner.is_none() && !state.parser_inserted && state.evidence==HtmlParseFormEvidence::Created
    )));
    let first = c.stop().unwrap();
    assert!(matches!(session.freeze(&mut c),Err(SessionError::Stop(stop)) if stop==first));
    tokenizer.release(&mut c).unwrap();
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(c.stop(), Some(first));
    storage_zero(&c);
}

#[test]
fn textarea_raw_child_moves_update_both_parents_and_ignore_descendant_text() {
    let source = crate::HtmlSource::utf8("<textarea>A<textarea>B");
    let (mut c, mut session, left, mut tokenizer) = textarea_owner(&source, limits());
    let mut a = next_text(&mut tokenizer, &mut c);
    let text = session
        .insert_text(&mut c, &left, None, &mut a, None)
        .unwrap()
        .unwrap()
        ._node;
    let tag = next_fixture_tag(
        &mut tokenizer,
        &mut c,
        &[116, 101, 120, 116, 97, 114, 101, 97],
    );
    let root = session.root(&mut c).unwrap();
    let right = session
        .allocate_html_tag_for_creation(
            &mut c,
            &mut Some(tag),
            &root,
            None,
            crate::construction::CreationContext::Ordinary,
        )
        .unwrap();
    session.insert(&mut c, &root, &right, None, None).unwrap();
    let mut b = next_text(&mut tokenizer, &mut c);
    session
        .insert_text(&mut c, &right, None, &mut b, None)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    session.insert(&mut c, &right, &text, None, None).unwrap();
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &left)
            .unwrap()
            .code_units(),
        []
    );
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &right)
            .unwrap()
            .code_units(),
        [66, 65]
    );
    session.insert(&mut c, &left, &right, None, None).unwrap();
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &left)
            .unwrap()
            .code_units(),
        []
    );
    assert_eq!(
        session
            .textarea_raw_value(&mut c, &right)
            .unwrap()
            .code_units(),
        [66, 65]
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
