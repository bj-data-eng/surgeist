use super::super::select::*;
use super::super::tests::{fresh_script_owner, limits, storage_zero};
use super::*;
use crate::parser::tokenizer::{
    states::{Raw, State},
    tests::LexerFixture,
    token::{Event, Token},
};
use crate::parser::tree_builder::ConstructionCapability;

// Literal HTML #selectedness-setting-algorithm oracles exercise the existing
// callable parser boundary before its language decisions are shared with DOM.
fn normalization_case(source: &str, selected: [bool; 3], expected: [bool; 3], changed: bool) {
    let request = request(source);
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let select = named(&nodes, "select", 0);
    let options = [0, 1, 2].map(|i| named(&nodes, "option", i));
    for (index, option) in options.iter().enumerate() {
        let storage = session.active_mut().unwrap();
        let (slot, SelectMetadata::Option(mut state)) =
            storage.select_metadata(&mut c, option.index).unwrap()
        else {
            panic!("actual option metadata")
        };
        state.flags = HtmlOptionFlags::new(selected[index], index != 1);
        storage
            .set_select_metadata(&mut c, slot, SelectMetadata::Option(state))
            .unwrap();
    }
    assert_eq!(
        session
            .active_mut()
            .unwrap()
            .selectedness_setting(&mut c, select.index, None)
            .unwrap(),
        changed
    );
    for (index, option) in options.iter().enumerate() {
        let flags = session.option_state(&mut c, option).unwrap().flags;
        assert_eq!(flags.selectedness(), expected[index]);
        assert_eq!(flags.dirtiness(), index != 1);
    }
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_normalization_keeps_last_selected_even_when_disabled_and_preserves_dirty_flags() {
    normalization_case(
        "<body><select><option>A</option><option>B</option><option disabled>C</option></select></body>",
        [true, true, true],
        [false, false, true],
        true,
    );
    normalization_case(
        "<body><select multiple><option>A</option><option>B</option><option disabled>C</option></select></body>",
        [true, true, true],
        [true, true, true],
        false,
    );
}

#[test]
fn select_normalization_fallback_requires_display_size_one_and_first_enabled_option() {
    for size in ["", " size=1", " size=bad", " size=1tail"] {
        normalization_case(
            &format!(
                "<body><select{size}><option disabled>A</option><option>B</option><option>C</option></select></body>"
            ),
            [false, false, false],
            [false, true, false],
            true,
        );
    }
    for size in [" size=0", " size=-0", " size=2", " multiple"] {
        normalization_case(
            &format!(
                "<body><select{size}><option disabled>A</option><option>B</option><option>C</option></select></body>"
            ),
            [false, false, false],
            [false, false, false],
            false,
        );
    }
    normalization_case(
        "<body><select><option disabled>A</option><option disabled>B</option><option disabled>C</option></select></body>",
        [false, false, false],
        [false, false, false],
        false,
    );
}

fn next(
    lexer: &mut crate::parser::tokenizer::tests::LexerFixture<'_>,
    c: &mut ParseControl,
) -> Token {
    loop {
        match lexer.next(c, false).unwrap().expect("actual fixture token") {
            Event::Token(event) => return event.into_token(c).unwrap(),
            event => event.release(c).unwrap(),
        }
    }
}

/// The actual fresh-driver issuance is reused, then its same controller and
/// arena consume real lexical records. The Vec is test observation/setup only.
fn graph<'a>(
    request: &'a crate::HtmlDocumentParseRequest,
) -> (
    ParseControl,
    ParserArenaSession,
    ConstructionCapability<'a>,
    Vec<(String, ParserNode)>,
) {
    let (mut c, mut session, proof) = fresh_script_owner(request);
    let html_index = session
        .active()
        .unwrap()
        .construction
        .node_at(0)
        .expect("retained construction position")
        .child_at(0)
        .expect("retained child ordinal")
        .position();
    let html = session.active().unwrap().node(&mut c, html_index).unwrap();
    let mut stack = vec![html];
    let mut nodes = vec![];
    let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
    loop {
        let token = next(&mut lexer, &mut c);
        match token.kind {
            TokenKind::Tag(tag) if tag.kind == TagKind::Start => {
                let name = String::from_utf16(tag.name.value.text().code_units()).unwrap();
                let mut input = Some(tag);
                let parent = stack.last().unwrap();
                let node = if matches!(
                    name.as_str(),
                    "select" | "option" | "optgroup" | "selectedcontent"
                ) {
                    session
                        .allocate_select_family_for_creation(
                            &mut c,
                            &mut input,
                            crate::construction::CreationLocation {
                                intended_parent: parent,
                                current_form: None,
                                context: crate::construction::CreationContext::Ordinary,
                            },
                            &token.source,
                            &proof,
                        )
                        .unwrap()
                } else if name == "script" {
                    session
                        .allocate_disabled_parser_script(
                            &mut c,
                            &mut input,
                            None,
                            crate::construction::CreationContext::Ordinary,
                            &proof,
                        )
                        .unwrap()
                } else if name == "input" {
                    session
                        .allocate_input_for_creation(
                            &mut c,
                            &mut input,
                            crate::construction::CreationLocation {
                                intended_parent: parent,
                                current_form: None,
                                context: crate::construction::CreationContext::Ordinary,
                            },
                            &token.source,
                            &proof,
                        )
                        .unwrap()
                } else {
                    session
                        .allocate_html_tag_for_creation(
                            &mut c,
                            &mut input,
                            parent,
                            None,
                            crate::construction::CreationContext::Ordinary,
                        )
                        .unwrap()
                };
                session
                    .insert(&mut c, parent, &node, None, Some(&proof))
                    .unwrap();
                if !matches!(name.as_str(), "hr" | "input") {
                    stack.push(node.clone());
                }
                nodes.push((name.clone(), node));
                if name == "script" {
                    lexer.set_state(State::Raw(Raw::Script), &mut c).unwrap();
                }
                if name == "textarea" {
                    lexer.set_state(State::Raw(Raw::Rcdata), &mut c).unwrap();
                }
            }
            TokenKind::Tag(tag) => {
                let name = String::from_utf16(tag.name.value.text().code_units()).unwrap();
                release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
                let removed = stack.pop().unwrap();
                if name == "script" {
                    session
                        .prepare_disabled_script(&mut c, &removed, &proof)
                        .unwrap();
                    lexer.set_state(State::Data, &mut c).unwrap();
                }
                if name == "textarea" {
                    lexer.set_state(State::Data, &mut c).unwrap();
                }
            }
            TokenKind::Text(text) => {
                session
                    .insert_text(
                        &mut c,
                        stack.last().unwrap(),
                        None,
                        &mut Some(text),
                        Some(&proof),
                    )
                    .unwrap();
            }
            TokenKind::Comment(text) => {
                let node = session
                    .allocate_comment(&mut c, &mut Some(text), None)
                    .unwrap();
                session
                    .insert(&mut c, stack.last().unwrap(), &node, None, Some(&proof))
                    .unwrap();
            }
            TokenKind::ProcessingInstruction { target, data } => {
                let node = session
                    .allocate_processing_instruction(&mut c, &mut Some((target, data)), None)
                    .unwrap();
                session
                    .insert(&mut c, stack.last().unwrap(), &node, None, Some(&proof))
                    .unwrap();
            }
            TokenKind::Eof => break,
            kind => release_token_kind(&mut Some(kind), &mut c).unwrap(),
        }
    }
    lexer.release(&mut c).unwrap();
    (c, session, proof, nodes)
}
fn named(nodes: &[(String, ParserNode)], name: &str, occurrence: usize) -> ParserNode {
    nodes
        .iter()
        .filter(|(local, _)| local == name)
        .nth(occurrence)
        .unwrap()
        .1
        .clone()
}
fn children(session: &ParserArenaSession, node: &ParserNode) -> Vec<usize> {
    session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .children()
        .map(|node| node.position())
        .collect::<Vec<_>>()
        .clone()
}
fn request(source: &str) -> crate::HtmlDocumentParseRequest {
    crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits())
}
fn update(session: &mut ParserArenaSession, c: &mut ParseControl, option: &ParserNode) {
    session
        .active_mut()
        .unwrap()
        .update_for_option(c, option.index)
        .unwrap();
    session
        .active_mut()
        .unwrap()
        .drain_construction_effects(c)
        .unwrap();
}

