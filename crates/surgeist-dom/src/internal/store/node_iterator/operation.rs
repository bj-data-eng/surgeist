use super::*;
use crate::internal::source::callback::CallbackWork;
use crate::{
    internal::node_iterator::DomNodeFilterDecision,
    internal::node_iterator::DomNodeIteratorProgress,
};
use std::cell::Cell;
use std::sync::Weak;
use traversal::Direction;

enum Work<A> {
    Finite { remaining: Cell<u64>, limit: u64 },
    Accounted(A),
}

impl<A, E> Work<A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn step(&self, cancel: &AtomicBool) -> Result<(), E> {
        check_cancel(cancel)?;
        match self {
            Self::Finite { remaining, limit } => {
                let next = remaining
                    .get()
                    .checked_sub(1)
                    .ok_or(DomError::NodeIteratorWorkLimit { limit: *limit })?;
                remaining.set(next);
                Ok(())
            }
            Self::Accounted(admit) => admit(),
        }
    }
}

struct Tree<'a, A> {
    dom: &'a Dom,
    work: &'a Work<A>,
    source: Option<&'a CallbackWork>,
    cancel: &'a AtomicBool,
}

impl<A, E> Tree<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn step(&self) -> Result<(), E> {
        self.work.step(self.cancel)?;
        if let Some(source) = self.source {
            source.step(self.cancel)?;
        }
        Ok(())
    }

    fn node(&self, id: NodeId) -> Result<&Node, E> {
        self.dom
            .state
            .node_with(self.dom.owner, id, &mut || self.step())
    }

    fn matching(
        &self,
        cursor: &Cursor,
        mut pointer: Pointer<NodeId>,
        direction: Direction,
        active: bool,
    ) -> Result<Option<Pointer<NodeId>>, E> {
        loop {
            self.step()?;
            let Some(next) = traversal::advance(self, cursor.root, pointer, direction)? else {
                return Ok(None);
            };
            if active {
                return Err(DomError::NodeIteratorFilterActive.into());
            }
            let bit = match self.node(next.node)?.kind() {
                DomNodeKind::Element(_) => 0,
                DomNodeKind::Attr(_) => 1,
                DomNodeKind::Text(_) => 2,
                DomNodeKind::ProcessingInstruction(_) => 6,
                DomNodeKind::Comment(_) => 7,
                DomNodeKind::Document(_) => 8,
                DomNodeKind::DocumentType(_) => 9,
                DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => 10,
            };
            if cursor.mask & (1 << bit) != 0 {
                return Ok(Some(next));
            }
            pointer = next;
        }
    }
}

impl<A, E> OrdinaryTree<NodeId> for Tree<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    type Error = E;

    fn position(&self, node: NodeId) -> Result<Option<(NodeId, usize)>, E> {
        self.step()?;
        let Some(position) = self.dom.state.slot_index.position(Key::Existing(node)) else {
            return Ok(None);
        };
        let Key::Existing(parent) = position.parent else {
            return Err(DomError::NodeIteratorPublicationUnavailable.into());
        };
        Ok(Some((parent, position.ordinal)))
    }

    fn child(&self, node: NodeId, ordinal: usize) -> Result<Option<NodeId>, E> {
        Ok(self.node(node)?.children().get(ordinal).copied())
    }

    fn last_child(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        Ok(self.node(node)?.children().last().copied())
    }
}

struct Activation {
    callback: Callback,
    _custody: bank::ObjectCustody,
    invocation: u64,
}

impl Activation {
    fn new(callback: Callback, custody: bank::ObjectCustody) -> Result<Self, DomError> {
        let invocation = callback
            .0
            .invocation
            .try_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                (value.is_multiple_of(2) && value < u64::MAX - 1).then(|| value + 1)
            })
            .map_err(|value| {
                if value == u64::MAX - 1 {
                    DomError::RepresentationExhausted
                } else {
                    DomError::NodeIteratorFilterActive
                }
            })?
            + 1;
        Ok(Self {
            callback,
            _custody: custody,
            invocation,
        })
    }
}

impl Drop for Activation {
    fn drop(&mut self) {
        let _ = self.callback.0.invocation.compare_exchange(
            self.invocation,
            self.invocation + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

/// An unfiltered result awaiting caller output preparation before cursor publication.
///
/// Dropping this request leaves the cursor unchanged. Acceptance requires the
/// original canonical publication and Source callback; projection may register
/// external observers but cannot publish canonical edits before acceptance.
#[must_use]
pub struct DomNodeIteratorMatchRequest<A = fn() -> Result<(), DomError>> {
    id: DomNodeIteratorId,
    pointer: Pointer<NodeId>,
    work: Work<A>,
    source: Option<Weak<CallbackWork>>,
    publication: Weak<State>,
}

impl<A> std::fmt::Debug for DomNodeIteratorMatchRequest<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomNodeIteratorMatchRequest")
            .field("iterator", &self.id)
            .field("node", &self.pointer.node)
            .finish_non_exhaustive()
    }
}

impl<A> DomNodeIteratorMatchRequest<A> {
    pub fn iterator(&self) -> DomNodeIteratorId {
        self.id
    }

