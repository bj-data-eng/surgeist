//! Interning of names: the entries a storage keeps for the names of its elements and attributes.
//!
//! A name is interned once. The storage keeps one entry for each distinct name, an exact
//! [`QualifiedName`] with its [`ElementKey`](crate::vocabulary::ElementKey), and hands back the
//! entry again for every later occurrence of the name. The table only grows: an entry is never
//! removed or changed, so a [`NameId`] stays valid for the life of its storage, and a name that
//! an operation interned stays interned when a later step of that operation, or a rolled back
//! append, refuses. Interning a name costs one hash of its local name and the probes and
//! comparisons that find it, and an entry that is new also costs the growth of the banks it needs.
//! Each lookup and each placement of an entry in the index is charged a fixed probe allowance
//! before it runs, and one visit before each step beyond it, so the work of a name does not depend
//! on the process's hash key unless its chain is longer than the allowance.
//!
//! Each growth of a new entry is admitted before the first is recorded, and recorded before it is
//! reserved, as every bank growth is. The entry's own allocation and the shared handle of a
//! copied local name are made after the last allocator request: stable Rust has no fallible
//! constructor for them, so an allocator failure there aborts, as any infallible allocation does.
//! When the index grows, its new slots are made with the banks and take their entries after those
//! allocations, each placement charged before it runs. A refusal at any step leaves the entries,
//! the index and the usage as they were, and drops what the operation had made. The residues are
//! the capacities of the banks: a bank that grew keeps the room it reserved, and the consumer
//! holds the capacity it last recorded for it.

use super::*;
use crate::internal::construction::names::{NameEntry, NameKey, PROBE_ALLOWANCE};
use crate::internal::values::{
    Namespace,
    name_index::{
        HtmlNameIndex, HtmlNameIndexSlot, HtmlNameIndexSlots, HtmlNameIndexStep, HtmlNameIndexTable,
    },
};

/// The identity of one interned name of one storage.
///
/// The storage that interned the name issues it, and refuses another storage's with the
/// foreign-builder cause.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct NameId {
    tree: u64,
    index: u32,
}
impl fmt::Debug for NameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NameId")
            .field("tree", &self.tree)
            .field("index", &self.index)
            .finish()
    }
}

/// What the probe of the table found.
enum Found {
    /// The entry at this number of the table is the name.
    Existing(usize),
    /// No entry is the name; the first empty slot of its probe sequence, when the table has
    /// slots.
    Vacant(Option<crate::internal::values::name_index::HtmlNameIndexVacancy>),
}

impl HtmlConstructionStorage {
    /// The entry `id` names, or the foreign-builder cause for another storage's id.
    pub(in crate::internal::construction::bounded) fn name_entry(
        &self,
        id: &NameId,
    ) -> Result<&Arc<NameEntry>, HtmlConstructionError> {
        if id.tree != self.arena.serial {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        let position = usize::try_from(id.index)
            .map_err(|_| HtmlConstructionError::from(HtmlConstructionCause::OperandMissing))?;
        self.arena
            .names
            .get(position)
            .ok_or(HtmlConstructionCause::OperandMissing.error())
    }

    /// The name `id` names: the exact name that was interned, shared by every use of it. The
    /// foreign-builder cause for the id of another storage.
    pub fn name(&self, id: &NameId) -> Result<&QualifiedName, HtmlConstructionError> {
        Ok(self.name_entry(id)?.name())
    }

    /// Interns the name of `namespace`, `prefix` and `local_name`, and returns its id. A name the
    /// storage already holds returns the id it has, with no growth. A new name is made with a copy
    /// of `local_name`, which the storage holds in an allocation of its own, and shares the
    /// namespace and the prefix handles it is given.
    ///
    /// The operation charges one hash of the local name and the probes and comparisons of the
    /// lookup before they run, and admits every growth a new entry needs. It does not change the
    /// usage. An interned name stays in the table whatever happens next.
    pub fn intern_name<A: HtmlRecordAdmission>(
        &mut self,
        namespace: &Namespace,
        prefix: Option<&DocumentString>,
        local_name: &[u16],
        admission: &mut A,
    ) -> Result<NameId, HtmlConstructionRecordError<A::Stop>> {
        let key = NameKey {
            namespace,
            prefix,
            local_name,
        };
        let (number, _) = self.intern(key, None, admission)?;
        self.name_id(number)
    }

    /// The id of the entry `number`.
    fn name_id<S>(&self, number: usize) -> Result<NameId, HtmlConstructionRecordError<S>> {
        Ok(NameId {
            tree: self.arena.serial,
            index: u32::try_from(number)
                .map_err(|_| fault(HtmlConstructionCause::RepresentationExhausted))?,
        })
    }

    /// Finds the entry of `key` in the table. The probe allowance is charged before the first
    /// step, so the work of a lookup does not depend on the process's hash key unless the chain
    /// is longer than the allowance; each step beyond it, and each comparison, is charged before
    /// it runs.
    fn find_interned<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        hash: u64,
        key: NameKey<'_>,
        charge: &mut C,
    ) -> Result<Found, HtmlConstructionRecordError<C::Stop>> {
        Work {
            record_visits: PROBE_ALLOWANCE,
            ..Work::default()
        }
        .charge(charge)?;
        let names = &mut self.arena.names;
        if names.slots.is_empty() {
            return Ok(Found::Vacant(None));
        }
        let table = HtmlNameIndexTable::new(&mut names.index, &mut names.slots)
            .map_err(|_| fault(HtmlConstructionCause::IndexUnavailable))?;
        let mut probe = table.probe(hash);
        let mut steps = 0usize;
        loop {
            if steps >= PROBE_ALLOWANCE {
                Work::visit(charge)?;
            }
            steps = plus(steps, 1).map_err(faulted)?;
            match table.advance(&mut probe) {
                Ok(Some(HtmlNameIndexStep::Occupied(number))) => {
                    let entry = names
                        .entries
                        .get(number)
                        .ok_or_else(|| fault(HtmlConstructionCause::IndexUnavailable))?;
                    if entry.hash() == hash {
                        Work {
                            inspected_bytes: times(key.comparison_units(entry), size_of::<u16>())
                                .map_err(faulted)?,
                            ..Work::default()
                        }
                        .charge(charge)?;
                        if key.is(entry) {
                            return Ok(Found::Existing(number));
                        }
                    }
                }
                Ok(Some(HtmlNameIndexStep::Vacant(vacancy))) => {
                    return Ok(Found::Vacant(Some(vacancy)));
                }
                Ok(None) | Err(_) => {
                    return Err(fault(HtmlConstructionCause::IndexUnavailable));
                }
            }
        }
    }

