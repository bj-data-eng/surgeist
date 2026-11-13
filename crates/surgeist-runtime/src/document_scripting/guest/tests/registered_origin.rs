//! Native Documents and Fragments in registered creation realms.

use super::super::interfaces::GuestNodeProjection;
use super::*;
use surgeist_dom::{DomBatch, DomNodeKind, NodeId};

pub(in crate::document_scripting) fn document_element_factories_use_native_names_and_identity() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancellation, 0);
    let mut host = attach_public_nodes(session, target, cancellation, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        assert_guest(scope, "typeof document.createElement === 'function' && typeof document.createElementNS === 'function'")?;
        let result = scope.evaluate(&units(
            r#"(() => {
                const throws = (action, name) => {
                    try { action(); } catch (error) { return error.name === name; }
                    return false;
                };
                const first = document.createElement("DIV");
                const second = document.createElement("div");
                const circle = document.createElementNS("http://www.w3.org/2000/svg", "s:circle");
                const generic = document.createElementNS("", "DIV");
                let conversionOrder = '';
                let inner;
                const original = {};
                const customized = document.createElement(
                    {toString() { conversionOrder += 'n'; return 'div'; }},
                    {get is() { conversionOrder += 'i'; return 'x-local'; },
                     get customElementRegistry() {
                         conversionOrder += 'r';
                         inner = document.createElement('span');
                         return undefined;
                     }});
                let preservedThrow = false;
                try { document.createElement('div', {get is() { throw original; }}); }
                catch (error) { preservedThrow = error === original; }
                const inserted = first.appendChild(second);
                const observed = {
                    arity: document.createElement.length === 1 && document.createElementNS.length === 2,
                    html: first !== second && first instanceof HTMLDivElement
                    && first.localName === "div" && first.ownerDocument === document
                    && first.parentNode === null,
                    retainedIdentity: inserted === second && second.parentNode === first,
                    svg: circle instanceof SVGCircleElement && circle.prefix === "s"
                    && circle.localName === "circle" && circle.ownerDocument === document,
                    generic: generic instanceof Element && !(generic instanceof HTMLElement)
                    && generic.namespaceURI === null && generic.localName === "DIV",
                    customized: customized instanceof HTMLDivElement && customized.getAttribute('is') === null,
                    conversionOrder: conversionOrder === 'nri',
                    reentry: inner instanceof HTMLSpanElement && inner.ownerDocument === document,
                    preservedThrow,
                    legacy: document.createElement('DIV', 'x-legacy') instanceof HTMLDivElement,
                    nullRegistry: document.createElement('DIV', {customElementRegistry: null}) instanceof HTMLDivElement,
                    incompatibleOptions: throws(() => document.createElement('div', {is: 'x-local', customElementRegistry: null}), 'NotSupportedError'),
                    nameBeforeFlattening: throws(() => document.createElement('bad name', {is: 'x-local', customElementRegistry: null}), 'InvalidCharacterError'),
                    invalidName: throws(() => document.createElement("bad name"), "InvalidCharacterError"),
                    invalidNamespace: throws(() => document.createElementNS(null, "p:div"), "NamespaceError"),
                    wrongReceiver: throws(() => Document.prototype.createElement.call({}, "div"), "TypeError")
                };
                for (const [test, ok] of Object.entries(observed)) {
                    if (!ok) return test;
                }
                return 'ok';
            })()"#,
        ))?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let result = scope.evaluate(&units("document.createElement('div', {is: 'x-native'})"))?;
        let node = scope.native_object::<GuestNodeProjection>(&result)?.unwrap().payload().id().unwrap().node();
        let null_registry = scope.evaluate(&units("document.createElement('div', {customElementRegistry: null})"))?;
        let null_node = scope.native_object::<GuestNodeProjection>(&null_registry)?.unwrap().payload().id().unwrap().node();
        Ok((node, null_node))
    })
    .map(|(node, null_node)| {
        let snapshot = host.snapshot().unwrap();
        let element = snapshot.node(node).unwrap();
        let DomNodeKind::Element(element) = element.kind() else { panic!("expected element") };
        assert_eq!(element.saved_is().unwrap().code_units(), units("x-native"));
        let DomNodeKind::Element(element) = snapshot.node(null_node).unwrap().kind() else { panic!("expected element") };
        assert_eq!(element.custom_element_registry(), surgeist_dom::DomRegistryBinding::Null);
    }).unwrap();
}

