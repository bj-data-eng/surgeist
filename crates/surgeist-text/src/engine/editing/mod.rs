//! Original-source interaction over the same immutable committed line rows.

mod cursor;
mod selection;

use super::{PlacedLine, PreparedData};
use crate::{Affinity, Error, ErrorCode, Range, Result};
pub(crate) use cursor::{cursor, hit, move_cursor};
pub(crate) use selection::selection;

pub(crate) struct InteractionData {
    stops: Vec<usize>,
}

impl InteractionData {
    pub(crate) fn caret(&self, index: usize, affinity: Affinity) -> usize {
        caret_at(&self.stops, index, affinity)
    }
    pub(crate) fn is_stop(&self, index: usize) -> bool {
        self.stops.binary_search(&index).is_ok()
    }
    #[cfg(feature = "text-accessibility")]
    pub(crate) fn selectable_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.stops
            .windows(2)
            .map(|pair| Range::new(pair[0], pair[1]))
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
    let stops = legal_stops(&prepared.analyzed.projection.graphemes, &intervals);
    if stops.first() != Some(&0) || stops.last() != Some(&prepared.analyzed.source.text().len()) {
        return Err(Error::new(
            ErrorCode::LayoutFailed,
            "committed source caret boundary coverage is incomplete",
        ));
    }
    Ok(InteractionData { stops })
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
