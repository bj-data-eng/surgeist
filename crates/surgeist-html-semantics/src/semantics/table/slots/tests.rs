//! The pure line helpers against naive scans, with seeded random lines.

use super::*;
use std::convert::Infallible;

struct Rng(u64);
impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        usize::try_from((self.0 >> 33) % u64::try_from(n).unwrap()).unwrap()
    }
}

/// Entries sorted by start, whose intervals may overlap, with running reach
/// and nearest header filled in as a finished line has them.
fn line(rng: &mut Rng, count: usize, width: usize, overlap: bool) -> Vec<SlotEntry> {
    let mut intervals = Vec::new();
    let mut at = 0;
    for cell in 0..count {
        let start = if overlap {
            rng.below(width)
        } else {
            at + rng.below(3)
        };
        let end = (start + 1 + rng.below(4)).min(width.max(start + 1));
        at = end;
        intervals.push((start, end, cell));
    }
    intervals.sort();
    let mut entries = Vec::new();
    let (mut reach, mut header) = (0, None);
    for (position, (start, end, cell)) in intervals.into_iter().enumerate() {
        reach = reach.max(end);
        if rng.below(2) == 0 {
            header = Some(position);
        }
        entries.push(SlotEntry {
            start,
            end,
            cell,
            reach,
            header,
            key: None,
            differs: None,
        });
    }
    entries
}

fn free(_: usize) -> Result<(), Infallible> {
    Ok(())
}

#[test]
fn leading_by_finds_every_prefix_length_in_logarithmic_probes() {
    for len in 0..40 {
        for prefix in 0..=len {
            let mut probes = 0;
            let found = leading_by(len, |i| i < prefix, &mut |n| {
                probes += n;
                Ok::<(), Infallible>(())
            })
            .unwrap();
            assert_eq!(found, prefix, "len={len}");
            assert!(probes <= usize::try_from(usize::BITS - len.leading_zeros()).unwrap());
        }
    }
}

#[test]
fn a_refused_charge_stops_a_search_before_it_probes() {
    // Allow three units: the search may test its predicate only after paying.
    let mut paid = 0;
    let tested = std::cell::Cell::new(0);
    let result = leading_by(
        1000,
        |_| {
            tested.set(tested.get() + 1);
            true
        },
        &mut |n| {
            if paid + n > 3 {
                return Err(());
            }
            paid += n;
            Ok(())
        },
    );
    assert_eq!(result, Err(()));
    assert_eq!((paid, tested.get()), (3, 3));
}

#[test]
fn cover_counts_the_cells_over_a_slot_even_where_cells_overlap() {
    let mut rng = Rng(0x1733);
    for round in 0..300 {
        let width = 12;
        let entries = line(&mut rng, 1 + round % 7, width, round % 2 == 0);
        for slot in 0..width + 2 {
            let covering: Vec<usize> = entries
                .iter()
                .filter(|e| e.start <= slot && slot < e.end)
                .map(|e| e.cell)
                .collect();
            let expected = match covering.as_slice() {
                [] => Cover::Empty,
                [one] => Cover::One(*one),
                _ => Cover::Many,
            };
            assert_eq!(cover(&entries, slot, &mut free).unwrap(), expected);
        }
    }
}

#[test]
fn leading_over_entries_counts_the_entries_before_a_slot() {
    let mut rng = Rng(0x1734);
    let entries = line(&mut rng, 9, 20, true);
    for slot in 0..22 {
        let expected = entries.iter().filter(|e| e.start < slot).count();
        assert_eq!(
            leading(&entries, |e| e.start < slot, &mut free).unwrap(),
            expected
        );
    }
}

fn empty_axis() -> SlotAxis {
    SlotAxis::default()
}

#[test]
fn runs_of_maps_lines_to_the_runs_that_hold_them() {
    // Runs begin at lines 0, 3, 4 and 9 of 12.
    let axis = SlotAxis {
        lines: 12,
        breaks: vec![0, 3, 4, 9],
        ..empty_axis()
    };
    let span = |start, length| axis.runs_of(Extent { start, length }, &mut free).unwrap();
    assert_eq!(span(0, 1), LineSpan { first: 0, last: 1 });
    assert_eq!(span(0, 3), LineSpan { first: 0, last: 1 });
    assert_eq!(span(0, 4), LineSpan { first: 0, last: 2 });
    assert_eq!(span(3, 7), LineSpan { first: 1, last: 4 });
    assert_eq!(span(5, 1), LineSpan { first: 2, last: 3 });
    assert_eq!(span(9, 1), LineSpan { first: 3, last: 4 });
    // A run covers the lines up to the next run, the last up to the end.
    let covered = |run| axis.run_lines(run);
    assert_eq!(
        covered(0),
        Extent {
            start: 0,
            length: 3
        }
    );
    assert_eq!(
        covered(2),
        Extent {
            start: 4,
            length: 5
        }
    );
    assert_eq!(
        covered(3),
        Extent {
            start: 9,
            length: 3
        }
    );
}

