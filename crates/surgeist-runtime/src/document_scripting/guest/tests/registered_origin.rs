//! Native Documents and Fragments in registered creation realms.

use super::*;
use surgeist_dom::{DomBatch, NodeId};

struct RegisteredOrigin {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    origin: DomHostedTopLevel,
    fragment: NodeId,
    element: NodeId,
    cancellation: Arc<AtomicBool>,
}

fn retired_origin_with_moved_element() -> RegisteredOrigin {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted
            .create_headless_host(&mut dom, &cancellation)
            .unwrap();
        hosted
            .create_top_level(&mut dom, owner, &cancellation)
            .unwrap()
    };
    let (document, fragment) = {
        let hosted = session.hosted_execution();
        let mut dom = session.dom();
        let context = hosted
            .with_current_top_level(&mut dom, origin, &cancellation, |source, dom| {
                source
                    .document_constructor_context(dom.raw(), origin.global(), &cancellation)
                    .map_err(DomHostedRuntimeError::Dom)
            })
            .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.construct_document(context);
        let fragment = batch.create_fragment(&document);
        let commit = dom.apply(batch, &cancellation).unwrap();
        (
            commit.document(&document).unwrap(),
            commit.created(&fragment).unwrap().unwrap(),
        )
    };
    let element = session
        .snapshot()
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancellation.clone(), 1_000, 20_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.originChild = window.document.documentElement",
            ))
            .map(drop)
    })
    .unwrap();
    host.with_native(|dom, mut hosted, _| {
        let retirement = hosted
            .destroy_top_level(dom, origin.navigable(), &cancellation)
            .unwrap();
        assert!((0..256).any(|_| matches!(
            hosted.run_next(dom, retirement.traversable(), &cancellation).unwrap(),
            crate::DomHostedTaskProgress::Complete { operation } if operation == retirement.operation()
        )));
        hosted.destroy_host(dom, origin.host(), &cancellation).unwrap();

        let mut move_child = DomBatch::new();
        move_child.adopt(element, document);
        move_child.insert_before(element, document, None);
        dom.apply(move_child, &cancellation).unwrap();
    })
    .unwrap();
    RegisteredOrigin {
        host,
        permit,
        origin,
        fragment,
        element,
        cancellation,
    }
}

