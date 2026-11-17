use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

const LIST_MEMBERS: [(&str, &str, &str); 7] = [
    ("ol", "reversed", "boolean"),
    ("ol", "start", "long"),
    ("ol", "type", "string"),
    ("ol", "compact", "boolean"),
    ("ul", "compact", "boolean"),
    ("ul", "type", "string"),
    ("menu", "compact", "boolean"),
];

pub(super) fn list_families_preserve_native_and_web_idl_contracts() {
    // Shared Pre/LI physical, large-input and Source/cancellation cohorts run
    // in this same entry-thread aggregate. These fixtures qualify each mapping.
    for (tag, member, kind) in LIST_MEMBERS {
        list_brands_precede_conversion(tag, member, kind);
        if kind == "boolean" {
            list_boolean_presence_and_conversion(tag, member, kind);
        }
        if kind == "string" {
            list_type_reflects_exact_strings(tag, member, kind);
        }
        list_customized_constructors_deliver_original_reactions(tag, member, kind);
        list_native_edits_and_retirement(tag, member, kind);
        list_accessors_keep_registered_realms(tag, member, kind);
        list_setters_admit_original_source_outputs(tag, member, kind);
    }
    ol_start_keeps_its_default_independent_of_reversed_children();
    signed_defaults_remain_one_for_ol_and_zero_for_pre_and_li();
    for cases in [
        r#"[['',1],['invalid',1],['+',1],['\u00a07',1],['2147483648',1],['-2147483649',1]]"#,
        r#"[[' \t\n\f\r-7tail',-7],['+007junk',7],['0',0],['2147483647',2147483647],['-2147483648',-2147483648],['1e2',1]]"#,
    ] {
        ol_start_getter_cases(cases);
    }
    for cases in [
        "[[undefined,0],[null,0],[NaN,0],[Infinity,0],[-Infinity,0],[-0,0]]",
        "[[1.9,1],[-1.9,-1],[2147483648,-2147483648],[-2147483649,2147483647],[4294967295,-1],[4294967296,0]]",
    ] {
        ol_start_setter_cases(cases);
    }
    ol_start_numeric_reentry_preserves_default_and_original_errors();
}

fn set_list_member(
    scope: &mut Scope<'_>,
    tag: &str,
    member: &str,
    kind: &str,
) -> Result<(), Error> {
    let global = scope.global()?;
    for (key, value) in [("listTag", tag), ("listMember", member), ("listKind", kind)] {
        let value = scope.string(&value.encode_utf16().collect::<Vec<_>>())?;
        scope.set(
            &global,
            PropertyKey::String(&key.encode_utf16().collect::<Vec<_>>()),
            &value,
        )?;
    }
    Ok(())
}

fn check_list_script(tag: &str, member: &str, kind: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_list_member(scope, tag, member, kind)?;
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{tag}.{member}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "{tag}.{member}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("{tag}.{member}: {error:?}"),
    });
}

fn list_brands_precede_conversion(tag: &str, member: &str, kind: &str) {
    check_list_script(
        tag,
        member,
        kind,
        r#"
        (() => {
            const Type = listTag === 'ol' ? HTMLOListElement : listTag === 'ul' ? HTMLUListElement : HTMLMenuElement;
            const element = document.createElement(listTag.toUpperCase());
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + listTag);
            const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, listMember);
            if (!(element instanceof Type) || !(prefixed instanceof Type) || element.localName !== listTag ||
                Object.getPrototypeOf(element) !== Type.prototype || Object.getPrototypeOf(Type) !== HTMLElement ||
                Object.getPrototypeOf(Type.prototype) !== HTMLElement.prototype ||
                descriptor.get.name !== 'get ' + listMember || descriptor.set.name !== 'set ' + listMember)
                return 'interface inheritance and accessor names';
            try { new Type(); return 'unregistered constructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            const expected = listKind === 'boolean' ? true : listKind === 'long' ? -3 : 'raw';
            descriptor.set.call(prefixed, expected);
            if (descriptor.get.call(prefixed) !== expected) return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', listTag);
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', listTag.toUpperCase());
            const other = document.createElement(listTag === 'ol' ? 'ul' : 'ol');
            Object.setPrototypeOf(other, Type.prototype);
            if (foreign instanceof Type || upper instanceof Type) return 'namespace and XML case';
            for (const receiver of [null, {}, document, foreign, upper, other, Object.create(element), new Proxy(element, {})]) {
                let converted = false;
                const input = {valueOf() { converted = true; return 7; }, toString() { converted = true; return 'raw'; }};
                try { descriptor.get.call(receiver); return 'invalid getter receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                try { descriptor.set.call(receiver, input); return 'invalid setter receiver'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before conversion'; }
            }
            return !other.hasAttributeNS(null, listMember) ? 'ok' : 'invalid receiver mutated';
        })()
    "#,
    );
}