#[test]
fn containing_finds_the_one_sorted_disjoint_group_holding_a_slot() {
    // Groups of (start, length): [2, 5), [5, 6) and [8, 10).
    let groups = [(2, 3), (5, 1), (8, 2)];
    for slot in 0..12 {
        let expected = groups
            .iter()
            .position(|(start, length)| *start <= slot && slot < start + length);
        assert_eq!(
            containing(
                &groups,
                slot,
                |(start, length)| Extent {
                    start: *start,
                    length: *length,
                },
                &mut free
            )
            .unwrap(),
            expected,
            "slot {slot}"
        );
    }
}

fn short_entry(start: usize, cell: usize) -> SlotEntry {
    SlotEntry {
        start,
        end: start + 1,
        cell,
        reach: start + 1,
        header: None,
        key: None,
        differs: None,
    }
}

/// Short lines 2, 5, 6 and 11 whose first entries start at 4, 0, 3 and 9.
fn short_axis() -> SlotAxis {
    let starts = [4, 0, 3, 9];
    let lines = [2, 5, 6, 11];
    let short: Vec<SlotEntry> = starts
        .iter()
        .enumerate()
        .map(|(cell, start)| short_entry(*start, cell))
        .collect();
    let short_lines: Vec<ShortLine> = lines
        .iter()
        .enumerate()
        .map(|(position, line)| ShortLine {
            line: *line,
            first: position,
            end: position + 1,
        })
        .collect();
    let mut least = vec![usize::MAX; 8];
    for (position, start) in starts.iter().enumerate() {
        least[4 + position] = *start;
    }
    for node in (1..4).rev() {
        least[node] = least[2 * node].min(least[2 * node + 1]);
    }
    SlotAxis {
        lines: 12,
        short_lines,
        short,
        short_least: LeastTree { nodes: least },
        ..empty_axis()
    }
}

#[test]
fn short_lists_are_found_by_line_and_absent_lines_have_none() {
    let axis = short_axis();
    for line in 0..13 {
        let list = axis.short_of(line, &mut free).unwrap();
        let expected = [2, 5, 6, 11].iter().position(|l| *l == line);
        match expected {
            Some(position) => {
                assert_eq!(
                    list.entries,
                    &axis.short[position..=position],
                    "line {line}"
                );
                assert_eq!(list.first, position);
            }
            None => assert!(list.entries.is_empty(), "line {line}"),
        }
    }
}

#[test]
fn visiting_left_finds_the_short_lines_with_a_cell_before_the_bound() {
    let axis = short_axis();
    for first in 0..=4 {
        for last in first..=4 {
            for bound in 0..11 {
                let mut visited = Vec::new();
                axis.visit_left(
                    LineSpan { first, last },
                    bound,
                    &mut free,
                    &mut |position| {
                        visited.push(position);
                        Ok::<(), Infallible>(())
                    },
                )
                .unwrap();
                let starts = [4, 0, 3, 9];
                let expected: Vec<usize> = (first..last).filter(|p| starts[*p] < bound).collect();
                assert_eq!(visited, expected, "{first}..{last} before {bound}");
            }
        }
    }
}

#[test]
fn short_positions_bound_the_short_lines_inside_a_range_of_lines() {
    let axis = short_axis();
    let positions = |start, length| {
        axis.short_positions(Extent { start, length }, &mut free)
            .unwrap()
    };
    assert_eq!(positions(0, 12), LineSpan { first: 0, last: 4 });
    assert_eq!(positions(3, 3), LineSpan { first: 1, last: 2 });
    assert_eq!(positions(5, 2), LineSpan { first: 1, last: 3 });
    assert_eq!(positions(7, 4), LineSpan { first: 3, last: 3 });
}

