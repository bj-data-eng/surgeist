#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    path::Path,
    rc::Rc,
    task::Poll,
};

use surgeist_js::{
    Context, JsError, JsNativeError, JsObject, JsValue, Module, Source,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{ContextTask, SimpleJobExecutor},
    js_string,
    module::{ModuleLoader, ModuleRequest, Referrer},
    object::builtins::JsPromise,
};

struct StoppingLoader {
    saved_error: RefCell<Option<JsError>>,
    called: Cell<bool>,
    observed_stop: Cell<bool>,
    returned_saved_error: Cell<bool>,
}

impl ModuleLoader for StoppingLoader {
    fn load_imported_module(
        self: Rc<Self>,
        _: Referrer,
        request: ModuleRequest,
    ) -> ContextTask<Module> {
        assert_eq!(request.specifier(), &js_string!("dependency"));
        let mut first = true;
        ContextTask::from_poll(move |cx, host| {
            if first {
                first = false;
                self.called.set(true);
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            let stopped = host
                .with(|mut access| access.eval(Source::from_bytes("0")))
                .unwrap_err();
            assert_eq!(
                stopped.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            self.observed_stop.set(true);
            let error = self
                .saved_error
                .borrow()
                .as_ref()
                .expect("saved guest throw staged before loading")
                .clone();
            self.returned_saved_error.set(true);
            Poll::Ready(Err(error))
        })
    }
}

fn setup() -> (Context, Rc<StoppingLoader>, JsObject, JsValue) {
    let loader = Rc::new(StoppingLoader {
        saved_error: RefCell::new(None),
        called: Cell::new(false),
        observed_stop: Cell::new(false),
        returned_saved_error: Cell::new(false),
    });
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let saved_object = JsNativeError::error()
        .with_message("saved loader error")
        .into_opaque(context.access())
        .unwrap();
    let saved_value = JsValue::from(saved_object.clone());
    context
        .global_object()
        .unwrap()
        .set(
            js_string!("savedError"),
            saved_value.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let stack_before = saved_object
        .get(js_string!("stack"), context.access())
        .unwrap();
    assert_eq!(stack_before, JsValue::undefined());

    // Capture a real guest throw backtrace without materializing it onto the
    // preexisting Error. This makes later unwanted materialization observable.
    let saved_error = context
        .eval(
            Source::from_bytes("throw savedError")
                .with_path(Path::new("saved-loader-error-origin.js")),
        )
        .unwrap_err();
    assert_eq!(saved_error.as_opaque(), Some(&saved_value));
    assert!(
        saved_error
            .to_string()
            .contains("saved-loader-error-origin.js")
    );
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before
    );
    *loader.saved_error.borrow_mut() = Some(saved_error);
    (context, loader, saved_object, stack_before)
}

fn assert_retained_stop_precedes_loader_error_materialization(dynamic: bool) {
    let (mut context, loader, saved_object, stack_before) = setup();
    // Parse/import, promise creation and handler registration all happen outside
    // the selected zero-instruction operation. Only the queued drain is scoped.
    let loaded = if dynamic {
        JsPromise::from_object(
            context
                .eval(Source::from_bytes("import('dependency')"))
                .unwrap()
                .as_object()
                .unwrap(),
        )
        .unwrap()
    } else {
        Module::parse(
            Source::from_bytes("import 'dependency'"),
            None,
            context.access(),
        )
        .unwrap()
        .load(context.access())
        .unwrap()
    };
    context
        .global_object()
        .unwrap()
        .set(js_string!("loaded"), loaded, true, context.access())
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.rejected = false; loaded.catch(() => { rejected = true; })",
        ))
        .unwrap();
    assert!(!loader.called.get());

    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert!(
        loader.called.get(),
        "the scoped drain must reach the loader"
    );
    assert!(loader.observed_stop.get());
    assert!(loader.returned_saved_error.get());
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before,
        "retained host stop must precede materializing the loader's saved guest Error"
    );
    assert_eq!(
        context.eval(Source::from_bytes("rejected")).unwrap(),
        JsValue::new(false)
    );
    context.run_jobs().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("rejected")).unwrap(),
        JsValue::new(false),
        "the stopped loader completion must not enqueue a later guest rejection"
    );
}

#[test]
fn retained_stop_precedes_source_graph_loader_error_materialization() {
    assert_retained_stop_precedes_loader_error_materialization(false);
}

#[test]
fn retained_stop_precedes_dynamic_import_loader_error_materialization() {
    assert_retained_stop_precedes_loader_error_materialization(true);
}
