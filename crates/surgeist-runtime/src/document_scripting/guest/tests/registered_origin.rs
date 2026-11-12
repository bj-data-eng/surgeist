//! Native Documents and Fragments in registered creation realms.

use super::*;

pub(in crate::document_scripting) fn native_document_and_fragment_use_their_registered_realm() {
    use surgeist_dom::DomBatch;

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
}
