//! What `CssSourceRetention::source` charges for a snapshot it has already
//! recorded, and for one that shares an original UTF-16 backing.
//!
//! A step is one `source`, `visit` or `charge_step` call. The independent
//! figure of every walk here is its number of calls: the charge of a `source`
//! call does not depend on the order of the walk, on how many snapshots the
//! collector holds, or on whether the snapshot is new, repeated or shares its
//! original backing with a recorded one.

use std::sync::Arc;
use surgeist_css::CssParser;
use surgeist_css::source::{CssSourceRetention, CssSourceSnapshot, CssUtf16InputLimits};

fn utf8(index: usize) -> CssSourceSnapshot {
    CssParser::default()
        .parse_sheet(format!(".s{index} {{ color: red }}"))
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}

/// A snapshot of `original`; two of them share one UTF-16 backing and are
/// distinct snapshots.
fn utf16(original: &Arc<Vec<u16>>) -> CssSourceSnapshot {
    CssParser::default()
        .parse_sheet_utf16(original.clone(), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap()
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .clone()
}

fn original(index: usize) -> Arc<Vec<u16>> {
    Arc::new(
        format!(".o{index} {{ margin: 1px }}")
            .encode_utf16()
            .collect(),
    )
}

/// Six UTF-8 snapshots, then three snapshots of each of three originals.
fn pool() -> Vec<CssSourceSnapshot> {
    let mut pool = (0..6).map(utf8).collect::<Vec<_>>();
    for index in 0..3 {
        let backing = original(index);
        pool.extend((0..3).map(|_| utf16(&backing)));
    }
    pool
}

/// How many times the snapshot at `index` is visited.
fn times(index: usize) -> usize {
    index % 3 + 1
}

/// The visits of every snapshot of the pool, `times` each, in the order of
/// `permutation`: repeats adjacent, or one round of the permutation per repeat.
fn visits(permutation: &[usize], adjacent: bool) -> Vec<usize> {
    if adjacent {
        permutation
            .iter()
            .flat_map(|index| std::iter::repeat_n(*index, times(*index)))
            .collect()
    } else {
        (0..3)
            .flat_map(|round| {
                permutation
                    .iter()
                    .filter(move |index| times(**index) > round)
            })
            .copied()
            .collect()
    }
}

#[test]
fn the_steps_of_a_walk_do_not_depend_on_the_order_of_its_snapshots() {
    let pool = pool();
    let count = pool.len();
    let ascending = (0..count).collect::<Vec<_>>();
    let descending = (0..count).rev().collect::<Vec<_>>();
    let scattered = (0..count)
        .map(|index| index * 7 % count)
        .collect::<Vec<_>>();
    let rotated = (0..count)
        .map(|index| (index + 5) % count)
        .collect::<Vec<_>>();
    let expected_calls = (0..count).map(times).sum::<usize>();
    let mut outcomes = Vec::new();
    for share in [false, true] {
        for permutation in [&ascending, &descending, &scattered, &rotated] {
            for adjacent in [false, true] {
                let mut sources = if share {
                    CssSourceRetention::with_shared_original_utf16(usize::MAX, usize::MAX, 8)
                } else {
                    CssSourceRetention::new(usize::MAX, usize::MAX, 8)
                };
                for index in visits(permutation, adjacent) {
                    sources.source(&pool[index]).unwrap();
                }
                outcomes.push((share, sources.steps(), sources.usage()));
            }
        }
    }
    for (share, steps, usage) in &outcomes {
        let reference = outcomes.iter().find(|(other, ..)| other == share).unwrap();
        assert_eq!(
            (*steps, *usage),
            (reference.1, reference.2),
            "sharing {share}: the order changed the figures"
        );
    }
    for (share, steps, _) in &outcomes {
        assert_eq!(*steps, expected_calls, "sharing {share}: one step per call");
    }
}

#[test]
fn the_steps_of_distinct_snapshots_grow_in_proportion_to_their_number() {
    for share in [false, true] {
        let mut figures = Vec::new();
        for count in [64, 128, 256] {
            // One original backing per four UTF-16 snapshots.
            let backings = (0..count / 4).map(original).collect::<Vec<_>>();
            let snapshots = (0..count)
                .map(|index| {
                    if index % 2 == 0 {
                        utf8(index)
                    } else {
                        utf16(&backings[index / 4])
                    }
                })
                .collect::<Vec<_>>();
            let mut sources = if share {
                CssSourceRetention::with_shared_original_utf16(usize::MAX, usize::MAX, 8)
            } else {
                CssSourceRetention::new(usize::MAX, usize::MAX, 8)
            };
            // Each snapshot is recorded, then recognized again.
            for _ in 0..2 {
                for snapshot in &snapshots {
                    sources.source(snapshot).unwrap();
                }
            }
            // Usage is the sum of the distinct snapshots' retained bytes, less the
            // backing of an original that an earlier snapshot already retains when
            // the collector shares originals.
            let mut backed = Vec::<&Arc<Vec<u16>>>::new();
            let mut expected = 0;
            for snapshot in &snapshots {
                expected += snapshot.storage_usage().retained_bytes();
                if let Some(original) = snapshot.original_utf16() {
                    if share && backed.iter().any(|seen| Arc::ptr_eq(seen, original)) {
                        expected -= original.capacity() * std::mem::size_of::<u16>();
                    }
                    backed.push(original);
                }
            }
            assert_eq!(
                sources.usage().retained_bytes(),
                expected,
                "sharing {share}, {count} snapshots: usage"
            );
            figures.push((count, sources.steps()));
        }
        for (count, steps) in figures {
            assert_eq!(steps, 2 * count, "sharing {share}, {count} snapshots");
        }
    }
}

/// A small fixed-seed generator (splitmix64), so every run draws the same walks.
struct Walk(u64);
impl Walk {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(bound).unwrap()).unwrap()
    }
}

