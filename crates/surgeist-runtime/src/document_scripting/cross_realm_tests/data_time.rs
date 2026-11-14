use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

const DATA_TIME_MEMBERS: [(&str, &str, &str); 2] =
    [("data", "value", "value"), ("time", "dateTime", "datetime")];

pub(super) fn data_time_families_preserve_native_and_web_idl_contracts() {
    for (tag, member, attribute) in DATA_TIME_MEMBERS {
        data_time_brands_precede_conversion(tag, member, attribute);
        data_time_strings_reflect_attributes_without_content_fallback(tag, member, attribute);
        data_time_customized_constructors_deliver_original_reactions(tag, member, attribute);
        data_time_reads_follow_native_edits_and_retirement(tag, member, attribute);
        data_time_accessors_keep_registered_realms_after_adoption(tag, member, attribute);
        data_time_setters_admit_source_outputs_before_publication(tag, member, attribute);
    }
}

fn set_data_time_member(
    scope: &mut Scope<'_>,
    tag: &str,
    member: &str,
    attribute: &str,
) -> Result<(), Error> {
    let global = scope.global()?;
    for (key, value) in [
        ("dataTimeTag", tag),
        ("dataTimeMember", member),
        ("dataTimeAttribute", attribute),
    ] {
        let value = scope.string(&value.encode_utf16().collect::<Vec<_>>())?;
        scope.set(
            &global,
            PropertyKey::String(&key.encode_utf16().collect::<Vec<_>>()),
            &value,
        )?;
    }
    Ok(())
}

