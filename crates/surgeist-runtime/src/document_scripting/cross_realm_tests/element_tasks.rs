//! Actual element tasks delivered through the document's transferred registry.

use super::*;
use crate::{
    DomElementTask, DomElementTaskProgress, DomElementTaskRunLimits, DomElementTaskRuntime,
    DomElementTaskSourceKind,
};

pub(super) fn verify_attached_element_tasks() {
    attached_registry_runs_the_original_queued_details_toggle();
    attached_registry_runs_the_original_dialog_toggle_and_close_tasks();
    attached_element_task_keeps_the_original_terminal_refusal();
    attached_element_task_skips_the_exact_retired_target();
    attached_element_task_refuses_a_foreign_window_before_dispatch();
}

struct AttachedDetails {
    host: DocumentScriptHost,
    details: NodeId,
    tasks: DomElementTaskRuntime,
    observations: Rc<RefCell<Vec<NativeDomEvent>>>,
}

fn attached_details_host(cancel: Arc<AtomicBool>) -> AttachedDetails {
    let (mut session, target, details) = hosted_element(&cancel, "details");
    let tasks = session.element_task_execution();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let native = observations.clone();
    registry
        .add_listener(
            &mut session.dom(),
            details,
            DomEventType::TOGGLE,
            DomEventCallback::new(move |_, _, _, event| {
                native.borrow_mut().push(event.clone());
                Ok(())
            }),
            Default::default(),
        )
        .unwrap();
    let mut registry = Some(registry);
    let mut host = super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel.clone(),
        1_000,
        20_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    super::guest::tests::expose_public_node_for_source_test(&mut host, &permit, details, "details");
    AttachedDetails {
        host,
        details,
        tasks,
        observations,
    }
}

fn attached_registry_runs_the_original_queued_details_toggle() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedDetails {
        mut host,
        details,
        mut tasks,
        observations,
    } = attached_details_host(cancel.clone());
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            globalThis.taskSentinel = {};
            globalThis.taskChecks = false;
            details.addEventListener("toggle", event => {
                globalThis.originalTaskEvent = event;
                throw taskSentinel;
            }, {once: true});
            details.addEventListener("toggle", function (event) {
                taskChecks = this === details && event === originalTaskEvent
                    && event === globalThis.event && event.target === details
                    && event.currentTarget === details && event instanceof ToggleEvent
                    && event.isTrusted && !event.cancelable && !event.bubbles
                    && event.oldState === "closed" && event.newState === "open"
                    && event.source === null && event.target.ownerDocument === document;
                event.preventDefault();
            }, {once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let source = DomElementTaskSourceKind::DomManipulation;
    let original = queue_attached_details_toggle(&mut host, &mut tasks, details, &cancel);
    assert_eq!(original.id().target(), details);
    let limits = DomElementTaskRunLimits {
        dispatch: EVENT.dispatch,
        source: EVENT.source,
    };
    let mut bounded = limits;
    bounded.dispatch.callbacks = 2;
    let progress = host
        .run_attached_element_task(&permit, source, bounded)
        .unwrap();
    assert!(
        matches!(
            progress,
            DomElementTaskProgress::Retained {
                task,
                dispatch: crate::DomEventDispatchOutcome::Retained(crate::DomEventResource::Callbacks),
                displaced_owner: None,
            } if task.id() == original.id()
        ),
        "the original task must retain after two callbacks: {progress:?}"
    );
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        original.id()
    );
    assert_eq!(observations.borrow().len(), 1);
    let result = host
        .run_attached_element_task(&permit, source, limits)
        .unwrap();
    let DomElementTaskProgress::Complete {
        task,
        receipt,
        dialog_toggle_disposition: None,
        displaced_owner: None,
    } = result
    else {
        panic!("queued Details must complete its exact original task: {result:?}");
    };
    assert_eq!(task.id(), original.id());
    assert_eq!(receipt.callbacks(), 3);
    assert_eq!(receipt.failures().len(), 1);
    assert!(!receipt.default_prevented());
    {
        let observed = observations.borrow();
        let [native] = observed.as_slice() else {
            panic!("the original Rust listener must run once");
        };
        let data = native.toggle_event().unwrap();
        assert_eq!(
            (data.old_state(), data.new_state(), data.source()),
            (
                "closed".encode_utf16().collect::<Vec<_>>().as_slice(),
                "open".encode_utf16().collect::<Vec<_>>().as_slice(),
                None
            )
        );
        assert!(native.current_target().is_none());
    }
    assert!(matches!(
        host.run_attached_element_task(&permit, source, limits)
            .unwrap(),
        DomElementTaskProgress::Empty,
    ));
    let exception = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    host.in_document(&permit, |scope| {
        let observed = scope.import(exception.exception.value())?;
        let expected = evaluate(scope, "taskSentinel")?;
        assert!(scope.strict_equals(&observed, &expected)?);
        let observed = evaluate(scope, "taskChecks && originalTaskEvent.currentTarget === null && !originalTaskEvent.defaultPrevented")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&observed, &expected)?);
        Ok(())
    }).unwrap();
}

