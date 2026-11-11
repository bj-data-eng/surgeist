//! One-shot native node retirement notices for retained host objects.

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

use super::*;
use crate::{DomSourceEventRequest, DomSourceEventRetention};

use custody::{Debit, Usage};

/// Admission for one ordinary retirement operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNodeRetirementLimits {
    canonical_bytes: usize,
    work: u64,
}

impl DomNodeRetirementLimits {
    /// Set the maximum prospective bytes in the original DOM custody ledger
    /// and the work allowance for this call. An armed watch also reserves
    /// bounded administrative disarm work from this allowance.
    pub const fn new(canonical_bytes: usize, work: u64) -> Self {
        Self {
            canonical_bytes,
            work,
        }
    }
}

struct OrdinaryAdmission<'a> {
    dom: &'a Dom,
    limits: DomNodeRetirementLimits,
    remaining: Cell<u64>,
    cancel: &'a AtomicBool,
}

impl<'a> OrdinaryAdmission<'a> {
    fn new(dom: &'a Dom, limits: DomNodeRetirementLimits, cancel: &'a AtomicBool) -> Self {
        Self {
            dom,
            limits,
            remaining: Cell::new(limits.work),
            cancel,
        }
    }

    fn admit_work(&self, steps: u64) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        let remaining =
            self.remaining
                .get()
                .checked_sub(steps)
                .ok_or(DomError::NodeRetirementWorkLimit {
                    limit: self.limits.work,
                })?;
        self.remaining.set(remaining);
        Ok(())
    }

    fn retain(&self, bytes: usize) -> Result<Debit, DomError> {
        self.dom
            .custody_registry
            .ledger()
            .retain(Usage::new(bytes, 0), |next| {
                self.admit_work(1)?;
                admit(
                    next.bytes(),
                    self.limits.canonical_bytes,
                    DomResource::NodeRetirementCanonicalBytes,
                )
            })
    }
}

struct Queue {
    open: bool,
    head: Option<Box<Notice>>,
    _backing: Debit,
}

/// Destination for affine retirement notices funded by the canonical owner.
/// The host drains it after releasing its native loan.
pub struct DomNodeRetirementInbox {
    owner: DomOwnerId,
    queue: Arc<Mutex<Queue>>,
}

impl DomNodeRetirementInbox {
    /// Take the next exact retired node and its caller-supplied custody key.
    /// The notice retains its admitted native backing until it is dropped.
    pub fn take(&mut self) -> Option<DomNodeRetirementNotice> {
        let mut queue = lock(&self.queue);
        let mut notice = queue.head.take()?;
        queue.head = notice.next.take();
        Some(DomNodeRetirementNotice(notice))
    }
}

impl Drop for DomNodeRetirementInbox {
    fn drop(&mut self) {
        lock(&self.queue).open = false;
        while self.take().is_some() {}
    }
}

/// A committed destruction of the exact watched generation. This notice
/// conveys no access or execution authority.
pub struct DomNodeRetirementNotice(Box<Notice>);

struct Notice {
    node: NodeId,
    custody_key: usize,
    next: Option<Box<Notice>>,
    _backing: Debit,
}

impl DomNodeRetirementNotice {
    pub const fn node(&self) -> NodeId {
        self.0.node
    }

    pub const fn custody_key(&self) -> usize {
        self.0.custody_key
    }
}

/// A canonical batch publication and the watch of one node it created.
pub struct DomWatchedCreation {
    commit: DomCommit,
    node: NodeId,
    watch: DomNodeRetirementWatch,
}

impl DomWatchedCreation {
    /// The exact newly published generation.
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// Transfer the publication receipt, created node and affine watch.
    pub fn into_parts(self) -> (DomCommit, NodeId, DomNodeRetirementWatch) {
        (self.commit, self.node, self.watch)
    }
}

struct Armed {
    notice: Box<Notice>,
    queue: Arc<Mutex<Queue>>,
    backing: Debit,
}

enum Row {
    Leaf(Box<Armed>),
    Branch(Box<Branch>),
}

struct Branch {
    bit: usize,
    children: [Option<Row>; 2],
    _backing: Debit,
}

fn side(slot: usize, bit: usize) -> usize {
    usize::from(slot & bit != 0)
}

impl Row {
    fn match_path(
        &self,
        node: NodeId,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        let mut row = self;
        let mut depth = 0;
        loop {
            step()?;
            depth += 1;
            match row {
                Self::Leaf(armed) => return Ok((armed.notice.node == node).then_some(depth)),
                Self::Branch(branch) => {
                    let Some(child) = branch.children[side(node.slot, branch.bit)].as_ref() else {
                        return Ok(None);
                    };
                    row = child;
                }
            }
        }
    }

