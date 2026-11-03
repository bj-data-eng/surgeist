//! The name index through its public stepwise pieces only, as a consumer that admits every probe
//! as its own action uses it.
use surgeist_html::bounded::values::{
    HtmlNameIndex, HtmlNameIndexError, HtmlNameIndexHasher, HtmlNameIndexInsertError,
    HtmlNameIndexLookup, HtmlNameIndexSlot, HtmlNameIndexSlots, HtmlNameIndexStep,
    HtmlNameIndexTable,
};

/// Why a stepwise lookup stopped without an answer.
#[derive(Debug, PartialEq)]
enum Stop {
    NoEmptySlot,
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
        match table.advance(&mut probe) {
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
        let mut stepped_slots = vec![HtmlNameIndexSlot::default(); slots.get()];
        let mut stepped = HtmlNameIndexTable::new(&mut stepped_index, &mut stepped_slots).unwrap();
        let mut closed_index = HtmlNameIndex::new(slots);
        let mut closed_slots = vec![HtmlNameIndexSlot::default(); slots.get()];
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
        // Two probes per name on average at most, whatever the key: the table is half full.
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
    let mut storage = vec![HtmlNameIndexSlot::default(); slots.get()];
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

#[test]
fn storage_of_another_length_is_refused_before_any_lookup() {
    let slots = HtmlNameIndexSlots::for_entries(5);
    let mut index = HtmlNameIndex::new(slots);
    let mut storage = vec![HtmlNameIndexSlot::default(); slots.get() + 1];
    let error = HtmlNameIndexTable::new(&mut index, &mut storage).unwrap_err();
    assert_eq!(
        (error.expected(), error.actual()),
        (slots.get(), slots.get() + 1)
    );
    assert!(error.to_string().contains("slots"));
}
