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
        2
    );
}