    // The first differing bit separates two exact slot identities. Return
    // it up the original descent until the parent of the splice is reached.
    // None means that child installed the watch and no more work may refuse.
    fn arm_below(&mut self, admission: &WatchAdmission<'_>) -> Result<Option<usize>, DomError> {
        admission.step()?;
        match self {
            Self::Leaf(armed) => {
                let differing = admission.node.slot ^ armed.notice.node.slot;
                let bit = differing
                    .checked_ilog2()
                    .ok_or(DomError::NodeRetirementAlreadyWatched)?;
                Ok(Some(1 << bit))
            }
            Self::Branch(branch) => {
                let child = &mut branch.children[side(admission.node.slot, branch.bit)];
                let Some(bit) = child
                    .as_mut()
                    .ok_or(DomError::RepresentationExhausted)?
                    .arm_below(admission)?
                else {
                    return Ok(None);
                };
                admission.step()?;
                if branch.bit > bit {
                    admission.splice(child, Some(bit))?;
                    Ok(None)
                } else {
                    Ok(Some(bit))
                }
            }
        }
    }

    fn remove(root: &mut Option<Self>, node: NodeId, key: Option<usize>) -> Option<Box<Armed>> {
        match root.as_mut()? {
            Self::Leaf(row)
                if row.notice.node == node
                    && key.is_none_or(|key| row.notice.custody_key == key) =>
            {
                match root.take() {
                    Some(Self::Leaf(row)) => Some(row),
                    _ => None,
                }
            }
            Self::Leaf(_) => None,
            Self::Branch(branch) => {
                let index = side(node.slot, branch.bit);
                let removed = Self::remove(&mut branch.children[index], node, key);
                if removed.is_some() && branch.children[index].is_none() {
                    *root = branch.children[1 - index].take();
                }
                removed
            }
        }
    }
}

pub(super) struct Index {
    armed: Mutex<Option<Row>>,
    _backing: Debit,
}

pub(super) struct PublicationPlan {
    index: Arc<Index>,
    nodes: Vec<NodeId>,
}

impl PublicationPlan {
    pub(super) fn new(index: Arc<Index>, nodes: Vec<NodeId>) -> Self {
        Self { index, nodes }
    }

    pub(super) fn publish(self) {
        self.index.publish(self.nodes);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Affine watch of one native generation. Dropping it disarms its unpublished
/// notice. A notice already published stays with its original inbox.
pub struct DomNodeRetirementWatch {
    index: Arc<Index>,
    node: NodeId,
    custody_key: usize,
}

impl Drop for DomNodeRetirementWatch {
    fn drop(&mut self) {
        // One administrative path per affine watch. Each branch tests a
        // strictly lower slot bit, so both descent and unwind have at most
        // usize::BITS branch visits. This also works after Source exhaustion.
        let armed = self.index.remove(self.node, Some(self.custody_key));
        drop(armed);
    }
}

impl Index {
    fn remove(&self, node: NodeId, key: Option<usize>) -> Option<Box<Armed>> {
        Row::remove(&mut lock(&self.armed), node, key)
    }

    pub(super) fn prepare_match(
        &self,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        let depth = {
            let root = lock(&self.armed);
            match root.as_ref() {
                Some(row) => row.match_path(node, &mut step)?,
                None => {
                    step()?;
                    None
                }
            }
        };
        let Some(depth) = depth else {
            return Ok(false);
        };
        // The committed path can only shrink if a watch is disarmed after
        // preparation. Admit the removal descent, branch updates and append.
        for _ in 0..depth {
            step()?;
            step()?;
        }
        step()?;
        Ok(true)
    }

    fn publish(&self, nodes: Vec<NodeId>) {
        for node in nodes {
            let Some(armed) = self.remove(node, None) else {
                continue;
            };
            let Armed {
                mut notice,
                queue,
                backing,
            } = {
                // Free the moved-out leaf before releasing its storage debit
                // and before waiting on the independently drained inbox.
                let leaf = armed;
                *leaf
            };
            drop(backing);
            let mut queue = lock(&queue);
            if queue.open {
                notice.next = queue.head.take();
                queue.head = Some(notice);
            }
        }
    }
}

enum Funding<'a> {
    Source(&'a Dom, &'a DomSourceEventRequest),
    Ordinary(OrdinaryAdmission<'a>),
}

enum Reserved {
    Source(DomSourceEventRetention),
    Ordinary(Debit),
}

impl Reserved {
    fn take(&mut self, bytes: usize) -> Result<Debit, DomError> {
        match self {
            Self::Source(retained) => retained
                .split_payload(bytes, 0)
                .map(|part| part.into_debit()),
            Self::Ordinary(retained) => retained.split(Usage::new(bytes, 0)),
        }
    }
}

struct WatchAdmission<'a> {
    funding: Funding<'a>,
    inbox: &'a DomNodeRetirementInbox,
    node: NodeId,
    custody_key: usize,
    cancel: &'a AtomicBool,
}

impl WatchAdmission<'_> {
    fn step(&self) -> Result<(), DomError> {
        match &self.funding {
            Funding::Source(dom, request) => dom.source_event_step(request, self.cancel),
            Funding::Ordinary(admission) => admission.admit_work(1),
        }
    }

