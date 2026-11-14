//! Owner-held cleanup records, without retaining their result-storage custody.

use std::cell::Cell;
use std::rc::{Rc, Weak};

pub(crate) trait CleanupCustody: 'static {}

pub(crate) trait CleanupRecord: std::fmt::Debug {
    fn owner(&self) -> u64;
    fn error(&self) -> Option<&(dyn std::error::Error + 'static)>;
    fn stage(&self) -> Option<super::CleanupStage>;
}

struct Node {
    next: Option<Box<Node>>,
    custody: Weak<dyn CleanupCustody>,
    _record: Rc<dyn CleanupRecord>,
}

/// Each node's body is admitted before registration. Records can retain their
/// original errors; the weak custody link never retains a converted result.
#[derive(Default)]
pub(crate) struct Registrations {
    head: Cell<Option<Box<Node>>>,
}

impl Registrations {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn registration_bytes() -> usize {
        size_of::<Node>()
    }

    pub(crate) fn register(
        &self,
        custody: Weak<dyn CleanupCustody>,
        record: Rc<dyn CleanupRecord>,
    ) {
        let node = Box::new(Node {
            next: self.head.take(),
            custody,
            _record: record,
        });
        self.head.set(Some(node));
    }

    /// Visit only the entry snapshot. Retired records drop with no registry
    /// loan; destructors may register records or run another reap.
    pub(crate) fn reap(&self) {
        let mut snapshot = Reaping {
            registrations: self,
            pending: Nodes {
                head: self.head.take(),
            },
            live: Nodes::default(),
        };
        while let Some(node) = snapshot.pending.pop() {
            if node.custody.strong_count() == 0 {
                drop(node);
            } else {
                snapshot.live.push(node);
            }
        }
    }
}

impl Drop for Registrations {
    fn drop(&mut self) {
        // Nodes drains iteratively, including when the first record panics.
        drop(Nodes {
            head: self.head.get_mut().take(),
        });
    }
}

#[derive(Default)]
struct Nodes {
    head: Option<Box<Node>>,
}

impl Nodes {
    fn pop(&mut self) -> Option<Box<Node>> {
        let mut node = self.head.take()?;
        self.head = node.next.take();
        Some(node)
    }

    fn push(&mut self, mut node: Box<Node>) {
        node.next = self.head.take();
        self.head = Some(node);
    }
}

impl Drop for Nodes {
    fn drop(&mut self) {
        if self.head.is_none() {
            return;
        }
        // This local owns the unvisited siblings during arbitrary record Drop.
        // Its destructor finishes the drain if one record starts unwinding.
        let mut remaining = Self {
            head: self.head.take(),
        };
        while let Some(node) = remaining.pop() {
            drop(node);
        }
    }
}

struct Reaping<'a> {
    registrations: &'a Registrations,
    pending: Nodes,
    live: Nodes,
}

