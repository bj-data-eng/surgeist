//! The attribute table of one construction storage: the attributes of every element in one vector
//! of rows, with one run of rows for each element that has attributes.
//!
//! A row holds the interned name of an attribute and a [`TextRef`] to its value in the storage's
//! text arena, so the table holds no string of its own. A run names the rows of one element: its
//! first row, how many rows it holds and how many it has room for. An attribute that finds its
//! element's run full extends it in place when the run ends at the tail of the table, and
//! otherwise moves the run once to a fresh region at the tail that is twice its new size, so the
//! rows that `n` appends copy are linear in the rows the run holds at the end. The rows a run
//! left when it moved, and the room reserved after a run, hold the filler row until a run writes
//! them. Publication copies the live rows of every run into the exact table of the result, and
//! the published rows hold the names and the values of the attributes of every element in one
//! vector that the runs of the result name.

use super::text::TextRef;
use super::{HtmlConstructionCause, HtmlConstructionError, names::NameEntry};
use crate::internal::values::{DocumentString, Namespace, QualifiedName};
use std::{fmt, num::NonZeroU32, sync::Arc};

/// An attribute of an element that is still being built: its interned name and the range of its
/// value in the text arena.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttrRow {
    name: Arc<NameEntry>,
    value: TextRef,
}
impl AttrRow {
    pub(crate) fn new(name: Arc<NameEntry>, value: TextRef) -> Self {
        Self { name, value }
    }
    /// The interned name of the attribute.
    pub(crate) fn entry(&self) -> &Arc<NameEntry> {
        &self.name
    }
    /// The exact name of the attribute.
    pub(crate) fn name(&self) -> &QualifiedName {
        self.name.name()
    }
    /// The range of the value in the text arena.
    pub(crate) fn value(&self) -> TextRef {
        self.value
    }
}

/// The position of the run of an element in the table's runs, which a record holds in four
/// bytes. The stored value is the position plus one, which gives `Option<RunIx>` the size of a
/// `RunIx`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RunIx(NonZeroU32);
impl RunIx {
    fn new(position: usize) -> Option<Self> {
        let stored = u32::try_from(position).ok()?.checked_add(1)?;
        NonZeroU32::new(stored).map(Self)
    }
    fn position(self) -> usize {
        // A `u32` fits `usize` on every supported target; the stored value is never zero.
        self.0.get().saturating_sub(1) as usize
    }
    /// The stored value, the position plus one, which a row holds in one word.
    pub(crate) fn stored(self) -> u32 {
        self.0.get()
    }
    /// The run whose stored value is `stored`, or `None` for zero, which names no run.
    pub(crate) fn from_stored(stored: u32) -> Option<Self> {
        NonZeroU32::new(stored).map(Self)
    }
    /// A run that no table of a few runs holds, for the tests of the refusals that name such a
    /// run.
    #[cfg(test)]
    pub(crate) fn beyond() -> Self {
        Self(NonZeroU32::MAX)
    }
}

/// The rows of one element: where its first row is, how many rows it holds and how many it has
/// room for. The rows after the first `len` and within `cap` hold the filler row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttrRun {
    start: u32,
    len: u32,
    cap: u32,
}
impl AttrRun {
    /// The number of attributes the run holds.
    pub(crate) fn len(self) -> usize {
        self.len as usize
    }
    /// The number of rows the run has room for.
    pub(crate) fn cap(self) -> usize {
        self.cap as usize
    }
    fn start(self) -> usize {
        self.start as usize
    }
    fn end(self) -> usize {
        self.start().saturating_add(self.cap())
    }
}

/// How an append of attributes reaches the run of an element, with the rows it writes to the
/// table beyond the table's length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RowAppend {
    /// The attributes fit the room of the run.
    Room,
    /// The element has no run: a new run of `cap` rows is made at the tail.
    New {
        /// The rows of the new run.
        cap: usize,
    },
    /// The run ends at the tail of the table and extends there by `extra` rows.
    Tail {
        /// The rows the run gains.
        extra: usize,
    },
    /// The run moves to a fresh region of `region` rows at the tail.
    Relocate {
        /// The rows of the new region: the attributes, the appended ones and the room after them.
        region: usize,
    },
}
impl RowAppend {
    /// The rows the table must have capacity for beyond its length.
    pub(crate) fn written(self) -> usize {
        match self {
            Self::Room => 0,
            Self::New { cap } => cap,
            Self::Tail { extra } => extra,
            Self::Relocate { region } => region,
        }
    }
    /// The runs the table must have capacity for beyond its length.
    pub(crate) fn runs(self) -> usize {
        usize::from(matches!(self, Self::New { .. }))
    }
}

