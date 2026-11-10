use super::*;
use surgeist_html::{
    construction::{ConstructionNodeKind, HtmlDocumentMode},
    elements::script::{HtmlScriptKind, HtmlScriptResult},
    values::{Attribute, DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

#[test]
fn disabled_preparation_retains_actual_documents_and_uninitialized_result_through_freeze() {
    for (attributes, kind) in [
        ("src=''", HtmlScriptKind::Classic),
        ("src='' type=module async", HtmlScriptKind::Module),
        ("src='' type=importmap", HtmlScriptKind::ImportMap),
        (
            "src='' type=speculationrules",
            HtmlScriptKind::SpeculationRules,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head><script {attributes}>")),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
        let initial = session.script_metadata(&mut c, &node).unwrap();
        assert!(!initial.flags.force_async() && !initial.flags.already_started());
        assert_eq!(initial.script_type, None);
        let root = session.root(&mut c).unwrap();
        let parser_document = session
            .script_parser_document(&mut c, &node)
            .unwrap()
            .unwrap();
        assert!(session.same_node(&mut c, &root, &parser_document).unwrap());
        assert!(
            session
                .script_preparation_document(&mut c, &node)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            session
                .prepare_disabled_script(&mut c, &node, &capability)
                .unwrap(),
            DisabledScriptPreparation::Disabled(kind)
        );
        let prepared = session.script_metadata(&mut c, &node).unwrap();
        assert!(prepared.flags.already_started() && !prepared.flags.force_async());
        assert_eq!(prepared.script_type, Some(kind));
        assert!(
            !prepared.flags.external_file()
                && !prepared.flags.ready_to_be_parser_executed()
                && !prepared.flags.delaying_load()
        );
        assert_eq!(prepared.result, HtmlScriptResult::Uninitialized);
        assert!(prepared.steps_when_result_ready.is_none());
        let preparation_document = session
            .script_preparation_document(&mut c, &node)
            .unwrap()
            .unwrap();
        assert!(
            session
                .same_node(&mut c, &root, &preparation_document)
                .unwrap()
        );
        assert_eq!(
            session
                .prepare_disabled_script(&mut c, &node, &capability)
                .unwrap(),
            DisabledScriptPreparation::AlreadyStarted
        );
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(frozen.script_metadata(&mut c, &node).unwrap(), prepared);
        let document = frozen.document().unwrap();
        assert_eq!(document.root().children().count(), 1);
        frozen.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn empty_disconnected_and_unknown_returns_keep_source_prefix_metadata_effects() {
    for (attributes, connected, text, outcome, force_async) in [
        ("", true, false, DisabledScriptPreparation::Empty, true),
        (
            "async",
            true,
            false,
            DisabledScriptPreparation::Empty,
            false,
        ),
        (
            "src=''",
            false,
            false,
            DisabledScriptPreparation::Disconnected,
            true,
        ),
        (
            "src='' type=' module '",
            true,
            false,
            DisabledScriptPreparation::UnrecognizedType,
            true,
        ),
    ] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(format!("<html><head><script {attributes}>")),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, connected, text);
        assert_eq!(
            session
                .prepare_disabled_script(&mut c, &node, &capability)
                .unwrap(),
            outcome
        );
        let metadata = session.script_metadata(&mut c, &node).unwrap();
        assert_eq!(metadata.flags.force_async(), force_async);
        assert!(!metadata.flags.already_started());
        assert_eq!(metadata.script_type, None);
        assert!(
            session
                .script_parser_document(&mut c, &node)
                .unwrap()
                .is_none()
        );
        assert!(
            session
                .script_preparation_document(&mut c, &node)
                .unwrap()
                .is_none()
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script>x"),
        limits(),
    );
    let (mut c, mut session, capability, node) = fresh_script(&request, true, true);
    assert_eq!(
        session
            .prepare_disabled_script(&mut c, &node, &capability)
            .unwrap(),
        DisabledScriptPreparation::Disabled(HtmlScriptKind::Classic)
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn script_eof_marks_started_without_preparation_and_cancelled_marker_has_no_effect() {
    for cancel in [false, true] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script>"),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
        let initial = session.script_metadata(&mut c, &node).unwrap();
        if cancel {
            request.cancellation().cancel();
        }
        let result = session.mark_script_eof(&mut c, &node, &capability);
        if cancel {
            assert_eq!(result, Err(SessionError::Stop(HtmlParseStop::Cancelled)));
            let (_, retained) = session
                .active()
                .unwrap()
                .payloads
                .values
                .iter()
                .find_map(|value| {
                    if let RetainedPayload::Script { node, metadata } = value {
                        Some((*node, metadata.clone()))
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(retained, initial);
            assert!(matches!(
                session.freeze(&mut c),
                Err(SessionError::Stop(HtmlParseStop::Cancelled))
            ));
        } else {
            result.unwrap();
            let marked = session.script_metadata(&mut c, &node).unwrap();
            assert!(marked.flags.already_started() && !marked.flags.force_async());
            assert_eq!(marked.script_type, None);
            assert_eq!(marked.preparation_document, None);
            assert_eq!(marked.parser_document, initial.parser_document);
            assert_eq!(
                session
                    .prepare_disabled_script(&mut c, &node, &capability)
                    .unwrap(),
                DisabledScriptPreparation::AlreadyStarted
            );
        }
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
}

#[test]
fn real_standalone_capability_cannot_be_reused_for_another_arena_on_the_same_control() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script>"),
        limits(),
    );
    let (mut c, mut original, capability, node) = fresh_script(&request, true, false);
    let mut unrelated =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let stop = HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance);
    let result = unrelated.mark_script_eof(&mut c, &node, &capability);
    assert_eq!(result, Err(SessionError::Stop(stop)));
    assert_eq!(c.stop(), Some(stop));
    assert_eq!(
        unrelated
            .active()
            .unwrap()
            .construction
            .test_storage()
            .len(),
        1
    );
    assert!(original.active().unwrap().payloads.values.iter().any(|entry| matches!(entry, RetainedPayload::Script { metadata, .. } if !metadata.flags.already_started())));
    assert!(matches!(unrelated.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop));
    unrelated.release(&mut c).unwrap();
    original.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
}

#[test]
fn preparation_clears_parser_document_before_ordinary_connectivity_work_can_stop() {
    let source = "<html><head><script src=x>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut baseline, mut session, capability, node) = fresh_script(&request, true, false);
    let setup_work = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    // Establish only the actual unchanged owner/node/metadata-read cost.
    // This performs no preparation or ordinary ancestry walk.
    capability
        .validate_session(&mut baseline, &session)
        .unwrap();
    let storage = session.active().unwrap();
    let index = storage.index(&mut baseline, &node).unwrap();
    let (_, initial) = script_metadata(&storage.payloads, &mut baseline, index).unwrap();
    assert_eq!(initial.parser_document, Some(DocumentRef::Owned(0)));
    let qualification_work = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - setup_work;
    // Cursor initialization and the step-1 reply precede HTML step 3.
    // Each copies the actual cursor and writes one state record. Then one
    // metadata-slot visit, fixed payload copy and write retain the clear.
    // Connectivity is step 7, after this required parser-document clear.
    let cursor_prefix_work = 2
        * (mem::size_of::<surgeist_html::bounded::script::HtmlScriptPreparationCursor>() as u64
            + 1);
    let first_write_work = 1 + mem::size_of::<ScriptMetadata>() as u64 + 1;
    let cap = setup_work + qualification_work + cursor_prefix_work + first_write_work;
    session.release(&mut baseline).unwrap();
    storage_zero(&baseline);

    let limits = limits().into_builder().work_credits(cap).finish().unwrap();
    let request = crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
    let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        setup_work
    );
    let result = session.prepare_disabled_script(&mut c, &node, &capability);
    assert!(matches!(
        result,
        Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        }))
    ));
    let retained = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .find_map(|entry| match entry {
            RetainedPayload::Script { metadata, .. } => Some(metadata.clone()),
            _ => None,
        })
        .unwrap();
    let first_stop = c.stop().unwrap();
    assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first_stop));
    let stopped_work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    let high_water = c
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes)
        .high_water;
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(first_stop));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        stopped_work
    );
    assert_eq!(
        c.usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water,
        high_water
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        3
    );
    assert_eq!(
        retained.parser_document, None,
        "HTML step 3 must be retained before ordinary connectivity work at step 7"
    );
    assert!(!retained.flags.already_started() && !retained.flags.force_async());
    assert_eq!(retained.script_type, None);
    assert_eq!(retained.preparation_document, None);
}

