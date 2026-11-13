use super::*;
use crate::document_scripting::guest::tests::{attach_public_nodes, exhaust_guest_reaction_work};

pub(super) fn scoped_registries_keep_independent_native_associations() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 14_000, 160_000);
    let permit = host.permit().unwrap();
    let result = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                let stage = 'constructor';
                try {
                    const global = customElements;
                    const first = new CustomElementRegistry();
                    const second = new CustomElementRegistry();
                    if (!(first instanceof CustomElementRegistry) || first === second ||
                        first === global || document.customElementRegistry !== global)
                        return 'registry identities';
                    class First extends HTMLElement {}
                    class Second extends HTMLElement {}
                    stage = 'define';
                    first.define('scope-choice', First);
                    second.define('scope-choice', Second);
                    if (first.get('scope-choice') !== First || second.get('scope-choice') !== Second ||
                        global.get('scope-choice') !== undefined || first.getName(Second) !== null ||
                        first.getName(First) !== 'scope-choice')
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
                    stage = 'factory';
                    const left = make(first);
                    const right = make(second);
                    if (!(left instanceof First) || !(right instanceof Second) ||
                        left instanceof Second || left.customElementRegistry !== first ||
                        right.customElementRegistry !== second) return 'factory association';
                    stage = 'shadow clone';
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
                    stage = 'import';
                    const destination = new Document();
                    const imported = destination.importNode(unassigned, {customElementRegistry: second});
                    if (imported.ownerDocument !== destination || !(imported instanceof Second) ||
                        imported.customElementRegistry !== second || unassigned.customElementRegistry !== null)
                        return 'import fallback';
                    stage = 'initialize';
                    first.initialize(unassigned);
                    if (unassigned.customElementRegistry !== first || !(unassigned instanceof First))
                        return 'initialized association';
                    const detached = new Document();
                    second.initialize(detached);
                    if (detached.customElementRegistry !== second) return 'initialized Document';
                    if (detached.cloneNode().customElementRegistry !== second) return 'Document clone scope';
                    const detachedElement = detached.createElementNS('http://www.w3.org/1999/xhtml', 'scope-choice');
                    if (detachedElement.ownerDocument !== detached) return 'factory result document';
                    if (detachedElement.customElementRegistry !== second) return 'factory result registry';
                    if (!(detachedElement instanceof HTMLUnknownElement))
                        return 'cross-document constructor result: ' + detachedElement.constructor.name;
                    if (detached.adoptNode(host) !== host || root.customElementRegistry !== first ||
                        root.firstChild.customElementRegistry !== first || root.ownerDocument !== detached)
                        return 'adopted scope';
                    if (detached.importNode(root.firstChild, {customElementRegistry: second})
                        .customElementRegistry !== first) return 'import preserves source scope';
                    stage = 'late definition';
                    const late = new CustomElementRegistry();
                    const early = document.createElement('scope-late', {customElementRegistry: late});
                    const disconnected = document.createElement('scope-late', {customElementRegistry: late});
                    const connectedHost = document.createElement('div');
                    (document.documentElement ?? document).appendChild(connectedHost);
                    const connectedRoot = connectedHost.attachShadow({mode: 'closed', customElementRegistry: late});
                    connectedRoot.appendChild(early);
                    const clonedDocument = document.cloneNode(true);
                    if (clonedDocument.customElementRegistry !== null)
                        return 'Global registry clone substitution';
                    const order = [];
                    class Late extends HTMLElement {
                        constructor() { super(); order.push('constructor'); }
                        connectedCallback() { order.push('connected'); }
                    }
                    const waiting = late.whenDefined('scope-late');
                    if (waiting !== late.whenDefined('scope-late')) return 'scoped promise identity';
                    late.define('scope-late', Late);
                    if (!(early instanceof Late) || disconnected instanceof Late ||
                        order.join() !== 'constructor,connected') return 'connected scoped upgrades';
                    late.upgrade(disconnected);
                    if (!(disconnected instanceof Late) ||
                        order.join() !== 'constructor,connected,constructor') return 'explicit scoped upgrade';
                    globalThis.scopedForRefusal = first;
                    globalThis.nullForRefusal = make(null);
                    return 'ok';
                } catch (error) {
                    return stage + ': ' + error.name + ': ' + error.message;
                }
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    });
    if let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = &result {
        panic!(
            "scoped registry scenario refused: {:?}; source: {:?}",
            failure.cause::<DocumentGuestEventError>(),
            failure.cause::<crate::DomSourceDriveError>()
        );
    }
    result.unwrap();
    let before = host.snapshot().unwrap().stamp();
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "scopedForRefusal.initialize(nullForRefusal)").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("registry initialization must refuse before publication: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
