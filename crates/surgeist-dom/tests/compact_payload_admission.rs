//! Source edits retain unchanged payloads under a fixed finite UTF-16 policy.
//! Expectations use authored literals and actual immutable backing, not receipts.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomLimits, DomNameLimits,
    DomSnapshot, DomSourceCompletion, DomSourceEdit, DomSourceLimits, DomSourceProgress, Namespace,
    NodeId,
};

const LARGE_UNITS: usize = 65_536;
const SOURCE_UNITS: usize = LARGE_UNITS + LARGE_UNITS / 2;
const PATTERN: [u16; 4] = [97, 0xd800, 0, 0xdc00];

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            1_000_000,
            512,
            0,
            0,
            16,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_input_processing(1_000_000, 10_000_000)
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    apply(dom, batch).document(&document).unwrap()
}
fn value(extent: usize) -> DocumentString {
    let value = DocumentString::from_utf16((0..extent).map(|i| PATTERN[i % 4]).collect::<Vec<_>>());
    assert_eq!(value.shared_code_units().capacity(), extent);
    value
}
fn assert_value(value: &DocumentString, extent: usize) {
    assert_eq!(value.code_units().len(), extent);
    for (i, unit) in value.code_units().iter().copied().enumerate() {
        assert_eq!(unit, PATTERN[i % 4]);
    }
}
fn complete(dom: &mut Dom, edit: DomSourceEdit, extent: usize) -> DomSourceCompletion {
    // The same policy admits one large original occurrence with ample space
    // for small literal names. The edit changes no large string and needs no
    // second full occurrence; bytes/work are deliberately generous here.
    let policy = DomSourceLimits::new(256_000_000, SOURCE_UNITS, 10_000_000, 16, 128, 128, 1);
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, policy, &cancel)
        .unwrap_or_else(|error| {
            panic!("extent={extent}, fixed retained units={SOURCE_UNITS}, Source begin: {error:?}")
        });
    let operation = lease.status().operation();
    let basis = lease.snapshot().stamp();
    let result = lease.advance(&cancel);
    match result {
        Ok(DomSourceProgress::Complete(done)) => {
            assert_eq!(done.operation(), operation);
            assert!(!done.cancellation_requested());
            assert_eq!(done.output(), None);
            done
        }
        Err(error) => {
            assert_eq!(lease.snapshot().stamp(), basis);
            assert_eq!(lease.status().operation(), operation);
            panic!(
                "extent={extent}, fixed retained units={SOURCE_UNITS}, unchanged-payload Source advance: {error:?}"
            );
        }
        Ok(other) => {
            panic!("plain local edit must complete without an external service: {other:?}")
        }
    }
}
fn topology_value(snapshot: &DomSnapshot, target: NodeId, element: bool) -> &DocumentString {
    let node = snapshot.node(target).unwrap();
    if element {
        let element = node.as_element().unwrap();
        assert!(element.name().namespace_uri().is_none());
        assert!(element.name().prefix().is_none());
        assert_eq!(
            element.name().local_name().code_units(),
            [99, 101, 108, 108]
        );
        let attributes = element.attributes();
        assert_eq!(attributes.len(), 1);
        assert!(attributes[0].name().namespace_uri().is_none());
        assert!(attributes[0].name().prefix().is_none());
        assert_eq!(
            attributes[0].name().local_name().code_units(),
            [112, 97, 121, 108, 111, 97, 100]
        );
        attributes[0].value()
    } else {
        assert!(node.as_element().is_none());
        node.data().unwrap()
    }
}
fn move_unchanged_payload(element: bool) {
    for extent in [1, LARGE_UNITS] {
        let mut dom = owner();
        let document = document(&mut dom);
        let caller = value(extent);
        let mut batch = DomBatch::new();
        let from = batch.create_fragment(document);
        let to = batch.create_fragment(document);
        let target = if element {
            let target = batch.create_element(document, "cell");
            batch.set_attribute(&target, "payload", caller.clone());
            target
        } else {
            batch.create_text(document, caller.clone())
        };
        batch.insert_before(&target, &from, None);
        let created = apply(&mut dom, batch);
        let from = created.created(&from).unwrap().unwrap();
        let to = created.created(&to).unwrap().unwrap();
        let target = created.created(&target).unwrap().unwrap();
        let old = dom.snapshot();
        assert_eq!(old.nodes().count(), 4);
        assert_eq!(old.node(from).unwrap().children(), [target]);
        assert!(old.node(to).unwrap().children().is_empty());
        assert_eq!(old.node(target).unwrap().parent(), Some(from));
        assert_eq!(old.node(target).unwrap().owner_document(), Some(document));
        let retained = topology_value(&old, target, element);
        assert_value(retained, extent);
        assert!(Arc::ptr_eq(
            caller.shared_code_units(),
            retained.shared_code_units()
        ));

        let done = complete(
            &mut dom,
            DomSourceEdit::insert_before(target, to, None),
            extent,
        );
        assert_eq!(
            done.commit().changes().changes(),
            [
                DomChange::Children(from),
                DomChange::Children(to),
                DomChange::Parent(target)
            ]
        );
        let now = dom.snapshot();
        assert_eq!(now.nodes().count(), 4);
        assert!(now.node(from).unwrap().children().is_empty());
        assert_eq!(now.node(to).unwrap().children(), [target]);
        assert_eq!(now.node(target).unwrap().id(), target);
        assert_eq!(now.node(target).unwrap().parent(), Some(to));
        assert_eq!(now.node(target).unwrap().owner_document(), Some(document));
        assert_eq!(old.node(from).unwrap().children(), [target]);
        assert!(old.node(to).unwrap().children().is_empty());
        assert_eq!(old.node(target).unwrap().parent(), Some(from));
        for snapshot in [&old, &now] {
            let value = topology_value(snapshot, target, element);
            assert_value(value, extent);
            assert!(Arc::ptr_eq(
                caller.shared_code_units(),
                value.shared_code_units()
            ));
        }
        assert!(dom.pending_source_edit().is_none());
        eprintln!(
            "topology payload element={element} extent={extent}: literal move and backing controls passed"
        );
    }
}

