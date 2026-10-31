//! Invocation-local custody of original default-ignorable scalars.
//!
//! This is deliberately not general glyph lineage. Only original DI scalars
//! have identities, and only native semantic operations establish normal output.

use super::buffer::{hb_buffer_t, GlyphInfo, ShapingRefusal, ShapingStatus};
use alloc::vec::Vec;
use core::mem::size_of;

const NORMAL: u32 = 1 << 31;
const ID: u32 = NORMAL - 1;

/// Borrowed default-ignorable accounting for one complete shaped buffer.
///
/// Input positions count original body scalars, excluding external context.
/// Glyph positions index the final glyph arrays, including direction reversal.
/// Normal inputs are omitted even if another descendant is a residual slot.
pub struct DefaultIgnorableReceipt<'a> {
    residual_inputs: &'a [u32],
    deleted_inputs: &'a [u32],
    residual_glyphs: &'a [u32],
}

impl DefaultIgnorableReceipt<'_> {
    /// Increasing scalar ordinals with only unresolved surviving descendants.
    pub fn residual_input_positions(&self) -> &[u32] {
        self.residual_inputs
    }

    /// Increasing scalar ordinals with no surviving descendant.
    pub fn deleted_input_positions(&self) -> &[u32] {
        self.deleted_inputs
    }

    /// Increasing final glyph indices of unresolved default-ignorable output.
    pub fn residual_glyph_positions(&self) -> &[u32] {
        self.residual_glyphs
    }
}

#[derive(Clone, Copy)]
struct Set {
    start: usize,
    len: usize,
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) active: bool,
    sets: Vec<Set>,
    members: Vec<u32>,
    candidates: Vec<u32>,
    residual_inputs: Vec<u32>,
    deleted_inputs: Vec<u32>,
    residual_glyphs: Vec<u32>,
    finalized: bool,
}

impl State {
    pub(crate) fn has_candidates(&self) -> bool {
        !self.candidates.is_empty()
    }

    pub(crate) fn bytes(&self) -> Option<usize> {
        self.sets
            .capacity()
            .checked_mul(size_of::<Set>())?
            .checked_add(self.members.capacity().checked_mul(size_of::<u32>())?)?
            .checked_add(self.candidates.capacity().checked_mul(size_of::<u32>())?)?
            .checked_add(
                self.residual_inputs
                    .capacity()
                    .checked_mul(size_of::<u32>())?,
            )?
            .checked_add(
                self.deleted_inputs
                    .capacity()
                    .checked_mul(size_of::<u32>())?,
            )?
            .checked_add(
                self.residual_glyphs
                    .capacity()
                    .checked_mul(size_of::<u32>())?,
            )
    }

    pub(crate) fn receipt(&self) -> DefaultIgnorableReceipt<'_> {
        debug_assert!(self.finalized);
        DefaultIgnorableReceipt {
            residual_inputs: &self.residual_inputs,
            deleted_inputs: &self.deleted_inputs,
            residual_glyphs: &self.residual_glyphs,
        }
    }
}

/// Unicode 17 DerivedCoreProperties, Default_Ignorable_Code_Point.
/// Kept separate from the provider's intentionally modified lookup-skip fact.
pub(crate) fn is_default_ignorable(cp: u32) -> bool {
    matches!(cp,
        0x00ad | 0x034f | 0x061c | 0x115f..=0x1160 | 0x17b4..=0x17b5 |
        0x180b..=0x180f | 0x200b..=0x200f | 0x202a..=0x202e |
        0x2060..=0x206f | 0x3164 | 0xfe00..=0xfe0f | 0xfeff | 0xffa0 |
        0xfff0..=0xfff8 | 0x1bca0..=0x1bca3 | 0x1d173..=0x1d17a |
        0xe0000..=0xe0fff)
}

pub(crate) fn filler(cp: u32) -> bool {
    matches!(cp, 0x115f | 0x1160 | 0x3164 | 0xffa0)
}

