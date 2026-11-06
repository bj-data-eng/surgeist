use super::{AccessError, AllocationCollection, EdgeVisitor, Handle, Heap, ObjectId, Trace};
use std::{cell::Cell, rc::Rc};

#[derive(Debug)]
struct Object {
    child: Option<Handle<Self>>,
}

impl Trace for Object {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) {
        if let Some(child) = self.child {
            edges.strong(child.erase());
        }
    }
}

#[derive(Debug)]
struct Other;

impl Trace for Other {
    fn trace(&self, _: &mut EdgeVisitor<'_>) {}
}

struct MutableEdges(Rc<Cell<Option<ObjectId>>>);

impl Trace for MutableEdges {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) {
        if let Some(child) = self.0.get() {
            edges.strong(child);
        }
    }
}

#[test]
fn checked_identity_survives_erasure_and_rejects_foreign_and_reused_slots() {
    let mut heap = Heap::new();
    let other_heap = Heap::new();
    let root = heap.allocate(Object { child: None }).expect("object").root;
    let object = root.handle();
    let erased = object.erase();

    assert_eq!(heap.downcast::<Object>(erased), Ok(object));
    assert_eq!(heap.downcast::<Other>(erased), Err(AccessError::WrongType));
    assert!(matches!(
        other_heap.get(object),
        Err(AccessError::ForeignHeap)
    ));

    let cloned = root.clone();
    drop(root);
    assert_eq!(
        heap.collect().expect("collect with one root").reclaimed(),
        0
    );
    drop(cloned);
    assert_eq!(
        heap.collect().expect("collect without roots").reclaimed(),
        1
    );
    assert!(matches!(heap.get(object), Err(AccessError::StaleHandle)));
    assert!(heap.upgrade(object).expect("same heap").is_none());

    let replacement = heap
        .allocate(Object { child: None })
        .expect("replacement")
        .root;
    assert_eq!(object.erase().slot, replacement.handle().erase().slot);
    assert_ne!(object, replacement.handle());
    assert!(matches!(heap.get(object), Err(AccessError::StaleHandle)));
}

#[test]
fn rooted_cycle_survives_and_is_reclaimed_after_its_root_is_released() {
    let mut heap = Heap::new();
    let parent = heap.allocate(Object { child: None }).expect("parent").root;
    let child = heap
        .allocate(Object {
            child: Some(parent.handle()),
        })
        .expect("child")
        .root;
    let child_handle = child.handle();
    heap.replace(
        parent.handle(),
        Object {
            child: Some(child_handle),
        },
    )
    .expect("close cycle");
    drop(child);

    assert_eq!(heap.collect().expect("rooted cycle").reclaimed(), 0);
    let upgraded = heap
        .upgrade(child_handle)
        .expect("same heap")
        .expect("reachable");
    drop(parent);
    assert_eq!(heap.collect().expect("upgraded cycle").reclaimed(), 0);
    drop(upgraded);
    assert_eq!(heap.collect().expect("unrooted cycle").reclaimed(), 2);
}

#[test]
fn collection_inside_allocation_keeps_inputs_and_reclaims_unrelated_objects() {
    let mut heap = Heap::new();
    let input = heap.allocate(Object { child: None }).expect("input").root;
    let canary = heap.allocate(Object { child: None }).expect("canary").root;
    let (input_handle, canary_handle) = (input.handle(), canary.handle());
    drop((input, canary));

    heap.set_allocation_collection(AllocationCollection::BeforeAllocation);
    let owner = heap
        .allocate(Object {
            child: Some(input_handle),
        })
        .expect("owner built from live input")
        .root;
    assert_eq!(
        heap.get(owner.handle()).expect("owner").child,
        Some(input_handle)
    );
    assert!(heap.get(input_handle).is_ok());
    assert!(matches!(
        heap.get(canary_handle),
        Err(AccessError::StaleHandle)
    ));

    drop(owner);
    assert_eq!(
        heap.collect()
            .expect("unrooted input and owner")
            .reclaimed(),
        2
    );
}

#[test]
fn replacement_with_a_foreign_edge_leaves_the_record_unchanged() {
    let mut heap = Heap::new();
    let mut other_heap = Heap::new();
    let local = heap.allocate(Object { child: None }).expect("local").root;
    let foreign = other_heap
        .allocate(Object { child: None })
        .expect("foreign")
        .root;

    let foreign_handle = foreign.handle();
    let rejected = heap
        .replace(
            local.handle(),
            Object {
                child: Some(foreign_handle),
            },
        )
        .expect_err("foreign edge");
    assert_eq!(rejected.reason, AccessError::ForeignHeap);
    assert_eq!(rejected.value.child, Some(foreign_handle));
    assert!(heap.get(local.handle()).expect("local").child.is_none());
    assert_eq!(heap.collect().expect("unchanged graph").reclaimed(), 0);
}