    pub fn node(&self) -> NodeId {
        self.pointer.node
    }

    /// Admits acceptance and publishes the prepared result's cursor position.
    pub fn accept<E>(self, dom: &mut Dom, cancel: &AtomicBool) -> Result<NodeId, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if dom.owner != self.id.owner {
            return Err(DomError::ForeignOwner.into());
        }
        dom.require_source_entry(cancel)?;
        let source = dom.dispatch_hosted_read_work(cancel)?;
        require_same_source(&self.source, source.as_ref())?;
        if !Weak::ptr_eq(&self.publication, &Arc::downgrade(&dom.state)) {
            return Err(DomError::NodeIteratorPublicationUnavailable.into());
        }
        let mut cursor = dom.state.node_iterators.cursor(dom.owner, self.id)?.clone();
        Tree {
            dom,
            work: &self.work,
            source: source.as_deref(),
            cancel,
        }
        .step()?;
        cursor.reference = staged_pointer(self.pointer);
        dom.publish_node_iterator(self.id, cursor, cancel)?;
        Ok(self.pointer.node)
    }
}

fn require_same_source(
    expected: &Option<Weak<CallbackWork>>,
    active: Option<&Arc<CallbackWork>>,
) -> Result<(), DomError> {
    let same_scope = match (expected, active) {
        (None, None) => true,
        (Some(expected), Some(active)) => expected
            .upgrade()
            .is_some_and(|expected| Arc::ptr_eq(&expected, active)),
        _ => false,
    };
    if same_scope {
        Ok(())
    } else {
        Err(DomError::InvalidSourceEvent)
    }
}

/// The sole response authority for a reached filter call. Dropping it ends the
/// traversal and leaves its last accepted reference and any tree edits intact.
#[must_use]
pub struct DomNodeIteratorFilterRequest<A = fn() -> Result<(), DomError>> {
    id: DomNodeIteratorId,
    node: NodeId,
    direction: Direction,
    work: Work<A>,
    source: Option<Weak<CallbackWork>>,
    activation: Activation,
}

impl<A> std::fmt::Debug for DomNodeIteratorFilterRequest<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomNodeIteratorFilterRequest")
            .field("iterator", &self.id)
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl<A> DomNodeIteratorFilterRequest<A> {
    pub fn iterator(&self) -> DomNodeIteratorId {
        self.id
    }

    /// The exact node passed to the callback. Callback edits may relocate the
    /// candidate pointer independently of this argument.
    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn respond<E>(
        mut self,
        dom: &mut Dom,
        decision: DomNodeFilterDecision,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress<A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if dom.owner != self.id.owner {
            return Err(DomError::ForeignOwner.into());
        }
        dom.require_source_entry(cancel)?;
        let source = dom.dispatch_hosted_read_work(cancel)?;
        require_same_source(&self.source, source.as_ref())?;
        let mut cursor = dom.state.node_iterators.cursor(dom.owner, self.id)?.clone();
        let candidate = cursor
            .live_candidate()
            .filter(|candidate| candidate.invocation == self.activation.invocation)
            .ok_or(DomError::StaleNodeIterator)?;
        let tree = Tree {
            dom,
            work: &self.work,
            source: source.as_deref(),
            cancel,
        };
        if decision == DomNodeFilterDecision::Accept {
            tree.step()?;
            cursor.reference = candidate.pointer;
            cursor.candidate = None;
            dom.publish_node_iterator(self.id, cursor, cancel)?;
            return Ok(DomNodeIteratorProgress::Complete(Some(self.node)));
        }
        let Some(next) = tree.matching(
            &cursor,
            public_pointer(candidate.pointer)?,
            self.direction,
            false,
        )?
        else {
            return Ok(DomNodeIteratorProgress::Complete(None));
        };
        self.node = next.node;
        cursor.candidate = Some(Candidate {
            pointer: staged_pointer(next),
            ..candidate
        });
        dom.publish_node_iterator(self.id, cursor, cancel)?;
        Ok(DomNodeIteratorProgress::Filter(self))
    }
}

fn public_pointer(pointer: Pointer<Key>) -> Result<Pointer<NodeId>, DomError> {
    let Key::Existing(node) = pointer.node else {
        return Err(DomError::NodeIteratorPublicationUnavailable);
    };
    Ok(Pointer {
        node,
        before: pointer.before,
    })
}

