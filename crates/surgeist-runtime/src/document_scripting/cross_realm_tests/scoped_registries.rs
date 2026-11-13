use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

pub(super) fn scoped_registries_keep_independent_native_associations() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 14_000, 160_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const global = customElements;
                let first;
                try { first = new CustomElementRegistry(); }
                catch (error) { return 'scoped constructor: ' + error.name + ': ' + error.message; }
                const second = new CustomElementRegistry();
                if (!(first instanceof CustomElementRegistry) || first === second ||
                    first === global || document.customElementRegistry !== global)
                    return 'registry identities';
                class First extends HTMLElement {}
                class Second extends HTMLElement {}
                first.define('scope-choice', First);
                second.define('scope-choice', Second);
                if (first.get('scope-choice') !== First || second.get('scope-choice') !== Second ||
                    global.get('scope-choice') !== undefined || first.getName(Second) !== null)
                    return 'independent definitions';
                try {
                    first.define('scope-built-in', class extends HTMLButtonElement {}, {extends: 'button'});
                    return 'scoped customized built-in';
                } catch (error) {
                    if (error.name !== 'NotSupportedError') return error.name;
                }
                const make = registry => document.createElement('scope-choice', {
                    customElementRegistry: registry,
                });
                const left = make(first);
                const right = make(second);
                if (!(left instanceof First) || !(right instanceof Second) ||
                    left instanceof Second || left.customElementRegistry !== first ||
                    right.customElementRegistry !== second) return 'factory association';
                const host = document.createElement('div');
                const root = host.attachShadow({mode: 'open', clonable: true, customElementRegistry: first});
                root.appendChild(left);
                const copy = host.cloneNode(true);
                if (root.customElementRegistry !== first || copy.shadowRoot === root ||
                    copy.shadowRoot.customElementRegistry !== first ||
                    copy.shadowRoot.firstChild.customElementRegistry !== first)
                    return 'shadow clone association';
                const unassigned = make(null);
                if (unassigned.customElementRegistry !== null) return 'explicit null';
                const destination = new Document();
                const imported = destination.importNode(unassigned, {customElementRegistry: second});
                if (imported.ownerDocument !== destination || !(imported instanceof Second) ||
                    imported.customElementRegistry !== second || unassigned.customElementRegistry !== null)
                    return 'import fallback';
                first.initialize(unassigned);
                if (unassigned.customElementRegistry !== first || !(unassigned instanceof First))
                    return 'initialized association';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
