//! The text arena of one construction storage: the units of every Text and Comment record in one
//! growing vector, which publication turns into the one block of the result.
//!
//! A record holds a [`TextRun`] into the arena, never a string of its own. A text that receives
//! characters extends in place while it has room reserved after it or ends at the tail of the
//! arena, and otherwise moves once to a fresh region at the tail that is twice its new size. The
//! units that `n` appends to one text copy are then linear in the units the text holds at the end.

use super::{HtmlConstructionCause, HtmlConstructionError};
use crate::internal::values::TextBlock;
use std::{collections::TryReserveError, ops::Range};

/// A range of the units of a text arena, in 32-bit fields so that a record holds it in eight
/// bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextRef {
    start: u32,
    len: u32,
}

impl TextRef {
    /// The range of no units at the start of an arena.
    pub(crate) const EMPTY: Self = Self { start: 0, len: 0 };
    /// The range of `len` units from `start`, for the tests of the modules that hold ranges.
    #[cfg(test)]
    pub(crate) fn at(start: u32, len: u32) -> Self {
        Self { start, len }
    }
    /// The number of units in the range.
    pub(crate) fn len(self) -> usize {
        self.len as usize
    }
    /// The first unit of the range, as a row holds it.
    pub(crate) fn start(self) -> u32 {
        self.start
    }
    /// The number of units in the range, as a row holds it.
    pub(crate) fn stored_len(self) -> u32 {
        self.len
    }
    /// The range a row holds as `start` and `len`.
    pub(crate) fn from_stored(start: u32, len: u32) -> Self {
        Self { start, len }
    }
    /// The range as indices. An end that `usize` cannot hold saturates, which no arena holds, so
    /// the range is then one that `get` refuses.
    pub(crate) fn range(self) -> Range<usize> {
        // A `u32` fits `usize` on every supported target.
        let start = self.start as usize;
        start..start.saturating_add(self.len as usize)
    }
}

/// The text of a Text or Comment record while its storage is building: its range, and the number
/// of units reserved after it, which a text gets when it moves to a fresh region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextRun {
    text: TextRef,
    room: u32,
}

impl TextRun {
    /// The number of units the text holds.
    pub(crate) fn len(self) -> usize {
        self.text.len as usize
    }
    /// The range of the text in its arena.
    pub(crate) fn text(self) -> TextRef {
        self.text
    }
    /// The units reserved after the text.
    pub(crate) fn room(self) -> u32 {
        self.room
    }
    /// The run of `text` with `room` units reserved after it, as a row holds them.
    pub(crate) fn from_parts(text: TextRef, room: u32) -> Self {
        Self { text, room }
    }
}

/// How an append reaches a text, with the arena units it writes beyond the arena's length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Append {
    /// The units fit the room reserved after the text.
    Room,
    /// The text ends at the tail of the arena and extends there.
    Tail,
    /// The text moves to a fresh region of `region` units at the tail.
    Relocate {
        /// The units of the new region: the text, the appended units and the room after them.
        region: usize,
    },
}

impl Append {
    /// The units the arena must have capacity for beyond its length.
    pub(crate) fn written(self, added: usize) -> usize {
        match self {
            Self::Room => 0,
            Self::Tail => added,
            Self::Relocate { region } => region,
        }
    }
}

/// The units of the Text and Comment records of one storage.
#[derive(Debug, Default)]
pub(crate) struct TextArena {
    units: Vec<u16>,
    /// The units that records' texts hold. The rest of the arena is the regions texts left when
    /// they moved, the room reserved after relocated texts, and the units of records that were
    /// rolled back.
    used: usize,
}

impl TextArena {
    /// The most units an arena holds, so that every range of it is a handle of its block.
    pub(crate) const MAX_UNITS: usize = TextBlock::MAX_RANGE_UNITS;

    /// The units written, including those that no text holds any more.
    pub(crate) fn len(&self) -> usize {
        self.units.len()
    }

    /// The units the arena's allocation holds.
    pub(crate) fn capacity(&self) -> usize {
        self.units.capacity()
    }

    /// The units that records' texts hold.
    #[cfg(test)]
    pub(crate) fn used(&self) -> usize {
        self.used
    }

    /// The units of `text`, or the typed invalid-reference cause when the range is not in the
    /// arena.
    pub(crate) fn get(&self, text: TextRef) -> Result<&[u16], HtmlConstructionError> {
        self.units
            .get(text.range())
            .ok_or_else(|| HtmlConstructionCause::InvalidTextReference.error())
    }

