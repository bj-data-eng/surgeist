//! Snapshot adapter for HTML's Document and native container focus chain.

use super::*;
use crate::internal::hosted::{DocumentPhase, DomHostedError, DomHostedNavigableId};
use crate::internal::snapshot::DomDocument;
use surgeist_html::focus::{
    HtmlFocusChainAction as Action, HtmlFocusChainCursor, HtmlFocusChainReply as Reply,
};

/// One exact Document or native navigable container in the captured chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFocusChainEntry {
    Document(DocumentId),
    NavigableContainer(NodeId),
}

/// Maximum retained entries and admitted read work for a focus-chain query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomFocusChainLimits {
    entries: usize,
    work: u64,
}

impl DomFocusChainLimits {
    pub const fn new(entries: usize, work: u64) -> Self {
        Self { entries, work }
    }
}

/// An immutable chain from its starting Document to the last reached Document.
#[derive(Debug)]
pub struct DomFocusChain {
    stamp: DomStamp,
    entries: Vec<DomFocusChainEntry>,
}

impl DomFocusChain {
    pub fn stamp(&self) -> DomStamp {
        self.stamp
    }
    pub fn entries(&self) -> &[DomFocusChainEntry] {
        &self.entries
    }
}

/// The first refusal of a captured focus-chain query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomFocusChainError {
    Dom(DomError),
    Hosted(DomHostedError),
    WorkLimit { limit: u64 },
    EntryLimit { limit: usize },
    RepeatedEntry(DomFocusChainEntry),
    UnavailableRole,
}

impl From<DomError> for DomFocusChainError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl From<DomHostedError> for DomFocusChainError {
    fn from(error: DomHostedError) -> Self {
        Self::Hosted(error)
    }
}

impl std::fmt::Display for DomFocusChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dom(error) => error.fmt(f),
            Self::Hosted(error) => error.fmt(f),
            Self::WorkLimit { limit } => write!(f, "focus chain work limit {limit} exceeded"),
            Self::EntryLimit { limit } => write!(f, "focus chain entry limit {limit} exceeded"),
            Self::RepeatedEntry(entry) => write!(f, "focus chain revisited {entry:?}"),
            Self::UnavailableRole => f.write_str("the reached focus-chain role is unavailable"),
        }
    }
}

impl std::error::Error for DomFocusChainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Hosted(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum Subject {
    Document(DocumentId),
    Container {
        element: NodeId,
        document: DocumentId,
    },
}

impl Subject {
    fn entry(self) -> DomFocusChainEntry {
        match self {
            Self::Document(id) => DomFocusChainEntry::Document(id),
            Self::Container { element, .. } => DomFocusChainEntry::NavigableContainer(element),
        }
    }
}

impl DomDocument<'_> {
    /// Resolve the Document's HTML focus chain from its exact captured graph.
    /// The chain describes historical relations and grants no focus execution authority.
    ///
    /// # Errors
    ///
    /// Refuses incomplete, retiring or inconsistent associations, cancellation,
    /// repeated entries, and exhausted work or entry allowances.
    pub fn focus_chain(
        self,
        limits: DomFocusChainLimits,
        cancel: &AtomicBool,
    ) -> Result<DomFocusChain, DomFocusChainError> {
        self.snapshot.require_complete_source()?;
        let mut query = Query {
            snapshot: self.snapshot,
            limits,
            remaining: limits.work,
            entries: Vec::new(),
            cancel,
        };
        query.resolve(self.id)?;
        Ok(DomFocusChain {
            stamp: self.snapshot.stamp(),
            entries: query.entries,
        })
    }
}

struct Query<'a> {
    snapshot: &'a DomSnapshot,
    limits: DomFocusChainLimits,
    remaining: u64,
    entries: Vec<DomFocusChainEntry>,
    cancel: &'a AtomicBool,
}

impl<'a> Query<'a> {
    fn step(&mut self) -> Result<(), DomFocusChainError> {
        self.admit(1)
    }

