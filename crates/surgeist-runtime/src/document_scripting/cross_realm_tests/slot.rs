use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, projected_node_id};

// Independent oracles: HTML877d §4.12.4 (HTMLSlotElement and assign),
// DOM3071 §4.2.2 (fresh find/flatten versus stored assigned nodes), and
// Web IDL's interface receiver, dictionary, union and sequence conversion rules.
// These run on the existing V8 entry thread, never individual Rust test threads.
pub(super) fn slot_family_observes_native_assignment_and_web_idl_boundaries() {
    slot_reflection_and_brands_validate_before_conversion();
    slot_dictionary_conversion_precedes_native_observation();
    manual_assignment_validates_all_inputs_and_preserves_source_order();
    flattened_assignment_recurses_only_through_shadow_tree_slots();
    slot_arrays_and_wrappers_follow_native_edits_and_lifetime();
    borrowed_slot_methods_use_their_function_realm();
}

fn check_slot_script(script: &str) {
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

fn slot_reflection_and_brands_validate_before_conversion() {
    check_slot_script(
        r#"
        (() => {
            const slot = document.createElement('slot');
            const prototype = HTMLSlotElement.prototype;
            const name = Object.getOwnPropertyDescriptor(prototype, 'name');
            if (Object.getPrototypeOf(prototype) !== HTMLElement.prototype ||
                Object.getPrototypeOf(HTMLSlotElement) !== HTMLElement ||
                Object.getPrototypeOf(slot) !== prototype) return 'slot inheritance';
            let constructed = false;
            try { new HTMLSlotElement(); constructed = true; }
            catch (error) { if (!(error instanceof TypeError)) return 'constructor error'; }
            if (constructed) return 'unregistered constructor';
            const foreign = document.createElementNS('urn:foreign', 'slot');
            const upper = document.createElementNS('http://www.w3.org/1999/xhtml', 'SLOT');
            const prefixed = document.createElementNS('http://www.w3.org/1999/xhtml', 'p:slot');
            if (foreign instanceof HTMLSlotElement || upper instanceof HTMLSlotElement ||
                !(prefixed instanceof HTMLSlotElement)) return 'actual namespace and local name';
            for (const receiver of [{}, null, foreign, upper, document.createElement('div'),
                Object.create(slot), new Proxy(slot, {})]) {
                let converted = false;
                try { name.get.call(receiver); return 'name getter brand'; }
                catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                try { name.set.call(receiver, {toString() { converted = true; return 'bad'; }});
                    return 'name setter brand'; }
                catch (error) { if (!(error instanceof TypeError) || converted) return 'name brand before coercion'; }
                for (const method of ['assignedNodes', 'assignedElements']) {
                    try { prototype[method].call(receiver, {get flatten() { converted = true; return true; }});
                        return 'method brand ' + method; }
                    catch (error) { if (!(error instanceof TypeError) || converted) return 'method brand before dictionary'; }
                }
                try { prototype.assign.call(receiver, slot); return 'assign brand'; }
                catch (error) { if (!(error instanceof TypeError)) return 'assign brand error'; }
            }
            for (const [owner, real] of [[Element.prototype, slot], [Text.prototype, document.createTextNode('t')]]) {
                const descriptor = Object.getOwnPropertyDescriptor(owner, 'assignedSlot');
                if (!descriptor?.get || descriptor.set !== undefined || !descriptor.enumerable ||
                    !descriptor.configurable || descriptor.get.length !== 0 || descriptor.get.call(real) !== null)
                    return 'Slottable descriptor';
                for (const receiver of [{}, Object.create(real), new Proxy(real, {}), document.createComment('c')]) {
                    try { descriptor.get.call(receiver); return 'Slottable brand'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'Slottable brand error'; }
                }
            }
            slot.setAttributeNS('urn:foreign', 'name', 'foreign');
            if (slot.name !== '') return 'foreign name ignored';
            slot.name = {toString() { slot.setAttributeNS(null, 'name', 'inner'); return 'exact\ud800\0'; }};
            if (slot.name !== 'exact\ud800\0' || slot.getAttributeNS(null, 'name') !== 'exact\ud800\0' ||
                slot.getAttributeNS('urn:foreign', 'name') !== 'foreign') return 'exact DOMString reflection';
            const original = {};
            try { slot.name = {toString() { throw original; }}; return 'name conversion must throw'; }
            catch (error) { if (error !== original || slot.name !== 'exact\ud800\0') return 'name original throw'; }
            try { slot.name = Symbol(); return 'Symbol name'; }
            catch (error) { if (!(error instanceof TypeError) || slot.name !== 'exact\ud800\0') return 'Symbol name state'; }
            slot.name = null;
            if (slot.name !== 'null') return 'null DOMString';
            slot.name = undefined;
            if (slot.name !== 'undefined') return 'undefined DOMString';
            slot.removeAttributeNS(null, 'name');
            return slot.name === '' ? 'ok' : 'removed name default';
        })()
    "#,
    );
}

fn slot_dictionary_conversion_precedes_native_observation() {
    check_slot_script(
        r#"
        (() => {
            const host = document.createElement('div');
            const root = host.attachShadow({mode: 'open'});
            const slot = root.appendChild(document.createElement('slot'));
            const fallback = slot.appendChild(document.createElement('em'));
            for (const method of ['assignedNodes', 'assignedElements']) {
                for (const options of [undefined, null, {}, {flatten: undefined}, {flatten: false}]) {
                    if (slot[method](options).length !== 0) return 'dictionary default ' + method;
                }
                for (const options of [true, 1, 'x', Symbol()]) {
                    try { slot[method](options); return 'primitive dictionary accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'dictionary error'; }
                }
                let gets = 0;
                const options = Object.create({get flatten() { gets++; return true; }});
                const result = slot[method](options);
                if (gets !== 1 || result.length !== 1 || result[0] !== fallback)
                    return 'inherited dictionary getter ' + method;
                let coerced = false;
                const truthy = {valueOf() { coerced = true; throw 0; }, toString() { coerced = true; throw 0; }};
                if (slot[method]({flatten: truthy})[0] !== fallback || coerced)
                    return 'boolean conversion must not coerce objects';
                const original = {};
                try { slot[method]({get flatten() { throw original; }}); return 'dictionary throw'; }
                catch (error) { if (error !== original) return 'dictionary throw identity'; }
                const child = document.createElement('span');
                let nested = false;
                const fresh = slot[method]({get flatten() {
                    nested = slot.assignedNodes({flatten: true})[0] === fallback;
                    host.appendChild(child);
                    return true;
                }});
                if (!nested || fresh.length !== 1 || fresh[0] !== child || child.assignedSlot !== slot)
                    return 'observe only after getter reentry ' + method;
                host.removeChild(child);
                // A mutation performed by a throwing getter survives; native slot work must not replace its exception.
                const sideEffect = document.createTextNode('kept');
                try { slot[method]({get flatten() { host.appendChild(sideEffect); throw original; }});
                    return 'mutating dictionary throw'; }
                catch (error) { if (error !== original) return 'mutating getter throw identity'; }
                if (slot.assignedNodes()[0] !== sideEffect || sideEffect.assignedSlot !== slot)
                    return 'getter side effect survives';
                host.removeChild(sideEffect);
            }
            return 'ok';
        })()
    "#,
    );
}

fn manual_assignment_validates_all_inputs_and_preserves_source_order() {
    check_slot_script(
        r#"
        (() => {
            const host = document.createElement('div');
            const root = host.attachShadow({mode: 'open', slotAssignment: 'manual'});
            const first = root.appendChild(document.createElement('slot'));
            const second = root.appendChild(document.createElement('slot'));
            const a = host.appendChild(document.createElement('span'));
            const b = host.appendChild(document.createTextNode('b'));
            const outside = document.createElement('em');
            if (first.assignedNodes().length || a.assignedSlot !== null) return 'manual initially empty';
            if (first.assign(b, a, b, a, outside) !== undefined) return 'assign result';
            const ordered = first.assignedNodes();
            if (ordered.length !== 2 || ordered[0] !== b || ordered[1] !== a ||
                first.assignedElements().length !== 1 || first.assignedElements()[0] !== a ||
                a.assignedSlot !== first || b.assignedSlot !== first || outside.assignedSlot !== null)
                return 'ordered deduplication and host child filter';
            let coerced = false;
            const stringLike = {toString() { coerced = true; return 'text'; }};
            for (const invalid of [undefined, null, 'text', 1, true, Symbol(), stringLike,
                document, document.createDocumentFragment(), document.createComment('c'),
                Object.create(a), new Proxy(a, {})]) {
                try { second.assign(a, invalid); return 'invalid variadic accepted'; }
                catch (error) { if (!(error instanceof TypeError)) return 'variadic error'; }
                const current = first.assignedNodes();
                if (coerced || current.length !== 2 || current[0] !== b || current[1] !== a ||
                    a.assignedSlot !== first || second.assignedNodes().length)
                    return 'all variadic inputs validated before transfer';
            }
            second.assign(a, b, a);
            if (first.assignedNodes().length || second.assignedNodes().length !== 2 ||
                second.assignedNodes()[0] !== a || second.assignedNodes()[1] !== b ||
                a.assignedSlot !== second || ordered[0] !== b || ordered[1] !== a)
                return 'same root manual transfer and snapshot array';
            second.assign();
            if (second.assignedNodes().length || a.assignedSlot !== null || b.assignedSlot !== null)
                return 'zero variadic clears manual intent';
            first.assign(a);
            const fallback = first.appendChild(document.createTextNode('fallback'));
            const otherHost = document.createElement('div');
            const otherRoot = otherHost.attachShadow({mode: 'open', slotAssignment: 'manual'});
            const otherSlot = otherRoot.appendChild(document.createElement('slot'));
            otherSlot.assign(a);
            if (first.assignedNodes().length !== 1 || first.assignedNodes()[0] !== a ||
                first.assignedElements()[0] !== a || first.assignedNodes({flatten: true})[0] !== fallback ||
                first.assignedElements({flatten: true}).length || otherSlot.assignedNodes().length ||
                a.assignedSlot !== null || a.parentNode !== host)
                return 'cross root stored history versus fresh find';
            host.removeChild(a);
            otherHost.appendChild(a);
            // DOM insertion assigns a slot directly only for a named host;
            // manual intent affects fresh find without refreshing this stored list.
            const afterInsertion = otherSlot.assignedNodes({flatten: true});
            if (first.assignedNodes().length || otherSlot.assignedNodes().length ||
                a.assignedSlot !== otherSlot || afterInsertion.length !== 1 || afterInsertion[0] !== a ||
                a.parentNode !== otherHost) return 'manual insertion stored history versus fresh find';
            otherSlot.assign(a);
            const afterAssign = otherSlot.assignedNodes();
            if (afterAssign.length !== 1 || afterAssign[0] !== a || a.assignedSlot !== otherSlot ||
                otherSlot.assignedNodes({flatten: true})[0] !== a)
                return 'explicit assign refreshes target root stored list';
            const namedHost = document.createElement('div');
            const namedRoot = namedHost.attachShadow({mode: 'open'});
            const named = namedRoot.appendChild(document.createElement('slot'));
            const namedChild = namedHost.appendChild(document.createTextNode('named'));
            named.assign(outside);
            return named.assignedNodes()[0] === namedChild && namedChild.assignedSlot === named &&
                outside.assignedSlot === null ? 'ok' : 'named roots ignore manual intent';
        })()
    "#,
    );
}

fn flattened_assignment_recurses_only_through_shadow_tree_slots() {
    check_slot_script(
        r#"
        (() => {
            const host = document.createElement('div');
            const root = host.attachShadow({mode: 'open'});
            const outer = root.appendChild(document.createElement('slot'));
            const lead = outer.appendChild(document.createTextNode('lead'));
            outer.appendChild(document.createComment('excluded'));
            const inner = outer.appendChild(document.createElement('slot'));
            inner.name = 'inner';
            const innerText = inner.appendChild(document.createTextNode('inner'));
            const innerElement = inner.appendChild(document.createElement('em'));
            inner.appendChild(document.createComment('excluded too'));
            const tail = outer.appendChild(document.createElement('strong'));
            const nodes = outer.assignedNodes({flatten: true});
            const elements = outer.assignedElements({flatten: true});
            if (outer.assignedNodes().length || inner.assignedNodes().length || nodes.length !== 4 ||
                nodes[0] !== lead || nodes[1] !== innerText || nodes[2] !== innerElement || nodes[3] !== tail ||
                elements.length !== 2 || elements[0] !== innerElement || elements[1] !== tail)
                return 'nested slottable fallback and element filtering';
            const named = host.appendChild(document.createElement('span'));
            named.setAttribute('slot', 'inner');
            const fresh = outer.assignedNodes({flatten: true});
            if (fresh.length !== 3 || fresh[0] !== lead || fresh[1] !== named || fresh[2] !== tail ||
                inner.assignedNodes()[0] !== named || named.assignedSlot !== inner || nodes.length !== 4)
                return 'nested fresh assigned content replaces fallback';
            host.removeChild(named);
            const lightSlot = host.appendChild(document.createElement('slot'));
            lightSlot.appendChild(document.createTextNode('do not flatten ordinary-tree slot'));
            if (outer.assignedNodes({flatten: true}).length !== 1 ||
                outer.assignedNodes({flatten: true})[0] !== lightSlot || lightSlot.assignedSlot !== outer)
                return 'ordinary tree assigned slot is retained';
            const detached = document.createElement('slot');
            detached.appendChild(document.createTextNode('outside shadow tree'));
            if (detached.assignedNodes({flatten: true}).length) return 'detached flattened result';
            for (const slotAssignment of ['named', 'manual']) {
                const closedHost = document.createElement('div');
                const closed = closedHost.attachShadow({mode: 'closed', slotAssignment});
                const closedSlot = closed.appendChild(document.createElement('slot'));
                const element = closedHost.appendChild(document.createElement('span'));
                const text = closedHost.appendChild(document.createTextNode('text'));
                if (slotAssignment === 'manual') closedSlot.assign(text, element);
                const assigned = closedSlot.assignedNodes();
                if (closedHost.shadowRoot !== null || element.assignedSlot !== null || text.assignedSlot !== null ||
                    assigned.length !== 2 || assigned[slotAssignment === 'manual' ? 1 : 0] !== element ||
                    assigned[slotAssignment === 'manual' ? 0 : 1] !== text ||
                    closedSlot.assignedElements({flatten: true})[0] !== element)
                    return 'closed root read access with hidden Slottable getter';
            }
            return 'ok';
        })()
    "#,
    );
}

fn slot_arrays_and_wrappers_follow_native_edits_and_lifetime() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let (slot, element) = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                globalThis.nativeSlotHost = document.createElement('div');
                const root = nativeSlotHost.attachShadow({mode: 'open'});
                globalThis.nativeSlot = root.appendChild(document.createElement('slot'));
                globalThis.nativeSlottable = nativeSlotHost.appendChild(document.createElement('span'));
                globalThis.nativeSlotText = nativeSlotHost.appendChild(document.createTextNode('text'));
                globalThis.savedSlotArray = nativeSlot.assignedNodes();
                const fresh = nativeSlot.assignedNodes();
                if (!Array.isArray(fresh) || Object.getPrototypeOf(fresh) !== Array.prototype ||
                    fresh === savedSlotArray || fresh.length !== 2 || fresh[0] !== nativeSlottable ||
                    fresh[1] !== nativeSlotText || Object.keys(fresh).join() !== '0,1' ||
                    !Object.isExtensible(fresh) || Object.isFrozen(fresh)) return 'fresh ordinary dense array';
                const index = Object.getOwnPropertyDescriptor(fresh, '0');
                if (!index.writable || !index.enumerable || !index.configurable) return 'ordinary array index';
                fresh[0] = 'changed';
                fresh.length = 0;
                if (nativeSlot.assignedNodes()[0] !== nativeSlottable || savedSlotArray.length !== 2)
                    return 'returned array mutation must not mutate native list';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let slot = evaluate(scope, "nativeSlot")?;
        let element = evaluate(scope, "nativeSlottable")?;
        Ok((projected_node_id(scope, &slot)?, projected_node_id(scope, &element)?))
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute(slot, "name", "native");
        batch.set_attribute(element, "slot", "native");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            nativeSlot.name === 'native' && nativeSlot.assignedNodes().length === 1 &&
            nativeSlot.assignedNodes()[0] === nativeSlottable && nativeSlottable.assignedSlot === nativeSlot &&
            nativeSlotText.assignedSlot === null && savedSlotArray.length === 2 &&
            savedSlotArray[0] === nativeSlottable && savedSlotArray[1] === nativeSlotText
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "nativeSlot.assignedNodes().length === 0 && nativeSlottable.assignedSlot === null && savedSlotArray[0] === nativeSlottable")?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.detach(slot);
        batch.destroy(slot);
        batch.destroy(element);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    for expression in [
        "nativeSlot.name",
        "nativeSlot.assignedNodes()",
        "nativeSlot.assignedElements({flatten: true})",
        "nativeSlot.assign()",
        "nativeSlottable.assignedSlot",
    ] {
        let result = host.in_document(&permit, |scope| evaluate(scope, expression).map(drop));
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
            panic!("retired native slot/slottable must refuse {expression}: {result:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
    }
}

fn borrowed_slot_methods_use_their_function_realm() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let exports = host.in_document(&permit, |scope| {
        evaluate(scope, r#"
            globalThis.realmSlotHost = document.createElement('div');
            globalThis.realmSlot = realmSlotHost.attachShadow({mode: 'open', slotAssignment: 'manual'})
                .appendChild(document.createElement('slot'));
            globalThis.realmSlotElement = realmSlotHost.appendChild(document.createElement('span'));
            globalThis.realmSlotText = realmSlotHost.appendChild(document.createTextNode('text'));
            realmSlot.assign(realmSlotText, realmSlotElement);
        "#)?;
        [
            ("windowSlot", "realmSlot"),
            ("windowElement", "realmSlotElement"),
            ("windowText", "realmSlotText"),
            ("windowArrayPrototype", "Array.prototype"),
            ("windowTypeError", "TypeError"),
            ("borrowedNodes", "HTMLSlotElement.prototype.assignedNodes"),
            ("borrowedElements", "HTMLSlotElement.prototype.assignedElements"),
            ("borrowedAssign", "HTMLSlotElement.prototype.assign"),
            ("borrowedElementSlot", "Object.getOwnPropertyDescriptor(Element.prototype, 'assignedSlot').get"),
            ("borrowedTextSlot", "Object.getOwnPropertyDescriptor(Text.prototype, 'assignedSlot').get"),
        ].into_iter().map(|(name, expression)| {
            let value = evaluate(scope, expression)?;
            Ok((name, scope.retain(&value)))
        }).collect::<Result<Vec<_>, Error>>()
    }).unwrap();
    let engine = host.engine.as_mut().unwrap();
    let mut execution = engine.process.execution();
    let caller = execution.create_realm().unwrap();
    execution.in_realm(&caller, |scope| {
        assert_ne!(scope.realm_authority()?.identity(), permit.realm);
        let global = scope.global()?;
        for (name, value) in &exports {
            let imported = scope.import(value)?;
            scope.set(&global, PropertyKey::String(&name.encode_utf16().collect::<Vec<_>>()), &imported)?;
        }
        let result = evaluate(scope, r#"
            (() => {
                const nodes = borrowedNodes.call(windowSlot);
                const elements = borrowedElements.call(windowSlot, {flatten: true});
                if (!Array.isArray(nodes) || nodes instanceof Array || elements instanceof Array ||
                    Object.getPrototypeOf(nodes) !== windowArrayPrototype ||
                    Object.getPrototypeOf(elements) !== windowArrayPrototype ||
                    nodes.length !== 2 || nodes[0] !== windowText || nodes[1] !== windowElement ||
                    elements.length !== 1 || elements[0] !== windowElement ||
                    borrowedElementSlot.call(windowElement) !== windowSlot ||
                    borrowedTextSlot.call(windowText) !== windowSlot) return 'function realm array and wrapper identity';
                for (const method of [borrowedNodes, borrowedElements, borrowedAssign, borrowedElementSlot, borrowedTextSlot]) {
                    try { method.call({}); return 'borrowed receiver accepted'; }
                    catch (error) { if (!(error instanceof windowTypeError) || error instanceof TypeError)
                        return 'function realm TypeError'; }
                }
                const original = {};
                try { borrowedNodes.call(windowSlot, {get flatten() { throw original; }}); return 'cross realm getter throw'; }
                catch (error) { if (error !== original) return 'cross realm throw identity'; }
                borrowedAssign.call(windowSlot, windowElement, windowText);
                return borrowedNodes.call(windowSlot)[0] === windowElement && nodes[0] === windowText
                    ? 'ok' : 'cross realm assign and retained array';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn slot_members_read_the_canonical_named_assignment() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const slot = document.createElement('slot');
                if (!(slot instanceof HTMLSlotElement)) return 'slot interface';
                const members = ['name', 'assignedNodes', 'assignedElements', 'assign'];
                if (Object.keys(HTMLSlotElement.prototype).join() !== members.join())
                    return 'slot declaration order';
                const name = Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype, 'name');
                if (!name?.get || !name?.set || !name.enumerable || !name.configurable ||
                    name.get.length !== 0 || name.set.length !== 1 || slot.name !== '')
                    return 'slot name descriptor';
                for (const member of members.slice(1)) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype, member);
                    if (typeof descriptor?.value !== 'function' || !descriptor.enumerable ||
                        !descriptor.configurable || !descriptor.writable || descriptor.value.length !== 0)
                        return 'slot method descriptor ' + member;
                }
                const host = document.createElement('div');
                const root = host.attachShadow({mode: 'open'});
                root.appendChild(slot);
                const element = host.appendChild(document.createElement('span'));
                const text = host.appendChild(document.createTextNode('text'));
                host.appendChild(document.createComment('excluded'));
                const first = slot.assignedNodes();
                if (!Array.isArray(first) || Object.getPrototypeOf(first) !== Array.prototype ||
                    first.length !== 2 || first[0] !== element || first[1] !== text ||
                    slot.assignedNodes() === first || slot.assignedElements().length !== 1 ||
                    slot.assignedElements()[0] !== element || element.assignedSlot !== slot ||
                    text.assignedSlot !== slot) return 'canonical named assignment';
                slot.name = 'chosen';
                if (slot.getAttribute('name') !== 'chosen' || slot.assignedNodes().length !== 0 ||
                    element.assignedSlot !== null || text.assignedSlot !== null || first.length !== 2)
                    return 'live name reflection';
                element.setAttribute('slot', 'chosen');
                const assigned = slot.assignedNodes({flatten: false});
                const flattened = slot.assignedNodes({flatten: true});
                return assigned.length === 1 && assigned[0] === element &&
                    flattened.length === 1 && flattened[0] === element &&
                    element.assignedSlot === slot && text.assignedSlot === null ? 'ok' : 'fresh assignment';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
