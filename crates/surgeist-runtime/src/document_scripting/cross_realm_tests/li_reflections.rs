use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn li_family_preserves_native_and_web_idl_contracts() {
    // The same aggregate runs Pre's shared signed-long raw-unit/physical/
    // cancellation checks and the existing DOMString admission regressions.
    li_value_reflects_attributes_in_real_native_lists();
    for member in ["value", "type"] {
        li_brands_precede_conversion(member);
        li_accessors_keep_registered_realms_after_adoption(member);
        li_setters_admit_original_source_outputs(member);
    }
    for cases in [
        r#"[['',0],['invalid',0],[' \t\n\f\r-7tail',-7],['+007junk',7],['0x10',0],['1e2',1]]"#,
        r#"[['2147483647',2147483647],['-2147483648',-2147483648],['2147483648',0],['-2147483649',0],['999999999999999999999999',0],['\u00a07',0]]"#,
    ] {
        li_signed_getter_cases(cases);
    }
    for cases in [
        "[[1.9,1],[-1.9,-1],[2147483648,-2147483648],[-2147483649,2147483647],[4294967295,-1],[4294967296,0]]",
        "[[4294967297,1],[-4294967297,-1],[NaN,0],[Infinity,0],[-Infinity,0],[-0,0],[null,0],[undefined,0]]",
    ] {
        li_wrapping_setter_cases(cases);
    }
    li_numeric_coercion_preserves_hints_reentry_and_original_throws();
    li_type_reflects_exact_raw_strings();
    li_customized_constructors_deliver_original_reactions();
    li_native_edits_and_retirement_keep_wrapper_identity();
}

fn set_li_member(scope: &mut Scope<'_>, member: &str) -> Result<(), Error> {
    let global = scope.global()?;
    let value = scope.string(&member.encode_utf16().collect::<Vec<_>>())?;
    scope.set(
        &global,
        PropertyKey::String(&"liMember".encode_utf16().collect::<Vec<_>>()),
        &value,
    )
}

fn check_li_script(member: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_li_member(scope, member)?;
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "li.{member}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "li.{member}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("li.{member}: {error:?}"),
    });
}

fn li_value_reflects_attributes_in_real_native_lists() {
    // Literal oracle: HTML877d's OL example has reflected values 0,3,0,
    // although the list ordinals described by the separate algorithm are 1,3,4.
    for parent_tag in [Some("ol"), Some("ul"), Some("menu"), None] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let parent = parent_tag
            .map(|tag| batch.create_element_ns(target.document(), Namespace::html(), tag));
        let items = (0..3)
            .map(|_| batch.create_element_ns(target.document(), Namespace::html(), "li"))
            .collect::<Vec<_>>();
        for (item, text) in items.iter().zip(["Item 1", "Item 3", "Item 4"]) {
            let text = batch.create_text(target.document(), text);
            batch.insert_before(&text, item, None);
        }
        batch.set_attribute_ns(&items[1], Namespace::none(), "value", "3");
        if let Some(parent) = &parent {
            for item in &items {
                batch.insert_before(item, parent, None);
            }
        }
        let commit = session.dom().apply(batch, &cancel).unwrap();
        let items = items
            .iter()
            .map(|item| commit.created(item).unwrap().unwrap())
            .collect::<Vec<_>>();
        if let Some(parent) = &parent {
            let parent = commit.created(parent).unwrap().unwrap();
            assert_eq!(
                session.snapshot().node(parent).unwrap().children(),
                items.as_slice()
            );
        }
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        for (item, name) in items
            .into_iter()
            .zip(["firstItem", "middleItem", "lastItem"])
        {
            expose_public_node_for_source_test(&mut host, &permit, item, name);
        }
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, r#"
                firstItem instanceof HTMLLIElement && middleItem instanceof HTMLLIElement && lastItem instanceof HTMLLIElement &&
                firstItem.value === 0 && middleItem.value === 3 && lastItem.value === 0 &&
                firstItem.getAttributeNS(null, 'value') === null && middleItem.getAttributeNS(null, 'value') === '3' &&
                lastItem.getAttributeNS(null, 'value') === null
            "#)?;
            assert!(scope.boolean(&result)?, "native parent={parent_tag:?}");
            Ok(())
        }).unwrap();
    }
}

