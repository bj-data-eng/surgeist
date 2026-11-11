//! Edits of the attribute list of an element of the construction storage.
//!
//! Each edit is one method of [`HtmlConstructionStorage`]. It reads and compares first, charging
//! its consumer for the work before it runs, and asks the consumer once to admit the growth of
//! the list. Only then does it change the list, so a refusal leaves the records, the usage and
//! the attributes as they were. Two residues remain: the name index is a cache of the names of
//! one element and a refused edit may have rebuilt it, and when the allocator refuses the room
//! that was admitted and the consumer then refuses to record the capacity the list kept, the
//! list holds the room the fallback reserved with no attribute changed. The consumer owns the
//! policy: it charges work through [`Charge`], chooses the capacity a growing list reserves, and
//! records the capacity.

use super::operations::attribute_units;
use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use crate::internal::values::attribute_name_index::{
    Binding, CLEAR_CHUNK, HtmlAttributeNameIndex, entry, occupant, tag,
};
use crate::internal::values::bounded::{
    ComparisonStep, Phase as NamePhase, advance_comparison, comparison_layout,
};
use crate::internal::values::name_index::{
    HtmlNameIndex, HtmlNameIndexSlot, HtmlNameIndexStep, HtmlNameIndexVacancy,
};
use crate::internal::values::{Attribute, DocumentString};
use surgeist_ledger::{Change, Charge};

/// The most bytes one charge covers when an edit moves a run of attributes, so that a consumer
/// that polls at its charges polls within every quantum of the work.
const CHARGE_QUANTUM: usize = 256;

/// Actual attribute-list dimensions; no allocation or release authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeCapacity {
    /// The number of attributes currently stored.
    pub initialized_len: usize,
    /// The reserved attribute capacity; at least `initialized_len`.
    pub capacity: usize,
}
impl HtmlConstructionAttributeCapacity {
    /// The size in bytes of one stored attribute.
    pub fn element_size(self) -> usize {
        size_of::<Attribute>()
    }
    /// The alignment in bytes of one stored attribute.
    pub fn element_alignment(self) -> usize {
        align_of::<Attribute>()
    }
}

/// What an edit asks its consumer to admit before it allocates: the list of one element must hold
/// at least [`minimum_capacity`](Self::minimum_capacity) attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionAttributeGrowth {
    before: HtmlConstructionAttributeCapacity,
    minimum_capacity: usize,
}
impl HtmlConstructionAttributeGrowth {
    /// The dimensions of the list before the growth.
    pub fn before(self) -> HtmlConstructionAttributeCapacity {
        self.before
    }
    /// The least capacity that holds the attributes the edit stores.
    pub fn minimum_capacity(self) -> usize {
        self.minimum_capacity
    }
}

/// The consumer of an attribute edit. It charges the edit's work like any [`Charge`], decides the
/// capacity that a growing list reserves, and records the capacity the reservation reached.
pub trait HtmlAttributeAdmission: Charge<HtmlWork, HtmlResource> {
    /// Admits the growth of one attribute list before it is reserved. Returns the capacity to
    /// reserve, which must be at least the growth's minimum; a smaller one is refused with the
    /// capacity-mismatch cause and nothing is reserved. A refusal leaves the edit unapplied.
    /// Nothing is recorded or reserved before this call, so a refusal changes nothing.
    fn admit_growth(
        &mut self,
        growth: HtmlConstructionAttributeGrowth,
    ) -> Result<usize, Self::Stop>;

    /// Records the capacity of the list. It is called with the capacity the consumer chose before
    /// the list is reserved, so a refusal there leaves the list as it was. It is called again
    /// with the capacity the reservation reached when that differs from the one chosen, which
    /// includes the unchanged capacity of a reservation the allocator refused. A refusal of that
    /// second record leaves the list with the capacity it reached and no attribute changed.
    fn record_growth(
        &mut self,
        reached: HtmlConstructionAttributeCapacity,
    ) -> Result<(), Self::Stop>;
}

/// The consumer of an attribute merge plan: the candidates to merge and where it keeps the
/// decisions.
pub trait HtmlAttributeMerge: Charge<HtmlWork, HtmlResource> {
    /// The number of candidates, in the order they will be appended.
    fn candidates(&self) -> usize;

    /// The local name of candidate `index`, which has no namespace. `None` past the candidates.
    fn name(&self, index: usize) -> Option<DocumentString>;

    /// Records that candidate `index` is kept: the next kept candidate in order.
    fn keep(&mut self, index: usize) -> Result<(), Self::Stop>;

    /// The candidate index of the `rank`th kept candidate. `None` for a rank not yet kept.
    fn kept(&self, rank: usize) -> Option<usize>;
}

/// A refusal of an attribute edit: the consumer's stop, or the edit's own fault.
pub type HtmlConstructionAttributeError<S> = HtmlChargedError<S, HtmlConstructionError>;

/// The work an edit charges as one batch.
#[derive(Clone, Copy, Debug, Default)]
struct Work {
    record_visits: usize,
    provider_calls: usize,
    inspected_bytes: usize,
    copied_bytes: usize,
    record_writes: usize,
}
impl Work {
    fn charge<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionAttributeError<C::Stop>> {
        charge
            .charge(&[
                Change::Work(HtmlWork::RecordVisit, wide(self.record_visits)),
                Change::Work(HtmlWork::ProviderCall, wide(self.provider_calls)),
                Change::Work(HtmlWork::InspectByte, wide(self.inspected_bytes)),
                Change::Work(HtmlWork::CopyByte, wide(self.copied_bytes)),
                Change::Work(HtmlWork::RecordWrite, wide(self.record_writes)),
            ])
            .map_err(HtmlChargedError::Stopped)
    }
}

fn fault<S>(cause: HtmlConstructionCause) -> HtmlConstructionAttributeError<S> {
    HtmlChargedError::Faulted(cause.error())
}
fn faulted<S>(error: HtmlConstructionError) -> HtmlConstructionAttributeError<S> {
    HtmlChargedError::Faulted(error)
}
/// The name index could not serve the edit: its storage, probe or vacancy did not agree with the
/// table the edit expected.
fn table_error<S>() -> HtmlConstructionAttributeError<S> {
    fault(HtmlConstructionCause::IndexUnavailable)
}
fn plus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_add(b)
        .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
}
fn minus(a: usize, b: usize) -> Result<usize, HtmlConstructionError> {
    a.checked_sub(b)
        .ok_or(HtmlConstructionCause::ReleaseUnderflow.error())
}
fn identical_name(left: &QualifiedName, right: &QualifiedName) -> bool {
    let namespace = match (left.namespace().uri(), right.namespace().uri()) {
        (None, None) => true,
        (Some(a), Some(b)) => a.shares_storage_with(b),
        _ => false,
    };
    namespace && left.local_name().shares_storage_with(right.local_name())
}

/// The work of reading one slot of the index and entering a name in the empty slot it finds.
fn probe_work() -> Work {
    Work {
        record_visits: 1,
        record_writes: 2,
        copied_bytes: size_of::<HtmlNameIndexSlot>() + size_of::<usize>(),
        ..Work::default()
    }
}
/// The work of starting a rebuild: a new index over the storage, an unbound table and an empty
/// storage.
fn begin_clear_work() -> Work {
    Work {
        record_writes: 3,
        copied_bytes: size_of::<HtmlNameIndex>() + size_of::<Binding>() + size_of::<usize>(),
        ..Work::default()
    }
}
/// The work of clearing the next slots of the index. The chunk that finishes the table also binds
/// it to its index, which reads each slot once.
fn clear_work(table: &HtmlAttributeNameIndex) -> Work {
    let slots = table.unwritten().min(CLEAR_CHUNK);
    let scanned = table.scanned_by_clearing(CLEAR_CHUNK);
    let finishing = usize::from(scanned != 0);
    Work {
        record_writes: slots + 2 * finishing,
        copied_bytes: slots * size_of::<HtmlNameIndexSlot>()
            + finishing * (size_of::<usize>() + size_of::<Binding>()),
        inspected_bytes: scanned * size_of::<HtmlNameIndexSlot>(),
        ..Work::default()
    }
}
/// The work of reading the lengths of a name to hash: the attribute and its two name headers.
fn hash_length_work() -> Work {
    Work {
        record_visits: 3,
        ..Work::default()
    }
}
fn hash_work(units: usize) -> Work {
    Work {
        inspected_bytes: units.saturating_mul(2),
        ..Work::default()
    }
}
fn hashed_units(name: &QualifiedName) -> usize {
    name.namespace()
        .uri()
        .map_or(0, DocumentString::len)
        .saturating_add(name.local_name().len())
}

