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

#[test]
fn lines_of_maps_positions_to_the_lines_that_hold_them() {
    // Lines begin at positions 0, 3, 4 and 9.
    let axis = SlotAxis {
        breaks: vec![0, 3, 4, 9],
        lines: Vec::new(),
        entries: Vec::new(),
    };
    let span = |start, end| axis.lines_of(start, end, &mut free).unwrap();
    assert_eq!(span(0, 1), LineSpan { first: 0, last: 1 });
    assert_eq!(span(0, 3), LineSpan { first: 0, last: 1 });
    assert_eq!(span(0, 4), LineSpan { first: 0, last: 2 });
    assert_eq!(span(3, 10), LineSpan { first: 1, last: 4 });
    assert_eq!(span(5, 6), LineSpan { first: 2, last: 3 });
    assert_eq!(span(9, 10), LineSpan { first: 3, last: 4 });
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

#[test]
fn a_line_view_holds_its_entries_and_a_missing_line_has_none() {
    let mut rng = Rng(7);
    let entries = line(&mut rng, 5, 10, false);
    let axis = SlotAxis {
        breaks: vec![0, 4],
        lines: vec![
            SlotLine {
                first: 0,
                end: 2,
                regular: true,
                data: true,
            },
            SlotLine {
                first: 2,
                end: 5,
                regular: false,
                data: false,
            },
        ],
        entries: entries.clone(),
    };
    assert_eq!(axis.line(0).unwrap().entries, &entries[0..2]);
    assert_eq!(axis.line(1).unwrap().entries, &entries[2..5]);
    assert!(axis.line(2).is_none());
}
