//! The payload registry: actual retained backing and exact record-qualified ownership.

use super::*;
use surgeist_html::{
    construction::{HtmlElement, HtmlNodeKind},
    elements::{
        form::{HtmlNativeFormAssociationKind, HtmlTextareaFlags},
        script::{HtmlScriptFlags, HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult},
    },
    values::{Attribute, DocumentString},
};

/// Actual backing and exact record-qualified ownership, not allocation proxies.
pub(super) struct PayloadStorage {
    pub(super) values: Vec<RetainedPayload>,
    pub(super) allocation: VectorCapacity<RetainedPayload>,
    pub(super) node_joins: Vec<NodeJoins>,
    pub(super) node_joins_allocation: VectorCapacity<NodeJoins>,
    /// Every row of a kind that is also searched across all nodes, in registry
    /// order. A scan of these lists visits only rows of that kind.
    pub(super) kind_slots: [KindSlots; KIND_LISTS],
    /// The `FormAssociation` rows of listed controls that have a `form`
    /// attribute, in registry order: the only rows an id event can affect.
    /// Created with the first such row. Only creation and cloning give a
    /// control its `form` attribute (the attribute mutators change `value`,
    /// `nonce`, `open` and `checked`, and a merge touches only html and body),
    /// so rows enter the list when they are registered.
    pub(super) explicit_forms: Option<KindSlots>,
    /// Auxiliary entries admitted for rows that are not appended yet: each
    /// [`PreparedSlot`] adds its entries when it is issued and gives them back
    /// when its row is appended. A slot that is dropped instead of appended
    /// leaves its entries here, so the release that retires the registry still
    /// returns them.
    pub(super) prepared_entries: u64,
    pub(super) preexisting_units: usize,
    pub(super) preexisting_attributes: usize,
    /// Registry rows below this slot were audited by an earlier creation-time
    /// `check_new_aliases`; the completed-freeze preflight and release audit
    /// every row again.
    pub(super) audited_rows: usize,
    /// Registry rows of selectedcontent metadata. Rows are never removed, so zero
    /// proves that no select has a selectedcontent descendant to update.
    pub(super) selectedcontent_rows: usize,
    /// Times a second row of a family that report preparation requires to be
    /// unique per record joined that record. Zero proves the join names the
    /// only row, so preparation need not scan the registry to count them.
    pub(super) repeated_unique_rows: usize,
    /// The generation of the cached tree roots in [`NodeJoins`]; `None` once no
    /// later generation can be named, when nothing is cached.
    pub(super) tree_stamp: Option<NonZeroU32>,
    /// How many topology changes have been announced, or `None` once the count
    /// cannot go on; see [`tree_root`].
    pub(super) topology_changes: Option<u64>,
}

pub(super) enum RetainedPayload {
    CspState(Option<Box<csp::State>>),
    CspPolicy(Box<csp::Policy>),
    CspTransition(Box<csp::Transition>),
    AssociationDocument(pending_association::DocumentCapture),
    AssociationOccurrence(OwnedVec<pending_association::Occurrence>),
    AssociationCapturedContext(pending_association::CapturedContext),
    AssociatedInputs {
        inputs: Arc<associated::Inputs>,
        usage: associated::AssociatedStorageUsage,
    },
    Dialog {
        node: usize,
        state: dialog::DialogMetadata,
    },
    Style {
        node: usize,
        state: style::StyleMetadata,
    },
    Hyperlink {
        node: usize,
        state: associated::HyperlinkMetadata,
    },
    Image {
        node: usize,
        state: associated::ImageMetadata,
    },
    Nonce {
        node: Option<usize>,
        raw: usize,
    },
    BaseDocument {
        node: usize,
        facts: crate::HtmlParseBaseFacts,
        first: Option<usize>,
    },
    Base {
        node: Option<usize>,
        domain: base::ResponseDomain,
        frozen: base::BaseValue,
    },
    Shadow {
        root: Option<usize>,
        state: context::ShadowState,
    },
    ConstructionCause {
        node: Option<usize>,
        cause: provenance::ConstructionCauseRef,
    },
    StandaloneDocumentCompletion {
        node: usize,
        state: HtmlParseDocumentState,
    },
    PiAttributes {
        node: Option<usize>,
        values: OwnedVec<pi::PseudoAttribute>,
        ready: bool,
    },
    LookupRegistry {
        node: Option<usize>,
        registry: context::RegistryRef,
        creation: Option<context::ElementCreationState>,
    },
    NodeDocument {
        node: Option<usize>,
        document: DocumentRef,
    },
    Document {
        node: Option<usize>,
        state: template::DocumentState,
        class: crate::HtmlDocumentClass,
        meta: meta::MetaHistory,
        interaction: dialog::InteractionHistory,
    },
    MetaMissing {
        document: usize,
        subject: usize,
        kind: meta::HtmlParseMetaMissingKind,
        cause: Option<provenance::ConstructionCauseRef>,
    },
    Template {
        node: Option<usize>,
        state: template::TemplateMetadata,
    },
    String {
        value: DocumentString,
        allocation: DocumentStringAllocation,
        origins: Option<OriginGroupRef>,
        location: StringLocation,
    },
    HistoricalString {
        origins: OriginGroupRef,
    },
    Attributes(Option<AttributeStorage>),
    // Presence is an actual closed-recipient proof issued by ordinary creation;
    // no task was queued and the source toggle tracker remains null.
    Details {
        node: Option<usize>,
        notification: details::NotificationDomain,
    },
    SvgScript {
        node: Option<usize>,
        state: SvgScriptState,
    },
    Link {
        node: Option<usize>,
        parser_document: Option<DocumentRef>,
    },
    Script {
        node: Option<usize>,
        metadata: ScriptMetadata,
    },
    FormAssociation {
        // None is an admitted, initialized construction slot. Only an actual
        // successful allocation can bind it; a stopped slot is cleanup-only.
        node: Option<usize>,
        state: FormAssociationState,
    },
    Textarea {
        node: Option<usize>,
        state: TextareaState,
    },
    Output {
        node: Option<usize>,
        state: OutputState,
    },
    Input {
        node: Option<usize>,
        raw: usize,
        complete: bool,
        state: InputState,
    },
    InputSource {
        node: Option<usize>,
        token: crate::HtmlSourceRange,
        value_attribute: Option<crate::HtmlSourceRange>,
    },
    SelectFamily {
        node: Option<usize>,
        state: SelectMetadata,
    },
    WorkList(Option<OwnedVec<usize>>),
    /// The style elements of one tree in tree order, shared by the style
    /// occurrences recorded while the tree held exactly a prefix of them.
    StyleOrder(OwnedVec<usize>),
    // A replaced raw backing was disposed; its initialized registry slot
    // remains admitted until the registry itself is released.
    Retired,
}

