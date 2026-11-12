//! Attr interface behavior at the real attached Window.

use super::super::interfaces::GuestNodeProjection;
use super::*;
use surgeist_dom::{DomBatch, DomNodeKind};
use surgeist_html::values::{DocumentString, Namespace};

pub(in crate::document_scripting) fn inherited_node_values_use_canonical_text() {
    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let (element, text) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(permit.document, Namespace::html(), "div");
            let inner = batch.create_element_ns(permit.document, Namespace::html(), "span");
            let text = batch.create_text(permit.document, DocumentString::from_utf16([0xd800, 0]));
            let comment = batch.create_comment(permit.document, "excluded");
            let tail = batch.create_text(permit.document, "tail");
            batch.insert_before(&text, &element, None);
            batch.insert_before(&inner, &element, None);
            batch.insert_before(&comment, &inner, None);
            batch.insert_before(&tail, &inner, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created(&element).unwrap().unwrap(),
                commit.created(&text).unwrap().unwrap(),
            )
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "contentElement");
    expose_public_node_for_source_test(&mut host, &permit, text, "contentText");
    host.in_document(&permit, |scope| {
        scope.evaluate(&units(r#"
            (() => {
                const check = (ok, label) => { if (!ok) throw new Error(label); };
                check(contentElement.textContent === '\ud800\0tail', 'ordinary descendant text');
                const root = contentElement.attachShadow({mode: 'open'});
                root.textContent = 'shadow';
                check(root.textContent === 'shadow' && contentElement.textContent === '\ud800\0tail', 'no shadow piercing');
                const attr = document.createAttribute('data-value');
                const set = Object.getOwnPropertyDescriptor(Node.prototype, 'nodeValue').set;
                set.call(attr, 'first\udfff');
                check(attr.textContent === 'first\udfff' && attr.value === attr.nodeValue, 'exact Attr value');
                attr.textContent = null;
                check(attr.value === '' && attr.nodeValue === '', 'null becomes empty');
                set.call(attr);
                check(attr.value === '', 'omission converts as nullable undefined');
                contentText.nodeValue = 'new';
                check(contentElement.textContent === 'newtail' && contentText.data === 'new', 'live character data');
                let calls = 0;
                contentElement.nodeValue = {toString() { calls++; return 'ignored'; }};
                document.textContent = {toString() { calls++; return 'ignored'; }};
                check(calls === 2 && contentElement.nodeValue === null && document.textContent === null
                    && contentElement.textContent === 'newtail', 'convert before native no-op');
                contentElement.textContent = '\udfff\0';
                check(contentText.parentNode === null && contentElement.textContent === '\udfff\0', 'canonical replacement');
                contentElement.textContent = null;
                check(contentElement.textContent === '' && root.textContent === 'shadow', 'ordinary children only');
                try { set.call({}, 'bad'); }
                catch (error) { check(error instanceof TypeError, 'exact receiver brand'); return; }
                throw new Error('accepted foreign Node receiver');
            })()
        "#)).map(drop)
    }).unwrap();
    assert!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .children()
            .is_empty()
    );
}

