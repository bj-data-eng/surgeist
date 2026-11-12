//! Managed string registration, text insertion and merging, and string disposal.

use super::string_storage::AttributeString;
use super::*;
use surgeist_html::{
    bounded::construction::NameId,
    construction::ConstructionNodeKind,
    values::{DocumentString, Namespace},
};

/// A string of a non-element field that a record is about to be created with, and the group of its
/// origins when the parse records them. The record copies the units of the string, so the string
/// is released once the record exists or the creation is refused.
pub(super) struct FieldString {
    pub(super) value: FrozenUtf16,
    pub(super) group: Option<OriginGroupRef>,
}
impl FieldString {
    /// Gives the string back, for a creation that did not use it. The group of its origins stays
    /// in the evidence.
    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.value.release(control)
    }
    /// The units the record is created with.
    pub(super) fn units(&self) -> &[u16] {
        self.value.text().code_units()
    }
}

/// A constant string that one parse creates once and shares between its occurrences.
#[derive(Clone, Copy)]
pub(super) enum Literal {
    HtmlNamespace,
    SvgNamespace,
    MathMlNamespace,
    XlinkNamespace,
    XmlNamespace,
    XmlnsNamespace,
    XlinkPrefix,
    XmlPrefix,
    XmlnsPrefix,
}
pub(super) const LITERALS: usize = 9;
impl Literal {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::HtmlNamespace => b"http://www.w3.org/1999/xhtml",
            Self::SvgNamespace => b"http://www.w3.org/2000/svg",
            Self::MathMlNamespace => b"http://www.w3.org/1998/Math/MathML",
            Self::XlinkNamespace => b"http://www.w3.org/1999/xlink",
            Self::XmlNamespace => b"http://www.w3.org/XML/1998/namespace",
            Self::XmlnsNamespace => b"http://www.w3.org/2000/xmlns/",
            Self::XlinkPrefix => b"xlink",
            Self::XmlPrefix => b"xml",
            Self::XmlnsPrefix => b"xmlns",
        }
    }
    /// The literal that is the namespace `uri`, if it is one.
    pub(super) fn of_uri(uri: &str) -> Option<Self> {
        Self::URIS
            .into_iter()
            .find(|literal| literal.bytes() == uri.as_bytes())
    }
    /// The literal that is the namespace URI of `units`, if it is one.
    pub(super) fn of_uri_units(units: &[u16]) -> Option<Self> {
        Self::URIS.into_iter().find(|literal| literal.is(units))
    }
    /// The literal that is the attribute prefix of `units`, if it is one.
    pub(super) fn of_prefix_units(units: &[u16]) -> Option<Self> {
        Self::PREFIXES.into_iter().find(|literal| literal.is(units))
    }
    fn is(self, units: &[u16]) -> bool {
        let bytes = self.bytes();
        bytes.len() == units.len()
            && bytes
                .iter()
                .zip(units)
                .all(|(&byte, &unit)| u16::from(byte) == unit)
    }
    const URIS: [Self; 6] = [
        Self::HtmlNamespace,
        Self::SvgNamespace,
        Self::MathMlNamespace,
        Self::XlinkNamespace,
        Self::XmlNamespace,
        Self::XmlnsNamespace,
    ];
    const PREFIXES: [Self; 3] = [Self::XlinkPrefix, Self::XmlPrefix, Self::XmlnsPrefix];
    /// The literal that is the attribute prefix `prefix`, if it is one.
    pub(super) fn of_prefix(prefix: &str) -> Option<Self> {
        Self::PREFIXES
            .into_iter()
            .find(|literal| literal.bytes() == prefix.as_bytes())
    }
}

