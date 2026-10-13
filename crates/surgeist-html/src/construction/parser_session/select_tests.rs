use super::super::tests::{fresh_script_owner, limits, storage_zero};
use super::provenance::ConstructionCauseRef;
use super::*;
use crate::parser::tokenizer::{
    states::{Raw, State},
    tests::LexerFixture,
    token::{Event, Token},
};
use crate::parser::tree_builder::ConstructionCapability;

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
    let html_index = session.active().unwrap().records[0].children[0];
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
                            crate::construction::parser_session::CreationLocation {
                                intended_parent: parent,
                                current_form: None,
                                context:
                                    crate::construction::parser_session::CreationContext::Ordinary,
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
                            crate::construction::parser_session::CreationContext::Ordinary,
                            &proof,
                        )
                        .unwrap()
                } else if name == "input" {
                    session
                        .allocate_input_for_creation(
                            &mut c,
                            &mut input,
                            crate::construction::parser_session::CreationLocation {
                                intended_parent: parent,
                                current_form: None,
                                context:
                                    crate::construction::parser_session::CreationContext::Ordinary,
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
                            crate::construction::parser_session::CreationContext::Ordinary,
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
    session.active().unwrap().records[node.index]
        .children
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
                .already_started,
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
        assert_eq!(state.already_started, started);
        assert_eq!(state.script_type, kind);
        assert_eq!(state.force_async, force_async);
        assert_eq!(state.result, ScriptResult::Uninitialized);
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
            .already_started
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
    assert!(before.already_started);
    assert_eq!(before.script_type, Some(ScriptType::Classic));
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
        matches!(&session.active().unwrap().records[text[0]].kind, HtmlNodeKind::Text(value) if value == &DocumentString::from("AB"))
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
            &storage.records[*original].kind,
            &storage.records[*copy].kind,
        ) {
            (HtmlNodeKind::Text(a), HtmlNodeKind::Text(b))
            | (HtmlNodeKind::Comment(a), HtmlNodeKind::Comment(b)) => {
                assert_eq!(a, b);
                assert!(!Arc::ptr_eq(a.backing(), b.backing()));
            }
            (HtmlNodeKind::ProcessingInstruction(a), HtmlNodeKind::ProcessingInstruction(b)) => {
                assert_eq!(a.target(), b.target());
                assert_eq!(a.data(), b.data());
                assert!(!Arc::ptr_eq(a.data().backing(), b.data().backing()));
            }
            (HtmlNodeKind::Element(a), HtmlNodeKind::Element(b)) => {
                assert_eq!(a.name(), b.name());
                assert_eq!(a.attributes(), b.attributes());
                assert!(!Arc::ptr_eq(
                    a.name().local_name().backing(),
                    b.name().local_name().backing()
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
            for (parent, record) in storage.records.iter().enumerate() {
                for child in &record.children {
                    assert_eq!(storage.records[*child].parent, Some(parent));
                }
            }
            progressed |= storage.records.len() > nodes.len() + 3;
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
            matches!(&session.active().unwrap().records[child[0]].kind, HtmlNodeKind::Text(value) if value == &DocumentString::from(expected))
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
    // Fresh fixture: Document/html two node slots plus html name/URI/attrs
    // three registry slots plus actual root facts2/html binding1/completion1 =9. Prologue stack coexistence needs10. Select adds
    // Select/form metadata2, name/URI/attrs3, node1, ordered-attribute staging1, reset option snapshot1:
    // completed empty Select exactly18; seventeen stops after take at snapshot.
    // Actual root/html/select node causes add three permanent registry slots.
    for cap in [12, 20, 21] {
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
            crate::construction::parser_session::CreationLocation {
                intended_parent: &parent,
                current_form: None,
                context: crate::construction::parser_session::CreationContext::Ordinary,
            },
            &token.source,
            &proof,
        );
        if cap == 21 {
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
                21
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
            assert_eq!(input.is_some(), cap == 12);
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
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &parent,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary
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
                crate::construction::parser_session::CreationLocation {
                    intended_parent: &foreign,
                    current_form: None,
                    context: crate::construction::parser_session::CreationContext::Ordinary
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
        (&session.active().unwrap().records[source].kind, kind),
        (HtmlNodeKind::Text(_), 0)
            | (HtmlNodeKind::Comment(_), 1)
            | (HtmlNodeKind::ProcessingInstruction(_), 2)
    ));
    // Make the allocator's two reserve attempts no-growth operations. This is
    // real admitted fixture capacity, not an allocation substitute.
    let storage = session.active_mut().unwrap();
    c.reserve_vector(&mut storage.records, &mut storage.records_allocation, 1)
        .unwrap();
    c.reserve_vector(
        &mut storage.children_allocations,
        &mut storage.tickets_allocation,
        1,
    )
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

fn non_element_copy_prefix(kind: usize) -> u64 {
    let (mut c, mut session, source, cancellation) = non_element_clone_fixture(kind);
    drop(cancellation);
    let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
    // The new actual clone lineage is part of field preparation, with the
    // original fixed native allocation/binding windows preserved below.
    let previous = session
        .active_mut()
        .unwrap()
        .begin_node_copy(&mut c, source)
        .unwrap();
    // Execute only the actual independent field-copy producers. No clone or
    // native allocation is used to produce the expected allocator/binding cost.
    c.charge(WorkCategory::RecordVisit, 1).unwrap();
    let locations = match kind {
        0 => vec![StringLocation::Text(source)],
        1 => vec![StringLocation::Comment(source)],
        2 => vec![
            StringLocation::ProcessingInstructionTarget(source),
            StringLocation::ProcessingInstructionData(source),
        ],
        _ => unreachable!(),
    };
    for location in locations {
        let storage = session.active_mut().unwrap();
        let slot = storage.field_slot(&mut c, location).unwrap();
        let (value, _) = storage.copy_registered_field(&mut c, slot).unwrap();
        drop(value);
    }
    let prefix = c.usage().resource(HtmlParseResource::WorkCredits).current - before;
    drop(previous);
    session.release(&mut c).unwrap();
    storage_zero(&c);
    prefix
}

fn non_element_clone_binding_boundary(kind: usize) {
    let copy_prefix = non_element_copy_prefix(kind);
    // allocate_owned(false): two no-growth reserves visit twice each; one
    // empty child Vec/ticket writes twice and copies its real fixed layout;
    // node_owned copies a handle and visits its provider; one Record copy plus
    // two record/ticket writes. Identity/storage/depth checks add no work.
    let allocation = 4
        + 2
        + mem::size_of::<Vec<usize>>() as u64
        + mem::size_of::<VectorCapacity<usize>>() as u64
        + 1
        + mem::size_of::<ParserNode>() as u64
        + mem::size_of::<Record>() as u64
        + 2;
    let fields = if kind == 2 { 2 } else { 1 };
    let binding_bytes = fields * mem::size_of::<StringLocation>() as u64;
    let binding = binding_bytes + fields;
    let usage = mem::size_of::<usize>() as u64 + 1;
    // validate_document_ref visits the root and its first Document slot (2).
    // The allocation then visits the same root/slot (2), state (1), and three
    // no-growth reserves (6). Binding-slot reserve/record costs R+3;
    // binding copy/write costs O+1: R+O+15, with both qualifications retained.
    let documents =
        mem::size_of::<RetainedPayload>() as u64 + mem::size_of::<Option<usize>>() as u64 + 15;
    // A PI additionally reserves its empty map (R+3) and its future node
    // binding (O+1), before copying either original field.
    let map = if kind == 2 {
        mem::size_of::<RetainedPayload>() as u64 + mem::size_of::<Option<usize>>() as u64 + 4
    } else {
        0
    };
    // The preserved PI data is literal D: three nonempty scalar reads12,
    // name-buffer constructor1/push6, offset U+1, three EOF reads6,
    // empty-map clear2, ready copy/write2 = U+30. Invalid whole grammar
    // remains a successfully established empty map, rather than an error.
    let grammar = if kind == 2 {
        let (mut fee_control, mut fee_session, source, cancellation) =
            non_element_clone_fixture(kind);
        drop(cancellation);
        // The new copied PI data field follows the map and target slots. Its
        // literal D contributes one source-origin visit, fixed origin copy,
        // handle clone and one first-capacity OwnedVec push (one planned slot).
        let field_visits = fee_session.active().unwrap().payloads.values.len() as u64 + 3;
        let storage = fee_session.active_mut().unwrap();
        let slot = storage
            .field_slot(
                &mut fee_control,
                StringLocation::ProcessingInstructionData(source),
            )
            .unwrap();
        // The real copied data has one Alias header over the literal D leaf.
        // Measure only that concrete cursor primitive, independently of clone
        // allocation/binding/PI parsing; all grammar/native expectations remain
        // literal below. The PI mapper still visits the yielded origin once.
        let group = storage
            .copied_slot_group(&mut fee_control, slot)
            .unwrap()
            .unwrap();
        let evidence = storage.evidence.as_ref().unwrap();
        let before = fee_control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut cursor =
            provenance::GroupOriginCursor::new(evidence, &group, &mut fee_control).unwrap();
        assert!(cursor.next(evidence, &mut fee_control).unwrap().is_some());
        assert!(cursor.next(evidence, &mut fee_control).unwrap().is_none());
        cursor.release(&mut fee_control).unwrap();
        let cursor_credits = fee_control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        fee_session.release(&mut fee_control).unwrap();
        storage_zero(&fee_control);
        let origin = mem::size_of::<TextOrigin>() as u64;
        mem::size_of::<usize>() as u64
            + 30
            + field_visits
            + cursor_credits
            + 1 // actual PI mapper visit, separate from cursor traversal
            + origin
            + 1
            + 1
            + origin
            + 1
            + origin
            + 1
    } else {
        0
    };
    // Source-qualified node allocation preadmits one cause handle, its
    // fixed native binding, and its no-growth retained registry slot.
    let cause_entry = mem::size_of::<ConstructionCauseRef>() as u64
        + mem::size_of::<Option<usize>>() as u64
        + mem::size_of::<RetainedPayload>() as u64
        + 6;
    let allocated_prefix = copy_prefix + allocation + documents + map + cause_entry;
    // Restore the actual previously retained parser cause after the fully
    // committed clone: one visit and its fixed copy/write.
    let complete =
        allocated_prefix + binding + usage + grammar + mem::size_of::<Option<usize>>() as u64 + 2;
    // Preserve the exact old native-installed/binding-not-admitted window and
    // both binding sub-boundaries, plus a successful clone and cancellation at
    // the first checkpoint after its fully bound native record.
    for (extra, cancel) in [
        (allocated_prefix, false),
        (allocated_prefix + binding_bytes, false),
        (allocated_prefix + binding, false),
        (allocated_prefix + binding + usage, false),
        (complete, false),
        (complete, true),
    ] {
        let (mut c, mut session, source, cancellation) = non_element_clone_fixture(kind);
        let nodes = session.active().unwrap().records.len();
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
        let installed = session.active().unwrap().records.len() - nodes;
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
            // The hand-derived complete budget leaves zero credits. Establish
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
                        || Arc::strong_count(value.backing()) == 1
                }
                _ => true,
            });
        let release = session.release(&mut c);
        assert!(
            coherent && release.is_ok(),
            "kind {kind}, credit {extra}, installed {installed}: binding coherence {coherent}, release {release:?}, first {first:?}"
        );
        storage_zero(&c);
        assert_eq!(c.stop(), first);
        assert_eq!(
            result.is_ok(),
            extra == complete,
            "kind {kind}, extra {extra}, complete {complete}, first {first:?}"
        );
        assert_eq!(installed, usize::from(extra >= allocated_prefix + binding));
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
    let original = input.as_ref().unwrap().name.value.text().backing().as_ptr();
    let slots = session.active().unwrap().payloads.values.len();
    let result = session.allocate_select_family_for_creation(
        &mut c,
        &mut input,
        crate::construction::parser_session::CreationLocation {
            intended_parent: &parent,
            current_form: None,
            context: crate::construction::parser_session::CreationContext::Ordinary,
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
        input.as_ref().unwrap().name.value.text().backing().as_ptr(),
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
                .already_started
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
            RetainedPayload::Script { node: Some(node), metadata } if *node == copy.index && metadata.already_started));
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
            && storage.records[parent].children.len() == cursor
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
        } else if let Some(frame) = storage.effects.current {
            measured
                .admit_atomic_layout(mem::size_of::<EffectFrame>())
                .unwrap();
            measured
                .charge(WorkCategory::CopyByte, mem::size_of::<EffectFrame>() as u64)
                .unwrap();
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
            let linked = &storage.records[parent].children;
            assert!(linked.len() <= 2);
            assert_eq!(linked.as_slice(), &roots.as_slice()[..linked.len()]);
            for (position, root) in roots.as_slice().iter().enumerate() {
                assert_eq!(
                    storage.records[*root].parent,
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
