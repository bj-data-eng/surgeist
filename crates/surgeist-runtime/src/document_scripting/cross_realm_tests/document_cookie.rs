use super::{evaluate, hosted_session};
use crate::document_scripting::guest::tests::attach_public_nodes;
use std::{sync::Arc, sync::atomic::AtomicBool};

pub(super) fn cookie_averse_documents_read_empty_and_ignore_assignment() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            r#"
            (() => {
                const detached = document.implementation.createHTMLDocument('native');
                if (!(document instanceof Document) || detached.title !== 'native')
                    return 'native cookie fixture';
                const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'cookie');
                if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function')
                    return 'document missing cookie accessor';
                for (const receiver of [document, detached]) {
                    if (descriptor.get.call(receiver) !== '') return 'initial cookie read';
                    if (descriptor.set.call(receiver, 'example=active') !== undefined ||
                        descriptor.get.call(receiver) !== '') return 'cookie-averse write';
                }
                return 'ok';
            })()
        "#,
        )?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    })
    .unwrap();
}
