//! Independent original-occurrence oracles from CSSOM6.2 and real parsed history.
use super::*;
use std::sync::Arc;
use surgeist_cssom::{CssomError, CssomLimits, CssomPublication};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseAssociationOccurrenceId as Occurrence, HtmlParseReport,
    HtmlSource,
};

fn utf16_packet() -> Arc<HtmlParseReport> {
    let mut input: Vec<u16> = "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content='".encode_utf16().collect();
    input.extend([0x42, 0xd800, 0x61]);
    input.extend(
        "'><meta http-equiv=default-style content=A><meta http-equiv=default-style content=A>"
            .encode_utf16(),
    );
    Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(input)),
            super::super::consumption::parsing(),
        )
        .parse()
        .unwrap(),
    )
}

fn original_metas(session: &DomSession, id: DomParsedHtmlAssociationId) -> Vec<Occurrence> {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap();
    let mut cursor = entry.report().pending_associations();
    let mut rows = Vec::new();
    loop {
        match cursor.advance() {
            Step::Fault(fault) => panic!("{fault}"),
            Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => rows.push(row.id()),
            Step::Complete => return rows,
            _ => (),
        }
    }
}
fn start(session: &mut DomSession, id: DomParsedHtmlAssociationId) {
    request_ready(
        session,
        id,
        super::super::style_decision::empty_policy(),
        envelope(),
    );
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    decide(session, &q, true);
}
fn first(
    source: &str,
    producer: CssomLimits,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    first_packet(report(source), producer)
}
fn first_packet(
    packet: Arc<HtmlParseReport>,
    producer: CssomLimits,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    let (mut session, installed) = installed(packet);
    let id = installed.pending_association().unwrap();
    start(&mut session, id);
    bind(&mut session, installed.document(), producer);
    let result = create(&mut session, id, envelope()).unwrap();
    let sheet = result.completion().sheet().clone();
    drop(result);
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    finish_rendering(&mut session, &q);
    (session, installed, q, sheet)
}
fn pop(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
) -> (DomParsedHtmlStyleQualification, DomStyleSheet) {
    let result = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    let q = result.qualification().clone();
    drop(result);
    decide(session, &q, true);
    let result = create(session, id, envelope()).unwrap();
    let sheet = result.completion().sheet().clone();
    drop(result);
    finish_rendering(session, &q);
    (q, sheet)
}
fn names(session: &mut DomSession, document: DocumentId) -> DomStyleSheetSetObservation {
    session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap()
}
fn disabled(session: &DomSession, sheet: &DomStyleSheet) -> bool {
    session
        .style_execution()
        .snapshot(sheet.root())
        .unwrap()
        .unwrap()
        .sheet(sheet.id())
        .unwrap()
        .inputs()
        .disabled
}
fn end(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
) -> Result<DomParsedHtmlSynchronousEnd, DomStyleParsedFrontierError> {
    session
        .style_execution()
        .complete_parsed_html_synchronous_execution(&mut session.dom(), id, envelope(), &cancel())
}
fn queue(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> DomStyleNetworkingTaskId {
    match session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    {
        DomStyleParsedNotificationProgress::Queued { task } => task,
        other => panic!("genuine NoImports queue: {other:?}"),
    }
}
fn dispatcher() -> DomEventDispatcher<Context> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap()
}
fn context(session: &DomSession) -> Context {
    Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    }
}
fn networking(
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
fn assert_remaining_meta(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    expected: &Occurrence,
) {
    assert!(
        matches!(end(session, id), Err(DomStyleParsedFrontierError::Frontier(DomParsedHtmlFrontierError::Remaining(DomParsedHtmlStyleNotificationRequirement::Occurrence { occurrence, kind: Kind::ChangePreferred, .. }))) if &occurrence == expected)
    );
}
fn consume(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    row: &Occurrence,
) -> DomStyleParsedMetaOccurrenceOutcome {
    session
        .style_execution()
        .consume_parsed_meta_occurrence(&mut session.dom(), id, row, envelope(), &cancel())
        .unwrap()
}
fn same_names(actual: &DomStyleSheetSetObservation, expected: &DomStyleSheetSetObservation) {
    assert_eq!(actual.document(), expected.document());
    assert_eq!(actual.preferred_name(), expected.preferred_name());
    assert_eq!(actual.last_selected_name(), expected.last_selected_name());
    assert_eq!(actual.revision(), expected.revision());
    assert_eq!(actual.cssom_revision(), expected.cssom_revision());
}

#[test]
fn original_utf16_b_then_a_then_a_preserves_owned_history_and_releases_real_fifo_only_after_end() {
    let (mut session, installed) = installed(utf16_packet());
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    let rows = original_metas(&session, id);
    assert_eq!(rows.len(), 3);
    let mut cursor = installed.report().pending_associations();
    let subject = loop {
        if let Step::Occurrence(row) = cursor.advance()
            && row.id() == rows[0]
        {
            break row.subject().id();
        }
    };
    let meta = installed.node_for(&subject).unwrap();
    let native_limits = session.dom().limits().with_meta_processing_work(1_000_000);
    session.dom().raw_mut().set_limits(native_limits).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.destroy(meta);
    session.dom().apply(batch, &cancel()).unwrap();
    assert!(matches!(
        session.snapshot().node(meta),
        Err(DomError::StaleNode)
    ));
    start(&mut session, id);
    bind(&mut session, document, Default::default());
    let result = create(&mut session, id, envelope()).unwrap();
    let old_sheet = result.completion().sheet().clone();
    drop(result);
    let old = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    finish_rendering(&mut session, &old);
    let first_task = queue(&mut session, &old);
    let (q, sheet) = pop(&mut session, id);
    let second_task = queue(&mut session, &q);
    let task_serial = session.state.borrow().next_task;
    assert_ne!(old_sheet.id(), sheet.id());
    let creation_revision = sheet.revision().clone();
    let before = names(&mut session, document);
    assert_eq!(before.preferred_name(), &DocumentString::from("A"));
    assert_eq!(before.last_selected_name(), None);
    assert!(!disabled(&session, &sheet));
    assert!(!disabled(&session, &old_sheet));
    assert_remaining_meta(&mut session, id, &rows[0]);
    let mut events = context(&session);
    let mut dispatch = dispatcher();
    dispatch
        .add_listener(
            &mut session.dom(),
            sheet.association().element(),
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
    assert!(
        matches!(networking(&mut session, &mut dispatch, &mut events), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == first_task)
    );
    let stamp = session.snapshot().stamp();
    drop(installed);
    let b = consume(&mut session, id, &rows[0]);
    assert_eq!(b.completion().occurrence().id(), &rows[0]);
    assert_eq!(b.completion().occurrence().subject(), &subject);
    assert_eq!(b.completion().occurrence().document(), document);
    assert_eq!(
        b.completion().occurrence().content().code_units(),
        &[0x42, 0xd800, 0x61]
    );
    assert_eq!(
        b.preference().preferred_name().code_units(),
        &[0x42, 0xd800, 0x61]
    );
    assert_eq!(b.preference().revision(), before.revision() + 1);
    assert_eq!(b.completion().stamp(), stamp);
    assert!(disabled(&session, &sheet));
    assert!(
        !disabled(&session, &old_sheet),
        "removed old sheet is outside the named set"
    );
    assert_remaining_meta(&mut session, id, &rows[1]);
    let a = consume(&mut session, id, &rows[1]);
    assert_eq!(a.preference().preferred_name(), &DocumentString::from("A"));
    assert_eq!(a.preference().revision(), before.revision() + 2);
    assert_eq!(a.preference().last_selected_name(), None);
    assert!(!disabled(&session, &sheet));
    assert_ne!(
        a.preference().cssom_revision(),
        b.preference().cssom_revision()
    );
    assert_eq!(
        b.preference().preferred_name().code_units(),
        &[0x42, 0xd800, 0x61]
    );
    assert_remaining_meta(&mut session, id, &rows[2]);
    let DomShutdownOutcome::Pending {
        session: recovered,
        reason,
    } = session.try_shutdown()
    else {
        panic!("whole original history remains pending")
    };
    assert_eq!(reason, DomShutdownReason::ParsedAssociations(id));
    session = recovered;
    assert_eq!(b.completion().occurrence().id(), &rows[0]);
    let repeated = consume(&mut session, id, &rows[2]);
    assert_eq!(repeated.completion().occurrence().id(), &rows[2]);
    same_names(repeated.preference(), a.preference());
    assert!(
        matches!(repeated.completion().cssom().publication(), CssomPublication::Unchanged { revision } if revision == a.preference().cssom_revision())
    );
    assert!(!disabled(&session, &sheet));
    assert!(
        matches!(networking(&mut session, &mut dispatch, &mut events), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == first_task)
    );
    assert!(events.events.is_empty());
    assert!(
        session
            .dom
            .parsed_html_synchronous_end(id)
            .unwrap()
            .is_none()
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(session.state.borrow().next_task, task_serial);
    assert_eq!(
        session
            .style_execution()
            .snapshot(sheet.root())
            .unwrap()
            .unwrap()
            .sheets(),
        &[sheet.id().clone()]
    );
    drop((b, a, repeated));
    drop(end(&mut session, id).unwrap());
    for expected in [first_task, second_task] {
        assert!(
            matches!(networking(&mut session, &mut dispatch, &mut events), DomStyleNetworkingProgress::Complete { task, .. } if task == expected)
        );
    }
    assert!(matches!(
        networking(&mut session, &mut dispatch, &mut events),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(
        events.events,
        vec![(sheet.association().element(), sheet.association().element()); 2]
    );
    assert_eq!(
        sheet.revision(),
        &creation_revision,
        "creation receipt stays historical"
    );
    pending(&session, id);
}

#[test]
fn skipped_foreign_and_duplicate_ids_never_consume_another_original_row() {
    let source = "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B><meta http-equiv=default-style content=A>";
    let packet = report(source);
    let (mut session, installed, old, _) = first_packet(packet.clone(), Default::default());
    let id = old.contribution();
    let (_, sheet) = pop(&mut session, id);
    let rows = original_metas(&session, id);
    let before = names(&mut session, installed.document());
    let (mut same_packet, same_installed, same_old, _) = first_packet(packet, Default::default());
    let same_id = same_old.contribution();
    let (_, same_sheet) = pop(&mut same_packet, same_id);
    let same_rows = original_metas(&same_packet, same_id);
    assert_eq!(same_rows, rows);
    assert_ne!(same_id, id);
    assert_ne!(same_installed.document(), installed.document());
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &rows[1],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::WrongOccurrence
        ))
    ));
    let (other, other_installed) = super::super::style_decision::installed(report(source));
    let foreign = original_metas(&other, other_installed.pending_association().unwrap());
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &foreign[0],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::WrongOccurrence
        ))
    ));
    same_names(&names(&mut session, installed.document()), &before);
    assert!(!disabled(&session, &sheet));
    assert_remaining_meta(&mut session, id, &rows[0]);
    drop(consume(&mut session, id, &rows[0]));
    let b = names(&mut session, installed.document());
    assert_eq!(
        names(&mut same_packet, same_installed.document()).preferred_name(),
        &DocumentString::from("A")
    );
    assert!(!disabled(&same_packet, &same_sheet));
    drop(consume(&mut same_packet, same_id, &same_rows[0]));
    assert_eq!(
        names(&mut same_packet, same_installed.document()).preferred_name(),
        &DocumentString::from("B")
    );
    assert!(disabled(&same_packet, &same_sheet));
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &rows[0],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::AlreadyConsumed
        ))
    ));
    same_names(&names(&mut session, installed.document()), &b);
    assert_remaining_meta(&mut session, id, &rows[1]);
    assert!(disabled(&session, &sheet));
    drop(consume(&mut session, id, &rows[1]));
    assert!(!disabled(&session, &sheet));
}

