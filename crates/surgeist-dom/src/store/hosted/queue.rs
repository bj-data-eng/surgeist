//! Queue progression in the same canonical HostedOperation row. Public handles
//! are permissions; dropping them cannot destroy this original request or Work.
use super::*;
use crate::hosted::{
    DomHostedHistoryJobReceipt, DomHostedHistoryJobRequest, DomHostedHistoryStep,
    DomHostedQueuedHistory,
};

#[derive(Clone, Copy)]
enum Kind {
    Creation,
    Removal,
}
struct Running {
    update: history::Update,
    cursor: usize,
    current: DomHostedHistoryJobRequest,
    charge: Option<source::checkpoint::Charge>,
}
struct Completed {
    update: history::Update,
    _charge: Option<source::checkpoint::Charge>,
}
enum Phase {
    Queued,
    Waiting(Running),
    Written(Running),
    Complete(Completed),
    Finished,
}
/// Embedded in the original Operation, whose existing debit owns this header.
/// There is no independently allocated controller, issuer, or Work registry.
pub(in crate::store) struct Continuation {
    request: Request,
    creation: DomHostedCreation,
    traversable: DomHostedNavigableId,
    kind: Kind,
    phase: Phase,
    reserved_dom: u64,
    executed: usize,
    destroyed: usize,
}
impl Continuation {
    pub(in crate::store) fn creation(
        request: Request,
        creation: DomHostedCreation,
        traversable: DomHostedNavigableId,
    ) -> Self {
        Self::new(request, creation, traversable, Kind::Creation)
    }
    pub(in crate::store) fn removal(
        request: Request,
        creation: DomHostedCreation,
        traversable: DomHostedNavigableId,
    ) -> Self {
        Self::new(request, creation, traversable, Kind::Removal)
    }
    fn new(
        request: Request,
        creation: DomHostedCreation,
        traversable: DomHostedNavigableId,
        kind: Kind,
    ) -> Self {
        Self {
            request,
            creation,
            traversable,
            kind,
            phase: Phase::Queued,
            reserved_dom: 0,
            executed: 0,
            destroyed: 0,
        }
    }
    pub(in crate::store) fn request(&self) -> &Request {
        &self.request
    }
    fn next_phase(&self) -> Result<u64, DomHostedError> {
        self.request
            .key
            .phase
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)
    }
    fn current(&self) -> Result<&Running, DomHostedError> {
        match &self.phase {
            Phase::Waiting(row) | Phase::Written(row) => Ok(row),
            _ => Err(DomHostedError::WrongPhase),
        }
    }
    fn step(&self) -> Result<DomHostedHistoryStep<'_>, DomHostedError> {
        match &self.phase {
            Phase::Waiting(row) | Phase::Written(row) => {
                Ok(DomHostedHistoryStep::Job(&row.current))
            }
            Phase::Complete(_) => Ok(DomHostedHistoryStep::Complete),
            _ => Err(DomHostedError::WrongPhase),
        }
    }
    fn check_permission(&self, permission: &DomHostedQueuedHistory) -> Result<(), DomHostedError> {
        if self.request.key.operation != permission.operation
            || self.traversable != permission.traversable
        {
            return Err(DomHostedError::WrongPhase);
        }
        Ok(())
    }
    fn issue(
        key: RequestKey,
        update: &history::Update,
        cursor: usize,
    ) -> Result<DomHostedHistoryJobRequest, DomHostedError> {
        let occurrence = u64::try_from(cursor)
            .map_err(|_| DomHostedError::RepresentationExhausted)?
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let job = *update
            .jobs
            .values
            .get(cursor)
            .ok_or(DomHostedError::WrongPhase)?;
        Ok(DomHostedHistoryJobRequest {
            key,
            occurrence,
            job,
        })
    }
    fn reserve(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        jobs: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let jobs = u64::try_from(jobs).map_err(|_| DomHostedError::RepresentationExhausted)?;
        let creation = u64::from(matches!(self.kind, Kind::Creation));
        let source_needed = jobs
            .checked_mul(2)
            .and_then(|n| n.checked_add(1 + creation))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let dom_needed = jobs
            .checked_add(1 + creation)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        // Every original permit, including close, advances the phase field.
        self.request
            .key
            .phase
            .checked_add(source_needed)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let dom_extra = dom_needed.saturating_sub(self.reserved_dom);
        if dom_extra != 0 {
            // This queued plan has not consumed any permit yet. An unchanged
            // DOM bound therefore needs no repeated reservation/read. On growth,
            // only the original issuer knows its actual remaining source credit.
            dom.reserve_hosted_operation(source, &self.request, source_needed, dom_extra, cancel)?;
            self.reserved_dom = self.reserved_dom.max(dom_needed);
        }
        Ok(())
    }
    fn start(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        if !matches!(self.phase, Phase::Queued) {
            return Ok(());
        }
        dom.validate_hosted_source(source)?;
        self.request.work.step(cancel)?;
        let mut draft = dom.hosted_request_draft(&self.request, source, cancel)?;
        if matches!(self.kind, Kind::Creation) {
            let fresh = source::formation::Formation::new(&draft, cancel)?;
            let base = fresh.as_ref().map_or(0, |f| f.owned_payload().bytes);
            let context = history::Context::new(source, &self.request.work, cancel, base);
            let context = if let Some(fresh) = &fresh {
                context.with_formation(fresh)
            } else {
                context
            };
            let graph = draft
                .hosted_history
                .as_deref()
                .ok_or(DomHostedError::MissingPublication)?;
            let candidate = graph.append_creation(
                self.creation.navigable,
                self.request.key.operation,
                &context,
            )?;
            draft.set_hosted_history(Some(candidate), fresh, cancel)?;
        }
        let fresh = source::formation::Formation::new(&draft, cancel)?;
        let context = history::Context::new(source, &self.request.work, cancel, 0);
        let context = if let Some(fresh) = &fresh {
            context.with_formation(fresh)
        } else {
            context
        };
        let graph = draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let update = graph.plan_update(self.traversable, &context, |parent, visit| {
            dom.visit_hosted_children(graph, parent, &self.request.work, cancel, visit)
        })?;
        let charge = fresh.map(source::formation::Formation::into_charge);
        self.reserve(dom, source, update.jobs.values.len(), cancel)?;
        let next = if matches!(self.kind, Kind::Creation) {
            self.next_phase()?
        } else {
            self.request.key.phase
        };
        let key = RequestKey {
            operation: self.request.key.operation,
            phase: next,
        };
        let first = if update.jobs.values.is_empty() {
            None
        } else {
            Some(Self::issue(key, &update, 0)?)
        };
        if matches!(self.kind, Kind::Creation) {
            let prepared = draft.finish(dom, cancel)?;
            let permit = source.permit(&self.request, source.revision()?, false, cancel)?;
            dom.publish_hosted(prepared, cancel)?;
            permit.commit();
            self.request.key.phase = next;
        }
        self.phase = match first {
            Some(current) => Phase::Waiting(Running {
                update,
                cursor: 0,
                current,
                charge,
            }),
            None => Phase::Complete(Completed {
                update,
                _charge: charge,
            }),
        };
        Ok(())
    }
    fn execute_job(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHistoryJobReceipt, DomHostedError> {
        if let Phase::Written(running) = &self.phase {
            // Recover only the original already-written evidence if its external
            // wrapper was lost; never execute or publish this body a second time.
            dom.validate_hosted_source(source)?;
            self.request.work.step(cancel)?;
            return Ok(DomHostedHistoryJobReceipt {
                key: running.current.key,
                occurrence: running.current.occurrence,
                job: running.current.job,
            });
        }
        let Phase::Waiting(running) = &self.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let job = running.current.job;
        dom.validate_hosted_source(source)?;
        self.request.work.step(cancel)?;
        let context = history::Context::new(source, &self.request.work, cancel, 0);
        let graph = dom
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        // Actual cleanup evidence precedes any current Document/Global access.
        // Retirement must be explicitly settled through its distinct path.
        if graph.job_destroyed(&job, &context)? {
            return Err(DomHostedError::StaleIdentity);
        }
        if !dom.hosted_document_fully_active(job.document, &self.request.work, cancel)? {
            return Err(DomHostedError::IncompleteOperation);
        }
        let node = dom.state.node(dom.owner, job.document.node())?;
        let DomNodeKind::Document(metadata) = node.kind() else {
            return Err(DomError::WrongDocument.into());
        };
        let environment = metadata.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::UnsupportedProfile)?;
        if association.global != job.global {
            return Err(DomHostedError::StaleBasis);
        }
        association.validate_current()?;
        let mut draft = dom.hosted_request_draft(&self.request, source, cancel)?;
        let fresh = source::formation::Formation::new(&draft, cancel)?;
        let context = history::Context::new(source, &self.request.work, cancel, 0);
        let context = if let Some(fresh) = &fresh {
            context.with_formation(fresh)
        } else {
            context
        };
        let graph = draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let candidate = graph.apply_history_job(&job, &context)?;
        draft.set_hosted_history(Some(candidate), fresh, cancel)?;
        let prepared = draft.finish(dom, cancel)?;
        let next = self.next_phase()?;
        let receipt = DomHostedHistoryJobReceipt {
            key: running.current.key,
            occurrence: running.current.occurrence,
            job,
        };
        let permit = source.permit(&self.request, source.revision()?, false, cancel)?;
        dom.publish_hosted(prepared, cancel)?;
        permit.commit();
        self.request.key.phase = next;
        let Phase::Waiting(running) = std::mem::replace(&mut self.phase, Phase::Finished) else {
            unreachable!("validated waiting job")
        };
        self.phase = Phase::Written(running);
        Ok(receipt)
    }
    fn settle_job(
        &mut self,
        dom: &Dom,
        source: &DomHostedSource,
        receipt: &mut Option<DomHostedHistoryJobReceipt>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let Phase::Written(running) = &self.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let actual = receipt.as_ref().ok_or(DomHostedError::MissingPublication)?;
        if actual.key != running.current.key
            || actual.occurrence != running.current.occurrence
            || actual.job != running.current.job
        {
            return Err(DomHostedError::WrongPhase);
        }
        dom.validate_hosted_source(source)?;
        self.request.work.step(cancel)?;
        let executed = self
            .executed
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        self.advance(source, cancel, true)?;
        self.executed = executed;
        receipt.take();
        Ok(())
    }
    fn settle_destroyed_job(
        &mut self,
        dom: &Dom,
        source: &DomHostedSource,
        proof: &DomHostedDocumentDestructionProof,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let Phase::Waiting(running) = &self.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        let job = running.current.job;
        dom.validate_hosted_source(source)?;
        self.request.work.step(cancel)?;
        proof.validate(source, job.document, job.global)?;
        let graph = dom
            .state
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let context = history::Context::new(source, &self.request.work, cancel, 0);
        if !graph.job_destroyed(&job, &context)? {
            return Err(DomHostedError::StaleBasis);
        }
        let destroyed = self
            .destroyed
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        self.advance(source, cancel, false)?;
        self.destroyed = destroyed;
        Ok(())
    }
    fn advance(
        &mut self,
        source: &DomHostedSource,
        cancel: &AtomicBool,
        written: bool,
    ) -> Result<(), DomHostedError> {
        let running = self.current()?;
        let cursor = running
            .cursor
            .checked_add(1)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let next = self.next_phase()?;
        let key = RequestKey {
            operation: self.request.key.operation,
            phase: next,
        };
        let request = if cursor < running.update.jobs.values.len() {
            Some(Self::issue(key, &running.update, cursor)?)
        } else {
            None
        };
        let permit = source.permit(&self.request, source.revision()?, false, cancel)?;
        permit.commit();
        self.request.key.phase = next;
        let old = std::mem::replace(&mut self.phase, Phase::Finished);
        let running = match (written, old) {
            (true, Phase::Written(row)) | (false, Phase::Waiting(row)) => row,
            _ => unreachable!("validated current settlement phase"),
        };
        self.phase = if let Some(current) = request {
            Phase::Waiting(Running {
                update: running.update,
                cursor,
                current,
                charge: running.charge,
            })
        } else {
            Phase::Complete(Completed {
                update: running.update,
                _charge: running.charge,
            })
        };
        Ok(())
    }
    fn finish(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let Phase::Complete(completed) = &self.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        if self.executed.checked_add(self.destroyed) != Some(completed.update.jobs.values.len()) {
            return Err(DomHostedError::IncompleteOperation);
        }
        dom.validate_hosted_source(source)?;
        let mut draft = dom.hosted_request_draft(&self.request, source, cancel)?;
        let fresh = source::formation::Formation::new(&draft, cancel)?;
        let context = history::Context::new(source, &self.request.work, cancel, 0);
        let context = if let Some(fresh) = &fresh {
            context.with_formation(fresh)
        } else {
            context
        };
        let graph = draft
            .hosted_history
            .as_deref()
            .ok_or(DomHostedError::MissingPublication)?;
        let candidate = graph.finish_update(&completed.update, &context)?;
        draft.set_hosted_history(Some(candidate), fresh, cancel)?;
        let prepared = draft.finish(dom, cancel)?;
        let used = u64::try_from(self.executed)
            .map_err(|_| DomHostedError::RepresentationExhausted)?
            .checked_add(1 + u64::from(matches!(self.kind, Kind::Creation)))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let unused = self
            .reserved_dom
            .checked_sub(used)
            .ok_or(DomHostedError::WrongPhase)?;
        let after_close = dom
            .hosted_revisions
            .checked_sub(1)
            .and_then(|n| n.checked_sub(unused))
            .ok_or(DomHostedError::WrongPhase)?;
        let invocation = match self.kind {
            Kind::Removal => Some(dom.prepare_hosted_history_completion(
                self.request.key.operation,
                &self.request.work,
                cancel,
            )?),
            Kind::Creation => None,
        };
        let permit = source.permit(&self.request, source.revision()?, true, cancel)?;
        dom.publish_hosted(prepared, cancel)?;
        permit.commit();
        // Release only this queue's precomputed unused publication headroom.
        dom.hosted_revisions = after_close;
        if let Some(invocation) = invocation {
            // The exact original Pending(H) row was validated before publication.
            // Nothing fallible or externally callable follows successful publish.
            dom.settle_hosted_history_completion(invocation, self.request.key.operation);
        }
        self.phase = Phase::Finished;
        Ok(())
    }
}

