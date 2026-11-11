//! Allocation of nodes, payload rows and prepared slots in the active session storage.

use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind,
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
};

/// The name and attributes of an element that exists before the parse, as a test hands them to the
/// allocator.
#[cfg(test)]
pub(crate) struct PreexistingElement {
    name: QualifiedName,
    attributes: OrderedAttributes,
}
#[cfg(test)]
impl PreexistingElement {
    pub(crate) fn new(name: QualifiedName, attributes: OrderedAttributes) -> Self {
        Self { name, attributes }
    }
    pub(crate) fn name(&self) -> &QualifiedName {
        &self.name
    }
}

impl SessionStorage {
    /// Consumes preexisting immutable payloads, not tokenizer-created backing.
    #[cfg(test)]
    pub(super) fn allocate_element(
        &mut self,
        control: &mut ParseControl,
        element: PreexistingElement,
    ) -> Result<ParserNode, SessionError> {
        let PreexistingElement { name, attributes } = element;
        self.require(control)?;
        // Actual preexisting input permanently leaves the closed producer
        // inventory. Admission precedes the monotone provenance transition.
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.standalone_ineligibility.is_none() {
            payload_write::<Option<HtmlParseUnsupportedState>>(control, 1)?;
            self.standalone_ineligibility = Some(HtmlParseUnsupportedState::Provenance);
        }
        let template = exact_ascii(
            control,
            name.namespace().uri(),
            "http://www.w3.org/1999/xhtml",
        )? && exact_ascii(control, Some(name.local_name()), "template")?;
        let mut units = name_units(control, &name)?;
        let attributes_len = attributes.len();
        for index in 0..attributes_len {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let attribute = &attributes.as_slice()[index];
            let Some(next) = units
                .checked_add(name_units(control, attribute.name())?)
                .and_then(|sum| sum.checked_add(attribute.value().len()))
            else {
                return control
                    .representation(HtmlParseResource::ManagedUtf16Units)
                    .map_err(Into::into);
            };
            units = next;
        }
        let node = self.allocate_native(
            control,
            NewRecord::Element(name, attributes, None),
            template,
            attributes_len,
            units,
        )?;
        self.payloads.preexisting_units += units;
        self.payloads.preexisting_attributes += attributes_len;
        Ok(node)
    }