fn li_brands_precede_conversion(member: &str) {
    check_li_script(
        member,
        r#"
        (() => {
            const item = document.createElement('LI');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:li');
            const descriptor = Object.getOwnPropertyDescriptor(HTMLLIElement.prototype, liMember);
            if (!(item instanceof HTMLLIElement) || !(prefixed instanceof HTMLLIElement) || item.localName !== 'li' ||
                Object.getPrototypeOf(item) !== HTMLLIElement.prototype || Object.getPrototypeOf(HTMLLIElement) !== HTMLElement ||
                Object.getPrototypeOf(HTMLLIElement.prototype) !== HTMLElement.prototype ||
                descriptor.get.name !== 'get ' + liMember || descriptor.set.name !== 'set ' + liMember) return 'interface and descriptors';
            try { new HTMLLIElement(); return 'unregistered constructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, liMember === 'value' ? -3 : 'raw');
            if (descriptor.get.call(prefixed) !== (liMember === 'value' ? -3 : 'raw')) return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', 'li');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'LI');
            const other = document.createElement('div');
            Object.setPrototypeOf(other, HTMLLIElement.prototype);
            if (foreign instanceof HTMLLIElement || upper instanceof HTMLLIElement) return 'namespace and XML case';
            for (const receiver of [null, {}, document, foreign, upper, other, Object.create(item), new Proxy(item, {})]) {
                let converted = false;
                const value = {valueOf() { converted = true; return 7; }, toString() { converted = true; return 'raw'; }};
                try { descriptor.get.call(receiver); return 'invalid getter receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                try { descriptor.set.call(receiver, value); return 'invalid setter receiver'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before conversion'; }
            }
            return !other.hasAttributeNS(null, liMember) ? 'ok' : 'rejected receiver mutated';
        })()
    "#,
    );
}

fn li_signed_getter_cases(cases: &str) {
    check_li_script(
        "value",
        &format!(
            r#"(() => {{
        const item = document.createElement('li');
        item.setAttributeNS('urn:foreign', 'value', '777');
        if (item.value !== 0) return 'foreign value ignored';
        for (const [raw, expected] of {cases}) {{
            item.setAttributeNS(null, 'value', raw);
            if (item.value !== expected || item.getAttributeNS(null, 'value') !== raw) return 'signed HTML integer recovery/range';
        }}
        item.removeAttributeNS(null, 'value');
        return item.value === 0 && item.getAttributeNS('urn:foreign', 'value') === '777' ? 'ok' : 'missing default';
    }})()"#
        ),
    );
}

fn li_wrapping_setter_cases(cases: &str) {
    check_li_script(
        "value",
        &format!(
            r#"(() => {{
        const item = document.createElement('li');
        for (const [input, expected] of {cases}) {{
            item.value = input;
            if (item.value !== expected || Object.is(item.value, -0) || item.getAttributeNS(null, 'value') !== String(expected))
                return 'wrapping long and canonical decimal';
        }}
        return 'ok';
    }})()"#
        ),
    );
}