#[test]
fn empty_preparation_work_is_independent_of_actual_ordinary_ancestor_depth() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let source = format!("<html><head><script>{}", "<div>".repeat(16));
    let mut observed_work = Vec::new();
    for depth in [0, 16] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source.clone()),
            limits(),
        );
        let (mut c, mut session, capability, node) = fresh_script(&request, false, false);
        let html_index = session
            .active()
            .unwrap()
            .construction
            .node_at(0)
            .expect("retained construction position")
            .child_at(0)
            .expect("retained child ordinal")
            .position();
        let mut parent = session.active().unwrap().node(&mut c, html_index).unwrap();
        let mut tokenizer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        for _ in 0..depth {
            let tag = next_fixture_tag(&mut tokenizer, &mut c, &[0x64, 0x69, 0x76]);
            let child = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
            session.insert(&mut c, &parent, &child, None, None).unwrap();
            parent = child;
        }
        session.insert(&mut c, &parent, &node, None, None).unwrap();
        tokenizer.release(&mut c).unwrap();
        let before = c.usage().resource(HtmlParseResource::WorkCredits).current;
        assert_eq!(
            session.prepare_disabled_script(&mut c, &node, &capability),
            Ok(DisabledScriptPreparation::Empty)
        );
        observed_work.push(c.usage().resource(HtmlParseResource::WorkCredits).current - before);
        let retained = session.script_metadata(&mut c, &node).unwrap();
        assert_eq!(retained.parser_document, None);
        assert!(retained.flags.force_async() && !retained.flags.already_started());
        assert_eq!(retained.script_type, None);
        assert_eq!(retained.preparation_document, None);
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            3 + depth
        );
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }
    // HTML step 6 returns Empty; ordinary connectedness at step 7 is never
    // needed. Both actual trees have the same script payload/qualifications.
    assert_eq!(observed_work[0], observed_work[1]);
}

