use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

const PARAM_MEMBERS: [(&str, &str); 4] = [
    ("name", "name"),
    ("value", "value"),
    ("type", "type"),
    ("valueType", "valuetype"),
];

pub(super) fn param_family_preserves_native_and_web_idl_contracts() {
    // Keep independent member stimuli in fresh real hosts, with semantic work
    // separate from the exact Source-output admission cases.
    for (member, attribute) in PARAM_MEMBERS {
        param_brands_precede_conversion(member, attribute);
        param_strings_use_exact_expanded_names(member, attribute);
        param_customized_constructors_deliver_original_reactions(member, attribute);
        param_accessors_keep_registered_realms_after_adoption(member, attribute);
        param_setters_admit_source_outputs_before_publication(member, attribute);
    }
    param_reads_follow_native_edits_and_retirement();
}

fn set_param_member(scope: &mut Scope<'_>, member: &str, attribute: &str) -> Result<(), Error> {
    let global = scope.global()?;
    for (key, value) in [("paramMember", member), ("paramAttribute", attribute)] {
        let value = scope.string(&value.encode_utf16().collect::<Vec<_>>())?;
        scope.set(
            &global,
            PropertyKey::String(&key.encode_utf16().collect::<Vec<_>>()),
            &value,
        )?;
    }
    Ok(())
}

fn check_param_script(member: &str, attribute: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_param_member(scope, member, attribute)?;
        let result = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&result)?),
            "ok",
            "{member}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "param {member}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("param {member}: {error:?}"),
    });
}

