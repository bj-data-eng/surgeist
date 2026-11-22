use super::{evaluate, units};
use surgeist_bindings_js::{Error, Process};

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
