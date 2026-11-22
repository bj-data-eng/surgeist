//! Native Element attributes exposed through the attached Window.

use super::{window_length::observe, *};

pub(super) fn follows_the_native_attribute_list() {
    let (session, top) = setup();
    let document = top.document();
    let (mut host, _) =
        attach_event_window_with_work(session, top, Arc::new(cancel()), 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let element = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let body = snapshot.node(html).unwrap().children()[1];
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(document, Namespace::html(), "div");
            batch.set_attribute(&element, "id", "attribute-map-owner");
            batch.set_attribute(
                &element,
                "data-exact",
                DocumentString::from_utf16([0xd800, 0, 0xdc00]),
            );
            batch.set_attribute(&element, "length", "collision");
            batch.insert_before(&element, body, None);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&element)
                .unwrap()
                .unwrap()
        })
        .unwrap();

    let initial = observe(
        &mut host,
        &permit,
        r#"(() => {
            const element = document.getElementById('attribute-map-owner');
            const map = element.attributes;
            if (typeof NamedNodeMap !== 'function' || map === undefined) {
                return 'the native Element must expose a NamedNodeMap';
            }
            const exact = element.getAttributeNode('data-exact');
            if (!(map instanceof NamedNodeMap) || map !== element.attributes
                || NamedNodeMap.prototype[Symbol.iterator] !== Array.prototype.values
                || Array.from(map)[1] !== exact
                || map.length !== 3 || map.item(1) !== exact || map[1] !== exact
                || map['data-exact'] !== exact || map.getNamedItem('DATA-EXACT') !== exact
                || exact.value !== '\ud800\0\udc00' || exact.ownerElement !== element
                || map.item(3) !== null || map[3] !== undefined
                || map.getNamedItem('absent') !== null
                || map['DATA-EXACT'] !== undefined
                || map.getNamedItem('length').value !== 'collision') {
                return 'the map must preserve live order, exact Attr identity, and name visibility';
            }
            const index = Object.getOwnPropertyDescriptor(map, '1');
            const named = Object.getOwnPropertyDescriptor(map, 'data-exact');
            if (Reflect.ownKeys(map).join() !== '0,1,2,id,data-exact'
                || index?.value !== exact || !index.configurable || !index.enumerable || index.writable
                || named?.value !== exact || !named.configurable || named.enumerable || named.writable
                || Object.hasOwn(map, 'length') || Reflect.preventExtensions(map)
                || Reflect.set(map, '1', exact) || Reflect.deleteProperty(map, 'data-exact')
                || Reflect.defineProperty(map, 'length', {value: 50})) {
                return 'the map must implement indexed and unenumerable named properties';
            }
            const throws = (action, name) => {
                try { action(); } catch (error) { return error.name === name; }
                return false;
            };
            let calls = 0;
            const raw = { toString() { ++calls; return 'data-exact'; } };
            if (!throws(() => new NamedNodeMap(), 'TypeError')
                || !throws(() => NamedNodeMap.prototype.getNamedItem.call(Object.create(map), raw), 'TypeError')
                || calls !== 0 || map.getNamedItem(raw) !== exact || calls !== 1
                || !throws(() => map.removeNamedItem('absent'), 'NotFoundError')) {
                return 'brand, required arguments, and native errors must survive exposure';
            }
            globalThis.attributeMapProbe = { element, map, exact };
            return 'ok';
        })()"#,
    );
    assert_eq!(initial, "ok");

    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.set_attribute(element, "data-exact", "updated in Rust");
        batch.set_attribute(element, "data-added", "added in Rust");
        dom.apply(batch, &cancel()).unwrap();
    })
    .unwrap();
    let mutations = observe(
        &mut host,
        &permit,
        r#"(() => {
            const { element, map, exact } = attributeMapProbe;
            if (element.attributes !== map || map.length !== 4 || map[1] !== exact
                || exact.value !== 'updated in Rust' || map['data-added'] !== map[3]) {
                return 'the map and previously projected Attr must observe native edits';
            }
            const first = document.createAttributeNS('urn:test', 'p:entry');
            const replacement = document.createAttributeNS('urn:test', 'q:entry');
            if (map.setNamedItem(first) !== null || map[4] !== first
                || map.getNamedItemNS('urn:test', 'entry') !== first
                || map.setNamedItemNS(replacement) !== first || first.ownerElement !== null
                || map[4] !== replacement || map['p:entry'] !== undefined
                || map['q:entry'] !== replacement || replacement.ownerElement !== element
                || map.setNamedItem(replacement) !== replacement) {
                return 'setNamedItem must preserve the exact native replacement and position';
            }
            if (map.removeNamedItemNS('urn:test', 'entry') !== replacement
                || replacement.ownerElement !== null || map[4] !== undefined
                || map.removeNamedItem('data-exact') !== exact || exact.ownerElement !== null
                || element.getAttribute('data-exact') !== null || map.length !== 3) {
                return 'removing by either name must return the original detached Attr';
            }
            return 'ok';
        })()"#,
    );
    assert_eq!(mutations, "ok");
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.node(element).unwrap().attributes().unwrap().len(),
        3
    );
}

