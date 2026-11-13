use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, exhaust_guest_reaction_work, projected_node_id,
};

pub(super) fn meta_reflections_read_and_mutate_the_live_native_element() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    let node = host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const meta = document.createElement('meta');
                if (!(meta instanceof HTMLMetaElement)) return 'meta interface';
                const members = ['name', 'httpEquiv', 'content', 'media', 'scheme'];
                if (Object.keys(HTMLMetaElement.prototype).join() !== members.join())
                    return 'declaration order';
                for (const member of members) {
                    const descriptor = Object.getOwnPropertyDescriptor(HTMLMetaElement.prototype, member);
                    if (!descriptor?.get || !descriptor?.set || !descriptor.enumerable ||
                        !descriptor.configurable || descriptor.get.length !== 0 ||
                        descriptor.set.length !== 1 || meta[member] !== '') return 'descriptor ' + member;
                }
                const setName = Object.getOwnPropertyDescriptor(HTMLMetaElement.prototype, 'name').set;
                let coerced = false;
                for (const receiver of [document.createElement('div'), new Proxy(meta, {}),
                    document.createElementNS('urn:foreign', 'meta')]) {
                    try { Object.getOwnPropertyDescriptor(HTMLMetaElement.prototype, 'name').get.call(receiver); return 'getter brand'; }
                    catch (error) { if (!(error instanceof TypeError)) return 'getter error'; }
                    try { setName.call(receiver, {toString() { coerced = true; }}); return 'brand'; }
                    catch (error) { if (!(error instanceof TypeError) || coerced) return 'coercion order'; }
                }
                meta.setAttributeNS('urn:foreign', 'name', 'foreign');
                if (meta.name !== '') return 'foreign attribute ignored';
                meta.name = {toString() {
                    meta.setAttributeNS(null, 'name', 'nested');
                    return 'viewport\ud800';
                }};
                meta.content = 'width=device-width\0';
                meta.httpEquiv = 'refresh';
                meta.media = 'screen';
                meta.scheme = 'tag';
                if (meta.name !== 'viewport\ud800' || meta.content !== 'width=device-width\0' ||
                    meta.httpEquiv !== 'refresh' || meta.media !== 'screen' || meta.scheme !== 'tag')
                    return 'live getters';
                if (meta.getAttributeNS(null, 'name') !== 'viewport\ud800' ||
                    meta.getAttributeNS('urn:foreign', 'name') !== 'foreign' ||
                    meta.getAttribute('content') !== 'width=device-width\0' ||
                    meta.getAttribute('http-equiv') !== 'refresh' || meta.getAttribute('media') !== 'screen' ||
                    meta.getAttribute('scheme') !== 'tag') return 'native reflection';
                globalThis.nativeMeta = meta;
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        let meta = evaluate(scope, "nativeMeta")?;
        projected_node_id(scope, &meta)
    }).unwrap_or_else(|error| match &error {
        DocumentScriptError::Script(Error::HostFailure(failure)) => {
            panic!("reflected meta setup: {:?}, {:?}", failure.cause::<DocumentGuestEventError>(), failure.cause::<crate::DomSourceDriveError>())
        }
        _ => panic!("reflected meta setup: {error:?}"),
    });
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(node, Namespace::none(), "name", "timeout");
        batch.set_attribute_ns(node, Namespace::none(), "content", "long");
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            "nativeMeta.name === 'timeout' && nativeMeta.content === 'long'",
        )?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })
    .unwrap();
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    exhaust_guest_reaction_work(&host);
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "nativeMeta.content = 'refused'").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("reflected meta mutation must respect native work: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::WorkLimit)
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
}
