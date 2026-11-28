//! The guest sees a native Show pause at its authentic focusing request.

use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_dom::source::DomSourcePhase;
use surgeist_dom::task::DomElementTaskSourceKind::DomManipulation;
use surgeist_html::elements::dialog::{HtmlDialogFocusControl, HtmlDialogFocusingAction};

pub(super) fn show_retains_the_reached_native_focusing_request() {
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
    let subject = insert_hosted_element(&mut session, target, "dialog", &cancel);
    let tasks = session.element_task_execution();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, subject, "pendingShow");
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            pendingShow.addEventListener('beforetoggle', function (event) {
                if (event.isTrusted && event.cancelable && event.oldState === 'closed' &&
                    event.newState === 'open' && event === globalThis.event && event.target === this) {
                    const count = Number(this.getAttribute('data-event-count') ?? '0');
                    this.setAttribute('data-event-count', String(count + 1));
                }
            });
        "#).map(drop)
    }).unwrap();
    let first = host.in_document(&permit, |scope| {
        evaluate(scope, "pendingShow.show()").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = first else {
        panic!("native Show must report its reached focus requirement: {first:?}");
    };
    let Some(DocumentGuestEventError::SourcePending(operation)) =
        failure.cause::<DocumentGuestEventError>()
    else {
        panic!("native Show must retain the exact original Source operation: {failure:?}");
    };
    let before = host.snapshot().unwrap();
    let node = before.node(subject).unwrap();
    assert_eq!(
        node.attribute(&"data-event-count".into()),
        Some(&"1".into())
    );
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(!dialog.is_modal());
    let watcher = dialog.close_watcher().unwrap();
    assert_eq!(watcher.global(), target.global());
    assert_eq!(watcher.subject(), subject);
    let queued = tasks.pending_task(DomManipulation, 0).unwrap().unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert_eq!(
        queued.dialog_toggle().unwrap().id(),
        dialog.toggle_task_tracker().unwrap().task()
    );

    host.with_native(|dom, _, mut source| {
        let pending = dom.pending_source_edit().unwrap();
        assert_eq!(pending.operation(), *operation);
        assert_eq!(pending.phase(), DomSourcePhase::DialogService);
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let resumed = source.resume(dom, *operation, SOURCE, &mut services, &cancel).unwrap();
        assert!(matches!(&resumed,
            DomSourceDriveProgress::Pending {
                request: DomSourceRequest::DialogFocusing(request),
                reason: DomSourceUnavailable::DialogFocusing(HtmlDialogFocusingAction::RunFocusingSteps {
                    control: HtmlDialogFocusControl::Subject,
                }),
            } if request.operation() == *operation && request.subject() == subject
        ), "the same owner must retain the exact focus request: {resumed:?}");
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert_eq!(
        tasks.pending_task(DomManipulation, 0).unwrap(),
        Some(queued)
    );
    let mut recovered = host.stop().into_session().unwrap();
    assert_eq!(
        recovered.dom().pending_source_edit().unwrap().operation(),
        *operation
    );
}

pub(super) fn show_modal_settles_the_open_reaction_before_reporting_a_native_failure() {
    use crate::{
        document_scripting::guest::tests::projected_node_id,
        internal::dom_source::DomSourceDriveError,
    };
    use surgeist_dom::source::DomSourceInvariantReason;
    use surgeist_html::elements::dialog::{HtmlDialogAction, HtmlDialogOperation};

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
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let subject = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                class WatchedDialog extends HTMLDialogElement {
                    static observedAttributes = ['open'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        if (name === 'open' && oldValue === null && newValue === '' && namespace === null) {
                            const count = Number(this.getAttribute('data-open-reactions') ?? '0');
                            this.setAttribute('data-open-reactions', String(count + 1));
                        }
                    }
                }
                customElements.define('watched-modal', WatchedDialog, {extends: 'dialog'});
                const subject = document.createElement('dialog', {is: 'watched-modal'});
                if (!(subject instanceof WatchedDialog)) throw new Error('custom Dialog setup');
                (document.documentElement || document).appendChild(subject);
                const destination = document.implementation.createHTMLDocument('inactive');
                subject.addEventListener('beforetoggle', event => {
                    if (!event.isTrusted || !event.cancelable) return;
                    destination.body.appendChild(subject);
                }, {once: true});
                globalThis.watchedModal = subject;
                return subject;
            })()
        "#)?;
        projected_node_id(scope, &result)
    }).unwrap();
    let native = host.in_document(&permit, |scope| {
        evaluate(scope, "watchedModal.showModal()").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = native else {
        panic!("the genuine native invariant failure is required: {native:?}");
    };
    let Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
        DomError::SourceInvariantFailed { failure, .. },
    ))) = failure.cause::<DomSourceDriveError>()
    else {
        panic!("the failure must identify the original native Source: {failure:?}");
    };
    assert_eq!(failure.dialog_operation(), HtmlDialogOperation::ShowModal);
    assert_eq!(failure.subject(), subject);
    assert_eq!(
        failure.action(),
        HtmlDialogAction::AssertCloseWatcherPresent
    );
    assert_eq!(
        failure.reason(),
        DomSourceInvariantReason::MissingDialogCloseWatcher
    );
    let snapshot = host.snapshot().unwrap();
    let dialog = snapshot.node(subject).unwrap();
    assert_ne!(dialog.owner_document(), Some(target.document()));
    assert!(dialog.is_connected());
    assert_eq!(dialog.attribute(&"open".into()), Some(&"".into()));
    assert!(!dialog.as_dialog().unwrap().is_modal());
    assert_eq!(
        dialog.attribute(&"data-open-reactions".into()),
        Some(&"1".into())
    );
}
