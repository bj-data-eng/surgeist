//! The sole mutable owner of shared HTML construction records.

use super::*;

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
        self.arena().lookup(id)
    }
    /// Reads a retained ordinal; it never interprets an integer as a result identity.
    pub fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        self.arena().record_at(position)
    }
    /// The number of retained records, including roots, template contents and shadow contents
    /// fragments.
    pub fn len(&self) -> usize {
        self.arena().records.len()
    }
    /// Whether the result retains no records. It is the exact counterpart of `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.arena().records.is_empty()
    }
    /// Resolves a builder handle issued by this result's construction owner to its retained
    /// ordinal. Returns the foreign-builder error for another owner's handle and the unknown-node
    /// error for an ordinal past the retained records.
    pub fn position(&self, handle: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        Ok(self.index(handle)?.position())
    }
    /// The record index of a builder handle, with the errors of `position`.
    pub(super) fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        self.arena().index(handle)
    }
    /// Resolves a result id into a retained-record view. Returns `ForeignHtmlResult` for an id of
    /// another construction result or one past the retained records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        self.arena().construction_node(id)
    }
    /// The usage that construction admitted for this result.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena().usage
    }
    /// Whether another owner holds the records of this result: a clone of the result, a fragment
    /// view of the same arena, or any weak handle on it. It is the condition under which the
    /// arena cannot be taken for exclusive use, so a `false` answer means that no other owner
    /// exists and none can appear unless this result is cloned.
    pub fn is_shared(&self) -> bool {
        let arena = self.arena();
        Arc::strong_count(arena) > 1 || Arc::weak_count(arena) > 0
    }
    pub(super) fn arena(&self) -> &Arc<Arena> {
        match self {
            Self::Document(v) => &v.arena,
            Self::Fragment(v) => &v.arena,
        }
    }
}

