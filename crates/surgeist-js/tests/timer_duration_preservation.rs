use std::{cell::Cell, rc::Rc, time::Duration};
use surgeist_js::{
    Context, JsValue,
    job::{IntervalJob, TimeoutJob},
};

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn supplied() -> [Duration; 3] {
    [
        Duration::from_nanos(1),
        // Whole seconds imply whole milliseconds. This is the first whole
        // second strictly beyond the u64 millisecond range, without a cast.
        Duration::from_secs(u64::MAX / 1_000 + 1),
        Duration::from_millis(7),
    ]
}

#[test]
fn timeout_from_duration_preserves_submillisecond_and_large_supplied_values() {
    let effects = Rc::new(Cell::new(0));
    let mut observed = Vec::new();
    for duration in supplied() {
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let seen = effects.clone();
        let timer = TimeoutJob::from_duration(
            move |_| {
                let _keep = &probe;
                seen.set(seen.get() + 1);
                Ok(JsValue::undefined())
            },
            duration,
        );
        observed.push(timer.timeout().as_nanos());
        assert_eq!(effects.get(), 0);
        assert_eq!(drops.get(), 0);
        drop(timer);
        assert_eq!(drops.get(), 1);
    }
    assert_eq!(observed, supplied().map(|duration| duration.as_nanos()));
}

#[test]
fn timeout_with_realm_preserves_submillisecond_and_large_supplied_values() {
    let owner = Context::default();
    let effects = Rc::new(Cell::new(0));
    let mut observed = Vec::new();
    for duration in supplied() {
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let seen = effects.clone();
        let timer = TimeoutJob::with_realm(
            move |_| {
                let _keep = &probe;
                seen.set(seen.get() + 1);
                Ok(JsValue::undefined())
            },
            owner.realm().clone(),
            duration,
        );
        observed.push(timer.timeout().as_nanos());
        assert_eq!(effects.get(), 0);
        assert_eq!(drops.get(), 0);
        drop(timer);
        assert_eq!(drops.get(), 1);
    }
    assert_eq!(observed, supplied().map(|duration| duration.as_nanos()));
}

#[test]
fn interval_from_duration_preserves_submillisecond_and_large_supplied_values() {
    let effects = Rc::new(Cell::new(0));
    let mut observed = Vec::new();
    for duration in supplied() {
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let seen = effects.clone();
        let timer = IntervalJob::from_duration(
            move |_| {
                let _keep = &probe;
                seen.set(seen.get() + 1);
                Ok(JsValue::undefined())
            },
            duration,
        );
        observed.push(timer.interval().as_nanos());
        assert_eq!(effects.get(), 0);
        assert_eq!(drops.get(), 0);
        drop(timer);
        assert_eq!(drops.get(), 1);
    }
    assert_eq!(observed, supplied().map(|duration| duration.as_nanos()));
}

#[test]
fn interval_with_realm_preserves_submillisecond_and_large_supplied_values() {
    let owner = Context::default();
    let effects = Rc::new(Cell::new(0));
    let mut observed = Vec::new();
    for duration in supplied() {
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let seen = effects.clone();
        let timer = IntervalJob::with_realm(
            move |_| {
                let _keep = &probe;
                seen.set(seen.get() + 1);
                Ok(JsValue::undefined())
            },
            owner.realm().clone(),
            duration,
        );
        observed.push(timer.interval().as_nanos());
        assert_eq!(effects.get(), 0);
        assert_eq!(drops.get(), 0);
        drop(timer);
        assert_eq!(drops.get(), 1);
    }
    assert_eq!(observed, supplied().map(|duration| duration.as_nanos()));
}

#[test]
fn ordinary_millisecond_inputs_remain_exact_in_all_four_duration_constructors() {
    let owner = Context::default();
    let duration = Duration::from_millis(7);
    let timeout = TimeoutJob::from_duration(|_| Ok(JsValue::undefined()), duration);
    let realm_timeout = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        duration,
    );
    let interval = IntervalJob::from_duration(|_| Ok(JsValue::undefined()), duration);
    let realm_interval = IntervalJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        duration,
    );
    let expected = duration.as_nanos();
    assert_eq!(timeout.timeout().as_nanos(), expected);
    assert_eq!(realm_timeout.timeout().as_nanos(), expected);
    assert_eq!(interval.interval().as_nanos(), expected);
    assert_eq!(realm_interval.interval().as_nanos(), expected);
}
