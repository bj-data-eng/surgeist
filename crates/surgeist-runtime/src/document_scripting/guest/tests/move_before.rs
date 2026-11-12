//! The real ParentNode method and original native move publication.

use super::super::interfaces::GuestNodeProjection;
use super::*;
use crate::{DomSourceDriveError, DomSourceRuntimeError};
use surgeist_dom::{DomBatch, DomChildListObservationLimits, DomResource, NodeId};
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
    let (from_doc, to_doc, moved, reference, comment) = host
        .in_document(&permit, |scope| {
            scope.evaluate(&units(
                "globalThis.fromDoc = new Document(); \
                 globalThis.toDoc = new Document(); \
                 globalThis.moved = new Text('retained'); \
                 globalThis.reference = new Text('reference'); \
                 globalThis.detached = new Text('detached'); \
                 globalThis.moveComment = new Comment('document child');",
            ))?;
            Ok((
                native_node(scope, "fromDoc")?,
                native_node(scope, "toDoc")?,
                native_node(scope, "moved")?,
                native_node(scope, "reference")?,
                native_node(scope, "moveComment")?,
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
        batch.insert_before(comment, from_doc, None);
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
                && rejected(() => Document.prototype.moveBefore.call(to, moved, reference), "TypeError")
                && rejected(() => to.moveBefore(moved, {}), "TypeError")
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
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "to.moveBefore(reference, null) === undefined && reference.parentNode === to",
        )
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(to).unwrap().children(),
        [moved, reference]
    );
    host.in_document(&permit, |scope| {
        assert_guest(scope, "fromDoc.moveBefore(moveComment, from) === undefined && moveComment.parentNode === fromDoc")
    })
    .unwrap();
    assert_eq!(
        host.snapshot().unwrap().node(from_doc).unwrap().children(),
        [comment, from]
    );

    let observer = host
        .with_native(|dom, _, _| {
            let raw = dom.raw_mut();
            let limits = DomChildListObservationLimits::new(2, 8, 1, 32).unwrap();
            raw.set_limits(raw.limits().with_child_list_observation(limits))
                .unwrap();
            let observer = raw.create_child_list_observer(&cancel).unwrap();
            raw.observe_child_list(observer, to, false, &cancel)
                .unwrap();
            observer
        })
        .unwrap();
    let before = host.snapshot().unwrap();
    let events = host.guest_events().unwrap();
    let bytes = events.budget.used.get();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("to.moveBefore(reference, moved)"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the second native observer record must refuse: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::ResourceLimit {
                resource: DomResource::ChildListRecords,
                limit: 1
            }
        )))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot().unwrap().node(to).unwrap().children(),
        [moved, reference]
    );
    assert_eq!(events.budget.used.get(), bytes);
    host.with_native(|dom, _, _| {
        assert!(dom.pending_source_edit().is_none());
        assert!(
            dom.raw_mut()
                .take_child_list_records(observer, &cancel)
                .unwrap()
                .is_empty()
        );
    })
    .unwrap();
}
