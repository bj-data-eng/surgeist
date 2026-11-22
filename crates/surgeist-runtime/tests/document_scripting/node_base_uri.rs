use super::{window_length::observe, *};

pub(super) fn node_base_uri_exposes_the_actual_node_document() {
    let (session, top) = setup();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 60_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r"(() => {
        const descriptor = Object.getOwnPropertyDescriptor(Node.prototype, 'baseURI');
        if (typeof descriptor?.get !== 'function') return 'missing Node baseURI getter';
        const {get, set, enumerable, configurable} = descriptor;
        if (set !== undefined || !enumerable || !configurable || get.name !== 'get baseURI' ||
            get.length !== 0 || Object.hasOwn(get, 'prototype') || Object.hasOwn(document, 'baseURI'))
            return 'incorrect Node baseURI descriptor';
        for (const receiver of [null, {}, Object.create(document), new Proxy(document, {}), window]) {
            try { Reflect.apply(get, receiver, []); return 'unbranded Node receiver accepted'; }
            catch (error) { if (!(error instanceof TypeError)) throw error; }
        }
        const node = document.createElementNS(null, 'witness');
        const attribute = document.createAttribute('data-source');
        node.setAttributeNode(attribute);
        node.appendChild(document.createTextNode('source'));
        document.body.appendChild(node);
        for (const native of [document, node, attribute, node.firstChild,
            document.createDocumentFragment(), document.createComment('source')]) {
            if (native.baseURI !== 'about:blank' || get.call(native) !== 'about:blank')
                return 'the native Node must read its node Document base URL';
        }
        const xml = document.implementation.createDocument('urn:fixture', 'root');
        xml.adoptNode(node);
        if (node.ownerDocument !== xml || attribute.ownerDocument !== xml || node.parentNode !== null ||
            node.baseURI !== 'about:blank' || attribute.baseURI !== 'about:blank' || xml.baseURI !== 'about:blank')
            return 'detached and adopted Nodes must read their current native Document';
        return 'ok';
    })()",
    );
    assert_eq!(result, "ok");
    host.stop().into_session().unwrap();
}
