//! Tests of the interned names: identity, equality across tables, the keys, the growth of the
//! table and its refusals.

use super::*;
use crate::internal::construction::names::PROBE_ALLOWANCE;
use crate::internal::construction::{Builder, Creation, NativeRoot};
use crate::internal::values::element_key::ElementKey;
use crate::vocabulary::{KnownHtmlElement, KnownMathmlElement, KnownSvgElement};
use std::hash::{DefaultHasher, Hash, Hasher};

fn hash_of(name: &QualifiedName) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

fn intern(storage: &mut HtmlConstructionStorage, local: &str) -> NameId {
    ok(storage.intern_name(
        &Namespace::html(),
        None,
        &utf16(local),
        &mut Meter::default(),
    ))
}

/// The logical state of the table: its entries in order and the number of its index entries. The
/// slots are a bank, whose growth is a residue of a refusal that follows it.
fn table_state(storage: &HtmlConstructionStorage) -> (Vec<String>, usize) {
    let names = &storage.arena.names;
    (
        names
            .entries
            .iter()
            .map(|entry| format!("{entry:?}"))
            .collect(),
        names.index.len(),
    )
}

/// A storage whose banks are all full, holding `count` names `n0`, `n1`, and so on, so that the
/// next new name grows the entries, the slots, the entry and the string.
fn full_table(count: usize) -> HtmlConstructionStorage {
    let exact = || Meter {
        exact: true,
        ..Meter::default()
    };
    let mut storage = ok(HtmlConstructionStorage::new(&mut exact()));
    for index in 0..count {
        ok(storage.intern_name(
            &Namespace::html(),
            None,
            &utf16(&format!("n{index}")),
            &mut exact(),
        ));
    }
    storage
}

#[test]
fn a_name_interned_again_is_the_entry_that_was_made_and_grows_nothing() {
    let mut storage = storage();
    let mut first = Meter::default();
    let id = ok(storage.intern_name(&Namespace::html(), None, &utf16("div"), &mut first));
    // The first name grows the vector of entries, the slots, the entry and its copied string.
    let banks: Vec<_> = first.growths.iter().map(|growth| growth.bank()).collect();
    assert_eq!(
        banks,
        [
            HtmlConstructionBank::Names,
            HtmlConstructionBank::NameSlots,
            HtmlConstructionBank::NameEntry,
            HtmlConstructionBank::Text,
        ]
    );
    assert_eq!(storage.arena.names.len(), 1);
    let mut again = Meter::default();
    let repeated = ok(storage.intern_name(&Namespace::html(), None, &utf16("div"), &mut again));
    assert_eq!(repeated, id);
    assert_eq!(storage.arena.names.len(), 1);
    assert!(again.growths.is_empty() && again.records.is_empty());
    // The repeat hashes the local name and compares it with the one entry that shares its hash.
    assert!(again.work[INSPECT] >= 2 * 3 * 2);
    assert_eq!(storage.name(&id).unwrap(), &name("div"));
    // A name interned from a name value is the same entry.
    let element = ok(storage.create_element(
        &name("div"),
        OrderedAttributes::default(),
        None,
        &mut Meter::default(),
    ));
    assert_eq!(storage.arena.names.len(), 1);
    let node = storage.node(&element).unwrap();
    let crate::internal::construction::ConstructionNodeKind::Element(view) = node.kind() else {
        panic!("an element record");
    };
    assert!(std::ptr::eq(view.name(), storage.name(&id).unwrap()));
}

#[test]
fn names_that_differ_in_any_component_are_different_entries() {
    let mut storage = storage();
    let local = utf16("title");
    let mut ids = Vec::new();
    for (namespace, prefix) in [
        (Namespace::html(), None),
        (Namespace::svg(), None),
        (Namespace::mathml(), None),
        (Namespace::none(), None),
        (Namespace::from_uri("urn:x"), None),
        (Namespace::html(), Some(DocumentString::from(""))),
        (Namespace::html(), Some(DocumentString::from("p"))),
        (Namespace::html(), Some(DocumentString::from("q"))),
    ] {
        ids.push(ok(storage.intern_name(
            &namespace,
            prefix.as_ref(),
            &local,
            &mut Meter::default(),
        )));
    }
    for other in ["Title", "titl", "title ", ""] {
        ids.push(intern(&mut storage, other));
    }
    let distinct: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(distinct.len(), ids.len());
    assert_eq!(storage.arena.names.len(), ids.len());
    // An absent prefix and a present empty prefix stay distinct, and each reads back exactly.
    assert_eq!(storage.name(&ids[0]).unwrap().prefix(), None);
    assert_eq!(
        storage.name(&ids[5]).unwrap().prefix(),
        Some(&DocumentString::from(""))
    );
    assert_eq!(storage.name(&ids[3]).unwrap().namespace().uri(), None);
}

