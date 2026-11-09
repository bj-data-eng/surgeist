use super::*;
use crate::{
    Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionOutcome},
    gc::heap::Trace as HeapTrace,
    js_string,
};
use std::{cell::Cell, fmt::Debug};

// This lawful scalar key deliberately makes every hash collide. Its scalar
// hash/equality add no delegated String or Name credits; the index itself and
// its controller are unchanged. The observer records actual hashing, including
// any accidental hashing of existing keys during table growth.
#[derive(Clone, Debug, HeapTrace)]
struct ScalarKey {
    #[heap_trace(skip)]
    id: u8,
    #[heap_trace(skip)]
    hashes: Rc<Cell<usize>>,
}

impl PartialEq for ScalarKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ScalarKey {}
impl Hash for ScalarKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashes.set(self.hashes.get() + 1);
        state.write_u8(0);
    }
}
impl Key for ScalarKey {
    fn hash_admitted(&self, control: &mut Control<'_>) -> JsResult<u64> {
        control.admit(Work::Checkpoint)?;
        let mut hash = rustc_hash::FxHasher::default();
        self.hash(&mut hash);
        Ok(hash.finish())
    }

    fn eq_admitted(&self, other: &Self, control: &mut Control<'_>) -> JsResult<bool> {
        control.admit(Work::Checkpoint)?;
        Ok(self == other)
    }
}

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

fn scoped<R>(
    context: &mut Context,
    limit: u64,
    action: impl FnOnce(&mut Control<'_>) -> JsResult<R>,
) -> (ExecutionOutcome<R>, Option<JsError>) {
    let mut direct = None;
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0).with_module_metadata_work_limit(limit),
            |access| {
                access.into_owner().with_module_metadata(|context| {
                    let result = action(&mut Control::new(context));
                    direct = result.as_ref().err().cloned();
                    result
                })
            },
        )
        .unwrap();
    (outcome, direct)
}

fn assert_refused<R: Debug>(
    outcome: ExecutionOutcome<R>,
    direct: Option<JsError>,
    limit: u64,
    accepted: u64,
) {
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(outcome.report.module_metadata_work().unwrap(), accepted);
    let error = outcome.result.unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::ModuleMetadataWork { limit }
        ))
    );
    assert_eq!(direct.as_ref(), Some(&error));
}

fn key(id: u8, hashes: &Rc<Cell<usize>>) -> ScalarKey {
    ScalarKey {
        id,
        hashes: hashes.clone(),
    }
}

fn sizes() -> (u64, u64) {
    (
        u64::try_from(size_of::<Option<usize>>()).unwrap(),
        u64::try_from(size_of::<Entry<ScalarKey, u8>>()).unwrap(),
    )
}

fn prepared(count: u8, context: &mut Context, hashes: &Rc<Cell<usize>>) -> Index<ScalarKey, u8> {
    let mut index = Index::default();
    scoped(context, u64::MAX, |control| {
        for id in 0..count {
            index.insert(key(id, hashes), id + 10, control)?;
        }
        Ok(())
    })
    .0
    .result
    .unwrap();
    hashes.set(0);
    index
}

fn assert_prefix(index: &Index<ScalarKey, u8>, count: u8, context: &mut Context) {
    assert_eq!(
        index.iter().map(|key| key.id).collect::<Vec<_>>(),
        (0..count).collect::<Vec<_>>()
    );
    let query_hashes = Rc::new(Cell::new(0));
    scoped(context, u64::MAX, |control| {
        for id in 0..count {
            assert_eq!(
                index.get(&key(id, &query_hashes), control)?.copied(),
                Some(id + 10)
            );
        }
        assert_eq!(
            index.get(&key(count, &query_hashes), control)?.copied(),
            None
        );
        Ok(())
    })
    .0
    .result
    .unwrap();
}

#[test]
fn insertion_record_refusal_precedes_hashing_and_duplicate_probes_preserve_the_first_value() {
    for limit in 0..=3 {
        let mut context = context();
        let hashes = Rc::new(Cell::new(0));
        let mut index = prepared(1, &mut context, &hashes);
        // Duplicate: attempt Record, occupied-slot Probe, cached-hash Probe.
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            index.insert(key(0, &hashes), 99, control)
        });
        assert_eq!(hashes.get(), if limit == 0 { 0 } else { 1 });
        if limit == 3 {
            assert!(outcome.result.is_ok());
            assert!(direct.is_none());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), 3);
        } else {
            assert_refused(outcome, direct, limit, limit);
        }
        assert_prefix(&index, 1, &mut context);
    }
}

