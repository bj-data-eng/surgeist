//! Attr interface behavior at the real attached Window.

use super::super::interfaces::GuestNodeProjection;
use super::*;
use surgeist_dom::{DomBatch, DomNodeKind};
use surgeist_html::values::{DocumentString, Namespace};

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
