//! The sole mutable owner of shared HTML construction records.

use super::*;
use std::slice;

pub(crate) mod attribute_list;

pub(crate) mod records;

pub(crate) mod text_data;

/// A storage and records made through the direct operations, with an admission that grants every
/// request, for the tests of the modules that edit records.
#[cfg(test)]
mod free {
    use super::*;
    use crate::internal::construction::{NativeAdmission, native};

    pub(super) fn storage() -> HtmlConstructionStorage {
        HtmlConstructionStorage::new(&mut NativeAdmission::exact())
            .map_err(native)
            .expect("an empty storage is made")
    }
    pub(super) fn element(
        storage: &mut HtmlConstructionStorage,
        name: QualifiedName,
        attributes: OrderedAttributes,
    ) -> HtmlBuilderNode {
        storage
            .create_element(&name, attributes, None, &mut NativeAdmission::exact())
            .map_err(native)
            .expect("an element is made")
    }
    pub(super) fn comment(storage: &mut HtmlConstructionStorage, text: &str) -> HtmlBuilderNode {
        let units: Vec<u16> = text.encode_utf16().collect();
        storage
            .create_comment(&units, &mut NativeAdmission::exact())
            .map_err(native)
            .expect("a comment is made")
    }
    pub(super) fn text(storage: &mut HtmlConstructionStorage, units: &[u16]) -> HtmlBuilderNode {
        storage
            .create_text(units, &mut NativeAdmission::exact())
            .map_err(native)
            .expect("a text node is made")
    }
}

/// Shared immutable output with the same identity and records as its storage.
#[derive(Clone, Debug)]
pub enum HtmlConstructionResult {
    /// A published Document result.
    Document(HtmlDocument),
    /// A published fragment result.
    Fragment(HtmlFragment),
}
impl HtmlConstructionResult {
    /// The Document when this result is one; `None` for a fragment result.
    pub fn document(&self) -> Option<&HtmlDocument> {
        match self {
            Self::Document(v) => Some(v),
            _ => None,
        }
    }
    /// The fragment when this result is one; `None` for a Document result.
    pub fn fragment(&self) -> Option<&HtmlFragment> {
        match self {
            Self::Fragment(v) => Some(v),
            _ => None,
        }
    }
    /// Resolves a result id into this result's node view. Returns `ForeignHtmlResult` when the id
    /// belongs to another construction result or is out of range.
    pub fn node(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        self.tree().lookup(id)
    }
    /// Reads a retained ordinal; it never interprets an integer as a result identity.
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        let tree = self.tree();
        tree.has_record(position)
            .map(|index| HtmlConstructionNode::published(tree, index))
    }
    /// The number of retained records, including roots, template contents and shadow contents
    /// fragments.
    pub fn len(&self) -> usize {
        self.tree().rows.len()
    }
    /// Whether the result retains no records. It is the exact counterpart of `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.tree().rows.is_empty()
    }
    /// Resolves a builder handle issued by this result's construction owner to its retained
    /// ordinal. Returns the foreign-builder error for another owner's handle and the unknown-node
    /// error for an ordinal past the retained records.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        Ok(self.index(handle)?.position())
    }
    /// The record index of a builder handle, with the errors of `position`.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        self.tree().index(handle)
    }
    /// Resolves a result id into a retained-record view. Returns `ForeignHtmlResult` for an id of
    /// another construction result or one past the retained records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        let tree = self.tree();
        tree.lookup_index(id)
            .map(|index| HtmlConstructionNode::published(tree, index))
    }
    /// The usage that construction admitted for this result.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.tree().usage
    }
    /// Whether another owner holds the records of this result: a clone of the result, a fragment
    /// view of the same tree, or any weak handle on it. It is the condition under which the
    /// tree cannot be taken for exclusive use, so a `false` answer means that no other owner
    /// exists and none can appear unless this result is cloned.
    pub fn is_shared(&self) -> bool {
        let tree = self.tree();
        Arc::strong_count(tree) > 1 || Arc::weak_count(tree) > 0
    }
    pub(super) fn tree(&self) -> &Arc<Tree> {
        match self {
            Self::Document(v) => &v.tree,
            Self::Fragment(v) => &v.tree,
        }
    }
}

