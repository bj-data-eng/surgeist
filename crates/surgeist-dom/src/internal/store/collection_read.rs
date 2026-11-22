use std::cell::Cell;
use std::ops::ControlFlow;

use super::node_iterator::traversal::{OrdinaryTree, following};
use super::{Dom, DomError, DomResource, DomSnapshot, check_cancel, read_source::ReadSource};
use crate::internal::identity::{DocumentId, NodeId};
use crate::internal::snapshot::{DomNode, DomNodeKind};
use std::sync::atomic::AtomicBool;

pub(super) struct CollectionRead<'dom, A> {
    pub(super) dom: &'dom Dom,
    pub(super) snapshot: DomSnapshot,
    pub(super) meter: Meter<'dom, A>,
}

pub(super) struct Meter<'dom, A> {
    pub(super) cancellation: &'dom AtomicBool,
    pub(super) source: Option<ReadSource<'dom>>,
    admission: A,
    resource: DomResource,
    work_limit: u64,
    work: Cell<u64>,
}

impl Dom {
    pub(super) fn validate_collection_root(
        &self,
        root: NodeId,
        cancel: &AtomicBool,
        check_kind: impl FnOnce(DomNodeKind<'_>) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if root.owner() != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_read_source(cancel)?;
        let snapshot = self.snapshot();
        let root = snapshot.node_with(root, || match &source {
            Some(source) => source.step(cancel),
            None => check_cancel(cancel),
        })?;
        check_kind(root.kind())
    }

    pub(super) fn validate_collection_document(
        &self,
        document: DocumentId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.validate_collection_root(document.node(), cancel, |kind| match kind {
            DomNodeKind::Document(_) => Ok(()),
            _ => Err(DomError::WrongNodeKind),
        })
    }

    pub(super) fn open_collection_read<'dom, A, E>(
        &'dom self,
        root: NodeId,
        resource: DomResource,
        work_limit: u64,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<CollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if root.owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_read_source(cancel)?;
        let meter = Meter::new(cancel, source, resource, work_limit, admission);
        let snapshot = self.snapshot();
        meter.step()?;
        Ok(CollectionRead {
            dom: self,
            snapshot,
            meter,
        })
    }
}

impl<'dom, A, E> Meter<'dom, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    pub(super) fn new(
        cancellation: &'dom AtomicBool,
        source: Option<ReadSource<'dom>>,
        resource: DomResource,
        work_limit: u64,
        admission: A,
    ) -> Self {
        Self {
            cancellation,
            source,
            admission,
            resource,
            work_limit,
            work: Cell::new(0),
        }
    }

    pub(super) fn step(&self) -> Result<(), E> {
        check_cancel(self.cancellation)?;
        if let Some(source) = &self.source {
            source.validate()?;
        }
        let next = self
            .work
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.work_limit {
            return Err(DomError::WorkLimit {
                resource: self.resource,
                limit: self.work_limit,
            }
            .into());
        }
        (self.admission)()?;
        match &self.source {
            Some(source) => source.step(self.cancellation)?,
            None => check_cancel(self.cancellation)?,
        }
        self.work.set(next);
        Ok(())
    }
}

impl<A, E> CollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    pub(super) fn node(&self, node: NodeId) -> Result<DomNode<'_>, E> {
        self.snapshot.node_with(node, || self.meter.step())
    }

    pub(super) fn descendants<T>(
        &self,
        root: NodeId,
        mut visit: impl FnMut(DomNode<'_>) -> Result<ControlFlow<T>, E>,
    ) -> Result<Option<T>, E> {
        let mut at = self.child(root, 0)?;
        while let Some(id) = at {
            let node = self.node(id)?;
            if let ControlFlow::Break(value) = visit(node)? {
                return Ok(Some(value));
            }
            at = match node.children().first().copied() {
                Some(child) => Some(child),
                None => following(self, root, id)?,
            };
        }
        Ok(None)
    }
}

impl<A, E> OrdinaryTree<NodeId> for CollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    type Error = E;

    fn position(&self, node: NodeId) -> Result<Option<(NodeId, usize)>, E> {
        self.dom
            .state
            .ordinary_child_position_with(node, &mut || self.meter.step())
    }

    fn child(&self, parent: NodeId, ordinal: usize) -> Result<Option<NodeId>, E> {
        Ok(self.node(parent)?.children().get(ordinal).copied())
    }

    fn last_child(&self, parent: NodeId) -> Result<Option<NodeId>, E> {
        Ok(self.node(parent)?.children().last().copied())
    }
}
