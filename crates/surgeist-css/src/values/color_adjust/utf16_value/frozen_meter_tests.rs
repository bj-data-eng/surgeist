//! `Control::reserve`, `string` and `boxed` now delegate to the parser crate's
//! `limits::grow`, `copy_str` and `copy_boxed`. These tests keep a frozen copy
//! of the bodies those methods had before the move and assert that both charge
//! the host identically: the same admissions and releases in the same order,
//! the same result and storage, and the same failure latched, for every
//! refusal point and limit. Step 18 of the parser extraction removes this
//! module together with the other comparison scaffolding.

use super::*;
use std::cell::{Cell, RefCell};

/// What the host saw: an admission (work or a live extent) or a release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Work,
    Live(usize, usize),
    Released(usize, usize),
}

/// How a run is constrained. `refuse_at` is the index of the admission the
/// host's callback refuses; the limits are those of the control itself.
#[derive(Clone, Copy, Debug)]
struct Config {
    refuse_at: usize,
    max_work: usize,
    max_live: usize,
}

impl Config {
    const FREE: Self = Self {
        refuse_at: usize::MAX,
        max_work: usize::MAX,
        max_live: usize::MAX,
    };
}

#[derive(Debug, Eq, PartialEq)]
struct Observed<R> {
    events: Vec<Event>,
    result: R,
    failure: Option<Failure>,
    live: CssUtf16ValueStorage,
}