/// The records a node view reads: those of a storage that builds, with the text arena its text
/// records name, or those of a published result with the view of the node it reads.
#[derive(Clone, Copy, Debug)]
enum Records<'a> {
    Build {
        arena: &'a BuildArena,
        text: &'a TextArena,
        attrs: &'a AttributeTable,
    },
    Published(HtmlNode<'a>),
}

/// The ordinary children of a record of either form, from both ends.
#[derive(Clone, Debug)]
enum ChildViews<'a> {
    Build(std::collections::vec_deque::Iter<'a, NodeIx>),
    Published(slice::Iter<'a, NodeIx>),
}
/// The ordinary children of a record in document order, as views of the same records.
#[derive(Clone, Debug)]
struct Children<'a> {
    of: HtmlConstructionNode<'a>,
    indices: ChildViews<'a>,
}
impl<'a> Iterator for Children<'a> {
    type Item = HtmlConstructionNode<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = match &mut self.indices {
            ChildViews::Build(indices) => indices.next(),
            ChildViews::Published(indices) => indices.next(),
        };
        index.map(|&index| self.of.at(index))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = match &mut self.indices {
            ChildViews::Build(indices) => indices.nth(n),
            ChildViews::Published(indices) => indices.nth(n),
        };
        index.map(|&index| self.of.at(index))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.indices {
            ChildViews::Build(indices) => indices.size_hint(),
            ChildViews::Published(indices) => indices.size_hint(),
        }
    }
}
impl DoubleEndedIterator for Children<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = match &mut self.indices {
            ChildViews::Build(indices) => indices.next_back(),
            ChildViews::Published(indices) => indices.next_back(),
        };
        index.map(|&index| self.of.at(index))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let index = match &mut self.indices {
            ChildViews::Build(indices) => indices.nth_back(n),
            ChildViews::Published(indices) => indices.nth_back(n),
        };
        index.map(|&index| self.of.at(index))
    }
}
impl ExactSizeIterator for Children<'_> {}

