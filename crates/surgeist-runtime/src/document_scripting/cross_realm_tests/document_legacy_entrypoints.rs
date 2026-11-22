use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, expose_reaction_work_exhaustion,
    fund_guest_operation_after_setup, guest_backing_used, set_guest_work_remaining,
};

// HTML877d 156302 defines these three operations without effects; 97492
// defines HTMLDocument as an alias of the existing Document interface object.
pub(super) fn document_legacy_entrypoints_preserve_tree_and_constructor_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const root = document.documentElement;
                const head = document.head;
                const body = document.body;
                if (!(document instanceof Document) || root?.localName !== 'html' ||
                    head?.parentNode !== root || head.localName !== 'head' ||
                    body?.parentNode !== root || body.localName !== 'body') return 'native Document fixture';
                for (const member of ['clear', 'captureEvents', 'releaseEvents']) {
                    const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, member);
                    if (typeof descriptor?.value !== 'function') return 'document missing legacy operation ' + member;
                    const method = descriptor.value;
                    if (!descriptor.writable || !descriptor.enumerable || !descriptor.configurable ||
                        method.name !== member || method.length !== 0) return 'legacy operation descriptor ' + member;
                    if (method.call(document) !== undefined) return 'legacy operation return ' + member;
                    if (document.documentElement !== root || document.head !== head || document.body !== body ||
                        head.parentNode !== root || body.parentNode !== root) return 'legacy operation changed tree ' + member;
                }
                const alias = Object.getOwnPropertyDescriptor(globalThis, 'HTMLDocument');
                if (!alias) return 'missing HTMLDocument alias';
                return alias.value === Document && alias.writable && !alias.enumerable && alias.configurable &&
                    Document.name === 'Document' && Document.prototype.constructor === Document &&
                    Object.prototype.toString.call(document) === '[object Document]'
                    ? 'ok' : 'HTMLDocument alias identity and descriptor';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

pub(super) fn document_legacy_entrypoints_preserve_native_qualification_and_realm_contracts() {
    legacy_methods_authenticate_documents_and_ignore_extra_arguments();
    legacy_methods_preserve_actual_listener_delivery();
    html_document_alias_preserves_constructor_and_independent_global_properties();
    legacy_methods_and_alias_keep_registered_realms();
    legacy_methods_preserve_the_original_pending_source_operation();
    legacy_methods_do_not_require_source_edits_or_outputs();
    for member in ["clear", "captureEvents", "releaseEvents"] {
        legacy_method_preparation_refusal_keeps_original_cause_and_native_state(member);
        legacy_method_cancellation_refuses_before_entry(member);
    }
}

fn check_legacy_script(label: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    // Fund the repeated native named lookups in the multi-receiver scenarios.
    set_guest_work_remaining(&host, 16_000);
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{label}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "legacy Document {label}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::internal::dom_source::DomSourceDriveError>()
        ),
        _ => panic!("legacy Document {label}: {error:?}"),
    });
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn legacy_methods_authenticate_documents_and_ignore_extra_arguments() {
    check_legacy_script(
        "brands, descriptors and ignored arguments",
        r#"
        (() => {
            const native = document;
            const empty = new Document();
            const factory = document.implementation.createHTMLDocument();
            const xml = document.implementation.createDocument('urn:document', 'root');
            const wrong = document.createElement('div');
            Object.setPrototypeOf(wrong, Document.prototype);
            let touched = false;
            const poison = new Proxy({}, {
                get() { touched = true; throw 0; }, ownKeys() { touched = true; throw 0; },
                getOwnPropertyDescriptor() { touched = true; throw 0; },
                getPrototypeOf() { touched = true; throw 0; }
            });
            const hooks = {
                toString() { touched = true; throw 0; }, valueOf() { touched = true; throw 0; },
                get [Symbol.toPrimitive]() { touched = true; throw 0; },
                get [Symbol.iterator]() { touched = true; throw 0; }
            };
            let previous = -1;
            for (const name of ['clear', 'captureEvents', 'releaseEvents']) {
                const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, name);
                const method = descriptor.value;
                const index = Object.keys(Document.prototype).indexOf(name);
                if (index <= previous || !descriptor.writable || !descriptor.enumerable || !descriptor.configurable ||
                    descriptor.get !== undefined || descriptor.set !== undefined || method.name !== name ||
                    method.length !== 0 || Object.hasOwn(method, 'prototype')) return 'ordinary ordered method descriptor ' + name;
                previous = index;
                try { new method(); return 'operation is constructible'; }
                catch (error) { if (!(error instanceof TypeError)) return 'nonconstructible operation error'; }
                for (const receiver of [native, empty, factory, xml]) {
                    const root = receiver.documentElement;
                    const head = receiver.head;
                    const body = receiver.body;
                    let evaluated = '';
                    if (method.call(receiver, (evaluated += 'a', poison), (evaluated += 'b', hooks),
                        Symbol(), 1n, null, undefined) !== undefined || touched || evaluated !== 'ab')
                        return 'extra arguments are evaluated but uninspected';
                    if (receiver.documentElement !== root || receiver.head !== head || receiver.body !== body)
                        return 'actual Document children preserved';
                }
                for (const receiver of [null, undefined, {}, Object.create(Document.prototype),
                    Object.create(document), new Proxy(document, {}), new Proxy(xml, {}), wrong,
                    document.createDocumentFragment(), document.createTextNode('not a Document')]) {
                    try { method.call(receiver, poison, hooks); return 'wrong Document accepted'; }
                    catch (error) { if (!(error instanceof TypeError) || touched) return 'brand before argument inspection'; }
                }
            }
            return 'ok';
        })()
    "#,
    );
}