/// The rows and runs of the attributes of one storage's elements.
///
/// The row that holds the room of a run, and the rows a run left behind, is a filler row: an empty
/// name and an empty value. No run counts it as an attribute. Every filler row of a table holds
/// the one name that the table made for them, so the reference count of that name is written by
/// the thread that owns the table alone. The table allocates that name, one small allocation that
/// no limit counts, when it first makes a filler row.
#[derive(Default)]
pub(crate) struct AttributeTable {
    rows: Vec<AttrRow>,
    runs: Vec<AttrRun>,
    filler: Option<Arc<NameEntry>>,
}
impl fmt::Debug for AttributeTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeTable")
            .field("rows", &self.rows)
            .field("runs", &self.runs)
            .finish()
    }
}

fn invalid() -> HtmlConstructionError {
    HtmlConstructionCause::InvalidAttributeReference.error()
}
fn exhausted() -> HtmlConstructionError {
    HtmlConstructionCause::RepresentationExhausted.error()
}
fn mismatch() -> HtmlConstructionError {
    HtmlConstructionCause::CapacityMismatch.error()
}

impl AttributeTable {
    /// A filler row, which holds the table's own filler name.
    fn filler_row(&mut self) -> AttrRow {
        let name = self.filler.get_or_insert_with(|| {
            Arc::new(NameEntry::new(
                QualifiedName::new(Namespace::none(), None, DocumentString::from("")),
                0,
                0,
            ))
        });
        AttrRow {
            name: name.clone(),
            value: TextRef::EMPTY,
        }
    }

    /// The most rows a table holds, so that every start and length of a run fits 32 bits.
    pub(crate) const MAX_ROWS: usize = u32::MAX as usize;

    /// The rows written, including the room and the rows no run holds any more.
    pub(crate) fn rows_len(&self) -> usize {
        self.rows.len()
    }
    /// The rows the table's allocation holds.
    pub(crate) fn rows_capacity(&self) -> usize {
        self.rows.capacity()
    }
    /// The runs made, including those of records that were rolled back.
    pub(crate) fn runs_len(&self) -> usize {
        self.runs.len()
    }
    /// The runs the table's allocation holds.
    pub(crate) fn runs_capacity(&self) -> usize {
        self.runs.capacity()
    }

    /// Reserves room for exactly `additional` more rows.
    pub(crate) fn reserve_rows(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.rows.try_reserve_exact(additional)
    }
    /// Reserves room for exactly `additional` more runs.
    pub(crate) fn reserve_runs(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.runs.try_reserve_exact(additional)
    }

    /// The run `ix` names, or the invalid-reference cause when the table has no such run.
    pub(crate) fn run(&self, ix: RunIx) -> Result<AttrRun, HtmlConstructionError> {
        self.runs.get(ix.position()).copied().ok_or_else(invalid)
    }

    /// The rows an element holds: none for an element without a run, and otherwise the live rows
    /// of its run, or the invalid-reference cause when the table does not hold them.
    pub(crate) fn live(&self, ix: Option<RunIx>) -> Result<&[AttrRow], HtmlConstructionError> {
        let Some(ix) = ix else {
            return Ok(&[]);
        };
        let run = self.run(ix)?;
        let end = run.start().checked_add(run.len()).ok_or_else(invalid)?;
        self.rows.get(run.start()..end).ok_or_else(invalid)
    }

    /// The length and capacity of the run of an element, or zero for an element without one.
    pub(crate) fn dimensions(
        &self,
        ix: Option<RunIx>,
    ) -> Result<(usize, usize), HtmlConstructionError> {
        match ix {
            None => Ok((0, 0)),
            Some(ix) => self.run(ix).map(|run| (run.len(), run.cap())),
        }
    }