    fn form(&self, extra: usize) -> Result<(Box<Armed>, Reserved), DomError> {
        let leaf = size_of::<Armed>();
        let notice = size_of::<Notice>();
        let bytes = leaf + notice + extra;
        let mut retained = match &self.funding {
            Funding::Source(dom, request) => {
                Reserved::Source(dom.retain_source_event_payload(request, bytes, 0, self.cancel)?)
            }
            Funding::Ordinary(admission) => {
                // The path can grow after arming. Reserve the upper bound for
                // descent, unwind and leaf removal before publishing the row.
                admission.admit_work(2 * u64::from(usize::BITS) + 1)?;
                Reserved::Ordinary(admission.retain(bytes)?)
            }
        };
        let backing = retained.take(leaf)?;
        let notice = Box::new(Notice {
            node: self.node,
            custody_key: self.custody_key,
            next: None,
            _backing: retained.take(notice)?,
        });
        let armed = Box::new(Armed {
            notice,
            queue: self.inbox.queue.clone(),
            backing,
        });
        Ok((armed, retained))
    }

    fn splice(&self, root: &mut Option<Row>, bit: Option<usize>) -> Result<(), DomError> {
        self.step()?;
        let branch_bytes = bit.map_or(0, |_| size_of::<Branch>());
        let (armed, mut retained) = self.form(branch_bytes)?;
        let branch = if let Some(bit) = bit {
            Some(Box::new(Branch {
                bit,
                children: [None, None],
                _backing: retained.take(branch_bytes)?,
            }))
        } else {
            None
        };
        drop(retained);
        check_cancel(self.cancel)?;
        *root = Some(if let Some(mut branch) = branch {
            let index = side(self.node.slot, branch.bit);
            branch.children[1 - index] = root.take();
            branch.children[index] = Some(Row::Leaf(armed));
            Row::Branch(branch)
        } else {
            Row::Leaf(armed)
        });
        Ok(())
    }
}

impl Dom {
    /// Admit an inbox for exact retirement notices under an original Source
    /// event. Its custody survives the original event and retains no engine.
    pub fn source_event_node_retirement_inbox(
        &self,
        request: &DomSourceEventRequest,
        cancel: &AtomicBool,
    ) -> Result<DomNodeRetirementInbox, DomError> {
        self.source_event_step(request, cancel)?;
        let bytes = size_of::<Mutex<Queue>>();
        let backing = self
            .retain_source_event_payload(request, bytes, 0, cancel)?
            .split_payload(bytes, 0)?
            .into_debit();
        Ok(retirement_inbox(self.owner, backing))
    }

