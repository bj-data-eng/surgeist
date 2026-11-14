use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

// Independent authorities: HTML877d BR/HR interfaces and their obsolete partial
// interfaces; HTML reflected DOMString/boolean rules; Web IDL conversion and
// receiver rules. Each case runs through the existing real-V8 entry thread.
pub(super) fn break_element_families_preserve_native_and_web_idl_contracts() {
    break_element_brands_precede_value_conversion();
    break_strings_use_exact_reflection_semantics();
    break_boolean_uses_presence_and_to_boolean();
    break_reflections_deliver_original_customized_reactions();
    break_reflections_observe_native_edits_and_retirement();
    break_reflections_keep_registered_creation_realms_after_adoption();
    break_reflections_admit_source_outputs_before_publication();
}

fn check_break_script(script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, script)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    })
    .unwrap();
}

fn break_element_brands_precede_value_conversion() {
    check_break_script(
        r#"
        (() => {
            for (const [tag, Type, members] of [
                ['br', HTMLBRElement, ['clear']],
                ['hr', HTMLHRElement, ['align', 'color', 'noShade', 'size', 'width']],
            ]) {
                const element = document.createElement(tag);
                const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + tag);
                const other = document.createElement(tag === 'br' ? 'hr' : 'br');
                if (Object.getPrototypeOf(Type) !== HTMLElement ||
                    Object.getPrototypeOf(Type.prototype) !== HTMLElement.prototype ||
                    Object.getPrototypeOf(element) !== Type.prototype || !(prefixed instanceof Type))
                    return tag + ' inheritance or actual local name';
                try { new Type(); return tag + ' unregistered HTMLConstructor'; }
                catch (error) { if (!(error instanceof TypeError)) return tag + ' constructor exception'; }
                const foreign = document.createElementNS('urn:foreign', tag);
                const upper = document.createElementNS('http://www.w3.org/1999/xhtml', tag.toUpperCase());
                if (foreign instanceof Type || upper instanceof Type) return tag + ' native namespace or case';
                // Even an actual native node with the desired JS prototype keeps its original family.
                Object.setPrototypeOf(other, Type.prototype);
                for (const receiver of [null, {}, document, foreign, upper, other,
                    Object.create(element), new Proxy(element, {})]) {
                    for (const member of members) {
                        const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, member);
                        let converted = false;
                        try { descriptor.get.call(receiver); return tag + ' getter brand ' + member; }
                        catch (error) { if (!(error instanceof TypeError)) return tag + ' getter exception'; }
                        try { descriptor.set.call(receiver, {toString() { converted = true; return 'bad'; },
                            valueOf() { converted = true; return 1; }}); return tag + ' setter brand ' + member; }
                        catch (error) { if (!(error instanceof TypeError) || converted)
                            return tag + ' receiver before conversion ' + member; }
                    }
                }
            }
            return 'ok';
        })()
    "#,
    );
}

fn break_strings_use_exact_reflection_semantics() {
    check_break_script(
        r#"
        (() => {
            const exact = ' Raw\ud800\0\udc00 ';
            for (const [tag, members] of [['br', ['clear']], ['hr', ['align', 'color', 'size', 'width']]]) {
                const element = document.createElement(tag);
                for (const member of members) {
                    if (element[member] !== '') return tag + ' absent string ' + member;
                    element.setAttributeNS('urn:foreign', member, 'foreign');
                    if (element[member] !== '') return tag + ' foreign attribute read ' + member;
                    let count = 0;
                    element[member] = {toString() {
                        count++;
                        element.setAttributeNS(null, member, 'nested');
                        if (element[member] !== 'nested') throw 'nested reflection unavailable';
                        return exact;
                    }};
                    if (count !== 1 || element[member] !== exact || element.getAttributeNS(null, member) !== exact ||
                        element.getAttributeNS('urn:foreign', member) !== 'foreign')
                        return tag + ' exact UTF16 and reentry ' + member;
                    const original = {};
                    try { element[member] = {toString() { throw original; }}; return 'throwing string accepted'; }
                    catch (error) { if (error !== original || element[member] !== exact) return 'original throw and state'; }
                    try { element[member] = Symbol(); return 'Symbol string accepted'; }
                    catch (error) { if (!(error instanceof TypeError) || element[member] !== exact) return 'Symbol string state'; }
                    element[member] = null;
                    if (element[member] !== 'null') return 'null DOMString ' + member;
                    element[member] = undefined;
                    if (element[member] !== 'undefined') return 'undefined DOMString ' + member;
                    element.setAttributeNS(null, member, '  INVALID 15% -7px RED BOTH ');
                    if (element[member] !== '  INVALID 15% -7px RED BOTH ') return 'raw string reflection ' + member;
                    element.removeAttributeNS(null, member);
                    if (element[member] !== '' || element.getAttributeNS('urn:foreign', member) !== 'foreign')
                        return 'expanded name removal ' + member;
                }
            }
            return 'ok';
        })()
    "#,
    );
}

