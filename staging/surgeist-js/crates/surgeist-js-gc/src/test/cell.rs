mod miri {
    use super::super::run_test;
    use crate::{Gc, GcRefCell, WeakGc, force_collect};

    #[test]
    fn boa_borrow_mut_test() {
        run_test(|| {
            let v = Gc::new(GcRefCell::new(Vec::new()));

            for _ in 1..=259 {
                let cell = Gc::new(GcRefCell::new([0u8; 10]));
                v.borrow_mut().push(cell);
            }
        });
    }

    #[test]
    fn collection_preserves_edges_during_a_mutable_borrow() {
        run_test(|| {
            let child = Gc::new(17u32);
            let weak_child = WeakGc::new(&child);
            let dead = Gc::new(23u32);
            let weak_dead = WeakGc::new(&dead);
            drop(dead);
            let parent = Gc::new(GcRefCell::new(Some(child)));

            let mut write = parent.borrow_mut();
            assert!(parent.try_borrow().is_err());
            force_collect();
            assert!(weak_dead.upgrade().is_none());
            assert_eq!(*weak_child.upgrade().unwrap(), 17);
            *write = None;
            drop(write);

            force_collect();
            assert!(weak_child.upgrade().is_none());
        });
    }
}
