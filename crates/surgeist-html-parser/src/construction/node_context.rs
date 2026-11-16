//! The context of every node of a parse, held while the parse builds and published by tree.
//!
//! The context of a node is its document and the custom element registry its lookups use. While
//! the parse builds, each record has one cell in a column: zero names no context, and any other
//! value names an entry of the list of the distinct contexts the parse has named, one more than
//! its position. A context is named by many nodes, so the list is deduplicated and short: the
//! nodes of an ordinary parse share one entry for each document, and a fragment parse adds an
//! entry for each document and registry it pairs. The column is written when a record is created
//! and when an adoption moves a node to another document or another registry. Freezing the parse
//! publishes what the column says and releases it: a tree has the document of its root, and a
//! table holds the entry named by each node that is asked for by its own key: a root that is not a
//! Document, a link, and an element whose registry is not Null. A Document root names itself.

use super::context::RegistryRef;
use super::family_table::FamilyTable;
use super::row_bank::RowBank;
use super::row_log::RowLog;
use super::session_error::latch_registry_row;
use super::*;
#[cfg(test)]
use surgeist_html::construction::HtmlConstructionResult;

/// The document and the custom element registry of the nodes that name an entry of the list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NodeContext {
    pub(super) document: DocumentRef,
    pub(super) registry: RegistryRef,
}

pub(super) struct NodeContexts {
    /// One cell per record, `None` once the parse is frozen.
    column: Option<RowBank<u32>>,
    /// The distinct contexts the nodes of the parse name, in the order they were first named.
    entries: RowLog<NodeContext>,
    /// Whether an entry names a registry that is not Null: no element of a parse without one has
    /// a registry to ask for.
    registered: bool,
    /// The entry of `entries` that each root which is not a Document names, each link that names
    /// one, and each element whose registry is not Null, built when the parse is frozen. A link is
    /// asked for its document by its own key, so the answer is one search whatever the depth it
    /// is nested at.
    named: Option<FamilyTable<u32>>,
    /// The column as it was when the parse was frozen, for tests that compare it with the
    /// published answers.
    #[cfg(test)]
    pub(super) frozen_column: Vec<u32>,
}

/// A row of the table of named documents holds an entry the list of documents does not hold. A
/// published parse is qualified once, so this is a defect of the report, not an answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DanglingEntry;

/// What a tree answers for its document.
pub(super) enum TreeDocument<'a> {
    /// The root is a Document, which is its own document.
    Itself(usize),
    /// The root names this document.
    Named(&'a DocumentRef),
    /// The root names no document.
    Unnamed,
}