fn break_boolean_uses_presence_and_to_boolean() {
    check_break_script(
        r#"
        (() => {
            const hr = document.createElement('hr');
            hr.setAttributeNS('urn:foreign', 'noshade', 'foreign');
            if (hr.noShade) return 'foreign Boolean ignored';
            let coerced = false;
            const object = {toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; },
                [Symbol.toPrimitive]() { coerced = true; throw 0; }};
            for (const value of [true, 'false', 1, -1, Infinity, Symbol(), object, [], {}]) {
                hr.noShade = value;
                if (!hr.noShade || hr.getAttributeNS(null, 'noshade') !== '' || coerced ||
                    hr.getAttributeNS('urn:foreign', 'noshade') !== 'foreign') return 'ToBoolean truthy';
            }
            hr.setAttributeNS(null, 'noshade', 'false');
            if (!hr.noShade) return 'Boolean presence ignores content';
            for (const value of [false, undefined, null, 0, -0, NaN, '']) {
                hr.noShade = true;
                hr.noShade = value;
                if (hr.noShade || hr.hasAttributeNS(null, 'noshade') ||
                    hr.getAttributeNS('urn:foreign', 'noshade') !== 'foreign') return 'ToBoolean falsy';
            }
            return 'ok';
        })()
    "#,
    );
}

fn break_reflections_deliver_original_customized_reactions() {
    check_break_script(
        r#"
        (() => {
            const calls = [];
            class WatchedBR extends HTMLBRElement {
                static observedAttributes = ['clear'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                    if (newValue === 'outer') this.clear = 'inner';
                }
            }
            class WatchedHR extends HTMLHRElement {
                static observedAttributes = ['align', 'color', 'noshade', 'size', 'width'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                }
            }
            customElements.define('x-break-br', WatchedBR, {extends: 'br'});
            customElements.define('x-break-hr', WatchedHR, {extends: 'hr'});
            const br = document.createElement('br', {is: 'x-break-br'});
            const hr = new WatchedHR();
            if (!(br instanceof WatchedBR) || !(hr instanceof WatchedHR) || hr.localName !== 'hr')
                return 'existing customized HTMLConstructor path';
            for (const [element, member, attr] of [[br, 'clear', 'clear'], [hr, 'align', 'align'],
                [hr, 'color', 'color'], [hr, 'noShade', 'noshade'], [hr, 'size', 'size'], [hr, 'width', 'width']]) {
                const count = calls.length;
                const value = member === 'noShade' ? '' : 'exact\ud800\0';
                element[member] = member === 'noShade' ? true : value;
                const call = calls[count];
                if (calls.length !== count + 1 || call[0] !== element || call[1] !== attr ||
                    call[2] !== null || call[3] !== value || call[4] !== null || call[5] !== value)
                    return 'synchronous original reaction ' + member;
            }
            const count = calls.length;
            hr.noShade = true;
            hr.noShade = false;
            if (calls.length !== count + 2 || calls[count][2] !== '' || calls[count][3] !== '' ||
                calls[count + 1][2] !== '' || calls[count + 1][3] !== null || hr.noShade)
                return 'Boolean change and removal reactions';
            br.clear = 'outer';
            if (calls.length !== count + 4 || calls[count + 2][3] !== 'outer' ||
                calls[count + 2][5] !== 'outer' || calls[count + 3][2] !== 'outer' ||
                calls[count + 3][3] !== 'inner' || calls[count + 3][5] !== 'inner' || br.clear !== 'inner')
                return 'reaction publication before callback reentry';
            return 'ok';
        })()
    "#,
    );
}

