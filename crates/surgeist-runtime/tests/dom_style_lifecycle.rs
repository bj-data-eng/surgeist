//! Public native Style lifecycle, using actual CSS processing and shared events.
//! HTML877d raw12138–12219 and CSSOM add/remove rules supply literal outcomes.
use std::sync::atomic::AtomicBool;
use surgeist_css::rule::CssRule;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_cssom::{CssomEditGuard, CssomSnapshot, CssomStore};
use surgeist_dom::*;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1)
}
fn event_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 128,
        snapshot_listeners: 128,
        failure_records: 128,
        callbacks: 128,
        work: 100_000,
    }
}
struct Context {
    styles: DomStyleRuntime,
    source: DomSourceRuntime,
    events: Vec<(DomEventType, NodeId, DocumentId)>,
    observations: Vec<NativeDomEvent>,
    replacement: Option<DomStyleRuntime>,
}
impl DomStyleEventContext for Context {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}
fn fixture() -> (DomSession, Context, DocumentId, NodeId, DocumentId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            200_000,
            1024,
            1024,
            1024,
            32,
            2_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_event_recipients(
            surgeist_dom::DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000)
                .unwrap(),
        )
        .with_dispatch_custody(
            surgeist_dom::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let d1 = batch.create_document();
    let r1 = batch.create_element_ns(&d1, Namespace::html(), "div");
    batch.insert_before(&r1, &d1, None);
    let d2 = batch.create_document();
    let r2 = batch.create_element_ns(&d2, Namespace::html(), "div");
    batch.insert_before(&r2, &d2, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let d1 = commit.document(&d1).unwrap();
    let r1 = commit.created(&r1).unwrap().unwrap();
    let d2 = commit.document(&d2).unwrap();
    let r2 = commit.created(&r2).unwrap().unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
        observations: Vec::new(),
        replacement: None,
    };
    {
        let dom = session.dom();
        for (document, label) in [(d1, "first-root"), (d2, "second-root")] {
            let mut input = Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            ));
            context
                .styles
                .bind_root(&dom, label.into(), &mut input, &cancel())
                .unwrap();
            assert!(input.is_none());
        }
    }
    (session, context, d1, r1, d2, r2)
}
fn detached(dom: &mut DomEventDom<'_>, document: DocumentId, css: &str) -> (NodeId, NodeId) {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, css);
    batch.insert_before(&text, &style, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        commit.created(&style).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}
fn policy(source: &DomStyleSource) -> Option<DomStyleInlineCspObservation> {
    Some(DomStyleInlineCspObservation::complete_empty(source, 1))
}
fn drive(
    dom: &mut DomEventDom<'_>,
    context: &mut Context,
    edit: DomSourceEdit,
) -> DomSourceCompletion {
    match context
        .source
        .apply(
            dom,
            edit,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Complete(done) => done,
        other => panic!("supported synchronous association did not finish: {other:?}"),
    }
}
fn connect(dom: &mut DomEventDom<'_>, context: &mut Context, style: NodeId, parent: NodeId) {
    drive(
        dom,
        context,
        DomSourceEdit::insert_before(style, parent, None),
    );
}
fn cssom(context: &Context, document: DocumentId) -> CssomSnapshot {
    context.styles.snapshot(document.node()).unwrap().unwrap()
}
fn association(snapshot: &DomSnapshot, style: NodeId) -> Option<DomStyleSheet> {
    snapshot
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .cloned()
}
fn registry() -> DomEventDispatcher<Context> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 32,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap()
}
fn listener() -> DomEventCallback<Context> {
    DomEventCallback::new(|dom, _, context: &mut Context, event| {
        let target = event.target().unwrap();
        context.events.push((
            event.event_type(),
            target,
            dom.snapshot()
                .node(target)
                .unwrap()
                .owner_document()
                .unwrap(),
        ));
        Ok(())
    })
}
fn listen(dom: &mut DomEventDom<'_>, dispatcher: &mut DomEventDispatcher<Context>, target: NodeId) {
    for kind in [DomEventType::Load, DomEventType::Error] {
        dispatcher
            .add_listener(
                dom,
                target,
                kind,
                listener(),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
}
fn delivered(
    dom: &mut DomEventDom<'_>,
    dispatcher: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) -> DomStyleNetworkingTaskId {
    match DomStyleRuntime::run_networking(dom, dispatcher, context, event_limits(), &cancel())
        .unwrap()
    {
        DomStyleNetworkingProgress::Complete { task, receipt } => {
            // Assert outside callbacks: caught listener unwinds cannot hide a bad fixture.
            assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
            assert_eq!(receipt.event_type(), DomEventType::Load);
            assert!(!receipt.default_prevented());
            task
        }
        other => panic!("expected actual load delivery: {other:?}"),
    }
}
fn empty(
    dom: &mut DomEventDom<'_>,
    dispatcher: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) {
    assert!(matches!(
        DomStyleRuntime::run_networking(dom, dispatcher, context, event_limits(), &cancel())
            .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
}

#[test]
fn imports_have_a_real_wait_and_no_networking_delivery_before_or_after_owner_removal() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(
        &mut dom,
        document,
        "@import url(\"pending.css\"); p { color: red }",
    );
    connect(&mut dom, &mut context, style, parent);
    let old_dom = dom.snapshot();
    let old_sheet = association(&old_dom, style).unwrap();
    let old_cssom = cssom(&context, document);
    assert!(matches!(
        old_cssom.sheet(old_sheet.id()).unwrap().authored().rules()[0],
        CssRule::Import(_)
    ));
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    assert_eq!(
        context.styles.pending_sheet(0).unwrap(),
        Some((document.node(), old_sheet.id().clone()))
    );
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    empty(&mut dom, &mut dispatcher, &mut context);
    drive(&mut dom, &mut context, DomSourceEdit::detach(style));
    assert!(association(&dom.snapshot(), style).is_none());
    let current = cssom(&context, document);
    assert!(current.sheets().is_empty());
    assert!(
        current
            .sheet(old_sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    empty(&mut dom, &mut dispatcher, &mut context);
    assert!(context.events.is_empty());
    // Absence of a loader supplies neither success nor failure. The detached
    // historical sheet remains readable; no fabricated terminal event follows.
    assert!(
        old_cssom
            .sheet(old_sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(association(&old_dom, style).unwrap(), old_sheet);
}

#[test]
fn destroyed_import_waiting_target_is_retired_without_dispatching_to_a_live_replacement() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(
        &mut dom,
        document,
        "@import url(\"pending.css\"); p { color: red }",
    );
    connect(&mut dom, &mut context, style, parent);
    let historical_dom = dom.snapshot();
    let sheet = association(&historical_dom, style).unwrap();
    let historical_cssom = cssom(&context, document);
    assert!(matches!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .authored()
            .rules()[0],
        CssRule::Import(_)
    ));
    let pending = (document.node(), sheet.id().clone());
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    assert_eq!(
        context.styles.pending_sheet(0).unwrap(),
        Some(pending.clone())
    );
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);

    // Clearing the owner is not target retirement or a resource outcome.
    drive(&mut dom, &mut context, DomSourceEdit::detach(style));
    assert!(dom.snapshot().node(style).is_ok());
    assert!(association(&dom.snapshot(), style).is_none());
    let removed_cssom = cssom(&context, document);
    assert!(removed_cssom.sheets().is_empty());
    assert!(
        removed_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    empty(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    assert_eq!(
        context.styles.pending_sheet(0).unwrap(),
        Some(pending.clone())
    );
    assert!(context.events.is_empty());

    drive(&mut dom, &mut context, DomSourceEdit::destroy(style));
    assert!(matches!(
        dom.snapshot().node(style),
        Err(DomError::StaleNode)
    ));
    // A real later allocation is unrelated; no public arena-slot assumption.
    let (replacement, _) = detached(&mut dom, document, "p { color: blue }");
    assert_ne!(replacement, style);
    listen(&mut dom, &mut dispatcher, replacement);
    let before_pump = dom.snapshot();
    assert!(before_pump.node(replacement).unwrap().parent().is_none());
    assert!(association(&before_pump, replacement).is_none());
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    assert_eq!(context.styles.pending_sheet(0).unwrap(), Some(pending));
    assert_eq!(
        cssom(&context, document).revision(),
        removed_cssom.revision()
    );
    assert_eq!(association(&historical_dom, style).unwrap(), sheet);
    assert!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );

    match DomStyleRuntime::run_networking(
        &mut dom,
        &mut dispatcher,
        &mut context,
        event_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomStyleNetworkingProgress::TargetDestroyed { target, .. } => assert_eq!(target, style),
        other => panic!("expected exact destroyed import-Waiting target retirement, got {other:?}"),
    }
    assert!(context.events.is_empty());
    assert_eq!(context.styles.waiting_resources().unwrap(), 0);
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(dom.snapshot().stamp(), before_pump.stamp());
    assert!(dom.snapshot().node(replacement).unwrap().parent().is_none());
    assert!(association(&dom.snapshot(), replacement).is_none());
    assert_eq!(
        cssom(&context, document).revision(),
        removed_cssom.revision()
    );
    assert!(cssom(&context, document).sheet(sheet.id()).is_ok());
    assert_eq!(association(&historical_dom, style).unwrap(), sheet);
    assert!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    empty(&mut dom, &mut dispatcher, &mut context);
    assert!(context.events.is_empty());
}

#[test]
fn canceled_destroyed_import_waiting_retirement_preserves_exact_row_for_one_retry() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "@import 'pending.css';p{color:red}");
    connect(&mut dom, &mut context, style, parent);
    let historical_dom = dom.snapshot();
    let sheet = association(&historical_dom, style).unwrap();
    let historical_cssom = cssom(&context, document);
    assert!(matches!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .authored()
            .rules()[0],
        CssRule::Import(_)
    ));
    let pending = (document.node(), sheet.id().clone());
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    drive(&mut dom, &mut context, DomSourceEdit::detach(style));
    drive(&mut dom, &mut context, DomSourceEdit::destroy(style));
    let retired_dom = dom.snapshot();
    let removed_cssom = cssom(&context, document);
    assert!(matches!(retired_dom.node(style), Err(DomError::StaleNode)));
    assert_eq!(
        context.styles.pending_sheet(0).unwrap(),
        Some(pending.clone())
    );
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    let stopped = AtomicBool::new(true);
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut dom,
            &mut dispatcher,
            &mut context,
            event_limits(),
            &stopped
        ),
        Err(DomStyleRuntimeError::Dom(DomError::Cancelled))
    ));
    assert_eq!(context.styles.pending_sheet(0).unwrap(), Some(pending));
    assert_eq!(context.styles.waiting_resources().unwrap(), 1);
    assert_eq!(dom.snapshot().stamp(), retired_dom.stamp());
    assert_eq!(
        cssom(&context, document).revision(),
        removed_cssom.revision()
    );
    assert!(context.events.is_empty());
    match DomStyleRuntime::run_networking(
        &mut dom,
        &mut dispatcher,
        &mut context,
        event_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomStyleNetworkingProgress::TargetDestroyed { target, .. } => assert_eq!(target, style),
        other => panic!("exact canceled intention must retire on retry: {other:?}"),
    }
    assert_eq!(context.styles.waiting_resources().unwrap(), 0);
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    empty(&mut dom, &mut dispatcher, &mut context);
    assert!(context.events.is_empty());
    assert_eq!(dom.snapshot().stamp(), retired_dom.stamp());
    assert_eq!(
        cssom(&context, document).revision(),
        removed_cssom.revision()
    );
    assert_eq!(association(&historical_dom, style).unwrap(), sheet);
    assert!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert!(
        removed_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
}