/// A read-only view qualified by one live storage borrow, or by a published result.
#[derive(Clone, Copy, Debug)]
pub struct HtmlConstructionNode<'a> {
    records: Records<'a>,
    index: NodeIx,
}
impl<'a> HtmlConstructionNode<'a> {
    /// A view of the record at `index` of a storage that builds.
    pub(super) fn building(
        arena: &'a BuildArena,
        text: &'a TextArena,
        attrs: &'a AttributeTable,
        index: NodeIx,
    ) -> Self {
        Self {
            records: Records::Build { arena, text, attrs },
            index,
        }
    }
    /// A view of the record at `index` of a published result.
    pub(super) fn published(tree: &'a Tree, index: NodeIx) -> Self {
        Self {
            records: Records::Published(tree.node(index)),
            index,
        }
    }
    /// The same views of another record of the same records.
    fn at(self, index: NodeIx) -> Self {
        match self.records {
            Records::Build { .. } => Self { index, ..self },
            Records::Published(node) => Self::published(node.tree, index),
        }
    }
    /// The retained record ordinal that this view reads, in creation order.
    pub fn position(self) -> usize {
        self.index.position()
    }
    /// Converts to the ordinary node view over the same record. A record of a storage that has
    /// not published it has no such view: the refusal is [`HtmlConstructionCause::NotPublished`].
    pub fn as_node(self) -> Result<HtmlNode<'a>, HtmlConstructionError> {
        match self.records {
            Records::Published(node) => Ok(node),
            Records::Build { .. } => Err(HtmlConstructionCause::NotPublished.error()),
        }
    }
    /// The fields of the record this view reads, or the unknown-node error when the view holds no
    /// record.
    fn record_with<R>(
        self,
        read: impl FnOnce(RecordFields) -> R,
    ) -> Result<R, HtmlConstructionError> {
        match self.records {
            Records::Build { arena, .. } => arena
                .records
                .get(self.index.position())
                .map(|record| read(RecordFields::of(record)))
                .ok_or(HtmlConstructionCause::UnknownNode.error()),
            Records::Published(node) => node
                .row
                .map(|row| read(RecordFields::published(node, row)))
                .ok_or(HtmlConstructionCause::UnknownNode.error()),
        }
    }
    /// Checks only this retained record's generic container invariant.
    pub fn check_container(self) -> Result<(), HtmlConstructionError> {
        let (has_template, container) =
            self.record_with(|record| (record.has_template_contents, record.container))?;
        if has_template {
            return Err(HtmlConstructionCause::TemplateContentsRequired.into());
        }
        if !container {
            return Err(HtmlConstructionCause::ParentCannotHaveChildren.into());
        }
        Ok(())
    }
    /// Checks the fixed movable-node invariant, without scanning ancestry or order.
    pub fn check_detachment(self) -> Result<(), HtmlConstructionError> {
        let (document, owned, fragment) =
            self.record_with(|record| (record.document, record.owned, record.fragment))?;
        if document {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        if owned {
            return Err(HtmlConstructionCause::AssociationMove.into());
        }
        if fragment {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        Ok(())
    }
    /// Qualifies an actual retained reference child against the supplied parent.
    pub fn check_reference_child_of(self, parent: Self) -> Result<(), HtmlConstructionError> {
        let same = match (self.records, parent.records) {
            (Records::Build { arena: a, .. }, Records::Build { arena: b, .. }) => {
                std::ptr::eq(a, b)
            }
            (Records::Published(a), Records::Published(b)) => std::ptr::eq(a.tree, b.tree),
            _ => false,
        };
        if !same {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        if self.record_with(|record| record.parent)? != Some(parent.index) {
            return Err(HtmlConstructionCause::WrongSibling.into());
        }
        Ok(())
    }
    /// Issues a builder handle for this record from the owner's serial. The handle holds no
    /// reference to the owner.
    pub fn handle(self) -> HtmlBuilderNode {
        HtmlBuilderNode {
            tree: match self.records {
                Records::Build { arena, .. } => arena.serial,
                Records::Published(node) => node.tree.serial,
            },
            index: self.index,
        }
    }
    /// The size in bytes of the content a record stores, which a consumer charges when it
    /// inspects a record's kind.
    pub const CONTENT_BYTES: usize = size_of::<NodeContent>();
    /// The record's content, by value, with its text as borrowed code units. A published record
    /// whose row names a reference its result does not hold reads as
    /// [`ConstructionNodeKind::Unreadable`], which a result a storage published never does.
    pub fn kind(self) -> ConstructionNodeKind<'a> {
        match self.records {
            Records::Build { arena, text, attrs } => arena.records[self.index.position()]
                .kind
                .building(text, attrs),
            Records::Published(node) => match node.row {
                Some(row) => node.tree.build_kind(row),
                None => ConstructionNodeKind::Unreadable(HtmlConstructionCause::UnknownNode),
            },
        }
    }
    /// The number of ordinary children, excluding template contents and shadow contents.
    pub fn child_count(self) -> usize {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()].children.len(),
            Records::Published(node) => node.children.len(),
        }
    }
    /// The child at `ordinal` in document order; `None` at or past `child_count`.
    pub fn child_at(self, ordinal: usize) -> Option<Self> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()]
                .children
                .get(ordinal)
                .copied(),
            Records::Published(node) => node.children.get(ordinal).copied(),
        }
        .map(|index| self.at(index))
    }
    /// The ordinary parent. Records without one, such as roots and association fragments, return
    /// `None`.
    pub fn parent(self) -> Option<Self> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()].parent,
            Records::Published(node) => node.row.and_then(|row| row.parent),
        }
        .map(|index| self.at(index))
    }
    /// The ordinary children in document order. The iterator knows its length and runs from both
    /// ends.
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'a {
        Children {
            of: self,
            indices: match self.records {
                Records::Build { arena, .. } => {
                    ChildViews::Build(arena.records[self.index.position()].children.iter())
                }
                Records::Published(node) => ChildViews::Published(node.children.iter()),
            },
        }
    }
    /// The contents fragment of a `template` element, whether owned or a shadow-contents
    /// reference. `None` for any other record.
    pub fn template_contents(self) -> Option<Self> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()]
                .template_contents
                .map(|value| self.at(value.position())),
            Records::Published(node) => node
                .template_contents()
                .and_then(|id| node.tree.lookup_index(&id).ok())
                .map(|index| self.at(index)),
        }
    }
    /// Whether the template's contents fragment is owned or a shadow reference; `None` for a
    /// record without template contents.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()]
                .template_contents
                .map(TemplateContents::kind),
            Records::Published(node) => node.template_contents_kind(),
        }
    }
    /// The declarative shadow root attached to this host element, if any.
    pub fn shadow_root(self) -> Option<HtmlDeclarativeShadowRoot> {
        match self.records {
            Records::Build { arena, .. } => arena.declarative_shadow_root(self.index),
            Records::Published(node) => node.declarative_shadow_root(),
        }
    }
    /// The template or shadow host that owns this fragment root. `None` for ordinary records.
    pub fn fragment_owner(self) -> Option<HtmlFragmentOwner> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()].owner,
            Records::Published(node) => node.fragment_owner(),
        }
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// The shadow properties of a shadow contents fragment; `None` otherwise.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        match self.records {
            Records::Build { arena, .. } => arena.records[self.index.position()].shadow_properties,
            Records::Published(node) => node.shadow_properties(),
        }
    }
}