impl NodeContexts {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let column = RowBank::new(control)?;
        let entries = match RowLog::new(control) {
            Ok(entries) => entries,
            Err(error) => {
                let owner = control.identity();
                // A refused release has latched its stop; the error that ended the creation is
                // the first reported.
                let _released = column.release(control, &owner, |_, _| Ok(()), |_| Ok(()));
                return Err(error);
            }
        };
        Ok(Self {
            column: Some(column),
            entries,
            registered: false,
            named: None,
            #[cfg(test)]
            frozen_column: Vec::new(),
        })
    }
    /// The distinct contexts the nodes of the parse name.
    pub(super) fn entries(&self) -> &RowLog<NodeContext> {
        &self.entries
    }
    /// Whether any entry names a registry that is not Null.
    pub(super) fn registered(&self) -> bool {
        self.registered
    }
    /// The bytes of the column's vector, which the parse holds until it is frozen.
    #[cfg(test)]
    pub(super) fn column_capacity_bytes(&self) -> usize {
        self.column.as_ref().map_or(0, RowBank::capacity_bytes)
    }
    /// The same list, for a test that corrupts what a report holds.
    #[cfg(test)]
    pub(super) fn entries_mut(&mut self) -> &mut RowLog<NodeContext> {
        &mut self.entries
    }
    /// Makes a parse that names no registry other than Null search its table for every registry
    /// read, as a parse that names one does, for a test that corrupts what a report holds.
    #[cfg(test)]
    pub(super) fn mark_registered_for_test(&mut self) {
        self.registered = true;
    }
    /// The table of the roots and links that name a document, for a test that corrupts what a
    /// report holds.
    #[cfg(test)]
    pub(super) fn named_mut(&mut self) -> Option<&mut FamilyTable<u32>> {
        self.named.as_mut()
    }
    /// The context an entry of the list names.
    pub(super) fn entry(&self, entry: u32) -> Option<&NodeContext> {
        self.entries
            .row(usize::try_from(entry.checked_sub(1)?).ok()?)
    }
    /// The entry that names `document` with `registry`, if the list holds one. One record visit
    /// is charged for each entry searched.
    fn find_context(
        &self,
        c: &mut ParseControl,
        document: &DocumentRef,
        registry: &RegistryRef,
    ) -> Result<Option<u32>, SessionError> {
        for (position, held) in self.entries.iter() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if held.document == *document && held.registry == *registry {
                return entry_of(c, position).map(Some);
            }
        }
        Ok(None)
    }
    /// The document an entry of the list names.
    pub(super) fn document(&self, entry: u32) -> Option<&DocumentRef> {
        self.entry(entry).map(|context| &context.document)
    }
    /// The entry a record's cell names, zero for none; `None` for a record without a cell or a
    /// frozen parse.
    pub(super) fn cell(&self, node: usize) -> Option<u32> {
        self.column.as_ref()?.row(node).copied()
    }
    /// The document the tree rooted at `root` has, with the number of keys the search read.
    /// `root_is_document` is whether the record is a Document. A row of the table whose entry the
    /// list does not hold is [`DanglingEntry`].
    pub(super) fn tree_document(
        &self,
        root: usize,
        root_is_document: bool,
    ) -> Result<(TreeDocument<'_>, u64), DanglingEntry> {
        if root_is_document {
            return Ok((TreeDocument::Itself(root), 0));
        }
        let (named, probes) = self.named_document(root)?;
        Ok((
            named.map_or(TreeDocument::Unnamed, TreeDocument::Named),
            probes,
        ))
    }
    /// The document the record at `node` names in the table built when the parse was frozen, with
    /// the number of keys the search read: a root that is not a Document, or a link, whose own cell
    /// named one. A node with no row, such as a link whose cell named none, is in no document; a
    /// row whose entry the list does not hold is [`DanglingEntry`].
    pub(super) fn named_document(
        &self,
        node: usize,
    ) -> Result<(Option<&DocumentRef>, u64), DanglingEntry> {
        let Some(named) = &self.named else {
            return Ok((None, 0));
        };
        let (found, probes) = named.find_published(node);
        let Some(index) = found else {
            return Ok((None, probes));
        };
        named
            .row(index)
            .and_then(|&entry| self.document(entry))
            .map(|document| (Some(document), probes))
            .ok_or(DanglingEntry)
    }
    /// The lookup registry of the record at `node` in the table built when the parse was frozen,
    /// with the number of keys the search read. A node with no row has the Null registry, and the
    /// answer is `None`: a root, a link and a node whose registry is not Null have a row. A parse
    /// that names no registry other than Null reads no key.
    pub(super) fn published_registry(
        &self,
        node: usize,
    ) -> Result<(Option<&RegistryRef>, u64), DanglingEntry> {
        if !self.registered {
            return Ok((None, 0));
        }
        let Some(named) = &self.named else {
            return Ok((None, 0));
        };
        let (found, probes) = named.find_published(node);
        let Some(index) = found else {
            return Ok((None, probes));
        };
        named
            .row(index)
            .and_then(|&entry| self.entry(entry))
            .map(|context| (Some(&context.registry), probes))
            .ok_or(DanglingEntry)
    }
    /// Whether the lookup registry of the record at `node` is Null when the parse was frozen, with
    /// the number of keys the search read.
    pub(super) fn registry_is_null(&self, node: usize) -> Result<(bool, u64), DanglingEntry> {
        let (registry, probes) = self.published_registry(node)?;
        Ok((
            registry.is_none_or(|registry| matches!(registry, RegistryRef::Null)),
            probes,
        ))
    }
    /// The rows of the table built when the parse was frozen.
    pub(super) fn named_len(&self) -> usize {
        self.named.as_ref().map_or(0, FamilyTable::len)
    }
    /// The node and the registry of the row at `index` of that table, in ascending order of node;
    /// `None` past its last row. A row whose entry the list does not hold is [`DanglingEntry`].
    pub(super) fn named_registry(
        &self,
        index: usize,
    ) -> Result<Option<(usize, &RegistryRef)>, DanglingEntry> {
        let Some(named) = &self.named else {
            return Ok(None);
        };
        let (Some(node), Some(&entry)) = (named.node_at(index), named.row(index)) else {
            return Ok(None);
        };
        let context = self.entry(entry).ok_or(DanglingEntry)?;
        Ok(Some((node, &context.registry)))
    }
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        let Self {
            column,
            entries,
            named,
            ..
        } = self;
        let mut refused = None;
        if let Some(column) = column {
            keep_first(
                &mut refused,
                column.release(control, owner, |_, _| Ok(()), |_| Ok(())),
            );
        }
        keep_first(&mut refused, entries.release(control, owner, |_, _| Ok(())));
        if let Some(named) = named {
            keep_first(&mut refused, named.release(control, owner, |_, _| Ok(())));
        }
        refused.map_or(Ok(()), Err)
    }
}

