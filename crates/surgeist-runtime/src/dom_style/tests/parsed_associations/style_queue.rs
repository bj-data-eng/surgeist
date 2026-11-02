//! Original completion enters the real queue without acquiring a dispatch frontier.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::envelope;
use super::style_sheet::{bind, create, pending, ready};
use surgeist_dom::*;
use surgeist_html::HtmlStyleTrigger;
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
};

fn rendered(
    text: &str,
    limits: DomParsedHtmlAssociationLimits,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    let (mut session, installed) = ready(text, limits);
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    let sheet = create(&mut session, id, limits)
        .unwrap()
        .completion()
        .sheet()
        .clone();
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), &q, limits, &cancel())
        .unwrap();
    assert!(matches!(
        session
            .style_execution()
            .process_parsed_style_rendering(&mut session.dom(), &q, limits, &cancel())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    (session, installed, q, sheet)
}
fn queue(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> DomStyleNetworkingTaskId {
    let DomStyleParsedNotificationProgress::Queued { task } = session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    else {
        panic!("actual first NoImports receipt")
    };
    task
}
fn cap(session: &DomSession) -> std::rc::Rc<DomStyleCssomRoot> {
    session.state.borrow().completions[0].owner().clone()
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
fn next_occurrence(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
    original: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
) -> surgeist_html_parser::HtmlParseAssociationOccurrenceId {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap();
    let mut cursor = entry.report().pending_associations();
    let mut after = false;
    loop {
        match cursor.advance() {
            Step::Fault(fault) => panic!("{fault}"),
            Step::Occurrence(row) if row.id() == *original => after = true,
            Step::Occurrence(row) if after && row.requires_processing() => {
                assert!(matches!(
                    row.kind(),
                    Kind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
                ));
                return row.id();
            }
            Step::Complete => panic!("literal source has original parser-pop work"),
            _ => (),
        }
    }
}

#[test]
fn authentic_queue_survives_current_authored_import_removal_output_drop_and_session_move() {
    let (mut session, installed, q, sheet) = rendered(
        "<!doctype html><style title=chosen>p{color:red}</style>",
        envelope(),
    );
    let expected = next_occurrence(&session, q.contribution(), q.occurrence());
    let owner = cap(&session);
    let before = owner.snapshot().unwrap();
    let mut edit = owner.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
    edit.insert_sheet_rule(sheet.id(), "@import 'later.css';", 0)
        .unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    let task = queue(&mut session, &q);
    {
        let state = session.state.borrow();
        let actual = state.completions[0].task();
        assert_eq!(actual.id(), task);
        assert!(
            matches!(actual.token.origin(),DomStyleNotificationOrigin::ParsedOccurrence{contribution,occurrence}
            if *contribution==q.contribution() && occurrence==q.occurrence())
        );
        let record = actual.parsed.as_ref().unwrap();
        assert_eq!(&record.sheet, &sheet);
        assert_eq!(record.qualification, q);
        assert_eq!(record.owner_label, "original-root:1");
        assert_eq!(record.dependencies, ImportDependencies::NoImports);
        assert_eq!(state.next_task, 2);
    }
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(
        &mut session.dom(),&q,envelope(),&cancel()),Err(DomStyleParsedNotificationError::AlreadyQueued{task:actual}) if actual==task)
    );
    let current = owner.snapshot().unwrap();
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&current))
        .unwrap();
    assert!(edit.remove_sheet(sheet.id()).unwrap());
    drop(owner.commit(&mut Some(edit)).unwrap());
    drop(installed);
    let DomShutdownOutcome::Pending {
        mut session,
        reason,
    } = session.try_shutdown()
    else {
        panic!("pending history")
    };
    assert_eq!(
        reason,
        DomShutdownReason::ParsedAssociations(q.contribution())
    );
    let mut events = context(&session);
    let mut dispatcher = dispatcher();
    dispatcher
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
    let stamp = session.snapshot().stamp();
    for _ in 0..2 {
        assert!(
            matches!(DomStyleRuntime::run_networking(&mut session.dom(),&mut dispatcher,&mut events,
            dispatch_limits(),&cancel()).unwrap(),DomStyleNetworkingProgress::OriginalSynchronousPending{
                task:actual,requirement:DomParsedHtmlStyleNotificationRequirement::Occurrence{
                    contribution,occurrence,document,kind:Kind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
                }} if actual==task && contribution==q.contribution() && occurrence==expected && document==q.document())
        );
    }
    assert!(events.events.is_empty());
    assert_eq!(session.snapshot().stamp(), stamp);
    let state = session.state.borrow();
    let actual = state.completions[0].task();
    assert!(!actual.running);
    assert_eq!(actual.dispatch_reserved, 0);
    assert!(matches!(actual.delivery, Delivery::NotStarted));
    assert_eq!(
        actual.cursor.action(),
        HtmlStyleNotificationAction::DispatchLoad
    );
    pending(&session, q.contribution());
}