#[test]
fn select_script_copies_only_started_then_runs_actual_disabled_connection_preparation() {
    for (attributes, body, started, kind, force_async) in [
        ("", "", false, None, true),
        ("async", "", false, None, false),
        ("type=unknown", "x", false, None, true),
        ("", "x", true, None, true),
        ("async", "x", true, None, false),
    ] {
        let request = request(&format!(
            "<body><select><selectedcontent></selectedcontent><option><script {attributes}>{body}</script></option></select></body>"
        ));
        let (mut c, mut session, _proof, nodes) = graph(&request);
        let original = named(&nodes, "script", 0);
        let option = named(&nodes, "option", 0);
        let target = named(&nodes, "selectedcontent", 0);
        assert_eq!(
            session
                .script_metadata(&mut c, &original)
                .unwrap()
                .flags
                .already_started(),
            started
        );
        update(&mut session, &mut c, &option);
        let copy = session
            .active()
            .unwrap()
            .node(&mut c, children(&session, &target)[0])
            .unwrap();
        assert!(!session.same_node(&mut c, &copy, &original).unwrap());
        let state = session.script_metadata(&mut c, &copy).unwrap();
        assert_eq!(state.flags.already_started(), started);
        assert_eq!(state.script_type, kind);
        assert_eq!(state.flags.force_async(), force_async);
        assert_eq!(state.result, HtmlScriptResult::Uninitialized);
        assert!(
            session
                .script_parser_document(&mut c, &copy)
                .unwrap()
                .is_none()
        );
        assert!(
            session
                .script_preparation_document(&mut c, &copy)
                .unwrap()
                .is_none()
        );
        // Empty/unknown clones reach their actual early return before Disabled;
        // already-started clones return at step1. No callback is elided.
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn select_connected_empty_script_clone_text_merge_runs_children_changed_preparation() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option><script></script></option></select><div>A</div><div>B</div></body>",
    );
    let (mut c, mut session, proof, nodes) = graph(&request);
    let option = named(&nodes, "option", 0);
    let target = named(&nodes, "selectedcontent", 0);
    update(&mut session, &mut c, &option);
    let copy = session
        .active()
        .unwrap()
        .node(&mut c, children(&session, &target)[0])
        .unwrap();
    assert!(
        !session
            .script_metadata(&mut c, &copy)
            .unwrap()
            .flags
            .already_started()
    );
    // A separately admitted real lexical replay supplies two useful text
    // payloads on the same controller; no detached synthetic Text is fabricated.
    let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
    let mut texts = Vec::new();
    loop {
        let token = next(&mut lexer, &mut c);
        match token.kind {
            TokenKind::Text(text) => texts.push(text),
            TokenKind::Eof => break,
            kind => release_token_kind(&mut Some(kind), &mut c).unwrap(),
        }
    }
    session
        .insert_text(
            &mut c,
            &copy,
            None,
            &mut Some(texts.remove(0)),
            Some(&proof),
        )
        .unwrap();
    let before = session.script_metadata(&mut c, &copy).unwrap();
    assert!(before.flags.already_started());
    assert_eq!(before.script_type, Some(HtmlScriptKind::Classic));
    assert!(
        session
            .script_preparation_document(&mut c, &copy)
            .unwrap()
            .is_some()
    );
    let slots_before_merge = session.active().unwrap().payloads.values.len();
    session
        .insert_text(
            &mut c,
            &copy,
            None,
            &mut Some(texts.remove(0)),
            Some(&proof),
        )
        .unwrap();
    let added_merge_slots = session.active().unwrap().payloads.values.len() - slots_before_merge;
    assert_eq!(session.script_metadata(&mut c, &copy).unwrap(), before);
    let text = children(&session, &copy);
    assert_eq!(text.len(), 1);
    assert!(
        matches!(&session.active().unwrap().construction.node_at(text[0]).expect("retained construction position").kind(), HtmlNodeKind::Text(value) if value == &DocumentString::from("AB"))
    );
    lexer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    // replaceData registers the incoming and merged strings. It invokes only
    // the parent's childrenChanged hook, not an insertion static-list owner.
    assert_eq!(added_merge_slots, 2);
}