impl SessionStorage {
    /// Admits the cells of `count` records about to be created. The cells are written by
    /// [`Self::push_context_cells`] once the records exist.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit_context_cells(
        &mut self,
        c: &mut ParseControl,
        count: usize,
    ) -> Result<(), SessionError> {
        let Some(column) = self.payloads.facts.contexts.column.as_mut() else {
            return Err(latch_registry_row(c));
        };
        column.admit(c, count)
    }
    /// Writes the cells of `count` records that were created, none naming a document.
    pub(super) fn push_context_cells(&mut self, count: usize) -> Result<(), SessionError> {
        let column = self
            .payloads
            .facts
            .contexts
            .column
            .as_mut()
            .ok_or_else(SessionError::registry_row)?;
        for _ in 0..count {
            column.push(0)?;
        }
        Ok(())
    }
    /// The entry of the list that names `document` with the Null registry, naming it first when
    /// it is new. One record visit is charged for each entry searched.
    #[cfg_attr(test, track_caller)]
    pub(super) fn intern_document(
        &mut self,
        c: &mut ParseControl,
        document: DocumentRef,
    ) -> Result<u32, SessionError> {
        self.intern_context(c, document, RegistryRef::Null)
    }
    /// The entry of the list that names `document` with `registry`, naming it first when it is
    /// new. One record visit is charged for each entry searched.
    #[cfg_attr(test, track_caller)]
    pub(super) fn intern_context(
        &mut self,
        c: &mut ParseControl,
        document: DocumentRef,
        registry: RegistryRef,
    ) -> Result<u32, SessionError> {
        if let Some(entry) = self
            .payloads
            .facts
            .contexts
            .find_context(c, &document, &registry)?
        {
            return Ok(entry);
        }
        self.name_context(c, NodeContext { document, registry })
    }
    /// The entry that names `document` with the registry of the entry `like`, naming it first
    /// when it is new. The registry is copied, and charged, only when a new entry names it.
    #[cfg_attr(test, track_caller)]
    pub(super) fn intern_like(
        &mut self,
        c: &mut ParseControl,
        document: DocumentRef,
        like: u32,
    ) -> Result<u32, SessionError> {
        let contexts = &self.payloads.facts.contexts;
        let Some(held) = contexts.entry(like) else {
            return Err(latch_registry_row(c));
        };
        if let Some(entry) = contexts.find_context(c, &document, &held.registry)? {
            return Ok(entry);
        }
        let registry = held.registry.try_clone(c)?;
        self.name_context(c, NodeContext { document, registry })
    }
    /// The entry that names the document of `entry` with `registry`: `entry` itself when it names
    /// that registry already.
    #[cfg_attr(test, track_caller)]
    pub(super) fn entry_with_registry(
        &mut self,
        c: &mut ParseControl,
        entry: u32,
        registry: RegistryRef,
    ) -> Result<u32, SessionError> {
        let contexts = &self.payloads.facts.contexts;
        let Some(held) = contexts.entry(entry) else {
            return Err(latch_registry_row(c));
        };
        if held.registry == registry {
            return Ok(entry);
        }
        if let Some(found) = contexts.find_context(c, &held.document, &registry)? {
            return Ok(found);
        }
        let document = held.document.try_clone(c)?;
        self.name_context(c, NodeContext { document, registry })
    }
    /// Names a context that the list does not hold yet.
    #[cfg_attr(test, track_caller)]
    fn name_context(
        &mut self,
        c: &mut ParseControl,
        context: NodeContext,
    ) -> Result<u32, SessionError> {
        let contexts = &mut self.payloads.facts.contexts;
        contexts.entries.admit(c, 1)?;
        contexts.registered |= !matches!(context.registry, RegistryRef::Null);
        let position = contexts.entries.push(context)?;
        entry_of(c, position)
    }
    /// Names `registry` as the lookup registry of the record at `node`, which has a cell: the
    /// record's document is kept, and a registry the record already has changes nothing. A record
    /// with no cell has no registry to change.
    #[cfg_attr(test, track_caller)]
    pub(super) fn set_registry(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        registry: RegistryRef,
    ) -> Result<(), SessionError> {
        let Some(held) = self
            .payloads
            .facts
            .contexts
            .cell(node)
            .filter(|&entry| entry != 0)
        else {
            return Err(SessionError::ContextFactUnavailable(
                crate::HtmlCaptureFact::Registry,
            ));
        };
        let entry = self.entry_with_registry(c, held, registry)?;
        if entry == held {
            return Ok(());
        }
        payload_write::<u32>(c, 1)?;
        self.set_context(c, node, entry)
    }
    /// Names `entry` as the document of the record at `node`, which has a cell. The write cannot
    /// be refused once the entry exists.
    pub(super) fn set_context(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        entry: u32,
    ) -> Result<(), SessionError> {
        let cell = self
            .payloads
            .facts
            .contexts
            .column
            .as_mut()
            .and_then(|column| column.row_mut(node));
        match cell {
            Some(cell) => {
                *cell = entry;
                Ok(())
            }
            None => Err(latch_registry_row(c)),
        }
    }
    /// Publishes the documents of the trees of the parse and releases the column: one record visit
    /// for each record, and one row for each node that names a context and is a root that is not
    /// a Document, a link, or a node whose registry is not Null. A Document root names itself; one
    /// that names another document has no representation, and is refused as the node document
    /// defect.
    pub(super) fn publish_node_contexts(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        let owner = self.control.clone();
        let mut named = FamilyTable::new(c)?;
        let result = (|| {
            for node in 0..self.construction.len() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let record = self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?;
                let is_link = matches!(
                    record.kind(),
                    ConstructionNodeKind::Element(element) if element.link_flags().is_some()
                );
                let contexts = &self.payloads.facts.contexts;
                let entry = contexts.cell(node).ok_or_else(|| latch_registry_row(c))?;
                // A node that is not a root or a link is asked for its registry by its own key
                // when the registry is not Null.
                let registered = contexts.registered()
                    && contexts
                        .entry(entry)
                        .is_some_and(|context| !matches!(context.registry, RegistryRef::Null));
                if record.parent().is_some() && !is_link && !registered {
                    continue;
                }
                if entry == 0 {
                    continue;
                }
                if matches!(record.kind(), ConstructionNodeKind::Document(_)) {
                    if self.payloads.facts.contexts.document(entry)
                        != Some(&DocumentRef::Owned(node))
                    {
                        return Err(SessionError::NodeDocumentUnavailable);
                    }
                    continue;
                }
                named.admit(c, 1)?;
                named.push(node, entry)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            return Err(abandon_table(c, &owner, named, error));
        }
        let contexts = &mut self.payloads.facts.contexts;
        contexts.named = Some(named);
        #[cfg(test)]
        {
            contexts.frozen_column = contexts
                .column
                .as_ref()
                .map(|column| column.rows().to_vec())
                .unwrap_or_default();
        }
        match contexts.column.take() {
            Some(column) => column.release(c, &owner, |_, _| Ok(()), |_| Ok(())),
            None => Ok(()),
        }
    }
}

