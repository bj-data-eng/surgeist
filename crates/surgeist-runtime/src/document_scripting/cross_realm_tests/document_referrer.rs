use super::{evaluate, hosted_session};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::sync::{Arc, atomic::AtomicBool};

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
