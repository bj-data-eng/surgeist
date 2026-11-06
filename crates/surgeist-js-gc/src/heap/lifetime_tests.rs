use super::{
    AccessError, AllocationCollection, EdgeVisitor, Handle, Heap, ObjectId, RecordRef, RuntimeHeap,
    Trace,
};
use std::{
    cell::{Cell, RefCell},
    num::NonZeroUsize,
    rc::Rc,
};

#[test]
#[cfg_attr(miri, ignore = "million-record graph is too slow under Miri")]
fn million_record_chain_survives_from_its_head_then_is_reclaimed() {
    #[derive(Trace)]
    struct Node(Option<Handle<Self>>);

    const COUNT: usize = 1_000_000;
    let mut heap = Heap::new();
    let mut head = heap.allocate(Node(None)).unwrap().root;
    let tail = head.handle();
    for _ in 1..COUNT {
        head = heap.allocate(Node(Some(head.handle()))).unwrap().root;
    }
    let head_id = head.handle();

    assert_eq!(heap.collect().unwrap().reclaimed(), 0);
    assert!(heap.get(tail).unwrap().0.is_none());
    drop(head);
    assert_eq!(heap.collect().unwrap().reclaimed(), COUNT);
    assert!(heap.upgrade(head_id).unwrap().is_none());
    assert!(heap.upgrade(tail).unwrap().is_none());
    assert_eq!(heap.collect().unwrap().reclaimed(), 0);
}

#[test]
fn erased_chain_preserves_typed_values_and_order_across_collection() {
    struct Node {
        value: i32,
        next: Option<ObjectId>,
    }

    impl Trace for Node {
        fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
            if let Some(next) = self.next {
                edges.strong(next);
            }
            Ok(())
        }
    }

    let mut heap = Heap::new();
    let mut head = heap
        .allocate(Node {
            value: 0,
            next: None,
        })
        .unwrap()
        .root;
    for value in 1..100 {
        head = heap
            .allocate(Node {
                value,
                next: Some(head.handle().erase()),
            })
            .unwrap()
            .root;
    }
    let head_id = head.handle().erase();
    assert_eq!(heap.downcast::<i32>(head_id), Err(AccessError::WrongType));
    assert_eq!(heap.collect().unwrap().reclaimed(), 0);

    let mut next = Some(head_id);
    for expected in (0..100).rev() {
        let node = heap
            .get(heap.downcast::<Node>(next.expect("another node")).unwrap())
            .unwrap();
        assert_eq!(node.value, expected);
        next = node.next;
    }
    assert!(next.is_none());

    drop(head);
    assert_eq!(heap.collect().unwrap().reclaimed(), 100);
    assert_eq!(
        heap.downcast::<Node>(head_id),
        Err(AccessError::StaleHandle)
    );
}

#[test]
fn a_mutable_list_keeps_all_children_across_repeated_allocations() {
    #[derive(Trace)]
    struct Bytes(#[heap_trace(skip)] [u8; 10]);

    let runtime = RuntimeHeap::default();
    runtime
        .set_allocation_collection(AllocationCollection::Opportunistic(
            NonZeroUsize::new(32).unwrap(),
        ))
        .unwrap();
    let parent = runtime
        .allocate(RefCell::new(Vec::<RecordRef<Bytes>>::new()))
        .unwrap();
    for _ in 0..259 {
        let child = runtime.allocate(Bytes([0; 10])).unwrap();
        parent.borrow_mut().push(child.record_ref());
    }

    assert_eq!(runtime.collect().unwrap(), 0);
    let children = parent.borrow();
    assert_eq!(children.len(), 259);
    for child in children.iter() {
        assert_eq!(child.upgrade().unwrap().unwrap().0, [0; 10]);
    }
    drop(children);
    drop(parent);
    assert_eq!(runtime.collect().unwrap(), 260);
}

#[test]
fn clearing_a_mutably_borrowed_edge_releases_its_child_after_collection_retries() {
    let runtime = RuntimeHeap::default();
    let child = runtime.allocate(17_u32).unwrap();
    let weak_child = child.record_ref();
    let parent = runtime
        .allocate(RefCell::new(Some(weak_child.clone())))
        .unwrap();
    drop(child);

    let mut write = parent.borrow_mut();
    assert!(parent.try_borrow().is_err());
    assert_eq!(runtime.collect(), Err(AccessError::PayloadBorrowed));
    assert_eq!(*weak_child.upgrade().unwrap().unwrap(), 17);
    *write = None;
    drop(write);

    assert_eq!(runtime.collect().unwrap(), 1);
    assert!(weak_child.upgrade().unwrap().is_none());
    assert!(parent.borrow().is_none());
}

struct ConditionalEdges(Vec<(ObjectId, ObjectId)>);

impl Trace for ConditionalEdges {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        for &(key, value) in &self.0 {
            edges.ephemeron(key, value);
        }
        Ok(())
    }
}

