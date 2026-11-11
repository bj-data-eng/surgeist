//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::clone::same_location;
use super::*;
use surgeist_html::{construction::ConstructionNodeKind, values::Attribute};

#[cfg(test)]
#[path = "mutation_managed.rs"]
mod managed;
#[cfg(test)]
pub(in crate::construction) use managed::ManagedFamily;

impl SessionStorage {
    // Actual one-attribute append shared by lexical and generated writeback.
    // Preserve the original admission, transfer and native commit order.
    pub(super) fn append_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [name_slot, value_slot] = fields;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let position = element_record(&self.construction, c, node)?
            .attributes()
            .len();
        payload_write::<Attribute>(c, 1)?;
        payload_write::<[FieldRef; 2]>(c, 2)?;
        self.holds.acquire_attribute(c)?;
        self.payloads.forget_attribute_strings(node);
        self.bind_string(name_slot, FieldRef::AttributeName(node, position))
            .map_err(|_| latch_registry_row(c))?;
        self.bind_string(value_slot, FieldRef::AttributeValue(node, position))
            .map_err(|_| latch_registry_row(c))?;
        let mut incoming = Some(incoming);
        self.append_attribute_with_effects(c, node, &mut incoming, None)?;
        if self.effects.form.is_some() {
            self.drain_form_effects(c)?;
        }
        self.flush_attribute_origins(c, name_slot.min(value_slot))?;
        self.attributes_installed(c, node, position, 1)
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
        if self.row_located(name, FieldRef::AttributeName(node, position))
            && self.row_located(value, FieldRef::AttributeValue(node, position))
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
        position: usize,
        incoming: Attribute,
        fields: [usize; 2],
    ) -> Result<(), SessionError> {
        let [name_slot, value_slot] = fields;
        payload_write::<Option<Attribute>>(c, 1)?;
        let mut incoming = Some(incoming);
        let replace = (|| {
            // All fixed registry progress is admitted before the edit; old source history
            // retires before the edit replaces the attribute.
            payload_write::<[FieldRef; 6]>(c, 6)?;
            let mut old_slots = [None; 4];
            if let Some([name, value]) = self.hinted_attribute_rows(c, node, position)? {
                for (which, slot) in [(0, name), (1, value)] {
                    payload_write::<Option<usize>>(c, 1)?;
                    if let Some(old) = old_slots.get_mut(which) {
                        *old = Some(slot);
                    }
                }
                #[cfg(test)]
                assert_eq!(
                    old_slots,
                    self.scanned_attribute_rows(node, position),
                    "a verified hint names the rows a scan finds"
                );
            } else {
                for (slot, payload) in self.payloads.values.iter().enumerate() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    if let RetainedPayload::String { location, .. } = payload {
                        let which = match *location {
                            FieldRef::AttributeName(o, p) if o == node && p == position => Some(0),
                            FieldRef::AttributeValue(o, p) if o == node && p == position => Some(1),
                            FieldRef::AttributeNamespace(o, p) if o == node && p == position => {
                                Some(2)
                            }
                            FieldRef::AttributePrefix(o, p) if o == node && p == position => {
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
            c.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
            for slot in old_slots.iter().flatten() {
                self.retire_field_group(c, *slot)?;
            }
            self.payloads.forget_attribute_strings(node);
            let old = self.replace_attribute(c, node, position, &mut incoming)?;
            for slot in old_slots.into_iter().flatten() {
                self.detach_attribute_string(c, slot)?;
            }
            self.bind_string(name_slot, FieldRef::AttributeName(node, position))
                .map_err(|_| latch_registry_row(c))?;
            self.bind_string(value_slot, FieldRef::AttributeValue(node, position))
                .map_err(|_| latch_registry_row(c))?;
            self.flush_attribute_origins(c, name_slot.min(value_slot))?;
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
            if !(self.row_located(name, FieldRef::AttributeName(node, position))
                && self.row_located(value, FieldRef::AttributeValue(node, position)))
            {
                return Ok(None);
            }
        }
        #[cfg(test)]
        self.assert_verified_layout(node, index, count, last_name);
        Ok(Some(last_name))
    }
    fn row_located(&self, slot: usize, expected: FieldRef) -> bool {
        matches!(
            self.payloads.values.get(slot),
            Some(RetainedPayload::String { location, .. }) if same_location(*location, expected)
        )
    }
    /// Rebinds the string row at `slot`, if it belongs to the attributes of
    /// `node` and follows the attribute removed at `index`, to the position
    /// it has now. The group of such a field stays in the table of attribute
    /// origins, whose keys follow in one step.
    fn rebind_removed_attribute_row(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        node: usize,
        index: usize,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<FieldRef>(c, 1)?;
        if let Some(RetainedPayload::String { location, .. }) = self.payloads.values.get_mut(slot)
            && let Some((owner, position, string)) = location.attribute()
            && owner == node
            && position > index
        {
            let lower = position
                .checked_sub(1)
                .ok_or_else(|| latch_registry_row(c))?;
            *location = FieldRef::of_attribute(node, lower, string);
        }
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
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
        let len = element.attributes().len();
        let layout = self.removal_layout(c, node, index)?;
        // Historical destinations stay at the original slot. Retire their original groups before
        // the attribute goes, preserving the cause.
        let mut removed_rows = [None; 4];
        if let Some(last_name) = layout {
            for (which, slot) in attribute_row_slots(last_name, len, index)
                .ok_or_else(|| latch_registry_row(c))?
                .into_iter()
                .enumerate()
            {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                self.retire_field_group(c, slot)?;
                *removed_rows
                    .get_mut(which)
                    .ok_or_else(|| latch_registry_row(c))? = Some(slot);
            }
        } else {
            let mut which = 0;
            for slot in 0..self.payloads.values.len() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if matches!(self.payloads.row(c, slot)?, RetainedPayload::String { location: FieldRef::AttributeName(owner, position) | FieldRef::AttributeValue(owner, position) | FieldRef::AttributeNamespace(owner, position) | FieldRef::AttributePrefix(owner, position), .. } if *owner == node && *position == index)
                {
                    self.retire_field_group(c, slot)?;
                    if let Some(entry) = removed_rows.get_mut(which) {
                        *entry = Some(slot);
                    }
                    which = which.saturating_add(1);
                }
            }
        }
        self.payloads.forget_attribute_strings(node);
        let removed = self.remove_attribute(c, node, index)?;
        drop(removed);
        self.holds.release_attribute(c)?;
        // The rows of the removed attribute keep its strings as history, detached; the rows of
        // the attributes after it follow them to their new positions.
        for slot in removed_rows.into_iter().flatten() {
            self.detach_attribute_string(c, slot)?;
        }
        if let Some(last_name) = layout {
            // The rows that can change are those of the attributes from `index`
            // on, and creation registered the last attribute first, so their
            // registry order is by descending position.
            for position in (index..len).rev() {
                for slot in attribute_row_slots(last_name, len, position)
                    .ok_or_else(|| latch_registry_row(c))?
                {
                    self.rebind_removed_attribute_row(c, slot, node, index)?;
                }
            }
        } else {
            for slot in 0..self.payloads.values.len() {
                self.rebind_removed_attribute_row(c, slot, node, index)?;
            }
        }
        self.payloads.detail_origins.close_gap(c, node, index)?;
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
