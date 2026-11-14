use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_source_output_limit,
    expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn pre_width_family_preserves_native_and_web_idl_contracts() {
    for tag in ["pre", "listing", "xmp"] {
        pre_width_brands_precede_numeric_conversion(tag);
        // Bounded independent tables keep cumulative semantic work separate.
        for cases in [
            r#"[['',0],['invalid',0],['+',0],['-',0],['+ 7',0],['--7',0],['\u00a07',0],['\ud8007',0]]"#,
            r#"[[' \t\n\f\r-7tail',-7],['+0007 junk',7],['-0000',0],['2.9',2],['0x10',0],['1e2',1],['7\0tail',7]]"#,
            r#"[['2147483647',2147483647],['-2147483648',-2147483648],['2147483648',0],['-2147483649',0],['999999999999999999999999',0],['000000000000000000000007',7]]"#,
        ] {
            pre_width_getter_cases(tag, cases);
        }
        for cases in [
            "[[1.9,1],[-1.9,-1],[null,0],[undefined,0],[true,1],[false,0],['0x10',16],['1e2',100]]",
            "[[2147483647,2147483647],[-2147483648,-2147483648],[2147483648,-2147483648],[-2147483649,2147483647],[4294967295,-1],[4294967296,0]]",
            "[[4294967297,1],[-4294967297,-1],[8589934593,1],[9007199254740991,-1],[-9007199254740991,1]]",
            "[[NaN,0],[Infinity,0],[-Infinity,0],[-0,0],[0,0],['',0]]",
        ] {
            pre_width_setter_cases(tag, cases);
        }
        pre_width_numeric_coercion_reenters_without_losing_original_errors(tag);
        pre_width_customized_reactions_use_the_original_source(tag);
        pre_width_native_edits_and_retirement_keep_wrapper_identity(tag);
        pre_width_registered_realm_and_output_admission(tag);
    }
}

fn set_pre_tag(scope: &mut Scope<'_>, tag: &str) -> Result<(), Error> {
    let global = scope.global()?;
    let value = scope.string(&tag.encode_utf16().collect::<Vec<_>>())?;
    scope.set(
        &global,
        PropertyKey::String(&"preTag".encode_utf16().collect::<Vec<_>>()),
        &value,
    )
}

fn check_pre_script(tag: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_pre_tag(scope, tag)?;
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
            "width {tag}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        ),
        _ => panic!("width {tag}: {error:?}"),
    });
}

fn pre_width_brands_precede_numeric_conversion(tag: &str) {
    check_pre_script(
        tag,
        r#"
        (() => {
            const element = document.createElement(preTag.toUpperCase());
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:' + preTag);
            const descriptor = Object.getOwnPropertyDescriptor(HTMLPreElement.prototype, 'width');
            if (!(element instanceof HTMLPreElement) || element instanceof HTMLUnknownElement ||
                !(prefixed instanceof HTMLPreElement) || element.localName !== preTag ||
                Object.getPrototypeOf(element) !== HTMLPreElement.prototype ||
                Object.getPrototypeOf(HTMLPreElement) !== HTMLElement ||
                Object.getPrototypeOf(HTMLPreElement.prototype) !== HTMLElement.prototype ||
                descriptor.get.name !== 'get width' || descriptor.set.name !== 'set width') return 'actual interface and descriptors';
            try { new HTMLPreElement(); return 'unregistered HTMLConstructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, -3);
            if (descriptor.get.call(prefixed) !== -3) return 'prefixed authentic receiver';
            const foreign = document.createElementNS('urn:foreign', preTag);
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', preTag.toUpperCase());
            const other = document.createElement('div');
            Object.setPrototypeOf(other, HTMLPreElement.prototype);
            if (foreign instanceof HTMLPreElement || upper instanceof HTMLPreElement) return 'namespace and XML case';
            for (const receiver of [null, {}, document, foreign, upper, other, Object.create(element), new Proxy(element, {})]) {
                let converted = false;
                try { descriptor.get.call(receiver); return 'getter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter brand error'; }
                try { descriptor.set.call(receiver, {valueOf() { converted = true; return 7; }}); return 'setter accepted unauthentic receiver'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'native brand before ToNumber'; }
            }
            return !other.hasAttributeNS(null, 'width') ? 'ok' : 'wrong native family mutated';
        })()
    "#,
    );
}

fn pre_width_getter_cases(tag: &str, cases: &str) {
    let script = format!(
        r#"(() => {{
        const element = document.createElement(preTag);
        element.setAttributeNS('urn:foreign', 'width', '777');
        if (element.width !== 0) return 'foreign attribute ignored';
        for (const [raw, expected] of {cases}) {{
            element.setAttributeNS(null, 'width', raw);
            if (element.width !== expected || element.getAttributeNS(null, 'width') !== raw)
                return 'HTML signed prefix and range ' + JSON.stringify(raw);
        }}
        element.removeAttributeNS(null, 'width');
        return element.width === 0 && element.getAttributeNS('urn:foreign', 'width') === '777'
            ? 'ok' : 'missing default and namespace isolation';
    }})()"#
    );
    check_pre_script(tag, &script);
}