/// Compares two expanded names, charging each step before it reads.
fn names_equal<C: Charge<HtmlWork, HtmlResource>>(
    charge: &mut C,
    left: (Option<&DocumentString>, &DocumentString),
    right: (Option<&DocumentString>, &DocumentString),
) -> Result<bool, HtmlConstructionAttributeError<C::Stop>> {
    let mut phase = NamePhase::Pair;
    loop {
        comparison_layout(phase)
            .charge(charge)
            .map_err(HtmlChargedError::Stopped)?;
        match advance_comparison(phase, left.0, left.1, right.0, right.1) {
            ComparisonStep::Continue(next) => phase = next,
            ComparisonStep::Different => return Ok(false),
            ComparisonStep::Equal => return Ok(true),
        }
    }
}
fn expanded(name: &QualifiedName) -> (Option<&DocumentString>, &DocumentString) {
    (name.namespace().uri(), name.local_name())
}

/// What the name index found for a name.
enum Lookup {
    /// An attribute below the limit has the name.
    Found(usize),
    /// No attribute has the name. `vacancy` is the empty slot its probe ended at, and `planned`
    /// whether the probe passed the entry a merge plan left for the attribute about to be
    /// appended: one for the position the list ends at, with the name's tag.
    Absent {
        vacancy: HtmlNameIndexVacancy,
        tag: u32,
        planned: bool,
    },
}

/// How an appended attribute enters the name index.
enum Entering {
    /// The plan's entry for the position is in the index, with this many planned entries still
    /// to append, this one among them.
    Planned(usize),
    /// Its entry goes in this empty slot; `planned` entries of a plan are in the index.
    Vacancy {
        vacancy: HtmlNameIndexVacancy,
        tag: u32,
        planned: usize,
    },
}

impl HtmlConstructionStorage {
    /// The position of `node` when it is an element.
    fn attribute_target(&self, node: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        let position = self.position(node)?;
        match self.arena.records.get(position).map(|record| &record.kind) {
            Some(NodeContent::Element(_)) => Ok(position),
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    fn attribute_slice(&self, target: usize) -> Result<&[Attribute], HtmlConstructionError> {
        match self.arena.records.get(target).map(|record| &record.kind) {
            Some(NodeContent::Element(element)) => {
                Ok(element.attributes.shared_attributes().as_slice())
            }
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }

    fn attribute_dimensions(
        &self,
        target: usize,
    ) -> Result<HtmlConstructionAttributeCapacity, HtmlConstructionError> {
        match self.arena.records.get(target).map(|record| &record.kind) {
            Some(NodeContent::Element(element)) => {
                let backing = element.attributes.shared_attributes();
                Ok(HtmlConstructionAttributeCapacity {
                    initialized_len: backing.len(),
                    capacity: backing.capacity(),
                })
            }
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }

    /// Refuses a list that another owner holds, before anything changes.
    fn require_exclusive_attributes(&self, target: usize) -> Result<(), HtmlConstructionError> {
        match self.arena.records.get(target).map(|record| &record.kind) {
            Some(NodeContent::Element(element)) => {
                let backing = element.attributes.shared_attributes();
                if std::sync::Arc::strong_count(backing) != 1
                    || std::sync::Arc::weak_count(backing) != 0
                {
                    return Err(HtmlConstructionCause::SharedAllocationAliased.into());
                }
                Ok(())
            }
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }

    /// The element's list for a change: the storage's attribute revision advances. Only after
    /// [`Self::require_exclusive_attributes`] passed.
    fn attributes_for_change(
        &mut self,
        target: usize,
    ) -> Result<&mut Vec<Attribute>, HtmlConstructionError> {
        let arena = &mut self.arena;
        match arena.records.get_mut(target).map(|record| &mut record.kind) {
            Some(NodeContent::Element(element)) => std::sync::Arc::get_mut(
                element
                    .attributes
                    .construction_backing_mut(&mut arena.attribute_revision),
            )
            .ok_or_else(|| HtmlConstructionCause::SharedAllocationAliased.into()),
            _ => Err(HtmlConstructionCause::OperationNotReady.into()),
        }
    }

    fn charge_qualification<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
    ) -> Result<(), HtmlConstructionAttributeError<C::Stop>> {
        Work {
            record_visits: 3,
            provider_calls: 1,
            ..Work::default()
        }
        .charge(charge)
    }

    /// What the name index describes now: the first `len` attributes of the element at `target`,
    /// followed by `planned` entries for attributes a merge plan has not appended yet.
    fn described(&self, target: usize, len: usize, planned: usize) -> Binding {
        Binding::Slots {
            storage: self.arena.serial,
            target,
            revision: self.arena.attribute_revision,
            len,
            planned,
        }
    }

    /// Prepares `index` to look names up in the element at `target`, which has `len` attributes
    /// and gets `extra` more: rebuilds it unless it already describes exactly these attributes
    /// and has room, and charges the rebuild. `Ok(false)` when the index cannot hold the names,
    /// so the caller compares every name instead.
    fn prepare_index<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        charge: &mut C,
        index: &mut HtmlAttributeNameIndex,
        target: usize,
        extra: usize,
        planned_ok: bool,
    ) -> Result<bool, HtmlConstructionAttributeError<C::Stop>> {
        let list = self.attribute_slice(target).map_err(faulted)?;
        let len = list.len();
        let Some(entries) = len.checked_add(extra) else {
            return Ok(false);
        };
        if HtmlAttributeNameIndex::slots_required(entries).is_none() {
            return Ok(false);
        }
        if let Binding::Slots {
            storage,
            target: held,
            revision,
            len: held_len,
            planned,
        } = index.binding()
            && storage == self.arena.serial
            && held == target
            && revision == self.arena.attribute_revision
            && held_len == len
            && (planned == 0 || planned_ok)
            && let Some(needed) = len.checked_add(planned).and_then(|n| n.checked_add(extra))
            && HtmlAttributeNameIndex::slots_required(needed).is_some_and(|n| n <= index.size())
        {
            return Ok(true);
        }
        let Some(geometry) = index.geometry_for(entries) else {
            return Ok(false);
        };
        begin_clear_work().charge(charge)?;
        index.begin_clear(geometry);
        while index.unwritten() != 0 {
            clear_work(index).charge(charge)?;
            index.clear_chunk(CLEAR_CHUNK);
            if index.unwritten() == 0 {
                index.bind_storage().map_err(|_| table_error())?;
            }
        }
        for (position, held) in list.iter().enumerate() {
            hash_length_work().charge(charge)?;
            hash_work(hashed_units(held.name())).charge(charge)?;
            let hash = index.hash(held.name());
            let tag = tag(hash);
            let mut probe = index.probe(hash).map_err(|_| table_error())?;
            loop {
                probe_work().charge(charge)?;
                match index.advance(&mut probe).map_err(|_| table_error())? {
                    Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                        let placed = entry(tag, position)
                            .is_some_and(|entry| index.insert(vacancy, entry).is_ok());
                        if !placed {
                            return Err(table_error());
                        }
                        break;
                    }
                    Some(HtmlNameIndexStep::Occupied(_)) => {}
                    None => return Err(table_error()),
                }
            }
        }
        index.bind(self.described(target, len, 0));
        Ok(true)
    }

    /// Looks `name` up through `index`, which describes the element's attributes. An entry for a
    /// position at or beyond the end of `list` names an attribute not appended yet and is
    /// skipped: the later of two equal names finds the earlier one.
    fn index_lookup<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        charge: &mut C,
        index: &mut HtmlAttributeNameIndex,
        list: &[Attribute],
        name: &QualifiedName,
    ) -> Result<Lookup, HtmlConstructionAttributeError<C::Stop>> {
        hash_length_work().charge(charge)?;
        hash_work(hashed_units(name)).charge(charge)?;
        let hash = index.hash(name);
        let tag = tag(hash);
        let mut probe = index.probe(hash).map_err(|_| table_error())?;
        let mut planned = false;
        loop {
            probe_work().charge(charge)?;
            match index.advance(&mut probe).map_err(|_| table_error())? {
                Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                    return Ok(Lookup::Absent {
                        vacancy,
                        tag,
                        planned,
                    });
                }
                Some(HtmlNameIndexStep::Occupied(held)) => {
                    let (position, held_tag) = occupant(held);
                    if held_tag != tag {
                        continue;
                    }
                    if position >= list.len() {
                        planned |= position == list.len();
                        continue;
                    }
                    let existing = list.get(position).ok_or_else(table_error)?;
                    if names_equal(charge, expanded(existing.name()), expanded(name))? {
                        return Ok(Lookup::Found(position));
                    }
                }
                None => return Err(table_error()),
            }
        }
    }

