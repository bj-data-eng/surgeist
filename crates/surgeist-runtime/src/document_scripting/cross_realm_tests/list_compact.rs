use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn list_compact_families_preserve_native_and_web_idl_contracts() {
    for tag in ["dl", "dir"] {
        compact_brands_authenticate_the_actual_native_family(tag);
        compact_reflects_presence_without_value_coercion(tag);
        compact_customized_constructors_deliver_original_reactions(tag);
        compact_reads_follow_native_edits_and_retirement(tag);
        compact_accessors_keep_registered_realms_after_adoption(tag);
        compact_setters_admit_source_outputs_before_publication(tag);
    }
}

fn set_compact_tag(scope: &mut Scope<'_>, tag: &str) -> Result<(), Error> {
    let global = scope.global()?;
    let value = scope.string(&tag.encode_utf16().collect::<Vec<_>>())?;
    scope.set(
        &global,
        PropertyKey::String(&"compactTag".encode_utf16().collect::<Vec<_>>()),
        &value,
    )
}

fn check_compact_script(tag: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_compact_tag(scope, tag)?;
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{tag}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "compact {tag}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("compact {tag}: {error:?}"),
    });
}

fn compact_brands_authenticate_the_actual_native_family(tag: &str) {
    check_compact_script(
        tag,
        r#"
        (() => {
            const Type = compactTag === 'dl' ? HTMLDListElement : HTMLDirectoryElement;
            const element = document.createElement(compactTag.toUpperCase());
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + compactTag);
            const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'compact');
            if (Object.getPrototypeOf(Type) !== HTMLElement || Object.getPrototypeOf(Type.prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(element) !== Type.prototype || element.localName !== compactTag ||
                element instanceof HTMLUnknownElement || !(prefixed instanceof Type) ||
                descriptor.get.name !== 'get compact' || descriptor.set.name !== 'set compact')
                return 'classification inheritance and accessor names';
            try { new Type(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, true);
            if (!descriptor.get.call(prefixed)) return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', compactTag);
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', compactTag.toUpperCase());
            const other = document.createElement(compactTag === 'dl' ? 'dir' : 'dl');
            const unrelated = document.createElement('div');
            if (foreign instanceof Type || upper instanceof Type) return 'namespace and XML case';
            Object.setPrototypeOf(other, Type.prototype);
            for (const receiver of [null, {}, document, foreign, upper, other, unrelated,
                Object.create(element), new Proxy(element, {})]) {
                let coerced = false;
                try { descriptor.get.call(receiver); return 'getter accepted wrong native family'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                try { descriptor.set.call(receiver, {toString() { coerced = true; throw 0; },
                    valueOf() { coerced = true; throw 0; }}); return 'setter accepted wrong native family'; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return 'receiver and no-coercion contract'; }
            }
            return !other.hasAttributeNS(null, 'compact') && !unrelated.hasAttributeNS(null, 'compact')
                ? 'ok' : 'rejected receiver must not mutate native state';
        })()
    "#,
    );
}

fn compact_reflects_presence_without_value_coercion(tag: &str) {
    check_compact_script(
        tag,
        r#"
        (() => {
            const element = document.createElement(compactTag);
            element.dir = 'rtl';
            if (element.compact !== false || typeof element.dir !== 'string' ||
                Object.hasOwn(Object.getPrototypeOf(element), 'dir')) return 'separate inherited direction';
            element.setAttributeNS('urn:foreign', 'compact', 'foreign');
            if (element.compact) return 'foreign presence ignored';
            for (const content of ['', 'false', '0', 'compact', '\ud800\0\udc00']) {
                element.setAttributeNS(null, 'compact', content);
                if (!element.compact || element.getAttributeNS(null, 'compact') !== content)
                    return 'presence independent of attribute content';
            }
            let coerced = false;
            const object = {toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; },
                [Symbol.toPrimitive]() { coerced = true; throw 0; }};
            for (const value of [true, 'false', '0', 1, -1, Infinity, 1n, -1n, Symbol(), object,
                [], {}, function() {}, new Boolean(false), new Number(0)]) {
                element.compact = value;
                if (element.compact !== true || element.getAttributeNS(null, 'compact') !== '' || coerced ||
                    element.getAttributeNS('urn:foreign', 'compact') !== 'foreign') return 'truthy ToBoolean without coercion';
            }
            for (const value of [false, null, undefined, '', 0, -0, NaN, 0n]) {
                element.compact = true;
                element.compact = value;
                if (element.compact !== false || element.hasAttributeNS(null, 'compact') ||
                    element.getAttributeNS('urn:foreign', 'compact') !== 'foreign')
                    return 'falsy ToBoolean removes only null-namespace attribute';
            }
            return element.dir === 'rtl' && element.getAttributeNS(null, 'dir') === 'rtl'
                ? 'ok' : 'compact must not replace inherited direction';
        })()
    "#,
    );
}