#[test]
fn actual_template_domain_adoption_qualifies_document_and_only_import_revokes_inventory() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
        limits(),
    );
    let (mut c, mut session, capability, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
    session.insert(&mut c, &div, &script, None, None).unwrap();
    let initial = session.script_metadata(&mut c, &script).unwrap();
    assert_eq!(session.active().unwrap().standalone_ineligibility, None);
    let mut foreign_control = control();
    assert!(matches!(
        session.insert(&mut foreign_control, &contents, &div, None, None),
        Err(SessionError::Stop(_))
    ));
    assert_eq!(session.active().unwrap().standalone_ineligibility, None);
    let mut foreign_arena =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let foreign_root = foreign_arena.root(&mut c).unwrap();
    assert_eq!(
        session.insert(&mut c, &contents, &foreign_root, None, None),
        Err(SessionError::ForeignNode)
    );
    assert_eq!(
        session.insert(&mut c, &contents, &div, Some(&script), None),
        Err(SessionError::WrongSibling)
    );
    assert_eq!(session.active().unwrap().standalone_ineligibility, None);
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(div.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        None
    );
    foreign_arena.release(&mut c).unwrap();
    // The ordinary div carries the actual script descendant into the real
    // template-owned fragment. No script-name guess or callback flag is used.
    session.insert(&mut c, &contents, &div, None, None).unwrap();
    assert_eq!(session.active().unwrap().standalone_ineligibility, None);
    let document = session
        .node_document(&mut c, &script)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    let contents_document = session
        .node_document(&mut c, &contents)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    assert!(
        session
            .same_node(&mut c, &document, &contents_document)
            .unwrap()
    );
    let root = session.root(&mut c).unwrap();
    assert!(!session.same_node(&mut c, &root, &document).unwrap());
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(script.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(div.index)
    );
    assert_eq!(session.script_metadata(&mut c, &script).unwrap(), initial);
    session.remove(&mut c, &div, None).unwrap();
    assert_eq!(
        session
            .active()
            .unwrap()
            .construction
            .node_at(div.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        None
    );
    session.allocate_element(&mut c, html("span")).unwrap();
    assert_eq!(
        session.active().unwrap().standalone_ineligibility,
        Some(HtmlParseUnsupportedState::Provenance)
    );
    let stop = HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance);
    assert_eq!(
        session.prepare_disabled_script(&mut c, &script, &capability),
        Err(SessionError::Stop(stop))
    );
    let retained = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .find_map(|entry| {
            if let RetainedPayload::Script { metadata, .. } = entry {
                Some(metadata.clone())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(retained, initial);
    assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop));
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
}

#[test]
fn cancelled_domain_insert_preserves_ordinary_eligibility_and_all_actual_owners() {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script src=x><template><div>"),
        limits(),
    );
    let (mut c, mut session, _, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
    session.insert(&mut c, &div, &script, None, None).unwrap();
    request.cancellation().cancel();
    assert_eq!(
        session.insert(&mut c, &contents, &div, None, None),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    );
    let storage = session.active().unwrap();
    assert_eq!(storage.standalone_ineligibility, None);
    assert_eq!(
        storage
            .construction
            .node_at(div.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        None
    );
    assert_eq!(
        storage
            .construction
            .node_at(script.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(div.index)
    );
    assert!(
        storage
            .construction
            .node_at(contents.index)
            .expect("retained construction position")
            .child_count()
            == 0
    );
    let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
    assert!(matches!(
        session.freeze(&mut c),
        Err(SessionError::Stop(HtmlParseStop::Cancelled))
    ));
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        c.usage().resource(HtmlParseResource::WorkCredits).current,
        work
    );
    assert_eq!(
        c.usage().resource(HtmlParseResource::ManagedNodes).current,
        7
    );
}

#[test]
fn exact_and_one_short_domain_insert_work_admit_eligibility_only_with_topology() {
    let source = "<html><head><script src=x><template><div>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut baseline, mut session, _, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut baseline, &mut session);
    session
        .insert(&mut baseline, &div, &script, None, None)
        .unwrap();
    session.validate_node(&mut baseline, &contents).unwrap();
    session.validate_node(&mut baseline, &div).unwrap();
    session
        .active_mut()
        .unwrap()
        .insert_indices(&mut baseline, contents.index, div.index, None)
        .unwrap();
    let exact = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session.release(&mut baseline).unwrap();
    storage_zero(&baseline);
    // Postcommit nonce traversal: fixed inspection125 plus eight actual visits.
    let atomic = exact - 133;
    for cap in [atomic - 1, atomic, exact] {
        let limits = limits().into_builder().work_credits(cap).finish().unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
        let (mut c, mut session, _, script) = fresh_script(&request, false, false);
        let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
        session.insert(&mut c, &div, &script, None, None).unwrap();
        // Both actual handles are already qualified by setup. Exercise the
        // fixed commit owner separately from its subsequent useful drain.
        session.validate_node(&mut c, &contents).unwrap();
        session.validate_node(&mut c, &div).unwrap();
        let result =
            session
                .active_mut()
                .unwrap()
                .insert_indices(&mut c, contents.index, div.index, None);
        let storage = session.active().unwrap();
        assert_eq!(
            storage
                .construction
                .node_at(script.index)
                .expect("retained construction position")
                .parent()
                .map(|node| node.position()),
            Some(div.index)
        );
        if cap == exact {
            assert_eq!(result, Ok(()));
            assert_eq!(
                storage
                    .construction
                    .node_at(div.index)
                    .expect("retained construction position")
                    .parent()
                    .map(|node| node.position()),
                Some(contents.index)
            );
            assert_eq!(
                storage
                    .construction
                    .node_at(contents.index)
                    .expect("retained construction position")
                    .children()
                    .map(|node| node.position())
                    .collect::<Vec<_>>(),
                [div.index]
            );
            assert_eq!(storage.standalone_ineligibility, None);
            assert_eq!(c.stop(), None);
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            if cap == atomic - 1 {
                assert_eq!(
                    storage
                        .construction
                        .node_at(div.index)
                        .expect("retained construction position")
                        .parent()
                        .map(|node| node.position()),
                    None
                );
                assert!(
                    storage
                        .construction
                        .node_at(contents.index)
                        .expect("retained construction position")
                        .child_count()
                        == 0
                );
            } else {
                assert_eq!(
                    storage
                        .construction
                        .node_at(div.index)
                        .expect("retained construction position")
                        .parent()
                        .map(|node| node.position()),
                    Some(contents.index)
                );
                assert_eq!(
                    storage
                        .construction
                        .node_at(contents.index)
                        .expect("retained construction position")
                        .children()
                        .map(|node| node.position())
                        .collect::<Vec<_>>(),
                    [div.index]
                );
            }
            assert_eq!(storage.standalone_ineligibility, None);
            let first = c.stop().unwrap();
            assert!(
                matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
            );
        }
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            7
        );
    }
}

#[test]
fn public_associated_domain_insert_stop_after_commit_never_publishes_the_completed_links() {
    let source = "<html><head><script src=x><template><div>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut measured, mut session, _, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut measured, &mut session);
    session
        .insert(&mut measured, &div, &script, None, None)
        .unwrap();
    let start = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session
        .insert(&mut measured, &contents, &div, None, None)
        .unwrap();
    let whole = measured
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - start;
    session.release(&mut measured).unwrap();
    storage_zero(&measured);
    let (mut c, mut session, _, script) = fresh_script(&request, false, false);
    let (contents, div) = actual_template_and_div(request.source(), &mut c, &mut session);
    session.insert(&mut c, &div, &script, None, None).unwrap();
    let unused =
        limits().work_credits() - c.usage().resource(HtmlParseResource::WorkCredits).current;
    c.charge(HtmlParseWorkCategory::StateStep, unused - whole + 1)
        .unwrap();
    let Err(SessionError::Stop(first)) = session.insert(&mut c, &contents, &div, None, None) else {
        panic!("real interrupted postcommit drain")
    };
    assert!(matches!(
        first,
        HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            requested_delta: 1,
            ..
        }
    ));
    let storage = session.active().unwrap();
    assert_eq!(
        storage
            .construction
            .node_at(div.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(contents.index)
    );
    assert_eq!(
        storage
            .construction
            .node_at(contents.index)
            .expect("retained construction position")
            .children()
            .map(|node| node.position())
            .collect::<Vec<_>>(),
        [div.index]
    );
    assert_eq!(
        storage
            .construction
            .node_at(script.index)
            .expect("retained construction position")
            .parent()
            .map(|node| node.position()),
        Some(div.index)
    );
    assert_eq!(storage.standalone_ineligibility, None);
    assert_eq!(
        session.freeze(&mut c).err(),
        Some(SessionError::Stop(first))
    );
    assert_eq!(session.root(&mut c).err(), Some(SessionError::Stop(first)));
    assert_eq!(
        session.insert(&mut c, &contents, &div, None, None),
        Err(SessionError::Stop(first))
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(first));
}

#[test]
fn preparation_work_boundaries_retain_step15_step16_and_complete_step18() {
    let source = "<html><head><script src=''>";
    let request =
        crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits());
    let (mut baseline, mut session, capability, node) = fresh_script(&request, true, false);
    session
        .prepare_disabled_script(&mut baseline, &node, &capability)
        .unwrap();
    let exact = baseline
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    session.release(&mut baseline).unwrap();
    storage_zero(&baseline);

    // Derive the step-16 write boundary independently of the successful
    // preparation's observed total. These probes perform qualification and
    // the actual node-document lookup, without running preparation.
    let (mut probe, mut session, capability, node) = fresh_script(&request, true, false);
    let setup_work = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    capability.validate_session(&mut probe, &session).unwrap();
    let storage = session.active().unwrap();
    let index = storage.index(&mut probe, &node).unwrap();
    let (_, initial) = script_metadata(&storage.payloads, &mut probe, index).unwrap();
    assert_eq!(initial.parser_document, Some(DocumentRef::Owned(0)));
    let qualified = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let qualification_work = qualified - setup_work;
    assert_eq!(
        storage.node_document_ref(&mut probe, index).unwrap(),
        DocumentRef::Owned(0)
    );
    let node_document_work = probe
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - qualified;
    // This literal fixture has one namespace-less src attribute, no text
    // children, and script -> html -> document ordinary ancestry. Reads:
    // async: element + attribute search (1 + 1+3+1); child text: two visits;
    // src: search plus three UTF-16/ASCII comparisons (1+3+1 + 3*3);
    // connectivity: three records; type: element, two unsuccessful attribute
    // searches and default-classic classification (1 + 5+5 + 2).
    let source_read_work = 6 + 2 + 14 + 3 + 13;
    // Source steps 3, 4, 9, 14, 15 and 16 each write one actual metadata
    // slot. Owned document references add no captured-provider clone work.
    // Initialization plus eleven acknowledged replies end at step 15;
    // the next adapter operation is the independently measured document
    // lookup and step-16 metadata write. Deny its last RecordWrite credit.
    let metadata_write_work = 1 + mem::size_of::<ScriptMetadata>() as u64 + 1;
    let cursor_prefix_work = 12
        * (mem::size_of::<surgeist_html::bounded::script::HtmlScriptPreparationCursor>() as u64
            + 1);
    let before_preparation_document = setup_work
        + qualification_work
        + source_read_work
        + node_document_work
        + 6 * metadata_write_work
        + cursor_prefix_work
        - 1;
    assert!(before_preparation_document < exact - 1);
    session.release(&mut probe).unwrap();
    storage_zero(&probe);

    for cap in [before_preparation_document, exact - 1, exact] {
        let limits = limits().into_builder().work_credits(cap).finish().unwrap();
        let request =
            crate::HtmlDocumentParseRequest::offline(crate::HtmlSource::utf8(source), limits);
        let (mut c, mut session, capability, node) = fresh_script(&request, true, false);
        let result = session.prepare_disabled_script(&mut c, &node, &capability);
        let retained = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .find_map(|entry| {
                if let RetainedPayload::Script { metadata, .. } = entry {
                    Some(metadata.clone())
                } else {
                    None
                }
            })
            .unwrap();
        assert!(retained.flags.already_started() && !retained.flags.force_async());
        assert_eq!(retained.script_type, Some(HtmlScriptKind::Classic));
        assert_eq!(retained.parser_document, Some(DocumentRef::Owned(0)));
        if cap == exact {
            assert_eq!(
                result,
                Ok(DisabledScriptPreparation::Disabled(HtmlScriptKind::Classic))
            );
            assert_eq!(retained.preparation_document, Some(DocumentRef::Owned(0)));
            assert_eq!(c.stop(), None);
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }))
            ));
            assert_eq!(
                retained.preparation_document,
                if cap == before_preparation_document {
                    None
                } else {
                    Some(DocumentRef::Owned(0))
                }
            );
            // One short of the complete call now admits step 16 but refuses
            // the final cursor reply. It remains stopped and unfreezable.
            let first = c.stop().unwrap();
            assert!(
                matches!(session.freeze(&mut c), Err(SessionError::Stop(stop)) if stop == first)
            );
        }
        // Exact useful work can leave no credit for active disposal. The
        // same retained owner then performs terminal cleanup, without work
        // refund or a usable partially prepared immutable tree.
        let first_stop = c.stop();
        if session.release(&mut c).is_err() {
            session.release(&mut c).unwrap();
        }
        storage_zero(&c);
        if let Some(first_stop) = first_stop {
            assert_eq!(c.stop(), Some(first_stop));
        }
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            3
        );
    }
}