#[test]
fn select_clones_managed_hidden_textarea_and_exact_comment_pi_text_backing_independently() {
    let mut units =
        "<body><select><selectedcontent></selectedcontent><option><input type=hidden value='"
            .encode_utf16()
            .collect::<Vec<_>>();
    let exact = [0xd800, 0xd83d, 0xde42, 0xdc00];
    units.extend(exact);
    units.extend("'><textarea>A&amp;B&#13;C</textarea><!--K--><?go data?>".encode_utf16());
    units.extend(exact);
    units.extend("</option></select></body>".encode_utf16());
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf16(DocumentString::from_utf16(units)),
        limits(),
    );
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let option = named(&nodes, "option", 0);
    let target = named(&nodes, "selectedcontent", 0);
    update(&mut session, &mut c, &option);
    let originals = children(&session, &option);
    let copies = children(&session, &target);
    assert_eq!(copies.len(), originals.len());
    for (original, copy) in originals.iter().zip(&copies) {
        assert_ne!(original, copy);
        let storage = session.active().unwrap();
        match (
            &storage
                .construction
                .node_at(*original)
                .expect("retained construction position")
                .kind(),
            &storage
                .construction
                .node_at(*copy)
                .expect("retained construction position")
                .kind(),
        ) {
            (HtmlNodeKind::Text(a), HtmlNodeKind::Text(b))
            | (HtmlNodeKind::Comment(a), HtmlNodeKind::Comment(b)) => {
                assert_eq!(a, b);
                assert!(!Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()));
            }
            (HtmlNodeKind::ProcessingInstruction(a), HtmlNodeKind::ProcessingInstruction(b)) => {
                assert_eq!(a.target(), b.target());
                assert_eq!(a.data(), b.data());
                assert!(!Arc::ptr_eq(
                    a.data().shared_code_units(),
                    b.data().shared_code_units()
                ));
            }
            (HtmlNodeKind::Element(a), HtmlNodeKind::Element(b)) => {
                assert_eq!(a.name(), b.name());
                assert_eq!(a.attributes(), b.attributes());
                assert!(!Arc::ptr_eq(
                    a.name().local_name().shared_code_units(),
                    b.name().local_name().shared_code_units()
                ));
            }
            _ => panic!("literal shallow clone kind"),
        }
    }
    let hidden = session.active().unwrap().node(&mut c, copies[0]).unwrap();
    let textarea = session.active().unwrap().node(&mut c, copies[1]).unwrap();
    let mut buffer = Utf16Buffer::new(&mut c).unwrap();
    session
        .append_input_default_value(&mut c, &hidden, &mut buffer)
        .unwrap();
    assert_eq!(buffer.units(), exact);
    buffer.release(&mut c).unwrap();
    assert_eq!(
        session.textarea_raw_value(&mut c, &textarea).unwrap(),
        &DocumentString::from("A&B\rC")
    );
    assert!(
        !session
            .textarea_state(&mut c, &textarea)
            .unwrap()
            .flags
            .dirty_value()
    );
    assert!(
        !session
            .form_association_state(&mut c, &hidden)
            .unwrap()
            .parser_inserted
    );
    assert!(
        session
            .form_association_owner(&mut c, &hidden)
            .unwrap()
            .is_none()
    );
    let mut frozen = session.freeze(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_stopped_clone_prefixes_retain_all_scratch_and_nested_payload_owners_for_cleanup() {
    let mut stops = 0;
    let mut progressed = false;
    for extra in (0..12_000).step_by(113) {
        let request = request(
            "<body><select><selectedcontent></selectedcontent><option><i data-x=v>A&amp;B</i></option></select></body>",
        );
        let (mut c, mut session, _proof, nodes) = graph(&request);
        let option = named(&nodes, "option", 0);
        let target = named(&nodes, "selectedcontent", 0);
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - extra)
            .unwrap();
        let result = session
            .active_mut()
            .unwrap()
            .update_for_option(&mut c, option.index)
            .and_then(|()| {
                session
                    .active_mut()
                    .unwrap()
                    .drain_construction_effects(&mut c)
            });
        if result.is_err() {
            stops += 1;
            let first = c
                .stop()
                .expect("all useful admitted work failures are canonical sticky stops");
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            assert_eq!(
                session.option_state(&mut c, &option).err(),
                Some(SessionError::Stop(first))
            );
            let storage = session.active().unwrap();
            for parent in 0..storage.construction.storage().len() {
                for child in storage.construction.node_at(parent).unwrap().children() {
                    assert_eq!(child.parent().map(|node| node.position()), Some(parent));
                }
            }
            progressed |= storage.construction.storage().len() > nodes.len() + 3;
            assert!(children(&session, &target).len() <= 1);
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    assert!(stops > 20);
    assert!(progressed);
}

#[test]
fn select_owners_active_and_frozen_facts_are_qualified_and_unavailable_for_wrong_kind() {
    let request = request(
        "<body><select><button><selectedcontent></selectedcontent></button><option>A</option></select></body>",
    );
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let select = named(&nodes, "select", 0);
    let option = named(&nodes, "option", 0);
    let target = named(&nodes, "selectedcontent", 0);
    let _: SelectState = session.select_state(&mut c, &select).unwrap();
    let state: OptionState = session.option_state(&mut c, &option).unwrap();
    assert!(state.flags.selectedness());
    assert!(!state.flags.dirtiness());
    let _: SelectedcontentState = session.selectedcontent_state(&mut c, &target).unwrap();
    let cached = session
        .option_nearest_select(&mut c, &option)
        .unwrap()
        .unwrap();
    assert!(session.same_node(&mut c, &cached, &select).unwrap());
    assert_eq!(
        session.option_state(&mut c, &select),
        Err(SessionError::SelectMetadataUnavailable)
    );
    let mut other = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let foreign = other.root(&mut c).unwrap();
    assert_eq!(
        session.select_state(&mut c, &foreign),
        Err(SessionError::ForeignNode)
    );
    other.release(&mut c).unwrap();
    let mut frozen = session.freeze(&mut c).unwrap();
    let document = frozen.document().unwrap();
    let mut ids = vec![];
    let mut next = vec![document.root().id()];
    while let Some(id) = next.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(e) = node.kind()
            && e.name().local_name() == &DocumentString::from("option")
        {
            ids.push(id.clone());
        }
        next.extend(node.children());
    }
    assert_eq!(frozen.option_state(&mut c, &ids[0]).unwrap(), state);
    assert!(
        frozen
            .option_nearest_select(&mut c, &ids[0])
            .unwrap()
            .is_some()
    );
    let foreign = HtmlDocument::builder(
        HtmlConstructionLimits::new(1, 1, 1, 1, 1),
        HtmlDocumentMode::NoQuirks,
    )
    .unwrap()
    .finish();
    assert_eq!(
        frozen.option_state(&mut c, &foreign.root().id()).err(),
        Some(SessionError::ForeignNode)
    );
    assert!(c.stop().is_none());
    drop(foreign);

    drop(ids);
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_removal_queue_refuses_freeze_without_stop_then_real_checkpoint_replaces_content() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option>A</option></select></body>",
    );
    let (mut c, mut session, proof, nodes) = graph(&request);
    let option = named(&nodes, "option", 0);
    let target = named(&nodes, "selectedcontent", 0);
    // Explicit intrinsic update of a completed option; the parser consumer's
    // real pop is independently exercised by the unchanged driver regression.
    session
        .active_mut()
        .unwrap()
        .update_for_option(&mut c, option.index)
        .unwrap();
    session
        .active_mut()
        .unwrap()
        .drain_construction_effects(&mut c)
        .unwrap();
    let old = children(&session, &target);
    assert_eq!(old.len(), 1);
    session.remove(&mut c, &option, Some(&proof)).unwrap();
    assert!(
        session
            .option_nearest_select(&mut c, &option)
            .unwrap()
            .is_none()
    );
    assert_eq!(children(&session, &target), old);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::PendingIntrinsicWork)
    );
    assert_eq!(c.stop(), None);
    assert!(session.storage.is_some());
    session
        .perform_intrinsic_checkpoint(&mut c, &proof)
        .unwrap();
    assert!(children(&session, &target).is_empty());
    assert_eq!(c.stop(), None);
    let mut frozen = session.freeze(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_same_parent_reinsert_runs_actual_removal_cache_and_queues_without_move_hook() {
    let request = request("<body><select><option>A</option></select></body>");
    let (mut c, mut session, proof, nodes) = graph(&request);
    let select = named(&nodes, "select", 0);
    let option = named(&nodes, "option", 0);
    session
        .insert(&mut c, &select, &option, Some(&option), Some(&proof))
        .unwrap();
    let cached = session
        .option_nearest_select(&mut c, &option)
        .unwrap()
        .unwrap();
    assert!(session.same_node(&mut c, &cached, &select).unwrap());
    assert_eq!(children(&session, &select), vec![option.index]);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::PendingIntrinsicWork)
    );
    assert_eq!(c.stop(), None);
    session
        .perform_intrinsic_checkpoint(&mut c, &proof)
        .unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_clone_cancellation_and_pending_cleanup_never_drains_useful_work() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option>A</option></select></body>",
    );
    let (mut c, mut session, proof, nodes) = graph(&request);
    let option = named(&nodes, "option", 0);
    let target = named(&nodes, "selectedcontent", 0);
    session
        .active_mut()
        .unwrap()
        .update_for_option(&mut c, option.index)
        .unwrap();
    session
        .active_mut()
        .unwrap()
        .drain_construction_effects(&mut c)
        .unwrap();
    session.remove(&mut c, &option, Some(&proof)).unwrap();
    let old = children(&session, &target);
    request.cancellation().cancel();
    assert_eq!(
        session.perform_intrinsic_checkpoint(&mut c, &proof),
        Err(SessionError::Stop(ParseStop::Cancelled))
    );
    assert_eq!(children(&session, &target), old);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(ParseStop::Cancelled))
    );
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(c.stop(), Some(ParseStop::Cancelled));
}