    /// The position of the attribute that has `name`, comparing every name when no index serves.
    fn find_name<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
        list: &[Attribute],
        name: &QualifiedName,
        except: Option<usize>,
    ) -> Result<Option<usize>, HtmlConstructionAttributeError<C::Stop>> {
        for (position, held) in list.iter().enumerate() {
            if Some(position) == except {
                continue;
            }
            if names_equal(charge, expanded(held.name()), expanded(name))? {
                return Ok(Some(position));
            }
        }
        Ok(None)
    }

    /// Grows the list of the element at `target` to at least `minimum` attributes: admits the
    /// growth once, reserves the capacity the consumer chose, and records the capacity reached.
    fn grow_attributes<A: HtmlAttributeAdmission>(
        &mut self,
        target: usize,
        minimum: usize,
        admission: &mut A,
    ) -> Result<HtmlConstructionAttributeCapacity, HtmlConstructionAttributeError<A::Stop>> {
        let before = self.attribute_dimensions(target).map_err(faulted)?;
        minimum
            .checked_mul(size_of::<Attribute>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        self.require_exclusive_attributes(target).map_err(faulted)?;
        let capacity = admission
            .admit_growth(HtmlConstructionAttributeGrowth {
                before,
                minimum_capacity: minimum,
            })
            .map_err(HtmlChargedError::Stopped)?;
        if capacity < minimum {
            return Err(fault(HtmlConstructionCause::CapacityMismatch));
        }
        // The consumer records the capacity before anything is reserved, so that its refusal
        // leaves the list as it was.
        admission
            .record_growth(HtmlConstructionAttributeCapacity {
                initialized_len: before.initialized_len,
                capacity,
            })
            .map_err(HtmlChargedError::Stopped)?;
        let reserve = |storage: &mut Self, additional: usize| match storage
            .arena
            .records
            .get_mut(target)
            .map(|r| &mut r.kind)
        {
            Some(NodeContent::Element(element)) => {
                element.attributes.reserve_exact_in_construction(additional)
            }
            _ => None,
        };
        let needed = minus(minimum, before.initialized_len).map_err(faulted)?;
        let spare = minus(capacity, before.initialized_len).map_err(faulted)?;
        let (mut result, mut reached) = reserve(self, spare)
            .ok_or_else(|| fault(HtmlConstructionCause::SharedAllocationAliased))?;
        // Only an allocator refusal of the spare room falls back to the room that is needed.
        if result.is_err() && capacity > minimum {
            (result, reached) = reserve(self, needed)
                .ok_or_else(|| fault(HtmlConstructionCause::SharedAllocationAliased))?;
        }
        let after = HtmlConstructionAttributeCapacity {
            initialized_len: before.initialized_len,
            capacity: reached,
        };
        // An overshoot, or the unchanged capacity of a reservation the allocator refused, takes
        // the place of the capacity recorded above.
        if reached != capacity {
            admission
                .record_growth(after)
                .map_err(HtmlChargedError::Stopped)?;
        }
        result.map_err(|_| fault(HtmlConstructionCause::AllocationFailed))?;
        Ok(after)
    }

    /// Ensures the list of the element `node` can hold `additional` more attributes, admitting
    /// the growth once when it cannot. Returns the dimensions of the list afterwards.
    pub fn reserve_attributes<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        additional: usize,
        admission: &mut A,
    ) -> Result<HtmlConstructionAttributeCapacity, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let before = self.attribute_dimensions(target).map_err(faulted)?;
        let minimum = plus(before.initialized_len, additional).map_err(faulted)?;
        if minimum <= before.capacity {
            return Ok(before);
        }
        self.grow_attributes(target, minimum, admission)
    }

    /// Appends the attribute in `incoming` to the element `node`, taking it from `incoming`. The
    /// name must not repeat an attribute of the element, or the edit is refused with the
    /// duplicate-attribute cause. A full list grows once, through `admission`.
    ///
    /// With `names`, a duplicate is found by probing the few names that share a hash, so that the
    /// work of appending `n` attributes is linear in `n` and the length of their names. The index
    /// describes the element's attributes while its binding matches them and is rebuilt when it
    /// does not; an index too small for the names is ignored and every name is compared. Returns
    /// the position the attribute took.
    pub fn append_attribute<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        incoming: &mut Option<Attribute>,
        names: Option<&mut HtmlAttributeNameIndex>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        self.append_one(node, incoming, names, None, admission)
    }

    /// As [`append_attribute`](Self::append_attribute) for the attribute a merge plan kept for
    /// `candidate`. The plan compared the candidate's name with the element's attributes and with
    /// the other kept candidates, and left an entry for it in `names`. The name is looked up
    /// through the index like any other, so an attribute whose name the list holds is refused as
    /// a duplicate. When no attribute has the name, the attribute has the candidate's backing,
    /// the index still describes the element as the plan left it and the lookup passed the
    /// plan's entry for the position, that entry stays and nothing more is entered. Any other
    /// attribute leaves the index describing nothing, and the next edit rebuilds it.
    pub fn append_planned_attribute<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        incoming: &mut Option<Attribute>,
        candidate: &DocumentString,
        names: Option<&mut HtmlAttributeNameIndex>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        self.append_one(node, incoming, names, Some(candidate), admission)
    }

    fn append_one<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        incoming: &mut Option<Attribute>,
        names: Option<&mut HtmlAttributeNameIndex>,
        candidate: Option<&DocumentString>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let attribute = incoming
            .as_ref()
            .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
        let before = self.attribute_dimensions(target).map_err(faulted)?;
        let len = before.initialized_len;
        let usage = self.arena.usage;
        let attributes = plus(usage.attributes, 1).map_err(faulted)?;
        let utf16_units = plus(
            usage.utf16_units,
            attribute_units(attribute).map_err(faulted)?,
        )
        .map_err(faulted)?;
        let grown = plus(len, 1).map_err(faulted)?;
        let is_candidate = candidate.is_some_and(|candidate| {
            attribute.name().namespace().uri().is_none()
                && attribute.name().local_name().shares_storage_with(candidate)
        });
        // Every comparison is charged before any change is made.
        let mut indexed = None;
        let mut entering = None;
        if let Some(index) = names
            && self.prepare_index(admission, index, target, usize::from(!is_candidate), true)?
        {
            let planned = match index.binding() {
                Binding::Slots { planned, .. } => planned,
                Binding::Unbound => 0,
            };
            // The lookup runs for a planned attribute too: the candidate is the caller's word, and
            // an attribute whose name an earlier attribute has is refused whatever the plan says.
            let list = self.attribute_slice(target).map_err(faulted)?;
            match self.index_lookup(admission, index, list, attribute.name())? {
                Lookup::Found(first) => return Err(fault_duplicate(first, len)),
                Lookup::Absent {
                    vacancy,
                    tag,
                    planned: entry_seen,
                } => {
                    // The entry the plan left for this position is kept only when the probe
                    // passed an entry at that position with this name's tag.
                    entering = Some(if is_candidate && planned != 0 && entry_seen {
                        Entering::Planned(planned)
                    } else {
                        Entering::Vacancy {
                            vacancy,
                            tag,
                            planned,
                        }
                    });
                }
            }
            indexed = Some(index);
        } else {
            let list = self.attribute_slice(target).map_err(faulted)?;
            if let Some(first) = Self::find_name(admission, list, attribute.name(), None)? {
                return Err(fault_duplicate(first, len));
            }
        }
        self.require_exclusive_attributes(target).map_err(faulted)?;
        Work {
            record_visits: 2,
            copied_bytes: 2 * size_of::<Attribute>(),
            record_writes: 3,
            ..Work::default()
        }
        .charge(admission)?;
        if before.capacity == len {
            self.grow_attributes(target, grown, admission)?;
        }
        let Some(attribute) = incoming.take() else {
            return Err(fault(HtmlConstructionCause::OperationNotReady));
        };
        self.attributes_for_change(target)
            .map_err(faulted)?
            .push(attribute);
        self.arena.usage.attributes = attributes;
        self.arena.usage.utf16_units = utf16_units;
        if let Some(index) = indexed {
            // Nothing but this push has changed the attributes or the table since the lookup, so
            // the vacancy is still the slot the name's probe ended at.
            let binding = match entering {
                Some(Entering::Planned(planned)) => {
                    self.described(target, grown, planned.saturating_sub(1))
                }
                Some(Entering::Vacancy {
                    vacancy,
                    tag,
                    planned: 0,
                }) => {
                    let placed =
                        entry(tag, len).is_some_and(|entry| index.insert(vacancy, entry).is_ok());
                    if placed {
                        self.described(target, grown, 0)
                    } else {
                        Binding::Unbound
                    }
                }
                // The attribute is not the one the plan meant to append, so the planned entries
                // no longer name the attributes that follow.
                _ => Binding::Unbound,
            };
            index.bind(binding);
        }
        Ok(len)
    }

    /// Replaces the attribute at `index` of the element `node` with `incoming`, which is taken,
    /// and returns the attribute it replaced. The replacement's name must not repeat another
    /// attribute of the element.
    pub fn replace_attribute<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        index: usize,
        incoming: &mut Option<Attribute>,
        admission: &mut A,
    ) -> Result<Attribute, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        let old = list
            .get(index)
            .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
        let new = incoming
            .as_ref()
            .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
        let usage = self.arena.usage;
        let utf16_units = plus(
            minus(usage.utf16_units, attribute_units(old).map_err(faulted)?).map_err(faulted)?,
            attribute_units(new).map_err(faulted)?,
        )
        .map_err(faulted)?;
        if !identical_name(old.name(), new.name())
            && let Some(first) = Self::find_name(admission, list, new.name(), Some(index))?
        {
            return Err(fault_duplicate(first, index));
        }
        self.require_exclusive_attributes(target).map_err(faulted)?;
        Work {
            record_visits: 2,
            copied_bytes: 2 * size_of::<Attribute>(),
            record_writes: 3,
            ..Work::default()
        }
        .charge(admission)?;
        let Some(attribute) = incoming.take() else {
            return Err(fault(HtmlConstructionCause::OperationNotReady));
        };
        let slot = self
            .attributes_for_change(target)
            .map_err(faulted)?
            .get_mut(index)
            .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
        let old = std::mem::replace(slot, attribute);
        self.arena.usage.utf16_units = utf16_units;
        Ok(old)
    }

    /// Removes the attribute at `index` of the element `node`, shifting the attributes after it
    /// down by one place, and returns it. The shift is charged before it runs, in quanta.
    pub fn remove_attribute<A: HtmlAttributeAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        index: usize,
        admission: &mut A,
    ) -> Result<Attribute, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        let old = list
            .get(index)
            .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
        let usage = self.arena.usage;
        let attributes = minus(usage.attributes, 1).map_err(faulted)?;
        let utf16_units =
            minus(usage.utf16_units, attribute_units(old).map_err(faulted)?).map_err(faulted)?;
        let moved = minus(list.len(), plus(index, 1).map_err(faulted)?).map_err(faulted)?;
        self.require_exclusive_attributes(target).map_err(faulted)?;
        let mut remaining = moved;
        let per_quantum = (CHARGE_QUANTUM / size_of::<Attribute>()).max(1);
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            Work {
                record_visits: 1,
                copied_bytes: chunk.saturating_mul(size_of::<Attribute>()),
                record_writes: chunk,
                ..Work::default()
            }
            .charge(admission)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Work {
            record_visits: 1,
            copied_bytes: size_of::<Attribute>(),
            record_writes: 2,
            ..Work::default()
        }
        .charge(admission)?;
        let removed = self.attributes_for_change(target).map_err(faulted)?;
        if index >= removed.len() {
            return Err(fault(HtmlConstructionCause::OperationNotReady));
        }
        let old = removed.remove(index);
        self.arena.usage.attributes = attributes;
        self.arena.usage.utf16_units = utf16_units;
        Ok(old)
    }

    /// Decides which candidates `merge` offers are kept: a candidate is kept when no attribute of
    /// the element `node` and no earlier kept candidate has its name, so the first of equal names
    /// wins. Each kept candidate is reported to `merge` in order. Returns how many are kept.
    /// Nothing changes: the candidates are appended one at a time with
    /// [`append_attribute`](Self::append_attribute), passing the same `names`.
    ///
    /// With `names`, the plan is linear in the attributes and candidates: the index also holds an
    /// entry for each kept candidate, at the position it will take, so the appends of the plan find
    /// the index already describing them.
    pub fn plan_attribute_merge<M: HtmlAttributeMerge>(
        &self,
        node: &HtmlBuilderNode,
        names: Option<&mut HtmlAttributeNameIndex>,
        merge: &mut M,
    ) -> Result<usize, HtmlConstructionAttributeError<M::Stop>> {
        Self::charge_qualification(merge)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        let count = merge.candidates();
        let len = list.len();
        let mut kept = 0;
        let indexed = match names {
            Some(index) => self
                .prepare_index(merge, index, target, count, false)?
                .then_some(index),
            None => None,
        };
        let Some(index) = indexed else {
            for candidate in 0..count {
                let name = merge
                    .name(candidate)
                    .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
                let mut duplicate = false;
                for held in list {
                    if names_equal(merge, expanded(held.name()), (None, &name))? {
                        duplicate = true;
                        break;
                    }
                }
                for rank in 0..kept {
                    if duplicate {
                        break;
                    }
                    let earlier = merge
                        .kept(rank)
                        .and_then(|earlier| merge.name(earlier))
                        .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
                    duplicate = names_equal(merge, (None, &earlier), (None, &name))?;
                }
                if !duplicate {
                    merge.keep(candidate).map_err(HtmlChargedError::Stopped)?;
                    kept += 1;
                }
            }
            return Ok(kept);
        };
        for candidate in 0..count {
            let name = merge
                .name(candidate)
                .ok_or_else(|| fault(HtmlConstructionCause::OperationNotReady))?;
            hash_length_work().charge(merge)?;
            hash_work(name.len()).charge(merge)?;
            let hash = index.hash_local(name.code_units());
            let tag = tag(hash);
            let mut probe = index.probe(hash).map_err(|_| table_error())?;
            let vacancy = loop {
                probe_work().charge(merge)?;
                match index.advance(&mut probe).map_err(|_| table_error())? {
                    Some(HtmlNameIndexStep::Vacant(vacancy)) => break Some(vacancy),
                    Some(HtmlNameIndexStep::Occupied(held)) => {
                        let (position, held_tag) = occupant(held);
                        if held_tag != tag {
                            continue;
                        }
                        let equal = if position < len {
                            let existing = list.get(position).ok_or_else(table_error)?;
                            names_equal(merge, expanded(existing.name()), (None, &name))?
                        } else {
                            let earlier = merge
                                .kept(minus(position, len).map_err(faulted)?)
                                .and_then(|earlier| merge.name(earlier))
                                .ok_or_else(table_error)?;
                            names_equal(merge, (None, &earlier), (None, &name))?
                        };
                        if equal {
                            break None;
                        }
                    }
                    None => return Err(table_error()),
                }
            };
            if let Some(vacancy) = vacancy {
                merge.keep(candidate).map_err(HtmlChargedError::Stopped)?;
                let placed = plus(len, kept)
                    .ok()
                    .and_then(|position| entry(tag, position))
                    .is_some_and(|entry| index.insert(vacancy, entry).is_ok());
                if !placed {
                    return Err(table_error());
                }
                kept += 1;
                index.bind(self.described(target, len, kept));
            }
        }
        Ok(kept)
    }
}

