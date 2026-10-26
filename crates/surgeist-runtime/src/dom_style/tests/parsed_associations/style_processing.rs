//! Creation-time resource facts remain in the actual Session lifecycle bank.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::envelope;
use super::style_sheet::{bind, create, pending, ready};
use surgeist_dom::*;

fn qualification(
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
fn record(session: &DomSession, inspect: impl FnOnce(&ParsedProcessing)) {
    let state = session.state.borrow();
    assert_eq!(state.completions.len(), 1);
    let Completion::ParsedProcessing(value) = &state.completions[0] else {
        panic!("original processing is not an import wait or networking task")
    };
    inspect(value);
}
fn unchanged(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    document: DocumentId,
    stamp: DomStamp,
    revision: &surgeist_cssom::CssomRevision,
) {
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(session.state.borrow().completions.is_empty());
    let cssom = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(cssom.revision(), revision);
    assert!(cssom.sheets().is_empty());
    assert!(
        session
            .style_execution()
            .sheet_sets(&session.dom(), document.node(), &cancel())
            .unwrap()
            .preferred_name()
            .is_empty()
    );
    pending(session, id);
}

#[test]
fn actual_first_parse_receipt_survives_output_drop_and_later_authored_import_and_removal() {
    let (mut session, installed) = ready(
        "<!doctype html><style title=chosen media='screen and'>@import 'final.css'; p{color:red}</style>",
        envelope(),
    );
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    bind(&mut session, document, Default::default());
    let original = qualification(&session, id);
    let created = create(&mut session, id, envelope()).unwrap();
    let sheet = created.completion().sheet().clone();
    assert_eq!(
        installed.node_for(original.subject()),
        Some(sheet.association().element())
    );
    assert!(!created.completion().media_diagnostics().is_empty());
    let snapshot = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap();
    assert!(
        snapshot
            .sheet(sheet.id())
            .unwrap()
            .authored()
            .rules()
            .is_empty()
    );
    drop(created);
    let owner = session.state.borrow().completions[0].owner().clone();
    record(&session, |value| {
        assert_eq!(value.qualification, original);
        assert_eq!(&value.sheet, &sheet);
        assert_eq!(value.owner.root(), document.node());
        assert_eq!(
            value.profile,
            HtmlStyleExecutionProfile::NoExecutableJavaScript
        );
        assert_eq!(value.dependencies, ImportDependencies::NoImports);
    });
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&snapshot))
        .unwrap();
    edit.insert_sheet_rule(sheet.id(), "@import 'later.css';", 0)
        .unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    let imported = owner.snapshot().unwrap();
    assert_ne!(imported.revision(), sheet.revision());
    assert!(!imported.sheet_rules(sheet.id()).unwrap().is_empty());
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&imported))
        .unwrap();
    assert!(edit.remove_sheet(sheet.id()).unwrap());
    drop(owner.commit(&mut Some(edit)).unwrap());
    assert!(owner.snapshot().unwrap().sheets().is_empty());
    record(&session, |value| {
        assert_eq!(value.dependencies, ImportDependencies::NoImports);
        assert_eq!(&value.sheet, &sheet);
        assert_eq!(value.qualification, original);
    });
    assert!(create(&mut session, id, envelope()).is_err());
    record(&session, |_| {});
    pending(&session, id);
}

