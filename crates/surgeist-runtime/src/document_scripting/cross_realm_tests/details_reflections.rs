use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, attach_public_nodes_with_registry,
    attach_public_nodes_with_source_output_limit, expose_public_node_for_source_test,
    projected_node_id,
};
use crate::{
    internal::element_tasks::DomElementTaskProgress,
    internal::element_tasks::DomElementTaskRunLimits,
    internal::element_tasks::DomElementTaskRuntimeError,
};
use surgeist_dom::task::DomElementTaskSourceKind;

// HTML877d's HTMLDetailsElement IDL specifies name then open. This semantic
// fixture explicitly funds native transition intentions for the open changes.
pub(super) fn details_members_reflect_canonical_attributes() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_details(&cancel, 0, 8);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const details = document.createElement('details');
                if (!(details instanceof HTMLDetailsElement)) return 'details interface';
                if (Object.keys(HTMLDetailsElement.prototype).join() !== 'name,open')
                    return 'details missing reflected members';
                for (const member of ['name', 'open']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLDetailsElement.prototype, member);
                    if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                        !descriptor.enumerable || !descriptor.configurable ||
                        descriptor.get.length !== 0 || descriptor.set.length !== 1)
                        return 'details descriptor ' + member;
                }
                if (details.name !== '' || details.open !== false) return 'details absent defaults';
                details.name = null;
                if (details.name !== 'null' || details.getAttributeNS(null, 'name') !== 'null')
                    return 'details ordinary name DOMString';
                details.open = true;
                if (details.open !== true || details.getAttributeNS(null, 'open') !== '')
                    return 'details Boolean presence';
                details.open = false;
                return details.open === false && !details.hasAttributeNS(null, 'open')
                    ? 'ok' : 'details Boolean removal';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn details_properties_preserve_native_and_web_idl_contracts() {
    for member in ["name", "open"] {
        details_brands_validate_before_conversion(member);
        details_accessors_keep_registered_realms_after_adoption(member);
        details_source_outputs_are_admitted_before_publication(member);
    }
    details_name_reflects_exact_dom_strings();
    details_open_uses_presence_and_to_boolean();
    details_opening_closes_peer_but_rename_and_insertion_close_given();
    details_groups_follow_ordinary_tree_scopes();
    details_customized_constructors_deliver_original_reactions();
    details_group_opening_delivers_both_original_reactions();
    details_native_edits_and_retirement_preserve_identity();
    details_properties_queue_coalesced_trusted_toggle_tasks();
    details_zero_intention_capacity_refuses_only_transitions();
    details_short_task_custody_preserves_original_intentions();
}

fn set_details_member(scope: &mut Scope<'_>, member: &str) -> Result<(), Error> {
    let global = scope.global()?;
    let value = scope.string(&member.encode_utf16().collect::<Vec<_>>())?;
    scope.set(
        &global,
        PropertyKey::String(&"detailsMember".encode_utf16().collect::<Vec<_>>()),
        &value,
    )
}

fn check_details_script(member: &str, script: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_details(&cancel, 0, 64);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        set_details_member(scope, member)?;
        let value = evaluate(scope, script)?;
        assert_eq!(
            String::from_utf16_lossy(&scope.to_utf16(&value)?),
            "ok",
            "details.{member}"
        );
        Ok(())
    })
    .unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => panic!(
            "details.{member}: guest={:?}; source={:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::internal::dom_source::DomSourceDriveError>()
        ),
        _ => panic!("details.{member}: {error:?}"),
    });
}

