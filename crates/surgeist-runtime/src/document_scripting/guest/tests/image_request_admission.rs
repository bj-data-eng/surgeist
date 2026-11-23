//! Read admission and original creation realm for native Image accessors.

use super::*;
use crate::internal::dom_source::{DomSourceDriveProgress, DomSourceServices};
use surgeist_bindings_js::ValueKind;
use surgeist_dom::{
    DomBatch,
    edit::DomChange,
    identity::NodeId,
    source::{DomSourceCompletion, DomSourceEdit},
    url::DomImageUpdateCancellationReason,
};
use surgeist_html::{elements::image::HtmlImgFreshRequestState, values::Namespace};

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, top) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut registry =
        DomEventDispatcher::new(crate::internal::dom_event::DomEventRegistryLimits {
            registrations: 8,
            pending_events: 2,
            nested_dispatches: 2,
            storage_bytes: 32_000,
            work: 10_000,
        })
        .unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    // Fund the two real realms and an additional native Document. The exact
    // admission test narrows the allowance only after preparing its call.
    let host = PreparedDocumentScriptHost::prepare(session, 8_000_000, cancel)
        .unwrap()
        .enable_with_guest_event_registry(
            top,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            None,
            &mut Some(registry),
            DocumentGuestEventLimits {
                storage_bytes: 6_000_000,
                work: 200_000,
                pending_exception_reports: 4,
                node_retirement: DomNodeRetirementLimits::new(16_000_000, 400_000),
                source_edits: public_node_source_limits(2_000_000, 64),
                dispatch: DomEventDispatchLimits {
                    path_items: 8,
                    snapshot_listeners: 8,
                    failure_records: 4,
                    callbacks: 8,
                    work: 10_000,
                },
            },
        )
        .unwrap();
    let permit = host.permit().unwrap();
    (host, permit)
}

fn drive(host: &mut DocumentScriptHost, edit: DomSourceEdit) -> DomSourceCompletion {
    host.with_native(|dom, _, mut source| {
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let result = source
            .apply(
                dom,
                edit,
                public_node_source_limits(2_000_000, 64),
                &mut services,
                &AtomicBool::new(false),
            )
            .unwrap();
        let DomSourceDriveProgress::Complete(completion) = result else {
            panic!("expected the original Source edit to complete, got {result:?}");
        };
        completion
    })
    .unwrap()
}