#[test]
fn processing_bank_limit_refuses_before_publication_and_raise_installs_one_receipt() {
    let (mut session, installed) =
        ready("<!doctype html><style title=chosen>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    bind(&mut session, document, Default::default());
    let mut runtime = session.style_execution();
    let mut small = limits();
    small.completions = 0;
    runtime
        .set_limits(&session.dom(), small, &cancel())
        .unwrap();
    let stamp = session.snapshot().stamp();
    let revision = runtime
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    let before_work = session.state.borrow().work.get();
    assert!(matches!(
        create(&mut session, id, envelope()),
        Err(DomStyleParsedSheetError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(session.state.borrow().work.get() > before_work);
    unchanged(&mut session, id, document, stamp, &revision);
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    let created = create(&mut session, id, envelope()).unwrap();
    record(&session, |value| {
        assert_eq!(&value.sheet, created.completion().sheet())
    });
    assert_eq!(
        session.state.borrow().next_task,
        1,
        "no task serial was issued"
    );
    assert!(create(&mut session, id, envelope()).is_err());
    record(&session, |_| {});
}

#[test]
fn original_output_refusal_retains_spent_work_and_retries_without_a_partial_receipt() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        3,
    )
    .unwrap();
    let (mut session, installed) = ready("<!doctype html><style title=chosen>x{}</style>", finite);
    let id = installed.pending_association().unwrap();
    let document = installed.document();
    bind(&mut session, document, Default::default());
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    let before_work = {
        let mut dom = session.dom();
        dom.raw_mut()
            .begin_parsed_html_style(id, finite, &cancel())
            .unwrap()
            .spent_work()
    };
    assert!(matches!(
        create(&mut session, id, finite),
        Err(DomStyleParsedSheetError::Runtime(
            DomStyleRuntimeError::Dom(DomError::ResourceLimit {
                resource: DomResource::ParsedAssociationProcessingOutputs,
                limit: 3
            })
        ))
    ));
    unchanged(&mut session, id, document, stamp, &revision);
    let after_work = {
        let mut dom = session.dom();
        dom.raw_mut()
            .begin_parsed_html_style(id, finite, &cancel())
            .unwrap()
            .spent_work()
    };
    assert!(after_work > before_work);
    drop(create(&mut session, id, large).unwrap());
    record(&session, |value| {
        assert_eq!(value.dependencies, ImportDependencies::NoImports)
    });
}

#[test]
fn receipt_uses_actual_runtime_storage_and_is_not_a_networking_or_critical_wait() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    let (before, capacity, sets) = {
        let state = session.state.borrow();
        (
            state.storage(None, &cancel(), state.limits.work).unwrap(),
            state.completions.capacity(),
            state
                .roots
                .iter()
                .map(|root| root.sheet_sets.backing_bytes().unwrap())
                .sum::<usize>(),
        )
    };
    drop(create(&mut session, id, envelope()).unwrap());
    let (after, new_capacity, new_sets) = {
        let state = session.state.borrow();
        (
            state.storage(None, &cancel(), state.limits.work).unwrap(),
            state.completions.capacity(),
            state
                .roots
                .iter()
                .map(|root| root.sheet_sets.backing_bytes().unwrap())
                .sum::<usize>(),
        )
    };
    assert_eq!(
        after - before,
        (new_capacity - capacity) * size_of::<Completion>() + new_sets - sets
            + session.state.borrow().completions[0]
                .processing()
                .unwrap()
                .owner_label
                .capacity()
    );
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    assert!(
        session
            .style_execution()
            .pending_sheet(0)
            .unwrap()
            .is_none()
    );
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    assert!(context.events.is_empty());
    record(&session, |_| {});
    pending(&session, id);
}

#[test]
fn whole_session_move_and_pending_shutdown_retain_receipt_until_terminal_drop() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    let original = qualification(&session, id);
    drop(create(&mut session, id, envelope()).unwrap());
    let weak = std::rc::Rc::downgrade(session.state.borrow().completions[0].owner());
    let state = std::rc::Rc::downgrade(&session.state);
    let moved = session;
    let DomShutdownOutcome::Pending { session, reason } = moved.try_shutdown() else {
        panic!("original history is still Pending")
    };
    assert_eq!(reason, DomShutdownReason::ParsedAssociations(id));
    record(&session, |value| assert_eq!(value.qualification, original));
    assert!(weak.upgrade().is_some());
    drop(session);
    assert!(state.upgrade().is_none());
    assert!(weak.upgrade().is_none());
}

