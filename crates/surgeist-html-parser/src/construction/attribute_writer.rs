//! Actual per-slot parser composition: final generic checks precede source retirement.
use super::*;
use surgeist_html::{
    bounded::construction::HtmlConstructionAttributeEditMode, construction::ConstructionNodeKind,
    values::Attribute,
};
impl SessionStorage {
    pub(super) fn replace_attribute_storage(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        container: usize,
        position: usize,
        incoming: &mut Option<Attribute>,
        retire_slots: &[Option<usize>],
    ) -> Result<Attribute, SessionError> {
        self.begin_attribute_edit(control, node, HtmlConstructionAttributeEditMode::Slots)?;
        let mut group = self.construction.take_attribute_edit(control)?;
        // Everything that can refuse is checked before ownership leaves the
        // container, and a refusal hands the edit back and aborts it. The edit
        // changes only the element's attributes, so the record checked here is
        // still an element when the container is restored below.
        if !matches!(
            group.storage_view().node_at(node).map(|node| node.kind()),
            Some(ConstructionNodeKind::Element(_))
        ) {
            return Err(self.refuse_attribute_edit(control, group));
        }
        let Some(RetainedPayload::Attributes(backing)) = self.payloads.values.get_mut(container)
        else {
            return Err(self.refuse_attribute_edit(control, group));
        };
        let taken = backing.take();
        let Some(AttributeStorage::Frozen {
            values,
            mut allocation,
            mut shared,
            element,
        }) = taken
        else {
            *backing = taken;
            return Err(self.refuse_attribute_edit(control, group));
        };
        drop(values);
        let result = (|| {
            let slot = group
                .replace(position, incoming)
                .map_err(|e| construction_owner::construction_error(control, e))?;
            let ready = prepare_attribute_slot(control, slot, &mut shared, &mut allocation)?;
            // The ready owner borrows only the generic group and incoming slot.
            // These source-history reads/writes cannot inspect or mutate the tree.
            for slot in retire_slots.iter().flatten() {
                self.retire_field_group(control, *slot)?;
            }
            ready
                .commit()
                .map_err(|refusal| {
                    construction_owner::construction_error(control, refusal.into_parts().1)
                })?
                .ok_or_else(|| latch_registry_row(control))
        })();
        // `node` named an element when the edit began and the edit only changes
        // its attributes, so the record is still an element; the container row
        // was checked above and the registry only grows.
        let Some(ConstructionNodeKind::Element(record)) =
            group.storage_view().node_at(node).map(|node| node.kind())
        else {
            return Err(self.refuse_attribute_edit(control, group));
        };
        let values = record.attributes().to_ordered();
        control.restore_attribute_weak(values.shared_attributes(), &mut shared);
        let restored = AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        };
        match self.payloads.values.get_mut(container) {
            Some(RetainedPayload::Attributes(backing)) => *backing = Some(restored),
            _ => return Err(self.refuse_attribute_edit(control, group)),
        }
        self.construction.put_attribute_edit(control, group)?;
        if result.is_ok() {
            self.construction.finish_attributes(control)?;
        } else {
            self.construction.abort_attributes(control)?;
        }
        result
    }
    /// Hands a taken attribute edit back to its owner, aborts it, and latches
    /// the invariant that made it impossible to continue.
    fn refuse_attribute_edit(
        &mut self,
        control: &mut ParseControl,
        group: surgeist_html::bounded::construction::HtmlConstructionAttributeEdit,
    ) -> SessionError {
        let invariant = latch_registry_row(control);
        // The invariant above is the reported first stop; a refusal to take the
        // edit back already ended it, and an abort refusal would be secondary
        // and is latched in the controller by its owner.
        if self.construction.put_attribute_edit(control, group).is_ok() {
            let _ = self.construction.abort_attributes(control);
        }
        invariant
    }
}
