use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, refuse_next_source_node_validation,
};

pub(super) fn import_node_uses_destination_and_standard_options() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 10_000, 100_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const method = Document.prototype.importNode;
                if (typeof method !== 'function') return 'missing importNode';
                const destination = new Document();
                const source = document.createElement('div');
                const original = source.appendChild(new Text('A\ud800\0'));
                let order = '';
                const options = {
                    get customElementRegistry() { order += 'r'; return undefined; },
                    get selfOnly() { order += 's'; return false; },
                };
                const absent = destination.importNode(source);
                const shallow = destination.importNode(source, {selfOnly: true});
                const deep = destination.importNode(source, true);
                const dictionary = destination.importNode(source, options);
                if (method.length !== 1 || order !== 'rs' ||
                    absent.firstChild !== null || shallow.firstChild !== null ||
                    [deep, dictionary].some(copy => copy === source ||
                        copy.parentNode !== null || copy.ownerDocument !== destination ||
                        copy.firstChild === original || copy.firstChild.ownerDocument !== destination ||
                        copy.firstChild.data !== 'A\ud800\0')) return 'option branches';
                if (destination.importNode(source, null).firstChild?.data !== 'A\ud800\0')
                    return 'null dictionary branch';
                try { method.call({}, source, options); return 'wrong receiver'; }
                catch (error) { if (!(error instanceof TypeError) || order !== 'rs') return 'receiver order'; }
                try { method.call(destination, {}, options); return 'wrong argument'; }
                catch (error) { if (!(error instanceof TypeError) || order !== 'rs') return 'argument order'; }
                const marker = {};
                try { destination.importNode(document, {get selfOnly() { throw marker; }}); return 'missing abrupt'; }
                catch (error) { if (error !== marker) return 'conversion precedes native rejection'; }
                try { destination.importNode(document); return 'imported Document'; }
                catch (error) { if (error.name !== 'NotSupportedError') return error.name; }
                try { destination.importNode(source, {customElementRegistry: null}); return 'nullable registry'; }
                catch (error) { if (!(error instanceof TypeError)) return 'registry brand'; }
                const root = source.attachShadow({mode: 'open'});
                try { destination.importNode(root); return 'imported ShadowRoot'; }
                catch (error) { if (error.name !== 'NotSupportedError') return error.name; }
                const mutable = source.appendChild(new Text('before'));
                const reentered = destination.importNode(source, {
                    get selfOnly() { mutable.data = 'after'; return false; }
                });
                if (reentered.lastChild.data !== 'after') return 'live source after coercion';
                let constructed = 0;
                class Imported extends HTMLElement {
                    constructor() { super(); ++constructed; }
                }
                const candidate = document.createElement('import-element', {customElementRegistry: null});
                customElements.define('import-element', Imported);
                const upgraded = document.importNode(candidate, {customElementRegistry: customElements});
                if (!(upgraded instanceof Imported) || constructed !== 1 ||
                    candidate instanceof Imported || upgraded.ownerDocument !== document)
                    return 'destination registry fallback';
                try { destination.importNode(source, {customElementRegistry: customElements}); return 'foreign Global'; }
                catch (error) { if (error.name !== 'NotSupportedError') return error.name; }
                globalThis.refusedImportSource = source;
                return source.firstChild === original && original.ownerDocument === document ? 'ok' : 'mutated source';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let before = host.snapshot().unwrap();
    refuse_next_source_node_validation(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "document.importNode(refusedImportSource, true)").map(drop)
    });
    assert!(
        matches!(
            result,
            Err(DocumentScriptError::Script(Error::BackendUnavailable))
        ),
        "{result:?}"
    );
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
}