#[test]
fn the_actual_script_metadata_entry_requires_its_own_slot_and_failed_admission_is_reclaimable() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    // Before script creation Document/html + completion/cause/nonce/index
    // owners retain16 slots, including html's independent creation row.
    // Script metadata is reserved first:17 stops at document/native preflight.
    // Script adds document/creation metadata, name/URI/attrs, native/index,
    // cause and two nonce slots: completed27. At26 the tag is transferred
    // but the native ticket cannot be admitted; every initialized row is retained.
    for cap in [17, 26, 27] {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<html><head><script>"),
            limits()
                .into_builder()
                .auxiliary_entries(cap)
                .active_formatting_entries(cap)
                .finish()
                .unwrap(),
        );
        let (mut c, mut session, capability) = fresh_script_owner(&request);
        let mut tokenizer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut incoming = loop {
            let kind = lexical_kind(&mut tokenizer, &mut c);
            if matches!(&kind, TokenKind::Tag(tag) if tag.name.value.text().code_units() == [0x73,0x63,0x72,0x69,0x70,0x74])
            {
                let TokenKind::Tag(tag) = kind else {
                    unreachable!()
                };
                break Some(tag);
            }
            release_token_kind(&mut Some(kind), &mut c).unwrap();
        };
        let result = session.allocate_disabled_parser_script(
            &mut c,
            &mut incoming,
            None,
            crate::construction::CreationContext::Ordinary,
            &capability,
        );
        assert_eq!(incoming.is_some(), cap == 17);
        assert_eq!(
            session.active().unwrap().construction.test_storage().len(),
            if cap == 27 { 3 } else { 2 }
        );
        let count = session
            .active()
            .unwrap()
            .payloads
            .values
            .iter()
            .filter(|entry| matches!(entry, RetainedPayload::Script { .. }))
            .count();
        if cap == 27 {
            result.unwrap();
            assert_eq!(count, 1);
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                27
            );
        } else {
            assert!(matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    ..
                }))
            ));
            assert_eq!(count, 1);
            let stop = c.stop().unwrap();
            assert!(
                matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
            );
        }
        let first = c.stop();
        if let Some(tag) = incoming {
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        }
        tokenizer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        storage_zero(&c);
        assert_eq!(c.stop(), first);
        assert_eq!(
            c.usage().resource(HtmlParseResource::ManagedNodes).current,
            if cap == 27 { 3 } else { 2 }
        );
    }
}