fn li_numeric_coercion_preserves_hints_reentry_and_original_throws() {
    check_li_script(
        "value",
        r#"
        (() => {
            const item = document.createElement('li');
            const hints = [];
            item.value = {[Symbol.toPrimitive](hint) { hints.push(hint); return -3.9; }};
            if (hints.join() !== 'number' || item.value !== -3) return 'numeric hint';
            item.value = {valueOf() {
                item.setAttributeNS(null, 'value', '99tail');
                if (item.value !== 99) throw 'nested read unavailable';
                return -7.9;
            }, toString() { throw 'unexpected string coercion'; }};
            if (item.value !== -7 || item.getAttributeNS(null, 'value') !== '-7') return 'numeric reentry';
            const original = {};
            try { item.value = {valueOf() { throw original; }}; return 'throwing input accepted'; }
            catch (error) { if (error !== original || item.value !== -7) return 'original throw and state'; }
            for (const value of [Symbol(), 1n, {valueOf() { return Symbol(); }}, {valueOf() { return 1n; }}]) {
                try { item.value = value; return 'Symbol or BigInt accepted'; }
                catch (error) { if (!(error instanceof TypeError) || item.value !== -7) return 'conversion TypeError state'; }
            }
            return 'ok';
        })()
    "#,
    );
}

fn li_type_reflects_exact_raw_strings() {
    check_li_script(
        "type",
        r#"
        (() => {
            const item = document.createElement('li');
            item.setAttributeNS('urn:foreign', 'type', 'foreign');
            if (item.type !== '') return 'foreign type ignored';
            item.type = null;
            if (item.type !== 'null') return 'ordinary null';
            item.type = {toString() { return null; }};
            if (item.type !== 'null') return 'object null';
            item.type = undefined;
            if (item.type !== 'undefined') return 'undefined';
            const exact = ' Raw\ud800\0\udc00 ';
            if (exact.length !== 8 || exact.charCodeAt(4) !== 0xd800 || exact.charCodeAt(5) !== 0 ||
                exact.charCodeAt(6) !== 0xdc00) return 'intended UTF16 fixture code units';
            item.type = {toString() {
                item.setAttributeNS(null, 'type', 'nested');
                if (item.type !== 'nested') throw 'nested read unavailable';
                return exact;
            }};
            if (item.type !== exact || item.getAttributeNS(null, 'type') !== exact ||
                item.getAttributeNS('urn:foreign', 'type') !== 'foreign') return 'exact UTF16 and namespace reentry';
            const original = {};
            try { item.type = {toString() { throw original; }}; return 'throwing input accepted'; }
            catch (error) { if (error !== original || item.type !== exact) return 'original string throw'; }
            try { item.type = Symbol(); return 'Symbol string accepted'; }
            catch (error) { if (!(error instanceof TypeError) || item.type !== exact) return 'Symbol string state'; }
            item.setAttributeNS(null, 'type', ' A i square INVALID ');
            if (item.type !== ' A i square INVALID ') return 'raw type without keyword normalization';
            item.removeAttributeNS(null, 'type');
            return item.type === '' && item.getAttributeNS('urn:foreign', 'type') === 'foreign' ? 'ok' : 'expanded name removal';
        })()
    "#,
    );
}

