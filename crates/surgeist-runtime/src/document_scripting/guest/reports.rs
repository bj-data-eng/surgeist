use std::collections::VecDeque;

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

pub(crate) enum ViolationBacking {
    Guest {
        _backing: GuestBacking,
    },
    Attribute {
        _backing: surgeist_dom::DomSourceAttributePayloadRetention,
    },
    Style {
        _backing: surgeist_dom::DomStyleCspRetention,
    },
}

/// The native Window and its associated Document at violation formation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ViolationOrigin {
    pub(super) global: DomHostedGlobalId,
    pub(super) document: DocumentId,
}

/// The immutable report and its original admission, shared by its task and Event.
pub(crate) struct QueuedViolation {
    report: ViolationReport<ViolationBacking>,
    element: Option<surgeist_dom::NodeId>,
}

impl QueuedViolation {
    pub(crate) fn report(&self) -> &ViolationReport<ViolationBacking> {
        &self.report
    }

    pub(crate) fn element(&self) -> Option<surgeist_dom::NodeId> {
        self.element
    }
}

pub(in crate::document_scripting) struct ViolationQueue {
    pub(super) origin: ViolationOrigin,
    reports: VecDeque<Rc<QueuedViolation>>,
    pub(super) active: Option<super::csp_task::ActiveViolationTask>,
    next_task: u64,
    backing: Option<ViolationBacking>,
    _header: ViolationBacking,
}

type ViolationBank = (VecDeque<Rc<QueuedViolation>>, ViolationBacking);

pub(super) enum PreparedViolationQueue<'a> {
    Existing(&'a mut ViolationQueue, Option<ViolationBank>),
    New(&'a mut Option<Box<ViolationQueue>>, Box<ViolationQueue>),
}

impl<'a> PreparedViolationQueue<'a> {
    pub(super) fn install(self) -> &'a mut ViolationQueue {
        match self {
            Self::Existing(queue, replacement) => {
                if let Some((mut reports, backing)) = replacement {
                    reports.append(&mut queue.reports);
                    queue.reports = reports;
                    queue.backing = Some(backing);
                }
                queue
            }
            Self::New(slot, new) => slot.insert(new),
        }
    }
}

impl ViolationQueue {
    pub(super) fn admit_report(
        queue: &mut Option<Box<Self>>,
        origin: ViolationOrigin,
        bytes: usize,
        mut reserve: impl FnMut(usize, usize) -> Result<ViolationBacking, DocumentGuestEventError>,
        charge_copy: impl FnOnce(u64) -> Result<(), DocumentGuestEventError>,
    ) -> Result<ViolationBacking, DocumentGuestEventError> {
        Self::prepare_batch(queue, origin, 1, &mut reserve, charge_copy)?.install();
        reserve(Self::record_bytes(bytes)?, bytes / size_of::<u16>())
    }

    pub(super) fn record_bytes(bytes: usize) -> Result<usize, DocumentScriptNativeError> {
        bytes
            .checked_add(size_of::<QueuedViolation>() + 2 * size_of::<usize>())
            .ok_or(DocumentScriptNativeError::StorageLimit)
    }

