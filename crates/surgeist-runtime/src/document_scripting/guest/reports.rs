use std::{any::Any, collections::VecDeque};

use super::*;

pub(super) type DispatchFailure =
    HostFailureSlot<DocumentGuestEventError, DomHostedRuntimeRetention>;

pub(in crate::document_scripting) struct GuestExceptionReports {
    queue: RefCell<VecDeque<DocumentGuestListenerException>>,
    capacity: usize,
    reserved: Cell<usize>,
    _backing: DomHostedRuntimeRetention,
}

impl GuestExceptionReports {
    pub(super) fn storage_bytes(capacity: usize) -> Option<usize> {
        capacity
            .checked_mul(size_of::<DocumentGuestListenerException>())?
            .checked_add(size_of::<Self>())
    }

    pub(super) fn new(capacity: usize, backing: DomHostedRuntimeRetention) -> Result<Self, Error> {
        let mut queue = VecDeque::new();
        queue
            .try_reserve_exact(capacity)
            .map_err(|_| Error::Allocation)?;
        if queue.capacity() != capacity {
            return Err(Error::Allocation);
        }
        Ok(Self {
            queue: RefCell::new(queue),
            capacity,
            reserved: Cell::new(0),
            _backing: backing,
        })
    }

    pub(super) fn take(&self) -> Option<DocumentGuestListenerException> {
        self.queue.borrow_mut().pop_front()
    }

    pub(super) fn reserve<'a>(
        &'a self,
        events: &GuestEvents,
        owner: &NativeOwner,
    ) -> Result<ReportReservation<'a>, DocumentGuestEventError> {
        let count = events.limits.dispatch.failure_records;
        let reserved = self
            .reserved
            .get()
            .checked_add(count)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        if self
            .queue
            .borrow()
            .len()
            .checked_add(reserved)
            .is_none_or(|total| total > self.capacity)
        {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let bytes = count
            .checked_mul(size_of::<Box<dyn Any>>())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let backing = events.reserve(owner, bytes)?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        if roots.capacity() != count {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let root_bytes = events
            .exception_root_bytes
            .checked_add(size_of::<GuestBacking>())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        for _ in 0..count {
            let root: Box<dyn Any> = Box::new(events.reserve(owner, root_bytes)?);
            roots.push(root);
        }
        self.reserved.set(reserved);
        Ok(ReportReservation {
            reports: self,
            roots,
            _backing: backing,
        })
    }
}

pub(super) struct ReportReservation<'a> {
    reports: &'a GuestExceptionReports,
    roots: Vec<Box<dyn Any>>,
    _backing: GuestBacking,
}

impl ReportReservation<'_> {
    pub(super) fn publish(
        &mut self,
        scope: &mut Scope<'_>,
        exception: DocumentGuestListenerException,
    ) -> Result<(), DocumentGuestEventError> {
        let backing = self
            .roots
            .pop()
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        self.reports.reserved.set(self.reports.reserved.get() - 1);
        scope.retain_exception_backing(&exception.exception, backing)?;
        self.reports.queue.borrow_mut().push_back(exception);
        Ok(())
    }
}

impl Drop for ReportReservation<'_> {
    fn drop(&mut self) {
        self.reports
            .reserved
            .set(self.reports.reserved.get() - self.roots.len());
    }
}
