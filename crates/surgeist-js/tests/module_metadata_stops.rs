#![allow(unused_crate_dependencies)]

use std::{cell::Cell, fmt::Debug, rc::Rc};

use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeErrorKind, JsObject, JsResult, JsValue, Module,
    Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    gc::heap::{AccessError, Trace as HeapTrace},
    js_string,
    module::SyntheticModuleInitializer,
    native_function::{CapturedObject, NativeCapture},
};

const LIMIT: u64 = 100_000;

#[derive(Default)]
struct Observation {
    stores: Cell<usize>,
    evaluations: Cell<usize>,
}

#[derive(Clone)]
struct Capture {
    value: JsObject,
    observation: Rc<Observation>,
}

#[derive(HeapTrace)]
struct StoredCapture {
    value: CapturedObject,
    #[heap_trace(skip)]
    observation: Rc<Observation>,
}

impl NativeCapture for Capture {
    type Stored = StoredCapture;

    fn store(&self) -> Self::Stored {
        // The observer contains only scalar cells outside the owning record.
        // Capture conversion does not enter the engine or change module state.
        self.observation
            .stores
            .set(self.observation.stores.get() + 1);
        StoredCapture {
            value: self.value.store(),
            observation: self.observation.clone(),
        }
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self {
            value: JsObject::restore(&stored.value),
            observation: stored.observation.clone(),
        }
    }
}

fn initializer(capture: Capture) -> SyntheticModuleInitializer {
    SyntheticModuleInitializer::from_fn_with_captures(
        |module, capture, _| {
            capture
                .observation
                .evaluations
                .set(capture.observation.evaluations.get() + 1);
            module.set_export(&js_string!("captured"), capture.value.clone().into())
        },
        capture,
    )
}

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

fn in_scope<T>(
    context: &mut Context,
    call: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
) -> JsResult<T> {
    context
        .with_execution(ExecutionOptions::new(LIMIT), call)
        .unwrap()
        .result
}

fn after_stop<T: Debug>(
    context: &mut Context,
    call: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
) -> (JsResult<T>, JsError) {
    let mut direct = None;
    let mut first = None;
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let error = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            first = Some(error.clone());
            direct = Some(call(access));
            Ok(())
        })
        .unwrap();
    let first = first.unwrap();
    assert_eq!(outcome.result.unwrap_err(), first);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    (
        direct.expect("the direct constructor must return its own result"),
        first,
    )
}

fn finish(module: &Module, context: &mut Context) {
    let loaded = in_scope(context, |access| module.load(access)).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    in_scope(context, |access| module.link(access)).unwrap();
    let evaluation = in_scope(context, |access| module.evaluate(access)).unwrap();
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    let cached = in_scope(context, |access| module.evaluate(access)).unwrap();
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(
        cached.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
}

#[test]
fn retained_stop_refuses_synthetic_construction_before_storing_or_evaluating_captures() {
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let object = JsObject::with_null_proto(context.runtime());
    let capture = Capture {
        value: object.clone(),
        observation: observation.clone(),
    };
    let stopped_initializer = initializer(capture.clone());
    assert_eq!(observation.stores.get(), 0);
    let (direct, first) = after_stop(&mut context, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            stopped_initializer,
            None,
            None,
            access,
        )
    });
    assert_eq!(
        observation.stores.get(),
        0,
        "direct construction: {direct:?}"
    );
    assert_eq!(observation.evaluations.get(), 0);
    assert_eq!(direct.unwrap_err(), first);

    let module = in_scope(&mut context, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            initializer(capture),
            None,
            None,
            access,
        )
    })
    .unwrap();
    assert!(observation.stores.get() > 0);
    assert_eq!(observation.evaluations.get(), 0);
    finish(&module, &mut context);
    assert_eq!(observation.evaluations.get(), 1);
    assert_eq!(
        module
            .get_value(js_string!("captured"), context.access())
            .unwrap(),
        object.into()
    );
}

#[test]
fn retained_stop_precedes_synthetic_foreign_realm_validation_without_storing_captures() {
    let mut owner = context();
    let foreign = context();
    let observation = Rc::new(Observation::default());
    let capture = Capture {
        value: JsObject::with_null_proto(owner.runtime()),
        observation: observation.clone(),
    };
    let steps = initializer(capture.clone());
    let (direct, first) = after_stop(&mut owner, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            steps,
            None,
            Some(foreign.realm().clone()),
            access,
        )
    });
    assert_eq!(observation.stores.get(), 0);
    assert_eq!(observation.evaluations.get(), 0);
    assert_eq!(direct.unwrap_err(), first);
    let module = in_scope(&mut owner, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            initializer(capture),
            None,
            None,
            access,
        )
    })
    .unwrap();
    finish(&module, &mut owner);
    assert_eq!(observation.evaluations.get(), 1);
}

#[test]
fn retained_stop_refuses_default_value_construction_and_preserves_a_fresh_attempt() {
    let mut context = context();
    let value: JsValue = JsObject::with_null_proto(context.runtime()).into();
    let (direct, first) = after_stop(&mut context, |access| {
        Module::from_value_as_default(value.clone(), access)
    });
    assert_eq!(direct.unwrap_err(), first);
    let module = in_scope(&mut context, |access| {
        Module::from_value_as_default(value.clone(), access)
    })
    .unwrap();
    finish(&module, &mut context);
    assert_eq!(
        module
            .get_value(js_string!("default"), context.access())
            .unwrap(),
        value
    );
}

