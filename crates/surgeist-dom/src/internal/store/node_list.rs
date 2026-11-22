use std::ops::ControlFlow;
use std::sync::atomic::AtomicBool;

use super::collection_read::CollectionRead;
use super::{DocumentId, DocumentString, Dom, DomError, DomResource, NodeId, admit};
use crate::internal::names::{in_html_namespace_with, same_code_units};
use crate::internal::snapshot::{DomNode, DomNodeKind};

const NAME: &[u16] = &[0x6e, 0x61, 0x6d, 0x65];

/// A live list of ordinary children or of a Document's named HTML elements.
#[derive(Debug)]
pub struct DomNodeList(Filter);

#[derive(Debug)]
enum Filter {
    Children(NodeId),
    DocumentName {
        document: DocumentId,
        name: DocumentString,
    },
}

impl Filter {
    fn root(&self) -> NodeId {
        match self {
            Self::Children(node) => *node,
            Self::DocumentName { document, .. } => document.node(),
        }
    }
}

/// A qualified read of the list in the current canonical tree. Operations
/// share the work allowance and reach the caller's admission at each step.
pub struct DomNodeListRead<'dom, A> {
    list: &'dom DomNodeList,
    view: CollectionRead<'dom, A>,
}

impl<A> std::fmt::Debug for DomNodeListRead<'_, A> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DomNodeListRead")
            .field("list", self.list)
            .field("stamp", &self.view.snapshot.stamp())
            .finish_non_exhaustive()
    }
}

impl Dom {
    /// Form the live child list for one exact node, including non-Elements.
    ///
    /// # Errors
    /// Returns an error for a foreign or unavailable node, invalid Source
    /// read custody, or cancellation.
    pub fn child_nodes(&self, node: NodeId, cancel: &AtomicBool) -> Result<DomNodeList, DomError> {
        self.validate_collection_root(node, cancel, |_| Ok(()))?;
        Ok(DomNodeList(Filter::Children(node)))
    }

    /// Form a live list of ordinary HTML descendants whose null-namespace
    /// `name` attribute contains exactly the supplied UTF-16 code units.
    ///
    /// # Errors
    /// Returns an error for an unavailable Document, invalid Source read
    /// custody, exhausted retained query capacity, or cancellation.
    pub fn get_elements_by_name(
        &self,
        document: DocumentId,
        name: DocumentString,
        max_retained_query_units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomNodeList, DomError> {
        self.validate_collection_document(document, cancel)?;
        admit(
            name.retained_units(),
            max_retained_query_units,
            DomResource::NodeListQueryUnits,
        )?;
        Ok(DomNodeList(Filter::DocumentName { document, name }))
    }

    /// Open a new read to observe current members after a tree change.
    ///
    /// # Errors
    /// Returns the first invalid owner or root, Source or caller refusal,
    /// cancellation, or exhausted work allowance.
    pub fn read_node_list<'dom, A, E>(
        &'dom self,
        list: &'dom DomNodeList,
        work_limit: u64,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomNodeListRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let view = self.open_collection_read(
            list.0.root(),
            DomResource::NodeListWork,
            work_limit,
            admission,
            cancel,
        )?;
        let kind = view.node(list.0.root())?.kind();
        if matches!(list.0, Filter::DocumentName { .. })
            && !matches!(kind, DomNodeKind::Document(_))
        {
            return Err(DomError::WrongNodeKind.into());
        }
        Ok(DomNodeListRead { list, view })
    }
}

impl<A, E> DomNodeListRead<'_, A>
where
    A: Fn() -> Result<(), E>,
    E: From<DomError>,
{
    /// Count the current nodes represented by the list.
    pub fn length(&self) -> Result<usize, E> {
        match &self.list.0 {
            Filter::Children(root) => Ok(self.view.node(*root)?.children().len()),
            Filter::DocumentName { document, name } => {
                let mut count = 0_usize;
                self.walk_named::<()>(*document, name, |_| {
                    count = count
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    Ok(ControlFlow::Continue(()))
                })?;
                Ok(count)
            }
        }
    }

    /// Resolve one zero-based index in current ordinary tree order.
    pub fn item(&self, mut index: usize) -> Result<Option<NodeId>, E> {
        match &self.list.0 {
            Filter::Children(root) => Ok(self.view.node(*root)?.children().get(index).copied()),
            Filter::DocumentName { document, name } => self.walk_named(*document, name, |node| {
                if index == 0 {
                    return Ok(ControlFlow::Break(node));
                }
                index -= 1;
                Ok(ControlFlow::Continue(()))
            }),
        }
    }

    fn walk_named<T>(
        &self,
        document: DocumentId,
        name: &DocumentString,
        mut visit: impl FnMut(NodeId) -> Result<ControlFlow<T>, E>,
    ) -> Result<Option<T>, E> {
        self.view.descendants(document.node(), |node| {
            if self.matches_name(node, name)? {
                visit(node.id())
            } else {
                Ok(ControlFlow::Continue(()))
            }
        })
    }

    fn matches_name(&self, node: DomNode<'_>, name: &DocumentString) -> Result<bool, E> {
        let Some(element) = node.as_element() else {
            return Ok(false);
        };
        let step = || self.view.meter.step();
        if !in_html_namespace_with(element.name(), step)? {
            return Ok(false);
        }
        let Some(attribute) = node
            .attributes()
            .ok_or(DomError::WrongNodeKind)?
            .get_null_namespace_attribute_with(NAME, step)?
        else {
            return Ok(false);
        };
        same_code_units(attribute.value().code_units(), name.code_units(), step)
    }
}