    pub(super) fn allocate(
        &mut self,
        control: &mut ParseControl,
        kind: NewRecord,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        self.allocate_owned(control, kind, template, attributes, units)
    }
    pub(super) fn allocate_owned(
        &mut self,
        control: &mut ParseControl,
        kind: NewRecord,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let document = matches!(
            self.construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(control))?
                .kind(),
            ConstructionNodeKind::Document(_)
        )
        .then_some(DocumentRef::Owned(0));
        self.allocate_in_document_owned(control, kind, template, attributes, units, document)
    }
    pub(super) fn allocate_native(
        &mut self,
        control: &mut ParseControl,
        kind: NewRecord,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        self.allocate_native_record(control, kind, template, attributes, units)
    }
    pub(super) fn allocate_native_record(
        &mut self,
        control: &mut ParseControl,
        kind: NewRecord<'_>,
        template: bool,
        attributes: usize,
        units: usize,
    ) -> Result<ParserNode, SessionError> {
        control.require_identity(&self.control)?;
        // A template is a record and its content fragment.
        let (count, entries): (usize, u64) = if template { (2, 2) } else { (1, 1) };
        let cause_slots = self.prepare_node_causes(control, count)?;
        // Each record retains one derived node-join cell; payload metadata rows remain
        // separately admitted.
        let requests = [
            (HtmlParseResource::ManagedNodes, count as u64),
            (HtmlParseResource::ManagedEdges, u64::from(template)),
            (HtmlParseResource::ManagedAttributes, attributes as u64),
            (HtmlParseResource::ManagedUtf16Units, units as u64),
            (HtmlParseResource::AuxiliaryEntries, entries),
        ];
        control.check_acquire_many(&requests)?;
        // Preserve the original node/edge/logical preflight before preparing
        // any new nonce backing; successful elements still bind the slot
        // before their native record can be published.
        let element_name = match &kind {
            NewRecord::Element(name, ..) => Some(name),
            _ => None,
        };
        let nonce_slot = self.prepare_nonce(control, element_name)?;
        if nonce_slot.is_some() {
            payload_write::<Option<usize>>(control, 1)?;
        }
        let index = self.construction.len();
        control.reserve_vector_geometric(
            &mut self.payloads.node_joins,
            &mut self.payloads.node_joins_allocation,
            count,
        )?;
        control.admit_atomic_layout(mem::size_of::<NodeJoins>())?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            count.saturating_mul(mem::size_of::<NodeJoins>()) as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, entries)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<ParserNode>() as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        let handle = self.create_record(control, kind, &requests)?;
        let Ok(position) = self
            .construction
            .node(&handle)
            .map(|record| record.position())
        else {
            return Err(latch_registry_row(control));
        };
        let handle = ParserNode {
            handle,
            index: position,
        };
        for _ in 0..count {
            self.payloads.node_joins.push(NodeJoins::default());
        }
        if let Some(slot) = nonce_slot {
            let Some(RetainedPayload::Nonce { node, .. }) = self.payloads.values.get_mut(slot)
            else {
                return Err(latch_registry_row(control));
            };
            *node = Some(index);
            self.payloads
                .join_row(slot)
                .map_err(|_| latch_registry_row(control))?;
        }
        for (offset, slot) in cause_slots.into_iter().enumerate().take(count) {
            if let Some(slot) = slot {
                let Some(RetainedPayload::ConstructionCause { node, .. }) =
                    self.payloads.values.get_mut(slot)
                else {
                    return Err(latch_registry_row(control));
                };
                *node = Some(
                    index
                        .checked_add(offset)
                        .ok_or_else(|| latch_registry_row(control))?,
                );
            }
        }
        Ok(handle)
    }

    #[cfg_attr(test, track_caller)]
    pub(super) fn prepare_payload_slot(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<PreparedSlot, SessionError> {
        self.require(control)?;
        self.prepare_payload_slot_owned(control)
    }
    // Only the synchronous effects owner may admit a raw replacement while
    // public/session entry points are blocked by the retained effects phase.
    #[cfg_attr(test, track_caller)]
    pub(super) fn prepare_payload_slot_owned(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<PreparedSlot, SessionError> {
        self.prepare_payload_slot_for(control, None)
    }
    /// Admits one registry row and, for a registry-wide kind, the entry of its
    /// list; `push_row` then appends both.
    #[cfg_attr(test, track_caller)]
    pub(super) fn prepare_payload_slot_for(
        &mut self,
        control: &mut ParseControl,
        list: Option<KindList>,
    ) -> Result<PreparedSlot, SessionError> {
        self.prepare_payload_slot_with(control, list, false)
    }
    /// As `prepare_payload_slot_for`, also admitting the entry of the
    /// explicit-form list when the row is of a listed control with a `form`
    /// attribute. All reservations precede the one acquisition.
    #[cfg_attr(test, track_caller)]
    pub(super) fn prepare_payload_slot_with(
        &mut self,
        control: &mut ParseControl,
        list: Option<KindList>,
        explicit_form: bool,
    ) -> Result<PreparedSlot, SessionError> {
        control.require_identity(&self.control)?;
        control.reserve_vector_geometric(
            &mut self.payloads.values,
            &mut self.payloads.allocation,
            1,
        )?;
        if explicit_form {
            if self.payloads.explicit_forms.is_none() {
                let (slots, allocation) = control.new_vector()?;
                self.payloads.explicit_forms = Some(KindSlots { slots, allocation });
            }
            let Some(KindSlots { slots, allocation }) = self.payloads.explicit_forms.as_mut()
            else {
                return Err(latch_registry_row(control));
            };
            control.reserve_vector_geometric(slots, allocation, 1)?;
            control.admit_atomic_layout(mem::size_of::<usize>())?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<usize>() as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        }
        if let Some(list) = list {
            let Some(KindSlots { slots, allocation }) =
                self.payloads.kind_slots.get_mut(list as usize)
            else {
                return Err(latch_registry_row(control));
            };
            control.reserve_vector_geometric(slots, allocation, 1)?;
            control.admit_atomic_layout(mem::size_of::<usize>())?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<usize>() as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        }
        control.admit_atomic_layout(mem::size_of::<RetainedPayload>())?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<RetainedPayload>() as u64,
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        let entries = 1u64
            .saturating_add(u64::from(list.is_some()))
            .saturating_add(u64::from(explicit_form));
        let prepared_entries = self
            .payloads
            .prepared_entries
            .checked_add(entries)
            .ok_or_else(|| latch_registry_row(control))?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, entries)?;
        self.payloads.prepared_entries = prepared_entries;
        #[cfg(test)]
        control.note_prepared_slot(std::panic::Location::caller());
        Ok(PreparedSlot {
            entries,
            list,
            explicit_form,
        })
    }
    pub(super) fn register_raw(
        &mut self,
        control: &mut ParseControl,
        raw: FrozenUtf16,
    ) -> Result<usize, SessionError> {
        let origins = if let Some(evidence) = self.graph() {
            match evidence.rule_group(control, raw.text().len()) {
                Ok(group) => Some(group),
                Err(error) => {
                    raw.release(control)?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                raw.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = raw.into_parts();
        let index = self
            .payloads
            .push_row(
                prepared,
                RetainedPayload::String {
                    value,
                    allocation,
                    origins,
                    location: FieldRef::Detached,
                },
            )
            .map_err(|_| latch_registry_row(control))?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(latch_registry_row(control));
        };
        control.move_document_string_to_managed(allocation, value)?;
        if let Some(origins) = origins {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, origins, index)?;
        }
        Ok(index)
    }

    /// Actual blank creation followed by source-order native attribute appends.
    pub(super) fn allocate_ordered_payload(
        &mut self,
        control: &mut ParseControl,
        name: Text,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
        prepared: Option<template::PreparedDocuments>,
        creation: OrderedCreation,
    ) -> Result<ParserNode, SessionError> {
        let result = (|| {
            if let Err(error) =
                payload_write::<OwnedVec<crate::parser::tokenizer::token::Attribute>>(control, 1)
            {
                name.release(control)?;
                return Err(error);
            }
            let empty = match OwnedVec::new(HtmlParseResource::TokenAttributes) {
                Ok(value) => value,
                Err(stop) => {
                    name.release(control)?;
                    return Err(stop.into());
                }
            };
            let node = self.allocate_html_payload_prepared(
                control,
                name.value,
                Some(name.origins),
                empty,
                prepared,
            )?;
            match creation {
                OrderedCreation::Details { metadata } => {
                    payload_write::<Option<usize>>(control, 1)?;
                    let Some(RetainedPayload::Details { node: bound, .. }) =
                        self.payloads.values.get_mut(metadata)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(metadata)
                        .map_err(|_| latch_registry_row(control))?;
                }
                OrderedCreation::Input {
                    input,
                    association,
                    source,
                } => {
                    payload_write::<[Option<usize>; 3]>(control, 3)?;
                    let Some(RetainedPayload::InputSource { node: bound, .. }) =
                        self.payloads.values.get_mut(source)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    let Some(RetainedPayload::Input { node: bound, .. }) =
                        self.payloads.values.get_mut(input)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(input)
                        .map_err(|_| latch_registry_row(control))?;
                    let Some(RetainedPayload::FormAssociation { node: bound, .. }) =
                        self.payloads.values.get_mut(association)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(association)
                        .map_err(|_| latch_registry_row(control))?;
                }
                OrderedCreation::Select {
                    metadata: Some(slot),
                } => {
                    payload_write::<Option<usize>>(control, 1)?;
                    let Some(RetainedPayload::SelectFamily { node: bound, .. }) =
                        self.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *bound = Some(node.index);
                    self.payloads
                        .join_row(slot)
                        .map_err(|_| latch_registry_row(control))?;
                }
                OrderedCreation::Select { metadata: None } => {}
            }
            let count = attributes.len();
            if count >= NAME_INDEX_MIN_ENTRIES {
                // The attributes are installed one at a time, each in its own edit: they look
                // up their names in one table, which the effects of an attribute may add to.
                self.holds
                    .provide_name_index(control, count.saturating_add(NAME_INDEX_SPARE_ENTRIES))?;
            }
            let mut stage = StagedAttributes::new(control)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let first_row = self.payloads.values.len();
            let installed = (|| {
                let mut remaining = count;
                let mut last_name_slot = None;
                while let Some(attribute) = attributes.try_pop(control)? {
                    let (local, local_ticket) = match self.register_text(control, attribute.name) {
                        Ok(value) => value,
                        Err(error) => {
                            attribute.value.release(control)?;
                            return Err(error);
                        }
                    };
                    last_name_slot.get_or_insert(local_ticket);
                    let (value, value_ticket) = self.register_text(control, attribute.value)?;
                    self.holds.acquire_attribute(control)?;
                    remaining = remaining
                        .checked_sub(1)
                        .ok_or_else(|| latch_registry_row(control))?;
                    stage.push(
                        control,
                        Attribute::new(QualifiedName::new(Namespace::none(), None, local), value),
                        count,
                    )?;
                    self.bind_string(local_ticket, FieldRef::AttributeName(node.index, remaining))
                        .map_err(|_| latch_registry_row(control))?;
                    self.bind_string(
                        value_ticket,
                        FieldRef::AttributeValue(node.index, remaining),
                    )
                    .map_err(|_| latch_registry_row(control))?;
                }
                self.reserve_attributes(control, node.index, count)?;
                self.payloads.forget_attribute_strings(node.index);
                for position in 0..count {
                    self.append_staged_attribute(control, node.index, &mut stage, None)?;
                    if self.effects.form.is_some() {
                        self.drain_form_effects(control)?;
                    }
                    self.initial_nonce_attribute(control, node.index, position)?;
                    match creation {
                        OrderedCreation::Details { .. } => {
                            self.initial_details_attribute(control, node.index, position)?
                        }
                        OrderedCreation::Input {
                            input, association, ..
                        } => self.initial_input_attribute(
                            control,
                            node.index,
                            input,
                            association,
                            position,
                            InputAttributeInitialization::Installed,
                        )?,
                        OrderedCreation::Select { .. } => {
                            self.initial_select_attribute(control, node.index, position)?
                        }
                    }
                }
                Ok(last_name_slot)
            })();
            let released = stage.release(control);
            let last_name_slot = installed?;
            released?;
            self.flush_attribute_origins(control, first_row)?;
            self.holds.retire_name_index(control)?;
            // The strings are registered last attribute first, as in the unordered
            // creation, once the edits that forget the layout are done.
            if let Some(slot) = last_name_slot {
                self.payloads
                    .record_attribute_strings(node.index, slot)
                    .map_err(|_| latch_registry_row(control))?;
            }
            match creation {
                OrderedCreation::Details { .. } => {}
                OrderedCreation::Input { input, .. } => {
                    self.reset_input(control, node.index, input)?
                }
                OrderedCreation::Select { .. } => self.initial_select_reset(control, node.index)?,
            }
            Ok(node)
        })();
        let cleanup = release_token_attributes(attributes, control);
        result.and_then(|node| cleanup.map(|()| node))
    }
    pub(super) fn allocate_html_tag(
        &mut self,
        control: &mut ParseControl,
        input: &mut Option<Tag>,
    ) -> Result<ParserNode, SessionError> {
        self.require(control)?;
        let tag = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_html_tag(control, tag)?;
        let Some(Tag {
            name, attributes, ..
        }) = input.take()
        else {
            return Err(SessionError::Consumed);
        };
        self.allocate_html_payload(control, name.value, Some(name.origins), attributes)
    }
    /// The existing generic borrowed validation, shared without transferring
    /// fields or reserving parser-created metadata.
    pub(super) fn check_html_tag(
        control: &mut ParseControl,
        tag: &mut Tag,
    ) -> Result<(), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if tag.kind != TagKind::Start {
            return Err(SessionError::WrongToken);
        }
        Self::check_text(control, &mut tag.name)?;
        lowercase_token_name(control, tag.name.value.text())?;
        tag.attributes.require_owner(control)?;
        for index in 0..tag.attributes.len() {
            let Some(attribute) = tag.attributes.get_mut(index, control)? else {
                return Err(latch_registry_row(control));
            };
            Self::check_text(control, &mut attribute.name)?;
            Self::check_text(control, &mut attribute.value)?;
            lowercase_token_name(control, attribute.name.value.text())?;
        }
        Ok(())
    }
    /// Shared real storage transfer for explicit and source-implied elements.
    /// An implied name has no fabricated token or source-origin vector.
    pub(super) fn allocate_html_payload(
        &mut self,
        control: &mut ParseControl,
        name: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
        attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
    ) -> Result<ParserNode, SessionError> {
        let preparation = (|| {
            let template = exact_ascii(control, Some(name.text()), "template")?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let document = matches!(
                self.construction
                    .node_at(0)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            )
            .then_some(DocumentRef::Owned(0));
            let ordinary_document = self.fragment.is_none() && document.is_some();
            let mut prepared = self.prepare_document_allocation(control, document, template)?;
            if ordinary_document {
                let mut is_value = None;
                for attribute in attributes.as_slice() {
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if exact_ascii(control, Some(attribute.name.value.text()), "is")? {
                        is_value = Some(&attribute.value);
                        break;
                    }
                }
                // This lexical helper creates in the actual ordinary root
                // Document. Its original token, not later merged attributes,
                // supplies the independent saved-is creation value.
                self.prepare_creation_registry(
                    control,
                    0,
                    name.text(),
                    is_value,
                    true,
                    &mut prepared,
                )?;
            }
            Ok(prepared)
        })();
        let prepared = match preparation {
            Ok(p) => p,
            Err(error) => {
                name.release(control)?;
                if let Some(origins) = origins {
                    origins.release(control)?;
                }
                release_token_attributes(attributes, control)?;
                return Err(error);
            }
        };
        self.allocate_html_payload_prepared(control, name, origins, attributes, prepared)
    }
    pub(super) fn allocate_html_payload_prepared(
        &mut self,
        control: &mut ParseControl,
        name: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
        mut attributes: OwnedVec<crate::parser::tokenizer::token::Attribute>,
        prepared: Option<template::PreparedDocuments>,
    ) -> Result<ParserNode, SessionError> {
        let result = (|| {
            let (name, name_ticket) = self.register_string(control, name, origins)?;
            // The selected HTML namespace is a defining literal, copied through
            // the same admitted exact buffer as token-created units.
            let (namespace, namespace_ticket) =
                self.register_literal(control, Literal::HtmlNamespace)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let mut units = name
                .len()
                .checked_add(namespace.len())
                .ok_or_else(|| control.stop_representation(HtmlParseResource::ManagedUtf16Units))?;
            let mut stage = StagedAttributes::new(control)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let original_len = attributes.len();
            // The record this creation makes is the next one, and names its attributes' strings.
            let node_index = self.construction.len();
            let first_row = self.payloads.values.len();
            // Names and values register as adjacent rows, last attribute first.
            let mut last_name_slot = None;
            let staged = (|| {
                while let Some(attribute) = attributes.try_pop(control)? {
                    let attribute_name = self.register_text(control, attribute.name);
                    let (local, local_ticket) = match attribute_name {
                        Ok(value) => value,
                        Err(error) => {
                            attribute.value.release(control)?;
                            return Err(error);
                        }
                    };
                    last_name_slot.get_or_insert(local_ticket);
                    let (value, value_ticket) = self.register_text(control, attribute.value)?;
                    control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                    units = units
                        .checked_add(local.len())
                        .and_then(|sum| sum.checked_add(value.len()))
                        .ok_or_else(|| {
                            control.stop_representation(HtmlParseResource::ManagedUtf16Units)
                        })?;
                    self.holds.acquire_attribute(control)?;
                    stage.push_plain(
                        control,
                        Attribute::new(QualifiedName::new(Namespace::none(), None, local), value),
                        original_len,
                    )?;
                    let position = attributes.len();
                    self.bind_string(local_ticket, FieldRef::AttributeName(node_index, position))
                        .map_err(|_| latch_registry_row(control))?;
                    self.bind_string(value_ticket, FieldRef::AttributeValue(node_index, position))
                        .map_err(|_| latch_registry_row(control))?;
                }
                stage.reverse(control)
            })();
            if let Err(error) = staged {
                stage.release(control)?;
                return Err(error);
            }
            let (values, allocation) = stage.into_parts();
            let (backing, shared) = match control.share_owned(values) {
                Ok(value) => value,
                Err((stop, values)) => {
                    release_vector(control, values, allocation)?;
                    return Err(stop.into());
                }
            };
            let ordered = match validate_parser_attributes(backing, control) {
                Ok(values) => values,
                Err((error, backing)) => {
                    // Preserve the admitted shared wrapper even when intrinsic duplicate
                    // checking or work stops.
                    return match control.release_shared(backing, shared) {
                        Ok(values) => {
                            release_vector(control, values, allocation)?;
                            Err(error)
                        }
                        Err(refused) => {
                            // Unpublished backing is uniquely owned, so a refusal is an
                            // invariant stop. The accounting of the refused list stays with
                            // the owner for its release.
                            let (_, backing, shared) = refused.into_parts();
                            drop(backing);
                            self.holds
                                .absorb_attribute_list(control, allocation, shared)?;
                            Err(latch_registry_row(control))
                        }
                    };
                }
            };
            // The list belongs to the record from here on, and the parse accounts for it with
            // the other lists of its storage.
            self.holds
                .absorb_attribute_list(control, allocation, shared)?;
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                mem::size_of::<OrderedAttributes>() as u64,
            )?;
            let attributes = ordered;
            let template = exact_ascii(control, Some(&name), "template")?;
            let link_flags = exact_ascii(control, Some(&name), "link")?
                .then(surgeist_html::elements::link::HtmlLinkFlags::fresh);
            control.charge(HtmlParseWorkCategory::RecordWrite, 6)?;
            let node = self.allocate_prepared_owned(
                control,
                NewRecord::Element(
                    QualifiedName::new(Namespace::from_uri(namespace), None, name),
                    attributes,
                    link_flags,
                ),
                template,
                0,
                0,
                prepared,
            )?;
            if node.index != node_index {
                return Err(latch_registry_row(control));
            }
            self.bind_string(name_ticket, FieldRef::ElementName(node.index))
                .map_err(|_| latch_registry_row(control))?;
            self.bind_string(namespace_ticket, FieldRef::Namespace(node.index))
                .map_err(|_| latch_registry_row(control))?;
            if let Some(slot) = last_name_slot {
                self.payloads
                    .record_attribute_strings(node.index, slot)
                    .map_err(|_| latch_registry_row(control))?;
            }
            self.flush_attribute_origins(control, first_row)?;
            self.initial_nonce_attributes(control, node.index)?;
            self.initial_associated_state(control, node.index)?;
            self.initial_meta_attributes(control, node.index)?;
            self.association_initial_attributes(control, node.index)?;
            self.register_base(control, node.index, base::ResponseDomain::Unavailable)?;
            self.record_event_handler_starts(control, node.index, 0)?;
            Ok(node)
        })();
        release_token_attributes(attributes, control)?;
        result
    }
}
