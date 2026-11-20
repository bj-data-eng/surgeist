use super::{
    DocumentGuestEventError, DocumentScriptError, DocumentScriptNativeError, DomBatch, DomError,
    Error, Namespace, PropertyKey, evaluate, hosted_session, hosted_session_with_dom_limits,
};
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use std::sync::{Arc, atomic::AtomicBool};

// HTML877d 58387..58393 declares disabled Boolean and label DOMString
// reflections. A detached group reaches no ancestor-select reset operation.
pub(super) fn optgroup_members_reflect_their_own_native_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const group = document.createElement('optgroup');
                if (!(group instanceof HTMLOptGroupElement) || group.localName !== 'optgroup')
                    return 'native optgroup interface';
                const parent = document.body;
                if (parent.appendChild(group) !== group || group.parentNode !== parent)
                    return 'native optgroup body parent';
                parent.removeChild(group);
                if (group.parentNode !== null) return 'detached optgroup fixture';
                for (const member of ['disabled', 'label']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLOptGroupElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function')
                        return 'optgroup missing reflected member ' + member;
                    if (!descriptor.enumerable || !descriptor.configurable || descriptor.get.length !== 0 ||
                        descriptor.set.length !== 1 || descriptor.get.name !== 'get ' + member ||
                        descriptor.set.name !== 'set ' + member) return 'optgroup accessor descriptor ' + member;
                }
                if (Object.keys(HTMLOptGroupElement.prototype).join() !== 'disabled,label')
                    return 'optgroup declaration order';
                if (group.disabled !== false || group.label !== '') return 'optgroup absent defaults';
                group.disabled = true;
                if (group.disabled !== true || group.getAttributeNS(null, 'disabled') !== '')
                    return 'own disabled presence';
                group.setAttributeNS(null, 'disabled', 'false');
                if (!group.disabled) return 'false-looking disabled text is present';
                group.disabled = false;
                if (group.disabled !== false || group.hasAttributeNS(null, 'disabled'))
                    return 'disabled false removes own attribute';
                group.label = null;
                if (group.label !== 'null' || group.getAttributeNS(null, 'label') !== 'null')
                    return 'label ordinary DOMString null';
                const raw = ' Raw\ud800\0\udc00 ';
                group.label = raw;
                return group.label === raw && group.getAttributeNS(null, 'label') === raw &&
                    group.label.length === 8 && group.label.charCodeAt(4) === 0xd800 &&
                    group.label.charCodeAt(5) === 0 && group.label.charCodeAt(6) === 0xdc00
                    ? 'ok' : 'label exact raw UTF16';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