#[test]
fn source_text_topology_edit_shares_large_unchanged_data_under_finite_admission() {
    move_unchanged_payload(false);
}

#[test]
fn source_element_topology_edit_shares_large_unchanged_attributes_under_finite_admission() {
    move_unchanged_payload(true);
}

#[test]
fn source_attribute_edit_shares_large_unchanged_native_value_and_children_under_finite_admission() {
    for extent in [1, LARGE_UNITS] {
        let mut dom = owner();
        let document = document(&mut dom);
        let caller = value(extent);
        let mut batch = DomBatch::new();
        let input = batch.create_element_ns(document, Namespace::html(), "input");
        batch.set_input_value(&input, caller.clone());
        let children: Vec<_> = (0..64)
            .map(|_| {
                let child = batch.create_comment(document, "child");
                batch.insert_before(&child, &input, None);
                child
            })
            .collect();
        let created = apply(&mut dom, batch);
        let input = created.created(&input).unwrap().unwrap();
        let children: Vec<_> = children
            .iter()
            .map(|child| created.created(child).unwrap().unwrap())
            .collect();
        let old = dom.snapshot();
        let node = old.node(input).unwrap();
        assert_eq!(node.children(), children);
        assert_eq!(node.owner_document(), Some(document));
        let native = node.as_input().unwrap();
        assert!(native.state().flags().dirty_value());
        assert_eq!(native.value(), caller.code_units());
        assert_value(native.state().current_value(), extent);
        // The real Input setter sanitizes its operand. Retain the resulting
        // native backing, rather than requiring it to alias the caller input.
        let native_backing = native.state().current_value().clone();
        let native_capacity = native_backing.shared_code_units().capacity();
        assert!(native_capacity < SOURCE_UNITS);
        assert!(node.as_element().unwrap().attributes().is_empty());
        eprintln!("native Input premise extent={extent} actual_capacity={native_capacity}");

        let done = complete(
            &mut dom,
            DomSourceEdit::set_attribute(input, "id", "changed"),
            extent,
        );
        assert_eq!(
            done.commit().changes().changes(),
            [DomChange::Attributes(input)]
        );
        let now = dom.snapshot();
        assert_eq!(now.nodes().count(), 66);
        let current = now.node(input).unwrap();
        assert_eq!(current.id(), input);
        assert_eq!(current.owner_document(), Some(document));
        assert_eq!(current.parent(), None);
        assert_eq!(current.children(), children);
        assert!(std::ptr::eq(node.children(), current.children()));
        let attributes = current.as_element().unwrap().attributes();
        assert_eq!(attributes.len(), 1);
        assert!(attributes[0].name().namespace_uri().is_none());
        assert!(attributes[0].name().prefix().is_none());
        assert_eq!(attributes[0].name().local_name().code_units(), [105, 100]);
        assert_eq!(
            attributes[0].value().code_units(),
            [99, 104, 97, 110, 103, 101, 100]
        );
        assert!(node.as_element().unwrap().attributes().is_empty());
        assert_eq!(node.children(), children);
        for snapshot in [&old, &now] {
            let node = snapshot.node(input).unwrap();
            let native = node.as_input().unwrap();
            assert!(native.state().flags().dirty_value());
            assert_eq!(native.value(), caller.code_units());
            assert_value(native.state().current_value(), extent);
            assert_eq!(
                native
                    .state()
                    .current_value()
                    .shared_code_units()
                    .capacity(),
                native_capacity
            );
            assert!(Arc::ptr_eq(
                native_backing.shared_code_units(),
                native.state().current_value().shared_code_units()
            ));
            for child in &children {
                let child = snapshot.node(*child).unwrap();
                assert_eq!(child.parent(), Some(input));
                assert_eq!(child.owner_document(), Some(document));
                assert_eq!(child.data().unwrap().code_units(), [99, 104, 105, 108, 100]);
            }
        }
        assert!(dom.pending_source_edit().is_none());
        eprintln!(
            "native Input extent={extent}: literal attribute, dirty-value, child and backing controls passed"
        );
    }
}