    /// How an append of `added` attributes reaches the run of an element, or the cause that the
    /// table cannot hold the run or the region it needs.
    pub(crate) fn plan(
        &self,
        ix: Option<RunIx>,
        added: usize,
    ) -> Result<RowAppend, HtmlConstructionError> {
        let available = Self::MAX_ROWS
            .checked_sub(self.rows.len())
            .ok_or_else(exhausted)?;
        let Some(ix) = ix else {
            if added > available {
                return Err(exhausted());
            }
            return Ok(RowAppend::New { cap: added });
        };
        let run = self.run(ix)?;
        let total = run
            .len()
            .checked_add(added)
            .filter(|total| *total <= Self::MAX_ROWS)
            .ok_or_else(exhausted)?;
        if total <= run.cap() {
            return Ok(RowAppend::Room);
        }
        let extra = total - run.cap();
        if run.end() == self.rows.len() {
            if extra > available {
                return Err(exhausted());
            }
            return Ok(RowAppend::Tail { extra });
        }
        // Twice the new size, or what the table can still hold when that is less.
        let region = total.saturating_mul(2).min(available);
        if region < total {
            return Err(exhausted());
        }
        Ok(RowAppend::Relocate { region })
    }

    /// Whether the table has capacity for the rows and runs `plan` writes.
    fn has_room(&self, plan: RowAppend) -> bool {
        self.rows.capacity().saturating_sub(self.rows.len()) >= plan.written()
            && self.runs.capacity().saturating_sub(self.runs.len()) >= plan.runs()
    }

    /// Makes room in the run of an element as `plan` says, which the caller reserved capacity
    /// for, and returns the run's index. An element without a run gets one; a relocation copies
    /// the live rows to the new region and leaves the old rows as filler.
    pub(crate) fn make_room(
        &mut self,
        ix: Option<RunIx>,
        plan: RowAppend,
    ) -> Result<RunIx, HtmlConstructionError> {
        if !self.has_room(plan) {
            return Err(mismatch());
        }
        let filler = self.filler_row();
        match (ix, plan) {
            (Some(ix), RowAppend::Room) => {
                self.run(ix)?;
                Ok(ix)
            }
            (None, RowAppend::New { cap }) => {
                let ix = RunIx::new(self.runs.len()).ok_or_else(exhausted)?;
                let start = u32::try_from(self.rows.len()).map_err(|_| exhausted())?;
                let cap32 = u32::try_from(cap).map_err(|_| exhausted())?;
                start.checked_add(cap32).ok_or_else(exhausted)?;
                self.runs.push(AttrRun {
                    start,
                    len: 0,
                    cap: cap32,
                });
                self.rows.extend((0..cap).map(|_| filler.clone()));
                Ok(ix)
            }
            (Some(ix), RowAppend::Tail { extra }) => {
                let run = self.run(ix)?;
                if run.end() != self.rows.len() {
                    return Err(mismatch());
                }
                let cap = u32::try_from(run.cap().checked_add(extra).ok_or_else(exhausted)?)
                    .map_err(|_| exhausted())?;
                self.rows.extend((0..extra).map(|_| filler.clone()));
                let slot = self.runs.get_mut(ix.position()).ok_or_else(invalid)?;
                slot.cap = cap;
                Ok(ix)
            }
            (Some(ix), RowAppend::Relocate { region }) => {
                let run = self.run(ix)?;
                let start = self.rows.len();
                let end = start.checked_add(region).ok_or_else(exhausted)?;
                if region < run.len() || end > Self::MAX_ROWS {
                    return Err(mismatch());
                }
                let live_end = run.start().checked_add(run.len()).ok_or_else(invalid)?;
                if self.rows.get(run.start()..live_end).is_none() {
                    return Err(invalid());
                }
                // The live rows are copied into the room reserved at the tail, and the rows they
                // leave become filler, which frees their names.
                for at in run.start()..live_end {
                    let row = self.rows.get(at).ok_or_else(invalid)?.clone();
                    self.rows.push(row);
                    *self.rows.get_mut(at).ok_or_else(invalid)? = filler.clone();
                }
                self.rows
                    .extend((run.len()..region).map(|_| filler.clone()));
                let slot = self.runs.get_mut(ix.position()).ok_or_else(invalid)?;
                slot.start = u32::try_from(start).map_err(|_| exhausted())?;
                slot.cap = u32::try_from(region).map_err(|_| exhausted())?;
                Ok(ix)
            }
            _ => Err(mismatch()),
        }
    }