#[test]
fn a_name_equals_and_hashes_like_the_same_name_from_another_table_or_made_on_its_own() {
    let (mut first, mut second) = (storage(), storage());
    let one = intern(&mut first, "section");
    let two = intern(&mut second, "section");
    let (one, two) = (first.name(&one).unwrap(), second.name(&two).unwrap());
    let fresh = name("section");
    // Two tables copy the local name into separate allocations.
    assert!(!one.local_name().shares_storage_with(two.local_name()));
    assert_eq!(one, two);
    assert_eq!(one, &fresh);
    assert_eq!(two, &fresh);
    assert_eq!(hash_of(one), hash_of(two));
    assert_eq!(hash_of(one), hash_of(&fresh));
    // The id of one storage is not the id of the other.
    let id = intern(&mut first, "section");
    let foreign = intern(&mut second, "section");
    assert_ne!(id, foreign);
}

#[test]
fn an_id_of_another_storage_is_refused_with_the_foreign_cause() {
    let (mut first, mut second) = (storage(), storage());
    let foreign = intern(&mut second, "div");
    assert_eq!(
        first.name(&foreign).unwrap_err().cause(),
        HtmlConstructionCause::ForeignBuilder
    );
    assert_eq!(
        cause(first.create_element_named(
            &foreign,
            OrderedAttributes::default(),
            None,
            &mut Meter::default(),
        )),
        HtmlConstructionCause::ForeignBuilder
    );
    assert!(first.is_empty());
}

#[test]
fn an_element_created_by_id_takes_the_entry_and_its_key_and_checks_link_flags_by_the_key() {
    let mut storage = storage();
    let template = intern(&mut storage, "template");
    let element = ok(storage.create_element_named(
        &template,
        OrderedAttributes::default(),
        None,
        &mut Meter::default(),
    ));
    // The key says that this is a template, so the contents were made.
    assert_eq!(storage.len(), 2);
    assert!(
        storage
            .node(&element)
            .unwrap()
            .template_contents()
            .is_some()
    );
    // Link flags are for the HTML `link` alone.
    assert_eq!(
        cause(storage.create_element_named(
            &template,
            OrderedAttributes::default(),
            Some(crate::internal::elements::rel::HtmlLinkFlags::fresh()),
            &mut Meter::default(),
        )),
        HtmlConstructionCause::WrongNodeKind
    );
    let svg_link = ok(storage.intern_name(
        &Namespace::svg(),
        None,
        &utf16("link"),
        &mut Meter::default(),
    ));
    assert_eq!(
        cause(storage.create_element_named(
            &svg_link,
            OrderedAttributes::default(),
            Some(crate::internal::elements::rel::HtmlLinkFlags::fresh()),
            &mut Meter::default(),
        )),
        HtmlConstructionCause::WrongNodeKind
    );
    let link = intern(&mut storage, "link");
    ok(storage.create_element_named(
        &link,
        OrderedAttributes::default(),
        Some(crate::internal::elements::rel::HtmlLinkFlags::fresh()),
        &mut Meter::default(),
    ));
}

/// The key of the element that the storage makes of the name of `namespace` and `local`.
fn key_of(
    storage: &mut HtmlConstructionStorage,
    namespace: Namespace,
    local: &[u16],
) -> ElementKey {
    let id = ok(storage.intern_name(&namespace, None, local, &mut Meter::default()));
    let element = ok(storage.create_element_named(
        &id,
        OrderedAttributes::default(),
        None,
        &mut Meter::default(),
    ));
    let node = storage.node(&element).unwrap();
    let crate::internal::construction::ConstructionNodeKind::Element(view) = node.kind() else {
        panic!("an element record");
    };
    view.key()
}

