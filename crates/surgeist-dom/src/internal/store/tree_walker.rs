//! Canonical TreeWalker navigation, suspension and accepted output.

mod navigation;
pub(super) mod storage;

use super::*;
use crate::internal::source::callback::CallbackWork;
use crate::internal::tree_walker::{
    DomTreeWalker, DomTreeWalkerDirection, DomTreeWalkerFilter, DomTreeWalkerId,
    DomTreeWalkerProgress,
};
use navigation::{ACCEPT, OrdinaryTree, SKIP};
use std::sync::Weak;
use storage::{Activation, Callback, Cursor, activate_tree_walker};

struct Walk<'cancel, A> {
    id: DomTreeWalkerId,
    admission: A,
    cancellation: &'cancel AtomicBool,
    source: Option<Weak<CallbackWork>>,
}

struct Read<'a, A> {
    dom: &'a Dom,
    admission: &'a A,
    cancellation: &'a AtomicBool,
    source: Option<&'a CallbackWork>,
}

impl<A, E> Read<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn node(&self, id: NodeId) -> Result<&Node, E> {
        self.dom
            .state
            .node_with(self.dom.owner, id, &mut || self.step())
    }

    fn sibling(&self, id: NodeId, forward: bool) -> Result<Option<NodeId>, E> {
        self.node(id)?;
        let Some(position) = self
            .dom
            .state
            .slot_index
            .position_with(Key::Existing(id), &mut || self.step())?
        else {
            return Ok(None);
        };
        let Key::Existing(parent) = position.parent else {
            return Err(DomError::TreeWalkerPublicationUnavailable.into());
        };
        let ordinal = if forward {
            Some(
                position
                    .ordinal
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
        } else {
            position.ordinal.checked_sub(1)
        };
        match ordinal {
            Some(ordinal) => Ok(self.node(parent)?.children().get(ordinal).copied()),
            None => Ok(None),
        }
    }
}

impl<A, E> OrdinaryTree for Read<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    type Error = E;

    fn step(&self) -> Result<(), E> {
        check_cancel(self.cancellation)?;
        (self.admission)()?;
        if let Some(source) = self.source {
            source.step(self.cancellation)?;
        }
        Ok(())
    }

    fn parent(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        Ok(self.node(node)?.parent())
    }

    fn first_child(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        Ok(self.node(node)?.children().first().copied())
    }

    fn last_child(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        Ok(self.node(node)?.children().last().copied())
    }

    fn previous_sibling(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        self.sibling(node, false)
    }

    fn next_sibling(&self, node: NodeId) -> Result<Option<NodeId>, E> {
        self.sibling(node, true)
    }
}

impl<'cancel, A, E> Walk<'cancel, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn read<'a>(&'a self, dom: &'a Dom, source: Option<&'a CallbackWork>) -> Read<'a, A> {
        Read {
            dom,
            admission: &self.admission,
            cancellation: self.cancellation,
            source,
        }
    }

    fn qualify(&self, dom: &Dom) -> Result<Option<Arc<CallbackWork>>, DomError> {
        if dom.owner != self.id.owner {
            return Err(DomError::ForeignOwner);
        }
        dom.require_source_entry(self.cancellation)?;
        let source = dom.dispatch_hosted_read_work(self.cancellation)?;
        CallbackWork::require_same_scope(&self.source, source.as_ref())?;
        Ok(source)
    }

    fn drive(
        self,
        dom: &Dom,
        source: Option<&CallbackWork>,
        cursor: &Cursor,
        mut progress: navigation::Progress,
    ) -> Result<DomTreeWalkerProgress<'cancel, A>, E> {
        let read = self.read(dom, source);
        let current = cursor.current;
        loop {
            match progress {
                navigation::Progress::None => return Ok(DomTreeWalkerProgress::NoMatch),
                navigation::Progress::Match(node) => {
                    read.node(node)?;
                    return Ok(DomTreeWalkerProgress::Match(DomTreeWalkerMatchRequest {
                        walk: self,
                        node,
                        publication: Arc::downgrade(&dom.state),
                    }));
                }
                navigation::Progress::Filter(pending) => {
                    read.step()?;
                    if cursor.callback.as_ref().is_some_and(Callback::is_active) {
                        return Err(DomError::TreeWalkerFilterActive.into());
                    }
                    let eligible =
                        cursor.mask & read.node(pending.node())?.kind().traversal_mask() != 0;
                    if eligible && cursor.callback.is_some() {
                        let active = activate_tree_walker(dom, self.id)?;
                        return Ok(DomTreeWalkerProgress::Filter(DomTreeWalkerFilterRequest {
                            walk: self,
                            pending,
                            active,
                        }));
                    }
                    let decision = if eligible { ACCEPT } else { SKIP };
                    progress = pending.resume(&read, current, decision)?;
                }
            }
        }
    }
}

/// The sole response authority for a reached TreeWalker filter call.
///
/// The caller can edit the DOM before responding. Dropping this request clears
/// the live filter state and leaves the current node and any callback edits intact.
#[must_use]
pub struct DomTreeWalkerFilterRequest<'cancel, A> {
    walk: Walk<'cancel, A>,
    pending: navigation::Filter,
    active: Activation,
}