/// A read-only view qualified by one live storage borrow.
#[derive(Clone, Copy, Debug)]
pub struct HtmlConstructionNode<'a> {
    pub(super) arena: &'a Arena,
    pub(super) index: NodeIx,
}
impl<'a> HtmlConstructionNode<'a> {
    /// The retained record ordinal that this view reads, in creation order.
    pub fn position(self) -> usize {
        self.index.position()
    }
    /// Converts to the ordinary node view over the same record.
    pub fn as_node(self) -> HtmlNode<'a> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
    }
    /// The record this view reads, or the unknown-node error when the index names none.
    fn record(self) -> Result<&'a Record, HtmlConstructionError> {
        self.arena
            .records
            .get(self.index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())
    }
    /// Checks only this retained record's generic container invariant.
    pub fn check_container(self) -> Result<(), HtmlConstructionError> {
        let record = self.record()?;
        if record.template_contents.is_some() {
            return Err(HtmlConstructionCause::TemplateContentsRequired.into());
        }
        if !matches!(
            record.kind,
            NodeContent::Document(_) | NodeContent::Fragment | NodeContent::Element(_)
        ) {
            return Err(HtmlConstructionCause::ParentCannotHaveChildren.into());
        }
        Ok(())
    }
    /// Checks the fixed movable-node invariant, without scanning ancestry or order.
    pub fn check_detachment(self) -> Result<(), HtmlConstructionError> {
        let record = self.record()?;
        if matches!(record.kind, NodeContent::Document(_)) {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        if record.owner.is_some() {
            return Err(HtmlConstructionCause::AssociationMove.into());
        }
        if matches!(record.kind, NodeContent::Fragment) {
            return Err(HtmlConstructionCause::RootMove.into());
        }
        Ok(())
    }
    /// Qualifies an actual retained reference child against the supplied parent.
    pub fn check_reference_child_of(self, parent: Self) -> Result<(), HtmlConstructionError> {
        if !std::ptr::eq(self.arena, parent.arena) {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        if self.record()?.parent != Some(parent.index) {
            return Err(HtmlConstructionCause::WrongSibling.into());
        }
        Ok(())
    }
    /// Issues a builder handle for this record from the owner's serial. The handle holds no
    /// reference to the owner.
    pub fn handle(self) -> HtmlBuilderNode {
        HtmlBuilderNode {
            tree: self.arena.serial,
            index: self.index,
        }
    }
    /// The size in bytes of the content a record stores, which a consumer charges when it
    /// inspects a record's kind.
    pub const CONTENT_BYTES: usize = size_of::<NodeContent>();
    /// The record's content, by value, with its text as borrowed code units.
    pub fn kind(self) -> ConstructionNodeKind<'a> {
        self.content().building()
    }
    fn content(self) -> &'a NodeContent {
        &self.arena.records[self.index.position()].kind
    }
    /// The number of ordinary children, excluding template contents and shadow contents.
    pub fn child_count(self) -> usize {
        self.arena.records[self.index.position()].children.len()
    }
    /// The child at `ordinal` in document order; `None` at or past `child_count`.
    pub fn child_at(self, ordinal: usize) -> Option<Self> {
        self.arena.records[self.index.position()]
            .children
            .get(ordinal)
            .copied()
            .map(|index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The ordinary parent. Records without one, such as roots and association fragments, return
    /// `None`.
    pub fn parent(self) -> Option<Self> {
        self.arena.records[self.index.position()]
            .parent
            .map(|index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The ordinary children in document order. The iterator knows its length and runs from both
    /// ends.
    pub fn children(self) -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + 'a {
        self.arena.records[self.index.position()]
            .children
            .iter()
            .map(move |&index| Self {
                arena: self.arena,
                index,
            })
    }
    /// The contents fragment of a `template` element, whether owned or a shadow-contents
    /// reference. `None` for any other record.
    pub fn template_contents(self) -> Option<Self> {
        self.arena.records[self.index.position()]
            .template_contents
            .map(|value| Self {
                arena: self.arena,
                index: value.position(),
            })
    }
    /// Whether the template's contents fragment is owned or a shadow reference; `None` for a
    /// record without template contents.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index.position()]
            .template_contents
            .map(TemplateContents::kind)
    }
    /// The declarative shadow root attached to this host element, if any.
    pub fn shadow_root(self) -> Option<HtmlDeclarativeShadowRoot<'a>> {
        HtmlNode {
            arena: self.arena,
            index: self.index,
        }
        .declarative_shadow_root()
    }
    /// The template or shadow host that owns this fragment root. `None` for ordinary records.
    pub fn fragment_owner(self) -> Option<&'a HtmlFragmentOwner> {
        self.arena.records[self.index.position()].owner.as_ref()
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// The shadow properties of a shadow contents fragment; `None` otherwise.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        self.arena.records[self.index.position()].shadow_properties
    }
}

/// Exclusive ownership of the actual construction arena, before publication.
#[derive(Debug)]
pub struct HtmlConstructionStorage {
    arena: Arena,
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
        self.arena.record_at(position)
    }
    /// Resolves a builder handle to its record view. Returns the foreign-builder error for another
    /// owner's handle and the unknown-node error for an ordinal past the created records.
    pub fn node(
        &self,
        handle: &HtmlBuilderNode,
    ) -> Result<HtmlConstructionNode<'_>, HtmlConstructionError> {
        let index = self.index(handle)?;
        Ok(HtmlConstructionNode {
            arena: &self.arena,
            index,
        })
    }
    /// Resolves a result id to its record view. Returns `ForeignHtmlResult` when the id's identity
    /// is not this owner's or it is past the created records.
    pub fn node_by_result_id(
        &self,
        id: &HtmlNodeId,
    ) -> Result<HtmlConstructionNode<'_>, ForeignHtmlResult> {
        self.arena.construction_node(id)
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
        for index in (len..self.arena.records.len()).rev() {
            let depth = self
                .arena
                .records
                .get(index)
                .map(|record| record.depth)
                .ok_or(HtmlConstructionCause::UnknownNode.error())?;
            self.arena.unpopulate(depth)?;
        }
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
        let weak = Arc::downgrade(result.arena());
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