fn compact_customized_constructors_deliver_original_reactions(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_compact_tag(scope, tag)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = compactTag === 'dl' ? HTMLDListElement : HTMLDirectoryElement;
                const calls = [];
                let reenter = false;
                let throwNext = false;
                class Watched extends Type {
                    static observedAttributes = ['compact'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (reenter && newValue === '') { reenter = false; this.compact = false; }
                        if (throwNext) { throwNext = false; direct.compact = true;
                            throw globalThis.compactCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-' + compactTag, Watched, {extends: compactTag});
                const direct = new Watched();
                const element = document.createElement(compactTag, {is: 'x-watched-' + compactTag});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== compactTag || element.localName !== compactTag || direct === element)
                    return 'customized factory and actual constructor tag';
                element.compact = true;
                if (calls.length !== 1 || calls[0][0] !== element || calls[0][1] !== 'compact' ||
                    calls[0][2] !== null || calls[0][3] !== '' || calls[0][4] !== null || calls[0][5] !== '')
                    return 'original synchronous presence reaction';
                element.compact = true;
                element.compact = false;
                if (calls.length !== 3 || calls[1][2] !== '' || calls[1][3] !== '' || calls[1][5] !== '' ||
                    calls[2][2] !== '' || calls[2][3] !== null || calls[2][5] !== null || element.compact)
                    return 'same-value set and removal reactions';
                reenter = true;
                element.compact = true;
                if (calls.length !== 5 || calls[3][2] !== null || calls[3][3] !== '' || calls[3][5] !== '' ||
                    calls[4][2] !== '' || calls[4][3] !== null || calls[4][5] !== null || element.compact)
                    return 'publication before nested Source removal';
                throwNext = true;
                element.compact = true;
                if (calls.length !== 7 || calls[5][0] !== element || calls[5][3] !== '' ||
                    calls[6][0] !== direct || calls[6][2] !== null || calls[6][3] !== '' ||
                    !element.compact || !direct.compact) return 'nested Source survives reported callback throw';
                class Throwing extends Type {
                    constructor() { super(); globalThis.constructedCompact = this;
                        throw globalThis.compactConstructorThrow = {}; }
                }
                customElements.define('x-failed-' + compactTag, Throwing, {extends: compactTag});
                const failed = document.createElement(compactTag, {is: 'x-failed-' + compactTag});
                if (failed !== constructedCompact || !(failed instanceof Throwing) || !(failed instanceof Type) ||
                    failed instanceof HTMLUnknownElement || failed.localName !== compactTag)
                    return 'customized failure keeps authentic native interface';
                const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'compact');
                descriptor.set.call(failed, true);
                return descriptor.get.call(failed) === true ? 'ok' : 'authentic failed-construction receiver';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}");
        Ok(())
    }).unwrap();
    let callback = host
        .take_guest_custom_element_callback_exception(&permit)
        .unwrap()
        .unwrap();
    let constructor = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    assert!(
        host.take_guest_custom_element_callback_exception(&permit)
            .unwrap()
            .is_none()
    );
    assert!(
        host.take_guest_constructor_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.in_document(&permit, |scope| {
        for (expression, exception) in [
            ("compactCallbackThrow", &callback.exception),
            ("compactConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn compact_reads_follow_native_edits_and_retirement(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        set_compact_tag(scope, tag)?;
        evaluate(scope, r#"
            globalThis.compactContainer = document.createElement('div');
            globalThis.nativeCompact = compactContainer.appendChild(document.createElement(compactTag));
            globalThis.savedCompact = nativeCompact;
            globalThis.compactPrototype = Object.getPrototypeOf(nativeCompact);
        "#)?;
        let element = evaluate(scope, "nativeCompact")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, Namespace::none(), "compact", "false");
        batch.set_attribute_ns(
            element,
            Namespace::from_uri("urn:foreign"),
            "compact",
            "foreign",
        );
        batch.set_attribute_ns(element, Namespace::none(), "dir", "RTL");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeCompact === savedCompact && compactContainer.firstChild === nativeCompact &&
            nativeCompact.compact === true && nativeCompact.getAttributeNS(null, 'compact') === 'false' &&
            nativeCompact.getAttributeNS('urn:foreign', 'compact') === 'foreign' && nativeCompact.dir === 'rtl'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.remove_attribute_ns(element, Namespace::none(), "compact");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "!nativeCompact.compact && nativeCompact.getAttributeNS('urn:foreign', 'compact') === 'foreign' && nativeCompact.dir === 'rtl'")?;
        assert!(scope.boolean(&result)?, "{tag}");
        evaluate(scope, r#"
            globalThis.compactDestination = new Document();
            compactDestination.adoptNode(nativeCompact);
        "#)?;
        let result = evaluate(scope, r#"
            nativeCompact === savedCompact && nativeCompact.ownerDocument === compactDestination &&
            nativeCompact.parentNode === null && Object.getPrototypeOf(nativeCompact) === compactPrototype &&
            nativeCompact.localName === compactTag && nativeCompact.compact === false && nativeCompact.dir === 'rtl'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        // Source adoption detached this exact native identity before retirement.
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in ["nativeCompact.compact", "nativeCompact.compact = true"] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired native {tag} must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn compact_accessors_keep_registered_realms_after_adoption(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let element = {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(origin.document(), Namespace::html(), tag);
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "originCompact");
    let foreign_type_error = {
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
                let value = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&value))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        set_compact_tag(scope, tag)?;
        let value = evaluate(scope, "originCompact")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignCompactTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = compactTag === 'dl' ? HTMLDListElement : HTMLDirectoryElement;
                globalThis.savedOriginCompact = originCompact;
                globalThis.originCompactPrototype = Object.getPrototypeOf(originCompact);
                if (originCompact instanceof Type || originCompactPrototype.constructor.name !== Type.name)
                    return 'registered creation realm';
                const descriptor = Object.getOwnPropertyDescriptor(originCompactPrototype, 'compact');
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try { accessor.call({}, true); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof foreignCompactTypeError) || error instanceof TypeError)
                        return 'accessor function realm exception'; }
                }
                let coerced = false;
                const object = {toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; }};
                for (const value of [object, Symbol(), 1n]) {
                    descriptor.set.call(originCompact, value);
                    if (descriptor.get.call(originCompact) !== true || coerced) return 'borrowed ToBoolean no coercion';
                }
                descriptor.set.call(originCompact, 0n);
                if (descriptor.get.call(originCompact) !== false) return 'borrowed falsy BigInt';
                descriptor.set.call(originCompact, true);
                originCompact.dir = 'rtl';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(element, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            originCompact === savedOriginCompact && originCompact.ownerDocument === document &&
            Object.getPrototypeOf(originCompact) === originCompactPrototype &&
            originCompact.localName === compactTag && originCompact.compact === true && originCompact.dir === 'rtl'
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
}

fn compact_setters_admit_source_outputs_before_publication(tag: &str) {
    for outputs in [1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(target.document(), Namespace::html(), tag);
        let element = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedCompact");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "limitedCompact.compact = true").map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short {tag} Source output must refuse before publication: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert_eq!(after.nodes().count(), before.nodes().count());
            assert!(
                after
                    .node(element)
                    .unwrap()
                    .attributes()
                    .unwrap()
                    .is_empty()
            );
        } else {
            result.unwrap();
            let name = "compact".into();
            let attr = after
                .node(element)
                .unwrap()
                .get_attribute_node_ns(&Namespace::none(), &name)
                .unwrap();
            assert!(
                attr.value().code_units().is_empty(),
                "{tag} admitted empty Boolean content"
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

// HTML877d specifies compact as a CEReactions/Reflect boolean on both the
// HTMLDListElement partial interface and the obsolete HTMLDirectoryElement.
pub(super) fn list_compact_reflects_canonical_presence() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, Type] of [['dl', HTMLDListElement], ['dir', HTMLDirectoryElement]]) {
                    const element = document.createElement(tag);
                    if (!(element instanceof Type)) return tag + ' interface';
                    if (Object.keys(Type.prototype).join() !== 'compact')
                        return tag + ' missing reflected compact';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, 'compact');
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' compact descriptor';
                    if (element.compact !== false) return tag + ' absent compact default';
                    element.dir = 'rtl';
                    element.compact = true;
                    if (element.compact !== true || element.getAttributeNS(null, 'compact') !== '')
                        return tag + ' compact presence';
                    element.compact = false;
                    if (element.compact !== false || element.hasAttributeNS(null, 'compact'))
                        return tag + ' compact removal';
                    if (element.dir !== 'rtl' || element.getAttributeNS(null, 'dir') !== 'rtl' ||
                        Object.hasOwn(Type.prototype, 'dir')) return tag + ' inherited direction string';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