pub(in crate::document_scripting) fn detached_attribute_keeps_its_live_native_identity() {
    let mut host = public_node_host(3_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let attr = host
        .in_document(&permit, |scope| {
            let result = scope.evaluate(&units(
                r#"
            (() => {
                const local = document.createAttribute('DATA-KEY');
                const attr = document.createAttributeNS('urn:example', 'x:token');
                const otherDocument = new Document();
                const otherAttr = Document.prototype.createAttribute.call(otherDocument, 'Mixed');
                globalThis.liveAttr = attr;
                if (!(attr instanceof Attr) || !(attr instanceof Node)
                    || attr.nodeType !== 2 || attr.nodeName !== 'x:token'
                    || attr.name !== 'x:token' || attr.localName !== 'token'
                    || attr.namespaceURI !== 'urn:example' || attr.prefix !== 'x'
                    || attr.ownerDocument !== document || attr.ownerElement !== null
                    || attr.parentNode !== null || attr.getRootNode() !== attr
                    || attr.isConnected || !attr.specified || attr.value !== ''
                    || local.name !== 'data-key' || local.namespaceURI !== null
                    || local.prefix !== null || otherAttr.name !== 'Mixed'
                    || otherAttr.ownerDocument !== otherDocument || otherAttr === local)
                    throw new Error('wrong detached Attr');
                const expect = (operation, name) => {
                    try { operation(); } catch (error) { return error.name === name; }
                    return false;
                };
                if (!expect(() => new Attr(), 'TypeError')
                    || !expect(() => document.createAttribute('not valid'), 'InvalidCharacterError')
                    || !expect(() => document.createAttributeNS('', 'x:token'), 'NamespaceError')
                    || !expect(() => document.createAttribute(), 'TypeError'))
                    throw new Error('wrong Attr constructor error');
                let namespaceConversions = 0;
                const namespace = { toString() { namespaceConversions++; throw new Error('coercion'); } };
                if (!expect(() => document.createAttributeNS(namespace), 'TypeError') || namespaceConversions !== 0)
                    throw new Error('missing qualified name must refuse before namespace conversion');
                attr.value = 'detached\ud800';
                if (attr.value !== 'detached\ud800') throw new Error('wrong detached Attr value');
                return attr;
            })()
        "#,
            ))?;
            let projection = scope
                .native_object::<GuestNodeProjection>(&result)?
                .unwrap();
            Ok(projection.payload().id().unwrap().node())
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let native_attr = snapshot.attr(attr).unwrap().id();
    assert!(matches!(
        snapshot.node(attr).unwrap().kind(),
        DomNodeKind::Attr(_)
    ));
    let exact = DocumentString::from_utf16(vec![u16::from(b'a'), 0xd800, 0]);
    let element = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(permit.document, Namespace::html(), "div");
            batch.attach_native_attribute(&element, native_attr);
            batch.set_attribute_ns(
                &element,
                Namespace::from_uri("urn:example"),
                "other:token",
                exact.clone(),
            );
            dom.apply(batch, &cancel)
                .unwrap()
                .created(&element)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "attrOwner");
    expose_public_node_for_source_test(&mut host, &permit, attr, "sameNativeAttr");
    host.in_document(&permit, |scope| {
        let value = scope.evaluate(&units(
            r#"
            sameNativeAttr === liveAttr && liveAttr.ownerElement === attrOwner
                && liveAttr.parentNode === null && !liveAttr.isConnected
                && liveAttr.value === 'a\ud800\u0000' && liveAttr.name === 'x:token'
        "#,
        ))?;
        assert!(scope.boolean(&value)?);
        Ok(())
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.remove_attribute_node(element, native_attr);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    host.in_document(&permit, |scope| {
        let value = scope.evaluate(&units(
            "liveAttr === sameNativeAttr && liveAttr.ownerElement === null && liveAttr.value.length === 3",
        ))?;
        assert!(scope.boolean(&value)?);
        Ok(())
    }).unwrap();
}

pub(in crate::document_scripting) fn element_methods_return_the_exact_attribute() {
    let mut host = public_node_host(5_000, 20_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let (element, other, unprojected) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let element = batch.create_element_ns(permit.document, Namespace::html(), "div");
            let other = batch.create_element_ns(permit.document, Namespace::html(), "span");
            batch.set_attribute(&element, "data-old", "native");
            let commit = dom.apply(batch, &cancel).unwrap();
            let element = commit.created(&element).unwrap().unwrap();
            let attr = dom
                .snapshot()
                .node(element)
                .unwrap()
                .get_attribute_node(&"data-old".into())
                .unwrap()
                .id()
                .node();
            (element, commit.created(&other).unwrap().unwrap(), attr)
        })
        .unwrap();
    expose_public_node_for_source_test(&mut host, &permit, element, "element");
    expose_public_node_for_source_test(&mut host, &permit, other, "other");
    let removed = host.in_document(&permit, |scope| {
        let value = scope.evaluate(&units(r#"
            (() => {
                const assert = (condition, message) => { if (!condition) throw new Error(message); };
                const throws = (operation, name) => {
                    try { operation(); } catch (error) { return error.name === name; }
                    return false;
                };
                const plain = document.createAttribute('data-old');
                const old = element.setAttributeNode(plain);
                assert(old instanceof Attr && old !== plain && old.ownerElement === null
                    && old.name === 'data-old' && old.value === 'native', 'previously unprojected replacement');
                assert(element.getAttributeNode('DATA-OLD') === plain
                    && element.getAttributeNodeNS('', 'data-old') === plain
                    && element.getAttributeNodeNS(null, 'DATA-OLD') === null
                    && element.setAttributeNodeNS(plain) === plain, 'qualified and expanded identity');
                const first = document.createAttributeNS('urn:test', 'p:entry');
                const second = document.createAttributeNS('urn:test', 'q:entry');
                assert(element.setAttributeNode(first) === null
                    && element.getAttributeNode('P:ENTRY') === first
                    && element.getAttributeNodeNS('urn:test', 'entry') === first
                    && element.setAttributeNodeNS(second) === first
                    && element.getAttributeNode('p:entry') === null
                    && element.getAttributeNode('q:entry') === second, 'namespace replacement');
                assert(throws(() => other.setAttributeNode(second), 'InUseAttributeError')
                    && throws(() => other.removeAttributeNode(second), 'NotFoundError')
                    && throws(() => element.setAttributeNode(Object.create(Attr.prototype)), 'TypeError')
                    && throws(() => element.removeAttributeNode(new Proxy(second, {})), 'TypeError')
                    && throws(() => Element.prototype.getAttributeNode.call({}), 'TypeError'), 'operation errors');
                let converted = 0;
                const namespace = { toString() { converted++; return ''; } };
                assert(throws(() => element.getAttributeNodeNS(namespace), 'TypeError') && converted === 0,
                    'arity before namespace coercion');
                assert(element.getAttributeNodeNS(namespace, 'data-old') === plain && converted === 1,
                    'namespace coercion once');
                assert(element.removeAttributeNode(second) === second && second.ownerElement === null
                    && element.getAttributeNodeNS('urn:test', 'entry') === null
                    && throws(() => element.removeAttributeNode(second), 'NotFoundError'), 'exact removal');
                return old;
            })()
        "#))?;
        let projection = scope.native_object::<GuestNodeProjection>(&value)?.unwrap();
        Ok(projection.payload().id().unwrap().node())
    }).unwrap();
    assert_eq!(removed, unprojected);
    assert_eq!(
        host.snapshot()
            .unwrap()
            .attr(removed)
            .unwrap()
            .owner_element(),
        None
    );

    let old = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.set_attribute(other, "data-old", "before");
            dom.apply(batch, &cancel).unwrap();
            dom.snapshot()
                .node(other)
                .unwrap()
                .get_attribute_node(&"data-old".into())
                .unwrap()
                .id()
        })
        .unwrap();
    let replacement = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&units(
                "globalThis.replacement = document.createAttribute('data-old')",
            ))?;
            Ok(scope
                .native_object::<GuestNodeProjection>(&value)?
                .unwrap()
                .payload()
                .id()
                .unwrap()
                .node())
        })
        .unwrap();
    let events = host.guest_events().unwrap();
    let spare = events.limits.storage_bytes - events.budget.used.get();
    let backing = events.reserve(&host.native, spare).unwrap();
    let before = events.budget.used.get();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("other.setAttributeNode(replacement)"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("new guest projection must refuse without backing: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::StorageLimit)
        ))
    ));
    assert_eq!(events.budget.used.get(), before);
    drop(backing);
    assert!(matches!(
        host.in_document(&permit, |_| Ok(())),
        Err(DocumentScriptError::Script(Error::HostFailure(_)))
    ));
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
        let snapshot = dom.snapshot();
        assert_eq!(snapshot.attr(old.node()).unwrap().owner_element(), None);
        assert_eq!(
            snapshot.attr(replacement).unwrap().owner_element(),
            Some(other)
        );
    })
    .unwrap();
}
