use std::cell::Cell;
use std::ops::ControlFlow;
use std::sync::Weak;

use super::node_iterator::traversal::{OrdinaryTree, following};
use super::*;
use crate::internal::{
    document::DomDocumentClass,
    hosted::{DocumentAssociation, ScriptRegistration},
    names::{in_html_namespace_with, qualified_equals_with, same_code_units},
    snapshot::{DomNode, DomNodeKind},
    source::{
        callback::{CallbackWork, DispatchScope},
        event::DomSourceEventRequest,
    },
};

const ID: &[u16] = &[0x69, 0x64];
const NAME: &[u16] = &[0x6e, 0x61, 0x6d, 0x65];

/// A live ordinary-descendant tag-name query for one exact Document.
#[derive(Debug)]
pub struct DomTagNameCollection {
    document: DocumentId,
    qualified_name: DocumentString,
}

/// Finite work and supported-name output for one collection read.
#[derive(Clone, Copy, Debug)]
pub struct DomTagNameCollectionLimits {
    work: u64,
    names: usize,
    name_units: usize,
}

impl DomTagNameCollectionLimits {
    /// Bound the read's cumulative work and one supported-name result.
    pub const fn new(work: u64, names: usize, name_units: usize) -> Self {
        Self {
            work,
            names,
            name_units,
        }
    }
}

/// A qualified read of the collection in the current canonical view.
///
/// Reopen the view after mutation to observe the new tree. Its allowance is
/// cumulative across its operations, and the caller's admission runs at
/// every reached step.
pub struct DomTagNameCollectionRead<'dom, A> {
    dom: &'dom Dom,
    snapshot: DomSnapshot,
    collection: &'dom DomTagNameCollection,
    class: DomDocumentClass,
    meter: Meter<'dom, A>,
    limits: DomTagNameCollectionLimits,
}

struct Meter<'dom, A> {
    cancellation: &'dom AtomicBool,
    admission: A,
    source: Option<ReadSource<'dom>>,
    work_limit: u64,
    work: Cell<u64>,
}

/// Owned ordered names with the backing admitted for this read.
pub struct DomTagNameCollectionNames {
    names: Vec<DocumentString>,
    vector_backing: Option<custody::Debit>,
    text_backing: Option<custody::Debit>,
    retained_units: usize,
}

impl DomTagNameCollectionNames {
    /// Borrow the exact names in their first-occurrence order.
    pub fn as_slice(&self) -> &[DocumentString] {
        &self.names
    }
}

impl std::fmt::Debug for DomTagNameCollectionNames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomTagNameCollectionNames")
            .field("len", &self.names.len())
            .finish_non_exhaustive()
    }
}

pub(super) struct ReadSource<'dom> {
    work: Arc<CallbackWork>,
    scope: Weak<DispatchScope>,
    authority: ReadAuthority<'dom>,
}

pub(super) enum ReadAuthority<'dom> {
    Event(&'dom DomSourceEventRequest),
    Script {
        registration: &'dom ScriptRegistration,
        association: Arc<DocumentAssociation>,
        attribute: bool,
    },
}

impl<'dom> ReadSource<'dom> {
    pub(super) fn new(
        work: Arc<CallbackWork>,
        authority: ReadAuthority<'dom>,
    ) -> Result<Self, DomError> {
        let scope = work.active_scope().ok_or(DomError::InvalidSourceEvent)?;
        let qualified = Self {
            work,
            scope,
            authority,
        };
        qualified.validate()?;
        Ok(qualified)
    }

    fn validate(&self) -> Result<(), DomError> {
        if !self
            .work
            .active_scope()
            .is_some_and(|active| active.ptr_eq(&self.scope))
        {
            return Err(DomError::InvalidSourceEvent);
        }
        match &self.authority {
            ReadAuthority::Event(request) => {
                let executor = request
                    .executor
                    .upgrade()
                    .ok_or(DomError::InvalidSourceEvent)?;
                request.validate_executor(&executor)
            }
            ReadAuthority::Script {
                registration,
                association,
                attribute,
            } => {
                if !registration.is_active() {
                    return Err(if *attribute {
                        DomError::SourceAttributeExecutorUnavailable
                    } else {
                        DomError::HostedSourceUnavailable
                    });
                }
                association
                    .validate_live()
                    .map(|_| ())
                    .map_err(|_| DomError::HostedSourceUnavailable)
            }
        }
    }

    fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.validate()?;
        self.work.step(cancel)
    }
}

impl<A> std::fmt::Debug for DomTagNameCollectionRead<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomTagNameCollectionRead")
            .field("document", &self.collection.document)
            .field("stamp", &self.snapshot.stamp())
            .finish_non_exhaustive()
    }
}

impl Dom {
    /// Form a live tag-name collection and admit the query's complete backing.
    pub fn get_elements_by_tag_name(
        &self,
        document: DocumentId,
        qualified_name: DocumentString,
        max_retained_query_units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomTagNameCollection, DomError> {
        if document.node().owner() != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_collection_read(cancel)?;
        let document_node = self.snapshot();
        let view = document_node.node_with(document.node(), || match &source {
            Some(source) => source.step(cancel),
            None => check_cancel(cancel),
        })?;
        if !matches!(view.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        admit(
            qualified_name.retained_units(),
            max_retained_query_units,
            DomResource::TagNameCollectionQueryUnits,
        )?;
        Ok(DomTagNameCollection {
            document,
            qualified_name,
        })
    }

    /// Revalidate the collection's Document against the current read authority.
    pub fn read_tag_name_collection<'dom, A, E>(
        &'dom self,
        collection: &'dom DomTagNameCollection,
        limits: DomTagNameCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomTagNameCollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if collection.document.node().owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_collection_read(cancel)?;
        let meter = Meter {
            cancellation: cancel,
            admission,
            source,
            work_limit: limits.work,
            work: Cell::new(0),
        };
        let snapshot = self.snapshot();
        meter.step()?;
        let DomNodeKind::Document(metadata) = snapshot
            .node_with(collection.document.node(), || meter.step())?
            .kind()
        else {
            return Err(DomError::WrongNodeKind.into());
        };
        let class = metadata.class();
        Ok(DomTagNameCollectionRead {
            dom: self,
            snapshot,
            collection,
            class,
            meter,
            limits,
        })
    }
}

impl<A, E> Meter<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn step(&self) -> Result<(), E> {
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
                resource: DomResource::TagNameCollectionWork,
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

impl<A, E> DomTagNameCollectionRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    fn node(&self, node: NodeId) -> Result<DomNode<'_>, E> {
        self.snapshot.node_with(node, || self.meter.step())
    }

