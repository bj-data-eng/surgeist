//! One-shot native node retirement notices for retained host objects.

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

use super::*;
use crate::{DomSourceEventRequest, DomSourceEventRetention};

use custody::{Debit, Usage};

mod watches;

use watches::{WatchSlot, Watches};

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
    Leaf(Watches),
    Replacement(Box<Replacement>),
    Branch(Box<Branch>),
}

// A single slot can hold both generations until canonical publication.
// The old watch can still disarm its exact row before the notice is delivered.
struct Replacement {
    retiring: Watches,
    created: Watches,
    _backing: Debit,
}

struct Branch {
    bit: usize,
    children: [Option<Row>; 2],
    _backing: Debit,
}

fn side(slot: usize, bit: usize) -> usize {
    usize::from(slot & bit != 0)
}

enum ArmPath {
    Splice(usize),
    Armed(WatchSlot),
}

struct Disarmed {
    armed: Box<Armed>,
    empty: Option<Watches>,
}

impl Row {
    fn match_path(
        &self,
        node: NodeId,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<(usize, &Watches)>, DomError> {
        let mut row = self;
        let mut depth = 0;
        loop {
            step()?;
            depth += 1;
            match row {
                Self::Leaf(watches) => {
                    return Ok((watches.node() == Some(node)).then_some((depth, watches)));
                }
                Self::Replacement(pair) => {
                    return Ok(pair.generation(node).map(|(_, watches)| (depth, watches)));
                }
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
    // An armed result means the child installed the watch and no more work may refuse.
    fn arm_below(
        root: &mut Option<Self>,
        admission: &WatchAdmission<'_>,
        publication: Option<&PublicationPlan>,
    ) -> Result<ArmPath, DomError> {
        admission.step()?;
        match root.as_mut().ok_or(DomError::RepresentationExhausted)? {
            Self::Leaf(watches) => {
                let existing = watches.node().ok_or(DomError::RepresentationExhausted)?;
                let differing = admission.node.slot ^ existing.slot;
                if let Some(bit) = differing.checked_ilog2() {
                    return Ok(ArmPath::Splice(1 << bit));
                }
                if existing == admission.node {
                    return watches.add(admission).map(ArmPath::Armed);
                }
                admission.replace(root, existing, publication)?;
                Ok(ArmPath::Armed(WatchSlot::FIRST))
            }
            Self::Replacement(_) => Err(DomError::NodeRetirementAlreadyWatched),
            Self::Branch(branch) => {
                let child = &mut branch.children[side(admission.node.slot, branch.bit)];
                let bit = match Self::arm_below(child, admission, publication)? {
                    armed @ ArmPath::Armed(_) => return Ok(armed),
                    ArmPath::Splice(bit) => bit,
                };
                admission.step()?;
                if branch.bit > bit {
                    admission.splice(child, Some(bit), publication)?;
                    Ok(ArmPath::Armed(WatchSlot::FIRST))
                } else {
                    Ok(ArmPath::Splice(bit))
                }
            }
        }
    }

    fn disarm(root: &mut Option<Self>, node: NodeId, slot: WatchSlot) -> Option<Disarmed> {
        match root.as_mut()? {
            Self::Leaf(watches) if watches.node() == Some(node) => {
                let armed = watches.disarm(slot)?;
                let empty = if watches.is_empty() {
                    match root.take() {
                        Some(Self::Leaf(watches)) => Some(watches),
                        _ => None,
                    }
                } else {
                    None
                };
                Some(Disarmed { armed, empty })
            }
            Self::Leaf(_) => None,
            Self::Replacement(pair) => {
                let (retiring, watches) = pair.generation_mut(node)?;
                let armed = watches.disarm(slot)?;
                let empty = if watches.is_empty() {
                    Self::collapse_pair(root, retiring)
                } else {
                    None
                };
                Some(Disarmed { armed, empty })
            }
            Self::Branch(branch) => {
                let index = side(node.slot, branch.bit);
                let removed = Self::disarm(&mut branch.children[index], node, slot);
                if removed.is_some() && branch.children[index].is_none() {
                    *root = branch.children[1 - index].take();
                }
                removed
            }
        }
    }

    fn take_node(root: &mut Option<Self>, node: NodeId) -> Option<Watches> {
        match root.as_mut()? {
            Self::Leaf(watches) if watches.node() == Some(node) => match root.take() {
                Some(Self::Leaf(watches)) => Some(watches),
                _ => None,
            },
            Self::Leaf(_) => None,
            Self::Replacement(pair) => {
                let (retiring, _) = pair.generation(node)?;
                Self::collapse_pair(root, retiring)
            }
            Self::Branch(branch) => {
                let index = side(node.slot, branch.bit);
                let removed = Self::take_node(&mut branch.children[index], node);
                if removed.is_some() && branch.children[index].is_none() {
                    *root = branch.children[1 - index].take();
                }
                removed
            }
        }
    }

    fn collapse_pair(root: &mut Option<Self>, remove_retiring: bool) -> Option<Watches> {
        let pair = match root.take() {
            Some(Self::Replacement(pair)) => pair,
            other => {
                *root = other;
                return None;
            }
        };
        let Replacement {
            retiring,
            created,
            _backing: backing,
        } = *pair;
        drop(backing);
        let (removed, remaining) = if remove_retiring {
            (retiring, created)
        } else {
            (created, retiring)
        };
        *root = Some(Self::Leaf(remaining));
        Some(removed)
    }
}

impl Replacement {
    fn generation(&self, node: NodeId) -> Option<(bool, &Watches)> {
        if self.retiring.node() == Some(node) {
            Some((true, &self.retiring))
        } else {
            (self.created.node() == Some(node)).then_some((false, &self.created))
        }
    }

    fn generation_mut(&mut self, node: NodeId) -> Option<(bool, &mut Watches)> {
        if self.retiring.node() == Some(node) {
            Some((true, &mut self.retiring))
        } else {
            (self.created.node() == Some(node)).then_some((false, &mut self.created))
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
    slot: WatchSlot,
}

impl Drop for DomNodeRetirementWatch {
    fn drop(&mut self) {
        // One administrative path per affine watch. Each branch tests a
        // strictly lower slot bit, so both descent and unwind have at most
        // usize::BITS branch visits. This also works after Source exhaustion.
        let removed = Row::disarm(&mut lock(&self.index.armed), self.node, self.slot);
        if let Some(Disarmed { armed, empty }) = removed {
            let released = *armed;
            drop(released);
            if let Some(empty) = empty {
                empty.release_empty();
            }
        }
    }
}

impl Index {
    pub(super) fn prepare_match(
        &self,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        let work = {
            let root = lock(&self.armed);
            match root.as_ref() {
                Some(row) => row
                    .match_path(node, &mut step)?
                    .map(|(depth, watches)| (depth, watches.delivery_work())),
                None => {
                    step()?;
                    None
                }
            }
        };
        let Some((depth, (visits, notices))) = work else {
            return Ok(false);
        };
        // Admit the removal descent, branch updates and append. A new arm
        // in the same ordinary publication admits any additional branch.
        for _ in 0..depth {
            step()?;
            step()?;
        }
        for _ in 0..visits {
            step()?;
        }
        for _ in 0..notices {
            step()?;
        }
        Ok(true)
    }

    fn publish(&self, nodes: Vec<NodeId>) {
        for node in nodes {
            let watches = Row::take_node(&mut lock(&self.armed), node);
            if let Some(watches) = watches {
                watches.publish();
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
        let bytes = leaf
            .checked_add(notice)
            .and_then(|bytes| bytes.checked_add(extra))
            .ok_or(DomError::RepresentationExhausted)?;
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

    fn splice(
        &self,
        root: &mut Option<Row>,
        bit: Option<usize>,
        publication: Option<&PublicationPlan>,
    ) -> Result<(), DomError> {
        self.step()?;
        if bit.is_some()
            && let Some(publication) = publication
        {
            // One new branch can add at most one descent and one unwind
            // visit to each pending exact removal.
            for _ in &publication.nodes {
                self.step()?;
                self.step()?;
            }
        }
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
            branch.children[index] = Some(Row::Leaf(Watches::one(armed)));
            Row::Branch(branch)
        } else {
            Row::Leaf(Watches::one(armed))
        });
        Ok(())
    }

    fn replace(
        &self,
        root: &mut Option<Row>,
        existing: NodeId,
        publication: Option<&PublicationPlan>,
    ) -> Result<(), DomError> {
        let publication = publication
            .filter(|_| existing != self.node)
            .ok_or(DomError::NodeRetirementAlreadyWatched)?;
        for &retiring in &publication.nodes {
            self.step()?;
            if retiring != existing {
                continue;
            }
            let bytes = size_of::<Replacement>();
            let (created, mut retained) = self.form(bytes)?;
            let backing = retained.take(bytes)?;
            drop(retained);
            check_cancel(self.cancel)?;
            let retiring = match root.take() {
                Some(Row::Leaf(retiring)) => retiring,
                other => {
                    *root = other;
                    return Err(DomError::RepresentationExhausted);
                }
            };
            *root = Some(Row::Replacement(Box::new(Replacement {
                retiring,
                created: Watches::one(created),
                _backing: backing,
            })));
            return Ok(());
        }
        Err(DomError::NodeRetirementAlreadyWatched)
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
    /// with this node's actual canonical destruction. An inbox can arm one
    /// watch per exact node generation.
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
        let prepared = arm(self.node_retirements.as_ref(), &admission, None)?;
        Ok(self.finish_watch(prepared, node))
    }

    /// Watch the exact live generation during an ordinary DOM operation.
    /// Detachment and adoption preserve the watch. Only committed destruction
    /// publishes the original key to the original inbox. Distinct inboxes
    /// can watch the same generation independently.
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
        let admission =
            self.ordinary_watch_admission(&self.state, node, inbox, custody_key, limits, cancel)?;
        let prepared = arm(self.node_retirements.as_ref(), &admission, None)?;
        Ok(self.finish_watch(prepared, node))
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
            let admission = self.ordinary_watch_admission(
                &publication.state,
                node,
                inbox,
                custody_key,
                limits,
                cancel,
            )?;
            let prepared = arm(
                self.node_retirements.as_ref(),
                &admission,
                publication.node_retirement_publication.as_ref(),
            )?;
            // Index insertion ends admission. Publication cannot refuse.
            let watch = self.finish_watch(prepared, node);
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

    fn ordinary_watch_admission<'a>(
        &'a self,
        state: &State,
        node: NodeId,
        inbox: &'a DomNodeRetirementInbox,
        custody_key: usize,
        limits: DomNodeRetirementLimits,
        cancel: &'a AtomicBool,
    ) -> Result<WatchAdmission<'a>, DomError> {
        let ordinary = OrdinaryAdmission::new(self, limits, cancel);
        ordinary.admit_work(1)?;
        state.node_with(self.owner, node, &mut || ordinary.admit_work(1))?;
        Ok(WatchAdmission {
            funding: Funding::Ordinary(ordinary),
            inbox,
            node,
            custody_key,
            cancel,
        })
    }

    fn finish_watch(&mut self, prepared: PreparedWatch, node: NodeId) -> DomNodeRetirementWatch {
        let PreparedWatch { index, slot } = prepared;
        if self.node_retirements.is_none() {
            self.node_retirements = Some(index.clone());
        }
        DomNodeRetirementWatch { index, node, slot }
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

struct PreparedWatch {
    index: Arc<Index>,
    slot: WatchSlot,
}

fn arm(
    current: Option<&Arc<Index>>,
    admission: &WatchAdmission<'_>,
    publication: Option<&PublicationPlan>,
) -> Result<PreparedWatch, DomError> {
    if let Some(index) = current {
        let mut root = lock(&index.armed);
        let slot = if root.is_some() {
            match Row::arm_below(&mut root, admission, publication)? {
                ArmPath::Armed(slot) => slot,
                ArmPath::Splice(bit) => {
                    admission.splice(&mut root, Some(bit), publication)?;
                    WatchSlot::FIRST
                }
            }
        } else {
            admission.splice(&mut root, None, publication)?;
            WatchSlot::FIRST
        };
        return Ok(PreparedWatch {
            index: index.clone(),
            slot,
        });
    }
    admission.step()?;
    let header = size_of::<Index>();
    let (armed, mut retained) = admission.form(header)?;
    let backing = retained.take(header)?;
    drop(retained);
    let index = Arc::new(Index {
        armed: Mutex::new(Some(Row::Leaf(Watches::one(armed)))),
        _backing: backing,
    });
    check_cancel(admission.cancel)?;
    Ok(PreparedWatch {
        index,
        slot: WatchSlot::FIRST,
    })
}

#[cfg(test)]
mod tests;