impl Drop for Reaping<'_> {
    fn drop(&mut self) {
        // Visited live nodes were reversed as they were selected. Restore their
        // original order ahead of any nodes not reached before an unwind.
        while let Some(node) = self.live.pop() {
            self.pending.push(node);
        }
        // New/reentrantly retained registrations remain ahead of the old
        // snapshot. Relinking moves ownership without dropping arbitrary data.
        let mut current = self.registrations.head.take();
        let mut tail = &mut current;
        while let Some(node) = tail {
            tail = &mut node.next;
        }
        *tail = self.pending.head.take();
        self.registrations.head.set(current);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    struct Custody;
    impl CleanupCustody for Custody {}

    struct Record {
        drops: Rc<Cell<usize>>,
        action: Option<Box<dyn FnOnce()>>,
        error: Option<std::io::Error>,
    }

    impl std::fmt::Debug for Record {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("Record").finish_non_exhaustive()
        }
    }

    impl CleanupRecord for Record {
        fn owner(&self) -> u64 {
            7
        }

        fn error(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.error
                .as_ref()
                .map(|error| error as &dyn std::error::Error)
        }

        fn stage(&self) -> Option<super::super::CleanupStage> {
            self.error
                .as_ref()
                .map(|_| super::super::CleanupStage::StorageRelease)
        }
    }

    impl Drop for Record {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
            if let Some(action) = self.action.take() {
                action();
            }
        }
    }

    fn custody() -> Rc<dyn CleanupCustody> {
        Rc::new(Custody)
    }

    fn expired() -> Weak<dyn CleanupCustody> {
        Rc::downgrade(&custody())
    }

    fn record(drops: &Rc<Cell<usize>>) -> Rc<dyn CleanupRecord> {
        Rc::new(Record {
            drops: drops.clone(),
            action: None,
            error: None,
        })
    }

    fn active_record(
        drops: &Rc<Cell<usize>>,
        action: impl FnOnce() + 'static,
    ) -> Rc<dyn CleanupRecord> {
        Rc::new(Record {
            drops: drops.clone(),
            action: Some(Box::new(action)),
            error: None,
        })
    }

    #[test]
    fn uninitialized_records_live_until_custody_expires_and_release_once() {
        let registrations = Registrations::new();
        let custody = custody();
        let drops = Rc::new(Cell::new(0));
        let record = record(&drops);
        assert_eq!(record.owner(), 7);
        assert!(record.error().is_none());
        assert!(record.stage().is_none());
        registrations.register(Rc::downgrade(&custody), record);
        registrations.reap();
        assert_eq!(drops.get(), 0);
        drop(custody);
        registrations.reap();
        assert_eq!(drops.get(), 1);
        registrations.reap();
        drop(registrations);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn a_filled_record_preserves_its_original_source_and_owner() {
        let registrations = Registrations::new();
        let custody = custody();
        let drops = Rc::new(Cell::new(0));
        let record: Rc<dyn CleanupRecord> = Rc::new(Record {
            drops: drops.clone(),
            action: None,
            error: Some(std::io::Error::other("original cleanup source")),
        });
        let weak = Rc::downgrade(&record);
        registrations.register(Rc::downgrade(&custody), record);
        registrations.reap();
        let observed = weak.upgrade().unwrap();
        assert_eq!(observed.owner(), 7);
        assert_eq!(
            observed.stage(),
            Some(super::super::CleanupStage::StorageRelease)
        );
        assert_eq!(
            observed.error().unwrap().to_string(),
            "original cleanup source"
        );
        drop(observed);
        drop(custody);
        registrations.reap();
        assert!(weak.upgrade().is_none());
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn registrations_created_during_record_drop_wait_for_a_later_reap() {
        let registrations = Rc::new(Registrations::new());
        let target = Rc::downgrade(&registrations);
        let parent_drops = Rc::new(Cell::new(0));
        let child_drops = Rc::new(Cell::new(0));
        let child = child_drops.clone();
        registrations.register(
            expired(),
            active_record(&parent_drops, move || {
                target
                    .upgrade()
                    .unwrap()
                    .register(expired(), record(&child));
            }),
        );
        registrations.reap();
        assert_eq!(parent_drops.get(), 1);
        assert_eq!(child_drops.get(), 0);
        registrations.reap();
        assert_eq!(child_drops.get(), 1);
    }

    #[test]
    fn nested_reap_visits_its_own_snapshot_and_preserves_new_registrations() {
        let registrations = Rc::new(Registrations::new());
        let target = Rc::downgrade(&registrations);
        let parent_drops = Rc::new(Cell::new(0));
        let child_drops = Rc::new(Cell::new(0));
        let later_drops = Rc::new(Cell::new(0));
        let child = child_drops.clone();
        let later = later_drops.clone();
        registrations.register(
            expired(),
            active_record(&parent_drops, move || {
                let registrations = target.upgrade().unwrap();
                let child_target = Rc::downgrade(&registrations);
                registrations.register(
                    expired(),
                    active_record(&child, move || {
                        child_target
                            .upgrade()
                            .unwrap()
                            .register(expired(), record(&later));
                    }),
                );
                registrations.reap();
            }),
        );
        registrations.reap();
        assert_eq!(parent_drops.get(), 1);
        assert_eq!(child_drops.get(), 1);
        assert_eq!(later_drops.get(), 0);
        registrations.reap();
        assert_eq!(later_drops.get(), 1);
    }

    #[test]
    fn retired_record_panic_restores_live_unvisited_and_new_registrations() {
        let registrations = Rc::new(Registrations::new());
        let custody = custody();
        let visited_live = Rc::new(Cell::new(0));
        let unvisited = Rc::new(Cell::new(0));
        let panicked = Rc::new(Cell::new(0));
        let added = Rc::new(Cell::new(0));
        registrations.register(expired(), record(&unvisited));
        let target = Rc::downgrade(&registrations);
        let added_record = added.clone();
        registrations.register(
            expired(),
            active_record(&panicked, move || {
                target
                    .upgrade()
                    .unwrap()
                    .register(expired(), record(&added_record));
                panic!("retired cleanup panic");
            }),
        );
        registrations.register(Rc::downgrade(&custody), record(&visited_live));
        let panic = catch_unwind(AssertUnwindSafe(|| registrations.reap())).unwrap_err();
        assert_eq!(panic.downcast_ref::<&str>(), Some(&"retired cleanup panic"));
        assert_eq!(visited_live.get(), 0);
        assert_eq!(unvisited.get(), 0);
        assert_eq!(panicked.get(), 1);
        assert_eq!(added.get(), 0);
        registrations.reap();
        assert_eq!(visited_live.get(), 0);
        assert_eq!(unvisited.get(), 1);
        assert_eq!(added.get(), 1);
        drop(custody);
        registrations.reap();
        assert_eq!(visited_live.get(), 1);
        assert_eq!(panicked.get(), 1);
    }

    #[test]
    fn final_drop_drains_a_long_chain_on_a_small_stack() {
        let released = std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let registrations = Registrations::new();
                let drops = Rc::new(Cell::new(0));
                for _ in 0..100_000 {
                    registrations.register(expired(), record(&drops));
                }
                drop(registrations);
                drops.get()
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(released, 100_000);
    }

    #[test]
    fn final_drop_releases_siblings_after_the_first_record_panics() {
        let registrations = Registrations::new();
        let drops = Rc::new(Cell::new(0));
        registrations.register(expired(), record(&drops));
        registrations.register(expired(), record(&drops));
        registrations.register(
            expired(),
            active_record(&drops, || panic!("first drop panic")),
        );
        let panic = catch_unwind(AssertUnwindSafe(|| drop(registrations))).unwrap_err();
        assert_eq!(panic.downcast_ref::<&str>(), Some(&"first drop panic"));
        assert_eq!(drops.get(), 3);
    }
}