fn observe<R>(config: Config, operation: impl Fn(&Control<'_>) -> R) -> Observed<R> {
    let events = RefCell::new(Vec::new());
    let admissions = Cell::new(0_usize);
    let admit = |action: CssUtf16ValueAdmission| {
        let index = admissions.get();
        admissions.set(index + 1);
        events.borrow_mut().push(match action {
            CssUtf16ValueAdmission::Work => Event::Work,
            CssUtf16ValueAdmission::Live(usage) => Event::Live(usage.bytes, usage.utf16_units),
        });
        if index == config.refuse_at {
            Err(())
        } else {
            Ok(())
        }
    };
    let release = |usage: CssUtf16ValueStorage| {
        events
            .borrow_mut()
            .push(Event::Released(usage.bytes, usage.utf16_units));
    };
    let limits = CssUtf16PropertyValueLimits {
        max_work: config.max_work,
        max_live_bytes: config.max_live,
        ..CssUtf16PropertyValueLimits::default()
    };
    let (result, failure, live) = {
        let control = Control::new(limits, &admit, &release);
        let result = operation(&control);
        (result, control.failure(), control.live())
    };
    Observed {
        events: events.into_inner(),
        result,
        failure,
        live,
    }
}

/// Runs both bodies under no constraint, then refuses each admission in turn,
/// limits the work to each count below the total and the live bytes to one
/// under each extent the unconstrained run admitted, and asserts equal
/// observations every time. Returns the unconstrained observation.
fn assert_same_trace<R: std::fmt::Debug + PartialEq>(
    frozen: impl Fn(&Control<'_>) -> R,
    shared: impl Fn(&Control<'_>) -> R,
) -> Observed<R> {
    let free = observe(Config::FREE, &frozen);
    assert_eq!(free, observe(Config::FREE, &shared), "unconstrained");
    let admissions = free
        .events
        .iter()
        .filter(|event| !matches!(event, Event::Released(..)))
        .count();
    let works = free
        .events
        .iter()
        .filter(|event| **event == Event::Work)
        .count();
    for refuse_at in 0..admissions {
        let config = Config {
            refuse_at,
            ..Config::FREE
        };
        assert_eq!(
            observe(config, &frozen),
            observe(config, &shared),
            "refusing admission {refuse_at}"
        );
    }
    for max_work in 0..works {
        let config = Config {
            max_work,
            ..Config::FREE
        };
        assert_eq!(
            observe(config, &frozen),
            observe(config, &shared),
            "work limited to {max_work}"
        );
    }
    for event in &free.events {
        if let Event::Live(bytes, _) = event {
            let config = Config {
                max_live: bytes.saturating_sub(1),
                ..Config::FREE
            };
            assert_eq!(
                observe(config, &frozen),
                observe(config, &shared),
                "live bytes limited to one under {bytes}"
            );
        }
    }
    free
}

/// An element that occupies no storage.
#[derive(Debug, PartialEq)]
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

type Vector<T> = (Result<(), ()>, Vec<T>, usize);

fn grown<T: From<u8>>(
    len: usize,
    capacity: usize,
    additional: usize,
    reserve: impl Fn(&Control<'_>, &mut Vec<T>, usize) -> Result<(), ()>,
) -> impl Fn(&Control<'_>) -> Vector<T> {
    move |control| {
        let mut values = vector::<T>(len, capacity);
        // The host charged the bank the vector already owns, which growth returns.
        let _ = control.add(values.capacity() * size_of::<T>());
        let result = reserve(control, &mut values, additional);
        let capacity = values.capacity();
        (result, values, capacity)
    }
}

#[test]
fn growth_charges_the_host_as_the_frozen_body_did_for_every_refusal_and_limit() {
    let cases: [(usize, usize, usize); 6] = [
        (0, 0, 1),  // an empty vector gets the minimum bank
        (3, 3, 1),  // a full vector doubles
        (3, 3, 10), // the need exceeds the doubling
        (2, 8, 1),  // there is already room: only the opening step
        (0, 0, 0),  // nothing is needed
        (5, 5, 3),  // moving several elements
    ];
    for (len, capacity, additional) in cases {
        let free = assert_same_trace(
            grown::<u64>(len, capacity, additional, frozen::reserve),
            grown::<u64>(len, capacity, additional, |control, values, additional| {
                control.reserve(values, additional)
            }),
        );
        assert_eq!(free.result.0, Ok(()), "{len} {capacity} {additional}");
        assert_eq!(free.failure, None);
        assert!(free.result.2 >= len + additional);
    }
    // A zero-sized element has no bank to move.
    assert_same_trace(
        grown::<Zero>(0, 0, 3, frozen::reserve),
        grown::<Zero>(0, 0, 3, |control, values, additional| {
            control.reserve(values, additional)
        }),
    );
}

#[test]
fn growth_that_cannot_be_represented_or_allocated_latches_the_same_failure() {
    // The length plus the addition overflows.
    let overflow = assert_same_trace(
        grown::<u64>(3, 3, usize::MAX, frozen::reserve),
        grown::<u64>(3, 3, usize::MAX, |control, values, additional| {
            control.reserve(values, additional)
        }),
    );
    assert_eq!(overflow.result.0, Err(()));
    assert_eq!(overflow.failure, Some(Failure::CapacityOverflow));
    // The planned bank in bytes overflows.
    let planned = assert_same_trace(
        grown::<u64>(0, 0, usize::MAX / 4, frozen::reserve),
        grown::<u64>(0, 0, usize::MAX / 4, |control, values, additional| {
            control.reserve(values, additional)
        }),
    );
    assert_eq!(planned.result.0, Err(()));
    assert_eq!(planned.failure, Some(Failure::CapacityOverflow));
    // A bank of isize::MAX bytes passes the arithmetic and the allocator's
    // capacity check refuses it without allocating.
    let allocation = assert_same_trace(
        grown::<u8>(0, 0, isize::MAX as usize, frozen::reserve),
        grown::<u8>(0, 0, isize::MAX as usize, |control, values, additional| {
            control.reserve(values, additional)
        }),
    );
    assert_eq!(allocation.result.0, Err(()));
    assert_eq!(allocation.failure, Some(Failure::Allocation));
}

fn copied<T>(
    value: &'static str,
    copy: impl Fn(&Control<'_>, &str) -> Result<T, ()>,
) -> impl Fn(&Control<'_>) -> Result<T, ()> {
    move |control| copy(control, value)
}

#[test]
fn string_copies_charge_the_host_as_the_frozen_body_did() {
    for value in ["", "a", "abc", "é😀 mixed"] {
        let free = assert_same_trace(
            copied(value, frozen::string),
            copied(value, |control, value| control.string(value)),
        );
        assert_eq!(free.result.as_deref(), Ok(value));
        assert_eq!(free.failure, None);
    }
}

#[test]
fn boxed_copies_charge_the_host_as_the_frozen_body_did() {
    for value in ["", "a", "abc", "é😀 mixed"] {
        let free = assert_same_trace(
            copied(value, frozen::boxed),
            copied(value, |control, value| control.boxed(value)),
        );
        assert_eq!(free.result.as_deref(), Ok(value));
        assert_eq!(free.failure, None);
    }
}

#[test]
fn a_refused_copy_latches_the_refusal_and_returns_no_text() {
    let observed = observe(
        Config {
            refuse_at: 2,
            ..Config::FREE
        },
        |control| control.string("abc"),
    );
    assert_eq!(observed.result, Err(()));
    assert_eq!(observed.failure, Some(Failure::Admission));
}

/// The bodies of `Control::reserve`, `string` and `boxed` as they were when
/// they lived in this file, with `self` renamed to `control`.
mod frozen {
    use super::super::*;

    pub(super) fn reserve<T>(
        control: &Control<'_>,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), ()> {
        control.step()?;
        let Some(required) = values.len().checked_add(additional) else {
            return control.fail(Failure::CapacityOverflow);
        };
        if required <= values.capacity() {
            return Ok(());
        }
        let Some(old) = values.capacity().checked_mul(size_of::<T>()) else {
            return control.fail(Failure::CapacityOverflow);
        };
        let target = required.max(values.capacity().saturating_mul(2)).max(4);
        let Some(planned) = target.checked_mul(size_of::<T>()) else {
            return control.fail(Failure::CapacityOverflow);
        };
        struct Growth<T> {
            next: Vec<T>,
            previous: std::vec::IntoIter<T>,
        }
        let header = size_of::<Growth<T>>();
        let Some(prospective) = planned.checked_add(header) else {
            return control.fail(Failure::CapacityOverflow);
        };
        // The actual old bank stays live while a distinct new bank is formed.
        // No opaque allocator reallocation hides the simultaneous old/new cost
        // or the cancellable movement of retained elements.
        control.add(prospective)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(target).is_err() {
            control.released(prospective);
            return control.fail(Failure::Allocation);
        }
        let Some(actual) = next.capacity().checked_mul(size_of::<T>()) else {
            return control.fail(Failure::CapacityOverflow);
        };
        if actual > planned && control.add(actual - planned).is_err() {
            drop(next);
            control.released(prospective);
            return Err(());
        }
        let mut growth = Growth {
            next,
            previous: std::mem::take(values).into_iter(),
        };
        while let Some(value) = growth.previous.next() {
            if control.step().is_err() {
                drop(value);
                drop(growth);
                control.released(old + actual + header);
                return Err(());
            }
            growth.next.push(value);
        }
        let Growth { next, previous } = growth;
        drop(previous);
        *values = next;
        control.released(old + header);
        Ok(())
    }
    pub(super) fn string(control: &Control<'_>, value: &str) -> Result<String, ()> {
        control.step()?;
        control.add(value.len())?;
        let mut out = String::new();
        if out.try_reserve_exact(value.len()).is_err() {
            control.released(value.len());
            return control.fail(Failure::Allocation);
        }
        if out.capacity() > value.len() {
            control.add(out.capacity() - value.len())?;
        }
        for point in value.chars() {
            if control.step().is_err() {
                let capacity = out.capacity();
                drop(out);
                control.released(capacity);
                return Err(());
            }
            out.push(point);
        }
        Ok(out)
    }
    pub(super) fn boxed(control: &Control<'_>, value: &str) -> Result<Box<str>, ()> {
        let out = string(control, value)?;
        let capacity = out.capacity();
        // The old String may coexist with the shrink destination allocation.
        if capacity != out.len() {
            control.add(out.len())?;
        }
        let result = out.into_boxed_str();
        if capacity != result.len() {
            control.released(capacity);
        }
        Ok(result)
    }
}
