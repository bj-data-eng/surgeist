//! Original parse/install/consume boundaries, independent of native Meta hooks.
use super::super::meta::{cancel, envelope, is_disabled, native_session, observe};
use super::super::*;
use std::sync::Arc;
use surgeist_dom::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits, HtmlParseReport,
    HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme, HtmlCaptureLimits, HtmlFragmentContext,
};

fn parsing() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}
fn report(text: &str) -> Arc<HtmlParseReport> {
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), parsing())
            .parse()
            .unwrap(),
    )
}
fn installation() -> DomParsedHtmlInstallLimits {
    DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000)
}
fn environment(dom: &Dom) -> DomParsedDocumentEnvironment {
    let snapshot = dom.snapshot();
    let origin = snapshot
        .nodes()
        .find_map(|node| {
            snapshot
                .node(node)
                .unwrap()
                .as_document()
                .map(|document| document.metadata().origin().clone())
        })
        .unwrap();
    DomParsedDocumentEnvironment::headless(
        origin,
        HtmlBaseUrlRecord::observed(
            HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap(),
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    )
}
fn raw(text: &str) -> (Dom, DomParsedHtmlInstallation) {
    raw_report(report(text))
}
fn raw_report(value: Arc<HtmlParseReport>) -> (Dom, DomParsedHtmlInstallation) {
    let mut dom = dispatch_dom();
    dom.set_limits(dom.limits().with_meta_processing_work(1_000_000))
        .unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &cancel()).unwrap();
    let environment = environment(&dom);
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            value,
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    (dom, installed)
}

#[test]
fn original_lone_surrogate_name_crosses_the_consumer_without_lossy_decoding() {
    let mut input = "<!doctype html><meta http-equiv=default-style content='"
        .encode_utf16()
        .collect::<Vec<_>>();
    input.extend([0xd800, 0x0061]);
    input.extend("'>".encode_utf16());
    let value = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(input)),
            parsing(),
        )
        .parse()
        .unwrap(),
    );
    let (dom, installed) = raw_report(value);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    bind(&mut session, document, "utf16", Default::default());
    let done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert_eq!(
        done.preferences()[0]
            .observation()
            .preferred_name()
            .code_units(),
        &[0xd800, 0x0061]
    );
    assert_eq!(
        observe(&mut session, document.node())
            .preferred_name()
            .code_units(),
        &[0xd800, 0x0061]
    );
}
fn named_style(session: &mut DomSession, document: NodeId, title: DocumentString) -> NodeId {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", title);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let style = made.created(&style).unwrap().unwrap();
    let snapshot = session.snapshot();
    let parent = snapshot
        .node(document)
        .unwrap()
        .children()
        .iter()
        .copied()
        .find(|node| snapshot.node(*node).unwrap().as_element().is_some())
        .unwrap();
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(style, parent, None),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    style
}
fn bind(
    session: &mut DomSession,
    document: DocumentId,
    label: &str,
    limits: surgeist_cssom::CssomLimits,
) {
    session
        .style_execution()
        .bind_root(
            &session.dom(),
            label.into(),
            &mut Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(limits, Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
}
fn fragment(
    session: &DomSession,
    document: NodeId,
    text: &str,
) -> (Arc<HtmlParseReport>, DomHtmlCapture) {
    let snapshot = session.snapshot();
    let document = snapshot.node(document).unwrap().as_document().unwrap().id();
    let context_node = snapshot.node(document.node()).unwrap().children()[0];
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(context_node).unwrap())
            .unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(HtmlSource::utf8(text), context, parsing())
            .unwrap()
            .parse()
            .unwrap(),
    );
    (report, capture)
}

