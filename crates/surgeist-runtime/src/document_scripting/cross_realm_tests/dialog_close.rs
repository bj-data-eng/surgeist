//! Reentry and terminal admission during an attached native Dialog close.

use super::*;
use crate::{
    document_scripting::guest::tests::{
        attach_public_nodes_with_registry_and_reports, expose_public_node_for_source_test,
    },
    internal::element_tasks::DomElementTaskRuntime,
};
use surgeist_dom::source::DomSourcePhase;
use surgeist_html::values::DocumentString;

struct AttachedClose {
    host: DocumentScriptHost,
    target: DomHostedTopLevel,
    dialog: NodeId,
    tasks: DomElementTaskRuntime,
    observed: Rc<RefCell<Vec<NativeDomEvent>>>,
}

fn setup(report_slots: usize) -> AttachedClose {
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
    let observed = Rc::new(RefCell::new(Vec::new()));
    let native_observed = observed.clone();
    registry
        .add_listener(
            &mut session.dom(),
            target.document().node(),
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::new(move |_, _, _, event| {
                assert_eq!(event.target().unwrap().node(), Some(dialog));
                assert!(!event.cancelable());
                native_observed.borrow_mut().push(event.clone());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut registry = Some(registry);
    let mut host = attach_public_nodes_with_registry_and_reports(
        session,
        target,
        cancel,
        200_000,
        2_000_000,
        &mut registry,
        report_slots,
    );
    assert!(registry.is_none());
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, dialog, "closeSubject");
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "closeSubject.returnValue = 'prior'; closeSubject.open = true;",
        )
        .map(drop)
    })
    .unwrap();
    assert!(
        host.snapshot()
            .unwrap()
            .node(dialog)
            .unwrap()
            .is_connected()
    );
    assert!(
        host.snapshot()
            .unwrap()
            .node(dialog)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    AttachedClose {
        host,
        target,
        dialog,
        tasks,
        observed,
    }
}

pub(super) fn adoption_preserves_the_original_dialog_dispatch() {
    let AttachedClose {
        mut host,
        target,
        dialog,
        tasks,
        observed,
    } = setup(4);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"
            (() => {
                const subject = closeSubject;
                const destination = document.implementation.createHTMLDocument();
                let original, adopted, next;
                subject.addEventListener('beforetoggle', function (event) {
                    original = event;
                    adopted = this === subject && subject.ownerDocument === document &&
                        destination.adoptNode(subject) === subject;
                }, { once: true });
                subject.addEventListener('beforetoggle', function (event) {
                    next = adopted && event === original && event === globalThis.event &&
                        event.isTrusted && this === subject && event.currentTarget === subject &&
                        event.target === subject && subject.ownerDocument === destination &&
                        subject.open;
                    subject.open = false;
                }, { once: true });
                const originalCompleted = subject.close('must not be written') === undefined &&
                    next && original.currentTarget === null && globalThis.event === undefined &&
                    !subject.open && subject.returnValue === 'prior';

                const alreadyAdopted = document.createElement('dialog');
                alreadyAdopted.returnValue = 'in the destination';
                alreadyAdopted.open = true;
                destination.adoptNode(alreadyAdopted);
                let destinationEvent;
                alreadyAdopted.addEventListener('beforetoggle', function (event) {
                    destinationEvent = event;
                    if (this === alreadyAdopted && event.isTrusted && event === globalThis.event &&
                        event.target === alreadyAdopted && this.ownerDocument === destination) {
                        this.open = false;
                    }
                }, { once: true });
                return originalCompleted && alreadyAdopted.close('also unwritten') === undefined &&
                    destinationEvent.currentTarget === null && globalThis.event === undefined &&
                    !alreadyAdopted.open && alreadyAdopted.returnValue === 'in the destination';
            })()
        "#,
        )?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(dialog).unwrap();
    assert_ne!(node.owner_document().unwrap(), target.document());
    assert!(!node.is_connected());
    assert_eq!(node.attribute(&"open".into()), None);
    assert_eq!(
        node.as_dialog().unwrap().return_value(),
        &DocumentString::from("prior")
    );
    assert!(!node.as_dialog().unwrap().has_close_watcher());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let observed = observed.borrow();
    let [event] = observed.as_slice() else {
        panic!("one original beforetoggle must finish after adoption");
    };
    assert_eq!(event.current_target(), None);
    assert_eq!(event.composed_path().count(), 0);
    let mut session = host.stop().into_session().unwrap();
    assert!(session.dom().pending_source_edit().is_none());
}

pub(super) fn listener_report_refusal_retains_the_original_close() {
    let AttachedClose {
        mut host,
        dialog,
        tasks,
        observed,
        ..
    } = setup(1);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            closeSubject.addEventListener('beforetoggle', function (event) {
                if (globalThis.event === event) throw { original: true };
            }, { once: true });
            closeSubject.addEventListener('beforetoggle', function () {
                this.setAttribute('data-unexpected-callback', 'entered');
            }, { once: true });
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let window_event = host.engine.as_ref().unwrap().window_event.clone();
    let Err(DocumentScriptError::Script(Error::HostFailure(first))) = host
        .in_document(&permit, |scope| {
            evaluate(scope, "closeSubject.close('unpublished')").map(drop)
        })
    else {
        panic!("the second listener must return the typed report-slot refusal");
    };
    let cause = first.cause::<DocumentGuestEventError>().unwrap();
    assert!(matches!(
        cause,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::StorageLimit
        ))
    ));
    let Err(DocumentScriptError::Script(Error::HostFailure(again))) = host.checkpoint() else {
        panic!("the original refusal must survive event cleanup");
    };
    assert!(std::ptr::eq(
        cause,
        again.cause::<DocumentGuestEventError>().unwrap()
    ));
    assert!(window_event.is_idle());
    assert!(host.native.attached_event_request().is_none());
    let observed = observed.borrow();
    let [event] = observed.as_slice() else {
        panic!("the native listener must observe the original event exactly once");
    };
    assert_eq!(event.current_target(), None);
    assert_eq!(event.composed_path().count(), 0);
    let report = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(report.realm, permit.realm);
    assert_eq!(
        report.exception.phase(),
        surgeist_bindings_js::ExceptionPhase::Invoke
    );
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    let mut session = host.stop().into_session().unwrap();
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.phase(), DomSourcePhase::EventObservation);
    assert_eq!(status.parent_operation(), None);
    let snapshot = session.snapshot();
    let node = snapshot.node(dialog).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"data-unexpected-callback".into()), None);
    let dialog = node.as_dialog().unwrap();
    assert_eq!(dialog.return_value(), &DocumentString::from("prior"));
    assert!(dialog.has_close_watcher());
    assert!(!dialog.has_toggle_task_tracker());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
}
