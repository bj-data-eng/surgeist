//! Defers native cursor retirement until the originating DOM can be borrowed.

use super::*;
use surgeist_dom::{Dom, tree::DomNodeIteratorId};

#[derive(Default)]
pub(super) struct IteratorRetirements {
    head: Cell<Option<Box<Record>>>,
}

pub(super) struct Record {
    id: Option<DomNodeIteratorId>,
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

impl IteratorRetirements {
    pub(super) fn enqueue(&self, id: DomNodeIteratorId, mut record: Box<Record>) {
        record.id = Some(id);
        record.next = self.head.take();
        self.head.set(Some(record));
    }

    pub(super) fn reap(&self, dom: &mut Dom) -> Result<(), DomError> {
        // Cursor retirement is a mandatory cleanup, including after a cancelled
        // or refused script entry. A busy native owner retains the current record.
        let cleanup = AtomicBool::new(false);
        while let Some(mut record) = self.head.take() {
            if let Some(id) = record.id {
                match dom.retire_node_iterator(id, &cleanup) {
                    Ok(()) | Err(DomError::StaleNodeIterator) => {}
                    Err(error) => {
                        self.head.set(Some(record));
                        return Err(error);
                    }
                }
            }
            self.head.set(record.next.take());
        }
        Ok(())
    }
}

impl Drop for IteratorRetirements {
    fn drop(&mut self) {
        let head = self.head.get_mut();
        while let Some(mut record) = head.take() {
            *head = record.next.take();
        }
    }
}