/// Registry-wide kind lists with their own admitted backing.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum KindList {
    FormAssociation,
    Base,
    AssociationDocument,
}

const KIND_LISTS: usize = KindList::AssociationDocument as usize + 1;

/// The admission of one registry row: its auxiliary entry, and the entries of
/// the lists it joins, charged before the row exists. Only
/// `SessionStorage::prepare_payload_slot*` issues one, and `PayloadStorage::push_row`
/// consumes it by appending the row it admitted, so a row cannot be appended
/// unadmitted. A slot that is dropped instead leaves its entries in
/// [`PayloadStorage::prepared_entries`], which the release of the registry
/// returns with the entries of the rows.
#[must_use = "an admitted slot is consumed by appending its row with `push_row`"]
pub(super) struct PreparedSlot {
    pub(super) entries: u64,
    pub(super) list: Option<KindList>,
    pub(super) explicit_form: bool,
}

pub(super) struct KindSlots {
    pub(super) slots: Vec<usize>,
    pub(super) allocation: VectorCapacity<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextareaState {
    pub(super) raw: usize,
    pub(crate) flags: HtmlTextareaFlags,
}

/// Pinned output override issued only by actual fresh creation reset.
/// Nonnull setters/general form reset have no producer in this checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HtmlParseOutputOverride {
    Null,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OutputState {
    pub(crate) default_value_override: HtmlParseOutputOverride,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InputState {
    pub(crate) flags: surgeist_html::elements::input::HtmlInputFlags,
    pub(crate) cursor: usize,
    pub(crate) selected_files_empty: bool,
    pub(crate) value_rewritten_during_creation: bool,
    /// Actual produced multiple-Email values, not a resplit of joined text.
    pub(crate) email_value_count: Option<usize>,
    pub(super) image: input_image::ImageState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FormAssociationState {
    pub(crate) kind: HtmlNativeFormAssociationKind,
    pub(super) owner: Option<usize>,
    pub(crate) parser_inserted: bool,
    pub(crate) evidence: HtmlParseFormEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseFormEvidence {
    Created,
    ParserAssociated,
    Reset,
}

/// Source processing model facts retained in the actual arena owner.
/// Owned document positions stay local; captured documents retain checked IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMetadata {
    pub(super) parser_document: Option<DocumentRef>,
    pub(super) preparation_document: Option<DocumentRef>,
    pub(crate) flags: HtmlScriptFlags,
    pub(crate) script_type: Option<HtmlScriptKind>,
    pub(crate) result: HtmlScriptResult,
    pub(crate) steps_when_result_ready: Option<HtmlScriptReadySteps>,
}

impl ScriptMetadata {
    pub(super) fn admit_clone(&self, c: &mut ParseControl) -> Result<(), SessionError> {
        for document in [
            self.parser_document.as_ref(),
            self.preparation_document.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            document.admit_clone(c)?;
        }
        Ok(())
    }
}

/// The join a registry row provides, when it is bound to a record.
fn row_join(row: &RetainedPayload) -> Option<(Join, usize)> {
    Some(match row {
        RetainedPayload::Attributes(Some(
            AttributeStorage::Frozen {
                element: Some(node),
                ..
            }
            | AttributeStorage::Removing {
                element: Some(node),
                ..
            },
        )) => (Join::Attributes, *node),
        RetainedPayload::NodeDocument {
            node: Some(node), ..
        } => (Join::NodeDocument, *node),
        RetainedPayload::Document {
            node: Some(node), ..
        } => (Join::Document, *node),
        RetainedPayload::Template {
            node: Some(node), ..
        } => (Join::Template, *node),
        RetainedPayload::FormAssociation {
            node: Some(node), ..
        } => (Join::FormAssociation, *node),
        RetainedPayload::Script {
            node: Some(node), ..
        } => (Join::Script, *node),
        RetainedPayload::Image { node, .. } => (Join::Image, *node),
        RetainedPayload::Hyperlink { node, .. } => (Join::Hyperlink, *node),
        RetainedPayload::Style { node, .. } => (Join::Style, *node),
        RetainedPayload::Dialog { node, .. } => (Join::Dialog, *node),
        RetainedPayload::Nonce {
            node: Some(node), ..
        } => (Join::Nonce, *node),
        RetainedPayload::Input {
            node: Some(node), ..
        } => (Join::Input, *node),
        RetainedPayload::SelectFamily {
            node: Some(node), ..
        } => (Join::SelectFamily, *node),
        RetainedPayload::Base {
            node: Some(node), ..
        } => (Join::Base, *node),
        RetainedPayload::BaseDocument { node, .. } => (Join::BaseDocument, *node),
        RetainedPayload::LookupRegistry {
            node: Some(node), ..
        } => (Join::LookupRegistry, *node),
        RetainedPayload::String {
            location: StringLocation::ProcessingInstructionData(node),
            ..
        } => (Join::ProcessingInstructionData, *node),
        RetainedPayload::PiAttributes {
            node: Some(node), ..
        } => (Join::ProcessingInstructionAttributes, *node),
        RetainedPayload::SvgScript {
            node: Some(node), ..
        } => (Join::SvgScript, *node),
        RetainedPayload::Details {
            node: Some(node), ..
        } => (Join::Details, *node),
        _ => return None,
    })
}

/// The registry-wide list a row belongs to, whatever its binding.
pub(super) fn row_list(row: &RetainedPayload) -> Option<KindList> {
    match row {
        RetainedPayload::FormAssociation { .. } => Some(KindList::FormAssociation),
        RetainedPayload::Base { .. } => Some(KindList::Base),
        RetainedPayload::AssociationDocument(_) => Some(KindList::AssociationDocument),
        _ => None,
    }
}

/// The element record at `node`. A missing position or a record of another
/// kind is an impossible registry state, latched as the controller's first stop.
pub(super) fn element_record<'a>(
    construction: &'a ConstructionOwner,
    control: &mut ParseControl,
    node: usize,
) -> Result<&'a HtmlElement, SessionError> {
    match construction.node_at(node).map(|record| record.kind()) {
        Some(HtmlNodeKind::Element(element)) => Ok(element),
        _ => Err(latch_registry_row(control)),
    }
}

/// The attribute at `position` of the element record at `node`, with the same
/// registry-state stop as [`element_record`].
pub(super) fn element_attribute<'a>(
    construction: &'a ConstructionOwner,
    control: &mut ParseControl,
    node: usize,
    position: usize,
) -> Result<&'a Attribute, SessionError> {
    match element_record(construction, control, node)?
        .attributes()
        .as_slice()
        .get(position)
    {
        Some(attribute) => Ok(attribute),
        None => Err(latch_registry_row(control)),
    }
}