/// The fields of a record that the checks of a view read.
#[derive(Clone, Copy)]
struct RecordFields {
    parent: Option<NodeIx>,
    has_template_contents: bool,
    owned: bool,
    container: bool,
    document: bool,
    fragment: bool,
}
impl RecordFields {
    fn of(record: &Record) -> Self {
        Self {
            parent: record.parent,
            has_template_contents: record.template_contents.is_some(),
            owned: record.owner.is_some(),
            container: record.kind.is_container(),
            document: matches!(record.kind, NodeContent::Document(_)),
            fragment: matches!(record.kind, NodeContent::Fragment),
        }
    }
    fn published(node: HtmlNode<'_>, row: &PublishedRow) -> Self {
        Self {
            parent: row.parent,
            has_template_contents: node.template_contents().is_some(),
            owned: node.fragment_owner().is_some(),
            container: matches!(
                row.content,
                RowContent::Document { .. }
                    | RowContent::Fragment { .. }
                    | RowContent::Element { .. }
            ),
            document: matches!(row.content, RowContent::Document { .. }),
            fragment: matches!(row.content, RowContent::Fragment { .. }),
        }
    }
}

/// Exclusive ownership of the actual construction arena, before publication.
#[derive(Debug)]
pub struct HtmlConstructionStorage {
    arena: BuildArena,
    /// The units of the text and comment records of `arena`, and of the values of the attributes
    /// of its element records.
    text: TextArena,
    /// The rows and runs of the attributes of the element records of `arena`.
    attrs: AttributeTable,
}
impl HtmlConstructionStorage {
    /// The admitted usage; it is zero for a new owner.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena.usage
    }
    /// The number of records created so far.
    pub fn len(&self) -> usize {
        self.arena.records.len()
    }
    /// Whether no record has been created.
    pub fn is_empty(&self) -> bool {
        self.arena.records.is_empty()
    }
    /// The record at `position`, or `None` past the last created record.
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        self.arena.has_record(position).map(|index| {
            HtmlConstructionNode::building(&self.arena, &self.text, &self.attrs, index)
        })
    }
    /// Resolves a builder handle to its record view. Returns the foreign-builder error for another
    /// owner's handle and the unknown-node error for an ordinal past the created records.
    pub fn node(
        &self,
        handle: &HtmlBuilderNode,
    ) -> Result<HtmlConstructionNode<'_>, HtmlConstructionError> {
        let index = self.index(handle)?;
        Ok(HtmlConstructionNode::building(
            &self.arena,
            &self.text,
            &self.attrs,
            index,
        ))
    }
    /// Resolves a result id to its record view. Returns `ForeignHtmlResult` when the id's identity
    /// is not this owner's or it is past the created records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        let index = self.arena.lookup_index(id)?;
        Ok(HtmlConstructionNode::building(
            &self.arena,
            &self.text,
            &self.attrs,
            index,
        ))
    }
    /// Resolves a builder handle to its record ordinal. Errors as `node` does.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        Ok(self.index(handle)?.position())
    }
    /// Resolves a builder handle to its record index. Errors as `node` does.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        self.arena.index(handle)
    }
    /// Resolves a builder handle to the result id that the published result will use for the same
    /// record. Errors as `node` does.
    pub fn result_id(&self, handle: &HtmlBuilderNode) -> Result<HtmlNodeId, HtmlConstructionError> {
        Ok(self.arena.id(self.index(handle)?))
    }
    /// The depth of the record at `position` within its own tree.
    pub(super) fn native_depth(&self, position: usize) -> Result<usize, HtmlConstructionError> {
        self.arena
            .records
            .get(position)
            .map(|record| record.depth)
            .ok_or(HtmlConstructionCause::UnknownNode.error())
    }
    // Native append composes creation and insertion before its fresh handle escapes.
    // Only an uncommitted insertion may reach this rollback; previous topology is intact.
    pub(super) fn rollback_native_append(
        &mut self,
        len: usize,
        usage: HtmlConstructionUsage,
    ) -> Result<(), HtmlConstructionError> {
        let mut forgotten = 0usize;
        for index in (len..self.arena.records.len()).rev() {
            let record = self
                .arena
                .records
                .get(index)
                .ok_or(HtmlConstructionCause::UnknownNode.error())?;
            let units = match &record.kind {
                NodeContent::Text(run) | NodeContent::Comment(run) => run.len(),
                // The values of the attributes of an element stay written in the arena as spare,
                // and the rows of its run stay in the table until publication leaves them out.
                NodeContent::Element(element) => {
                    let mut units = 0usize;
                    for row in self.attrs.live(element.attrs)? {
                        units = units
                            .checked_add(row.value().len())
                            .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
                    }
                    units
                }
                _ => 0,
            };
            forgotten = forgotten
                .checked_add(units)
                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
            let depth = record.depth;
            self.arena.unpopulate(depth)?;
        }
        self.text.forget(forgotten)?;
        self.arena.records.truncate(len);
        // A later record may take an ordinal given up here, so a table of attribute names that
        // described one of them must not be trusted.
        self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
        self.arena.usage = usage;
        Ok(())
    }
    /// The capacity of the record bank.
    #[cfg(test)]
    pub(super) fn records_capacity(&self) -> usize {
        self.arena.records.capacity()
    }
    /// The capacity of the child order of the record at `index`.
    #[cfg(test)]
    pub(super) fn children_capacity(&self, index: usize) -> Option<usize> {
        self.arena
            .records
            .get(index)
            .map(|record| record.children.capacity())
    }
}