#[test]
fn select_option_list_excludes_nested_select_hr_option_datalist_and_second_optgroup_subtrees() {
    let request = request(
        "<body><select><option>A<option>B</option></option><hr><option>C</option><datalist><option>D</option></datalist><optgroup><optgroup><option>E</option></optgroup></optgroup><select><option>F</option></select></select></body>",
    );
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let select = named(&nodes, "select", 0);
    let list = session
        .active_mut()
        .unwrap()
        .retain_work_list(
            &mut c,
            select.index,
            WorkListKind::Options { detached: None },
        )
        .unwrap();
    let storage = session.active().unwrap();
    assert_eq!(
        storage.work_list_node(&mut c, list, 0).unwrap(),
        Some(named(&nodes, "option", 0).index)
    );
    assert_eq!(
        storage.work_list_node(&mut c, list, 1).unwrap(),
        Some(named(&nodes, "option", 2).index)
    );
    assert_eq!(storage.work_list_node(&mut c, list, 2).unwrap(), None);
    session
        .active_mut()
        .unwrap()
        .release_work_list(&mut c, list)
        .unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_fifo_consumes_two_actual_removal_tasks_in_source_order_and_keeps_exact_targets() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option>A</option><option>B</option></select><select><selectedcontent></selectedcontent><option>C</option><option>D</option></select></body>",
    );
    let (mut c, mut session, proof, nodes) = graph(&request);
    let first = named(&nodes, "select", 0);
    let second = named(&nodes, "select", 1);
    for index in [0, 2] {
        let option = named(&nodes, "option", index);
        update(&mut session, &mut c, &option);
        session.remove(&mut c, &option, Some(&proof)).unwrap();
    }
    let storage = session.active().unwrap();
    let tasks = storage.effects.tasks.as_slice();
    assert!(
        matches!(&tasks[0], Some(QueuedTask { task: IntrinsicTask::UpdateSelect(index), .. }) if *index == first.index)
    );
    assert!(
        matches!(&tasks[1], Some(QueuedTask { task: IntrinsicTask::UpdateSelect(index), .. }) if *index == second.index)
    );
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::PendingIntrinsicWork)
    );
    session
        .perform_intrinsic_checkpoint(&mut c, &proof)
        .unwrap();
    for (target, expected) in [(0, "B"), (1, "D")] {
        let target = named(&nodes, "selectedcontent", target);
        let child = children(&session, &target);
        assert_eq!(child.len(), 1);
        assert!(
            matches!(&session.active().unwrap().construction.node_at(child[0]).expect("retained construction position").kind(), HtmlNodeKind::Text(value) if value == &DocumentString::from(expected))
        );
    }
    let storage = session.active().unwrap();
    assert_eq!(storage.effects.dequeue, 2);
    assert!(storage.effects.tasks.as_slice().iter().all(Option::is_none));
    assert!(storage.effects.task.is_none());
    assert!(!storage.effects.checkpoint);
    let mut frozen = session.freeze(&mut c).unwrap();
    frozen.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_fifo_stopped_current_task_and_static_lists_remain_owned_without_pending_publication() {
    let mut with_current = false;
    for extra in (0..3000).step_by(97) {
        let request = request(
            "<body><select><selectedcontent></selectedcontent><option>A</option><option><i>B</i></option></select></body>",
        );
        let (mut c, mut session, proof, nodes) = graph(&request);
        let option = named(&nodes, "option", 0);
        update(&mut session, &mut c, &option);
        session.remove(&mut c, &option, Some(&proof)).unwrap();
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - extra)
            .unwrap();
        if let Err(SessionError::Stop(first)) = session.perform_intrinsic_checkpoint(&mut c, &proof)
        {
            let storage = session.active().unwrap();
            with_current |= storage.effects.task.is_some();
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            assert_eq!(
                session.perform_intrinsic_checkpoint(&mut c, &proof),
                Err(SessionError::Stop(first))
            );
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            session.release(&mut c).unwrap();
            storage_zero(&c);
            assert_eq!(c.stop(), Some(first));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        } else {
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }
    assert!(
        with_current,
        "real dequeue transfers its task before bounded effects stop"
    );
}