#[test]
fn retained_stop_precedes_a_later_json_syntax_error_and_preserves_json_construction() {
    let mut context = context();
    let (direct, first) = after_stop(&mut context, |access| {
        Module::parse_json(js_string!("{"), access)
    });
    assert_eq!(direct.unwrap_err(), first);
    let malformed = in_scope(&mut context, |access| {
        Module::parse_json(js_string!("{"), access)
    })
    .unwrap_err();
    assert_eq!(
        malformed.as_native().unwrap().kind(),
        &JsNativeErrorKind::Syntax
    );
    let module = in_scope(&mut context, |access| {
        Module::parse_json(js_string!("{\"answer\":42}"), access)
    })
    .unwrap();
    finish(&module, &mut context);
    let value = module
        .get_value(js_string!("default"), context.access())
        .unwrap();
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .get(js_string!("answer"), context.access())
            .unwrap(),
        42.into()
    );
}

#[test]
fn ordinary_receiving_checks_reject_foreign_realm_and_capture_but_accept_a_peer() {
    let mut owner = context();
    let foreign = context();
    let peer = Context::builder()
        .runtime(owner.runtime().clone())
        .build()
        .unwrap();
    let observation = Rc::new(Observation::default());
    let value = JsObject::with_null_proto(owner.runtime());
    let capture = Capture {
        value: value.clone(),
        observation: observation.clone(),
    };
    let bad_realm = in_scope(&mut owner, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            initializer(capture.clone()),
            None,
            Some(foreign.realm().clone()),
            access,
        )
    })
    .unwrap_err();
    let native = bad_realm.as_native().unwrap();
    assert_eq!(native.kind(), &JsNativeErrorKind::Type);
    assert_eq!(
        native.message(),
        "module realm belongs to a different runtime"
    );
    assert_eq!(observation.stores.get(), 0);
    assert_eq!(observation.evaluations.get(), 0);

    let foreign_capture = Capture {
        value: JsObject::with_null_proto(foreign.runtime()),
        observation: Rc::new(Observation::default()),
    };
    let foreign_observation = foreign_capture.observation.clone();
    let bad_capture = in_scope(&mut owner, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            initializer(foreign_capture),
            None,
            None,
            access,
        )
    })
    .unwrap_err();
    let native = bad_capture.as_native().unwrap();
    assert_eq!(native.kind(), &JsNativeErrorKind::Type);
    assert_eq!(native.message(), AccessError::ForeignHeap.to_string());
    assert_eq!(foreign_observation.evaluations.get(), 0);

    let module = in_scope(&mut owner, |access| {
        Module::synthetic(
            &[js_string!("captured")],
            initializer(capture),
            None,
            Some(peer.realm().clone()),
            access,
        )
    })
    .unwrap();
    assert_eq!(module.realm(), *peer.realm());
    finish(&module, &mut owner);
    assert_eq!(
        module
            .get_value(js_string!("captured"), owner.access())
            .unwrap(),
        value.into()
    );
    assert_eq!(observation.evaluations.get(), 1);
}

#[test]
fn retained_stop_refuses_native_export_construction_before_pulling_its_input() {
    use std::{cell::Cell, rc::Rc};
    use surgeist_js::{
        IntoJsModule, JsString, JsValue, NativeFunction, builtins::promise::PromiseState,
    };

    #[derive(Clone)]
    struct ExportInput {
        item: Option<(JsString, NativeFunction)>,
        pulls: Rc<Cell<usize>>,
        reached_eof: Rc<Cell<bool>>,
    }

    impl Iterator for ExportInput {
        type Item = (JsString, NativeFunction);

        fn next(&mut self) -> Option<Self::Item> {
            self.pulls.set(self.pulls.get() + 1);
            if let Some(item) = self.item.take() {
                Some(item)
            } else {
                self.reached_eof.set(true);
                None
            }
        }
    }

    fn exports(pulls: Rc<Cell<usize>>, reached_eof: Rc<Cell<bool>>) -> ExportInput {
        ExportInput {
            item: Some((
                js_string!("answer"),
                NativeFunction::from_fn_ptr(|_, _, _| Ok(42.into())),
            )),
            pulls,
            reached_eof,
        }
    }

    let mut context = context();
    let pulls = Rc::new(Cell::new(0));
    let reached_eof = Rc::new(Cell::new(false));
    let stopped_input = exports(pulls.clone(), reached_eof.clone());
    let (direct, first) = after_stop(&mut context, |access| stopped_input.into_js_module(access));
    assert_eq!(pulls.get(), 0, "direct construction: {direct:?}");
    assert!(!reached_eof.get());
    assert_eq!(direct.unwrap_err(), first);

    let fresh_pulls = Rc::new(Cell::new(0));
    let fresh_eof = Rc::new(Cell::new(false));
    let fresh_input = exports(fresh_pulls.clone(), fresh_eof.clone());
    let module = in_scope(&mut context, |access| fresh_input.into_js_module(access)).unwrap();
    assert!(fresh_pulls.get() > 0);
    assert!(
        fresh_eof.get(),
        "normal construction must reach the finite input's end"
    );
    finish(&module, &mut context);
    let exported = module
        .get_value(js_string!("answer"), context.access())
        .unwrap();
    let repeated = in_scope(&mut context, |access| module.evaluate(access)).unwrap();
    assert_eq!(
        repeated.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    let cached = module
        .get_value(js_string!("answer"), context.access())
        .unwrap();
    let function = exported.as_object().expect("the named export is callable");
    assert_eq!(function.identity(), cached.as_object().unwrap().identity());
    let result = in_scope(&mut context, |access| {
        function.call(&JsValue::undefined(), &[], access)
    })
    .unwrap();
    assert_eq!(result, 42.into());
}