#[test]
fn the_key_of_an_entry_is_the_vocabulary_key_of_every_known_name() {
    let mut storage = storage();
    for &known in KnownHtmlElement::ALL {
        let key = key_of(&mut storage, Namespace::html(), &utf16(known.local_name()));
        assert_eq!(key.html(), Some(known), "{}", known.local_name());
        assert_eq!(key, ElementKey::from(known));
    }
    for &known in KnownMathmlElement::ALL {
        let key = key_of(
            &mut storage,
            Namespace::mathml(),
            &utf16(known.local_name()),
        );
        assert_eq!(key.mathml(), Some(known), "{}", known.local_name());
        assert_eq!(key, ElementKey::from(known));
    }
    for &known in KnownSvgElement::ALL {
        let key = key_of(&mut storage, Namespace::svg(), &utf16(known.local_name()));
        assert_eq!(key.svg(), Some(known), "{}", known.local_name());
        assert_eq!(key, ElementKey::from(known));
    }
}

#[test]
fn the_key_of_an_unknown_or_foreign_name_is_the_unknown_key_of_its_namespace() {
    let mut storage = storage();
    let units: [Vec<u16>; 7] = [
        utf16("x-unknown"),
        utf16("DIV"),
        utf16("div "),
        utf16(""),
        utf16("\u{e9}l\u{e9}ment"),
        vec![0xd800],
        utf16("foreignObject"),
    ];
    for unit in &units {
        for namespace in [
            Namespace::html(),
            Namespace::mathml(),
            Namespace::svg(),
            Namespace::none(),
            Namespace::from_uri("urn:x"),
        ] {
            let uri: Option<Vec<u16>> = namespace.uri().map(|uri| uri.code_units().to_vec());
            let key = key_of(&mut storage, namespace.clone(), unit);
            assert_eq!(
                key,
                ElementKey::from_units(uri.as_deref(), unit),
                "{unit:?} in {uri:?}"
            );
            // Of these names only `foreignObject` is known, and only to the SVG vocabulary.
            assert_eq!(
                key.is_known(),
                unit == &utf16("foreignObject")
                    && matches!(key.namespace(), crate::vocabulary::ElementNamespace::Svg)
            );
        }
    }
    // A HTML name outside HTML is not the HTML element.
    let title = key_of(&mut storage, Namespace::none(), &utf16("title"));
    assert_eq!(title.html(), None);
    assert!(!title.is_known());
    // The classes of a known foreign name are in its key.
    let mi = key_of(&mut storage, Namespace::mathml(), &utf16("mi"));
    assert!(mi.is_mathml_text_integration_point());
    let desc = key_of(&mut storage, Namespace::svg(), &utf16("desc"));
    assert!(desc.is_svg_html_integration_point());
}

#[test]
fn a_table_that_grows_through_many_names_finds_each_of_them_again() {
    let mut storage = storage();
    let mut ids = Vec::new();
    for index in 0..300 {
        ids.push(intern(&mut storage, &format!("name-{index}")));
        // Every name interned so far is still found, and nothing grows for it.
        if index % 37 == 0 {
            for (earlier, id) in ids.iter().enumerate() {
                let mut meter = Meter::default();
                let found = ok(storage.intern_name(
                    &Namespace::html(),
                    None,
                    &utf16(&format!("name-{earlier}")),
                    &mut meter,
                ));
                assert_eq!(&found, id);
                assert!(meter.growths.is_empty());
            }
        }
    }
    assert_eq!(storage.arena.names.len(), 300);
    let distinct: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(distinct.len(), 300);
    let names = &storage.arena.names;
    assert_eq!(names.index.len(), 300);
    assert!(names.slots.len().is_power_of_two());
    // The index holds its entries at most half full.
    assert!(2 * names.entries.len() <= names.slots.len());
    for (index, id) in ids.iter().enumerate() {
        assert_eq!(
            storage.name(id).unwrap().local_name().code_units(),
            utf16(&format!("name-{index}"))
        );
    }
}

#[test]
fn a_new_name_refused_at_each_admission_point_leaves_the_table_as_it_was() {
    let before = table_state(&full_table(8));
    let mut calls = None;
    for index in 0..10_000 {
        let mut storage = full_table(8);
        let usage = storage.usage();
        let mut meter = Meter::refusing_at(index);
        match storage.intern_name(&Namespace::html(), None, &utf16("fresh"), &mut meter) {
            Err(HtmlChargedError::Stopped(Refused)) => {
                assert_eq!(
                    table_state(&storage),
                    before,
                    "a refusal at the {index}th call ({:?}) changed the table",
                    meter.calls.last()
                );
                assert_eq!(storage.usage(), usage);
                // The table serves the names it held and takes the one that was refused.
                for number in 0..8 {
                    let mut again = Meter::default();
                    ok(storage.intern_name(
                        &Namespace::html(),
                        None,
                        &utf16(&format!("n{number}")),
                        &mut again,
                    ));
                    assert!(again.growths.is_empty());
                }
                intern(&mut storage, "fresh");
                assert_eq!(storage.arena.names.len(), 9);
            }
            Err(HtmlChargedError::Faulted(error)) => panic!("faulted at {index}: {error:?}"),
            Ok(_) => {
                assert!(index > 0);
                calls = Some(meter.calls);
                break;
            }
        }
    }
    let calls = calls.expect("the intern completed");
    // Four growths, each admitted before the first is recorded and recorded before the next.
    assert_eq!(
        calls.iter().filter(|call| **call == "admit_growth").count(),
        4
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| **call == "record_growth")
            .count(),
        4
    );
    let last_admission = calls.iter().rposition(|call| *call == "admit_growth");
    let first_record = calls.iter().position(|call| *call == "record_growth");
    assert!(last_admission < first_record);
}