impl GlyphInfo {
    pub(crate) fn di_residual(&self) -> bool {
        self.default_ignorable != 0 && self.default_ignorable & NORMAL == 0
    }

    pub(crate) fn di_normal(&mut self) {
        if self.default_ignorable != 0 {
            self.default_ignorable |= NORMAL;
        }
    }
}

pub(crate) fn fail(buffer: &mut hb_buffer_t, reason: ShapingRefusal) -> bool {
    buffer.refuse(reason);
    buffer.successful = false;
    false
}

fn reserve(buffer: &mut hb_buffer_t, targets: [usize; 6]) -> bool {
    let state = &buffer.default_ignorables;
    let capacities = [
        state.sets.capacity(),
        state.members.capacity(),
        state.candidates.capacity(),
        state.residual_inputs.capacity(),
        state.deleted_inputs.capacity(),
        state.residual_glyphs.capacity(),
    ];
    let lengths = [
        state.sets.len(),
        state.members.len(),
        state.candidates.len(),
        state.residual_inputs.len(),
        state.deleted_inputs.len(),
        state.residual_glyphs.len(),
    ];
    let sizes = [size_of::<Set>(), 4, 4, 4, 4, 4];
    let mut overlap = 0usize;
    let mut relocation = 0usize;
    for i in 0..6 {
        if targets[i] > capacities[i] {
            let Some(bytes) = targets[i]
                .checked_mul(sizes[i])
                .and_then(|n| overlap.checked_add(n))
            else {
                return fail(buffer, ShapingRefusal::GrowthLimit);
            };
            overlap = bytes;
            let Some(work) = relocation.checked_add(lengths[i]) else {
                return fail(buffer, ShapingRefusal::WorkLimit);
            };
            relocation = work;
        }
    }
    if !super::original_joining::room(buffer, buffer.len, overlap)
        || !super::original_joining::work(buffer, relocation)
    {
        return false;
    }
    let state = &mut buffer.default_ignorables;
    if targets[0] > capacities[0] {
        state.sets.reserve_exact(targets[0] - lengths[0]);
    }
    if targets[1] > capacities[1] {
        state.members.reserve_exact(targets[1] - lengths[1]);
    }
    if targets[2] > capacities[2] {
        state.candidates.reserve_exact(targets[2] - lengths[2]);
    }
    if targets[3] > capacities[3] {
        state.residual_inputs.reserve_exact(targets[3] - lengths[3]);
    }
    if targets[4] > capacities[4] {
        state.deleted_inputs.reserve_exact(targets[4] - lengths[4]);
    }
    if targets[5] > capacities[5] {
        state.residual_glyphs.reserve_exact(targets[5] - lengths[5]);
    }
    true
}

pub(crate) fn begin(buffer: &mut hb_buffer_t) -> bool {
    buffer.default_ignorables.active = true;
    if !super::original_joining::work(buffer, buffer.len) {
        return false;
    }
    let count = buffer.info[..buffer.len]
        .iter()
        .filter(|info| is_default_ignorable(info.glyph_id))
        .count();
    if buffer.len > u32::MAX as usize || count > ID as usize {
        return fail(buffer, ShapingRefusal::GrowthLimit);
    }
    if !reserve(buffer, [count, count, count, 0, 0, 0])
        || !super::original_joining::work(buffer, buffer.len.saturating_add(count))
    {
        return false;
    }
    for i in 0..buffer.len {
        buffer.info[i].default_ignorable = 0;
        if !is_default_ignorable(buffer.info[i].glyph_id) {
            continue;
        }
        let state = &mut buffer.default_ignorables;
        state.sets.push(Set {
            start: state.members.len(),
            len: 1,
        });
        state.members.push(i as u32);
        state.candidates.push(i as u32);
        buffer.info[i].default_ignorable = state.sets.len() as u32;
    }
    true
}

