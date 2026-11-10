//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::clone::same_location;
use super::*;
use surgeist_html::{
    construction::ConstructionNodeKind,
    values::{Attribute, OrderedAttributes},
};

#[cfg(test)]
#[path = "mutation_managed.rs"]
mod managed;
#[cfg(test)]
pub(in crate::construction) use managed::ManagedFamily;

impl SessionStorage {
    // Actual one-attribute stage shared by lexical and generated writeback.
    // Preserve the original admission, transfer and native commit order.
    pub(super) fn append_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [name_slot, value_slot] = fields;
        let (values, allocation) = c.new_vector()?;
        let admission = match self.prepare_payload_slot(c) {
            Ok(admission) => admission,
            Err(error) => {
                release_vector(c, values, allocation)?;
                drop(incoming);
                return Err(error);
            }
        };
        let stage = self
            .payloads
            .push_row(
                admission,
                RetainedPayload::Attributes(Some(AttributeStorage::Building {
                    values,
                    allocation,
                    original_len: 1,
                    reversed_pairs: 0,
                })),
            )
            .map_err(|_| latch_registry_row(c))?;
        let prepared = (|| {
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                allocation,
                ..
            }))) = self.payloads.values.get_mut(stage)
            else {
                return Err(latch_registry_row(c));
            };
            c.reserve_vector(values, allocation, 1)?;
            payload_write::<Attribute>(c, 1)?;
            payload_write::<[StringLocation; 2]>(c, 2)?;
            c.acquire(HtmlParseResource::ManagedAttributes, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = prepared {
            drop(incoming);
            return Err(error);
        }
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. }))) =
            self.payloads.values.get_mut(stage)
        else {
            return Err(latch_registry_row(c));
        };
        values.push(incoming);
        self.bind_string(name_slot, StringLocation::AttributeName(stage, 0))
            .map_err(|_| latch_registry_row(c))?;
        self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0))
            .map_err(|_| latch_registry_row(c))?;
        self.edit_attributes(c, node, container, stage, 1, false)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let start = element_record(&self.construction, c, node)?
            .attributes()
            .len();
        self.begin_attribute_transfer(c, stage, container, start)?;
        self.finish_attribute_merge(
            c,
            PreparedAttributeMerge {
                target: node,
                container,
                stage,
                start,
                count: 1,
                earlier: [Some(name_slot), Some(value_slot), None, None],
            },
        )?;
        Ok(())
    }
    /// The registry rows of the name and value of the attribute at `position`
    /// of `node`, found from the layout creation registered instead of by a
    /// scan of the registry, or `None` when the layout cannot be relied on.
    /// The attribute must have no namespace and no prefix, which are strings of
    /// their own, and each candidate row is used only when its location names
    /// this attribute, the location a scan matches, so a hint that an edit made
    /// stale falls back to the scan and finds the same rows.
    fn hinted_attribute_rows(
        &self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        position: usize,
    ) -> Result<Option<[usize; 2]>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let attributes = element_record(&self.construction, c, node)?.attributes();
        let Some(attribute) = attributes.get(position) else {
            return Ok(None);
        };
        if attribute.name().namespace().uri().is_some() || attribute.name().prefix().is_some() {
            return Ok(None);
        }
        let Some(value) = self
            .payloads
            .attribute_value_candidate(node, attributes.len(), position)
        else {
            return Ok(None);
        };
        let Some(name) = value.checked_sub(1) else {
            return Ok(None);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        if self.row_located(name, StringLocation::AttributeName(container, position))
            && self.row_located(value, StringLocation::AttributeValue(container, position))
        {
            Ok(Some([name, value]))
        } else {
            Ok(None)
        }
    }
    pub(super) fn replace_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        location: [usize; 2],
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [container, position] = location;
        let [name_slot, value_slot] = fields;
        payload_write::<Option<Attribute>>(c, 1)?;
        let mut incoming = Some(incoming);
        let replace = (|| {
            // All fixed registry progress is admitted before the generic ready
            // boundary; old source history retires while that owner is exclusive.
            payload_write::<[StringLocation; 6]>(c, 6)?;
            let mut old_slots = [None; 4];
            if let Some([name, value]) = self.hinted_attribute_rows(c, node, container, position)? {
                for (which, slot) in [(0, name), (1, value)] {
                    payload_write::<Option<usize>>(c, 1)?;
                    if let Some(old) = old_slots.get_mut(which) {
                        *old = Some(slot);
                    }
                }
                #[cfg(test)]
                assert_eq!(
                    old_slots,
                    self.scanned_attribute_rows(container, position),
                    "a verified hint names the rows a scan finds"
                );
            } else {
                for (slot, payload) in self.payloads.values.iter().enumerate() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let RetainedPayload::String { location, .. } = payload {
                        let which = match *location {
                            StringLocation::AttributeName(o, p)
                                if o == container && p == position =>
                            {
                                Some(0)
                            }
                            StringLocation::AttributeValue(o, p)
                                if o == container && p == position =>
                            {
                                Some(1)
                            }
                            StringLocation::AttributeNamespace(o, p)
                                if o == container && p == position =>
                            {
                                Some(2)
                            }
                            StringLocation::AttributePrefix(o, p)
                                if o == container && p == position =>
                            {
                                Some(3)
                            }
                            _ => None,
                        };
                        if let Some(which) = which {
                            payload_write::<Option<usize>>(c, 1)?;
                            let Some(old) = old_slots.get_mut(which) else {
                                return Err(latch_registry_row(c));
                            };
                            *old = Some(slot);
                        }
                    }
                }
            }
            payload_write::<Option<AttributeStorage>>(c, 2)?;
            payload_write::<OrderedAttributes>(c, 1)?;
            c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
            let old = self.replace_attribute_storage(
                c,
                node,
                container,
                position,
                &mut incoming,
                &old_slots,
            )?;
            for slot in old_slots.into_iter().flatten() {
                self.bind_string(slot, StringLocation::Detached)
                    .map_err(|_| latch_registry_row(c))?;
            }
            self.bind_string(
                name_slot,
                StringLocation::AttributeName(container, position),
            )
            .map_err(|_| latch_registry_row(c))?;
            self.bind_string(
                value_slot,
                StringLocation::AttributeValue(container, position),
            )
            .map_err(|_| latch_registry_row(c))?;

            drop(old);
            Ok::<_, SessionError>(())
        })();
        drop(incoming);
        replace?;
        self.dialog_attribute_at(c, node, position, true)?;
        self.association_attribute_hook(c, node, position)?;
        Ok(())
    }
    /// Where the name rows of the attributes from `index` on sit, as the slot of
    /// the last attribute's name, when creation's layout still holds for all of
    /// them: each name and value row is verified by its location, and none of
    /// the attributes has a namespace or prefix row. `None` sends the removal to
    /// the registry scan, which finds the same rows.
    fn removal_layout(
        &self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        index: usize,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let attributes = element_record(&self.construction, c, node)?.attributes();
        let count = attributes.len();
        let Some(last) = count.checked_sub(1) else {
            return Ok(None);
        };
        let Some(last_name) = self
            .payloads
            .attribute_value_candidate(node, count, last)
            .and_then(|value| value.checked_sub(1))
        else {
            return Ok(None);
        };
        for position in index..count {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            let Some(attribute) = attributes.get(position) else {
                return Ok(None);
            };
            if attribute.name().namespace().uri().is_some() || attribute.name().prefix().is_some() {
                return Ok(None);
            }
            let Some([name, value]) = attribute_row_slots(last_name, count, position) else {
                return Ok(None);
            };
            if !(self.row_located(name, StringLocation::AttributeName(container, position))
                && self.row_located(value, StringLocation::AttributeValue(container, position)))
            {
                return Ok(None);
            }
        }
        #[cfg(test)]
        self.assert_verified_layout(container, index, count, last_name);
        Ok(Some(last_name))
    }
    fn row_located(&self, slot: usize, expected: StringLocation) -> bool {
        matches!(
            self.payloads.values.get(slot),
            Some(RetainedPayload::String { location, .. }) if same_location(*location, expected)
        )
    }
    /// Rebinds the string row at `slot`, if it belongs to the attributes of
    /// `container`, for the removal of the attribute at `index`, and records
    /// that the registry up to it has been walked.
    fn rebind_removed_attribute_row(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        container: usize,
        index: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<StringLocation>(c, 1)?;
        payload_write::<usize>(c, 1)?;
        if let Some(RetainedPayload::String { location, .. }) = self.payloads.values.get_mut(slot) {
            // The constructor rebuilds the same location kind at its new
            // position, so no second match can disagree with the first.
            let (owner, position, rebuild): (usize, usize, fn(usize, usize) -> StringLocation) =
                match *location {
                    StringLocation::AttributeName(o, p) => (o, p, StringLocation::AttributeName),
                    StringLocation::AttributeValue(o, p) => (o, p, StringLocation::AttributeValue),
                    StringLocation::AttributeNamespace(o, p) => {
                        (o, p, StringLocation::AttributeNamespace)
                    }
                    StringLocation::AttributePrefix(o, p) => {
                        (o, p, StringLocation::AttributePrefix)
                    }
                    _ => (usize::MAX, 0, StringLocation::AttributeName),
                };
            if owner == container {
                *location = if position == index {
                    StringLocation::Detached
                } else {
                    rebuild(
                        owner,
                        position.saturating_sub(usize::from(position > index)),
                    )
                };
            }
        }
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Removing { rebound, .. }))) =
            self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        *rebound = slot.saturating_add(1);
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        let container = self.element_attribute_container(c, node)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let mut selected = None;
        for (index, attr) in element.attributes().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                let style_effect = surgeist_html::vocabulary::KnownHtmlAttribute::from_name(
                    attr.name(),
                )
                .map(|attribute| {
                    surgeist_html::elements::style::style_attribute_effect(
                        attr.name().namespace(),
                        attribute,
                    )
                });
                selected = Some((index, style_effect));
                break;
            }
        }
        let Some((index, style_effect)) = selected else {
            return Ok(());
        };
        let layout = self.removal_layout(c, node, container, index)?;
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
        };
        let len = values.len();
        *storage = Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            index,
            shifted: 0,
            popped: false,
            rebound: 0,
        });
        // Historical destinations stay at the original slot. Retire their
        // original groups before any location is rebound, preserving the cause.
        if let Some(last_name) = layout {
            for slot in
                attribute_row_slots(last_name, len, index).ok_or_else(|| latch_registry_row(c))?
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                self.retire_field_group(c, slot)?;
            }
        } else {
            for slot in 0..self.payloads.values.len() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if matches!(self.payloads.row(c, slot)?, RetainedPayload::String { location: StringLocation::AttributeName(owner, position) | StringLocation::AttributeValue(owner, position) | StringLocation::AttributeNamespace(owner, position) | StringLocation::AttributePrefix(owner, position), .. } if *owner == container && *position == index)
                {
                    self.retire_field_group(c, slot)?;
                }
            }
        }
        let Some(last) = len.checked_sub(1) else {
            return Err(latch_registry_row(c));
        };
        for position in index..last {
            self.edit_removing_attribute(c, node, container, Some(position))?;
        }
        self.edit_removing_attribute(c, node, container, None)?;
        if let Some(last_name) = layout {
            // The rows that can change are those of the attributes from `index`
            // on, and creation registered the last attribute first, so their
            // registry order is by descending position.
            for position in (index..len).rev() {
                for slot in attribute_row_slots(last_name, len, position)
                    .ok_or_else(|| latch_registry_row(c))?
                {
                    self.rebind_removed_attribute_row(c, slot, container, index)?;
                }
            }
        } else {
            for slot in 0..self.payloads.values.len() {
                self.rebind_removed_attribute_row(c, slot, container, index)?;
            }
        }
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        payload_write::<usize>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            ..
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
        };
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        if local == "open" {
            self.dialog_open_changed(c, node, true, false)?;
        }
        if style_effect
            == Some(surgeist_html::elements::style::HtmlStyleAttributeEffect::OwnerAttributes)
            && self.html_local(c, node, "style")?
        {
            self.record_association(
                c,
                node,
                pending_association::HtmlParseAssociationOccurrenceKind::StyleOwnerAttributes,
                None,
                None,
                false,
            )?;
        }
        if base::is_base(
            self.construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?,
        ) && local == "href"
        {
            self.base_change(c, node, None, true, None)?;
        }
        Ok(())
    }
}
/// The registry slots of the name and value of the attribute at `position` of
/// an element of `count` attributes, as creation registered them: the last
/// attribute's name at `last_name`, its value after it, then the previous
/// attribute's name and value, and so on.
fn attribute_row_slots(last_name: usize, count: usize, position: usize) -> Option<[usize; 2]> {
    let name = last_name.checked_add(
        count
            .checked_sub(1)?
            .checked_sub(position)?
            .checked_mul(2)?,
    )?;
    Some([name, name.checked_add(1)?])
}
