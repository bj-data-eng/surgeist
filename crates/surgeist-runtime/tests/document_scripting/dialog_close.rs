//! Attached guest Dialog Close runs the original synchronous native Source.

use super::*;

pub(super) fn attached_close_drives_beforetoggle_and_resumes_the_original_source() {
    let cancellation = Arc::new(cancel());
    let mut policy = limits();
    policy.source_events.jobs = 8;
    let mut session = session(policy);
    let (_, top) = create(&mut session);
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
                storage_bytes: ATTACHED_GUEST_STORAGE,
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
            const descriptor = Object.getOwnPropertyDescriptor(HTMLDialogElement.prototype, 'close');
            if (typeof descriptor?.value !== 'function') {
                return 'missing HTMLDialogElement.prototype.close method';
            }
            const close = descriptor.value;
            if (close.name !== 'close' || close.length !== 0 ||
                !descriptor.writable || !descriptor.enumerable || !descriptor.configurable) {
                return 'Dialog close method descriptor';
            }
            let coerced = false;
            try {
                close.call(document.createElement('div'), {toString() { coerced = true; return 'bad'; }});
                return 'close accepted a non-Dialog receiver';
            } catch (error) {
                if (!(error instanceof TypeError) || coerced) return 'close brand check before coercion';
            }

            function opened(prior) {
                const dialog = document.body.appendChild(document.createElement('dialog'));
                dialog.returnValue = prior;
                dialog.open = true;
                return dialog;
            }
            globalThis.closeProbe = {
                cases: ['prior omitted', 'prior undefined', 'prior null', 'prior empty'].map(opened),
                outer: opened('outer prior'),
                inner: opened('inner prior'),
                sentinel: {},
            };
            return 'ok';
        })()"#,
    );

    let opened = host.snapshot().unwrap();
    let root = opened
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = opened.node(root).unwrap().children()[1];
    let dialogs: [NodeId; 6] = opened
        .node(body)
        .unwrap()
        .children()
        .try_into()
        .expect("six connected test Dialogs");
    for node in dialogs {
        let dialog = opened.node(node).unwrap();
        assert!(dialog.is_connected());
        assert_eq!(dialog.attribute(&"open".into()), Some(&"".into()));
        let view = dialog.as_dialog().unwrap();
        assert_eq!(view.close_watcher().unwrap().global(), top.global());
        assert!(!view.has_toggle_task_tracker());
    }
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);

    observe(
        &mut host,
        &permit,
        r#"(() => {
            const {cases: [omitted, undefinedArg, nullArg, emptyArg], outer, inner, sentinel} = closeProbe;
            if ([omitted.close(), undefinedArg.close(undefined), nullArg.close(null), emptyArg.close('')]
                    .some(result => result !== undefined)) return 'Dialog close completion value';
            if ([omitted.returnValue, undefinedArg.returnValue, nullArg.returnValue, emptyArg.returnValue]
                    .some((value, index) => value !== ['prior omitted', 'prior undefined', 'null', ''][index])) {
                return 'optional close DOMString distinguishes absence, null, and empty';
            }

            const order = [];
            let outerEvent, outerFacts, innerFacts, nestedRestored, laterFacts;
            function beforeClose(event, dialog) {
                event.preventDefault();
                return event instanceof ToggleEvent && event.isTrusted && !event.cancelable &&
                    !event.defaultPrevented && event.oldState === 'open' && event.newState === 'closed' &&
                    event.source === null && event.target === dialog && event.currentTarget === dialog &&
                    globalThis.event === event;
            }
            inner.addEventListener('beforetoggle', function (event) {
                order.push('inner');
                innerFacts = this === inner && event !== outerEvent && beforeClose(event, inner) &&
                    inner.open && inner.returnValue === 'inner prior';
            });
            outer.addEventListener('beforetoggle', function (event) {
                order.push('outer first');
                outerEvent = event;
                outerFacts = this === outer && beforeClose(event, outer) && outer.open &&
                    outer.returnValue === 'outer prior';
                nestedRestored = inner.close('nested') === undefined && globalThis.event === event &&
                    !inner.open && inner.returnValue === 'nested';
                outer.open = false;
                throw sentinel;
            }, {once: true});
            outer.addEventListener('beforetoggle', function (event) {
                order.push('outer later');
                laterFacts = this === outer && event === outerEvent && globalThis.event === event &&
                    !this.open && this.returnValue === 'outer prior';
            });
            if (outer.close('must not be written') !== undefined) return 'outer close completion value';
            if (order.join(',') !== 'outer first,inner,outer later' || !outerFacts || !innerFacts ||
                !nestedRestored || !laterFacts || globalThis.event !== undefined) {
                return 'synchronous trusted beforetoggle and nested Source event restoration';
            }
            return !outer.open && outer.returnValue === 'outer prior'
                ? 'ok' : 'listener open removal selects the original Close early return';
        })()"#,
    );

    let report = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .expect("the ordinary outer listener exception was reported");
    host.in_document(&permit, |scope| {
        let original = scope.evaluate(&"closeProbe.sentinel".encode_utf16().collect::<Vec<_>>())?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&reported, &original)?);
        Ok(())
    })
    .unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );

    let completed = host.snapshot().unwrap();
    for (node, expected) in dialogs.into_iter().zip([
        "prior omitted",
        "prior undefined",
        "null",
        "",
        "outer prior",
        "nested",
    ]) {
        let element = completed.node(node).unwrap();
        assert_eq!(element.attribute(&"open".into()), None);
        let dialog = element.as_dialog().unwrap();
        assert_eq!(dialog.return_value(), &DocumentString::from(expected));
        assert!(!dialog.has_close_watcher());
    }
    assert!(
        !completed
            .node(dialogs[4])
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_toggle_task_tracker()
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        5
    );
    assert_eq!(
        tasks
            .pending_tasks(DomElementTaskSourceKind::UserInteraction)
            .unwrap(),
        5
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