/// Releases `named`, the table a freeze stopped building, and returns `error`, the first reported.
/// A refused release has latched its stop on the controller, so it needs no second report.
pub(super) fn abandon_table(
    c: &mut ParseControl,
    owner: &ControlIdentity,
    named: FamilyTable<u32>,
    error: SessionError,
) -> SessionError {
    let _released = named.release(c, owner, |_, _| Ok(()));
    error
}

/// The entry that names the document at `position` of the list.
fn entry_of(c: &mut ParseControl, position: usize) -> Result<u32, SessionError> {
    u32::try_from(position)
        .ok()
        .and_then(|position| position.checked_add(1))
        .ok_or_else(|| latch_registry_row(c))
}

#[cfg(test)]
impl FrozenArena {
    /// Gives the row of the first link in the table of named documents the key of the root
    /// Document, so that the link has no row. Whether a link with a row was found.
    pub(crate) fn drop_link_document_for_test(&mut self) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        let Some(link) = storage.payloads.values.iter().find_map(|row| match row {
            RetainedPayload::Link { node, .. } => *node,
            _ => None,
        }) else {
            return false;
        };
        let Some(named) = storage.payloads.facts.contexts.named.as_mut() else {
            return false;
        };
        let (found, _) = named.find_published(link);
        found.is_some_and(|index| named.rekey_for_test(index, 0))
    }
}

