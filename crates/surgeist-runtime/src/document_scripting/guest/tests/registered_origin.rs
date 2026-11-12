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
    assert_native_projection_admission(
        || {
            let RegisteredOrigin { host, permit, .. } = retired_origin_with_moved_element();
            (host, permit)
        },
        "originChild.ownerDocument",
    );

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

pub(in crate::document_scripting) fn borrowed_fragment_factory_keeps_dom_on_wrapper_refusal() {
    let RegisteredOrigin {
        mut host, permit, ..
    } = retired_origin_with_moved_element();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.factoryDocument = originChild.ownerDocument",
            ))
            .map(drop)
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    events
        .public
        .as_ref()
        .unwrap()
        .refuse_next_ordinary_wrapper_validation();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "Document.prototype.createDocumentFragment.call(factoryDocument)",
            ))
            .map(drop)
    });
    assert!(matches!(
        result,
        Err(DocumentScriptError::Script(Error::BackendUnavailable))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.nodes().count(), before.nodes().count());
}

pub(in crate::document_scripting) fn shadow_root_keeps_its_registered_creation_realm() {
    use surgeist_dom::{DomShadowRootInit, HtmlShadowRootMode};
    use surgeist_html::values::Namespace;

    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        element,
        cancellation,
        ..
    } = retired_origin_with_moved_element();
    let document = host
        .snapshot()
        .unwrap()
        .node(element)
        .unwrap()
        .owner_document()
        .unwrap();
    let shadow_host = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let host = batch.create_element_ns(document, Namespace::html(), "div");
            batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
            batch.replace_child(&host, document, element);
            let commit = dom.apply(batch, &cancellation).unwrap();
            commit.created(&host).unwrap().unwrap()
        })
        .unwrap();
    let (realm, root) = host
        .in_document(&permit, |scope| {
            let root = scope.evaluate(&units(
                "globalThis.originShadow = originChild.ownerDocument.documentElement.shadowRoot",
            ))?;
            let realm = scope.callback_object(&root)?.associated_realm().identity();
            assert_ne!(realm, scope.realm_authority()?.identity());
            Ok((realm, scope.retain(&root)))
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(shadow_host, permit.document);
        batch.insert_before(shadow_host, permit.document, None);
        dom.apply(batch, &cancellation).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "window.document.documentElement === originShadow.host \
             && window.document.documentElement.shadowRoot === originShadow \
             && originShadow.ownerDocument === window.document \
             && originShadow.getRootNode({composed: true}) === window.document",
        )
    })
    .unwrap();
    let engine = host.engine.as_mut().unwrap();
    let original = engine
        .creations
        .find(origin.global(), || Ok::<(), Error>(()))
        .unwrap()
        .unwrap();
    engine
        .process
        .execution()
        .in_realm(original, |scope| {
            assert_eq!(scope.realm_authority()?.identity(), realm);
            let value = scope.import(&root)?;
            let global = scope.global()?;
            scope.set(&global, PropertyKey::String(&units("originShadow")), &value)?;
            assert_guest(
                scope,
                "Object.getPrototypeOf(originShadow) === ShadowRoot.prototype \
                 && originShadow instanceof DocumentFragment",
            )
        })
        .unwrap();
}

pub(in crate::document_scripting) fn registered_origin_character_factories_use_receiver_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        ..
    } = retired_origin_with_moved_element();
    let result = host.in_document(&permit, |scope| {
        scope.evaluate(&units(
            r#"globalThis.characterDocument = originChild.ownerDocument;
               globalThis.originText = Document.prototype.createTextNode.call(
                   characterDocument, "\uD800\u0000B");
               globalThis.originComment = characterDocument.createComment("native");"#,
        ))?;
        assert_guest(
            scope,
            "originText.ownerDocument === characterDocument && originText.length === 3 \
             && originText.data.charCodeAt(0) === 0xd800 && originText.data.charCodeAt(1) === 0 \
             && originComment.ownerDocument === characterDocument && originComment.data === 'native'",
        )?;
        let text = scope.evaluate(&units("originText"))?;
        let comment = scope.evaluate(&units("originComment"))?;
        let realm = scope.callback_object(&text)?.associated_realm().identity();
        assert_eq!(scope.callback_object(&comment)?.associated_realm().identity(), realm);
        assert_ne!(scope.realm_authority()?.identity(), realm);
        Ok((realm, scope.retain(&text), scope.retain(&comment)))
    });
    let (realm, text, comment) = match result {
        Ok(value) => value,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
            "registered CharacterData factory failed: {:?}",
            failure.cause::<DocumentGuestEventError>()
        ),
        Err(error) => panic!("registered CharacterData factory failed: {error:?}"),
    };
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
            let text = scope.import(&text)?;
            let comment = scope.import(&comment)?;
            scope.set(&global, PropertyKey::String(&units("nativeText")), &text)?;
            scope.set(
                &global,
                PropertyKey::String(&units("nativeComment")),
                &comment,
            )?;
            assert_eq!(scope.realm_authority()?.identity(), realm);
            assert_guest(
                scope,
                "Object.getPrototypeOf(nativeText) === Text.prototype \
             && Object.getPrototypeOf(nativeComment) === Comment.prototype \
             && Object.getPrototypeOf(Text.prototype) === CharacterData.prototype \
             && Object.getPrototypeOf(Comment.prototype) === CharacterData.prototype \
             && typeof window === 'undefined'",
            )?;
            scope.evaluate(&units("nativeText.replaceData(2, 1, 'C')"))?;
            Ok(())
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "originText.data.charCodeAt(2) === 67 && originText.length === 3",
        )
    })
    .unwrap();
}