#[test]
fn script_allocation_alias_rejection_preserves_the_actual_lexical_owner_and_arena() {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><script src=''>"),
        limits(),
    );
    let (mut c, mut session, capability, _) = fresh_script(&request, true, false);
    let mut tokenizer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let tag = loop {
        let kind = lexical_kind(&mut tokenizer, &mut c);
        if let TokenKind::Tag(tag) = kind {
            if tag.name.value.text().code_units() == [0x73, 0x63, 0x72, 0x69, 0x70, 0x74] {
                break tag;
            }
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        } else {
            panic!("only tags before script")
        }
    };
    let alias = tag.name.value.text().clone();
    let before = session.active().unwrap().construction.test_storage().len();
    let mut input = Some(tag);
    assert!(matches!(
        session.allocate_disabled_parser_script(
            &mut c,
            &mut input,
            None,
            crate::construction::CreationContext::Ordinary,
            &capability
        ),
        Err(SessionError::AliasedPayload)
    ));
    assert!(input.is_some());
    assert_eq!(
        session.active().unwrap().construction.test_storage().len(),
        before
    );
    assert_eq!(c.stop(), None);
    drop(alias);
    release_token_kind(&mut input.map(TokenKind::Tag), &mut c).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

fn lexical_script_type(attributes: &str) -> Option<HtmlScriptKind> {
    lexical_script_type_source(crate::HtmlSource::utf8(format!("<script {attributes}>")))
}
fn lexical_script_type_source(source: crate::HtmlSource) -> Option<HtmlScriptKind> {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let mut c = control();
    c.admit_source(&source).unwrap();
    let mut tokenizer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
    let TokenKind::Tag(tag) = lexical_kind(&mut tokenizer, &mut c) else {
        panic!("script tag")
    };
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    let node = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
    let ConstructionNodeKind::Element(element) = session
        .active()
        .unwrap()
        .construction
        .node_at(node.index)
        .expect("retained construction position")
        .kind()
    else {
        panic!("script")
    };
    let kind = classify_script(&mut c, element).unwrap();
    tokenizer.release(&mut c).unwrap();
    session.release(&mut c).unwrap();
    storage_zero(&c);
    kind
}