fn details_brands_validate_before_conversion(member: &str) {
    check_details_script(
        member,
        r#"
        (() => {
            const element = document.createElement('DETAILS');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:details');
            const descriptor = Object.getOwnPropertyDescriptor(HTMLDetailsElement.prototype, detailsMember);
            if (Object.getPrototypeOf(HTMLDetailsElement) !== HTMLElement ||
                Object.getPrototypeOf(HTMLDetailsElement.prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(element) !== HTMLDetailsElement.prototype ||
                element instanceof HTMLUnknownElement || !(prefixed instanceof HTMLDetailsElement) ||
                descriptor.get.name !== 'get ' + detailsMember || descriptor.set.name !== 'set ' + detailsMember)
                return 'inheritance classification accessor names';
            try { new HTMLDetailsElement(); return 'unregistered constructor'; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            descriptor.set.call(prefixed, detailsMember === 'name' ? 'prefixed' : true);
            if (descriptor.get.call(prefixed) !== (detailsMember === 'name' ? 'prefixed' : true))
                return 'authentic prefixed receiver';
            const foreign = document.createElementNS('urn:foreign', 'details');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'DETAILS');
            const wrong = document.createElement('div');
            Object.setPrototypeOf(wrong, HTMLDetailsElement.prototype);
            if (foreign instanceof HTMLDetailsElement || upper instanceof HTMLDetailsElement) return 'namespace XML case';
            for (const receiver of [null, {}, document, foreign, upper, wrong,
                Object.create(element), new Proxy(element, {})]) {
                let coerced = false;
                try { descriptor.get.call(receiver); return 'getter accepted wrong brand'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                try { descriptor.set.call(receiver, {[Symbol.toPrimitive]() { coerced = true; throw 0; },
                    toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; }});
                    return 'setter accepted wrong brand'; }
                catch (error) { if (!(error instanceof TypeError) || coerced) return 'brand before conversion'; }
            }
            return !wrong.hasAttributeNS(null, detailsMember) ? 'ok' : 'wrong brand mutation';
        })()
    "#,
    );
}

fn details_name_reflects_exact_dom_strings() {
    check_details_script(
        "name",
        r#"
        (() => {
            const element = document.createElement('details');
            element.setAttributeNS('urn:foreign', 'name', 'foreign');
            element.setAttributeNS(null, 'NAME', 'case-sensitive');
            if (element.name !== '') return 'exact null namespace lowercase name';
            const exact = ' Raw\ud800\0\udc00 ';
            if (exact.length !== 8 || exact.charCodeAt(4) !== 0xd800 || exact.charCodeAt(5) !== 0 ||
                exact.charCodeAt(6) !== 0xdc00) return 'literal code unit oracle';
            for (const [input, expected] of [[null, 'null'], [undefined, 'undefined'],
                [{toString() { return null; }}, 'null'], [exact, exact], ['', ''], [' Case ', ' Case ']]) {
                element.name = input;
                if (element.name !== expected || element.getAttributeNS(null, 'name') !== expected ||
                    element.getAttributeNS('urn:foreign', 'name') !== 'foreign' ||
                    element.getAttributeNS(null, 'NAME') !== 'case-sensitive') return 'raw DOMString reflection';
            }
            element.name = 'before';
            const original = {};
            try { element.name = {toString() { throw original; }}; return 'conversion throw swallowed'; }
            catch (error) { if (error !== original || element.name !== 'before') return 'original conversion throw'; }
            try { element.name = Symbol(); return 'Symbol accepted'; }
            catch (error) { if (!(error instanceof TypeError) || element.name !== 'before') return 'Symbol error'; }
            const order = [];
            element.name = {[Symbol.toPrimitive](hint) {
                order.push(hint); element.name = 'inner';
                if (element.name !== 'inner') throw 0;
                return 'outer';
            }};
            if (order.join() !== 'string' || element.name !== 'outer') return 'string hint and nested Source';
            try { element.name = {toString() { element.name = 'retained'; throw original; }}; return 'nested throw'; }
            catch (error) { if (error !== original || element.name !== 'retained') return 'nested publication and original throw'; }
            element.removeAttributeNS(null, 'name');
            return element.name === '' && element.getAttributeNS('urn:foreign', 'name') === 'foreign'
                ? 'ok' : 'absence and foreign isolation';
        })()
    "#,
    );
}

fn details_open_uses_presence_and_to_boolean() {
    check_details_script(
        "open",
        r#"
        (() => {
            const element = document.createElement('details');
            element.setAttributeNS('urn:foreign', 'open', 'foreign');
            element.setAttributeNS(null, 'OPEN', 'case-sensitive');
            if (element.open) return 'foreign and uppercase ignored';
            for (const raw of ['', 'false', '0', 'OPEN', '\ud800\0\udc00']) {
                element.setAttributeNS(null, 'open', raw);
                if (!element.open || element.getAttributeNS(null, 'open') !== raw) return 'presence independent of text';
            }
            let coerced = false;
            const object = {[Symbol.toPrimitive]() { coerced = true; throw 0; },
                toString() { coerced = true; throw 0; }, valueOf() { coerced = true; throw 0; }};
            for (const value of [true, 'false', '0', 1, -1, Infinity, 1n, -1n, Symbol(), object,
                [], {}, function() {}, new Boolean(false), new Number(0)]) {
                element.open = value;
                if (!element.open || element.getAttributeNS(null, 'open') !== '' || coerced)
                    return 'truthy no coercion';
            }
            for (const value of [false, null, undefined, '', 0, -0, NaN, 0n]) {
                element.open = true;
                element.open = value;
                if (element.open || element.hasAttributeNS(null, 'open')) return 'falsy removes presence';
            }
            return element.getAttributeNS('urn:foreign', 'open') === 'foreign' &&
                element.getAttributeNS(null, 'OPEN') === 'case-sensitive' ? 'ok' : 'expanded name isolation';
        })()
    "#,
    );
}

// HTML877d interactive 205..289: opening closes a peer, while name changes
// and insertion close the given open element. Empty names do not form groups.
fn details_opening_closes_peer_but_rename_and_insertion_close_given() {
    check_details_script(
        "group mutation",
        r#"
        (() => {
            const container = document.createElement('div');
            const first = container.appendChild(document.createElement('details'));
            const second = container.appendChild(document.createElement('details'));
            first.name = second.name = 'group\ud800\0';
            first.open = true;
            second.open = true;
            if (first.open || !second.open || first.hasAttributeNS(null, 'open')) return 'opening closes peer';
            first.name = 'other'; first.open = true;
            if (!first.open || !second.open) return 'distinct exact names';
            first.name = second.name;
            if (first.open || !second.open) return 'renaming closes given';
            const inserted = document.createElement('details');
            inserted.name = second.name; inserted.open = true;
            if (!inserted.open || !second.open) return 'different ordinary trees';
            container.appendChild(inserted);
            if (inserted.open || !second.open) return 'insertion closes given';
            first.name = ''; second.name = '';
            first.open = true;
            if (!first.open || !second.open) return 'empty name excluded';
            first.name = 'Case'; second.name = 'case';
            if (!first.open || !second.open) return 'case-sensitive names';
            first.setAttributeNS('urn:foreign', 'name', 'case');
            return first.open && second.open ? 'ok' : 'foreign name cannot join group';
        })()
    "#,
    );
}

fn details_groups_follow_ordinary_tree_scopes() {
    check_details_script(
        "tree scopes",
        r#"
        (() => {
            const container = document.createElement('div');
            const light = container.appendChild(document.createElement('details'));
            const host = container.appendChild(document.createElement('div'));
            const shadow = host.attachShadow({mode: 'closed'});
            const inner = shadow.appendChild(document.createElement('details'));
            light.name = inner.name = 'group'; light.open = inner.open = true;
            if (!light.open || !inner.open) return 'shadow tree separation';
            const peer = shadow.appendChild(document.createElement('details'));
            peer.name = 'group'; peer.open = true;
            if (!peer.open || inner.open || !light.open) return 'same shadow ordinary tree group';
            const template = document.createElement('template');
            container.appendChild(template);
            const first = template.content.appendChild(document.createElement('details'));
            const second = template.content.appendChild(document.createElement('details'));
            first.name = second.name = 'group'; first.open = true; second.open = true;
            return !first.open && second.open && light.open && peer.open
                ? 'ok' : 'template fragment separate ordinary tree';
        })()
    "#,
    );
}

fn details_customized_constructors_deliver_original_reactions() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_details(&cancel, 0, 16);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const calls = [];
                let reenter = false, throwNext = false;
                class Watched extends HTMLDetailsElement {
                    static observedAttributes = ['name', 'open'];
                    attributeChangedCallback(name, oldValue, newValue, namespace) {
                        calls.push([this, name, oldValue, newValue, namespace, this.getAttributeNS(namespace, name)]);
                        if (reenter && name === 'open' && newValue === '') { reenter = false; this.open = false; }
                        if (throwNext) { throwNext = false; direct.name = 'nested';
                            throw globalThis.detailsCallbackThrow = {}; }
                    }
                }
                customElements.define('x-watched-details', Watched, {extends: 'details'});
                const direct = new Watched();
                const element = document.createElement('details', {is: 'x-watched-details'});
                if (!(direct instanceof Watched) || !(element instanceof Watched) ||
                    direct.localName !== 'details' || element.localName !== 'details' || direct === element)
                    return 'actual factory and direct constructor';
                element.name = null;
                if (calls.length !== 1 || calls[0][0] !== element || calls[0][1] !== 'name' ||
                    calls[0][2] !== null || calls[0][3] !== 'null' || calls[0][4] !== null || calls[0][5] !== 'null')
                    return 'synchronous original name reaction';
                element.name = null;
                element.open = true;
                element.open = true;
                element.open = false;
                if (calls.length !== 5 || calls[1][2] !== 'null' || calls[1][3] !== 'null' ||
                    calls[2][1] !== 'open' || calls[2][2] !== null || calls[2][3] !== '' ||
                    calls[3][2] !== '' || calls[3][3] !== '' || calls[4][2] !== '' ||
                    calls[4][3] !== null || calls[4][5] !== null || element.open)
                    return 'same-value set and Boolean removal reactions';
                reenter = true;
                element.open = true;
                if (calls.length !== 7 || calls[5][2] !== null || calls[5][3] !== '' || calls[5][5] !== '' ||
                    calls[6][2] !== '' || calls[6][3] !== null || calls[6][5] !== null || element.open)
                    return 'published open before nested Source close';
                throwNext = true;
                element.name = 'outer';
                if (calls.length !== 9 || calls[7][0] !== element || calls[7][3] !== 'outer' ||
                    calls[8][0] !== direct || calls[8][1] !== 'name' || calls[8][3] !== 'nested' ||
                    element.name !== 'outer' || direct.name !== 'nested') return 'nested Source survives callback report';
                class Throwing extends HTMLDetailsElement {
                    constructor() { super(); globalThis.constructedDetails = this;
                        throw globalThis.detailsConstructorThrow = {}; }
                }
                customElements.define('x-failed-details', Throwing, {extends: 'details'});
                const failed = document.createElement('details', {is: 'x-failed-details'});
                if (failed !== constructedDetails || !(failed instanceof Throwing) ||
                    !(failed instanceof HTMLDetailsElement) || failed instanceof HTMLUnknownElement)
                    return 'authentic failed customized construction';
                for (const member of ['name', 'open']) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLDetailsElement.prototype, member);
                    descriptor.set.call(failed, member === 'name' ? 'failed' : true);
                    if (descriptor.get.call(failed) !== (member === 'name' ? 'failed' : true))
                        return 'failed customized receiver remains native Details';
                }
                return 'ok';
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
            ("detailsCallbackThrow", &callback.exception),
            ("detailsConstructorThrow", &constructor.exception),
        ] {
            let original = evaluate(scope, expression)?;
            let reported = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        Ok(())
    })
    .unwrap();
}

