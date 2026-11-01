//! Session composition of the exclusive HTML attribute owner and original tickets.
use super::*;
#[cfg(test)]
mod tests;
use crate::parser::control::{SharedAllocation, VectorCapacity};
impl ConstructionOwner {
    pub(in crate::construction) fn take_attribute_edit(&mut self) -> HtmlConstructionAttributeEdit {
        self.attribute_edit.take().expect("active attribute writer")
    }
    pub(in crate::construction) fn put_attribute_edit(
        &mut self,
        group: HtmlConstructionAttributeEdit,
    ) {
        assert!(self.attribute_edit.is_none());
        self.attribute_edit = Some(group);
    }
    pub(in crate::construction) fn has_attribute_edit(&self) -> bool {
        self.attribute_edit.is_some()
    }
    pub(in crate::construction) fn attribute_edit(&mut self) -> &mut HtmlConstructionAttributeEdit {
        self.attribute_edit
            .as_mut()
            .expect("active exclusive attribute owner")
    }
    pub(in crate::construction) fn begin_attributes(
        &mut self,
        control: &mut ParseControl,
        target: usize,
        mode: HtmlConstructionAttributeEditMode,
    ) -> Result<(), SessionError> {
        control.require_identity(&self.control)?;
        assert!(
            self.attribute_edit.is_none(),
            "only one construction writer"
        );
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        let handle = self
            .storage()
            .node_at(target)
            .ok_or(SessionError::ForeignNode)?
            .handle();
        // Inline owner/phase/counter initialization is control bookkeeping;
        // real backing, plan, witness and slot work is admitted by each action.
        let storage = self.owner.take().expect("actual storage transfer");
        let mut group = HtmlConstructionAttributeEdit::begin(storage, handle, mode)
            .unwrap_or_else(|_| unreachable!("begin retains unchecked operands"));
        let result = (|| {
            while let Some(action) = group
                .next_action()
                .map_err(|e| construction_error(control, e))?
            {
                let description = action.description();
                control.admit_attribute_work(description.work())?;
                let report = matches!(
                    description,
                    HtmlConstructionAttributeDescription::PrepareAllocationReport(_)
                );
                if report {
                    ticket_work(control)?;
                }
                let observation = action
                    .perform()
                    .map_err(|e| construction_error(control, e))?;
                if report {
                    control.observe_construction_allocation(
                        observation.allocation().expect("actual plan report"),
                        &mut self.attribute_plan,
                        0,
                    )?;
                }
                observation.accept();
            }
            Ok(())
        })();
        self.attribute_edit = Some(group);
        if result.is_err() {
            self.abort_attributes(control)?;
        }
        result
    }
    pub(in crate::construction) fn finish_attributes(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let completion = HtmlConstructionAttributeEdit::try_finish(&mut self.attribute_edit)
            .map_err(|e| construction_error(control, e))?;
        self.adopt_attribute_completion(control, completion)
    }
    pub(in crate::construction) fn abort_attributes(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if let Some(group) = self.attribute_edit.take() {
            self.adopt_attribute_completion(control, group.abort())?;
        }
        Ok(())
    }
    fn adopt_attribute_completion(
        &mut self,
        control: &mut ParseControl,
        completion: HtmlConstructionAttributeCompletion,
    ) -> Result<(), SessionError> {
        let (storage, mut retirement) = completion.into_parts();
        self.owner = Some(storage);
        while let Some(action) = retirement.next_action() {
            match action.description() {
                HtmlConstructionAttributeRetirementDescription::NameWitness {
                    auxiliary_entries,
                    ..
                } => {
                    action.perform();
                    control.cleanup_auxiliary_entries(auxiliary_entries as u64)?;
                }
                HtmlConstructionAttributeRetirementDescription::Allocation => {
                    let ticket = self.attribute_plan.take().expect("actual plan ticket");
                    control.require_construction_retirement(
                        &ticket,
                        action.allocation().expect("actual plan bank"),
                    )?;
                    control.retire_construction_allocation(
                        ticket,
                        action.perform().expect("actual retired plan"),
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(in crate::construction) fn plan_attribute_candidate(
        &mut self,
        control: &mut ParseControl,
        index: usize,
        names: &[crate::parser::tokenizer::token::Attribute],
    ) -> Result<bool, SessionError> {
        let Self {
            attribute_edit,
            attribute_plan,
            ..
        } = self;
        let group = attribute_edit.as_mut().expect("named merge owner");
        let mut candidate = Some(
            group
                .candidate(index, names[index].name.value.text())
                .map_err(|e| construction_error(control, e))?,
        );
        (|| {
            while let Some(action) = candidate
                .as_mut()
                .expect("pending candidate")
                .next_action()
                .map_err(|e| construction_error(control, e))?
            {
                let description = action.description();
                let mut work = description.work();
                // The original reserve policy below admits the same actual
                // allocator provider once; it is not a second provider call.
                if matches!(
                    description,
                    HtmlConstructionAttributeDescription::ReservePlan { .. }
                ) {
                    work.provider_calls -= 1;
                }
                control.admit_attribute_work(work)?;
                let prior = description.prior_index();
                let expected = if let HtmlConstructionAttributeDescription::ReservePlan {
                    allocation,
                    minimum_capacity,
                    ..
                } = description
                {
                    control.prepare_construction_reserve(
                        allocation,
                        minimum_capacity,
                        attribute_plan.as_ref(),
                    )?;
                    ticket_work(control)?;
                    let Some(expected) = minimum_capacity.checked_mul(allocation.element_size())
                    else {
                        return Err(control
                            .stop_representation(crate::HtmlParseResource::ManagedCapacityBytes)
                            .into());
                    };
                    Some(expected)
                } else {
                    None
                };
                match action.perform(prior.map(|i| names[i].name.value.text())) {
                    Ok(observation) => {
                        if let Some(expected) = expected {
                            control.observe_construction_allocation(
                                observation.allocation().expect("actual reserve"),
                                attribute_plan,
                                expected,
                            )?;
                        }
                        observation.accept();
                    }
                    Err(error) => {
                        if let Some(expected) = expected {
                            control.record_construction_allocation(
                                candidate.as_ref().expect("failed candidate").allocation(),
                                attribute_plan,
                                expected,
                            )?;
                        }
                        return Err(if error == HtmlConstructionError::AllocationFailed {
                            control
                                .construction_allocation_failed(expected.unwrap_or(0))
                                .into()
                        } else {
                            construction_error(control, error)
                        });
                    }
                }
            }
            HtmlConstructionAttributeCandidate::try_finish(&mut candidate)
                .map_err(|e| construction_error(control, e))
        })()
    }
}
/// The returned ready owner keeps the same group and incoming slot exclusively
/// borrowed; the caller commits or drops it before restoring its original Weak.
pub(in crate::construction) fn prepare_attribute_slot<'a>(
    control: &mut ParseControl,
    slot: HtmlConstructionAttributeSlot<'a>,
    shared: &mut SharedAllocation<Vec<Attribute>>,
    allocation: &mut VectorCapacity<Attribute>,
) -> Result<HtmlConstructionReadyAttributeSlot<'a>, SessionError> {
    let backing = slot
        .shared_attributes()
        .map_err(|e| construction_error(control, e))?;
    control.prepare_controlled_attribute_edit(backing, shared, allocation)?;
    let mut slot = Some(slot);
    while let Some(action) = slot
        .as_mut()
        .expect("retained slot")
        .next_action()
        .map_err(|e| construction_error(control, e))?
    {
        let description = action.description();
        control.admit_attribute_work(description.work())?;
        let unique = matches!(
            description,
            HtmlConstructionAttributeDescription::UniqueBacking(_)
        );
        let expected = if let HtmlConstructionAttributeDescription::ReserveAttributes {
            before,
            minimum_capacity,
            ..
        } = description
        {
            Some(control.prepare_controlled_attribute_reserve(
                before,
                minimum_capacity,
                allocation,
            )?)
        } else {
            None
        };
        if unique || expected.is_some() {
            control.suspend_attribute_weak(shared);
        }
        match action.perform() {
            Ok(observation) => {
                if let Some(expected) = expected {
                    control.record_controlled_attribute_capacity(
                        observation.capacity().expect("actual reserve capacity"),
                        allocation,
                        expected,
                    )?;
                    let backing = observation
                        .shared_attributes()
                        .map_err(|e| construction_error(control, e))?;
                    control.restore_attribute_weak(backing, shared);
                    control.checkpoint()?;
                }
                observation.accept();
                if unique {
                    break;
                }
            }
            Err(error) => {
                if let Some(expected) = expected {
                    control.record_controlled_attribute_capacity(
                        slot.as_ref().expect("failed actual slot").capacity(),
                        allocation,
                        expected,
                    )?;
                }
                return Err(if error == HtmlConstructionError::AllocationFailed {
                    control
                        .construction_allocation_failed(expected.unwrap_or(0))
                        .into()
                } else {
                    construction_error(control, error)
                });
            }
        }
    }
    HtmlConstructionAttributeSlot::try_ready(&mut slot).map_err(|e| construction_error(control, e))
}