#[test]
fn script_classification_cursor_admission_refuses_before_the_next_read() {
    fn classify(
        c: &mut ParseControl,
        value: &DocumentString,
    ) -> Result<Option<surgeist_html::elements::script::HtmlScriptKind>, SessionError> {
        let mut cursor =
            surgeist_html::bounded::script::HtmlScriptTypeCursor::preparation(Some(value), None);
        while cursor.result().is_none() {
            cursor.advance(c)?;
        }
        Ok(cursor.result().expect("all admitted actions completed"))
    }
    let value = DocumentString::from("text/javascript");
    let mut c = control();
    assert_eq!(
        classify(&mut c, &value).unwrap(),
        Some(surgeist_html::elements::script::HtmlScriptKind::Classic)
    );
    assert!(c.stop().is_none());
    for available in [0, 18] {
        let mut c = control();
        c.charge(
            HtmlParseWorkCategory::StateStep,
            limits().work_credits() - available,
        )
        .unwrap();
        assert!(classify(&mut c, &value).is_err());
        let stopped = c.usage().clone();
        let first = c.stop();
        assert!(classify(&mut c, &value).is_err());
        assert_eq!(c.stop(), first);
        assert_eq!(c.usage(), stopped);
    }
    let cancellation = HtmlParseCancellation::new();
    cancellation.cancel();
    let mut c = ParseControl::new(limits(), cancellation);
    assert!(classify(&mut c, &value).is_err());
    assert_eq!(c.usage().work(HtmlParseWorkCategory::InspectByte), 0);
}