    fn admit(&mut self, work: u64) -> Result<(), DomFocusChainError> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        self.remaining = self
            .remaining
            .checked_sub(work)
            .ok_or(DomFocusChainError::WorkLimit {
                limit: self.limits.work,
            })?;
        Ok(())
    }

    fn append(&mut self, entry: DomFocusChainEntry) -> Result<(), DomFocusChainError> {
        for index in 0..self.entries.len() {
            self.step()?;
            if self.entries.get(index) == Some(&entry) {
                return Err(DomFocusChainError::RepeatedEntry(entry));
            }
        }
        let len = self.entries.len();
        let limit = self.limits.entries;
        if len == limit {
            return Err(DomFocusChainError::EntryLimit { limit });
        }
        if len == self.entries.capacity() {
            // Growth can relocate each currently retained entry.
            self.admit(u64::try_from(len).map_err(|_| DomError::RepresentationExhausted)?)?;
            let capacity = len.saturating_mul(2).max(4).min(limit);
            self.entries
                .try_reserve_exact(capacity - len)
                .map_err(|_| DomError::AllocationFailed)?;
            if self.entries.capacity() > limit {
                return Err(DomFocusChainError::EntryLimit { limit });
            }
        }
        self.entries.push(entry);
        Ok(())
    }

    fn active_navigable(
        &mut self,
        document: DocumentId,
    ) -> Result<Option<crate::internal::hosted::DomHostedNavigable<'a>>, DomFocusChainError> {
        let snapshot = self.snapshot;
        let node = snapshot.node_with(document.node(), || self.step())?;
        let DomNodeKind::Document(metadata) = node.kind() else {
            return Err(DomError::WrongDocument.into());
        };
        self.step()?;
        let environment = metadata.document_environment();
        let association = environment.hosted.as_deref();
        let active = match snapshot.state.hosted_history.as_deref() {
            Some(graph) => {
                graph.unique_active_navigable_for_document_with(document, || self.step())?
            }
            None => None,
        };
        let Some(association) = association else {
            return if active.is_none() {
                Ok(None)
            } else {
                Err(DomHostedError::StaleBasis.into())
            };
        };
        let expected = match association.phase {
            DocumentPhase::InitialEntry { navigable, .. } => navigable,
            DocumentPhase::Retired { .. } if active.is_none() => return Ok(None),
            DocumentPhase::Retired { .. } => return Err(DomHostedError::StaleBasis.into()),
            _ => return Err(DomHostedError::MissingPublication.into()),
        };
        if active != Some(expected) {
            return Err(DomHostedError::StaleBasis.into());
        }
        let row = snapshot.captured_hosted_navigable_with(expected, &mut || self.step())?;
        self.step()?;
        if row.active_document() != Some(document)
            || row.is_retiring()
            || row.is_retired()
            || row.host() != association.host
            || row.global() != association.global
            || row.browsing_context() != association.context
        {
            return Err(DomHostedError::StaleBasis.into());
        }
        Ok(Some(row))
    }

    fn container(
        &mut self,
        child: crate::internal::hosted::DomHostedNavigable<'a>,
        parent: DomHostedNavigableId,
    ) -> Result<Subject, DomFocusChainError> {
        let id = child
            .container()
            .ok_or(DomHostedError::MissingPublication)?;
        let node = self.snapshot.node_with(id, || self.step())?;
        let document = node.owner_document().ok_or(DomError::WrongNodeKind)?;
        let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
        self.step()?;
        let insertion = element
            .hosted_insertion()
            .ok_or(DomHostedError::MissingPublication)?;
        if insertion.document() != document
            || child.container_insertion() != Some(insertion)
            || element.child_navigable() != Some(Some(child.id()))
        {
            return Err(DomHostedError::StaleBasis.into());
        }
        let active_parent = self
            .active_navigable(document)?
            .ok_or(DomHostedError::StaleBasis)?;
        if active_parent.id() != parent
            || active_parent.host() != child.host()
            || active_parent.traversable() != child.traversable()
        {
            return Err(DomHostedError::StaleBasis.into());
        }
        Ok(Subject::Container {
            element: id,
            document,
        })
    }

    fn resolve(&mut self, start: DocumentId) -> Result<(), DomFocusChainError> {
        let mut cursor = HtmlFocusChainCursor::new();
        let mut subject = Subject::Document(start);
        let mut active = None;
        loop {
            self.step()?;
            let reply = match cursor.action() {
                Action::InitializeEmptyChain | Action::CaptureSubject => Reply::Applied,
                Action::AppendCurrentObject => {
                    self.append(subject.entry())?;
                    Reply::Applied
                }
                Action::ReadAreaShape => Reply::AreaShape(false),
                Action::ReadFocusableArea => {
                    Reply::FocusableArea(matches!(subject, Subject::Container { .. }))
                }
                Action::ReadAnchorIsDifferentElement
                    if matches!(subject, Subject::Container { .. }) =>
                {
                    Reply::AnchorIsDifferentElement(false)
                }
                Action::CaptureAnchorNodeDocument => {
                    let Subject::Container { document, .. } = subject else {
                        return Err(DomFocusChainError::UnavailableRole);
                    };
                    subject = Subject::Document(document);
                    Reply::Applied
                }
                Action::ReadDocument => Reply::Document(matches!(subject, Subject::Document(_))),
                Action::ReadNodeNavigable => {
                    let Subject::Document(document) = subject else {
                        return Err(DomFocusChainError::UnavailableRole);
                    };
                    active = self.active_navigable(document)?;
                    Reply::NodeNavigablePresent(active.is_some())
                }
                Action::ReadNavigableParent => {
                    let row = active.ok_or(DomHostedError::MissingPublication)?;
                    let has_parent = row.parent().is_some();
                    if !has_parent
                        && (row.container().is_some()
                            || row.former_parent().is_some()
                            || row.traversable() != row.id())
                    {
                        return Err(DomHostedError::StaleBasis.into());
                    }
                    Reply::NavigableParentPresent(has_parent)
                }
                Action::CaptureChildNavigableContainerElement => {
                    let child = active.ok_or(DomHostedError::MissingPublication)?;
                    let parent = child.parent().ok_or(DomHostedError::MissingPublication)?;
                    subject = self.container(child, parent)?;
                    Reply::Applied
                }
                Action::Complete => return Ok(()),
                _ => return Err(DomFocusChainError::UnavailableRole),
            };
            cursor
                .reply(reply)
                .map_err(|_| DomFocusChainError::UnavailableRole)?;
        }
    }
}