fn fault_duplicate<S>(first: usize, duplicate: usize) -> HtmlConstructionAttributeError<S> {
    fault(HtmlConstructionCause::DuplicateAttribute { first, duplicate })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::construction::bounded::operations::HtmlConstructionCommit;
    use crate::internal::values::{Namespace, OrderedAttributes};
    use std::collections::HashSet;

    /// The refusal of a [`Meter`].
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Refused;

    /// A consumer that records what it is asked, and refuses at a chosen charge or growth.
    #[derive(Default)]
    struct Meter {
        charges: usize,
        refuse_from_charge: Option<usize>,
        refuse_growth: bool,
        /// Refuses the record of a growth whose place among the records is at least this.
        refuse_record_from: Option<usize>,
        spare: usize,
        work: u64,
        largest_copy: u64,
        growths: Vec<HtmlConstructionAttributeGrowth>,
        recorded: Vec<HtmlConstructionAttributeCapacity>,
    }
    impl Charge<HtmlWork, HtmlResource> for Meter {
        type Stop = Refused;
        fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            self.charges += 1;
            if self
                .refuse_from_charge
                .is_some_and(|from| self.charges > from)
            {
                return Err(Refused);
            }
            for change in batch {
                if let Change::Work(work, amount) = *change {
                    self.work += amount;
                    if work == HtmlWork::CopyByte {
                        self.largest_copy = self.largest_copy.max(amount);
                    }
                }
            }
            Ok(())
        }
        fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
            Ok(())
        }
    }
    impl HtmlAttributeAdmission for Meter {
        fn admit_growth(
            &mut self,
            growth: HtmlConstructionAttributeGrowth,
        ) -> Result<usize, Refused> {
            self.growths.push(growth);
            if self.refuse_growth {
                return Err(Refused);
            }
            Ok(growth.minimum_capacity() + self.spare)
        }
        fn record_growth(
            &mut self,
            reached: HtmlConstructionAttributeCapacity,
        ) -> Result<(), Refused> {
            if self
                .refuse_record_from
                .is_some_and(|from| self.recorded.len() >= from)
            {
                return Err(Refused);
            }
            self.recorded.push(reached);
            Ok(())
        }
    }

    /// Candidates that are plain local names, and where the decisions go.
    struct Candidates<'a> {
        meter: Meter,
        names: &'a [DocumentString],
        kept: Vec<usize>,
    }
    impl Charge<HtmlWork, HtmlResource> for Candidates<'_> {
        type Stop = Refused;
        fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            self.meter.charge(batch)
        }
        fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), Refused> {
            self.meter.record_largest(resource, value)
        }
    }
    impl HtmlAttributeMerge for Candidates<'_> {
        fn candidates(&self) -> usize {
            self.names.len()
        }
        fn name(&self, index: usize) -> Option<DocumentString> {
            self.names.get(index).cloned()
        }
        fn keep(&mut self, index: usize) -> Result<(), Refused> {
            self.kept.push(index);
            Ok(())
        }
        fn kept(&self, rank: usize) -> Option<usize> {
            self.kept.get(rank).copied()
        }
    }

    fn attribute(name: impl Into<DocumentString>, value: &str) -> Attribute {
        Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
    }
    fn names(texts: &[&str]) -> Vec<DocumentString> {
        texts
            .iter()
            .map(|text| DocumentString::from(*text))
            .collect()
    }
    fn make_storage(attributes: Vec<Attribute>) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        while let Some(action) = initialization.as_mut().unwrap().next_action().unwrap() {
            action.perform().accept();
        }
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        add_element(storage, "root", attributes)
    }
    fn add_element(
        storage: HtmlConstructionStorage,
        name: &str,
        attributes: Vec<Attribute>,
    ) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut operation = Some(storage.prepare(HtmlConstructionCommand::CreateElement {
            name: QualifiedName::new(Namespace::none(), None, name),
            attributes: OrderedAttributes::new(attributes).unwrap(),
            link_flags: None,
        }));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        let HtmlConstructionCommit::Changed { storage, created } =
            slots::operation(&mut operation).unwrap().commit().unwrap()
        else {
            panic!("create")
        };
        (storage, created.unwrap())
    }
    fn names_of(storage: &HtmlConstructionStorage, position: usize) -> Vec<String> {
        storage
            .attribute_slice(position)
            .unwrap()
            .iter()
            .map(|held| held.name().local_name().to_utf8().unwrap())
            .collect()
    }
    /// What an edit must leave alone when it is refused.
    fn snapshot(
        storage: &HtmlConstructionStorage,
    ) -> (Vec<Vec<(String, String)>>, HtmlConstructionUsage, u64) {
        let lists = (0..storage.len())
            .map(|position| {
                storage
                    .attribute_slice(position)
                    .unwrap()
                    .iter()
                    .map(|held| {
                        (
                            held.name().local_name().to_utf8().unwrap(),
                            held.value().to_utf8().unwrap(),
                        )
                    })
                    .collect()
            })
            .collect();
        (lists, storage.usage(), storage.arena.attribute_revision)
    }
    fn index(entries: usize) -> HtmlAttributeNameIndex {
        let slots = HtmlAttributeNameIndex::slots_required(entries).unwrap();
        HtmlAttributeNameIndex::new(Vec::with_capacity(slots))
    }
    fn cause(error: HtmlConstructionAttributeError<Refused>) -> HtmlConstructionCause {
        match error {
            HtmlChargedError::Faulted(error) => error.cause(),
            HtmlChargedError::Stopped(Refused) => panic!("the consumer refused"),
        }
    }

    #[test]
    fn an_append_adds_the_incoming_attribute_and_counts_it() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "22")]);
        let before = storage.usage();
        let mut incoming = Some(attribute("c", "333"));
        let mut meter = Meter::default();
        let position = storage
            .append_attribute(&node, &mut incoming, None, &mut meter)
            .unwrap();
        assert_eq!(position, 2);
        assert_eq!(names_of(&storage, 0), ["a", "b", "c"]);
        assert!(incoming.is_none(), "the appended attribute is taken");
        // One attribute of a one-unit name and a three-unit value.
        assert_eq!(storage.usage().attributes(), before.attributes() + 1);
        assert_eq!(storage.usage().utf16_units(), before.utf16_units() + 4);
    }

    #[test]
    fn a_qualified_attribute_counts_its_namespace_prefix_name_and_value_units() {
        let (mut storage, node) = make_storage(Vec::new());
        let name = QualifiedName::new(
            Namespace::from_uri("urn:x"),
            Some("pre".into()),
            DocumentString::from("loc"),
        );
        let mut incoming = Some(Attribute::new(name, "value"));
        let before = storage.usage().utf16_units();
        storage
            .append_attribute(&node, &mut incoming, None, &mut Meter::default())
            .unwrap();
        assert_eq!(storage.usage().utf16_units(), before + 5 + 3 + 3 + 5);
    }

    #[test]
    fn an_append_of_a_repeated_expanded_name_is_refused_and_changes_nothing() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let before = snapshot(&storage);
        // A prefix and a value do not make a name new.
        let repeated = Attribute::new(
            QualifiedName::new(
                Namespace::none(),
                Some("p".into()),
                DocumentString::from("b"),
            ),
            "other",
        );
        let mut incoming = Some(repeated);
        let error = storage
            .append_attribute(&node, &mut incoming, None, &mut Meter::default())
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 2
            }
        );
        assert_eq!(snapshot(&storage), before);
        assert!(
            incoming.is_some(),
            "the incoming attribute stays with the caller"
        );
        // The same local name in another namespace is another name.
        let other = Attribute::new(
            QualifiedName::new(
                Namespace::from_uri("urn:x"),
                None,
                DocumentString::from("b"),
            ),
            "v",
        );
        storage
            .append_attribute(&node, &mut Some(other), None, &mut Meter::default())
            .unwrap();
        assert_eq!(names_of(&storage, 0), ["a", "b", "b"]);
    }

    #[test]
    fn a_full_list_grows_once_and_is_told_the_capacity_it_needs() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = storage.attribute_dimensions(0).unwrap();
        let mut meter = Meter {
            spare: 3,
            ..Meter::default()
        };
        storage
            .append_attribute(&node, &mut Some(attribute("b", "2")), None, &mut meter)
            .unwrap();
        assert_eq!(meter.growths.len(), 1);
        assert_eq!(meter.growths[0].before(), before);
        assert_eq!(
            meter.growths[0].minimum_capacity(),
            before.initialized_len + 1
        );
        assert_eq!(meter.recorded.len(), 1);
        assert!(meter.recorded[0].capacity >= before.initialized_len + 1 + 3);
        // The spare room the consumer chose absorbs the next appends without another growth.
        for name in ["c", "d", "e"] {
            storage
                .append_attribute(&node, &mut Some(attribute(name, "v")), None, &mut meter)
                .unwrap();
        }
        assert_eq!(meter.growths.len(), 1);
        assert_eq!(names_of(&storage, 0), ["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn an_admission_below_the_minimum_is_refused_before_any_reservation() {
        struct Stingy(Meter);
        impl Charge<HtmlWork, HtmlResource> for Stingy {
            type Stop = Refused;
            fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
                self.0.charge(batch)
            }
            fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), Refused> {
                Ok(())
            }
        }
        impl HtmlAttributeAdmission for Stingy {
            fn admit_growth(
                &mut self,
                g: HtmlConstructionAttributeGrowth,
            ) -> Result<usize, Refused> {
                Ok(g.minimum_capacity() - 1)
            }
            fn record_growth(
                &mut self,
                _: HtmlConstructionAttributeCapacity,
            ) -> Result<(), Refused> {
                panic!("nothing was reserved")
            }
        }
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let error = storage
            .append_attribute(
                &node,
                &mut Some(attribute("b", "2")),
                None,
                &mut Stingy(Meter::default()),
            )
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::CapacityMismatch);
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_refusal_at_any_admission_point_of_an_append_changes_nothing() {
        // Every charge and the growth are admission points; the edit passes once none refuses.
        let mut points = 0;
        for refuse_from in 0.. {
            let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
            let before = snapshot(&storage);
            let mut incoming = Some(attribute("b", "2"));
            let mut meter = Meter {
                refuse_from_charge: Some(refuse_from),
                ..Meter::default()
            };
            let result = storage.append_attribute(&node, &mut incoming, None, &mut meter);
            if result.is_ok() {
                assert_eq!(names_of(&storage, 0), ["a", "b"]);
                break;
            }
            points += 1;
            assert_eq!(
                snapshot(&storage),
                before,
                "refused after {refuse_from} charges"
            );
            assert!(incoming.is_some());
        }
        assert!(points >= 2, "the append makes several admission points");
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let mut meter = Meter {
            refuse_growth: true,
            ..Meter::default()
        };
        let error = storage
            .append_attribute(&node, &mut Some(attribute("b", "2")), None, &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_list_that_another_owner_holds_is_refused_before_it_changes() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let held = match storage.node_at(0).unwrap().kind() {
            ConstructionNodeKind::Element(element) => element.attributes().to_ordered(),
            _ => unreachable!(),
        };
        let before = snapshot(&storage);
        let error = storage
            .append_attribute(
                &node,
                &mut Some(attribute("b", "2")),
                None,
                &mut Meter::default(),
            )
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::SharedAllocationAliased);
        assert_eq!(snapshot(&storage), before);
        drop(held);
    }

    #[test]
    fn a_target_that_is_not_an_element_or_not_this_storage_is_refused() {
        let (mut storage, _) = make_storage(Vec::new());
        // The initialization creates no record; a foreign handle and a missing record are refused.
        let (other, foreign) = make_storage(Vec::new());
        let error = storage
            .append_attribute(
                &foreign,
                &mut Some(attribute("a", "1")),
                None,
                &mut Meter::default(),
            )
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::ForeignBuilder);
        drop(other);
    }

    #[test]
    fn a_replacement_returns_the_old_attribute_and_updates_the_units() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "22")]);
        let units = storage.usage().utf16_units();
        let mut incoming = Some(attribute("a", "4444"));
        let old = storage
            .replace_attribute(&node, 0, &mut incoming, &mut Meter::default())
            .unwrap();
        assert_eq!(old.value().to_utf8().unwrap(), "1");
        assert!(incoming.is_none());
        assert_eq!(storage.usage().utf16_units(), units - 1 + 4);
        assert_eq!(storage.usage().attributes(), 2);
        // The same name under a new prefix keeps the slot too.
        let mut renamed = Some(attribute("c", "v"));
        let old = storage
            .replace_attribute(&node, 1, &mut renamed, &mut Meter::default())
            .unwrap();
        assert_eq!(old.name().local_name().to_utf8().unwrap(), "b");
        assert_eq!(names_of(&storage, 0), ["a", "c"]);
    }

    #[test]
    fn a_replacement_that_repeats_another_name_or_misses_its_slot_is_refused() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let before = snapshot(&storage);
        let mut incoming = Some(attribute("b", "x"));
        let error = storage
            .replace_attribute(&node, 0, &mut incoming, &mut Meter::default())
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 0
            }
        );
        assert!(incoming.is_some());
        let error = storage
            .replace_attribute(&node, 2, &mut incoming, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::OperationNotReady);
        let error = storage
            .replace_attribute(&node, 0, &mut None, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::OperationNotReady);
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_refusal_at_any_admission_point_of_a_replacement_changes_nothing() {
        for refuse_from in 0.. {
            let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
            let before = snapshot(&storage);
            let mut incoming = Some(attribute("c", "3"));
            let mut meter = Meter {
                refuse_from_charge: Some(refuse_from),
                ..Meter::default()
            };
            if storage
                .replace_attribute(&node, 0, &mut incoming, &mut meter)
                .is_ok()
            {
                assert_eq!(names_of(&storage, 0), ["c", "b"]);
                assert!(refuse_from > 0);
                break;
            }
            assert_eq!(snapshot(&storage), before);
            assert!(incoming.is_some());
        }
    }

    #[test]
    fn a_removal_closes_the_gap_and_gives_the_attribute_back() {
        let (mut storage, node) = make_storage(
            ["a", "b", "c", "d"]
                .iter()
                .map(|name| attribute(*name, "vv"))
                .collect(),
        );
        let before = storage.usage();
        let removed = storage
            .remove_attribute(&node, 1, &mut Meter::default())
            .unwrap();
        assert_eq!(removed.name().local_name().to_utf8().unwrap(), "b");
        assert_eq!(names_of(&storage, 0), ["a", "c", "d"]);
        assert_eq!(storage.usage().attributes(), before.attributes() - 1);
        assert_eq!(storage.usage().utf16_units(), before.utf16_units() - 3);
        let error = storage
            .remove_attribute(&node, 3, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::OperationNotReady);
    }

    #[test]
    fn a_removal_charges_the_shift_in_quanta_before_it_moves_anything() {
        let attributes: Vec<_> = (0..40)
            .map(|n| attribute(format!("n{n}").as_str(), "v"))
            .collect();
        let (mut storage, node) = make_storage(attributes);
        let mut meter = Meter::default();
        storage.remove_attribute(&node, 0, &mut meter).unwrap();
        // 39 attributes move; no charge covers more than the 256 byte quantum or the one
        // attribute that leaves.
        assert!(meter.largest_copy <= CHARGE_QUANTUM as u64);
        assert!(meter.work >= (39 * size_of::<Attribute>()) as u64);
        // A refusal after the first shift charge leaves the list intact.
        let before = snapshot(&storage);
        let mut refusing = Meter {
            refuse_from_charge: Some(3),
            ..Meter::default()
        };
        assert!(storage.remove_attribute(&node, 0, &mut refusing).is_err());
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_merge_keeps_the_first_of_equal_names_against_the_element_and_earlier_candidates() {
        for indexed in [false, true] {
            let (storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
            let texts = names(&["b", "c", "c", "d", "a", "e"]);
            let mut merge = Candidates {
                meter: Meter::default(),
                names: &texts,
                kept: Vec::new(),
            };
            let mut table = index(16);
            let kept = storage
                .plan_attribute_merge(&node, indexed.then_some(&mut table), &mut merge)
                .unwrap();
            assert_eq!(merge.kept, [1, 3, 5], "indexed: {indexed}");
            assert_eq!(kept, 3);
        }
    }

    #[test]
    fn a_planned_merge_appends_without_comparing_and_leaves_the_index_describing_the_element() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let texts = names(&["b", "c", "d"]);
        let mut table = index(32);
        let mut merge = Candidates {
            meter: Meter::default(),
            names: &texts,
            kept: Vec::new(),
        };
        storage
            .plan_attribute_merge(&node, Some(&mut table), &mut merge)
            .unwrap();
        assert_eq!(merge.kept, [1, 2]);
        let mut meter = Meter::default();
        for &candidate in &merge.kept {
            let mut incoming = Some(attribute(texts[candidate].clone(), "v"));
            storage
                .append_planned_attribute(
                    &node,
                    &mut incoming,
                    &texts[candidate],
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        assert_eq!(names_of(&storage, 0), ["a", "b", "c", "d"]);
        assert_eq!(
            table.binding(),
            storage.described(0, 4, 0),
            "the index describes the four attributes at the revision they left"
        );
        // A later append is compared through the same index, without a rebuild.
        let mut later = Meter::default();
        let error = storage
            .append_attribute(
                &node,
                &mut Some(attribute("c", "x")),
                Some(&mut table),
                &mut later,
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 2,
                duplicate: 4
            }
        );
        assert_eq!(table.binding(), storage.described(0, 4, 0));
    }

    #[test]
    fn an_attribute_other_than_the_planned_one_is_compared_and_drops_the_plan() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let texts = names(&["b", "c"]);
        let mut table = index(32);
        let mut merge = Candidates {
            meter: Meter::default(),
            names: &texts,
            kept: Vec::new(),
        };
        storage
            .plan_attribute_merge(&node, Some(&mut table), &mut merge)
            .unwrap();
        // The element gets an attribute the plan did not mean, with a name the plan planned later.
        let mut meter = Meter::default();
        storage
            .append_planned_attribute(
                &node,
                &mut Some(attribute("c", "x")),
                &texts[0],
                Some(&mut table),
                &mut meter,
            )
            .unwrap();
        assert_eq!(table.binding(), Binding::Unbound);
        // The planned "c" now repeats the attribute that was appended, and is refused.
        let error = storage
            .append_planned_attribute(
                &node,
                &mut Some(attribute("c", "y")),
                &texts[1],
                Some(&mut table),
                &mut meter,
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 2
            }
        );
    }

    /// A model of the attribute names an element holds, for checking the list against.
    fn model_outcome(held: &mut Vec<String>, name: &str) -> Result<usize, usize> {
        match held.iter().position(|held| held == name) {
            Some(first) => Err(first),
            None => {
                held.push(name.to_owned());
                Ok(held.len() - 1)
            }
        }
    }

    #[test]
    fn indexed_and_pairwise_appends_agree_with_a_set_of_names_through_edits() {
        // A deterministic generator over a small alphabet, so that names repeat often.
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for indexed in [false, true] {
            let (mut storage, node) = make_storage(Vec::new());
            let mut table = index(128);
            let mut held: Vec<String> = Vec::new();
            for step in 0..600 {
                let name = format!("n{}", next(90));
                let action = next(10);
                if action == 0 && !held.is_empty() {
                    let at = usize::try_from(next(held.len() as u64)).unwrap();
                    storage
                        .remove_attribute(&node, at, &mut Meter::default())
                        .unwrap();
                    held.remove(at);
                    continue;
                }
                if action == 1 && !held.is_empty() {
                    let at = usize::try_from(next(held.len() as u64)).unwrap();
                    let mut incoming = Some(attribute(name.as_str(), "r"));
                    let outcome =
                        storage.replace_attribute(&node, at, &mut incoming, &mut Meter::default());
                    let expected = held
                        .iter()
                        .enumerate()
                        .find(|(position, held)| *position != at && **held == name)
                        .map(|(position, _)| position);
                    match (outcome, expected) {
                        (Ok(_), None) => held[at] = name,
                        (Err(error), Some(first)) => assert_eq!(
                            cause(error),
                            HtmlConstructionCause::DuplicateAttribute {
                                first,
                                duplicate: at
                            }
                        ),
                        other => panic!("step {step}: replace disagrees with the model: {other:?}"),
                    }
                    continue;
                }
                let mut incoming = Some(attribute(name.as_str(), "v"));
                let outcome = storage.append_attribute(
                    &node,
                    &mut incoming,
                    indexed.then_some(&mut table),
                    &mut Meter {
                        spare: 4,
                        ..Meter::default()
                    },
                );
                match (outcome, model_outcome(&mut held, &name)) {
                    (Ok(position), Ok(expected)) => assert_eq!(position, expected),
                    (Err(error), Err(first)) => assert_eq!(
                        cause(error),
                        HtmlConstructionCause::DuplicateAttribute {
                            first,
                            duplicate: held.len()
                        }
                    ),
                    other => panic!("step {step}: append disagrees with the model: {other:?}"),
                }
            }
            assert_eq!(names_of(&storage, 0), held, "indexed: {indexed}");
        }
    }

    #[test]
    fn indexed_merge_plans_decide_as_a_set_of_names_does_on_random_candidates() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        for round in 0..40 {
            let existing: Vec<String> = (0..next(20))
                .map(|n| format!("e{n}-{}", next(3)))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            let (storage, node) = make_storage(
                existing
                    .iter()
                    .map(|name| attribute(name.as_str(), "v"))
                    .collect(),
            );
            let texts: Vec<String> = (0..next(30))
                .map(|_| {
                    if next(2) == 0 {
                        existing
                            .get(next(7) as usize)
                            .cloned()
                            .unwrap_or_else(|| "x".into())
                    } else {
                        format!("e{}-{}", next(25), next(3))
                    }
                })
                .collect();
            let documents: Vec<DocumentString> = texts
                .iter()
                .map(|text| DocumentString::from(text.as_str()))
                .collect();
            let mut seen: HashSet<&str> = existing.iter().map(String::as_str).collect();
            let expected: Vec<usize> = texts
                .iter()
                .enumerate()
                .filter(|(_, text)| seen.insert(text.as_str()))
                .map(|(index, _)| index)
                .collect();
            for indexed in [false, true] {
                let mut table = index(64);
                let mut merge = Candidates {
                    meter: Meter::default(),
                    names: &documents,
                    kept: Vec::new(),
                };
                storage
                    .plan_attribute_merge(&node, indexed.then_some(&mut table), &mut merge)
                    .unwrap();
                assert_eq!(merge.kept, expected, "round {round}, indexed: {indexed}");
            }
        }
    }

    /// The work of `n` appends of distinct names through an index, in total.
    fn appends_through_an_index(n: usize) -> u64 {
        let (mut storage, node) = make_storage(Vec::new());
        let mut table = index(n + 8);
        let mut meter = Meter {
            spare: n,
            ..Meter::default()
        };
        for name in 0..n {
            storage
                .append_attribute(
                    &node,
                    &mut Some(attribute(format!("n{name}").as_str(), "v")),
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        meter.work
    }
    /// The work of `n` merges of one new name each into one element, in total.
    fn merges_of_one_name(n: usize) -> u64 {
        let (mut storage, node) = make_storage(Vec::new());
        let mut table = index(2 * n + 8);
        let mut meter = Meter {
            spare: n,
            ..Meter::default()
        };
        let mut planning = 0;
        for name in 0..n {
            let texts = [DocumentString::from(format!("n{name}").as_str())];
            let mut merge = Candidates {
                meter: Meter::default(),
                names: &texts,
                kept: Vec::new(),
            };
            storage
                .plan_attribute_merge(&node, Some(&mut table), &mut merge)
                .unwrap();
            planning += merge.meter.work;
            assert_eq!(merge.kept, [0]);
            storage
                .append_planned_attribute(
                    &node,
                    &mut Some(attribute(texts[0].clone(), "v")),
                    &texts[0],
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        meter.work + planning
    }
    fn assert_linear(measure: impl Fn(usize) -> u64) {
        let costs: Vec<u64> = [100, 200, 400, 800].iter().map(|&n| measure(n)).collect();
        for pair in costs.windows(2) {
            // A doubling of the appends at most doubles the work, plus 10 percent.
            assert!(
                u128::from(pair[1]) * 10 <= u128::from(pair[0]) * 22,
                "work grew from {} to {} for twice the appends",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn appends_through_an_index_cost_work_in_proportion_to_their_number() {
        assert_linear(appends_through_an_index);
    }

    #[test]
    fn repeated_one_name_merges_cost_work_in_proportion_to_their_number() {
        assert_linear(merges_of_one_name);
    }

    #[test]
    fn an_index_that_served_another_edit_is_rebuilt_before_it_is_trusted() {
        let (storage, first) = make_storage(vec![attribute("a", "1")]);
        let (mut storage, second) = add_element(storage, "other", vec![attribute("b", "2")]);
        let mut table = index(16);
        let mut meter = Meter {
            spare: 4,
            ..Meter::default()
        };
        storage
            .append_attribute(
                &first,
                &mut Some(attribute("c", "v")),
                Some(&mut table),
                &mut meter,
            )
            .unwrap();
        // The index describes the first element; the second has no "c" and must accept it.
        storage
            .append_attribute(
                &second,
                &mut Some(attribute("c", "v")),
                Some(&mut table),
                &mut meter,
            )
            .unwrap();
        // Back on the first element the index describes the second, and "b" is new there.
        storage
            .append_attribute(
                &first,
                &mut Some(attribute("b", "v")),
                Some(&mut table),
                &mut meter,
            )
            .unwrap();
        let error = storage
            .append_attribute(
                &first,
                &mut Some(attribute("c", "v")),
                Some(&mut table),
                &mut meter,
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 3
            }
        );
    }

    #[test]
    fn an_index_that_is_too_small_is_ignored_and_every_name_is_compared() {
        let (mut storage, node) = make_storage(Vec::new());
        let mut table = HtmlAttributeNameIndex::new(Vec::with_capacity(2));
        let mut meter = Meter {
            spare: 64,
            ..Meter::default()
        };
        for n in 0..40 {
            storage
                .append_attribute(
                    &node,
                    &mut Some(attribute(format!("n{n}").as_str(), "v")),
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        let error = storage
            .append_attribute(
                &node,
                &mut Some(attribute("n17", "v")),
                Some(&mut table),
                &mut meter,
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 17,
                duplicate: 40
            }
        );
    }

    #[test]
    fn reserving_grows_the_list_only_when_it_is_short() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let mut meter = Meter::default();
        let held = storage.reserve_attributes(&node, 3, &mut meter).unwrap();
        assert!(held.capacity >= 4);
        assert_eq!(meter.growths.len(), 1);
        assert_eq!(meter.growths[0].minimum_capacity(), 4);
        let again = storage.reserve_attributes(&node, 3, &mut meter).unwrap();
        assert_eq!(again, held);
        assert_eq!(
            meter.growths.len(),
            1,
            "room that is there is not admitted again"
        );
        assert_eq!(names_of(&storage, 0), ["a"]);
    }

    /// The most attributes whose list layout is below the largest an allocation can have, which no
    /// allocator can supply.
    fn unallocatable() -> usize {
        isize::MAX as usize / size_of::<Attribute>() - 16
    }

    fn weak_of_backing(
        storage: &HtmlConstructionStorage,
    ) -> (std::sync::Weak<Vec<Attribute>>, usize, usize) {
        match storage.node_at(0).unwrap().kind() {
            ConstructionNodeKind::Element(element) => {
                let ordered = element.attributes().to_ordered();
                let backing = ordered.shared_attributes();
                (
                    std::sync::Arc::downgrade(backing),
                    backing.as_ptr() as usize,
                    backing.capacity(),
                )
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn a_planned_append_of_a_name_the_list_holds_is_refused_as_a_duplicate() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let texts = names(&["b", "c", "d"]);
        let mut table = index(32);
        let mut merge = Candidates {
            meter: Meter::default(),
            names: &texts,
            kept: Vec::new(),
        };
        storage
            .plan_attribute_merge(&node, Some(&mut table), &mut merge)
            .unwrap();
        assert_eq!(
            table.binding(),
            storage.described(0, 2, 2),
            "the plan left entries for `c` and `d`"
        );
        // A fresh string named like the first attribute, offered as the candidate it is.
        let fresh = DocumentString::from("a");
        let before = snapshot(&storage);
        let mut incoming = Some(attribute(fresh.clone(), "again"));
        let error = storage
            .append_planned_attribute(
                &node,
                &mut incoming,
                &fresh,
                Some(&mut table),
                &mut Meter::default(),
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 0,
                duplicate: 2
            }
        );
        assert!(incoming.is_some());
        assert_eq!(snapshot(&storage), before);
        assert_eq!(names_of(&storage, 0), ["a", "b"]);
    }

    #[test]
    fn a_refused_first_record_of_a_growth_leaves_the_list_as_it_was() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let (_, pointer, capacity) = weak_of_backing(&storage);
        let mut meter = Meter {
            refuse_record_from: Some(0),
            ..Meter::default()
        };
        let error = storage
            .reserve_attributes(&node, 4, &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        assert_eq!(snapshot(&storage), before);
        assert!(meter.recorded.is_empty());
        // Nothing was reserved: the backing is the same, with the capacity it had.
        let (_, held, held_capacity) = weak_of_backing(&storage);
        assert_eq!((held, held_capacity), (pointer, capacity));
        let mut meter = Meter {
            refuse_record_from: Some(0),
            ..Meter::default()
        };
        assert_eq!(
            storage
                .append_attribute(&node, &mut Some(attribute("b", "2")), None, &mut meter)
                .unwrap_err(),
            HtmlChargedError::Stopped(Refused)
        );
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn an_allocator_refusal_gives_the_unchanged_capacity_back_to_the_consumer() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let dimensions = storage.attribute_dimensions(0).unwrap();
        let mut meter = Meter::default();
        let error = storage
            .reserve_attributes(&node, unallocatable(), &mut meter)
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::AllocationFailed);
        assert_eq!(snapshot(&storage), before);
        assert_eq!(storage.attribute_dimensions(0).unwrap(), dimensions);
        // The chosen capacity was recorded, and then the capacity the list kept replaced it.
        assert_eq!(meter.recorded.len(), 2);
        assert_eq!(meter.recorded[0].capacity, unallocatable() + 1);
        assert_eq!(meter.recorded[1], dimensions);
    }

    #[test]
    fn an_allocator_refusal_of_the_spare_room_falls_back_to_the_room_that_is_needed() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let mut meter = Meter {
            spare: unallocatable(),
            ..Meter::default()
        };
        storage
            .append_attribute(&node, &mut Some(attribute("b", "2")), None, &mut meter)
            .unwrap();
        assert_eq!(names_of(&storage, 0), ["a", "b"]);
        assert_eq!(meter.growths.len(), 1);
        // The capacity the consumer chose, then the one the fallback reached.
        assert_eq!(meter.recorded.len(), 2);
        assert_eq!(meter.recorded[0].capacity, 2 + unallocatable());
        assert_eq!(
            meter.recorded[1],
            storage
                .attribute_dimensions(0)
                .map(|now| {
                    HtmlConstructionAttributeCapacity {
                        initialized_len: 1,
                        capacity: now.capacity,
                    }
                })
                .unwrap()
        );
        assert!(meter.recorded[1].capacity >= 2);
    }

    #[test]
    fn a_refused_correcting_record_leaves_the_attributes_unchanged() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let mut meter = Meter {
            spare: unallocatable(),
            refuse_record_from: Some(1),
            ..Meter::default()
        };
        let error = storage
            .append_attribute(&node, &mut Some(attribute("b", "2")), None, &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        // The attributes and the usage are as they were. The list holds the room the fallback
        // reserved, which the consumer that refused to record it has stopped for.
        assert_eq!(snapshot(&storage), before);
        assert_eq!(names_of(&storage, 0), ["a"]);
        assert!(storage.attribute_dimensions(0).unwrap().capacity >= 2);
    }

    #[test]
    fn a_list_held_by_another_owner_refuses_a_reservation_a_replacement_and_a_removal() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let held = match storage.node_at(0).unwrap().kind() {
            ConstructionNodeKind::Element(element) => element.attributes().to_ordered(),
            _ => unreachable!(),
        };
        let before = snapshot(&storage);
        let dimensions = storage.attribute_dimensions(0).unwrap();
        let aliased = HtmlConstructionCause::SharedAllocationAliased;
        let error = storage
            .reserve_attributes(&node, 8, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), aliased);
        let error = storage
            .replace_attribute(
                &node,
                0,
                &mut Some(attribute("c", "3")),
                &mut Meter::default(),
            )
            .unwrap_err();
        assert_eq!(cause(error), aliased);
        let error = storage
            .remove_attribute(&node, 1, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), aliased);
        assert_eq!(snapshot(&storage), before);
        assert_eq!(storage.attribute_dimensions(0).unwrap(), dimensions);
        drop(held);
    }

    #[test]
    fn a_reservation_refused_by_the_consumer_keeps_the_contents_the_capacity_and_the_weak_holder() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let before = snapshot(&storage);
        let (weak, pointer, capacity) = weak_of_backing(&storage);
        // A weak holder makes the list another owner's, so the reservation is refused before the
        // consumer is asked, and the holder still names the same backing afterwards.
        let mut meter = Meter::default();
        let error = storage
            .reserve_attributes(&node, 8, &mut meter)
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::SharedAllocationAliased);
        assert!(meter.growths.is_empty());
        assert_eq!(snapshot(&storage), before);
        let kept = weak.upgrade().unwrap();
        assert_eq!(
            (kept.as_ptr() as usize, kept.capacity()),
            (pointer, capacity)
        );
        assert_eq!(kept.len(), 2);
    }
}
