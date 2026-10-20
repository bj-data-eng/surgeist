//! Native fullscreen model facts, distinct from requestFullscreen admission.
use crate::{DocumentId, DomChanges, DomDocumentStamp, DomOwnerId};
use crate::{DomDocument, DomError, DomShadowRoot, DomSnapshot, NodeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomFullscreenExitId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenExitReturn {
    RejectedTypeError,
    Pending(DomFullscreenExitId),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenExitStep {
    FullyUnlockScreenOrientation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenExitFact {
    NodeNavigable,
    Container,
    TopLevelDocument,
}
/// Owned observation, without execution or service-fulfillment authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomFullscreenExitStatus {
    pub(crate) id: DomFullscreenExitId,
    pub(crate) document: DocumentId,
    pub(crate) resize: bool,
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) command: usize,
    pub(crate) occurrence: u64,
}
impl DomFullscreenExitStatus {
    pub const fn id(self) -> DomFullscreenExitId {
        self.id
    }
    pub const fn document(self) -> DocumentId {
        self.document
    }
    pub const fn resize(self) -> bool {
        self.resize
    }
    pub const fn operation(self) -> crate::DomSourceOperationId {
        self.operation
    }
    pub const fn command(self) -> usize {
        self.command
    }
    pub const fn current_occurrence(self) -> u64 {
        self.occurrence
    }
    pub const fn step(self) -> DomFullscreenExitStep {
        DomFullscreenExitStep::FullyUnlockScreenOrientation
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenEventKind {
    Change,
}
/// Historical tuple. Its subject is never remapped through a reused slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomFullscreenEvent {
    pub(crate) document: DocumentId,
    pub(crate) subject: NodeId,
    pub(crate) kind: DomFullscreenEventKind,
}
impl DomFullscreenEvent {
    pub const fn document(self) -> DocumentId {
        self.document
    }
    pub const fn subject(self) -> NodeId {
        self.subject
    }
    pub const fn kind(self) -> DomFullscreenEventKind {
        self.kind
    }
}
#[derive(Debug)]
pub struct DomFullscreenEventBatch {
    pub(crate) basis: DomDocumentStamp,
    pub(crate) events: Vec<DomFullscreenEvent>,
    pub(crate) changes: DomChanges,
}
impl DomFullscreenEventBatch {
    pub fn basis(&self) -> DomDocumentStamp {
        self.basis
    }
    pub fn events(&self) -> &[DomFullscreenEvent] {
        &self.events
    }
    pub fn changes(&self) -> &DomChanges {
        &self.changes
    }
}
impl<'a> DomDocument<'a> {
    pub fn pending_fullscreen_events(self) -> Result<&'a [DomFullscreenEvent], DomError> {
        self.snapshot.require_complete_source()?;
        Ok(self.snapshot.node(self.id.node())?.node.fullscreen_events())
    }
}

/// The first reached fullscreen model action without an actual implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFullscreenModelStep {
    ReadTopLayer,
    ExitFullscreen,
    FullyExitFullscreen,
}

impl DomDocument<'_> {
    /// The last flagged raw member, including detached native membership.
    /// This is not the standard retargeted fullscreenElement getter.
    pub fn fullscreen_model_element(self) -> Result<Option<NodeId>, DomError> {
        for &id in self.top_layer()?.raw_members().iter().rev() {
            if self
                .snapshot
                .node(id)?
                .as_element()
                .ok_or(DomError::WrongNodeKind)?
                .fullscreen_flag()
            {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }
    /// Legacy fullscreen reflects the raw model result, independently of retargeting.
    pub fn fullscreen(self) -> Result<bool, DomError> {
        Ok(self.fullscreen_model_element()?.is_some())
    }
    pub fn is_simple_fullscreen_document(self) -> Result<bool, DomError> {
        let mut count = 0;
        for &id in self.top_layer()?.raw_members() {
            if self
                .snapshot
                .node(id)?
                .as_element()
                .ok_or(DomError::WrongNodeKind)?
                .fullscreen_flag()
            {
                count += 1;
                if count > 1 {
                    return Ok(false);
                }
            }
        }
        Ok(count == 1)
    }
    pub fn keyboard_lock_active(self) -> Result<bool, DomError> {
        match self.fullscreen_model_element()? {
            Some(id) => Ok(self
                .snapshot
                .node(id)?
                .as_element()
                .ok_or(DomError::WrongNodeKind)?
                .keyboard_lock_flag()),
            None => Ok(false),
        }
    }
    /// Standard Document fullscreenElement: retarget, then require the same tree.
    pub fn fullscreen_element(self) -> Result<Option<NodeId>, DomError> {
        retarget_fullscreen(
            self.snapshot,
            self.id.node(),
            self.fullscreen_model_element()?,
        )
    }
}
impl DomShadowRoot<'_> {
    /// Standard ShadowRoot fullscreenElement, including closed shadow trees.
    pub fn fullscreen_element(self) -> Result<Option<NodeId>, DomError> {
        let node = self.node();
        let snapshot = node.snapshot;
        snapshot.require_complete_source()?;
        let document = snapshot.document(node.owner_document().ok_or(DomError::WrongDocument)?)?;
        let candidate = document.fullscreen_model_element()?;
        if !snapshot.node(self.host())?.is_connected() {
            return Ok(None);
        }
        retarget_fullscreen(snapshot, self.id(), candidate)
    }
}
fn retarget_fullscreen(
    snapshot: &DomSnapshot,
    against: NodeId,
    candidate: Option<NodeId>,
) -> Result<Option<NodeId>, DomError> {
    let Some(mut candidate) = candidate else {
        return Ok(None);
    };
    loop {
        let root = snapshot.node(candidate)?.root();
        let root = snapshot.node(root)?;
        let Some(shadow) = root.as_shadow_root() else {
            break;
        };
        if root.is_shadow_including_inclusive_ancestor_of(against)? {
            break;
        }
        candidate = shadow.host();
    }
    Ok((snapshot.node(candidate)?.root() == snapshot.node(against)?.root()).then_some(candidate))
}
