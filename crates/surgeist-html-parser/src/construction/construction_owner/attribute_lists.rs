//! The session's side of edits to the attribute lists of HTML's storage: the admission and
//! accounting of the growth of a list, and the table of attribute names that long lists are
//! edited through.
use super::super::storage_attributes::KeptCandidate;
use super::*;
use crate::parser::control::HtmlParseInvariant;
use surgeist_html::{
    accounting::{HtmlChange, HtmlChargedError, HtmlResource, HtmlWork},
    bounded::construction::{
        HtmlAttributeAdmission, HtmlAttributeMerge, HtmlConstructionAttributeCapacity,
        HtmlConstructionAttributeError, HtmlConstructionAttributeGrowth,
    },
    construction::{ConstructionNodeKind, HtmlBuilderNode},
    values::{Attribute, DocumentString},
};
use surgeist_ledger::Charge;

/// Attribute lists at least this long are edited through a table of their names, so that the
/// check for a repeated name costs work in proportion to the list instead of its square. A
/// shorter list compares every pair, which is cheaper than building the table.
pub(in crate::construction) const NAME_INDEX_MIN_ENTRIES: usize = 32;
/// Names that effects of the attribute being installed may add to the list: the table is sized
/// for them too, so that they are looked up in it like the rest.
pub(in crate::construction) const NAME_INDEX_SPARE_ENTRIES: usize = 8;

/// What the controller admits and records for one edit of an attribute list.
struct Admission<'a> {
    control: &'a mut ParseControl,
    /// The bytes the parse holds for all attribute lists.
    bytes: &'a mut u64,
    /// The growth under way: the list before it, and the bytes of the layout it was admitted to.
    growing: Option<(HtmlConstructionAttributeCapacity, u64)>,
}

impl Charge<HtmlWork, HtmlResource> for Admission<'_> {
    type Stop = HtmlParseStop;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), HtmlParseStop> {
        Charge::charge(&mut *self.control, batch)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), HtmlParseStop> {
        Charge::record_largest(&mut *self.control, resource, value)
    }
}

impl HtmlAttributeAdmission for Admission<'_> {
    fn admit_growth(
        &mut self,
        growth: HtmlConstructionAttributeGrowth,
    ) -> Result<usize, HtmlParseStop> {
        let before = growth.before();
        let target = self
            .control
            .attribute_growth_target(before, growth.minimum_capacity());
        let expected = self.control.prepare_attribute_growth(before, target)?;
        self.growing = Some((before, expected));
        Ok(target)
    }

    fn record_growth(
        &mut self,
        reached: HtmlConstructionAttributeCapacity,
    ) -> Result<(), HtmlParseStop> {
        let Some((before, expected)) = self.growing.take() else {
            return Err(self
                .control
                .fail_invariant(HtmlParseInvariant::CapacityMismatch));
        };
        let recorded = self
            .control
            .record_attribute_growth(before, reached, expected, self.bytes);
        // The capacity just recorded is the one the next record replaces, and an allocator that
        // refused the reservation reports the layout it was asked for.
        self.growing = Some((reached, expected));
        recorded
    }
}

impl<'a> Admission<'a> {
    fn new(control: &'a mut ParseControl, bytes: &'a mut u64) -> Self {
        Self {
            control,
            bytes,
            growing: None,
        }
    }

    /// The session error for an edit's refusal. An allocation the allocator refused reports the
    /// layout the growth was admitted to.
    fn error(self, error: HtmlConstructionAttributeError<HtmlParseStop>) -> SessionError {
        match error {
            HtmlChargedError::Stopped(stop) => SessionError::Stop(stop),
            HtmlChargedError::Faulted(error) => {
                if error.cause() == HtmlConstructionCause::AllocationFailed {
                    let layout = self
                        .growing
                        .map_or(0, |(_, bytes)| usize::try_from(bytes).unwrap_or(usize::MAX));
                    return SessionError::Stop(self.control.construction_allocation_failed(layout));
                }
                construction_error(self.control, error)
            }
        }
    }
}

/// Where the candidates of a merge come from and where its decisions go.
pub(in crate::construction) struct MergeCandidates<'a> {
    control: &'a mut ParseControl,
    names: &'a [crate::parser::tokenizer::token::Attribute],
    kept: &'a mut OwnedVec<KeptCandidate>,
}

impl<'a> MergeCandidates<'a> {
    pub(in crate::construction) fn new(
        control: &'a mut ParseControl,
        names: &'a [crate::parser::tokenizer::token::Attribute],
        kept: &'a mut OwnedVec<KeptCandidate>,
    ) -> Self {
        Self {
            control,
            names,
            kept,
        }
    }
}

