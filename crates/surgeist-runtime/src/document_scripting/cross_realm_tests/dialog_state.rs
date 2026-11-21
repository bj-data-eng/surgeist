//! Native modal state and original Source refusal through attached accessors.

use super::*;
use crate::{
    document_scripting::guest::tests::{
        attach_public_nodes, attach_public_nodes_with_registry, expose_public_node_for_source_test,
    },
    internal::{
        dom_source::DomSourceDriveError,
        element_tasks::{DomElementTaskProgress, DomElementTaskRunLimits},
    },
};
use surgeist_dom::{
    dialog::DomDialogToggleTaskDisposition,
    task::DomElementTaskSourceKind::{DomManipulation, UserInteraction},
};

pub(super) fn modal_dialog_accessors_preserve_the_original_queued_toggle() {
    let cancel = Arc::new(AtomicBool::new(false));
    let dom = Dom::new(scripting_dom_limits(200_000, 0)).unwrap();
    let mut session = scripting_session(dom, 4);
    let mut hosted = session.hosted_execution();
    let owner = hosted
        .create_headless_host(&mut session.dom(), &cancel)
        .unwrap();
    let target = hosted
        .create_top_level(&mut session.dom(), owner, &cancel)
        .unwrap();
    let document = target.document();
    let initial = session.snapshot();
    let html = initial
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = initial.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&subject, body, None);
    let subject = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    // A real empty, inert Dialog in the sealed host completes focusing
    // through the existing no-change branch.
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_native_attribute(subject, "inert", ""),
                SOURCE,
                &mut services,
                &cancel,
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));

    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::new(|_, _, _: &mut (), event| {
                assert!(event.cancelable());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
                passive: None,
            },
        )
        .unwrap();
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
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
        panic!("modal formation must reach its actual beforetoggle: {progress:?}");
    };
    assert_eq!(request.subject(), subject);
    assert!(matches!(
        session.source_execution().service_event(
            &mut session.dom(), &request, &mut registry, &mut (), EVENT, &cancel
        ).unwrap(),
        DomSourceEventServiceProgress::Serviced { receipt }
            if receipt.callbacks() == 1 && receipt.failures().is_empty() && !receipt.default_prevented()
    ));
    let done = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            SOURCE,
            &mut services,
            &cancel,
        )
        .unwrap();
    assert!(
        matches!(done, DomSourceDriveProgress::Complete(_)),
        "{done:?}"
    );
    assert!(session.dom().pending_source_edit().is_none());
    let shown = session.snapshot();
    let node = shown.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"closedby".into()), None);
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().unwrap();
    assert_eq!(watcher.global(), target.global());
    assert_eq!(watcher.subject(), subject);
    let layer = shown.document(document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let saved_task = tasks.pending_task(DomManipulation, 0).unwrap().unwrap();
    let tracker = dialog.toggle_task_tracker().unwrap();
    assert_eq!(saved_task.dialog_toggle().unwrap().id(), tracker.task());

    let mut registry = Some(registry);
    let mut host = attach_public_nodes_with_registry(
        session,
        target,
        cancel,
        200_000,
        2_000_000,
        &mut registry,
    );
    assert!(registry.is_none());
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, subject, "modalProbe");
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const dialog = modalProbe;
                if (!(dialog instanceof HTMLDialogElement) || !dialog.open ||
                    dialog.hasAttributeNS(null, 'closedby') || dialog.closedBy !== 'closerequest') {
                    return 'missing closedby in a live modal';
                }
                for (const [raw, expected] of [['', 'closerequest'], ['bad\ud800', 'closerequest'],
                    ['AnY', 'any'], ['CLOSEREQUEST', 'closerequest'], ['NoNe', 'none']]) {
                    dialog.closedBy = raw;
                    if (dialog.closedBy !== expected || dialog.getAttributeNS(null, 'closedby') !== raw) {
                        return 'modal closedBy state and raw content';
                    }
                }
                dialog.removeAttributeNS(null, 'closedby');
                globalThis.observedModalEvents = [];
                dialog.addEventListener('close', () => observedModalEvents.push('close'));
                dialog.addEventListener('toggle', event => {
                    observedModalEvents.push([event.target === dialog, event.oldState, event.newState, event.source]);
                });
                dialog.open = false;
                return !dialog.open && !dialog.hasAttributeNS(null, 'open') &&
                    dialog.closedBy === 'closerequest' && observedModalEvents.length === 0
                    ? 'ok' : 'direct removal preserves the modal Auto state';
            })()
        "#)?;
        assert_eq!(String::from_utf16(&scope.to_utf16(&result)?).unwrap(), "ok");
        Ok(())
    }).unwrap();
    let after = host.snapshot().unwrap();
    let node = after.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), None);
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    assert_eq!(dialog.close_watcher(), None);
    assert_eq!(dialog.toggle_task_tracker(), Some(tracker));
    let layer = after.document(document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert_eq!(
        tasks.pending_task(DomManipulation, 0).unwrap(),
        Some(saved_task)
    );
    assert_eq!(tasks.pending_task(UserInteraction, 0).unwrap(), None);
    let delivered = host
        .run_attached_element_task(
            &permit,
            DomManipulation,
            DomElementTaskRunLimits {
                dispatch: EVENT.dispatch,
                source: EVENT.source,
            },
        )
        .unwrap();
    assert!(matches!(
        delivered,
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition: Some(DomDialogToggleTaskDisposition::TrackerCleared),
            displaced_owner: None,
        } if task.id() == saved_task.id() && receipt.callbacks() == 1 && receipt.failures().is_empty()
    ));
    host.in_document(&permit, |scope| {
        let value = evaluate(
            scope,
            r#"JSON.stringify(observedModalEvents) === '[[true,"closed","open",null]]'"#,
        )?;
        assert!(scope.boolean(&value)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

pub(super) fn dialog_edits_preserve_native_state_on_source_refusal() {
    for script in [
        "limitedDialog.open = false",
        "limitedDialog.returnValue = 'changed'",
        "limitedDialog.closedBy = 'any'",
        "limitedDialog.close('changed')",
    ] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let subject = batch.create_element_ns(target.document(), Namespace::html(), "dialog");
        batch.set_attribute_ns(&subject, Namespace::none(), "open", "authored");
        batch.set_attribute_ns(&subject, Namespace::none(), "closedby", "NoNe");
        batch.set_dialog_return_value(&subject, "saved");
        let subject = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&subject)
            .unwrap()
            .unwrap();
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, subject, "limitedDialog");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("Dialog Source work refusal for {script}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DomSourceDriveError>(),
            Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
                DomError::SourceWorkLimit { limit: 0 }
            )))
        ));
        let after = host.snapshot().unwrap();
        assert_eq!(after.stamp(), before.stamp());
        let node = after.node(subject).unwrap();
        assert_eq!(node.as_dialog().unwrap().return_value(), &"saved".into());
        assert_eq!(node.attribute(&"open".into()), Some(&"authored".into()));
        assert_eq!(node.attribute(&"closedby".into()), Some(&"NoNe".into()));
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}