#[test]
fn collection_with_an_invalid_edge_does_not_reclaim_other_slots() {
    let mut heap = Heap::new();
    let mut other_heap = Heap::new();
    let rooted = heap
        .allocate(MutableEdges(Rc::new(Cell::new(None))))
        .expect("root")
        .root;
    let canary = heap.allocate(Other).expect("canary").root;
    let foreign = other_heap.allocate(Other).expect("foreign").root;
    let canary_handle = canary.handle();
    drop(canary);
    heap.get(rooted.handle())
        .expect("root")
        .0
        .set(Some(foreign.handle().erase()));

    assert!(matches!(heap.collect(), Err(AccessError::ForeignHeap)));
    assert!(heap.get(canary_handle).is_ok());

    heap.get(rooted.handle()).expect("root").0.set(None);
    assert_eq!(heap.collect().expect("valid graph").reclaimed(), 1);
}

#[test]
fn allocation_transfers_reclaimed_payloads_before_they_can_mutate_the_input() {
    struct ReplaceOnDrop {
        edge: Rc<Cell<Option<ObjectId>>>,
        target: ObjectId,
    }

    impl Trace for ReplaceOnDrop {
        fn trace(&self, _: &mut EdgeVisitor<'_>) {}
    }

    impl Drop for ReplaceOnDrop {
        fn drop(&mut self) {
            self.edge.set(Some(self.target));
        }
    }

    let mut heap = Heap::new();
    let shared = Rc::new(Cell::new(None));
    let target = heap.allocate(Other).expect("target").root;
    let mutation = heap
        .allocate(ReplaceOnDrop {
            edge: Rc::clone(&shared),
            target: target.handle().erase(),
        })
        .expect("mutation")
        .root;
    drop((target, mutation));

    heap.set_allocation_collection(AllocationCollection::BeforeAllocation);
    let allocation = heap
        .allocate(MutableEdges(Rc::clone(&shared)))
        .expect("pending input");
    assert_eq!(allocation.collection.reclaimed(), 2);
    assert!(shared.get().is_none());
    assert!(heap.get(allocation.root.handle()).is_ok());

    drop(allocation.collection);
    assert!(matches!(heap.collect(), Err(AccessError::StaleHandle)));
}

#[test]
fn replacement_returns_the_displaced_payload_before_it_can_drop() {
    struct Record {
        edge: Rc<Cell<Option<ObjectId>>>,
        edge_on_drop: Option<ObjectId>,
    }

    impl Trace for Record {
        fn trace(&self, edges: &mut EdgeVisitor<'_>) {
            if let Some(child) = self.edge.get() {
                edges.strong(child);
            }
        }
    }

    impl Drop for Record {
        fn drop(&mut self) {
            if let Some(edge) = self.edge_on_drop {
                self.edge.set(Some(edge));
            }
        }
    }

    let mut heap = Heap::new();
    let mut other_heap = Heap::new();
    let foreign = other_heap.allocate(Other).expect("foreign").root;
    let shared = Rc::new(Cell::new(None));
    let root = heap
        .allocate(Record {
            edge: Rc::clone(&shared),
            edge_on_drop: Some(foreign.handle().erase()),
        })
        .expect("old record")
        .root;
    let _displaced = heap
        .replace(
            root.handle(),
            Record {
                edge: Rc::clone(&shared),
                edge_on_drop: None,
            },
        )
        .expect("replacement");
    assert!(shared.get().is_none());
    assert_eq!(heap.collect().expect("valid replacement").reclaimed(), 0);
}

#[test]
fn forced_allocation_finishes_before_a_reclaimed_destructor_changes_a_live_child() {
    struct ChangeOnDrop(Rc<Cell<Option<ObjectId>>>, ObjectId);

    impl Trace for ChangeOnDrop {
        fn trace(&self, _: &mut EdgeVisitor<'_>) {}
    }

    impl Drop for ChangeOnDrop {
        fn drop(&mut self) {
            self.0.set(Some(self.1));
        }
    }

    struct Parent(Handle<MutableEdges>);

    impl Trace for Parent {
        fn trace(&self, edges: &mut EdgeVisitor<'_>) {
            edges.strong(self.0.erase());
        }
    }

    let mut heap = Heap::new();
    let shared = Rc::new(Cell::new(None));
    let child = heap
        .allocate(MutableEdges(Rc::clone(&shared)))
        .expect("child")
        .root;
    let target = heap.allocate(Other).expect("target").root;
    let mutation = heap
        .allocate(ChangeOnDrop(Rc::clone(&shared), target.handle().erase()))
        .expect("mutation")
        .root;
    drop((target, mutation));
    heap.set_allocation_collection(AllocationCollection::BeforeAllocation);

    let parent = heap.allocate(Parent(child.handle())).expect("parent");
    assert_eq!(parent.collection.reclaimed(), 2);
    assert!(shared.get().is_none());

    drop(parent.collection);
    assert!(matches!(heap.collect(), Err(AccessError::StaleHandle)));
}