fn details_native_edits_and_retirement_preserve_identity() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_details(&cancel, 0, 8);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let element = host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.detailsContainer = document.createElement('div');
            globalThis.nativeDetails = detailsContainer.appendChild(document.createElement('details'));
            globalThis.savedDetails = nativeDetails;
            globalThis.detailsPrototype = Object.getPrototypeOf(nativeDetails);
        "#)?;
        let element = evaluate(scope, "nativeDetails")?;
        projected_node_id(scope, &element)
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            element,
            Namespace::none(),
            "name",
            surgeist_html::values::DocumentString::from_utf16([0xd800, 0, 0xdc00]),
        );
        batch.set_attribute_ns(element, Namespace::none(), "open", "false");
        batch.set_attribute_ns(
            element,
            Namespace::from_uri("urn:foreign"),
            "open",
            "foreign",
        );
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"
            nativeDetails === savedDetails && detailsContainer.firstChild === nativeDetails &&
            nativeDetails.name === '\ud800\0\udc00' && nativeDetails.name.length === 3 &&
            nativeDetails.name.charCodeAt(0) === 0xd800 && nativeDetails.name.charCodeAt(1) === 0 &&
            nativeDetails.name.charCodeAt(2) === 0xdc00 && nativeDetails.open === true &&
            nativeDetails.getAttributeNS(null, 'open') === 'false'
        "#,
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.remove_attribute_ns(element, Namespace::none(), "open");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            !nativeDetails.open && nativeDetails.getAttributeNS('urn:foreign', 'open') === 'foreign'
        "#)?;
        assert!(scope.boolean(&result)?);
        evaluate(scope, "globalThis.detailsDestination = new Document(); detailsDestination.adoptNode(nativeDetails)")?;
        let result = evaluate(scope, r#"
            nativeDetails === savedDetails && nativeDetails.ownerDocument === detailsDestination &&
            nativeDetails.parentNode === null && Object.getPrototypeOf(nativeDetails) === detailsPrototype &&
            nativeDetails.name === '\ud800\0\udc00' && !nativeDetails.open
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeDetails.name",
        "nativeDetails.open",
        "nativeDetails.name = 'retired'",
        "nativeDetails.open = true",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired Details must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn details_accessors_keep_registered_realms_after_adoption(member: &str) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 0, 8);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let element = {
        let mut batch = DomBatch::new();
        let element = batch.create_element_ns(origin.document(), Namespace::html(), "details");
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
    expose_public_node_for_source_test(&mut host, &permit, element, "originDetails");
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
        set_details_member(scope, member)?;
        let value = evaluate(scope, "originDetails")?;
        assert_ne!(scope.callback_object(&value)?.associated_realm().identity(), permit.realm);
        let global = scope.global()?;
        let error = scope.import(&foreign_type_error)?;
        scope.set(&global, PropertyKey::String(&"foreignDetailsTypeError".encode_utf16().collect::<Vec<_>>()), &error)?;
        let result = evaluate(scope, r#"
            (() => {
                globalThis.savedOriginDetails = originDetails;
                globalThis.originDetailsPrototype = Object.getPrototypeOf(originDetails);
                if (originDetails instanceof HTMLDetailsElement ||
                    originDetailsPrototype.constructor.name !== 'HTMLDetailsElement') return 'registered creation realm';
                const descriptor = Object.getOwnPropertyDescriptor(originDetailsPrototype, detailsMember);
                for (const accessor of [descriptor.get, descriptor.set]) {
                    try { accessor.call({}, true); return 'foreign accessor receiver'; }
                    catch (error) { if (!(error instanceof foreignDetailsTypeError) || error instanceof TypeError)
                        return 'accessor function realm error'; }
                }
                if (detailsMember === 'name') {
                    const original = {};
                    try { descriptor.set.call(originDetails, {toString() { throw original; }}); return 'conversion throw'; }
                    catch (error) { if (error !== original) return 'borrowed original error'; }
                } else {
                    let coerced = false;
                    descriptor.set.call(originDetails, {[Symbol.toPrimitive]() { coerced = true; throw 0; }});
                    if (!originDetails.open || coerced) return 'borrowed Boolean no conversion';
                }
                descriptor.set.call(originDetails, detailsMember === 'name' ? 'foreign\ud800\0' : true);
                if (detailsMember === 'name' && (originDetails.name.length !== 9 ||
                    originDetails.name.charCodeAt(7) !== 0xd800 || originDetails.name.charCodeAt(8) !== 0))
                    return 'foreign literal units';
                return descriptor.get.call(originDetails) === (detailsMember === 'name' ? 'foreign\ud800\0' : true)
                    ? 'ok' : 'borrowed reflection';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok", "{member}");
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
            originDetails === savedOriginDetails && originDetails.ownerDocument === document &&
            Object.getPrototypeOf(originDetails) === originDetailsPrototype &&
            originDetails[detailsMember] === (detailsMember === 'name' ? 'foreign\ud800\0' : true)
        "#,
        )?;
        assert!(scope.boolean(&result)?, "{member}");
        Ok(())
    })
    .unwrap();
}