fn list_boolean_presence_and_conversion(tag: &str, member: &str, kind: &str) {
    check_list_script(
        tag,
        member,
        kind,
        r#"
        (() => {
            const element = document.createElement(listTag);
            element.setAttributeNS('urn:foreign', listMember, 'foreign');
            if (element[listMember] !== false) return 'foreign namespace ignored';
            for (const raw of ['', 'false', '0', '\ud800\0']) {
                element.setAttributeNS(null, listMember, raw);
                if (element[listMember] !== true || element.getAttributeNS(null, listMember) !== raw) return 'Boolean presence';
            }
            let coerced = false;
            const object = {valueOf() { coerced = true; throw 0; }, toString() { coerced = true; throw 0; },
                [Symbol.toPrimitive]() { coerced = true; throw 0; }};
            for (const value of [true, 'false', 1, -1, 1n, Symbol(), object, [], {}, new Boolean(false)]) {
                element[listMember] = value;
                if (element[listMember] !== true || element.getAttributeNS(null, listMember) !== '' || coerced)
                    return 'truthy ToBoolean without coercion';
            }
            for (const value of [false, null, undefined, 0, -0, NaN, '', 0n]) {
                element[listMember] = true;
                element[listMember] = value;
                if (element[listMember] !== false || element.hasAttributeNS(null, listMember) ||
                    element.getAttributeNS('urn:foreign', listMember) !== 'foreign') return 'falsy removes only null namespace';
            }
            return 'ok';
        })()
    "#,
    );
}

fn list_type_reflects_exact_strings(tag: &str, member: &str, kind: &str) {
    check_list_script(
        tag,
        member,
        kind,
        r#"
        (() => {
            const element = document.createElement(listTag);
            element.setAttributeNS('urn:foreign', 'type', 'foreign');
            if (element.type !== '') return 'foreign type ignored';
            element.type = null;
            if (element.type !== 'null') return 'ordinary null';
            element.type = {toString() { return null; }};
            if (element.type !== 'null') return 'object null';
            element.type = undefined;
            if (element.type !== 'undefined') return 'undefined';
            const exact = ' Raw\ud800\0\udc00 ';
            if (exact.length !== 8 || exact.charCodeAt(4) !== 0xd800 || exact.charCodeAt(5) !== 0 ||
                exact.charCodeAt(6) !== 0xdc00) return 'intended UTF16 fixture';
            element.type = {toString() {
                element.setAttributeNS(null, 'type', 'nested');
                if (element.type !== 'nested') throw 'nested read unavailable';
                return exact;
            }};
            if (element.type !== exact || element.getAttributeNS(null, 'type') !== exact ||
                element.getAttributeNS('urn:foreign', 'type') !== 'foreign') return 'exact UTF16 namespace and reentry';
            const original = {};
            try { element.type = {toString() { throw original; }}; return 'throwing input accepted'; }
            catch (error) { if (error !== original || element.type !== exact) return 'original throw and state'; }
            try { element.type = Symbol(); return 'Symbol accepted'; }
            catch (error) { if (!(error instanceof TypeError) || element.type !== exact) return 'Symbol state'; }
            element.setAttributeNS(null, 'type', ' A i circle INVALID ');
            if (element.type !== ' A i circle INVALID ') return 'raw type without canonicalization';
            element.removeAttributeNS(null, 'type');
            return element.type === '' && element.getAttributeNS('urn:foreign', 'type') === 'foreign' ? 'ok' : 'expanded-name removal';
        })()
    "#,
    );
}