#[test]
fn irregular_lines_are_the_merged_extents() {
    let axis = SlotAxis {
        irregular: vec![
            Extent {
                start: 2,
                length: 3,
            },
            Extent {
                start: 8,
                length: 1,
            },
        ],
        ..empty_axis()
    };
    for line in 0..12 {
        let expected = (2..5).contains(&line) || line == 8;
        assert_eq!(axis.irregular_at(line, &mut free).unwrap(), expected);
    }
    assert!(!empty_axis().irregular_at(3, &mut free).unwrap());
}

#[test]
fn data_spans_tell_whether_a_data_cell_crosses_any_line_of_a_range() {
    // Data cells cross lines [0, 2), [1, 6), [4, 5) and [9, 10).
    let spans = [(0, 2), (1, 6), (4, 5), (9, 10)];
    let mut reach = 0;
    let data = DataSpans {
        starts: spans.iter().map(|(s, _)| *s).collect(),
        reach: spans
            .iter()
            .map(|(_, e)| {
                reach = reach.max(*e);
                reach
            })
            .collect(),
    };
    for start in 0..12 {
        for length in 1..4 {
            let expected = spans.iter().any(|(s, e)| *s < start + length && *e > start);
            assert_eq!(
                data.crosses(Extent { start, length }, &mut free).unwrap(),
                expected,
                "lines {start}..{}",
                start + length
            );
        }
    }
}

#[test]
fn a_list_finds_the_nearest_header_at_or_before_a_position() {
    let entries: Vec<SlotEntry> = (0..4)
        .map(|position| SlotEntry {
            start: position,
            end: position + 1,
            cell: position,
            reach: position + 1,
            // Table indexes: a header at table index 11 (list position 1).
            header: [None, Some(11), Some(11), Some(13)][position],
            key: None,
            differs: None,
        })
        .collect();
    let list = List {
        entries: &entries,
        first: 10,
    };
    assert_eq!(list.header_at(0), None);
    assert_eq!(list.header_at(1), Some(1));
    assert_eq!(list.header_at(2), Some(1));
    assert_eq!(list.header_at(3), Some(3));
    assert_eq!(list.header_at(9), None);
}

#[test]
fn headers_crossing_other_lines_are_found_past_those_crossing_the_same_ones() {
    let (a, b) = (0usize, 1usize);
    // Headers at positions 0 and 2 cross line a, 3 crosses b, 4 crosses a;
    // positions 1 and 5 are data cells.
    let spec = [
        (a, None, Some(0)),
        (a, None, Some(0)),
        (a, None, Some(2)),
        (b, Some(2), Some(3)),
        (a, Some(3), Some(4)),
        (a, None, Some(4)),
    ];
    let entries: Vec<SlotEntry> = spec
        .iter()
        .enumerate()
        .map(|(position, (key, differs, header))| SlotEntry {
            start: position,
            end: position + 1,
            cell: position,
            reach: position + 1,
            header: *header,
            key: Some(*key),
            differs: *differs,
        })
        .collect();
    let list = List {
        entries: &entries,
        first: 0,
    };
    let cell = |position, key| list.header_other_than(Some(position), key).map(|e| e.cell);
    assert_eq!(cell(5, a), Some(3));
    assert_eq!(cell(5, b), Some(4));
    assert_eq!(cell(2, a), None);
    assert_eq!(cell(3, a), Some(3));
    assert_eq!(list.header_other_than(None, a).map(|e| e.cell), None);
}

#[test]
fn the_first_position_below_a_bound_is_found_within_a_range() {
    let starts = [7, 3, 9, 0, 5, 8, 2, 6];
    let mut nodes = vec![usize::MAX; 16];
    nodes[8..].copy_from_slice(&starts);
    for node in (1..8).rev() {
        nodes[node] = nodes[2 * node].min(nodes[2 * node + 1]);
    }
    let tree = LeastTree { nodes };
    for first in 0..=8 {
        for last in first..=8 {
            for bound in 0..11 {
                let expected = (first..last).find(|p| starts[*p] < bound);
                let found = tree
                    .first_below(LineSpan { first, last }, bound, &mut free)
                    .unwrap();
                assert_eq!(found, expected, "{first}..{last} below {bound}");
            }
        }
    }
    // The empty tree has no positions.
    let none = LeastTree::default()
        .first_below(LineSpan { first: 0, last: 1 }, 5, &mut free)
        .unwrap();
    assert_eq!(none, None);
}
