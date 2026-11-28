//! Native HTML Select user action, Source publication, and real notification delivery.

use super::*;
use surgeist_dom::{
    DomSnapshot,
    identity::{DocumentId, NodeId},
    source::DomSourceEdit,
    task::DomElementTaskSourceKind,
    tree::DomNodeKind,
};
use surgeist_html::values::DocumentString;
use surgeist_runtime::{
    element_task::{
        DomElementTaskEventContext, DomElementTaskProgress, DomElementTaskRunLimits,
        DomElementTaskRuntime,
    },
    event::{
        DomEventCallback, DomEventDispatchLimits, DomEventDispatcher, DomEventListenerOptions,
        DomEventRegistryLimits, DomEventType,
    },
    source::DomSourceDriveError,
};

struct Fixture {
    session: DomSession,
    document: DocumentId,
    select: NodeId,
    option: NodeId,
}

fn fixture(tasks: usize, cancel: &AtomicBool) -> Fixture {
    let mut limits = session_limits();
    limits.element_tasks.tasks = tasks;
    let dom = Dom::new(
        DomLimits::new(
            128,
            32_768,
            256,
            64,
            64,
            16,
            1_000_000,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::event::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        )
        .with_select_processing(32, 4_000_000, 10_000_000)
        .unwrap()
        .with_meta_processing_work(1_000_000),
    )
    .unwrap();
    let mut session = DomSession::new(dom, limits).unwrap();
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), cancel).unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, cancel)
        .unwrap();
    let document = top.document();
    let snapshot = session.snapshot();
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let select = batch.create_element_ns(document, Namespace::html(), "select");
    let option = batch.create_element_ns(document, Namespace::html(), "option");
    batch.set_attribute(&select, "multiple", "");
    batch.set_attribute(&option, "label", "original label");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&select, body, None);
    let done = session.dom().apply(batch, cancel).unwrap();
    Fixture {
        session,
        document,
        select: done.created(&select).unwrap().unwrap(),
        option: done.created(&option).unwrap().unwrap(),
    }
}

fn limits() -> DomElementTaskRunLimits {
    DomElementTaskRunLimits {
        source: DomSourceLimits::new(32_000_000, 2_000_000, 1_000_000, 64, 1024, 1024, 4),
        dispatch: DomEventDispatchLimits {
            path_items: 128,
            snapshot_listeners: 128,
            failure_records: 32,
            callbacks: 32,
            work: 100_000,
        },
    }
}

fn no_services<'a>(
    style_policy: &'a mut dyn FnMut(&DomStyleSource) -> Option<DomStyleInlineCspObservation>,
) -> DomSourceServices<'a> {
    DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy,
        url: None,
    }
}

fn selected(snapshot: &DomSnapshot, option: NodeId) -> (bool, bool) {
    let flags = snapshot
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags();
    (flags.selectedness(), flags.dirtiness())
}

fn fallback(snapshot: &DomSnapshot, select: NodeId) -> NodeId {
    let root = snapshot.node(select).unwrap().hosted_shadow_root().unwrap();
    let slot = snapshot.node(root).unwrap().children()[0];
    snapshot.node(slot).unwrap().children()[0]
}

struct Context {
    tasks: DomElementTaskRuntime,
    notifications: Vec<DomEventType>,
    retire_on_input: bool,
}

impl DomElementTaskEventContext for Context {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.tasks
    }
}

