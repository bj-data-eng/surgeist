//! Attached listeners observe the original Select task's real effects.

use super::{window_length::observe, *};
use surgeist_dom::style::{DomStyleInlineCspObservation, DomStyleSource};
use surgeist_runtime::element_task::{DomElementTaskProgress, DomElementTaskRunLimits};

fn block_style(source: &DomStyleSource) -> DomStyleInlineCspObservation {
    DomStyleInlineCspObservation::evaluated(
        source,
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}

struct AttachedSelect {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    select: NodeId,
    content: NodeId,
    original: NodeId,
}

fn task_limits() -> DomElementTaskRunLimits {
    DomElementTaskRunLimits {
        dispatch: DomEventDispatchLimits {
            path_items: 32,
            snapshot_listeners: 32,
            failure_records: 16,
            callbacks: 32,
            work: 100_000,
        },
        source: DomSourceLimits::new(32_000_000, 2_000_000, 1_000_000, 64, 1024, 1024, 4),
    }
}

fn attach_content(tag: &str, text: &str, active_type: &str) -> AttachedSelect {
    let cancellation = Arc::new(cancel());
    let mut dom = native_dom();
    dom.set_limits(
        dom.limits()
            .with_select_processing(16, 4_000_000, 1_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let (_, top) = create(&mut session);
    let document = top.document();
    let snapshot = session.snapshot();
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = *snapshot.node(root).unwrap().children().last().unwrap();
    let mut setup = DomBatch::new();
    let select = setup.create_element_ns(document, Namespace::html(), "select");
    let option = setup.create_element_ns(document, Namespace::html(), "option");
    let content = setup.create_element_ns(document, Namespace::html(), "selectedcontent");
    let original = setup.create_element_ns(document, Namespace::html(), tag);
    let text = setup.create_text(document, text);
    setup.set_attribute(&select, "id", "subject");
    setup.set_attribute(&select, "multiple", "");
    setup.set_attribute(&option, "id", "chosen");
    setup.set_attribute(&option, "label", "live label");
    setup.set_attribute(&original, "type", "text/plain");
    setup.insert_before(&text, &original, None);
    setup.insert_before(&original, &option, None);
    setup.insert_before(&option, &select, None);
    setup.insert_before(&content, &select, None);
    setup.insert_before(&select, body, None);
    let setup = session.dom().apply(setup, &cancellation).unwrap();
    let select = setup.created(&select).unwrap().unwrap();
    let option = setup.created(&option).unwrap().unwrap();
    let content = setup.created(&content).unwrap().unwrap();
    let original = setup.created(&original).unwrap().unwrap();

    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation.clone(), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let registered = observe(
        &mut host,
        &permit,
        r#"(() => {
            const select = document.getElementById('subject');
            globalThis.selectEvents = [];
            for (const type of ['input', 'change']) {
                document.body.addEventListener(type, event => {
                    selectEvents.push(
                        Object.getPrototypeOf(event) === Event.prototype &&
                        event.target === select && event.currentTarget === document.body &&
                        event.isTrusted && event.bubbles && !event.cancelable &&
                        event.composed === (type === 'input') ? type : 'wrong event'
                    );
                });
            }
            return 'ok';
        })()"#,
    );
    assert_eq!(registered, "ok");
    let source_limits = task_limits().source;
    let observed = host.snapshot().unwrap().stamp();
    host.with_native(|dom, _, mut source| {
        for edit in [
            DomSourceEdit::toggle_select_option_for_user(select, option, observed),
            DomSourceEdit::remove_attribute(select, "multiple"),
            DomSourceEdit::set_attribute(original, "type", active_type),
        ] {
            let result = source
                .apply(
                    dom,
                    edit,
                    source_limits,
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |value| {
                            assert_eq!(value.element(), original);
                            Some(block_style(value))
                        },
                        url: None,
                    },
                    &cancellation,
                )
                .unwrap();
            assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
        }
    })
    .unwrap();
    AttachedSelect {
        host,
        permit,
        select,
        content,
        original,
    }
}

pub(super) fn attached_select_serves_current_content_before_trusted_events() {
    let AttachedSelect {
        mut host,
        permit,
        select,
        content,
        original,
    } = attach_content("style", "a { color: red }", "text/css");
    let mut copied = None;
    let result = host
        .run_attached_element_task_with_source_services(
            &permit,
            DomElementTaskSourceKind::UserInteraction,
            task_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |source| {
                    assert_ne!(source.element(), original);
                    assert!(copied.replace(source.element()).is_none());
                    Some(block_style(source))
                },
                url: None,
            },
        )
        .unwrap();
    let DomElementTaskProgress::SelectNotificationComplete {
        task,
        input,
        change,
        ..
    } = result
    else {
        panic!("attached continuation must finish after its own Style observation: {result:?}")
    };
    assert_eq!(task.id().target(), select);
    assert_eq!(input.callbacks(), 1);
    assert_eq!(change.unwrap().callbacks(), 1);
    assert_eq!(
        host.snapshot().unwrap().node(content).unwrap().children(),
        &[copied.unwrap()]
    );
    let notified = observe(
        &mut host,
        &permit,
        "selectEvents.join() === 'input,change' ? 'ok' : selectEvents.join()",
    );
    assert_eq!(notified, "ok");
}