#[test]
fn admitted_pending_session_consumes_original_names_once_and_later_native_style_uses_them() {
    let (dom, installed) = raw(
        "<!doctype html><meta http-equiv=default-style content='A&amp;B'><meta http-equiv=default-style content='Ω🦀'><meta http-equiv=default-style content='Ω🦀'>",
    );
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let stamp = dom.snapshot().stamp();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .is_none(),
        "admission does not execute producer work"
    );
    let DomShutdownOutcome::Pending {
        session: retained,
        reason: DomShutdownReason::ParsedAssociations(actual),
    } = session.try_shutdown()
    else {
        panic!("same pending Session");
    };
    assert_eq!(actual, id);
    session = retained;
    bind(&mut session, document, "parsed", Default::default());
    let done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert_eq!(done.completion().contribution(), id);
    assert_eq!(
        done.completion().stamp(),
        stamp,
        "sidecar ack fabricates no tree revision"
    );
    assert_eq!(done.preferences().len(), 3);
    assert_eq!(
        done.preferences()[0].observation().preferred_name(),
        &DocumentString::from("A&B")
    );
    assert_eq!(
        done.preferences()[1].observation().preferred_name(),
        &DocumentString::from("Ω🦀")
    );
    assert_eq!(done.preferences()[1].observation().revision(), 2);
    assert_eq!(done.preferences()[2].observation().revision(), 2);
    assert_ne!(done.preferences()[1].id(), done.preferences()[2].id());
    assert!(matches!(
        done.completion().roots()[0].cssom().publication(),
        surgeist_cssom::CssomPublication::Unchanged { .. }
    ));
    assert!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap()
            .is_none()
    );
    assert!(
        matches!(
            session.style_execution().consume_parsed_meta(
                &mut session.dom(),
                id,
                envelope(),
                &cancel()
            ),
            Err(DomStyleParsedMetaError::Meta(DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::NotPending { contribution }))) if contribution == id
        ),
        "completed evidence cannot be consumed twice"
    );
    let a = named_style(&mut session, document.node(), "A&B".into());
    let chosen = named_style(&mut session, document.node(), "Ω🦀".into());
    assert!(is_disabled(&mut session, a));
    assert!(!is_disabled(&mut session, chosen));
    assert_eq!(
        installed.pending_association(),
        Some(id),
        "historical receipt remains original Pending evidence"
    );
}

#[test]
fn contextual_meta_is_no_effect_and_preserves_the_actual_selected_owner() {
    let (mut session, document) = native_session();
    let a = named_style(&mut session, document, "A".into());
    let b = named_style(&mut session, document, "B".into());
    let before = observe(&mut session, document);
    session
        .style_execution()
        .select_sheet_set(&session.dom(), &before, &"A".into(), 1, &cancel())
        .unwrap();
    let before = observe(&mut session, document);
    let producer = session
        .style_execution()
        .snapshot(document)
        .unwrap()
        .unwrap();
    let (value, capture) = fragment(
        &session,
        document,
        "<meta http-equiv=default-style content=B>",
    );
    let installed = session
        .install_parsed_html_fragment(value, &capture, installation(), &cancel())
        .unwrap();
    assert!(installed.pending_association().is_none());
    assert!(session.pending_parsed_html_associations().next().is_none());
    let after = observe(&mut session, document);
    assert_eq!(after.preferred_name(), before.preferred_name());
    assert_eq!(after.last_selected_name(), Some(&DocumentString::from("A")));
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.cssom_revision(), before.cssom_revision());
    assert_eq!(
        session
            .style_execution()
            .snapshot(document)
            .unwrap()
            .unwrap()
            .revision(),
        producer.revision()
    );
    assert!(!is_disabled(&mut session, a));
    assert!(is_disabled(&mut session, b));
}

#[test]
fn influencing_style_history_refuses_admission_without_losing_original_dom() {
    let (dom, installed) = raw(
        "<!doctype html><style title=A>p {color:red}</style><meta http-equiv=default-style content=B>",
    );
    let id = installed.pending_association().unwrap();
    let stamp = dom.snapshot().stamp();
    let rejected =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomSessionCreateFailure::Parsed(DomStyleParsedAdmissionError::Meta(
            DomParsedHtmlMetaAdmissionError::Unsupported { contribution, kind: surgeist_html_parser::HtmlParseAssociationOccurrenceKind::StyleUpdate(_), .. }
        )) if *contribution == id
    ));
    let dom = rejected.into_dom();
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        id
    );
    assert!(!dom.has_element_task_source());
    assert!(!dom.has_source_event_executor());
}

