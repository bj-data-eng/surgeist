use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, guest_backing_used,
};

pub(super) fn document_referrer_reads_the_canonical_known_empty_value() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const doc = document.implementation.createHTMLDocument('original');
                if (!(document instanceof Document) || doc.title !== 'original') return 'native fixture';
                const {get, set, enumerable, configurable} =
                    Object.getOwnPropertyDescriptor(Document.prototype, 'referrer') ?? {};
                if (typeof get !== 'function') return 'document missing referrer accessor';
                if (set !== undefined || !enumerable || !configurable || get.name !== 'get referrer' ||
                    get.length !== 0) return 'referrer descriptor';
                return get.call(document) === '' && get.call(doc) === '' ? 'ok' : 'canonical empty referrer';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn document_referrer_preserves_creation_defaults_and_refuses_unfunded_reads() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const {get} = Object.getOwnPropertyDescriptor(Document.prototype, 'referrer');
                for (const receiver of [Object.create(Document.prototype), document.body]) {
                    try { get.call(receiver); return 'unbranded receiver accepted'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'wrong receiver error'; }
                }
                const xml = document.implementation.createDocument('urn:example', 'p:root');
                const clone = document.cloneNode(false);
                if (!(xml instanceof XMLDocument) || clone === document || !(clone instanceof Document))
                    return 'qualified Document construction';
                document.body.appendChild(document.createTextNode('unrelated tree edit'));
                if ([new Document(), xml, clone, document].some(node => get.call(node) !== ''))
                    return 'creation-derived empty value';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
    let stamp = host.snapshot().unwrap().stamp();
    let backing = guest_backing_used(&host);
    exhaust_guest_reaction_work(&host);
    let refused = host.in_document(&permit, |scope| {
        evaluate(scope, "document.referrer").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = refused else {
        panic!("referrer must admit even an empty native observation: {refused:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), stamp);
    assert_eq!(guest_backing_used(&host), backing);
    host.stop().into_session().unwrap();
}
