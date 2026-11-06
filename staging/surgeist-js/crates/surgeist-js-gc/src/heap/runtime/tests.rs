use super::*;
use crate::heap::{EdgeVisitor, ObjectId};
use std::{cell::Cell, rc::Weak};

struct Record(RefCell<Option<ObjectId>>);

impl Trace for Record {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        if let Some(id) = *self
            .0
            .try_borrow()
            .map_err(|_| AccessError::PayloadBorrowed)?
        {
            edges.strong(id);
        }
        Ok(())
    }
}

#[test]
fn independently_borrowed_records_share_one_heap() {
    let runtime = RuntimeHeap::default();
    let peer = runtime.clone();
    let left = runtime.allocate(Record(RefCell::new(None))).expect("left");
    let right = peer.allocate(Record(RefCell::new(None))).expect("right");

    let mut write = left.0.borrow_mut();
    assert!(right.0.borrow().is_none());
    assert!(matches!(
        runtime.collect(),
        Err(AccessError::PayloadBorrowed)
    ));
    *write = Some(right.handle().erase());
    drop(write);

    let right_id = right.handle();
    assert_eq!(peer.downcast::<Record>(right_id.erase()), Ok(right_id));
    assert_eq!(
        peer.downcast::<ReentrantDisposal>(right_id.erase()),
        Err(AccessError::WrongType)
    );
    drop(right);
    assert_eq!(runtime.collect().expect("trace left to right"), 0);
    assert!(peer.upgrade(right_id).expect("same heap").is_some());
    assert!(matches!(
        RuntimeHeap::default().upgrade(right_id),
        Err(AccessError::ForeignHeap)
    ));

    drop(left);
    assert_eq!(runtime.collect().expect("reclaim both records"), 2);
    assert!(peer.upgrade(right_id).expect("same heap").is_none());
}

#[test]
fn runtime_borrow_refusal_returns_input_and_roots_retain_the_runtime() {
    let runtime = RuntimeHeap::default();
    let anchor = runtime
        .allocate(Record(RefCell::new(None)))
        .expect("anchor");
    let id = anchor.handle().erase();
    let active_heap = runtime.0.borrow_mut();
    let rejected = runtime
        .allocate(Record(RefCell::new(Some(id))))
        .expect_err("heap access is active");
    assert_eq!(rejected.reason, AccessError::HeapBorrowed);
    assert_eq!(*rejected.value.0.borrow(), Some(id));
    drop(active_heap);

    let child = runtime.allocate(rejected.value).expect("retry allocation");
    drop(runtime);
    drop(anchor);
    assert_eq!(
        child.runtime().collect().expect("child keeps its runtime"),
        0
    );
    let handle = child
        .runtime()
        .downcast::<Record>(id)
        .expect("child keeps anchor");
    assert!(
        child
            .runtime()
            .upgrade(handle)
            .expect("same runtime")
            .is_some()
    );
}

struct ReentrantDisposal {
    runtime: Weak<RefCell<Heap>>,
    reentered: Rc<Cell<bool>>,
}

impl Trace for ReentrantDisposal {
    fn trace(&self, _: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        Ok(())
    }
}

impl Drop for ReentrantDisposal {
    fn drop(&mut self) {
        if let Some(heap) = self.runtime.upgrade() {
            self.reentered
                .set(matches!(RuntimeHeap(heap).collect(), Ok(0)));
        }
    }
}

#[test]
fn allocation_disposes_after_borrow_and_keeps_its_new_root_during_reentry() {
    let runtime = RuntimeHeap::default();
    let reentered = Rc::new(Cell::new(false));
    let canary = runtime
        .allocate(ReentrantDisposal {
            runtime: Rc::downgrade(&runtime.0),
            reentered: Rc::clone(&reentered),
        })
        .expect("canary");
    let canary_id = canary.handle();
    drop(canary);

    runtime
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .expect("enable forced collection");
    let fresh = runtime
        .allocate(Record(RefCell::new(None)))
        .expect("new root");
    assert!(reentered.get());
    assert!(runtime.upgrade(canary_id).expect("check canary").is_none());
    assert!(
        runtime
            .upgrade(fresh.handle())
            .expect("check new root")
            .is_some()
    );
}