#[test]
fn owner_removed_before_queue_releases_terminal_row_without_issuing_a_task() {
    let (mut session, _installed, q, sheet) =
        rendered("<!doctype html><style>x{}</style>", envelope());
    let owner = cap(&session);
    let native_document = session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node != q.document().node())
        .unwrap()
        .node;
    append_style(&mut session, native_document, None);
    let native = session.state.borrow().completions[1].task().id();
    let current = owner.snapshot().unwrap();
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&current))
        .unwrap();
    edit.remove_sheet(sheet.id()).unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    let before = session.snapshot().stamp();
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(
        &mut session.dom(),&q,envelope(),&cancel()).unwrap(),
        DomStyleParsedNotificationProgress::OwnerAssociationRemoved{association} if association==sheet.association())
    );
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(session.state.borrow().completions.len(), 1);
    assert_eq!(session.state.borrow().completions[0].task().id(), native);
    assert_eq!(
        session.state.borrow().next_task,
        2,
        "only the native task spent a serial"
    );
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(
        &mut session.dom(),&q,envelope(),&cancel()),Err(DomStyleParsedNotificationError::ProcessingUnavailable{contribution})
            if contribution==q.contribution())
    );
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    let mut dispatcher = dispatcher();
    let mut events = context(&session);
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(),&mut dispatcher,&mut events,
        dispatch_limits(),&cancel()).unwrap(),DomStyleNetworkingProgress::OriginalSynchronousPending{task,..} if task==native)
    );
    pending(&session, q.contribution());
}

#[test]
fn current_owner_attribute_revision_preserves_creation_label_and_allows_queue() {
    let (mut session, _installed, q, sheet) =
        rendered("<!doctype html><style>x{}</style>", envelope());
    let owner = cap(&session);
    let current = owner.snapshot().unwrap();
    let mut version = current.sheet(sheet.id()).unwrap().inputs().version.clone();
    version.revision += 1;
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&current))
        .unwrap();
    edit.update_sheet_owner_attributes(
        sheet.id(),
        CssomSheetOwnerAttributes {
            owner: "original-root:1",
            version,
            title: Some(&[b'x' as u16]),
            media: CssomOwnerMediaInput::Utf8(None),
        },
    )
    .unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    assert_ne!(owner.snapshot().unwrap().revision(), sheet.revision());
    queue(&mut session, &q);
}