fn queue_attached_details_toggle(
    host: &mut DocumentScriptHost,
    tasks: &mut DomElementTaskRuntime,
    details: NodeId,
    cancel: &AtomicBool,
) -> DomElementTask {
    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.set_attribute(details, "open", "");
        dom.apply(edit, cancel).unwrap();
        tasks.ingest(dom, cancel).unwrap();
    })
    .unwrap();
    tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap()
}

fn attached_element_task_keeps_the_original_terminal_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedDetails {
        mut host,
        details,
        mut tasks,
        observations,
    } = attached_details_host(cancel.clone());
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            let first;
            details.addEventListener("toggle", event => {
                first = event;
                throw { original: true };
            }, {once: true});
            details.addEventListener("toggle", event => {
                if (event === first && event === globalThis.event) detachIf();
            }, {once: true});
            details.addEventListener("toggle", () => { throw { replayed: true }; },
                {once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let task = queue_attached_details_toggle(&mut host, &mut tasks, details, &cancel);
    let limits = DomElementTaskRunLimits {
        dispatch: EVENT.dispatch,
        source: EVENT.source,
    };
    let source = task.source();
    let window_event = host.engine.as_ref().unwrap().window_event.clone();
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        first,
    )))) = host.run_attached_element_task(&permit, source, limits)
    else {
        panic!("the actual queued callback must return its host refusal");
    };
    let cause = first.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(cause, DocumentScriptNativeError::SourceAccess));
    assert!(window_event.is_idle());
    {
        let observed = observations.borrow();
        let [native] = observed.as_slice() else {
            panic!("the original Rust callback must run once");
        };
        assert!(native.current_target().is_none());
        assert_eq!(native.composed_path().count(), 0);
    }
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        retry,
    )))) = host.run_attached_element_task(&permit, source, limits)
    else {
        panic!("retry must return the same original task refusal");
    };
    assert!(std::ptr::eq(
        cause,
        retry.cause::<DocumentScriptNativeError>().unwrap()
    ));
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_some()
    );
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        task.id()
    );
    let session = host.stop().into_session().unwrap();
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        1
    );
}

fn attached_element_task_skips_the_exact_retired_target() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedDetails {
        mut host,
        details,
        mut tasks,
        observations,
    } = attached_details_host(cancel.clone());
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(scope, "globalThis.retiredCalled = false; details.addEventListener('toggle', () => { retiredCalled = true; })").map(drop)
    }).unwrap();
    let task = queue_attached_details_toggle(&mut host, &mut tasks, details, &cancel);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(details);
        batch.destroy(details);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let result = host
        .run_attached_element_task(
            &permit,
            task.source(),
            DomElementTaskRunLimits {
                dispatch: EVENT.dispatch,
                source: EVENT.source,
            },
        )
        .unwrap();
    assert!(matches!(
        result, DomElementTaskProgress::SkippedTargetDestroyed(skipped) if skipped.id() == task.id()
    ));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(observations.borrow().is_empty());
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, "retiredCalled")?;
        let expected = scope.primitive(Primitive::Boolean(false))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    })
    .unwrap();
}

fn attached_element_task_refuses_a_foreign_window_before_dispatch() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 8, 4);
    let other = session
        .hosted_execution()
        .create_top_level(&mut session.dom(), target.host(), &cancel)
        .unwrap();
    let details = insert_hosted_element(&mut session, other, "details", &cancel);
    let mut tasks = session.element_task_execution();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let calls = Rc::new(Cell::new(0));
    let native = calls.clone();
    registry
        .add_listener(
            &mut session.dom(),
            details,
            DomEventType::TOGGLE,
            DomEventCallback::new(move |_, _, _, _| {
                native.set(native.get() + 1);
                Ok(())
            }),
            Default::default(),
        )
        .unwrap();
    let mut registry = Some(registry);
    let mut host = super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel.clone(),
        1_000,
        20_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    let foreign = queue_attached_details_toggle(&mut host, &mut tasks, details, &cancel);
    assert_eq!(
        foreign.queue_association().unwrap().global(),
        other.global()
    );
    let source = foreign.source();
    let limits = DomElementTaskRunLimits {
        dispatch: EVENT.dispatch,
        source: EVENT.source,
    };
    let result = host.run_attached_element_task(&permit, source, limits);
    assert!(matches!(
        result,
        Err(DocumentScriptEventError::Task(
            crate::DomElementTaskRuntimeError::Binding
        ))
    ));
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        foreign.id()
    );
    assert_eq!(calls.get(), 0);
}

