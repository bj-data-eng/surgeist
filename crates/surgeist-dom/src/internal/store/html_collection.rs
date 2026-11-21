use std::cell::Cell;
use std::ops::ControlFlow;

use super::node_iterator::traversal::{OrdinaryTree, following};
use super::{read_source::ReadSource, *};
use crate::internal::{
    document::DomDocumentClass,
    names::{in_html_namespace_with, qualified_equals_with, same_code_units},
    snapshot::{DomNode, DomNodeKind},
};

use surgeist_html::{values::HtmlNameInput, vocabulary::KnownHtmlElement};

const ID: &[u16] = &[0x69, 0x64];
const NAME: &[u16] = &[0x6e, 0x61, 0x6d, 0x65];

fn window_name_element<E>(
    element: &(impl HtmlNameInput + ?Sized),
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if !in_html_namespace_with(element, &mut step)? {
        return Ok(false);
    }
    step()?;
    // HTML's exact recognition is bounded by its longest spelling.
    Ok(matches!(
        KnownHtmlElement::from_units(element.local_name().code_units()),
        Some(
            KnownHtmlElement::Embed
                | KnownHtmlElement::Form
                | KnownHtmlElement::Img
                | KnownHtmlElement::Object
        )
    ))
}

/// A live collection of ordinary descendants of one exact Document.
#[derive(Debug)]
pub struct DomHtmlCollection {
    document: DocumentId,
    filter: Filter,
}

#[derive(Debug)]
enum Filter {
    TagName(DocumentString),
    WindowName(DocumentString),
}

#[derive(Clone, Copy)]
enum ElementNameScope {
    Collection,
    Window,
}

/// The current native element candidates for an exact Window property name.
#[derive(Debug)]
pub enum DomWindowNamedElements {
    /// The name has no matching Element in the Document's ordinary tree.
    Absent,
    /// Exactly one Element matches the name.
    Element(NodeId),
    /// At least two Elements match. The collection remains live across edits.
    Collection(DomHtmlCollection),
}

/// Finite work and supported-name output for one collection read.
#[derive(Clone, Copy, Debug)]
pub struct DomHtmlCollectionLimits {
    work: u64,
    names: usize,
    name_units: usize,
}

impl DomHtmlCollectionLimits {
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
pub struct DomHtmlCollectionRead<'dom, A> {
    dom: &'dom Dom,
    snapshot: DomSnapshot,
    collection: &'dom DomHtmlCollection,
    class: DomDocumentClass,
    meter: Meter<'dom, A>,
    limits: DomHtmlCollectionLimits,
}

struct Meter<'dom, A> {
    cancellation: &'dom AtomicBool,
    admission: A,
    source: Option<ReadSource<'dom>>,
    work_limit: u64,
    work: Cell<u64>,
}

/// Owned ordered names with the backing admitted for this read.
pub struct DomHtmlCollectionNames {
    names: Vec<DocumentString>,
    vector_backing: Option<custody::Debit>,
    text_backing: Option<custody::Debit>,
    retained_units: usize,
}

impl DomHtmlCollectionNames {
    /// Borrow the exact names in their first-occurrence order.
    pub fn as_slice(&self) -> &[DocumentString] {
        &self.names
    }
}

impl std::fmt::Debug for DomHtmlCollectionNames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHtmlCollectionNames")
            .field("len", &self.names.len())
            .finish_non_exhaustive()
    }
}