impl Dom {
    pub fn resume_queued_history<'a>(
        &'a mut self,
        source: &mut DomHostedSource,
        queued: &mut DomHostedQueuedHistory,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHistoryStep<'a>, DomHostedError> {
        self.require_sync_source_idle()?;
        self.with_hosted_operation(queued.operation, |dom, operation| {
            let CreationPhase::Queued(continuation) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            continuation.check_permission(queued)?;
            continuation.start(dom, source, cancel)?;
            Ok(OperationResult::Retain(()))
        })?;
        let operation = self.hosted_operation(queued.operation)?;
        let CreationPhase::Queued(continuation) = &operation.phase else {
            return Err(DomHostedError::WrongPhase);
        };
        continuation.check_permission(queued)?;
        continuation.step()
    }
}
impl DomHostedQueuedHistory {
    pub fn execute_job(
        &mut self,
        dom: &mut Dom,
        source: &mut DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHistoryJobReceipt, DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.with_hosted_operation(self.operation, |dom, operation| {
            let CreationPhase::Queued(continuation) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            continuation.check_permission(self)?;
            Ok(OperationResult::Retain(
                continuation.execute_job(dom, source, cancel)?,
            ))
        })
    }
    pub fn settle_job(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        receipt: &mut Option<DomHostedHistoryJobReceipt>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.with_hosted_operation(self.operation, |dom, operation| {
            let CreationPhase::Queued(continuation) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            continuation.check_permission(self)?;
            continuation.settle_job(dom, source, receipt, cancel)?;
            Ok(OperationResult::Retain(()))
        })
    }
    pub fn settle_destroyed_job(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        proof: &DomHostedDocumentDestructionProof,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.with_hosted_operation(self.operation, |dom, operation| {
            let CreationPhase::Queued(continuation) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            continuation.check_permission(self)?;
            continuation.settle_destroyed_job(dom, source, proof, cancel)?;
            Ok(OperationResult::Retain(()))
        })
    }
    pub fn finish(
        &mut self,
        dom: &mut Dom,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        dom.require_sync_source_idle()?;
        dom.with_hosted_operation(self.operation, |dom, operation| {
            let CreationPhase::Queued(continuation) = &mut operation.phase else {
                return Err(DomHostedError::WrongPhase);
            };
            continuation.check_permission(self)?;
            continuation.finish(dom, source, cancel)?;
            Ok(OperationResult::Complete(()))
        })
    }
}