fn check_data_time_script(tag: &str, member: &str, attribute: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_data_time_member(scope, tag, member, attribute)?;
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

fn data_time_brands_precede_conversion(tag: &str, member: &str, attribute: &str) {
    check_data_time_script(
        tag,
        member,
        attribute,
        r#"
        (() => {
            const Type = dataTimeTag === 'data' ? HTMLDataElement : HTMLTimeElement;
            const element = document.createElement(dataTimeTag.toUpperCase());
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + dataTimeTag);
            const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, dataTimeMember);
            if (Object.getPrototypeOf(Type) !== HTMLElement || Object.getPrototypeOf(Type.prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(element) !== Type.prototype || element.localName !== dataTimeTag ||
                !(prefixed instanceof Type) || Object.keys(Type.prototype).join() !== dataTimeMember ||
                descriptor.get.name !== 'get ' + dataTimeMember || descriptor.set.name !== 'set ' + dataTimeMember)
                return 'classification inheritance and accessor names';
            try { new Type(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, 'prefixed');
            if (descriptor.get.call(prefixed) !== 'prefixed') return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', dataTimeTag);
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', dataTimeTag.toUpperCase());
            const other = document.createElement(dataTimeTag === 'data' ? 'time' : 'data');
            Object.setPrototypeOf(other, Type.prototype);
            if (foreign instanceof Type || upper instanceof Type) return 'native namespace and XML case';
            for (const receiver of [null, {}, document, foreign, upper, other,
                Object.create(element), new Proxy(element, {})]) {
                let converted = false;
                try { descriptor.get.call(receiver); return 'getter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                try { descriptor.set.call(receiver, {toString() { converted = true; return 'bad'; }});
                    return 'setter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before conversion'; }
            }
            return !other.hasAttributeNS(null, dataTimeAttribute) ? 'ok' : 'invalid receiver mutated native state';
        })()
    "#,
    );
}

fn data_time_strings_reflect_attributes_without_content_fallback(
    tag: &str,
    member: &str,
    attribute: &str,
) {
    check_data_time_script(
        tag,
        member,
        attribute,
        r#"
        (() => {
            const element = document.createElement(dataTimeTag);
            const child = element.appendChild(document.createTextNode('2026-11-14'));
            if (element[dataTimeMember] !== '' || element.getAttributeNS(null, dataTimeAttribute) !== null)
                return 'missing attribute reflects empty despite actual child date text';
            if (dataTimeTag === 'time') {
                element.setAttributeNS(null, 'dateTime', 'case-distinct');
                element.setAttributeNS(null, 'datetime', 'lowercase');
                if (element.dateTime !== 'lowercase' || element.getAttributeNS(null, 'dateTime') !== 'case-distinct' ||
                    element.getAttributeNS(null, 'datetime') !== 'lowercase' || element.getAttribute('dateTime') !== 'lowercase')
                    return 'literal lowercase reflection and exact namespace case controls';
                element.removeAttributeNS(null, 'datetime');
                if (element.dateTime !== '') return 'camelCase content attribute is distinct';
            }
            element.setAttributeNS('urn:foreign', dataTimeAttribute, 'foreign');
            if (element[dataTimeMember] !== '') return 'foreign namespace ignored';
            element[dataTimeMember] = null;
            if (element[dataTimeMember] !== 'null' || element.getAttributeNS(null, dataTimeAttribute) !== 'null')
                return 'ordinary null DOMString';
            element[dataTimeMember] = {toString() { return null; }};
            if (element[dataTimeMember] !== 'null') return 'object returning null';
            element[dataTimeMember] = undefined;
            if (element[dataTimeMember] !== 'undefined') return 'undefined DOMString';
            const exact = ' Raw\ud800\0\udc00 ';
            let count = 0;
            element[dataTimeMember] = {toString() {
                count++;
                element.setAttributeNS(null, dataTimeAttribute, 'nested');
                if (element[dataTimeMember] !== 'nested') throw 'nested reflection unavailable';
                return exact;
            }};
            if (count !== 1 || element[dataTimeMember] !== exact || element.getAttributeNS(null, dataTimeAttribute) !== exact ||
                element.getAttributeNS('urn:foreign', dataTimeAttribute) !== 'foreign') return 'UTF16 reentry and namespace isolation';
            const original = {};
            try { element[dataTimeMember] = {toString() { throw original; }}; return 'throwing value accepted'; }
            catch (error) { if (error !== original || element[dataTimeMember] !== exact) return 'original exception and state'; }
            for (const value of [Symbol(), {toString() { return Symbol(); }}]) {
                try { element[dataTimeMember] = value; return 'Symbol accepted'; }
                catch (error) { if (!(error instanceof TypeError) || element[dataTimeMember] !== exact) return 'Symbol state'; }
            }
            element.setAttributeNS(null, dataTimeAttribute, ' 2026-99-99 INVALID +25:99 ');
            if (element[dataTimeMember] !== ' 2026-99-99 INVALID +25:99 ') return 'raw content without date parsing';
            element.removeAttributeNS(null, dataTimeAttribute);
            child.data = '2027-01-01';
            if (element[dataTimeMember] !== '' || element.firstChild !== child || child.data !== '2027-01-01' ||
                element.getAttributeNS('urn:foreign', dataTimeAttribute) !== 'foreign')
                return 'attribute removal does not fall back to live child content';
            return dataTimeTag !== 'time' || element.getAttributeNS(null, 'dateTime') === 'case-distinct'
                ? 'ok' : 'case-distinct content attribute changed';
        })()
    "#,
    );
}

fn data_time_customized_constructors_deliver_original_reactions(
    tag: &str,
    member: &str,
    attribute: &str,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_data_time_member(scope, tag, member, attribute)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = dataTimeTag === 'data' ? HTMLDataElement : HTMLTimeElement;
                const calls = [];
                let throwNext = false;
                class Watched extends Type {
                    static observedAttributes = [dataTimeAttribute];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (newValue === 'outer') this[dataTimeMember] = 'inner';
                        if (throwNext) { throwNext = false; direct[dataTimeMember] = 'nested';
                            throw globalThis.dataTimeCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-' + dataTimeTag, Watched, {extends: dataTimeTag});
                const direct = new Watched();
                const element = document.createElement(dataTimeTag, {is: 'x-watched-' + dataTimeTag});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== dataTimeTag || element.localName !== dataTimeTag || direct === element)
                    return 'customized factory and actual direct constructor tag';
                element[dataTimeMember] = null;
                if (calls.length !== 1 || calls[0][0] !== element || calls[0][1] !== dataTimeAttribute ||
                    calls[0][2] !== null || calls[0][3] !== 'null' || calls[0][4] !== null || calls[0][5] !== 'null')
                    return 'original ordinary-null reaction and literal attribute mapping';
                element[dataTimeMember] = 'null';
                if (calls.length !== 2 || calls[1][2] !== 'null' || calls[1][3] !== 'null') return 'same-value reaction';
                element[dataTimeMember] = 'outer';
                if (calls.length !== 4 || calls[2][2] !== 'null' || calls[2][3] !== 'outer' || calls[2][5] !== 'outer' ||
                    calls[3][2] !== 'outer' || calls[3][3] !== 'inner' || calls[3][5] !== 'inner' || element[dataTimeMember] !== 'inner')
                    return 'publication before nested Source reentry';
                throwNext = true;
                element[dataTimeMember] = 'throw';
                if (calls.length !== 6 || calls[4][0] !== element || calls[4][3] !== 'throw' ||
                    calls[5][0] !== direct || calls[5][2] !== null || calls[5][3] !== 'nested' ||
                    element[dataTimeMember] !== 'throw' || direct[dataTimeMember] !== 'nested')
                    return 'nested Source survives reported callback throw';
                class Throwing extends Type {
                    constructor() { super(); globalThis.constructedDataTime = this;
                        throw globalThis.dataTimeConstructorThrow = {}; }
                }
                customElements.define('x-failed-' + dataTimeTag, Throwing, {extends: dataTimeTag});
                const failed = document.createElement(dataTimeTag, {is: 'x-failed-' + dataTimeTag});
                if (failed !== constructedDataTime || !(failed instanceof Type) || !(failed instanceof Throwing) ||
                    failed.localName !== dataTimeTag || failed instanceof HTMLUnknownElement)
                    return 'customized failure keeps authentic native interface';
                const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, dataTimeMember);
                descriptor.set.call(failed, null);
                return descriptor.get.call(failed) === 'null' ? 'ok' : 'authentic failed-construction receiver';
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
            ("dataTimeCallbackThrow", &callback.exception),
            ("dataTimeConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn data_time_reads_follow_native_edits_and_retirement(tag: &str, member: &str, attribute: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        set_data_time_member(scope, tag, member, attribute)?;
        evaluate(scope, r#"
            globalThis.dataTimeContainer = document.createElement('div');
            globalThis.nativeDataTime = dataTimeContainer.appendChild(document.createElement(dataTimeTag));
            nativeDataTime.appendChild(document.createTextNode('2026-11-14'));
            globalThis.savedDataTime = nativeDataTime;
            globalThis.dataTimePrototype = Object.getPrototypeOf(nativeDataTime);
        "#)?;
        let element = evaluate(scope, "nativeDataTime")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            element,
            Namespace::none(),
            attribute,
            surgeist_dom::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        batch.set_attribute_ns(
            element,
            Namespace::from_uri("urn:foreign"),
            attribute,
            "foreign",
        );
        if tag == "time" {
            batch.set_attribute_ns(element, Namespace::none(), "dateTime", "case-distinct");
        }
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeDataTime === savedDataTime && dataTimeContainer.firstChild === nativeDataTime &&
            nativeDataTime[dataTimeMember] === '\ud800\0\udc00' &&
            nativeDataTime.getAttributeNS(null, dataTimeAttribute) === '\ud800\0\udc00' &&
            nativeDataTime.getAttributeNS('urn:foreign', dataTimeAttribute) === 'foreign' &&
            (dataTimeTag !== 'time' || nativeDataTime.getAttributeNS(null, 'dateTime') === 'case-distinct')
        "#)?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.remove_attribute_ns(element, Namespace::none(), attribute);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "nativeDataTime[dataTimeMember] === '' && nativeDataTime.firstChild.data === '2026-11-14' && nativeDataTime.getAttributeNS('urn:foreign', dataTimeAttribute) === 'foreign'")?;
        assert!(scope.boolean(&result)?, "{tag}");
        evaluate(scope, r#"
            globalThis.dataTimeDestination = new Document();
            dataTimeDestination.adoptNode(nativeDataTime);
        "#)?;
        let result = evaluate(scope, r#"
            nativeDataTime === savedDataTime && nativeDataTime.ownerDocument === dataTimeDestination &&
            nativeDataTime.parentNode === null && Object.getPrototypeOf(nativeDataTime) === dataTimePrototype &&
            nativeDataTime.localName === dataTimeTag && nativeDataTime[dataTimeMember] === '' &&
            nativeDataTime.firstChild.data === '2026-11-14'
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
    for expression in [
        "nativeDataTime[dataTimeMember]",
        "nativeDataTime[dataTimeMember] = 'retired'",
    ] {
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

fn data_time_accessors_keep_registered_realms_after_adoption(
    tag: &str,
    member: &str,
    attribute: &str,
) {
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
    expose_public_node_for_source_test(&mut host, &permit, element, "originDataTime");
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
        set_data_time_member(scope, tag, member, attribute)?;
        let value = evaluate(scope, "originDataTime")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignDataTimeTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                const Type = dataTimeTag === 'data' ? HTMLDataElement : HTMLTimeElement;
                globalThis.savedOriginDataTime = originDataTime;
                globalThis.originDataTimePrototype = Object.getPrototypeOf(originDataTime);
                if (originDataTime instanceof Type || originDataTimePrototype.constructor.name !== Type.name)
                    return 'registered creation realm';
                const descriptor = Object.getOwnPropertyDescriptor(originDataTimePrototype, dataTimeMember);
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try { accessor.call({}, null); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof foreignDataTimeTypeError) || error instanceof TypeError)
                        return 'accessor function realm exception'; }
                }
                descriptor.set.call(originDataTime, null);
                if (descriptor.get.call(originDataTime) !== 'null' || originDataTime.getAttributeNS(null, dataTimeAttribute) !== 'null')
                    return 'borrowed ordinary null conversion and literal attribute mapping';
                const original = {};
                try { descriptor.set.call(originDataTime, {toString() { throw original; }}); return 'borrowed conversion throw'; }
                catch (error) { if (error !== original) return 'borrowed original exception identity'; }
                descriptor.set.call(originDataTime, 'foreign\ud800\0');
                return descriptor.get.call(originDataTime) === 'foreign\ud800\0' ? 'ok' : 'borrowed native reflection';
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
        let result = evaluate(
            scope,
            r#"
            originDataTime === savedOriginDataTime && originDataTime.ownerDocument === document &&
            Object.getPrototypeOf(originDataTime) === originDataTimePrototype &&
            originDataTime[dataTimeMember] === 'foreign\ud800\0' &&
            originDataTime.getAttributeNS(null, dataTimeAttribute) === 'foreign\ud800\0'
        "#,
        )?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    })
    .unwrap();
}

fn data_time_setters_admit_source_outputs_before_publication(
    tag: &str,
    member: &str,
    attribute: &str,
) {
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
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedDataTime");
        host.in_document(&permit, |scope| {
            set_data_time_member(scope, tag, member, attribute)
        })
        .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "limitedDataTime[dataTimeMember] = null").map(drop)
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
            let name = attribute.into();
            let attr = after
                .node(element)
                .unwrap()
                .get_attribute_node_ns(&Namespace::none(), &name)
                .unwrap();
            assert_eq!(
                attr.value().code_units(),
                &[0x6e, 0x75, 0x6c, 0x6c],
                "{tag} admitted native value"
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

// HTML877d's HTMLDataElement/HTMLTimeElement IDLs require ordinary DOMString
// reflection. HTML's Reflect naming rule maps dateTime to datetime.
pub(super) fn data_and_time_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                for (const [tag, Type, member, attribute] of [
                    ['data', HTMLDataElement, 'value', 'value'],
                    ['time', HTMLTimeElement, 'dateTime', 'datetime'],
                ]) {
                    const element = document.createElement(tag);
                    if (!(element instanceof Type)) return tag + ' interface';
                    if (Object.keys(Type.prototype).join() !== member) return tag + ' missing reflected member';
                    const descriptor = Object.getOwnPropertyDescriptor(Type.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return tag + ' descriptor';
                    element.appendChild(document.createTextNode('2026-11-14'));
                    if (element[member] !== '') return tag + ' reflection must not fall back to child text';
                    element[member] = null;
                    if (element[member] !== 'null' || element.getAttributeNS(null, attribute) !== 'null')
                        return tag + ' ordinary DOMString reflection';
                    if (tag === 'time' && element.hasAttributeNS(null, 'dateTime')) return 'camelCase content attribute';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