    pub(super) fn prepare_batch<'a>(
        queue: &'a mut Option<Box<Self>>,
        origin: ViolationOrigin,
        count: usize,
        reserve: &mut impl FnMut(usize, usize) -> Result<ViolationBacking, DocumentGuestEventError>,
        charge_copy: impl FnOnce(u64) -> Result<(), DocumentGuestEventError>,
    ) -> Result<PreparedViolationQueue<'a>, DocumentGuestEventError> {
        if let Some(current) = queue {
            if current.origin != origin {
                return Err(DocumentScriptNativeError::ForeignPermit.into());
            }
            let replacement = current.prepare_bank(count, reserve, charge_copy)?;
            return Ok(PreparedViolationQueue::Existing(current, replacement));
        }
        let header = reserve(size_of::<Self>(), 0)?;
        let mut new = Box::new(Self {
            origin,
            reports: VecDeque::new(),
            active: None,
            next_task: 0,
            backing: None,
            _header: header,
        });
        if let Some((reports, backing)) = new.prepare_bank(count, reserve, charge_copy)? {
            new.reports = reports;
            new.backing = Some(backing);
        }
        Ok(PreparedViolationQueue::New(queue, new))
    }

    fn prepare_bank(
        &self,
        count: usize,
        reserve: &mut impl FnMut(usize, usize) -> Result<ViolationBacking, DocumentGuestEventError>,
        charge_copy: impl FnOnce(u64) -> Result<(), DocumentGuestEventError>,
    ) -> Result<Option<ViolationBank>, DocumentGuestEventError> {
        let needed = self
            .reports
            .len()
            .checked_add(count)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        if needed <= self.reports.capacity() {
            return Ok(None);
        }
        let capacity = self
            .reports
            .capacity()
            .checked_mul(2)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?
            .max(needed);
        let bytes = capacity
            .checked_mul(size_of::<Rc<QueuedViolation>>())
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
        Ok(Some((reports, backing)))
    }

    pub(super) fn push(&mut self, report: ViolationReport<ViolationBacking>) {
        self.reports.push_back(Rc::new(QueuedViolation {
            report,
            element: None,
        }));
    }

    pub(super) fn push_style(
        &mut self,
        element: surgeist_dom::NodeId,
        reports: Vec<ViolationReport<ViolationBacking>>,
    ) {
        self.reports.extend(reports.into_iter().map(|report| {
            Rc::new(QueuedViolation {
                report,
                element: Some(element),
            })
        }));
    }

    pub(super) fn head(&self) -> Option<Rc<QueuedViolation>> {
        self.reports.front().cloned()
    }

    pub(super) fn start_head(
        &mut self,
    ) -> Result<&mut super::csp_task::ActiveViolationTask, DocumentGuestEventError> {
        if self.reports.is_empty() {
            return Err(DocumentGuestEventError::RegistryState);
        }
        if self.active.is_none() {
            let task = self
                .next_task
                .checked_add(1)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            self.next_task = task;
            self.active = Some(super::csp_task::ActiveViolationTask::new(task));
        }
        self.active
            .as_mut()
            .ok_or(DocumentGuestEventError::RegistryState)
    }

    pub(super) fn finish_head(&mut self) {
        self.active = None;
        self.reports.pop_front();
    }

    #[cfg(test)]
    pub(super) fn origin(&self) -> ViolationOrigin {
        self.origin
    }

    #[cfg(test)]
    pub(super) fn iter(&self) -> impl Iterator<Item = &ViolationReport<ViolationBacking>> {
        self.reports.iter().map(|report| report.report())
    }
}

pub(in crate::document_scripting) type DispatchFailure =
    HostFailureSlot<DocumentGuestEventError, GuestBacking>;

pub(in crate::document_scripting) struct GuestExceptionReports {
    queue: RefCell<VecDeque<GuestExceptionReport>>,
    capacity: usize,
    reserved: Cell<usize>,
    _backing: GuestBacking,
}

enum GuestExceptionReport {
    Listener(DocumentGuestListenerException),
    Constructor(DocumentGuestConstructorException),
    CustomElementCallback(DocumentGuestCustomElementCallbackException),
}

impl GuestExceptionReports {
    pub(super) fn storage_bytes(capacity: usize) -> Option<usize> {
        capacity
            .checked_mul(size_of::<GuestExceptionReport>())?
            .checked_add(size_of::<Self>())
    }

    pub(super) fn new(capacity: usize, backing: GuestBacking) -> Result<Self, Error> {
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
        let mut queue = self.queue.borrow_mut();
        let index = queue
            .iter()
            .position(|report| matches!(report, GuestExceptionReport::Listener(_)))?;
        match queue.remove(index)? {
            GuestExceptionReport::Listener(exception) => Some(exception),
            _ => None,
        }
    }