#[test]
fn incomplete_successor_and_later_unsupported_style_remain_actual_dependencies() {
    let source = "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B><style>q{}</style><meta http-equiv=default-style content=C>";
    let (mut session, installed, old, _) = first(source, Default::default());
    let id = old.contribution();
    let rows = original_metas(&session, id);
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &rows[0],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::Remaining(_)
        ))
    ));
    let result = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    let q = result.qualification().clone();
    drop(result);
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &rows[0],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::Remaining(
                DomParsedHtmlStyleNotificationRequirement::Invocation {
                    phase: DomParsedHtmlStylePhase::InlinePolicyPending,
                    ..
                }
            )
        ))
    ));
    decide(&mut session, &q, true);
    drop(create(&mut session, id, envelope()).unwrap());
    finish_rendering(&mut session, &q);
    drop(consume(&mut session, id, &rows[0]));
    let before = names(&mut session, installed.document());
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &rows[1],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::Remaining(
                DomParsedHtmlStyleNotificationRequirement::Occurrence {
                    kind: Kind::StyleUpdate(_),
                    ..
                }
            )
        ))
    ));
    assert!(matches!(
        end(&mut session, id),
        Err(DomStyleParsedFrontierError::Frontier(
            DomParsedHtmlFrontierError::Remaining(
                DomParsedHtmlStyleNotificationRequirement::Occurrence {
                    kind: Kind::StyleUpdate(_),
                    ..
                }
            )
        ))
    ));
    same_names(&names(&mut session, installed.document()), &before);
    pending(&session, id);
}