    /// The number and the entry of the name `key` spells, made when the table has none. A new
    /// entry holds `given` when it is supplied, and otherwise a name built from a copy of the
    /// local name of `key` with its namespace and prefix.
    pub(in crate::internal::construction::bounded) fn intern<A: HtmlRecordAdmission>(
        &mut self,
        key: NameKey<'_>,
        given: Option<&QualifiedName>,
        admission: &mut A,
    ) -> Result<(usize, Arc<NameEntry>), HtmlConstructionRecordError<A::Stop>> {
        Work {
            record_visits: 1,
            inspected_bytes: times(key.local_name.len(), size_of::<u16>()).map_err(faulted)?,
            ..Work::default()
        }
        .charge(admission)?;
        let hash = self.arena.names.hash(key.local_name);
        let vacancy = match self.find_interned(hash, key, admission)? {
            Found::Existing(number) => {
                let entry = self
                    .arena
                    .names
                    .get(number)
                    .ok_or_else(|| fault(HtmlConstructionCause::IndexUnavailable))?;
                return Ok((number, entry.clone()));
            }
            Found::Vacant(vacancy) => vacancy,
        };
        let number = self.arena.names.len();
        let required = plus(number, 1).map_err(faulted)?;
        u32::try_from(required)
            .map_err(|_| fault(HtmlConstructionCause::RepresentationExhausted))?;
        // The banks the new entry needs. Only a name that is copied grows a text bank.
        let entries_growth = {
            let entries = &self.arena.names.entries;
            (required > entries.capacity()).then_some(HtmlConstructionBankGrowth {
                bank: HtmlConstructionBank::Names,
                before: HtmlConstructionBankCapacity {
                    initialized_len: entries.len(),
                    capacity: entries.capacity(),
                },
                minimum_capacity: required,
                element_size: size_of::<Arc<NameEntry>>(),
                wrapper_bytes: 0,
            })
        };
        let geometry = self.arena.names.growth();
        let slots_growth = geometry.map(|geometry| {
            let slots = &self.arena.names.slots;
            HtmlConstructionBankGrowth {
                bank: HtmlConstructionBank::NameSlots,
                before: HtmlConstructionBankCapacity {
                    initialized_len: slots.len(),
                    capacity: slots.capacity(),
                },
                minimum_capacity: geometry.get(),
                element_size: size_of::<HtmlNameIndexSlot>(),
                wrapper_bytes: 0,
            }
        });
        let entry_growth = HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::NameEntry,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: 1,
            element_size: NameEntry::ALLOCATION_BYTES,
            wrapper_bytes: 0,
        };
        let text_growth = given.is_none().then_some(HtmlConstructionBankGrowth {
            bank: HtmlConstructionBank::Text,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: key.local_name.len(),
            element_size: size_of::<u16>(),
            wrapper_bytes: size_of::<Vec<u16>>(),
        });
        if text_growth.is_some() {
            times(key.local_name.len(), size_of::<u16>())
                .ok()
                .filter(|bytes| *bytes <= isize::MAX as usize)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        }
        let growths = plus(
            plus(
                usize::from(entries_growth.is_some()),
                usize::from(slots_growth.is_some()),
            )
            .map_err(faulted)?,
            plus(1, usize::from(text_growth.is_some())).map_err(faulted)?,
        )
        .map_err(faulted)?;
        for growth in [
            entries_growth,
            slots_growth,
            Some(entry_growth),
            text_growth,
        ]
        .into_iter()
        .flatten()
        {
            admission
                .record_largest(
                    HtmlResource::AtomicLayoutBytes,
                    wide(times(growth.minimum_capacity, growth.element_size).map_err(faulted)?),
                )
                .map_err(stopped)?;
        }
        // Every write and copy is charged before any change is made: the entry, its handle, the
        // copied units, and the rewrite of the index when it grows. That rewrite places each
        // entry and the new one, and each placement is charged the probe allowance here and one
        // visit for each step beyond it before the step.
        let rebuilt_slots = geometry.map_or(0, HtmlNameIndexSlots::get);
        let placements = if geometry.is_some() { required } else { 0 };
        let copied_units = if text_growth.is_some() {
            key.local_name.len()
        } else {
            0
        };
        Work {
            record_visits: times(placements, PROBE_ALLOWANCE).map_err(faulted)?,
            provider_calls: growths,
            copied_bytes: plus(
                plus(
                    NameEntry::ALLOCATION_BYTES,
                    plus(
                        size_of::<Arc<NameEntry>>(),
                        times(copied_units, size_of::<u16>()).map_err(faulted)?,
                    )
                    .map_err(faulted)?,
                )
                .map_err(faulted)?,
                times(rebuilt_slots, size_of::<HtmlNameIndexSlot>()).map_err(faulted)?,
            )
            .map_err(faulted)?,
            record_writes: if geometry.is_some() {
                plus(required, 1).map_err(faulted)?
            } else {
                1
            },
            ..Work::default()
        }
        .charge(admission)?;
        // Every growth is admitted before the first is recorded.
        let chosen_entries = entries_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let chosen_slots = slots_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        let chosen_entry = Self::admit_bank(entry_growth, admission)?;
        let chosen_text = text_growth
            .map(|growth| Self::admit_bank(growth, admission))
            .transpose()?;
        if let (Some(growth), Some(chosen)) = (entries_growth, chosen_entries) {
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_name_entries(additional)
            })?;
        }
        // The new slots are made here and take their entries below, after the last allocation.
        let mut staged: Option<Vec<HtmlNameIndexSlot>> = None;
        if let (Some(growth), Some(chosen), Some(geometry)) = (slots_growth, chosen_slots, geometry)
        {
            self.grow(growth, chosen, admission, |_, additional| {
                Self::stage_name_slots(geometry, growth, additional, &mut staged)
            })?;
        }
        // The entry's own allocation is made after this step, so its bytes are recorded here.
        self.grow(entry_growth, chosen_entry, admission, |_, additional| {
            Reserved {
                ok: true,
                reached: HtmlConstructionBankCapacity {
                    initialized_len: 0,
                    capacity: additional,
                },
            }
        })?;
        let name = match (given, text_growth, chosen_text) {
            (Some(name), ..) => name.clone(),
            (None, Some(growth), Some(chosen)) => {
                let mut reserved = Vec::new();
                self.grow(growth, chosen, admission, |_, additional| {
                    reserved = Vec::new();
                    let ok = reserved.try_reserve_exact(additional).is_ok();
                    Reserved {
                        ok,
                        reached: HtmlConstructionBankCapacity {
                            initialized_len: 0,
                            capacity: reserved.capacity(),
                        },
                    }
                })?;
                reserved.extend_from_slice(key.local_name);
                QualifiedName::new(
                    key.namespace.clone(),
                    key.prefix.cloned(),
                    DocumentString::from_utf16(reserved),
                )
            }
            _ => return Err(fault(HtmlConstructionCause::CapacityMismatch)),
        };
        // Nothing below reserves. A placement is charged before it runs and may be refused, which
        // drops the staged slots and leaves the table as it was; the writes after the last
        // placement cannot refuse.
        let entry = Arc::new(NameEntry::new(
            name,
            hash,
            u32::try_from(number)
                .map_err(|_| fault(HtmlConstructionCause::RepresentationExhausted))?,
        ));
        if let Some(mut slots) = staged.take() {
            let geometry =
                geometry.ok_or_else(|| fault(HtmlConstructionCause::CapacityMismatch))?;
            let mut index = HtmlNameIndex::new(geometry);
            {
                let mut table = HtmlNameIndexTable::new(&mut index, &mut slots)
                    .map_err(|_| fault(HtmlConstructionCause::IndexUnavailable))?;
                for (placed, earlier) in self.arena.names.entries.iter().enumerate() {
                    Self::place_name(&mut table, earlier.hash(), placed, admission)?;
                }
                Self::place_name(&mut table, hash, number, admission)?;
            }
            self.arena.names.index = index;
            self.arena.names.slots = slots;
        } else {
            let vacancy = vacancy.ok_or_else(|| fault(HtmlConstructionCause::IndexUnavailable))?;
            let mut table = self
                .arena
                .names
                .lend()
                .map_err(fault)?
                .ok_or_else(|| fault(HtmlConstructionCause::IndexUnavailable))?;
            table
                .insert(vacancy, number)
                .map_err(|_| fault(HtmlConstructionCause::IndexUnavailable))?;
        }
        self.arena.names.entries.push(entry.clone());
        Ok((number, entry))
    }

    fn reserve_name_entries(&mut self, additional: usize) -> Reserved {
        let entries = &mut self.arena.names.entries;
        let ok = entries.try_reserve_exact(additional).is_ok();
        Reserved {
            ok,
            reached: HtmlConstructionBankCapacity {
                initialized_len: entries.len(),
                capacity: entries.capacity(),
            },
        }
    }

    /// Makes the empty slots of an index of `geometry` in new storage with room for the old slots
    /// and `additional` more, and leaves them in `staged` without installing them.
    fn stage_name_slots(
        geometry: HtmlNameIndexSlots,
        growth: HtmlConstructionBankGrowth,
        additional: usize,
        staged: &mut Option<Vec<HtmlNameIndexSlot>>,
    ) -> Reserved {
        let before = growth.before();
        let failed = Reserved {
            ok: false,
            reached: before,
        };
        let Some(target) = before
            .initialized_len()
            .checked_add(additional)
            .filter(|target| *target >= geometry.get())
        else {
            return failed;
        };
        let mut fresh: Vec<HtmlNameIndexSlot> = Vec::new();
        if fresh.try_reserve_exact(target).is_err() {
            return failed;
        }
        fresh.resize_with(geometry.get(), HtmlNameIndexSlot::default);
        let reached = HtmlConstructionBankCapacity {
            initialized_len: before.initialized_len(),
            capacity: fresh.capacity(),
        };
        *staged = Some(fresh);
        Reserved { ok: true, reached }
    }

    /// Writes entry `number`, whose name hashes to `hash`, at the first empty slot of its probe
    /// sequence. The allowance of the placement is charged in advance, and each step beyond it is
    /// charged before the step.
    fn place_name<C: Charge<HtmlWork, HtmlResource>>(
        table: &mut HtmlNameIndexTable<'_>,
        hash: u64,
        number: usize,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        let mut probe = table.probe(hash);
        let mut steps = 0usize;
        loop {
            if steps >= PROBE_ALLOWANCE {
                Work::visit(charge)?;
            }
            steps = plus(steps, 1).map_err(faulted)?;
            match table.advance(&mut probe) {
                Ok(Some(HtmlNameIndexStep::Vacant(vacancy))) => {
                    return table
                        .insert(vacancy, number)
                        .map_err(|_| fault(HtmlConstructionCause::IndexUnavailable));
                }
                Ok(Some(HtmlNameIndexStep::Occupied(_))) => {}
                Ok(None) | Err(_) => {
                    return Err(fault(HtmlConstructionCause::IndexUnavailable));
                }
            }
        }
    }

    /// Creates a detached element named by `name`, an id this storage issued, with no
    /// attributes. As [`create_element`](Self::create_element), without interning the name again
    /// and without attributes, which the element receives from
    /// [`append_named_attribute`](Self::append_named_attribute). The foreign-builder cause for the
    /// id of another storage.
    pub fn create_element_named<A: HtmlRecordAdmission>(
        &mut self,
        name: &NameId,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        Work::visit(admission)?;
        let entry = self.name_entry(name).map_err(faulted)?.clone();
        self.create_element_of(entry, link_flags, super::Authored::NONE, admission)
    }

    /// Creates the element of the interned `name`.
    pub(super) fn create_element_of<A: HtmlRecordAdmission>(
        &mut self,
        name: Arc<NameEntry>,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
        authored: super::Authored<'_>,
        admission: &mut A,
    ) -> Result<HtmlBuilderNode, HtmlConstructionRecordError<A::Stop>> {
        let known = name.key().html();
        let template = known == Some(crate::vocabulary::KnownHtmlElement::Template);
        if link_flags.is_some() && known != Some(crate::vocabulary::KnownHtmlElement::Link) {
            return Err(fault(HtmlConstructionCause::WrongNodeKind));
        }
        self.create_with(
            Created::Element {
                name,
                link_flags,
                template,
            },
            authored,
            admission,
        )
    }
}