impl Charge<HtmlWork, HtmlResource> for MergeCandidates<'_> {
    type Stop = SessionError;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), SessionError> {
        Charge::charge(&mut *self.control, batch).map_err(SessionError::Stop)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), SessionError> {
        Charge::record_largest(&mut *self.control, resource, value).map_err(SessionError::Stop)
    }
}

impl HtmlAttributeMerge for MergeCandidates<'_> {
    fn candidates(&self) -> usize {
        self.names.len()
    }

    fn name(&self, index: usize) -> Option<DocumentString> {
        self.names
            .get(index)
            .map(|candidate| candidate.name.value.text().clone())
    }

    fn keep(&mut self, index: usize) -> Result<(), SessionError> {
        let Some(name) = self.name(index) else {
            return Err(latch_registry_row(self.control));
        };
        self.kept
            .try_push_geometric_toward(
                self.control,
                KeptCandidate {
                    index,
                    name: Some(name),
                },
                self.names.len(),
            )
            .map_err(SessionError::Stop)
    }

    fn kept(&self, rank: usize) -> Option<usize> {
        self.kept.as_slice().get(rank).map(|kept| kept.index)
    }
}

impl ConstructionOwner {
    /// Makes sure the table of attribute names can hold `entries` names, reserving its slots
    /// through the controller. Nothing is built: an edit that borrows the table clears and
    /// fills it. A count too large to size a table leaves it as it is, and the edit then
    /// compares every pair.
    pub(in crate::construction) fn provide_name_index(
        &mut self,
        control: &mut ParseControl,
        entries: usize,
    ) -> Result<(), SessionError> {
        let Some(slots) = HtmlAttributeNameIndex::slots_required(entries) else {
            return Ok(());
        };
        if self
            .name_index
            .as_ref()
            .is_some_and(|index| index.capacity() >= slots)
        {
            return Ok(());
        }
        self.retire_name_index(control)?;
        let (mut vector, mut ticket) = control.new_vector::<HtmlNameIndexSlot>()?;
        if let Err(stop) = control.reserve_vector(&mut vector, &mut ticket, slots) {
            // The reserve may have changed the capacity before it stopped.
            release_vector(control, vector, ticket)?;
            return Err(stop.into());
        }
        self.name_index = Some(HtmlAttributeNameIndex::new(vector));
        self.name_index_slots = Some(ticket);
        Ok(())
    }

    /// Releases the table of attribute names, crediting its slots. Nothing happens when there is
    /// none.
    pub(in crate::construction) fn retire_name_index(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let (Some(index), Some(ticket)) = (self.name_index.take(), self.name_index_slots.take())
        else {
            return Ok(());
        };
        release_vector(control, index.into_storage(), ticket)
    }

    /// Takes the accounting of a list built by the parse and handed to the storage in a record,
    /// which the parse releases with the lists it holds.
    pub(in crate::construction) fn absorb_attribute_list(
        &mut self,
        control: &mut ParseControl,
        capacity: VectorCapacity<Attribute>,
        shared: SharedAllocation<Vec<Attribute>>,
    ) -> Result<(), SessionError> {
        let bytes = control.absorb_attribute_list(capacity, shared)?;
        self.attribute_bytes = self
            .attribute_bytes
            .checked_add(bytes)
            .ok_or_else(|| control.stop_representation(HtmlParseResource::ManagedCapacityBytes))?;
        Ok(())
    }

    /// Acquires the logical slot of one attribute that the parse is about to store.
    pub(in crate::construction) fn acquire_attribute(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let held = self
            .attribute_entries
            .checked_add(1)
            .ok_or_else(|| control.stop_representation(HtmlParseResource::ManagedAttributes))?;
        control.acquire(HtmlParseResource::ManagedAttributes, 1)?;
        self.attribute_entries = held;
        Ok(())
    }

    /// Releases the logical slot of one attribute that the parse stopped storing.
    pub(in crate::construction) fn release_attribute(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let Some(held) = self.attribute_entries.checked_sub(1) else {
            return Err(latch_registry_row(control));
        };
        control.release(HtmlParseResource::ManagedAttributes, 1)?;
        self.attribute_entries = held;
        Ok(())
    }