#[test]
fn genuinely_blocked_first_and_pop_still_change_preferred_with_an_actual_empty_owner() {
    let (policy, profile) = text_policy::policy(false);
    let (mut session, installed, old, _) = blocked_first(
        "<!doctype html><style>p{color:red}</style><meta http-equiv=default-style content=B>",
        policy,
        &profile,
    );
    let id = old.contribution();
    let rows = original_metas(&session, id);
    let result = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        result.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    drop(result);
    let mut input = observe(&session, id, &mut |request| {
        Some(text_policy::decision(request, &profile))
    });
    assert!(!input.as_ref().unwrap().allowed());
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut input,
                envelope(),
                &cancel()
            )
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(
        session
            .style_execution()
            .snapshot(installed.document().node())
            .unwrap()
            .is_none()
    );
    assert!(
        session
            .style_execution()
            .consume_parsed_meta_occurrence(&mut session.dom(), id, &rows[0], envelope(), &cancel())
            .is_err()
    );
    bind(&mut session, installed.document(), Default::default());
    let before = names(&mut session, installed.document());
    let result = consume(&mut session, id, &rows[0]);
    assert_eq!(
        result.preference().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(result.preference().revision(), before.revision() + 1);
    assert_eq!(result.preference().last_selected_name(), None);
    assert!(
        matches!(result.completion().cssom().publication(), CssomPublication::Unchanged { revision } if revision == before.cssom_revision())
    );
    assert!(
        session
            .style_execution()
            .snapshot(installed.document().node())
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    assert!(session.state.borrow().completions.is_empty());
    drop(result);
    drop(end(&mut session, id).unwrap());
    pending(&session, id);
}

#[test]
fn cancellation_busy_runtime_owner_and_actual_work_refusal_preserve_row_until_raised_retry() {
    let (mut session, installed, old, _) = first(
        "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B>",
        Default::default(),
    );
    let id = old.contribution();
    let (_, sheet) = pop(&mut session, id);
    let row = original_metas(&session, id).remove(0);
    let before = names(&mut session, installed.document());
    assert!(
        session
            .style_execution()
            .consume_parsed_meta_occurrence(
                &mut session.dom(),
                id,
                &row,
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    let runtime_owner = session.state.clone();
    let locked = runtime_owner.borrow_mut();
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &row,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Runtime(
            DomStyleRuntimeError::Binding
        ))
    ));
    drop(locked);
    same_names(&names(&mut session, installed.document()), &before);
    assert!(!disabled(&session, &sheet));
    let mut runtime = session.style_execution();
    let owner = runtime.original().unwrap();
    let work = owner.borrow().work.get();
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    let cost = owner.borrow().work.get() - work;
    let mut tight = limits();
    tight.work = owner.borrow().work.get() + cost;
    runtime
        .set_limits(&session.dom(), tight, &cancel())
        .unwrap();
    assert!(matches!(
        runtime.consume_parsed_meta_occurrence(&mut session.dom(), id, &row, envelope(), &cancel()),
        Err(DomStyleParsedMetaOccurrenceError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    same_names(&names(&mut session, installed.document()), &before);
    assert_remaining_meta(&mut session, id, &row);
    let result = consume(&mut session, id, &row);
    assert_eq!(result.preference().revision(), before.revision() + 1);
    assert!(disabled(&session, &sheet));
}

#[test]
fn real_fixed_producer_revision_limit_refuses_after_prefix_without_partial_preference_or_progress()
{
    let (mut session, installed, old, _) = first(
        "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B>",
        CssomLimits {
            max_revision: 3,
            ..Default::default()
        },
    );
    let id = old.contribution();
    let (_, sheet) = pop(&mut session, id);
    let row = original_metas(&session, id).remove(0);
    let before = names(&mut session, installed.document());
    assert_eq!(
        before.cssom_revision().value(),
        3,
        "actual preceding creation/removal/recreation reaches producer limit"
    );
    assert!(matches!(
        session.style_execution().consume_parsed_meta_occurrence(
            &mut session.dom(),
            id,
            &row,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedMetaOccurrenceError::Meta(
            DomParsedHtmlMetaOccurrenceError::Cssom(CssomError::RevisionExhausted)
        ))
    ));
    same_names(&names(&mut session, installed.document()), &before);
    assert!(!disabled(&session, &sheet));
    assert_remaining_meta(&mut session, id, &row);
}

#[test]
fn actual_import_processing_wait_does_not_block_meta_or_authentic_synchronous_end() {
    let (mut session, installed, old, _) = first(
        "<!doctype html><style title=A>@import 'theme.css';p{color:red}</style><meta http-equiv=default-style content=B>",
        Default::default(),
    );
    let id = old.contribution();
    let old_task = queue(&mut session, &old);
    let (q, _) = pop(&mut session, id);
    let row = original_metas(&session, id).remove(0);
    let serial = session.state.borrow().next_task;
    assert!(matches!(
        session
            .style_execution()
            .queue_parsed_style_notification(&mut session.dom(), &q, envelope(), &cancel())
            .unwrap(),
        DomStyleParsedNotificationProgress::PendingImports { .. }
    ));
    drop(consume(&mut session, id, &row));
    drop(end(&mut session, id).unwrap());
    assert_eq!(session.state.borrow().next_task, serial);
    let mut events = context(&session);
    let mut dispatch = dispatcher();
    assert!(
        matches!(networking(&mut session, &mut dispatch, &mut events), DomStyleNetworkingProgress::Complete { task, .. } if task == old_task)
    );
    assert!(matches!(
        networking(&mut session, &mut dispatch, &mut events),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    assert!(session.state.borrow().completions.iter().any(|row| {
        row.processing()
            .is_some_and(|p| p.dependencies == ImportDependencies::Imports)
    }));
    assert_eq!(
        names(&mut session, installed.document()).preferred_name(),
        &DocumentString::from("B")
    );
    pending(&session, id);
}
