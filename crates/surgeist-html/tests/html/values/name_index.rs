//! The name index through its public stepwise pieces only, as a consumer that admits every probe
//! as its own action uses it, and the refusals that the types give a misused index.
use surgeist_html::bounded::values::{
    HtmlNameIndex, HtmlNameIndexError, HtmlNameIndexForeignProbe, HtmlNameIndexHasher,
    HtmlNameIndexInsertError, HtmlNameIndexLookup, HtmlNameIndexSlot, HtmlNameIndexSlots,
    HtmlNameIndexStep, HtmlNameIndexStorageError, HtmlNameIndexTable, HtmlNameIndexVacancy,
};

fn empty_storage(slots: HtmlNameIndexSlots) -> Vec<HtmlNameIndexSlot> {
    (0..slots.get())
        .map(|_| HtmlNameIndexSlot::default())
        .collect()
}

/// Why a stepwise lookup stopped without an answer.
#[derive(Debug, PartialEq)]
enum Stop {
    NoEmptySlot,
    Foreign,
    Insert(HtmlNameIndexInsertError),
}

/// `find_or_insert` rebuilt from `probe`, `advance` and `insert` alone, counting the probes that a
/// consumer would admit one by one.
fn stepwise(
    table: &mut HtmlNameIndexTable<'_>,
    hash: u64,
    entry: usize,
    names: &[Vec<u16>],
    name: &[u16],
    probes: &mut usize,
) -> Result<HtmlNameIndexLookup, Stop> {
    let mut probe = table.probe(hash);
    loop {
        *probes += 1;
        match table.advance(&mut probe).map_err(|_| Stop::Foreign)? {
            None => return Err(Stop::NoEmptySlot),
            Some(HtmlNameIndexStep::Occupied(existing)) => {
                if names
                    .get(existing)
                    .is_some_and(|candidate| candidate == name)
                {
                    return Ok(HtmlNameIndexLookup::Found(existing));
                }
            }
            Some(HtmlNameIndexStep::Vacant(vacancy)) => {
                table.insert(vacancy, entry).map_err(Stop::Insert)?;
                return Ok(HtmlNameIndexLookup::Inserted);
            }
        }
    }
}

fn names_of(count: usize) -> Vec<Vec<u16>> {
    (0..count)
        .map(|n| format!("name{n}").encode_utf16().collect())
        .collect()
}

#[test]
fn stepwise_lookups_agree_with_find_or_insert_and_a_set_of_names() {
    let hasher = HtmlNameIndexHasher::new();
    for count in [0_usize, 1, 2, 7, 64, 300] {
        let slots = HtmlNameIndexSlots::for_entries(count);
        let mut stepped_index = HtmlNameIndex::new(slots);
        let mut stepped_slots = empty_storage(slots);
        let mut stepped = HtmlNameIndexTable::new(&mut stepped_index, &mut stepped_slots).unwrap();
        let mut closed_index = HtmlNameIndex::new(slots);
        let mut closed_slots = empty_storage(slots);
        let mut closed = HtmlNameIndexTable::new(&mut closed_index, &mut closed_slots).unwrap();
        let mut committed: Vec<Vec<u16>> = Vec::new();
        let mut probes = 0;
        // Each name twice: the second lookup finds the first, in either implementation.
        for name in names_of(count) {
            for expected in [
                HtmlNameIndexLookup::Inserted,
                HtmlNameIndexLookup::Found(committed.len()),
            ] {
                let entry = committed.len();
                let hash = hasher.hash(&name);
                let by_steps = stepwise(&mut stepped, hash, entry, &committed, &name, &mut probes);
                let by_loop = closed.find_or_insert(
                    hash,
                    entry,
                    &mut (),
                    |_| Ok::<_, ()>(()),
                    |_, existing| Ok(committed.get(existing).is_some_and(|c| *c == name)),
                );
                assert_eq!(by_steps, Ok(expected));
                assert_eq!(by_loop, Ok(expected));
                if expected == HtmlNameIndexLookup::Inserted {
                    committed.push(name.clone());
                }
            }
        }
        assert_eq!(stepped_index.len(), count);
        assert_eq!(closed_index.len(), count);
        // A generous bound on the probes whatever the key: the table is at most half full.
        assert!(
            probes <= 2 * 8 * count.max(1) + 8,
            "{probes} probes for {count} names"
        );
    }
}

#[test]
fn a_full_index_still_finds_its_names_and_refuses_a_new_one_with_a_typed_cause() {
    let hasher = HtmlNameIndexHasher::new();
    let slots = HtmlNameIndexSlots::for_entries(3);
    assert_eq!((slots.get(), slots.capacity()), (8, 3));
    let mut index = HtmlNameIndex::new(slots);
    let mut storage = empty_storage(slots);
    let mut table = HtmlNameIndexTable::new(&mut index, &mut storage).unwrap();
    let names = names_of(4);
    let mut committed: Vec<Vec<u16>> = Vec::new();
    let mut probes = 0;
    for name in &names[..3] {
        let hash = hasher.hash(name);
        let entry = committed.len();
        assert_eq!(
            stepwise(&mut table, hash, entry, &committed, name, &mut probes),
            Ok(HtmlNameIndexLookup::Inserted)
        );
        committed.push(name.clone());
    }
    // The capacity is reached with empty slots left: the fourth name finds a vacancy and is refused.
    assert_eq!(
        stepwise(
            &mut table,
            hasher.hash(&names[3]),
            3,
            &committed,
            &names[3],
            &mut probes
        ),
        Err(Stop::Insert(HtmlNameIndexInsertError::CapacityReached))
    );
    let closed = table.find_or_insert(
        hasher.hash(&names[3]),
        3,
        &mut (),
        |_| Ok::<_, ()>(()),
        |_, existing| Ok(committed.get(existing).is_some_and(|c| *c == names[3])),
    );
    assert_eq!(closed, Err(HtmlNameIndexError::CapacityReached));
    // Every name already in the index is still found, so a trusted answer is never a false none.
    for (entry, name) in committed.iter().enumerate() {
        assert_eq!(
            stepwise(
                &mut table,
                hasher.hash(name),
                99,
                &committed,
                name,
                &mut probes
            ),
            Ok(HtmlNameIndexLookup::Found(entry))
        );
    }
    assert_eq!(index.len(), 3);
}

