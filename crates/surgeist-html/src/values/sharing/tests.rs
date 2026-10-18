//! New API characterization: no preimplementation callable pool existed.
use super::*;
use crate::{Namespace, QualifiedName};
use std::{cell::Cell, sync::Mutex};

#[derive(Clone, Copy, Default)]
struct Cleanup {
    slots: usize,
    entries: usize,
}
thread_local! {
    static CLEANUP: Cell<Option<Cleanup>> = const { Cell::new(None) };
}
struct ObserveCleanup;
impl ObserveCleanup {
    fn start() -> Self {
        CLEANUP.with(|observed| {
            assert!(observed.get().is_none(), "cleanup observer already active");
            observed.set(Some(Cleanup::default()));
        });
        Self
    }
    fn reset(&self) {
        CLEANUP.with(|observed| observed.set(Some(Cleanup::default())));
    }
    fn read(&self) -> Cleanup {
        CLEANUP.with(|observed| observed.get().unwrap())
    }
}
impl Drop for ObserveCleanup {
    fn drop(&mut self) {
        CLEANUP.with(|observed| observed.set(None));
    }
}
pub(super) fn observe_slot_drop() {
    CLEANUP.with(|observed| {
        if let Some(mut cleanup) = observed.get() {
            cleanup.slots += 1;
            observed.set(Some(cleanup));
        }
    });
}
pub(super) fn observe_entry_drop() {
    CLEANUP.with(|observed| {
        if let Some(mut cleanup) = observed.get() {
            cleanup.entries += 1;
            observed.set(Some(cleanup));
        }
    });
}

#[derive(Debug, Eq, PartialEq)]
enum Stop {
    Bytes,
    Units,
    Work,
    Cancelled,
}
struct Budget {
    pool: usize,
    candidate: usize,
    units: usize,
    work: usize,
    inspected: usize,
    visits: usize,
    upgrades: usize,
    cancel: bool,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            pool: usize::MAX,
            candidate: usize::MAX,
            units: usize::MAX,
            work: usize::MAX,
            inspected: 0,
            visits: 0,
            upgrades: 0,
            cancel: false,
        }
    }
}
impl Budget {
    fn admit(&mut self, action: &HtmlValuePoolAction) -> Result<(), Stop> {
        if self.cancel {
            return Err(Stop::Cancelled);
        }
        let layout = action.layout();
        if layout.pool().bytes() > self.pool || layout.candidate().bytes() > self.candidate {
            return Err(Stop::Bytes);
        }
        if layout.candidate().utf16_units() > self.units {
            return Err(Stop::Units);
        }
        let immediate = layout.work();
        let closing = layout.closing_work();
        let work = immediate.record_visits()
            + immediate.inspected_bytes()
            + immediate.copied_bytes()
            + immediate.provider_calls()
            + immediate.reserve_bytes()
            + closing.record_visits()
            + closing.inspected_bytes()
            + closing.copied_bytes()
            + closing.provider_calls()
            + closing.reserve_bytes();
        if work > self.work {
            return Err(Stop::Work);
        }
        self.work -= work;
        self.inspected += layout.work().inspected_bytes();
        self.visits += layout.work().record_visits();
        self.upgrades += usize::from(matches!(action.phase, Phase::Upgrade));
        Ok(())
    }
}
#[derive(Debug)]
enum Outcome<T> {
    Hit(T),
    Miss,
    Offered,
    Bypassed,
}
macro_rules! driver {
    ($name:ident, $pool:ty, $op:ident, $result:ident, $value:ty) => {
        fn $name(
            pool: &mut $pool,
            value: &$value,
            offer: bool,
            budget: &mut Budget,
        ) -> Result<Outcome<$value>, Stop> {
            let mut owner = Some(if offer {
                pool.offer(value)
            } else {
                pool.probe(value)
            });
            while let Some(action) = pool.next_action(owner.as_mut().unwrap()).unwrap() {
                // The input and operation are still owned if admission refuses.
                // No pool method receives this callback or cancellation policy.
                budget.admit(&action)?;
                match action.kind() {
                    HtmlValuePoolActionKind::Read => {
                        owner.as_mut().unwrap().read(action).unwrap();
                    }
                    HtmlValuePoolActionKind::Pool => {
                        pool.perform(owner.as_mut().unwrap(), action).unwrap();
                    }
                }
            }
            Ok(match $op::try_finish(&mut owner).unwrap() {
                $result::Hit(candidate) => Outcome::Hit(candidate.into_value()),
                $result::Miss => Outcome::Miss,
                $result::Offered(_) => Outcome::Offered,
                $result::Bypassed => Outcome::Bypassed,
            })
        }
    };
}
driver!(
    strings,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString
);
driver!(
    attributes,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes
);