fn ol_start_keeps_its_default_independent_of_reversed_children() {
    for count in [0, 3] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let list = batch.create_element_ns(target.document(), Namespace::html(), "ol");
        for _ in 0..count {
            let item = batch.create_element_ns(target.document(), Namespace::html(), "li");
            batch.insert_before(&item, &list, None);
        }
        let list = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&list)
            .unwrap()
            .unwrap();
        assert_eq!(
            session.snapshot().node(list).unwrap().children().len(),
            count
        );
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, list, "nativeList");
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, r#"(() => {
                if (!(nativeList instanceof HTMLOListElement) || nativeList.start !== 1) return false;
                nativeList.reversed = true;
                if (nativeList.start !== 1 || nativeList.hasAttributeNS(null, 'start')) return false;
                nativeList.start = undefined;
                return nativeList.start === 0 && nativeList.getAttributeNS(null, 'start') === '0';
            })()"#)?;
            assert!(scope.boolean(&result)?, "actual native item count={count}");
            Ok(())
        }).unwrap();
    }
}

fn signed_defaults_remain_one_for_ol_and_zero_for_pre_and_li() {
    // This native raw-input cohort distinguishes the fallback parameter;
    // the shared large-input cohorts qualify numeric parsing and admission.
    let raw = format!("{}2147483648tail", "0".repeat(256));
    for (tag, member, expected) in [("ol", "start", 1), ("pre", "width", 0), ("li", "value", 0)] {
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
        let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "rawSigned");
        host.with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.set_attribute_ns(element, Namespace::none(), member, raw.as_str());
            dom.apply(batch, &cancel).unwrap();
        })
        .unwrap();
        host.in_document(&permit, |scope| {
            set_list_member(scope, tag, member, "long")?;
            let result = evaluate(scope, "rawSigned[listMember]")?;
            let expected = scope.primitive(Primitive::Number(f64::from(expected)))?;
            assert!(scope.strict_equals(&result, &expected)?, "{tag}.{member}");
            Ok(())
        })
        .unwrap();
    }
}

fn ol_start_getter_cases(cases: &str) {
    check_list_script(
        "ol",
        "start",
        "long",
        &format!(
            r#"(() => {{
        const list = document.createElement('ol');
        list.reversed = true;
        list.setAttributeNS('urn:foreign', 'start', '777');
        if (list.start !== 1) return 'foreign default';
        for (const [raw, expected] of {cases}) {{
            list.setAttributeNS(null, 'start', raw);
            if (list.start !== expected || list.getAttributeNS(null, 'start') !== raw) return 'signed parsing/default one';
        }}
        list.removeAttributeNS(null, 'start');
        return list.start === 1 && list.getAttributeNS('urn:foreign', 'start') === '777' ? 'ok' : 'missing default one';
    }})()"#
        ),
    );
}

fn ol_start_setter_cases(cases: &str) {
    check_list_script(
        "ol",
        "start",
        "long",
        &format!(
            r#"(() => {{
        const list = document.createElement('ol');
        for (const [input, expected] of {cases}) {{
            list.start = input;
            if (list.start !== expected || Object.is(list.start, -0) || list.getAttributeNS(null, 'start') !== String(expected))
                return 'wrapping long/canonical decimal independent of default one';
        }}
        return 'ok';
    }})()"#
        ),
    );
}

fn ol_start_numeric_reentry_preserves_default_and_original_errors() {
    check_list_script(
        "ol",
        "start",
        "long",
        r#"
        (() => {
            const list = document.createElement('ol');
            const hints = [];
            list.start = {[Symbol.toPrimitive](hint) { hints.push(hint); return -3.9; }};
            if (hints.join() !== 'number' || list.start !== -3) return 'numeric hint';
            list.start = {valueOf() {
                list.setAttributeNS(null, 'start', 'invalid');
                if (list.start !== 1) throw 'reentrant default mismatch';
                return -7.9;
            }, toString() { throw 'unexpected string coercion'; }};
            if (list.start !== -7 || list.getAttributeNS(null, 'start') !== '-7') return 'reentry and canonical write';
            const original = {};
            try { list.start = {valueOf() { throw original; }}; return 'throwing input accepted'; }
            catch (error) { if (error !== original || list.start !== -7) return 'original error and state'; }
            for (const value of [Symbol(), 1n]) {
                try { list.start = value; return 'Symbol/BigInt accepted'; }
                catch (error) { if (!(error instanceof TypeError) || list.start !== -7) return 'numeric error state'; }
            }
            return 'ok';
        })()
    "#,
    );
}