fn legacy_methods_preserve_actual_listener_delivery() {
    check_legacy_script(
        "listener preservation",
        r#"
        (() => {
            const root = document.documentElement, head = document.head, body = document.body;
            const calls = [];
            let valid = true;
            document.addEventListener('legacy-probe', function(event) {
                valid = valid && this === document && event.target === body && event.currentTarget === document &&
                    event.eventPhase === Event.CAPTURING_PHASE;
                calls.push('capture');
            }, true);
            body.addEventListener('legacy-probe', function(event) {
                valid = valid && this === body && event.target === body && event.currentTarget === body;
                calls.push('body');
            });
            document.addEventListener('legacy-probe', function(event) {
                valid = valid && this === document && event.currentTarget === document &&
                    event.eventPhase === Event.BUBBLING_PHASE;
                calls.push('bubble');
            });
            for (const member of ['clear', 'captureEvents', 'releaseEvents']) {
                if (document[member]() !== undefined || document.documentElement !== root ||
                    document.head !== head || document.body !== body) return 'operation altered document';
                const event = new Event('legacy-probe', {bubbles: true});
                if (!body.dispatchEvent(event) || event.target !== body || event.currentTarget !== null)
                    return 'actual authored listener delivery';
            }
            return valid && calls.join() === 'capture,body,bubble,capture,body,bubble,capture,body,bubble'
                ? 'ok' : 'existing capture and bubble listeners survive all operations';
        })()
    "#,
    );
}

fn html_document_alias_preserves_constructor_and_independent_global_properties() {
    check_legacy_script(
        "alias constructor and globals",
        r#"
        (() => {
            const original = Document;
            const alias = HTMLDocument;
            const prototype = Document.prototype;
            const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'HTMLDocument');
            if (alias !== original || alias.prototype !== prototype || !descriptor.writable || descriptor.enumerable ||
                !descriptor.configurable || descriptor.get !== undefined || descriptor.set !== undefined ||
                alias.name !== 'Document' || prototype.constructor !== original ||
                Object.prototype.toString.call(document) !== '[object Document]') return 'alias is existing interface object';
            for (const Type of [Document, HTMLDocument]) {
                try { Type(); return 'Document call accepted'; }
                catch (error) { if (!(error instanceof TypeError)) return 'Document call behavior'; }
                const created = new Type();
                if (Object.getPrototypeOf(created) !== prototype || !(created instanceof Document) ||
                    !(created instanceof HTMLDocument) || created.documentElement !== null ||
                    Object.prototype.toString.call(created) !== '[object Document]') return 'shared constructor behavior';
            }
            HTMLDocument = 17;
            if (HTMLDocument !== 17 || Document !== original || prototype.constructor !== original)
                return 'alias reassignment independent of Document';
            if (!delete globalThis.HTMLDocument || Object.hasOwn(globalThis, 'HTMLDocument') ||
                Document !== original || Document.prototype !== prototype || document.constructor !== original)
                return 'alias deletion independent of Document';
            const created = new Document();
            return created instanceof original && Object.prototype.toString.call(created) === '[object Document]'
                ? 'ok' : 'Document remains usable after alias deletion';
        })()
    "#,
    );
}

