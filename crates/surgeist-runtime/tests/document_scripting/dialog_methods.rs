//! The attached Dialog methods run the original native Source and task owners.

use super::*;
use surgeist_html::elements::dialog::HtmlDialogState::{Closed, Open};

pub(super) fn attached_dialog_methods_use_original_native_events_and_tasks() {
    let cancellation = Arc::new(cancel());
    let mut policy = limits();
    policy.source_events.jobs = 8;
    let mut session = session(policy);
    let mut hosted = session.hosted_execution();
    let headless = hosted
        .create_headless_host(&mut session.dom(), &cancellation)
        .unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), headless, &cancellation)
        .unwrap();
    let tasks = session.element_task_execution();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 8,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry.bind(&mut session.dom(), &cancellation).unwrap();
    let mut registry = Some(registry);
    let mut host = prepare(session, cancellation)
        .enable_with_guest_event_registry(
            top,
            ALLOW,
            hooks(&hook()),
            &mut registry,
            DocumentGuestEventLimits {
                storage_bytes: 640 * 1024,
                work: 200_000,
                pending_exception_reports: 8,
                node_retirement: DomNodeRetirementLimits::new(16_000_000, 1_000),
                source_edits: DomSourceLimits::new(
                    32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8,
                ),
                dispatch: DomEventDispatchLimits {
                    path_items: 16,
                    snapshot_listeners: 16,
                    failure_records: 8,
                    callbacks: 16,
                    work: 100_000,
                },
            },
        )
        .unwrap();
    assert!(registry.is_none());
    let permit = host.permit().unwrap();

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const prototype = HTMLDialogElement.prototype;
        const names = ['show', 'showModal', 'close', 'requestClose'];
        for (const name of names) {
            const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
            if (typeof descriptor?.value !== 'function') {
                return 'missing HTMLDialogElement.prototype.' + name + ' method';
            }
            if (descriptor.value.name !== name || descriptor.value.length !== 0 ||
                !descriptor.writable || !descriptor.enumerable || !descriptor.configurable) {
                return name + ' method descriptor';
            }
        }
        if (Object.getOwnPropertyNames(prototype).filter(name => names.includes(name)).join() !== names.join()) {
            return 'Dialog method declaration order';
        }
        let coercions = 0;
        const excess = {toString() { coercions++; throw new Error('coerced'); }};
        for (const name of names) {
            for (const invalid of [document.body, Object.create(prototype)]) {
                try {
                    prototype[name].call(invalid, excess);
                    return name + ' accepted a non-Dialog receiver';
                } catch (error) {
                    if (!(error instanceof TypeError) || coercions !== 0) return name + ' brand before coercion';
                }
            }
        }

        const detached = document.createElement('dialog');
        let detachedEvents = 0;
        detached.addEventListener('beforetoggle', () => detachedEvents++);
        try {
            detached.showModal(excess);
            return 'detached showModal returned';
        } catch (error) {
            if (!(error instanceof DOMException) || error.name !== 'InvalidStateError' ||
                detached.open || detachedEvents !== 0 || coercions !== 0) return 'detached showModal precedence';
        }

        const dialogs = [];
        for (const [open, opposite, closedBy] of [
            ['show', 'showModal', 'none'], ['showModal', 'show', 'closerequest'],
        ]) {
            const dialog = document.body.appendChild(document.createElement('dialog'));
            dialog.setAttribute('inert', '');
            const events = [];
            let correct = true;
            function beforeOpen(event) {
                correct &&= this === dialog && event instanceof ToggleEvent && event.isTrusted &&
                    event.cancelable && event.target === dialog && event.currentTarget === dialog &&
                    event.oldState === 'closed' && event.newState === 'open' && event.source === null &&
                    !dialog.open && globalThis.event === event;
                events.push(event);
                if (events.length === 1) event.preventDefault();
            }
            dialog.addEventListener('beforetoggle', beforeOpen);
            if (dialog[open](excess) !== undefined || dialog.open || events.length !== 1 ||
                !events[0].defaultPrevented) return open + ' cancelable opening';
            if (dialog[open](excess) !== undefined || !dialog.open || dialog.closedBy !== closedBy ||
                dialog.getAttribute('open') !== '' || events.length !== 2 || events[1].defaultPrevented ||
                !correct || events[0] === events[1] || events.some(event => event.currentTarget !== null)) {
                return open + ' actual native open';
            }
            if (dialog[open](excess) !== undefined || events.length !== 2) return open + ' same-mode return';
            try {
                dialog[opposite](excess);
                return opposite + ' returned in the opposite mode';
            } catch (error) {
                if (!(error instanceof DOMException) || error.name !== 'InvalidStateError' ||
                    events.length !== 2 || !dialog.open || coercions !== 0) return opposite + ' opposite-mode precedence';
            }
            dialog.removeEventListener('beforetoggle', beforeOpen);
            dialogs.push(dialog);
        }
        globalThis.dialogMethodProbes = dialogs;
        return 'ok';
    })()"#,
    );

    let shown = host.snapshot().unwrap();
    let document = shown.document(top.document()).unwrap();
    let html = document.document_element().unwrap();
    let body = shown.node(html).unwrap().children()[1];
    let [nonmodal, modal]: [NodeId; 2] = shown.node(body).unwrap().children().try_into().unwrap();
    let layer = document.top_layer().unwrap();
    assert_eq!(layer.raw_members(), [modal]);
    assert!(layer.pending_removals().is_empty());
    let [nonmodal_state, modal_state] = [nonmodal, modal].map(|id| {
        let node = shown.node(id).unwrap();
        assert!(node.is_connected());
        assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
        let dialog = node.as_dialog().unwrap();
        let watcher = dialog.close_watcher().unwrap();
        assert_eq!(watcher.subject(), id);
        assert_eq!(watcher.global(), top.global());
        dialog
    });
    assert!(!nonmodal_state.is_modal());
    assert!(modal_state.is_modal());
    let [nonmodal_toggle, modal_toggle] = [0, 1].map(|index| {
        let task = tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, index)
            .unwrap()
            .unwrap();
        let toggle = task.dialog_toggle().unwrap();
        let association = task.queue_association().unwrap();
        assert_eq!(association.global(), top.global());
        assert_eq!(association.document(), top.document());
        assert_eq!(toggle.old_state(), Closed);
        assert_eq!(toggle.new_state(), Open);
        assert_eq!(toggle.source(), None);
        task
    });
    assert_eq!(
        nonmodal_toggle.id(),
        nonmodal_state.toggle_task_tracker().unwrap().task()
    );
    assert_eq!(
        modal_toggle.id(),
        modal_state.toggle_task_tracker().unwrap().task()
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const [dialog] = dialogMethodProbes;
        dialog.returnValue = 'prior';
        const trace = [];
        let correct = true;
        function onCancel(event) {
            correct &&= this === dialog && event.constructor === Event && event.isTrusted &&
                event.cancelable && !event.bubbles && !event.composed && event.target === dialog &&
                event.currentTarget === dialog && dialog.open && globalThis.event === event;
            trace.push(event);
            if (trace.length === 1) event.preventDefault();
        }
        function onBeforeClose(event) {
            event.preventDefault();
            correct &&= this === dialog && event instanceof ToggleEvent && event.isTrusted &&
                !event.cancelable && !event.defaultPrevented && event.target === dialog &&
                event.currentTarget === dialog && event.oldState === 'open' && event.newState === 'closed' &&
                event.source === null && dialog.open && dialog.returnValue === 'prior' && globalThis.event === event;
            trace.push(event);
        }
        dialog.addEventListener('cancel', onCancel);
        dialog.addEventListener('beforetoggle', onBeforeClose);
        globalThis.requestProbe = {trace, onCancel, onBeforeClose, correct: () => correct};
        return dialog.requestClose('prevented') === undefined && dialog.open &&
            dialog.returnValue === 'prior' && trace.length === 1 && correct &&
            trace[0].defaultPrevented && trace[0].currentTarget === null
            ? 'ok' : 'prevented original cancel keeps the Dialog open';
    })()"#,
    );

    let canceled = host.snapshot().unwrap();
    let canceled_dialog = canceled.node(nonmodal).unwrap().as_dialog().unwrap();
    assert_eq!(
        canceled_dialog.close_watcher(),
        nonmodal_state.close_watcher()
    );
    assert_eq!(
        canceled_dialog.toggle_task_tracker(),
        nonmodal_state.toggle_task_tracker()
    );
    assert_eq!(
        canceled_dialog.request_close_return_value(),
        Some(&"prevented".into())
    );
    assert_eq!(canceled_dialog.request_close_source(), None);
    assert!(!canceled_dialog.force_close_watcher_enabled());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    assert_eq!(canceled.incomplete_source_operation(), None);

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const [dialog, modal] = dialogMethodProbes;
        const {trace, onCancel, onBeforeClose, correct} = requestProbe;
        let receiver = dialog;
        let conversions = 0;
        const argument = {toString() {
            conversions++;
            dialog.setAttribute('data-conversion', 'ran');
            receiver = modal;
            return 'exact\ud800';
        }};
        if (receiver.requestClose(argument) !== undefined ||
            conversions !== 1 || dialog.open || dialog.returnValue !== 'exact\ud800' ||
            dialog.getAttribute('data-conversion') !== 'ran' || !modal.open || modal.returnValue !== '') {
            return 'requestClose uses the original receiver after reentrant conversion';
        }
        if (!correct() || trace.map(event => event.type).join() !== 'cancel,cancel,beforetoggle' ||
            new Set(trace).size !== 3 || trace.some(event => event.currentTarget !== null) ||
            trace[1].defaultPrevented || globalThis.event !== undefined) {
            return 'original trusted Cancel then nested Close beforetoggle';
        }
        dialog.removeEventListener('cancel', onCancel);
        dialog.removeEventListener('beforetoggle', onBeforeClose);
        return 'ok';
    })()"#,
    );

    let closed = host.snapshot().unwrap();
    let node = closed.node(nonmodal).unwrap();
    assert_eq!(node.attribute(&"open".into()), None);
    let dialog = node.as_dialog().unwrap();
    assert_eq!(
        dialog.return_value(),
        &DocumentString::from_utf16("exact".encode_utf16().chain([0xd800]).collect::<Vec<_>>())
    );
    assert!(!dialog.has_close_watcher());
    assert!(!dialog.force_close_watcher_enabled());
    assert_eq!(dialog.request_close_return_value(), None);
    assert_eq!(dialog.request_close_source(), None);
    assert_eq!(closed.incomplete_source_operation(), None);
    assert_eq!(
        closed
            .node(modal)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        modal_state.close_watcher()
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        2
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        1
    );
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(modal_toggle)
    );
    let replacement = (0..2)
        .map(|index| {
            tasks
                .pending_task(DomElementTaskSourceKind::DomManipulation, index)
                .unwrap()
                .unwrap()
        })
        .find(|task| task.id().target() == nonmodal)
        .unwrap();
    assert_ne!(replacement.id(), nonmodal_toggle.id());
    assert_eq!(
        replacement.id(),
        dialog.toggle_task_tracker().unwrap().task()
    );
    assert_eq!(
        replacement.queue_association(),
        nonmodal_toggle.queue_association()
    );
    let toggle = replacement.dialog_toggle().unwrap();
    assert_eq!(toggle.old_state(), Closed);
    assert_eq!(toggle.new_state(), Closed);
    assert_eq!(toggle.source(), None);
    let close = tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_eq!(close.dialog_close().unwrap().id().target(), nonmodal);
    assert_eq!(
        close.queue_association(),
        nonmodal_toggle.queue_association()
    );

    observe(
        &mut host,
        &permit,
        r#"(() => {
        const [dialog] = dialogMethodProbes;
        for (const [argumentsList, result] of [[[], 'prior'], [[undefined], 'prior'], [[null], 'null'], [[''], '']]) {
            dialog.returnValue = 'prior';
            dialog.open = true;
            if (dialog.requestClose(...argumentsList) !== undefined || dialog.open || dialog.returnValue !== result) {
                return 'optional requestClose DOMString';
            }
        }
        const failure = {};
        dialog.returnValue = 'unchanged';
        dialog.open = true;
        try {
            dialog.requestClose({toString() { throw failure; }});
            return 'requestClose ignored abrupt conversion';
        } catch (error) {
            if (error !== failure || !dialog.open || dialog.returnValue !== 'unchanged') return 'abrupt requestClose conversion';
        }
        dialog.open = false;
        return 'ok';
    })()"#,
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn observe(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit, script: &str) {
    let result = host
        .in_document(permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