#[test]
fn select_creation_metadata_and_empty_option_snapshot_have_independent_exact_slot_caps() {
    // Document/html contribute two append-field index entries before lexical
    // take; the successful Select record contributes the third. The pre-take
    // metadata cap is 16. The completed Select needs 28 coexisting entries from
    // nodes, index, causes, nonce, fields, metadata, staging and snapshot;
    // twenty-seven stops after take at the reset option snapshot.
    for cap in [16, 27, 28] {
        let caps = limits()
            .into_builder()
            .auxiliary_entries(cap)
            .finish()
            .unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8("<select>"), caps);
        let (mut c, mut session, proof) = fresh_script_owner(&request);
        let parent = session.active().unwrap().node(&mut c, 1).unwrap();
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let token = next(&mut lexer, &mut c);
        let TokenKind::Tag(tag) = token.kind else {
            panic!("actual select")
        };
        let mut input = Some(tag);
        let result = session.allocate_select_family_for_creation(
            &mut c,
            &mut input,
            crate::construction::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::CreationContext::Ordinary,
            },
            &token.source,
            &proof,
        );
        if cap == 28 {
            let node = result.unwrap();
            assert!(input.is_none());
            assert!(
                !session
                    .select_state(&mut c, &node)
                    .unwrap()
                    .flags
                    .user_validity()
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                28
            );
            assert_eq!(
                c.usage().resource(HtmlParseResource::ManagedNodes).current,
                3
            );
        } else {
            let Err(SessionError::Stop(first)) = result else {
                panic!("exact selected auxiliary cap")
            };
            assert!(
                matches!(first, ParseStop::LimitExceeded { resource: HtmlParseResource::AuxiliaryEntries, limit, .. } if limit == cap)
            );
            assert_eq!(input.is_some(), cap == 16);
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
            assert_eq!(c.stop(), Some(first));
        }
        if let Some(tag) = input {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn select_creation_rejects_real_alias_and_foreign_parent_before_retaining_metadata_or_taking_tag() {
    let request = request("<select>");
    let (mut c, mut session, proof) = fresh_script_owner(&request);
    let parent = session.active().unwrap().node(&mut c, 1).unwrap();
    let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
    let token = next(&mut lexer, &mut c);
    let TokenKind::Tag(tag) = token.kind else {
        panic!("actual select")
    };
    let mut input = Some(tag);
    let original_slots = session.active().unwrap().payloads.values.len();
    let alias = input.as_ref().unwrap().name.value.text().clone();
    assert_eq!(
        session
            .allocate_select_family_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary
                },
                &token.source,
                &proof
            )
            .err(),
        Some(SessionError::AliasedPayload)
    );
    assert!(input.is_some());
    assert_eq!(
        session.active().unwrap().payloads.values.len(),
        original_slots
    );
    drop(alias);
    let mut other = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let foreign = other.root(&mut c).unwrap();
    assert_eq!(
        session
            .allocate_select_family_for_creation(
                &mut c,
                &mut input,
                crate::construction::CreationLocation {
                    intended_parent: &foreign,
                    current_form: None,
                    context: crate::construction::CreationContext::Ordinary
                },
                &token.source,
                &proof
            )
            .err(),
        Some(SessionError::ForeignNode)
    );
    assert!(input.is_some());
    assert_eq!(
        session.active().unwrap().payloads.values.len(),
        original_slots
    );
    assert!(c.stop().is_none());
    release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
    lexer.release(&mut c).unwrap();
    other.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn select_text_clone_every_interruption_keeps_ticket_binding_coherent_for_disposal() {
    for extra in 0..1200 {
        let request = request("<body><select><option>A</option></select></body>");
        let (mut c, mut session, _proof, nodes) = graph(&request);
        let option = named(&nodes, "option", 0);
        let text = children(&session, &option)[0];
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - extra)
            .unwrap();
        let _result = session
            .active_mut()
            .unwrap()
            .clone_node(&mut c, text, DocumentRef::Owned(0));
        if c.stop().is_some() {
            assert!(session.freeze(&mut c).is_err());
        }
        let release = session.release(&mut c);
        assert!(
            release.is_ok(),
            "interruption credit {extra}: actual copied-string/node binding must remain disposal-coherent: {release:?}"
        );
        storage_zero(&c);
    }
}