pub(in crate::document_scripting) fn borrowed_element_factory_keeps_dom_on_wrapper_refusal() {
    let RegisteredOrigin {
        mut host, permit, ..
    } = retired_origin_with_moved_element();
    host.in_document(&permit, |scope| {
        assert_guest(scope, r#"(() => {
            globalThis.factoryDocument = originChild.ownerDocument;
            const svg = Document.prototype.createElementNS.call(factoryDocument, 'http://www.w3.org/2000/svg', 's:circle');
            return svg.ownerDocument === factoryDocument && svg.localName === 'circle'
                && svg.prefix === 's' && !(svg instanceof SVGCircleElement)
                && Object.getPrototypeOf(svg).constructor.name === 'SVGCircleElement';
        })()"#)
    }).unwrap();
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let backing = events.budget.used.get();
    events
        .public
        .as_ref()
        .unwrap()
        .refuse_next_ordinary_wrapper_validation();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "Document.prototype.createElement.call(factoryDocument, 'Exact')",
            ))
            .map(drop)
    });
    assert!(matches!(
        result,
        Err(DocumentScriptError::Script(Error::BackendUnavailable))
    ));
    assert_eq!(events.budget.used.get(), backing);
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.nodes().count(), before.nodes().count());
}

pub(in crate::document_scripting) fn element_factory_admits_exact_projection_work() {
    let make_host = || {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) =
            crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
        let host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
        let permit = host.permit().unwrap();
        (host, permit)
    };
    let create = |scope: &mut Scope<'_>| {
        scope
            .evaluate(&units("document.createElement('div')"))
            .map(drop)
    };
    let required = {
        let (mut host, permit) = make_host();
        let events = host.guest_events().unwrap();
        let before = events.budget.work.get();
        host.in_document(&permit, create).unwrap();
        events.budget.work.get() - before
    };
    assert!(required > 0);
    for available in [required - 1, required] {
        let (mut host, permit) = make_host();
        let events = host.guest_events().unwrap();
        let before = host.snapshot().unwrap();
        let bytes = events.budget.used.get();
        events.budget.work.set(events.limits.work - available);
        let result = host.in_document(&permit, create);
        let after = host.snapshot().unwrap();
        assert_eq!(events.budget.work.get(), events.limits.work);
        if available == required {
            result.unwrap();
            assert_ne!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count() + 1);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short factory must refuse: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit)
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert_eq!(events.budget.used.get(), bytes);
        }
    }
}

struct RegisteredOrigin {
    host: DocumentScriptHost,
    permit: DocumentScriptPermit,
    origin: DomHostedTopLevel,
    fragment: NodeId,
    element: NodeId,
    cancellation: Arc<AtomicBool>,
}