pub(super) fn attached_select_serves_actual_script_settings_and_cancellation() {
    let AttachedSelect {
        mut host,
        permit,
        select,
        content,
        original,
    } = attach_content(
        "script",
        "globalThis.unexpectedScript = true",
        "text/javascript",
    );
    let interaction = DomElementTaskSourceKind::UserInteraction;
    let snapshot = host.snapshot().unwrap();
    let reached = |host: &mut DocumentScriptHost, step| {
        let progress = host
            .run_attached_element_task(&permit, interaction, task_limits())
            .unwrap();
        let DomElementTaskProgress::SelectNotificationSourcePending { task, source, .. } = progress
        else {
            panic!("the original task must retain its Script operation: {progress:?}")
        };
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Script(request),
            reason: DomSourceUnavailable::Script(reason),
        } = *source
        else {
            panic!("the exact prepared Script needs a real producer")
        };
        assert_eq!(task.id().target(), select);
        assert_eq!(request.step(), step);
        assert_eq!(reason, step);
        assert_ne!(request.subject(), Some(original));
        assert_eq!(
            request.relevant_global(),
            snapshot.relevant_global(select).unwrap()
        );
        (task.id(), request)
    };
    let (task, settings) = reached(&mut host, DomSourceScriptStep::ReadScripting);
    assert!(
        host.service_script_settings(&permit, settings, task_limits().source)
            .unwrap()
            .unwrap()
            .enabled()
    );
    let (continued, executable) = reached(&mut host, DomSourceScriptStep::ExecutablePreparation);
    assert_eq!(continued, task);
    assert_eq!(executable.operation(), settings.operation());
    assert_eq!(executable.subject(), settings.subject());
    host.with_native(|dom, _, source| {
        source
            .cancel_script_preparation(dom, executable.operation(), task_limits().source, &cancel())
            .unwrap();
    })
    .unwrap();
    let settled = host
        .run_attached_element_task(&permit, interaction, task_limits())
        .unwrap();
    let DomElementTaskProgress::SelectNotificationSourceCancelled {
        task: settled_task,
        source,
        ..
    } = settled
    else {
        panic!("cancellation must settle the actual task without dispatch: {settled:?}")
    };
    assert_eq!(settled_task.id(), task);
    assert!(matches!(
        *source,
        DomSourceDriveProgress::Complete(ref done)
            if done.operation() == executable.operation() && done.cancellation_requested()
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(content).unwrap().children(),
        after.node(content).unwrap().children()
    );
    let notified = observe(
        &mut host,
        &permit,
        "selectEvents.length === 0 && globalThis.unexpectedScript === undefined ? 'ok' : 'unexpected delivery'",
    );
    assert_eq!(notified, "ok");
}

pub(super) fn attached_select_schedules_copied_reactions_before_listener_microtasks() {
    let AttachedSelect {
        mut host,
        permit,
        select,
        ..
    } = attach_content("span", "", "text/plain");
    let ready = observe(
        &mut host,
        &permit,
        r#"(() => {
            globalThis.SelectProbe = class extends HTMLElement {
                constructor() {
                    super();
                    selectEvents.push('upgrade');
                }
                connectedCallback() { selectEvents.push('connected'); }
            };
            customElements.define('select-probe', SelectProbe);
            const original = document.createElement('select-probe');
            document.getElementById('chosen').appendChild(original);
            if (selectEvents.join() !== 'upgrade,connected') return selectEvents.join();
            selectEvents.length = 0;
            document.getElementById('subject').addEventListener('input', () => {
                Promise.resolve().then(() => selectEvents.push('listener job'));
            });
            return 'ok';
        })()"#,
    );
    assert_eq!(ready, "ok");
    let delivered = host
        .run_attached_element_task(
            &permit,
            DomElementTaskSourceKind::UserInteraction,
            task_limits(),
        )
        .unwrap();
    assert!(matches!(
        delivered,
        DomElementTaskProgress::SelectNotificationComplete { task, .. } if task.id().target() == select
    ));
    assert_eq!(
        observe(&mut host, &permit, "selectEvents.join()"),
        "input,change"
    );
    host.checkpoint().unwrap();
    assert_eq!(
        observe(&mut host, &permit, "selectEvents.join()"),
        "input,change,upgrade,connected,listener job"
    );
    host.checkpoint().unwrap();
    assert_eq!(
        observe(&mut host, &permit, "selectEvents.join()"),
        "input,change,upgrade,connected,listener job"
    );
}