#[test]
fn first_slot_capacity_vacant_probe_and_entry_write_have_independent_refusal_boundaries() {
    let (slot, entry) = sizes();
    // First insertion: R + Capacity(8S) + 8R + Capacity(4E) + P + 2W.
    let exact = 8 * slot + 4 * entry + 12;
    for (limit, accepted) in [
        (8 * slot, 1),
        (exact - 3, exact - 3),
        (exact - 1, exact - 1),
        (exact, exact),
    ] {
        let mut context = context();
        let hashes = Rc::new(Cell::new(0));
        let mut index = Index::default();
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            index.insert(key(0, &hashes), 10, control)
        });
        assert_eq!(hashes.get(), 1);
        if limit == exact {
            assert!(outcome.result.is_ok());
            assert!(direct.is_none());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), exact);
            assert_prefix(&index, 1, &mut context);
        } else {
            assert_refused(outcome, direct, limit, accepted);
            assert_prefix(&index, 0, &mut context);
        }
    }
}

#[test]
fn staged_growth_refusal_preserves_the_accepted_prefix_and_rehashes_only_cached_hashes() {
    let (slot, entry) = sizes();
    // Fifth collision: R + 9P + Capacity(16S) + 16R + 10P + 4W
    // + Capacity(4E) + Relocate(4E) + 9P + 2W = 16S + 8E + 51.
    let exact = 16 * slot + 8 * entry + 51;
    let before_relocate = 16 * slot + 4 * entry + 40;
    for (limit, accepted) in [
        (16 * slot + 39, 16 * slot + 39),
        (before_relocate + 4 * entry - 1, before_relocate),
        (exact - 1, exact - 1),
        (exact, exact),
    ] {
        let mut context = context();
        let hashes = Rc::new(Cell::new(0));
        let mut index = prepared(4, &mut context, &hashes);
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            index.insert(key(4, &hashes), 14, control)
        });
        assert_eq!(
            hashes.get(),
            1,
            "growth must reuse the four admitted hashes"
        );
        if limit == exact {
            assert!(outcome.result.is_ok());
            assert!(direct.is_none());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), exact);
            assert_prefix(&index, 5, &mut context);
        } else {
            assert_refused(outcome, direct, limit, accepted);
            assert_prefix(&index, 4, &mut context);
        }
    }
}

#[test]
fn five_colliding_insertions_have_a_derived_exact_limit_and_a_last_write_refusal() {
    let (slot, entry) = sizes();
    // Entry capacity targets4 then8: deltas4E+4E, relocation0E+4E,
    // for12E. Insertions2..4 use retained capacity. Fresh tables request8S+16S.
    // Unit events: first12, then9+13+17, then51: total102.
    let exact = 24 * slot + 12 * entry + 102;
    for limit in [exact - 1, exact] {
        let mut context = context();
        let hashes = Rc::new(Cell::new(0));
        let mut index = Index::default();
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            for id in 0..5 {
                index.insert(key(id, &hashes), id + 10, control)?;
            }
            Ok(())
        });
        assert_eq!(hashes.get(), 5);
        if limit == exact {
            assert!(outcome.result.is_ok());
            assert!(direct.is_none());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), exact);
            assert_prefix(&index, 5, &mut context);
        } else {
            assert_refused(outcome, direct, limit, exact - 1);
            assert_prefix(&index, 4, &mut context);
        }
    }
}

#[test]
fn append_reservation_credits_do_not_depend_on_allocator_spare_capacity() {
    let unit = u64::try_from(size_of::<u16>()).unwrap();
    // Four initialized values fill logical capacity4. Appending the fifth
    // targets8: Capacity(4U), Relocate(4U), Record1, Write1.
    let exact = 8 * unit + 2;
    for spare in [false, true] {
        for (limit, accepted) in [
            (8 * unit - 1, 4 * unit),
            (exact - 1, exact - 1),
            (exact, exact),
        ] {
            let mut context = context();
            let mut values = Buffer::default();
            scoped(&mut context, u64::MAX, |control| {
                for value in 1_u16..=4 {
                    control.push(&mut values, value)?;
                }
                Ok(())
            })
            .0
            .result
            .unwrap();
            if spare {
                // Fixture-only physical over-allocation leaves the admitted
                // logical capacity unchanged; no production operation is replaced.
                values.values.try_reserve_exact(64).unwrap();
            }
            let (outcome, direct) = scoped(&mut context, limit, |control| {
                control.push(&mut values, 5_u16)
            });
            if limit == exact {
                assert!(outcome.result.is_ok());
                assert!(direct.is_none());
                assert_eq!(outcome.report.module_metadata_work().unwrap(), exact);
                assert_eq!(&*values, &[1, 2, 3, 4, 5]);
            } else {
                assert_refused(outcome, direct, limit, accepted);
                assert_eq!(&*values, &[1, 2, 3, 4]);
            }
        }
    }
}

fn attributes(order: &str) -> Vec<ImportAttribute> {
    order
        .chars()
        .map(|key| match key {
            'a' => ImportAttribute::new(js_string!("a"), js_string!("A")),
            'b' => ImportAttribute::new(js_string!("b"), js_string!("B")),
            'c' => ImportAttribute::new(js_string!("c"), js_string!("C")),
            _ => panic!("the fixture has only three declared keys"),
        })
        .collect()
}