pub(in crate::document_scripting) fn custom_element_factories_use_the_correct_realm() {
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
    let mut host = attach_public_nodes(session, target, cancellation, 20_000, 200_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    let primary_interface = host
        .in_document(&permit, |scope| {
            let constructor = scope.evaluate(&units("HTMLElement"))?;
            Ok(scope.retain(&constructor))
        })
        .unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "foreignDocument",
    );
    let realm_id = {
        let engine = host.engine.as_mut().unwrap();
        let realm = engine
            .creations
            .find(origin.global(), || Ok::<_, Error>(()))
            .unwrap()
            .unwrap();
        engine.process.execution().in_realm(realm, |scope| {
            let global = scope.global()?;
            let primary_interface = scope.import(&primary_interface)?;
            scope.set(&global, PropertyKey::String(&units("primaryHTMLElement")), &primary_interface)?;
            let result = scope.evaluate(&units(r#"
                function Foreign() {
                    const node = Reflect.construct(primaryHTMLElement, [], new.target);
                    if (Object.getPrototypeOf(node) !== HTMLElement.prototype) throw new Error('wrong prototype realm');
                    throw globalThis.originalForeignThrow = {};
                }
                customElements.define('x-foreign', Foreign);
                Foreign.prototype = null;
            "#));
            if let Err(Error::Exception(exception)) = &result {
                let thrown = scope.import(exception.value())?;
                panic!("origin setup: {}", String::from_utf16_lossy(&scope.to_utf16(&thrown)?));
            }
            result?;
            Ok(scope.realm_authority()?.identity())
        }).unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let fallback = scope.evaluate(&units("Document.prototype.createElement.call(foreignDocument, 'x-foreign')"))?;
        assert_eq!(scope.callback_object(&fallback)?.associated_realm().identity(), realm_id);
        scope.set(&global, PropertyKey::String(&units("foreignFallback")), &fallback)?;
        assert_guest(scope, "foreignFallback.ownerDocument === foreignDocument && !(foreignFallback instanceof HTMLUnknownElement)")
    }).unwrap();
    let report = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    assert_eq!(report.realm, realm_id);
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
            let original = scope.evaluate(&units("originalForeignThrow"))?;
            let reported = scope.import(report.exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
            Ok(())
        })
        .unwrap();
}

pub(in crate::document_scripting) fn registered_origin_registry_keeps_exact_wrapper() {
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
    let mut host = attach_public_nodes(session, target, cancellation, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    let (realm, registry) = {
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
                let value = scope.evaluate(&units("customElements"))?;
                Ok((scope.realm_authority()?.identity(), scope.retain(&value)))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let registry = scope.import(&registry)?;
        let origin = scope.callback_object(&registry)?;
        assert_eq!(origin.associated_realm().identity(), realm);
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("otherRegistry")), &registry)?;
        assert_guest(scope, r#"(() => {
            if (otherRegistry === customElements || otherRegistry instanceof CustomElementRegistry) return false;
            try { document.createElement('div', { customElementRegistry: otherRegistry }); }
            catch (error) { return error.name === 'NotSupportedError'; }
            return false;
        })()"#)
    }).unwrap();
}

pub(in crate::document_scripting) fn definition_queues_candidates_before_resolving_waiter() {
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
    let mut host = attach_public_nodes(session, target, cancellation, 20_000, 200_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "otherDocument",
    );
    let foreign = {
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
                let value = scope.evaluate(&units(
                    r#"
                class Foreign extends HTMLElement {}
                customElements.define('x-waiter-adoption', Foreign);
                Foreign
            "#,
                ))?;
                Ok(scope.retain(&value))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let foreign = scope.import(&foreign)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("Foreign")), &foreign)?;
        assert_guest(
            scope,
            r#"(() => {
            const name = 'x-waiter-adoption';
            const candidate = document.createElement(name);
            document.documentElement.appendChild(candidate);
            customElements.whenDefined(name);
            const calls = [];
            class Original extends HTMLElement {
                constructor() {
                    calls.push('original constructor');
                    super();
                }
                static get then() {
                    calls.push('then getter');
                    otherDocument.documentElement.appendChild(candidate);
                    calls.push('then resumes');
                }
            }
            customElements.define(name, Original);
            return candidate.ownerDocument === otherDocument && !(candidate instanceof Foreign) &&
                calls.join(',') === 'then getter,original constructor,then resumes';
        })()"#,
        )
    })
    .unwrap();
    let report = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    host.in_document(&permit, |scope| {
        let original = scope.import(report.exception.value())?;
        let global = scope.global()?;
        scope.set(
            &global,
            PropertyKey::String(&units("originalUpgradeFailure")),
            &original,
        )?;
        assert_guest(scope, "originalUpgradeFailure instanceof TypeError")
    })
    .unwrap();
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