fn set(state: &State, carrier: u32) -> Set {
    if carrier & ID == 0 {
        Set { start: 0, len: 0 }
    } else {
        state.sets[(carrier & ID) as usize - 1]
    }
}

/// Exact immutable-set union. Kind is preserved until its semantic owner
/// explicitly establishes normal output; cluster custody never calls this.
pub(crate) fn union(buffer: &mut hb_buffer_t, a: u32, b: u32) -> Option<u32> {
    if a & ID == 0 {
        return Some(b);
    }
    if b & ID == 0 {
        return Some(a);
    }
    if a & ID == b & ID {
        return Some(a | b);
    }
    let sa = set(&buffer.default_ignorables, a);
    let sb = set(&buffer.default_ignorables, b);
    if !super::original_joining::work(buffer, sa.len.saturating_add(sb.len).saturating_mul(2)) {
        return None;
    }
    let Some(count) = sa.len.checked_add(sb.len) else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    };
    let state = &buffer.default_ignorables;
    let Some(records) = state
        .sets
        .len()
        .checked_add(1)
        .filter(|&n| n <= ID as usize)
    else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    };
    let Some(members) = state.members.len().checked_add(count) else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return None;
    };
    if !reserve(buffer, [records, members, 0, 0, 0, 0]) {
        return None;
    }
    let state = &mut buffer.default_ignorables;
    let start = state.members.len();
    let (mut i, mut j) = (0, 0);
    while i < sa.len || j < sb.len {
        let av = (i < sa.len).then(|| state.members[sa.start + i]);
        let bv = (j < sb.len).then(|| state.members[sb.start + j]);
        let v = match (av, bv) {
            (Some(av), Some(bv)) => av.min(bv),
            (Some(v), None) | (None, Some(v)) => v,
            (None, None) => break,
        };
        if av == Some(v) {
            i += 1;
        }
        if bv == Some(v) {
            j += 1;
        }
        state.members.push(v);
    }
    state.sets.push(Set {
        start,
        len: state.members.len() - start,
    });
    Some(state.sets.len() as u32 | ((a | b) & NORMAL))
}

/// Preflight metadata work and complete native output room before mutation.
pub(crate) fn output_room(
    buffer: &mut hb_buffer_t,
    num_in: usize,
    count: usize,
    work: usize,
) -> bool {
    if !buffer.default_ignorables.active {
        return true;
    }
    let Some(length) = buffer.out_len.checked_add(count) else {
        return fail(buffer, ShapingRefusal::GrowthLimit);
    };
    let Some(in_end) = buffer.idx.checked_add(num_in) else {
        return fail(buffer, ShapingRefusal::GrowthLimit);
    };
    let prefix = if !buffer.have_separate_output && length > in_end {
        buffer.out_len
    } else {
        0
    };
    let Some(work) = work.checked_add(prefix) else {
        return fail(buffer, ShapingRefusal::WorkLimit);
    };
    buffer.ensure(length.max(buffer.len)) && can_copy(buffer, work)
}

/// A sequence of outputs retains its source until the final skip. If outputs
/// fill an input/output gap first, switching copies the prefix at idx, rather
/// than the shorter prefix at the start of the semantic operation.
pub(crate) fn series_room(buffer: &mut hb_buffer_t, count: usize, work: usize) -> bool {
    if !buffer.default_ignorables.active {
        return true;
    }
    let Some(length) = buffer.out_len.checked_add(count) else {
        return fail(buffer, ShapingRefusal::GrowthLimit);
    };
    let prefix = if !buffer.have_separate_output && length > buffer.idx {
        buffer.idx
    } else {
        0
    };
    let Some(work) = work.checked_add(prefix) else {
        return fail(buffer, ShapingRefusal::WorkLimit);
    };
    buffer.ensure(length.max(buffer.len)) && can_copy(buffer, work)
}