fn non_element_clone_fixture(
    kind: usize,
) -> (
    ParseControl,
    ParserArenaSession,
    usize,
    crate::HtmlParseCancellation,
) {
    let request = request("<body><select><option>A<!--C--><?p D?></option></select></body>");
    let (mut c, mut session, _proof, nodes) = graph(&request);
    let option = named(&nodes, "option", 0);
    let source = children(&session, &option)[kind];
    assert!(matches!(
        (
            &session
                .active()
                .unwrap()
                .construction
                .node_at(source)
                .expect("retained construction position")
                .kind(),
            kind
        ),
        (HtmlNodeKind::Text(_), 0)
            | (HtmlNodeKind::Comment(_), 1)
            | (HtmlNodeKind::ProcessingInstruction(_), 2)
    ));
    // Make the allocator's two reserve attempts no-growth operations. This is
    // real admitted fixture capacity, not an allocation substitute.
    let storage = session.active_mut().unwrap();
    storage
        .construction
        .apply(
            &mut c,
            bounded::HtmlConstructionCommand::ReserveRecords { additional: 1 },
            &[],
        )
        .unwrap();
    storage
        .construction
        .reserve_child_tickets(&mut c, 1)
        .unwrap();
    // PI owns its map plus two copied fields, a node-document binding,
    // and the source-qualified construction-cause binding;
    // Text/comment need fewer. All later registry reserves are no-growth.
    c.reserve_vector(
        &mut storage.payloads.values,
        &mut storage.payloads.allocation,
        5,
    )
    .unwrap();
    (c, session, source, request.cancellation().clone())
}

fn non_element_clone_binding_boundary(kind: usize) {
    // The observed envelope bounds coverage only. Correctness below is the
    // independent invariant that every installed field has its exact registered
    // backing, at every finite cut through the composed shared-owner phases.
    let (mut probe_control, mut probe, source, cancellation) = non_element_clone_fixture(kind);
    drop(cancellation);
    let before = probe_control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    probe
        .active_mut()
        .unwrap()
        .clone_node(&mut probe_control, source, DocumentRef::Owned(0))
        .unwrap();
    let complete = probe_control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    probe.release(&mut probe_control).unwrap();
    storage_zero(&probe_control);
    assert!(
        complete <= 32_768,
        "bounded single-node clone envelope: {complete}"
    );
    for (extra, cancel) in (0..=complete)
        .map(|extra| (extra, false))
        .chain(std::iter::once((complete, true)))
    {
        let (mut c, mut session, source, cancellation) = non_element_clone_fixture(kind);
        let nodes = session.active().unwrap().construction.storage().len();
        let slots = session.active().unwrap().payloads.values.len();
        let remaining =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - extra)
            .unwrap();
        let result =
            session
                .active_mut()
                .unwrap()
                .clone_node(&mut c, source, DocumentRef::Owned(0));
        let installed = session.active().unwrap().construction.storage().len() - nodes;
        if result.is_ok() {
            assert_eq!(extra, complete);
            assert_eq!(installed, 1);
        } else {
            assert!(matches!(result, Err(SessionError::Stop(_))));
        }
        if cancel {
            cancellation.cancel();
            assert!(c.checkpoint().is_err());
        } else if result.is_ok() {
            // The complete measured envelope leaves zero credits. Establish
            // the terminal cleanup boundary explicitly, rather than spending
            // nonexistent credits on an active cleanup ownership walk.
            assert!(c.charge(WorkCategory::StateStep, 1).is_err());
        }
        let first = c.stop();
        if let Some(first) = first {
            assert_eq!(
                session.freeze(&mut c).err(),
                Some(SessionError::Stop(first))
            );
        }
        let coherent = session.active().unwrap().payloads.values[slots..]
            .iter()
            .all(|payload| match payload {
                RetainedPayload::String {
                    location, value, ..
                } => {
                    !matches!(location, StringLocation::Detached)
                        || Arc::strong_count(value.shared_code_units()) == 1
                }
                _ => true,
            });
        let installed_bound=(nodes..nodes+installed).all(|node| {
            let storage=session.active().unwrap();
            let bound=|location: StringLocation,value:&DocumentString|storage.payloads.values[slots..].iter().any(|payload|matches!(payload,RetainedPayload::String{location:actual,value:stored,..} if *actual==location && Arc::ptr_eq(value.shared_code_units(),stored.shared_code_units())));
            match storage.construction.node_at(node).unwrap().kind() {
                HtmlNodeKind::Text(value)=>bound(StringLocation::Text(node),value),
                HtmlNodeKind::Comment(value)=>bound(StringLocation::Comment(node),value),
                HtmlNodeKind::ProcessingInstruction(value)=>bound(StringLocation::ProcessingInstructionTarget(node),value.target())&&bound(StringLocation::ProcessingInstructionData(node),value.data()),
                _=>false,
            }
        });
        let release = session.release(&mut c);
        assert!(
            coherent && installed_bound && release.is_ok(),
            "kind {kind}, credit {extra}, installed {installed}: binding coherence {coherent}, release {release:?}, first {first:?}"
        );
        storage_zero(&c);
        assert_eq!(c.stop(), first);
        assert_eq!(
            result.is_ok(),
            extra == complete,
            "kind {kind}, extra {extra}, complete {complete}, first {first:?}"
        );
        assert!(
            installed <= 1,
            "one clone creates at most one actual record"
        );
        assert!(
            matches!(
                first,
                Some(ParseStop::Cancelled) if cancel
            ) || matches!(
                first,
                Some(ParseStop::LimitExceeded { resource: HtmlParseResource::WorkCredits, .. }) if !cancel
            )
        );
    }
}

#[test]
fn select_text_clone_native_binding_boundary_keeps_cleanup_owned() {
    non_element_clone_binding_boundary(0);
}

#[test]
fn select_comment_clone_native_binding_boundary_keeps_cleanup_owned() {
    non_element_clone_binding_boundary(1);
}

#[test]
fn select_pi_clone_native_binding_boundary_keeps_cleanup_owned() {
    non_element_clone_binding_boundary(2);
}

#[test]
fn select_same_source_foreign_lexical_owner_cannot_transfer_or_consume_nested_tickets() {
    let request = request("<select>");
    let (mut c, mut session, proof) = fresh_script_owner(&request);
    let parent = session.active().unwrap().node(&mut c, 1).unwrap();
    let mut foreign = ParseControl::new(limits(), crate::HtmlParseCancellation::new());
    foreign.admit_source(request.source()).unwrap();
    let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut foreign).unwrap();
    let token = next(&mut lexer, &mut foreign);
    let TokenKind::Tag(tag) = token.kind else {
        panic!("real foreign select token")
    };
    let mut input = Some(tag);
    let original = input
        .as_ref()
        .unwrap()
        .name
        .value
        .text()
        .shared_code_units()
        .as_ptr();
    let slots = session.active().unwrap().payloads.values.len();
    let result = session.allocate_select_family_for_creation(
        &mut c,
        &mut input,
        crate::construction::CreationLocation {
            intended_parent: &parent,
            current_form: None,
            context: crate::construction::CreationContext::Ordinary,
        },
        &token.source,
        &proof,
    );
    let first = c.stop().unwrap();
    assert_eq!(result.err(), Some(SessionError::Stop(first)));
    assert!(matches!(
        first,
        ParseStop::InternalInvariant(crate::parser::control::Invariant::ForeignControl)
    ));
    assert_eq!(
        input
            .as_ref()
            .unwrap()
            .name
            .value
            .text()
            .shared_code_units()
            .as_ptr(),
        original
    );
    assert_eq!(session.active().unwrap().payloads.values.len(), slots);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(first))
    );
    release_token_kind(&mut input.map(TokenKind::Tag), &mut foreign).unwrap();
    lexer.release(&mut foreign).unwrap();
    storage_zero(&foreign);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(first));
}