fn li_customized_constructors_deliver_original_reactions() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const calls = [];
                let throwNext = false;
                class Watched extends HTMLLIElement {
                    static observedAttributes = ['value', 'type'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (name === 'value' && newValue === '11') this.value = 12;
                        if (throwNext) { throwNext = false; direct.value = 7; throw globalThis.liCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-li', Watched, {extends: 'li'});
                const direct = new Watched();
                const item = document.createElement('li', {is: 'x-watched-li'});
                if (!(direct instanceof Watched) || !(item instanceof Watched) ||
                    direct.localName !== 'li' || item.localName !== 'li' || direct === item) return 'actual customized construction';
                item.value = 2.9;
                item.type = null;
                item.value = '2';
                if (calls.length !== 3 || calls[0][0] !== item || calls[0][1] !== 'value' || calls[0][2] !== null ||
                    calls[0][3] !== '2' || calls[0][4] !== null || calls[0][5] !== '2' ||
                    calls[1][1] !== 'type' || calls[1][2] !== null || calls[1][3] !== 'null' ||
                    calls[2][2] !== '2' || calls[2][3] !== '2') return 'original conversion and same-value reactions';
                item.value = 11;
                if (calls.length !== 5 || calls[3][3] !== '11' || calls[3][5] !== '11' ||
                    calls[4][2] !== '11' || calls[4][3] !== '12' || calls[4][5] !== '12' || item.value !== 12)
                    return 'publication before nested Source reentry';
                const original = {};
                try { item.value = {valueOf() { direct.type = '\ud800\0'; throw original; }}; return 'conversion throw'; }
                catch (error) { if (error !== original) return 'original conversion throw identity'; }
                if (calls.length !== 6 || calls[5][0] !== direct || calls[5][1] !== 'type' ||
                    calls[5][3] !== '\ud800\0' || item.value !== 12) return 'nested string Source survives numeric conversion throw';
                throwNext = true;
                item.type = 'throw';
                if (calls.length !== 8 || calls[6][0] !== item || calls[6][1] !== 'type' || calls[6][3] !== 'throw' ||
                    calls[7][0] !== direct || calls[7][1] !== 'value' || calls[7][3] !== '7' ||
                    item.type !== 'throw' || direct.value !== 7) return 'nested numeric Source survives reported callback throw';
                class Throwing extends HTMLLIElement {
                    constructor() { super(); globalThis.constructedLI = this; throw globalThis.liConstructorThrow = {}; }
                }
                customElements.define('x-failed-li', Throwing, {extends: 'li'});
                const failed = document.createElement('li', {is: 'x-failed-li'});
                if (failed !== constructedLI || !(failed instanceof Throwing) || !(failed instanceof HTMLLIElement) ||
                    failed instanceof HTMLUnknownElement) return 'authentic customized failure';
                failed.value = -7;
                failed.type = null;
                return failed.value === -7 && failed.type === 'null' ? 'ok' : 'authentic failed-construction reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
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
            ("liCallbackThrow", &callback.exception),
            ("liConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn li_native_edits_and_retirement_keep_wrapper_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let item = host
        .in_document(&permit, |scope| {
            evaluate(
                scope,
                "globalThis.nativeLI = document.createElement('li'); globalThis.savedLI = nativeLI",
            )?;
            let item = evaluate(scope, "nativeLI")?;
            projected_node_id(scope, &item)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(item, Namespace::none(), "value", " -2147483648tail");
        batch.set_attribute_ns(
            item,
            Namespace::none(),
            "type",
            surgeist_dom::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        for attribute in ["value", "type"] {
            batch.set_attribute_ns(
                item,
                Namespace::from_uri("urn:foreign"),
                attribute,
                "foreign",
            );
        }
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"nativeLI === savedLI && nativeLI.value === -2147483648 &&
            nativeLI.type === '\ud800\0\udc00' && nativeLI.type.length === 3 &&
            nativeLI.type.charCodeAt(0) === 0xd800 && nativeLI.type.charCodeAt(1) === 0 &&
            nativeLI.type.charCodeAt(2) === 0xdc00 && nativeLI.getAttributeNS('urn:foreign', 'value') === 'foreign' &&
            nativeLI.getAttributeNS('urn:foreign', 'type') === 'foreign'"#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "globalThis.liDestination = new Document(); liDestination.adoptNode(nativeLI)",
        )?;
        let result = evaluate(
            scope,
            r#"nativeLI === savedLI && nativeLI.ownerDocument === liDestination &&
            nativeLI.parentNode === null && nativeLI instanceof HTMLLIElement &&
            nativeLI.value === -2147483648 && nativeLI.type === '\ud800\0\udc00'"#,
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(item);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeLI.value",
        "nativeLI.type",
        "nativeLI.value = 7",
        "nativeLI.type = 'retired'",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired LI must refuse {expression}: {result:?}")
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn li_accessors_keep_registered_realms_after_adoption(member: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let item = {
        let mut batch = DomBatch::new();
        let item = batch.create_element_ns(origin.document(), Namespace::html(), "li");
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&item)
            .unwrap()
            .unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, item, "originLI");
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
        set_li_member(scope, member)?;
        let item = evaluate(scope, "originLI")?;
        assert_ne!(scope.callback_object(&item)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignLITypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"(() => {
            globalThis.originLIPrototype = Object.getPrototypeOf(originLI);
            if (originLI instanceof HTMLLIElement || originLIPrototype.constructor.name !== 'HTMLLIElement') return 'creation realm';
            const descriptor = Object.getOwnPropertyDescriptor(originLIPrototype, liMember);
            for (const accessor of [descriptor.get, descriptor.set]) {
                try { accessor.call({}, 7); return 'borrowed receiver accepted'; }
                catch (error) { if (!(error instanceof foreignLITypeError) || error instanceof TypeError) return 'function realm exception'; }
            }
            const original = {};
            try { descriptor.set.call(originLI, {[Symbol.toPrimitive]() { throw original; }}); return 'borrowed conversion throw'; }
            catch (error) { if (error !== original) return 'borrowed original exception'; }
            descriptor.set.call(originLI, liMember === 'value' ? 4294967295 : 'foreign\ud800\0');
            if (liMember === 'type' && (originLI.type.length !== 9 || originLI.type.charCodeAt(7) !== 0xd800 ||
                originLI.type.charCodeAt(8) !== 0)) return 'borrowed intended UTF16 code units';
            return descriptor.get.call(originLI) === (liMember === 'value' ? -1 : 'foreign\ud800\0') ? 'ok' : 'borrowed reflection';
        })()"#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{member}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(item, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"originLI.ownerDocument === document &&
            Object.getPrototypeOf(originLI) === originLIPrototype && !(originLI instanceof HTMLLIElement) &&
            originLI[liMember] === (liMember === 'value' ? -1 : 'foreign\ud800\0')"#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
}

fn li_setters_admit_original_source_outputs(member: &str) {
    for outputs in [1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let item = batch.create_element_ns(target.document(), Namespace::html(), "li");
        let item = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&item)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, item, "limitedLI");
        host.in_document(&permit, |scope| set_li_member(scope, member))
            .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(
                scope,
                "limitedLI[liMember] = liMember === 'value' ? -2147483648 : null",
            )
            .map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short LI {member} output: {result:?}")
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
            assert!(after.node(item).unwrap().attributes().unwrap().is_empty());
        } else {
            result.unwrap();
            let expected = if member == "value" {
                "-2147483648"
            } else {
                "null"
            };
            assert_eq!(
                after.node(item).unwrap().attribute(&member.into()),
                Some(&expected.into())
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let item = batch.create_element_ns(target.document(), Namespace::html(), "li");
    let item = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&item)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, item, "limitedLI");
    host.in_document(&permit, |scope| set_li_member(scope, member))
        .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "limitedLI[liMember] = 7").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("LI {member} must consume Source work: {result:?}")
    };
    assert!(matches!(
        failure.cause::<crate::DomSourceDriveError>(),
        Some(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit { limit: 0 })
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    assert!(
        host.snapshot()
            .unwrap()
            .node(item)
            .unwrap()
            .attributes()
            .unwrap()
            .is_empty()
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    drop(failure);
    host.stop().into_session().unwrap();
}

// HTML877d's modern HTMLLIElement and obsolete partial interface specify
// reflected signed-long value followed by ordinary DOMString type.
pub(super) fn li_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const item = document.createElement('li');
                if (!(item instanceof HTMLLIElement)) return 'li interface';
                if (Object.keys(HTMLLIElement.prototype).join() !== 'value,type')
                    return 'li missing reflected members';
                for (const member of ['value', 'type']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLLIElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'li descriptor ' + member;
                }
                if (item.value !== 0 || item.type !== '') return 'li absent defaults';
                item.value = -7;
                if (item.value !== -7 || item.getAttributeNS(null, 'value') !== '-7')
                    return 'li canonical signed reflection';
                item.type = null;
                return item.type === 'null' && item.getAttributeNS(null, 'type') === 'null'
                    ? 'ok' : 'li ordinary DOMString reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
