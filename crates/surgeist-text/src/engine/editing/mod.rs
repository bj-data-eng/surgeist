//! Original-source interaction over the same immutable committed line rows.

mod cursor;
pub(crate) mod position;
mod selection;

use super::{PlacedLine, PreparedData};
use crate::{Affinity, Range, Result};
pub(crate) use cursor::{cursor, hit, move_cursor};
pub(crate) use selection::selection;

/// Original-side ownership of a nonempty source unit. At a shared endpoint,
/// Before borrows the upstream unit and After the downstream unit. Actual
/// deleted-anchor geometry uses this occurrence eligibility relation.
pub(super) fn owns_source_side(source: Range, value: crate::Cursor) -> bool {
    match value.affinity() {
        Affinity::Before => source.start < value.index() && value.index() <= source.end,
        Affinity::After => source.start <= value.index() && value.index() < source.end,
    }
}

pub(crate) struct InteractionData {
    stops: Vec<usize>,
    pub(crate) positions: Vec<position::Position>,
}

impl InteractionData {
    /// SourceCursor remains in the original source domain. Executed deletion
    /// correspondence is intentionally not a lossy round trip for these stops.
    pub(crate) fn caret(&self, index: usize, affinity: Affinity) -> usize {
        caret_at(&self.stops, index, affinity)
    }
    pub(crate) fn is_stop(&self, index: usize) -> bool {
        self.stops.binary_search(&index).is_ok()
    }
    pub(crate) fn selection_range(&self, range: Range) -> Range {
        Range::new(
            self.caret(range.start, Affinity::Before),
            self.caret(range.end, Affinity::After),
        )
    }
}

/// Shared affinity policy, also used by font-independent analyzed-source AX.
/// Tables are actual ordered source boundaries, including zero and source end.
pub(crate) fn caret_at(stops: &[usize], index: usize, affinity: Affinity) -> usize {
    match stops.binary_search(&index) {
        Ok(position) => stops[position],
        Err(position) => match affinity {
            Affinity::Before => stops[position.saturating_sub(1)],
            Affinity::After => stops[position.min(stops.len() - 1)],
        },
    }
}

pub(crate) fn prepare(prepared: &PreparedData, lines: &[PlacedLine]) -> Result<InteractionData> {
    let mut intervals: Vec<_> = lines
        .iter()
        .flat_map(|line| {
            line.finalized
                .data
                .glyphs
                .iter()
                .map(|glyph| glyph.glyph.source)
        })
        .filter(|range| !range.is_empty())
        .collect();
    intervals.sort_unstable_by_key(|range| (range.start, range.end));
    // Original editing still snaps to genuine host EGC edges. A placed output
    // range can cover only one contribution of that EGC, so its source hull
    // need not itself be an original caret boundary. Exact output positions
    // are admitted independently by the selected presentation owner.
    let stops = legal_stops(&prepared.analyzed.host_projection.graphemes, &intervals);
    let positions = position::prepare(lines);
    Ok(InteractionData { stops, positions })
}

fn legal_stops(graphemes: &[usize], intervals: &[Range]) -> Vec<usize> {
    let mut interval = 0;
    let mut covered_end = 0;
    graphemes
        .iter()
        .copied()
        .filter(|&unit| {
            while let Some(range) = intervals.get(interval) {
                if range.start >= unit {
                    break;
                }
                covered_end = covered_end.max(range.end);
                interval += 1;
            }
            covered_end <= unit
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_overlapping_glyph_extents_remove_only_internal_grapheme_stops() {
        let intervals = [Range::new(1, 4), Range::new(2, 6), Range::new(5, 7)];
        let graphemes: Vec<_> = (0..=8).collect();
        assert_eq!(legal_stops(&graphemes, &intervals), [0, 1, 7, 8]);
        assert_eq!(legal_stops(&graphemes, &[]), graphemes);
        assert_eq!(caret_at(&[0, 1, 7, 8], 4, Affinity::Before), 1);
        assert_eq!(caret_at(&[0, 1, 7, 8], 4, Affinity::After), 7);
    }
}
