//! The test switch that makes every nested subgrid computation repeat instead of being reused,
//! so a test can lay out one document both ways and compare the results.

use std::cell::Cell;

thread_local! {
    static REUSE_DISABLED: Cell<bool> = const { Cell::new(false) };
}

/// Runs `body` with every nested subgrid computation repeated instead of
/// reused, so a test can lay out one document with and without the reuse and
/// compare the results.
pub(crate) fn with_subgrid_reuse_disabled<R>(body: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            REUSE_DISABLED.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(REUSE_DISABLED.with(|cell| cell.replace(true)));
    body()
}

pub(super) fn reuse_disabled() -> bool {
    REUSE_DISABLED.with(Cell::get)
}