impl<A> std::fmt::Debug for DomTreeWalkerFilterRequest<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomTreeWalkerFilterRequest")
            .field("walker", &self.walk.id)
            .field("node", &self.pending.node())
            .finish_non_exhaustive()
    }
}

impl<'cancel, A> DomTreeWalkerFilterRequest<'cancel, A> {
    pub fn walker(&self) -> DomTreeWalkerId {
        self.walk.id
    }

    pub fn node(&self) -> NodeId {
        self.pending.node()
    }

    /// Resume after the callback's Web IDL unsigned-short conversion.
    pub fn respond<E>(
        self,
        dom: &Dom,
        decision: u16,
    ) -> Result<DomTreeWalkerProgress<'cancel, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let Self {
            walk,
            pending,
            active,
        } = self;
        let source = walk.qualify(dom)?;
        let cursor = dom.state.tree_walkers.cursor(dom.owner, walk.id)?;
        let current = cursor.current;
        let read = walk.read(dom, source.as_deref());
        drop(active);
        let progress = pending.resume(&read, current, decision)?;
        walk.drive(dom, source.as_deref(), cursor, progress)
    }
}

/// An accepted result awaiting output preparation before cursor publication.
///
/// Dropping preserves the current node. Acceptance requires the original
/// publication and callback scope. Output preparation can retain canonical
/// observations but cannot publish intervening edits.
#[must_use]
pub struct DomTreeWalkerMatchRequest<'cancel, A> {
    walk: Walk<'cancel, A>,
    node: NodeId,
    publication: Weak<State>,
}

impl<A> std::fmt::Debug for DomTreeWalkerMatchRequest<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomTreeWalkerMatchRequest")
            .field("walker", &self.walk.id)
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

impl<A> DomTreeWalkerMatchRequest<'_, A> {
    pub fn walker(&self) -> DomTreeWalkerId {
        self.walk.id
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    /// Publish this prepared result as the walker's current node.
    pub fn accept<E>(self, dom: &mut Dom) -> Result<NodeId, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let source = self.walk.qualify(dom)?;
        if !self.publication.ptr_eq(&Arc::downgrade(&dom.state)) {
            return Err(DomError::TreeWalkerPublicationUnavailable.into());
        }
        self.walk.read(dom, source.as_deref()).step()?;
        dom.set_tree_walker_current(self.walk.id, self.node, self.walk.cancellation)?;
        Ok(self.node)
    }
}

impl Dom {
    /// Create a live walker with its current node initially at the root.
    pub fn create_tree_walker(
        &mut self,
        root: NodeId,
        what_to_show: u32,
        filter: DomTreeWalkerFilter,
        cancel: &AtomicBool,
    ) -> Result<DomTreeWalkerId, DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.create_tree_walker(root, what_to_show, filter, cancel)
        })
    }

    /// Observe the published state of the exact live walker.
    pub fn tree_walker(&self, id: DomTreeWalkerId) -> Result<DomTreeWalker, DomError> {
        let cursor = self.state.tree_walkers.cursor(self.owner, id)?;
        Ok(DomTreeWalker {
            id,
            root: cursor.root,
            current: cursor.current,
            mask: cursor.mask,
            filter: if cursor.callback.is_some() {
                DomTreeWalkerFilter::Callback
            } else {
                DomTreeWalkerFilter::None
            },
        })
    }

    /// Select any exact live node in this owner, including outside the root.
    pub fn set_tree_walker_current(
        &mut self,
        id: DomTreeWalkerId,
        node: NodeId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.set_tree_walker_current(id, node, cancel)
        })
    }

    /// Retire this native walker identity and release its node memberships.
    pub fn retire_tree_walker(
        &mut self,
        id: DomTreeWalkerId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| draft.retire_tree_walker(id, cancel))
    }

    /// Navigate with caller admission before each reached step.
    ///
    /// A suspended traversal retains this exact admission and cancellation.
    /// DOM charges the live native Source scope separately. On `Match`, prepare
    /// the result and accept it before exposing the new current node.
    pub fn navigate_tree_walker<'cancel, A, E>(
        &self,
        id: DomTreeWalkerId,
        direction: DomTreeWalkerDirection,
        admission: A,
        cancellation: &'cancel AtomicBool,
    ) -> Result<DomTreeWalkerProgress<'cancel, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if self.owner != id.owner {
            return Err(DomError::ForeignOwner.into());
        }
        self.require_source_entry(cancellation)?;
        let source = self.dispatch_hosted_read_work(cancellation)?;
        let walk = Walk {
            id,
            admission,
            cancellation,
            source: source.as_ref().map(Arc::downgrade),
        };
        let read = walk.read(self, source.as_deref());
        read.step()?;
        let cursor = self.state.tree_walkers.cursor(self.owner, id)?;
        let progress = navigation::start(&read, cursor.root, cursor.current, direction)?;
        walk.drive(self, source.as_deref(), cursor, progress)
    }
}