#[test]
fn select_script_parent_preparation_precedes_every_static_list_admission_prefix() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option><script></script></option></select><div>A</div></body>",
    );
    let mut saw_list = false;
    for credit in (0..240_000).step_by(113) {
        let (mut c, mut session, proof, nodes) = graph(&request);
        let option = named(&nodes, "option", 0);
        let target = named(&nodes, "selectedcontent", 0);
        update(&mut session, &mut c, &option);
        let copy = session
            .active()
            .unwrap()
            .node(&mut c, children(&session, &target)[0])
            .unwrap();
        assert!(
            !session
                .script_metadata(&mut c, &copy)
                .unwrap()
                .flags
                .already_started()
        );
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let mut text = loop {
            let token = next(&mut lexer, &mut c);
            if let TokenKind::Text(text) = token.kind {
                break Some(text);
            }
            release_token_kind(&mut Some(token.kind), &mut c).unwrap();
        };
        let base = session.active().unwrap().payloads.values.len();
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - credit)
            .unwrap();
        let result = session.insert_text(&mut c, &copy, None, &mut text, Some(&proof));
        let storage = session.active().unwrap();
        let collected = storage.payloads.values[base..]
            .iter()
            .any(|p| matches!(p, RetainedPayload::WorkList(_)));
        let prepared = storage.payloads.values.iter().any(|p| matches!(p,
            RetainedPayload::Script { node: Some(node), metadata } if *node == copy.index && metadata.flags.already_started()));
        saw_list |= collected;
        if result.is_err() {
            let first = c.stop().expect("canonical stopped prefix");
            assert!(session.freeze(&mut c).is_err());
            assert_eq!(c.stop(), Some(first));
        }
        if let Some(text) = text {
            text.release(&mut c).unwrap();
        }
        lexer.release(&mut c).unwrap();
        if let Err(error) = session.release(&mut c) {
            assert!(matches!(error, SessionError::Stop(_)) && c.stop().is_some());
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        assert!(
            !collected || prepared,
            "static collection preceded parent preparation at {credit}"
        );
        if result.is_ok() {
            break;
        }
    }
    assert!(saw_list, "actual static collection admission exercised");
}

#[test]
fn select_fragment_insert_interruption_keeps_original_sibling_snapshot_and_coherent_prefix() {
    let request = request(
        "<body><select><selectedcontent></selectedcontent><option><div><option selected>A</option></div><div><option selected>B</option></div></option></select></body>",
    );
    // Calibrate the two concrete pre-insertion boundaries through the actual
    // step producers and the exact canonical drain admissions. This derives
    // credits from the changed group/cursor layout; expected topology is still
    // the literal original two-root sibling list, never parser output.
    let (mut measured, mut owner, _, nodes) = graph(&request);
    let outer = named(&nodes, "option", 0);
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let storage = owner.active_mut().unwrap();
    storage
        .update_for_option(&mut measured, outer.index)
        .unwrap();
    let mut boundaries = Vec::new();
    loop {
        if storage.effects.form.is_none()
            && let Some(EffectFrame::FragmentInsertion {
                parent,
                cursor,
                stage: FragmentStage::Insert,
                ..
            }) = storage.effects.current
            && cursor <= 1
            && storage
                .construction
                .node_at(parent)
                .expect("retained construction position")
                .child_count()
                == cursor
        {
            boundaries.push(
                measured
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current
                    - start,
            );
            if cursor == 1 {
                break;
            }
        }
        measured.require_identity(&storage.control).unwrap();
        measured.charge(WorkCategory::RecordVisit, 2).unwrap();
        if storage.effects.form.is_some() {
            storage.step_form_effects(&mut measured).unwrap();
        } else if let Some(frame) = storage.effects.current.as_ref() {
            measured
                .admit_atomic_layout(mem::size_of::<EffectFrame>())
                .unwrap();
            measured
                .charge(WorkCategory::CopyByte, mem::size_of::<EffectFrame>() as u64)
                .unwrap();
            frame.admit_clone(&mut measured).unwrap();
            let frame = frame.clone();
            storage.step_effect(&mut measured, frame).unwrap();
        } else {
            payload_write::<Option<EffectFrame>>(&mut measured, 1).unwrap();
            match storage.effects.frames.try_pop(&mut measured).unwrap() {
                Some(EffectFrame::Topology(form)) => {
                    payload_write::<Option<FormEffects>>(&mut measured, 1).unwrap();
                    storage.effects.form = Some(form);
                }
                Some(frame) => storage
                    .write_current_effect(&mut measured, Some(frame))
                    .unwrap(),
                None => panic!("literal target requires a two-root fragment insertion"),
            }
        }
    }
    owner.release(&mut measured).unwrap();
    storage_zero(&measured);
    assert_eq!(boundaries.len(), 2);
    let mut saw_insert = false;
    for credit in boundaries
        .into_iter()
        .flat_map(|base| [base, base + 1, base + 2])
        .chain((0..2_000_000).step_by(1999))
    {
        let (mut c, mut session, _, nodes) = graph(&request);
        let outer = named(&nodes, "option", 0);
        let target = named(&nodes, "selectedcontent", 0);
        let remaining = request.limits().work_credits()
            - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - credit)
            .unwrap();
        let result = (|| {
            let storage = session.active_mut()?;
            storage.update_for_option(&mut c, outer.index)?;
            storage.drain_construction_effects(&mut c)
        })();
        let storage = session.active().unwrap();
        if let Some(EffectFrame::FragmentInsertion {
            parent,
            list,
            stage: FragmentStage::Insert,
            ..
        }) = storage.effects.current
        {
            saw_insert = true;
            assert_eq!(parent, target.index);
            let RetainedPayload::WorkList(Some(roots)) = &storage.payloads.values[list] else {
                panic!("owned original siblings");
            };
            assert_eq!(roots.len(), 2);
            let linked = &storage
                .construction
                .node_at(parent)
                .expect("retained construction position")
                .children()
                .map(|node| node.position())
                .collect::<Vec<_>>();
            assert!(linked.len() <= 2);
            assert_eq!(linked.as_slice(), &roots.as_slice()[..linked.len()]);
            for (position, root) in roots.as_slice().iter().enumerate() {
                assert_eq!(
                    storage
                        .construction
                        .node_at(*root)
                        .expect("retained construction position")
                        .parent()
                        .map(|node| node.position()),
                    (position < linked.len()).then_some(parent)
                );
            }
        }
        if result.is_err() {
            let first = c.stop().expect("canonical stopped batch");
            assert!(session.freeze(&mut c).is_err());
            assert_eq!(c.stop(), Some(first));
        }
        let running_before_cleanup = c.stop().is_none();
        let released = session.release(&mut c);
        if let Err(SessionError::Stop(
            stop @ ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            },
        )) = released
        {
            // The enlarged fixed Range receiver changes the exact credit at
            // which useful effects finish. Running alias-preflight cleanup can
            // now encounter a real one-short boundary before storage.take().
            assert!(running_before_cleanup && result.is_ok());
            assert!(session.storage.is_some());
            assert_eq!(c.stop(), Some(stop));
            let at_stop = c.first_stop_usage().unwrap().clone();
            let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
            session.release(&mut c).unwrap();
            assert_eq!(c.stop(), Some(stop));
            assert_eq!(c.first_stop_usage(), Some(&at_stop));
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                work
            );
        } else {
            assert!(
                released.is_ok(),
                "fragment credit {credit}: {released:?}, first {:?}",
                c.stop()
            );
        }
        storage_zero(&c);
        if result.is_ok() {
            break;
        }
    }
    assert!(
        saw_insert,
        "real retained fragment insertion prefix exercised"
    );
}