pub(in crate::document_scripting) fn native_doctype_uses_its_registered_creation_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        element,
        cancellation,
        ..
    } = retired_origin_with_moved_element();
    host.with_native(|dom, _, _| {
        let snapshot = dom.snapshot();
        let document = snapshot.node(element).unwrap().owner_document().unwrap();
        let mut batch = DomBatch::new();
        let doctype = batch.create_document_type(document, "html", "", "");
        batch.insert_before(&doctype, document, Some(element.into()));
        dom.apply(batch, &cancellation).unwrap();
    })
    .unwrap();
    let doctype = host.in_document(&permit, |scope| {
        assert_guest(scope, r#"
            (() => {
                const doc = originChild.ownerDocument;
                const getDoctype = Object.getOwnPropertyDescriptor(Document.prototype, 'doctype').get;
                return doc.doctype === getDoctype.call(doc) && doc.doctype.parentNode === doc
                    && doc.doctype.name === 'html' && !(doc.doctype instanceof DocumentType);
            })()
        "#)?;
        let doctype = scope.evaluate(&units("originChild.ownerDocument.doctype"))?;
        Ok(scope.retain(&doctype))
    }).unwrap();
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
            let doctype = scope.import(&doctype)?;
            let global = scope.global()?;
            scope.set(
                &global,
                PropertyKey::String(&units("originalDoctype")),
                &doctype,
            )?;
            assert_guest(
                scope,
                "Object.getPrototypeOf(originalDoctype) === DocumentType.prototype",
            )
        })
        .unwrap();
}

pub(in crate::document_scripting) fn implementation_factories_use_the_receiver_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        ..
    } = retired_origin_with_moved_element();
    let results = host
        .in_document(&permit, |scope| {
            let results = scope.evaluate(&units(
                r#"(() => {
            const doc = originChild.ownerDocument;
            const get = Object.getOwnPropertyDescriptor(Document.prototype, 'implementation').get;
            const impl = get.call(doc);
            const type = DOMImplementation.prototype.createDocumentType.call(impl, 'root', '', '');
            const xml = DOMImplementation.prototype.createDocument.call(impl, null, 'root', type);
            const html = DOMImplementation.prototype.createHTMLDocument.call(impl, 'title');
            return {impl, xml, html, type, same: impl === doc.implementation};
        })()"#,
            ))?;
            Ok(scope.retain(&results))
        })
        .unwrap();
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
            let results = scope.import(&results)?;
            let global = scope.global()?;
            scope.set(
                &global,
                PropertyKey::String(&units("factoryResults")),
                &results,
            )?;
            assert_guest(
                scope,
                r#"(() => {
            const {impl, xml, html, type, same} = factoryResults;
            return same && Object.getPrototypeOf(impl) === DOMImplementation.prototype &&
                Object.getPrototypeOf(xml) === XMLDocument.prototype &&
                Object.getPrototypeOf(html) === Document.prototype &&
                Object.getPrototypeOf(type) === DocumentType.prototype &&
                Object.getPrototypeOf(xml.implementation) === DOMImplementation.prototype &&
                Object.getPrototypeOf(xml.documentElement) === Element.prototype &&
                Object.getPrototypeOf(html.doctype) === DocumentType.prototype &&
                html.documentElement instanceof HTMLElement;
        })()"#,
            )
        })
        .unwrap();
}

