use super::*;
use crate::document_scripting::guest::tests::attach_public_nodes;

// HTML877d's obsolete HTMLPreElement.width is a reflected long, including
// listing/xmp's HTMLPreElement mapping and the signed-long reflection rules.
pub(super) fn pre_width_reflects_the_canonical_signed_attribute() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const elements = ['pre', 'listing', 'xmp'].map(tag => document.createElement(tag));
                if (!elements.every(element => element instanceof HTMLPreElement)) return 'pre family interfaces';
                if (Object.keys(HTMLPreElement.prototype).join() !== 'width') return 'pre missing reflected width';
                const descriptor = Object.getOwnPropertyDescriptor(HTMLPreElement.prototype, 'width');
                if (typeof descriptor?.get !== 'function' || typeof descriptor?.set !== 'function' ||
                    !descriptor.enumerable || !descriptor.configurable ||
                    descriptor.get.length !== 0 || descriptor.set.length !== 1) return 'pre width descriptor';
                for (const element of elements) {
                    if (element.width !== 0) return element.localName + ' missing width default';
                    element.width = -7;
                    if (element.width !== -7 || element.getAttributeNS(null, 'width') !== '-7')
                        return element.localName + ' canonical signed reflection';
                }
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}
