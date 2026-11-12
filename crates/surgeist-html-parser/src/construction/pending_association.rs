//! Bounded original occurrences and immutable inputs; no CSSOM execution.
use super::*;
use history::FrozenHistory;
use surgeist_html::construction::{ConstructionNodeKind, HtmlDocumentMode, HtmlNode, HtmlNodeId};

/// An original occurrence, qualified by its immutable construction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlParseAssociationOccurrenceId {
    result: HtmlNodeId,
    slot: usize,
}
/// An original context frontier. This is not resource-completion order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlParseAssociationContextEpochId {
    result: HtmlNodeId,
    slot: usize,
}
/// A deliberately unbound input at one exact original context frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlParseAssociationContextSlotId {
    epoch: HtmlParseAssociationContextEpochId,
    role: HtmlParseAssociationContextRole,
}
impl HtmlParseAssociationContextSlotId {
    pub fn epoch(&self) -> &HtmlParseAssociationContextEpochId {
        &self.epoch
    }
    pub fn role(&self) -> HtmlParseAssociationContextRole {
        self.role
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseAssociationContextRole {
    DocumentUrl,
    FallbackBase,
    Encoding,
    InitialPolicy,
    ExternalEnvironment,
    BodyPresence,
    DocumentEligibility,
}
/// Pending external input is distinct from lost original evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseAssociationContextRequirement {
    Known,
    OfflineUnbound,
    DeferredCaptured,
    Unavailable(HtmlParseAssociationMissingKind),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseAssociationMissingKind {
    UnsupportedLinkStylesheetHistory,
    ExtensionLinkHistory,
    OriginalContext,
    PolicyHistory,
}
/// Completeness of capture, never completeness of CSSOM processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseAssociationCaptureDisposition {
    PendingComplete,
    Unavailable(HtmlParseAssociationMissingKind),
}
/// Closed original-controller operations. None asserts successful sheet addition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseAssociationOccurrenceKind {
    StyleCreated,
    StyleParserCreated,
    StyleUpdate(surgeist_html::elements::style::HtmlStyleTrigger),
    StyleOwnerAttributes,
    StyleDocumentChanged,
    ChangePreferred,
    BaseChanged,
    UnsupportedLink,
    UnsupportedLinkExtension,
}

/// A value an occurrence reads after the parse: a copy of a field as it was
/// when the occurrence was recorded, held by a row of its own. The field may
/// change or be replaced afterwards, and the occurrence keeps what it saw.
#[derive(Clone, Copy)]
pub(super) struct ValueRow(usize);

pub(super) struct DocumentCapture {
    document: DocumentRef,
    count: usize,
    obligations: usize,
    last: Option<usize>,
    missing: Option<HtmlParseAssociationMissingKind>,
    missing_at: Option<usize>,
}
pub(super) struct CapturedContext {
    value: surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
    usage: associated::AssociatedStorageUsage,
}
pub(super) struct Inputs {
    state: surgeist_html::elements::style::HtmlStyleState,
    parser_document: Option<DocumentRef>,
    connected: bool,
    type_eligible: bool,
    document_tree: bool,
    attributes: [Option<ValueRow>; 5],
    text: OwnedVec<ValueRow>,
    /// The tree's style elements in tree order when this occurrence was
    /// recorded: the first `len` entries of a shared `StyleOrder` row.
    order: Option<OrderRef>,
}
#[derive(Clone, Copy)]
struct OrderRef {
    slot: usize,
    len: usize,
}
/// Record 0 roots the main tree.
const MAIN_TREE: usize = 0;
/// The row listing the styles of the subtree at `root` that is being removed,
/// built by the first removal trigger of its styles. The subtree keeps its
/// styles while the removal hooks run, so every trigger of the removal lists
/// the same ones; any insertion hook, and the removal of anything else, ends
/// the removal and drops the row.
#[derive(Clone, Copy)]
pub(super) struct DetachedOrder {
    root: usize,
    slot: usize,
}
/// A subtree being inserted whose first style could not extend its tree's
/// order. Its hooks run in tree order after the whole subtree is in place, so
/// a row built from `from` on, by the trigger of that first style, lists every
/// style of the subtree and the hooks of the others have nothing to apply.
#[derive(Clone, Copy)]
pub(super) struct StyleSubtree {
    root: usize,
    /// The length of the registry when the first hook found the order
    /// unable to take its style; a row at or after it was built since.
    from: usize,
}
pub(super) struct Context {
    input_document: usize,
    mode: HtmlDocumentMode,
    class: crate::HtmlDocumentClass,
    body: bool,
    base_document: Option<usize>,
    first_base: Option<usize>,
    base_input: Option<ValueRow>,
    base_observation: Option<usize>,
    captured: Option<usize>,
    policy_missing: bool,
    csp: Option<usize>,
    missing: bool,
}
pub(super) struct Occurrence {
    subject: usize,
    document: DocumentRef,
    root: usize,
    previous_document: Option<DocumentRef>,
    prior_document: Option<usize>,
    prior_style: Option<usize>,
    cause: Option<provenance::ConstructionCauseRef>,
    kind: HtmlParseAssociationOccurrenceKind,
    inputs: Option<Inputs>,
    preferred: Option<ValueRow>,
    base_changed_input: Option<ValueRow>,
    context: Context,
    requires_processing: bool,
    may_have_sheet: bool,
}

#[cfg(test)]
impl FrozenArena {
    /// Names, as the first Base of one occurrence, a node the arena does not
    /// hold. Whether an occurrence was found to name it.
    pub(crate) fn corrupt_first_base_for_test(&mut self) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        for row in &mut storage.payloads.values {
            if let RetainedPayload::AssociationOccurrence(values) = row
                && let Some(first) = values.as_mut_slice_for_test().first_mut()
            {
                first.context.first_base = Some(usize::MAX);
                return true;
            }
        }
        false
    }
}

/// The subject of a style occurrence row: one that carries style inputs.
#[cfg(test)]
pub(super) fn style_occurrence_subject(row: &RetainedPayload) -> Option<usize> {
    match row {
        RetainedPayload::AssociationOccurrence(values) => {
            let first = &values.as_slice()[0];
            first.inputs.is_some().then_some(first.subject)
        }
        _ => None,
    }
}