#[test]
fn script_type_classification_preserves_preexisting_charge_totals() {
    for (explicit, language, expected, visits_expected, inspections_expected) in [
        (None, None, Some(HtmlScriptKind::Classic), 4, 0),
        (
            Some(""),
            Some("unknown"),
            Some(HtmlScriptKind::Classic),
            16,
            36,
        ),
        (
            Some("text/javascript"),
            None,
            Some(HtmlScriptKind::Classic),
            24,
            75,
        ),
        (
            Some(" \tTEXT/JAVASCRIPT\n"),
            None,
            Some(HtmlScriptKind::Classic),
            56,
            103,
        ),
        (Some("MoDuLe"), None, Some(HtmlScriptKind::Module), 78, 34),
        (Some(" module "), None, None, 82, 20),
        (
            None,
            Some("javascript"),
            Some(HtmlScriptKind::Classic),
            24,
            77,
        ),
        (None, Some(" javascript "), None, 50, 50),
        (Some("unknown"), Some("javascript"), None, 86, 40),
        (Some("application/json"), None, None, 82, 19),
    ] {
        let mut builder = surgeist_html::construction::HtmlFragment::builder(
            surgeist_html::construction::HtmlConstructionLimits::new(8, 8, 8, 4096, 8),
        )
        .unwrap();
        let root = builder.root();
        builder
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "script"),
                OrderedAttributes::new(
                    explicit
                        .map(|v| {
                            Attribute::new(
                                surgeist_html::vocabulary::KnownHtmlAttribute::Type
                                    .qualified_name(),
                                v,
                            )
                        })
                        .into_iter()
                        .chain(language.map(|v| {
                            Attribute::new(
                                QualifiedName::new(Namespace::none(), None, "language"),
                                v,
                            )
                        })),
                )
                .unwrap(),
            )
            .unwrap();
        let fragment = builder.finish().unwrap();
        let child = fragment.root().children().next().unwrap();
        let surgeist_html::construction::HtmlNodeKind::Element(element) =
            fragment.node(&child).unwrap().kind()
        else {
            panic!("script")
        };
        let mut c = control();
        let before = c.usage().clone();
        assert_eq!(classify_script(&mut c, element).unwrap(), expected);
        let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit)
            - before.work(HtmlParseWorkCategory::RecordVisit);
        let inspections = c.usage().work(HtmlParseWorkCategory::InspectByte)
            - before.work(HtmlParseWorkCategory::InspectByte);
        assert_eq!(
            (visits, inspections),
            (visits_expected, inspections_expected),
            "{explicit:?}/{language:?}"
        );
        for available in [0, visits_expected + inspections_expected - 1] {
            let mut short = control();
            short
                .charge(
                    HtmlParseWorkCategory::StateStep,
                    limits().work_credits() - available,
                )
                .unwrap();
            assert!(classify_script(&mut short, element).is_err());
            assert!(short.stop().is_some());
        }
    }
}

#[test]
fn isolated_surrogates_and_supplementary_units_are_not_mime_normalized() {
    for extra in [vec![0xd800], vec![0xdc00], vec![0xd83d, 0xde00]] {
        let units = "<script type='text/javascript"
            .encode_utf16()
            .chain(extra)
            .chain("'>".encode_utf16())
            .collect::<Vec<_>>();
        assert_eq!(
            lexical_script_type_source(crate::HtmlSource::utf16(DocumentString::from_utf16(units))),
            None
        );
    }
}

#[test]
fn every_javascript_essence_and_ascii_case_variant_is_classic_without_parameters() {
    // Independently enumerated selected MIME §4.6 expectations.
    for essence in [
        "application/ecmascript",
        "application/javascript",
        "application/x-ecmascript",
        "application/x-javascript",
        "text/ecmascript",
        "text/javascript",
        "text/javascript1.0",
        "text/javascript1.1",
        "text/javascript1.2",
        "text/javascript1.3",
        "text/javascript1.4",
        "text/javascript1.5",
        "text/jscript",
        "text/livescript",
        "text/x-ecmascript",
        "text/x-javascript",
    ] {
        for value in [essence.to_owned(), essence.to_ascii_uppercase()] {
            assert_eq!(
                lexical_script_type(&format!("type='{value}'")),
                Some(HtmlScriptKind::Classic)
            );
        }
    }
    for value in [
        "text/javascript;charset=utf-8",
        "text/javascript1.6",
        "application/json",
        "text/javaſcript",
        "text/javascript&#xFFFD;",
    ] {
        assert_eq!(
            lexical_script_type(&format!("type='{value}'")),
            None,
            "{value}"
        );
    }
}

