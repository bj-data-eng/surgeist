//! Terminal readiness follows actual End, notification delivery and cleanup.
use super::*;
use std::{convert::Infallible, mem::size_of, sync::Arc};
use surgeist_cssom::CssomSnapshot;
use surgeist_html::{
    HtmlCspArtifactLayout, HtmlCspDisposalWork, HtmlCspLayout, HtmlCspNativePolicy, HtmlCspOwner,
    HtmlCspPolicyArtifact, HtmlCspProcessorAuthority, HtmlCspStorage,
};
use surgeist_html_parser::HtmlParseAssociationOccurrenceId as Occurrence;

const SOURCE: &str = "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B><style title=C>c{color:blue}</style><style title=B>d{color:green}</style><style type=text/plain title=ignored>ignored</style>";

fn ids(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> (Vec<Occurrence>, Option<Occurrence>) {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap();
    let mut cursor = entry.report().pending_associations();
    let mut styles = Vec::new();
    let mut meta = None;
    loop {
        match cursor.advance() {
            Step::Fault(fault) => panic!("{fault}"),
            Step::Occurrence(row) if row.requires_processing() => match row.kind() {
                Kind::StyleUpdate(_) => styles.push(row.id()),
                Kind::ChangePreferred => {
                    assert!(meta.replace(row.id()).is_none());
                }
                other => panic!("unexpected original fixture work: {other:?}"),
            },
            Step::Complete => return (styles, meta),
            _ => (),
        }
    }
}
fn current(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleQualification {
    session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone()
}
fn snapshot(session: &DomSession, document: DocumentId) -> CssomSnapshot {
    session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
}
fn names(session: &mut DomSession, document: DocumentId) -> DomStyleSheetSetObservation {
    session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap()
}
fn complete(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    cap: DomParsedHtmlAssociationLimits,
) -> Result<DomParsedHtmlContributionCompletion, DomStyleParsedTerminalError> {
    session.style_execution().complete_parsed_html_contribution(
        &mut session.dom(),
        id,
        cap,
        &cancel(),
    )
}
fn end(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    cap: DomParsedHtmlAssociationLimits,
) -> DomParsedHtmlSynchronousEnd {
    session
        .style_execution()
        .complete_parsed_html_synchronous_execution(&mut session.dom(), id, cap, &cancel())
        .unwrap()
}
fn create_render(session: &mut DomSession, q: &DomParsedHtmlStyleQualification) -> DomStyleSheet {
    let output = create(session, q.contribution(), envelope()).unwrap();
    let sheet = output.completion().sheet().clone();
    drop(output);
    finish_rendering(session, q);
    sheet
}
fn pop_allow(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
    expected: &Occurrence,
    text: &str,
) -> (DomParsedHtmlStyleQualification, DomStyleSheet) {
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), q.contribution(), envelope(), &cancel())
        .unwrap();
    let next = output.qualification().clone();
    assert_eq!(next.occurrence(), expected);
    drop(output);
    let mut input = observe(session, q.contribution(), &mut |request| {
        assert_eq!(request.text(), &DocumentString::from(text));
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            q.contribution(),
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    let sheet = create_render(session, &next);
    (next, sheet)
}
fn next_allow(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    expected: &Occurrence,
) -> (DomParsedHtmlStyleQualification, DomStyleSheet) {
    let output = session
        .style_execution()
        .advance_parsed_style_subject(&mut session.dom(), id, expected, envelope(), &cancel())
        .unwrap();
    let q = output.qualification().clone();
    drop(output);
    let mut input = observe(session, id, &mut |request| {
        assert!(request.text().is_empty());
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    let sheet = create_render(session, &q);
    (q, sheet)
}
fn queued(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> DomStyleNetworkingTaskId {
    match session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    {
        DomStyleParsedNotificationProgress::Queued { task } => task,
        other => panic!("genuine NoImports notification: {other:?}"),
    }
}
fn removed(session: &mut DomSession, q: &DomParsedHtmlStyleQualification, sheet: &DomStyleSheet) {
    let serial = session.state.borrow().next_task;
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel()).unwrap(), DomStyleParsedNotificationProgress::OwnerAssociationRemoved { association } if association == sheet.association())
    );
    assert_eq!(session.state.borrow().next_task, serial);
}
fn events(session: &DomSession) -> (DomEventDispatcher<Context>, Context) {
    (
        DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 32,
            pending_events: 16,
            nested_dispatches: 8,
        })
        .unwrap(),
        Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        },
    )
}
fn listen(session: &mut DomSession, dispatcher: &mut DomEventDispatcher<Context>, target: NodeId) {
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.events.push((
                    event.target().unwrap(),
                    event.current_target().unwrap().node().unwrap(),
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
}
fn run(
    session: &mut DomSession,
    dispatcher: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) -> DomStyleNetworkingProgress {
    DomStyleRuntime::run_networking(
        &mut session.dom(),
        dispatcher,
        context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
}
fn assert_pending_readback(snapshot: &DomSnapshot, target: NodeId) {
    let style = snapshot.node(target).unwrap().as_style().unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert!(matches!(
        style.sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    assert!(matches!(
        style.is_render_blocking(),
        Err(DomError::StyleProcessingUnavailable)
    ));
}
fn assert_ready(snapshot: &DomSnapshot, target: NodeId, expected: Option<&DomStyleSheet>) {
    let style = snapshot.node(target).unwrap().as_style().unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::Established
    );
    assert_eq!(style.sheet().unwrap(), expected);
    assert!(!style.is_render_blocking().unwrap());
}
fn probe(session: &mut DomSession) -> NodeId {
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let style = batch.create_element_ns(&other, Namespace::html(), "style");
    session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap()
}
fn adopt_probe(
    session: &mut DomSession,
    probe: NodeId,
    document: DocumentId,
) -> Result<DomCommit, DomError> {
    let mut batch = DomBatch::new();
    batch.adopt(probe, document.node());
    session.dom().apply(batch, &cancel())
}
fn simple() -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    let (mut session, installation, first, old) =
        created_first("<!doctype html><style title=A>p{color:red}</style>");
    let (rows, _) = ids(&session, first.contribution());
    let (q, sheet) = pop_allow(&mut session, &first, &rows[1], "p{color:red}");
    removed(&mut session, &first, &old);
    (session, installation, q, sheet)
}
fn blocked_in(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    policy: HtmlCspAssociation,
    profile: &HtmlCspProcessorProfile,
    cap: DomParsedHtmlAssociationLimits,
) -> DomParsedHtmlStyleQualification {
    request_ready(session, id, policy, cap);
    let mut input = observe(session, id, &mut |request| {
        Some(text_policy::decision(request, profile))
    });
    assert!(!input.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(&mut session.dom(), id, &mut input, cap, &cancel())
        .unwrap();
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, cap, &cancel())
        .unwrap();
    assert!(matches!(
        output.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    let q = output.qualification().clone();
    drop(output);
    let mut input = observe(session, id, &mut |request| {
        Some(text_policy::decision(request, profile))
    });
    assert!(!input.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(&mut session.dom(), id, &mut input, cap, &cancel())
        .unwrap();
    q
}

#[test]
fn settled_a_meta_c_d_ack_publishes_exact_readiness_preserves_snapshots_and_enables_native_work() {
    let (mut session, installation, a0, old_a) = created_first(SOURCE);
    let id = a0.contribution();
    let document = installation.document();
    let probe = probe(&mut session);
    let probe_document = session
        .snapshot()
        .node(probe)
        .unwrap()
        .owner_document()
        .unwrap();
    let (rows, m) = ids(&session, id);
    assert_eq!(rows.len(), 6);
    let first_task = queued(&mut session, &a0);
    let (a1, a) = pop_allow(&mut session, &a0, &rows[1], "p{color:red}");
    let meta_output = session
        .style_execution()
        .consume_parsed_meta_occurrence(&mut session.dom(), id, &m.unwrap(), envelope(), &cancel())
        .unwrap();
    let (c0, old_c) = next_allow(&mut session, id, &rows[2]);
    let (c1, c) = pop_allow(&mut session, &c0, &rows[3], "c{color:blue}");
    let (d0, old_d) = next_allow(&mut session, id, &rows[4]);
    let (d1, d) = pop_allow(&mut session, &d0, &rows[5], "d{color:green}");
    removed(&mut session, &c0, &old_c);
    removed(&mut session, &d0, &old_d);
    let later = [
        queued(&mut session, &a1),
        queued(&mut session, &c1),
        queued(&mut session, &d1),
    ];
    let (mut dispatcher, mut context) = events(&session);
    for sheet in [&a, &c, &d] {
        listen(&mut session, &mut dispatcher, sheet.association().element());
    }
    assert!(
        matches!(run(&mut session, &mut dispatcher, &mut context), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == first_task)
    );
    assert!(complete(&mut session, id, envelope()).is_err());
    let end_output = end(&mut session, id, envelope());
    let mut receipts = Vec::new();
    for task in std::iter::once(first_task).chain(later) {
        let DomStyleNetworkingProgress::Complete {
            task: actual,
            receipt,
        } = run(&mut session, &mut dispatcher, &mut context)
        else {
            panic!("real FIFO after End")
        };
        assert_eq!(actual, task);
        assert_eq!(receipt.callbacks(), 1);
        receipts.push(receipt);
    }
    assert!(matches!(
        run(&mut session, &mut dispatcher, &mut context),
        DomStyleNetworkingProgress::Empty
    ));
    let before = session.snapshot();
    let old_packet = installation.snapshot().clone();
    let report = installation.report();
    let ignored = before
        .nodes()
        .find(|node| {
            before.node(*node).is_ok_and(|node| {
                node.as_element().is_some_and(|e| {
                    e.attribute(&DocumentString::from("title"))
                        == Some(&DocumentString::from("ignored"))
                })
            })
        })
        .unwrap();
    let ignored_state = before
        .node(ignored)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .clone();
    assert_ready(&before, ignored, None);
    for sheet in [&a, &c, &d] {
        assert_pending_readback(&before, sheet.association().element());
    }
    assert!(
        matches!(adopt_probe(&mut session, probe, document), Err(DomError::ParsedAssociationPending { document:actual,pending }) if actual == document && pending == id)
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let css_before = snapshot(&session, document);
    let names_before = names(&mut session, document);
    let output = complete(&mut session, id, envelope()).unwrap();
    assert_eq!(output.contribution(), id);
    let after = session.snapshot();
    assert_ne!(after.stamp(), before.stamp());
    assert_eq!(output.dom().changes().to(), after.stamp());
    let changed: Vec<_> = output
        .dom()
        .changes()
        .changes()
        .iter()
        .filter_map(|change| {
            if let DomChange::ElementState(node) = change {
                Some(*node)
            } else {
                None
            }
        })
        .collect();
    for sheet in [&a, &c, &d] {
        let target = sheet.association().element();
        assert!(changed.contains(&target));
        assert_ready(&after, target, Some(sheet));
        assert_pending_readback(&before, target);
        let before_node = before.node(target).unwrap();
        let after_node = after.node(target).unwrap();
        assert_eq!(after_node.parent(), before_node.parent());
        assert_eq!(after_node.children(), before_node.children());
        assert_eq!(
            after_node.owner_document().unwrap(),
            before_node.owner_document().unwrap()
        );
        assert_eq!(
            after_node.as_style().unwrap().state().creation_state(),
            before_node.as_style().unwrap().state().creation_state()
        );
        assert_eq!(
            after_node.as_style().unwrap().parser_document(),
            before_node.as_style().unwrap().parser_document()
        );
    }
    assert_eq!(changed.len(), 3);
    assert!(!changed.contains(&ignored));
    assert!(
        output
            .dom()
            .changes()
            .changes()
            .iter()
            .all(|change| matches!(change, DomChange::ElementState(_)))
    );
    assert_eq!(
        after.node(ignored).unwrap().as_style().unwrap().state(),
        &ignored_state
    );
    assert!(
        session
            .pending_parsed_html_associations()
            .all(|entry| entry.id() != id)
    );
    assert_eq!(
        session
            .dom()
            .raw()
            .pending_parsed_html_association(document)
            .unwrap(),
        None
    );
    let css_after = snapshot(&session, document);
    assert_eq!(css_after.revision(), css_before.revision());
    assert_eq!(css_after.sheets(), css_before.sheets());
    for sheet in [&a, &c, &d, &old_a] {
        assert_eq!(
            css_after.sheet_rules(sheet.id()).unwrap(),
            css_before.sheet_rules(sheet.id()).unwrap()
        );
        assert_eq!(
            css_after.sheet(sheet.id()).unwrap().inputs(),
            css_before.sheet(sheet.id()).unwrap().inputs()
        );
    }
    let names_after = names(&mut session, document);
    assert_eq!(names_after.preferred_name(), &DocumentString::from("B"));
    assert_eq!(names_after.preferred_name(), names_before.preferred_name());
    assert_eq!(
        names_after.last_selected_name(),
        names_before.last_selected_name()
    );
    assert_eq!(names_after.revision(), names_before.revision());
    assert_eq!(end_output.contribution(), id);
    assert_eq!(
        meta_output.preference().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(receipts[0].original_target(), a.association().element());
    assert_eq!(
        a0.original_document(),
        &DomParsedHtmlHistoricalDocument::Constructed(report.document().unwrap().root().id())
    );
    assert_pending_readback(&old_packet, a.association().element());
    drop(adopt_probe(&mut session, probe, document).unwrap());
    assert_ne!(probe_document, document);
    assert_eq!(
        session
            .snapshot()
            .node(probe)
            .unwrap()
            .owner_document()
            .unwrap(),
        document
    );
    assert_eq!(session.snapshot().node(probe).unwrap().parent(), None);
    assert_ready(&session.snapshot(), probe, None);
    let serial = session.state.borrow().next_task;
    let mut source = session.source_execution();
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_style_disabled(a.association().element(), false),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &cancel()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let actual = session
        .style_execution()
        .disabled(&session.dom(), a.association().element())
        .unwrap();
    assert!(!actual.disabled());
    assert_eq!(actual.association(), Some(a.association()));
    assert_eq!(session.state.borrow().next_task, serial);
    assert_eq!(
        names(&mut session, document).preferred_name(),
        &DocumentString::from("B")
    );
    assert_pending_readback(&before, a.association().element());
}

#[test]
fn all_blocked_requires_real_end_then_establishes_null_and_duplicate_cannot_reuse_it() {
    let (policy, profile) = text_policy::policy(false);
    let (mut session, installation) = installed(report("<!doctype html><style>blocked</style>"));
    let id = installation.pending_association().unwrap();
    let q = blocked_in(&mut session, id, policy, &profile, envelope());
    let target = installation.node_for(q.subject()).unwrap();
    assert!(
        matches!(complete(&mut session, id, envelope()), Err(DomStyleParsedTerminalError::Terminal(DomParsedHtmlTerminalError::SynchronousEndRequired { contribution })) if contribution == id)
    );
    let historical = session.snapshot();
    assert_pending_readback(&historical, target);
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    let end_output = end(&mut session, id, envelope());
    let output = complete(&mut session, id, envelope()).unwrap();
    assert!(
        output
            .dom()
            .changes()
            .changes()
            .contains(&DomChange::ElementState(target))
    );
    assert_ready(&session.snapshot(), target, None);
    assert_pending_readback(&historical, target);
    let mut batch = DomBatch::new();
    batch.set_style_disabled(target, true);
    drop(session.dom().apply(batch, &cancel()).unwrap());
    assert!(
        session
            .style_execution()
            .disabled(&session.dom(), target)
            .unwrap()
            .association()
            .is_none()
    );
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    let stamp = session.snapshot().stamp();
    assert!(complete(&mut session, id, envelope()).is_err());
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(end_output.contribution(), id);
}

#[derive(Debug)]
struct EmptyOnly {
    allow_empty: bool,
}
impl HtmlCspPolicyArtifact for EmptyOnly {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>(), 0, 1).unwrap(),
            HtmlCspDisposalWork::try_new(1, size_of::<Self>()).unwrap(),
        )
        .unwrap()
    }
}
fn admit(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
fn empty_only() -> (HtmlCspAssociation, HtmlCspProcessorProfile) {
    let mut owner = HtmlCspOwner::new_with(&mut admit).unwrap();
    let mut association = owner.initial_document_with(&mut admit).unwrap();
    let mut authority = HtmlCspProcessorAuthority::new_with(&mut admit).unwrap();
    let profile = authority.profile_with(1, &mut admit).unwrap();
    let request = owner
        .request_meta_insertion_with(
            &association,
            &profile,
            "fixture: allow only empty inline text".into(),
            &mut admit,
        )
        .unwrap();
    let prepared = authority
        .attest_changed_with(
            &request,
            Arc::new(EmptyOnly { allow_empty: true }),
            &mut admit,
        )
        .unwrap();
    drop(
        owner
            .prepare_publication_with(&mut association, request, prepared, &mut admit)
            .unwrap()
            .publish(),
    );
    (association, profile)
}
fn empty_decision(
    request: &DomParsedHtmlStyleInlinePolicyRequest,
    profile: &HtmlCspProcessorProfile,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    let HtmlCspNativePolicy::External(artifact) = request
        .policy()
        .policy()
        .borrow_native_with::<EmptyOnly, _>(profile, &mut admit)
        .unwrap()
    else {
        panic!("actual attested empty-only artifact")
    };
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(
        request,
        artifact.allow_empty && request.text().is_empty(),
    )
}

#[test]
fn created_empty_then_genuinely_blocked_pop_establishes_final_null_after_owner_removed_settlement()
{
    let (policy, profile) = empty_only();
    let (mut session, installation) =
        installed(report("<!doctype html><style>p{color:red}</style>"));
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, policy, envelope());
    let mut observation = observe(&session, id, &mut |request| {
        Some(empty_decision(request, &profile))
    });
    assert!(observation.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut observation,
            envelope(),
            &cancel(),
        )
        .unwrap();
    bind(&mut session, installation.document(), Default::default());
    let q0 = current(&session, id);
    let created = create(&mut session, id, envelope()).unwrap();
    let old = created.completion().sheet().clone();
    drop(created);
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), &q0, envelope(), &cancel())
        .unwrap();
    let DomParsedHtmlStyleRenderingProgress::Complete(rendering) = session
        .style_execution()
        .process_parsed_style_rendering(&mut session.dom(), &q0, envelope(), &cancel())
        .unwrap()
    else {
        panic!("actual original head/default-media rendering")
    };
    assert!(rendering.render_blocking_member());
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    let q1 = output.qualification().clone();
    drop(output);
    let mut observation = observe(&session, id, &mut |request| {
        assert_eq!(request.text(), &DocumentString::from("p{color:red}"));
        Some(empty_decision(request, &profile))
    });
    assert!(!observation.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut observation,
            envelope(),
            &cancel(),
        )
        .unwrap();
    let end_output = end(&mut session, id, envelope());
    assert!(
        matches!(complete(&mut session,id,envelope()),Err(DomStyleParsedTerminalError::TailPending { contribution }) if contribution==id)
    );
    removed(&mut session, &q0, &old);
    let before = session.snapshot();
    let css_before = snapshot(&session, installation.document());
    let result = complete(&mut session, id, envelope()).unwrap();
    let target = installation.node_for(q1.subject()).unwrap();
    assert_pending_readback(&before, target);
    let after = session.snapshot();
    let style = after.node(target).unwrap().as_style().unwrap();
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::Established
    );
    assert_eq!(style.sheet().unwrap(), None);
    // CSSOM removal and a CSP-blocked pop do not execute notification cleanup.
    // OwnerAssociationRemoved settles without that task: ACK preserves the
    // actual original rendering membership while making its readback available.
    assert_eq!(
        style.is_render_blocking().unwrap(),
        rendering.render_blocking_member()
    );
    assert!(style.is_render_blocking().unwrap());
    assert!(
        result
            .dom()
            .changes()
            .changes()
            .contains(&DomChange::ElementState(target))
    );
    let css_after = snapshot(&session, installation.document());
    assert!(css_after.sheets().is_empty());
    assert_eq!(css_after.revision(), css_before.revision());
    assert_eq!(css_after.sheet(old.id()).unwrap().inputs().owner, None);
    assert_eq!(end_output.contribution(), id);
}

#[test]
fn unresolved_real_imports_do_not_become_terminal_from_end_empty_queue_or_output_drop() {
    let (mut session, installation, q0, old) =
        created_first("<!doctype html><style>@import 'required.css';p{color:red}</style>");
    let id = q0.contribution();
    let (rows, _) = ids(&session, id);
    let (q1, sheet) = pop_allow(
        &mut session,
        &q0,
        &rows[1],
        "@import 'required.css';p{color:red}",
    );
    removed(&mut session, &q0, &old);
    let end_output = end(&mut session, id, envelope());
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(&mut session.dom(),&q1,envelope(),&cancel()).unwrap(),DomStyleParsedNotificationProgress::PendingImports { association } if association==sheet.association())
    );
    let (mut dispatcher, mut context) = events(&session);
    assert!(matches!(
        run(&mut session, &mut dispatcher, &mut context),
        DomStyleNetworkingProgress::Empty
    ));
    drop(installation);
    drop(end_output);
    let before = session.snapshot();
    assert!(
        matches!(complete(&mut session,id,envelope()),Err(DomStyleParsedTerminalError::TailPending { contribution }) if contribution==id)
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_pending_readback(&session.snapshot(), sheet.association().element());
    pending(&session, id);
    assert!(context.events.is_empty());
}

struct RetryContext {
    styles: DomStyleRuntime,
    calls: Vec<&'static str>,
}
impl DomStyleEventContext for RetryContext {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}

#[test]
fn inflight_then_delivered_cleanup_refusal_cannot_ack_or_replay_callbacks() {
    let (mut session, installation, q, sheet) = simple();
    let id = q.contribution();
    let task = queued(&mut session, &q);
    let end_output = end(&mut session, id, envelope());
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            sheet.association().element(),
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut RetryContext, _| {
                context.calls.push("a");
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            sheet.association().element(),
            DomEventType::Load,
            DomEventCallback::new(|dom, _, context: &mut RetryContext, _| {
                context.calls.push("b");
                let owner = context.styles.original().unwrap();
                let before = owner.borrow().work.get();
                context.styles.set_limits(dom, limits(), &cancel()).unwrap();
                let cost = owner.borrow().work.get() - before;
                let mut tight = limits();
                tight.work = owner.borrow().work.get().checked_add(cost).unwrap();
                context.styles.set_limits(dom, tight, &cancel()).unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
    let mut context = RetryContext {
        styles: session.style_execution(),
        calls: Vec::new(),
    };
    let mut one = dispatch_limits();
    one.callbacks = 1;
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(),&mut dispatcher,&mut context,one,&cancel()).unwrap(),DomStyleNetworkingProgress::Retained { task:actual,.. } if actual==task)
    );
    assert_eq!(context.calls, ["a"]);
    assert!(matches!(
        complete(&mut session, id, envelope()),
        Err(DomStyleParsedTerminalError::TailPending { .. })
    ));
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel()
        ),
        Err(DomStyleRuntimeError::Admission)
    ));
    assert_eq!(context.calls, ["a", "b"]);
    assert!(session.state.borrow().completions.iter().any(|row| {
        row.as_task()
            .is_some_and(|row| row.id() == task && matches!(row.delivery, Delivery::Delivered(_)))
    }));
    context
        .styles
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    assert!(matches!(
        complete(&mut session, id, envelope()),
        Err(DomStyleParsedTerminalError::TailPending { .. })
    ));
    let DomStyleNetworkingProgress::Complete {
        task: actual,
        receipt,
    } = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
    else {
        panic!("resume actual cleanup")
    };
    assert_eq!(actual, task);
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(context.calls, ["a", "b"]);
    let output = complete(&mut session, id, envelope()).unwrap();
    assert_ready(
        &session.snapshot(),
        sheet.association().element(),
        Some(&sheet),
    );
    assert_eq!(output.contribution(), end_output.contribution());
    assert_eq!(installation.pending_association(), Some(id));
}