impl PayloadStorage {
    /// The registry row at `slot`. A slot past the registry is an impossible
    /// registry state, latched as the controller's first stop.
    pub(super) fn row(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<&RetainedPayload, SessionError> {
        self.values
            .get(slot)
            .ok_or_else(|| latch_registry_row(control))
    }
    /// As [`Self::row`], for editing.
    pub(super) fn row_mut(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<&mut RetainedPayload, SessionError> {
        match self.values.get_mut(slot) {
            Some(row) => Ok(row),
            None => Err(latch_registry_row(control)),
        }
    }
    /// The first registry row of `join` bound to `node`: one join cell, however
    /// many other rows the registry retains. The caller checks the row it gets.
    pub(super) fn joined(
        &self,
        control: &mut ParseControl,
        node: usize,
        join: Join,
    ) -> Result<Option<usize>, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self.node_joins.get(node).and_then(|cell| cell.get(join)))
    }
    /// Forgets the join of the row at `slot`, for a test that leaves the row
    /// bound to no node.
    #[cfg(test)]
    pub(super) fn unjoin_for_test(&mut self, slot: usize) {
        if let Some((join, node)) = self.values.get(slot).and_then(row_join)
            && let Some(cell) = self.node_joins.get_mut(node)
        {
            *cell.rows.get_mut(join) = None;
        }
    }
    /// The same join cell without a charge, for a published report, which has
    /// no controller. The caller checks the row it gets.
    pub(super) fn joined_uncharged(&self, node: usize, join: Join) -> Option<usize> {
        self.node_joins.get(node).and_then(|cell| cell.get(join))
    }
    /// Registers the row at `slot` under the record it is bound to. Binding a
    /// row and joining it are one infallible step at every binding site; the
    /// test-build check at release compares every join with the registry.
    pub(super) fn join_row(&mut self, slot: usize) -> Result<(), SessionError> {
        let Some((join, node)) = self.values.get(slot).and_then(row_join) else {
            return Ok(());
        };
        let repeated = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?
            .set_first(join, slot)?;
        if repeated && join.unique_per_record() {
            self.repeated_unique_rows = self
                .repeated_unique_rows
                .checked_add(1)
                .ok_or_else(SessionError::registry_row)?;
        }
        Ok(())
    }
    /// How many rows of `join` name `node`, or `None` when only a registry scan
    /// can tell. With no repeated row of any unique family, the join names the
    /// only row, so one cell and one row decide the count.
    pub(super) fn unique_row_count(
        &self,
        control: &mut ParseControl,
        node: usize,
        join: Join,
    ) -> Result<Option<usize>, SessionError> {
        if self.repeated_unique_rows != 0 || !join.unique_per_record() {
            return Ok(None);
        }
        let Some(slot) = self.joined(control, node, join)? else {
            return Ok(Some(0));
        };
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self
            .values
            .get(slot)
            .and_then(row_join)
            .filter(|&(row, owner)| row == join && owner == node)
            .map(|_| 1))
    }
    /// Names `slot` as the latest row of `join` for `node`; used by the kinds
    /// whose join follows a back-to-front scan rather than the first row.
    pub(super) fn join_latest(
        &mut self,
        node: usize,
        join: Join,
        slot: usize,
    ) -> Result<(), SessionError> {
        self.node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?
            .set_latest(join, slot)
    }
    /// Forgets the `join` row of `node`.
    pub(super) fn clear_join(&mut self, node: usize, join: Join) {
        if let Some(cell) = self.node_joins.get_mut(node) {
            cell.clear(join);
        }
    }
    /// The registry row now holding a record's merged text.
    pub(super) fn join_text(&mut self, node: usize, slot: usize) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.text = Some(joined_cell(slot)?);
        Ok(())
    }
    /// The registry row holding a record's textarea state.
    pub(super) fn join_textarea(&mut self, node: usize, slot: usize) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.textarea = Some(joined_cell(slot)?);
        Ok(())
    }
    /// The registry slots of one registry-wide kind, in registry order.
    pub(super) fn kind_list(&self, list: KindList) -> &[usize] {
        self.kind_slots
            .get(list as usize)
            .map_or(&[], |kind| kind.slots.as_slice())
    }
    /// The `FormAssociation` rows of listed controls with a `form` attribute.
    pub(super) fn explicit_form_list(&self) -> &[usize] {
        self.explicit_forms
            .as_ref()
            .map_or(&[], |kind| kind.slots.as_slice())
    }
    /// Appends the row `prepared` admitted. A row already bound to a record is
    /// joined; rows of a registry-wide kind enter that kind's list, and a row
    /// of a listed control with a `form` attribute enters the explicit-form
    /// list, whose entries were admitted with the row. A row of another kind
    /// than the one admitted, or a list slot that was not admitted, refuses the
    /// row before anything is appended; the refused row stays in the registry
    /// for its release (see [`Self::retain_refused`]).
    pub(super) fn push_row(
        &mut self,
        prepared: PreparedSlot,
        row: RetainedPayload,
    ) -> Result<usize, SessionError> {
        let (prepared_entries, selectedcontent_rows) = match self.admit_row(&prepared, &row) {
            Ok(admitted) => admitted,
            Err(error) => {
                self.retain_refused(row);
                return Err(error);
            }
        };
        let list = row_list(&row);
        crate::parser::control::tally::row(format!("{:?}", std::mem::discriminant(&row)));
        let slot = self.values.len();
        self.prepared_entries = prepared_entries;
        self.selectedcontent_rows = selectedcontent_rows;
        self.values.push(row);
        if let Some(kind) = list.and_then(|list| self.kind_slots.get_mut(list as usize)) {
            kind.slots.push(slot);
        }
        if prepared.explicit_form
            && let Some(kind) = self.explicit_forms.as_mut()
        {
            kind.slots.push(slot);
        }
        self.join_row(slot)?;
        Ok(slot)
    }
    /// Checks that `row` is the kind of row `prepared` admitted and that
    /// everything the append writes was admitted with it, and returns the
    /// counts the append leaves. Writes nothing.
    fn admit_row(
        &self,
        prepared: &PreparedSlot,
        row: &RetainedPayload,
    ) -> Result<(u64, usize), SessionError> {
        let list = row_list(row);
        if list != prepared.list
            || prepared.explicit_form && list != Some(KindList::FormAssociation)
        {
            return Err(SessionError::registry_row());
        }
        if let Some(list) = list
            && self
                .kind_slots
                .get(list as usize)
                .is_none_or(|kind| kind.slots.len() >= kind.slots.capacity())
        {
            return Err(SessionError::registry_row());
        }
        if prepared.explicit_form
            && self
                .explicit_forms
                .as_ref()
                .is_none_or(|kind| kind.slots.len() >= kind.slots.capacity())
        {
            return Err(SessionError::registry_row());
        }
        let prepared_entries = self
            .prepared_entries
            .checked_sub(prepared.entries)
            .ok_or_else(SessionError::registry_row)?;
        let selectedcontent_rows = if matches!(
            row,
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Selectedcontent(_),
                ..
            }
        ) {
            self.selectedcontent_rows
                .checked_add(1)
                .ok_or_else(SessionError::registry_row)?
        } else {
            self.selectedcontent_rows
        };
        Ok((prepared_entries, selectedcontent_rows))
    }
    /// Keeps a refused row in the registry, unlisted and unjoined, so that the
    /// release of the registry returns whatever tickets the row owns: only that
    /// release knows how to return each kind of row. The slot the prepare
    /// reserved holds the row, and the row takes over the entry of its slot.
    /// The entries admitted for its lists stay outstanding in
    /// `prepared_entries`. A refusal is an inconsistency of the registry, which
    /// the caller reports as the registry row.
    fn retain_refused(&mut self, row: RetainedPayload) {
        if self.values.len() < self.values.capacity() {
            self.prepared_entries = self.prepared_entries.saturating_sub(1);
            self.values.push(row);
        }
    }
    /// Records where creation registered the record's attribute strings: the
    /// name of its last attribute, followed by that attribute's value, then the
    /// previous attribute's name and value, and so on.
    pub(super) fn record_attribute_strings(
        &mut self,
        node: usize,
        last_name_slot: usize,
    ) -> Result<(), SessionError> {
        let cell = self
            .node_joins
            .get_mut(node)
            .ok_or_else(SessionError::registry_row)?;
        cell.attribute_strings = Some(joined_cell(last_name_slot)?);
        Ok(())
    }
    /// An attribute edit makes the creation-time layout unreliable.
    pub(super) fn forget_attribute_strings(&mut self, node: usize) {
        if let Some(cell) = self.node_joins.get_mut(node) {
            cell.attribute_strings = None;
        }
    }
    /// The slot where `position`'s value row sits if the record's attributes
    /// are as creation registered them. Only a candidate: the caller verifies
    /// the row's identity before use.
    pub(super) fn attribute_value_candidate(
        &self,
        node: usize,
        attribute_count: usize,
        position: usize,
    ) -> Option<usize> {
        let last_name = cell_slot(self.node_joins.get(node)?.attribute_strings?)?;
        let from_last = attribute_count.checked_sub(1)?.checked_sub(position)?;
        last_name
            .checked_add(from_last.checked_mul(2)?)?
            .checked_add(1)
    }
    /// The first registry string row sharing `value`'s backing: what the
    /// fallback scan finds. Test builds compare every verified hint with it.
    #[cfg(test)]
    pub(super) fn first_string_sharing(&self, value: &DocumentString) -> Option<usize> {
        self.values.iter().position(|row| {
            matches!(row, RetainedPayload::String { value: stored, .. }
                if Arc::ptr_eq(value.shared_code_units(), stored.shared_code_units()))
        })
    }
    /// Every join must name exactly what a front-to-back scan of the registry
    /// finds, and every list exactly the rows of its kind.
    #[cfg(test)]
    fn assert_joins_match_registry(&self) {
        let mut expected: Vec<NodeJoins> = vec![NodeJoins::default(); self.node_joins.len()];
        let mut lists: [Vec<usize>; KIND_LISTS] = Default::default();
        for (slot, row) in self.values.iter().enumerate() {
            if let Some((join, node)) = row_join(row) {
                let cell = &mut expected[node];
                assert!(
                    cell.get(join).is_none(),
                    "two {join:?} rows are bound to record {node}"
                );
                cell.set_first(join, slot).unwrap();
            }
            if let Some(list) = row_list(row) {
                lists[list as usize].push(slot);
            }
            if let Some(subject) = pending_association::style_occurrence_subject(row) {
                expected[subject]
                    .set_latest(Join::StyleOccurrence, slot)
                    .unwrap();
            }
        }
        for (node, (cell, joined)) in expected.iter().zip(&self.node_joins).enumerate() {
            // A style order join names a row the registry cannot derive: the
            // row that was current when it was written.
            let mut cell = *cell;
            *cell.rows.get_mut(Join::StyleOrder) = joined.rows.copy(Join::StyleOrder);
            if let Some(slot) = joined.get(Join::StyleOrder) {
                assert!(
                    matches!(self.values.get(slot), Some(RetainedPayload::StyleOrder(_))),
                    "the style order join of record {node} names row {slot}, which is not a style order"
                );
            }
            assert!(
                cell.rows == joined.rows,
                "joins of record {node} differ from the registry: {:?} against {:?}",
                cell.rows,
                joined.rows
            );
        }
        for (list, kind) in lists.iter().zip(&self.kind_slots) {
            assert!(list == &kind.slots, "a kind list differs from the registry");
        }
    }
    /// Cleanup is a bounded ownership walk over admitted entries, permitted
    /// after a terminal stop. No parser work is resumed and no work refunded.
    /// Each live location is checked against its actual immutable record.
    pub(super) fn check_aliases(
        &self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.audit_aliases(records, control, 0..self.values.len())
    }
    /// The audit of the one row at `slot`, for an edit of the record that row
    /// backs: the edit is refused if that row is aliased, without the cost of
    /// auditing the registry.
    pub(super) fn check_row_aliases(
        &self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        self.audit_aliases(records, control, slot..slot.saturating_add(1))
    }
    /// The creation-time audit: only the rows registered since the previous one,
    /// so the cost is the rows the operations in between added, not the registry.
    /// Every row is audited again by the completed-freeze preflight and release.
    pub(super) fn check_new_aliases(
        &mut self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.audit_aliases(records, control, self.audited_rows..self.values.len())?;
        self.audited_rows = self.values.len();
        Ok(())
    }
    fn audit_aliases(
        &self,
        records: ConstructionView<'_>,
        control: &mut ParseControl,
        rows: std::ops::Range<usize>,
    ) -> Result<(), SessionError> {
        for index in rows {
            cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
            let payload = self
                .values
                .get(index)
                .ok_or_else(|| latch_registry_row(control))?;
            match payload {
                RetainedPayload::String {
                    value,
                    allocation,
                    location,
                    ..
                } => {
                    if allocation.finalized() {
                        control.require_document_string_for_disposal(allocation, value)?;
                    }
                    let stored = location.value(records, &self.values, index, control)?;
                    cleanup_charge(control, HtmlParseWorkCategory::ProviderCall, 2)?;
                    let expected = if let Some(stored) = stored {
                        if !Arc::ptr_eq(stored.shared_code_units(), value.shared_code_units()) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    if !allocation.finalized()
                        && Arc::strong_count(value.shared_code_units()) != expected
                    {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(
                    AttributeStorage::Frozen {
                        values,
                        element,
                        shared,
                        ..
                    }
                    | AttributeStorage::Removing {
                        values,
                        element,
                        shared,
                        ..
                    },
                )) => {
                    let expected = if let Some(index) = element {
                        cleanup_charge(control, HtmlParseWorkCategory::RecordVisit, 1)?;
                        cleanup_charge(control, HtmlParseWorkCategory::ProviderCall, 1)?;
                        let Some(HtmlNodeKind::Element(record)) =
                            records.node_at(*index).map(|node| node.kind())
                        else {
                            return Err(SessionError::AliasedPayload);
                        };
                        if !Arc::ptr_eq(
                            values.shared_attributes(),
                            record.attributes().shared_attributes(),
                        ) {
                            return Err(SessionError::AliasedPayload);
                        }
                        2
                    } else {
                        1
                    };
                    cleanup_charge(control, HtmlParseWorkCategory::ProviderCall, 1)?;
                    if !shared.finalized()
                        && Arc::strong_count(values.shared_attributes()) != expected
                    {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(AttributeStorage::Unvalidated {
                    backing,
                    shared,
                    ..
                })) => {
                    cleanup_charge(control, HtmlParseWorkCategory::ProviderCall, 1)?;
                    if !shared.finalized() && Arc::strong_count(backing) != 1 {
                        return Err(SessionError::AliasedPayload);
                    }
                }
                RetainedPayload::Attributes(Some(
                    AttributeStorage::Building { .. } | AttributeStorage::Transferring { .. },
                )) => {}
                RetainedPayload::Attributes(None) => return Err(SessionError::Consumed),
                RetainedPayload::CspState(_)
                | RetainedPayload::CspPolicy(_)
                | RetainedPayload::CspTransition(_)
                | RetainedPayload::BaseDocument { .. }
                | RetainedPayload::Base { .. }
                | RetainedPayload::SvgScript { .. }
                | RetainedPayload::ConstructionCause { .. }
                | RetainedPayload::Script { .. }
                | RetainedPayload::NodeDocument { .. }
                | RetainedPayload::LookupRegistry { .. }
                | RetainedPayload::Shadow { .. }
                | RetainedPayload::StandaloneDocumentCompletion { .. }
                | RetainedPayload::Document { .. }
                | RetainedPayload::MetaMissing { .. }
                | RetainedPayload::Template { .. }
                | RetainedPayload::FormAssociation { .. }
                | RetainedPayload::Textarea { .. }
                | RetainedPayload::Nonce { .. }
                | RetainedPayload::Output { .. }
                | RetainedPayload::InputSource { .. }
                | RetainedPayload::Input { .. }
                | RetainedPayload::SelectFamily { .. }
                | RetainedPayload::Details { .. }
                | RetainedPayload::Link { .. }
                | RetainedPayload::AssociatedInputs { .. }
                | RetainedPayload::Hyperlink { .. }
                | RetainedPayload::Image { .. }
                | RetainedPayload::Dialog { .. }
                | RetainedPayload::Style { .. }
                | RetainedPayload::AssociationDocument(_)
                | RetainedPayload::AssociationOccurrence(_)
                | RetainedPayload::AssociationCapturedContext(_)
                | RetainedPayload::Retired => {}
                RetainedPayload::HistoricalString { origins } => {
                    origins.require_disposal_owner(control)?
                }
                RetainedPayload::PiAttributes { values, .. } => {
                    values.require_disposal_owner(control)?
                }
                RetainedPayload::WorkList(Some(list)) => list.require_disposal_owner(control)?,
                RetainedPayload::WorkList(None) => return Err(SessionError::Consumed),
                RetainedPayload::StyleOrder(list) => list.require_disposal_owner(control)?,
            }
        }
        Ok(())
    }

    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> Result<(), SessionError> {
        #[cfg(test)]
        self.assert_joins_match_registry();
        let Self {
            mut values,
            allocation,
            node_joins,
            node_joins_allocation,
            kind_slots,
            explicit_forms,
            prepared_entries,
            ..
        } = self;
        let mut csp_storage = crate::meta_csp::Storage::ZERO;
        let join_entries = node_joins.len() as u64;
        // A refused release latches its stop in the controller (`ParseControl::fail`),
        // and cleanup continues so every other owner is still released; the first
        // refusal is returned once the walk is complete.
        let mut refused = release_vector(control, node_joins, node_joins_allocation).err();
        keep_first(
            &mut refused,
            release_logical(
                control,
                owner,
                HtmlParseResource::AuxiliaryEntries,
                join_entries,
            ),
        );
        for list in kind_slots.into_iter().chain(explicit_forms) {
            let list_entries = list.slots.len() as u64;
            if let Err(error) = release_vector(control, list.slots, list.allocation) {
                refused.get_or_insert(error);
            }
            keep_first(
                &mut refused,
                release_logical(
                    control,
                    owner,
                    HtmlParseResource::AuxiliaryEntries,
                    list_entries,
                ),
            );
        }
        // Entries admitted for rows that were never appended belong to no row.
        let entries = (values.len() as u64).saturating_add(prepared_entries);
        // Attribute containers own references to registered string backings.
        // Dispose containers first; reverse-pop then retains the real Vec until
        // its capacity ticket is consumed.
        for payload in &mut values {
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                // PI maps own the second references to registered backings.
                // Remove those references before string-ticket disposal.
                loop {
                    let popped = if control.stop().is_some() {
                        values.cleanup_pop(control)
                    } else {
                        values.try_pop(control)
                    };
                    let popped = match popped {
                        Ok(popped) => popped,
                        Err(_) => match values.cleanup_pop(control) {
                            Ok(popped) => popped,
                            Err(stop) => {
                                refused.get_or_insert(SessionError::Stop(stop));
                                break;
                            }
                        },
                    };
                    match popped {
                        Some(pair) => drop(pair),
                        None => break,
                    }
                }
            }
            if let RetainedPayload::Attributes(storage) = payload {
                let Some(storage) = storage.take() else {
                    refused.get_or_insert(SessionError::Consumed);
                    continue;
                };
                let count = match &storage {
                    AttributeStorage::Building { values, .. }
                    | AttributeStorage::Transferring { values, .. } => values.len(),
                    AttributeStorage::Frozen { values, .. }
                    | AttributeStorage::Removing { values, .. } => values.len(),
                    AttributeStorage::Unvalidated { backing, .. } => backing.len(),
                };
                match storage {
                    AttributeStorage::Building {
                        values, allocation, ..
                    }
                    | AttributeStorage::Transferring {
                        values, allocation, ..
                    } => {
                        if let Err(error) = release_vector(control, values, allocation) {
                            refused.get_or_insert(error);
                        }
                    }
                    AttributeStorage::Frozen {
                        values,
                        allocation,
                        shared,
                        ..
                    }
                    | AttributeStorage::Removing {
                        values,
                        allocation,
                        shared,
                        ..
                    } => {
                        if shared.finalized() {
                            // A refused bank and tickets are dropped with the
                            // payload; the latched stop reports the skipped credit.
                            keep_first(
                                &mut refused,
                                control
                                    .release_final_attributes(values, allocation, shared)
                                    .map_err(FinalReleaseError::discard),
                            );
                            continue;
                        }
                        let backing = values.into_shared_attributes();
                        match control.release_shared(backing, shared) {
                            Ok(vector) => {
                                if let Err(error) = release_vector(control, vector, allocation) {
                                    refused.get_or_insert(error);
                                }
                            }
                            Err(_) => {
                                refused.get_or_insert(SessionError::AliasedPayload);
                            }
                        }
                    }
                    AttributeStorage::Unvalidated {
                        backing,
                        allocation,
                        shared,
                        ..
                    } => match control.release_shared(backing, shared) {
                        Ok(vector) => {
                            if let Err(error) = release_vector(control, vector, allocation) {
                                refused.get_or_insert(error);
                            }
                        }
                        Err(_) => {
                            refused.get_or_insert(SessionError::AliasedPayload);
                        }
                    },
                }
                keep_first(
                    &mut refused,
                    release_logical(
                        control,
                        owner,
                        HtmlParseResource::ManagedAttributes,
                        count as u64,
                    ),
                );
            }
        }
        while let Some(payload) = values.pop() {
            let associated_usage = match &payload {
                RetainedPayload::AssociatedInputs { usage, .. } => Some(*usage),
                RetainedPayload::Hyperlink { state, .. } => Some(state.usage),
                RetainedPayload::Image { state, .. } => Some(state.usage),
                RetainedPayload::AssociationCapturedContext(value) => {
                    Some(pending_association::captured_usage(value))
                }
                _ => None,
            };
            if let Some(usage) = associated_usage {
                drop(payload);
                keep_first(&mut refused, usage.release(control, owner));
                continue;
            }
            if let RetainedPayload::CspState(value) = payload {
                if let Some(value) = value {
                    csp_storage = value.account.live;
                    drop(value);
                }
                continue;
            }
            if let RetainedPayload::Document { mut meta, .. } = payload {
                if let Some(scheme) = meta.scheme.take() {
                    keep_first(&mut refused, scheme.release(control));
                }
                continue;
            }
            if let RetainedPayload::AssociationOccurrence(value) = payload {
                keep_first(
                    &mut refused,
                    pending_association::release_occurrence(value, control),
                );
                continue;
            }
            if let RetainedPayload::PiAttributes { values, .. } = payload {
                keep_first(&mut refused, values.release(control));
                continue;
            }
            if let RetainedPayload::WorkList(Some(list)) = payload {
                keep_first(&mut refused, list.release(control));
                continue;
            }
            if let RetainedPayload::StyleOrder(list) = payload {
                keep_first(&mut refused, list.release(control));
                continue;
            }
            if let RetainedPayload::InputSource {
                value_attribute, ..
            } = &payload
            {
                keep_first(
                    &mut refused,
                    release_logical(
                        control,
                        owner,
                        HtmlParseResource::SourceMapEntries,
                        if value_attribute.is_some() { 2 } else { 1 },
                    ),
                );
            }
            if let RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } = payload
            {
                drop(value);
                keep_first(&mut refused, control.release_document_string(allocation));
                drop(origins);
            }
        }
        keep_first(&mut refused, csp_storage.release(control));
        keep_first(&mut refused, release_vector(control, values, allocation));
        keep_first(
            &mut refused,
            release_logical(control, owner, HtmlParseResource::AuxiliaryEntries, entries),
        );
        refused.map_or(Ok(()), Err)
    }
}

pub(crate) fn tally_sizes() -> String {
    use std::mem::size_of;
    let mut out = String::new();
    macro_rules! s {
        ($t:ty) => {
            out.push_str(&format!("SIZE {} = {}\n", stringify!($t), size_of::<$t>()));
        };
    }
    s!(RetainedPayload);
    s!(pending_association::DocumentCapture);
    s!(pending_association::CapturedContext);
    s!(dialog::DialogMetadata);
    s!(style::StyleMetadata);
    s!(associated::HyperlinkMetadata);
    s!(associated::ImageMetadata);
    s!(crate::HtmlParseBaseFacts);
    s!(base::BaseValue);
    s!(context::ShadowState);
    s!(HtmlParseDocumentState);
    s!(template::DocumentState);
    s!(meta::MetaHistory);
    s!(dialog::InteractionHistory);
    s!(context::ElementCreationState);
    s!(template::TemplateMetadata);
    s!(SvgScriptState);
    s!(ScriptMetadata);
    s!(FormAssociationState);
    s!(TextareaState);
    s!(OutputState);
    s!(InputState);
    s!(SelectMetadata);
    s!(AttributeStorage);
    s!(csp::State);
    s!(StringLocation);
    s!(DocumentStringAllocation);
    out
}
