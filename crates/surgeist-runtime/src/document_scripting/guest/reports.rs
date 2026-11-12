use std::{any::Any, collections::VecDeque};

use super::*;
use crate::trusted_types_csp::{ReportError, ViolationReport};
use surgeist_dom::DocumentId;

pub(super) fn violation_error(
    error: ReportError<DocumentGuestEventError>,
) -> DocumentGuestEventError {
    match error {
        ReportError::Stopped(error) => error,
        ReportError::RepresentationExhausted => {
            DocumentScriptNativeError::RepresentationExhausted.into()
        }
        ReportError::Allocation => Error::Allocation.into(),
    }
}

pub(super) enum ViolationBacking {
    Guest {
        _backing: GuestBacking,
    },
    Attribute {
        _backing: surgeist_dom::DomSourceAttributePayloadRetention,
    },
}

/// The native Window and its associated Document at violation formation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ViolationOrigin {
    pub(super) global: DomHostedGlobalId,
    pub(super) document: DocumentId,
}

pub(in crate::document_scripting) struct ViolationQueue {
    origin: ViolationOrigin,
    reports: VecDeque<ViolationReport<ViolationBacking>>,
    backing: Option<ViolationBacking>,
    _header: ViolationBacking,
}

impl ViolationQueue {
    pub(super) fn admit_report(
        queue: &mut Option<Box<Self>>,
        origin: ViolationOrigin,
        bytes: usize,
        mut reserve: impl FnMut(usize, usize) -> Result<ViolationBacking, DocumentGuestEventError>,
        charge_copy: impl FnOnce(u64) -> Result<(), DocumentGuestEventError>,
    ) -> Result<ViolationBacking, DocumentGuestEventError> {
        if queue.is_none() {
            let header = reserve(size_of::<Self>(), 0)?;
            *queue = Some(Box::new(Self {
                origin,
                reports: VecDeque::new(),
                backing: None,
                _header: header,
            }));
        }
        let queue = queue.as_mut().ok_or(DocumentScriptNativeError::Busy)?;
        if queue.origin != origin {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        queue.reserve(&mut reserve, charge_copy)?;
        reserve(bytes, bytes / size_of::<u16>())
    }

    fn reserve(
        &mut self,
        reserve: &mut impl FnMut(usize, usize) -> Result<ViolationBacking, DocumentGuestEventError>,
        charge_copy: impl FnOnce(u64) -> Result<(), DocumentGuestEventError>,
    ) -> Result<(), DocumentGuestEventError> {
        if self.reports.len() < self.reports.capacity() {
            return Ok(());
        }
        let capacity = self
            .reports
            .capacity()
            .checked_mul(2)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?
            .max(1);
        let bytes = capacity
            .checked_mul(size_of::<ViolationReport<ViolationBacking>>())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let backing = reserve(bytes, 0)?;
        let mut reports = VecDeque::new();
        reports
            .try_reserve_exact(capacity)
            .map_err(|_| Error::Allocation)?;
        if reports.capacity() != capacity {
            return Err(Error::Allocation.into());
        }
        charge_copy(
            u64::try_from(self.reports.len())
                .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?,
        )?;
        reports.append(&mut self.reports);
        self.reports = reports;
        self.backing = Some(backing);
        Ok(())
    }

    pub(super) fn push(&mut self, report: ViolationReport<ViolationBacking>) {
        self.reports.push_back(report);
    }

    #[cfg(test)]
    pub(super) fn origin(&self) -> ViolationOrigin {
        self.origin
    }

    #[cfg(test)]
    pub(super) fn iter(&self) -> impl Iterator<Item = &ViolationReport<ViolationBacking>> {
        self.reports.iter()
    }
}

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
        self.reserve_count(events, owner, events.limits.dispatch.failure_records)
    }

    pub(super) fn reserve_count<'a>(
        &'a self,
        events: &GuestEvents,
        owner: &NativeOwner,
        count: usize,
    ) -> Result<ReportReservation<'a>, DocumentGuestEventError> {
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
        let roots = ExceptionRoots::reserve(events, owner, count)?;
        self.reserved.set(reserved);
        Ok(ReportReservation {
            reports: self,
            roots,
        })
    }
}

pub(super) struct ExceptionRoots {
    roots: Vec<Box<dyn Any>>,
    _backing: GuestBacking,
}

impl ExceptionRoots {
    pub(super) fn reserve(
        events: &GuestEvents,
        owner: &NativeOwner,
        count: usize,
    ) -> Result<Self, DocumentGuestEventError> {
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
        Ok(Self {
            roots,
            _backing: backing,
        })
    }

    pub(super) fn take(&mut self) -> Result<Box<dyn Any>, DocumentGuestEventError> {
        self.roots
            .pop()
            .ok_or_else(|| DocumentScriptNativeError::StorageLimit.into())
    }
}

pub(super) struct DirectExceptions {
    exceptions: Vec<DocumentGuestListenerException>,
    _backing: GuestBacking,
}

impl DirectExceptions {
    pub(super) fn reserve(
        events: &GuestEvents,
        owner: &NativeOwner,
        count: usize,
    ) -> Result<Box<Self>, DocumentGuestEventError> {
        let bytes = count
            .checked_mul(size_of::<DocumentGuestListenerException>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let backing = events.reserve(owner, bytes)?;
        let mut exceptions = Vec::new();
        exceptions
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        if exceptions.capacity() != count {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        Ok(Box::new(Self {
            exceptions,
            _backing: backing,
        }))
    }

    pub(super) fn publish(
        &mut self,
        scope: &mut Scope<'_>,
        roots: &mut ExceptionRoots,
        exception: DocumentGuestListenerException,
    ) -> Result<(), DocumentGuestEventError> {
        scope.retain_exception_backing(&exception.exception, roots.take()?)?;
        self.exceptions.push(exception);
        Ok(())
    }

    pub(super) fn observations(&self) -> &[DocumentGuestListenerException] {
        &self.exceptions
    }
}

pub(super) struct ReportReservation<'a> {
    reports: &'a GuestExceptionReports,
    roots: ExceptionRoots,
}

impl ReportReservation<'_> {
    pub(super) fn publish(
        &mut self,
        scope: &mut Scope<'_>,
        exception: DocumentGuestListenerException,
    ) -> Result<(), DocumentGuestEventError> {
        let backing = self.roots.take()?;
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
            .set(self.reports.reserved.get() - self.roots.roots.len());
    }
}
