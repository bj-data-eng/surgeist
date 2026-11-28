use super::{evaluate, units};
use std::{cell::Cell, rc::Rc};
use surgeist_bindings_js::preparation::{Admission, Request, Retention};
use surgeist_bindings_js::{Error, MicrotaskProgress, Process};

pub(super) fn raw_guest_throw_keeps_microtask_checkpoint_in_fifo(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        scope.evaluate(&units(
            r#"
            globalThis.order = [];
            globalThis.thrown = new Proxy({}, {
                get() { order.push("throw-observed"); }
            });
            "#,
        ))?;
        let first = scope.evaluate(&units(
            r#"() => {
                order.push("raw-throw");
                Promise.resolve().then(() => order.push("nested-promise"));
                throw thrown;
            }"#,
        ))?;
        scope.enqueue_microtask(&first)?;
        scope.evaluate(&units(
            "Promise.resolve().then(() => order.push('promise'));",
        ))?;
        let last = scope.evaluate(&units("() => order.push('raw-tail')"))?;
        scope.enqueue_microtask(&last)?;
        scope.evaluate(&units(
            r#"Promise.reject(thrown).catch(value => {
                order.push(value === thrown ? "promise-tail" : "wrong-rejection");
            });"#,
        ))?;
        Ok(())
    })?;

    let read = "order.join(',')";
    assert_eq!(evaluate(&mut execution, &realm, read)?, units(""));
    execution
        .checkpoint()
        .expect("an ordinary raw microtask throw must leave the checkpoint successful");
    let expected = units("raw-throw,promise,raw-tail,promise-tail,nested-promise");
    assert_eq!(evaluate(&mut execution, &realm, read)?, expected);
    execution.checkpoint()?;
    assert_eq!(evaluate(&mut execution, &realm, read)?, expected);
    Ok(())
}

pub(super) fn suspended_host_job_keeps_the_original_fifo_position(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let mut resumed = false;
    execution.in_realm(&realm, |scope| {
        scope.evaluate(&units(
            "globalThis.hostOrder = []; Promise.resolve().then(() => hostOrder.push('before'));",
        ))?;
        scope.enqueue_host_microtask(move |scope| {
            if resumed {
                scope.evaluate(&units("hostOrder.push('resumed')"))?;
                return Ok(MicrotaskProgress::Complete);
            }
            resumed = true;
            scope.evaluate(&units(
                "hostOrder.push('prefix'); Promise.resolve().then(() => hostOrder.push('nested'));",
            ))?;
            Ok(MicrotaskProgress::Suspended)
        })?;
        scope.evaluate(&units(
            "Promise.resolve().then(() => hostOrder.push('after'));",
        ))?;
        let tail = scope.evaluate(&units("() => hostOrder.push('raw')"))?;
        scope.enqueue_microtask(&tail)
    })?;

    let read = "hostOrder.join(',')";
    assert!(matches!(
        execution.checkpoint(),
        Err(Error::MicrotaskCheckpointRequired)
    ));
    assert_eq!(evaluate(&mut execution, &realm, read)?, units(""));
    assert_eq!(
        execution.checkpoint_microtasks()?,
        MicrotaskProgress::Suspended
    );
    assert_eq!(
        evaluate(&mut execution, &realm, read)?,
        units("before,prefix")
    );
    assert!(matches!(
        execution.checkpoint(),
        Err(Error::MicrotaskCheckpointRequired)
    ));
    assert_eq!(
        evaluate(&mut execution, &realm, read)?,
        units("before,prefix")
    );
    assert_eq!(
        execution.checkpoint_microtasks()?,
        MicrotaskProgress::Complete
    );
    let expected = units("before,prefix,resumed,after,raw,nested");
    assert_eq!(evaluate(&mut execution, &realm, read)?, expected);
    execution.checkpoint()?;
    assert_eq!(evaluate(&mut execution, &realm, read)?, expected);
    Ok(())
}

struct JobRetention {
    fail_release: bool,
}

impl Retention for JobRetention {
    fn release(self: Box<Self>) -> Result<(), Error> {
        if self.fail_release {
            return Err(Error::IdentityExhausted);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum CallbackBacking {
    QueuedJob,
    LocalRoot,
    ReturnedFrame,
}

pub(super) fn host_job_release_failure_stops_the_guest_suffix() -> Result<(), Error> {
    for backing in [
        CallbackBacking::QueuedJob,
        CallbackBacking::LocalRoot,
        CallbackBacking::ReturnedFrame,
    ] {
        check_host_job_release_failure(backing)?;
    }
    Ok(())
}

fn check_host_job_release_failure(backing: CallbackBacking) -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let host_entered = Rc::new(Cell::new(false));
    let entered_for_admission = Rc::clone(&host_entered);
    let _policy = Admission::new(
        move |request| {
            Ok(JobRetention {
                fail_release: match (backing, request) {
                    (CallbackBacking::QueuedJob, Request::HostMicrotask { .. })
                    | (CallbackBacking::ReturnedFrame, Request::CallbackFrame { .. }) => true,
                    (CallbackBacking::LocalRoot, Request::RootRecord { .. }) => {
                        entered_for_admission.get()
                    }
                    _ => false,
                },
            })
        },
        |_| Ok(()),
    )
    .install(&mut process, ())?;
    process.enable_instruction_counting()?;
    process.execution().in_realm(&realm, |scope| {
        scope.enqueue_host_microtask(move |scope| {
            host_entered.set(true);
            if matches!(backing, CallbackBacking::LocalRoot) {
                scope.string(&[])?;
            }
            Ok(MicrotaskProgress::Complete)
        })?;
        scope.evaluate(&units(
            "Promise.resolve().then(() => globalThis.entered = true)",
        ))?;
        Ok(())
    })?;
    let before = process.instruction_count();
    assert!(matches!(
        process.execution().checkpoint_microtasks(),
        Err(Error::PreparationCleanup(_))
    ));
    assert_eq!(
        process.instruction_count(),
        before,
        "guest work after {backing:?} release failure"
    );
    Ok(())
}