#[test]
fn an_allocator_that_refuses_the_spare_room_of_the_name_banks_is_asked_for_the_room_needed() {
    for bank in [HtmlConstructionBank::Names, HtmlConstructionBank::NameSlots] {
        let mut storage = full_table(8);
        let mut meter = Meter {
            huge_bank: Some(bank),
            ..Meter::default()
        };
        let id = ok(storage.intern_name(&Namespace::html(), None, &utf16("fresh"), &mut meter));
        // The retry for the room needed is a second request to the allocator.
        assert_eq!(meter.work[PROVIDER], 5);
        assert_eq!(storage.arena.names.len(), 9);
        assert_eq!(storage.name(&id).unwrap(), &name("fresh"));
        for number in 0..8 {
            let found = intern(&mut storage, &format!("n{number}"));
            assert_eq!(storage.name(&found).unwrap(), &name(&format!("n{number}")));
        }
        assert_eq!(storage.arena.names.len(), 9);
    }
}

#[test]
fn a_creation_refused_after_its_name_was_interned_leaves_the_name_and_no_record() {
    let mut interned = 0;
    for index in 0..10_000 {
        let mut storage = storage();
        let mut meter = Meter::refusing_at(index);
        let outcome = storage.create_element(
            &name("figure"),
            OrderedAttributes::default(),
            None,
            &mut meter,
        );
        match outcome {
            Err(HtmlChargedError::Stopped(Refused)) => {
                assert!(storage.is_empty(), "a record after a refusal at {index}");
                assert_eq!(storage.usage(), HtmlConstructionUsage::default());
                // The name that was interned stays, and is the one entry of that name.
                assert!(storage.arena.names.len() <= 1);
                interned += storage.arena.names.len();
                let id = intern(&mut storage, "figure");
                assert_eq!(storage.arena.names.len(), 1);
                assert_eq!(storage.name(&id).unwrap(), &name("figure"));
            }
            Err(HtmlChargedError::Faulted(error)) => panic!("faulted at {index}: {error:?}"),
            Ok(_) => break,
        }
    }
    assert!(
        interned > 0,
        "no refusal followed the interning of the name"
    );
}

#[test]
fn elements_of_one_name_in_a_published_result_borrow_one_name() {
    let mut builder = Builder::new(
        HtmlConstructionLimits::new(16, 16, 0, 256, 4),
        NativeRoot::Fragment,
    )
    .unwrap();
    let root = builder.root;
    let mut ids = Vec::new();
    for local in ["p", "p", "em", "p"] {
        let node = builder
            .append(
                &root,
                Creation::Element {
                    name: name(local),
                    attributes: OrderedAttributes::default(),
                    link_flags: None,
                },
            )
            .unwrap();
        ids.push(node);
    }
    let result = builder.finish().unwrap();
    let mut views = Vec::new();
    for handle in &ids {
        let position = result.position(handle).unwrap();
        let node = result.node_at(position).unwrap().as_node().unwrap();
        let crate::internal::construction::HtmlNodeKind::Element(view) = node.kind() else {
            panic!("an element");
        };
        views.push(view);
    }
    assert!(std::ptr::eq(views[0].name(), views[1].name()));
    assert!(std::ptr::eq(views[0].name(), views[3].name()));
    assert!(!std::ptr::eq(views[0].name(), views[2].name()));
    assert_eq!(views[0].key().html(), Some(KnownHtmlElement::P));
    assert_eq!(views[2].key().html(), Some(KnownHtmlElement::Em));
    assert_eq!(views[0].name(), &name("p"));
}

