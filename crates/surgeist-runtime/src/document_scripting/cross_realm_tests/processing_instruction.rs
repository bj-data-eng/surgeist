use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};

pub(super) fn native_processing_instruction_uses_its_canonical_wrapper() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let document = target.document();
    let mut batch = DomBatch::new();
    let instruction = batch.create_processing_instruction(
        document,
        "example",
        surgeist_html::values::DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
    );
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let instruction = commit.created(&instruction).unwrap().unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 5_000, 60_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, instruction, "instruction");
    expose_public_node_for_source_test(&mut host, &permit, instruction, "repeatedInstruction");
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const member = Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target');
                let rejected = false;
                try { member.get.call(document.createTextNode('')); }
                catch (error) { rejected = error instanceof TypeError; }
                return rejected && instruction === repeatedInstruction &&
                    Object.getPrototypeOf(instruction) === ProcessingInstruction.prototype &&
                    Object.getPrototypeOf(ProcessingInstruction.prototype) === CharacterData.prototype &&
                    instruction instanceof Node && instruction.nodeType === Node.PROCESSING_INSTRUCTION_NODE &&
                    instruction.nodeName === 'example' && instruction.target === 'example' &&
                    instruction.ownerDocument === document && instruction.parentNode === null &&
                    instruction.length === 3 && instruction.data === '\ud800\0\udc00' &&
                    instruction.substringData(1, 2) === '\0\udc00' &&
                    instruction.nodeValue === instruction.data && instruction.textContent === instruction.data &&
                    instruction.replaceData(1, 1, 'x') === undefined &&
                    instruction.data === '\ud800x\udc00' && instruction.target === 'example';
            })()
        "#)?;
        assert!(scope.boolean(&result)?);
        Ok(())
    }).unwrap();
    let snapshot = host.snapshot().unwrap();
    let surgeist_dom::DomNodeKind::ProcessingInstruction(value) =
        snapshot.node(instruction).unwrap().kind()
    else {
        panic!("native instruction");
    };
    assert_eq!(value.data().code_units(), &[0xd800, 120, 0xdc00]);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(instruction);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let retired = host.in_document(&permit, |scope| {
        evaluate(scope, "instruction.target").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = retired else {
        panic!("expected retired instruction refusal: {retired:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}

pub(super) fn authored_processing_instructions_use_native_creation() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const method = Document.prototype.createProcessingInstruction;
                if (typeof method !== 'function') return 'missing Document factory';
                const fails = (action, name) => {
                    try { action(); } catch (error) { return error.name === name; }
                    return false;
                };
                let order = '';
                const name = { toString() { order += 't'; return 'target'; } };
                const data = { toString() { order += 'd'; return '\ud800\0\udc00'; } };
                if (!fails(() => method.call({}, name, data), 'TypeError') || order ||
                    !fails(() => method.call(document, name), 'TypeError') || order ||
                    !fails(() => new ProcessingInstruction(), 'TypeError')) return 'receiver or arity';
                if (!fails(() => method.call(document, 'bad target', ''), 'InvalidCharacterError') ||
                    !fails(() => new ProcessingInstruction('good', '?>'), 'InvalidCharacterError'))
                    return 'native validation';
                const separate = new Document();
                const node = method.call(separate, name, data);
                if (order !== 'td' || node.target !== 'target' || node.data !== '\ud800\0\udc00' ||
                    node.ownerDocument !== separate || node.parentNode !== null ||
                    Object.getPrototypeOf(node) !== ProcessingInstruction.prototype)
                    return 'Document factory result';
                class Derived extends ProcessingInstruction {}
                const created = new Derived('derived');
                if (!(created instanceof Derived) || created.target !== 'derived' ||
                    created.data !== '' || created.ownerDocument !== document || created.parentNode !== null ||
                    ProcessingInstruction.length !== 1 || method.length !== 2) return 'constructor result';
                const exact = {};
                const fail = new Proxy(function () {}, {
                    get(_, key) { if (key === 'prototype') throw exact; }
                });
                try { Reflect.construct(ProcessingInstruction, ['valid'], fail); return 'prototype throw'; }
                catch (error) { if (error !== exact) return 'wrong prototype exception'; }
                let calls = '';
                function Fallback() {}
                Fallback.prototype = 0;
                const reenter = new Proxy(Fallback, {
                    get(object, key) {
                        if (key === 'prototype') {
                            calls += 'p';
                            document.createTextNode('nested');
                        }
                        return Reflect.get(object, key);
                    }
                });
                const fallback = Reflect.construct(ProcessingInstruction, [
                    {toString() { calls += 't'; return 'fallback'; }},
                    {toString() { calls += 'd'; return ''; }}
                ], reenter);
                if (calls !== 'tdp' || Object.getPrototypeOf(fallback) !== ProcessingInstruction.prototype ||
                    fallback.ownerDocument !== document || fallback.target !== 'fallback')
                    return 'prototype fallback or reentry';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let after = host.snapshot().unwrap();
    assert_eq!(
        after
            .nodes()
            .filter(|id| before.node(*id).is_err()
                && matches!(
                    after.node(*id).unwrap().kind(),
                    surgeist_dom::DomNodeKind::ProcessingInstruction(_)
                ))
            .count(),
        3
    );
}

pub(super) fn authored_instruction_wrapper_refusal_is_atomic() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    let before = host.snapshot().unwrap();
    crate::document_scripting::guest::tests::refuse_next_source_node_validation(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "document.createProcessingInstruction('target', '')").map(drop)
    });
    assert!(
        matches!(
            result,
            Err(DocumentScriptError::Script(Error::BackendUnavailable))
        ),
        "{result:?}"
    );
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.nodes().count(), before.nodes().count());
}

