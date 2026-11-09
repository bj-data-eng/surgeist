use super::UnwindCleanup;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    rc::Rc,
};

#[derive(Debug)]
struct FirstPanic;
#[derive(Debug)]
struct LaterPanic;
#[derive(Debug)]
struct OuterPanic;

struct PanickingDrop {
    drops: Rc<Cell<usize>>,
    first: bool,
}
impl Drop for PanickingDrop {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        if self.first {
            panic_any(FirstPanic);
        } else {
            panic_any(LaterPanic);
        }
    }
}

#[test]
fn returned_value_cleanup_keeps_the_initiating_panic_and_finishes_both_drops() {
    let first_drops = Rc::new(Cell::new(0));
    let later_drops = Rc::new(Cell::new(0));
    let remaining_cleanup = Rc::new(Cell::new(false));
    let observed = catch_unwind(AssertUnwindSafe(|| {
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            drop(PanickingDrop {
                drops: first_drops.clone(),
                first: true,
            })
        });
        cleanup.run(|| remaining_cleanup.set(true));
        let returned = cleanup.finish_with(PanickingDrop {
            drops: later_drops.clone(),
            first: false,
        });
        drop(returned);
    }))
    .expect_err("the initiating destructor must still unwind");
    assert!(observed.is::<FirstPanic>());
    assert_eq!(first_drops.get(), 1);
    assert_eq!(later_drops.get(), 1);
    assert!(remaining_cleanup.get());
}

struct DuringOuterUnwind {
    secondary_drops: Rc<Cell<usize>>,
    returned: Rc<Cell<bool>>,
}
impl Drop for DuringOuterUnwind {
    fn drop(&mut self) {
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            drop(PanickingDrop {
                drops: self.secondary_drops.clone(),
                first: false,
            })
        });
        let value = cleanup.finish_with(29);
        self.returned.set(value == 29);
    }
}

#[test]
fn existing_outer_unwind_keeps_its_payload_and_the_result_remains_owned() {
    let secondary_drops = Rc::new(Cell::new(0));
    let returned = Rc::new(Cell::new(false));
    let observed = catch_unwind(AssertUnwindSafe(|| {
        let _guard = DuringOuterUnwind {
            secondary_drops: secondary_drops.clone(),
            returned: returned.clone(),
        };
        panic_any(OuterPanic);
    }))
    .expect_err("the original outer unwind must remain observable");
    assert!(observed.is::<OuterPanic>());
    assert_eq!(secondary_drops.get(), 1);
    assert!(returned.get());
}
