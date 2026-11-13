//! Edits of the attributes of an element of the construction storage.
//!
//! The attributes of every element are the rows of one table of the storage, with one run of rows
//! for each element, and the value of an attribute is a range of the storage's text arena. Each
//! edit is one method of [`HtmlConstructionStorage`]. It reads and compares first, charging its
//! consumer for the work before it runs, and asks the consumer once to admit the usage and the
//! growth of each bank it needs: the rows, the runs and the text arena, and the names of the
//! attribute when it interns them. Only then does it change the table, so a refusal leaves the
//! records, the usage and the attributes as they were. Residues remain: the name index is a cache
//! of the names of one element and a refused edit may have rebuilt it; a name that an edit
//! interned stays interned; and a bank that grew keeps the room it reserved, which the consumer
//! holds at the capacity it last recorded. The consumer owns the policy: it charges work through
//! [`Charge`], chooses the capacity a growing bank reserves, and records the capacity.

use super::records::{
    HtmlConstructionChange, HtmlRecordAdmission, NameId, attribute_units, stopped,
};
use super::*;
use crate::internal::accounting::{HtmlChargedError, HtmlResource, HtmlWork, wide};
use crate::internal::construction::attribute_table::{AttrRow, AttrRun, RowAppend, RunIx};
use crate::internal::construction::names::{NameEntry, NameKey};
use crate::internal::values::attribute_name_index::{
    Binding, CLEAR_CHUNK, HtmlAttributeNameIndex, entry, occupant, tag,
};
use crate::internal::values::bounded::{
    ComparisonStep, Phase as NamePhase, advance_comparison, comparison_layout,
};
use crate::internal::values::name_index::{
    HtmlNameIndex, HtmlNameIndexSlot, HtmlNameIndexStep, HtmlNameIndexVacancy,
};
use crate::internal::values::semantic_input::{HtmlAttributeInput, HtmlNameInput};
use crate::internal::values::{DocumentString, Namespace};
use surgeist_ledger::{Change, Charge};

/// The most bytes one charge covers when an edit moves a run of attributes, so that a consumer
/// that polls at its charges polls within every quantum of the work.
const CHARGE_QUANTUM: usize = 256;

/// The units one charge covers when an edit copies a value, so that a consumer that polls at its
/// charges polls within every quantum of the work.
const CHUNK_UNITS: usize = CHARGE_QUANTUM / size_of::<u16>();