#[test]
fn a_weak_wrapper_remains_a_live_ephemeron_key_after_its_target_dies() {
    #[derive(Trace)]
    struct WeakTarget(#[heap_trace(skip)] RecordRef<u32>);

    let runtime = RuntimeHeap::default();
    let target = runtime.allocate(23_u32).unwrap();
    let weak = target.record_ref();
    let wrapper = runtime.allocate(WeakTarget(weak.clone())).unwrap();
    let value = runtime.allocate(3_u32).unwrap().record_ref();
    let owner = runtime
        .allocate(ConditionalEdges(vec![(
            wrapper.record_ref().identity(),
            value.identity(),
        )]))
        .unwrap();

    let upgraded = weak.upgrade().unwrap().unwrap();
    let weak_clone = weak.clone();
    drop((weak, target));
    assert_eq!(runtime.collect().unwrap(), 0);
    assert_eq!(upgraded.record_ref(), weak_clone);
    assert_eq!(*wrapper.0.upgrade().unwrap().unwrap(), 23);
    assert_eq!(*value.upgrade().unwrap().unwrap(), 3);

    drop(upgraded);
    assert_eq!(runtime.collect().unwrap(), 1);
    assert!(weak_clone.upgrade().unwrap().is_none());
    assert!(wrapper.0.upgrade().unwrap().is_none());
    assert_eq!(*value.upgrade().unwrap().unwrap(), 3);

    drop(wrapper);
    assert_eq!(runtime.collect().unwrap(), 2);
    assert!(value.upgrade().unwrap().is_none());
    assert!(owner.record_ref().is_live().unwrap());
}

#[test]
fn removing_the_only_live_owner_edge_reclaims_a_conditional_cycle() {
    let mut heap = Heap::new();
    let key = heap.allocate(0_u32).unwrap().root;
    let owner = heap.allocate(ConditionalEdges(Vec::new())).unwrap().root;
    let first = heap.allocate(ConditionalEdges(Vec::new())).unwrap().root;
    let second = heap.allocate(ConditionalEdges(Vec::new())).unwrap().root;
    let key_id = key.handle().erase();
    let first_id = first.handle();
    let second_id = second.handle();
    for (from, to) in [
        (owner.handle(), first_id),
        (first_id, second_id),
        (second_id, first_id),
    ] {
        heap.replace(from, ConditionalEdges(vec![(key_id, to.erase())]))
            .unwrap();
    }
    drop((first, second));
    assert_eq!(heap.collect().unwrap().reclaimed(), 0);
    assert!(heap.get(first_id).is_ok());
    assert!(heap.get(second_id).is_ok());

    heap.replace(owner.handle(), ConditionalEdges(Vec::new()))
        .unwrap();
    assert_eq!(heap.collect().unwrap().reclaimed(), 2);
    assert!(heap.upgrade(first_id).unwrap().is_none());
    assert!(heap.upgrade(second_id).unwrap().is_none());
    assert!(heap.get(owner.handle()).is_ok());
    assert!(heap.get(key.handle()).is_ok());
}

#[test]
fn self_edges_do_not_keep_an_unrooted_record_alive() {
    struct Node {
        strong: [ObjectId; 2],
        conditional: Option<(ObjectId, ObjectId)>,
    }

    impl Trace for Node {
        fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
            for id in self.strong {
                edges.strong(id);
            }
            if let Some((key, value)) = self.conditional {
                edges.ephemeron(key, value);
            }
            Ok(())
        }
    }

    let mut heap = Heap::new();
    let key = heap.allocate(0_u32).unwrap().root;
    let key_id = key.handle().erase();
    let owner = heap
        .allocate(Node {
            strong: [key_id; 2],
            conditional: None,
        })
        .unwrap()
        .root;
    let owner_id = owner.handle();
    for conditional in [
        Some((owner_id.erase(), owner_id.erase())),
        Some((key_id, owner_id.erase())),
        None,
    ] {
        heap.replace(
            owner_id,
            Node {
                strong: [owner_id.erase(); 2],
                conditional,
            },
        )
        .unwrap();
        assert_eq!(heap.collect().unwrap().reclaimed(), 0);
        assert!(heap.get(owner_id).is_ok());
    }
    drop(owner);
    assert_eq!(heap.collect().unwrap().reclaimed(), 1);
    assert!(heap.upgrade(owner_id).unwrap().is_none());
    assert!(heap.get(key.handle()).is_ok());
}