#[test]
fn shared_actual_rule_classifier_distinguishes_imports_recovery_and_typed_admission() {
    let context = CssParserContext::default();
    let imported =
        CssParser::new(context, Default::default()).parse_sheet("@import 'real.css'; p{color:red}");
    assert_eq!(
        import_dependencies_with(&imported, || Ok::<_, ()>(())).unwrap(),
        ImportDependencies::Imports
    );
    let recovered = CssParser::new(context, Default::default()).parse_sheet("p{color:}");
    assert!(!recovered.diagnostics().is_empty());
    assert_eq!(
        import_dependencies_with(&recovered, || Ok::<_, ()>(())).unwrap(),
        ImportDependencies::NoImports
    );
    #[derive(Debug, PartialEq)]
    enum Refusal {
        OriginalAdmission,
    }
    let mut visits = 0;
    assert_eq!(
        import_dependencies_with(&imported, || {
            visits += 1;
            Err(Refusal::OriginalAdmission)
        }),
        Err(Refusal::OriginalAdmission)
    );
    assert_eq!(visits, 1);
}

#[test]
fn native_wait_and_delivery_remain_distinct_beside_original_processing_rows() {
    let (mut session, installed) = ready("<!doctype html><style>x{}</style>", envelope());
    let id = installed.pending_association().unwrap();
    bind(&mut session, installed.document(), Default::default());
    let created = create(&mut session, id, envelope()).unwrap();
    let first_sheet = created.completion().sheet().clone();
    drop(created);
    let native_document = session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node != installed.document().node())
        .unwrap()
        .node;
    let target = append_style(&mut session, native_document, None);
    let mut batch = DomBatch::new();
    let imported = batch.create_element_ns(native_document, Namespace::html(), "style");
    let text = batch.create_text(native_document, "@import 'real.css';");
    batch.insert_before(&text, &imported, None);
    let setup = session.dom().apply(batch, &cancel()).unwrap();
    let imported = setup.created(&imported).unwrap().unwrap();
    let parent = session.snapshot().node(native_document).unwrap().children()[0];
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(imported, parent, None),
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
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 1);
    assert_eq!(session.state.borrow().completions.len(), 3);
    let task = session.state.borrow().completions[1].task().id();
    let waiting = session.state.borrow().completions[2].task().id();
    assert_eq!(
        session
            .style_execution()
            .pending_sheet(0)
            .unwrap()
            .unwrap()
            .1,
        session
            .snapshot()
            .node(target)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .id()
            .clone()
    );
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
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
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher,
        &mut context, dispatch_limits(), &cancel()).unwrap(),
        DomStyleNetworkingProgress::OriginalSynchronousPending { task: actual, .. } if actual == task)
    );
    assert!(context.events.is_empty());
    let first = qualification(&session, id);
    super::style_recreation::finish_rendering(&mut session, &first);
    let next = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    let q = next.qualification().clone();
    drop(next);
    super::style_recreation::decide(&mut session, &q, true);
    let created = create(&mut session, id, envelope()).unwrap();
    let second_sheet = created.completion().sheet().clone();
    drop(created);
    super::style_recreation::finish_rendering(&mut session, &q);
    drop(
        session
            .style_execution()
            .complete_parsed_html_synchronous_execution(
                &mut session.dom(),
                id,
                envelope(),
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 1);
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher,
        &mut context, dispatch_limits(), &cancel()).unwrap(),
        DomStyleNetworkingProgress::Complete { task: actual, .. } if actual == task)
    );
    assert_eq!(context.events, [(target, target)]);
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 1);
    assert_eq!(session.state.borrow().completions.len(), 3);
    assert!(matches!(
        &session.state.borrow().completions[0],
        Completion::ParsedProcessing(_)
    ));
    assert!(matches!(
        &session.state.borrow().completions[1],
        Completion::Waiting(_)
    ));
    assert!(matches!(
        &session.state.borrow().completions[2],
        Completion::ParsedProcessing(_)
    ));
    {
        let state = session.state.borrow();
        let original = state.completions[0].processing().unwrap();
        assert_eq!(original.qualification, first);
        assert_eq!(&original.sheet, &first_sheet);
        assert_eq!(state.completions[1].task().id(), waiting);
        assert_eq!(state.completions[1].task().token.target(), imported);
        let recreated = state.completions[2].processing().unwrap();
        assert_eq!(recreated.qualification, q);
        assert_eq!(&recreated.sheet, &second_sheet);
        assert_ne!(first_sheet.id(), second_sheet.id());
    }
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    pending(&session, id);
}
