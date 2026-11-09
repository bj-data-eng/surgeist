use surgeist_ledger::{Change, Error, Limits, Operation, RefusalReason, Tariff, Tariffs};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Actions, Credits }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

type Ledger = Operation<Work, Resource>;

const ACTIONS: Work = Work::Actions;
const CREDITS: Work = Work::Credits;
const BYTES: Resource = Resource::Bytes;

fn limits(actions: u64, credits: u64, bytes: u64) -> Limits<Work, Resource> {
    Limits::new(
        WorkTable {
            Actions: actions,
            Credits: credits,
        },
        ResourceTable { Bytes: bytes },
    )
}

mod html_fixture {
    use super::*;

    pub fn visit(
        ledger: &mut Ledger,
        scope: surgeist_ledger::Scope,
    ) -> Result<(), Error<Work, Resource>> {
        ledger.apply(scope, Change::Work(CREDITS, 3))
    }
}

mod css_fixture {
    use super::*;

    pub fn parse(
        ledger: &mut Ledger,
        scope: surgeist_ledger::Scope,
    ) -> Result<(), Error<Work, Resource>> {
        ledger.apply_batch(
            scope,
            &[Change::Work(ACTIONS, 2), Change::Acquire(BYTES, 5)],
        )
    }
}

fn css_tariffs(rate: u64) -> Tariffs<Work, Resource> {
    Tariffs::new(
        WorkTable {
            Actions: Tariff::new(CREDITS, rate),
            Credits: Tariff::new(CREDITS, 1),
        },
        ResourceTable {
            Bytes: Tariff::new(BYTES, 1),
        },
    )
}

#[test]
fn delegated_native_units_are_converted_once_through_nested_scopes() {
    let mut ledger = Ledger::new("document", limits(0, 11, 20), 3).unwrap();
    let root = ledger.root();
    let css = ledger
        .child(root, "css", limits(4, 0, 10), css_tariffs(2))
        .unwrap();
    let syntax = ledger
        .child(css, "syntax", limits(2, 0, 5), Tariffs::identity())
        .unwrap();

    assert_eq!(ledger.scope_name(syntax).unwrap(), "syntax");
    html_fixture::visit(&mut ledger, root).unwrap();
    css_fixture::parse(&mut ledger, syntax).unwrap();
    css_fixture::parse(&mut ledger, css).unwrap();

    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), 11);
    assert_eq!(ledger.usage(root).unwrap().work(ACTIONS), 0);
    assert_eq!(ledger.usage(css).unwrap().work(ACTIONS), 4);
    assert_eq!(ledger.usage(syntax).unwrap().work(ACTIONS), 2);
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 10);
    assert_eq!(ledger.usage(css).unwrap().live(BYTES), 10);
    assert_eq!(ledger.usage(syntax).unwrap().live(BYTES), 5);

    let error = ledger.apply(syntax, Change::Work(ACTIONS, 1)).unwrap_err();
    let Error::Refused(refusal) = error else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.scope, syntax);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 2 });
    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), 11);
}

#[test]
fn root_refusal_rolls_back_child_work_storage_and_peak() {
    let mut ledger = Ledger::new("document", limits(0, 3, 20), 2).unwrap();
    let root = ledger.root();
    let css = ledger
        .child(root, "css", limits(4, 0, 10), css_tariffs(2))
        .unwrap();
    ledger.apply(css, Change::Work(ACTIONS, 1)).unwrap();

    let error = ledger
        .apply_batch(css, &[Change::Acquire(BYTES, 5), Change::Work(ACTIONS, 1)])
        .unwrap_err();
    let Error::Refused(refusal) = error else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.scope, root);
    assert_eq!(refusal.item, 1);
    assert_eq!(refusal.change, Change::Work(CREDITS, 2));
    assert_eq!(refusal.current, 2);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 3 });
    assert_eq!(ledger.usage(css).unwrap().work(ACTIONS), 1);
    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), 2);
    assert_eq!(ledger.usage(css).unwrap().peak(BYTES), 0);
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 0);
    assert_eq!(ledger.usage(root).unwrap().peak(BYTES), 0);
}

#[test]
fn ordered_checks_preserve_earlier_refusals_and_check_local_before_conversion() {
    let mut ledger = Ledger::new("operation", Limits::unlimited(), 2).unwrap();
    let root = ledger.root();
    let child = ledger
        .child(root, "child", limits(3, 0, 4), css_tariffs(u64::MAX))
        .unwrap();

    let Error::Refused(refusal) = ledger
        .apply_batch(
            child,
            &[Change::Acquire(BYTES, 5), Change::Work(ACTIONS, 2)],
        )
        .unwrap_err()
    else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.item, 0);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 4 });

    let Error::Refused(refusal) = ledger.apply(child, Change::Work(ACTIONS, 4)).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 3 });

    let Error::Refused(refusal) = ledger.apply(child, Change::Work(ACTIONS, 2)).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.scope, child);
    assert_eq!(refusal.reason, RefusalReason::Overflow);
    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), 0);
}