#[test]
fn discovery_refusal_keeps_original_structure_and_success_carries_its_exact_runtime_work() {
    let (dom, installed) = raw("<!doctype html><meta http-equiv=default-style content=B>");
    let id = installed.pending_association().unwrap();
    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    let mut constrained = session_limits(limits());
    constrained.style.work = 0;
    let rejected = DomSession::admit_parsed(dom, constrained, envelope(), &cancel()).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomSessionCreateFailure::Parsed(DomStyleParsedAdmissionError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    let dom = rejected.into_dom();
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        id
    );
    assert!(!dom.has_element_task_source());
    assert!(!dom.has_source_event_executor());
    constrained.style.work = 4;
    let one_short = DomSession::admit_parsed(dom, constrained, envelope(), &cancel()).unwrap_err();
    assert!(matches!(
        one_short.error(),
        DomSessionCreateFailure::Parsed(DomStyleParsedAdmissionError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    let dom = one_short.into_dom();
    assert_eq!(dom.snapshot().stamp(), before);
    constrained.style.work = 5;
    let mut session = DomSession::admit_parsed(dom, constrained, envelope(), &cancel()).unwrap();
    // Iterator formation, one ID and the exhausted boundary, capacity formation
    // and Hold entry are five prepaid Runtime actions. Original report work is
    // charged to the contribution, not replayed into the Runtime prefix.
    assert_eq!(session.state.borrow().work.get(), 5);
    drop(installed);
    let document = {
        let pending = session.pending_parsed_html_associations().next().unwrap();
        assert_eq!(pending.id(), id);
        pending.documents()[0].document()
    };
    let mut input = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    assert!(matches!(
        session
            .style_execution()
            .bind_root(&session.dom(), "prefix".into(), &mut input, &cancel()),
        Err(DomStyleRuntimeError::Admission)
    ));
    assert!(
        input.is_some(),
        "the real next operation cannot spend a reset counter"
    );
    session
        .style_execution()
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    session
        .style_execution()
        .bind_root(&session.dom(), "prefix".into(), &mut input, &cancel())
        .unwrap();
    assert!(input.is_none());
}

#[test]
fn refusal_after_an_earlier_complete_contribution_preserves_its_actual_original_work() {
    let (mut dom, first) = raw(
        "<!doctype html><meta http-equiv=default-style content=A><meta http-equiv=default-style content=B>",
    );
    let first_id = first.pending_association().unwrap();
    let value = report("<!doctype html><style title=C>p {color:blue}</style>");
    let environment = environment(&dom);
    let second = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            value,
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    let second_id = second.pending_association().unwrap();
    let before_work = {
        let hold = dom
            .begin_parsed_html_meta(first_id, envelope(), &cancel())
            .unwrap();
        hold.spent_work()
    };
    let before = dom.snapshot().stamp();
    let rejected =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomSessionCreateFailure::Parsed(DomStyleParsedAdmissionError::Meta(_))
    ));
    let mut dom = rejected.into_dom();
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.pending_parsed_html_associations().count(), 2);
    let after_work = {
        let hold = dom
            .begin_parsed_html_meta(first_id, envelope(), &cancel())
            .unwrap();
        assert_eq!(hold.limits(), envelope());
        hold.spent_work()
    };
    assert!(
        after_work > before_work,
        "earlier validation work stays on its original contribution after constructor refusal"
    );
    let ids = dom
        .pending_parsed_html_associations()
        .map(|entry| entry.id())
        .collect::<Vec<_>>();
    assert!(ids.contains(&first_id));
    assert!(ids.contains(&second_id));
    assert!(!dom.has_element_task_source());
    assert!(!dom.has_source_event_executor());
}

#[test]
fn removed_original_meta_still_executes_its_retained_occurrence_without_a_live_subject() {
    let (mut dom, installed) =
        raw("<!doctype html><meta http-equiv=default-style content=survives>");
    let mut cursor = installed.report().pending_associations();
    let subject = loop {
        if let surgeist_html_parser::HtmlParsePendingAssociationStep::Occurrence(row) =
            cursor.advance()
            && row.kind()
                == surgeist_html_parser::HtmlParseAssociationOccurrenceKind::ChangePreferred
        {
            break row.subject().id();
        }
    };
    let meta = installed.node_for(&subject).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.destroy(meta);
    dom.apply(batch, &cancel()).unwrap();
    assert!(matches!(
        dom.snapshot().node(meta),
        Err(DomError::StaleNode)
    ));
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    bind(&mut session, document, "removed", Default::default());
    let done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert_eq!(done.preferences()[0].subject(), &subject);
    assert_eq!(
        done.preferences()[0].observation().preferred_name(),
        &DocumentString::from("survives")
    );
}

