//! Original traversal work after the synchronous Source hook has queued it.
//! Runtime owns FIFO/task/checkpoint execution; canonical DOM owns these values
//! and the actual History writes. Queueing is not execution or settlement.
use super::*;
use crate::DocumentId;

/// Unique permission for one captured creation C or removal H continuation.
/// The canonical operation bank retains its original request, cursor and Work
/// until successful explicit finish; dropping this permission settles nothing.
#[derive(Debug)]
pub struct DomHostedQueuedHistory {
    pub(crate) operation: DomHostedOperationId,
    pub(crate) traversable: DomHostedNavigableId,
}

/// A captured Global job remains pending until its real checkpoint or exact
/// same-generation destruction has settled the original obligation.
#[derive(Debug)]
pub enum DomHostedHistoryStep<'a> {
    Job(&'a DomHostedHistoryJobRequest),
    Complete,
}

#[derive(Debug)]
pub struct DomHostedHistoryJobRequest {
    pub(crate) key: RequestKey,
    pub(crate) occurrence: u64,
    pub(crate) job: crate::store::hosted::history::Job,
}
impl DomHostedHistoryJobRequest {
    pub fn operation(&self) -> DomHostedOperationId {
        self.key.operation
    }
    pub fn occurrence(&self) -> u64 {
        self.occurrence
    }
    pub fn document(&self) -> DocumentId {
        self.job.document
    }
    pub fn global(&self) -> DomHostedGlobalId {
        self.job.global
    }
    pub fn history_length(&self) -> usize {
        self.job.length
    }
    pub fn history_index(&self) -> usize {
        self.job.index
    }
}

/// Proof of this job's actual canonical write, not its later checkpoint.
/// Public callers cannot construct, clone or substitute another job's receipt.
#[derive(Debug)]
pub struct DomHostedHistoryJobReceipt {
    pub(crate) key: RequestKey,
    pub(crate) occurrence: u64,
    pub(crate) job: crate::store::hosted::history::Job,
}

/// Exact Source publication that hands off the original captured C.
#[derive(Debug)]
pub struct DomHostedChildHistoryPublication {
    pub(crate) key: RequestKey,
    pub(crate) revision: DomHostedSourceRevision,
}
impl DomHostedSource {
    pub fn seal_child_history(
        &self,
        request: &DomHostedChildHistoryRequest,
        cancel: &AtomicBool,
    ) -> Result<DomHostedChildHistoryPublication, DomHostedError> {
        let revision = self.validate_request(&request.request, cancel)?;
        Ok(DomHostedChildHistoryPublication {
            key: request.request.key,
            revision,
        })
    }
}

impl DomHostedQueuedHistory {
    pub fn operation(&self) -> DomHostedOperationId {
        self.operation
    }
    pub fn traversable(&self) -> DomHostedNavigableId {
        self.traversable
    }
}