    /// Writes `row` as the next attribute of the run `ix`, which has room for it.
    pub(crate) fn push(&mut self, ix: RunIx, row: AttrRow) -> Result<usize, HtmlConstructionError> {
        let run = self.run(ix)?;
        if run.len >= run.cap {
            return Err(mismatch());
        }
        let position = run.start().checked_add(run.len()).ok_or_else(invalid)?;
        let slot = self.rows.get_mut(position).ok_or_else(invalid)?;
        *slot = row;
        let index = run.len();
        let held = self.runs.get_mut(ix.position()).ok_or_else(invalid)?;
        held.len = run.len.checked_add(1).ok_or_else(exhausted)?;
        Ok(index)
    }

    /// Replaces the attribute at `index` of the run `ix` with `row`.
    pub(crate) fn replace(
        &mut self,
        ix: RunIx,
        index: usize,
        row: AttrRow,
    ) -> Result<(), HtmlConstructionError> {
        let run = self.run(ix)?;
        if index >= run.len() {
            return Err(HtmlConstructionCause::OperandMissing.error());
        }
        let position = run.start().checked_add(index).ok_or_else(invalid)?;
        *self.rows.get_mut(position).ok_or_else(invalid)? = row;
        Ok(())
    }

    /// Removes the attribute at `index` of the run `ix`, moving the rows after it down by one
    /// place; the last row becomes the filler.
    pub(crate) fn remove(&mut self, ix: RunIx, index: usize) -> Result<(), HtmlConstructionError> {
        let run = self.run(ix)?;
        if index >= run.len() {
            return Err(HtmlConstructionCause::OperandMissing.error());
        }
        let from = run.start().checked_add(index).ok_or_else(invalid)?;
        let to = run.start().checked_add(run.len()).ok_or_else(invalid)?;
        self.rows
            .get_mut(from..to)
            .ok_or_else(invalid)?
            .rotate_left(1);
        let filler = self.filler_row();
        let last = to.checked_sub(1).ok_or_else(invalid)?;
        *self.rows.get_mut(last).ok_or_else(invalid)? = filler;
        let held = self.runs.get_mut(ix.position()).ok_or_else(invalid)?;
        held.len = run.len.saturating_sub(1);
        Ok(())
    }
}

