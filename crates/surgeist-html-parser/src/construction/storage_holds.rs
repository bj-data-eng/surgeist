//! What the parse holds for HTML's construction storage: the bytes of its banks, attribute lists
//! and strings, the logical slots of its attributes, and the table of attribute names that edits
//! of long lists look names up in. Each is credited once, when the storage is released.
use super::*;
use surgeist_html::bounded::values::{HtmlAttributeNameIndex, HtmlNameIndexSlot};
use surgeist_html::values::Attribute;

/// Attribute lists at least this long are edited through a table of their names, so that the
/// check for a repeated name costs work in proportion to the list instead of its square. A
/// shorter list compares every pair, which is cheaper than building the table.
pub(super) const NAME_INDEX_MIN_ENTRIES: usize = 32;
/// Names that effects of the attribute being installed may add to the list: the table is sized
/// for them too, so that they are looked up in it like the rest.
pub(super) const NAME_INDEX_SPARE_ENTRIES: usize = 8;

/// The accounting the parse holds for one construction storage.
pub(super) struct StorageHolds {
    /// The table of attribute names that edits of a long attribute list look names up in, and
    /// the ticket of its slots. It stays as long as the lists it describes may be edited again.
    pub(super) name_index: Option<HtmlAttributeNameIndex>,
    pub(super) name_index_slots: Option<VectorCapacity<HtmlNameIndexSlot>>,
    /// The bytes held for the record bank, the child order of every record and the shared arena
    /// of a publication: the capacity each reserved.
    pub(super) bank_bytes: u64,
    /// The bytes held for every attribute list: the capacity each reserved and the shared
    /// wrapper each record holds its list in.
    pub(super) attribute_bytes: u64,
    /// The logical attribute slots held for the attributes of the storage.
    pub(super) attribute_entries: u64,
    /// The units held for the strings of the non-element fields of the storage: the text,
    /// comment, instruction and doctype strings its records hold.
    pub(super) string_units: u64,
    /// The bytes held for those strings: the capacity each reserved and the wrapper each holds
    /// its units in.
    pub(super) string_bytes: u64,
}

impl StorageHolds {
    pub(super) fn new() -> Self {
        Self {
            name_index: None,
            name_index_slots: None,
            bank_bytes: 0,
            attribute_bytes: 0,
            attribute_entries: 0,
            string_units: 0,
            string_bytes: 0,
        }
    }

    /// Makes sure the table of attribute names can hold `entries` names, reserving its slots
    /// through the controller. Nothing is built: an edit that borrows the table clears and
    /// fills it. A count too large to size a table leaves it as it is, and the edit then
    /// compares every pair.
    pub(super) fn provide_name_index(
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
    pub(super) fn retire_name_index(
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
    pub(super) fn absorb_attribute_list(
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
    pub(super) fn acquire_attribute(
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
    pub(super) fn release_attribute(
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

    /// Credits everything the parse holds for the storage: the table of attribute names, the
    /// attribute lists and their slots, the strings, and the banks. A refusal is remembered and
    /// the walk continues, so each remaining account is still credited; the first refusal is
    /// returned at the end.
    pub(super) fn release(
        mut self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
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
                owner,
                HtmlParseResource::ManagedAttributes,
                entries,
            ),
        );
        let units = std::mem::take(&mut self.string_units);
        let bytes = std::mem::take(&mut self.string_bytes);
        keep_first(
            &mut refused,
            control
                .release_text_accounts(units, bytes)
                .map_err(SessionError::Stop),
        );
        let bytes = std::mem::take(&mut self.bank_bytes);
        let credited = control.release_banks(bytes).map_err(SessionError::Stop);
        #[cfg(test)]
        inject_walk_refusal(control, WalkSite::ConstructionBank, &mut refused);
        keep_first(&mut refused, credited);
        #[cfg(test)]
        if control.take_construction_retire_fault() {
            keep_first(&mut refused, Err(latch_registry_row(control)));
        }
        refused.map_or(Ok(()), Err)
    }
}
