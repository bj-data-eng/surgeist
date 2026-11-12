//! The origin groups of the string fields of records, for a parse that records the origin graph.
//!
//! A field is an attribute string of an element (its name, value, namespace or prefix) or a
//! string of a non-element record: the data of a text or comment, the target and data of a
//! processing instruction, or the name and identifiers of a doctype. Each record that has a
//! group owns a run of entries, one per field that has one, sorted by attribute position and
//! string; the strings of a non-element record follow the attribute strings. A slot per node
//! position, indexed by the node, names its run, so an edit of one record reads and moves only
//! that record's entries. A parse that records no graph has no group to hold and never
//! allocates the table.
//!
//! The group of a merged-away text, the tokens a text absorbed, is no field's: the group that
//! replaced the field's own names it as a part, and the evidence keeps it.

use super::string_storage::AttributeString;
use super::*;
use crate::parser::control::WORK_QUANTUM;

pub(super) struct DetailOrigin {
    field: FieldRef,
    group: OriginGroupRef,
}

/// The sort key of a field: its record, its position in the attribute list (zero for a
/// non-element field) and its string.
fn key(field: FieldRef) -> Option<(usize, usize, u8)> {
    if let Some((node, index, string)) = field.attribute() {
        let which = match string {
            AttributeString::Name => 0,
            AttributeString::Value => 1,
            AttributeString::Namespace => 2,
            AttributeString::Prefix => 3,
        };
        return Some((node, index, which));
    }
    match field {
        FieldRef::Text(node) => Some((node, 0, 4)),
        FieldRef::Comment(node) => Some((node, 0, 5)),
        FieldRef::ProcessingInstructionTarget(node) => Some((node, 0, 6)),
        FieldRef::ProcessingInstructionData(node) => Some((node, 0, 7)),
        FieldRef::DoctypeName(node) => Some((node, 0, 8)),
        FieldRef::DoctypePublic(node) => Some((node, 0, 9)),
        FieldRef::DoctypeSystem(node) => Some((node, 0, 10)),
        _ => None,
    }
}

/// The entries of one record, sorted by attribute position and string.
pub(super) struct Run {
    entries: OwnedVec<DetailOrigin>,
}

/// The slots and runs of a table that holds a group. They are allocated when the first group is
/// inserted.
struct Table {
    /// The run of each node position, as its place in `runs` plus one; zero for a node with no
    /// run. It is as long as the highest position that has had a group, plus one.
    slots: Vec<usize>,
    slots_allocation: VectorCapacity<usize>,
    runs: Vec<Run>,
    runs_allocation: VectorCapacity<Run>,
}

pub(super) struct DetailOrigins {
    table: Option<Table>,
}

impl DetailOrigins {
    pub(super) fn new() -> Self {
        Self { table: None }
    }