/// An attribute of a published element: its interned name and its value, a range of the one
/// block of the result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PublishedAttr {
    name: Arc<NameEntry>,
    value: DocumentString,
}
impl PublishedAttr {
    pub(crate) fn new(name: Arc<NameEntry>, value: DocumentString) -> Self {
        Self { name, value }
    }
    pub(crate) fn name(&self) -> &QualifiedName {
        self.name.name()
    }
    pub(crate) fn value(&self) -> &DocumentString {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(local: &str, start: u32) -> AttrRow {
        AttrRow::new(
            Arc::new(NameEntry::new(
                QualifiedName::new(Namespace::none(), None, DocumentString::from(local)),
                u64::from(start),
                0,
            )),
            TextRef::at(start, 1),
        )
    }
    fn table(rows: usize, runs: usize) -> AttributeTable {
        let mut table = AttributeTable::default();
        table.reserve_rows(rows).unwrap();
        table.reserve_runs(runs).unwrap();
        table
    }
    fn append(table: &mut AttributeTable, ix: Option<RunIx>, row: AttrRow) -> RunIx {
        let plan = table.plan(ix, 1).unwrap();
        let needed = table.rows_len() + plan.written();
        if needed > table.rows_capacity() {
            let target = needed.max(table.rows_capacity() * 2);
            table.reserve_rows(target - table.rows_len()).unwrap();
        }
        if table.runs_len() + plan.runs() > table.runs_capacity() {
            table.reserve_runs(plan.runs()).unwrap();
        }
        let ix = table.make_room(ix, plan).unwrap();
        table.push(ix, row).unwrap();
        ix
    }
    fn names(table: &AttributeTable, ix: RunIx) -> Vec<String> {
        table
            .live(Some(ix))
            .unwrap()
            .iter()
            .map(|row| row.name().local_name().to_utf8().unwrap())
            .collect()
    }

    #[test]
    fn an_element_without_a_run_holds_no_rows_and_a_run_the_table_lacks_is_invalid() {
        let table = table(4, 2);
        assert!(table.live(None).unwrap().is_empty());
        let ix = RunIx::new(0).unwrap();
        let error = table.live(Some(ix)).unwrap_err();
        assert_eq!(
            error.cause(),
            HtmlConstructionCause::InvalidAttributeReference
        );
    }

    #[test]
    fn the_rows_that_fill_a_run_belong_to_their_table_alone() {
        // Filler rows hold a reference to one name, so a name shared by every table of every
        // thread would make each table's reference counts a contended write.
        let mut first = table(3, 1);
        let mut second = table(3, 1);
        for table in [&mut first, &mut second] {
            let plan = table.plan(None, 3).unwrap();
            table.make_room(None, plan).unwrap();
        }
        let held = |table: &AttributeTable, at: usize| table.rows.get(at).unwrap().name.clone();
        assert!(Arc::ptr_eq(&held(&first, 0), &held(&first, 2)));
        assert!(!Arc::ptr_eq(&held(&first, 0), &held(&second, 0)));
    }

    #[test]
    fn a_run_at_the_tail_extends_in_place_and_any_other_run_moves_once_with_room() {
        let mut table = table(32, 4);
        let a = append(&mut table, None, row("a0", 0));
        let b = append(&mut table, None, row("b0", 1));
        // The run of `a` is not at the tail: it moves to a region of twice its new size.
        assert_eq!(
            table.plan(Some(a), 1).unwrap(),
            RowAppend::Relocate { region: 4 }
        );
        append(&mut table, Some(a), row("a1", 2));
        assert_eq!(names(&table, a), ["a0", "a1"]);
        assert_eq!(names(&table, b), ["b0"]);
        // The move left room after the rows, so the next append takes it in place.
        assert_eq!(table.plan(Some(a), 1).unwrap(), RowAppend::Room);
        append(&mut table, Some(a), row("a2", 3));
        assert_eq!(names(&table, a), ["a0", "a1", "a2"]);
        // A run at the tail extends without moving.
        let c = append(&mut table, None, row("c0", 4));
        assert_eq!(
            table.plan(Some(c), 1).unwrap(),
            RowAppend::Tail { extra: 1 }
        );
        append(&mut table, Some(c), row("c1", 5));
        assert_eq!(names(&table, c), ["c0", "c1"]);
        assert_eq!(names(&table, b), ["b0"]);
    }

    #[test]
    fn the_rows_copied_by_alternating_appends_to_two_runs_are_linear_in_the_rows_held() {
        for appends in [1_000usize, 2_000, 4_000, 8_000] {
            let mut table = table(0, 2);
            let mut a = append(&mut table, None, row("x", 0));
            let mut b = append(&mut table, None, row("y", 0));
            let mut copied = 0usize;
            for _ in 0..appends {
                for run in [&mut a, &mut b] {
                    let plan = table.plan(Some(*run), 1).unwrap();
                    if let RowAppend::Relocate { .. } = plan {
                        copied += table.run(*run).unwrap().len();
                    }
                    *run = append(&mut table, Some(*run), row("z", 0));
                }
            }
            let total = 2 * (appends + 1);
            assert!(copied < 4 * total, "{copied} rows copied for {total}");
            assert_eq!(table.dimensions(Some(a)).unwrap().0, appends + 1);
            assert_eq!(table.dimensions(Some(b)).unwrap().0, appends + 1);
        }
    }

    #[test]
    fn a_removal_closes_the_gap_and_a_replacement_keeps_the_place() {
        let mut table = table(8, 1);
        let ix = append(&mut table, None, row("a", 0));
        append(&mut table, Some(ix), row("b", 1));
        append(&mut table, Some(ix), row("c", 2));
        table.replace(ix, 1, row("B", 3)).unwrap();
        assert_eq!(names(&table, ix), ["a", "B", "c"]);
        table.remove(ix, 0).unwrap();
        assert_eq!(names(&table, ix), ["B", "c"]);
        let error = table.remove(ix, 2).unwrap_err();
        assert_eq!(error.cause(), HtmlConstructionCause::OperandMissing);
    }
}