#[derive(Trace)]
struct CountDrops(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for CountDrops {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn ephemeron_owners_retire_independently_and_dispose_values_once() {
    let mut heap = Heap::new();
    let left_key = heap.allocate(1_u32).unwrap().root;
    let cloned_key = left_key.clone();
    let right_key = heap.allocate(2_u32).unwrap().root;
    let left_drops = Rc::new(Cell::new(0));
    let right_drops = Rc::new(Cell::new(0));
    let left_value = heap.allocate(CountDrops(left_drops.clone())).unwrap().root;
    let right_value = heap.allocate(CountDrops(right_drops.clone())).unwrap().root;
    let left_id = left_value.handle();
    let right_id = right_value.handle();
    let left_entry = (left_key.handle().erase(), left_id.erase());
    let right_entry = (right_key.handle().erase(), right_id.erase());
    let left = heap
        .allocate(ConditionalEdges(vec![left_entry]))
        .unwrap()
        .root;
    let right = heap
        .allocate(ConditionalEdges(vec![right_entry]))
        .unwrap()
        .root;
    drop((left_key, left_value, right_value));

    heap.replace(left.handle(), ConditionalEdges(Vec::new()))
        .unwrap();
    heap.replace(left.handle(), ConditionalEdges(vec![left_entry]))
        .unwrap();
    assert_eq!(heap.collect().unwrap().reclaimed(), 0);
    assert!(heap.get(left_id).is_ok());
    assert!(heap.get(right_id).is_ok());
    assert_eq!((left_drops.get(), right_drops.get()), (0, 0));

    drop(cloned_key);
    let retired = heap.collect().unwrap();
    assert_eq!(retired.reclaimed(), 2);
    assert!(heap.upgrade(left_id).unwrap().is_none());
    assert!(heap.get(left.handle()).is_ok());
    assert!(heap.get(right_id).is_ok());
    assert_eq!((left_drops.get(), right_drops.get()), (0, 0));
    drop(retired);
    assert_eq!((left_drops.get(), right_drops.get()), (1, 0));

    let left_owner_id = left.handle();
    drop(left);
    assert_eq!(heap.collect().unwrap().reclaimed(), 1);
    assert!(heap.upgrade(left_owner_id).unwrap().is_none());
    assert!(heap.get(right.handle()).is_ok());
    assert!(heap.get(right_id).is_ok());
    assert_eq!((left_drops.get(), right_drops.get()), (1, 0));

    let right_owner_id = right.handle();
    drop(right);
    assert_eq!(heap.collect().unwrap().reclaimed(), 2);
    assert!(heap.upgrade(right_owner_id).unwrap().is_none());
    assert!(heap.upgrade(right_id).unwrap().is_none());
    assert!(heap.get(right_key.handle()).is_ok());
    assert_eq!((left_drops.get(), right_drops.get()), (1, 1));
    assert_eq!(heap.collect().unwrap().reclaimed(), 0);
    assert_eq!((left_drops.get(), right_drops.get()), (1, 1));
}

#[test]
fn heap_teardown_disposes_remaining_payloads() {
    let drops = Rc::new(Cell::new(0));
    {
        let mut heap = Heap::new();
        let key = heap.allocate(1_u32).unwrap().root;
        let value = heap.allocate(CountDrops(drops.clone())).unwrap().root;
        let owner = heap
            .allocate(ConditionalEdges(vec![(
                key.handle().erase(),
                value.handle().erase(),
            )]))
            .unwrap()
            .root;
        drop(value);
        assert_eq!(heap.collect().unwrap().reclaimed(), 0);
        assert_eq!(drops.get(), 0);
        drop((key, owner));
    }
    assert_eq!(drops.get(), 1);
}