pub(in crate::document_scripting) fn native_processing_instruction_uses_its_creation_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        element,
        cancellation,
        ..
    } = retired_origin_with_moved_element();
    let instruction = host
        .with_native(|dom, _, _| {
            let document = dom
                .snapshot()
                .node(element)
                .unwrap()
                .owner_document()
                .unwrap();
            let mut batch = DomBatch::new();
            let created = batch.create_processing_instruction(document, "foreign", "data");
            batch.insert_before(&created, document, Some(element.into()));
            dom.apply(batch, &cancellation)
                .unwrap()
                .created(&created)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, instruction, "foreignInstruction");
    let value = host.in_document(&permit, |scope| {
        assert_guest(scope, r#"
            foreignInstruction.ownerDocument === originChild.ownerDocument &&
            foreignInstruction.parentNode === originChild.parentNode &&
            !(foreignInstruction instanceof ProcessingInstruction) &&
            Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target').get.call(foreignInstruction) === 'foreign'
        "#)?;
        let value = scope.evaluate(&units("foreignInstruction"))?;
        Ok(scope.retain(&value))
    }).unwrap();
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
            let value = scope.import(&value)?;
            let global = scope.global()?;
            scope.set(
                &global,
                PropertyKey::String(&units("foreignInstruction")),
                &value,
            )?;
            assert_guest(
                scope,
                "Object.getPrototypeOf(foreignInstruction) === ProcessingInstruction.prototype",
            )
        })
        .unwrap();
}

pub(in crate::document_scripting) fn implementation_factories_keep_the_projection_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    let child = host
        .in_document(&permit, |scope| {
            let child =
                scope.evaluate(&units("globalThis.probe = document.createElement('div')"))?;
            super::projected_node_id(scope, &child)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        batch.insert_before(child, &document, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let created = host.in_document(&permit, |scope| {
        let doctype = scope.evaluate(&units(
            r#"
            globalThis.impl = probe.ownerDocument.implementation;
            globalThis.type = impl.createDocumentType('root', '', '');
        "#,
        ))?;
        assert_guest(
            scope,
            r#"(() => {
            const xml = impl.createDocument(null, 'root', type);
            const html = impl.createHTMLDocument();
            return xml.doctype === type && xml.documentElement instanceof Element &&
                html.documentElement instanceof HTMLElement && html.doctype instanceof DocumentType;
        })()"#,
        )?;
        super::projected_node_id(scope, &doctype)
    });
    let created = match created {
        Ok(created) => created,
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => {
            panic!(
                "factory lost its projection realm: {:?}",
                failure.cause::<DocumentGuestEventError>()
            )
        }
        Err(error) => panic!("{error:?}"),
    };
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(created)
            .unwrap()
            .relevant_global()
            .unwrap(),
        Some(target.global())
    );
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
                        && root.moveBefore(originChild, originChild) === undefined
                        && originChild.parentNode === root
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