/// The allowance covers every chain of a table of at most seven entries.
const _: () = assert!(PROBE_ALLOWANCE >= 8);

/// The visits a lookup is charged before any step beyond the allowance: one with the hash, plus
/// the allowance.
const LOOKUP_VISITS: u64 = 1 + PROBE_ALLOWANCE as u64;

#[test]
fn lookups_whose_chains_stay_within_the_allowance_charge_the_allowance_and_nothing_else() {
    // Seven entries fill at most seven slots of the 16 the first growth makes, so a probe
    // examines at most eight slots, whatever the hash key: no chain exceeds the allowance.
    let mut storage = storage();
    for index in 0..7 {
        intern(&mut storage, &format!("n{index}"));
    }
    let mut meter = Meter::default();
    let lookups = 7 * 5;
    for _ in 0..5 {
        for index in 0..7 {
            ok(storage.intern_name(
                &Namespace::html(),
                None,
                &utf16(&format!("n{index}")),
                &mut meter,
            ));
        }
    }
    assert_eq!(meter.work[VISIT], lookups * LOOKUP_VISITS);
    // The eighth name is a miss that takes a free slot without growing the index.
    let mut miss = Meter::default();
    intern_with(&mut storage, "n7", &mut miss);
    assert!(miss.growths.iter().all(|growth| matches!(
        growth.bank(),
        HtmlConstructionBank::Names | HtmlConstructionBank::NameEntry | HtmlConstructionBank::Text
    )));
    assert_eq!(miss.work[VISIT], LOOKUP_VISITS);
}

fn intern_with(storage: &mut HtmlConstructionStorage, local: &str, meter: &mut Meter) -> NameId {
    ok(storage.intern_name(&Namespace::html(), None, &utf16(local), meter))
}

/// `count` names whose hashes select one slot of an index of `slots` slots, found under the
/// process's hash key. Interned in order, they sit in one chain, the `k`th at step `k`.
fn colliding_names(storage: &HtmlConstructionStorage, slots: usize, count: usize) -> Vec<Vec<u16>> {
    let mask = slots - 1;
    let mut chain: Vec<Vec<u16>> = Vec::new();
    let mut start = None;
    for number in 0.. {
        let units = utf16(&format!("c{number}"));
        let slot = storage.arena.names.hash(&units) as usize & mask;
        if *start.get_or_insert(slot) == slot {
            chain.push(units);
        }
        if chain.len() == count {
            break;
        }
    }
    chain
}

#[test]
fn a_lookup_past_the_allowance_is_charged_each_step_beyond_it() {
    let mut storage = storage();
    // Sixteen names that start at one slot of the 32-slot index sit in one chain of 16 slots, in
    // the order they were interned, so the name interned `k`th is found at step `k`.
    let chain = colliding_names(&storage, 32, 16);
    for units in &chain {
        ok(storage.intern_name(&Namespace::html(), None, units, &mut Meter::default()));
    }
    assert_eq!(storage.arena.names.slots.len(), 32);
    for (found_at, units) in chain
        .iter()
        .enumerate()
        .map(|(index, units)| (index + 1, units))
    {
        let mut meter = Meter::default();
        ok(storage.intern_name(&Namespace::html(), None, units, &mut meter));
        let beyond = found_at.saturating_sub(PROBE_ALLOWANCE) as u64;
        assert_eq!(
            meter.work[VISIT],
            LOOKUP_VISITS + beyond,
            "the name interned {found_at}th"
        );
    }
}

#[test]
fn a_rebuild_past_the_allowance_charges_each_step_beyond_it_and_a_refusal_leaves_the_table() {
    // Nine names that start at one slot, of both the 16-slot index and the 32-slot index that
    // the ninth makes. The first eight fill a chain of eight; the ninth is a miss that examines
    // those eight and the empty slot, step 9, so its lookup is charged one visit beyond the
    // allowance. It grows the index: the nine entries are placed, each charged the allowance, and
    // the ninth, placed ninth in the chain, one visit beyond it.
    let chain = colliding_names(&storage(), 32, 9);
    let setup = || {
        let mut storage = storage();
        for units in &chain[..8] {
            ok(storage.intern_name(&Namespace::html(), None, units, &mut Meter::default()));
        }
        assert_eq!(storage.arena.names.slots.len(), 16);
        storage
    };
    let allowance = PROBE_ALLOWANCE as u64;
    let expected = LOOKUP_VISITS + 1 + 9 * allowance + 1;
    let mut storage = setup();
    let mut meter = Meter::default();
    ok(storage.intern_name(&Namespace::html(), None, &chain[8], &mut meter));
    assert_eq!(storage.arena.names.slots.len(), 32);
    assert_eq!(meter.work[VISIT], expected);
    // Refused at each call of the operation, the table, the usage and the records are as they
    // were, and the name is interned afterwards.
    let calls = meter.calls.len();
    for index in 0..calls {
        let mut storage = setup();
        let before = (table_state(&storage), storage.usage());
        let mut meter = Meter::refusing_at(index);
        match storage.intern_name(&Namespace::html(), None, &chain[8], &mut meter) {
            Err(HtmlChargedError::Stopped(Refused)) => {}
            other => panic!("not refused at call {index}: {other:?}"),
        }
        assert_eq!(
            (table_state(&storage), storage.usage()),
            before,
            "call {index}"
        );
        ok(storage.intern_name(&Namespace::html(), None, &chain[8], &mut Meter::default()));
        assert_eq!(storage.arena.names.len(), 9);
    }
}