/// Actual dimensions of the run of one element: how many attributes it holds and how many rows it
/// has room for; no allocation or release authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionAttributeCapacity {
    /// The number of attributes currently stored.
    pub initialized_len: usize,
    /// The rows the run has room for; at least `initialized_len`.
    pub capacity: usize,
}
impl HtmlConstructionAttributeCapacity {
    /// The size in bytes of one row of the attribute table.
    pub fn element_size(self) -> usize {
        size_of::<AttrRow>()
    }
    /// The alignment in bytes of one row of the attribute table.
    pub fn element_alignment(self) -> usize {
        align_of::<AttrRow>()
    }
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
/// Whether two names have one expanded name by the storage of their strings, which is the fast
/// path of the comparison: names of one interned entry share it.
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
        match self.arena.rows.get(position).map(|row| row.kind) {
            Some(RowKind::Element) => Ok(position),
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    /// The run of the element at `target`, which is none while the element has no attribute.
    fn element_run(&self, target: usize) -> Result<Option<RunIx>, HtmlConstructionError> {
        match self.arena.rows.get(target) {
            Some(row) if row.kind == RowKind::Element => Ok(row.run()),
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    /// Gives the element at `target` the run `ix`.
    pub(super) fn set_element_run(
        &mut self,
        target: usize,
        ix: RunIx,
    ) -> Result<(), HtmlConstructionError> {
        match self.arena.rows.get_mut(target) {
            Some(row) if row.kind == RowKind::Element => {
                row.b = ix.stored();
                Ok(())
            }
            _ => Err(HtmlConstructionCause::WrongNodeKind.into()),
        }
    }

    /// The attributes of the element at `target`, which the table holds or refuses as invalid.
    fn attribute_slice(&self, target: usize) -> Result<&[AttrRow], HtmlConstructionError> {
        self.attrs.live(self.element_run(target)?)
    }

    fn attribute_dimensions(
        &self,
        target: usize,
    ) -> Result<HtmlConstructionAttributeCapacity, HtmlConstructionError> {
        let (initialized_len, capacity) = self.attrs.dimensions(self.element_run(target)?)?;
        Ok(HtmlConstructionAttributeCapacity {
            initialized_len,
            capacity,
        })
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

    /// Charges the move of `rows` rows in quanta, before it runs.
    fn charge_moved_rows<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
        rows: usize,
    ) -> Result<(), HtmlConstructionAttributeError<C::Stop>> {
        let per_quantum = (CHARGE_QUANTUM / size_of::<AttrRow>()).max(1);
        let mut remaining = rows;
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            Work {
                record_visits: 1,
                copied_bytes: chunk.saturating_mul(size_of::<AttrRow>()),
                record_writes: chunk,
                ..Work::default()
            }
            .charge(charge)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Ok(())
    }

    /// Charges the filler rows that making room writes, one write each, in quanta before any is
    /// written: a plan writes `plan.written()` rows beyond what the copy of the live rows of a
    /// relocation charges, which are the rows of the new run for an element without one, the rows
    /// it gains at the tail, and for a relocation the rows of the region that the old rows and
    /// the room are filled with.
    pub(super) fn charge_filled_rows<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
        plan: RowAppend,
    ) -> Result<(), HtmlConstructionAttributeError<C::Stop>> {
        let per_quantum = (CHARGE_QUANTUM / size_of::<AttrRow>()).max(1);
        let mut remaining = plan.written();
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            Work {
                record_writes: chunk,
                ..Work::default()
            }
            .charge(charge)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Ok(())
    }

    /// Charges the copy of the units of a value in quanta, before it runs.
    pub(super) fn charge_value<C: Charge<HtmlWork, HtmlResource>>(
        charge: &mut C,
        units: &[u16],
    ) -> Result<(), HtmlConstructionAttributeError<C::Stop>> {
        for chunk in units.chunks(CHUNK_UNITS) {
            Work {
                copied_bytes: size_of_val(chunk),
                record_writes: 1,
                ..Work::default()
            }
            .charge(charge)?;
        }
        Ok(())
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
        list: &[AttrRow],
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
        list: &[AttrRow],
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

    /// Ensures the run of the element `node` can hold `additional` more attributes, admitting
    /// the growth of the banks it needs once when it cannot. Returns the dimensions of the run
    /// afterwards.
    pub fn reserve_attributes<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        additional: usize,
        admission: &mut A,
    ) -> Result<HtmlConstructionAttributeCapacity, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let before = self.attribute_dimensions(target).map_err(faulted)?;
        if additional == 0 {
            return Ok(before);
        }
        let ix = self.element_run(target).map_err(faulted)?;
        let plan = self.attrs.plan(ix, additional).map_err(faulted)?;
        if plan == RowAppend::Room {
            return Ok(before);
        }
        let moved = match plan {
            RowAppend::Relocate { .. } => before.initialized_len,
            _ => 0,
        };
        Self::charge_moved_rows(admission, moved)?;
        Self::charge_filled_rows(admission, plan)?;
        Work {
            record_visits: 1,
            copied_bytes: size_of::<AttrRun>(),
            record_writes: 2,
            ..Work::default()
        }
        .charge(admission)?;
        self.grow_attribute_banks(plan, 0, admission)?;
        let run = self.attrs.make_room(ix, plan).map_err(faulted)?;
        if ix.is_none() {
            self.set_element_run(target, run).map_err(faulted)?;
        }
        self.attribute_dimensions(target).map_err(faulted)
    }

    /// Appends the attribute `incoming` to the element `node`, interning its name and copying its
    /// value to the text arena. The name must not repeat an attribute of the element, or the edit
    /// is refused with the duplicate-attribute cause. A full run moves or extends once, through
    /// `admission`.
    ///
    /// With `names`, a duplicate is found by probing the few names that share a hash, so that the
    /// work of appending `n` attributes is linear in `n` and the length of their names. The index
    /// describes the element's attributes while its binding matches them and is rebuilt when it
    /// does not; an index too small for the names is ignored and every name is compared. Returns
    /// the position the attribute took.
    pub fn append_attribute<A: HtmlRecordAdmission, I: HtmlAttributeInput + ?Sized>(
        &mut self,
        node: &HtmlBuilderNode,
        incoming: &I,
        names: Option<&mut HtmlAttributeNameIndex>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        let entry = self.intern_attribute_name(incoming.name(), admission)?;
        self.append_entry(
            node,
            entry,
            incoming.value().code_units(),
            names,
            None,
            admission,
        )
    }

    /// As [`append_attribute`](Self::append_attribute) for the attribute a merge plan kept for
    /// `candidate`. The plan compared the candidate's name with the element's attributes and with
    /// the other kept candidates, and left an entry for it in `names`. The name is looked up
    /// through the index like any other, so an attribute whose name the list holds is refused as
    /// a duplicate. When no attribute has the name, the attribute has the candidate's backing,
    /// the index still describes the element as the plan left it and the lookup passed the
    /// plan's entry for the position, that entry stays and nothing more is entered. Any other
    /// attribute leaves the index describing nothing, and the next edit rebuilds it.
    pub fn append_planned_attribute<A: HtmlRecordAdmission, I: HtmlAttributeInput + ?Sized>(
        &mut self,
        node: &HtmlBuilderNode,
        incoming: &I,
        candidate: &DocumentString,
        names: Option<&mut HtmlAttributeNameIndex>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        let entry = self.intern_attribute_name(incoming.name(), admission)?;
        self.append_entry(
            node,
            entry,
            incoming.value().code_units(),
            names,
            Some(candidate),
            admission,
        )
    }

    /// Appends an attribute of the interned name `name`, an id this storage issued, whose value
    /// is a copy of `value`, as [`append_attribute`](Self::append_attribute) does without
    /// interning the name again; `candidate` names the candidate of a merge plan that the
    /// attribute is, as [`append_planned_attribute`](Self::append_planned_attribute) says. The
    /// foreign-builder cause for the id of another storage.
    pub fn append_named_attribute<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        name: &NameId,
        value: &[u16],
        candidate: Option<&DocumentString>,
        names: Option<&mut HtmlAttributeNameIndex>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        Work {
            record_visits: 1,
            ..Work::default()
        }
        .charge(admission)?;
        let entry = self.name_entry(name).map_err(faulted)?.clone();
        self.append_entry(node, entry, value, names, candidate, admission)
    }