impl SessionStorage {
    fn association_document(
        &mut self,
        c: &mut ParseControl,
        document: &DocumentRef,
    ) -> Result<usize, SessionError> {
        for &slot in self.payloads.kind_list(KindList::AssociationDocument) {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values.get(slot), Some(RetainedPayload::AssociationDocument(value)) if &value.document == document)
            {
                return Ok(slot);
            }
        }
        let document = document.try_clone(c)?;
        payload_write::<DocumentCapture>(c, 1)?;
        let prepared = self.prepare_payload_slot_for(c, Some(KindList::AssociationDocument))?;
        let slot = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::AssociationDocument(DocumentCapture {
                    document,
                    count: 0,
                    obligations: 0,
                    last: None,
                    missing: None,
                    missing_at: None,
                }),
            )
            .map_err(|_| latch_registry_row(c))?;
        Ok(slot)
    }
    /// A copy of the string at `slot` in a row of its own, so the occurrence
    /// adds no alias to mutable or reclaimable backing and keeps the value it
    /// saw.
    fn association_string(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<ValueRow, SessionError> {
        let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(slot) else {
            return Err(latch_registry_row(c));
        };
        let mut buffer = Utf16Buffer::new(c)?;
        if let Err(stop) = buffer.extend_units(value.code_units(), c) {
            buffer.release(c)?;
            return Err(stop.into());
        }
        let frozen = buffer.freeze(c)?;
        let (native, row) = self.register_group_string_owned(c, frozen, None)?;
        drop(native);
        Ok(ValueRow(row))
    }
    /// A copy of the string of `field`, a field of a record, in a row of its own, as
    /// [`Self::association_string`] makes of a row's string.
    fn association_field(
        &mut self,
        c: &mut ParseControl,
        field: FieldRef,
    ) -> Result<ValueRow, SessionError> {
        let mut buffer = Utf16Buffer::new(c)?;
        let read = (|| {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let records = ConstructionView::Active(&self.construction);
            let Some(units) = field.units(records) else {
                return Err(latch_registry_row(c));
            };
            buffer.extend_units(units, c).map_err(SessionError::from)
        })();
        if let Err(error) = read {
            buffer.release(c)?;
            return Err(error);
        }
        let frozen = buffer.freeze(c)?;
        let (native, row) = self.register_group_string_owned(c, frozen, None)?;
        drop(native);
        Ok(ValueRow(row))
    }
    fn association_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<Option<ValueRow>, SessionError> {
        let mut position = None;
        let ConstructionNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Ok(None);
        };
        for (index, attr) in element.attributes().each(c)?.enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                position = Some(index);
                break;
            }
        }
        let Some(position) = position else {
            return Ok(None);
        };
        self.association_field(c, FieldRef::AttributeValue(node, position))
            .map(Some)
    }
    /// The internal nonce row's value, separate from the potentially hidden
    /// authored attribute. One join cell and one row, whatever else is retained.
    fn association_nonce(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<ValueRow>, SessionError> {
        if let Some(slot) = self.payloads.joined(c, node, Join::Nonce)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Nonce {
                node: Some(owner),
                raw,
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                let raw = *raw;
                return self.association_string(c, raw).map(Some);
            }
        }
        Ok(None)
    }
    fn association_style_inputs(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<Inputs>, SessionError> {
        if !self.html_local(c, node, "style")? {
            return Ok(None);
        }
        let mut state = None;
        if let Some(slot) = self.payloads.joined(c, node, Join::Style)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Style {
                node: owner,
                state: value,
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                state = Some((
                    value.state,
                    value
                        .parser_document
                        .as_ref()
                        .map(|d| d.try_clone(c))
                        .transpose()?,
                ));
            }
        }
        let (state, parser_document) = state.ok_or(SessionError::WrongToken)?;
        let connected = self.shadow_including_connected(c, node, detached)?;
        let type_units = self.form_attribute(c, node, "type")?;
        let type_eligible =
            surgeist_html::elements::style::style_type_allows_css_with(type_units, || {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)
            })?;
        let root = self.ordinary_root(c, node, detached)?;
        let document_tree = matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::WrongToken)?
                .kind(),
            ConstructionNodeKind::Document(_)
        );
        let mut text = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let result = (|| {
            let mut attributes = [None, None, None, None, None];
            // One slot per name: the array and the names are the same length.
            for (slot, local) in attributes
                .iter_mut()
                .zip(["type", "media", "title", "blocking", "nonce"])
            {
                *slot = self.association_attribute(c, node, local)?;
            }
            // Internal nonce is separate from the potentially hidden authored attribute.
            if let Some(nonce) = self.association_nonce(c, node)? {
                attributes[4] = Some(nonce);
            }
            let mut next = self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .child_at(0)
                .map(|n| n.position());
            while let Some(child) = next {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let record = self
                    .construction
                    .node_at(child)
                    .ok_or(SessionError::ForeignNode)?;
                next = self.next_sibling(c, child)?;
                if detached == Some(child) {
                    continue;
                }
                if matches!(record.kind(), ConstructionNodeKind::Text(_)) {
                    let group = self.association_field(c, FieldRef::Text(child))?;
                    text.try_push_geometric(c, group)?;
                }
            }
            let order = self.style_order(c, root, detached)?;
            Ok::<_, SessionError>((attributes, order))
        })();
        match result {
            Ok((attributes, order)) => Ok(Some(Inputs {
                state,
                parser_document,
                connected,
                type_eligible,
                document_tree,
                attributes,
                text,
                order,
            })),
            Err(error) => {
                text.release(c)?;
                Err(error)
            }
        }
    }
    /// The tree's style elements in tree order, as a prefix of a shared row.
    /// Each tree keeps one row, named by a join of the tree's root, that every
    /// insertion of a style element at the end of the tree extends; any other
    /// change to a tree's style elements drops its row, and a removal drops
    /// every tree's, for the next trigger to rebuild by walking the tree.
    fn style_order(
        &mut self,
        c: &mut ParseControl,
        root: usize,
        detached: Option<usize>,
    ) -> Result<Option<OrderRef>, SessionError> {
        let cacheable = detached.is_none();
        if cacheable && root == MAIN_TREE && self.main_tree_styleless {
            #[cfg(test)]
            assert_eq!(
                self.uncharged_style_order(root),
                Vec::<usize>::new(),
                "a main tree known to be styleless holds no style element"
            );
            return Ok(None);
        }
        // The tree of a removal's trigger is the subtree being removed when the
        // climb from the style stops at the removed root.
        let removed_tree = detached == Some(root);
        let cached = if cacheable {
            self.current_style_order(c, root)?
        } else if removed_tree {
            self.detached_order
                .filter(|order| order.root == root)
                .map(|order| order.slot)
        } else {
            None
        };
        if let Some(slot) = cached {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let Some(RetainedPayload::StyleOrder(list)) = self.payloads.values.get(slot) else {
                return Err(latch_registry_row(c));
            };
            let order = Some(OrderRef {
                slot,
                len: list.len(),
            });
            #[cfg(test)]
            assert_eq!(
                list.as_slice(),
                self.uncharged_style_order(root).as_slice(),
                "the extended style order equals the tree's walk"
            );
            return Ok(order);
        }
        let mut list = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let walked = (|| {
            let mut next = Some(root);
            while let Some(candidate) = next {
                if self.html_local(c, candidate, "style")? {
                    list.try_push_geometric(c, candidate)?;
                }
                next = self.next_view_node(c, root, candidate, detached)?;
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = walked {
            list.release(c)?;
            return Err(error);
        }
        let order = self.retain_style_order(c, list)?;
        if let Some(order) = order {
            if cacheable {
                self.payloads
                    .join_latest(root, Join::StyleOrder, order.slot)?;
            } else if removed_tree {
                self.detached_order = Some(DetachedOrder {
                    root,
                    slot: order.slot,
                });
            }
        }
        Ok(order)
    }
    /// The row listing the style elements of the tree at `root`, when it is
    /// current.
    fn current_style_order(
        &self,
        c: &mut ParseControl,
        root: usize,
    ) -> Result<Option<usize>, SessionError> {
        Ok(self
            .payloads
            .joined(c, root, Join::StyleOrder)?
            .filter(|slot| *slot >= self.style_order_floor))
    }
    /// Registers a style order row, or releases an empty list.
    fn retain_style_order(
        &mut self,
        c: &mut ParseControl,
        list: OwnedVec<usize>,
    ) -> Result<Option<OrderRef>, SessionError> {
        if list.len() == 0 {
            list.release(c)?;
            return Ok(None);
        }
        let prepared = match self.prepare_payload_slot_owned(c) {
            Ok(prepared) => prepared,
            Err(error) => {
                list.release(c)?;
                return Err(error);
            }
        };
        let len = list.len();
        let slot = self
            .payloads
            .push_row(prepared, RetainedPayload::StyleOrder(list))
            .map_err(|_| latch_registry_row(c))?;
        Ok(Some(OrderRef { slot, len }))
    }
    /// Applies a style element's insertion or removal to the style order of
    /// its tree. A removal, or an insertion into a detaching subtree, drops
    /// every tree's order; an insertion that is not after every listed style
    /// drops its tree's order, for the next trigger to rebuild. `root` is the
    /// root of the subtree whose insertion or removal is being applied.
    fn style_order_changed(
        &mut self,
        c: &mut ParseControl,
        style: usize,
        detached: Option<usize>,
        removal: bool,
        root: usize,
    ) -> Result<(), SessionError> {
        if !(removal
            && self
                .detached_order
                .is_some_and(|order| detached == Some(order.root)))
        {
            self.detached_order = None;
        }
        if removal || detached.is_some() {
            self.style_order_floor = self.payloads.values.len();
            self.main_tree_styleless = false;
            self.style_subtree = None;
            return Ok(());
        }
        let tree = self.ordinary_root(c, style, None)?;
        let current = self.current_style_order(c, tree)?;
        if root != style
            && let Some(slot) = current
            && self
                .style_subtree
                .is_some_and(|subtree| subtree.root == root && slot >= subtree.from)
        {
            return Ok(());
        }
        // The main tree before its first style has an empty order and no row.
        let empty = current.is_none() && tree == MAIN_TREE && self.main_tree_styleless;
        if current.is_none() && !empty {
            self.note_unextended_subtree(root, style);
            return Ok(());
        }
        let last = match current {
            Some(slot) => {
                let Some(RetainedPayload::StyleOrder(list)) = self.payloads.values.get(slot) else {
                    return Err(latch_registry_row(c));
                };
                // A rebuild during this subtree's insertion already lists the
                // style.
                let Some(last) = list.as_slice().last().copied() else {
                    return Err(latch_registry_row(c));
                };
                if last == style {
                    return Ok(());
                }
                Some(last)
            }
            None => None,
        };
        // Style elements hold text, so only the style itself can follow it.
        let record = self
            .construction
            .node_at(style)
            .ok_or_else(|| latch_registry_row(c))?;
        let mut elements = false;
        for position in 0..record.child_count() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            elements |= record
                .child_at(position)
                .is_some_and(|child| matches!(child.kind(), ConstructionNodeKind::Element(_)));
        }
        if elements {
            self.drop_style_order(tree);
            self.note_unextended_subtree(root, style);
            return Ok(());
        }
        // The order lists every style of the tree but those in the subtree being
        // applied, whose own hooks have not run. A style that is the whole
        // subtree has none after it that is unlisted, so it belongs at the end
        // of the order when it follows the last listed style, wherever in the
        // tree that is. Within a larger subtree it must end the tree.
        let appends = if root == style {
            match last {
                None => true,
                Some(last) => self.precedes(c, last, style)?,
            }
        } else {
            self.ends_subtree(c, tree, style)?
        };
        if !appends {
            self.drop_style_order(tree);
            self.note_unextended_subtree(root, style);
            return Ok(());
        }
        match current {
            Some(slot) => {
                let Some(RetainedPayload::StyleOrder(list)) = self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(c));
                };
                list.try_push_geometric(c, style)?;
            }
            None => {
                let mut list = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
                if let Err(error) = list.try_push_geometric(c, style) {
                    list.release(c)?;
                    return Err(error.into());
                }
                if let Some(order) = self.retain_style_order(c, list)? {
                    self.payloads
                        .join_latest(tree, Join::StyleOrder, order.slot)?;
                }
                self.main_tree_styleless = false;
            }
        }
        Ok(())
    }
    /// Notes that the order could not take `style`, one style of the subtree
    /// at `root` whose insertion is being applied, so that the row the next
    /// trigger builds is known to list the subtree.
    fn note_unextended_subtree(&mut self, root: usize, style: usize) {
        if root != style
            && self
                .style_subtree
                .is_none_or(|subtree| subtree.root != root)
        {
            self.style_subtree = Some(StyleSubtree {
                root,
                from: self.payloads.values.len(),
            });
        }
    }
    /// Forgets the order of the tree at `tree`, for the next trigger to
    /// rebuild by walking the tree.
    fn drop_style_order(&mut self, tree: usize) {
        self.payloads.clear_join(tree, Join::StyleOrder);
        if tree == MAIN_TREE {
            self.main_tree_styleless = false;
        }
    }
    fn association_captured_context(
        &mut self,
        c: &mut ParseControl,
        document: &crate::HtmlCaptureDocumentId,
    ) -> Result<Option<usize>, SessionError> {
        for (index, row) in self.payloads.values.iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::AssociationCapturedContext(value) = row
                && value.value.document() == document
            {
                return Ok(Some(index));
            }
        }
        let mut selected = None;
        for row in &self.payloads.values {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::AssociatedInputs { inputs, .. } = row {
                for value in &inputs.captured {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if value.document() == document {
                        selected = Some(value);
                        break;
                    }
                }
            }
        }
        let Some(value) = selected else {
            return Ok(None);
        };
        let retained = value.retained_storage_usage();
        let mut usage = associated::AssociatedStorageUsage::default();
        let bytes = retained
            .retained_bytes()
            .checked_sub(mem::size_of::<
                surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
            >())
            .ok_or(SessionError::WrongToken)?;
        usage.add(bytes, retained.utf16_extent(), 0, c)?;
        usage.acquire(c)?;
        let value = value.clone();
        let prepared = match self.prepare_payload_slot_owned(c) {
            Ok(prepared) => prepared,
            Err(error) => {
                drop(value);
                usage.release(c, &self.control)?;
                return Err(error);
            }
        };
        let index = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::AssociationCapturedContext(CapturedContext { value, usage }),
            )
            .map_err(|_| latch_registry_row(c))?;
        Ok(Some(index))
    }
    fn association_context(
        &mut self,
        c: &mut ParseControl,
        document: &DocumentRef,
        detached: Option<usize>,
        input_document: usize,
    ) -> Result<Context, SessionError> {
        let mut context = Context {
            input_document,
            mode: HtmlDocumentMode::NoQuirks,
            class: crate::HtmlDocumentClass::Unavailable,
            body: false,
            base_document: None,
            first_base: None,
            base_input: None,
            base_observation: None,
            captured: None,
            policy_missing: false,
            csp: None,
            missing: false,
        };
        match document {
            DocumentRef::Captured(document) => {
                let capture = self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context
                    .capture();
                let node = capture
                    .node(document.subject())
                    .map_err(|_| SessionError::ContextUnavailable)?;
                let crate::HtmlCapturedNodeKind::Document(facts) = node.kind() else {
                    return Err(SessionError::WrongToken);
                };
                context.mode = facts.mode();
                context.class = facts.document_class();
                context.captured = self.association_captured_context(c, document)?;
            }
            DocumentRef::Owned(document) => {
                let ConstructionNodeKind::Document(mode) = self
                    .construction
                    .node_at(*document)
                    .ok_or(SessionError::WrongToken)?
                    .kind()
                else {
                    return Err(SessionError::WrongToken);
                };
                context.mode = mode;
                let slot = self.document_slot(c, *document)?;
                let Some(RetainedPayload::Document { class, .. }) = self.payloads.values.get(slot)
                else {
                    return Err(latch_registry_row(c));
                };
                context.class = *class;
                // First document Element and only its direct body/frameset child.
                let mut child = self
                    .construction
                    .node_at(*document)
                    .ok_or(SessionError::WrongToken)?
                    .child_at(0)
                    .map(|n| n.position());
                while let Some(index) = child {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let node = self
                        .construction
                        .node_at(index)
                        .ok_or(SessionError::WrongToken)?;
                    child = self.next_sibling(c, index)?;
                    if detached == Some(index) {
                        continue;
                    }
                    if let ConstructionNodeKind::Element(element) = node.kind() {
                        if surgeist_html::elements::style::document_body_role_with(
                            element.name(),
                            |_| c.charge(HtmlParseWorkCategory::RecordVisit, 1),
                        )? == surgeist_html::elements::style::HtmlDocumentBodyRole::Html
                        {
                            let mut body = node.child_at(0).map(|n| n.position());
                            while let Some(index) = body {
                                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                                let node = self
                                    .construction
                                    .node_at(index)
                                    .ok_or(SessionError::WrongToken)?;
                                body = self.next_sibling(c, index)?;
                                if detached == Some(index) {
                                    continue;
                                }
                                if let ConstructionNodeKind::Element(element) = node.kind()
                                    && surgeist_html::elements::style::document_body_role_with(
                                        element.name(),
                                        |_| c.charge(HtmlParseWorkCategory::RecordVisit, 1),
                                    )? == surgeist_html::elements::style::HtmlDocumentBodyRole::Body
                                {
                                    context.body = true;
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
                context.policy_missing = self.meta_policy_missing(c, *document)?.is_some();
                if let Some(slot) = self.payloads.joined(c, *document, Join::BaseDocument)? {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let Some(RetainedPayload::BaseDocument { node, first, .. }) =
                        self.payloads.values.get(slot)
                        && node == document
                    {
                        context.base_document = Some(slot);
                        context.first_base = *first;
                    }
                }
                if let Some(first) = context.first_base {
                    context.base_input = self.association_attribute(c, first, "href")?;
                    let slot = self.base_slot(c, first)?;
                    let Some(RetainedPayload::Base { frozen, .. }) = self.payloads.values.get(slot)
                    else {
                        return Err(latch_registry_row(c));
                    };
                    if let base::BaseValue::Frozen(value) = frozen {
                        let Some(facts_slot) = context.base_document else {
                            return Err(SessionError::WrongToken);
                        };
                        let Some(RetainedPayload::BaseDocument { facts, .. }) =
                            self.payloads.values.get(facts_slot)
                        else {
                            return Err(latch_registry_row(c));
                        };
                        let raw = self
                            .form_attribute(c, first, "href")?
                            .ok_or(SessionError::WrongToken)?;
                        let fallback = facts
                            .document()
                            .fallback()
                            .map_err(|_| SessionError::WrongToken)?;
                        let encoding = facts
                            .document()
                            .encoding()
                            .ok_or(SessionError::WrongToken)?;
                        for (index, observation) in facts.observations().iter().enumerate() {
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            if observation.inputs_match(
                                fallback,
                                encoding,
                                facts.document().revision(),
                            ) && exact_string_equal(c, observation.raw(), raw)?
                                && facts.same_owner(&value.environment)
                            {
                                context.base_observation = Some(index);
                                break;
                            }
                        }
                        context.missing = context.base_observation.is_none();
                    } else {
                        context.missing = true;
                    }
                }
            }
        }
        if let DocumentRef::Owned(document) = document {
            self.ensure_csp_policy(c, *document)?;
            context.csp = self.csp_policy_slot(c, *document)?;
        }
        Ok(context)
    }
    pub(super) fn record_association(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        kind: HtmlParseAssociationOccurrenceKind,
        detached: Option<usize>,
        previous_document: Option<DocumentRef>,
        preferred: bool,
    ) -> Result<(), SessionError> {
        // Generic native-only sessions cannot issue original-controller evidence.
        if self.evidence.is_none() {
            return Ok(());
        }
        let document = self.node_document_ref(c, node)?;
        let document_slot = self.association_document(c, &document)?;
        let context = self.association_context(c, &document, detached, document_slot)?;
        let preferred = if preferred {
            self.association_attribute(c, node, "content")?
        } else {
            None
        };
        let base_changed_input = if kind == HtmlParseAssociationOccurrenceKind::BaseChanged {
            self.association_attribute(c, node, "href")?
        } else {
            None
        };
        let inputs = self.association_style_inputs(c, node, detached)?;
        let result = (|| {
            let root = self.ordinary_root(c, node, detached)?;
            let prior_style = self.payloads.joined(c, node, Join::StyleOccurrence)?;
            let cause = self.associated_cause(c)?;
            let prior_sheet = prior_style.is_some_and(|slot| matches!(self.payloads.values.get(slot), Some(RetainedPayload::AssociationOccurrence(values)) if values.as_slice().first().is_some_and(|first| first.may_have_sheet)));
            let Some(RetainedPayload::AssociationDocument(summary)) =
                self.payloads.values.get(document_slot)
            else {
                return Err(latch_registry_row(c));
            };
            let count = summary
                .count
                .checked_add(1)
                .ok_or(SessionError::WrongToken)?;
            let mut obligation = matches!(
                kind,
                HtmlParseAssociationOccurrenceKind::ChangePreferred
                    | HtmlParseAssociationOccurrenceKind::UnsupportedLink
                    | HtmlParseAssociationOccurrenceKind::UnsupportedLinkExtension
            );
            let mut may_have_sheet = prior_sheet;
            if let Some(inputs) = &inputs {
                // Removal is conditional on the prior real association, never parser None.
                obligation |= match kind {
                    HtmlParseAssociationOccurrenceKind::StyleUpdate(_) => {
                        may_have_sheet = inputs.connected && inputs.type_eligible;
                        prior_sheet || may_have_sheet
                    }
                    HtmlParseAssociationOccurrenceKind::StyleOwnerAttributes => prior_sheet,
                    _ => false,
                };
            }
            let obligations = summary
                .obligations
                .checked_add(usize::from(obligation))
                .ok_or(SessionError::WrongToken)?;
            let prior_document = summary.last;
            let creating = matches!(kind, HtmlParseAssociationOccurrenceKind::StyleUpdate(_))
                && may_have_sheet;
            let missing = if creating && context.policy_missing {
                Some(HtmlParseAssociationMissingKind::PolicyHistory)
            } else if creating && context.missing {
                Some(HtmlParseAssociationMissingKind::OriginalContext)
            } else if kind == HtmlParseAssociationOccurrenceKind::UnsupportedLink {
                Some(HtmlParseAssociationMissingKind::UnsupportedLinkStylesheetHistory)
            } else if kind == HtmlParseAssociationOccurrenceKind::UnsupportedLinkExtension {
                Some(HtmlParseAssociationMissingKind::ExtensionLinkHistory)
            } else {
                None
            };
            payload_write::<DocumentCapture>(c, 1)?;
            Ok::<_, SessionError>((
                root,
                cause,
                prior_style,
                prior_document,
                count,
                obligations,
                missing,
                obligation,
                may_have_sheet,
            ))
        })();
        match result {
            Ok((
                root,
                cause,
                prior_style,
                prior_document,
                count,
                obligations,
                missing,
                requires_processing,
                may_have_sheet,
            )) => {
                let styled = inputs.is_some();
                let value = Occurrence {
                    subject: node,
                    document,
                    root,
                    previous_document,
                    prior_document,
                    prior_style,
                    cause,
                    kind,
                    inputs,
                    preferred,
                    base_changed_input,
                    context,
                    requires_processing,
                    may_have_sheet,
                };
                let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
                if let Err((stop, value)) = values.try_push_recover(c, value) {
                    // The stop returned is the first; a refusal of the release
                    // is latched behind it.
                    let _ = release_inputs(value, c);
                    values.release(c)?;
                    return Err(SessionError::Stop(stop));
                }
                let prepared = match self.prepare_payload_slot_owned(c) {
                    Ok(prepared) => prepared,
                    Err(error) => {
                        // The error returned is the first; a refusal of the
                        // release is latched behind it.
                        let _ = release_occurrence(values, c);
                        return Err(error);
                    }
                };
                let slot = self
                    .payloads
                    .push_row(prepared, RetainedPayload::AssociationOccurrence(values))
                    .map_err(|_| latch_registry_row(c))?;
                if styled {
                    self.payloads
                        .join_latest(node, Join::StyleOccurrence, slot)
                        .map_err(|_| latch_registry_row(c))?;
                }
                let Some(RetainedPayload::AssociationDocument(summary)) =
                    self.payloads.values.get_mut(document_slot)
                else {
                    return Err(latch_registry_row(c));
                };
                summary.count = count;
                summary.obligations = obligations;
                summary.last = Some(slot);
                if summary.missing.is_none() {
                    summary.missing = missing;
                    summary.missing_at = missing.map(|_| slot);
                }
                Ok(())
            }
            Err(error) => {
                if let Some(inputs) = inputs {
                    inputs.text.release(c)?;
                }
                Err(error)
            }
        }
    }
    /// Test harness: reads the association nonce of `node`.
    #[cfg(test)]
    pub(super) fn association_nonce_present(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<bool, SessionError> {
        Ok(self.association_nonce(c, node)?.is_some())
    }
    pub(super) fn association_style_trigger(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        trigger: surgeist_html::elements::style::HtmlStyleTrigger,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "style")?
            && surgeist_html::elements::style::style_update_is_required(trigger)
        {
            self.record_association(
                c,
                node,
                HtmlParseAssociationOccurrenceKind::StyleUpdate(trigger),
                detached,
                None,
                false,
            )?;
        }
        Ok(())
    }
    pub(super) fn association_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
        old_parent: Option<usize>,
        root: usize,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "style")? {
            self.style_order_changed(c, node, detached, removal, root)?;
            let Some(slot) = self.style_slot(c, node)? else {
                return Ok(());
            };
            let Some(RetainedPayload::Style { state, .. }) = self.payloads.values.get(slot) else {
                return Err(latch_registry_row(c));
            };
            let on_parser_stack = state.state.on_parser_stack();
            let connected = self.shadow_including_connected(c, node, detached)?;
            let old_connected = match old_parent {
                Some(parent) => self.shadow_including_connected(c, parent, None)?,
                None => false,
            };
            let trigger = if removal {
                surgeist_html::elements::style::HtmlStyleTrigger::Removal {
                    disconnected: old_connected && !connected,
                    on_parser_stack,
                }
            } else {
                surgeist_html::elements::style::HtmlStyleTrigger::Insertion {
                    connected,
                    on_parser_stack,
                }
            };
            self.association_style_trigger(c, node, trigger, detached)?;
        }
        if !removal {
            self.association_link(c, node)?;
        }
        Ok(())
    }
    pub(super) fn association_children(
        &mut self,
        c: &mut ParseControl,
        parent: usize,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.style_slot(c, parent)? else {
            return Ok(());
        };
        let Some(RetainedPayload::Style { state, .. }) = self.payloads.values.get(slot) else {
            return Err(latch_registry_row(c));
        };
        self.association_style_trigger(
            c,
            parent,
            surgeist_html::elements::style::HtmlStyleTrigger::ChildrenChanged {
                on_parser_stack: state.state.on_parser_stack(),
            },
            detached,
        )
    }
    pub(super) fn association_attribute_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() {
            return Ok(());
        }
        let ConstructionNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::WrongToken)?
            .kind()
        else {
            return Ok(());
        };
        let attr = element
            .attributes()
            .at(c, position)?
            .ok_or(SessionError::WrongToken)?;
        let effect =
            surgeist_html::vocabulary::KnownHtmlAttribute::from_name(attr.name()).map(|name| {
                surgeist_html::elements::style::style_attribute_effect(
                    attr.name().namespace(),
                    name,
                )
            });
        if effect == Some(surgeist_html::elements::style::HtmlStyleAttributeEffect::OwnerAttributes)
            && self.html_local(c, node, "style")?
        {
            self.record_association(
                c,
                node,
                HtmlParseAssociationOccurrenceKind::StyleOwnerAttributes,
                None,
                None,
                false,
            )?;
        }
        self.association_link(c, node)
    }
    pub(super) fn association_initial_attributes(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() || !self.html_local(c, node, "style")? {
            return Ok(());
        }
        let ConstructionNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::WrongToken)?
            .kind()
        else {
            return Ok(());
        };
        let count = element.attributes().count(c)?;
        for position in 0..count {
            self.association_attribute_hook(c, node, position)?;
        }
        Ok(())
    }
    pub(super) fn prepare_association_documents(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() {
            return Ok(());
        }
        let extent = self.payloads.values.len();
        #[cfg(test)]
        let extent = extent.saturating_add(c.take_registry_walk_overshoot());
        for index in 0..extent {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let document = match self.payloads.row(c, index)? {
                RetainedPayload::Document {
                    node: Some(node), ..
                } => Some(DocumentRef::Owned(*node)),
                RetainedPayload::NodeDocument {
                    document: DocumentRef::Captured(id),
                    ..
                } => {
                    c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    Some(DocumentRef::Captured(id.clone()))
                }
                _ => None,
            };
            if let Some(document) = document {
                self.association_document(c, &document)?;
            }
        }
        Ok(())
    }
    fn association_link(&mut self, c: &mut ParseControl, node: usize) -> Result<(), SessionError> {
        if self.evidence.is_none()
            || !self.html_local(c, node, "link")?
            || !self.shadow_including_connected(c, node, None)?
            || self.form_attribute(c, node, "href")?.is_none()
        {
            return Ok(());
        }
        let Some(rel) = self.form_attribute(c, node, "rel")? else {
            return Ok(());
        };
        let mut cursor = surgeist_html::bounded::lexical::HtmlSpaceTokenCursor::new(rel);
        let mut stylesheet = false;
        let mut extension = false;
        loop {
            c.charge(
                HtmlParseWorkCategory::RecordVisit,
                surgeist_html::bounded::lexical::HtmlSpaceTokenCursor::STEP_WORK as u64,
            )?;
            match cursor.step() {
                surgeist_html::bounded::lexical::HtmlSpaceTokenStep::Continue => {}
                surgeist_html::bounded::lexical::HtmlSpaceTokenStep::Done => break,
                surgeist_html::bounded::lexical::HtmlSpaceTokenStep::Token { start, end } => {
                    #[cfg(test)]
                    let end = end.saturating_add(c.take_space_token_overshoot());
                    struct Meter<'a>(&'a mut ParseControl);
                    impl surgeist_html::bounded::rel::HtmlRelMeter for Meter<'_> {
                        type Error = SessionError;
                        fn work(&mut self, units: usize) -> Result<(), Self::Error> {
                            self.0
                                .charge(HtmlParseWorkCategory::RecordVisit, units as u64)
                                .map_err(Into::into)
                        }
                        fn reserve(
                            &mut self,
                            _: &mut surgeist_html::bounded::rel::HtmlRelReservation<'_>,
                        ) -> Result<(), Self::Error> {
                            Err(SessionError::WrongToken)
                        }
                    }
                    let Some(token) = rel.get(start..end) else {
                        return Err(latch_registry_row(c));
                    };
                    let (keyword, _) = surgeist_html::bounded::rel::classify_rel(
                        token,
                        &mut Meter(c),
                    )
                    .map_err(|error| match error {
                        surgeist_html::bounded::rel::HtmlRelActionError::Admission(error) => error,
                        surgeist_html::bounded::rel::HtmlRelActionError::Protocol(_) => {
                            SessionError::WrongToken
                        }
                    })?;
                    stylesheet |=
                        keyword == Some(surgeist_html::elements::link::HtmlRelKeyword::Stylesheet);
                    extension |= keyword.is_none();
                }
            }
        }
        if stylesheet {
            self.record_association(
                c,
                node,
                HtmlParseAssociationOccurrenceKind::UnsupportedLink,
                None,
                None,
                false,
            )?;
        } else if extension {
            self.record_association(
                c,
                node,
                HtmlParseAssociationOccurrenceKind::UnsupportedLinkExtension,
                None,
                None,
                false,
            )?;
        }
        Ok(())
    }
}