fn listeners(
    session: &mut DomSession,
    select: NodeId,
    label: &'static str,
) -> DomEventDispatcher<Context> {
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    for event_type in [DomEventType::INPUT, DomEventType::CHANGE] {
        registry
            .add_listener(
                &mut session.dom(),
                select,
                event_type.clone(),
                DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                    assert!(event.is_trusted());
                    assert_eq!(event.target(), Some(select.into()));
                    assert_eq!(event.current_target(), Some(select.into()));
                    assert!(event.bubbles());
                    assert!(!event.cancelable());
                    assert_eq!(event.composed(), event_type == DomEventType::INPUT);
                    let snapshot = dom.snapshot();
                    let node = snapshot.node(select).unwrap().as_select().unwrap();
                    assert!(node.state().flags().user_validity());
                    let div = snapshot.node(fallback(&snapshot, select)).unwrap();
                    let [text] = div.children() else {
                        panic!("one actual label Text")
                    };
                    assert!(matches!(
                        snapshot.node(*text).unwrap().kind(),
                        DomNodeKind::Text(value) if value == &DocumentString::from(label)
                    ));
                    context.notifications.push(event_type.clone());
                    if context.retire_on_input && event_type == DomEventType::INPUT {
                        let mut batch = DomBatch::new();
                        batch.detach(select);
                        batch.destroy(select);
                        dom.apply(batch, &AtomicBool::new(false)).unwrap();
                    }
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    registry
}

#[test]
fn multiple_select_queue_refusal_keeps_the_original_toggle_and_current_label() {
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut session,
        document,
        select,
        option,
    } = fixture(0, &cancel);
    let mut source = session.source_execution();
    let before = session.snapshot();
    let fallback_id = fallback(&before, select);
    let edit = DomSourceEdit::toggle_select_option_for_user(select, option, before.stamp());
    assert!(matches!(
        source.apply(
            &mut session.dom(),
            edit,
            limits().source,
            &mut no_services(&mut |_| None),
            &cancel
        ),
        Err(DomSourceDriveError::Source(
            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Admission)
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(selected(&session.snapshot(), option), (false, false));
    let mut tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let operation = session.dom().pending_source_edit().unwrap().operation();
    tasks.set_limits(session_limits().element_tasks).unwrap();
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                operation,
                limits().source,
                &mut no_services(&mut |_| None),
                &cancel
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let stamped = session.snapshot();
    assert_eq!(selected(&stamped, option), (true, true));
    assert!(
        !stamped
            .node(select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .flags()
            .user_validity()
    );
    let interaction = DomElementTaskSourceKind::UserInteraction;
    let queued = tasks.pending_task(interaction, 0).unwrap().unwrap();
    assert_eq!(queued.id().target(), select);
    assert_eq!(queued.queue_association().unwrap().document(), document);

    let mut updated = DomBatch::new();
    updated.set_attribute(option, "label", "current label");
    updated.remove_attribute(select, "multiple");
    session.dom().apply(updated, &cancel).unwrap();
    let mut registry = listeners(&mut session, select, "current label");
    let mut context = Context {
        tasks,
        notifications: Vec::new(),
        retire_on_input: false,
    };
    let mut one_callback = limits();
    one_callback.dispatch.callbacks = 1;
    let retained = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        one_callback,
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::Retained { task, .. } = retained else {
        panic!("change's callback must retain the original task: {retained:?}");
    };
    assert_eq!(task.id(), queued.id());
    assert_eq!(context.notifications, [DomEventType::INPUT]);
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        limits(),
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::SelectNotificationComplete {
        task,
        input,
        change,
        ..
    } = result
    else {
        panic!("real native Select notification completes: {result:?}")
    };
    assert_eq!(task.id(), queued.id());
    assert_eq!(input.callbacks(), 1);
    assert_eq!(change.unwrap().callbacks(), 1);
    assert_eq!(
        context.notifications,
        [DomEventType::INPUT, DomEventType::CHANGE]
    );
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 0);
    assert_eq!(fallback(&session.snapshot(), select), fallback_id);
    assert!(before.node(fallback_id).unwrap().children().is_empty());
}

#[test]
fn retiring_select_in_input_preserves_the_real_receipt() {
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut session,
        select,
        option,
        ..
    } = fixture(8, &cancel);
    let observed = session.snapshot().stamp();
    let edit = DomSourceEdit::toggle_select_option_for_user(select, option, observed);
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                edit,
                limits().source,
                &mut no_services(&mut |_| None),
                &cancel
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let mut registry = listeners(&mut session, select, "original label");
    let mut context = Context {
        tasks: session.element_task_execution(),
        notifications: Vec::new(),
        retire_on_input: true,
    };
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::UserInteraction,
        &mut registry,
        &mut context,
        limits(),
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::SelectNotificationComplete { input, change, .. } = result else {
        panic!("retirement preserves the input receipt: {result:?}")
    };
    assert_eq!(input.callbacks(), 1);
    assert!(change.is_none());
    assert_eq!(context.notifications, [DomEventType::INPUT]);
    assert!(matches!(
        session.snapshot().node(select),
        Err(DomError::StaleNode)
    ));
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 0);
}

#[test]
fn current_selectedcontent_resumes_its_original_style_request_before_dispatch() {
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut session,
        document,
        select,
        option,
    } = fixture(8, &cancel);
    let mut setup = DomBatch::new();
    let content = setup.create_element_ns(document, Namespace::html(), "selectedcontent");
    let style = setup.create_element_ns(document, Namespace::html(), "style");
    let css = setup.create_text(document, "a { color: red }");
    setup.set_attribute(&style, "type", "text/plain");
    setup.insert_before(&css, &style, None);
    setup.insert_before(&style, option, None);
    setup.insert_before(&content, select, None);
    let setup = session.dom().apply(setup, &cancel).unwrap();
    let content = setup.created(&content).unwrap().unwrap();
    let style = setup.created(&style).unwrap().unwrap();
    let before = session.snapshot();
    assert!(before.node(content).unwrap().children().is_empty());
    let mut source = session.source_execution();
    let toggle = DomSourceEdit::toggle_select_option_for_user(select, option, before.stamp());
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                toggle,
                limits().source,
                &mut no_services(&mut |_| None),
                &cancel
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let mut single = DomBatch::new();
    single.remove_attribute(select, "multiple");
    session.dom().apply(single, &cancel).unwrap();

    // Only the original Style becomes eligible; its copy must make its own
    // separately qualified CSP request inside the queued notification.
    let mut original_policy = |value: &DomStyleSource| {
        assert_eq!(value.element(), style);
        Some(DomStyleInlineCspObservation::evaluated(
            value,
            std::sync::Arc::new(vec!["style-src 'none'".into()]),
            19,
            false,
        ))
    };
    assert!(matches!(
        source
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(style, "type", "text/css"),
                limits().source,
                &mut no_services(&mut original_policy),
                &cancel,
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    let mut registry = listeners(&mut session, select, "original label");
    let mut context = Context {
        tasks: session.element_task_execution(),
        notifications: Vec::new(),
        retire_on_input: false,
    };
    let interaction = DomElementTaskSourceKind::UserInteraction;
    let queued = context.tasks.pending_task(interaction, 0).unwrap().unwrap();
    let parked = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        limits(),
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::SelectNotificationSourcePending { task, source, .. } = parked
    else {
        panic!("the current selectedcontent reaches its copied Style policy: {parked:?}")
    };
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Style(request),
        reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
    } = *source
    else {
        panic!("the actual inline Style observation is outstanding")
    };
    assert_eq!(task.id(), queued.id());
    assert_ne!(request.source().element(), style);
    assert_eq!(
        session.snapshot().node(content).unwrap().children(),
        &[request.source().element()]
    );
    assert!(context.notifications.is_empty());
    assert!(
        session
            .snapshot()
            .node(fallback(&session.snapshot(), select))
            .unwrap()
            .children()
            .is_empty()
    );

    let mut evaluations = 0;
    let mut policy = |value: &DomStyleSource| {
        assert_eq!(value.operation(), request.source().operation());
        assert_eq!(value.ordinal(), request.source().ordinal());
        assert_eq!(value.element(), request.source().element());
        evaluations += 1;
        Some(DomStyleInlineCspObservation::evaluated(
            value,
            std::sync::Arc::new(vec!["style-src 'none'".into()]),
            19,
            false,
        ))
    };
    let result = DomElementTaskRuntime::run_next_with_source_services(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        limits(),
        &mut no_services(&mut policy),
        &cancel,
    )
    .unwrap();
    assert!(matches!(
        result,
        DomElementTaskProgress::SelectNotificationComplete { task, change: Some(_), .. }
            if task.id() == queued.id()
    ));
    assert_eq!(evaluations, 1);
    assert_eq!(
        context.notifications,
        [DomEventType::INPUT, DomEventType::CHANGE]
    );
    assert!(session.dom().pending_source_edit().is_none());
}

#[test]
fn selectedcontent_preserves_its_joined_style_preference_before_events() {
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut session,
        document,
        select,
        option,
    } = fixture(8, &cancel);
    session
        .style_execution()
        .bind_root(
            &session.dom(),
            "select-preference".into(),
            &mut Some(surgeist_dom::style::DomStyleCssomRoot::new(
                document.node(),
                surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel,
        )
        .unwrap();
    let mut batch = DomBatch::new();
    let content = batch.create_element_ns(document, Namespace::html(), "selectedcontent");
    let meta = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&meta, "content", "Selection preference");
    batch.insert_before(&meta, option, None);
    batch.insert_before(&content, select, None);
    let done = session.dom().apply(batch, &cancel).unwrap();
    let meta = done.created(&meta).unwrap().unwrap();
    let content = done.created(&content).unwrap().unwrap();
    let observed = session.snapshot().stamp();
    let mut source = session.source_execution();
    for edit in [
        DomSourceEdit::toggle_select_option_for_user(select, option, observed),
        DomSourceEdit::remove_attribute(select, "multiple"),
        DomSourceEdit::set_attribute(meta, "http-equiv", "default-style"),
    ] {
        let progress = source
            .apply(
                &mut session.dom(),
                edit,
                limits().source,
                &mut no_services(&mut |_| None),
                &cancel,
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    }
    let mut registry = listeners(&mut session, select, "original label");
    let mut context = Context {
        tasks: session.element_task_execution(),
        notifications: Vec::new(),
        retire_on_input: false,
    };
    let interaction = DomElementTaskSourceKind::UserInteraction;
    let result = DomElementTaskRuntime::run_next_with_source_services(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        limits(),
        &mut DomSourceServices {
            meta: Some(
                surgeist_dom::meta::DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128)
                    .unwrap(),
            ),
            ..no_services(&mut |_| None)
        },
        &cancel,
    )
    .unwrap();
    let DomElementTaskProgress::SelectNotificationSourcePending { task, source, .. } = result
    else {
        panic!("yield the original joined Source publication: {result:?}")
    };
    let DomSourceDriveProgress::MetaSource(outcome) = *source else {
        panic!("the actual stylesheet preference must remain observable")
    };
    assert!(matches!(
        outcome.source(),
        surgeist_dom::source::DomSourceProgress::Complete(_)
    ));
    assert_eq!(outcome.roots().len(), 1);
    assert_eq!(outcome.preferences().len(), 1);
    assert_eq!(
        outcome.preferences()[0].observation().preferred_name(),
        &DocumentString::from("Selection preference")
    );
    assert_ne!(
        session.snapshot().node(content).unwrap().children(),
        &[meta]
    );
    assert!(context.notifications.is_empty());
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        interaction,
        &mut registry,
        &mut context,
        limits(),
        &cancel,
    )
    .unwrap();
    assert!(matches!(
        result,
        DomElementTaskProgress::SelectNotificationComplete { task: complete, .. }
            if complete.id() == task.id()
    ));
    assert_eq!(
        context.notifications,
        [DomEventType::INPUT, DomEventType::CHANGE]
    );
    assert!(session.dom().pending_source_edit().is_none());
}

#[test]
fn user_toggle_refuses_stale_foreign_disabled_and_nonmember_observations() {
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut session,
        document,
        select,
        option,
    } = fixture(8, &cancel);
    let foreign = fixture(8, &cancel);
    let old = session.snapshot();
    let mut edit = DomBatch::new();
    let group = edit.create_element_ns(document, Namespace::html(), "optgroup");
    edit.set_attribute(&group, "disabled", "");
    edit.insert_before(&group, select, None);
    edit.insert_before(option, &group, None);
    let published = session.dom().apply(edit, &cancel).unwrap();
    let group = published.created(&group).unwrap().unwrap();
    let reject = |session: &mut DomSession, option, observed, expected| {
        let before = session.snapshot();
        let result = session.source_execution().apply(
            &mut session.dom(),
            DomSourceEdit::toggle_select_option_for_user(select, option, observed),
            limits().source,
            &mut no_services(&mut |_| None),
            &cancel,
        );
        let Err(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(error))) = result else {
            panic!("unqualified user action must refuse: {result:?}");
        };
        assert_eq!(error, expected);
        assert_eq!(session.snapshot().stamp(), before.stamp());
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    };
    let current = session.snapshot().stamp();
    reject(
        &mut session,
        option,
        old.stamp(),
        DomError::StaleSelectUserAction,
    );
    reject(
        &mut session,
        option,
        foreign.session.snapshot().stamp(),
        DomError::StaleSelectUserAction,
    );
    reject(
        &mut session,
        foreign.option,
        current,
        DomError::ForeignOwner,
    );
    reject(
        &mut session,
        option,
        current,
        DomError::SelectUserActionInapplicable,
    );
    assert_eq!(selected(&session.snapshot(), option), (false, false));

    let mut edit = DomBatch::new();
    edit.remove_attribute(group, "disabled");
    edit.set_attribute(select, "disabled", "");
    session.dom().apply(edit, &cancel).unwrap();
    let current = session.snapshot().stamp();
    reject(
        &mut session,
        option,
        current,
        DomError::SelectUserActionInapplicable,
    );

    let mut edit = DomBatch::new();
    edit.remove_attribute(select, "disabled");
    edit.detach(option);
    session.dom().apply(edit, &cancel).unwrap();
    let current = session.snapshot().stamp();
    reject(
        &mut session,
        option,
        current,
        DomError::SelectUserActionInapplicable,
    );
    assert_eq!(selected(&session.snapshot(), option), (false, false));

    let mut edit = DomBatch::new();
    edit.insert_before(option, group, None);
    edit.set_option_selected(option, true);
    session.dom().apply(edit, &cancel).unwrap();
    assert_eq!(selected(&session.snapshot(), option), (true, true));
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
}