    /// Checks that `added` more units keep the arena within [`Self::MAX_UNITS`].
    pub(crate) fn check_add(&self, added: usize) -> Result<(), HtmlConstructionError> {
        self.units
            .len()
            .checked_add(added)
            .filter(|total| *total <= Self::MAX_UNITS)
            .map(|_| ())
            .ok_or_else(|| HtmlConstructionCause::RepresentationExhausted.error())
    }

    /// Reserves room for exactly `additional` more units.
    pub(crate) fn reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.units.try_reserve_exact(additional)
    }

    /// Takes the units out of the arena, which is left empty, for the block of a publication.
    pub(crate) fn take_units(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.units)
    }

    /// Puts back the units `take_units` took, when the publication they were taken for was
    /// refused. The count of units that texts hold is unchanged by taking and restoring them.
    pub(crate) fn restore(&mut self, units: Vec<u16>) {
        self.units = units;
    }

    /// Whether the arena has capacity for `written` more units.
    fn has_room(&self, written: usize) -> bool {
        self.units.capacity().saturating_sub(self.units.len()) >= written
    }

    /// The run of a text of `units` written at the tail, which the caller reserved capacity for.
    pub(crate) fn push(&mut self, units: &[u16]) -> Result<TextRun, HtmlConstructionError> {
        let end = self
            .units
            .len()
            .checked_add(units.len())
            .filter(|end| *end <= Self::MAX_UNITS)
            .ok_or_else(|| HtmlConstructionCause::RepresentationExhausted.error())?;
        if !self.has_room(units.len()) {
            return Err(HtmlConstructionCause::CapacityMismatch.error());
        }
        let used = self
            .used
            .checked_add(units.len())
            .ok_or_else(|| HtmlConstructionCause::RepresentationExhausted.error())?;
        // Both fit `u32`: `end` is within `MAX_UNITS`, which is below `u32::MAX`.
        let start = self.units.len() as u32;
        let len = units.len() as u32;
        debug_assert!(end <= u32::MAX as usize);
        self.units.extend_from_slice(units);
        self.used = used;
        Ok(TextRun {
            text: TextRef { start, len },
            room: 0,
        })
    }

    /// How an append of `added` units reaches `run`, or the cause that the arena cannot hold the
    /// text or the region it needs.
    pub(crate) fn plan(&self, run: TextRun, added: usize) -> Result<Append, HtmlConstructionError> {
        let exhausted = || HtmlConstructionCause::RepresentationExhausted.error();
        let total = run
            .len()
            .checked_add(added)
            .filter(|total| *total <= Self::MAX_UNITS)
            .ok_or_else(exhausted)?;
        if added <= run.room as usize {
            return Ok(Append::Room);
        }
        let end = run.text.range().end;
        if run.room == 0 && end == self.units.len() {
            self.units
                .len()
                .checked_add(added)
                .filter(|end| *end <= Self::MAX_UNITS)
                .ok_or_else(exhausted)?;
            return Ok(Append::Tail);
        }
        // Twice the new size, or what the arena can still hold when that is less.
        let available = Self::MAX_UNITS
            .checked_sub(self.units.len())
            .ok_or_else(exhausted)?;
        let region = total.saturating_mul(2).min(available);
        if region < total {
            return Err(exhausted());
        }
        Ok(Append::Relocate { region })
    }

    /// Appends `units` to `run` as `plan` says, which the caller reserved capacity for, and
    /// returns the run after the append. A relocation copies the text, writes the units after it
    /// and fills the rest of the region with zero units.
    pub(crate) fn append(
        &mut self,
        run: TextRun,
        units: &[u16],
        plan: Append,
    ) -> Result<TextRun, HtmlConstructionError> {
        let added = units.len();
        let exhausted = || HtmlConstructionCause::RepresentationExhausted.error();
        if self.units.get(run.text.range()).is_none() {
            return Err(HtmlConstructionCause::InvalidTextReference.error());
        }
        let total = run
            .len()
            .checked_add(added)
            .filter(|total| *total <= Self::MAX_UNITS)
            .ok_or_else(exhausted)?;
        let used = self.used.checked_add(added).ok_or_else(exhausted)?;
        if !self.has_room(plan.written(added)) {
            return Err(HtmlConstructionCause::CapacityMismatch.error());
        }
        let len = total as u32;
        match plan {
            Append::Room => {
                let end = run.text.range().end;
                let room = run
                    .room
                    .checked_sub(added as u32)
                    .ok_or_else(|| HtmlConstructionCause::CapacityMismatch.error())?;
                let place = self
                    .units
                    .get_mut(end..end.checked_add(added).ok_or_else(exhausted)?)
                    .ok_or_else(|| HtmlConstructionCause::InvalidTextReference.error())?;
                place.copy_from_slice(units);
                self.used = used;
                Ok(TextRun {
                    text: TextRef {
                        start: run.text.start,
                        len,
                    },
                    room,
                })
            }
            Append::Tail => {
                if run.room != 0 || run.text.range().end != self.units.len() {
                    return Err(HtmlConstructionCause::CapacityMismatch.error());
                }
                self.units.extend_from_slice(units);
                self.used = used;
                Ok(TextRun {
                    text: TextRef {
                        start: run.text.start,
                        len,
                    },
                    room: 0,
                })
            }
            Append::Relocate { region } => {
                let start = self.units.len();
                let end = start.checked_add(region).ok_or_else(exhausted)?;
                if region < total || end > Self::MAX_UNITS {
                    return Err(HtmlConstructionCause::CapacityMismatch.error());
                }
                self.units.extend_from_within(run.text.range());
                self.units.extend_from_slice(units);
                self.units.resize(end, 0);
                self.used = used;
                Ok(TextRun {
                    text: TextRef {
                        start: start as u32,
                        len,
                    },
                    room: (region - total) as u32,
                })
            }
        }
    }

    /// Writes `units` over the range `text` names, which they do not exceed, and returns the
    /// range they take. The units of `text` beyond them stay written and no longer count as held.
    pub(crate) fn overwrite(
        &mut self,
        text: TextRef,
        units: &[u16],
    ) -> Result<TextRef, HtmlConstructionError> {
        let released = text
            .len()
            .checked_sub(units.len())
            .ok_or_else(|| HtmlConstructionCause::CapacityMismatch.error())?;
        let used = self
            .used
            .checked_sub(released)
            .ok_or_else(|| HtmlConstructionCause::ReleaseUnderflow.error())?;
        let start = text.range().start;
        let end = start
            .checked_add(units.len())
            .ok_or_else(|| HtmlConstructionCause::RepresentationExhausted.error())?;
        self.units
            .get_mut(start..end)
            .ok_or_else(|| HtmlConstructionCause::InvalidTextReference.error())?
            .copy_from_slice(units);
        self.used = used;
        Ok(TextRef {
            start: text.start,
            // The count fits `u32`: it does not exceed the length of `text`.
            len: units.len() as u32,
        })
    }

    /// Takes `units` out of the count of units that records' texts hold, for records rolled back.
    /// The units themselves stay written.
    pub(crate) fn forget(&mut self, units: usize) -> Result<(), HtmlConstructionError> {
        self.used = self
            .used
            .checked_sub(units)
            .ok_or_else(|| HtmlConstructionCause::ReleaseUnderflow.error())?;
        Ok(())
    }

    /// Whether the units no text holds exceed a quarter of the arena's allocation, so that
    /// publication packs the arena before it becomes the block.
    pub(crate) fn wants_packing(&self) -> bool {
        self.units
            .capacity()
            .saturating_sub(self.used)
            .saturating_mul(4)
            > self.units.capacity()
    }
}