pub(super) fn pseudo_attributes_and_data_synchronize() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const pi = new ProcessingInstruction('target', "b='two' a='&amp;'");
                if (typeof pi.hasAttributes !== 'function') return 'missing PI pseudo-attributes';
                if (!pi.hasAttributes() || pi.getAttributeNames().join(',') !== 'b,a' ||
                    pi.getAttribute('a') !== '&' || pi.getAttribute('missing') !== null ||
                    !pi.hasAttribute('b') || pi.hasAttribute('B')) return 'initial ordered map';
                const marker = {};
                const abrupt = {toString() { throw marker; }};
                try { pi.getAttribute(abrupt); return 'missing abrupt'; }
                catch (error) { if (error !== marker) return 'wrong abrupt'; }
                let coerced = false;
                try {
                    pi.setAttribute.call({}, {toString() { coerced = true; }}, 'value');
                    return 'wrong brand';
                } catch (error) { if (!(error instanceof TypeError) || coerced) return 'brand order'; }
                try { pi.setAttribute('bad name', ''); return 'missing invalid name'; }
                catch (error) { if (error.name !== 'InvalidCharacterError') return 'invalid name error'; }
                if (pi.setAttribute('b', '&<>"\ud800') !== undefined ||
                    pi.data !== 'b="&amp;&lt;&gt;&quot;\ud800" a="&amp;"' ||
                    pi.getAttribute('b') !== '&<>"\ud800') return 'native map serialization';
                pi.removeAttribute('a');
                if (pi.toggleAttribute('c') !== true || pi.toggleAttribute('c', true) !== true ||
                    pi.toggleAttribute('c', false) !== false || pi.hasAttribute('c') ||
                    pi.getAttributeNames().join(',') !== 'b') return 'toggle presence';
                pi.data = "b='uncanonical'";
                if (!pi.toggleAttribute('b', true) || pi.toggleAttribute('absent', false) ||
                    pi.data !== "b='uncanonical'") return 'toggle no-op leaves raw data';
                pi.removeAttribute('invalid name');
                if (pi.data !== 'b="uncanonical"') return 'missing remove serializes';
                if (pi.toggleAttribute('b', undefined) !== false || pi.data !== '')
                    return 'undefined force is omitted';
                const name = {toString() { pi.data = "q='side effect'"; return 'q'; }};
                const value = {toString() {
                    if (pi.getAttribute('q') !== 'side effect') throw marker;
                    pi.setAttribute('r', 'nested');
                    return 'outer';
                }};
                pi.setAttribute(name, value);
                if (pi.data !== 'q="outer" r="nested"') return 'reentry before native mutation';
                pi.data = "z='last' k='first'";
                if (pi.getAttributeNames().join(',') !== 'z,k' ||
                    pi.getAttribute('z') !== 'last' || pi.hasAttribute('b')) return 'replace data';
                pi.data = 'malformed';
                if (pi.hasAttributes() || pi.getAttributeNames().length !== 0) return 'invalid data';
                pi.setAttribute('9name', 'value');
                if (pi.getAttribute('9name') !== 'value' || pi.data !== '9name="value"')
                    return 'authored map preserved independently of XML parsing';
                pi.data = pi.data;
                if (pi.hasAttributes()) return 'ordinary replacement reparses exact data';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
