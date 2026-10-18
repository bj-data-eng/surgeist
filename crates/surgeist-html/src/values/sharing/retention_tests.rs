//! A recurring live candidate competes with a stream of one-use values.
//! The collision controller selects one bucket; the driver uses public actions.
use super::*;
use crate::{Namespace, QualifiedName};

fn pay(remaining: &mut usize, work: HtmlValuePoolWork) {
    for amount in [
        work.record_visits(),
        work.inspected_bytes(),
        work.copied_bytes(),
        work.provider_calls(),
        work.reserve_bytes(),
    ] {
        *remaining = remaining.checked_sub(amount).expect("finite measured work");
    }
}
macro_rules! recurring {
    ($test:ident, $pool:ty, $operation:ident, $result:ident, $value:ty, $make:expr, $shared:ident) => {
        #[test]
        fn $test() {
            fn run(pool: &mut $pool, value: &$value, offer: bool, remaining: &mut usize) -> Option<$value> {
                let mut operation = Some(if offer { pool.offer(value) } else { pool.probe(value) });
                while let Some(action) = pool.next_action(operation.as_mut().unwrap()).unwrap() {
                    let layout = action.layout();
                    pay(remaining, layout.work());
                    pay(remaining, layout.closing_work());
                    assert!(layout.pool().bytes() <= 256 * 1024);
                    assert!(layout.candidate().bytes() <= 256 * 1024);
                    match action.kind() {
                        HtmlValuePoolActionKind::Read => { operation.as_mut().unwrap().read(action).unwrap(); }
                        HtmlValuePoolActionKind::Pool => { pool.perform(operation.as_mut().unwrap(), action).unwrap(); }
                    }
                }
                match $operation::try_finish(&mut operation).unwrap() {
                    $result::Hit(candidate) => Some(candidate.into_value()),
                    $result::Offered(_) if offer => None,
                    $result::Miss if !offer => None,
                    result => panic!("unexpected optimization outcome: {result:?}"),
                }
            }
            let mut remaining = 1_000_000;
            pay(&mut remaining, <$pool>::disposal_work());
            let mut pool = <$pool>::new();
            pool.inner.collision = true;
            let make = $make;
            let hot = make(500_u16);
            pay(&mut remaining, <$pool>::root_work());
            let root = pool.begin_root().unwrap();
            assert!(run(&mut pool, &hot, true, &mut remaining).is_none());
            pay(&mut remaining, <$pool>::seal_work());
            let seal = pool.prepare_seal(root).unwrap();
            pool.seal(seal).unwrap();
            let baseline = pool.heap_usage();
            pay(&mut remaining, <$pool>::root_work());
            let root = pool.begin_root().unwrap();
            let mut outputs = Vec::new();
            for unique in 0..12_u16 {
                let value = make(unique);
                assert!(run(&mut pool, &value, true, &mut remaining).is_none());
                outputs.push(value); // Report-like external liveness.
                let equal = make(500);
                let selected = run(&mut pool, &equal, false, &mut remaining)
                    .unwrap_or_else(|| panic!("recurring live candidate lost after {} unique insertions despite a hit after each prior insertion", unique + 1));
                assert_eq!(selected, hot);
                assert!(Arc::ptr_eq(selected.$shared(), hot.$shared()));
                assert!(!Arc::ptr_eq(selected.$shared(), equal.$shared()));
                outputs.push(selected);
            }
            pool.abort_root(root).unwrap();
            assert_eq!(pool.heap_usage(), baseline);
            let selected = run(&mut pool, &make(500), false, &mut remaining).unwrap();
            assert!(Arc::ptr_eq(selected.$shared(), hot.$shared()));
            drop(pool);
            assert_eq!(outputs.len(), 24);
            for output in outputs.iter().skip(1).step_by(2) { assert_eq!(output, &hot); }
        }
    };
}
recurring!(
    recurring_string_survives_one_use_collision_pollution,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    |id: u16| DocumentString::from_utf16(vec![0xd800, 0, id]),
    shared_code_units
);
recurring!(
    recurring_attribute_bank_survives_one_use_collision_pollution,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    |id: u16| OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800, 0, id])
    )])
    .unwrap(),
    shared_attributes
);