#[test]
fn live_import_waits_survive_exact_retirement_and_do_not_overtake_queued_fifo() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let mut dispatcher = registry();
    let (retired, _) = detached(&mut dom, document, "@import 'retired.css';");
    connect(&mut dom, &mut context, retired, parent);
    let old_dom = dom.snapshot();
    let old_sheet = association(&old_dom, retired).unwrap();
    let old_cssom = cssom(&context, document);
    let (live_a, _) = detached(&mut dom, document, "@import 'live-a.css';");
    connect(&mut dom, &mut context, live_a, parent);
    let live_a_sheet = association(&dom.snapshot(), live_a).unwrap();
    let (queued_a, _) = detached(&mut dom, document, "a{color:red}");
    connect(&mut dom, &mut context, queued_a, parent);
    let (live_b, _) = detached(&mut dom, document, "@import 'live-b.css';");
    connect(&mut dom, &mut context, live_b, parent);
    let live_b_sheet = association(&dom.snapshot(), live_b).unwrap();
    let (queued_b, _) = detached(&mut dom, document, "b{color:blue}");
    connect(&mut dom, &mut context, queued_b, parent);
    for target in [retired, live_a, queued_a, live_b, queued_b] {
        listen(&mut dom, &mut dispatcher, target);
    }
    assert_eq!(context.styles.waiting_resources().unwrap(), 3);
    drive(&mut dom, &mut context, DomSourceEdit::detach(retired));
    drive(&mut dom, &mut context, DomSourceEdit::destroy(retired));
    let current_cssom = cssom(&context, document);
    let mut cancellation = None;
    let mut deliveries = Vec::new();
    for _ in 0..3 {
        let before_callbacks = context.events.len();
        match DomStyleRuntime::run_networking(
            &mut dom,
            &mut dispatcher,
            &mut context,
            event_limits(),
            &cancel(),
        )
        .unwrap()
        {
            DomStyleNetworkingProgress::TargetDestroyed { task, target } => {
                assert_eq!(target, retired);
                assert!(cancellation.replace(task).is_none());
                assert_eq!(context.events.len(), before_callbacks);
            }
            DomStyleNetworkingProgress::Complete { task, receipt } => {
                assert_eq!(receipt.event_type(), DomEventType::Load);
                assert!(receipt.failures().is_empty());
                deliveries.push(task);
            }
            other => panic!("live waits must not hide retired or queued work: {other:?}"),
        }
    }
    let cancellation = cancellation.expect("one exact retired intention");
    assert_eq!(deliveries.len(), 2);
    assert_ne!(deliveries[0], deliveries[1]);
    assert!(deliveries.iter().all(|task| *task != cancellation));
    assert_eq!(
        context.events,
        vec![
            (DomEventType::Load, queued_a, document),
            (DomEventType::Load, queued_b, document)
        ]
    );
    assert_eq!(context.styles.waiting_resources().unwrap(), 2);
    assert_eq!(
        context.styles.pending_sheet(0).unwrap(),
        Some((document.node(), live_a_sheet.id().clone()))
    );
    assert_eq!(
        context.styles.pending_sheet(1).unwrap(),
        Some((document.node(), live_b_sheet.id().clone()))
    );
    assert!(context.styles.pending_sheet(2).unwrap().is_none());
    empty(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(context.events.len(), 2);
    assert_eq!(association(&dom.snapshot(), live_a).unwrap(), live_a_sheet);
    assert_eq!(association(&dom.snapshot(), live_b).unwrap(), live_b_sheet);
    assert_eq!(
        cssom(&context, document).revision(),
        current_cssom.revision()
    );
    assert!(
        cssom(&context, document)
            .sheet(old_sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert_eq!(association(&old_dom, retired).unwrap(), old_sheet);
    assert!(
        old_cssom
            .sheet(old_sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
}

#[test]
fn retired_shadow_import_wait_releases_last_session_custody_only_after_reconciliation() {
    // Only the shadow root is bound. A live unrelated Document-root binding
    // would truthfully remain a separate graceful-shutdown obligation.
    let mut owner = Dom::new(
        DomLimits::new(
            256,
            200_000,
            1024,
            1024,
            1024,
            32,
            2_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_event_recipients(
            DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        )
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&host, &document, None);
    let made = owner.apply(batch, &cancel()).unwrap();
    let document = made.document(&document).unwrap();
    let host = made.created(&host).unwrap().unwrap();
    let shadow = made.created(&shadow).unwrap().unwrap();
    let mut session = DomSession::new(owner, limits()).unwrap();
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
        observations: Vec::new(),
        replacement: None,
    };
    context
        .styles
        .bind_root(
            &session.dom(),
            "only-shadow".into(),
            &mut Some(DomStyleCssomRoot::new(
                shadow,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
    let mut dispatcher = registry();
    let (target, historical_dom, historical_cssom, sheet) = {
        let mut dom = session.dom();
        let (target, _) = detached(&mut dom, document, "@import 'shadow.css';p{color:red}");
        connect(&mut dom, &mut context, target, shadow);
        listen(&mut dom, &mut dispatcher, target);
        let historical_dom = dom.snapshot();
        let sheet = association(&historical_dom, target).unwrap();
        let historical_cssom = context.styles.snapshot(shadow).unwrap().unwrap();
        assert!(matches!(
            historical_cssom
                .sheet(sheet.id())
                .unwrap()
                .authored()
                .rules()[0],
            CssRule::Import(_)
        ));
        assert_eq!(sheet.root(), shadow);
        assert_eq!(context.styles.waiting_resources().unwrap(), 1);
        drive(&mut dom, &mut context, DomSourceEdit::detach(host));
        drive(&mut dom, &mut context, DomSourceEdit::destroy(host));
        for retired in [host, shadow, target] {
            assert!(matches!(
                dom.snapshot().node(retired),
                Err(DomError::StaleNode)
            ));
        }
        assert!(context.styles.snapshot(shadow).unwrap().is_none());
        assert_eq!(
            context.styles.pending_sheet(0).unwrap(),
            Some((shadow, sheet.id().clone()))
        );
        (target, historical_dom, historical_cssom, sheet)
    };
    let mut session = match session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::CriticalResources);
            session
        }
        other => panic!("actual waiting custody survives root retirement: {other:?}"),
    };
    match DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        event_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomStyleNetworkingProgress::TargetDestroyed {
            target: retired, ..
        } => assert_eq!(retired, target),
        other => panic!("retired shadow wait must be reconciled: {other:?}"),
    }
    assert!(context.events.is_empty());
    assert_eq!(context.styles.waiting_resources().unwrap(), 0);
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    empty(&mut session.dom(), &mut dispatcher, &mut context);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
    assert_eq!(association(&historical_dom, target).unwrap(), sheet);
    assert_eq!(historical_dom.shadow_root(shadow).unwrap().host(), host);
    assert!(
        historical_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert!(matches!(
        context.styles.snapshot(shadow),
        Err(DomStyleRuntimeError::SessionUnavailable)
    ));
}

#[test]
fn already_queued_parse_recovery_load_survives_detachment_and_retains_old_objects() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "@unknown x; p { color: red }");
    connect(&mut dom, &mut context, style, parent);
    let old_dom = dom.snapshot();
    let old = association(&old_dom, style).unwrap();
    let old_cssom = cssom(&context, document);
    assert_eq!(old_cssom.sheet(old.id()).unwrap().diagnostics().len(), 1);
    assert_eq!(context.styles.waiting_resources().unwrap(), 0);
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    drive(&mut dom, &mut context, DomSourceEdit::detach(style));
    let removed = cssom(&context, document);
    assert!(removed.sheets().is_empty());
    assert!(removed.sheet(old.id()).unwrap().inputs().owner.is_none());
    assert!(association(&dom.snapshot(), style).is_none());
    delivered(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(context.events, vec![(DomEventType::Load, style, document)]);
    empty(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(association(&old_dom, style).unwrap(), old);
    assert!(old_cssom.sheet(old.id()).unwrap().inputs().owner.is_some());
    assert!(removed.sheet(old.id()).is_ok());
}

#[test]
fn queued_old_and_replacement_sheets_each_deliver_without_replacing_current_association() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, text) = detached(&mut dom, document, "p { color: red }");
    connect(&mut dom, &mut context, style, parent);
    let first_dom = dom.snapshot();
    let first = association(&first_dom, style).unwrap();
    let first_cssom = cssom(&context, document);
    drive(
        &mut dom,
        &mut context,
        DomSourceEdit::replace_data(text, 0, usize::MAX, "p { color: blue }"),
    );
    let second = association(&dom.snapshot(), style).unwrap();
    assert_ne!(first.id(), second.id());
    assert_ne!(first.association(), second.association());
    let current = cssom(&context, document);
    assert_eq!(current.sheets(), std::slice::from_ref(second.id()));
    assert!(current.sheet(first.id()).unwrap().inputs().owner.is_none());
    assert!(current.sheet(second.id()).unwrap().inputs().owner.is_some());
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    let a = delivered(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(association(&dom.snapshot(), style).unwrap(), second);
    let b = delivered(&mut dom, &mut dispatcher, &mut context);
    assert_ne!(a, b);
    assert_eq!(association(&dom.snapshot(), style).unwrap(), second);
    assert_eq!(
        context.events,
        vec![
            (DomEventType::Load, style, document),
            (DomEventType::Load, style, document)
        ]
    );
    empty(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(association(&first_dom, style).unwrap(), first);
    assert!(
        first_cssom
            .sheet(first.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
}

#[test]
fn queued_historical_load_uses_the_same_live_element_after_adoption() {
    let (mut session, mut context, document, parent, other, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "p { color: red }");
    connect(&mut dom, &mut context, style, parent);
    let old = association(&dom.snapshot(), style).unwrap();
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    drive(
        &mut dom,
        &mut context,
        DomSourceEdit::adopt(style, other.node()),
    );
    assert_eq!(
        dom.snapshot().node(style).unwrap().owner_document(),
        Some(other)
    );
    assert!(!dom.snapshot().node(style).unwrap().is_connected());
    assert!(association(&dom.snapshot(), style).is_none());
    let original_list = cssom(&context, document);
    assert!(original_list.sheets().is_empty());
    assert!(
        original_list
            .sheet(old.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert!(cssom(&context, other).sheets().is_empty());
    delivered(&mut dom, &mut dispatcher, &mut context);
    assert_eq!(context.events, vec![(DomEventType::Load, style, other)]);
    empty(&mut dom, &mut dispatcher, &mut context);
}

#[test]
fn explicit_target_destruction_skips_queued_load_without_dispatching_to_reuse() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "p { color: red }");
    connect(&mut dom, &mut context, style, parent);
    let old_dom = dom.snapshot();
    let old = association(&old_dom, style).unwrap();
    let mut dispatcher = registry();
    listen(&mut dom, &mut dispatcher, style);
    drive(&mut dom, &mut context, DomSourceEdit::detach(style));
    let mut batch = DomBatch::new();
    batch.destroy(style);
    let fresh = batch.create_element_ns(document, Namespace::html(), "style");
    let commit = dom.apply(batch, &cancel()).unwrap();
    let fresh = commit.created(&fresh).unwrap().unwrap();
    assert_ne!(fresh, style);
    match DomStyleRuntime::run_networking(
        &mut dom,
        &mut dispatcher,
        &mut context,
        event_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomStyleNetworkingProgress::TargetDestroyed { target, .. } => assert_eq!(target, style),
        other => panic!("{other:?}"),
    }
    assert!(context.events.is_empty());
    empty(&mut dom, &mut dispatcher, &mut context);
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    assert!(matches!(
        dom.snapshot().node(style),
        Err(DomError::StaleNode)
    ));
    assert_eq!(association(&old_dom, style).unwrap(), old);
    assert!(
        cssom(&context, document)
            .sheet(old.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
}

#[test]
fn missing_policy_preserves_incomplete_availability_then_exact_resume_establishes_association() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "p { color: red }");
    let old_dom = dom.snapshot();
    let old_cssom = cssom(&context, document);
    let edit = DomSourceEdit::insert_before(style, parent, None);
    let operation = {
        let DomSourceDriveProgress::Pending { request, reason } = context
            .source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap()
        else {
            panic!("missing actual policy cannot imply Allowed")
        };
        assert_eq!(
            reason,
            DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy)
        );
        let DomSourceRequest::Style(request) = request else {
            panic!("the reached Style operation must request its own inline policy")
        };
        assert_eq!(request.source().element(), style);
        assert!(matches!(
            dom.snapshot()
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet(),
            Err(DomError::IncompleteSourceOperation)
        ));
        request.source().operation()
    };
    assert!(association(&old_dom, style).is_none());
    assert!(old_cssom.sheets().is_empty());
    assert!(cssom(&context, document).sheets().is_empty());
    assert!(matches!(
        context
            .source
            .resume(
                &mut dom,
                operation,
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
    let joined = association(&dom.snapshot(), style).unwrap();
    let current = cssom(&context, document);
    assert_eq!(joined.root(), document.node());
    assert_eq!(joined.association().element(), style);
    assert_eq!(joined.revision(), current.revision());
    assert_eq!(current.sheets(), std::slice::from_ref(joined.id()));
    assert!(current.sheet(joined.id()).unwrap().inputs().owner.is_some());
    assert!(association(&old_dom, style).is_none());
    assert!(old_cssom.sheets().is_empty());
}

#[test]
fn callback_replacing_execution_access_recovers_exact_session_without_replaying_delivery() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dom = session.dom();
    let (style, _) = detached(&mut dom, document, "p { color: red }");
    connect(&mut dom, &mut context, style, parent);
    let (replacement_session, _, _, _, _, _) = fixture();
    context.replacement = Some(replacement_session.style_execution());
    let pending = context.styles.pending_sheet(0).unwrap().unwrap();
    let mut dispatcher = registry();
    dispatcher
        .add_listener(
            &mut dom,
            style,
            DomEventType::Load,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.events.push((
                    event.event_type(),
                    style,
                    dom.snapshot()
                        .node(style)
                        .unwrap()
                        .owner_document()
                        .unwrap(),
                ));
                drop(std::mem::replace(
                    &mut context.styles,
                    context.replacement.take().unwrap(),
                ));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let (task, owner) = match DomStyleRuntime::run_networking(
        &mut dom,
        &mut dispatcher,
        &mut context,
        event_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomStyleNetworkingProgress::RetainedAccess { task, owner } => (task, owner),
        other => panic!("{other:?}"),
    };
    assert_eq!(context.events, vec![(DomEventType::Load, style, document)]);
    assert!(context.styles.snapshot(document.node()).unwrap().is_none());
    assert_eq!(owner.pending_sheet(0).unwrap(), Some(pending));
    context.styles = owner;
    assert_eq!(delivered(&mut dom, &mut dispatcher, &mut context), task);
    assert_eq!(context.events.len(), 1);
    empty(&mut dom, &mut dispatcher, &mut context);
}

#[test]
fn reacquired_execution_reads_the_same_current_cssom_owner_after_external_flag_edit() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let (style, old_dom, old, current) = {
        let mut dom = session.dom();
        let (style, _) = detached(&mut dom, document, "p { color: red }");
        connect(&mut dom, &mut context, style, parent);
        let old_dom = dom.snapshot();
        let old = association(&old_dom, style).unwrap();
        let old_cssom = cssom(&context, document);
        assert!(!context.styles.disabled(&dom, style).unwrap().disabled());
        let root = dom.style_cssom_root(document.node()).unwrap().unwrap();
        let before = root.snapshot().unwrap();
        let mut batch = root.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        assert!(batch.set_disabled(old.id(), true).unwrap());
        root.commit(&mut Some(batch)).unwrap();
        let current = root.snapshot().unwrap();
        assert_ne!(current.revision(), old_cssom.revision());
        assert!(current.sheet(old.id()).unwrap().inputs().disabled);
        assert!(!old_cssom.sheet(old.id()).unwrap().inputs().disabled);
        drop(root);
        (style, old_dom, old, current)
    };
    // Execution permission is ephemeral; its replacement does not replace the
    // durable session or the exact canonical CSSOM owner.
    drop(std::mem::replace(
        &mut context.styles,
        session.style_execution(),
    ));
    let dom = session.dom();
    let observed = context.styles.disabled(&dom, style).unwrap();
    assert!(observed.disabled());
    assert_eq!(observed.stamp(), dom.snapshot().stamp());
    assert_eq!(observed.element(), style);
    assert_eq!(observed.association(), Some(old.association()));
    assert_eq!(observed.cssom_revision(), Some(current.revision()));
    assert_eq!(association(&dom.snapshot(), style).unwrap(), old);
    assert_eq!(association(&old_dom, style).unwrap(), old);
    assert_eq!(cssom(&context, document).revision(), current.revision());
}

#[test]
fn graceful_pending_preserves_custody_and_terminal_drop_does_not_finish_observed_event() {
    let (mut session, mut context, document, parent, _, _) = fixture();
    let mut dispatcher = registry();
    let (style, old_dom, sheet, old_cssom, task) = {
        let mut dom = session.dom();
        let (style, _) = detached(&mut dom, document, "p { color: red }");
        connect(&mut dom, &mut context, style, parent);
        let old_dom = dom.snapshot();
        let sheet = association(&old_dom, style).unwrap();
        let old_cssom = cssom(&context, document);
        dispatcher
            .add_listener(
                &mut dom,
                style,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, event| {
                    context.observations.push(event.clone());
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        dispatcher
            .add_listener(
                &mut dom,
                style,
                DomEventType::Load,
                listener(),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let mut one_callback = event_limits();
        one_callback.callbacks = 1;
        let task = match DomStyleRuntime::run_networking(
            &mut dom,
            &mut dispatcher,
            &mut context,
            one_callback,
            &cancel(),
        )
        .unwrap()
        {
            DomStyleNetworkingProgress::Retained { task, .. } => task,
            other => panic!("one actual callback must leave the second pending: {other:?}"),
        };
        assert_eq!(context.observations.len(), 1);
        assert!(context.events.is_empty());
        assert_eq!(context.observations[0].event_type(), DomEventType::Load);
        assert_eq!(context.observations[0].target(), Some(style));
        (style, old_dom, sheet, old_cssom, task)
    };
    let session = match session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::NetworkingTask(task));
            session
        }
        DomShutdownOutcome::Complete => {
            panic!("pending actual dispatch cannot complete shutdown")
        }
    };
    assert_eq!(session.snapshot().stamp(), old_dom.stamp());
    assert_eq!(association(&session.snapshot(), style).unwrap(), sheet);
    assert_eq!(cssom(&context, document).revision(), old_cssom.revision());
    // A retained public event observation does not own the private dispatch
    // continuation, live DOM or Style session. Terminal disposal abandons them.
    drop(session);
    assert!(matches!(
        context.styles.snapshot(document.node()),
        Err(DomStyleRuntimeError::SessionUnavailable)
    ));
    assert_eq!(context.observations[0].current_target(), None);
    assert!(context.observations[0].composed_path().next().is_none());
    assert!(context.events.is_empty());
    assert_eq!(association(&old_dom, style).unwrap(), sheet);
    assert_eq!(old_cssom.sheets(), std::slice::from_ref(sheet.id()));
    assert_eq!(
        old_cssom
            .sheet(sheet.id())
            .unwrap()
            .authored()
            .rules()
            .len(),
        1
    );
}