    fn walk<T>(
        &self,
        mut visit: impl FnMut(DomNode<'_>) -> Result<ControlFlow<T>, E>,
    ) -> Result<Option<T>, E> {
        let root = self.collection.document.node();
        let mut at = self.child(root, 0)?;
        while let Some(id) = at {
            let node = self.node(id)?;
            if let Some(element) = node.as_element() {
                let query = &self.collection.qualified_name;
                let matches = query.code_units() == [0x2a]
                    || qualified_equals_with(
                        element.name(),
                        query,
                        self.class == DomDocumentClass::Html
                            && in_html_namespace_with(element.name(), || self.meter.step())?,
                        || self.meter.step(),
                    )?;
                if matches && let ControlFlow::Break(value) = visit(node)? {
                    return Ok(Some(value));
                }
            }
            at = match node.children().first().copied() {
                Some(child) => Some(child),
                None => following(self, root, id)?,
            };
        }
        Ok(None)
    }

    /// Count every represented element in the current ordinary tree.
    pub fn length(&self) -> Result<usize, E> {
        let mut length = 0usize;
        self.walk::<()>(|_| {
            length = length
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(ControlFlow::Continue(()))
        })?;
        Ok(length)
    }

    /// Read the current element at the given zero-based index.
    pub fn item(&self, mut index: usize) -> Result<Option<NodeId>, E> {
        self.walk(|node| {
            if index == 0 {
                return Ok(ControlFlow::Break(node.id()));
            }
            index -= 1;
            Ok(ControlFlow::Continue(()))
        })
    }

    /// Read the first current element with the exact ID or eligible name.
    pub fn named_item(&self, name: &DocumentString) -> Result<Option<NodeId>, E> {
        self.meter.step()?;
        if name.is_empty() {
            return Ok(None);
        }
        self.walk(|node| {
            self.names(node, |candidate| {
                if same_code_units(candidate.code_units(), name.code_units(), || {
                    self.meter.step()
                })? {
                    Ok(ControlFlow::Break(node.id()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            })
        })
    }

    fn names<T>(
        &self,
        node: DomNode<'_>,
        mut visit: impl FnMut(&DocumentString) -> Result<ControlFlow<T>, E>,
    ) -> Result<ControlFlow<T>, E> {
        let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
        if let Some(id) = attributes.get_null_namespace_attribute_with(ID, || self.meter.step())?
            && !id.value().is_empty()
            && let ControlFlow::Break(value) = visit(id.value())?
        {
            return Ok(ControlFlow::Break(value));
        }
        let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
        if in_html_namespace_with(element.name(), || self.meter.step())?
            && let Some(name) =
                attributes.get_null_namespace_attribute_with(NAME, || self.meter.step())?
            && !name.value().is_empty()
        {
            return visit(name.value());
        }
        Ok(ControlFlow::Continue(()))
    }

    /// Return each nonempty supported name once, in first occurrence order.
    pub fn supported_names(&self) -> Result<DomTagNameCollectionNames, E> {
        let mut output = DomTagNameCollectionNames {
            names: Vec::new(),
            vector_backing: None,
            text_backing: None,
            retained_units: 0,
        };
        self.walk::<()>(|node| {
            self.names(node, |value| {
                self.append_supported_name(&mut output, value)?;
                Ok(ControlFlow::Continue(()))
            })
        })?;
        Ok(output)
    }

    fn append_supported_name(
        &self,
        output: &mut DomTagNameCollectionNames,
        value: &DocumentString,
    ) -> Result<(), E> {
        for present in &output.names {
            self.meter.step()?;
            if same_code_units(present.code_units(), value.code_units(), || {
                self.meter.step()
            })? {
                return Ok(());
            }
        }
        let count = output
            .names
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let next_units = output
            .retained_units
            .checked_add(value.retained_units())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            count,
            self.limits.names,
            DomResource::TagNameCollectionNames,
        )?;
        admit(
            next_units,
            self.limits.name_units,
            DomResource::TagNameCollectionNameUnits,
        )?;
        if output.names.capacity() < count {
            self.grow_names(output, count)?;
        }
        self.meter.step()?;
        if self.meter.source.is_some() {
            let mut payload = source::storage::Payload::default();
            payload.string(value)?;
            let usage = custody::Usage::new(payload.bytes, payload.units);
            let admit = |usage| {
                self.dom
                    .admit_collection_output_usage(usage, self.meter.cancellation)
            };
            if let Some(backing) = &mut output.text_backing {
                backing.grow(usage, admit)?;
            } else {
                output.text_backing =
                    Some(self.dom.custody_registry.ledger().retain(usage, admit)?);
            }
        }
        output.names.push(value.clone());
        output.retained_units = next_units;
        Ok(())
    }

    fn grow_names(&self, output: &mut DomTagNameCollectionNames, count: usize) -> Result<(), E> {
        self.meter.step()?;
        let capacity = count.max(
            output
                .names
                .capacity()
                .saturating_mul(2)
                .min(self.limits.names),
        );
        let usage = |slots: usize| {
            slots
                .checked_mul(size_of::<DocumentString>())
                .map(|bytes| custody::Usage::new(bytes, 0))
                .ok_or(DomError::RepresentationExhausted)
        };
        let admit = |usage| {
            self.dom
                .admit_collection_output_usage(usage, self.meter.cancellation)
        };
        let mut backing = self
            .meter
            .source
            .as_ref()
            .map(|_| {
                self.dom
                    .custody_registry
                    .ledger()
                    .retain(usage(capacity)?, admit)
            })
            .transpose()?;
        let mut names = Vec::new();
        names
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        super::admit(
            names.capacity(),
            self.limits.names,
            DomResource::TagNameCollectionNames,
        )?;
        if let Some(backing) = &mut backing {
            transaction::nodes::resize_debit(backing, usage(names.capacity())?, admit)?;
        }
        // Source holds both old and replacement vector storage until the move.
        // The text occurrence receipts stay with the whole output.
        for value in output.names.drain(..) {
            self.meter.step()?;
            names.push(value);
        }
        output.names = names;
        output.vector_backing = backing;
        Ok(())
    }
}

impl<A, E> OrdinaryTree<NodeId> for DomTagNameCollectionRead<'_, A>
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
