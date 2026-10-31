//! Actual per-slot parser composition: final generic checks precede source retirement.
use super::*;
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
        let mut group = self.construction.take_attribute_edit();
        let Some(RetainedPayload::Attributes(backing)) = self.payloads.values.get_mut(container)
        else {
            return Err(SessionError::WrongToken);
        };
        let AttributeStorage::Frozen {
            values,
            mut allocation,
            mut shared,
            element,
        } = backing.take().expect("actual attribute bank")
        else {
            unreachable!()
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
            Ok::<_, SessionError>(ready.commit().expect("actual replacement returns old slot"))
        })();
        let HtmlNodeKind::Element(record) = group
            .storage_view()
            .node_at(node)
            .expect("retained element")
            .kind()
        else {
            unreachable!()
        };
        control.restore_attribute_weak(record.attributes().shared_attributes(), &mut shared);
        let values = record.attributes().clone();
        let Some(RetainedPayload::Attributes(backing)) = self.payloads.values.get_mut(container)
        else {
            return Err(SessionError::WrongToken);
        };
        *backing = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        self.construction.put_attribute_edit(group);
        if result.is_ok() {
            self.construction.finish_attributes(control)?;
        } else {
            self.construction.abort_attributes(control)?;
        }
        result
    }
}