fn details_source_outputs_are_admitted_before_publication(member: &str) {
    // A new Attr and Attributes change owe two outputs. An open transition
    // additionally publishes the canonical DetailsToggleIntentions change.
    let exact = if member == "open" { 3 } else { 2 };
    for outputs in [exact - 1, exact] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session_with_details(&cancel, 0, 4);
        let element = insert_hosted_element(&mut session, target, "details", &cancel);
        let mut host =
            attach_public_nodes_with_source_output_limit(session, target, cancel, outputs);
        let permit = host.permit().unwrap();
        expose_public_node_for_source_test(&mut host, &permit, element, "limitedDetails");
        host.in_document(&permit, |scope| set_details_member(scope, member))
            .unwrap();
        let before = host.snapshot().unwrap();
        let result = host.in_document(&permit, |scope| {
            evaluate(
                scope,
                "limitedDetails[detailsMember] = detailsMember === 'name' ? null : true",
            )
            .map(drop)
        });
        let after = host.snapshot().unwrap();
        if outputs < exact {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("short Details {member} output: {result:?}");
            };
            assert!(matches!(
                failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
                Some(crate::internal::dom_source::DomSourceDriveError::Source(
                    crate::internal::dom_source::DomSourceRuntimeError::Dom(
                        DomError::SourceOutputLimit
                    )
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
            host.with_native(|dom, _, _| assert_eq!(dom.pending_details_toggle_intentions(), 0))
                .unwrap();
        } else {
            result.unwrap();
            let expected = if member == "name" { "null" } else { "" };
            assert_eq!(
                after.node(element).unwrap().attribute(&member.into()),
                Some(&expected.into())
            );
            host.with_native(|dom, _, _| {
                assert_eq!(
                    dom.pending_details_toggle_intentions(),
                    usize::from(member == "open")
                )
            })
            .unwrap();
        }
        host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
            .unwrap();
        host.stop().into_session().unwrap();
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 0, 4);
    let element = insert_hosted_element(&mut session, target, "details", &cancel);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "limitedDetails");
    host.in_document(&permit, |scope| set_details_member(scope, member))
        .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "limitedDetails[detailsMember] = detailsMember === 'name' ? 'work' : true",
        )
        .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("Details {member} Source work refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
        Some(crate::internal::dom_source::DomSourceDriveError::Source(
            crate::internal::dom_source::DomSourceRuntimeError::Dom(DomError::SourceWorkLimit {
                limit: 0
            })
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
    })
    .unwrap();
    host.stop().into_session().unwrap();
}

fn details_zero_intention_capacity_refuses_only_transitions() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 0, 0);
    let element = insert_hosted_element(&mut session, target, "details", &cancel);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "zeroDetails");
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "zeroDetails.name = 'closed'; zeroDetails.open = false; zeroDetails.name === 'closed' && !zeroDetails.open")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    let before = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "zeroDetails.open = true").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("zero intention capacity must refuse real transition: {result:?}");
    };
    assert!(matches!(
        failure.cause::<crate::internal::dom_source::DomSourceDriveError>(),
        Some(crate::internal::dom_source::DomSourceDriveError::Source(
            crate::internal::dom_source::DomSourceRuntimeError::Dom(DomError::ResourceLimit {
                resource: surgeist_dom::limits::DomResource::DetailsToggleIntentions,
                limit: 0
            })
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    assert!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
    })
    .unwrap();
    host.stop().into_session().unwrap();
}

fn details_short_task_custody_preserves_original_intentions() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 0, 4);
    let first = insert_hosted_element(&mut session, target, "details", &cancel);
    let second = insert_hosted_element(&mut session, target, "details", &cancel);
    let mut tasks = session.element_task_execution();
    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 1,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        })
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, first, "firstCustodyDetails");
    expose_public_node_for_source_test(&mut host, &permit, second, "secondCustodyDetails");
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            "firstCustodyDetails.open = true; secondCustodyDetails.open = true",
        )
        .map(drop)
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.with_native(|dom, _, _| {
        assert_eq!(dom.pending_details_toggle_intentions(), 2);
        assert!(matches!(
            tasks.ingest(dom, &cancel),
            Err(DomElementTaskRuntimeError::Admission)
        ));
        assert_eq!(dom.pending_details_toggle_intentions(), 2);
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    // The exact same original intentions remain available for admitted custody.
    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 2,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        assert_eq!(tasks.ingest(dom, &cancel).unwrap(), 2);
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    host.stop().into_session().unwrap();
}

fn details_properties_queue_coalesced_trusted_toggle_tasks() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_details(&cancel, 8, 16);
    let details = insert_hosted_element(&mut session, target, "details", &cancel);
    let mut tasks = session.element_task_execution();
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    dispatcher.bind(&mut session.dom(), &cancel).unwrap();
    let mut registry = Some(dispatcher);
    let mut host = attach_public_nodes_with_registry(
        session,
        target,
        cancel.clone(),
        200_000,
        2_000_000,
        &mut registry,
    );
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, details, "taskDetails");
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            globalThis.detailsEvents = [];
            globalThis.detailsEventChecks = true;
            taskDetails.addEventListener('toggle', function(event) {
                detailsEvents.push(event);
                detailsEventChecks = detailsEventChecks && this === taskDetails && event === globalThis.event &&
                    event instanceof ToggleEvent && event.isTrusted && !event.bubbles && !event.cancelable &&
                    event.target === taskDetails && event.currentTarget === taskDetails && event.source === null;
                event.preventDefault();
                detailsEventChecks = detailsEventChecks && !event.defaultPrevented;
                if (detailsEvents.length === 1) taskDetails.open = false;
            });
            taskDetails.name = 'notifications';
            taskDetails.open = true;
            taskDetails.open = true;
            taskDetails.open = false;
            taskDetails.open = true;
            detailsEvents.length === 0 && taskDetails.open
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    host.with_native(|dom, _, _| {
        // Same-value presence writes and name changes create no transition.
        assert_eq!(dom.pending_details_toggle_intentions(), 3);
        assert_eq!(tasks.ingest(dom, &cancel).unwrap(), 3);
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
    })
    .unwrap();
    let source = DomElementTaskSourceKind::DomManipulation;
    assert_eq!(tasks.pending_tasks(source).unwrap(), 1);
    let original = tasks.pending_task(source, 0).unwrap().unwrap();
    let limits = DomElementTaskRunLimits {
        dispatch: EVENT.dispatch,
        source: EVENT.source,
    };
    let progress = host
        .run_attached_element_task(&permit, source, limits)
        .unwrap();
    assert!(
        matches!(progress, DomElementTaskProgress::Complete { task, receipt, .. }
        if task.id() == original.id() && receipt.callbacks() == 1 && receipt.failures().is_empty())
    );
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            detailsEventChecks && detailsEvents.length === 1 && detailsEvents[0].oldState === 'closed' &&
            detailsEvents[0].newState === 'open' && detailsEvents[0].currentTarget === null &&
            detailsEvents[0].target === taskDetails && !taskDetails.open
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        assert_eq!(dom.pending_details_toggle_intentions(), 1);
        assert_eq!(tasks.ingest(dom, &cancel).unwrap(), 1);
    })
    .unwrap();
    assert_eq!(tasks.pending_tasks(source).unwrap(), 1);
    let next = tasks.pending_task(source, 0).unwrap().unwrap();
    assert_ne!(next.id(), original.id());
    let progress = host
        .run_attached_element_task(&permit, source, limits)
        .unwrap();
    assert!(
        matches!(progress, DomElementTaskProgress::Complete { task, receipt, .. }
        if task.id() == next.id() && receipt.callbacks() == 1 && receipt.failures().is_empty())
    );
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            detailsEventChecks && detailsEvents.length === 2 && detailsEvents[1] !== detailsEvents[0] &&
            detailsEvents[1].oldState === 'open' && detailsEvents[1].newState === 'closed' &&
            detailsEvents[1].currentTarget === null && !taskDetails.open
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(matches!(
        host.run_attached_element_task(&permit, source, limits)
            .unwrap(),
        DomElementTaskProgress::Empty
    ));
    host.with_native(|dom, _, _| {
        assert_eq!(dom.pending_details_toggle_intentions(), 0);
        assert!(dom.pending_source_edit().is_none());
    })
    .unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    host.stop().into_session().unwrap();
}