pub(crate) fn copy_work(buffer: &mut hb_buffer_t, count: usize) -> bool {
    !buffer.default_ignorables.active || super::original_joining::work(buffer, count)
}

/// Check a whole copy/expansion before its first output. Individual writes
/// subsequently debit this same allowance rather than a separate DI budget.
pub(crate) fn can_copy(buffer: &mut hb_buffer_t, count: usize) -> bool {
    if !buffer.default_ignorables.active {
        return true;
    }
    if i32::try_from(count)
        .ok()
        .is_some_and(|count| buffer.max_ops >= count)
    {
        true
    } else {
        fail(buffer, ShapingRefusal::WorkLimit)
    }
}

pub(crate) fn finish(buffer: &mut hb_buffer_t) {
    if buffer.default_ignorables.finalized {
        return;
    }
    if !matches!(buffer.shaping_status(), ShapingStatus::Complete) {
        return;
    }
    if u32::try_from(buffer.len).is_err() {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return;
    }
    let count = buffer.default_ignorables.candidates.len();
    // Temporary classification is indexed only by candidates, never ordinary inputs.
    // Preflight its overlap with all retained sets and all final receipt vectors.
    let Some(work) = buffer
        .default_ignorables
        .members
        .len()
        .checked_add(buffer.len)
        .and_then(|n| n.checked_add(count.saturating_mul(3)))
    else {
        fail(buffer, ShapingRefusal::WorkLimit);
        return;
    };
    if !super::original_joining::work(buffer, work) {
        return;
    }
    let Some(temp) = count.checked_mul(size_of::<u8>()) else {
        fail(buffer, ShapingRefusal::GrowthLimit);
        return;
    };
    if !super::original_joining::room(buffer, buffer.len, temp) {
        return;
    }
    // Reserve final vectors while the temporary is still logically charged.
    let residual_count = buffer.info[..buffer.len]
        .iter()
        .filter(|i| i.di_residual() && !i.is_aat_deleted())
        .count();
    // Include temporary overlap in reserve's room calculation.
    buffer.default_ignorables_temporary = temp;
    let admitted = reserve(buffer, [0, 0, 0, count, count, residual_count]);
    buffer.default_ignorables_temporary = 0;
    if !admitted {
        return;
    }
    let mut classes = alloc::vec![0u8; count];
    // Work is charged per actual live association (sets can have many copies).
    for i in 0..buffer.len {
        let info = buffer.info[i];
        if info.default_ignorable == 0 || info.is_aat_deleted() {
            continue;
        }
        let members = set(&buffer.default_ignorables, info.default_ignorable);
        let searches = (usize::BITS - count.leading_zeros()) as usize;
        if !super::original_joining::work(
            buffer,
            members.len.saturating_mul(searches.saturating_add(1)),
        ) {
            return;
        }
        let state = &mut buffer.default_ignorables;
        let class = if info.di_residual() { 1 } else { 2 };
        for &candidate in &state.members[members.start..members.start + members.len] {
            let index = state
                .candidates
                .binary_search(&candidate)
                .expect("DI candidate identity");
            classes[index] = classes[index].max(class);
        }
    }
    let state = &mut buffer.default_ignorables;
    for (i, class) in classes.into_iter().enumerate() {
        match class {
            0 => state.deleted_inputs.push(state.candidates[i]),
            1 => state.residual_inputs.push(state.candidates[i]),
            _ => {}
        }
    }
    for (i, info) in buffer.info[..buffer.len].iter().enumerate() {
        if info.di_residual() && !info.is_aat_deleted() {
            let Ok(i) = u32::try_from(i) else {
                fail(buffer, ShapingRefusal::GrowthLimit);
                return;
            };
            buffer.default_ignorables.residual_glyphs.push(i);
        }
    }
    buffer.default_ignorables.finalized = true;
}

#[cfg(test)]
#[path = "default_ignorable_tests.rs"]
mod tests;