    pub(super) fn take_constructor(&self) -> Option<DocumentGuestConstructorException> {
        let mut queue = self.queue.borrow_mut();
        let index = queue
            .iter()
            .position(|report| matches!(report, GuestExceptionReport::Constructor(_)))?;
        match queue.remove(index)? {
            GuestExceptionReport::Constructor(exception) => Some(exception),
            _ => None,
        }
    }

    pub(super) fn take_custom_element_callback(
        &self,
    ) -> Option<DocumentGuestCustomElementCallbackException> {
        let mut queue = self.queue.borrow_mut();
        let index = queue
            .iter()
            .position(|report| matches!(report, GuestExceptionReport::CustomElementCallback(_)))?;
        match queue.remove(index)? {
            GuestExceptionReport::CustomElementCallback(exception) => Some(exception),
            _ => None,
        }
    }

    pub(super) fn reserve<'a>(
        &'a self,
        events: &GuestEvents,
    ) -> Result<ReportReservation<'a>, DocumentGuestEventError> {
        self.reserve_count(events.limits.dispatch.failure_records)
    }

    pub(super) fn reserve_count<'a>(
        &'a self,
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
        self.reserved.set(reserved);
        Ok(ReportReservation {
            reports: self,
            remaining: count,
        })
    }

    pub(super) fn reserve_constructor(
        self: &Rc<Self>,
    ) -> Result<ConstructorReportReservation, DocumentGuestEventError> {
        let mut reservation = self.reserve_count(1)?;
        // Transfer this logical slot; only the constructor reservation releases
        // it, including when no exception is eventually published.
        reservation.remaining = 0;
        Ok(ConstructorReportReservation {
            reports: self.clone(),
            prepared: false,
        })
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
        exception: DocumentGuestListenerException,
    ) -> Result<(), DocumentGuestEventError> {
        if self.exceptions.len() == self.exceptions.capacity() {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        self.exceptions.push(exception);
        Ok(())
    }

    pub(super) fn observations(&self) -> &[DocumentGuestListenerException] {
        &self.exceptions
    }
}

pub(super) struct ReportReservation<'a> {
    reports: &'a GuestExceptionReports,
    remaining: usize,
}

impl ReportReservation<'_> {
    pub(super) fn publish(
        &mut self,
        exception: DocumentGuestListenerException,
    ) -> Result<(), DocumentGuestEventError> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        self.reports.reserved.set(self.reports.reserved.get() - 1);
        self.reports
            .queue
            .borrow_mut()
            .push_back(GuestExceptionReport::Listener(exception));
        Ok(())
    }
}

pub(super) struct ConstructorReportReservation {
    reports: Rc<GuestExceptionReports>,
    prepared: bool,
}

impl ConstructorReportReservation {
    pub(super) fn needs_preparation(&self) -> bool {
        !self.prepared
    }

    pub(super) fn prepare(&mut self) -> Result<(), Error> {
        if self.prepared {
            return Err(Error::RegistryConflict);
        }
        self.prepared = true;
        Ok(())
    }

    pub(super) fn publish_custom_element_callback(
        &self,
        exception: DocumentGuestCustomElementCallbackException,
    ) {
        self.reports
            .queue
            .borrow_mut()
            .push_back(GuestExceptionReport::CustomElementCallback(exception));
    }

    pub(super) fn publish(&self, exception: DocumentGuestConstructorException) {
        self.reports
            .queue
            .borrow_mut()
            .push_back(GuestExceptionReport::Constructor(exception));
    }
}

impl Drop for ConstructorReportReservation {
    fn drop(&mut self) {
        self.reports.reserved.set(self.reports.reserved.get() - 1);
    }
}

impl Drop for ReportReservation<'_> {
    fn drop(&mut self) {
        self.reports
            .reserved
            .set(self.reports.reserved.get() - self.remaining);
    }
}