impl TextRun {
    /// A run whose range no arena of a few units holds, for the tests of the refusals that name
    /// such a range.
    #[cfg(test)]
    pub(crate) fn invalid() -> Self {
        Self {
            text: TextRef {
                start: u32::MAX - 8,
                len: 4,
            },
            room: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arena(capacity: usize) -> TextArena {
        let mut arena = TextArena::default();
        arena.reserve_exact(capacity).unwrap();
        arena
    }
    fn units(arena: &TextArena, run: TextRun) -> Vec<u16> {
        arena.get(run.text()).unwrap().to_vec()
    }
    fn append(arena: &mut TextArena, run: TextRun, units: &[u16]) -> TextRun {
        let plan = arena.plan(run, units.len()).unwrap();
        let needed = arena.len() + plan.written(units.len());
        if needed > arena.capacity() {
            arena.reserve_exact(needed - arena.len()).unwrap();
        }
        arena.append(run, units, plan).unwrap()
    }

    #[test]
    fn a_range_the_arena_does_not_hold_is_the_typed_invalid_reference() {
        let mut arena = arena(8);
        let run = arena.push(&[1, 2, 3]).unwrap();
        assert_eq!(units(&arena, run), [1, 2, 3]);
        let error = arena.get(TextRun::invalid().text()).unwrap_err();
        assert_eq!(error.cause(), HtmlConstructionCause::InvalidTextReference);
    }

    #[test]
    fn a_text_at_the_tail_extends_in_place_and_any_other_text_moves_once_with_room() {
        let mut arena = arena(64);
        let first = arena.push(&[1]).unwrap();
        let second = arena.push(&[2]).unwrap();
        // The first text is not at the tail: it moves to a region of twice its new size.
        assert_eq!(
            arena.plan(first, 2).unwrap(),
            Append::Relocate { region: 6 }
        );
        let first = append(&mut arena, first, &[3, 4]);
        assert_eq!(units(&arena, first), [1, 3, 4]);
        assert_eq!(units(&arena, second), [2]);
        // The move left the room after the text, so the next append takes it in place.
        assert_eq!(arena.plan(first, 3).unwrap(), Append::Room);
        let first = append(&mut arena, first, &[5, 6, 7]);
        assert_eq!(units(&arena, first), [1, 3, 4, 5, 6, 7]);
        // The room is used up: the text is at the tail of its region but not of the arena.
        assert!(matches!(
            arena.plan(first, 1).unwrap(),
            Append::Relocate { .. } | Append::Tail
        ));
        // The second text still reads its units.
        assert_eq!(units(&arena, second), [2]);
        // A text at the tail extends without room.
        let third = arena.push(&[9]).unwrap();
        assert_eq!(arena.plan(third, 4).unwrap(), Append::Tail);
        let third = append(&mut arena, third, &[1, 1, 1, 1]);
        assert_eq!(units(&arena, third), [9, 1, 1, 1, 1]);
    }

    #[test]
    fn the_units_of_a_text_extended_around_another_text_are_written_a_bounded_number_of_times() {
        for appends in [1_000usize, 2_000, 4_000, 8_000] {
            let mut arena = arena(0);
            let mut a = {
                arena.reserve_exact(1).unwrap();
                arena.push(&[1]).unwrap()
            };
            let mut b = {
                arena.reserve_exact(1).unwrap();
                arena.push(&[2]).unwrap()
            };
            let mut written = 0usize;
            let growth = |arena: &mut TextArena, run: TextRun, unit: u16, written: &mut usize| {
                let plan = arena.plan(run, 1).unwrap();
                *written += plan.written(1);
                let needed = arena.len() + plan.written(1);
                if needed > arena.capacity() {
                    let target = needed.max(arena.capacity() * 2);
                    arena.reserve_exact(target - arena.len()).unwrap();
                }
                arena.append(run, &[unit], plan).unwrap()
            };
            for _ in 0..appends {
                a = growth(&mut arena, a, 7, &mut written);
                b = growth(&mut arena, b, 8, &mut written);
            }
            assert_eq!(a.len(), appends + 1);
            assert_eq!(b.len(), appends + 1);
            // The texts of `2 * (appends + 1)` units are written fewer than four times each, and
            // what no text holds is less than three times what they do.
            let total = 2 * (appends + 1);
            assert!(written < 4 * total, "{written} units for {total}");
            assert!(arena.len() - arena.used() < 3 * total);
            assert!(units(&arena, a).iter().skip(1).all(|unit| *unit == 7));
            assert!(units(&arena, b).iter().skip(1).all(|unit| *unit == 8));
        }
    }

    #[test]
    fn rolled_back_units_stay_written_and_leave_the_count_of_held_units() {
        let mut arena = arena(8);
        let kept = arena.push(&[1, 2]).unwrap();
        let dropped = arena.push(&[3, 4, 5]).unwrap();
        assert_eq!(arena.used(), 5);
        arena.forget(dropped.len()).unwrap();
        assert_eq!(arena.used(), 2);
        assert_eq!(arena.len(), 5);
        assert_eq!(units(&arena, kept), [1, 2]);
        let error = arena.forget(3).unwrap_err();
        assert_eq!(error.cause(), HtmlConstructionCause::ReleaseUnderflow);
    }

    #[test]
    fn an_arena_with_more_than_a_quarter_spare_asks_to_be_packed() {
        let mut arena = arena(100);
        arena.push(&[0; 75]).unwrap();
        assert!(
            !arena.wants_packing(),
            "a quarter spare is not more than a quarter"
        );
        let mut arena = self::arena(100);
        arena.push(&[0; 74]).unwrap();
        assert!(arena.wants_packing());
    }

    #[test]
    fn an_append_the_arena_cannot_hold_is_refused_before_it_writes() {
        let mut arena = arena(4);
        let run = arena.push(&[1, 2]).unwrap();
        // Two units fit the capacity at the tail, but the append is planned for a capacity the
        // caller did not reserve when it moves.
        let other = arena.push(&[3]).unwrap();
        let plan = arena.plan(run, 4).unwrap();
        let error = arena.append(run, &[9; 4], plan).unwrap_err();
        assert_eq!(error.cause(), HtmlConstructionCause::CapacityMismatch);
        assert_eq!(units(&arena, run), [1, 2]);
        assert_eq!(units(&arena, other), [3]);
        assert_eq!(arena.len(), 3);
        assert_eq!(arena.used(), 3);
    }
}