/// The serials issued to construction storages. A serial names its storage among all storages in
/// the process: node ids and builder handles carry it, and no two storages share one.
static NEXT_STORAGE_SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// Takes the next serial from `counter`, or `RepresentationExhausted` when none is left. The
/// counter keeps its value on refusal, so a serial is never reissued.
fn issue_storage_serial(
    counter: &std::sync::atomic::AtomicU64,
) -> Result<u64, HtmlConstructionCause> {
    use std::sync::atomic::Ordering;
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .map_err(|_| HtmlConstructionCause::RepresentationExhausted)
}
#[cfg(test)]
mod serial_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn storage_serials_are_issued_in_order_and_exhaustion_is_a_typed_refusal() {
        let counter = AtomicU64::new(u64::MAX - 2);
        assert_eq!(issue_storage_serial(&counter), Ok(u64::MAX - 2));
        assert_eq!(issue_storage_serial(&counter), Ok(u64::MAX - 1));
        // The last value cannot be followed by another, so it is never issued: the counter would
        // have to wrap onto a serial an earlier storage may still hold.
        let refusal = issue_storage_serial(&counter);
        assert_eq!(refusal, Err(HtmlConstructionCause::RepresentationExhausted));
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        // The refusal repeats and the counter keeps its value.
        assert_eq!(
            issue_storage_serial(&counter),
            Err(HtmlConstructionCause::RepresentationExhausted)
        );
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}

#[cfg(test)]
mod shared_result_tests {
    use super::*;
    use crate::internal::construction::{Builder, NativeRoot};

