//! The real ParentNode method and original native move publication.

use super::super::interfaces::GuestNodeProjection;
use super::*;
use surgeist_dom::{DomBatch, NodeId};
use surgeist_html::values::Namespace;

fn native_node(scope: &mut Scope<'_>, expression: &str) -> Result<NodeId, Error> {
    let value = scope.evaluate(&units(expression))?;
    let projection = scope.native_object::<GuestNodeProjection>(&value)?.unwrap();
    Ok(projection.payload().id().unwrap().node())
}

pub(in crate::document_scripting) fn parent_node_move_uses_the_live_native_identities() {
    let mut host = public_node_host(2_000, 100_000);
    let permit = host.permit().unwrap();
    let cancel = host.native.cancellation.clone();
    let (from_doc, to_doc, moved, reference) = host
        .in_document(&permit, |scope| {
            scope.evaluate(&units(
                "globalThis.fromDoc = new Document(); \
                 globalThis.toDoc = new Document(); \
                 globalThis.moved = new Text('retained'); \
                 globalThis.reference = new Text('reference'); \
                 globalThis.detached = new Text('detached');",
            ))?;
            Ok((
                native_node(scope, "fromDoc")?,
                native_node(scope, "toDoc")?,
                native_node(scope, "moved")?,
                native_node(scope, "reference")?,
            ))
        })
        .unwrap();
    let (from, to) = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let from = batch.create_element_ns(from_doc, Namespace::html(), "section");
            let to = batch.create_element_ns(to_doc, Namespace::html(), "article");
            batch.insert_before(&from, from_doc, None);
            batch.insert_before(&to, to_doc, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.created(&from).unwrap().unwrap(),
                commit.created(&to).unwrap().unwrap(),
            )
        })
        .unwrap();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.from = fromDoc.documentElement; \
             globalThis.to = toDoc.documentElement; \
             globalThis.marker = Symbol(); moved.marker = marker;",
            ))
            .map(drop)
    })
    .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.insert_before(moved, from, None);
        batch.insert_before(to, from, None);
        batch.insert_before(reference, to, None);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();

    host.in_document(&permit, |scope| {
        assert_guest(scope, r#"(() => {
            const method = Element.prototype.moveBefore;
            const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, "moveBefore");
            return typeof method === "function" && method.name === "moveBefore" && method.length === 2
                && descriptor.enumerable && descriptor.configurable && descriptor.writable
                && typeof Document.prototype.moveBefore === "function"
                && typeof DocumentFragment.prototype.moveBefore === "function"
                && ShadowRoot.prototype.moveBefore === DocumentFragment.prototype.moveBefore;
        })()"#)?;
        assert_guest(scope, r#"(() => {
            const rejected = (call, name) => {
                try { call(); return false; }
                catch (error) { return error.name === name; }
            };
            return moved.parentNode === from
                && rejected(() => to.moveBefore(moved), "TypeError")
                && rejected(() => to.moveBefore({}, null), "TypeError")
                && rejected(() => Element.prototype.moveBefore.call(moved, reference, null), "TypeError")
                && rejected(() => to.moveBefore(detached, from), "HierarchyRequestError")
                && rejected(() => to.moveBefore(moved, from), "NotFoundError")
                && moved.parentNode === from
                && to.moveBefore(moved, reference) === undefined
                && moved.parentNode === to && moved.ownerDocument === fromDoc && moved.marker === marker;
        })()"#)
    }).unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(from).unwrap().children(),
        [to]
    );
    assert_eq!(
        host.snapshot().unwrap().node(to).unwrap().children(),
        [moved, reference]
    );

    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "to.moveBefore(moved, moved) === undefined && moved.parentNode === to",
        )
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(to).unwrap().children(),
        [moved, reference]
    );
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "to.moveBefore(moved, undefined) === undefined && moved.marker === marker",
        )
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(to).unwrap().children(),
        [reference, moved]
    );
}