fn list_customized_constructors_deliver_original_reactions(tag: &str, member: &str, kind: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_list_member(scope, tag, member, kind)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = listTag === 'ol' ? HTMLOListElement : listTag === 'ul' ? HTMLUListElement : HTMLMenuElement;
                const calls = [];
                let reenter = false;
                let throwNext = false;
                class Watched extends Type {
                    static observedAttributes = [listMember];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (reenter) { reenter = false;
                            this[listMember] = listKind === 'boolean' ? false : listKind === 'long' ? 12 : 'inner'; }
                        if (throwNext) { throwNext = false;
                            direct[listMember] = listKind === 'boolean' ? true : listKind === 'long' ? -7 : 'nested\ud800\0';
                            throw globalThis.listCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-list', Watched, {extends: listTag});
                const direct = new Watched();
                const element = document.createElement(listTag, {is: 'x-watched-list'});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== listTag || element.localName !== listTag || direct === element) return 'actual construction tag';
                const input = listKind === 'boolean' ? true : listKind === 'long' ? 2.9 : null;
                const canonical = listKind === 'boolean' ? '' : listKind === 'long' ? '2' : 'null';
                element[listMember] = input;
                element[listMember] = input;
                if (calls.length !== 2 || calls[0][0] !== element || calls[0][1] !== listMember ||
                    calls[0][2] !== null || calls[0][3] !== canonical || calls[0][4] !== null || calls[0][5] !== canonical ||
                    calls[1][2] !== canonical || calls[1][3] !== canonical) return 'original conversion and same-value reactions';
                reenter = true;
                element[listMember] = listKind === 'boolean' ? true : listKind === 'long' ? 11 : 'outer';
                const outer = listKind === 'boolean' ? '' : listKind === 'long' ? '11' : 'outer';
                const inner = listKind === 'boolean' ? null : listKind === 'long' ? '12' : 'inner';
                if (calls.length !== 4 || calls[2][3] !== outer || calls[2][5] !== outer ||
                    calls[3][2] !== outer || calls[3][3] !== inner || calls[3][5] !== inner ||
                    element[listMember] !== (listKind === 'boolean' ? false : listKind === 'long' ? 12 : 'inner'))
                    return 'publication before nested Source mutation';
                throwNext = true;
                element[listMember] = listKind === 'boolean' ? true : listKind === 'long' ? 21 : 'throw';
                if (calls.length !== 6 || calls[4][0] !== element || calls[5][0] !== direct ||
                    calls[5][2] !== null || element[listMember] !== (listKind === 'boolean' ? true : listKind === 'long' ? 21 : 'throw') ||
                    direct[listMember] !== (listKind === 'boolean' ? true : listKind === 'long' ? -7 : 'nested\ud800\0'))
                    return 'nested Source survives reported original callback throw';
                class Throwing extends Type {
                    constructor() { super(); globalThis.constructedList = this; throw globalThis.listConstructorThrow = {}; }
                }
                customElements.define('x-failed-list', Throwing, {extends: listTag});
                const failed = document.createElement(listTag, {is: 'x-failed-list'});
                if (failed !== constructedList || !(failed instanceof Throwing) || !(failed instanceof Type) ||
                    failed.localName !== listTag || failed instanceof HTMLUnknownElement) return 'authentic customized failure';
                failed[listMember] = listKind === 'boolean' ? true : listKind === 'long' ? -3 : null;
                return failed[listMember] === (listKind === 'boolean' ? true : listKind === 'long' ? -3 : 'null')
                    ? 'ok' : 'authentic failed-construction reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}.{member}");
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
            ("listCallbackThrow", &callback.exception),
            ("listConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn list_native_edits_and_retirement(tag: &str, member: &str, kind: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        set_list_member(scope, tag, member, kind)?;
        evaluate(scope, "globalThis.nativeList = document.createElement(listTag); globalThis.savedList = nativeList; globalThis.listPrototype = Object.getPrototypeOf(nativeList)")?;
        let element = evaluate(scope, "nativeList")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        if kind == "string" {
            batch.set_attribute_ns(
                element,
                Namespace::none(),
                member,
                surgeist_html::values::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
            );
        } else {
            batch.set_attribute_ns(
                element,
                Namespace::none(),
                member,
                if kind == "long" {
                    " -2147483648tail"
                } else {
                    "false"
                },
            );
        }
        batch.set_attribute_ns(
            element,
            Namespace::from_uri("urn:foreign"),
            member,
            "foreign",
        );
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let stamp = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"(() => {
            const observed = nativeList[listMember];
            return nativeList === savedList &&
                observed === (listKind === 'boolean' ? true : listKind === 'long' ? -2147483648 : '\ud800\0\udc00') &&
                (listKind !== 'string' || (observed.length === 3 && observed.charCodeAt(0) === 0xd800 &&
                    observed.charCodeAt(1) === 0 && observed.charCodeAt(2) === 0xdc00)) &&
                nativeList.getAttributeNS('urn:foreign', listMember) === 'foreign';
        })()"#)?;
        assert!(scope.boolean(&result)?, "{tag}.{member}");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    host.in_document(&permit, |scope| {
        evaluate(scope, "globalThis.listDestination = new Document(); listDestination.adoptNode(nativeList)")?;
        let result = evaluate(scope, r#"nativeList === savedList && nativeList.ownerDocument === listDestination &&
            nativeList.parentNode === null && Object.getPrototypeOf(nativeList) === listPrototype &&
            nativeList.localName === listTag &&
            nativeList[listMember] === (listKind === 'boolean' ? true : listKind === 'long' ? -2147483648 : '\ud800\0\udc00')"#)?;
        assert!(scope.boolean(&result)?, "{tag}.{member}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in ["nativeList[listMember]", "nativeList[listMember] = 7"] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired {tag}.{member}: {result:?}")
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn list_accessors_keep_registered_realms(tag: &str, member: &str, kind: &str) {
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
    expose_public_node_for_source_test(&mut host, &permit, element, "originList");
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
        set_list_member(scope, tag, member, kind)?;
        let element = evaluate(scope, "originList")?;
        assert_ne!(scope.callback_object(&element)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignListTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"(() => {
            const Type = listTag === 'ol' ? HTMLOListElement : listTag === 'ul' ? HTMLUListElement : HTMLMenuElement;
            globalThis.originListPrototype = Object.getPrototypeOf(originList);
            if (originList instanceof Type || originListPrototype.constructor.name !== Type.name) return 'creation realm';
            const descriptor = Object.getOwnPropertyDescriptor(originListPrototype, listMember);
            for (const accessor of [descriptor.get, descriptor.set]) {
                try { accessor.call({}, 7); return 'borrowed receiver accepted'; }
                catch (error) { if (!(error instanceof foreignListTypeError) || error instanceof TypeError) return 'function realm error'; }
            }
            const original = {};
            let converted = false;
            const input = {[Symbol.toPrimitive]() { converted = true; throw original; }};
            if (listKind === 'boolean') {
                descriptor.set.call(originList, input);
                if (converted || descriptor.get.call(originList) !== true) return 'borrowed Boolean no coercion';
            } else {
                try { descriptor.set.call(originList, input); return 'borrowed conversion throw'; }
                catch (error) { if (error !== original) return 'original borrowed exception'; }
            }
            descriptor.set.call(originList, listKind === 'boolean' ? true : listKind === 'long' ? 4294967295 : 'foreign\ud800\0');
            const observed = descriptor.get.call(originList);
            if (listKind === 'string' && (observed.length !== 9 || observed.charCodeAt(7) !== 0xd800 ||
                observed.charCodeAt(8) !== 0)) return 'borrowed intended UTF16 units';
            return observed === (listKind === 'boolean' ? true : listKind === 'long' ? -1 : 'foreign\ud800\0')
                ? 'ok' : 'borrowed reflection';
        })()"#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}.{member}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(element, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"originList.ownerDocument === document &&
            Object.getPrototypeOf(originList) === originListPrototype &&
            originList[listMember] === (listKind === 'boolean' ? true : listKind === 'long' ? -1 : 'foreign\ud800\0')"#)?;
        assert!(scope.boolean(&result)?, "{tag}.{member}");
        Ok(())
    }).unwrap();
}

fn list_setters_admit_original_source_outputs(tag: &str, member: &str, kind: &str) {
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
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedList");
        host.in_document(&permit, |scope| set_list_member(scope, tag, member, kind))
            .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| evaluate(scope,
            "limitedList[listMember] = listKind === 'boolean' ? true : listKind === 'long' ? undefined : null").map(drop));
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short {tag}.{member} Source outputs: {result:?}")
            };
            assert!(matches!(
                failure.cause::<crate::DomSourceDriveError>(),
                Some(crate::DomSourceDriveError::Source(
                    crate::DomSourceRuntimeError::Dom(DomError::SourceOutputLimit)
                ))
            ));
            assert_eq!(after.stamp(), before.stamp());
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
            let expected = if kind == "boolean" {
                ""
            } else if kind == "long" {
                "0"
            } else {
                "null"
            };
            assert_eq!(
                after.node(element).unwrap().attribute(&member.into()),
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
    let element = batch.create_element_ns(target.document(), Namespace::html(), tag);
    let element = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "limitedList");
    host.in_document(&permit, |scope| set_list_member(scope, tag, member, kind))
        .unwrap();
    let stamp = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "limitedList[listMember] = 7").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("{tag}.{member} must consume original Source work: {result:?}")
    };
    assert!(matches!(
        failure.cause::<crate::DomSourceDriveError>(),
        Some(crate::DomSourceDriveError::Source(
            crate::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit { limit: 0 })
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    assert!(
        host.snapshot()
            .unwrap()
            .node(element)
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

// HTML877d's modern/obsolete list IDLs specify these member orders and
// OL.start's reflected default 1, independently of its list starting value.
pub(super) fn list_members_reflect_canonical_attributes() {
    for tag in ["ol", "ul", "menu"] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) = hosted_session(&cancel, 0);
        let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
        let permit = host.permit().unwrap();
        let script = format!(
            r#"
            (() => {{
                const tag = '{tag}';
                const Type = tag === 'ol' ? HTMLOListElement : tag === 'ul' ? HTMLUListElement : HTMLMenuElement;
                const members = tag === 'ol' ? ['reversed', 'start', 'type', 'compact'] :
                    tag === 'ul' ? ['compact', 'type'] : ['compact'];
                const element = document.createElement(tag);
                if (!(element instanceof Type)) return tag + ' interface';
                if (Object.keys(Type.prototype).join() !== members.join()) return tag + ' missing reflected members';
                for (const member of members) {{
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' descriptor ' + member;
                    const expected = member === 'start' ? 1 : member === 'type' ? '' : false;
                    if (element[member] !== expected) return tag + ' absent default ' + member;
                }}
                if (tag === 'ol') {{
                    for (let index = 0; index < 3; index++) element.appendChild(document.createElement('li'));
                    element.reversed = true;
                    if (element.start !== 1 || element.hasAttributeNS(null, 'start'))
                        return 'reflected start is not the reversed list starting value';
                    element.start = undefined;
                    if (element.start !== 0 || element.getAttributeNS(null, 'start') !== '0')
                        return 'undefined setter writes canonical zero';
                }}
                for (const member of members.filter(member => member === 'reversed' || member === 'compact')) {{
                    element[member] = true;
                    if (element[member] !== true || element.getAttributeNS(null, member) !== '')
                        return tag + ' Boolean presence ' + member;
                    element[member] = false;
                    if (element[member] !== false || element.hasAttributeNS(null, member))
                        return tag + ' Boolean removal ' + member;
                }}
                if (tag !== 'menu') {{
                    element.type = null;
                    if (element.type !== 'null' || element.getAttributeNS(null, 'type') !== 'null')
                        return tag + ' ordinary type DOMString';
                }}
                return 'ok';
            }})()
        "#
        );
        host.in_document(&permit, |scope| {
            let result = evaluate(scope, &script)?;
            assert_eq!(
                String::from_utf16_lossy(&scope.to_utf16(&result)?),
                "ok",
                "{tag}"
            );
            Ok(())
        })
        .unwrap();
    }
}