type Fault = read::HtmlParseUnresolvedReference;
fn unresolved(kind: read::HtmlParseUnresolvedKind) -> Fault {
    read::HtmlParseUnresolvedReference::new(kind)
}
/// The report storage the readers answer from, with the one node that every
/// identity names. Resolved when a cursor is created, so no reader looks it up.
#[derive(Clone, Copy)]
struct Anchor<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    first: HtmlNode<'a>,
}
/// A Document reference resolved against the report that holds it.
#[derive(Clone, Copy)]
enum Resolved<'a> {
    Owned(HtmlNode<'a>),
    Captured(crate::HtmlCapturedNode<'a>),
}
impl<'a> Resolved<'a> {
    /// The fault if the report no longer holds the Document.
    fn new(storage: &'a FrozenStorage, value: &DocumentRef) -> Result<Self, Fault> {
        match value {
            DocumentRef::Owned(node) => native(storage, *node).map(Self::Owned),
            DocumentRef::Captured(id) => storage
                .fragment
                .as_ref()
                .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fragment))?
                .context
                .capture()
                .node(id.subject())
                .map(Self::Captured)
                .map_err(|_| unresolved(read::HtmlParseUnresolvedKind::CapturedNode)),
        }
    }
    fn optional(
        storage: &'a FrozenStorage,
        value: Option<&DocumentRef>,
    ) -> Result<Option<Self>, Fault> {
        value.map(|value| Self::new(storage, value)).transpose()
    }
    fn lend(self) -> report::HtmlParseMetadataDocument<'a> {
        match self {
            Self::Owned(node) => report::HtmlParseMetadataDocument::Owned(node),
            Self::Captured(node) => report::HtmlParseMetadataDocument::Captured(node),
        }
    }
}
/// One actual payload slot per step; no hidden next-match traversal.
///
/// Publication qualifies every row and what it names (`qualify`), and a
/// report is immutable afterwards, so every row it holds can be lent. A row
/// whose references the report cannot resolve is a defect in that
/// qualification: the cursor reports it as a fault, and keeps reporting it,
/// instead of skipping the row, because a consumer that skipped it would act
/// on a report that lacks an association. `HtmlParsePendingAssociationStep::Skipped` means only a row
/// that is not an association.
pub struct HtmlParsePendingAssociationCursor<'a> {
    anchor: Result<Anchor<'a>, Fault>,
    next: usize,
    fault: Option<Fault>,
}
#[expect(
    clippy::large_enum_variant,
    reason = "The occurrence holds what it resolved, and is lent by value so the read path allocates nothing; boxing it would add an allocation where no controller exists to charge it"
)]
pub enum HtmlParsePendingAssociationStep<'a> {
    Document(HtmlParseAssociationDocument<'a>),
    Occurrence(HtmlParseAssociationOccurrence<'a>),
    /// A row that is not an association.
    Skipped,
    Complete,
    /// The row is an association that names a reference the report cannot
    /// resolve. The cursor stays faulted, so a caller that continues is told
    /// again.
    Fault(read::HtmlParseUnresolvedReference),
}
impl<'a> HtmlParsePendingAssociationCursor<'a> {
    pub(crate) fn new(history: &'a FrozenHistory) -> Self {
        let storage = history.storage();
        let anchor = native(storage, 0).map(|first| Anchor {
            history,
            storage,
            first,
        });
        Self {
            anchor,
            next: 0,
            fault: None,
        }
    }
    fn fail(&mut self, fault: Fault) -> HtmlParsePendingAssociationStep<'a> {
        self.fault = Some(fault);
        HtmlParsePendingAssociationStep::Fault(fault)
    }
    pub fn advance(&mut self) -> HtmlParsePendingAssociationStep<'a> {
        if let Some(fault) = self.fault {
            return HtmlParsePendingAssociationStep::Fault(fault);
        }
        let anchor = match self.anchor {
            Ok(anchor) => anchor,
            Err(fault) => return self.fail(fault),
        };
        let slot = self.next;
        let Some(row) = anchor.storage.payloads.values.get(slot) else {
            return HtmlParsePendingAssociationStep::Complete;
        };
        // `slot` indexes a vector, so it is below the address space.
        self.next = slot.saturating_add(1);
        let view = match row {
            RetainedPayload::AssociationDocument(value) => {
                HtmlParseAssociationDocument::new(anchor, value)
                    .map(HtmlParsePendingAssociationStep::Document)
            }
            RetainedPayload::AssociationOccurrence(values) => values
                .as_slice()
                .first()
                .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fact))
                .and_then(|value| HtmlParseAssociationOccurrence::new(anchor, slot, value))
                .map(HtmlParsePendingAssociationStep::Occurrence),
            _ => return HtmlParsePendingAssociationStep::Skipped,
        };
        view.unwrap_or_else(|fault| self.fail(fault))
    }
}
fn native(storage: &FrozenStorage, index: usize) -> Result<HtmlNode<'_>, Fault> {
    storage
        .view
        .node_at(index)
        .and_then(|node| node.as_node().ok())
        .ok_or(unresolved(read::HtmlParseUnresolvedKind::Node))
}
fn identity(anchor: Anchor<'_>, slot: usize) -> HtmlParseAssociationOccurrenceId {
    HtmlParseAssociationOccurrenceId {
        result: anchor.first.id(),
        slot,
    }
}
fn context_epoch(anchor: Anchor<'_>, slot: usize) -> HtmlParseAssociationContextEpochId {
    HtmlParseAssociationContextEpochId {
        result: anchor.first.id(),
        slot,
    }
}
/// The value a stored row holds, or the fault.
fn value<'a>(storage: &'a FrozenStorage, row: &ValueRow) -> Result<&'a DocumentString, Fault> {
    match storage.payloads.values.get(row.0) {
        Some(RetainedPayload::String { value, .. }) => Ok(value),
        _ => Err(unresolved(read::HtmlParseUnresolvedKind::Fact)),
    }
}
fn optional_value<'a>(
    storage: &'a FrozenStorage,
    row: Option<&ValueRow>,
) -> Result<Option<&'a DocumentString>, Fault> {
    row.map(|row| value(storage, row)).transpose()
}
pub struct HtmlParseAssociationDocument<'a> {
    anchor: Anchor<'a>,
    value: &'a DocumentCapture,
    document: Resolved<'a>,
}
impl<'a> HtmlParseAssociationDocument<'a> {
    fn new(anchor: Anchor<'a>, value: &'a DocumentCapture) -> Result<Self, Fault> {
        Ok(Self {
            anchor,
            value,
            document: Resolved::new(anchor.storage, &value.document)?,
        })
    }
    pub fn document(&self) -> report::HtmlParseMetadataDocument<'a> {
        self.document.lend()
    }
    pub fn disposition(&self) -> HtmlParseAssociationCaptureDisposition {
        self.value.missing.map_or(
            HtmlParseAssociationCaptureDisposition::PendingComplete,
            HtmlParseAssociationCaptureDisposition::Unavailable,
        )
    }
    pub fn occurrence_count(&self) -> usize {
        self.value.count
    }
    pub fn has_obligations(&self) -> bool {
        self.value.obligations != 0
    }
    pub fn last_occurrence(&self) -> Option<HtmlParseAssociationOccurrenceId> {
        self.value.last.map(|slot| identity(self.anchor, slot))
    }
    pub fn first_missing(&self) -> Option<HtmlParseAssociationOccurrenceId> {
        self.value
            .missing_at
            .map(|slot| identity(self.anchor, slot))
    }
}
/// An occurrence with everything it names resolved when it is lent, so the
/// readers of the view cannot fail afterwards.
#[derive(Clone, Copy)]
pub struct HtmlParseAssociationOccurrence<'a> {
    anchor: Anchor<'a>,
    slot: usize,
    value: &'a Occurrence,
    subject: HtmlNode<'a>,
    root: HtmlNode<'a>,
    document: Resolved<'a>,
    previous_document: Option<Resolved<'a>>,
    cause: Option<read::HtmlParseCause<'a>>,
    preferred: Option<&'a DocumentString>,
    base_changed_input: Option<&'a DocumentString>,
    style: Option<HtmlParseAssociationStyleInputs<'a>>,
    context: HtmlParseAssociationContext<'a>,
}
impl<'a> HtmlParseAssociationOccurrence<'a> {
    /// The fault if the report no longer holds a node, Document, cause, group
    /// or row the occurrence names.
    fn new(anchor: Anchor<'a>, slot: usize, value: &'a Occurrence) -> Result<Self, Fault> {
        let storage = anchor.storage;
        let history = anchor.history;
        let parser_document = Resolved::optional(
            storage,
            value
                .inputs
                .as_ref()
                .and_then(|inputs| inputs.parser_document.as_ref()),
        )?;
        let style = value
            .inputs
            .as_ref()
            .map(|inputs| HtmlParseAssociationStyleInputs::new(anchor, inputs, parser_document))
            .transpose()?;
        let cause = value
            .cause
            .as_ref()
            .map(|cause| {
                history
                    .evidence()
                    .retained_cause(cause)
                    .map(|retained| read::HtmlParseCause::qualified(history, retained))
                    .ok_or(unresolved(read::HtmlParseUnresolvedKind::Cause))
            })
            .transpose()?;
        Ok(Self {
            anchor,
            slot,
            value,
            subject: native(storage, value.subject)?,
            root: native(storage, value.root)?,
            document: Resolved::new(storage, &value.document)?,
            previous_document: Resolved::optional(storage, value.previous_document.as_ref())?,
            cause,
            preferred: optional_value(storage, value.preferred.as_ref())?,
            base_changed_input: optional_value(storage, value.base_changed_input.as_ref())?,
            style,
            context: HtmlParseAssociationContext::new(
                anchor,
                slot,
                &value.document,
                &value.context,
            )?,
        })
    }
    pub fn id(&self) -> HtmlParseAssociationOccurrenceId {
        identity(self.anchor, self.slot)
    }
    pub fn kind(&self) -> HtmlParseAssociationOccurrenceKind {
        self.value.kind
    }
    pub fn requires_processing(&self) -> bool {
        self.value.requires_processing
    }
    pub fn subject(&self) -> HtmlNode<'a> {
        self.subject
    }
    pub fn document(&self) -> report::HtmlParseMetadataDocument<'a> {
        self.document.lend()
    }
    pub fn root(&self) -> HtmlNode<'a> {
        self.root
    }
    pub fn previous_document(&self) -> Option<report::HtmlParseMetadataDocument<'a>> {
        matches!(
            self.value.kind,
            HtmlParseAssociationOccurrenceKind::StyleDocumentChanged
        )
        .then(|| self.document.lend())
    }
    /// Adoption is captured before replacing the old Document relation.
    pub fn next_document(&self) -> Option<report::HtmlParseMetadataDocument<'a>> {
        self.previous_document.map(Resolved::lend)
    }
    /// Prior original invocation in this Document. This records causal source
    /// order, not successful sheet addition or asynchronous resource completion.
    pub fn prior_document(&self) -> Option<HtmlParseAssociationOccurrenceId> {
        self.value
            .prior_document
            .map(|slot| identity(self.anchor, slot))
    }
    /// Prior lifecycle invocation for this Style. Consumers preserve the HTML
    /// synchronous prefix and resource dependencies; this edge does not
    /// impose lexical completion order on outstanding resource operations.
    pub fn prior_style(&self) -> Option<HtmlParseAssociationOccurrenceId> {
        self.value
            .prior_style
            .map(|slot| identity(self.anchor, slot))
    }
    pub(crate) fn recorded_cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.cause
    }
    /// The name of a preferred-name change as it was when the change was
    /// recorded. `None` when there is no preferred name.
    pub fn preferred_name(&self) -> Option<&'a DocumentString> {
        self.preferred
    }
    /// The `href` of a Base change as it was when the change was recorded.
    /// `None` when there is no changed input.
    pub fn base_changed_input(&self) -> Option<&'a DocumentString> {
        self.base_changed_input
    }
    pub fn style(&self) -> Option<HtmlParseAssociationStyleInputs<'a>> {
        self.style
    }
    pub fn context(&self) -> HtmlParseAssociationContext<'a> {
        self.context
    }
}
#[derive(Clone, Copy)]
pub struct HtmlParseAssociationStyleInputs<'a> {
    anchor: Anchor<'a>,
    value: &'a Inputs,
    parser_document: Option<Resolved<'a>>,
    attributes: [Option<&'a DocumentString>; 5],
}
impl<'a> HtmlParseAssociationStyleInputs<'a> {
    fn new(
        anchor: Anchor<'a>,
        value: &'a Inputs,
        parser_document: Option<Resolved<'a>>,
    ) -> Result<Self, Fault> {
        let mut attributes = [None; 5];
        for (resolved, stored) in attributes.iter_mut().zip(&value.attributes) {
            *resolved = optional_value(anchor.storage, stored.as_ref())?;
        }
        Ok(Self {
            anchor,
            value,
            parser_document,
            attributes,
        })
    }
    pub fn state(&self) -> surgeist_html::elements::style::HtmlStyleState {
        self.value.state
    }
    pub fn creating_parser_document(&self) -> Option<report::HtmlParseMetadataDocument<'a>> {
        self.parser_document.map(Resolved::lend)
    }
    pub fn connected(&self) -> bool {
        self.value.connected
    }
    pub fn type_eligible(&self) -> bool {
        self.value.type_eligible
    }
    pub fn in_document_tree(&self) -> bool {
        self.value.document_tree
    }
    pub fn text_len(&self) -> usize {
        self.value.text.len()
    }
    /// The text of the style at `index`, as it was when the occurrence was
    /// recorded. `None` past the text. A value the report no longer holds is a
    /// fault.
    pub fn text(&self, index: usize) -> Result<Option<&'a DocumentString>, Fault> {
        optional_value(self.anchor.storage, self.value.text.as_slice().get(index))
    }
    pub fn sheet_order_len(&self) -> usize {
        self.value.order.map_or(0, |order| order.len)
    }
    /// `None` past the order. An entry the report no longer holds is a fault.
    pub fn sheet_order(&self, index: usize) -> Result<Option<HtmlNode<'a>>, Fault> {
        let Some(order) = self.value.order else {
            return Ok(None);
        };
        if index >= order.len {
            return Ok(None);
        }
        let Some(RetainedPayload::StyleOrder(list)) =
            self.anchor.storage.payloads.values.get(order.slot)
        else {
            return Err(unresolved(read::HtmlParseUnresolvedKind::Fact));
        };
        let entry = list
            .as_slice()
            .get(index)
            .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fact))?;
        native(self.anchor.storage, *entry).map(Some)
    }
    pub fn attribute(
        &self,
        attribute: surgeist_html::vocabulary::KnownHtmlAttribute,
    ) -> Option<&'a DocumentString> {
        let [kind, media, title, blocking, nonce] = self.attributes;
        match attribute {
            surgeist_html::vocabulary::KnownHtmlAttribute::Type => kind,
            surgeist_html::vocabulary::KnownHtmlAttribute::Media => media,
            surgeist_html::vocabulary::KnownHtmlAttribute::Title => title,
            surgeist_html::vocabulary::KnownHtmlAttribute::Blocking => blocking,
            surgeist_html::vocabulary::KnownHtmlAttribute::Nonce => nonce,
            _ => None,
        }
    }
}
/// The context of an occurrence, with the policy, the base facts, the captured
/// context, the first Base and the input it names resolved when it is lent.
#[derive(Clone, Copy)]
pub struct HtmlParseAssociationContext<'a> {
    anchor: Anchor<'a>,
    slot: usize,
    document: &'a DocumentRef,
    value: &'a Context,
    csp: Option<&'a surgeist_html::security::csp::HtmlCspAssociation>,
    base_facts: Option<&'a crate::HtmlParseBaseFacts>,
    captured_context: Option<&'a surgeist_html_semantics::HtmlDocumentEncodingUrlContext>,
    first_base: Option<HtmlNode<'a>>,
    base_input: Option<&'a DocumentString>,
    observation: Option<&'a surgeist_html_semantics::HtmlBaseParseObservation>,
    /// For a captured Document: its identity and the context that captured it.
    captured: Option<(
        &'a crate::HtmlCaptureDocumentId,
        &'a crate::HtmlFragmentContext,
    )>,
}
impl<'a> HtmlParseAssociationContext<'a> {
    fn new(
        anchor: Anchor<'a>,
        slot: usize,
        document: &'a DocumentRef,
        value: &'a Context,
    ) -> Result<Self, Fault> {
        let storage = anchor.storage;
        let row = |slot: usize| {
            storage
                .payloads
                .values
                .get(slot)
                .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fact))
        };
        // A policy that is missing from the history has no association.
        let csp = match value.csp {
            Some(slot) if !value.policy_missing => match row(slot)? {
                RetainedPayload::CspPolicy(policy) => policy.value.as_ref(),
                _ => return Err(unresolved(read::HtmlParseUnresolvedKind::Fact)),
            },
            _ => None,
        };
        let base_facts = match value.base_document {
            Some(slot) => match row(slot)? {
                RetainedPayload::BaseDocument { facts, .. } => Some(facts),
                _ => return Err(unresolved(read::HtmlParseUnresolvedKind::Fact)),
            },
            None => None,
        };
        let captured_context = match value.captured {
            Some(slot) => match row(slot)? {
                RetainedPayload::AssociationCapturedContext(captured) => Some(&captured.value),
                _ => return Err(unresolved(read::HtmlParseUnresolvedKind::Fact)),
            },
            None => None,
        };
        let first_base = value
            .first_base
            .map(|index| native(storage, index))
            .transpose()?;
        // The observation is consulted only to select a frozen Base of the
        // facts, for a first Base that names one.
        let observation = match (base_facts, first_base, value.base_observation) {
            (Some(facts), Some(_), Some(index)) => Some(
                facts
                    .observations()
                    .get(index)
                    .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fact))?,
            ),
            _ => None,
        };
        Ok(Self {
            anchor,
            slot,
            document,
            value,
            csp,
            base_facts,
            captured_context,
            first_base,
            base_input: optional_value(storage, value.base_input.as_ref())?,
            observation,
            captured: match document {
                DocumentRef::Captured(id) => Some((
                    id,
                    &storage
                        .fragment
                        .as_ref()
                        .ok_or(unresolved(read::HtmlParseUnresolvedKind::Fragment))?
                        .context,
                )),
                DocumentRef::Owned(_) => None,
            },
        })
    }
    /// The exact association applicable at this occurrence, never final policy.
    /// `None` means the occurrence has no association.
    pub fn csp_association(&self) -> Option<&'a surgeist_html::security::csp::HtmlCspAssociation> {
        self.csp
    }
    pub fn epoch(&self) -> HtmlParseAssociationContextEpochId {
        context_epoch(self.anchor, self.slot)
    }
    /// Immutable initial Document input lineage for URL, fallback, encoding and
    /// initial policy; other roles retain their original occurrence frontier.
    /// This never identifies a derived selected Base result: original Base
    /// inputs, selection epochs and known providers remain occurrence-specific.
    pub fn input_epoch(
        &self,
        role: HtmlParseAssociationContextRole,
    ) -> HtmlParseAssociationContextEpochId {
        if matches!(
            role,
            HtmlParseAssociationContextRole::DocumentUrl
                | HtmlParseAssociationContextRole::FallbackBase
                | HtmlParseAssociationContextRole::Encoding
                | HtmlParseAssociationContextRole::InitialPolicy
        ) {
            context_epoch(self.anchor, self.value.input_document)
        } else {
            self.epoch()
        }
    }
    pub fn document_mode(&self) -> HtmlDocumentMode {
        self.value.mode
    }
    pub fn document_class(&self) -> crate::HtmlDocumentClass {
        self.value.class
    }
    pub fn body_present(&self) -> Option<bool> {
        matches!(self.document, DocumentRef::Owned(_)).then_some(self.value.body)
    }
    pub fn requirement(
        &self,
        role: HtmlParseAssociationContextRole,
    ) -> HtmlParseAssociationContextRequirement {
        if self.value.missing {
            return HtmlParseAssociationContextRequirement::Unavailable(
                HtmlParseAssociationMissingKind::OriginalContext,
            );
        }
        if role == HtmlParseAssociationContextRole::InitialPolicy && self.value.policy_missing {
            return HtmlParseAssociationContextRequirement::Unavailable(
                HtmlParseAssociationMissingKind::PolicyHistory,
            );
        }
        if role == HtmlParseAssociationContextRole::InitialPolicy && self.value.csp.is_some() {
            return HtmlParseAssociationContextRequirement::Known;
        }
        if matches!(self.document, DocumentRef::Captured(_)) {
            if (self.value.captured.is_some()
                && matches!(
                    role,
                    HtmlParseAssociationContextRole::FallbackBase
                        | HtmlParseAssociationContextRole::Encoding
                ))
                || (role == HtmlParseAssociationContextRole::DocumentEligibility
                    && self.value.class != crate::HtmlDocumentClass::Unavailable)
            {
                HtmlParseAssociationContextRequirement::Known
            } else {
                HtmlParseAssociationContextRequirement::DeferredCaptured
            }
        } else if let Some(facts) = self.base_facts() {
            let known = match role {
                HtmlParseAssociationContextRole::DocumentUrl => {
                    facts.document().document_url().is_some()
                }
                HtmlParseAssociationContextRole::FallbackBase => self.selected_base().is_ok(),
                HtmlParseAssociationContextRole::Encoding => facts.document().encoding().is_some(),
                HtmlParseAssociationContextRole::BodyPresence
                | HtmlParseAssociationContextRole::DocumentEligibility => true,
                _ => false,
            };
            if known {
                HtmlParseAssociationContextRequirement::Known
            } else {
                HtmlParseAssociationContextRequirement::OfflineUnbound
            }
        } else if matches!(
            role,
            HtmlParseAssociationContextRole::BodyPresence
                | HtmlParseAssociationContextRole::DocumentEligibility
        ) {
            HtmlParseAssociationContextRequirement::Known
        } else {
            HtmlParseAssociationContextRequirement::OfflineUnbound
        }
    }
    pub fn slot(
        &self,
        role: HtmlParseAssociationContextRole,
    ) -> Option<HtmlParseAssociationContextSlotId> {
        (self.requirement(role) == HtmlParseAssociationContextRequirement::OfflineUnbound).then(
            || HtmlParseAssociationContextSlotId {
                epoch: self.input_epoch(role),
                role,
            },
        )
    }
    /// Exact original capture authority for a deferred observation. This lender
    /// is a request basis, not permission to replace it with current host facts.
    pub fn captured_observation(
        &self,
        role: HtmlParseAssociationContextRole,
    ) -> Option<HtmlParseAssociationCapturedObservation<'a>> {
        if self.requirement(role) != HtmlParseAssociationContextRequirement::DeferredCaptured {
            return None;
        }
        let (document, context) = self.captured?;
        Some(HtmlParseAssociationCapturedObservation {
            epoch: self.input_epoch(role),
            role,
            document,
            context,
        })
    }
    /// `None` means the occurrence has no base facts of this document.
    pub fn base_facts(&self) -> Option<&'a crate::HtmlParseBaseFacts> {
        self.base_facts
    }
    /// `None` means the occurrence has no captured context of this document.
    pub fn captured_context(
        &self,
    ) -> Option<&'a surgeist_html_semantics::HtmlDocumentEncodingUrlContext> {
        self.captured_context
    }
    /// `None` when there is no first Base.
    pub fn first_base(&self) -> Option<HtmlNode<'a>> {
        self.first_base
    }
    /// The first Base's `href` as it was when the occurrence was recorded.
    /// `None` when there is no input.
    pub fn base_input(&self) -> Option<&'a DocumentString> {
        self.base_input
    }
    pub fn base_observation(&self) -> Option<usize> {
        self.value.base_observation
    }
    pub fn selected_base(&self) -> Result<&'a crate::HtmlBaseUrlRecord, crate::HtmlBaseMissing> {
        if let Some(context) = self.captured_context() {
            return Ok(context.base_url().value());
        }
        let facts = self
            .base_facts()
            .ok_or(crate::HtmlBaseMissing::DocumentFacts)?;
        if self.value.first_base.is_some() {
            if self.value.base_observation.is_none() {
                return Err(crate::HtmlBaseMissing::FrozenBase);
            }
            let fallback = facts.document().fallback()?;
            let observation = self.observation.ok_or(crate::HtmlBaseMissing::FrozenBase)?;
            crate::select_frozen_base(fallback, observation.outcome()).map(|(value, _)| value)
        } else {
            surgeist_html_semantics::select_document_base(
                Some(facts.document()),
                surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
            )
        }
    }
}
pub struct HtmlParseAssociationCapturedObservation<'a> {
    epoch: HtmlParseAssociationContextEpochId,
    role: HtmlParseAssociationContextRole,
    document: &'a crate::HtmlCaptureDocumentId,
    context: &'a crate::HtmlFragmentContext,
}
impl HtmlParseAssociationCapturedObservation<'_> {
    pub fn epoch(&self) -> &HtmlParseAssociationContextEpochId {
        &self.epoch
    }
    pub fn role(&self) -> HtmlParseAssociationContextRole {
        self.role
    }
    pub fn document(&self) -> &crate::HtmlCaptureDocumentId {
        self.document
    }
    pub fn context(&self) -> &crate::HtmlFragmentContext {
        self.context
    }
}

