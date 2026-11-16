//! The document of every node of a parse, held while the parse builds and published by tree.
//!
//! While the parse builds, each record has one cell in a column: zero names no document, and any
//! other value names an entry of the list of the distinct documents the parse has named, one more
//! than its position. A document is named by many nodes, so the list is deduplicated and short.
//! The column is written when a record is created and when an adoption moves a node to another
//! document. Freezing the parse publishes what the column says and releases it: a tree has the
//! document of its root, and a table holds the entry named by each node that is asked for by its
//! own key, a root that is not a Document and a link. A Document root names itself.

use super::family_table::FamilyTable;
use super::row_bank::RowBank;
use super::row_log::RowLog;
use super::session_error::latch_registry_row;
use super::*;
#[cfg(test)]
use surgeist_html::construction::HtmlConstructionResult;

pub(super) struct NodeContexts {
    /// One cell per record, `None` once the parse is frozen.
    column: Option<RowBank<u32>>,
    /// The distinct documents the nodes of the parse name, in the order they were first named.
    documents: RowLog<DocumentRef>,
    /// The entry of `documents` that each root which is not a Document names, and each link that
    /// names one, built when the parse is frozen. A link is asked for its document by its own key,
    /// so the answer is one search whatever the depth it is nested at.
    named: Option<FamilyTable<u32>>,
    /// The column as it was when the parse was frozen, for tests that compare it with the
    /// published answers.
    #[cfg(test)]
    pub(super) frozen_column: Vec<u32>,
}

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
        let documents = match RowLog::new(control) {
            Ok(documents) => documents,
            Err(error) => {
                let owner = control.identity();
                column.release(control, &owner, |_, _| Ok(()), |_| Ok(()))?;
                return Err(error);
            }
        };
        Ok(Self {
            column: Some(column),
            documents,
            named: None,
            #[cfg(test)]
            frozen_column: Vec::new(),
        })
    }
    /// The distinct documents the nodes of the parse name.
    pub(super) fn documents(&self) -> &RowLog<DocumentRef> {
        &self.documents
    }
    /// The bytes of the column's vector, which the parse holds until it is frozen.
    #[cfg(test)]
    pub(super) fn column_capacity_bytes(&self) -> usize {
        self.column.as_ref().map_or(0, RowBank::capacity_bytes)
    }
    /// The same list, for a test that corrupts what a report holds.
    #[cfg(test)]
    pub(super) fn documents_mut(&mut self) -> &mut RowLog<DocumentRef> {
        &mut self.documents
    }
    /// The table of the roots and links that name a document, for a test that corrupts what a
    /// report holds.
    #[cfg(test)]
    pub(super) fn named_mut(&mut self) -> Option<&mut FamilyTable<u32>> {
        self.named.as_mut()
    }
    /// The document an entry of the list names.
    pub(super) fn document(&self, entry: u32) -> Option<&DocumentRef> {
        self.documents
            .row(usize::try_from(entry.checked_sub(1)?).ok()?)
    }
    /// The entry a record's cell names, zero for none; `None` for a record without a cell or a
    /// frozen parse.
    pub(super) fn cell(&self, node: usize) -> Option<u32> {
        self.column.as_ref()?.row(node).copied()
    }
    /// The document the tree rooted at `root` has, with the number of keys the search read.
    /// `root_is_document` is whether the record is a Document.
    pub(super) fn tree_document(
        &self,
        root: usize,
        root_is_document: bool,
    ) -> (TreeDocument<'_>, u64) {
        if root_is_document {
            return (TreeDocument::Itself(root), 0);
        }
        let (named, probes) = self.named_document(root);
        (
            named.map_or(TreeDocument::Unnamed, TreeDocument::Named),
            probes,
        )
    }
    /// The document the record at `node` names in the table built when the parse was frozen, with
    /// the number of keys the search read: a root that is not a Document, or a link, whose own cell
    /// named one. A link whose cell named none is in no document.
    pub(super) fn named_document(&self, node: usize) -> (Option<&DocumentRef>, u64) {
        let Some(named) = &self.named else {
            return (None, 0);
        };
        let (found, probes) = named.find_published(node);
        let document = found
            .and_then(|index| named.row(index))
            .and_then(|&entry| self.document(entry));
        (document, probes)
    }
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        let Self {
            column,
            documents,
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
        keep_first(
            &mut refused,
            documents.release(control, owner, |_, _| Ok(())),
        );
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
    /// The entry of the list that names `document`, naming it first when it is new. One record
    /// visit is charged for each entry searched.
    #[cfg_attr(test, track_caller)]
    pub(super) fn intern_document(
        &mut self,
        c: &mut ParseControl,
        document: DocumentRef,
    ) -> Result<u32, SessionError> {
        let documents = &mut self.payloads.facts.contexts.documents;
        for (position, held) in documents.iter() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if *held == document {
                return entry_of(c, position);
            }
        }
        documents.admit(c, 1)?;
        let position = documents.push(document)?;
        entry_of(c, position)
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
    /// for each record, and one row for each node that names a document and is a root that is not
    /// a Document or a link. A Document root names itself; one that names another document has no
    /// representation, and is refused as the node document defect.
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
                if record.parent().is_some() && !is_link {
                    continue;
                }
                let entry = self
                    .payloads
                    .facts
                    .contexts
                    .cell(node)
                    .ok_or_else(|| latch_registry_row(c))?;
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
            let released = named.release(c, &owner, |_, _| Ok(()));
            return released.and(Err(error));
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

/// The entry that names the document at `position` of the list.
fn entry_of(c: &mut ParseControl, position: usize) -> Result<u32, SessionError> {
    u32::try_from(position)
        .ok()
        .and_then(|position| position.checked_add(1))
        .ok_or_else(|| latch_registry_row(c))
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
    let (document, probes) = contexts.tree_document(root, is_document);
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