#[test]
fn queue_original_output_refusal_and_raised_retry_preserve_all_three_owners() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        // The explicit lower continuation adds one genuine cumulative output;
        // the following command must still refuse and preserve its owners.
        11,
    )
    .unwrap();
    let (mut session, _installed, q, sheet) = rendered("<!doctype html><style>x{}</style>", finite);
    let before = session.snapshot().stamp();
    let revision = cap(&session).snapshot().unwrap().revision().clone();
    assert!(matches!(
        session.style_execution().queue_parsed_style_notification(
            &mut session.dom(),
            &q,
            finite,
            &cancel()
        ),
        Err(DomStyleParsedNotificationError::Runtime(
            DomStyleRuntimeError::Dom(DomError::ResourceLimit {
                resource: DomResource::ParsedAssociationProcessingOutputs,
                limit: 11
            })
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before);
    assert_eq!(cap(&session).snapshot().unwrap().revision(), &revision);
    assert!(matches!(
        session.state.borrow().completions[0],
        Completion::ParsedProcessing(_)
    ));
    assert_eq!(session.state.borrow().next_task, 1);
    let task = queue(&mut session, &q);
    assert_eq!(
        session.state.borrow().completions[0].task().sheet,
        *sheet.id()
    );
    assert_eq!(session.state.borrow().completions[0].task().id(), task);
}

#[test]
fn wrong_phase_foreign_owner_and_cancellation_do_not_consume_queue_issuance() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    drop(create(&mut session, id, envelope()).unwrap());
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    let runtime = session.style_execution();
    let expected = session.state.borrow().completions[0]
        .processing()
        .unwrap()
        .sheet
        .association();
    assert!(matches!(
        runtime.queue_parsed_style_notification(&mut session.dom(), &q, envelope(), &cancel()),
        Err(DomStyleParsedNotificationError::RenderingIncomplete { association }) if association == expected
    ));
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    let (mut foreign, _) = bound_dispatch_session();
    assert!(matches!(
        runtime.queue_parsed_style_notification(&mut foreign.dom(), &q, envelope(), &cancel()),
        Err(DomStyleParsedNotificationError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        runtime.queue_parsed_style_notification(
            &mut session.dom(),
            &q,
            envelope(),
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedNotificationError::Notification(
            DomParsedHtmlStyleNotificationError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ))
    ));
    assert!(matches!(
        session.state.borrow().completions[0],
        Completion::ParsedProcessing(_)
    ));
    assert_eq!(session.state.borrow().next_task, 1);
}

#[test]
fn blocked_parsed_fifo_head_does_not_overtake_into_native_task_and_keeps_source_guard() {
    let (mut session, _installed, q, _sheet) =
        rendered("<!doctype html><style>x{}</style>", envelope());
    let parsed = queue(&mut session, &q);
    let native_document = session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node != q.document().node())
        .unwrap()
        .node;
    let native_target = append_style(&mut session, native_document, None);
    let native = session.state.borrow().completions[1].task().id();
    let mut dispatcher = dispatcher();
    let mut events = context(&session);
    for _ in 0..2 {
        assert!(
            matches!(DomStyleRuntime::run_networking(&mut session.dom(),&mut dispatcher,&mut events,
            dispatch_limits(),&cancel()).unwrap(),DomStyleNetworkingProgress::OriginalSynchronousPending{task,..} if task==parsed)
        );
    }
    assert!(events.events.is_empty());
    assert_eq!(session.state.borrow().completions[1].task().id(), native);
    assert!(matches!(
        session.state.borrow().completions[1].task().delivery,
        Delivery::NotStarted
    ));
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(native_document, Namespace::html(), "dialog");
    let parent = session.snapshot().node(native_document).unwrap().children()[0];
    batch.insert_before(&dialog, parent, None);
    let setup = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = setup.created(&dialog).unwrap().unwrap();
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::show_dialog(dialog),
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
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(_),
            ..
        }
    ));
    let operation = session.dom().pending_source_edit().unwrap().operation();
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut events,
            dispatch_limits(),
            &cancel()
        ),
        Err(DomStyleRuntimeError::Dispatch(
            crate::DomEventDispatchError::Dom(DomError::IncompleteSourceOperation)
        ))
    ));
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    assert!(events.events.is_empty());
    assert_ne!(native_target, q.document().node());
}
