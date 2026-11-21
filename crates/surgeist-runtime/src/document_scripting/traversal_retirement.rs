//! Defers native cursor retirement until the originating DOM can be borrowed.

use super::*;
use surgeist_dom::{
    Dom,
    tree::{DomNodeIteratorId, DomTreeWalkerId},
};

#[derive(Clone, Copy)]
pub(super) enum CursorId {
    Iterator(DomNodeIteratorId),
    Walker(DomTreeWalkerId),
}

impl CursorId {
    fn retire(self, dom: &mut Dom, cleanup: &AtomicBool) -> Result<(), DomError> {
        match self {
            Self::Iterator(id) => match dom.retire_node_iterator(id, cleanup) {
                Err(DomError::StaleNodeIterator) => Ok(()),
                result => result,
            },
            Self::Walker(id) => match dom.retire_tree_walker(id, cleanup) {
                Err(DomError::StaleTreeWalker) => Ok(()),
                result => result,
            },
        }
    }
}

#[derive(Default)]
pub(super) struct Retirements {
    head: Cell<Option<Box<Record>>>,
}

pub(super) struct Record {
    id: Option<CursorId>,
    next: Option<Box<Record>>,
    _backing: guest::GuestBacking,
}

impl Record {
    pub(super) fn new(backing: guest::GuestBacking) -> Box<Self> {
        Box::new(Self {
            id: None,
            next: None,
            _backing: backing,
        })
    }
}

impl Retirements {
    pub(super) fn enqueue(&self, id: CursorId, mut record: Box<Record>) {
        record.id = Some(id);
        record.next = self.head.take();
        self.head.set(Some(record));
    }

    pub(super) fn reap(&self, dom: &mut Dom) -> Result<(), DomError> {
        // Cursor retirement is a mandatory cleanup, including after a cancelled
        // or refused script entry. A busy native owner retains the current record.
        let cleanup = AtomicBool::new(false);
        while let Some(mut record) = self.head.take() {
            if let Some(id) = record.id
                && let Err(error) = id.retire(dom, &cleanup)
            {
                self.head.set(Some(record));
                return Err(error);
            }
            self.head.set(record.next.take());
        }
        Ok(())
    }
}

impl Drop for Retirements {
    fn drop(&mut self) {
        let head = self.head.get_mut();
        while let Some(mut record) = head.take() {
            *head = record.next.take();
        }
    }
}