fn attached_registry_runs_the_original_dialog_toggle_and_close_tasks() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target, dialog) = hosted_element(&cancel, "dialog");
    let tasks = session.element_task_execution();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(registry);
    let mut host = super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel.clone(),
        1_000,
        20_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    super::guest::tests::expose_public_node_for_source_test(&mut host, &permit, dialog, "dialog");
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            globalThis.deliveredTasks = [];
            dialog.addEventListener("toggle", function (event) {
                globalThis.dialogToggle = event;
                deliveredTasks.push(this === dialog && event.target === dialog
                    && event.currentTarget === dialog && event === globalThis.event
                    && event instanceof ToggleEvent && event.isTrusted && !event.cancelable
                    && event.oldState === "open" && event.newState === "closed"
                    && event.source === null && event.target.ownerDocument === document);
            });
            dialog.addEventListener("close", function (event) {
                globalThis.dialogClose = event;
                deliveredTasks.push(this === dialog && event.target === dialog
                    && event.currentTarget === dialog && event === globalThis.event
                    && Object.getPrototypeOf(event) === Event.prototype
                    && event.isTrusted && !event.cancelable && !event.bubbles);
            });
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let beforetoggle = host
        .with_native(|dom, _, mut source| {
            assert!(matches!(
                source
                    .apply(
                        dom,
                        DomSourceEdit::set_attribute(dialog, "open", ""),
                        SOURCE,
                        &mut services,
                        &cancel
                    )
                    .unwrap(),
                DomSourceDriveProgress::Complete(_)
            ));
            let progress = source
                .apply(
                    dom,
                    DomSourceEdit::close_dialog(dialog, Some("closed".into())),
                    SOURCE,
                    &mut services,
                    &cancel,
                )
                .unwrap();
            let DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(request),
                reason: DomSourceUnavailable::Event,
            } = progress
            else {
                panic!("actual native Close must reach beforetoggle: {progress:?}");
            };
            assert_eq!(
                request.kind(),
                surgeist_dom::DomSourceEventKind::BeforeToggle
            );
            assert!(!request.cancelable());
            request
        })
        .unwrap();
    assert!(matches!(
        host.service_attached_source_event(&permit, &beforetoggle, EVENT).unwrap(),
        DomSourceEventServiceProgress::Serviced { receipt } if receipt.callbacks() == 0
    ));
    host.with_native(|dom, _, mut source| {
        let progress = source
            .resume(
                dom,
                beforetoggle.operation(),
                SOURCE,
                &mut services,
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(progress, DomSourceDriveProgress::Complete(_)),
            "{progress:?}"
        );
    })
    .unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    let toggle_source = DomElementTaskSourceKind::DomManipulation;
    let close_source = DomElementTaskSourceKind::UserInteraction;
    let toggle = tasks.pending_task(toggle_source, 0).unwrap().unwrap();
    let close = tasks.pending_task(close_source, 0).unwrap().unwrap();
    assert!(toggle.dialog_toggle().is_some());
    assert!(close.dialog_close().is_some());
    let limits = DomElementTaskRunLimits {
        dispatch: EVENT.dispatch,
        source: EVENT.source,
    };
    let toggled = host
        .run_attached_element_task(&permit, toggle_source, limits)
        .unwrap();
    assert!(matches!(
        toggled,
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition: Some(surgeist_dom::DomDialogToggleTaskDisposition::TrackerCleared),
            displaced_owner: None,
        } if task.id() == toggle.id() && receipt.callbacks() == 1 && receipt.failures().is_empty()
    ));
    let closed = host
        .run_attached_element_task(&permit, close_source, limits)
        .unwrap();
    assert!(matches!(
        closed,
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition: None,
            displaced_owner: None,
        } if task.id() == close.id() && receipt.callbacks() == 1 && receipt.failures().is_empty()
    ));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let actual = evaluate(scope, "deliveredTasks.length === 2 && deliveredTasks.every(Boolean) && dialogToggle !== dialogClose && dialogToggle.currentTarget === null && dialogClose.currentTarget === null")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).unwrap();
}