#[test]
fn the_same_immutable_report_installed_twice_has_independent_destination_execution() {
    let (mut dom, first) = raw("<!doctype html><meta http-equiv=default-style content=chosen>");
    let value = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><meta http-equiv=default-style content=shared>"),
            parsing(),
        )
        .parse()
        .unwrap(),
    );
    let environment = environment(&dom);
    let a = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            value.clone(),
            environment.clone(),
            installation(),
            &cancel(),
        )
        .unwrap();
    let b = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            value.clone(),
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    for (index, installed) in [&first, &a, &b].into_iter().enumerate() {
        bind(
            &mut session,
            installed.document(),
            &format!("independent-{index}"),
            Default::default(),
        );
    }
    let a_id = a.pending_association().unwrap();
    let b_id = b.pending_association().unwrap();
    assert_ne!(a_id, b_id);
    let a_done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), a_id, envelope(), &cancel())
        .unwrap();
    assert_eq!(
        a_done.preferences()[0].observation().document(),
        a.document().node()
    );
    {
        let pending = session.dom();
        let entries = pending
            .raw()
            .pending_parsed_html_associations()
            .collect::<Vec<_>>();
        assert!(
            entries.iter().any(
                |entry| entry.id() == b_id && std::ptr::eq(entry.report(), Arc::as_ptr(&value))
            )
        );
    }
    let b_done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), b_id, envelope(), &cancel())
        .unwrap();
    assert_eq!(
        b_done.preferences()[0].observation().document(),
        b.document().node()
    );
    assert_eq!(
        a_done.preferences()[0].id(),
        b_done.preferences()[0].id(),
        "the original report identity is shared, not destination execution identity"
    );
    assert_eq!(
        b_done.preferences()[0].observation().preferred_name(),
        &DocumentString::from("shared")
    );
    assert!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(first.document())
            .unwrap()
            .is_some(),
        "unrelated contribution remains owned"
    );
}

#[test]
fn completing_original_meta_keeps_a_real_later_shadow_style_contribution_guarded() {
    let (dom, first) =
        raw("<!doctype html><meta http-equiv=default-style content=A><body><div></div>");
    let first_id = first.pending_association().unwrap();
    let document = first.document();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    bind(&mut session, document, "pending", Default::default());
    let snapshot = session.snapshot();
    let host = snapshot
        .nodes()
        .find(|node| {
            let node = snapshot.node(*node).unwrap();
            node.owner_document() == Some(document)
                && node.as_element().is_some_and(|element| {
                    element.name().local_name() == &DocumentString::from("div")
                })
        })
        .unwrap();
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let value = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><style title=B>p {color:red}</style></template>",
            ),
            context,
            parsing(),
            surgeist_html::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let second = session
        .install_parsed_html_fragment(value, &capture, installation(), &cancel())
        .unwrap();
    let second_id = second
        .pending_association()
        .expect("actual connected shadow Style processing");
    assert_ne!(first_id, second_id);
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap(),
        Some(first_id)
    );
    let basis = observe(&mut session, document.node());
    for action in 0..3 {
        let mut styles = session.style_execution();
        let result = match action {
            0 => styles.enable_sheet_set(&session.dom(), &basis, &"B".into(), 1, &cancel()),
            1 => styles.select_sheet_set(&session.dom(), &basis, &"B".into(), 1, &cancel()),
            _ => styles.change_preferred_sheet_set_name(
                &session.dom(),
                &basis,
                &"B".into(),
                1,
                &cancel(),
            ),
        };
        assert!(
            matches!(result, Err(DomStyleRuntimeError::PendingParsedAssociations(actual)) if actual == first_id)
        );
    }
    let done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), first_id, envelope(), &cancel())
        .unwrap();
    assert_eq!(
        done.preferences()[0].observation().preferred_name(),
        &DocumentString::from("A")
    );
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap(),
        Some(second_id)
    );
    let basis = observe(&mut session, document.node());
    assert!(
        matches!(session.style_execution().select_sheet_set(&session.dom(), &basis, &"B".into(), 1, &cancel()), Err(DomStyleRuntimeError::PendingParsedAssociations(actual)) if actual == second_id)
    );
    assert!(
        matches!(session.style_execution().consume_parsed_meta(&mut session.dom(), second_id, envelope(), &cancel()), Err(DomStyleParsedMetaError::Meta(DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Unsupported {contribution, ..}))) if contribution == second_id)
    );
    assert_eq!(
        session
            .pending_parsed_html_associations()
            .map(|entry| entry.id())
            .collect::<Vec<_>>(),
        vec![second_id]
    );
}