fn details_group_opening_delivers_both_original_reactions() {
    // DOM handle-attribute-changes enqueues the initiating custom reaction
    // before HTML's attribute steps remove the peer's open attribute.
    check_details_script(
        "group reactions",
        r#"
        (() => {
            const calls = [];
            class Grouped extends HTMLDetailsElement {
                static observedAttributes = ['open'];
                attributeChangedCallback(name, oldValue, newValue, namespace) {
                    calls.push([this, name, oldValue, newValue, namespace, this.open]);
                }
            }
            customElements.define('x-grouped-details', Grouped, {extends: 'details'});
            const container = document.createElement('div');
            const first = container.appendChild(new Grouped());
            const second = container.appendChild(document.createElement('details', {is: 'x-grouped-details'}));
            first.name = second.name = 'reactions';
            first.open = true;
            calls.length = 0;
            second.open = true;
            return calls.length === 2 && calls[0][0] === second && calls[0][1] === 'open' &&
                calls[0][2] === null && calls[0][3] === '' && calls[0][4] === null && calls[0][5] === true &&
                calls[1][0] === first && calls[1][1] === 'open' && calls[1][2] === '' &&
                calls[1][3] === null && calls[1][4] === null && calls[1][5] === false &&
                !first.open && second.open ? 'ok' : 'initiating and peer reactions from original Source';
        })()
    "#,
    );
}