impl<A> std::fmt::Debug for DomHtmlCollectionRead<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomHtmlCollectionRead")
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
    ) -> Result<DomHtmlCollection, DomError> {
        if document.node().owner() != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_read_source(cancel)?;
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
            DomResource::HtmlCollectionQueryUnits,
        )?;
        Ok(DomHtmlCollection {
            document,
            filter: Filter::TagName(qualified_name),
        })
    }

    /// Resolve the current ordinary-tree Element candidates for a Window name.
    ///
    /// The read admits the query's complete retained UTF-16 backing and stops
    /// at the second matching Element. The returned live collection retains the
    /// exact Document and name to read again after edits.
    ///
    /// # Errors
    ///
    /// Returns an error for an unavailable Document or Source scope,
    /// cancellation, exhausted work or backing, or the caller's refusal.
    pub fn window_named_elements<A, E>(
        &self,
        document: DocumentId,
        name: DocumentString,
        max_retained_query_units: usize,
        work_limit: u64,
        admission: A,
        cancel: &AtomicBool,
    ) -> Result<DomWindowNamedElements, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        let empty = name.is_empty();
        let retained = name.retained_units();
        let collection = DomHtmlCollection {
            document,
            filter: Filter::WindowName(name),
        };
        let read = self.read_html_collection(
            &collection,
            DomHtmlCollectionLimits::new(work_limit, 0, 0),
            admission,
            cancel,
        )?;
        admit(
            retained,
            max_retained_query_units,
            DomResource::HtmlCollectionQueryUnits,
        )?;
        if empty {
            return Ok(DomWindowNamedElements::Absent);
        }

        let mut first = None;
        let second = read.walk(|node| {
            if first.is_some() {
                return Ok(ControlFlow::Break(()));
            }
            first = Some(node.id());
            Ok(ControlFlow::Continue(()))
        })?;
        if second.is_some() {
            return Ok(DomWindowNamedElements::Collection(collection));
        }
        Ok(first.map_or(
            DomWindowNamedElements::Absent,
            DomWindowNamedElements::Element,
        ))
    }

    /// Revalidate the collection's Document against the current read authority.
    pub fn read_html_collection<'dom, A, E>(
        &'dom self,
        collection: &'dom DomHtmlCollection,
        limits: DomHtmlCollectionLimits,
        admission: A,
        cancel: &'dom AtomicBool,
    ) -> Result<DomHtmlCollectionRead<'dom, A>, E>
    where
        A: Fn() -> Result<(), E>,
        E: From<DomError>,
    {
        if collection.document.node().owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        self.require_source_entry(cancel)?;
        let source = self.qualify_read_source(cancel)?;
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
        Ok(DomHtmlCollectionRead {
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
                resource: DomResource::HtmlCollectionWork,
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

impl<A, E> DomHtmlCollectionRead<'_, A>
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
                let matches = match &self.collection.filter {
                    Filter::TagName(query) => {
                        query.code_units() == [0x2a]
                            || qualified_equals_with(
                                element.name(),
                                query,
                                self.class == DomDocumentClass::Html
                                    && in_html_namespace_with(element.name(), || {
                                        self.meter.step()
                                    })?,
                                || self.meter.step(),
                            )?
                    }
                    Filter::WindowName(name) => matches!(
                        self.names(node, ElementNameScope::Window, |value| self.name_match(
                            value,
                            name,
                            ()
                        ))?,
                        ControlFlow::Break(())
                    ),
                };
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
            self.names(node, ElementNameScope::Collection, |candidate| {
                self.name_match(candidate, name, node.id())
            })
        })
    }

    fn name_match<T>(
        &self,
        candidate: &DocumentString,
        query: &DocumentString,
        matched: T,
    ) -> Result<ControlFlow<T>, E> {
        if same_code_units(candidate.code_units(), query.code_units(), || {
            self.meter.step()
        })? {
            Ok(ControlFlow::Break(matched))
        } else {
            Ok(ControlFlow::Continue(()))
        }
    }

    // Borrow the exact ID and eligible name contributions from one Element.
    fn names<T>(
        &self,
        node: DomNode<'_>,
        scope: ElementNameScope,
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
        let eligible = match scope {
            ElementNameScope::Collection => {
                in_html_namespace_with(element.name(), || self.meter.step())?
            }
            ElementNameScope::Window => window_name_element(element.name(), || self.meter.step())?,
        };
        if !eligible {
            return Ok(ControlFlow::Continue(()));
        }
        if let Some(name) =
            attributes.get_null_namespace_attribute_with(NAME, || self.meter.step())?
            && !name.value().is_empty()
        {
            return visit(name.value());
        }
        Ok(ControlFlow::Continue(()))
    }

    /// Return each nonempty supported name once, in first occurrence order.
    pub fn supported_names(&self) -> Result<DomHtmlCollectionNames, E> {
        let mut output = DomHtmlCollectionNames {
            names: Vec::new(),
            vector_backing: None,
            text_backing: None,
            retained_units: 0,
        };
        self.walk::<()>(|node| {
            self.names(node, ElementNameScope::Collection, |value| {
                self.append_supported_name(&mut output, value)?;
                Ok(ControlFlow::Continue(()))
            })
        })?;
        Ok(output)
    }

    fn append_supported_name(
        &self,
        output: &mut DomHtmlCollectionNames,
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
        admit(count, self.limits.names, DomResource::HtmlCollectionNames)?;
        admit(
            next_units,
            self.limits.name_units,
            DomResource::HtmlCollectionNameUnits,
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

    fn grow_names(&self, output: &mut DomHtmlCollectionNames, count: usize) -> Result<(), E> {
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
            DomResource::HtmlCollectionNames,
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

impl<A, E> OrdinaryTree<NodeId> for DomHtmlCollectionRead<'_, A>
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

#[cfg(test)]
mod tests {
    use super::window_name_element;
    use surgeist_html::values::{Namespace, QualifiedName};

    #[test]
    fn window_name_eligibility_uses_the_four_exact_html_expanded_names() {
        for (namespace, local, expected) in [
            (Namespace::html(), "embed", true),
            (Namespace::html(), "form", true),
            (Namespace::html(), "img", true),
            (Namespace::html(), "object", true),
            (Namespace::html(), "iframe", false),
            (Namespace::html(), "IMG", false),
            (Namespace::svg(), "embed", false),
        ] {
            let name = QualifiedName::new(namespace, Some("p".into()), local);
            assert_eq!(
                window_name_element(&name, || Ok::<_, ()>(())),
                Ok(expected),
                "{name:?}"
            );
        }
    }
}
