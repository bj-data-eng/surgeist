mod miri {

    use indoc::indoc;

    use crate::{TestAction, run_test_actions};

    #[test]
    fn finalization_registry_simple() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });

            registry.register(["foo"]);
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
            }),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Callback should run at least once
            TestAction::assert_eq("counter", 1),
        ]);
    }

    #[test]
    fn finalization_registry_cleans_up_after_an_idle_job_run() {
        run_test_actions([
            TestAction::run(indoc! {r#"
                let counter = 0;
                let target = {};
                const registry = new FinalizationRegistry(() => { counter++; });
                registry.register(target);
            "#}),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            TestAction::assert_eq("counter", 0),
            TestAction::run("target = null"),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
                ctx.run_jobs().unwrap();
            }),
            TestAction::assert_eq("counter", 1),
        ]);
    }

    #[test]
    fn finalization_registry_cleans_up_on_a_second_collection() {
        run_test_actions([
            TestAction::run(indoc! {r#"
                let counter = 0;
                let secondTarget = {};
                const registry = new FinalizationRegistry(value => { counter += value; });
                registry.register({}, 1);
                registry.register(secondTarget, 10);
            "#}),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
                ctx.run_jobs().unwrap();
            }),
            TestAction::assert_eq("counter", 1),
            TestAction::run("secondTarget = null"),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
                ctx.run_jobs().unwrap();
            }),
            TestAction::assert_eq("counter", 11),
        ]);
    }

    #[test]
    fn finalization_registry_unregister() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry(() => {
                counter++;
            });

            {
                let array = ["foo"];
                registry.register(array, undefined, array);
                registry.unregister(array);
            }

        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
            }),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Callback shouldn't run
            TestAction::assert_eq("counter", 0),
        ]);
    }

    #[test]
    fn finalization_registry_held_value_handover() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;
            const registry = new FinalizationRegistry((value) => {
                counter += value.increment;
            });

            registry.register(["foo"], { increment: 5 });
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
            }),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Registry should handover the held value as argument
            TestAction::assert_eq("counter", 5),
        ]);
    }

    #[test]
    fn finalization_registry_unrelated_unregister_token() {
        run_test_actions([
            TestAction::run(indoc! {r#"
            let counter = 0;

            const registry = new FinalizationRegistry((value) => {
                counter += 1;
            });

            registry.register(["foo"], undefined, {});
            registry.unregister({});
        "#}),
            TestAction::assert_eq("counter", 0),
            TestAction::inspect_context(|ctx| {
                ctx.runtime().collect().unwrap();
            }),
            TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
            // Object should not have been unregistered if the token is not the correct one
            TestAction::assert_eq("counter", 1),
        ]);
    }
}

#[test]
fn cleanup_retains_job_callback_host_data() {
    use std::rc::Rc;

    use crate::{
        Context, JsObject, JsResult, JsValue, Source,
        context::HostHooks,
        job::JobCallback,
        native_function::{CapturedObject, NativeCapture},
        object::JsFunction,
    };

    struct HostData(JsObject);

    impl NativeCapture for HostData {
        type Stored = CapturedObject;

        fn store(&self) -> Self::Stored {
            self.0.store()
        }

        fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
            Ok(Self(JsObject::restore(stored)?))
        }
    }

    struct Hooks;

    impl HostHooks for Hooks {
        fn make_job_callback(
            &self,
            callback: JsFunction,
            context: crate::ContextAccess<'_>,
        ) -> JobCallback {
            JobCallback::new(callback, HostData(JsObject::default(context.intrinsics())))
        }

        fn call_job_callback(
            &self,
            job: &JobCallback,
            this: &JsValue,
            args: &[JsValue],
            context: crate::ContextAccess<'_>,
        ) -> JsResult<JsValue> {
            let host = job.host_defined().downcast_ref::<HostData>().unwrap();
            assert!(host.0.is_ordinary());
            job.callback().call(this, args, context)
        }
    }

    let mut context = Context::builder()
        .host_hooks(Rc::new(Hooks))
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes(
            "let count = 0; const registry = new FinalizationRegistry(() => ++count); registry.register({});",
        ))
        .unwrap();
    context.runtime().collect().unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("count")).unwrap(),
        JsValue::from(1)
    );
}