pub(in crate::document_scripting) fn character_data_observation_admits_native_lookup() {
    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.textForWork = new Text('native work probe')",
            ))
            .map(drop)
    })
    .unwrap();
    let snapshot = host.snapshot().unwrap();
    let expected = units("native work probe");
    let node = snapshot
        .nodes()
        .find(|id| {
            matches!(
                snapshot.node(*id).unwrap().kind(),
                surgeist_dom::DomNodeKind::Text(data) if data.code_units() == expected
            )
        })
        .unwrap();
    let mut lookup_work = 0;
    snapshot
        .node_with(node, || {
            lookup_work += 1;
            Ok::<(), DomError>(())
        })
        .unwrap();
    assert!(lookup_work >= 2);
    let events = host.guest_events().unwrap();
    events.budget.work.set(events.limits.work - lookup_work);
    let result = host.in_document(&permit, |scope| {
        assert_guest(scope, "textForWork.length === 17")
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("CharacterData read must refuse one lookup unit short: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), snapshot.stamp());
}

pub(in crate::document_scripting) fn registered_origin_character_factories_use_receiver_realm() {
    let RegisteredOrigin {
        mut host,
        permit,
        origin,
        element,
        cancellation,
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
                r#"(() => {
                    const refused = [Text, Comment].every(Interface => {
                        try { new Interface("unauthorized"); }
                        catch (error) { return error instanceof TypeError; }
                        return false;
                    });
                    return Object.getPrototypeOf(nativeText) === Text.prototype
                        && Object.getPrototypeOf(nativeComment) === Comment.prototype
                        && Object.getPrototypeOf(Text.prototype) === CharacterData.prototype
                        && Object.getPrototypeOf(Comment.prototype) === CharacterData.prototype
                        && typeof window === "undefined" && refused;
                })()"#,
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
    qualify_native_character_projection(&mut host, &permit, element, &cancellation);
}

pub(in crate::document_scripting) fn registered_character_factory_refuses_without_publication() {
    let RegisteredOrigin {
        mut host, permit, ..
    } = retired_origin_with_moved_element();
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            r#"(() => {
                let converted = false;
                try {
                    Document.prototype.createTextNode.call(
                        Object.create(Document.prototype),
                        { toString() { converted = true; return "unreachable"; } });
                } catch (error) {
                    return error instanceof TypeError && !converted;
                }
                return false;
            })()"#,
        )?;
        scope
            .evaluate(&units(
                "globalThis.factoryDocument = originChild.ownerDocument",
            ))
            .map(drop)
    })
    .unwrap();
    let events = host.guest_events().unwrap();
    let spare = events.limits.storage_bytes - events.budget.used.get();
    let backing = events.reserve(&host.native, spare).unwrap();
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "Document.prototype.createTextNode.call(factoryDocument, 'refused')",
            ))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("CharacterData creation must refuse without backing: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
        ))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.nodes().count(), before.nodes().count());
    assert_eq!(events.budget.used.get(), events.limits.storage_bytes);
    drop(backing);
}