#[test]
fn finite_output_refusal_and_cancellation_keep_pending_until_a_raised_retry() {
    let (dom, installed) = raw("<!doctype html><meta http-equiv=default-style content=B>");
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let large = envelope();
    let no_outputs = DomMetaAtomicLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        0,
    )
    .unwrap();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), no_outputs, &cancel()).unwrap();
    bind(&mut session, document, "finite", Default::default());
    let original = observe(&mut session, document.node());
    let producer = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    let stamp = session.snapshot().stamp();
    let error = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, no_outputs, &cancel())
        .unwrap_err();
    assert!(matches!(
        error,
        DomStyleParsedMetaError::Runtime(DomStyleRuntimeError::Dom(DomError::ResourceLimit {
            resource: DomResource::MetaAtomicOutputs,
            limit: 0
        }))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        observe(&mut session, document.node()).preferred_name(),
        original.preferred_name()
    );
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap(),
        Some(id)
    );
    // One preferred-name output fits the Runtime bank. The remaining genuine
    // root/completion outputs must still be admitted before canonical ack.
    let one_output = DomMetaAtomicLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        1,
    )
    .unwrap();
    let error = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, one_output, &cancel())
        .unwrap_err();
    assert!(matches!(
        error,
        DomStyleParsedMetaError::Meta(DomParsedHtmlMetaError::Admission(
            DomParsedHtmlMetaAdmissionError::Dom(DomError::ResourceLimit {
                resource: DomResource::MetaAtomicOutputs,
                limit: 1
            })
        ))
    ));
    let after = observe(&mut session, document.node());
    assert_eq!(after.preferred_name(), original.preferred_name());
    assert_eq!(after.last_selected_name(), original.last_selected_name());
    assert_eq!(after.revision(), original.revision());
    assert_eq!(after.cssom_revision(), original.cssom_revision());
    let current = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(current.revision(), producer.revision());
    assert_eq!(current.sheets(), producer.sheets());
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap(),
        Some(id)
    );
    let cancel_now = AtomicBool::new(true);
    assert!(matches!(
        session
            .style_execution()
            .consume_parsed_meta(&mut session.dom(), id, large, &cancel_now),
        Err(DomStyleParsedMetaError::Meta(
            DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    let completed = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, large, &cancel())
        .unwrap();
    assert_eq!(completed.preferences().len(), 1);
    assert_eq!(completed.preferences()[0].observation().revision(), 1);
    assert!(matches!(
        session
            .style_execution()
            .consume_parsed_meta(&mut session.dom(), id, large, &cancel()),
        Err(DomStyleParsedMetaError::Meta(DomParsedHtmlMetaError::Admission(
            DomParsedHtmlMetaAdmissionError::NotPending { contribution }
        ))) if contribution == id
    ));
}

#[test]
fn a_real_parked_source_tail_and_foreign_session_refuse_without_acknowledgement() {
    let (mut session, document) = bound_dispatch_session();
    let parent = super::super::meta::container(&session, document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = made.created(&dialog).unwrap().unwrap();
    let value = report("<!doctype html><meta http-equiv=default-style content=B>");
    let environment = environment(session.dom().raw());
    let installed = session
        .install_parsed_html_document(
            session.snapshot().stamp(),
            value,
            environment,
            installation(),
            &cancel(),
        )
        .unwrap();
    let id = installed.pending_association().unwrap();
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(dialog),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        ..
    } = progress
    else {
        panic!("real original BeforeToggle tail");
    };
    assert!(
        matches!(session.style_execution().consume_parsed_meta(&mut session.dom(),id,envelope(),&cancel()),Err(DomStyleParsedMetaError::Runtime(DomStyleRuntimeError::SourcePending(actual))) if actual==request.operation())
    );
    let (mut other, _) = native_session();
    assert!(matches!(
        session
            .style_execution()
            .consume_parsed_meta(&mut other.dom(), id, envelope(), &cancel()),
        Err(DomStyleParsedMetaError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        other
            .style_execution()
            .consume_parsed_meta(&mut other.dom(), id, envelope(), &cancel()),
        Err(DomStyleParsedMetaError::Meta(
            DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Dom(
                DomError::ForeignOwner
            ))
        ))
    ));
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(installed.document())
            .unwrap(),
        Some(id)
    );
}

#[test]
fn original_type_ineligible_style_has_no_association_effect_or_external_context_demand() {
    let (dom, installed) = raw(
        "<!doctype html><style type=text/plain title=A>ignored</style><meta http-equiv=default-style content=B>",
    );
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let mut session =
        DomSession::admit_parsed(dom, session_limits(limits()), envelope(), &cancel()).unwrap();
    bind(
        &mut session,
        document,
        "no-effect-style",
        Default::default(),
    );
    let done = session
        .style_execution()
        .consume_parsed_meta(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert_eq!(done.preferences().len(), 1);
    assert_eq!(
        done.preferences()[0].observation().preferred_name(),
        &DocumentString::from("B")
    );
    assert!(
        session
            .style_execution()
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    assert!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap()
            .is_none()
    );
}
