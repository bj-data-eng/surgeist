use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

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

pub(super) fn cookie_assignment_preserves_conversion_reentry_and_abrupt_completion() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const {get, set, enumerable, configurable} =
                    Object.getOwnPropertyDescriptor(Document.prototype, 'cookie');
                if (!enumerable || !configurable || get.name !== 'get cookie' || get.length !== 0 ||
                    set.name !== 'set cookie' || set.length !== 1) return 'cookie descriptor';
                let conversions = 0;
                const input = {[Symbol.toPrimitive](hint) {
                    ++conversions;
                    if (hint !== 'string' || get.call(document) !== '') throw new Error('conversion order');
                    document.body.appendChild(document.createTextNode('reentrant edit'));
                    return 'test=\ud800';
                }};
                try { set.call(Object.create(Document.prototype), input); return 'forged receiver'; }
                catch (error) { if (!(error instanceof TypeError) || conversions !== 0) return 'brand ordering'; }
                if (set.call(document, input) !== undefined || conversions !== 1 ||
                    document.body.lastChild.data !== 'reentrant edit') return 'reentrant edit lost';
                const original = {};
                try { set.call(document, {toString() { throw original; }}); return 'throw lost'; }
                catch (error) { if (error !== original) return 'wrong abrupt completion'; }
                try { set.call(document, Symbol()); return 'Symbol accepted'; }
                catch (error) { if (!(error instanceof TypeError)) return 'wrong Symbol failure'; }
                return set.call(document) === undefined && get.call(document) === '' &&
                    new Document().cookie === '' ? 'ok' : 'cookie-averse state';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