fn qualify_native_character_projection(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    element: NodeId,
    cancellation: &AtomicBool,
) {
    let document = host
        .snapshot()
        .unwrap()
        .node(element)
        .unwrap()
        .owner_document()
        .unwrap();
    let (text, comment) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let text = batch.create_text(document, "native text");
            let comment = batch.create_comment(document, "native comment");
            batch.insert_before(&text, element, None);
            batch.insert_before(&comment, element, None);
            let result = dom.apply(batch, cancellation).unwrap();
            (
                result.created(&text).unwrap().unwrap(),
                result.created(&comment).unwrap().unwrap(),
            )
        })
        .unwrap();
    let events = host.guest_events().unwrap();
    let owner = host.native.clone();
    let public = events.public.as_ref().unwrap();
    host.in_document(permit, |scope| {
        let global = scope.global()?;
        for (name, node) in [("exposedText", text), ("exposedComment", comment)] {
            let value = public.project_node(scope, &owner, &events, node).unwrap();
            let repeated = public.project_node(scope, &owner, &events, node).unwrap();
            assert!(scope.strict_equals(&value, &repeated)?);
            scope.set(&global, PropertyKey::String(&units(name)), &value)?;
        }
        assert_guest(
            scope,
            r#"(() => {
                let observed = false;
                originChild.addEventListener("native-character", event => {
                    observed = event.target === exposedText && event.composedPath()[0] === exposedText;
                }, { once: true });
                return Object.getPrototypeOf(exposedText) === Object.getPrototypeOf(originText)
                    && Object.getPrototypeOf(exposedComment) === Object.getPrototypeOf(originComment)
                    && exposedText.parentNode === originChild && exposedComment.parentNode === originChild
                    && exposedText.data === "native text" && exposedComment.data === "native comment"
                    && exposedText.dispatchEvent(new Event("native-character", { bubbles: true }))
                    && observed;
            })()"#,
        )
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(text, permit.document);
        batch.detach(comment);
        batch.destroy(comment);
        dom.apply(batch, cancellation).unwrap();
    })
    .unwrap();
    host.in_document(permit, |scope| {
        let repeated = public.project_node(scope, &owner, &events, text).unwrap();
        let original = scope.evaluate(&units("exposedText"))?;
        assert!(scope.strict_equals(&repeated, &original)?);
        assert_guest(
            scope,
            "exposedText.ownerDocument === window.document && exposedText.getRootNode() === exposedText",
        )
    })
    .unwrap();
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = host
        .in_document(permit, |scope| {
            scope.evaluate(&units("exposedComment.length")).map(drop)
        })
    else {
        panic!("the original Comment wrapper must observe native retirement");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(in crate::document_scripting) fn scripted_shadow_uses_the_current_document_creation_realm() {
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
            let host = batch.create_element_ns(permit.document(), Namespace::html(), "div");
            batch.replace_child(&host, document, element);
            let commit = dom.apply(batch, &cancellation).unwrap();
            commit.created(&host).unwrap().unwrap()
        })
        .unwrap();
    let before = host.snapshot().unwrap();
    assert_ne!(
        before.relevant_global(shadow_host).unwrap(),
        Some(origin.global())
    );
    let (root, expected_realm) = host
        .in_document(&permit, |scope| {
            assert_guest(
                scope,
                r#"(() => {
                    const attach = Element.prototype.attachShadow;
                    try { attach.call(originChild, {mode: "open"}); return false; }
                    catch (error) {
                        if (!(error instanceof DOMException) || error.name !== "NotSupportedError")
                            return false;
                    }
                    const target = originChild.ownerDocument.documentElement;
                    globalThis.borrowedShadow = attach.call(target, {mode: "open"});
                    return borrowedShadow.host === target && target.shadowRoot === borrowedShadow;
                })()"#,
            )?;
            let document = scope.evaluate(&units("originChild.ownerDocument"))?;
            let realm = scope
                .callback_object(&document)?
                .associated_realm()
                .identity();
            assert_ne!(realm, scope.realm_authority()?.identity());
            let root = scope.evaluate(&units("borrowedShadow"))?;
            assert_eq!(
                scope.callback_object(&root)?.associated_realm().identity(),
                realm
            );
            Ok((scope.retain(&root), realm))
        })
        .unwrap();
    let after = host.snapshot().unwrap();
    let id = after.node(shadow_host).unwrap().shadow_root().unwrap();
    assert_eq!(after.relevant_global(id).unwrap(), Some(origin.global()));
    let engine = host.engine.as_mut().unwrap();
    let realm = engine
        .creations
        .find(origin.global(), || Ok::<(), Error>(()))
        .unwrap()
        .unwrap();
    engine
        .process
        .execution()
        .in_realm(realm, |scope| {
            assert_eq!(scope.realm_authority()?.identity(), expected_realm);
            let value = scope.import(&root)?;
            let global = scope.global()?;
            scope.set(
                &global,
                PropertyKey::String(&units("borrowedShadow")),
                &value,
            )?;
            assert_guest(
                scope,
                "Object.getPrototypeOf(borrowedShadow) === ShadowRoot.prototype",
            )
        })
        .unwrap();
}

pub(in crate::document_scripting) fn native_document_constructor_refusal_releases_private_wrapper()
{
    let mut host = public_node_host(1_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let limit = host
        .with_native(|dom, _, _| {
            let limit = dom.limits().managed_nodes();
            let existing = dom.snapshot().nodes().count();
            let mut batch = DomBatch::new();
            for _ in existing..limit {
                batch.create_comment(permit.document(), "");
            }
            dom.apply(batch, &cancel).unwrap();
            assert_eq!(dom.snapshot().nodes().count(), limit);
            limit
        })
        .unwrap();
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let backing = events.budget.used.get();
    let outcome = host.in_document(&permit, |scope| {
        scope.evaluate(&units("new Document()")).map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = outcome else {
        panic!("native node admission must refuse the new Document: {outcome:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(DocumentScriptError::Native(
            DocumentScriptNativeError::Dom(DomError::ResourceLimit {
                resource: surgeist_dom::DomResource::ManagedNodes,
                limit: observed,
            })
        ))) if *observed == limit
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(events.budget.used.get(), backing);
}