    /// Releases the accounting of every attribute list the owner holds.
    pub(super) fn release_attribute_accounts(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut refused = None;
        keep_first(&mut refused, self.retire_name_index(control));
        let bytes = std::mem::take(&mut self.attribute_bytes);
        keep_first(
            &mut refused,
            control
                .release_attribute_lists(bytes)
                .map_err(SessionError::Stop),
        );
        let entries = std::mem::take(&mut self.attribute_entries);
        keep_first(
            &mut refused,
            release_logical(
                control,
                &self.control,
                HtmlParseResource::ManagedAttributes,
                entries,
            ),
        );
        refused.map_or(Ok(()), Err)
    }

    /// The handle of the element at `node` and the length of its attribute list, after the
    /// visit and provider call that qualify an edit.
    fn attribute_target(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<(HtmlBuilderNode, usize), SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let record = self
            .storage(control)?
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        let len = match record.kind() {
            ConstructionNodeKind::Element(element) => element.attributes().len(),
            _ => return Err(SessionError::WrongToken),
        };
        Ok((record.handle(), len))
    }

    /// Ensures the attribute list of the element at `node` holds `additional` more attributes,
    /// growing it once when it does not.
    pub(in crate::construction) fn reserve_attributes(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        additional: usize,
    ) -> Result<(), SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            owner,
            attribute_bytes,
            ..
        } = self;
        let Some(storage) = owner.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let mut admission = Admission::new(control, attribute_bytes);
        match storage.reserve_attributes(&handle, additional, &mut admission) {
            Ok(_) => Ok(()),
            Err(error) => Err(admission.error(error)),
        }
    }

    /// Appends the attribute in `incoming` to the element at `node`. `planned` names the
    /// candidate of a merge plan that the attribute is.
    pub(in crate::construction) fn append_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        incoming: &mut Option<Attribute>,
        planned: Option<&DocumentString>,
    ) -> Result<usize, SessionError> {
        let (handle, len) = self.attribute_target(control, node)?;
        let Self {
            owner,
            attribute_bytes,
            name_index,
            ..
        } = self;
        let Some(storage) = owner.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let names = if len.saturating_add(1) < NAME_INDEX_MIN_ENTRIES {
            None
        } else {
            name_index.as_mut()
        };
        let mut admission = Admission::new(control, attribute_bytes);
        let result = match planned {
            Some(candidate) => storage.append_planned_attribute(
                &handle,
                incoming,
                candidate,
                names,
                &mut admission,
            ),
            None => storage.append_attribute(&handle, incoming, names, &mut admission),
        };
        result.map_err(|error| admission.error(error))
    }

    /// Replaces the attribute at `index` of the element at `node` with the one in `incoming`
    /// and returns the one it replaced.
    pub(in crate::construction) fn replace_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
        incoming: &mut Option<Attribute>,
    ) -> Result<Attribute, SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            owner,
            attribute_bytes,
            ..
        } = self;
        let Some(storage) = owner.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let mut admission = Admission::new(control, attribute_bytes);
        storage
            .replace_attribute(&handle, index, incoming, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Removes the attribute at `index` of the element at `node` and returns it.
    pub(in crate::construction) fn remove_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        index: usize,
    ) -> Result<Attribute, SessionError> {
        let (handle, _) = self.attribute_target(control, node)?;
        let Self {
            owner,
            attribute_bytes,
            ..
        } = self;
        let Some(storage) = owner.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let mut admission = Admission::new(control, attribute_bytes);
        storage
            .remove_attribute(&handle, index, &mut admission)
            .map_err(|error| admission.error(error))
    }

    /// Decides which of the candidates of `names` the element at `node` keeps, pushing the
    /// index of each kept candidate to `kept`. Returns how many are kept.
    pub(in crate::construction) fn plan_attribute_merge(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        names: &[crate::parser::tokenizer::token::Attribute],
        kept: &mut OwnedVec<KeptCandidate>,
    ) -> Result<usize, SessionError> {
        let (handle, len) = self.attribute_target(control, node)?;
        let Self {
            owner, name_index, ..
        } = self;
        let Some(storage) = owner.as_ref() else {
            return Err(latch_registry_row(control));
        };
        let entries = len.saturating_add(names.len());
        let index = if entries < NAME_INDEX_MIN_ENTRIES {
            None
        } else {
            name_index.as_mut()
        };
        let mut merge = MergeCandidates::new(control, names, kept);
        match storage.plan_attribute_merge(&handle, index, &mut merge) {
            Ok(count) => Ok(count),
            Err(HtmlChargedError::Stopped(error)) => Err(error),
            Err(HtmlChargedError::Faulted(error)) => Err(construction_error(merge.control, error)),
        }
    }
}