fn captured_effect_owner() -> (
    ParseControl,
    ParserArenaSession,
    crate::HtmlCaptureDocumentId,
) {
    let mut builder = crate::HtmlCaptureBuilder::new(
        crate::HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 10, 10),
        crate::HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            crate::HtmlDocumentMode::NoQuirks,
            crate::HtmlScriptingStatus::Disabled,
            crate::HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let element = builder
        .add_element(
            &document,
            crate::QualifiedName::new(crate::Namespace::html(), None, "div"),
            crate::OrderedAttributes::default(),
            crate::HtmlCapturedIsValue::Absent,
            crate::HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        crate::HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap())
            .unwrap();
    let document = context.target_document().clone();
    let request =
        crate::HtmlFragmentParseRequest::new(crate::HtmlSource::utf8(""), context, limits())
            .unwrap();
    let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
    c.admit_source(request.source()).unwrap();
    let mut session = ParserArenaSession::new(
        &mut c,
        RootKind::Document(crate::HtmlDocumentMode::NoQuirks),
    )
    .unwrap();
    session.initialize_standalone_document(&mut c).unwrap();
    session.initialize_fragment(&mut c, &request).unwrap();
    (c, session, document)
}

#[test]
fn captured_current_frame_survives_stops_before_and_after_identity_clone_admission() {
    for after_provider in [false, true] {
        let (mut c, mut owner, document) = captured_effect_owner();
        let storage = owner.active_mut().unwrap();
        storage
            .start_effect(
                &mut c,
                EffectFrame::AdoptSubtree {
                    root: 0,
                    next: None,
                    document: DocumentRef::Captured(document.clone()),
                },
            )
            .unwrap();
        // Two header visits and the complete frame layout precede the clone.
        // The next cut admits the clone but refuses its publication receiver.
        let credit = 2 + mem::size_of::<EffectFrame>() as u64 + u64::from(after_provider);
        let remaining =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - credit)
            .unwrap();
        let error = storage.drain_construction_effects(&mut c).unwrap_err();
        let first = c.stop().unwrap();
        assert_eq!(error, SessionError::Stop(first));
        assert!(matches!(storage.effects.current.as_ref(),
            Some(EffectFrame::AdoptSubtree { next: None, document: DocumentRef::Captured(id), .. }) if id == &document));
        assert!(storage.effects.transfer.is_none());
        owner.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
    }
    let (mut c, mut owner, document) = captured_effect_owner();
    let storage = owner.active_mut().unwrap();
    storage
        .start_effect(
            &mut c,
            EffectFrame::AdoptSubtree {
                root: 0,
                next: None,
                document: DocumentRef::Captured(document),
            },
        )
        .unwrap();
    storage.drain_construction_effects(&mut c).unwrap();
    assert!(storage.effects.current.is_none());
    owner.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn captured_parent_and_child_transfer_remain_owned_when_parent_clone_or_push_stops() {
    for after_provider in [false, true] {
        let (mut c, mut owner, document) = captured_effect_owner();
        let storage = owner.active_mut().unwrap();
        storage
            .start_effect(
                &mut c,
                EffectFrame::AdoptSubtree {
                    root: 0,
                    next: None,
                    document: DocumentRef::Captured(document.clone()),
                },
            )
            .unwrap();
        // Child transfer and parent receiver layouts are admitted before the
        // conditional parent identity transfer and work-list reservation.
        let credit =
            2 * (mem::size_of::<Option<EffectFrame>>() as u64 + 1) + u64::from(after_provider);
        let remaining =
            limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
        c.charge(WorkCategory::StateStep, remaining - credit)
            .unwrap();
        let error = storage
            .start_effect(&mut c, EffectFrame::AppendClone { parent: 0, copy: 0 })
            .unwrap_err();
        let first = c.stop().unwrap();
        assert_eq!(error, SessionError::Stop(first));
        assert!(matches!(storage.effects.current.as_ref(),
            Some(EffectFrame::AdoptSubtree { document: DocumentRef::Captured(id), .. }) if id == &document));
        assert!(matches!(
            storage.effects.transfer,
            Some(EffectFrame::AppendClone { parent: 0, copy: 0 })
        ));
        assert_eq!(storage.effects.frames.len(), 0);
        owner.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), Some(first));
    }
}