    /// The interned entry of an attribute name given by its components.
    fn intern_attribute_name<A: HtmlRecordAdmission>(
        &mut self,
        name: &(impl HtmlNameInput + ?Sized),
        admission: &mut A,
    ) -> Result<Arc<NameEntry>, HtmlConstructionAttributeError<A::Stop>> {
        let namespace = name
            .namespace_uri()
            .cloned()
            .map_or_else(Namespace::none, Namespace::from_uri);
        let key = NameKey {
            namespace: &namespace,
            prefix: name.prefix(),
            local_name: name.local_name().code_units(),
        };
        self.intern(key, None, admission).map(|(_, entry)| entry)
    }

    #[allow(clippy::too_many_arguments)]
    fn append_entry<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        entry: Arc<NameEntry>,
        value: &[u16],
        names: Option<&mut HtmlAttributeNameIndex>,
        candidate: Option<&DocumentString>,
        admission: &mut A,
    ) -> Result<usize, HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let ix = self.element_run(target).map_err(faulted)?;
        let before = self.attribute_dimensions(target).map_err(faulted)?;
        let len = before.initialized_len;
        let usage = self.arena.usage;
        let mut after = usage;
        after.attributes = plus(usage.attributes, 1).map_err(faulted)?;
        after.utf16_units = plus(
            usage.utf16_units,
            attribute_units(entry.name(), value.len()).map_err(faulted)?,
        )
        .map_err(faulted)?;
        let grown = plus(len, 1).map_err(faulted)?;
        self.text.check_add(value.len()).map_err(faulted)?;
        let is_candidate = candidate.is_some_and(|candidate| {
            entry.name().namespace().uri().is_none()
                && entry.name().local_name().shares_storage_with(candidate)
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
            // The lookup runs for a planned attribute too: the candidate is the caller's
            // word, and an attribute whose name an earlier attribute has is refused whatever
            // the plan says.
            let list = self.attribute_slice(target).map_err(faulted)?;
            match self.index_lookup(admission, index, list, entry.name())? {
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
            if let Some(first) = Self::find_name(admission, list, entry.name(), None)? {
                return Err(fault_duplicate(first, len));
            }
        }
        let plan = self.attrs.plan(ix, 1).map_err(faulted)?;
        let change = HtmlConstructionChange {
            before: usage,
            after,
        };
        admission.admit_change(self, change).map_err(stopped)?;
        let moved = match plan {
            RowAppend::Relocate { .. } => len,
            _ => 0,
        };
        Self::charge_moved_rows(admission, moved)?;
        Self::charge_filled_rows(admission, plan)?;
        Self::charge_value(admission, value)?;
        Work {
            record_visits: 2,
            copied_bytes: size_of::<AttrRow>(),
            record_writes: 3,
            ..Work::default()
        }
        .charge(admission)?;
        self.grow_attribute_banks(plan, value.len(), admission)?;
        admission.record_change(self, change).map_err(stopped)?;
        // The banks have room for the writes below, which allocate nothing.
        let run = self.attrs.make_room(ix, plan).map_err(faulted)?;
        if ix.is_none() {
            self.set_element_run(target, run).map_err(faulted)?;
        }
        let text = self.text.push(value).map_err(faulted)?.text();
        let index = self
            .attrs
            .push(run, AttrRow::new(entry, text))
            .map_err(faulted)?;
        self.arena.usage = after;
        self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
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
                    let placed = crate::internal::values::attribute_name_index::entry(tag, len)
                        .is_some_and(|slot| index.insert(vacancy, slot).is_ok());
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
        Ok(index)
    }

    /// Replaces the attribute at `index` of the element `node` with `incoming`, interning its
    /// name and copying its value. The replacement's name must not repeat another attribute of
    /// the element. A value no longer than the one it replaces is written over its units;
    /// a longer one is written at the tail of the text arena, and the units of the old value stay
    /// written as spare.
    pub fn replace_attribute<A: HtmlRecordAdmission, I: HtmlAttributeInput + ?Sized>(
        &mut self,
        node: &HtmlBuilderNode,
        index: usize,
        incoming: &I,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        if list.get(index).is_none() {
            return Err(fault(HtmlConstructionCause::OperandMissing));
        }
        let entry = self.intern_attribute_name(incoming.name(), admission)?;
        self.replace_entry(
            target,
            index,
            entry,
            incoming.value().code_units(),
            admission,
        )
    }

    /// Replaces the attribute at `index` of the element `node` with one of the interned name
    /// `name`, an id this storage issued, and a copy of `value`, as
    /// [`replace_attribute`](Self::replace_attribute) does without interning the name again. The
    /// foreign-builder cause for the id of another storage.
    pub fn replace_named_attribute<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        index: usize,
        name: &NameId,
        value: &[u16],
        admission: &mut A,
    ) -> Result<(), HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let entry = self.name_entry(name).map_err(faulted)?.clone();
        self.replace_entry(target, index, entry, value, admission)
    }

    fn replace_entry<A: HtmlRecordAdmission>(
        &mut self,
        target: usize,
        index: usize,
        entry: Arc<NameEntry>,
        value: &[u16],
        admission: &mut A,
    ) -> Result<(), HtmlConstructionAttributeError<A::Stop>> {
        let ix = self.element_run(target).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        let old = list
            .get(index)
            .ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
        let usage = self.arena.usage;
        let mut after = usage;
        after.utf16_units = plus(
            minus(
                usage.utf16_units,
                attribute_units(old.name(), old.value().len()).map_err(faulted)?,
            )
            .map_err(faulted)?,
            attribute_units(entry.name(), value.len()).map_err(faulted)?,
        )
        .map_err(faulted)?;
        if !identical_name(old.name(), entry.name())
            && let Some(first) = Self::find_name(admission, list, entry.name(), Some(index))?
        {
            return Err(fault_duplicate(first, index));
        }
        let (old_value, in_place) = (old.value(), value.len() <= old.value().len());
        let tail = if in_place { 0 } else { value.len() };
        self.text.check_add(tail).map_err(faulted)?;
        let change = HtmlConstructionChange {
            before: usage,
            after,
        };
        admission.admit_change(self, change).map_err(stopped)?;
        Self::charge_value(admission, value)?;
        Work {
            record_visits: 2,
            copied_bytes: size_of::<AttrRow>(),
            record_writes: 3,
            ..Work::default()
        }
        .charge(admission)?;
        self.grow_attribute_banks(RowAppend::Room, tail, admission)?;
        admission.record_change(self, change).map_err(stopped)?;
        let run = ix.ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
        let text = if in_place {
            self.text.overwrite(old_value, value).map_err(faulted)?
        } else {
            let text = self.text.push(value).map_err(faulted)?.text();
            self.text.forget(old_value.len()).map_err(faulted)?;
            text
        };
        self.attrs
            .replace(run, index, AttrRow::new(entry, text))
            .map_err(faulted)?;
        self.arena.usage = after;
        self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
        Ok(())
    }

    /// Removes the attribute at `index` of the element `node`, shifting the attributes after it
    /// down by one place. The shift is charged before it runs, in quanta, and the units of the
    /// value stay written as spare.
    pub fn remove_attribute<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        index: usize,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionAttributeError<A::Stop>> {
        Self::charge_qualification(admission)?;
        let target = self.attribute_target(node).map_err(faulted)?;
        let ix = self.element_run(target).map_err(faulted)?;
        let list = self.attribute_slice(target).map_err(faulted)?;
        let old = list
            .get(index)
            .ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
        let usage = self.arena.usage;
        let mut after = usage;
        after.attributes = minus(usage.attributes, 1).map_err(faulted)?;
        after.utf16_units = minus(
            usage.utf16_units,
            attribute_units(old.name(), old.value().len()).map_err(faulted)?,
        )
        .map_err(faulted)?;
        let released = old.value().len();
        let moved = minus(list.len(), plus(index, 1).map_err(faulted)?).map_err(faulted)?;
        let change = HtmlConstructionChange {
            before: usage,
            after,
        };
        admission.admit_change(self, change).map_err(stopped)?;
        Self::charge_moved_rows(admission, moved)?;
        Work {
            record_visits: 1,
            copied_bytes: size_of::<AttrRow>(),
            record_writes: 2,
            ..Work::default()
        }
        .charge(admission)?;
        admission.record_change(self, change).map_err(stopped)?;
        let run = ix.ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
        self.attrs.remove(run, index).map_err(faulted)?;
        self.text.forget(released).map_err(faulted)?;
        self.arena.usage = after;
        self.arena.attribute_revision = self.arena.attribute_revision.wrapping_add(1);
        Ok(())
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
                    .ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
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
                        .ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
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
                .ok_or_else(|| fault(HtmlConstructionCause::OperandMissing))?;
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
    use super::super::records::{
        HtmlConstructionBank, HtmlConstructionBankGrowth, HtmlConstructionGrowthRecord,
    };
    use super::*;
    use crate::internal::values::{Attribute, OrderedAttributes};
    use std::collections::HashSet;

    /// The refusal of a [`Meter`].
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Refused;

    /// A consumer that records what it is asked, and refuses at a chosen call: the
    /// `refuse_from`th call of any of its methods, counting from zero, and every call after it.
    #[derive(Default)]
    struct Meter {
        calls: usize,
        refuse_from: Option<usize>,
        /// Refuses every admission of growth.
        refuse_growth: bool,
        /// The capacity it chooses beyond the minimum.
        spare: usize,
        work: u64,
        largest_copy: u64,
        growths: Vec<HtmlConstructionBankGrowth>,
        records: Vec<HtmlConstructionGrowthRecord>,
    }
    impl Meter {
        fn call(&mut self) -> Result<(), Refused> {
            let index = self.calls;
            self.calls += 1;
            match self.refuse_from {
                Some(from) if index >= from => Err(Refused),
                _ => Ok(()),
            }
        }
        fn refusing_from(index: usize) -> Self {
            Self {
                refuse_from: Some(index),
                ..Self::default()
            }
        }
        fn banks(&self, bank: HtmlConstructionBank) -> usize {
            self.growths
                .iter()
                .filter(|growth| growth.bank() == bank)
                .count()
        }
    }
    impl Charge<HtmlWork, HtmlResource> for Meter {
        type Stop = Refused;
        fn charge(&mut self, batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Refused> {
            self.call()?;
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
    impl HtmlRecordAdmission for Meter {
        fn admit_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            self.call()
        }
        fn admit_growth(&mut self, growth: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
            self.call()?;
            self.growths.push(growth);
            if self.refuse_growth {
                return Err(Refused);
            }
            Ok(growth.minimum_capacity() + self.spare)
        }
        fn record_growth(&mut self, record: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
            self.call()?;
            self.records.push(record);
            Ok(())
        }
        fn record_change(
            &mut self,
            _: &HtmlConstructionStorage,
            _: HtmlConstructionChange,
        ) -> Result<(), Refused> {
            self.call()
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
        add_element(super::super::free::storage(), "root", attributes)
    }
    fn add_element(
        mut storage: HtmlConstructionStorage,
        name: &str,
        attributes: Vec<Attribute>,
    ) -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let created = super::super::free::element(
            &mut storage,
            QualifiedName::new(Namespace::none(), None, name),
            OrderedAttributes::new(attributes).unwrap(),
        );
        (storage, created)
    }
    fn names_of(storage: &HtmlConstructionStorage, position: usize) -> Vec<String> {
        storage
            .attribute_slice(position)
            .unwrap()
            .iter()
            .map(|held| held.name().local_name().to_utf8().unwrap())
            .collect()
    }
    fn values_of(storage: &HtmlConstructionStorage, position: usize) -> Vec<String> {
        storage
            .attribute_slice(position)
            .unwrap()
            .iter()
            .map(|held| String::from_utf16(storage.text.get(held.value()).unwrap()).unwrap())
            .collect()
    }
    /// What an edit must leave alone when it is refused.
    type Snapshot = (
        Vec<Vec<(String, String)>>,
        HtmlConstructionUsage,
        u64,
        usize,
    );
    fn snapshot(storage: &HtmlConstructionStorage) -> Snapshot {
        let lists = (0..storage.len())
            .map(|position| {
                names_of(storage, position)
                    .into_iter()
                    .zip(values_of(storage, position))
                    .collect()
            })
            .collect();
        (
            lists,
            storage.usage(),
            storage.arena.attribute_revision,
            storage.text.used(),
        )
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
    fn an_append_adds_the_attribute_with_its_value_in_the_arena_and_counts_it() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "22")]);
        let before = storage.usage();
        let position = storage
            .append_attribute(&node, &attribute("c", "333"), None, &mut Meter::default())
            .unwrap();
        assert_eq!(position, 2);
        assert_eq!(names_of(&storage, 0), ["a", "b", "c"]);
        assert_eq!(values_of(&storage, 0), ["1", "22", "333"]);
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
        let before = storage.usage().utf16_units();
        storage
            .append_attribute(
                &node,
                &Attribute::new(name, "value"),
                None,
                &mut Meter::default(),
            )
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
        for indexed in [false, true] {
            let mut table = index(16);
            let error = storage
                .append_attribute(
                    &node,
                    &repeated,
                    indexed.then_some(&mut table),
                    &mut Meter::default(),
                )
                .unwrap_err();
            assert_eq!(
                cause(error),
                HtmlConstructionCause::DuplicateAttribute {
                    first: 1,
                    duplicate: 2
                }
            );
            assert_eq!(snapshot(&storage), before);
        }
        // The same local name in another namespace is another name, and two prefixes of one
        // namespace and local name are one name.
        let other = |prefix: Option<&str>| {
            Attribute::new(
                QualifiedName::new(
                    Namespace::from_uri("urn:x"),
                    prefix.map(Into::into),
                    DocumentString::from("b"),
                ),
                "v",
            )
        };
        storage
            .append_attribute(&node, &other(Some("p1")), None, &mut Meter::default())
            .unwrap();
        assert_eq!(names_of(&storage, 0), ["a", "b", "b"]);
        let error = storage
            .append_attribute(&node, &other(Some("p2")), None, &mut Meter::default())
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 2,
                duplicate: 3
            }
        );
    }

    #[test]
    fn a_full_run_grows_the_rows_once_and_is_told_the_capacity_it_needs() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = storage.attribute_dimensions(0).unwrap();
        assert_eq!(before.capacity, 1);
        let mut meter = Meter {
            spare: 3,
            ..Meter::default()
        };
        storage
            .append_attribute(&node, &attribute("b", "2"), None, &mut meter)
            .unwrap();
        // The run is at the tail of the table, so it extends there: the rows and the text arena
        // grow once each, and the runs do not.
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRows), 1);
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRuns), 0);
        let rows = meter
            .growths
            .iter()
            .find(|growth| growth.bank() == HtmlConstructionBank::AttributeRows)
            .unwrap();
        assert_eq!(rows.element_size(), size_of::<AttrRow>());
        assert_eq!(rows.minimum_capacity(), storage.attrs.rows_len());
        assert!(storage.attrs.rows_capacity() >= rows.minimum_capacity() + 3);
        // The spare room the consumer chose absorbs the next appends without another growth.
        let growths = meter.growths.len();
        for name in ["c", "d", "e"] {
            storage
                .append_attribute(&node, &attribute(name, "v"), None, &mut meter)
                .unwrap();
        }
        assert_eq!(
            meter
                .growths
                .iter()
                .filter(|growth| growth.bank() == HtmlConstructionBank::AttributeRows)
                .count(),
            1
        );
        assert!(meter.growths.len() >= growths);
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
        impl HtmlRecordAdmission for Stingy {
            fn admit_change(
                &mut self,
                storage: &HtmlConstructionStorage,
                change: HtmlConstructionChange,
            ) -> Result<(), Refused> {
                self.0.admit_change(storage, change)
            }
            fn admit_growth(&mut self, g: HtmlConstructionBankGrowth) -> Result<usize, Refused> {
                Ok(g.minimum_capacity() - 1)
            }
            fn record_growth(&mut self, _: HtmlConstructionGrowthRecord) -> Result<(), Refused> {
                panic!("nothing was reserved")
            }
            fn record_change(
                &mut self,
                storage: &HtmlConstructionStorage,
                change: HtmlConstructionChange,
            ) -> Result<(), Refused> {
                self.0.record_change(storage, change)
            }
        }
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let error = storage
            .append_attribute(
                &node,
                &attribute("b", "2"),
                None,
                &mut Stingy(Meter::default()),
            )
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::CapacityMismatch);
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_refusal_at_any_admission_point_of_an_append_changes_nothing() {
        // Every charge, change and growth is an admission point; the edit passes once none
        // refuses, and a refusal at any earlier point leaves the records, the usage and the
        // values as they were.
        let mut points = 0;
        for refuse_from in 0.. {
            let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
            let before = snapshot(&storage);
            let mut meter = Meter::refusing_from(refuse_from);
            let result = storage.append_attribute(&node, &attribute("b", "2"), None, &mut meter);
            if result.is_ok() {
                assert_eq!(names_of(&storage, 0), ["a", "b"]);
                break;
            }
            points += 1;
            assert_eq!(
                snapshot(&storage),
                before,
                "refused after {refuse_from} calls"
            );
        }
        assert!(points >= 6, "the append makes several admission points");
        let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
        let before = snapshot(&storage);
        let mut meter = Meter {
            refuse_growth: true,
            ..Meter::default()
        };
        let error = storage
            .append_attribute(&node, &attribute("b", "2"), None, &mut meter)
            .unwrap_err();
        assert_eq!(error, HtmlChargedError::Stopped(Refused));
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_target_that_is_not_an_element_or_not_this_storage_is_refused() {
        let (mut storage, _) = make_storage(Vec::new());
        let (other, foreign) = make_storage(Vec::new());
        let error = storage
            .append_attribute(&foreign, &attribute("a", "1"), None, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::ForeignBuilder);
        drop(other);
        let text = super::super::free::text(&mut storage, &[1]);
        let error = storage
            .append_attribute(&text, &attribute("a", "1"), None, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::WrongNodeKind);
    }

    #[test]
    fn a_replacement_updates_the_value_and_the_units() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "22")]);
        let units = storage.usage().utf16_units();
        storage
            .replace_attribute(&node, 0, &attribute("a", "4444"), &mut Meter::default())
            .unwrap();
        assert_eq!(values_of(&storage, 0), ["4444", "22"]);
        assert_eq!(storage.usage().utf16_units(), units - 1 + 4);
        assert_eq!(storage.usage().attributes(), 2);
        storage
            .replace_attribute(&node, 1, &attribute("c", "v"), &mut Meter::default())
            .unwrap();
        assert_eq!(names_of(&storage, 0), ["a", "c"]);
        assert_eq!(values_of(&storage, 0), ["4444", "v"]);
    }

    #[test]
    fn a_value_no_longer_than_the_old_one_is_written_in_place_and_a_longer_one_at_the_tail() {
        let (mut storage, node) = make_storage(vec![attribute("a", "abcd"), attribute("b", "xy")]);
        let written = storage.text.len();
        storage
            .replace_attribute(&node, 0, &attribute("a", "ef"), &mut Meter::default())
            .unwrap();
        assert_eq!(values_of(&storage, 0), ["ef", "xy"]);
        assert_eq!(storage.text.len(), written, "the units were overwritten");
        assert_eq!(storage.text.used(), 4);
        storage
            .replace_attribute(&node, 1, &attribute("b", "longer"), &mut Meter::default())
            .unwrap();
        assert_eq!(values_of(&storage, 0), ["ef", "longer"]);
        assert_eq!(
            storage.text.len(),
            written + 6,
            "the units went to the tail"
        );
        assert_eq!(
            storage.text.used(),
            8,
            "the old units no longer count as held"
        );
    }

    #[test]
    fn repeated_replacements_never_leave_more_units_than_the_text_arena_admits() {
        let (mut storage, node) = make_storage(vec![attribute("a", "x")]);
        let mut meter = Meter::default();
        for round in 0..200 {
            let value = "y".repeat(2 + round % 5);
            storage
                .replace_attribute(&node, 0, &attribute("a", value.as_str()), &mut meter)
                .unwrap();
            assert_eq!(values_of(&storage, 0), [value]);
        }
        // Each replacement that grows the value is one tail write of at most six units.
        assert!(storage.text.len() <= 1 + 200 * 6);
        assert!(storage.text.used() <= 6);
        assert!(storage.text.capacity() >= storage.text.len());
    }

    #[test]
    fn a_replacement_that_repeats_another_name_or_misses_its_slot_is_refused() {
        let (mut storage, node) = make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
        let before = snapshot(&storage);
        let error = storage
            .replace_attribute(&node, 0, &attribute("b", "x"), &mut Meter::default())
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 1,
                duplicate: 0
            }
        );
        let error = storage
            .replace_attribute(&node, 2, &attribute("c", "x"), &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::OperandMissing);
        assert_eq!(snapshot(&storage), before);
    }

    #[test]
    fn a_refusal_at_any_admission_point_of_a_replacement_changes_nothing() {
        for value in ["3", "3333333"] {
            for refuse_from in 0.. {
                let (mut storage, node) =
                    make_storage(vec![attribute("a", "1"), attribute("b", "2")]);
                let before = snapshot(&storage);
                let mut meter = Meter::refusing_from(refuse_from);
                if storage
                    .replace_attribute(&node, 0, &attribute("c", value), &mut meter)
                    .is_ok()
                {
                    assert_eq!(names_of(&storage, 0), ["c", "b"]);
                    assert!(refuse_from > 0);
                    break;
                }
                assert_eq!(snapshot(&storage), before, "{value}: from {refuse_from}");
            }
        }
    }

    #[test]
    fn a_removal_closes_the_gap_and_counts_the_attribute_out() {
        let (mut storage, node) = make_storage(
            ["a", "b", "c", "d"]
                .iter()
                .map(|name| attribute(*name, "vv"))
                .collect(),
        );
        let before = storage.usage();
        storage
            .remove_attribute(&node, 1, &mut Meter::default())
            .unwrap();
        assert_eq!(names_of(&storage, 0), ["a", "c", "d"]);
        assert_eq!(storage.usage().attributes(), before.attributes() - 1);
        assert_eq!(storage.usage().utf16_units(), before.utf16_units() - 3);
        let error = storage
            .remove_attribute(&node, 3, &mut Meter::default())
            .unwrap_err();
        assert_eq!(cause(error), HtmlConstructionCause::OperandMissing);
        // The removed value's units stay written and no longer count as held.
        assert_eq!(storage.text.used(), 6);
    }

    #[test]
    fn a_removal_charges_the_shift_in_quanta_before_it_moves_anything() {
        let attributes: Vec<_> = (0..40)
            .map(|n| attribute(format!("n{n}").as_str(), "v"))
            .collect();
        let (mut storage, node) = make_storage(attributes);
        let mut meter = Meter::default();
        storage.remove_attribute(&node, 0, &mut meter).unwrap();
        // 39 rows move; no charge covers more than the 256 byte quantum or the one row that
        // leaves.
        assert!(meter.largest_copy <= CHARGE_QUANTUM as u64);
        assert!(meter.work >= (39 * size_of::<AttrRow>()) as u64);
        // A refusal after the first shift charge leaves the list intact.
        let before = snapshot(&storage);
        let mut refusing = Meter::refusing_from(3);
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
            storage
                .append_planned_attribute(
                    &node,
                    &attribute(texts[candidate].clone(), "v"),
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
        let error = storage
            .append_attribute(
                &node,
                &attribute("c", "x"),
                Some(&mut table),
                &mut Meter::default(),
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
    fn a_named_append_takes_the_interned_name_and_a_copy_of_the_units() {
        let (mut storage, node) = make_storage(Vec::new());
        let id = storage
            .intern_name(
                &Namespace::none(),
                None,
                &[b'k' as u16],
                &mut crate::internal::construction::NativeAdmission::exact(),
            )
            .map_err(crate::internal::construction::native)
            .unwrap();
        let value = [b'v' as u16, 0xd800];
        let position = storage
            .append_named_attribute(&node, &id, &value, None, None, &mut Meter::default())
            .unwrap();
        assert_eq!(position, 0);
        assert_eq!(names_of(&storage, 0), ["k"]);
        assert_eq!(
            storage
                .text
                .get(storage.attribute_slice(0).unwrap()[0].value())
                .unwrap(),
            value
        );
        // The row holds the interned entry: the same name allocation.
        let held = storage.attribute_slice(0).unwrap()[0].name();
        assert!(
            held.local_name()
                .shares_storage_with(storage.name(&id).unwrap().local_name())
        );
        let (other, _) = make_storage(Vec::new());
        let error = storage
            .append_named_attribute(
                &node,
                &other.arena.names.get(0).map(|_| id).unwrap_or(id),
                &value,
                None,
                None,
                &mut Meter::default(),
            )
            .unwrap_err();
        assert_eq!(
            cause(error),
            HtmlConstructionCause::DuplicateAttribute {
                first: 0,
                duplicate: 1
            }
        );
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
                &attribute("c", "x"),
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
                &attribute("c", "y"),
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
                    let outcome = storage.replace_attribute(
                        &node,
                        at,
                        &attribute(name.as_str(), "r"),
                        &mut Meter::default(),
                    );
                    let expected = held
                        .iter()
                        .enumerate()
                        .find(|(position, held)| *position != at && **held == name)
                        .map(|(position, _)| position);
                    match (outcome, expected) {
                        (Ok(()), None) => held[at] = name,
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
                let outcome = storage.append_attribute(
                    &node,
                    &attribute(name.as_str(), "v"),
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
                    &attribute(format!("n{name}").as_str(), "v"),
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        meter.work
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

    /// The work of `n` appends alternating between two elements, so that the runs move each time
    /// one outgrows its room.
    fn alternating_appends(n: usize) -> u64 {
        let (storage, first) = make_storage(Vec::new());
        let (mut storage, second) = add_element(storage, "other", Vec::new());
        let mut meter = Meter {
            spare: 2 * n,
            ..Meter::default()
        };
        for step in 0..n {
            for node in [&first, &second] {
                storage
                    .append_attribute(
                        node,
                        &attribute(format!("n{step}").as_str(), "v"),
                        None,
                        &mut meter,
                    )
                    .unwrap();
            }
        }
        meter.work
    }

    #[test]
    fn appends_alternating_between_two_elements_cost_work_in_proportion_to_their_number() {
        // The pairwise comparison of names is quadratic, so the measure is the rows copied:
        // the charge of moved rows alone, which relocation bounds by the rows the runs hold.
        let moved = |n: usize| -> u64 {
            let (storage, first) = make_storage(Vec::new());
            let (mut storage, second) = add_element(storage, "other", Vec::new());
            let mut meter = Meter {
                spare: 2 * n,
                ..Meter::default()
            };
            let mut relocations = 0u64;
            for step in 0..n {
                for node in [&first, &second] {
                    let target = storage.attribute_target(node).unwrap();
                    let ix = storage.element_run(target).unwrap();
                    if let Ok(RowAppend::Relocate { .. }) = storage.attrs.plan(ix, 1) {
                        relocations += storage
                            .attribute_dimensions(target)
                            .unwrap()
                            .initialized_len as u64;
                    }
                    let id = storage
                        .intern_name(
                            &Namespace::none(),
                            None,
                            &[step as u16 % 97 + 1, (step / 97) as u16 + 1],
                            &mut crate::internal::construction::NativeAdmission::exact(),
                        )
                        .map_err(crate::internal::construction::native)
                        .unwrap();
                    storage
                        .append_named_attribute(node, &id, &[1], None, None, &mut meter)
                        .unwrap();
                }
            }
            relocations
        };
        for n in [100usize, 200, 400, 800] {
            // At most four rows copied for each row appended.
            assert!(
                moved(n) <= 4 * 2 * n as u64,
                "{} rows for {n} appends each",
                moved(n)
            );
        }
        let _ = alternating_appends;
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
            .append_attribute(&first, &attribute("c", "v"), Some(&mut table), &mut meter)
            .unwrap();
        // The index describes the first element; the second has no "c" and must accept it.
        storage
            .append_attribute(&second, &attribute("c", "v"), Some(&mut table), &mut meter)
            .unwrap();
        // Back on the first element the index describes the second, and "b" is new there.
        storage
            .append_attribute(&first, &attribute("b", "v"), Some(&mut table), &mut meter)
            .unwrap();
        let error = storage
            .append_attribute(&first, &attribute("c", "v"), Some(&mut table), &mut meter)
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
                    &attribute(format!("n{n}").as_str(), "v"),
                    Some(&mut table),
                    &mut meter,
                )
                .unwrap();
        }
        let error = storage
            .append_attribute(&node, &attribute("n17", "v"), Some(&mut table), &mut meter)
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
    fn reserving_makes_a_run_once_and_only_when_it_is_short() {
        let (mut storage, node) = make_storage(Vec::new());
        let mut meter = Meter::default();
        let reserved = storage.reserve_attributes(&node, 5, &mut meter).unwrap();
        assert_eq!((reserved.initialized_len, reserved.capacity), (0, 5));
        assert_eq!(reserved.element_size(), size_of::<AttrRow>());
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRuns), 1);
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRows), 1);
        for name in ["a", "b", "c", "d", "e"] {
            storage
                .append_attribute(&node, &attribute(name, "v"), None, &mut meter)
                .unwrap();
        }
        let growths = meter.growths.len();
        // Every append took the room the reservation made: only the text arena grew.
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRuns), 1);
        assert_eq!(meter.banks(HtmlConstructionBank::AttributeRows), 1);
        assert!(growths >= 2);
        let again = storage.reserve_attributes(&node, 0, &mut meter).unwrap();
        assert_eq!((again.initialized_len, again.capacity), (5, 5));
        assert_eq!(meter.growths.len(), growths);
    }

    #[test]
    fn a_reservation_the_consumer_refuses_keeps_the_attributes_and_the_banks_it_recorded() {
        for refuse_from in 0.. {
            let (mut storage, node) = make_storage(vec![attribute("a", "1")]);
            let before = snapshot(&storage);
            let mut meter = Meter::refusing_from(refuse_from);
            if storage.reserve_attributes(&node, 6, &mut meter).is_ok() {
                assert!(refuse_from > 0);
                break;
            }
            assert_eq!(
                snapshot(&storage),
                before,
                "refused from call {refuse_from}"
            );
        }
    }

    #[test]
    fn a_native_creation_with_attributes_copies_the_values_to_the_arena_and_a_refusal_takes_it_back()
     {
        let (mut storage, _) = make_storage(Vec::new());
        let attributes =
            OrderedAttributes::new(vec![attribute("a", "1"), attribute("b", "22")]).unwrap();
        let before = snapshot(&storage);
        let mut refusals = 0;
        for refuse_from in 0.. {
            let mut meter = Meter::refusing_from(refuse_from);
            let created = storage.create_element(
                &QualifiedName::new(Namespace::none(), None, "e"),
                attributes.clone(),
                None,
                &mut meter,
            );
            if let Ok(created) = created {
                let position = storage.position(&created).unwrap();
                assert_eq!(names_of(&storage, position), ["a", "b"]);
                assert_eq!(values_of(&storage, position), ["1", "22"]);
                break;
            }
            refusals += 1;
            // A refused creation leaves no record and no usage, though the names it interned
            // stay interned.
            let after = snapshot(&storage);
            assert_eq!(after.0, before.0, "refused from call {refuse_from}");
            assert_eq!(after.1, before.1, "refused from call {refuse_from}");
            assert_eq!(after.3, before.3, "refused from call {refuse_from}");
        }
        assert!(refusals > 5);
    }
}