    /// Admit an inbox for ordinary exact-generation retirement notices.
    /// The original DOM owns its backing and may later publish into it.
    pub fn node_retirement_inbox(
        &self,
        limits: DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<DomNodeRetirementInbox, DomError> {
        self.require_source_idle()?;
        let admission = OrdinaryAdmission::new(self, limits, cancel);
        admission.admit_work(1)?;
        let backing = admission.retain(size_of::<Mutex<Queue>>())?;
        check_cancel(cancel)?;
        Ok(retirement_inbox(self.owner, backing))
    }

    /// Admit an exact-generation watch under the original Source event before
    /// exposing a retained host object. The caller's key is returned only
    /// with this node's actual canonical destruction. A node has at most one
    /// armed watch.
    pub fn watch_source_event_node_retirement(
        &mut self,
        request: &DomSourceEventRequest,
        node: NodeId,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        cancel: &AtomicBool,
    ) -> Result<DomNodeRetirementWatch, DomError> {
        if inbox.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.source_event_step(request, cancel)?;
        self.state.node(self.owner, node)?;
        let admission = WatchAdmission {
            funding: Funding::Source(self, request),
            inbox,
            node,
            custody_key,
            cancel,
        };
        let index = arm(self.node_retirements.as_ref(), &admission)?;
        Ok(self.finish_watch(index, node, custody_key))
    }

    /// Watch the exact live generation during an ordinary DOM operation.
    /// Detachment and adoption preserve the watch. Only committed destruction
    /// publishes the original key to the original inbox.
    pub fn watch_node_retirement(
        &mut self,
        node: NodeId,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        limits: DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<DomNodeRetirementWatch, DomError> {
        if inbox.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.require_source_idle()?;
        let index =
            self.arm_ordinary_watch(&self.state, node, inbox, custody_key, limits, cancel)?;
        Ok(self.finish_watch(index, node, custody_key))
    }

    /// Apply an ordinary batch and arm a retirement watch for one of its
    /// newly created nodes at the same publication boundary. The ticket must
    /// belong to the batch and remain live in its resulting state. A refusal
    /// leaves both the canonical state and the retirement index unchanged.
    pub fn apply_with_node_retirement_watch(
        &mut self,
        batch: DomBatch,
        ticket: &CreatedNode,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        limits: DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<DomWatchedCreation, DomError> {
        if inbox.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.require_source_idle()?;
        let result = (|| {
            let prepared = self.prepare(batch, cancel)?;
            let node = resolve_created_node(&prepared.owner, &prepared.created, ticket)?
                .ok_or(DomError::UnavailableCreation)?;
            let mut publication = self.prepare_publication(prepared, cancel)?;
            publication.close_ordinary_sharing(cancel)?;
            let index = self.arm_ordinary_watch(
                &publication.state,
                node,
                inbox,
                custody_key,
                limits,
                cancel,
            )?;
            // Index insertion ends admission. Publication cannot refuse.
            let watch = self.finish_watch(index, node, custody_key);
            let commit = self.publish_prepared(publication);
            Ok(DomWatchedCreation {
                commit,
                node,
                watch,
            })
        })();
        if result.is_err() {
            // Abandoned staged frontiers have dropped. Release their registry
            // tickets before returning the caller's original refusal.
            self.custody_registry.reconcile_releases();
        }
        result
    }

    fn arm_ordinary_watch(
        &self,
        state: &State,
        node: NodeId,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        limits: DomNodeRetirementLimits,
        cancel: &AtomicBool,
    ) -> Result<Arc<Index>, DomError> {
        let ordinary = OrdinaryAdmission::new(self, limits, cancel);
        ordinary.admit_work(1)?;
        state.node_with(self.owner, node, &mut || ordinary.admit_work(1))?;
        let admission = WatchAdmission {
            funding: Funding::Ordinary(ordinary),
            inbox,
            node,
            custody_key,
            cancel,
        };
        arm(self.node_retirements.as_ref(), &admission)
    }

    fn finish_watch(
        &mut self,
        index: Arc<Index>,
        node: NodeId,
        custody_key: usize,
    ) -> DomNodeRetirementWatch {
        if self.node_retirements.is_none() {
            self.node_retirements = Some(index.clone());
        }
        DomNodeRetirementWatch {
            index,
            node,
            custody_key,
        }
    }
}

fn retirement_inbox(owner: DomOwnerId, backing: Debit) -> DomNodeRetirementInbox {
    DomNodeRetirementInbox {
        owner,
        queue: Arc::new(Mutex::new(Queue {
            open: true,
            head: None,
            _backing: backing,
        })),
    }
}

fn arm(
    current: Option<&Arc<Index>>,
    admission: &WatchAdmission<'_>,
) -> Result<Arc<Index>, DomError> {
    if let Some(index) = current {
        let mut root = lock(&index.armed);
        if let Some(row) = root.as_mut() {
            if let Some(bit) = row.arm_below(admission)? {
                admission.splice(&mut root, Some(bit))?;
            }
        } else {
            admission.splice(&mut root, None)?;
        }
        return Ok(index.clone());
    }
    admission.step()?;
    let header = size_of::<Index>();
    let (armed, mut retained) = admission.form(header)?;
    let backing = retained.take(header)?;
    drop(retained);
    let index = Arc::new(Index {
        armed: Mutex::new(Some(Row::Leaf(armed))),
        _backing: backing,
    });
    check_cancel(admission.cancel)?;
    Ok(index)
}

#[cfg(test)]
mod tests;