fn pre_width_setter_cases(tag: &str, cases: &str) {
    let script = format!(
        r#"(() => {{
        const element = document.createElement(preTag);
        for (const [input, expected] of {cases}) {{
            element.width = input;
            if (element.width !== expected || Object.is(element.width, -0) ||
                element.getAttributeNS(null, 'width') !== String(expected)) return 'wrapping long and canonical decimal';
        }}
        return 'ok';
    }})()"#
    );
    check_pre_script(tag, &script);
}

fn pre_width_numeric_coercion_reenters_without_losing_original_errors(tag: &str) {
    check_pre_script(
        tag,
        r#"
        (() => {
            const element = document.createElement(preTag);
            const hints = [];
            element.width = {[Symbol.toPrimitive](hint) { hints.push(hint); return -3.9; }};
            if (hints.join() !== 'number' || element.width !== -3 || element.getAttributeNS(null, 'width') !== '-3')
                return 'numeric primitive hint';
            let order = '';
            element.width = {valueOf() { order += 'v'; return {}; }, toString() { order += 's'; return '17.9'; }};
            if (order !== 'vs' || element.width !== 17) return 'ordinary numeric conversion order';
            element.width = {valueOf() {
                element.setAttributeNS(null, 'width', '99tail');
                if (element.width !== 99) throw 'nested read unavailable';
                return -7.9;
            }, toString() { throw 'unexpected string conversion'; }};
            if (element.width !== -7 || element.getAttributeNS(null, 'width') !== '-7') return 'reentry then original numeric write';
            const original = {};
            try { element.width = {valueOf() { throw original; }}; return 'throwing value accepted'; }
            catch (error) { if (error !== original || element.width !== -7) return 'original throw and state'; }
            for (const value of [Symbol(), 1n, {valueOf() { return Symbol(); }}, {valueOf() { return 1n; }}]) {
                try { element.width = value; return 'Symbol or BigInt accepted'; }
                catch (error) { if (!(error instanceof TypeError) || element.width !== -7) return 'conversion TypeError and state'; }
            }
            return 'ok';
        })()
    "#,
    );
}