pub(super) fn optgroup_bindings_preserve_brands_and_reactions() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits
            .with_select_processing(16, 256_000, 1_000_000)
            .unwrap()
    });
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const html = 'http://www.w3.org/1999/xhtml';
                const xml = document.implementation.createDocument(null, '');
                const prefixed = xml.createElementNS(html, 'h:optgroup');
                const namespaced = xml.createElementNS('urn:foreign', 'optgroup');
                const upper = xml.createElementNS(html, 'OPTGROUP');
                const prototype = HTMLOptGroupElement.prototype;
                const group = document.createElement('optgroup');
                const select = document.createElement('select');
                select.setAttributeNS(null, 'disabled', '');
                select.appendChild(group);
                group.appendChild(document.createElement('legend')).textContent = 'legend';
                group.setAttributeNS('urn:foreign', 'x:disabled', 'foreign');
                group.setAttributeNS('urn:foreign', 'x:label', 'foreign');
                if (group.disabled || group.label !== '') return 'own attributes only';
                select.removeChild(group);
                if (!(prefixed instanceof HTMLOptGroupElement) || prefixed.prefix !== 'h')
                    return 'prefixed HTML interface';
                for (const member of ['disabled', 'label']) {
                    const {get, set} = Object.getOwnPropertyDescriptor(prototype, member);
                    for (const receiver of [namespaced, upper, {}, Object.create(prototype), new Proxy(group, {})]) {
                        let converted = false;
                        const input = {[Symbol.toPrimitive]() { converted = true; throw 0; }};
                        try { get.call(receiver); return 'invalid getter receiver ' + member; }
                        catch (error) { if (!(error instanceof TypeError)) return 'getter brand type'; }
                        try { set.call(receiver, input); return 'invalid setter receiver ' + member; }
                        catch (error) { if (!(error instanceof TypeError) || converted) return 'brand before conversion'; }
                    }
                    set.call(prefixed, member === 'disabled' ? true : '');
                    if (get.call(prefixed) !== (member === 'disabled' ? true : ''))
                        return 'borrowed accessor on prefixed OptGroup';
                }
                group.disabled = {[Symbol.toPrimitive]() { throw 'unexpected Boolean conversion'; }};
                if (!group.disabled) return 'Boolean object truthiness';
                group.label = undefined;
                if (group.label !== 'undefined') return 'ordinary undefined conversion';
                const original = {};
                try { group.label = {[Symbol.toPrimitive]() { throw original; }}; return 'lost conversion exception'; }
                catch (error) { if (error !== original || group.label !== 'undefined') return 'conversion exception identity'; }
                group.label = {[Symbol.toPrimitive]() {
                    group.label = 'inner';
                    return group.label === 'inner' ? 'outer' : 'lost nested assignment';
                }};
                if (group.label !== 'outer') return 'reentrant conversion';
                const calls = [];
                class Watched extends HTMLOptGroupElement {
                    static observedAttributes = ['label', 'disabled'];
                    attributeChangedCallback(name, oldValue, value, namespace) {
                        calls.push([this, name, oldValue, value, namespace, this.getAttributeNS(null, name)]);
                        if (name === 'disabled' && value === '') this.label = 'during reaction';
                    }
                }
                customElements.define('x-watched-optgroup', Watched, {extends: 'optgroup'});
                const direct = new Watched();
                const factory = document.createElement('optgroup', {is: 'x-watched-optgroup'});
                if (!(direct instanceof Watched) || !(factory instanceof Watched) ||
                    direct === factory || direct.localName !== 'optgroup' || factory.localName !== 'optgroup')
                    return 'customized native constructors';
                factory.disabled = true;
                factory.disabled = false;
                direct.label = null;
                const expected = [
                    [factory, 'disabled', null, '', null, ''],
                    [factory, 'label', null, 'during reaction', null, 'during reaction'],
                    [factory, 'disabled', '', null, null, null],
                    [direct, 'label', null, 'null', null, 'null'],
                ];
                if (calls.length !== expected.length ||
                    !calls.every((row, index) => row.every((value, column) => value === expected[index][column])))
                    return 'original nested attribute reactions';
                class Throwing extends HTMLOptGroupElement {
                    constructor() { super(); globalThis.failedOptGroup = this; throw {}; }
                }
                customElements.define('x-failed-optgroup', Throwing, {extends: 'optgroup'});
                const failed = document.createElement('optgroup', {is: 'x-failed-optgroup'});
                if (failed !== failedOptGroup || !(failed instanceof HTMLOptGroupElement) || failed instanceof HTMLUnknownElement)
                    return 'failed customized constructor retains its native interface';
                failed.label = 'retained';
                return failed.label === 'retained' ? 'ok' : 'failed customized reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    assert!(
        host.take_guest_constructor_exception(&permit)
            .unwrap()
            .is_some()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
}

pub(super) fn optgroup_bindings_follow_realm_adoption_and_retirement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut batch = DomBatch::new();
    let node = batch.create_element_ns(origin.document(), Namespace::html(), "optgroup");
    let node = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&node)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, node, "originOptGroup");
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
                let error = evaluate(scope, "TypeError")?;
                Ok(scope.retain(&error))
            })
            .unwrap()
    };
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"optGroupOriginTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"(() => {
            globalThis.optGroupOriginPrototype = Object.getPrototypeOf(originOptGroup);
            if (originOptGroup instanceof HTMLOptGroupElement ||
                optGroupOriginPrototype.constructor.name !== 'HTMLOptGroupElement') return 'creation realm';
            for (const [name, value] of [['disabled', true], ['label', 'foreign']]) {
                const {get, set} = Object.getOwnPropertyDescriptor(optGroupOriginPrototype, name);
                for (const accessor of [get, set]) {
                    try { accessor.call({}, value); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof optGroupOriginTypeError) || error instanceof TypeError)
                        return 'wrong error realm'; }
                }
                set.call(originOptGroup, value);
                if (get.call(originOptGroup) !== value) return 'borrowed reflection';
            }
            return 'ok';
        })()"#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(node, target.document());
        batch.set_attribute_ns(node, Namespace::none(), "label", "native");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"originOptGroup.ownerDocument === document &&
            Object.getPrototypeOf(originOptGroup) === optGroupOriginPrototype &&
            originOptGroup.disabled && originOptGroup.label === 'native'"#,
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(node);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "originOptGroup.disabled",
        "originOptGroup.label = 'retired'",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired OptGroup must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}