#[test]
fn cleanup_releases_only_originating_custody_and_keeps_transient_peak() {
    let mut ledger = Ledger::new("operation", limits(0, 0, 10), 3).unwrap();
    let root = ledger.root();
    let a = ledger
        .child(root, "a", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    let b = ledger
        .child(root, "b", Limits::unlimited(), Tariffs::identity())
        .unwrap();

    ledger.apply(root, Change::Acquire(BYTES, 2)).unwrap();
    ledger.apply(a, Change::Acquire(BYTES, 3)).unwrap();
    ledger
        .apply_batch(b, &[Change::Acquire(BYTES, 5), Change::Release(BYTES, 5)])
        .unwrap();
    assert_eq!(ledger.usage(root).unwrap().peak(BYTES), 10);
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 5);
    assert_eq!(ledger.usage(b).unwrap().peak(BYTES), 5);

    let Error::Refused(refusal) = ledger.apply(root, Change::Release(BYTES, 3)).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.current, 2);
    assert_eq!(refusal.reason, RefusalReason::Underflow);
    assert!(ledger.apply(b, Change::Release(BYTES, 1)).is_err());

    ledger.apply(root, Change::Release(BYTES, 2)).unwrap();
    ledger.apply(a, Change::Release(BYTES, 3)).unwrap();
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 0);
    assert_eq!(ledger.usage(root).unwrap().peak(BYTES), 10);
}

#[test]
fn foreign_scope_and_capacity_refusals_leave_state_intact() {
    let mut ledger = Ledger::new("operation", Limits::unlimited(), 1).unwrap();
    let other = Ledger::new("other", Limits::unlimited(), 1).unwrap();
    let root = ledger.root();

    assert_eq!(
        ledger.apply(other.root(), Change::Work(CREDITS, 1)),
        Err(Error::ForeignScope)
    );
    assert_eq!(
        ledger.child(root, "child", Limits::unlimited(), Tariffs::identity()),
        Err(Error::ScopeLimit { limit: 1 })
    );
    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), 0);
    ledger.apply(root, Change::Work(CREDITS, u64::MAX)).unwrap();
    let Error::Refused(refusal) = ledger.apply(root, Change::Work(CREDITS, 1)).unwrap_err() else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.reason, RefusalReason::Overflow);
    assert_eq!(ledger.usage(root).unwrap().work(CREDITS), u64::MAX);
}

#[test]
fn partial_releases_use_the_original_resource_tariff() {
    let mut ledger = Ledger::new("operation", limits(0, 0, 24), 3).unwrap();
    let root = ledger.root();
    let tariffs = |scale| {
        Tariffs::new(
            WorkTable {
                Actions: Tariff::new(ACTIONS, 1),
                Credits: Tariff::new(CREDITS, 1),
            },
            ResourceTable {
                Bytes: Tariff::new(BYTES, scale),
            },
        )
    };
    let child = ledger
        .child(root, "child", limits(0, 0, 8), tariffs(3))
        .unwrap();
    let nested = ledger
        .child(child, "nested", limits(0, 0, 4), tariffs(2))
        .unwrap();

    ledger.apply(nested, Change::Acquire(BYTES, 4)).unwrap();
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 24);
    assert_eq!(ledger.usage(child).unwrap().live(BYTES), 8);
    ledger.apply(nested, Change::Release(BYTES, 1)).unwrap();
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 18);
    assert_eq!(ledger.usage(child).unwrap().live(BYTES), 6);
    ledger.apply(nested, Change::Release(BYTES, 3)).unwrap();
    assert_eq!(ledger.usage(root).unwrap().live(BYTES), 0);
    assert_eq!(ledger.usage(root).unwrap().peak(BYTES), 24);
}

#[test]
fn a_consumer_stop_keeps_prior_work_and_allows_cleanup() {
    let mut ledger = Ledger::new("operation", Limits::unlimited(), 1).unwrap();
    let root = ledger.root();
    ledger
        .apply_batch(root, &[Change::Work(CREDITS, 2), Change::Acquire(BYTES, 3)])
        .unwrap();

    let cancelled = true;
    let continue_parse = |ledger: &mut Ledger| {
        if cancelled {
            return Err("cancelled");
        }
        ledger
            .apply(root, Change::Work(CREDITS, 7))
            .map_err(|_| "refused")
    };
    assert_eq!(continue_parse(&mut ledger), Err("cancelled"));
    ledger.apply(root, Change::Release(BYTES, 3)).unwrap();

    let usage = ledger.usage(root).unwrap();
    assert_eq!(usage.work(CREDITS), 2);
    assert_eq!(usage.live(BYTES), 0);
    assert_eq!(usage.peak(BYTES), 3);
}