fn pre_width_customized_reactions_use_the_original_source(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_pre_tag(scope, tag)?;
        let result = evaluate(scope, r#"
            (() => {
                const calls = [];
                class Watched extends HTMLPreElement {
                    static observedAttributes = ['width'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (newValue === '11') this.width = 12;
                    }
                }
                customElements.define('x-watched-pre', Watched, {extends: preTag});
                const direct = new Watched();
                const element = document.createElement(preTag, {is: 'x-watched-pre'});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== preTag || element.localName !== preTag || direct === element) return 'actual constructor tag';
                element.width = 2.9;
                element.width = '2';
                if (calls.length !== 2 || calls[0][0] !== element || calls[0][1] !== 'width' || calls[0][2] !== null ||
                    calls[0][3] !== '2' || calls[0][4] !== null || calls[0][5] !== '2' || calls[1][2] !== '2' || calls[1][3] !== '2')
                    return 'original numeric and same-value reactions';
                element.width = 11;
                if (calls.length !== 4 || calls[2][3] !== '11' || calls[2][5] !== '11' ||
                    calls[3][2] !== '11' || calls[3][3] !== '12' || calls[3][5] !== '12' || element.width !== 12)
                    return 'publication before callback reentry';
                const original = {};
                try { element.width = {valueOf() { direct.width = -3; throw original; }}; return 'coercion throw'; }
                catch (error) { if (error !== original) return 'original coercion throw identity'; }
                if (calls.length !== 5 || calls[4][0] !== direct || calls[4][3] !== '-3' || element.width !== 12)
                    return 'nested Source survives original conversion throw';
                class Throwing extends HTMLPreElement {
                    constructor() { super(); globalThis.constructedPre = this; throw globalThis.preConstructorThrow = {}; }
                }
                customElements.define('x-failed-pre', Throwing, {extends: preTag});
                const failed = document.createElement(preTag, {is: 'x-failed-pre'});
                if (failed !== constructedPre || !(failed instanceof Throwing) || !(failed instanceof HTMLPreElement) ||
                    failed instanceof HTMLUnknownElement || failed.localName !== preTag) return 'authentic customized failure';
                failed.width = -9;
                return failed.width === -9 ? 'ok' : 'failed customized construction keeps authentic reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{tag}");
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
        let original = evaluate(scope, "preConstructorThrow")?;
        let reported = scope.import(report.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        Ok(())
    })
    .unwrap();
}

fn pre_width_native_edits_and_retirement_keep_wrapper_identity(tag: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        set_pre_tag(scope, tag)?;
        evaluate(scope, "globalThis.nativePre = document.createElement(preTag); globalThis.savedPre = nativePre")?;
        let element = evaluate(scope, "nativePre")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(element, Namespace::none(), "width", " \t-2147483648tail");
        batch.set_attribute_ns(element, Namespace::from_uri("urn:foreign"), "width", "777");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "nativePre === savedPre && nativePre.width === -2147483648 && nativePre.getAttributeNS('urn:foreign', 'width') === '777'")?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.in_document(&permit, |scope| {
        evaluate(scope, "globalThis.preDestination = new Document(); preDestination.adoptNode(nativePre)")?;
        let result = evaluate(scope, "nativePre === savedPre && nativePre.ownerDocument === preDestination && nativePre.parentNode === null && nativePre instanceof HTMLPreElement && nativePre.localName === preTag && nativePre.width === -2147483648")?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in ["nativePre.width", "nativePre.width = -7"] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired {tag} must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn pre_width_registered_realm_and_output_admission(tag: &str) {
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
    expose_public_node_for_source_test(&mut host, &permit, element, "originPre");
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
        let value = evaluate(scope, "originPre")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignPreTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"(() => {
            globalThis.originPrePrototype = Object.getPrototypeOf(originPre);
            if (originPre instanceof HTMLPreElement || originPrePrototype.constructor.name !== 'HTMLPreElement') return 'creation realm';
            const descriptor = Object.getOwnPropertyDescriptor(originPrePrototype, 'width');
            for (const accessor of [descriptor.get, descriptor.set]) {
                try { accessor.call({}, 7); return 'borrowed receiver'; }
                catch (error) { if (!(error instanceof foreignPreTypeError) || error instanceof TypeError) return 'function realm error'; }
            }
            const original = {};
            try { descriptor.set.call(originPre, {valueOf() { throw original; }}); return 'borrowed conversion throw'; }
            catch (error) { if (error !== original) return 'original borrowed conversion exception'; }
            descriptor.set.call(originPre, 4294967295);
            return descriptor.get.call(originPre) === -1 && originPre.getAttributeNS(null, 'width') === '-1' ? 'ok' : 'borrowed wrapping long';
        })()"#)?;
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
        let result = evaluate(scope, "originPre.ownerDocument === document && Object.getPrototypeOf(originPre) === originPrePrototype && !(originPre instanceof HTMLPreElement) && originPre.width === -1")?;
        assert!(scope.boolean(&result)?, "{tag}");
        Ok(())
    }).unwrap();
    host.stop().into_session().unwrap();

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
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedPre");
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(scope, "limitedPre.width = -2147483648").map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs == 1 {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short width Source output: {result:?}")
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
            assert_eq!(
                after.node(element).unwrap().attribute(&"width".into()),
                Some(&"-2147483648".into())
            );
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
}

// HTML877d's obsolete HTMLPreElement.width is a reflected long, including
// listing/xmp's HTMLPreElement mapping and the signed-long reflection rules.
pub(super) fn pre_width_reflects_the_canonical_signed_attribute() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const elements = ['pre', 'listing', 'xmp'].map(tag => document.createElement(tag));
                if (!elements.every(element => element instanceof HTMLPreElement)) return 'pre family interfaces';
                if (Object.keys(HTMLPreElement.prototype).join() !== 'width') return 'pre missing reflected width';
                const descriptor = Object.getOwnPropertyDescriptor(HTMLPreElement.prototype, 'width');
                if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                    !descriptor.enumerable || !descriptor.configurable ||
                    descriptor.get.length !== 0 || descriptor.set.length !== 1) return 'pre width descriptor';
                for (const element of elements) {
                    if (element.width !== 0) return element.localName + ' missing width default';
                    element.width = -7;
                    if (element.width !== -7 || element.getAttributeNS(null, 'width') !== '-7')
                        return element.localName + ' canonical signed reflection';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