#[cfg(test)]
mod tests {
    // Reservation-domain fixtures use actual issuer roles and a native Document
    // identity payload. They attest no hosted Global, event loop or checkpoint.
    use super::*;
    use std::sync::atomic::Ordering;

    fn fixture() -> (Dom, DomHostedSource, Continuation) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                4096,
                64,
                0,
                0,
                16,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let ticket = batch.create_document();
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.document(&ticket).unwrap();
        let origin = dom
            .snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .origin()
            .clone();
        let mut source = dom
            .register_hosted_source(
                DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    8_000_000,
                    8_000_000,
                    1_000_000,
                ),
                &cancel,
            )
            .unwrap();
        let host = source.issue_host(&cancel).unwrap();
        let prefix = source.prepare_top_level(host, &cancel).unwrap();
        let work = prefix.work;
        let operation = source.begin_operation(&work, 10, 0, &cancel).unwrap();
        let mut request = Request {
            key: RequestKey {
                operation,
                phase: 0,
            },
            host,
            context: prefix.context,
            work,
            parent: None,
        };
        advance(&source, &mut request);
        let inputs =
            crate::document::HostedDocumentInputs::form(&source, &request.work, &origin, &cancel)
                .unwrap();
        let execution = DomHostedExecutionRequest {
            request,
            origin,
            inputs,
        };
        let global = source.issue_global(&execution, &cancel).unwrap();
        let mut request = execution.request;
        for _ in 0..3 {
            advance(&source, &mut request);
        }
        let entry_request = DomHostedEntryRequest {
            request,
            document,
            global,
        };
        let entry = source.issue_entry(&entry_request, &cancel).unwrap();
        let navigable = source.issue_navigable(&entry_request, &cancel).unwrap();
        let creation = DomHostedCreation {
            document,
            host,
            context: entry_request.request.context,
            global,
            entry,
            navigable,
        };
        (
            dom,
            source,
            Continuation::creation(entry_request.request, creation, navigable),
        )
    }
    fn advance(source: &DomHostedSource, request: &mut Request) {
        source
            .permit(
                request,
                source.revision().unwrap(),
                false,
                &AtomicBool::new(false),
            )
            .unwrap()
            .commit();
        request.key.phase += 1;
    }

    #[test]
    fn creation_reserves_actual_write_settlement_and_close_headroom_only_once() {
        let cancel = AtomicBool::new(false);
        let (mut dom, source, mut queue) = fixture();
        let original = queue.request.key;
        let before = dom.hosted_revisions;
        // begin(10), the original operation issue, three role issues and four
        // permits leave two credits in this exact real issuer row.
        assert_eq!(
            source
                .operation_reserved_revisions(&queue.request, &cancel)
                .unwrap(),
            2
        );
        queue.reserve(&mut dom, &source, 2, &cancel).unwrap();
        // C + two actual writes + close are four canonical publications;
        // both job settlements additionally need their original source permits.
        assert_eq!(dom.hosted_revisions, before + 4);
        assert_eq!(
            source
                .operation_reserved_revisions(&queue.request, &cancel)
                .unwrap(),
            6
        );
        assert_eq!(queue.reserved_dom, 4);
        let spent = queue.request.work.spent.load(Ordering::Relaxed);
        queue.reserve(&mut dom, &source, 2, &cancel).unwrap();
        assert_eq!(dom.hosted_revisions, before + 4);
        assert_eq!(queue.request.work.spent.load(Ordering::Relaxed), spent);
        assert_eq!(queue.request.key, original);
        queue.reserve(&mut dom, &source, 3, &cancel).unwrap();
        assert_eq!(dom.hosted_revisions, before + 5);
        assert_eq!(
            source
                .operation_reserved_revisions(&queue.request, &cancel)
                .unwrap(),
            8
        );
        assert_eq!(queue.reserved_dom, 5);
        assert_eq!(queue.request.key, original);
    }

    #[test]
    fn exhausted_canonical_headroom_refuses_before_source_reservation_or_exposure() {
        let cancel = AtomicBool::new(false);
        let (mut dom, source, mut queue) = fixture();
        let key = queue.request.key;
        let source_revision = source.revision().unwrap();
        let original_credit = source
            .operation_reserved_revisions(&queue.request, &cancel)
            .unwrap();
        let spent = queue.request.work.spent.load(Ordering::Relaxed);
        let used = source.limits().retained_bytes() - source.remaining_bytes().unwrap();
        dom.hosted_revisions = u64::MAX;
        let actual = queue.reserve(&mut dom, &source, 1, &cancel);
        assert!(
            matches!(
                actual,
                Err(DomHostedError::Dom(DomError::RevisionExhausted))
            ),
            "actual canonical headroom refusal: {actual:?}"
        );
        assert_eq!(dom.hosted_revisions, u64::MAX);
        assert_eq!(queue.reserved_dom, 0);
        assert_eq!(queue.request.key, key);
        assert_eq!(source.revision().unwrap(), source_revision);
        assert_eq!(queue.request.work.spent.load(Ordering::Relaxed), spent);
        assert_eq!(
            source
                .operation_reserved_revisions(&queue.request, &cancel)
                .unwrap(),
            original_credit
        );
        assert_eq!(
            source.limits().retained_bytes() - source.remaining_bytes().unwrap(),
            used
        );
        assert!(matches!(queue.step(), Err(DomHostedError::WrongPhase)));
    }

    #[test]
    fn canceled_reservation_preserves_the_original_request_and_can_retry() {
        let cancel = AtomicBool::new(true);
        let (mut dom, source, mut queue) = fixture();
        let key = queue.request.key;
        let work = queue.request.work.clone();
        let original_credit = source
            .operation_reserved_revisions(&queue.request, &AtomicBool::new(false))
            .unwrap();
        let spent = work.spent.load(Ordering::Relaxed);
        let before = dom.hosted_revisions;
        assert!(matches!(
            queue.reserve(&mut dom, &source, 2, &cancel),
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert_eq!(queue.request.key, key);
        assert_eq!(dom.hosted_revisions, before);
        assert!(Arc::ptr_eq(&work, &queue.request.work));
        assert_eq!(work.spent.load(Ordering::Relaxed), spent);
        assert_eq!(
            source
                .operation_reserved_revisions(&queue.request, &AtomicBool::new(false))
                .unwrap(),
            original_credit
        );
        cancel.store(false, Ordering::Relaxed);
        queue.reserve(&mut dom, &source, 2, &cancel).unwrap();
        assert_eq!(dom.hosted_revisions, before + 4);
        assert_eq!(queue.request.key, key);
        assert!(Arc::ptr_eq(&work, &queue.request.work));
    }
}