    fn published() -> HtmlConstructionResult {
        let builder = Builder::new(
            HtmlConstructionLimits::new(4, 4, 0, 8, 3),
            NativeRoot::Fragment,
        )
        .unwrap();
        builder.finish().unwrap()
    }

    #[test]
    fn a_result_is_shared_while_a_clone_a_fragment_view_or_a_weak_handle_exists() {
        let result = published();
        assert!(!result.is_shared());
        let clone = result.clone();
        assert!(result.is_shared() && clone.is_shared());
        drop(clone);
        assert!(!result.is_shared());
        let view = result
            .fragment()
            .unwrap()
            .fragment(&result.fragment().unwrap().root().id());
        let view = view.unwrap().unwrap();
        assert!(result.is_shared());
        drop(view);
        assert!(!result.is_shared());
        let weak = Arc::downgrade(result.tree());
        assert!(result.is_shared());
        drop(weak);
        assert!(!result.is_shared());
    }
}

#[cfg(test)]
mod native_finish_tests {
    use super::*;
    use crate::internal::construction::{Builder, Creation, NativeRoot};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    /// Publication validates the whole tree again. A tree it refuses is an expected condition for
    /// a fallible publisher, so the native builder reports it and does not abort the caller. The
    /// child below lost the parent link that its parent's child list still names, which is the
    /// refusal the publication's own pass reports.
    #[test]
    fn finish_reports_a_tree_that_publication_refuses_and_never_panics() {
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(4, 4, 0, 8, 3),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root;
        builder
            .append(&root, Creation::Comment("c".into()))
            .unwrap();
        builder.storage.arena.records[1].parent = None;
        let outcome = catch_unwind(AssertUnwindSafe(|| builder.finish()));
        let Ok(result) = outcome else {
            panic!("finish panicked on a refused publication");
        };
        let error = result.expect_err("a tree with a lost parent link is refused");
        assert!(
            matches!(error.cause(), HtmlConstructionCause::ParentChildConflict),
            "{error}"
        );
    }
}

#[cfg(test)]
mod failed_append_tests {
    use super::*;
    use crate::internal::construction::{Builder, Creation, NativeRoot};

    /// An append whose record is created but whose insertion the allocator refuses is undone:
    /// the builder reports the typed error, its accounting gives back what the append admitted,
    /// and the storage holds the records and the usage it held before the append.
    #[test]
    fn an_insertion_the_allocator_refuses_after_a_creation_leaves_the_builder_as_it_was() {
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(8, 8, 0, 64, 4),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root;
        // The child order of the root grows to 4 entries with the first append, so four appends
        // fill it and the fifth needs the allocator.
        for _ in 0..4 {
            builder
                .append(&root, Creation::Comment("a".into()))
                .unwrap();
        }
        let (usage, accounted, len) = (
            builder.storage.usage(),
            builder.usage(),
            builder.storage.len(),
        );
        assert_eq!(usage, accounted);
        {
            let _refused = crate::internal::amortized::fault::refuse_children();
            let error = builder
                .append(&root, Creation::Comment("bc".into()))
                .unwrap_err();
            assert_eq!(error.cause(), HtmlConstructionCause::AllocationFailed);
        }
        assert_eq!(builder.usage(), accounted);
        assert_eq!(builder.storage.usage(), usage);
        assert_eq!(builder.storage.len(), len);
        assert_eq!(builder.storage.node(&root).unwrap().child_count(), 4);
        // The builder takes the same append once the allocator admits it, to the usage a builder
        // that never failed reaches.
        builder
            .append(&root, Creation::Comment("bc".into()))
            .unwrap();
        let mut fresh = Builder::new(
            HtmlConstructionLimits::new(8, 8, 0, 64, 4),
            NativeRoot::Fragment,
        )
        .unwrap();
        let fresh_root = fresh.root;
        for text in ["a", "a", "a", "a", "bc"] {
            fresh
                .append(&fresh_root, Creation::Comment(text.into()))
                .unwrap();
        }
        assert_eq!(builder.usage(), fresh.usage());
        assert_eq!(builder.storage.usage(), fresh.storage.usage());
        assert_eq!(builder.storage.len(), fresh.storage.len());
    }
}
