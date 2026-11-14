use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn font_family_preserves_native_and_web_idl_contracts() {
    font_brands_precede_conversion_and_preserve_obsolete_classification();
    font_conversion_distinguishes_actual_null_from_object_coercion();
    font_reflections_deliver_original_customized_reactions();
    failed_customized_font_construction_keeps_its_authentic_interface();
    font_reflections_observe_native_edits_and_retirement();
    font_reflections_keep_creation_and_accessor_realms_after_adoption();
    font_reflections_admit_source_outputs_before_publication();
}

fn check_font_script(script: &str) {
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

fn font_brands_precede_conversion_and_preserve_obsolete_classification() {
    check_font_script(
        r#"
        (() => {
            const font = document.createElement('FONT');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:font');
            if (!(font instanceof HTMLFontElement) || font instanceof HTMLUnknownElement ||
                !(prefixed instanceof HTMLFontElement) || font.localName !== 'font' ||
                Object.getPrototypeOf(font) !== HTMLFontElement.prototype ||
                Object.getPrototypeOf(HTMLFontElement) !== HTMLElement ||
                Object.getPrototypeOf(HTMLFontElement.prototype) !== HTMLElement.prototype)
                return 'obsolete font classification and inheritance';
            try { new HTMLFontElement(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            const foreign = document.createElementNS('urn:foreign', 'font');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'FONT');
            const other = document.createElement('hr');
            Object.setPrototypeOf(other, HTMLFontElement.prototype);
            if (foreign instanceof HTMLFontElement || upper instanceof HTMLFontElement)
                return 'namespace and case-sensitive classification';
            for (const member of ['color', 'face', 'size']) {
                const descriptor = Object.getOwnPropertyDescriptor(HTMLFontElement.prototype, member);
                if (descriptor.get.name !== 'get ' + member || descriptor.set.name !== 'set ' + member)
                    return 'accessor function names ' + member;
                descriptor.set.call(prefixed, 'prefixed');
                if (descriptor.get.call(prefixed) !== 'prefixed') return 'prefixed authentic receiver';
                for (const receiver of [null, {}, document, other, foreign, upper,
                    Object.create(font), new Proxy(font, {})]) {
                    let converted = false;
                    try { descriptor.get.call(receiver); return 'getter brand ' + member; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                    try { descriptor.set.call(receiver, {toString() { converted = true; return 'bad'; }});
                        return 'setter brand ' + member; }
                    catch (error) { if (!(error instanceof TypeError) || converted)
                        return 'receiver before DOMString conversion ' + member; }
                }
            }
            return 'ok';
        })()
    "#,
    );
}

fn font_conversion_distinguishes_actual_null_from_object_coercion() {
    check_font_script(
        r#"
        (() => {
            const font = document.createElement('font');
            const exact = ' Raw\ud800\0\udc00 ';
            for (const member of ['color', 'face', 'size']) {
                if (font[member] !== '') return 'absent default ' + member;
                font.setAttributeNS('urn:foreign', member, 'foreign');
                if (font[member] !== '') return 'foreign attribute ignored ' + member;
                font[member] = null;
                const nullValue = member === 'color' ? '' : 'null';
                if (font[member] !== nullValue || font.getAttributeNS(null, member) !== nullValue)
                    return 'actual null conversion ' + member;
                let coercions = 0;
                font[member] = {toString() { coercions++; return null; }};
                if (coercions !== 1 || font[member] !== 'null' || font.getAttributeNS(null, member) !== 'null')
                    return 'object returning null uses ordinary ToString ' + member;
                font[member] = undefined;
                if (font[member] !== 'undefined') return 'undefined conversion ' + member;
                font[member] = {toString() { return undefined; }};
                if (font[member] !== 'undefined') return 'object returning undefined ' + member;
                font[member] = {toString() {
                    font.setAttributeNS(null, member, 'nested');
                    if (font[member] !== 'nested') throw 'nested reflection unavailable';
                    return exact;
                }};
                if (font[member] !== exact || font.getAttributeNS(null, member) !== exact ||
                    font.getAttributeNS('urn:foreign', member) !== 'foreign')
                    return 'exact UTF16 namespace isolation and reentry ' + member;
                const original = {};
                try { font[member] = {toString() { throw original; }}; return 'throwing value accepted'; }
                catch (error) { if (error !== original || font[member] !== exact) return 'original exception and state'; }
                for (const value of [Symbol(), {toString() { return Symbol(); }}]) {
                    try { font[member] = value; return 'Symbol value accepted'; }
                    catch (error) { if (!(error instanceof TypeError) || font[member] !== exact) return 'Symbol state'; }
                }
                const raw = member === 'color' ? ' NOT-A-COLOR #aBcDeF ' :
                    member === 'face' ? ' "Case Sensitive", unlisted font ' : ' +99px nonsense ';
                font.setAttributeNS(null, member, raw);
                if (font[member] !== raw) return 'raw content is not parsed or canonicalized ' + member;
                font.removeAttributeNS(null, member);
                if (font[member] !== '' || font.getAttributeNS('urn:foreign', member) !== 'foreign')
                    return 'expanded-name removal and absent default ' + member;
            }
            return 'ok';
        })()
    "#,
    );
}

fn font_reflections_deliver_original_customized_reactions() {
    check_font_script(
        r#"
        (() => {
            const calls = [];
            class WatchedFont extends HTMLFontElement {
                static observedAttributes = ['color', 'face', 'size'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                    if (name === 'face' && newValue === 'outer') this.face = 'inner';
                }
            }
            customElements.define('x-watched-font', WatchedFont, {extends: 'font'});
            const font = document.createElement('font', {is: 'x-watched-font'});
            const direct = new WatchedFont();
            if (!(font instanceof WatchedFont) || !(direct instanceof WatchedFont) || direct.localName !== 'font')
                return 'customized built-in HTMLConstructor';
            for (const member of ['color', 'face', 'size']) {
                const count = calls.length;
                const expected = member === 'color' ? '' : 'null';
                font[member] = null;
                const first = calls[count];
                if (calls.length !== count + 1 || first[0] !== font || first[1] !== member ||
                    first[2] !== null || first[3] !== expected || first[4] !== null || first[5] !== expected)
                    return 'original null-converted reaction ' + member;
                font[member] = {toString() { return null; }};
                const second = calls[count + 1];
                if (calls.length !== count + 2 || second[2] !== expected || second[3] !== 'null' ||
                    second[4] !== null || second[5] !== 'null') return 'object conversion reaction ' + member;
            }
            const count = calls.length;
            font.face = 'outer';
            if (calls.length !== count + 2 || calls[count][2] !== 'null' || calls[count][3] !== 'outer' ||
                calls[count][5] !== 'outer' || calls[count + 1][2] !== 'outer' || calls[count + 1][3] !== 'inner' ||
                calls[count + 1][5] !== 'inner' || font.face !== 'inner') return 'publication before callback reentry';
            const original = {};
            try { font.color = {toString() { font.size = 'nested'; throw original; }}; return 'coercion throw'; }
            catch (error) { if (error !== original) return 'reaction reentry original exception'; }
            if (calls.length !== count + 3 || calls[count + 2][1] !== 'size' ||
                calls[count + 2][3] !== 'nested' || font.color !== 'null' || font.size !== 'nested')
                return 'nested Source reaction survives outer conversion throw';
            return 'ok';
        })()
    "#,
    );
}

fn failed_customized_font_construction_keeps_its_authentic_interface() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                class ThrowingFont extends HTMLFontElement {
                    constructor() { super(); globalThis.constructedFont = this;
                        throw globalThis.fontConstructionThrow = {}; }
                }
                customElements.define('x-throwing-font', ThrowingFont, {extends: 'font'});
                const font = document.createElement('font', {is: 'x-throwing-font'});
                if (font !== constructedFont || !(font instanceof ThrowingFont) ||
                    !(font instanceof HTMLFontElement) || font instanceof HTMLUnknownElement)
                    return 'customized failure retains authentic font interface';
                for (const member of ['color', 'face', 'size']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLFontElement.prototype, member);
                    descriptor.set.call(font, null);
                    if (descriptor.get.call(font) !== (member === 'color' ? '' : 'null'))
                        return 'authentic failed customized receiver ' + member;
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let report = host
        .take_guest_constructor_exception(&permit)
        .unwrap()
        .unwrap();
    assert!(
        host.take_guest_constructor_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.in_document(&permit, |scope| {
        let original = evaluate(scope, "fontConstructionThrow")?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
}

fn font_reflections_observe_native_edits_and_retirement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let font = host
        .in_document(&permit, |scope| {
            evaluate(
                scope,
                r#"
            globalThis.fontContainer = document.createElement('div');
            globalThis.nativeFont = fontContainer.appendChild(document.createElement('font'));
            globalThis.savedFont = nativeFont;
        "#,
            )?;
            let font = evaluate(scope, "nativeFont")?;
            projected_node_id(scope, &font)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        for member in ["color", "face", "size"] {
            batch.set_attribute_ns(
                font,
                Namespace::none(),
                member,
                surgeist_dom::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
            );
            batch.set_attribute_ns(font, Namespace::from_uri("urn:foreign"), member, "foreign");
        }
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"
            nativeFont === savedFont && fontContainer.firstChild === nativeFont &&
            ['color', 'face', 'size'].every(member => nativeFont[member] === '\ud800\0\udc00' &&
                nativeFont.getAttributeNS('urn:foreign', member) === 'foreign')
        "#,
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.fontDestination = new Document();
            fontDestination.adoptNode(nativeFont);
        "#)?;
        let result = evaluate(scope, r#"
            nativeFont === savedFont && nativeFont.ownerDocument === fontDestination && nativeFont.parentNode === null &&
            nativeFont instanceof HTMLFontElement && Object.getPrototypeOf(nativeFont) === HTMLFontElement.prototype &&
            ['color', 'face', 'size'].every(member => nativeFont[member] === '\ud800\0\udc00')
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        // Source adoption detached the node before this exact native retirement.
        batch.destroy(font);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeFont.color",
        "nativeFont.face",
        "nativeFont.size",
        "nativeFont.color = null",
        "nativeFont.face = 'retired'",
        "nativeFont.size = 'retired'",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired native font must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn font_reflections_keep_creation_and_accessor_realms_after_adoption() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let font = {
        let mut batch = DomBatch::new();
        let font = batch.create_element_ns(origin.document(), Namespace::html(), "font");
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&font)
            .unwrap()
            .unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, font, "originFont");
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
        let font = evaluate(scope, "originFont")?;
        assert_ne!(scope.callback_object(&font)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignFontTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                globalThis.savedOriginFont = originFont;
                globalThis.originFontPrototype = Object.getPrototypeOf(originFont);
                if (originFont instanceof HTMLFontElement || originFontPrototype.constructor.name !== 'HTMLFontElement')
                    return 'registered font creation realm';
                for (const member of ['color', 'face', 'size']) {
                    const descriptor = Object.getOwnPropertyDescriptor(originFontPrototype, member);
                    for (const accessor of [descriptor.get, descriptor.set]) {
                        try { accessor.call({}, null); return 'borrowed receiver accepted'; }
                        catch (error) { if (!(error instanceof foreignFontTypeError) || error instanceof TypeError)
                            return 'accessor function realm exception'; }
                    }
                    descriptor.set.call(originFont, null);
                    if (descriptor.get.call(originFont) !== (member === 'color' ? '' : 'null'))
                        return 'borrowed accessor actual null conversion';
                    descriptor.set.call(originFont, {toString() { return null; }});
                    if (descriptor.get.call(originFont) !== 'null') return 'borrowed object null conversion';
                    const original = {};
                    try { descriptor.set.call(originFont, {toString() { throw original; }}); return 'borrowed conversion throw'; }
                    catch (error) { if (error !== original) return 'borrowed original throw identity'; }
                    descriptor.set.call(originFont, 'foreign\ud800\0');
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(font, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            originFont === savedOriginFont && originFont.ownerDocument === document &&
            Object.getPrototypeOf(originFont) === originFontPrototype && !(originFont instanceof HTMLFontElement) &&
            ['color', 'face', 'size'].every(member => originFont[member] === 'foreign\ud800\0')
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
}

fn font_reflections_admit_source_outputs_before_publication() {
    for (property, script, expected) in [
        ("color", "limitedFont.color = null", &[][..]),
        (
            "face",
            "limitedFont.face = null",
            &[0x6e, 0x75, 0x6c, 0x6c][..],
        ),
        ("size", "limitedFont.size = -3", &[0x2d, 0x33][..]),
    ] {
        for outputs in [1, 2] {
            let cancel = Arc::new(AtomicBool::new(false));
            let (mut session, target) = hosted_session(&cancel, 0);
            let mut batch = DomBatch::new();
            let font = batch.create_element_ns(target.document(), Namespace::html(), "font");
            let font = session
                .dom()
                .apply(batch, &cancel)
                .unwrap()
                .created(&font)
                .unwrap()
                .unwrap();
            let mut host =
                attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
            let permit = host.permit().unwrap();
            expose_public_node_for_source_test(&mut host, &permit, font, "limitedFont");
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
                assert!(after.node(font).unwrap().attributes().unwrap().is_empty());
            } else {
                result.unwrap();
                let name = property.into();
                let attr = after
                    .node(font)
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

// HTML877d obsolete-elements HTMLFontElement IDL independently specifies this
// declaration order, reflection, and color's LegacyNullToEmptyString conversion.
pub(super) fn font_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const font = document.createElement('font');
                if (!(font instanceof HTMLFontElement)) return 'font interface';
                const members = ['color', 'face', 'size'];
                if (Object.keys(HTMLFontElement.prototype).join() !== members.join())
                    return 'font missing reflected members';
                for (const member of members) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLFontElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'font descriptor ' + member;
                    if (font[member] !== '') return 'font absent default ' + member;
                    font[member] = null;
                    const expected = member === 'color' ? '' : 'null';
                    if (font[member] !== expected || font.getAttributeNS(null, member) !== expected)
                        return 'font null reflection ' + member;
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