#[test]
fn explicit_type_and_legacy_language_follow_distinct_whitespace_and_default_branches() {
    for attributes in [
        "",
        "type=''",
        "language=''",
        "type='' language=unknown",
        "language=javascript",
        "type=' \tTEXT/JAVASCRIPT\n\r\x0c '",
    ] {
        assert_eq!(
            lexical_script_type(attributes),
            Some(HtmlScriptKind::Classic),
            "{attributes}"
        );
    }
    for (value, kind) in [
        ("MoDuLe", HtmlScriptKind::Module),
        ("IMPORTMAP", HtmlScriptKind::ImportMap),
        ("speculationrules", HtmlScriptKind::SpeculationRules),
    ] {
        assert_eq!(lexical_script_type(&format!("type={value}")), Some(kind));
        assert_eq!(lexical_script_type(&format!("type=' {value} '")), None);
    }
    for attributes in [
        "language=' javascript '",
        "language=module",
        "type=unknown language=javascript",
        "type='   '",
        "type='&#xA0;text/javascript&#xA0;'",
        "type=text/javascript1.0x",
    ] {
        assert_eq!(lexical_script_type(attributes), None, "{attributes}");
    }
}

#[test]
fn actual_import_permanently_revokes_standalone_inventory_but_foreign_control_does_not() {
    let mut c = control();
    let mut session =
        ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks)).unwrap();
    session.validate_standalone_document(&mut c).unwrap();
    let mut foreign = control();
    assert!(matches!(
        session.allocate_element(&mut foreign, html("div")),
        Err(SessionError::Stop(_))
    ));
    assert!(session.active().unwrap().standalone_ineligibility.is_none());
    session.validate_standalone_document(&mut c).unwrap();
    session.allocate_element(&mut c, html("div")).unwrap();
    let stop = HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance);
    assert_eq!(
        session.validate_standalone_document(&mut c),
        Err(SessionError::Stop(stop))
    );
    assert!(
        matches!(session.allocate_implicit_html(&mut c), Err(SessionError::Stop(reason)) if reason == stop)
    );
    assert_eq!(
        session.validate_standalone_document(&mut c),
        Err(SessionError::Stop(stop))
    );
    assert!(matches!(session.freeze(&mut c), Err(SessionError::Stop(reason)) if reason == stop));
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert_eq!(c.stop(), Some(stop));
}

#[test]
fn generic_fragment_and_preexisting_input_cannot_establish_ordinary_document_provenance() {
    let mut c = control();
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    assert_eq!(
        session.validate_standalone_document(&mut c),
        Err(SessionError::Stop(
            HtmlParseStop::UnsupportedStandaloneState(HtmlParseUnsupportedState::Provenance)
        ))
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn actual_template_contents_adoption_qualifies_inert_document_and_preserves_disconnected_script_prefix()
 {
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8("<html><head><template><script src=''>"),
        limits(),
    );
    let (mut c, mut session, capability, node) = fresh_script(&request, false, false);
    let initial = session.script_metadata(&mut c, &node).unwrap();
    let mut tokenizer =
        LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
    let tag = loop {
        let kind = lexical_kind(&mut tokenizer, &mut c);
        if let TokenKind::Tag(tag) = kind {
            if tag.name.value.text().code_units()
                == [0x74, 0x65, 0x6d, 0x70, 0x6c, 0x61, 0x74, 0x65]
            {
                break tag;
            }
            release_token_kind(&mut Some(TokenKind::Tag(tag)), &mut c).unwrap();
        } else {
            panic!("template tag")
        }
    };
    let template = session.allocate_html_tag(&mut c, &mut Some(tag)).unwrap();
    let contents = session
        .template_contents(&mut c, &template)
        .unwrap()
        .unwrap();
    session
        .insert(&mut c, &contents, &node, None, None)
        .unwrap();
    tokenizer.release(&mut c).unwrap();
    let parser = session
        .script_parser_document(&mut c, &node)
        .unwrap()
        .unwrap();
    let document = session
        .node_document(&mut c, &node)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    let contents_document = session
        .node_document(&mut c, &contents)
        .unwrap()
        .node(&mut c, &session)
        .unwrap();
    assert!(!session.same_node(&mut c, &parser, &document).unwrap());
    assert!(
        session
            .same_node(&mut c, &document, &contents_document)
            .unwrap()
    );
    assert_eq!(
        session.prepare_disabled_script(&mut c, &node, &capability),
        Ok(DisabledScriptPreparation::Disconnected)
    );
    let retained = session
        .active()
        .unwrap()
        .payloads
        .values
        .iter()
        .find_map(|entry| {
            if let RetainedPayload::Script { metadata, .. } = entry {
                Some(metadata.clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut expected = initial;
    expected.parser_document = None;
    expected.flags.parser_document_cleared_without_async();
    assert_eq!(retained, expected);
    assert!(retained.preparation_document.is_none());
    session.release(&mut c).unwrap();
    storage_zero(&c);
    assert!(c.stop().is_none());
}