fn break_reflections_observe_native_edits_and_retirement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let (br, hr) = host
        .in_document(&permit, |scope| {
            evaluate(
                scope,
                r#"
            globalThis.breakContainer = document.createElement('div');
            globalThis.nativeBR = breakContainer.appendChild(document.createElement('br'));
            globalThis.nativeHR = breakContainer.appendChild(document.createElement('hr'));
            globalThis.savedBR = nativeBR;
            globalThis.savedHR = nativeHR;
        "#,
            )?;
            let br = evaluate(scope, "nativeBR")?;
            let hr = evaluate(scope, "nativeHR")?;
            Ok((
                projected_node_id(scope, &br)?,
                projected_node_id(scope, &hr)?,
            ))
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(br, Namespace::none(), "clear", "native\0\u{fffd}");
        for attr in ["align", "color", "size", "width"] {
            batch.set_attribute_ns(hr, Namespace::none(), attr, " Native RAW -5% ");
        }
        batch.set_attribute_ns(hr, Namespace::none(), "noshade", "false");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeBR === savedBR && nativeHR === savedHR && breakContainer.firstChild === nativeBR &&
            breakContainer.lastChild === nativeHR && nativeBR.clear === 'native\0\ufffd' &&
            ['align', 'color', 'size', 'width'].every(member => nativeHR[member] === ' Native RAW -5% ') &&
            nativeHR.noShade
        "#)?;
        assert!(scope.boolean(&result)?);
        evaluate(scope, r#"
            globalThis.breakDestination = new Document();
            breakDestination.adoptNode(nativeBR);
            breakDestination.adoptNode(nativeHR);
        "#)?;
        let result = evaluate(scope, r#"
            nativeBR === savedBR && nativeHR === savedHR && nativeBR.ownerDocument === breakDestination &&
            nativeHR.ownerDocument === breakDestination && nativeBR.parentNode === null && nativeHR.parentNode === null &&
            nativeBR instanceof HTMLBRElement && nativeHR instanceof HTMLHRElement &&
            nativeBR.clear === 'native\0\ufffd' && nativeHR.width === ' Native RAW -5% ' && nativeHR.noShade
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_ne!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        // Adoption already detached these nodes; retire only those exact identities.
        batch.destroy(br);
        batch.destroy(hr);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeBR.clear",
        "nativeBR.clear = 'retired'",
        "nativeHR.align",
        "nativeHR.color",
        "nativeHR.noShade",
        "nativeHR.noShade = true",
        "nativeHR.size",
        "nativeHR.width",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired break identity must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn break_reflections_keep_registered_creation_realms_after_adoption() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let (br, hr) = {
        let mut batch = DomBatch::new();
        let br = batch.create_element_ns(origin.document(), Namespace::html(), "br");
        let hr = batch.create_element_ns(origin.document(), Namespace::html(), "hr");
        let commit = session.dom().apply(batch, &cancel).unwrap();
        (
            commit.created(&br).unwrap().unwrap(),
            commit.created(&hr).unwrap().unwrap(),
        )
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, br, "originBR");
    expose_public_node_for_source_test(&mut host, &permit, hr, "originHR");
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
        let br = evaluate(scope, "originBR")?;
        let hr = evaluate(scope, "originHR")?;
        assert_ne!(scope.callback_object(&br)?.associated_realm().identity(), permit.realm);
        assert_eq!(scope.callback_object(&br)?.associated_realm().identity(),
            scope.callback_object(&hr)?.associated_realm().identity());
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignBreakTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                globalThis.originalBRPrototype = Object.getPrototypeOf(originBR);
                globalThis.originalHRPrototype = Object.getPrototypeOf(originHR);
                if (originBR instanceof HTMLBRElement || originHR instanceof HTMLHRElement ||
                    originalBRPrototype.constructor.name !== 'HTMLBRElement' ||
                    originalHRPrototype.constructor.name !== 'HTMLHRElement') return 'registered wrapper realm';
                for (const [element, prototype, members] of [[originBR, originalBRPrototype, ['clear']],
                    [originHR, originalHRPrototype, ['align', 'color', 'noShade', 'size', 'width']]]) {
                    for (const member of members) {
                        const descriptor = Object.getOwnPropertyDescriptor(prototype, member);
                        for (const accessor of [descriptor.get, descriptor.set]) {
                            try { accessor.call({}, 'value'); return 'borrowed accessor brand'; }
                            catch (error) { if (!(error instanceof foreignBreakTypeError) || error instanceof TypeError)
                                return 'accessor function realm TypeError'; }
                        }
                        descriptor.set.call(element, member === 'noShade' ? true : 'foreign\ud800\0');
                        if (descriptor.get.call(element) !== (member === 'noShade' ? true : 'foreign\ud800\0'))
                            return 'borrowed authentic accessor';
                        if (member !== 'noShade') {
                            const original = {};
                            try { descriptor.set.call(element, {toString() { throw original; }}); return 'borrowed conversion throw'; }
                            catch (error) { if (error !== original) return 'borrowed original exception identity'; }
                        }
                    }
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(br, target.document());
        batch.adopt(hr, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            originBR.ownerDocument === document && originHR.ownerDocument === document &&
            Object.getPrototypeOf(originBR) === originalBRPrototype && Object.getPrototypeOf(originHR) === originalHRPrototype &&
            !(originBR instanceof HTMLBRElement) && !(originHR instanceof HTMLHRElement) &&
            originBR.clear === 'foreign\ud800\0' && originHR.color === 'foreign\ud800\0' && originHR.noShade
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
}

fn break_reflections_admit_source_outputs_before_publication() {
    for (tag, property, attribute, script, expected) in [
        (
            "br",
            "clear",
            "clear",
            "limitedBreak.clear = '\\ud800\\0'",
            &[0xd800, 0][..],
        ),
        (
            "hr",
            "noShade",
            "noshade",
            "limitedBreak.noShade = true",
            &[][..],
        ),
    ] {
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
            expose_public_node_for_source_test(&mut host, &permit, element, "limitedBreak");
            let before = host.snapshot().unwrap();
            let result = host.in_document(&permit, |scope| evaluate(scope, script).map(drop));
            let after = host.snapshot().unwrap();
            if outputs == 1 {
                let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                    panic!(
                        "one-short {property} Source output must refuse before publication: {result:?}"
                    );
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
                let name = attribute.into();
                let attr = after
                    .node(element)
                    .unwrap()
                    .get_attribute_node_ns(&Namespace::none(), &name)
                    .unwrap();
                assert_eq!(
                    attr.value().code_units(),
                    expected,
                    "{property} admitted native value"
                );
            }
            host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
                .unwrap();
            host.stop().into_session().unwrap();
        }
    }
}

pub(super) fn break_elements_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, name, members] of [
                    ['br', 'HTMLBRElement', ['clear']],
                    ['hr', 'HTMLHRElement', ['align', 'color', 'noShade', 'size', 'width']],
                ]) {
                    const element = document.createElement(tag);
                    const type = globalThis[name];
                    if (!(element instanceof type)) return tag + ' interface';
                    if (Object.keys(type.prototype).join() !== members.join())
                        return tag + ' missing reflected members';
                    for (const member of members) {
                        const descriptor = Object.getOwnPropertyDescriptor(type.prototype, member);
                        if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                            !descriptor.enumerable || !descriptor.configurable ||
                            descriptor.get.length !== 0 || descriptor.set.length !== 1)
                            return tag + ' descriptor ' + member;
                        const attribute = member === 'noShade' ? 'noshade' : member;
                        const boolean = member === 'noShade';
                        if (element[member] !== (boolean ? false : '')) return tag + ' default ' + member;
                        element[member] = boolean ? true : 'value';
                        if (element[member] !== (boolean ? true : 'value') ||
                            element.getAttributeNS(null, attribute) !== (boolean ? '' : 'value'))
                            return tag + ' canonical reflection ' + member;
                    }
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