pub(super) fn qualify(storage: &FrozenStorage, c: &mut ParseControl) -> Result<(), SessionError> {
    for (slot, row) in storage.payloads.values.iter().enumerate() {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match row {
            RetainedPayload::AssociationDocument(value) => {
                storage.validate_association_document(c, &value.document)?;
                if value
                    .last
                    .is_some_and(|index| index <= slot || index >= storage.payloads.values.len())
                {
                    return Err(latch_registry_row(c));
                }
            }
            RetainedPayload::AssociationOccurrence(values) => {
                values.require_disposal_owner(c)?;
                let [value] = values.as_slice() else {
                    return Err(latch_registry_row(c));
                };
                if value.context.input_document >= slot
                    || !matches!(
                        storage.payloads.values.get(value.context.input_document),
                        Some(RetainedPayload::AssociationDocument(summary)) if summary.document == value.document
                    )
                {
                    return Err(latch_registry_row(c));
                }
                storage.validate_association_document(c, &value.document)?;
                if let Some(document) = &value.previous_document {
                    storage.validate_association_document(c, document)?;
                }
                if storage.view.node_at(value.subject).is_none()
                    || storage.view.node_at(value.root).is_none()
                {
                    return Err(latch_registry_row(c));
                }
                for prior in [value.prior_document, value.prior_style]
                    .into_iter()
                    .flatten()
                {
                    if prior >= slot
                        || !matches!(
                            storage.payloads.values.get(prior),
                            Some(RetainedPayload::AssociationOccurrence(_))
                        )
                    {
                        return Err(latch_registry_row(c));
                    }
                }
                let evidence = storage
                    .evidence
                    .as_ref()
                    .ok_or_else(|| latch_registry_row(c))?;
                if let Some(cause) = &value.cause
                    && evidence.retained_cause(cause).is_none()
                {
                    return Err(latch_registry_row(c));
                }
                for reference in [
                    value.preferred.as_ref(),
                    value.base_changed_input.as_ref(),
                    value.context.base_input.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    require_value_row(storage, reference, slot, c)?;
                }
                if let Some(inputs) = &value.inputs {
                    inputs.text.require_disposal_owner(c)?;
                    if let Some(order) = inputs.order {
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        // Entries are qualified once, with their row.
                        if order.slot >= slot
                            || !matches!(
                                storage.payloads.values.get(order.slot),
                                Some(RetainedPayload::StyleOrder(list)) if order.len != 0 && order.len <= list.len()
                            )
                        {
                            return Err(latch_registry_row(c));
                        }
                    }
                    for reference in inputs
                        .attributes
                        .iter()
                        .flatten()
                        .chain(inputs.text.as_slice())
                    {
                        require_value_row(storage, reference, slot, c)?;
                    }
                    if let Some(document) = &inputs.parser_document {
                        storage.validate_association_document(c, document)?;
                    }
                }
                if let Some(provider) = value.context.base_document
                    && !matches!((&value.document, storage.payloads.values.get(provider)), (DocumentRef::Owned(document), Some(RetainedPayload::BaseDocument { node, facts, .. })) if node == document && value.context.base_observation.is_none_or(|index| index < facts.observations().len()))
                {
                    return Err(latch_registry_row(c));
                }
                // `HtmlParseAssociationContext::first_base` and `selected_base` read this node
                // without a controller, so it is established here, at publication.
                if let Some(first) = value.context.first_base {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if storage.view.node_at(first).is_none() {
                        return Err(latch_registry_row(c));
                    }
                }
                // `HtmlParseAssociationContext::csp_association` reads this row without a controller,
                // so its kind is established here, at publication.
                if let Some(policy) = value.context.csp
                    && !matches!(
                        storage.payloads.values.get(policy),
                        Some(RetainedPayload::CspPolicy(_))
                    )
                {
                    return Err(latch_registry_row(c));
                }
                if let Some(provider) = value.context.captured
                    && !matches!((&value.document, storage.payloads.values.get(provider)), (DocumentRef::Captured(document), Some(RetainedPayload::AssociationCapturedContext(context))) if context.value.document() == document)
                {
                    return Err(latch_registry_row(c));
                }
            }
            RetainedPayload::StyleOrder(list) => {
                list.require_disposal_owner(c)?;
                for node in list.as_slice() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if storage.view.node_at(*node).is_none() {
                        return Err(latch_registry_row(c));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
impl FrozenStorage {
    fn validate_association_document(
        &self,
        c: &mut ParseControl,
        document: &DocumentRef,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match document {
            DocumentRef::Owned(node)
                if matches!(
                    self.view.node_at(*node).map(|n| n.kind()),
                    Some(ConstructionNodeKind::Document(_))
                ) =>
            {
                Ok(())
            }
            DocumentRef::Captured(id)
                if self
                    .fragment
                    .as_ref()
                    .and_then(|f| f.context.capture().node(id.subject()).ok())
                    .is_some() =>
            {
                Ok(())
            }
            _ => Err(SessionError::WrongToken),
        }
    }
}
/// Releases the receiver and what each occurrence holds. A refusal is
/// remembered and the release goes on, so everything else is still credited;
/// the first refusal is returned.
pub(super) fn release_occurrence(
    mut values: OwnedVec<Occurrence>,
    c: &mut ParseControl,
) -> Result<(), HtmlParseStop> {
    let mut refused = None;
    loop {
        let popped = if c.stop().is_some() {
            values.cleanup_pop(c)
        } else {
            match values.try_pop(c) {
                Ok(popped) => Ok(popped),
                // The refusal latched a stop, which opens the cleanup pop.
                Err(stop) => {
                    refused.get_or_insert(stop);
                    values.cleanup_pop(c)
                }
            }
        };
        match popped {
            Ok(Some(value)) => {
                if let Err(stop) = release_inputs(value, c) {
                    refused.get_or_insert(stop);
                }
            }
            Ok(None) => break,
            Err(stop) => {
                refused.get_or_insert(stop);
                break;
            }
        }
    }
    if let Err(stop) = values.release(c) {
        refused.get_or_insert(stop);
    }
    refused.map_or(Ok(()), Err)
}
fn release_inputs(value: Occurrence, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
    match value.inputs {
        Some(inputs) => inputs.text.release(c),
        None => Ok(()),
    }
}
/// A value row an occurrence names must be a string row registered before the
/// occurrence's own row.
fn require_value_row(
    storage: &FrozenStorage,
    row: &ValueRow,
    occurrence: usize,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    if row.0 >= occurrence
        || !matches!(
            storage.payloads.values.get(row.0),
            Some(RetainedPayload::String { .. })
        )
    {
        return Err(latch_registry_row(c));
    }
    Ok(())
}
pub(super) fn captured_usage(value: &CapturedContext) -> associated::AssociatedStorageUsage {
    value.usage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture};
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };

    fn lexical_node(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
    ) -> ParserNode {
        let TokenKind::Tag(tag) = super::super::tests::lexical_kind(lexer, c) else {
            panic!("literal tag")
        };
        session.allocate_html_tag(c, &mut Some(tag)).unwrap()
    }
    fn generated(c: &mut ParseControl, units: &[u16]) -> Text {
        let mut value = Utf16Buffer::new(c).unwrap();
        value.extend_units(units, c).unwrap();
        Text {
            value: value.freeze(c).unwrap(),
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap(),
        }
    }
    fn read_group(session: &ParserArenaSession, _c: &mut ParseControl, row: &ValueRow) -> Vec<u16> {
        let storage = session.active().unwrap();
        let Some(RetainedPayload::String { value, .. }) = storage.payloads.values.get(row.0) else {
            panic!("a value row holds a string")
        };
        value.code_units().to_vec()
    }
    #[test]
    fn original_coalesced_text_retitle_removal_and_document_move_keep_old_exact_inputs() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<style title=A><i title=B>"),
            super::super::tests::limits(),
        );
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let root = session.root(&mut c).unwrap();
        let style = lexical_node(&mut session, &mut c, &mut lexer);
        session
            .insert(&mut c, &root, &style, None, Some(&capability))
            .unwrap();
        for units in [&[0xd800, 0][..], &[0x41, 0xdfff][..], &[0x42][..]] {
            let mut input = Some(generated(&mut c, units));
            session
                .insert_text(&mut c, &style, None, &mut input, Some(&capability))
                .unwrap();
            assert!(input.is_none());
        }
        let TokenKind::Tag(mut tag) = super::super::tests::lexical_kind(&mut lexer, &mut c) else {
            panic!("literal replacement")
        };
        let mut attr = tag.attributes.try_pop(&mut c).unwrap();
        tag.name.release(&mut c).unwrap();
        tag.attributes.release(&mut c).unwrap();
        session
            .set_managed_attribute(
                &mut c,
                &style,
                &mut attr,
                &capability,
                mutation::ManagedFamily::Style,
            )
            .unwrap();
        assert!(attr.is_none());
        session
            .remove_managed_attribute(&mut c, &style, "title", mutation::ManagedFamily::Style)
            .unwrap();
        let document = session
            .active_mut()
            .unwrap()
            .appropriate_contents_document(&mut c, &DocumentRef::Owned(0))
            .unwrap();
        let next = session
            .active()
            .unwrap()
            .node(&mut c, document.owned().unwrap())
            .unwrap();
        session
            .insert(&mut c, &next, &style, None, Some(&capability))
            .unwrap();
        let storage = session.active().unwrap();
        let rows: Vec<_> = storage
            .payloads
            .values
            .iter()
            .filter_map(|row| match row {
                RetainedPayload::AssociationOccurrence(values)
                    if values.as_slice()[0].subject == style.index =>
                {
                    Some(&values.as_slice()[0])
                }
                _ => None,
            })
            .collect();
        let texts: Vec<_> = rows
            .iter()
            .filter(|row| {
                matches!(
                    row.kind,
                    HtmlParseAssociationOccurrenceKind::StyleUpdate(
                        surgeist_html::elements::style::HtmlStyleTrigger::ChildrenChanged { .. }
                    )
                )
            })
            .collect();
        assert_eq!(texts.len(), 3);
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[0].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0]
        );
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[1].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0, 0x41, 0xdfff]
        );
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[2].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0, 0x41, 0xdfff, 0x42]
        );
        let attributes: Vec<_> = rows
            .iter()
            .filter(|row| row.kind == HtmlParseAssociationOccurrenceKind::StyleOwnerAttributes)
            .collect();
        assert!(attributes.iter().any(|row| {
            row.inputs.as_ref().unwrap().attributes[2]
                .as_ref()
                .is_some_and(|group| read_group(&session, &mut c, group) == [66])
        }));
        assert!(
            attributes
                .last()
                .unwrap()
                .inputs
                .as_ref()
                .unwrap()
                .attributes[2]
                .is_none()
        );
        let moved = rows
            .iter()
            .find(|row| row.kind == HtmlParseAssociationOccurrenceKind::StyleDocumentChanged)
            .unwrap();
        assert_eq!(moved.document, DocumentRef::Owned(0));
        assert_eq!(moved.previous_document.as_ref(), Some(&document));
        assert!(rows.iter().any(|row| row.document == document
            && matches!(row.kind, HtmlParseAssociationOccurrenceKind::StyleUpdate(_))));
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn original_evidence_rejects_native_style_without_document_binding() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(""),
            super::super::tests::limits(),
        );
        let (mut c, mut session, _) = super::super::tests::fresh_script_owner(&request);
        let storage = session.active_mut().unwrap();
        let style = storage
            .allocate_element(
                &mut c,
                PreexistingElement::new(
                    QualifiedName::new(Namespace::html(), None, "style"),
                    OrderedAttributes::default(),
                ),
            )
            .unwrap();
        assert!(matches!(
            storage.initialize_style(&mut c, style.index),
            Err(SessionError::NodeDocumentUnavailable)
        ));
        assert!(
            !storage
                .payloads
                .values
                .iter()
                .any(|row| matches!(row, RetainedPayload::AssociationOccurrence(_)))
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn an_association_value_survives_the_retirement_of_its_source_and_sticky_cancel_cleanup() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(""),
            super::super::tests::limits(),
        );
        let (mut c, mut session, _) = super::super::tests::fresh_script_owner(&request);
        let mut raw = Utf16Buffer::new(&mut c).unwrap();
        raw.extend_units(&[0xd800, 0, 65], &mut c).unwrap();
        let raw = raw.freeze(&mut c).unwrap();
        let (handle, slot) = session
            .active_mut()
            .unwrap()
            .register_group_string_owned(&mut c, raw, None)
            .unwrap();
        drop(handle);
        let reference = session
            .active_mut()
            .unwrap()
            .association_string(&mut c, slot)
            .unwrap();
        session
            .active_mut()
            .unwrap()
            .retire_registered_string(&mut c, slot)
            .unwrap();
        assert_eq!(read_group(&session, &mut c, &reference), [0xd800, 0, 65]);
        request.cancellation().cancel();
        let first = c.checkpoint().unwrap_err();
        assert_eq!(c.checkpoint(), Err(first));
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(first));
        super::super::tests::storage_zero(&c);
    }
    // Impossible states met while recording associations are reported as the
    // controller's first stop, never as a panic. Each test forces the state
    // through a controller hook, asserts the exact typed error and that the
    // stop is latched first, and that the session's own release drains.
    fn registry_row() -> HtmlParseStop {
        HtmlParseStop::InternalInvariant(crate::parser::control::HtmlParseInvariant::RegistryRow)
    }
    #[test]
    fn a_registry_walk_past_the_end_reports_the_registry_row() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(""),
            super::super::tests::limits(),
        );
        let (mut c, mut session, _) = super::super::tests::fresh_script_owner(&request);
        // The walk may add rows as it goes, so it must overshoot them all.
        c.overshoot_next_registry_walk(1_000);
        let result = session
            .active_mut()
            .unwrap()
            .prepare_association_documents(&mut c);
        assert_eq!(result, Err(SessionError::Stop(registry_row())));
        assert_eq!(c.stop(), Some(registry_row()));
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn a_stylesheet_token_reaching_past_its_relation_reports_the_registry_row() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<link rel=stylesheet href=a>"),
            super::super::tests::limits(),
        );
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let root = session.root(&mut c).unwrap();
        let link = lexical_node(&mut session, &mut c, &mut lexer);
        session
            .insert(&mut c, &root, &link, None, Some(&capability))
            .unwrap();
        c.overshoot_next_space_token(1);
        let result = session
            .active_mut()
            .unwrap()
            .association_link(&mut c, link.index);
        assert_eq!(result, Err(SessionError::Stop(registry_row())));
        assert_eq!(c.stop(), Some(registry_row()));
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    fn forged_occurrence(inputs: Option<Inputs>) -> Occurrence {
        Occurrence {
            subject: 0,
            document: DocumentRef::Owned(0),
            root: 0,
            previous_document: None,
            prior_document: None,
            prior_style: None,
            cause: None,
            kind: HtmlParseAssociationOccurrenceKind::StyleCreated,
            inputs,
            preferred: None,
            base_changed_input: None,
            context: Context {
                input_document: 0,
                mode: HtmlDocumentMode::NoQuirks,
                class: crate::HtmlDocumentClass::Html,
                body: false,
                base_document: None,
                first_base: None,
                base_input: None,
                base_observation: None,
                captured: None,
                policy_missing: false,
                csp: None,
                missing: false,
            },
            requires_processing: false,
            may_have_sheet: false,
        }
    }
    fn foreign_stop() -> HtmlParseStop {
        HtmlParseStop::InternalInvariant(crate::parser::control::HtmlParseInvariant::ForeignControl)
    }
    #[test]
    fn releasing_occurrences_to_a_foreign_controller_stops_instead_of_panicking() {
        let mut owner = super::super::tests::control();
        let mut foreign = super::super::tests::control();
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values
            .try_push_geometric(&mut owner, forged_occurrence(None))
            .unwrap();
        assert_eq!(
            release_occurrence(values, &mut foreign),
            Err(foreign_stop())
        );
        assert_eq!(foreign.stop(), Some(foreign_stop()));
    }
    #[test]
    fn releasing_an_occurrence_whose_text_inputs_are_foreign_stops_and_still_releases_the_rest() {
        let mut owner = super::super::tests::control();
        let mut foreign = super::super::tests::control();
        // An empty vector bound to the other controller by an empty pop.
        let mut text = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        assert!(text.try_pop(&mut foreign).unwrap().is_none());
        let inputs = Inputs {
            state: surgeist_html::elements::style::HtmlStyleState::default(),
            parser_document: None,
            connected: false,
            type_eligible: false,
            document_tree: false,
            attributes: [None, None, None, None, None],
            text,
            order: None,
        };
        let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries).unwrap();
        values
            .try_push_geometric(&mut owner, forged_occurrence(Some(inputs)))
            .unwrap();
        assert_eq!(release_occurrence(values, &mut owner), Err(foreign_stop()));
        assert_eq!(owner.stop(), Some(foreign_stop()));
        // The receiver itself was released although its inputs were refused.
        assert_eq!(
            owner
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
    }

    // A published report was qualified once, so a reference one of its rows
    // holds that its tables cannot resolve is a defect in that qualification.
    // Each test corrupts one such reference and requires the cursor to report
    // the fault at that row, and to stay faulted, rather than skip the row or
    // read the reference as absent. A skipped row is only one that is not an
    // association.
    fn styled_report() -> crate::HtmlParseReport {
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<!doctype html><style title=a type=text/css>b{}</style><p>c"),
            super::super::tests::limits(),
        )
        .parse()
        .unwrap()
    }
    fn frozen(report: &mut crate::HtmlParseReport) -> &mut FrozenStorage {
        report.storage_mut().history_mut().storage_mut()
    }
    fn fault(kind: read::HtmlParseUnresolvedKind) -> read::HtmlParseUnresolvedReference {
        read::HtmlParseUnresolvedReference::new(kind)
    }
    /// Edits the first style occurrence a report holds that the filter accepts.
    fn corrupt_style_occurrence(
        report: &mut crate::HtmlParseReport,
        select: impl Fn(&Occurrence) -> bool,
        edit: impl FnOnce(&mut Occurrence),
    ) {
        let row = frozen(report)
            .payloads
            .values
            .iter_mut()
            .find_map(|row| match row {
                RetainedPayload::AssociationOccurrence(values)
                    if values.as_slice()[0].inputs.is_some() && select(&values.as_slice()[0]) =>
                {
                    Some(&mut values.as_mut_slice_for_test()[0])
                }
                _ => None,
            })
            .unwrap();
        edit(row);
    }
    /// What a walk of the cursor lends: the associations presented, the rows
    /// skipped, and the first fault, which the cursor must keep reporting.
    struct Walk {
        presented: usize,
        skipped: usize,
        fault: Option<read::HtmlParseUnresolvedReference>,
    }
    fn walk(report: &crate::HtmlParseReport) -> Walk {
        let mut cursor = report.pending_associations();
        let mut walk = Walk {
            presented: 0,
            skipped: 0,
            fault: None,
        };
        loop {
            match cursor.advance() {
                HtmlParsePendingAssociationStep::Occurrence(_)
                | HtmlParsePendingAssociationStep::Document(_) => walk.presented += 1,
                HtmlParsePendingAssociationStep::Skipped => walk.skipped += 1,
                HtmlParsePendingAssociationStep::Complete => return walk,
                HtmlParsePendingAssociationStep::Fault(first) => {
                    assert!(
                        matches!(cursor.advance(), HtmlParsePendingAssociationStep::Fault(again) if again == first)
                    );
                    walk.fault = Some(first);
                    return walk;
                }
            }
        }
    }
    #[test]
    fn an_uncorrupted_report_lends_every_association_and_skips_only_other_rows() {
        let report = styled_report();
        let walk = walk(&report);
        assert!(walk.presented > 0 && walk.skipped > 0);
        assert_eq!(walk.fault, None);
    }
    fn assert_row_fault(report: &crate::HtmlParseReport, kind: read::HtmlParseUnresolvedKind) {
        assert_eq!(walk(report).fault, Some(fault(kind)));
    }
    #[test]
    fn an_occurrence_whose_subject_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(&mut report, |_| true, |row| row.subject = usize::MAX);
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }
    #[test]
    fn an_occurrence_whose_root_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(&mut report, |_| true, |row| row.root = usize::MAX);
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }
    #[test]
    fn an_occurrence_whose_document_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(
            &mut report,
            |_| true,
            |row| row.document = DocumentRef::Owned(usize::MAX),
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }
    #[test]
    fn an_occurrence_whose_previous_document_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(
            &mut report,
            |_| true,
            |row| row.previous_document = Some(DocumentRef::Owned(usize::MAX)),
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }
    #[test]
    fn an_occurrence_whose_creating_parser_document_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(
            &mut report,
            |_| true,
            |row| {
                row.inputs.as_mut().unwrap().parser_document = Some(DocumentRef::Owned(usize::MAX));
            },
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }
    #[test]
    fn an_occurrence_whose_cause_is_missing_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(
            &mut report,
            |row| row.cause.is_some(),
            |row| row.cause.as_mut().unwrap().corrupt_for_test(),
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Cause);
    }
    #[test]
    fn an_occurrence_whose_attribute_value_is_missing_is_a_fault() {
        let mut report = styled_report();
        let with_attribute = |row: &Occurrence| {
            row.inputs
                .as_ref()
                .is_some_and(|inputs| inputs.attributes.iter().any(Option::is_some))
        };
        corrupt_style_occurrence(&mut report, with_attribute, |row| {
            let attributes = &mut row.inputs.as_mut().unwrap().attributes;
            attributes.iter_mut().flatten().next().unwrap().0 = usize::MAX;
        });
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Fact);
    }
    #[test]
    fn an_occurrence_whose_policy_row_is_not_a_policy_is_a_fault() {
        let mut report = styled_report();
        corrupt_style_occurrence(
            &mut report,
            |row| !row.context.policy_missing,
            |row| row.context.csp = Some(usize::MAX),
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Fact);
    }
    #[test]
    fn an_occurrence_whose_base_facts_row_is_not_base_facts_is_a_fault() {
        let mut report =
            super::super::base::tests::request("<!doctype html><base href=A><style>b{}</style>")
                .parse()
                .unwrap();
        corrupt_style_occurrence(
            &mut report,
            |row| row.context.base_document.is_some(),
            |row| row.context.base_document = Some(usize::MAX),
        );
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Fact);
    }
    #[test]
    fn a_text_value_that_is_missing_is_a_fault_of_the_text_reader() {
        let mut report = styled_report();
        let with_text = |row: &Occurrence| row.inputs.as_ref().is_some_and(|i| i.text.len() != 0);
        corrupt_style_occurrence(&mut report, with_text, |row| {
            let text = row.inputs.as_mut().unwrap().text.as_mut_slice_for_test();
            text[0].0 = usize::MAX;
        });
        let mut cursor = report.pending_associations();
        let mut checked = false;
        loop {
            match cursor.advance() {
                HtmlParsePendingAssociationStep::Occurrence(view) => {
                    if let Some(style) = view.style()
                        && style.text_len() != 0
                    {
                        assert_eq!(
                            style.text(0).err(),
                            Some(fault(read::HtmlParseUnresolvedKind::Fact))
                        );
                        assert!(matches!(style.text(style.text_len()), Ok(None)));
                        checked = true;
                    }
                }
                HtmlParsePendingAssociationStep::Complete => break,
                HtmlParsePendingAssociationStep::Fault(fault) => panic!("{fault}"),
                _ => {}
            }
        }
        assert!(checked);
    }
    #[test]
    fn a_sheet_order_entry_that_is_missing_is_a_fault_of_the_order_reader() {
        let mut report = styled_report();
        for row in frozen(&mut report).payloads.values.iter_mut() {
            if let RetainedPayload::StyleOrder(list) = row {
                list.as_mut_slice_for_test()[0] = usize::MAX;
            }
        }
        let mut cursor = report.pending_associations();
        let mut checked = false;
        loop {
            match cursor.advance() {
                HtmlParsePendingAssociationStep::Occurrence(view) => {
                    if let Some(style) = view.style()
                        && style.sheet_order_len() != 0
                    {
                        assert_eq!(
                            style.sheet_order(0).err(),
                            Some(fault(read::HtmlParseUnresolvedKind::Node))
                        );
                        assert!(matches!(
                            style.sheet_order(style.sheet_order_len()),
                            Ok(None)
                        ));
                        checked = true;
                    }
                }
                HtmlParsePendingAssociationStep::Complete => break,
                HtmlParsePendingAssociationStep::Fault(fault) => panic!("{fault}"),
                _ => {}
            }
        }
        assert!(checked);
    }
    #[test]
    fn documents_whose_capture_is_missing_are_a_fault() {
        let mut report =
            super::super::fragment_document::tests::fragment_report("<style>a</style>");
        frozen(&mut report).fragment = None;
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Fragment);
    }
    #[test]
    fn a_base_observation_that_is_missing_is_a_fault() {
        let mut report =
            super::super::base::tests::request("<!doctype html><base href=A><style>b{}</style>")
                .parse()
                .unwrap();
        let with_base = |row: &Occurrence| {
            row.context.base_document.is_some()
                && row.context.first_base.is_some()
                && row.context.base_observation.is_some()
        };
        corrupt_style_occurrence(&mut report, with_base, |row| {
            row.context.base_observation = Some(usize::MAX);
        });
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Fact);
    }
    #[test]
    fn a_first_base_that_is_missing_is_a_fault() {
        let mut report =
            super::super::base::tests::request("<!doctype html><base href=A><style>b{}</style>")
                .parse()
                .unwrap();
        let with_base = |row: &Occurrence| row.context.first_base.is_some();
        corrupt_style_occurrence(&mut report, with_base, |row| {
            row.context.first_base = Some(usize::MAX);
        });
        assert_row_fault(&report, read::HtmlParseUnresolvedKind::Node);
    }

    /// The order remembered for a subtree being removed is the order of the
    /// subtree as it was then: a style inserted into the subtree afterwards
    /// ends it, so a second removal of the same subtree lists the new style.
    /// No parsed document reaches this, because the removal of the children of a
    /// moved element always builds another subtree's order in between.
    #[test]
    fn a_style_inserted_into_a_removed_subtree_ends_its_remembered_order() {
        let request = super::super::effects::tests::request(
            "<body><div><style>1{}</style><style>2{}</style>",
        );
        let (mut c, mut session, proof, nodes) = super::super::effects::tests::graph(&request);
        let find = |name: &str, nth: usize| {
            nodes
                .iter()
                .filter(|(found, _)| found == name)
                .nth(nth)
                .map(|(_, node)| *node)
                .expect("a parsed element")
        };
        let (div, second) = (find("div", 0), find("style", 1));
        session.remove(&mut c, &second, Some(&proof)).unwrap();
        let storage = session.active_mut().unwrap();
        // The order of the subtree being removed, remembered.
        let first = storage
            .style_order(&mut c, div.index, Some(div.index))
            .unwrap()
            .expect("the first style");
        assert_eq!(first.len, 1);
        session
            .insert(&mut c, &div, &second, None, Some(&proof))
            .unwrap();
        let storage = session.active_mut().unwrap();
        let listed = storage
            .style_order(&mut c, div.index, Some(div.index))
            .unwrap()
            .expect("both styles");
        assert_eq!(listed.len, 2);
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
}
