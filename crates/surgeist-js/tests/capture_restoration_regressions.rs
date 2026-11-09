use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

use surgeist_js::native_function::NativeCapture;
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Rooted, RuntimeHeap, Trace};

#[derive(Debug)]
struct Payload {
    value: u64,
    drops: Rc<Cell<usize>>,
}

impl Trace for Payload {
    fn trace(&self, _: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        Ok(())
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn restoring_an_expired_native_capture_does_not_unwind_rust() {
    let heap = RuntimeHeap::default();
    let drops = Rc::new(Cell::new(0));
    let original = heap
        .allocate(Payload {
            value: 41,
            drops: drops.clone(),
        })
        .unwrap();
    let identity = original.handle().erase();
    let stored = <Rooted<Payload> as NativeCapture>::store(&original);
    assert_eq!(stored.identity(), identity);
    drop(original);

    assert_eq!(heap.collect().unwrap(), 1);
    assert_eq!(drops.get(), 1, "the original payload actually expired");
    assert!(stored.upgrade().unwrap().is_none());

    // Deliberately present an expired dynamic capability to the public front.
    let attempt = catch_unwind(AssertUnwindSafe(|| {
        <Rooted<Payload> as NativeCapture>::restore(&stored)
    }));
    assert!(
        attempt.is_ok(),
        "expired capture restoration must refuse without a Rust unwind"
    );
    assert!(matches!(
        attempt.unwrap(),
        Err(surgeist_js::error::RootingError::Expired)
    ));
    assert_eq!(heap.collect().unwrap(), 0);
    assert_eq!(drops.get(), 1);
    drop(stored);
    drop(heap);
    assert_eq!(drops.get(), 1);
}

#[test]
fn restoring_a_live_native_capture_preserves_identity_and_root_custody() {
    let heap = RuntimeHeap::default();
    let drops = Rc::new(Cell::new(0));
    let original = heap
        .allocate(Payload {
            value: 73,
            drops: drops.clone(),
        })
        .unwrap();
    let identity = original.handle().erase();
    let stored = <Rooted<Payload> as NativeCapture>::store(&original);
    let restored = <Rooted<Payload> as NativeCapture>::restore(&stored).unwrap();
    assert_eq!(restored.handle().erase(), identity);
    assert_eq!(restored.value, 73);
    drop(original);

    assert_eq!(heap.collect().unwrap(), 0);
    assert_eq!(drops.get(), 0);
    assert_eq!(restored.handle().erase(), identity);
    assert_eq!(restored.value, 73);
    drop(restored);

    assert_eq!(heap.collect().unwrap(), 1);
    assert_eq!(drops.get(), 1);
    assert!(stored.upgrade().unwrap().is_none());
    drop(stored);
    drop(heap);
    assert_eq!(drops.get(), 1);
}