pub(in crate::document_scripting) fn native_document_and_fragment_use_their_registered_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        fragment,
        element,
        cancellation,
    } = retired_origin_with_moved_element();
    let projected = host.in_document(&permit, |scope| {
        let wrapper = scope.evaluate(&units("originChild.ownerDocument"))?;
        let realm = scope
            .callback_object(&wrapper)?
            .associated_realm()
            .identity();
        assert_ne!(realm, scope.realm_authority()?.identity());
        assert_guest(
            scope,
            r#"(() => {
                const root = originChild.ownerDocument;
                let observed = false;
                root.addEventListener("origin", event => {
                    observed = event.target === originChild && event.currentTarget === root
                        && event.composedPath()[1] === root;
                }, { once: true });
                return root === originChild.getRootNode() && root === originChild.parentNode
                    && root.documentElement === originChild
                    && originChild.dispatchEvent(new Event("origin", { bubbles: true }))
                    && observed;
            })()"#,
        )?;
        Ok((realm, scope.retain(&wrapper)))
    });
    let (realm, document_wrapper) = match projected {
        Ok(value) => value,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
            "native Document observation failed: {:?}",
            failure.cause::<DocumentGuestEventError>()
        ),
        Err(error) => panic!("native Document observation failed: {error:?}"),
    };
    host.with_native(|dom, _, _| {
        let mut move_child = DomBatch::new();
        move_child.insert_before(element, fragment, None);
        dom.apply(move_child, &cancellation).unwrap();
    })
    .unwrap();
    let fragment_wrapper = host
        .in_document(&permit, |scope| {
            let wrapper = scope.evaluate(&units("originChild.parentNode"))?;
            assert_eq!(
                scope
                    .callback_object(&wrapper)?
                    .associated_realm()
                    .identity(),
                realm
            );
            assert_guest(
                scope,
                r#"(() => {
                    const root = originChild.parentNode;
                    let observed = false;
                    root.addEventListener("origin", event => {
                        observed = event.target === originChild && event.currentTarget === root
                            && event.composedPath()[1] === root;
                    }, { once: true });
                    return originChild.getRootNode() === root && root.nodeType === 11
                        && root.ownerDocument === originChild.ownerDocument
                        && originChild.dispatchEvent(new Event("origin", { bubbles: true }))
                        && observed;
                })()"#,
            )?;
            Ok(scope.retain(&wrapper))
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
                const document = originChild.ownerDocument;
                const root = originChild.parentNode;
                const actual = document.createDocumentFragment();
                const borrowed = Document.prototype.createDocumentFragment.call(document);
                return actual !== borrowed && actual !== root && borrowed !== root
                    && Object.getPrototypeOf(actual) === Object.getPrototypeOf(root)
                    && Object.getPrototypeOf(borrowed) === Object.getPrototypeOf(root)
                    && actual.ownerDocument === document && borrowed.ownerDocument === document;
            })()"#,
        )
    })
    .unwrap();
    let engine = host.engine.as_mut().unwrap();
    let registered = engine
        .creations
        .find(origin.global(), || Ok::<(), Error>(()))
        .unwrap()
        .unwrap();
    engine
        .process
        .execution()
        .in_realm(registered, |scope| {
            let global = scope.global()?;
            let document = scope.import(&document_wrapper)?;
            let fragment = scope.import(&fragment_wrapper)?;
            scope.set(
                &global,
                PropertyKey::String(&units("nativeDocument")),
                &document,
            )?;
            scope.set(
                &global,
                PropertyKey::String(&units("nativeFragment")),
                &fragment,
            )?;
            assert_guest(
                scope,
                r#"(() => {
                    const constructorsRefused = [Document, DocumentFragment].every(Interface => {
                        try { new Interface(); }
                        catch (error) { return error instanceof TypeError; }
                        return false;
                    });
                    return Object.getPrototypeOf(nativeDocument) === Document.prototype
                        && Object.getPrototypeOf(nativeFragment) === DocumentFragment.prototype
                        && nativeDocument instanceof Node && nativeFragment instanceof Node
                        && nativeFragment.ownerDocument === nativeDocument
                        && typeof window === "undefined" && typeof document === "undefined"
                        && typeof Window === "undefined" && constructorsRefused;
                })()"#,
            )?;
            Ok(())
        })
        .unwrap();

    host.with_native(|dom, _, _| {
        let mut retire = DomBatch::new();
        retire.detach(element);
        retire.destroy(fragment);
        dom.apply(retire, &cancellation).unwrap();
    })
    .unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
        host.in_document(&permit, |scope| {
            let global = scope.global()?;
            let retired = scope.import(&fragment_wrapper)?;
            scope.set(
                &global,
                PropertyKey::String(&units("retiredOriginFragment")),
                &retired,
            )?;
            scope
                .evaluate(&units("retiredOriginFragment.nodeType"))
                .map(drop)
        })
    else {
        panic!("the original Fragment wrapper must observe native retirement");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn registered_origin_refusals_preserve_state() {
    assert_projection_admission();

    let RegisteredOrigin {
        mut host, permit, ..
    } = retired_origin_with_moved_element();
    let before = host.snapshot().unwrap().stamp();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) =
        host.in_document(&permit, |scope| {
            scope
                .evaluate(&units(
                    "Document.prototype.createTextNode.call(originChild.ownerDocument, 'text')",
                ))
                .map(drop)
        })
    else {
        panic!("a borrowed factory requires its receiver's CharacterData profile");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::CreationGlobalUnavailable)
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}

fn assert_projection_admission() {
    let read = |scope: &mut Scope<'_>| {
        scope
            .evaluate(&units("originChild.ownerDocument"))
            .map(drop)
    };
    let (work, bytes) = {
        let RegisteredOrigin {
            mut host, permit, ..
        } = retired_origin_with_moved_element();
        let events = host.guest_events().unwrap();
        let before = (events.budget.work.get(), events.budget.used.get());
        host.in_document(&permit, read).unwrap();
        (
            events.budget.work.get() - before.0,
            events.budget.used.get() - before.1,
        )
    };
    assert!(work > 0 && bytes > 0);
    for (work_available, bytes_available) in [(work, bytes), (work - 1, bytes), (work, bytes - 1)] {
        let RegisteredOrigin {
            mut host, permit, ..
        } = retired_origin_with_moved_element();
        let events = host.guest_events().unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let original_bytes = events.budget.used.get();
        let reserve = events.limits.storage_bytes - original_bytes - bytes_available;
        let reservation = events.reserve(&host.native, reserve).unwrap();
        events.budget.work.set(events.limits.work - work_available);
        let before = events.budget.used.get();
        let result = host.in_document(&permit, read);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        if work_available == work && bytes_available == bytes {
            result.unwrap();
            assert_eq!(events.budget.work.get(), events.limits.work);
            assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
            continue;
        }
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("one-short Document projection must refuse");
        };
        assert!(matches!(
            (
                work_available < work,
                failure.cause::<DocumentGuestEventError>()
            ),
            (true, Some(DocumentGuestEventError::WorkLimit))
                | (
                    false,
                    Some(DocumentGuestEventError::Document(
                        DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
                    ))
                )
        ));
        assert_eq!(events.budget.used.get(), before);
        drop(reservation);
        assert_eq!(events.budget.used.get(), original_bytes);
    }
}