fn raw<T: Value>(pool: &Pool<T>) -> HtmlValuePoolUsage {
    let pages = pool
        .pages
        .iter()
        .flatten()
        .map(|page| page.capacity() * size_of::<Slot<T>>())
        .sum::<usize>();
    let entries = pool
        .pages
        .iter()
        .flatten()
        .flat_map(|page| page.iter())
        .filter(|slot| slot.entry.is_some())
        .count();
    let undo = pool.log.iter().filter(|undo| undo.old.is_some()).count();
    HtmlValuePoolUsage {
        bytes: pages
            + pool.log.capacity() * size_of::<Undo<T>>()
            + (entries + undo) * size_of::<Vec<T::Item>>(),
        utf16_units: 0,
    }
}
fn name(
    namespace: Namespace,
    prefix: Option<DocumentString>,
    local: &[u16],
    value: &[u16],
) -> Attribute {
    Attribute::new(
        QualifiedName::new(namespace, prefix, DocumentString::from_utf16(local)),
        DocumentString::from_utf16(value),
    )
}
fn bank(entries: Vec<Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(entries).unwrap()
}
fn seed_string(pool: &mut HtmlStringPool, value: &DocumentString) {
    let root = pool.begin_root().unwrap();
    assert!(matches!(
        strings(pool, value, true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    let seal = pool.prepare_seal(root).unwrap();
    pool.seal(seal).unwrap();
}
fn seed_attributes(pool: &mut HtmlAttributePool, value: &OrderedAttributes) {
    let root = pool.begin_root().unwrap();
    assert!(matches!(
        attributes(pool, value, true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    let seal = pool.prepare_seal(root).unwrap();
    pool.seal(seal).unwrap();
}

#[test]
fn independently_allocated_exact_utf16_reuses_live_backing_without_copying_input() {
    for units in [
        vec![],
        vec![0, 0xd800, 0xdc00, 0xdfff, 32, 65],
        vec![0xd800],
        vec![0xfffd],
    ] {
        let mut reserved = Vec::with_capacity(units.len() + 11);
        reserved.extend_from_slice(&units);
        let original = DocumentString::from_shared_code_units(Arc::new(reserved));
        let input = DocumentString::from_utf16(units.clone());
        assert!(!Arc::ptr_eq(
            original.shared_code_units(),
            input.shared_code_units()
        ));
        let mut pool = HtmlStringPool::new();
        seed_string(&mut pool, &original);
        let Outcome::Hit(output) =
            strings(&mut pool, &input, false, &mut Budget::default()).unwrap()
        else {
            panic!("exact live match required");
        };
        assert!(Arc::ptr_eq(
            original.shared_code_units(),
            output.shared_code_units()
        ));
        assert_eq!(
            output.shared_code_units().capacity(),
            original.shared_code_units().capacity()
        );
        assert_eq!(input.code_units(), units);
        assert!(!Arc::ptr_eq(
            input.shared_code_units(),
            output.shared_code_units()
        ));
        assert_eq!(pool.heap_usage(), raw(&pool.inner));
    }
}

#[test]
fn forced_collisions_preserve_order_namespace_prefix_and_all_raw_units() {
    let original = bank(vec![
        name(Namespace::none(), None, &[97], &[0xd800]),
        name(
            Namespace::none(),
            Some(DocumentString::from("")),
            &[98],
            &[0],
        ),
    ]);
    let equal = bank(vec![
        name(Namespace::none(), None, &[97], &[0xd800]),
        name(
            Namespace::none(),
            Some(DocumentString::from("")),
            &[98],
            &[0],
        ),
    ]);
    let mut pool = HtmlAttributePool::new();
    pool.inner.collision = true;
    seed_attributes(&mut pool, &original);
    let Outcome::Hit(output) =
        attributes(&mut pool, &equal, false, &mut Budget::default()).unwrap()
    else {
        panic!("equal ordered bank");
    };
    assert!(Arc::ptr_eq(
        output.shared_attributes(),
        original.shared_attributes()
    ));
    let variants = [
        bank(original.iter().cloned().rev().collect()),
        bank(vec![
            name(
                Namespace::none(),
                Some(DocumentString::from("")),
                &[97],
                &[0xd800],
            ),
            original.as_slice()[1].clone(),
        ]),
        bank(vec![
            name(Namespace::from_uri("urn:x"), None, &[97], &[0xd800]),
            original.as_slice()[1].clone(),
        ]),
        bank(vec![
            name(Namespace::none(), None, &[65], &[0xd800]),
            original.as_slice()[1].clone(),
        ]),
        bank(vec![
            name(Namespace::none(), None, &[97], &[0xfffd]),
            original.as_slice()[1].clone(),
        ]),
        bank(vec![
            name(Namespace::none(), None, &[97], &[0xd800]),
            name(
                Namespace::none(),
                Some(DocumentString::from("p")),
                &[98],
                &[0],
            ),
        ]),
    ];
    for different in variants {
        assert!(matches!(
            attributes(&mut pool, &different, false, &mut Budget::default()).unwrap(),
            Outcome::Miss
        ));
    }
    // Sharing equality does not change duplicate-expanded-name semantics.
    let duplicate = OrderedAttributes::new(vec![
        name(Namespace::none(), None, &[97], &[1]),
        name(
            Namespace::none(),
            Some(DocumentString::from("p")),
            &[97],
            &[2],
        ),
    ])
    .unwrap_err();
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (0, 1)
    );
}

#[test]
fn forced_string_collisions_compare_each_raw_unit_without_decoding_or_folding() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let original = DocumentString::from_utf16(vec![32, 65, 0xd800, 0]);
    seed_string(&mut pool, &original);
    for units in [
        vec![32, 97, 0xd800, 0],
        vec![32, 65, 0xfffd, 0],
        vec![9, 65, 0xd800, 0],
        vec![32, 65, 0xd800, 1],
        vec![32, 65, 0xd800],
    ] {
        let input = DocumentString::from_utf16(units);
        let mut budget = Budget::default();
        assert!(matches!(
            strings(&mut pool, &input, false, &mut budget).unwrap(),
            Outcome::Miss
        ));
        assert_eq!(budget.upgrades, 1);
    }
    let mut pool = HtmlAttributePool::new();
    pool.inner.collision = true;
    let original = bank(vec![name(
        Namespace::from_uri("urn:x"),
        Some(DocumentString::from("p")),
        &[97],
        &[32, 65],
    )]);
    seed_attributes(&mut pool, &original);
    for different in [
        bank(vec![name(
            Namespace::from_uri("urn:X"),
            Some(DocumentString::from("p")),
            &[97],
            &[32, 65],
        )]),
        bank(vec![name(
            Namespace::from_uri("urn:x"),
            Some(DocumentString::from("P")),
            &[97],
            &[32, 65],
        )]),
        bank(vec![name(
            Namespace::from_uri("urn:x"),
            Some(DocumentString::from("p")),
            &[97],
            &[32, 97],
        )]),
    ] {
        let mut budget = Budget::default();
        assert!(matches!(
            attributes(&mut pool, &different, false, &mut budget).unwrap(),
            Outcome::Miss
        ));
        assert_eq!(budget.upgrades, 1);
    }
}

#[test]
fn caller_shared_values_keep_capacity_and_empty_reserved_banks_are_real_candidates() {
    let backing = Arc::new(Vec::<u16>::with_capacity(31));
    let original = DocumentString::from_shared_code_units(backing.clone());
    let mut pool = HtmlStringPool::new();
    seed_string(&mut pool, &original);
    let mut budget = Budget::default();
    let Outcome::Hit(value) = strings(&mut pool, &original, false, &mut budget).unwrap() else {
        panic!("same backing");
    };
    assert!(Arc::ptr_eq(&backing, value.shared_code_units()));
    assert_eq!(budget.inspected, 0);
    let values = Vec::<Attribute>::with_capacity(7);
    let mut validation = Some(
        crate::bounded::HtmlAttributeStorage::from_shared_attributes(Arc::new(values)).validate(),
    );
    let original = crate::bounded::HtmlAttributeValidation::try_finish(&mut validation)
        .unwrap()
        .into_attributes();
    let mut pool = HtmlAttributePool::new();
    seed_attributes(&mut pool, &original);
    let empty = OrderedAttributes::default();
    let Outcome::Hit(value) = attributes(&mut pool, &empty, false, &mut Budget::default()).unwrap()
    else {
        panic!("equal empty bank");
    };
    assert_eq!(value.shared_attributes().capacity(), 7);
    assert!(Arc::ptr_eq(
        value.shared_attributes(),
        original.shared_attributes()
    ));
}

#[test]
fn one_short_entry_refusal_then_exact_admission_keeps_input_and_old_pool_valid() {
    let input = DocumentString::from("alpha");
    let backing = input.shared_code_units().clone();
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let requested = PAGE * size_of::<Slot<DocumentString>>()
        + LOG * size_of::<Undo<DocumentString>>()
        + size_of::<Vec<u16>>();
    let mut short = Budget {
        pool: requested - 1,
        ..Budget::default()
    };
    assert_eq!(
        strings(&mut pool, &input, true, &mut short).unwrap_err(),
        Stop::Bytes
    );
    assert_eq!(pool.heap_usage().bytes(), 0);
    assert!(Arc::ptr_eq(&backing, input.shared_code_units()));
    let mut exact = Budget {
        pool: requested,
        ..Budget::default()
    };
    assert!(matches!(
        strings(&mut pool, &input, true, &mut exact).unwrap(),
        Outcome::Offered
    ));
    assert_eq!(pool.heap_usage(), raw(&pool.inner));
    assert_eq!(pool.heap_usage().bytes(), requested);
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage().bytes(), 0);
}

#[test]
fn optional_metadata_byte_cap_bypasses_without_hiding_mandatory_refusal() {
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    pool.inner.metadata_limit = 1;
    let input = DocumentString::from("bounded");
    assert!(matches!(
        strings(
            &mut pool,
            &input,
            true,
            &mut Budget {
                pool: 0,
                ..Budget::default()
            }
        )
        .unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(
        strings(
            &mut pool,
            &input,
            true,
            &mut Budget {
                work: 0,
                ..Budget::default()
            }
        )
        .unwrap_err(),
        Stop::Work
    );
    assert!(matches!(
        strings(&mut pool, &input, true, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(pool.heap_usage().bytes(), 0);
    pool.inner.metadata_limit = MAX_METADATA_BYTES;
    assert_eq!(
        strings(
            &mut pool,
            &input,
            true,
            &mut Budget {
                pool: 0,
                ..Budget::default()
            }
        )
        .unwrap_err(),
        Stop::Bytes
    );
    pool.abort_root(root).unwrap();
}

#[test]
fn strong_upgrade_is_reserved_before_obtaining_the_candidate_and_transfers_its_capacity() {
    let mut units = Vec::with_capacity(23);
    units.extend_from_slice(&[97, 0xd800]);
    let original = DocumentString::from_utf16(units);
    let mut pool = HtmlStringPool::new();
    seed_string(&mut pool, &original);
    let input = DocumentString::from_utf16(vec![97, 0xd800]);
    let bytes = size_of::<Vec<u16>>() + 23 * size_of::<u16>();
    for (candidate, units, error) in [(bytes - 1, 23, Stop::Bytes), (bytes, 22, Stop::Units)] {
        let mut budget = Budget {
            candidate,
            units,
            ..Budget::default()
        };
        assert_eq!(
            strings(&mut pool, &input, false, &mut budget).unwrap_err(),
            error
        );
        assert_eq!(budget.upgrades, 0, "refusal precedes actual upgrade");
        assert_eq!(
            Arc::strong_count(original.shared_code_units()),
            1,
            "no unadmitted strong holder was retained"
        );
        assert_eq!(pool.heap_usage(), raw(&pool.inner));
    }
    let mut budget = Budget {
        candidate: bytes,
        units: 23,
        ..Budget::default()
    };
    let Outcome::Hit(output) = strings(&mut pool, &input, false, &mut budget).unwrap() else {
        panic!("exact upgrade");
    };
    assert_eq!(budget.upgrades, 1);
    assert!(Arc::ptr_eq(
        original.shared_code_units(),
        output.shared_code_units()
    ));
    drop(pool);
    assert_eq!(output.code_units(), &[97, 0xd800]);
}

#[test]
fn dead_weak_keeps_only_target_space_and_incremental_replacement_releases_it() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let original = DocumentString::from("old");
    let weak = Arc::downgrade(original.shared_code_units());
    seed_string(&mut pool, &original);
    let retained = pool.heap_usage();
    drop(original);
    assert!(weak.upgrade().is_none());
    assert_eq!(
        pool.heap_usage(),
        retained,
        "dead Weak target space lasts until pool occurrence drops"
    );
    let root = pool.begin_root().unwrap();
    let next = DocumentString::from("new");
    assert!(matches!(
        strings(&mut pool, &next, true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    assert_eq!(pool.heap_usage(), raw(&pool.inner));
    let seal = pool.prepare_seal(root).unwrap();
    let change = pool.seal(seal).unwrap();
    assert_eq!(change.released_occurrences(), 1);
    assert_eq!(pool.heap_usage(), retained);
}

#[test]
fn probe_misses_never_offer_the_original_and_final_bank_offer_preserves_components() {
    let mut pool = HtmlAttributePool::new();
    let original = bank(vec![name(Namespace::none(), None, &[97], &[98])]);
    assert!(matches!(
        attributes(&mut pool, &original, false, &mut Budget::default()).unwrap(),
        Outcome::Miss
    ));
    assert_eq!(pool.heap_usage().bytes(), 0);
    let component = original.as_slice()[0].value().clone();
    let final_bank = bank(vec![Attribute::new(
        original.as_slice()[0].name().clone(),
        component.clone(),
    )]);
    seed_attributes(&mut pool, &final_bank);
    let Outcome::Hit(selected) =
        attributes(&mut pool, &original, false, &mut Budget::default()).unwrap()
    else {
        panic!("offered final bank");
    };
    assert!(Arc::ptr_eq(
        selected.shared_attributes(),
        final_bank.shared_attributes()
    ));
    assert!(Arc::ptr_eq(
        selected.as_slice()[0].value().shared_code_units(),
        component.shared_code_units()
    ));
    assert!(!Arc::ptr_eq(
        selected.shared_attributes(),
        original.shared_attributes()
    ));
}

#[test]
fn replacement_overlap_requires_another_weak_occurrence_and_abort_restores_old_backing() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let old: Vec<_> = (0..4)
        .map(|n| DocumentString::from(format!("old{n}")))
        .collect();
    for value in &old {
        seed_string(&mut pool, value);
    }
    let baseline = raw(&pool.inner);
    let root = pool.begin_root().unwrap();
    let replacement = DocumentString::from("replacement");
    let exact = baseline.bytes() + size_of::<Vec<u16>>();
    assert_eq!(
        strings(
            &mut pool,
            &replacement,
            true,
            &mut Budget {
                pool: exact - 1,
                ..Budget::default()
            }
        )
        .unwrap_err(),
        Stop::Bytes
    );
    assert_eq!(pool.heap_usage(), baseline);
    for value in &old {
        assert!(matches!(
            strings(&mut pool, value, false, &mut Budget::default()).unwrap(),
            Outcome::Hit(_)
        ));
    }
    assert!(matches!(
        strings(
            &mut pool,
            &replacement,
            true,
            &mut Budget {
                pool: exact,
                ..Budget::default()
            }
        )
        .unwrap(),
        Outcome::Offered
    ));
    assert_eq!(pool.heap_usage().bytes(), exact);
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage(), baseline);
    for value in &old {
        let equal = DocumentString::from_utf16(value.code_units().to_vec());
        let Outcome::Hit(selected) =
            strings(&mut pool, &equal, false, &mut Budget::default()).unwrap()
        else {
            panic!("restored candidate");
        };
        assert!(Arc::ptr_eq(
            selected.shared_code_units(),
            value.shared_code_units()
        ));
    }
}

#[test]
fn candidate_output_moves_its_reservation_and_drops_data_before_the_receipt() {
    struct Receipt {
        target: Weak<Vec<u16>>,
    }
    impl Drop for Receipt {
        fn drop(&mut self) {
            assert!(
                self.target.upgrade().is_none(),
                "selected strong data drops before its receipt"
            );
        }
    }
    struct Held<T> {
        value: T,
        receipt: Receipt,
    }
    impl<T> Held<T> {
        fn map<U>(self, convert: impl FnOnce(T) -> U) -> Held<U> {
            Held {
                value: convert(self.value),
                receipt: self.receipt,
            }
        }
    }
    let mut pool = HtmlStringPool::new();
    let original = DocumentString::from("transfer");
    let target = Arc::downgrade(original.shared_code_units());
    seed_string(&mut pool, &original);
    let input = DocumentString::from("transfer");
    let mut owner = Some(pool.probe(&input));
    let mut receipt = None;
    while let Some(action) = pool.next_action(owner.as_mut().unwrap()).unwrap() {
        Budget::default().admit(&action).unwrap();
        if action.phase == Phase::Upgrade {
            assert!(action.layout().closing_work().record_visits() >= 1);
            assert!(action.layout().closing_work().provider_calls() >= 2);
            // The receipt exists BEFORE upgrade. A real consumer uses its own
            // admitted affine debit here; this guard checks release ordering.
            receipt = Some(Receipt {
                target: target.clone(),
            });
        }
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                owner.as_mut().unwrap().read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.perform(owner.as_mut().unwrap(), action).unwrap();
            }
        }
    }
    let HtmlStringPoolResult::Hit(candidate) =
        HtmlStringPoolOperation::try_finish(&mut owner).unwrap()
    else {
        panic!("live candidate");
    };
    let selected = Held {
        value: candidate,
        receipt: receipt.unwrap(),
    }
    .map(HtmlStringPoolCandidate::into_value);
    drop(original);
    drop(pool);
    assert_eq!(selected.value.code_units(), input.code_units());
    assert!(target.upgrade().is_some());
    drop(selected);
    assert!(target.upgrade().is_none());
}

#[test]
fn capped_collision_bucket_evicts_incrementally_and_unique_stream_retention_is_bounded() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let values: Vec<_> = (0..400)
        .map(|n| DocumentString::from(n.to_string()))
        .collect();
    for value in &values {
        seed_string(&mut pool, value);
        assert_eq!(pool.heap_usage(), raw(&pool.inner));
        assert_eq!(pool.inner.pages.iter().flatten().count(), 1);
        assert!(
            pool.inner
                .pages
                .iter()
                .flatten()
                .flat_map(|page| page.iter())
                .filter(|slot| slot.entry.is_some())
                .count()
                <= 4
        );
    }
    let mut budget = Budget::default();
    assert!(matches!(
        strings(&mut pool, &values[0], false, &mut budget).unwrap(),
        Outcome::Miss
    ));
    assert!(budget.upgrades <= 4);
    let mut budget = Budget::default();
    assert!(matches!(
        strings(&mut pool, values.last().unwrap(), false, &mut budget).unwrap(),
        Outcome::Hit(_)
    ));
    assert!(budget.upgrades <= 4);
    let mut pool = HtmlStringPool::new();
    for value in &values {
        seed_string(&mut pool, value);
    }
    assert!(pool.inner.pages.iter().flatten().count() <= PAGES);
    assert!(
        pool.inner
            .pages
            .iter()
            .flatten()
            .flat_map(|page| page.iter())
            .filter(|slot| slot.entry.is_some())
            .count()
            <= BUCKETS * CANDIDATES
    );
    assert!(pool.heap_usage().bytes() <= MAX_METADATA_BYTES);
    assert_eq!(pool.heap_usage(), raw(&pool.inner));
}

#[test]
fn hot_repeated_content_mixed_with_unique_ids_shares_while_live_and_remains_bounded() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let mut retained_hot = DocumentString::from("repeated cell value");
    seed_string(&mut pool, &retained_hot);
    let mut hits = 0;
    let mut misses = 0;
    let rows: Vec<_> = (0..64)
        .map(|row| DocumentString::from(format!("row-{row}")))
        .collect();
    for unique in &rows {
        let root = pool.begin_root().unwrap();
        assert!(matches!(
            strings(&mut pool, unique, true, &mut Budget::default()).unwrap(),
            Outcome::Offered
        ));
        let incoming = DocumentString::from("repeated cell value");
        match strings(&mut pool, &incoming, false, &mut Budget::default()).unwrap() {
            Outcome::Hit(selected) => {
                hits += 1;
                assert!(Arc::ptr_eq(
                    selected.shared_code_units(),
                    retained_hot.shared_code_units()
                ));
                assert_eq!(selected.code_units(), incoming.code_units());
            }
            Outcome::Miss => {
                misses += 1;
                assert!(matches!(
                    strings(&mut pool, &incoming, true, &mut Budget::default()).unwrap(),
                    Outcome::Offered
                ));
                retained_hot = incoming;
            }
            other => panic!("unexpected hot probe: {other:?}"),
        }
        let seal = pool.prepare_seal(root).unwrap();
        pool.seal(seal).unwrap();
        assert_eq!(pool.heap_usage(), raw(&pool.inner));
        assert!(
            pool.inner
                .pages
                .iter()
                .flatten()
                .flat_map(|page| page.iter())
                .filter(|slot| slot.entry.is_some())
                .count()
                <= CANDIDATES
        );
    }
    // Collisions and eviction make canonical addresses optional; a held value
    // survives eviction, while actual live-candidate hits reuse that backing.
    assert!(hits > 0 && misses > 0);
    drop(pool);
    assert_eq!(
        retained_hot.code_units(),
        "repeated cell value".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn mixed_live_outputs_cross_root_log_capacity_and_restore_original_candidates() {
    fn measured(
        pool: &mut HtmlStringPool,
        input: &DocumentString,
        offer: bool,
    ) -> Outcome<DocumentString> {
        const SHORT_UNITS: usize = 32;
        assert!(input.len() <= SHORT_UNITS);
        let mut budget = Budget::default();
        let result = strings(pool, input, offer, &mut budget).unwrap();
        // One input fingerprint and at most four exact candidate comparisons.
        // Header/slot visits and new metadata are separate, bounded work.
        let reads =
            input.len() * size_of::<u16>() + CANDIDATES * 2 * SHORT_UNITS * size_of::<u16>();
        let visits = 1 + CANDIDATES * 4 + 1;
        let allocation =
            2 * PAGE * size_of::<Slot<DocumentString>>() + LOG * size_of::<Undo<DocumentString>>();
        let transition =
            2 * (size_of::<Undo<DocumentString>>() + size_of::<Slot<DocumentString>>());
        // Each of four upgrades prepays one Vec target and at most two
        // releases. A Prepare additionally owns an initialized page, one Weak,
        // and up to three release calls, even if the caller never commits it.
        let temporary_cleanup = CANDIDATES * 3 + PAGE + 1 + 3;
        assert!(budget.inspected <= reads);
        assert!(budget.visits <= visits);
        assert!(budget.upgrades <= CANDIDATES);
        assert!(
            usize::MAX - budget.work
                <= reads
                    + visits
                    + 1
                    + CANDIDATES
                    + 3
                    + allocation
                    + transition
                    + temporary_cleanup
        );
        assert_eq!(pool.heap_usage(), raw(&pool.inner));
        assert!(pool.heap_usage().bytes() <= MAX_METADATA_BYTES);
        assert!(pool.inner.log.len() <= LOG);
        result
    }

    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let literals = [
        vec![99, 101, 108, 108, 32, 99, 111, 109, 109, 111, 110],
        vec![0xd800, 0, 32, 65],
    ];
    let original: Vec<_> = literals
        .iter()
        .map(|units| DocumentString::from_utf16(units.clone()))
        .collect();
    for value in &original {
        seed_string(&mut pool, value);
    }
    let baseline = raw(&pool.inner);
    let cursors = pool.inner.cursors;
    let root = pool.begin_root().unwrap();
    let mark = pool.mark(&root).unwrap().unwrap();
    let rows: Vec<_> = (0..3 * LOG)
        .map(|row| DocumentString::from(format!("row-{row}")))
        .collect();
    let mut outputs = Vec::new();
    let mut offered_hot = original.clone();
    let mut last_offered = None;
    let mut plateau = None;
    let mut hot_hits = 0;
    let mut hot_misses = 0;
    let mut offered = 0;
    let mut bypassed = 0;
    for (row, unique) in rows.iter().enumerate() {
        let full_before = pool.inner.log.len() == LOG;
        match measured(&mut pool, unique, true) {
            Outcome::Offered => {
                assert!(!full_before);
                offered += 1;
                last_offered = Some(unique.clone());
            }
            Outcome::Bypassed => {
                assert!(full_before);
                bypassed += 1;
            }
            other => panic!("fresh distinct row ID: {other:?}"),
        }
        outputs.push(unique.clone());
        let units = &literals[row % literals.len()];
        let incoming = DocumentString::from_utf16(units.clone());
        let selected = match measured(&mut pool, &incoming, false) {
            Outcome::Hit(selected) => {
                hot_hits += 1;
                assert_eq!(selected.code_units(), units);
                assert!(offered_hot.iter().any(|prior| Arc::ptr_eq(
                    prior.shared_code_units(),
                    selected.shared_code_units()
                )));
                assert!(!Arc::ptr_eq(
                    incoming.shared_code_units(),
                    selected.shared_code_units()
                ));
                selected
            }
            Outcome::Miss => {
                hot_misses += 1;
                let full_before = pool.inner.log.len() == LOG;
                match measured(&mut pool, &incoming, true) {
                    Outcome::Offered => {
                        assert!(!full_before);
                        offered += 1;
                        last_offered = Some(incoming.clone());
                        offered_hot.push(incoming.clone());
                    }
                    Outcome::Bypassed => {
                        assert!(full_before);
                        bypassed += 1;
                    }
                    other => panic!("same miss offered without intervening mutation: {other:?}"),
                }
                let fallback = incoming.clone();
                assert!(Arc::ptr_eq(
                    fallback.shared_code_units(),
                    incoming.shared_code_units()
                ));
                assert_eq!(
                    fallback.shared_code_units().capacity(),
                    incoming.shared_code_units().capacity()
                );
                fallback
            }
            other => panic!("qualified read-only hot probe: {other:?}"),
        };
        outputs.push(selected);
        if pool.inner.log.len() == LOG {
            let observed = (pool.heap_usage(), pool.inner.cursors);
            if let Some(expected) = plateau {
                assert_eq!(observed, expected);
            } else {
                plateau = Some(observed);
            }
        }
    }
    assert_eq!(offered, LOG);
    assert!(hot_hits > 0 && hot_misses > 0 && bypassed > 0);
    assert_eq!(pool.inner.log.len(), LOG);
    // The last actually retained occurrence cannot be evicted after all writes
    // are suspended by the full log. This is a local live-hit assertion, not a
    // promise that every hot value survives collision-driven eviction.
    let retained = last_offered.unwrap();
    let equal = DocumentString::from_utf16(retained.code_units().to_vec());
    let Outcome::Hit(selected) = measured(&mut pool, &equal, false) else {
        panic!("last retained live candidate remains probeable");
    };
    assert!(Arc::ptr_eq(
        selected.shared_code_units(),
        retained.shared_code_units()
    ));
    pool.restore_mark(mark).unwrap();
    assert_eq!(pool.heap_usage(), baseline);
    assert_eq!(pool.inner.cursors, cursors);
    for (literal, original) in literals.iter().zip(&original) {
        let input = DocumentString::from_utf16(literal.clone());
        let Outcome::Hit(restored) = measured(&mut pool, &input, false) else {
            panic!("known pre-root candidate");
        };
        assert!(Arc::ptr_eq(
            restored.shared_code_units(),
            original.shared_code_units()
        ));
    }
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage(), baseline);
    drop(pool);
    assert_eq!(outputs.len(), rows.len() * 2);
    for (row, pair) in outputs.chunks_exact(2).enumerate() {
        assert!(Arc::ptr_eq(
            pair[0].shared_code_units(),
            rows[row].shared_code_units()
        ));
        assert_eq!(pair[1].code_units(), literals[row % literals.len()]);
    }
    eprintln!(
        "mixed root: offered={offered}, hot_hits={hot_hits}, hot_misses={hot_misses}, bypassed={bypassed}, retained_outputs={}",
        outputs.len()
    );
}

#[test]
fn nested_restore_and_root_abort_restore_evicted_backings_cursors_and_actual_metadata() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let baseline = DocumentString::from("baseline");
    seed_string(&mut pool, &baseline);
    let usage = pool.heap_usage();
    let cursors = pool.inner.cursors;
    let root = pool.begin_root().unwrap();
    let mark = pool.mark(&root).unwrap().unwrap();
    let values: Vec<_> = (0..8)
        .map(|n| DocumentString::from(format!("new{n}")))
        .collect();
    for value in &values {
        assert!(matches!(
            strings(&mut pool, value, true, &mut Budget::default()).unwrap(),
            Outcome::Offered
        ));
    }
    assert!(pool.heap_usage().bytes() > usage.bytes());
    pool.restore_mark(mark).unwrap();
    assert_eq!(pool.heap_usage(), usage);
    assert_eq!(pool.inner.cursors, cursors);
    let equal = DocumentString::from("baseline");
    let Outcome::Hit(restored) = strings(&mut pool, &equal, false, &mut Budget::default()).unwrap()
    else {
        panic!("original candidate restored");
    };
    assert!(Arc::ptr_eq(
        restored.shared_code_units(),
        baseline.shared_code_units()
    ));
    let mark = pool.mark(&root).unwrap().unwrap();
    assert!(matches!(
        strings(&mut pool, &values[0], true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    pool.accept_mark(mark).unwrap();
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage(), usage);
    assert_eq!(pool.inner.cursors, cursors);
}

#[test]
fn finite_session_log_stops_mutations_without_losing_root_rollback_authority() {
    let mut pool = HtmlStringPool::new();
    pool.inner.collision = true;
    let root = pool.begin_root().unwrap();
    let values: Vec<_> = (0..LOG + 3)
        .map(|n| DocumentString::from(n.to_string()))
        .collect();
    for value in &values[..LOG] {
        assert!(matches!(
            strings(&mut pool, value, true, &mut Budget::default()).unwrap(),
            Outcome::Offered
        ));
    }
    let usage = pool.heap_usage();
    for value in &values[LOG..] {
        assert!(matches!(
            strings(&mut pool, value, true, &mut Budget::default()).unwrap(),
            Outcome::Bypassed
        ));
    }
    assert_eq!(pool.heap_usage(), usage);
    assert_eq!(pool.inner.log.len(), LOG);
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage().bytes(), 0);
}

#[test]
fn cancelled_spent_work_and_unaccepted_preparation_do_not_mutate_or_replace_inputs() {
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let input = DocumentString::from("keep");
    let backing = input.shared_code_units().clone();
    assert_eq!(
        strings(
            &mut pool,
            &input,
            true,
            &mut Budget {
                cancel: true,
                ..Budget::default()
            }
        )
        .unwrap_err(),
        Stop::Cancelled
    );
    let mut budget = Budget {
        work: 3,
        ..Budget::default()
    };
    assert_eq!(
        strings(&mut pool, &input, true, &mut budget).unwrap_err(),
        Stop::Work
    );
    assert_eq!(budget.work, 0);
    assert_eq!(budget.inspected, 2);
    let mut operation = pool.offer(&input);
    while let Some(action) = pool.next_action(&mut operation).unwrap() {
        if action.phase == Phase::Commit {
            break;
        }
        Budget::default().admit(&action).unwrap();
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                operation.read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.perform(&mut operation, action).unwrap();
            }
        }
    }
    assert!(operation.inner.prepared.is_some());
    drop(operation);
    assert_eq!(pool.heap_usage().bytes(), 0);
    assert!(Arc::ptr_eq(&backing, input.shared_code_units()));
    pool.abort_root(root).unwrap();
}

#[test]
fn stale_actions_bypass_and_foreign_actions_are_protocol_errors() {
    let mut pool = HtmlStringPool::new();
    let input = DocumentString::from("a");
    let mut operation = pool.probe(&input);
    let stale = pool.next_action(&mut operation).unwrap().unwrap();
    let _later = pool.next_action(&mut operation).unwrap().unwrap();
    assert_eq!(
        operation.read(stale).unwrap(),
        HtmlValuePoolProgress::Finished
    );
    assert!(matches!(
        HtmlStringPoolOperation::try_finish(&mut Some(operation)).unwrap(),
        HtmlStringPoolResult::Bypassed
    ));
    let mut operation = pool.probe(&input);
    while operation.inner.phase == Phase::Hash {
        let action = pool.next_action(&mut operation).unwrap().unwrap();
        operation.read(action).unwrap();
    }
    let action = pool.next_action(&mut operation).unwrap().unwrap();
    let mut other = HtmlStringPool::new();
    assert_eq!(
        other.perform(&mut operation, action).unwrap_err(),
        HtmlValuePoolError::ForeignOwner
    );
    let mut operation = pool.probe(&input);
    while operation.inner.phase == Phase::Hash {
        let action = pool.next_action(&mut operation).unwrap().unwrap();
        operation.read(action).unwrap();
    }
    let action = pool.next_action(&mut operation).unwrap().unwrap();
    let _new_operation = pool.probe(&input);
    assert_eq!(
        pool.perform(&mut operation, action).unwrap(),
        HtmlValuePoolProgress::Finished
    );
    assert_eq!(pool.heap_usage().bytes(), 0);
}

#[test]
fn admission_callbacks_run_after_releasing_the_consumer_mutex() {
    let pool = Mutex::new(HtmlStringPool::new());
    let input = DocumentString::from("callback");
    let root = pool.lock().unwrap().begin_root().unwrap();
    let mut owner = Some(pool.lock().unwrap().offer(&input));
    loop {
        let action = pool
            .lock()
            .unwrap()
            .next_action(owner.as_mut().unwrap())
            .unwrap();
        let Some(action) = action else {
            break;
        };
        // This is the real consumer callback boundary. It can lock the owner,
        // proving the owned action retained neither lock nor hidden borrow.
        assert!(pool.try_lock().is_ok());
        Budget::default().admit(&action).unwrap();
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                owner.as_mut().unwrap().read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.lock()
                    .unwrap()
                    .perform(owner.as_mut().unwrap(), action)
                    .unwrap();
            }
        }
    }
    assert!(matches!(
        HtmlStringPoolOperation::try_finish(&mut owner).unwrap(),
        HtmlStringPoolResult::Offered(_)
    ));
    pool.lock().unwrap().abort_root(root).unwrap();
}

#[test]
fn qualifier_exhaustion_bypasses_but_existing_marks_and_seals_close_without_new_ids() {
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let mark = pool.mark(&root).unwrap().unwrap();
    let input = DocumentString::from("before");
    assert!(matches!(
        strings(&mut pool, &input, true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    pool.inner.next = u64::MAX;
    assert!(matches!(
        strings(&mut pool, &input, false, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    pool.restore_mark(mark).unwrap();
    assert_eq!(pool.heap_usage().bytes(), 0);
    let next = pool.inner.next;
    let seal = pool.prepare_seal(root).unwrap();
    pool.validate_seal(&seal).unwrap();
    pool.seal(seal).unwrap();
    assert_eq!(pool.inner.next, next);
    assert!(pool.begin_root().is_none());
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    pool.inner.next = u64::MAX;
    assert!(pool.mark(&root).unwrap().is_none());
    assert!(pool.inner.root.as_ref().unwrap().suspended);
    assert!(matches!(
        strings(&mut pool, &input, true, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    pool.abort_root(root).unwrap();
    assert_eq!(pool.heap_usage().bytes(), 0);
}

#[test]
fn exhausted_entry_generation_does_not_allocate_and_existing_authority_can_abort() {
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let input = DocumentString::from("entry");
    let mut operation = pool.offer(&input);
    loop {
        let action = pool.next_action(&mut operation).unwrap().unwrap();
        Budget::default().admit(&action).unwrap();
        if action.phase == Phase::Prepare {
            pool.inner.next = u64::MAX;
            assert_eq!(
                pool.perform(&mut operation, action).unwrap(),
                HtmlValuePoolProgress::Finished
            );
            break;
        }
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                operation.read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.perform(&mut operation, action).unwrap();
            }
        }
    }
    assert!(operation.inner.prepared.is_none());
    assert_eq!(pool.heap_usage().bytes(), 0);
    pool.abort_root(root).unwrap();
    assert_eq!(pool.inner.next, u64::MAX);
}

#[test]
fn unavailable_nested_mark_suspends_writes_and_old_nested_tokens_cannot_affect_new_root() {
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let mut marks = Vec::new();
    for _ in 0..MARKS {
        marks.push(pool.mark(&root).unwrap().unwrap());
    }
    assert!(pool.mark(&root).unwrap().is_none());
    let input = DocumentString::from("unlogged");
    assert!(matches!(
        strings(&mut pool, &input, true, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    pool.abort_root(root).unwrap();
    let root = pool.begin_root().unwrap();
    assert!(matches!(
        strings(&mut pool, &input, true, &mut Budget::default()).unwrap(),
        Outcome::Offered
    ));
    for mark in marks {
        pool.restore_mark(mark).unwrap();
    }
    let equal = DocumentString::from("unlogged");
    assert!(matches!(
        strings(&mut pool, &equal, false, &mut Budget::default()).unwrap(),
        Outcome::Hit(_)
    ));
    let seal = pool.prepare_seal(root).unwrap();
    pool.abort_seal(seal).unwrap();
    assert_eq!(pool.heap_usage().bytes(), 0);
}

#[test]
fn two_concrete_pools_can_prepare_both_seals_before_either_commit_or_abort() {
    let mut strings_pool = HtmlStringPool::new();
    let mut attrs_pool = HtmlAttributePool::new();
    let string_root = strings_pool.begin_root().unwrap();
    let attrs_root = attrs_pool.begin_root().unwrap();
    let string = DocumentString::from("same");
    let attrs = bank(vec![name(Namespace::none(), None, &[97], &[98])]);
    strings(&mut strings_pool, &string, true, &mut Budget::default()).unwrap();
    attributes(&mut attrs_pool, &attrs, true, &mut Budget::default()).unwrap();
    let first = strings_pool.prepare_seal(string_root).unwrap();
    let second = attrs_pool.prepare_seal(attrs_root).unwrap();
    strings_pool.validate_seal(&first).unwrap();
    attrs_pool.validate_seal(&second).unwrap();
    strings_pool.inner.next = u64::MAX;
    attrs_pool.inner.next = u64::MAX;
    strings_pool.seal(first).unwrap();
    attrs_pool.seal(second).unwrap();
    assert!(matches!(
        strings(&mut strings_pool, &string, false, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(attrs_pool.heap_usage(), raw(&attrs_pool.inner));
    // A consumer can restore the first acquired nested mark when the second
    // pool cannot issue one. No mark acquisition itself mutates entries.
    let mut first = HtmlStringPool::new();
    let root = first.begin_root().unwrap();
    let mark = first.mark(&root).unwrap().unwrap();
    let mut second = HtmlAttributePool::new();
    second.inner.next = u64::MAX;
    assert!(second.begin_root().is_none());
    first.restore_mark(mark).unwrap();
    first.suspend_mutations(&root).unwrap();
    assert!(matches!(
        strings(&mut first, &string, true, &mut Budget::default()).unwrap(),
        Outcome::Bypassed
    ));
    first.abort_root(root).unwrap();
    assert_eq!(first.heap_usage().bytes(), 0);
}

#[test]
fn ineligible_lengths_and_capacities_bypass_before_unit_reads_and_pool_drop_releases_weak() {
    let mut pool = HtmlStringPool::new();
    for value in [
        DocumentString::from_utf16(vec![0; MAX_UNITS + 1]),
        DocumentString::from_utf16(Vec::with_capacity(MAX_UNITS + 1)),
    ] {
        let mut budget = Budget::default();
        assert!(matches!(
            strings(&mut pool, &value, false, &mut budget).unwrap(),
            Outcome::Bypassed
        ));
        assert_eq!(budget.inspected, 0);
        assert_eq!(pool.heap_usage().bytes(), 0);
    }
    let value = DocumentString::from("held");
    let weak = Arc::downgrade(value.shared_code_units());
    seed_string(&mut pool, &value);
    assert_eq!(
        Arc::weak_count(value.shared_code_units()),
        2,
        "test observes physical references, never grants custody"
    );
    drop(pool);
    assert_eq!(Arc::weak_count(value.shared_code_units()), 1);
    assert_eq!(weak.upgrade().unwrap().as_slice(), &[104, 101, 108, 100]);
}

#[test]
fn invariant_validated_bank_proof_transfer_keeps_original_order_capacity_and_backing() {
    let mut entries = Vec::with_capacity(19);
    entries.push(name(Namespace::none(), None, &[98], &[0xd800]));
    entries.push(name(
        Namespace::from_uri("urn:x"),
        Some(DocumentString::from("")),
        &[97],
        &[0],
    ));
    let original = bank(entries);
    let backing = original.shared_attributes().clone();
    let proof = crate::bounded::HtmlValidatedAttributes::from(original);
    assert_eq!(proof.as_slice()[0].name().local_name().code_units(), &[98]);
    let result = proof.into_attributes();
    assert!(Arc::ptr_eq(&backing, result.shared_attributes()));
    assert_eq!(result.shared_attributes().capacity(), 19);
}

#[test]
fn incomplete_finish_retains_the_original_operation_and_large_components_bypass_before_units() {
    let mut pool = HtmlStringPool::new();
    let input = DocumentString::from("incomplete");
    let mut owner = Some(pool.probe(&input));
    assert_eq!(
        HtmlStringPoolOperation::try_finish(&mut owner).unwrap_err(),
        HtmlValuePoolError::NotReady
    );
    assert!(owner.is_some());
    drop(owner);
    let large = DocumentString::from_utf16(vec![0xd800; MAX_UNITS + 1]);
    let original = bank(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, "a"),
        large.clone(),
    )]);
    let mut pool = HtmlAttributePool::new();
    let mut budget = Budget::default();
    assert!(matches!(
        attributes(&mut pool, &original, false, &mut budget).unwrap(),
        Outcome::Bypassed
    ));
    assert_eq!(
        budget.inspected,
        size_of::<u16>(),
        "only the eligible local name was read"
    );
    assert_eq!(pool.heap_usage().bytes(), 0);
    assert!(Arc::ptr_eq(
        original.as_slice()[0].value().shared_code_units(),
        large.shared_code_units()
    ));
}

// Existing callable-boundary regression: Prepare has already retained a real
// provisional Weak when a caller can abandon/refuse the following Commit.
// Fixed root/mark work belongs to that authority's own close; the action must
// separately prepay cleanup of storage it introduces before it introduces it.
macro_rules! abandoned_preparation_cleanup {
    ($test:ident, $pool:ty, $input:expr, $shared:ident) => {
        #[test]
        fn $test() {
            let input = $input;
            let backing = input.$shared().clone();
            let capacity = backing.capacity();
            let mut pool = <$pool>::new();
            let before = pool.heap_usage();
            let root = pool.begin_root().unwrap();
            let mut operation = pool.offer(&input);
            let mut budget = Budget::default();
            let cleanup = loop {
                let action = pool.next_action(&mut operation).unwrap().unwrap();
                budget.admit(&action).unwrap();
                if action.phase == Phase::Prepare {
                    let cleanup = action.layout().closing_work();
                    assert_eq!(Arc::weak_count(&backing), 0);
                    assert_eq!(
                        pool.perform(&mut operation, action).unwrap(),
                        HtmlValuePoolProgress::Continue
                    );
                    break cleanup;
                }
                match action.kind() {
                    HtmlValuePoolActionKind::Read => {
                        operation.read(action).unwrap();
                    }
                    HtmlValuePoolActionKind::Pool => {
                        pool.perform(&mut operation, action).unwrap();
                    }
                }
            };
            assert_eq!(
                Arc::weak_count(&backing),
                1,
                "the completed action actually retained a provisional Weak"
            );
            drop(operation); // The caller refuses Commit, without a callback.
            assert_eq!(
                Arc::weak_count(&backing),
                0,
                "abandonment actually destroyed that provisional occurrence"
            );
            assert_eq!(pool.heap_usage(), before);
            assert!(Arc::ptr_eq(&backing, input.$shared()));
            assert_eq!(input.$shared().capacity(), capacity);
            pool.abort_root(root).unwrap();
            assert!(
                cleanup.record_visits() >= 1,
                "Prepare prepaid {} cleanup visits, but abandoning it released one real Weak; Arc counts here observe lifetime only, never authorize custody",
                cleanup.record_visits()
            );
        }
    };
}

abandoned_preparation_cleanup!(
    abandoned_string_preparation_prepays_its_actual_weak_cleanup,
    HtmlStringPool,
    DocumentString::from_utf16(vec![0xd800, 0, 65]),
    shared_code_units
);
abandoned_preparation_cleanup!(
    abandoned_attribute_preparation_prepays_its_actual_weak_cleanup,
    HtmlAttributePool,
    bank(vec![name(Namespace::none(), None, &[97], &[0xd800, 0])]),
    shared_attributes
);

fn string_before<'a>(
    pool: &mut HtmlStringPool,
    input: &'a DocumentString,
    offer: bool,
    phase: Phase,
) -> (HtmlStringPoolOperation<'a>, HtmlValuePoolAction) {
    let mut operation = if offer {
        pool.offer(input)
    } else {
        pool.probe(input)
    };
    loop {
        let action = pool.next_action(&mut operation).unwrap().unwrap();
        Budget::default().admit(&action).unwrap();
        if action.phase == phase {
            return (operation, action);
        }
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                operation.read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.perform(&mut operation, action).unwrap();
            }
        }
    }
}

#[test]
fn prepared_cleanup_covers_real_page_drop_when_the_following_log_reserve_refuses() {
    let observed = ObserveCleanup::start();
    let mut pool = HtmlStringPool::new();
    pool.inner.refuse_log_reserve = true;
    let root = pool.begin_root().unwrap();
    let input = DocumentString::from("partial allocation");
    let (mut operation, action) = string_before(&mut pool, &input, true, Phase::Prepare);
    let closing = action.layout().closing_work();
    assert_eq!(
        pool.perform(&mut operation, action).unwrap(),
        HtmlValuePoolProgress::Finished
    );
    let cleanup = observed.read();
    assert_eq!(cleanup.slots, 8, "the real initialized page was destroyed");
    assert_eq!(cleanup.entries, 0, "refusal preceded Weak creation");
    assert!(closing.record_visits() >= cleanup.slots);
    assert!(closing.provider_calls() >= 1, "the page backing was freed");
    assert_eq!(closing.copied_bytes(), 0);
    assert_eq!(Arc::weak_count(input.shared_code_units()), 0);
    assert_eq!(pool.heap_usage().bytes(), 0);
    drop(operation);
    pool.abort_root(root).unwrap();
}

#[test]
fn prepared_cleanup_survives_abandonment_staleness_rejection_and_root_close() {
    let observed = ObserveCleanup::start();
    for mode in 0..7 {
        let mut pool = HtmlStringPool::new();
        let mut root = Some(pool.begin_root().unwrap());
        let input = DocumentString::from("prepared");
        let (mut operation, action) = string_before(&mut pool, &input, true, Phase::Prepare);
        let closing = action.layout().closing_work();
        pool.perform(&mut operation, action).unwrap();
        assert_eq!(Arc::weak_count(input.shared_code_units()), 1);
        observed.reset();
        match mode {
            0 => {} // Dropping the caller-owned operation abandons Commit.
            1 => {
                let newer = pool.probe(&input);
                assert!(pool.next_action(&mut operation).unwrap().is_none());
                drop(newer);
            }
            2 => {
                // A changed optional byte cap rejects the already allocated
                // candidate. Its storage must drop before its receipt releases.
                pool.inner.metadata_limit = 0;
                let commit = pool.next_action(&mut operation).unwrap().unwrap();
                Budget::default().admit(&commit).unwrap();
                pool.perform(&mut operation, commit).unwrap();
            }
            3 => {
                pool.abort_root(root.take().unwrap()).unwrap();
                assert!(pool.next_action(&mut operation).unwrap().is_none());
            }
            4 | 5 => {
                let commit = pool.next_action(&mut operation).unwrap().unwrap();
                let mut budget = Budget {
                    cancel: mode == 4,
                    work: 0,
                    ..Budget::default()
                };
                assert_eq!(
                    budget.admit(&commit).unwrap_err(),
                    if mode == 4 {
                        Stop::Cancelled
                    } else {
                        Stop::Work
                    }
                );
            }
            6 => {
                let commit = pool.next_action(&mut operation).unwrap().unwrap();
                assert_eq!(
                    operation.read(commit).unwrap_err(),
                    HtmlValuePoolError::WrongAction
                );
            }
            _ => unreachable!(),
        }
        drop(operation);
        let cleanup = observed.read();
        assert_eq!(cleanup.slots, 8);
        assert_eq!(cleanup.entries, 1);
        assert!(closing.record_visits() >= cleanup.slots + cleanup.entries);
        assert!(closing.provider_calls() >= 3, "Weak/page/log releases");
        assert_eq!(Arc::weak_count(input.shared_code_units()), 0);
        assert_eq!(pool.heap_usage().bytes(), 0);
        if let Some(root) = root {
            pool.abort_root(root).unwrap();
        }
    }
}

#[test]
fn string_upgrade_prepays_last_strong_cleanup_on_abandonment_and_stale_action() {
    for mode in 0..4 {
        let mut pool = HtmlStringPool::new();
        let original = DocumentString::from_utf16(vec![0xd800, 0, 65]);
        let target = Arc::downgrade(original.shared_code_units());
        seed_string(&mut pool, &original);
        let input = DocumentString::from_utf16(vec![0xd800, 0, 65]);
        let (mut operation, action) = string_before(&mut pool, &input, false, Phase::Upgrade);
        let closing = action.layout().closing_work();
        assert_eq!(
            pool.perform(&mut operation, action).unwrap(),
            HtmlValuePoolProgress::CandidateAcquired
        );
        drop(original);
        assert!(
            target.upgrade().is_some(),
            "the operation is the last strong owner"
        );
        match mode {
            0 => {}
            1 => {
                let newer = pool.probe(&input);
                assert!(pool.next_action(&mut operation).unwrap().is_none());
                drop(newer);
            }
            2 => {
                let comparison = pool.next_action(&mut operation).unwrap().unwrap();
                let mut budget = Budget {
                    cancel: true,
                    ..Budget::default()
                };
                assert_eq!(budget.admit(&comparison).unwrap_err(), Stop::Cancelled);
            }
            3 => {
                let comparison = pool.next_action(&mut operation).unwrap().unwrap();
                assert_eq!(
                    pool.perform(&mut operation, comparison).unwrap_err(),
                    HtmlValuePoolError::WrongAction
                );
            }
            _ => unreachable!(),
        }
        drop(operation);
        assert!(target.upgrade().is_none());
        assert!(closing.record_visits() >= 1);
        assert!(
            closing.provider_calls() >= 2,
            "Vec target and UTF-16 backing"
        );
        assert_eq!(closing.copied_bytes(), 0);
        assert_eq!(input.code_units(), &[0xd800, 0, 65]);
    }
}

#[test]
fn preparation_cleanup_work_refuses_one_short_then_retries_the_same_action_exactly() {
    let observed = ObserveCleanup::start();
    let mut pool = HtmlStringPool::new();
    let root = pool.begin_root().unwrap();
    let input = DocumentString::from_utf16(vec![0xd800, 0, 65]);
    let (mut operation, action) = string_before(&mut pool, &input, true, Phase::Prepare);
    // First Prepare initializes eight Slots and reserves 128 Undo records.
    // Immediate work has one visit and three providers (page/log/downgrade).
    // Cleanup visits the eight Slots and one Entry and can release three
    // allocations. Destruction performs no element copies or reads.
    let initialized = 8 * size_of::<Slot<DocumentString>>();
    let reserved = initialized + 128 * size_of::<Undo<DocumentString>>();
    let exact = 1 + initialized + 3 + reserved + 8 + 1 + 3;
    let mut short = Budget {
        work: exact - 1,
        ..Budget::default()
    };
    assert_eq!(short.admit(&action).unwrap_err(), Stop::Work);
    assert_eq!(observed.read().slots, 0);
    assert_eq!(Arc::weak_count(input.shared_code_units()), 0);
    assert_eq!(pool.heap_usage().bytes(), 0);
    let mut admitted = Budget {
        work: exact,
        ..Budget::default()
    };
    admitted.admit(&action).unwrap();
    assert_eq!(admitted.work, 0);
    pool.perform(&mut operation, action).unwrap();
    assert_eq!(Arc::weak_count(input.shared_code_units()), 1);
    drop(operation);
    assert_eq!(observed.read().slots, 8);
    assert_eq!(observed.read().entries, 1);
    assert_eq!(Arc::weak_count(input.shared_code_units()), 0);
    pool.abort_root(root).unwrap();
}

#[test]
fn attribute_upgrade_prepays_last_strong_bank_and_every_nested_string_handle() {
    fn full_bank() -> OrderedAttributes {
        bank(
            (0..64)
                .map(|n| {
                    name(
                        Namespace::from_uri(DocumentString::from(format!("uri:{n}"))),
                        Some(DocumentString::from(format!("p{n}"))),
                        &format!("a{n}").encode_utf16().collect::<Vec<_>>(),
                        &[0xd800, n, 0],
                    )
                })
                .collect(),
        )
    }
    let mut pool = HtmlAttributePool::new();
    let original = full_bank();
    let target = Arc::downgrade(original.shared_attributes());
    let nested: Vec<_> = original
        .iter()
        .flat_map(|attribute| {
            [
                attribute.name().namespace().uri().unwrap(),
                attribute.name().prefix().unwrap(),
                attribute.name().local_name(),
                attribute.value(),
            ]
            .map(|string| Arc::downgrade(string.shared_code_units()))
        })
        .collect();
    assert_eq!(nested.len(), 256);
    seed_attributes(&mut pool, &original);
    let input = full_bank();
    let mut operation = pool.probe(&input);
    let closing = loop {
        let action = pool.next_action(&mut operation).unwrap().unwrap();
        Budget::default().admit(&action).unwrap();
        if action.phase == Phase::Upgrade {
            let closing = action.layout().closing_work();
            assert_eq!(
                pool.perform(&mut operation, action).unwrap(),
                HtmlValuePoolProgress::CandidateAcquired
            );
            break closing;
        }
        match action.kind() {
            HtmlValuePoolActionKind::Read => {
                operation.read(action).unwrap();
            }
            HtmlValuePoolActionKind::Pool => {
                pool.perform(&mut operation, action).unwrap();
            }
        }
    };
    drop(original);
    assert!(target.upgrade().is_some());
    assert!(nested.iter().all(|weak| weak.upgrade().is_some()));
    drop(operation);
    assert!(target.upgrade().is_none());
    assert!(nested.iter().all(|weak| weak.upgrade().is_none()));
    assert!(closing.record_visits() >= 1 + 64 + 256);
    assert!(closing.provider_calls() >= 2 * (1 + 256));
    assert_eq!(closing.copied_bytes(), 0);
    assert_eq!(closing.inspected_bytes(), 0);
    assert_eq!(input.len(), 64);
    assert_eq!(input.as_slice()[63].value().code_units(), &[0xd800, 63, 0]);
}

// A consumer must admit the associated allowance before invoking the real
// constructor. The pool deliberately receives neither the callback nor policy.
fn construct_prepaid<T>(
    work: HtmlValuePoolWork,
    remaining: &mut usize,
    construct: impl FnOnce() -> T,
) -> Result<T, Stop> {
    assert_eq!(work.copied_bytes(), 0);
    assert_eq!(work.inspected_bytes(), 0);
    assert_eq!(work.reserve_bytes(), 0);
    let required = work.record_visits() + work.provider_calls();
    if required > *remaining {
        return Err(Stop::Work);
    }
    *remaining -= required;
    Ok(construct())
}

#[test]
fn disposal_allowance_is_admitted_before_construction_exactly_or_refused_one_short() {
    macro_rules! check {
        ($pool:ty) => {{
            let work = <$pool>::disposal_work();
            let required = work.record_visits() + work.provider_calls();
            let constructed = Cell::new(0);
            let mut short = required - 1;
            assert!(matches!(
                construct_prepaid(work, &mut short, || {
                    constructed.set(constructed.get() + 1);
                    <$pool>::new()
                }),
                Err(Stop::Work)
            ));
            assert_eq!(constructed.get(), 0, "refusal precedes owner existence");
            assert_eq!(short, required - 1);
            let mut exact = required;
            let owner = construct_prepaid(work, &mut exact, || {
                constructed.set(constructed.get() + 1);
                <$pool>::new()
            })
            .unwrap();
            assert_eq!(constructed.get(), 1);
            assert_eq!(exact, 0);
            drop(owner); // No admission callback occurs after construction.
        }};
    }
    check!(HtmlStringPool);
    check!(HtmlAttributePool);
}

#[test]
fn prepaid_disposal_bounds_empty_sealed_active_undo_and_replaced_owner_drops() {
    let observed = ObserveCleanup::start();
    for state in 0..5 {
        // Admit the independent allowance BEFORE either real construction.
        let disposal = HtmlStringPool::disposal_work();
        let mut remaining = disposal.record_visits() + disposal.provider_calls();
        let mut pool = construct_prepaid(disposal, &mut remaining, HtmlStringPool::new).unwrap();
        assert_eq!(remaining, 0);
        pool.inner.collision = true;
        let values: Vec<_> = (0..132)
            .map(|n| DocumentString::from(format!("held:{n}")))
            .collect();
        let mut _authority = None;
        if state != 0 {
            let root = pool.begin_root().unwrap();
            for value in &values[..4] {
                assert!(matches!(
                    strings(&mut pool, value, true, &mut Budget::default()).unwrap(),
                    Outcome::Offered
                ));
            }
            let seal = pool.prepare_seal(root).unwrap();
            pool.seal(seal).unwrap();
        }
        if matches!(state, 2 | 4) {
            _authority = Some(pool.begin_root().unwrap());
            let end = if state == 4 { 132 } else { 8 };
            for value in &values[4..end] {
                assert!(matches!(
                    strings(&mut pool, value, true, &mut Budget::default()).unwrap(),
                    Outcome::Offered
                ));
            }
        }
        let slots: usize = pool.inner.pages.iter().flatten().map(Vec::len).sum();
        let entries: usize = values
            .iter()
            .map(|value| Arc::weak_count(value.shared_code_units()))
            .sum();
        let undo = pool.inner.log.len();
        let pages = pool.inner.pages.iter().flatten().count();
        let backings = pages + usize::from(pool.inner.log.capacity() != 0);
        observed.reset();
        if state == 3 {
            let replacement_disposal = HtmlStringPool::disposal_work();
            let mut remaining =
                replacement_disposal.record_visits() + replacement_disposal.provider_calls();
            let replacement =
                construct_prepaid(replacement_disposal, &mut remaining, HtmlStringPool::new)
                    .unwrap();
            assert_eq!(remaining, 0);
            let old = std::mem::replace(&mut pool, replacement);
            drop(old);
        } else {
            drop(pool);
        }
        let cleanup = observed.read();
        assert_eq!(cleanup.slots, slots);
        assert_eq!(cleanup.entries, entries);
        assert!(disposal.record_visits() >= slots + entries + undo);
        assert!(disposal.provider_calls() >= entries + backings);
        assert_eq!(disposal.copied_bytes(), 0);
        assert_eq!(disposal.inspected_bytes(), 0);
        assert_eq!(disposal.reserve_bytes(), 0);
        assert!(
            values
                .iter()
                .all(|value| Arc::weak_count(value.shared_code_units()) == 0)
        );
        assert_eq!(
            values[131].code_units(),
            "held:131".encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[test]
fn prepaid_attribute_disposal_drops_only_metadata_and_preserves_held_report_like_values() {
    let observed = ObserveCleanup::start();
    for state in 0..5 {
        let disposal = HtmlAttributePool::disposal_work();
        let mut remaining = disposal.record_visits() + disposal.provider_calls();
        let mut pool = construct_prepaid(disposal, &mut remaining, HtmlAttributePool::new).unwrap();
        assert_eq!(remaining, 0);
        pool.inner.collision = true;
        let values: Vec<_> = (0..132)
            .map(|n| bank(vec![name(Namespace::none(), None, &[97], &[n, 0xd800])]))
            .collect();
        let mut _authority = None;
        if state != 0 {
            let root = pool.begin_root().unwrap();
            for value in &values[..4] {
                assert!(matches!(
                    attributes(&mut pool, value, true, &mut Budget::default()).unwrap(),
                    Outcome::Offered
                ));
            }
            let seal = pool.prepare_seal(root).unwrap();
            pool.seal(seal).unwrap();
        }
        if matches!(state, 2 | 4) {
            _authority = Some(pool.begin_root().unwrap());
            let end = if state == 4 { 132 } else { 8 };
            for value in &values[4..end] {
                assert!(matches!(
                    attributes(&mut pool, value, true, &mut Budget::default()).unwrap(),
                    Outcome::Offered
                ));
            }
        }
        let slots: usize = pool.inner.pages.iter().flatten().map(Vec::len).sum();
        let entries: usize = values
            .iter()
            .map(|value| Arc::weak_count(value.shared_attributes()))
            .sum();
        let undo = pool.inner.log.len();
        let backings =
            pool.inner.pages.iter().flatten().count() + usize::from(pool.inner.log.capacity() != 0);
        observed.reset();
        if state == 3 {
            let work = HtmlAttributePool::disposal_work();
            let mut remaining = work.record_visits() + work.provider_calls();
            let replacement =
                construct_prepaid(work, &mut remaining, HtmlAttributePool::new).unwrap();
            assert_eq!(remaining, 0);
            let old = std::mem::replace(&mut pool, replacement);
            drop(old);
        } else {
            drop(pool);
        }
        let cleanup = observed.read();
        assert_eq!(cleanup.slots, slots);
        assert_eq!(cleanup.entries, entries);
        assert!(disposal.record_visits() >= slots + entries + undo);
        assert!(disposal.provider_calls() >= entries + backings);
        assert!(
            values
                .iter()
                .all(|value| Arc::weak_count(value.shared_attributes()) == 0)
        );
        // Destroying weak metadata must leave every immutable bank, its raw
        // surrogate-containing value, and its nested backing readable.
        for (n, value) in values.iter().enumerate() {
            assert_eq!(value.len(), 1);
            assert_eq!(value.as_slice()[0].name().local_name().code_units(), &[97]);
            assert_eq!(
                value.as_slice()[0].value().code_units(),
                &[n as u16, 0xd800]
            );
        }
    }
}