fn snapshot(values: &[ImportAttribute]) -> Vec<(JsString, JsString)> {
    values
        .iter()
        .map(|value| (value.key.clone(), value.value.clone()))
        .collect()
}

#[test]
fn sort_refuses_before_comparison_scans_and_swaps_without_losing_attribute_values() {
    // On [a,b,c], each one-unit comparison costs P+Scan(1).
    // Heap build: compare(b,c), compare(a,c), swap(a,c): 2+2+1.
    // Extract c: swap, compare(a,b), swap(a,b): 1+2+1.
    // Extract b: swap: 1. Total10, with states abc -> cba -> abc -> bac -> abc.
    for (limit, expected) in [
        (0, "abc"),
        (1, "abc"),
        (4, "abc"),
        (5, "cba"),
        (9, "bac"),
        (10, "abc"),
    ] {
        let mut context = context();
        let mut values = attributes("abc");
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            sort_attributes(&mut values, control)
        });
        assert_eq!(snapshot(&values), snapshot(&attributes(expected)));
        if limit == 10 {
            assert!(outcome.result.is_ok());
            assert!(direct.is_none());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), 10);
        } else {
            assert_refused(outcome, direct, limit, limit);
        }
    }
    // [b,a] is already a max-heap: one P+Scan(1), no sift swap, one extract W.
    for limit in [2, 3] {
        let mut context = context();
        let mut values = attributes("ba");
        let (outcome, direct) = scoped(&mut context, limit, |control| {
            sort_attributes(&mut values, control)
        });
        if limit == 3 {
            assert!(outcome.result.is_ok());
            assert_eq!(outcome.report.module_metadata_work().unwrap(), 3);
            assert_eq!(snapshot(&values), snapshot(&attributes("ab")));
        } else {
            assert_refused(outcome, direct, limit, 2);
            assert_eq!(snapshot(&values), snapshot(&attributes("ba")));
        }
    }
}

#[test]
fn sort_prices_utf16_comparison_and_preserves_code_unit_order_including_surrogates() {
    for (low, high) in [
        (
            JsString::from(&[0xD800_u16, 0xDC00][..]),
            JsString::from(&[0xE000_u16][..]),
        ),
        (
            JsString::from(&[0xD800_u16][..]),
            JsString::from(&[0xDC00_u16][..]),
        ),
    ] {
        // Both descending pairs differ in their first code unit: one Probe,
        // one Scan(1), and one extraction Write. Scalar Unicode order is not used.
        for limit in [2, 3] {
            let mut context = context();
            let mut values = vec![
                ImportAttribute::new(high.clone(), js_string!("HIGH")),
                ImportAttribute::new(low.clone(), js_string!("LOW")),
            ];
            let (outcome, direct) = scoped(&mut context, limit, |control| {
                sort_attributes(&mut values, control)
            });
            if limit == 3 {
                assert!(outcome.result.is_ok());
                assert!(direct.is_none());
                assert_eq!(outcome.report.module_metadata_work().unwrap(), 3);
                assert_eq!(
                    snapshot(&values),
                    vec![
                        (low.clone(), js_string!("LOW")),
                        (high.clone(), js_string!("HIGH"))
                    ]
                );
            } else {
                assert_refused(outcome, direct, limit, 2);
                assert_eq!(
                    snapshot(&values),
                    vec![
                        (high.clone(), js_string!("HIGH")),
                        (low.clone(), js_string!("LOW"))
                    ]
                );
            }
        }
    }
}

#[test]
fn a_retained_instruction_stop_wins_before_index_hashing_or_sort_mutation() {
    for sorting in [false, true] {
        let mut context = context();
        let hashes = Rc::new(Cell::new(0));
        let mut index = Index::default();
        let mut values = attributes("ba");
        let mut direct = None;
        let mut first = None;
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_module_metadata_work_limit(0),
                |mut access| {
                    let error = access.eval(Source::from_bytes("0")).unwrap_err();
                    assert_eq!(
                        error.as_engine(),
                        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                            limit: 0
                        }))
                    );
                    first = Some(error);
                    let mut control = Control::new(access.into_owner());
                    let result = if sorting {
                        sort_attributes(&mut values, &mut control)
                    } else {
                        index.insert(key(0, &hashes), 10_u8, &mut control)
                    };
                    direct = result.as_ref().err().cloned();
                    result
                },
            )
            .unwrap();
        let first = first.unwrap();
        assert_eq!(outcome.result.unwrap_err(), first);
        assert_eq!(direct.as_ref(), Some(&first));
        assert_eq!(outcome.report.module_metadata_work().unwrap(), 0);
        assert_eq!(outcome.report.instructions().unwrap(), 0);
        assert_eq!(hashes.get(), 0);
        assert_prefix(&index, 0, &mut context);
        assert_eq!(snapshot(&values), snapshot(&attributes("ba")));
    }
}
