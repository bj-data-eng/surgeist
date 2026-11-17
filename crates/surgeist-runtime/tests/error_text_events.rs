//! Message text and `Error` contract of the native DOM event error types.
//!
//! Each expected message is written from the documented meaning of the variant
//! and from every place the crate raises it. `DomEventDispatchError` is
//! `#[non_exhaustive]`, so its index function panics on an unlisted variant
//! instead of failing to compile. A wrapping variant states only its own
//! context and returns the wrapped error from `source()`.
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::Debug;

use surgeist_dom::{DomError, event::DomSourceEventClockFailure, hosted::DomHostedError};
use surgeist_runtime::{
    DomEventDispatchError, DomEventDispatcher, DomEventHostError, DomEventRegistryLimits,
    DomEventResource, DomEventTimestampUnavailable,
};

/// Checks one type's table: row order against the exhaustive `index`, the exact
/// message, no trailing period, no `Debug` form, and `source()` present exactly
/// for the wrapping variants. `skipped` lists row numbers whose payload cannot be
/// built through the public API.
fn check_table<E>(
    rows: Vec<(E, &'static str)>,
    index: fn(&E) -> usize,
    wraps: fn(&E) -> bool,
    skipped: &[usize],
) where
    E: Error + 'static + Debug,
{
    let mut position = 0;
    for (value, expected) in &rows {
        while skipped.contains(&position) {
            position += 1;
        }
        assert_eq!(index(value), position, "row order for {value:?}");
        position += 1;
        let text = value.to_string();
        assert_eq!(&text, expected, "message for {value:?}");
        assert!(!text.ends_with('.'), "no trailing period: {text}");
        assert!(
            text.chars().next().unwrap().is_lowercase(),
            "a lowercase clause: {text}"
        );
        assert_ne!(text, format!("{value:?}"), "message is the Debug form");
        assert_eq!(
            value.source().is_some(),
            wraps(value),
            "source presence for {value:?}"
        );
    }
}

/// `source()` returns exactly the wrapped error and the message does not repeat
/// its text.
fn assert_chains<E: Error>(outer: &E, inner: &dyn Error) {
    let source = outer.source().expect("a wrapping variant has a source");
    assert_eq!(source.to_string(), inner.to_string());
    assert!(
        !outer.to_string().contains(&inner.to_string()),
        "the wrapped error is reported by source(), not repeated: {outer}"
    );
}

fn assert_send_sync<E: Error + Send + Sync + 'static>() {}

#[test]
fn dispatch_errors_state_the_refused_dispatch_step() {
    assert_send_sync::<DomEventDispatchError>();
    // Row 3 (`ParsedSynchronousPending`) carries a parsed notification requirement
    // that only a parsed installation can issue, so it has no constructed row.
    check_table(
        vec![
            (
                DomEventDispatchError::Dom(DomError::StaleRange),
                "canonical identity or source validation refused the native DOM event dispatch",
            ),
            (
                DomEventDispatchError::Hosted(DomHostedError::ForeignSource),
                "the hosted source or Global association of the native DOM event could not be qualified",
            ),
            (
                DomEventDispatchError::Admission(DomEventResource::Storage),
                "native DOM event dispatch admission refused event, target, listener and invocation storage",
            ),
            (
                DomEventDispatchError::AlreadyDispatching,
                "native DOM event is already dispatching",
            ),
            (
                DomEventDispatchError::Uninitialized,
                "native DOM event is uninitialized",
            ),
            (
                DomEventDispatchError::AlreadyCompleted,
                "native DOM event already completed dispatch",
            ),
            (
                DomEventDispatchError::Abandoned,
                "native DOM event invocation was abandoned",
            ),
            (
                DomEventDispatchError::WrongContinuation,
                "native DOM event dispatch continuation does not match the state this dispatcher retained",
            ),
            (
                DomEventDispatchError::WrongTargetRegistry,
                "standalone EventTarget belongs to another dispatcher",
            ),
            (
                DomEventDispatchError::HostRefused(DomEventHostError { code: 7 }),
                "native DOM event host refused execution",
            ),
            (
                DomEventDispatchError::IdentityExhausted,
                "native DOM event registry or registration identity or capacity is exhausted",
            ),
            (
                DomEventDispatchError::CyclicPath,
                "native DOM event parent path contains a cycle",
            ),
        ],
        |error| match error {
            DomEventDispatchError::Dom(_) => 0,
            DomEventDispatchError::Hosted(_) => 1,
            DomEventDispatchError::Admission(_) => 2,
            DomEventDispatchError::ParsedSynchronousPending { .. } => 3,
            DomEventDispatchError::AlreadyDispatching => 4,
            DomEventDispatchError::Uninitialized => 5,
            DomEventDispatchError::AlreadyCompleted => 6,
            DomEventDispatchError::Abandoned => 7,
            DomEventDispatchError::WrongContinuation => 8,
            DomEventDispatchError::WrongTargetRegistry => 9,
            DomEventDispatchError::HostRefused(_) => 10,
            DomEventDispatchError::IdentityExhausted => 11,
            DomEventDispatchError::CyclicPath => 12,
            // `#[non_exhaustive]`: a variant added later must get a row here.
            _ => unreachable!("a new DomEventDispatchError variant needs a table row"),
        },
        |error| {
            matches!(
                error,
                DomEventDispatchError::Dom(_)
                    | DomEventDispatchError::Hosted(_)
                    | DomEventDispatchError::HostRefused(_)
            )
        },
        &[3],
    );
    let inner = DomError::StaleRange;
    assert_chains(&DomEventDispatchError::Dom(inner.clone()), &inner);
    let inner = DomHostedError::ForeignSource;
    assert_chains(&DomEventDispatchError::Hosted(inner.clone()), &inner);
}

#[test]
fn dispatch_admission_names_each_refused_resource() {
    for (resource, phrase) in [
        (
            DomEventResource::Storage,
            "event, target, listener and invocation storage",
        ),
        (
            DomEventResource::Registrations,
            "listener registration storage",
        ),
        (
            DomEventResource::PendingEvents,
            "running and retained dispatch state",
        ),
        (
            DomEventResource::NestedDispatches,
            "active nested execution",
        ),
        (DomEventResource::PathItems, "event path storage"),
        (
            DomEventResource::SnapshotListeners,
            "the listener snapshot of an invoke step",
        ),
        (DomEventResource::FailureRecords, "callback error output"),
        (DomEventResource::Callbacks, "callback execution count"),
        (DomEventResource::Work, "traversal and listener work"),
        (DomEventResource::Allocation, "a fallible allocation"),
    ] {
        assert_eq!(
            DomEventDispatchError::Admission(resource).to_string(),
            format!("native DOM event dispatch admission refused {phrase}")
        );
    }
}

#[test]
fn timestamp_errors_state_the_unavailable_creation_clock() {
    check_table(
        vec![
            (
                DomEventTimestampUnavailable::NoAuthenticatedClock,
                "authenticated Event creation clock unavailable",
            ),
            (
                DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Unavailable),
                "the original Event creation clock refused its firing observation",
            ),
            (
                DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Regressed),
                "the original Event creation clock refused its firing observation",
            ),
            (
                DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Overflow),
                "the original Event creation clock refused its firing observation",
            ),
        ],
        |error| match error {
            DomEventTimestampUnavailable::NoAuthenticatedClock => 0,
            DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Unavailable) => 1,
            DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Regressed) => 2,
            DomEventTimestampUnavailable::Provider(DomSourceEventClockFailure::Overflow) => 3,
        },
        |error| matches!(error, DomEventTimestampUnavailable::Provider(_)),
        &[],
    );
    // The clock failure itself says which refusal it was.
    for failure in [
        DomSourceEventClockFailure::Unavailable,
        DomSourceEventClockFailure::Regressed,
        DomSourceEventClockFailure::Overflow,
    ] {
        assert_chains(&DomEventTimestampUnavailable::Provider(failure), &failure);
    }
}

// End to end: errors returned by public operations.

#[test]
fn a_dispatcher_refused_for_its_limits_names_the_exhausted_resource() {
    for (limits, resource, phrase) in [
        (
            DomEventRegistryLimits {
                registrations: 8,
                pending_events: 8,
                nested_dispatches: 8,
                storage_bytes: 0,
                work: 1_000_000,
            },
            DomEventResource::Storage,
            "event, target, listener and invocation storage",
        ),
        (
            DomEventRegistryLimits {
                registrations: 8,
                pending_events: 8,
                nested_dispatches: 8,
                storage_bytes: 16_000_000,
                work: 0,
            },
            DomEventResource::Work,
            "traversal and listener work",
        ),
    ] {
        let error = match DomEventDispatcher::<()>::new(limits) {
            Err(error) => error,
            Ok(_) => panic!("limits without {resource:?} must refuse the dispatcher"),
        };
        assert!(
            matches!(&error, DomEventDispatchError::Admission(refused) if *refused == resource),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            format!("native DOM event dispatch admission refused {phrase}")
        );
    }
}