/// The root of the tree that holds the record at `index` of a frozen parse, and the number of
/// parent edges followed to reach it. A test reads the document of a node it does not know the root
/// of; the parse itself never climbs.
#[cfg(test)]
pub(super) fn root_of(arena: &HtmlConstructionResult, index: usize) -> Option<(usize, u64)> {
    let mut at = index;
    let mut hops = 0_u64;
    for _ in 0..=arena.len() {
        match arena.node_at(at)?.parent() {
            Some(parent) => {
                at = parent.position();
                hops = hops.saturating_add(1);
            }
            None => return Some((at, hops)),
        }
    }
    None
}

/// The document of the record at `index` of a frozen parse, by the tree it is in, with the edges
/// followed to its root and the keys searched for it. A test reads this where the parse once kept
/// a row for the record.
#[cfg(test)]
pub(super) fn document_of<'a>(
    contexts: &'a NodeContexts,
    arena: &HtmlConstructionResult,
    index: usize,
) -> Option<(TreeDocument<'a>, u64)> {
    let (root, hops) = root_of(arena, index)?;
    let is_document = matches!(
        arena.node_at(root)?.kind(),
        ConstructionNodeKind::Document(_)
    );
    let (document, probes) = contexts.tree_document(root, is_document).ok()?;
    Some((document, hops.saturating_add(probes)))
}

#[cfg(test)]
pub(super) fn frozen_document(
    contexts: &NodeContexts,
    arena: &HtmlConstructionResult,
    index: usize,
) -> Option<DocumentRef> {
    match document_of(contexts, arena, index)?.0 {
        TreeDocument::Itself(root) => Some(DocumentRef::Owned(root)),
        TreeDocument::Named(document) => Some(document.clone()),
        TreeDocument::Unnamed => None,
    }
}