fn legacy_methods_and_alias_keep_registered_realms() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "originLegacyDocument",
    );
    let (
        foreign_document,
        foreign_alias,
        foreign_type_error,
        foreign_all_collection,
        foreign_collection,
        foreign_node_list,
    ) = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine
            .process
            .execution()
            .in_realm(realm, |scope| {
                let document = evaluate(scope, "Document")?;
                let alias = evaluate(scope, "HTMLDocument")?;
                let error = evaluate(scope, "TypeError")?;
                let all_collection = evaluate(scope, "HTMLAllCollection")?;
                let collection = evaluate(scope, "HTMLCollection")?;
                let node_list = evaluate(scope, "NodeList")?;
                Ok((
                    scope.retain(&document),
                    scope.retain(&alias),
                    scope.retain(&error),
                    scope.retain(&all_collection),
                    scope.retain(&collection),
                    scope.retain(&node_list),
                ))
            })
            .unwrap()
    };
    let before = host.snapshot().unwrap().stamp();
    fund_guest_operation_after_setup(&host);
    host.in_document(&permit, |scope| {
        let value = evaluate(scope, "originLegacyDocument")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        for (name, root) in [("foreignLegacyDocument", &foreign_document),
            ("foreignLegacyAlias", &foreign_alias), ("foreignLegacyTypeError", &foreign_type_error),
            ("foreignAllCollection", &foreign_all_collection),
            ("foreignHTMLCollection", &foreign_collection),
            ("foreignNodeList", &foreign_node_list)] {
            let value = scope.import(root)?;
            scope.set(&global, PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()), &value)?;
        }
        let result = evaluate(scope, r#"
            (() => {
                if (HTMLDocument !== Document || foreignLegacyAlias !== foreignLegacyDocument ||
                    foreignLegacyDocument === Document || foreignLegacyAlias === HTMLDocument ||
                    originLegacyDocument instanceof Document || !(originLegacyDocument instanceof foreignLegacyDocument) ||
                    Object.getPrototypeOf(originLegacyDocument).constructor !== foreignLegacyDocument ||
                    foreignLegacyDocument.name !== 'Document' || foreignLegacyDocument.prototype.constructor !== foreignLegacyDocument ||
                    Object.prototype.toString.call(originLegacyDocument) !== '[object Document]') return 'distinct realm aliases and prototypes';
                const originalRoot = originLegacyDocument.documentElement;
                const originalHead = originLegacyDocument.head;
                const originalBody = originLegacyDocument.body;
                const localAll = Object.getOwnPropertyDescriptor(Document.prototype, 'all').get;
                const foreignAll = Object.getOwnPropertyDescriptor(foreignLegacyDocument.prototype, 'all').get;
                const nativeCollection = foreignAll.call(document);
                const registeredCollection = localAll.call(originLegacyDocument);
                if (!(nativeCollection instanceof HTMLAllCollection)
                    || !(registeredCollection instanceof foreignAllCollection)
                    || registeredCollection instanceof HTMLAllCollection
                    || registeredCollection === nativeCollection || nativeCollection !== document.all
                    || registeredCollection !== originLegacyDocument.all
                    || Reflect.apply(registeredCollection, nativeCollection, ['0']) !== originalRoot
                    || Reflect.apply(nativeCollection, registeredCollection, ['0']) !== document.documentElement)
                    return 'borrowed collection getters retain the Document creation realm and holder';
                globalThis.originalLegacyAll = registeredCollection;
                globalThis.originalLegacyRoot = originalRoot;
                const localForms = Object.getOwnPropertyDescriptor(Document.prototype, 'forms').get;
                const foreignForms = Object.getOwnPropertyDescriptor(foreignLegacyDocument.prototype, 'forms').get;
                const registeredForms = localForms.call(originLegacyDocument);
                if (!(registeredForms instanceof foreignHTMLCollection)
                    || registeredForms instanceof HTMLCollection
                    || !(foreignForms.call(document) instanceof HTMLCollection)
                    || registeredForms !== originLegacyDocument.forms
                    || registeredForms === document.forms || registeredForms.length !== 0)
                    return 'standard collections follow foreign Document creation and identity';
                globalThis.originalLegacyForms = registeredForms;
                const childNodes = Object.getOwnPropertyDescriptor(Node.prototype, 'childNodes').get;
                const foreignNode = Object.getPrototypeOf(foreignLegacyDocument.prototype);
                const foreignChildNodes = Object.getOwnPropertyDescriptor(foreignNode, 'childNodes').get;
                const foreignChildren = childNodes.call(originLegacyDocument);
                const foreignNames = Document.prototype.getElementsByName.call(originLegacyDocument, 'absent');
                if (!(foreignChildren instanceof foreignNodeList)
                    || !(foreignNames instanceof foreignNodeList) || foreignNames.length !== 0
                    || foreignChildren instanceof NodeList || foreignNames instanceof NodeList
                    || foreignChildren !== originLegacyDocument.childNodes
                    || !(foreignChildNodes.call(document) instanceof NodeList)
                    || !(foreignLegacyDocument.prototype.getElementsByName.call(document, 'absent') instanceof NodeList)
                    || [...foreignChildren].every(node => node !== originalRoot))
                    return 'NodeLists follow the exact native receiver and original creation realm';
                globalThis.originalLegacyChildren = foreignChildren;
                globalThis.originalLegacyNames = foreignNames;
                const documentChildren = Object.getOwnPropertyDescriptor(Document.prototype, 'children').get;
                const foreignDocumentChildren = Object.getOwnPropertyDescriptor(foreignLegacyDocument.prototype, 'children').get;
                const originalElements = documentChildren.call(originLegacyDocument);
                const originalDescendants = Document.prototype.getElementsByTagName.call(originLegacyDocument, '*');
                const namespacedDescendants = Document.prototype.getElementsByTagNameNS.call(originLegacyDocument, '*', 'body');
                const elementClassQuery = Element.prototype.getElementsByClassName.call(originalRoot, 'absent');
                if (!(originalElements instanceof foreignHTMLCollection) || originalElements[0] !== originalRoot
                    || originalElements !== originLegacyDocument.children || originalElements instanceof HTMLCollection
                    || !(foreignDocumentChildren.call(document) instanceof HTMLCollection)
                    || !(originalDescendants instanceof foreignHTMLCollection) || originalDescendants[0] !== originalRoot
                    || !(foreignLegacyDocument.prototype.getElementsByTagName.call(document, '*') instanceof HTMLCollection)
                    || !(namespacedDescendants instanceof foreignHTMLCollection) || namespacedDescendants[0] !== originalBody
                    || !(elementClassQuery instanceof foreignHTMLCollection) || elementClassQuery.length !== 0)
                    return 'Element collections inherit the exact native root realm when called across realms';
                globalThis.originalLegacyElements = originalElements;
                globalThis.originalLegacyDescendants = originalDescendants;
                const poison = new Proxy({}, {get() { throw 0; }, ownKeys() { throw 0; }});
                for (const name of ['clear', 'captureEvents', 'releaseEvents']) {
                    const foreign = Object.getOwnPropertyDescriptor(foreignLegacyDocument.prototype, name).value;
                    const local = Document.prototype[name];
                    if (foreign === local || foreign.call(originLegacyDocument, poison) !== undefined ||
                        foreign.call(document, poison) !== undefined || local.call(originLegacyDocument, poison) !== undefined)
                        return 'borrowed calls authenticate actual receiver Documents';
                    for (const receiver of [{}, Object.create(originLegacyDocument), new Proxy(originLegacyDocument, {})]) {
                        try { foreign.call(receiver, poison); return 'foreign wrong receiver accepted'; }
                        catch (error) { if (!(error instanceof foreignLegacyTypeError) || error instanceof TypeError)
                            return 'foreign function realm TypeError'; }
                    }
                }
                return originLegacyDocument.documentElement === originalRoot && originLegacyDocument.head === originalHead &&
                    originLegacyDocument.body === originalBody && originalHead.ownerDocument === originLegacyDocument &&
                    originalBody.ownerDocument === originLegacyDocument ? 'ok' : 'foreign children identity preserved';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let all = evaluate(scope, "originalLegacyAll")?;
        assert_eq!(
            scope.callback_object(&all)?.associated_realm().identity(),
            scope.callback_object(&value)?.associated_realm().identity()
        );
        let forms = evaluate(scope, "originalLegacyForms")?;
        assert_eq!(
            scope.callback_object(&forms)?.associated_realm().identity(),
            scope.callback_object(&value)?.associated_realm().identity()
        );
        for name in [
            "originalLegacyChildren",
            "originalLegacyNames",
            "originalLegacyElements",
            "originalLegacyDescendants",
        ] {
            let list = evaluate(scope, name)?;
            assert_eq!(
                scope.callback_object(&list)?.associated_realm().identity(),
                scope.callback_object(&value)?.associated_realm().identity()
            );
        }
        Ok(())
    }).unwrap();
    {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine.process.execution().in_realm(realm, |scope| {
            let result = evaluate(scope, r#"
                (() => {
                    const original = Document;
                    const alias = HTMLDocument;
                    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'HTMLDocument');
                    if (descriptor.value !== original || !descriptor.writable || descriptor.enumerable || !descriptor.configurable)
                        return false;
                    HTMLDocument = 'origin replacement';
                    if (HTMLDocument !== 'origin replacement' || Document !== original) return false;
                    return delete globalThis.HTMLDocument && !Object.hasOwn(globalThis, 'HTMLDocument') &&
                        Document === original && Document.prototype.constructor === alias;
                })()
            "#)?;
            assert!(scope.boolean(&result)?);
            Ok(())
        }).unwrap();
    }
    super::super::guest::tests::fund_guest_operation_after_setup(&host);
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "HTMLDocument === Document && HTMLDocument !== foreignLegacyAlias && foreignLegacyAlias === foreignLegacyDocument")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, mut hosted, _| {
        let retirement = hosted
            .destroy_top_level(dom, origin.navigable(), &cancel)
            .unwrap();
        assert!((0..256).any(|_| matches!(
            hosted.run_next(dom, retirement.traversable(), &cancel).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == retirement.operation()
        )));
        hosted.destroy_host(dom, origin.host(), &cancel).unwrap();
        assert!(
            !dom.snapshot()
                .document(origin.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                // Retiring the native navigable retains its Document identity.
                // These algorithms impose no fully-active Document condition.
                for (const name of ['clear', 'captureEvents', 'releaseEvents']) {
                    if (Document.prototype[name].call(originLegacyDocument) !== undefined ||
                        foreignLegacyDocument.prototype[name].call(originLegacyDocument) !== undefined)
                        return false;
                }
                const collection = originLegacyDocument.all;
                return HTMLDocument === Document && foreignLegacyAlias === foreignLegacyDocument
                    && collection === originalLegacyAll
                    && Object.getPrototypeOf(collection) === foreignAllCollection.prototype
                    && collection[0] === originalLegacyRoot
                    && Reflect.apply(collection, null, ['0']) === originalLegacyRoot
                    && originLegacyDocument.forms === originalLegacyForms
                    && originalLegacyForms.length === 0
                    && originLegacyDocument.childNodes === originalLegacyChildren
                    && [...originalLegacyChildren].includes(originalLegacyRoot)
                    && originalLegacyNames.length === 0
                    && originLegacyDocument.getElementsByName('absent') instanceof foreignNodeList
                    && originLegacyDocument.children === originalLegacyElements
                    && originalLegacyElements[0] === originalLegacyRoot
                    && originalLegacyDescendants[0] === originalLegacyRoot
                    && originLegacyDocument.getElementsByClassName('absent') instanceof foreignHTMLCollection;
            })()
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}

fn legacy_methods_preserve_the_original_pending_source_operation() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host(cancel.clone());
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, dialog, "legacySourceDialog");
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.legacySourceChecks = false;
            legacySourceDialog.addEventListener('cancel', function(event) {
                globalThis.legacyOriginalSourceEvent = event;
                const owner = this.ownerDocument;
                const root = owner.documentElement, head = owner.head, body = owner.body;
                const poison = {[Symbol.toPrimitive]() { throw 0; }, get [Symbol.iterator]() { throw 0; }};
                let valid = event.isTrusted && event === globalThis.event && event.currentTarget === this;
                for (const member of ['clear', 'captureEvents', 'releaseEvents']) {
                    valid = valid && Document.prototype[member].call(owner, poison) === undefined &&
                        owner.documentElement === root && owner.head === head && owner.body === body &&
                        event === globalThis.event && event.currentTarget === this;
                }
                legacySourceChecks = valid;
                event.preventDefault();
            }, {once: true});
        "#).map(drop)
    }).unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    host.with_native(|dom, _, _| {
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
    })
    .unwrap();
    let result = host
        .service_attached_source_event(&permit, &request, EVENT)
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = result else {
        panic!("legacy methods must complete inside the original Source event: {result:?}");
    };
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    assert_eq!(observations.borrow().len(), 1);
    host.with_native(|dom, _, mut source| {
        // The event service owns the same original pending Source operation;
        // the three no-op calls introduce no child operation or edit.
        assert_eq!(dom.pending_source_edit().unwrap().operation(), request.operation());
        assert!(matches!(source.resume(dom, request.operation(), SOURCE, &mut services, &cancel).unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == request.operation()));
        assert!(dom.pending_source_edit().is_none());
    }).unwrap();
    host.in_document(&permit, |scope| {
        let value = evaluate(scope, "legacySourceChecks && legacyOriginalSourceEvent.currentTarget === null && globalThis.event === undefined")?;
        assert!(scope.boolean(&value)?);
        Ok(())
    }).unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn legacy_methods_do_not_require_source_edits_or_outputs() {
    for zero_outputs in [false, true] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) = hosted_session(&cancel, 0);
        let mut host = if zero_outputs {
            attach_public_nodes_with_source_output_limit(session, target, cancel, 0)
        } else {
            attach_public_nodes(session, target, cancel, 200_000, 0)
        };
        let permit = host.permit().unwrap();
        let before = host.snapshot().unwrap();
        let backing = guest_backing_used(&host);
        host.in_document(&permit, |scope| {
            let value = evaluate(scope, r#"
                document.clear() === undefined && document.captureEvents() === undefined && document.releaseEvents() === undefined
            "#)?;
            assert!(scope.boolean(&value)?);
            Ok(())
        }).unwrap();
        assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
        assert_eq!(
            host.snapshot().unwrap().nodes().count(),
            before.nodes().count()
        );
        assert_eq!(guest_backing_used(&host), backing);
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

fn legacy_method_preparation_refusal_keeps_original_cause_and_native_state(member: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let exhausted = Rc::new(Cell::new(false));
    expose_reaction_work_exhaustion(&mut host, &permit, exhausted.clone());
    let before = host.snapshot().unwrap();
    let backing = guest_backing_used(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(
            scope,
            &format!("exhaustReactionWork(); document.{member}()"),
        )
        .map(drop)
    });
    assert!(exhausted.get());
    let Err(DocumentScriptError::Script(Error::HostFailure(first))) = result else {
        panic!("legacy {member} must refuse unfunded preparation: {result:?}");
    };
    let original = first.cause::<DocumentGuestEventError>().unwrap();
    assert!(matches!(original, DocumentGuestEventError::WorkLimit));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot().unwrap().nodes().count(),
        before.nodes().count()
    );
    assert_eq!(guest_backing_used(&host), backing);
    let entered = Cell::new(false);
    let result = host.in_document(&permit, |scope| {
        entered.set(true);
        evaluate(scope, "42").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(retry))) = result else {
        panic!("legacy {member} must preserve the first terminal refusal: {result:?}");
    };
    assert!(!entered.get());
    assert!(std::ptr::eq(
        original,
        retry.cause::<DocumentGuestEventError>().unwrap()
    ));
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    assert_eq!(guest_backing_used(&host), backing);
    drop(retry);
    drop(first);
    host.stop().into_session().unwrap();
}