/// Hole a: a vacancy that another index produced, even one of the same slot count, is refused,
/// so a name cannot be recorded off its own probe sequence.
#[test]
fn a_vacancy_from_another_index_of_the_same_size_is_refused() {
    let slots = HtmlNameIndexSlots::for_entries(4);
    let mut first_index = HtmlNameIndex::new(slots);
    let mut first_storage = empty_storage(slots);
    let first = HtmlNameIndexTable::new(&mut first_index, &mut first_storage).unwrap();
    let mut second_index = HtmlNameIndex::new(slots);
    let mut second_storage = empty_storage(slots);
    let mut second = HtmlNameIndexTable::new(&mut second_index, &mut second_storage).unwrap();
    let mut probe = first.probe(5);
    let Ok(Some(HtmlNameIndexStep::Vacant(foreign))) = first.advance(&mut probe) else {
        panic!("an empty table is vacant at once");
    };
    let foreign: HtmlNameIndexVacancy = foreign;
    assert_eq!(
        second.insert(foreign, 0),
        Err(HtmlNameIndexInsertError::ForeignVacancy)
    );
    assert_eq!(second_index.len(), 0);
    assert!(second_storage.iter().all(HtmlNameIndexSlot::is_empty));
}

/// Hole b: a probe that another index made, with the same or another geometry, is refused, and a
/// probe cannot be made any other way (see the compile-fail example on `HtmlNameIndexProbe`).
#[test]
fn a_probe_from_another_index_is_refused_whatever_its_geometry() {
    let small = HtmlNameIndexSlots::for_entries(1);
    let large = HtmlNameIndexSlots::for_entries(40);
    let mut a_index = HtmlNameIndex::new(small);
    let mut a_storage = empty_storage(small);
    let a = HtmlNameIndexTable::new(&mut a_index, &mut a_storage).unwrap();
    let mut b_index = HtmlNameIndex::new(large);
    let mut b_storage = empty_storage(large);
    let b = HtmlNameIndexTable::new(&mut b_index, &mut b_storage).unwrap();
    let mut c_index = HtmlNameIndex::new(small);
    let mut c_storage = empty_storage(small);
    let c = HtmlNameIndexTable::new(&mut c_index, &mut c_storage).unwrap();
    for (owner, other) in [(&a, &b), (&b, &a), (&a, &c)] {
        let mut probe = other.probe(3);
        assert_eq!(owner.advance(&mut probe), Err(HtmlNameIndexForeignProbe));
    }
    // The probe's own index still takes it.
    let mut probe = a.probe(3);
    assert!(matches!(
        a.advance(&mut probe),
        Ok(Some(HtmlNameIndexStep::Vacant(_)))
    ));
}

/// Hole c: an index cannot count from zero over populated storage, nor guard other storage than
/// the first it was lent, and cannot be copied (see the compile-fail examples on `HtmlNameIndex`).
#[test]
fn an_index_refuses_populated_other_and_wrongly_sized_storage() {
    let hasher = HtmlNameIndexHasher::new();
    let slots = HtmlNameIndexSlots::for_entries(2);
    let mut index = HtmlNameIndex::new(slots);
    let mut storage = empty_storage(slots);
    {
        let mut table = HtmlNameIndexTable::new(&mut index, &mut storage).unwrap();
        let mut committed = vec![];
        let mut probes = 0;
        for name in names_of(2) {
            let entry = committed.len();
            assert_eq!(
                stepwise(
                    &mut table,
                    hasher.hash(&name),
                    entry,
                    &committed,
                    &name,
                    &mut probes
                ),
                Ok(HtmlNameIndexLookup::Inserted)
            );
            committed.push(name);
        }
    }
    // A fresh index lent the populated storage would count from zero: refused.
    let mut fresh = HtmlNameIndex::new(slots);
    let occupied = HtmlNameIndexTable::new(&mut fresh, &mut storage).unwrap_err();
    assert!(matches!(
        occupied,
        HtmlNameIndexStorageError::Occupied { slot } if slot < slots.get()
    ));
    // The used index accepts only its own storage.
    let mut other = empty_storage(slots);
    assert_eq!(
        HtmlNameIndexTable::new(&mut index, &mut other).unwrap_err(),
        HtmlNameIndexStorageError::OtherStorage
    );
    assert!(HtmlNameIndexTable::new(&mut index, &mut storage).is_ok());
    // Storage of another length is refused before any lookup.
    let mut short = vec![HtmlNameIndexSlot::default()];
    let wrong = HtmlNameIndexTable::new(&mut HtmlNameIndex::new(slots), &mut short).unwrap_err();
    assert_eq!(
        wrong,
        HtmlNameIndexStorageError::Length {
            expected: slots.get(),
            actual: 1
        }
    );
    assert!(wrong.to_string().contains("slots"));
}