impl SessionStorage {
    /// Registers an occurrence of `literal`. The first one of a parse creates the string and
    /// charges its storage; later ones register the same string with only their units.
    pub(super) fn register_literal(
        &mut self,
        control: &mut ParseControl,
        literal: Literal,
    ) -> Result<(DocumentString, usize), SessionError> {
        self.require(control)?;
        let Some(first) = self.first_literal(control, literal)? else {
            let frozen = literal_string(control, literal.bytes())?;
            let (value, slot) = self.register_string(control, frozen, None)?;
            self.cache_literal(control, literal, slot)?;
            return Ok((value, slot));
        };
        // The origin group is a fact of each occurrence, as for any created string.
        let units = self.literal_len(control, first)?;
        let group = if !control.detail().records_graph() {
            None
        } else if let Some(evidence) = &mut self.evidence {
            Some(evidence.rule_group(control, units)?)
        } else {
            None
        };
        self.register_shared_occurrence(control, first, group)
    }
    /// An occurrence of `literal` as the namespace or prefix of an attribute: the string of the
    /// parse's first registration of it, and the group of this occurrence's origins when the parse
    /// records them. The first one of a parse creates the string and charges its storage.
    pub(super) fn literal_field(
        &mut self,
        control: &mut ParseControl,
        literal: Literal,
    ) -> Result<storage_attributes::LiteralField, SessionError> {
        self.require(control)?;
        let (first, counted) = match self.first_literal(control, literal)? {
            Some(first) => (first, false),
            None => {
                let frozen = literal_string(control, literal.bytes())?;
                let (value, slot) = self.register_string_owned(control, frozen, None)?;
                drop(value);
                self.cache_literal(control, literal, slot)?;
                (slot, true)
            }
        };
        let units = self.literal_len(control, first)?;
        let group = if !control.detail().records_graph() {
            None
        } else if let Some(evidence) = &mut self.evidence {
            Some(evidence.rule_group(control, units)?)
        } else {
            None
        };
        let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(first) else {
            return Err(latch_registry_row(control));
        };
        Ok(storage_attributes::LiteralField {
            string: value.clone(),
            group,
            counted,
        })
    }
    /// As [`Self::register_literal`] for a copy of a registered field whose value is `literal`,
    /// which keeps the origin group the copy made of the field's own.
    pub(super) fn register_literal_copy(
        &mut self,
        control: &mut ParseControl,
        literal: Literal,
        origins: Option<OriginGroupRef>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let Some(first) = self.first_literal(control, literal)? else {
            let frozen = literal_string(control, literal.bytes())?;
            let (value, slot) = self.register_group_string_owned(control, frozen, origins)?;
            self.cache_literal(control, literal, slot)?;
            return Ok((value, slot));
        };
        self.register_shared_occurrence(control, first, origins)
    }
    /// The row of the first registration of `literal` in this parse, if there was one.
    fn first_literal(
        &self,
        control: &mut ParseControl,
        literal: Literal,
    ) -> Result<Option<usize>, SessionError> {
        match self.payloads.literals.get(literal as usize) {
            Some(entry) => Ok(*entry),
            None => Err(latch_registry_row(control)),
        }
    }
    fn literal_len(&self, control: &mut ParseControl, row: usize) -> Result<usize, SessionError> {
        match self.payloads.values.get(row) {
            Some(RetainedPayload::String { value, .. }) => Ok(value.len()),
            _ => Err(latch_registry_row(control)),
        }
    }
    /// Records the row of a literal's first registration, whose ticket is shared from then on.
    fn cache_literal(
        &mut self,
        control: &mut ParseControl,
        literal: Literal,
        slot: usize,
    ) -> Result<(), SessionError> {
        let Some(RetainedPayload::String { allocation, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        allocation.mark_shared();
        let Some(entry) = self.payloads.literals.get_mut(literal as usize) else {
            return Err(latch_registry_row(control));
        };
        *entry = Some(slot);
        Ok(())
    }
    /// Registers a row that shares the string of the first registration at `first`.
    fn register_shared_occurrence(
        &mut self,
        control: &mut ParseControl,
        first: usize,
        group: Option<OriginGroupRef>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(first) else {
            return Err(latch_registry_row(control));
        };
        let value = value.clone();
        let prepared = self.prepare_payload_slot_owned(control)?;
        let allocation = control.share_document_string(&value)?;
        self.push_string_row(control, prepared, value, allocation, group, false)
    }
    pub(super) fn check_text(
        control: &mut ParseControl,
        text: &mut Text,
    ) -> Result<(), SessionError> {
        control.require_document_string(text.value.allocation(), text.value.text())?;
        text.origins.require_owner(control)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if text.value.text().block().holders() != 1 {
            return Err(SessionError::AliasedPayload);
        }
        Ok(())
    }
    pub(super) fn register_string(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        if let Err(error) = self.require(control) {
            value.release(control)?;
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            return Err(error);
        }
        self.register_string_owned(control, value, origins)
    }
    pub(super) fn register_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let group = match self.origin_group(control, origins, value.text().len()) {
            Ok(group) => group,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        self.register_group_string_owned(control, value, group)
    }
    /// The group of the origins of a string of `units` units, when the parse records the origin
    /// graph: the group the retained `origins` make, or a group of no source origins when there
    /// are none. A parse that records no graph holds no group. The origins are released
    /// whatever the result.
    pub(super) fn origin_group(
        &mut self,
        control: &mut ParseControl,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
        units: usize,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        if !control.detail().records_graph() {
            // No group names this value, and no origin was recorded for it.
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            Ok(None)
        } else if let Some(origins) = origins {
            let evidence = match self.origin_evidence(control) {
                Ok(evidence) => evidence,
                Err(error) => {
                    origins.release(control)?;
                    return Err(error);
                }
            };
            evidence.retain_group(control, origins, units).map(Some)
        } else if let Some(evidence) = &mut self.evidence {
            evidence.rule_group(control, units).map(Some)
        } else {
            Ok(None)
        }
    }
    /// The group is committed in the sole evidence owner before the actual
    /// native backing slot is reserved. A refused slot leaves no logical slot
    /// reservation and the original backing is reclaimed by this owner.
    pub(super) fn register_group_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OriginGroupRef>,
    ) -> Result<(DocumentString, usize), SessionError> {
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = value.into_parts();
        self.push_string_row(control, prepared, value, allocation, origins, true)
    }
    /// Appends the row of a string whose ticket is made, and binds its origin group. A string in
    /// the buffered domain moves to the managed one; a ticket of the managed domain is already
    /// there.
    fn push_string_row(
        &mut self,
        control: &mut ParseControl,
        prepared: PreparedSlot,
        value: DocumentString,
        allocation: DocumentStringAllocation,
        origins: Option<OriginGroupRef>,
        buffered: bool,
    ) -> Result<(DocumentString, usize), SessionError> {
        let index =
            self.push_unbound_row(control, prepared, value, allocation, origins, buffered)?;
        let Some(RetainedPayload::String { value, origins, .. }) =
            self.payloads.values.get_mut(index)
        else {
            return Err(latch_registry_row(control));
        };
        if let Some(origins) = origins {
            self.evidence
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .bind_leaf_value(control, origins, index)?;
        }
        Ok((value.clone(), index))
    }
    /// Appends the row of a string whose ticket is made, leaving its origin group unbound, and
    /// returns its slot.
    fn push_unbound_row(
        &mut self,
        control: &mut ParseControl,
        prepared: PreparedSlot,
        value: DocumentString,
        allocation: DocumentStringAllocation,
        origins: Option<OriginGroupRef>,
        buffered: bool,
    ) -> Result<usize, SessionError> {
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
        if buffered {
            let Some(RetainedPayload::String {
                value, allocation, ..
            }) = self.payloads.values.get_mut(index)
            else {
                return Err(latch_registry_row(control));
            };
            control.move_document_string_to_managed(allocation, value)?;
        }
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<DocumentString>() as u64,
        )?;
        Ok(index)
    }
    /// Retires the groups of the `strings` of the attribute at `position` of `node` before the
    /// attribute is replaced or removed, when the parse records the origin graph: each group
    /// leaves the table of origins and is retired in the evidence by a cause that names its
    /// field. A group whose leaf names the attribute's string keeps its history: the leaf is
    /// rebound to a copy of the units in a row of the registry, which the edit does not change.
    pub(super) fn retire_attribute_strings(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        position: usize,
        strings: &[AttributeString],
    ) -> Result<(), SessionError> {
        for &string in strings {
            let field = FieldRef::of_attribute(node, position, string);
            let Some(group) = self.payloads.detail_origins.take(control, field)? else {
                continue;
            };
            self.retire_attribute_group(control, field, group)?;
        }
        Ok(())
    }
    /// Retires `group`, the group of the string of the attribute field `field`, as
    /// [`Self::retire_attribute_strings`] does.
    fn retire_attribute_group(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
        group: OriginGroupRef,
    ) -> Result<(), SessionError> {
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        evidence.retire_group(control, &group, field)?;
        if !evidence.names_attribute_string(control, &group)? {
            return Ok(());
        }
        let mut buffer = Utf16Buffer::new(control)?;
        let copied = (|| {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let Some(units) = field.units(ConstructionView::Active(&self.construction)) else {
                return Err(latch_registry_row(control));
            };
            buffer.extend_units(units, control)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = copied {
            buffer.release(control)?;
            return Err(error);
        }
        let frozen = buffer.freeze(control)?;
        let prepared = match self.prepare_payload_slot_owned(control) {
            Ok(prepared) => prepared,
            Err(error) => {
                frozen.release(control)?;
                return Err(error);
            }
        };
        let (value, allocation) = frozen.into_parts();
        let slot =
            self.push_unbound_row(control, prepared, value, allocation, Some(group), true)?;
        let Some(RetainedPayload::String {
            origins: Some(group),
            ..
        }) = self.payloads.values.get(slot)
        else {
            return Err(latch_registry_row(control));
        };
        self.evidence
            .as_mut()
            .ok_or(SessionError::WrongToken)?
            .rebind_attribute_leaf(control, group, slot)
    }
    pub(super) fn register_text(
        &mut self,
        control: &mut ParseControl,
        text: Text,
    ) -> Result<(DocumentString, usize), SessionError> {
        self.register_string(control, text.value, Some(text.origins))
    }
    /// Interns the name of `namespace`, no prefix and the local name that the string row `slot`
    /// holds, and makes the row hold the storage's string for that name. The row's own string is
    /// freed: the units are hashed once and the storage keeps one allocation for every
    /// occurrence of the name. The row keeps the units of its occurrence and its origin group,
    /// and holds the ticket of an occurrence, which admits only its units. Returns the id of the
    /// name and its local name as the storage holds it.
    ///
    /// The caller holds no other handle on the row's string.
    pub(super) fn intern_row_name(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        namespace: &Namespace,
    ) -> Result<(NameId, DocumentString), SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String { value, .. }) = self.payloads.values.get(slot) else {
            return Err(latch_registry_row(control));
        };
        let units = value.clone();
        let id = self.intern_name(control, namespace, units.code_units())?;
        drop(units);
        let local = match self.construction.name(&id) {
            Ok(name) => name.local_name().clone(),
            Err(_) => return Err(latch_registry_row(control)),
        };
        let Some(RetainedPayload::String {
            value, allocation, ..
        }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        control.rebind_document_string_to_shared(allocation, value, &local)?;
        Ok((id, local))
    }
    /// Implements the pinned character-insertion operation at an already
    /// selected insertion location. Foster-parent selection belongs to the
    /// later tree builder. The returned offset locates the original token text
    /// inside the native text node; retained origins remain exact token facts.
    pub(super) fn insert_text(
        &mut self,
        control: &mut ParseControl,
        parent: &ParserNode,
        before: Option<&ParserNode>,
        input: &mut Option<Text>,
    ) -> Result<Option<TextInsertion>, SessionError> {
        let parent_index = self.index(control, parent)?;
        let before_index = before.map(|node| self.index(control, node)).transpose()?;
        self.container(control, parent_index)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(before) = before_index {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self
                .construction
                .node_at(before)
                .ok_or_else(|| latch_registry_row(control))?
                .parent()
                .map(|node| node.position())
                != Some(parent_index)
            {
                return Err(SessionError::WrongSibling);
            }
        }
        let text = input.as_mut().ok_or(SessionError::Consumed)?;
        Self::check_text(control, text)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if text.value.text().is_empty()
            || matches!(
                self.construction
                    .node_at(parent_index)
                    .ok_or_else(|| latch_registry_row(control))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            )
        {
            let Some(text) = input.take() else {
                return Err(SessionError::Consumed);
            };
            text.release(control)?;
            return Ok(None);
        }
        let mut preceding = None;
        if before_index.is_none() {
            // Appending: the preceding sibling is the last child, which needs
            // no walk over the parent's other children.
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            if let Some(last) = parent.child_count().checked_sub(1) {
                preceding = Some(
                    parent
                        .child_at(last)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        } else {
            // Walk back from the last child to the reference sibling, so the
            // work is the children behind it and not the ones in front of it;
            // the entry just before it is the preceding sibling.
            let parent = self
                .construction
                .node_at(parent_index)
                .ok_or_else(|| latch_registry_row(control))?;
            let mut position = parent.child_count();
            loop {
                let Some(found) = position.checked_sub(1) else {
                    return Err(latch_registry_row(control));
                };
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                let child = parent
                    .child_at(found)
                    .ok_or_else(|| latch_registry_row(control))?
                    .position();
                position = found;
                if Some(child) == before_index {
                    break;
                }
            }
            if let Some(previous) = position.checked_sub(1) {
                control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                preceding = Some(
                    parent
                        .child_at(previous)
                        .ok_or_else(|| latch_registry_row(control))?
                        .position(),
                );
            }
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let preceding = match preceding {
            Some(index) => match self.construction.node_at(index).map(|record| record.kind()) {
                Some(ConstructionNodeKind::Text(_)) => Some(index),
                Some(_) => None,
                None => return Err(latch_registry_row(control)),
            },
            None => None,
        };
        let prepared = if preceding.is_none() {
            let document = match self.node_document_ref(control, parent_index) {
                Ok(document) => Some(document),
                Err(SessionError::NodeDocumentUnavailable)
                    if matches!(
                        self.construction
                            .node_at(0)
                            .ok_or_else(|| latch_registry_row(control))?
                            .kind(),
                        ConstructionNodeKind::Fragment
                    ) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            self.prepare_document_allocation(control, document, false)?
        } else {
            None
        };
        let Some(text) = input.take() else {
            return Err(SessionError::Consumed);
        };
        if let Some(index) = preceding {
            self.merge_text(control, parent_index, index, text, before_index.is_none())
                .map(Some)
        } else {
            let field = self.text_field(control, text)?;
            let created = (|| {
                control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
                self.allocate_prepared_owned(
                    control,
                    NewRecord::Text(field.units()),
                    false,
                    0,
                    field.units().len(),
                    prepared,
                )
            })();
            let FieldString { value, group } = field;
            let released = value.release(control);
            let node = created?;
            released?;
            self.bind_field(control, FieldRef::Text(node.index), group, 0)?;
            self.insert(control, parent, &node, before, InsertionCause::NonParser)?;
            Ok(Some(TextInsertion {
                _node: node,
                _offset: 0,
            }))
        }
    }
    /// Takes the string of a token for a field of a record that is about to be created. The
    /// group of its origins is made when the parse records the origin graph, and
    /// [`Self::bind_field`] binds it to the field once the record exists.
    pub(super) fn text_field(
        &mut self,
        control: &mut ParseControl,
        text: Text,
    ) -> Result<FieldString, SessionError> {
        let Text { value, origins } = text;
        self.field_string(control, value, Some(origins))
    }
    /// As [`Self::text_field`] for a string that is not a token's, such as the empty
    /// identifier of a doctype.
    pub(super) fn field_string(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<FieldString, SessionError> {
        if let Err(error) = self.require(control) {
            value.release(control)?;
            if let Some(origins) = origins {
                origins.release(control)?;
            }
            return Err(error);
        }
        self.field_string_owned(control, value, origins)
    }
    /// As [`Self::field_string`] for a caller that is already inside an operation of the storage
    /// and owns its phase, such as the change of a nonce during an insertion's effects.
    pub(super) fn field_string_owned(
        &mut self,
        control: &mut ParseControl,
        value: FrozenUtf16,
        origins: Option<OwnedVec<HtmlParseTextOrigin>>,
    ) -> Result<FieldString, SessionError> {
        let group = match self.origin_group(control, origins, value.text().len()) {
            Ok(group) => group,
            Err(error) => {
                value.release(control)?;
                return Err(error);
            }
        };
        Ok(FieldString { value, group })
    }
    /// Binds the group of the string of `field`, a non-element field of the record that holds
    /// it, to the field: the leaf of the group names the units `offset..` of the field's string,
    /// and the group becomes the field's in the table of origins. A string without a group
    /// binds nothing. A refusal leaves the record without the group of the field: the parse is
    /// stopped by it and publishes no record.
    pub(super) fn bind_field(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
        group: Option<OriginGroupRef>,
        offset: usize,
    ) -> Result<(), SessionError> {
        let Some(group) = group else {
            return Ok(());
        };
        self.evidence
            .as_mut()
            .ok_or(SessionError::WrongToken)?
            .bind_leaf_field(control, &group, field, offset)?;
        self.payloads.detail_origins.insert(control, field, group)
    }
    pub(super) fn releasable_string(
        &self,
        control: &mut ParseControl,
        slot: usize,
        native: bool,
    ) -> Result<bool, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::String {
            value,
            allocation,
            origins,
            ..
        }) = self.payloads.values.get(slot)
        else {
            return Err(latch_registry_row(control));
        };
        control.require_document_string(allocation, value)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        // The block of an occurrence of a shared string has other holders by definition: the
        // row that charged it and every other occurrence. The ticket that charged the block keeps
        // the check.
        if !allocation.is_occurrence() && value.block().holders() != if native { 2 } else { 1 } {
            return Err(SessionError::AliasedPayload);
        }
        if let Some(group) = origins {
            let evidence = self.evidence.as_ref().ok_or(SessionError::WrongToken)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<provenance::OriginGroup>() as u64,
            )?;
            let group = evidence.require_group(group)?;
            Ok(!matches!(group.body, provenance::GroupBody::Leaf { .. }) || group.units == 0)
        } else {
            Ok(true)
        }
    }
    /// Called only after the admitted native/raw slot commit has removed every
    /// live alias. Leaf backings remain owned; aggregate history retains IDs,
    /// not another materialized prefix. This deallocation never polls useful
    /// work or refunds cumulative work/high-water observations.
    pub(super) fn dispose_aggregate_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let Some(row) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(control));
        };
        let (value, allocation, origins) = match mem::replace(row, RetainedPayload::Retired) {
            RetainedPayload::String {
                value,
                allocation,
                origins,
                ..
            } => (value, allocation, origins),
            // A row of another kind is handed back where it was found.
            other => {
                *row = other;
                return Err(SessionError::WrongToken);
            }
        };
        drop(value);
        control.release_document_string(allocation)?;
        if let Some(origins) = origins {
            *row = RetainedPayload::HistoricalString { origins };
        }
        Ok(())
    }
    /// Retired raw fields may be unique immutable history leaves. Keep those
    /// actual allocations; only reconstructed aggregate backings are redundant.
    pub(super) fn retire_registered_string(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<(), SessionError> {
        let dispose = self.releasable_string(control, slot, false)?;
        if dispose {
            payload_write::<RetainedPayload>(control, 2)?;
        }
        self.retire_field_group(control, slot)?;
        if dispose {
            self.dispose_aggregate_string(control, slot)?;
        }
        Ok(())
    }
    pub(super) fn bind_string(
        &mut self,
        index: usize,
        location: FieldRef,
    ) -> Result<(), SessionError> {
        let Some(RetainedPayload::String {
            location: current, ..
        }) = self.payloads.values.get_mut(index)
        else {
            return Err(SessionError::registry_row());
        };
        // The string of a field of a record is held by its record alone.
        if location.is_held_by_record() {
            return Err(SessionError::registry_row());
        }
        *current = location;
        Ok(())
    }
}