#[test]
fn original_output_refusal_preserves_pending_and_availability_until_same_owner_raise() {
    let cap = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 8).unwrap();
    let (policy, profile) = text_policy::policy(false);
    let (mut session, installation) = installed(report("<!doctype html><style>blocked</style>"));
    let id = installation.pending_association().unwrap();
    let q = blocked_in(&mut session, id, policy, &profile, cap);
    let target = installation.node_for(q.subject()).unwrap();
    let end_output = end(&mut session, id, cap);
    let before = session.snapshot();
    let error = complete(&mut session, id, cap).expect_err("actual terminal output cap");
    let DomStyleParsedTerminalError::Terminal(error) = error else {
        panic!("original terminal error")
    };
    let DomParsedHtmlTerminalError::Frontier(error) = error else {
        panic!("original admission")
    };
    let DomParsedHtmlFrontierError::Style(error) = error else {
        panic!("original Style admission")
    };
    let DomParsedHtmlStyleError::Admission(error) = error else {
        panic!("original admission")
    };
    let DomParsedHtmlAssociationAdmissionError::Dom(error) = error else {
        panic!("original DOM quota")
    };
    assert!(matches!(
        error,
        DomError::ResourceLimit {
            resource: DomResource::ParsedAssociationProcessingOutputs,
            limit: 8
        }
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_pending_readback(&session.snapshot(), target);
    pending(&session, id);
    let output = complete(&mut session, id, envelope()).unwrap();
    assert_ready(&session.snapshot(), target, None);
    assert_ne!(session.snapshot().stamp(), before.stamp());
    assert_eq!(output.contribution(), end_output.contribution());
}

#[test]
fn exact_contribution_completion_preserves_other_lineage_across_cancel_permission_and_move() {
    let packet = report("<!doctype html><style>blocked</style>");
    let (mut session, first) = installed(packet.clone());
    let second = super::super::style_decision::install_into(&mut session, packet);
    let first_id = first.pending_association().unwrap();
    let second_id = second.pending_association().unwrap();
    assert_ne!(first_id, second_id);
    let (policy, profile) = text_policy::policy(false);
    let a = blocked_in(&mut session, first_id, policy.clone(), &profile, envelope());
    let b = blocked_in(&mut session, second_id, policy, &profile, envelope());
    assert_eq!(a.occurrence(), b.occurrence());
    let first_end = end(&mut session, first_id, envelope());
    let before = session.snapshot();
    let runtime = session.style_execution();
    assert!(
        runtime
            .complete_parsed_html_contribution(
                &mut session.dom(),
                first_id,
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let (foreign, _) = bound_dispatch_session();
    assert!(matches!(
        foreign.style_execution().complete_parsed_html_contribution(
            &mut session.dom(),
            first_id,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedTerminalError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    drop(complete(&mut session, first_id, envelope()).unwrap());
    assert_ready(
        &session.snapshot(),
        first.node_for(a.subject()).unwrap(),
        None,
    );
    assert_pending_readback(&session.snapshot(), second.node_for(b.subject()).unwrap());
    assert!(complete(&mut session, first_id, envelope()).is_err());
    let DomShutdownOutcome::Pending {
        session: recovered,
        reason,
    } = session.try_shutdown()
    else {
        panic!("other original contribution retains owner")
    };
    assert_eq!(reason, DomShutdownReason::ParsedAssociations(second_id));
    session = recovered;
    assert_eq!(first_end.contribution(), first_id);
    let second_end = end(&mut session, second_id, envelope());
    drop(complete(&mut session, second_id, envelope()).unwrap());
    assert_ready(
        &session.snapshot(),
        second.node_for(b.subject()).unwrap(),
        None,
    );
    assert_eq!(session.pending_parsed_html_associations().count(), 0);
    assert_eq!(second_end.contribution(), second_id);
    assert_pending_readback(&before, first.node_for(a.subject()).unwrap());
}
