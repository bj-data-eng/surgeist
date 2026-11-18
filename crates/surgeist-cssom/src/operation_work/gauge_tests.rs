//! What a store with a host holds in the `cssom` gauges.
//!
//! The independent figures are the smallest `max_objects`, `max_entries` and
//! `max_string_bytes` under which the same state is admitted without a host:
//! those limits are checked against exactly the figures the gauges hold.

use super::tests_support::*;
use crate::accounting::{CssResource, CssResourceFields, CssWorkTable};
use crate::*;

/// The objects, entries and string bytes the root holds.
fn held(ledger: &CssLedger) -> [u64; 3] {
    let usage = ledger.usage(ledger.root()).unwrap();
    [
        CssResource::RetainedObject,
        CssResource::RetainedEntry,
        CssResource::RetainedStringByte,
    ]
    .map(|dimension| usage.live(dimension))
}

/// The smallest of the three limits under which `admitted` holds.
fn smallest_sizes(admitted: impl Fn(CssomLimits) -> bool) -> [u64; 3] {
    [
        smallest(|max_objects| {
            admitted(CssomLimits {
                max_objects,
                ..CssomLimits::default()
            })
        }),
        smallest(|max_entries| {
            admitted(CssomLimits {
                max_entries,
                ..CssomLimits::default()
            })
        }),
        smallest(|max_string_bytes| {
            admitted(CssomLimits {
                max_string_bytes,
                ..CssomLimits::default()
            })
        }),
    ]
}

#[test]
fn a_store_with_a_host_holds_its_empty_state() {
    let (ledger, host) = open_host();
    let store = make(CssomLimits::default(), Some(host)).unwrap();
    let sizes = smallest_sizes(|limits| make(limits, None).is_ok());
    assert_eq!(held(&ledger), sizes);
    drop(store);
    assert_eq!(held(&ledger), [0, 0, 0]);
}

#[test]
fn a_commit_holds_the_size_of_the_state_it_publishes() {
    let (ledger, host) = open_host();
    let (store, _) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let sizes = smallest_sizes(|limits| with_sheet(limits, None).is_ok());
    assert!(sizes.iter().all(|size| *size > 1));
    assert_eq!(held(&ledger), sizes);
    drop(store);
    assert_eq!(held(&ledger), [0, 0, 0]);
}

#[test]
fn a_commit_that_shrinks_the_state_releases_the_difference() {
    let (ledger, host) = open_host();
    let (mut store, sheet) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let before = held(&ledger);
    let mut edit = store.batch(guard(&store)).unwrap();
    assert!(edit.remove_sheet(&sheet).unwrap());
    store.commit(edit).unwrap();
    let after = held(&ledger);
    let census =
        crate::state_limits::Census::of(&store.state, &CssomLimits::default(), std::iter::empty())
            .unwrap()
            .retained();
    assert_eq!(super::Retained(after), census);
    assert!(after[1] < before[1], "{before:?} -> {after:?}");
}

#[test]
fn a_gauge_cap_refuses_the_commit_and_publishes_nothing() {
    let (ledger, host) = open_host();
    let (probe, _) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    let needed = held(&ledger);
    drop(probe);
    let caps = CssLimits::new(
        CssWorkTable::splat(u64::MAX),
        CssResourceTable::new(CssResourceFields {
            RetainedEntry: needed[1] - 1,
            ..CssResourceFields::splat(u64::MAX)
        }),
    );
    let (ledger, host) = self::host(caps, ProfileMode::Off);
    let mut store = make(CssomLimits::default(), Some(host)).unwrap();
    let empty = held(&ledger);
    let published = store.snapshot();
    let mut edit = store.batch(guard(&store)).unwrap();
    edit.create_parsed_sheet(SOURCE, constructed()).unwrap();
    let Err(CssomError::Accounting(stop)) = store.commit(edit) else {
        panic!("the host refuses the commit's entries");
    };
    let refusal = stop.host_refusal().unwrap();
    assert_eq!(
        refusal.change(),
        CssChange::Acquire(CssResource::RetainedEntry, needed[1] - empty[1])
    );
    assert_eq!(store.snapshot().revision(), published.revision());
    assert_eq!(held(&ledger), empty);
}

#[test]
fn a_prepared_commit_dropped_unpublished_holds_nothing_more() {
    let (ledger, host) = open_host();
    let mut store = make(CssomLimits::default(), Some(host)).unwrap();
    let empty = held(&ledger);
    let mut edit = store.batch(guard(&store)).unwrap();
    edit.create_parsed_sheet(SOURCE, constructed()).unwrap();
    let prepared = store.prepare_commit(edit).unwrap();
    assert!(held(&ledger)[1] > empty[1]);
    drop(prepared);
    assert_eq!(held(&ledger), empty);
}

#[test]
fn two_stores_below_one_host_hold_their_sum() {
    let (ledger, host) = open_host();
    let (first, _) = with_sheet(CssomLimits::default(), Some(host.clone())).unwrap();
    let one = held(&ledger);
    let (second, _) = with_sheet(CssomLimits::default(), Some(host)).unwrap();
    assert_eq!(held(&ledger), one.map(|size| size * 2));
    drop(first);
    assert_eq!(held(&ledger), one);
    drop(second);
    assert_eq!(held(&ledger), [0, 0, 0]);
}