/// Sixty UTF-8 snapshots, then six snapshots of each of forty original backings.
fn large_pool() -> Vec<CssSourceSnapshot> {
    let mut pool = (0..60).map(utf8).collect::<Vec<_>>();
    for index in 0..40 {
        let backing = original(index);
        pool.extend((0..6).map(|_| utf16(&backing)));
    }
    pool
}

/// The payload of one snapshot alone, with its original backing counted.
fn alone(snapshot: &CssSourceSnapshot) -> usize {
    let mut sources = CssSourceRetention::new(usize::MAX, usize::MAX, 8);
    sources.source(snapshot).unwrap();
    sources
        .complete_payload_bytes_with(|| Ok::<_, ()>(()))
        .unwrap()
}

/// The collector's contract as a list scanned with pointer equality: the
/// snapshots recorded in order, the steps (one for each call), the retained
/// bytes, the payload and the admissions the payload observation makes.
struct Model<'a> {
    pool: &'a [CssSourceSnapshot],
    alone: &'a [usize],
    share: bool,
    bank: Vec<usize>,
    steps: usize,
    usage: usize,
    payload: usize,
    actions: usize,
}
impl Model<'_> {
    fn source(&mut self, index: usize) {
        self.steps += 1;
        let snapshot = &self.pool[index];
        if self
            .bank
            .iter()
            .any(|recorded| self.pool[*recorded].same_snapshot(snapshot))
        {
            return;
        }
        let shared = snapshot.original_utf16().filter(|original| {
            self.bank.iter().any(|recorded| {
                self.pool[*recorded]
                    .original_utf16()
                    .is_some_and(|other| Arc::ptr_eq(other, original))
            })
        });
        let mut bytes = snapshot.storage_usage().retained_bytes();
        let mut payload = self.alone[index];
        if let Some(original) = shared {
            if self.share {
                bytes -= original.capacity() * std::mem::size_of::<u16>();
            }
            payload -=
                std::mem::size_of::<Vec<u16>>() + original.capacity() * std::mem::size_of::<u16>();
        }
        self.usage += bytes;
        self.payload += payload;
        // The observation admits each snapshot, its original lookup if it has
        // an original backing, and its payload.
        self.actions += 2 + usize::from(snapshot.original_utf16().is_some());
        self.bank.push(index);
    }
}

#[test]
fn walks_that_grow_the_bank_agree_with_a_scanning_model_after_every_call() {
    let pool = large_pool();
    let alone = pool.iter().map(alone).collect::<Vec<_>>();
    let (mut walks, mut calls, mut largest) = (0, 0, 0);
    for share in [false, true] {
        // Each target is a number of distinct snapshots to reach; the bank
        // doubles its room from one snapshot to the first power of two above it.
        for (seed, target) in [(1, 3), (2, 17), (3, 70), (4, 150), (5, 300), (6, 300)] {
            let mut walk = Walk(seed);
            // The snapshots in the order a walk first meets them.
            let mut order = (0..pool.len()).collect::<Vec<_>>();
            for index in (1..order.len()).rev() {
                order.swap(index, walk.below(index + 1));
            }
            let mut sources = if share {
                CssSourceRetention::with_shared_original_utf16(usize::MAX, usize::MAX, 8)
            } else {
                CssSourceRetention::new(usize::MAX, usize::MAX, 8)
            };
            let mut model = Model {
                pool: &pool,
                alone: &alone,
                share,
                bank: Vec::new(),
                steps: 0,
                usage: 0,
                payload: 0,
                actions: 0,
            };
            let total = 3 * target;
            for call in 0..total {
                let new = model.bank.len() < target
                    && (model.bank.is_empty()
                        || total - call <= target - model.bank.len()
                        || walk.below(3) == 0);
                let index = if new {
                    order[model.bank.len()]
                } else {
                    model.bank[walk.below(model.bank.len())]
                };
                sources.source(&pool[index]).unwrap();
                model.source(index);
                let context = format!("sharing {share}, seed {seed}, call {call}");
                assert_eq!(sources.steps(), model.steps, "{context}: steps");
                assert_eq!(
                    sources.usage().retained_bytes(),
                    model.usage,
                    "{context}: usage"
                );
                let mut actions = 0;
                let payload = sources.complete_payload_bytes_with(|| {
                    actions += 1;
                    Ok::<_, ()>(())
                });
                assert_eq!(payload, Ok(model.payload), "{context}: payload");
                assert_eq!(actions, 1 + model.actions, "{context}: payload admissions");
            }
            assert_eq!(model.bank.len(), target);
            walks += 1;
            calls += total;
            largest = largest.max(target);
        }
    }
    // Walks of 12 and up to 300 distinct snapshots: the bank grows through
    // rooms of 1, 2, 4, ..., 512, rebuilding both indexes at each growth.
    assert_eq!(
        (walks, calls, largest),
        (12, 3 * 2 * (3 + 17 + 70 + 150 + 300 + 300), 300)
    );
}