    /// How many entries the table holds.
    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.table.as_ref().map_or(0, |table| {
            table.runs.iter().map(|run| run.entries.len()).sum()
        })
    }

    /// The group of `field` when it has one, without a charge.
    #[cfg(test)]
    pub(super) fn peek(&self, field: FieldRef) -> Option<&OriginGroupRef> {
        let (node, _, _) = key(field)?;
        let table = self.table.as_ref()?;
        let run = table.runs.get(table.slots.get(node)?.checked_sub(1)?)?;
        let position = run
            .entries
            .as_slice()
            .binary_search_by_key(&key(field), |entry| key(entry.field))
            .ok()?;
        run.entries
            .as_slice()
            .get(position)
            .map(|entry| &entry.group)
    }

    /// Whether `field` has a group, without a charge.
    #[cfg(test)]
    pub(super) fn has(&self, field: FieldRef) -> bool {
        self.peek(field).is_some()
    }

    /// Whether the table has allocated anything.
    #[cfg(test)]
    pub(super) fn is_unallocated(&self) -> bool {
        self.table.is_none()
    }

    /// The element, attribute position and string of each entry, in node order and then run
    /// order.
    #[cfg(test)]
    pub(super) fn keys(&self) -> Vec<(usize, usize, u8)> {
        let Some(table) = &self.table else {
            return Vec::new();
        };
        table
            .slots
            .iter()
            .filter_map(|slot| slot.checked_sub(1))
            .filter_map(|run| table.runs.get(run))
            .flat_map(|run| run.entries.as_slice())
            .filter_map(|entry| key(entry.field))
            .collect()
    }

    /// The bytes the table's vectors hold in capacity: the slots, the runs and each run's
    /// entries.
    #[cfg(test)]
    pub(super) fn capacity_bytes(&self) -> usize {
        self.table.as_ref().map_or(0, |table| {
            table.slots.capacity() * size_of::<usize>()
                + table.runs.capacity() * size_of::<Run>()
                + table
                    .runs
                    .iter()
                    .map(|run| run.entries.capacity() * size_of::<DetailOrigin>())
                    .sum::<usize>()
        })
    }

    /// The position of `field` in `run`, or where it would go. The probes of the binary search
    /// are charged before it reads.
    fn search(
        control: &mut ParseControl,
        run: &Run,
        field: FieldRef,
    ) -> Result<Result<usize, usize>, SessionError> {
        let Some(wanted) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let len = run.entries.len();
        let probes = usize::BITS
            .saturating_sub(len.leading_zeros())
            .saturating_add(1);
        control.charge(HtmlParseWorkCategory::RecordVisit, u64::from(probes))?;
        Ok(run
            .entries
            .as_slice()
            .binary_search_by_key(&wanted, |entry| key(entry.field).unwrap_or_default()))
    }

    /// The run of the element at `node`, after charging the read of its slot.
    fn run_of(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        let Some(table) = &self.table else {
            return Ok(None);
        };
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match table.slots.get(node).copied().filter(|slot| *slot != 0) {
            None => Ok(None),
            Some(slot) => slot
                .checked_sub(1)
                .map(Some)
                .ok_or_else(|| latch_registry_row(control)),
        }
    }

    /// The group of `field`, when it has one.
    pub(super) fn get(
        &self,
        control: &mut ParseControl,
        field: FieldRef,
    ) -> Result<Option<&OriginGroupRef>, SessionError> {
        let Some((node, _, _)) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let Some(run) = self.run_of(control, node)? else {
            return Ok(None);
        };
        let run = self
            .table
            .as_ref()
            .and_then(|table| table.runs.get(run))
            .ok_or_else(|| latch_registry_row(control))?;
        Ok(match Self::search(control, run, field)? {
            Ok(position) => run
                .entries
                .as_slice()
                .get(position)
                .map(|entry| &entry.group),
            Err(_) => None,
        })
    }

    /// Charges the move of `count` elements of `size` bytes before it happens, in quanta.
    fn charge_moves(
        control: &mut ParseControl,
        count: usize,
        size: usize,
    ) -> Result<(), SessionError> {
        let mut remaining = count;
        let per_quantum = WORK_QUANTUM.checked_div(size).unwrap_or(1).max(1);
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                chunk.saturating_mul(size) as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordWrite, chunk as u64)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Ok(())
    }

    /// Charges the rewriting of the leaves of `count` groups before it happens, in quanta: the
    /// visit of each group and the write of its leaf.
    fn charge_leaf_writes(control: &mut ParseControl, count: usize) -> Result<(), SessionError> {
        let mut remaining = count;
        let per_quantum = WORK_QUANTUM.max(1);
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            control.charge(HtmlParseWorkCategory::RecordVisit, chunk as u64)?;
            control.charge(HtmlParseWorkCategory::RecordWrite, chunk as u64)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Ok(())
    }

    /// Charges the filling of `count` empty slots before it happens, in quanta. Like the growth
    /// of a vector it visits no record: the slots are not looked up.
    fn charge_fill(control: &mut ParseControl, count: usize) -> Result<(), SessionError> {
        let mut remaining = count;
        let per_quantum = WORK_QUANTUM
            .checked_div(size_of::<usize>())
            .unwrap_or(1)
            .max(1);
        while remaining != 0 {
            let chunk = remaining.min(per_quantum);
            control.charge(
                HtmlParseWorkCategory::CopyByte,
                chunk.saturating_mul(size_of::<usize>()) as u64,
            )?;
            control.charge(HtmlParseWorkCategory::RecordWrite, chunk as u64)?;
            remaining = remaining.saturating_sub(chunk);
        }
        Ok(())
    }

    /// The table, created when the first group goes in.
    fn table(&mut self, control: &mut ParseControl) -> Result<&mut Table, SessionError> {
        if self.table.is_none() {
            let (slots, slots_allocation) = control.new_vector()?;
            let (runs, runs_allocation) = control.new_vector()?;
            self.table = Some(Table {
                slots,
                slots_allocation,
                runs,
                runs_allocation,
            });
        }
        self.table
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))
    }

    /// The run of the element at `node`, made when it has none. Everything that can refuse is
    /// admitted before the slots or the runs change.
    fn run_for(&mut self, control: &mut ParseControl, node: usize) -> Result<usize, SessionError> {
        let table = self.table(control)?;
        let needed = node
            .checked_add(1)
            .ok_or_else(|| latch_registry_row(control))?;
        let missing = needed.saturating_sub(table.slots.len());
        if missing != 0 {
            control.reserve_vector_geometric(
                &mut table.slots,
                &mut table.slots_allocation,
                missing,
            )?;
            Self::charge_fill(control, missing)?;
            table.slots.resize(needed, 0);
        }
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let existing = table.slots.get(node).copied().unwrap_or(0);
        if let Some(run) = existing.checked_sub(1) {
            return Ok(run);
        }
        control.reserve_vector_geometric(&mut table.runs, &mut table.runs_allocation, 1)?;
        control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            (const { size_of::<Run>() + size_of::<usize>() }) as u64,
        )?;
        let run = Run {
            entries: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
        };
        table.runs.push(run);
        let place = table.runs.len();
        let Some(slot) = table.slots.get_mut(node) else {
            return Err(latch_registry_row(control));
        };
        *slot = place;
        place
            .checked_sub(1)
            .ok_or_else(|| latch_registry_row(control))
    }

    /// Records the group of `field`, which has none yet. A field already in the table is a
    /// registry inconsistency. Only the entries of its element after the new one move.
    pub(super) fn insert(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
        group: OriginGroupRef,
    ) -> Result<(), SessionError> {
        let Some((node, _, _)) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let run = self.run_for(control, node)?;
        let table = self
            .table
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))?;
        let run = table
            .runs
            .get_mut(run)
            .ok_or_else(|| latch_registry_row(control))?;
        let position = match Self::search(control, run, field)? {
            Ok(_) => return Err(latch_registry_row(control)),
            Err(position) => position,
        };
        let len = run.entries.len();
        Self::charge_moves(
            control,
            len.saturating_sub(position),
            size_of::<DetailOrigin>(),
        )?;
        run.entries
            .try_push_geometric(control, DetailOrigin { field, group })?;
        // The shifted entries were charged above. The new entry was pushed at the end.
        if let Some(tail) = run
            .entries
            .as_mut_slice_prepaid(control)?
            .get_mut(position..)
        {
            tail.rotate_right(1);
        }
        Ok(())
    }

    /// Takes the group of `field` out of the table, when it has one. Only the entries of its
    /// element after it move.
    pub(super) fn take(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let Some((node, _, _)) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let Some(run) = self.run_of(control, node)? else {
            return Ok(None);
        };
        let table = self
            .table
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))?;
        let run = table
            .runs
            .get_mut(run)
            .ok_or_else(|| latch_registry_row(control))?;
        let Ok(position) = Self::search(control, run, field)? else {
            return Ok(None);
        };
        let len = run.entries.len();
        Self::charge_moves(
            control,
            len.saturating_sub(position).saturating_sub(1),
            size_of::<DetailOrigin>(),
        )?;
        // The last entry goes out first, which is the only step that can refuse; the entries
        // after the taken one then close over it in place.
        let Some(last) = run.entries.try_pop(control)? else {
            return Err(latch_registry_row(control));
        };
        let entries = run.entries.as_mut_slice_prepaid(control)?;
        let Some(slot) = entries.get_mut(position) else {
            // The taken entry was the last.
            return Ok(Some(last.group));
        };
        let taken = std::mem::replace(slot, last);
        if let Some(tail) = entries.get_mut(position..) {
            tail.rotate_left(1);
        }
        Ok(Some(taken.group))
    }

    /// Charges what [`Self::replace_prepaid`] does for `field`: the read of the slot of its
    /// record, the search of the record's run, and the write of the group.
    pub(super) fn charge_replace(
        &self,
        control: &mut ParseControl,
        field: FieldRef,
    ) -> Result<(), SessionError> {
        let Some((node, _, _)) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let Some(run) = self.run_of(control, node)? else {
            return Ok(());
        };
        let run = self
            .table
            .as_ref()
            .and_then(|table| table.runs.get(run))
            .ok_or_else(|| latch_registry_row(control))?;
        // The probes of the search are charged; the position is not needed here.
        let _ = Self::search(control, run, field)?;
        control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            size_of::<OriginGroupRef>() as u64,
        )?;
        Ok(())
    }

    /// Puts `group` in place of the group of `field`, which has one, and returns the group it
    /// replaced. No entry moves and nothing is allocated. The caller has charged it through
    /// [`Self::charge_replace`]; only a record that holds no group for the field, which the
    /// caller made sure of beforehand, refuses it, as the registry row.
    pub(super) fn replace_prepaid(
        &mut self,
        control: &mut ParseControl,
        field: FieldRef,
        group: OriginGroupRef,
    ) -> Result<OriginGroupRef, SessionError> {
        let Some((node, _, _)) = key(field) else {
            return Err(latch_registry_row(control));
        };
        let Some(run) = self.table.as_mut().and_then(|table| {
            let run = table.slots.get(node).copied()?.checked_sub(1)?;
            table.runs.get_mut(run)
        }) else {
            return Err(latch_registry_row(control));
        };
        let Ok(position) = run
            .entries
            .as_slice()
            .binary_search_by_key(&key(field), |entry| key(entry.field))
        else {
            return Err(latch_registry_row(control));
        };
        let entries = run.entries.as_mut_slice_prepaid(control)?;
        match entries.get_mut(position) {
            Some(entry) => Ok(std::mem::replace(&mut entry.group, group)),
            None => Err(latch_registry_row(control)),
        }
    }

    /// The entries of the attributes of `node` after position `removed` name one place lower, as
    /// the attributes themselves do once the one at `removed` is gone, and so do the leaves of
    /// their groups that name the attributes' strings in `evidence`. The fields of the removed
    /// attribute have been taken out already. Each moved entry costs one write of its field and
    /// one write of its group's leaf, so the step costs what the entries after the removed
    /// attribute are.
    pub(super) fn close_gap(
        &mut self,
        control: &mut ParseControl,
        evidence: &mut ParseEvidence,
        node: usize,
        removed: usize,
    ) -> Result<(), SessionError> {
        let Some(run) = self.run_of(control, node)? else {
            return Ok(());
        };
        let table = self
            .table
            .as_mut()
            .ok_or_else(|| latch_registry_row(control))?;
        let run = table
            .runs
            .get_mut(run)
            .ok_or_else(|| latch_registry_row(control))?;
        let first = match Self::search(control, run, FieldRef::AttributeName(node, removed))? {
            Ok(position) | Err(position) => position,
        };
        let moved = run.entries.len().saturating_sub(first);
        Self::charge_moves(control, moved, size_of::<DetailOrigin>())?;
        Self::charge_leaf_writes(control, moved)?;
        let entries = run.entries.as_mut_slice_prepaid(control)?;
        let Some(tail) = entries.get_mut(first..) else {
            return Err(latch_registry_row(control));
        };
        for entry in tail {
            // The fields of the removed attribute are gone, so every field here follows it.
            let lowered = entry
                .field
                .attribute()
                .filter(|&(owner, index, _)| owner == node && index > removed)
                .and_then(|(_, index, string)| {
                    index
                        .checked_sub(1)
                        .map(|lower| (lower, string, FieldRef::of_attribute(node, lower, string)))
                });
            let Some((lower, string, lowered)) = lowered else {
                return Err(latch_registry_row(control));
            };
            evidence.lower_attribute_leaf(control, &entry.group, node, lower, string)?;
            entry.field = lowered;
        }
        Ok(())
    }

    pub(super) fn release(self, control: &mut ParseControl) -> Result<(), SessionError> {
        let Some(Table {
            slots,
            slots_allocation,
            mut runs,
            runs_allocation,
        }) = self.table
        else {
            return Ok(());
        };
        let mut refused = None;
        for run in runs.drain(..) {
            keep_first(&mut refused, run.entries.release(control));
        }
        keep_first(
            &mut refused,
            release_vector(control, slots, slots_allocation),
        );
        keep_first(&mut refused, release_vector(control, runs, runs_allocation));
        refused.map_or(Ok(()), Err)
    }
}
