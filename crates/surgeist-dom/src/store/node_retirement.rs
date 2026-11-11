//! One-shot native node retirement notices for retained host objects.

use std::sync::{Mutex, MutexGuard};

use super::*;
use crate::{DomSourceEventPayloadRetention, DomSourceEventRequest};

struct Queue {
    open: bool,
    head: Option<Box<Notice>>,
    _backing: DomSourceEventPayloadRetention,
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
    _backing: DomSourceEventPayloadRetention,
}

impl DomNodeRetirementNotice {
    pub const fn node(&self) -> NodeId {
        self.0.node
    }

    pub const fn custody_key(&self) -> usize {
        self.0.custody_key
    }
}

struct Armed {
    notice: Box<Notice>,
    queue: Arc<Mutex<Queue>>,
    backing: DomSourceEventPayloadRetention,
}

enum Row {
    Leaf(Box<Armed>),
    Branch(Box<Branch>),
}

struct Branch {
    bit: usize,
    children: [Option<Row>; 2],
    _backing: DomSourceEventPayloadRetention,
}

fn side(slot: usize, bit: usize) -> usize {
    usize::from(slot & bit != 0)
}

impl Row {
    fn leaf(&self, slot: usize) -> Option<&Armed> {
        let mut row = self;
        loop {
            match row {
                Self::Leaf(armed) => return Some(armed),
                Self::Branch(branch) => {
                    row = branch.children[side(slot, branch.bit)].as_ref()?;
                }
            }
        }
    }

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

    // The first differing bit separates two exact slot identities. Each
    // branch tests a lower bit than its parent, so the path has fixed depth.
    fn insert(
        root: &mut Option<Self>,
        row: Box<Armed>,
        bit: usize,
        backing: DomSourceEventPayloadRetention,
    ) {
        let slot = row.notice.node.slot;
        if let Some(Self::Branch(branch)) = root
            && branch.bit > bit
        {
            Self::insert(
                &mut branch.children[side(slot, branch.bit)],
                row,
                bit,
                backing,
            );
            return;
        }
        let index = side(slot, bit);
        let mut children = [None, None];
        children[1 - index] = root.take();
        children[index] = Some(Self::Leaf(row));
        *root = Some(Self::Branch(Box::new(Branch {
            bit,
            children,
            _backing: backing,
        })));
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
    _backing: DomSourceEventPayloadRetention,
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
            } = *armed;
            drop(backing);
            let mut queue = lock(&queue);
            if queue.open {
                notice.next = queue.head.take();
                queue.head = Some(notice);
            }
        }
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
            .split_payload(bytes, 0)?;
        Ok(DomNodeRetirementInbox {
            owner: self.owner,
            queue: Arc::new(Mutex::new(Queue {
                open: true,
                head: None,
                _backing: backing,
            })),
        })
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
        let reserve_branch = if let Some(index) = &self.node_retirements {
            let root = lock(&index.armed);
            if root
                .as_ref()
                .and_then(|row| row.leaf(node.slot))
                .is_some_and(|row| row.notice.node.slot == node.slot)
            {
                return Err(DomError::NodeRetirementAlreadyWatched);
            }
            root.is_some()
        } else {
            false
        };
        let leaf = size_of::<Armed>();
        let notice = size_of::<Notice>();
        let branch = if reserve_branch {
            size_of::<Branch>()
        } else {
            0
        };
        let header = if self.node_retirements.is_none() {
            size_of::<Index>()
        } else {
            0
        };
        let mut retained =
            self.retain_source_event_payload(request, leaf + notice + branch + header, 0, cancel)?;
        let leaf_backing = retained.split_payload(leaf, 0)?;
        let notice_backing = retained.split_payload(notice, 0)?;
        let index = match self.node_retirements.as_ref() {
            Some(index) => index.clone(),
            None => {
                let index = Arc::new(Index {
                    armed: Mutex::new(None),
                    _backing: retained.split_payload(header, 0)?,
                });
                self.node_retirements = Some(index.clone());
                index
            }
        };
        let notice = Box::new(Notice {
            node,
            custody_key,
            next: None,
            _backing: notice_backing,
        });
        let armed = Box::new(Armed {
            notice,
            queue: inbox.queue.clone(),
            backing: leaf_backing,
        });
        {
            let mut root = lock(&index.armed);
            if let Some(existing) = root.as_ref() {
                let leaf = existing
                    .leaf(node.slot)
                    .ok_or(DomError::RepresentationExhausted)?;
                let differing_bit = (node.slot ^ leaf.notice.node.slot)
                    .checked_ilog2()
                    .ok_or(DomError::NodeRetirementAlreadyWatched)?;
                Row::insert(
                    &mut root,
                    armed,
                    1 << differing_bit,
                    retained.split_payload(branch, 0)?,
                );
            } else {
                *root = Some(Row::Leaf(armed));
            }
        }
        Ok(DomNodeRetirementWatch {
            index,
            node,
            custody_key,
        })
    }
}