fn staged_pointer(pointer: Pointer<NodeId>) -> Pointer<Key> {
    Pointer {
        node: Key::Existing(pointer.node),
        before: pointer.before,
    }
}

impl Dom {
    /// Creates a live iterator rooted at this owner's exact node generation.
    pub fn create_node_iterator(
        &mut self,
        root: NodeId,
        what_to_show: u32,
        filter: DomNodeIteratorFilter,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorId, DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.create_node_iterator(root, what_to_show, filter, cancel)
        })
    }

    /// Retires the native identity and releases its root membership.
    pub fn retire_node_iterator(
        &mut self,
        id: DomNodeIteratorId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.node_iterators.value.cursor(draft.store, id)?;
            draft.retire_iterator_slot(id.slot, cancel)
        })
    }

    pub fn node_iterator_next(
        &mut self,
        id: DomNodeIteratorId,
        work_limit: u64,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress, DomError> {
        self.advance_node_iterator(
            id,
            Direction::Next,
            Work::Finite {
                limit: work_limit,
                remaining: Cell::new(work_limit),
            },
            cancel,
        )
    }

    pub fn node_iterator_previous(
        &mut self,
        id: DomNodeIteratorId,
        work_limit: u64,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress, DomError> {
        self.advance_node_iterator(
            id,
            Direction::Previous,
            Work::Finite {
                limit: work_limit,
                remaining: Cell::new(work_limit),
            },
            cancel,
        )
    }

    /// Advances with caller-owned admission before each reached traversal step.
    ///
    /// The same admission remains owned by any returned filter request and is
    /// used on every response. Its first failure is returned unchanged, before
    /// that step runs. An unfiltered match waits for output preparation and
    /// explicit acceptance before publishing its cursor. Native Source work is
    /// charged separately by DOM; admission must not charge that same Source
    /// account again.
    pub fn node_iterator_next_accounted<A, E>(
        &mut self,
        id: DomNodeIteratorId,
        admission: A,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress<A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.advance_node_iterator(id, Direction::Next, Work::Accounted(admission), cancel)
    }

    /// Moves backward with the same admission contract as [`Self::node_iterator_next_accounted`].
    pub fn node_iterator_previous_accounted<A, E>(
        &mut self,
        id: DomNodeIteratorId,
        admission: A,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress<A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.advance_node_iterator(id, Direction::Previous, Work::Accounted(admission), cancel)
    }

    fn advance_node_iterator<A, E>(
        &mut self,
        id: DomNodeIteratorId,
        direction: Direction,
        work: Work<A>,
        cancel: &AtomicBool,
    ) -> Result<DomNodeIteratorProgress<A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        self.require_source_entry(cancel)?;
        let source = self.dispatch_hosted_read_work(cancel)?;
        let mut cursor = self.state.node_iterators.cursor(self.owner, id)?.clone();
        let tree = Tree {
            dom: self,
            work: &work,
            source: source.as_deref(),
            cancel,
        };
        let active = cursor.live_candidate().is_some();
        let Some(next) = tree.matching(
            &cursor,
            public_pointer(cursor.reference)?,
            direction,
            active,
        )?
        else {
            return Ok(DomNodeIteratorProgress::Complete(None));
        };
        if let Some(callback) = cursor.callback.clone() {
            let controls = &self.state_custody.node_iterators.slots;
            let slot = self
                .state
                .node_iterators
                .slots
                .controlled_record(controls, id.slot)
                .ok_or(DomError::StaleNodeIterator)?;
            let custody = slot
                .dependency::<bank::ObjectCustody>()
                .ok_or(DomError::StaleNodeIterator)?;
            let activation = Activation::new(callback, (*custody).clone())?;
            cursor.candidate = Some(Candidate {
                invocation: activation.invocation,
                pointer: staged_pointer(next),
            });
            self.publish_node_iterator(id, cursor, cancel)?;
            Ok(DomNodeIteratorProgress::Filter(
                DomNodeIteratorFilterRequest {
                    id,
                    node: next.node,
                    direction,
                    work,
                    source: source.as_ref().map(Arc::downgrade),
                    activation,
                },
            ))
        } else if matches!(&work, Work::Accounted(_)) {
            Ok(DomNodeIteratorProgress::Match(
                DomNodeIteratorMatchRequest {
                    id,
                    pointer: next,
                    work,
                    source: source.as_ref().map(Arc::downgrade),
                    publication: Arc::downgrade(&self.state),
                },
            ))
        } else {
            cursor.reference = staged_pointer(next);
            self.publish_node_iterator(id, cursor, cancel)?;
            Ok(DomNodeIteratorProgress::Complete(Some(next.node)))
        }
    }

    fn publish_node_iterator(
        &mut self,
        id: DomNodeIteratorId,
        cursor: Cursor,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.write_node_iterator(id, cursor, cancel)
        })
    }
}