// Immutable lookups may return retained candidates without gaining authority
// to change the replacement policy.
macro_rules! probe_authority {
    ($test:ident, $pool:ty, $operation:ident, $result:ident, $value:ty, $make:expr, $shared:ident, $mode:expr) => {
        #[test]
        fn $test() {
            fn run(pool: &mut $pool, input: &$value, offer: bool, remaining: &mut usize) -> Option<$value> {
                let mut operation = Some(if offer { pool.offer(input) } else { pool.probe(input) });
                while let Some(action) = pool.next_action(operation.as_mut().unwrap()).unwrap() {
                    let layout = action.layout();
                    pay(remaining, layout.work());
                    pay(remaining, layout.closing_work());
                    assert!(layout.pool().bytes() <= 256 * 1024);
                    assert!(layout.candidate().bytes() <= 256 * 1024);
                    match action.kind() {
                        HtmlValuePoolActionKind::Read => { operation.as_mut().unwrap().read(action).unwrap(); }
                        HtmlValuePoolActionKind::Pool => { pool.perform(operation.as_mut().unwrap(), action).unwrap(); }
                    }
                }
                match $operation::try_finish(&mut operation).unwrap() {
                    $result::Hit(candidate) => Some(candidate.into_value()),
                    $result::Offered(_) if offer => None,
                    $result::Miss if !offer => None,
                    result => panic!("unexpected optimization outcome: {result:?}"),
                }
            }
            let mut remaining = 1_000_000;
            pay(&mut remaining, <$pool>::disposal_work());
            let mut pool = <$pool>::new();
            pool.inner.collision = true;
            let make = $make;
            let original = make(500_u16);
            pay(&mut remaining, <$pool>::root_work());
            let root = pool.begin_root().unwrap();
            assert!(run(&mut pool, &original, true, &mut remaining).is_none());
            pay(&mut remaining, <$pool>::seal_work());
            let seal = pool.prepare_seal(root).unwrap();
            pool.seal(seal).unwrap();
            let baseline = pool.heap_usage();
            let suspended = if $mode == 0 { None } else {
                pay(&mut remaining, <$pool>::root_work());
                Some(pool.begin_root().unwrap())
            };
            let mut marks = Vec::new();
            if $mode == 1 { pool.suspend_mutations(suspended.as_ref().unwrap()).unwrap(); }
            if $mode == 2 {
                for _ in 0..16 {
                    pay(&mut remaining, <$pool>::mark_work());
                    marks.push(pool.mark(suspended.as_ref().unwrap()).unwrap().unwrap());
                }
                pay(&mut remaining, <$pool>::mark_work());
                assert!(pool.mark(suspended.as_ref().unwrap()).unwrap().is_none());
            }
            let equal = make(500);
            let selected = run(&mut pool, &equal, false, &mut remaining).expect("immutable lookup remains usable without mutation authority");
            assert_eq!(selected, original);
            assert!(Arc::ptr_eq(selected.$shared(), original.$shared()));
            assert_eq!(pool.heap_usage(), baseline);
            for mark in marks { pool.accept_mark(mark).unwrap(); }
            if let Some(root) = suspended {
                pay(&mut remaining, <$pool>::seal_work());
                let seal = pool.prepare_seal(root).unwrap();
                pool.seal(seal).unwrap();
            }
            assert_eq!(pool.heap_usage(), baseline);
            pay(&mut remaining, <$pool>::root_work());
            let root = pool.begin_root().unwrap();
            let held: Vec<_> = (0..4_u16).map(make).collect();
            for value in &held { assert!(run(&mut pool, value, true, &mut remaining).is_none()); }
            assert!(run(&mut pool, &equal, false, &mut remaining).is_none(), "an unauthorized immutable probe must not grant the original cold candidate a second chance");
            pool.abort_root(root).unwrap();
            assert_eq!(pool.heap_usage(), baseline);
            drop(pool);
            assert_eq!(selected, original);
            assert_eq!(held.len(), 4);
        }
    };
}
probe_authority!(
    rootless_string_probe_does_not_change_retention_policy,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    |id: u16| DocumentString::from_utf16(vec![0xd800, 0, id]),
    shared_code_units,
    0
);
probe_authority!(
    suspended_string_probe_does_not_change_retention_policy,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    |id: u16| DocumentString::from_utf16(vec![0xd800, 0, id]),
    shared_code_units,
    1
);
probe_authority!(
    unavailable_mark_string_probe_does_not_change_retention_policy,
    HtmlStringPool,
    HtmlStringPoolOperation,
    HtmlStringPoolResult,
    DocumentString,
    |id: u16| DocumentString::from_utf16(vec![0xd800, 0, id]),
    shared_code_units,
    2
);
probe_authority!(
    rootless_attribute_probe_does_not_change_retention_policy,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    |id: u16| OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800, 0, id])
    )])
    .unwrap(),
    shared_attributes,
    0
);
probe_authority!(
    suspended_attribute_probe_does_not_change_retention_policy,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    |id: u16| OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800, 0, id])
    )])
    .unwrap(),
    shared_attributes,
    1
);
probe_authority!(
    unavailable_mark_attribute_probe_does_not_change_retention_policy,
    HtmlAttributePool,
    HtmlAttributePoolOperation,
    HtmlAttributePoolResult,
    OrderedAttributes,
    |id: u16| OrderedAttributes::new(vec![Attribute::new(
        QualifiedName::new(Namespace::none(), None, DocumentString::from("cell")),
        DocumentString::from_utf16(vec![0xd800, 0, id])
    )])
    .unwrap(),
    shared_attributes,
    2
);