fn param_brands_precede_conversion(member: &str, attribute: &str) {
    check_param_script(
        member,
        attribute,
        r#"
        (() => {
            const param = document.createElement('PARAM');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:param');
            const descriptor = Object.getOwnPropertyDescriptor(HTMLParamElement.prototype, paramMember);
            if (!(param instanceof HTMLParamElement) || param instanceof HTMLUnknownElement || param.localName !== 'param' ||
                !(prefixed instanceof HTMLParamElement) || Object.getPrototypeOf(param) !== HTMLParamElement.prototype ||
                Object.getPrototypeOf(HTMLParamElement) !== HTMLElement ||
                Object.getPrototypeOf(HTMLParamElement.prototype) !== HTMLElement.prototype ||
                descriptor.get.name !== 'get ' + paramMember || descriptor.set.name !== 'set ' + paramMember)
                return 'obsolete classification inheritance and accessor names';
            try { new HTMLParamElement(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, 'prefixed');
            if (descriptor.get.call(prefixed) !== 'prefixed') return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', 'param');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'PARAM');
            const other = document.createElement('div');
            Object.setPrototypeOf(other, HTMLParamElement.prototype);
            if (foreign instanceof HTMLParamElement || upper instanceof HTMLParamElement)
                return 'actual namespace and XML case';
            for (const receiver of [null, {}, document, foreign, upper, other,
                Object.create(param), new Proxy(param, {})]) {
                let converted = false;
                try { descriptor.get.call(receiver); return 'getter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                try { descriptor.set.call(receiver, {toString() { converted = true; return 'bad'; }});
                    return 'setter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before conversion'; }
            }
            return !other.hasAttributeNS(null, paramAttribute) ? 'ok' : 'invalid receiver mutated native state';
        })()
    "#,
    );
}

fn param_strings_use_exact_expanded_names(member: &str, attribute: &str) {
    check_param_script(
        member,
        attribute,
        r#"
        (() => {
            const param = document.createElement('param');
            if (param[paramMember] !== '') return 'absent default';
            if (paramMember === 'valueType') {
                param.setAttributeNS(null, 'valueType', 'case-distinct');
                param.setAttributeNS(null, 'valuetype', 'lowercase');
                if (param.valueType !== 'lowercase' || param.getAttributeNS(null, 'valueType') !== 'case-distinct' ||
                    param.getAttributeNS(null, 'valuetype') !== 'lowercase' || param.getAttribute('valueType') !== 'lowercase')
                    return 'literal lowercase reflection and exact namespace local-name controls';
                param.removeAttributeNS(null, 'valuetype');
                if (param.valueType !== '') return 'camelCase content attribute is distinct';
            }
            param.setAttributeNS('urn:foreign', paramAttribute, 'foreign');
            if (param[paramMember] !== '') return 'foreign namespace ignored';
            param[paramMember] = null;
            if (param[paramMember] !== 'null' || param.getAttributeNS(null, paramAttribute) !== 'null')
                return 'ordinary null DOMString';
            param[paramMember] = {toString() { return null; }};
            if (param[paramMember] !== 'null') return 'object returning null';
            param[paramMember] = undefined;
            if (param[paramMember] !== 'undefined') return 'undefined DOMString';
            const exact = ' Raw\ud800\0\udc00 ';
            let count = 0;
            param[paramMember] = {toString() {
                count++;
                param.setAttributeNS(null, paramAttribute, 'nested');
                if (param[paramMember] !== 'nested') throw 'nested reflection unavailable';
                return exact;
            }};
            if (count !== 1 || param[paramMember] !== exact || param.getAttributeNS(null, paramAttribute) !== exact ||
                param.getAttributeNS('urn:foreign', paramAttribute) !== 'foreign') return 'UTF16 reentry and namespace isolation';
            const original = {};
            try { param[paramMember] = {toString() { throw original; }}; return 'throwing value accepted'; }
            catch (error) { if (error !== original || param[paramMember] !== exact) return 'original exception and state'; }
            for (const value of [Symbol(), {toString() { return Symbol(); }}]) {
                try { param[paramMember] = value; return 'Symbol accepted'; }
                catch (error) { if (!(error instanceof TypeError) || param[paramMember] !== exact) return 'Symbol state'; }
            }
            const raw = paramMember === 'type' ? ' NOT a/valid-MIME ; RAW ' :
                paramMember === 'valueType' ? ' REFERENCE invalid object DATA ' : ' Raw Unparsed Value ';
            param.setAttributeNS(null, paramAttribute, raw);
            if (param[paramMember] !== raw) return 'raw content without parsing or keyword canonicalization';
            param.removeAttributeNS(null, paramAttribute);
            if (param[paramMember] !== '' || param.getAttributeNS('urn:foreign', paramAttribute) !== 'foreign')
                return 'expanded-name removal';
            return paramMember !== 'valueType' || param.getAttributeNS(null, 'valueType') === 'case-distinct'
                ? 'ok' : 'case-distinct content attribute changed';
        })()
    "#,
    );
}

fn param_customized_constructors_deliver_original_reactions(member: &str, attribute: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_param_member(scope, member, attribute)?;
        let result = evaluate(scope, r#"
            (() => {
                const calls = [];
                let throwNext = false;
                class Watched extends HTMLParamElement {
                    static observedAttributes = ['name', 'value', 'type', 'valuetype'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (newValue === 'outer') this[paramMember] = 'inner';
                        if (throwNext) { throwNext = false; direct[paramMember] = 'nested';
                            throw globalThis.paramCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-param', Watched, {extends: 'param'});
                const direct = new Watched();
                const param = document.createElement('param', {is: 'x-watched-param'});
                if (!(direct instanceof Watched) || !(param instanceof Watched) ||
                    direct.localName !== 'param' || param.localName !== 'param' || direct === param)
                    return 'customized factory and direct construction';
                param[paramMember] = null;
                if (calls.length !== 1 || calls[0][0] !== param || calls[0][1] !== paramAttribute ||
                    calls[0][2] !== null || calls[0][3] !== 'null' || calls[0][4] !== null || calls[0][5] !== 'null')
                    return 'original ordinary-null reaction and literal attribute mapping';
                param[paramMember] = 'null';
                if (calls.length !== 2 || calls[1][2] !== 'null' || calls[1][3] !== 'null') return 'same-value reaction';
                param[paramMember] = 'outer';
                if (calls.length !== 4 || calls[2][2] !== 'null' || calls[2][3] !== 'outer' || calls[2][5] !== 'outer' ||
                    calls[3][2] !== 'outer' || calls[3][3] !== 'inner' || calls[3][5] !== 'inner' || param[paramMember] !== 'inner')
                    return 'publication before nested Source reentry';
                throwNext = true;
                param[paramMember] = 'throw';
                if (calls.length !== 6 || calls[4][0] !== param || calls[4][3] !== 'throw' ||
                    calls[5][0] !== direct || calls[5][2] !== null || calls[5][3] !== 'nested' ||
                    param[paramMember] !== 'throw' || direct[paramMember] !== 'nested')
                    return 'nested Source survives reported callback throw';
                class Throwing extends HTMLParamElement {
                    constructor() { super(); globalThis.constructedParam = this;
                        throw globalThis.paramConstructorThrow = {}; }
                }
                customElements.define('x-failed-param', Throwing, {extends: 'param'});
                const failed = document.createElement('param', {is: 'x-failed-param'});
                if (failed !== constructedParam || !(failed instanceof HTMLParamElement) || !(failed instanceof Throwing) ||
                    failed instanceof HTMLUnknownElement) return 'customized failure keeps authentic native interface';
                const descriptor = Object.getOwnPropertyDescriptor(HTMLParamElement.prototype, paramMember);
                descriptor.set.call(failed, null);
                return descriptor.get.call(failed) === 'null' ? 'ok' : 'authentic failed-construction receiver';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{member}");
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
            ("paramCallbackThrow", &callback.exception),
            ("paramConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn param_reads_follow_native_edits_and_retirement() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let param = host
        .in_document(&permit, |scope| {
            evaluate(
                scope,
                r#"
            globalThis.paramContainer = document.createElement('div');
            globalThis.nativeParam = paramContainer.appendChild(document.createElement('param'));
            globalThis.savedParam = nativeParam;
        "#,
            )?;
            let param = evaluate(scope, "nativeParam")?;
            projected_node_id(scope, &param)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        for (_, attribute) in PARAM_MEMBERS {
            batch.set_attribute_ns(
                param,
                Namespace::none(),
                attribute,
                surgeist_dom::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
            );
            batch.set_attribute_ns(
                param,
                Namespace::from_uri("urn:foreign"),
                attribute,
                "foreign",
            );
        }
        batch.set_attribute_ns(param, Namespace::none(), "valueType", "case-distinct");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeParam === savedParam && paramContainer.firstChild === nativeParam &&
            ['name', 'value', 'type', 'valueType'].every(member => nativeParam[member] === '\ud800\0\udc00') &&
            nativeParam.getAttributeNS(null, 'valueType') === 'case-distinct' &&
            nativeParam.getAttributeNS(null, 'valuetype') === '\ud800\0\udc00' &&
            nativeParam.getAttributeNS('urn:foreign', 'valuetype') === 'foreign'
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.paramDestination = new Document();
            paramDestination.adoptNode(nativeParam);
        "#)?;
        let result = evaluate(scope, r#"
            nativeParam === savedParam && nativeParam.ownerDocument === paramDestination && nativeParam.parentNode === null &&
            nativeParam instanceof HTMLParamElement && Object.getPrototypeOf(nativeParam) === HTMLParamElement.prototype &&
            ['name', 'value', 'type', 'valueType'].every(member => nativeParam[member] === '\ud800\0\udc00')
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        // Source adoption detached this exact identity before native retirement.
        batch.destroy(param);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeParam.name",
        "nativeParam.value",
        "nativeParam.type",
        "nativeParam.valueType",
        "nativeParam.name = 'retired'",
        "nativeParam.value = 'retired'",
        "nativeParam.type = 'retired'",
        "nativeParam.valueType = 'retired'",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired native param must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn param_accessors_keep_registered_realms_after_adoption(member: &str, attribute: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let param = {
        let mut batch = DomBatch::new();
        let param = batch.create_element_ns(origin.document(), Namespace::html(), "param");
        session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&param)
            .unwrap()
            .unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(&mut host, &permit, param, "originParam");
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
        set_param_member(scope, member, attribute)?;
        let param = evaluate(scope, "originParam")?;
        assert_ne!(scope.callback_object(&param)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignParamTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                globalThis.savedOriginParam = originParam;
                globalThis.originParamPrototype = Object.getPrototypeOf(originParam);
                if (originParam instanceof HTMLParamElement || originParamPrototype.constructor.name !== 'HTMLParamElement')
                    return 'registered creation realm';
                const descriptor = Object.getOwnPropertyDescriptor(originParamPrototype, paramMember);
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try { accessor.call({}, null); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof foreignParamTypeError) || error instanceof TypeError)
                        return 'accessor function realm exception'; }
                }
                descriptor.set.call(originParam, null);
                if (descriptor.get.call(originParam) !== 'null' || originParam.getAttributeNS(null, paramAttribute) !== 'null')
                    return 'borrowed ordinary null conversion and exact attribute mapping';
                const original = {};
                try { descriptor.set.call(originParam, {toString() { throw original; }}); return 'borrowed conversion throw'; }
                catch (error) { if (error !== original) return 'borrowed original throw identity'; }
                descriptor.set.call(originParam, 'foreign\ud800\0');
                return descriptor.get.call(originParam) === 'foreign\ud800\0' ? 'ok' : 'borrowed native reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{member}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.adopt(param, target.document());
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            originParam === savedOriginParam && originParam.ownerDocument === document &&
            Object.getPrototypeOf(originParam) === originParamPrototype && !(originParam instanceof HTMLParamElement) &&
            originParam[paramMember] === 'foreign\ud800\0' && originParam.getAttributeNS(null, paramAttribute) === 'foreign\ud800\0'
        "#)?;
        assert!(scope.boolean(&result)?, "{member}");
        Ok(())
    }).unwrap();
}

fn param_setters_admit_source_outputs_before_publication(member: &str, attribute: &str) {
    for outputs in [1, 2] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let mut batch = DomBatch::new();
        let param = batch.create_element_ns(target.document(), Namespace::html(), "param");
        let param = session
            .dom()
            .apply(batch, &cancel)
            .unwrap()
            .created(&param)
            .unwrap()
            .unwrap();
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, param, "limitedParam");
        host.in_document(&permit, |scope| set_param_member(scope, member, attribute))
            .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "limitedParam[paramMember] = null").map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!(
                    "one-short {member} Source output must refuse before publication: {result:?}"
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
            assert!(after.node(param).unwrap().attributes().unwrap().is_empty());
        } else {
            result.unwrap();
            let name = attribute.into();
            let attr = after
                .node(param)
                .unwrap()
                .get_attribute_node_ns(&Namespace::none(), &name)
                .unwrap();
            assert_eq!(
                attr.value().code_units(),
                &[0x6e, 0x75, 0x6c, 0x6c],
                "{member} admitted native value"
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

// HTML877d's complete obsolete HTMLParamElement IDL and Reflect naming rule
// specify this order, ordinary DOMStrings, and valueType's valuetype attribute.
pub(super) fn param_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const param = document.createElement('param');
                if (!(param instanceof HTMLParamElement)) return 'param interface';
                if (Object.keys(HTMLParamElement.prototype).join() !== 'name,value,type,valueType')
                    return 'param missing reflected members';
                for (const [member, attribute] of [['name', 'name'], ['value', 'value'],
                    ['type', 'type'], ['valueType', 'valuetype']]) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLParamElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'param descriptor ' + member;
                    if (param[member] !== '') return 'param absent default ' + member;
                    param[member] = null;
                    if (param[member] !== 'null' || param.getAttributeNS(null, attribute) !== 'null')
                        return 'param plain DOMString reflection ' + member;
                }
                return param.hasAttributeNS(null, 'valueType') ? 'camelCase content attribute' : 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