pub(super) fn verifies_attachment_before_rechecking_ownership() {
    let cancellation = Arc::new(cancel());
    let mut dom = native_dom();
    dom.set_limits(dom.limits().with_meta_processing_work(1_000_000))
        .unwrap();
    let mut session = DomSession::new(dom, limits()).unwrap();
    let (_, top) = create(&mut session);
    session.enable_trusted_types_csp(&cancellation).unwrap();
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let destination = batch.create_element_ns(document, Namespace::html(), "div");
    let source = batch.create_element_ns(document, Namespace::html(), "div");
    batch.set_attribute(&destination, "id", "map-destination");
    batch.set_attribute(&destination, "onclick", "original");
    batch.insert_before(&destination, body, None);
    batch.set_attribute(&source, "id", "map-source");
    batch.set_attribute(
        &source,
        "onclick",
        DocumentString::from_utf16([b'a' as u16, 0xd800, 0]),
    );
    batch.insert_before(&source, body, None);
    let commit = session.dom().apply(batch, &cancellation).unwrap();
    let destination = commit.created(&destination).unwrap().unwrap();
    let source = commit.created(&source).unwrap().unwrap();
    let before = session.snapshot();
    let incoming = before
        .node(source)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();
    let old = before
        .node(destination)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();
    let (mut host, _) =
        attach_event_window_with_work(session, top, cancellation, 160_000, |_, _, _| {});
    let permit = host.permit().unwrap();
    let result = observe(
        &mut host,
        &permit,
        r#"(() => {
            const destination = document.getElementById('map-destination');
            const source = document.getElementById('map-source');
            const target = destination.attributes;
            const origin = source.attributes;
            if (!target || !origin) return 'the native Attr operation must be reachable through both maps';

            const meta = document.createElement('meta');
            meta.httpEquiv = 'Content-Security-Policy';
            meta.content = "require-trusted-types-for 'script'";
            document.documentElement.firstChild.appendChild(meta);

            const old = destination.getAttributeNode('onclick');
            const incoming = source.getAttributeNode('onclick');
            const marker = {};
            const calls = [];
            let move = false;
            trustedTypes.createPolicy('default', {
                createScript(value, type, sink) {
                    calls.push(value);
                    if (type !== 'TrustedScript' || sink !== 'Element onclick')
                        throw new Error('wrong native sink');
                    if (value === 'refuse') throw marker;
                    if (move && (origin.getNamedItem('ONCLICK') !== incoming
                        || origin.removeNamedItem('ONCLICK') !== incoming
                        || incoming.ownerElement !== null || target.onclick !== old)) {
                        throw new Error('nested removal lost the live Attr or its owner');
                    }
                    return value + '!';
                }
            });
            if (target.setNamedItem(old) !== old || old.value !== 'original' || calls.length !== 1)
                return 'verification must precede the same-identity return';
            try { target.setNamedItem(incoming); return 'accepted an attribute in use'; }
            catch (error) { if (error.name !== 'InUseAttributeError') throw error; }
            if (calls.length !== 2 || calls[1] !== 'a\ud800\0' || incoming.ownerElement !== source)
                return 'verification must precede the InUseAttributeError';

            move = true;
            const previous = target.setNamedItemNS(incoming);
            if (previous !== old || old.ownerElement !== null || incoming.ownerElement !== destination
                || target.onclick !== incoming || origin.getNamedItem('onclick') !== null
                || incoming.value !== 'a\ud800\0!' || calls.length !== 3 || calls[2] !== calls[1]) {
                return 'the original write must observe the reentrant removal exactly once';
            }
            const refused = document.createAttribute('onclick');
            refused.value = 'refuse';
            try { target.setNamedItem(refused); return 'accepted a thrown policy callback'; }
            catch (error) { if (error !== marker) throw error; }
            return target.onclick === incoming && refused.ownerElement === null && calls.length === 4
                ? 'ok' : 'policy failure changed the accepted Attr';
        })()"#,
    );
    assert_eq!(result, "ok");
    let snapshot = host.snapshot().unwrap();
    assert_eq!(
        snapshot.attr(incoming.node()).unwrap().owner_element(),
        Some(destination)
    );
    assert_eq!(snapshot.attr(old.node()).unwrap().owner_element(), None);
    assert!(
        snapshot
            .node(source)
            .unwrap()
            .get_attribute_node(&"onclick".into())
            .is_none()
    );
}