fn legacy_method_cancellation_refuses_before_entry(member: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let signal = cancel.clone();
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let function = scope.function(&Callback::new(move |_, _| {
            signal.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(Completion::Undefined)
        }))?;
        scope.set(
            &global,
            PropertyKey::String(&"cancelLegacyCall".encode_utf16().collect::<Vec<_>>()),
            &function,
        )
    })
    .unwrap();
    let before = host.snapshot().unwrap();
    let backing = guest_backing_used(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, &format!("cancelLegacyCall(); document.{member}()")).map(drop)
    });
    cancel.store(false, std::sync::atomic::Ordering::Relaxed);
    let Err(DocumentScriptError::Script(Error::HostFailure(first))) = result else {
        panic!("legacy {member} must refuse cancelled callback preparation: {result:?}");
    };
    // Cancellation is set before the method's callback frame. Hosted payload
    // retention therefore refuses before this fixture can enter NativeRead.
    let original = first.cause::<DocumentGuestEventError>().unwrap();
    assert!(matches!(
        original,
        DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Hosted(
                crate::internal::dom_lifecycle::DomHostedRuntimeError::Cancelled
            )
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot().unwrap().nodes().count(),
        before.nodes().count()
    );
    assert_eq!(guest_backing_used(&host), backing);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "document.clear()").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(retry))) = result else {
        panic!("clearing cancellation must preserve the selected legacy refusal: {result:?}");
    };
    assert!(std::ptr::eq(
        original,
        retry.cause::<DocumentGuestEventError>().unwrap()
    ));
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    drop(retry);
    drop(first);
    host.stop().into_session().unwrap();
}