#[test]
fn growing_the_index_charges_the_allowance_of_each_placement() {
    let mut storage = storage();
    for index in 0..8 {
        intern(&mut storage, &format!("n{index}"));
    }
    // The ninth name misses and grows the index: eight entries and the new one are placed.
    let mut meter = Meter::default();
    intern_with(&mut storage, "n8", &mut meter);
    assert!(meter.work[VISIT] >= LOOKUP_VISITS + 9 * PROBE_ALLOWANCE as u64);
}

fn link_flags() -> Option<crate::internal::elements::rel::HtmlLinkFlags> {
    Some(crate::internal::elements::rel::HtmlLinkFlags::fresh())
}

#[test]
fn link_flags_on_another_element_are_refused_after_its_name_is_interned_and_charged() {
    let mut first = storage();
    let mut meter = Meter::default();
    let outcome = first.create_element(
        &name("figure"),
        OrderedAttributes::default(),
        link_flags(),
        &mut meter,
    );
    assert_eq!(cause(outcome), HtmlConstructionCause::WrongNodeKind);
    let charged = meter.calls.len();
    // The name was interned first: the growths of the entries, the slots and the entry, which
    // adopts the caller's name without a copy, were admitted and recorded before the refusal.
    let count = |call: &str| meter.calls.iter().filter(|name| **name == call).count();
    assert_eq!(count("admit_growth"), 3);
    assert_eq!(count("record_growth"), 3);
    assert_eq!(
        meter
            .growths
            .iter()
            .map(|growth| growth.bank())
            .collect::<Vec<_>>(),
        [
            HtmlConstructionBank::Names,
            HtmlConstructionBank::NameSlots,
            HtmlConstructionBank::NameEntry,
        ]
    );
    // A limit that stops anywhere in the charged prefix is the limit stop, and the call after it
    // is the refusal of the flags.
    for index in 0..charged {
        let mut storage = storage();
        let mut meter = Meter::refusing_at(index);
        let outcome = storage.create_element(
            &name("figure"),
            OrderedAttributes::default(),
            link_flags(),
            &mut meter,
        );
        assert!(
            matches!(outcome, Err(HtmlChargedError::Stopped(Refused))),
            "call {index}: {outcome:?}"
        );
        assert!(storage.is_empty());
    }
    let mut storage = storage();
    let mut meter = Meter::refusing_at(charged);
    let outcome = storage.create_element(
        &name("figure"),
        OrderedAttributes::default(),
        link_flags(),
        &mut meter,
    );
    assert_eq!(cause(outcome), HtmlConstructionCause::WrongNodeKind);
    assert!(storage.is_empty());
}

#[test]
fn link_flags_on_another_element_created_by_id_are_refused_after_the_id_is_charged() {
    let mut charged = None;
    for index in 0..100 {
        let mut storage = storage();
        let id = intern(&mut storage, "figure");
        let mut meter = Meter::refusing_at(index);
        let outcome = storage.create_element_named(
            &id,
            OrderedAttributes::default(),
            link_flags(),
            &mut meter,
        );
        match outcome {
            Err(HtmlChargedError::Stopped(Refused)) => {
                assert!(storage.is_empty(), "call {index}");
            }
            other => {
                assert_eq!(cause(other), HtmlConstructionCause::WrongNodeKind);
                charged = Some(index);
                break;
            }
        }
    }
    // The id is read after one charged visit: the call after it is the refusal of the flags.
    assert_eq!(charged, Some(1));
}
