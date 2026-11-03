//! `Control::reserve`, `string` and `boxed` charge the host through the parser
//! crate's `limits::grow`, `copy_str` and `copy_boxed`. These tests state what
//! the host observes of each: the result, the storage and the failure latched,
//! including every way a refused admission or an impossible request ends.

use super::*;
use std::cell::{Cell, RefCell};

/// What the host saw: an admission (work or a live extent) or a release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Work,
    Live(usize, usize),
    Released(usize, usize),
}

#[derive(Debug, Eq, PartialEq)]
struct Observed<R> {
    events: Vec<Event>,
    result: R,
    failure: Option<Failure>,
}

/// Runs `operation` on a control with no work or live-byte limit whose host
/// refuses the admission with index `refuse_at` (`usize::MAX` refuses none).
fn observe<R>(refuse_at: usize, operation: impl Fn(&Control<'_>) -> R) -> Observed<R> {
    let events = RefCell::new(Vec::new());
    let admissions = Cell::new(0_usize);
    let admit = |action: CssUtf16ValueAdmission| {
        let index = admissions.get();
        admissions.set(index + 1);
        events.borrow_mut().push(match action {
            CssUtf16ValueAdmission::Work => Event::Work,
            CssUtf16ValueAdmission::Live(usage) => Event::Live(usage.bytes, usage.utf16_units),
        });
        if index == refuse_at { Err(()) } else { Ok(()) }
    };
    let release = |usage: CssUtf16ValueStorage| {
        events
            .borrow_mut()
            .push(Event::Released(usage.bytes, usage.utf16_units));
    };
    let (result, failure) = {
        let limits = CssUtf16PropertyValueLimits {
            max_work: usize::MAX,
            max_live_bytes: usize::MAX,
            ..CssUtf16PropertyValueLimits::default()
        };
        let control = Control::new(limits, &admit, &release);
        let result = operation(&control);
        (result, control.failure())
    };
    Observed {
        events: events.into_inner(),
        result,
        failure,
    }
}

/// An element that occupies no storage.
struct Zero;

impl From<u8> for Zero {
    fn from(_: u8) -> Self {
        Self
    }
}

/// A vector with `len` elements and room for at least `capacity`.
fn vector<T: From<u8>>(len: usize, capacity: usize) -> Vec<T> {
    let mut values = Vec::with_capacity(capacity);
    values.extend((0..len).map(|index| T::from(u8::try_from(index).unwrap())));
    values
}

/// Reserves `additional` elements in a vector of `len` elements and room for
/// `capacity`, and returns the result with the capacity afterwards.
fn grown<T: From<u8>>(
    len: usize,
    capacity: usize,
    additional: usize,
) -> impl Fn(&Control<'_>) -> (Result<(), ()>, usize) {
    move |control| {
        let mut values = vector::<T>(len, capacity);
        // The host charged the bank the vector already owns, which growth returns.
        let _ = control.add(values.capacity() * size_of::<T>());
        let result = control.reserve(&mut values, additional);
        (result, values.capacity())
    }
}

#[test]
fn growth_makes_room_and_latches_no_failure_when_the_host_admits_everything() {
    let cases: [(usize, usize, usize); 6] = [
        (0, 0, 1),  // an empty vector gets the minimum bank
        (3, 3, 1),  // a full vector doubles
        (3, 3, 10), // the need exceeds the doubling
        (2, 8, 1),  // there is already room: only the opening step
        (0, 0, 0),  // nothing is needed
        (5, 5, 3),  // moving several elements
    ];
    for (len, capacity, additional) in cases {
        let observed = observe(usize::MAX, grown::<u64>(len, capacity, additional));
        assert_eq!(observed.result.0, Ok(()), "{len} {capacity} {additional}");
        assert_eq!(observed.failure, None, "{len} {capacity} {additional}");
        assert!(
            observed.result.1 >= len + additional,
            "{len} {capacity} {additional}"
        );
        assert!(
            observed.events.contains(&Event::Work),
            "{:?}",
            observed.events
        );
    }
    // A zero-sized element has no bank to move.
    let observed = observe(usize::MAX, grown::<Zero>(0, 0, 3));
    assert_eq!(observed.result.0, Ok(()));
    assert_eq!(observed.failure, None);
}

#[test]
fn growth_that_cannot_be_represented_or_allocated_latches_its_failure() {
    // The length plus the addition overflows.
    let overflow = observe(usize::MAX, grown::<u64>(3, 3, usize::MAX));
    assert_eq!(overflow.result.0, Err(()));
    assert_eq!(overflow.failure, Some(Failure::CapacityOverflow));
    // The planned bank in bytes overflows.
    let planned = observe(usize::MAX, grown::<u64>(0, 0, usize::MAX / 4));
    assert_eq!(planned.result.0, Err(()));
    assert_eq!(planned.failure, Some(Failure::CapacityOverflow));
    // A bank of isize::MAX bytes passes the arithmetic and the allocator's
    // capacity check refuses it without allocating.
    let allocation = observe(usize::MAX, grown::<u8>(0, 0, isize::MAX as usize));
    assert_eq!(allocation.result.0, Err(()));
    assert_eq!(allocation.failure, Some(Failure::Allocation));
}

#[test]
fn growth_refused_at_any_admission_latches_the_refusal() {
    let free = observe(usize::MAX, grown::<u64>(3, 3, 1));
    let admissions = free
        .events
        .iter()
        .filter(|event| !matches!(event, Event::Released(..)))
        .count();
    assert!(admissions >= 2, "{:?}", free.events);
    // A refusal at any admission, the charge for the bank the vector already
    // owns included, latches the refusal and fails the growth.
    for refuse_at in 0..admissions {
        let observed = observe(refuse_at, grown::<u64>(3, 3, 1));
        assert_eq!(observed.result.0, Err(()), "refusing admission {refuse_at}");
        assert_eq!(
            observed.failure,
            Some(Failure::Admission),
            "refusing admission {refuse_at}"
        );
    }
}

#[test]
fn string_copies_hold_exactly_their_text() {
    for value in ["", "a", "abc", "é😀 mixed"] {
        let observed = observe(usize::MAX, |control| control.string(value));
        assert_eq!(observed.result.as_deref(), Ok(value));
        assert_eq!(observed.failure, None);
    }
}

#[test]
fn boxed_copies_hold_exactly_their_text() {
    for value in ["", "a", "abc", "é😀 mixed"] {
        let observed = observe(usize::MAX, |control| control.boxed(value));
        assert_eq!(observed.result.as_deref(), Ok(value));
        assert_eq!(observed.failure, None);
    }
}

#[test]
fn a_refused_copy_latches_the_refusal_and_returns_no_text() {
    let observed = observe(2, |control| control.string("abc"));
    assert_eq!(observed.result, Err(()));
    assert_eq!(observed.failure, Some(Failure::Admission));
}