fn add_real_child_window(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) {
    let frame = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let detached = batch.create_html_document(permit.document(), None);
            let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
            dom.apply(batch, &AtomicBool::new(false))
                .unwrap()
                .created(&frame)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(permit.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    drive(host, DomSourceEdit::insert_before(frame, body, None));
    let live = host.snapshot().unwrap();
    let child = live
        .node(frame)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    assert!(
        live.hosted_navigable(child)
            .unwrap()
            .active_document()
            .is_some()
    );
}

pub(in crate::document_scripting) fn image_getters_keep_the_child_creation_realm_and_refuse_retired_identity()
 {
    let (mut host, permit) = fixture();
    add_real_child_window(&mut host, &permit);
    let id: NodeId = host
        .in_document(&permit, |scope| {
            let image = scope.evaluate(&units(
                r#"(() => {
                    const child = window[0];
                    // Create in the child's realm through a receiver whose
                    // node Document has no navigable.
                    const owner = child.document.implementation.createHTMLDocument();
                    const image = owner.createElement('img');
                    image.setAttribute('src', 'held-in-child.png');
                    const prototype = child.HTMLImageElement.prototype;
                    if (owner.defaultView !== null || image.ownerDocument !== owner ||
                        Object.getPrototypeOf(image) !== prototype ||
                        prototype === HTMLImageElement.prototype)
                        throw Error('expected an inactive Image with the original child brand');
                    const members = ['naturalWidth', 'naturalHeight', 'complete', 'currentSrc'];
                    for (const [index, expected] of [0, 0, false, ''].entries()) {
                        const name = members[index];
                        const original = Object.getOwnPropertyDescriptor(prototype, name)?.get;
                        const local = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, name)?.get;
                        if (typeof original !== 'function' || typeof local !== 'function')
                            return 'missing Image prototype getter ' + name;
                        if (original === local || Object.getPrototypeOf(original) !== child.Function.prototype ||
                            original.call(image) !== expected || local.call(image) !== expected)
                            throw Error('incorrect original or borrowed getter ' + name);
                        try {
                            original.call({});
                            throw Error('foreign getter accepted an ordinary receiver');
                        } catch (error) {
                            if (Object.getPrototypeOf(error) !== child.TypeError.prototype) throw error;
                        }
                    }
                    const actual = Object.getOwnPropertyNames(prototype).filter(name => members.includes(name));
                    if (actual.join() !== members.join()) throw Error('wrong authored Image read order');
                    globalThis.childImageProbe = image;
                    globalThis.originalImageGetter = Object.getOwnPropertyDescriptor(prototype, 'currentSrc').get;
                    return image;
                })()"#,
            ))?;
            if scope.kind(&image)? == ValueKind::String {
                let message = scope.to_utf16(&image)?;
                panic!("{}", String::from_utf16_lossy(&message));
            }
            projected_node_id(scope, &image)
        })
        .unwrap_or_else(|error| match error {
            DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
                "Image creation realm fixture refused: {:?}",
                failure.cause::<DocumentGuestEventError>()
            ),
            other => panic!("Image creation realm fixture failed: {other:?}"),
        });
    let historical = host.snapshot().unwrap();
    let node = historical.node(id).unwrap();
    assert!(node.parent().is_none());
    let owner = node.owner_document().unwrap();
    assert!(
        !historical
            .document(owner)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    let image = node.as_img().unwrap();
    let waiting = image.suspended_update().unwrap().unwrap();
    assert_eq!(waiting.document(), owner);
    assert_eq!(
        image.state().current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(image.state().current_request().current_url().is_empty());

    let retired = drive(&mut host, DomSourceEdit::destroy(id));
    assert_eq!(
        retired
            .commit()
            .changes()
            .changes()
            .iter()
            .filter(|change| matches!(
                change,
                DomChange::ImageUpdateCancelled {
                    update,
                    flags,
                    reason: DomImageUpdateCancellationReason::TargetDestroyed,
                } if *update == waiting.id() && *flags == waiting.flags()
            ))
            .count(),
        1
    );
    assert!(matches!(
        host.snapshot().unwrap().node(id),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        image.suspended_update().unwrap().unwrap().id(),
        waiting.id()
    );

    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("originalImageGetter.call(childImageProbe)"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("retained Image must refuse its stale native receiver: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
    host.stop().into_session().unwrap();
}

struct CompleteCall {
    getter: Root,
    image: Root,
}

impl CompleteCall {
    fn prepare(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) -> Self {
        let call = host
            .in_document(permit, |scope| {
                let image = scope.evaluate(&units(
                    r#"(() => {
                        const owner = document.implementation.createHTMLDocument();
                        const image = owner.createElement('img');
                        image.setAttribute('src', '');
                        return image;
                    })()"#,
                ))?;
                let getter = scope.evaluate(&units(
                    "Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'complete')?.get",
                ))?;
                assert!(
                    scope.is_callable(&getter)?,
                    "missing HTMLImageElement.prototype.complete getter"
                );
                Ok(Self {
                    getter: scope.retain(&getter),
                    image: scope.retain(&image),
                })
            })
            .unwrap();
        fund_guest_operation_after_setup(host);
        call
    }

    fn read(
        &self,
        host: &mut DocumentScriptHost,
        permit: &DocumentScriptPermit,
    ) -> Result<(), DocumentScriptError> {
        host.in_document(permit, |scope| {
            let getter = scope.import(&self.getter)?;
            let image = scope.import(&self.image)?;
            let value = scope.call(&getter, &image, &[])?;
            assert_eq!(scope.kind(&value)?, ValueKind::Boolean);
            assert!(
                scope.boolean(&value)?,
                "the unavailable request with empty src and absent srcset is complete"
            );
            Ok(())
        })
    }
}

pub(in crate::document_scripting) fn image_complete_admits_exact_native_read_work() {
    let (mut host, permit) = fixture();
    let call = CompleteCall::prepare(&mut host, &permit);
    let events = host.guest_events().unwrap();
    let before = events.budget.work().unwrap();
    call.read(&mut host, &permit).unwrap();
    // This scalar getter has no nonempty UTF-16 payload to copy. Measure the
    // actual read work after preparing and retaining the exact getter/receiver.
    let required = events.budget.work().unwrap() - before;
    assert!(required > 0);
    drop(call);
    drop(events);
    host.stop().into_session().unwrap();

    for work in [required, required - 1] {
        let (mut host, permit) = fixture();
        let call = CompleteCall::prepare(&mut host, &permit);
        let events = host.guest_events().unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let retained = events.budget.used() - events.budget.capture_used();
        events.budget.set_work_remaining(work);
        let result = call.read(&mut host, &permit);
        if work == required {
            result.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short Image.complete observation must refuse: {result:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(cause, DocumentGuestEventError::WorkLimit));
            let Err(DocumentScriptError::Script(Error::HostFailure(retry))) = host
                .in_document::<()>(&permit, |_| {
                    panic!("terminal Image read reentered guest work")
                })
            else {
                panic!("retry must report the original Image read refusal");
            };
            assert!(std::ptr::eq(
                cause,
                retry.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(
            events.budget.used() - events.budget.capture_used(),
            retained
        );
        let budget = events.budget.clone();
        drop(call);
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}
