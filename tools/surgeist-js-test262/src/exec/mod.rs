//! Execution module for the test runner.

use surgeist_js_test_host::test262 as js262;

use crate::{
    Harness, Outcome, Phase, SpecEdition, Statistics, SuiteResult, Test, TestFlags,
    TestOutcomeResult, TestResult, TestSuite, VersionedStats, read::ErrorType,
};
use colored::Colorize;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::{cell::RefCell, eprintln, path::Path, rc::Rc};
use surgeist_js::{
    Context, JsArgs, JsError, JsNativeErrorKind, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    gc::heap::Trace,
    js_str,
    module::{Module, SimpleModuleLoader},
    native_function::{NativeCapture, NativeFunction},
    object::FunctionObjectBuilder,
    optimizer::OptimizerOptions,
    parser::source::ReadChar,
    property::Attribute,
    script::Script,
};

use js262::WorkerHandles;

fn render_result(passed: bool, rendered: JsResult<String>) -> (bool, String) {
    match rendered {
        Ok(text) => (passed, text),
        // Checked rendering refuses only with rootless host errors. Reporting
        // that cause does not attempt to access the original guest value again.
        Err(error) => (false, format!("Uncaught {error}")),
    }
}

fn error_result(passed: bool, error: &JsError) -> (bool, String) {
    render_result(
        passed,
        error.render().map(|text| format!("Uncaught {text}")),
    )
}

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        verbose: u8,
        parallel: bool,
        max_edition: SpecEdition,
        optimizer_options: OptimizerOptions,
        console: bool,
    ) -> SuiteResult {
        if verbose != 0 {
            println!("Suite {}:", self.path.display());
        }

        let suites: Vec<_> = if parallel {
            self.suites
                .par_iter()
                .map(|suite| {
                    suite.run(
                        harness,
                        verbose,
                        parallel,
                        max_edition,
                        optimizer_options,
                        console,
                    )
                })
                .collect()
        } else {
            self.suites
                .iter()
                .map(|suite| {
                    suite.run(
                        harness,
                        verbose,
                        parallel,
                        max_edition,
                        optimizer_options,
                        console,
                    )
                })
                .collect()
        };

        let tests: Vec<_> = if parallel {
            self.tests
                .par_iter()
                .filter(|test| test.edition <= max_edition)
                .map(|test| test.run(harness, verbose, optimizer_options, console))
                .collect()
        } else {
            self.tests
                .iter()
                .filter(|test| test.edition <= max_edition)
                .map(|test| test.run(harness, verbose, optimizer_options, console))
                .collect()
        };

        let mut features = FxHashSet::default();
        for test_iter in &*self.tests {
            features.extend(test_iter.features.iter().map(ToString::to_string));
        }

        if verbose != 0 {
            println!();
        }

        // Count passed tests and es specs
        let mut versioned_stats = VersionedStats::default();
        let mut es_next = Statistics::default();

        for test in &tests {
            match test.result {
                TestOutcomeResult::Passed => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.passed += 1;
                    });
                    es_next.passed += 1;
                }
                TestOutcomeResult::Ignored => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.ignored += 1;
                    });
                    es_next.ignored += 1;
                }
                TestOutcomeResult::Panic => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.panic += 1;
                    });
                    es_next.panic += 1;
                }
                TestOutcomeResult::Failed => {}
            }
            versioned_stats.apply(test.edition, |stats| {
                stats.total += 1;
            });
            es_next.total += 1;
        }

        // Count total tests
        for suite in &suites {
            versioned_stats += suite.versioned_stats;
            es_next += suite.stats;
            features.extend(suite.features.iter().cloned());
        }

        if verbose != 0 {
            println!(
                "Suite {} results: total: {}, passed: {}, ignored: {}, failed: {} {}, conformance: {:.2}%",
                self.path.display(),
                es_next.total,
                es_next.passed.to_string().green(),
                es_next.ignored.to_string().yellow(),
                (es_next.total - es_next.passed - es_next.ignored)
                    .to_string()
                    .red(),
                if es_next.panic == 0 {
                    String::new()
                } else {
                    format!("({})", format!("{} panics", es_next.panic).red())
                },
                (es_next.passed as f64 / es_next.total as f64) * 100.0
            );
        }
        SuiteResult {
            name: self.name.clone(),
            stats: es_next,
            versioned_stats,
            suites,
            tests,
            features,
        }
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        verbose: u8,
        optimizer_options: OptimizerOptions,
        console: bool,
    ) -> TestResult {
        if self.flags.contains(TestFlags::MODULE) || self.flags.contains(TestFlags::RAW) {
            return self.run_once(harness, false, verbose, optimizer_options, console);
        }

        if self
            .flags
            .contains(TestFlags::STRICT | TestFlags::NO_STRICT)
        {
            let r = self.run_once(harness, false, verbose, optimizer_options, console);
            if r.result != TestOutcomeResult::Passed {
                return r;
            }
            self.run_once(harness, true, verbose, optimizer_options, console)
        } else {
            self.run_once(
                harness,
                self.flags.contains(TestFlags::STRICT),
                verbose,
                optimizer_options,
                console,
            )
        }
    }

    /// Creates the test result from the outcome and message.
    fn create_result<S: Into<Box<str>>>(
        &self,
        outcome: TestOutcomeResult,
        text: S,
        strict: bool,
        verbosity: u8,
    ) -> TestResult {
        let result_text = text.into();

        if verbosity > 1 {
            println!(
                "`{}`{}: {}",
                self.path.display(),
                if strict { " (strict)" } else { "" },
                match outcome {
                    TestOutcomeResult::Passed => "Passed".green(),
                    TestOutcomeResult::Ignored => "Ignored".yellow(),
                    TestOutcomeResult::Failed => "Failed".red(),
                    TestOutcomeResult::Panic => "⚠ Panic ⚠".red(),
                }
            );
        } else {
            let symbol = match outcome {
                TestOutcomeResult::Passed => ".".green(),
                TestOutcomeResult::Ignored => "-".yellow(),
                TestOutcomeResult::Failed | TestOutcomeResult::Panic => "F".red(),
            };

            print!("{symbol}");
        }

        if verbosity > 2 {
            println!(
                "`{}`{}: result text\n{result_text}\n",
                self.path.display(),
                if strict { " (strict)" } else { "" },
            );
        }

        TestResult {
            name: self.name.clone(),
            edition: self.edition,
            result_text,
            result: outcome,
        }
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(
        &self,
        harness: &Harness,
        strict: bool,
        verbosity: u8,
        optimizer_options: OptimizerOptions,
        console: bool,
    ) -> TestResult {
        let Ok(source) = Source::from_filepath(&self.path) else {
            return self.create_result(
                TestOutcomeResult::Failed,
                "Could not read test file",
                strict,
                verbosity,
            );
        };

        if self.ignored {
            return self.create_result(TestOutcomeResult::Ignored, "", strict, verbosity);
        }

        if verbosity > 1 {
            println!(
                "`{}`{}: starting",
                self.path.display(),
                if strict { " (strict mode)" } else { "" }
            );
        }

        let result = std::panic::catch_unwind(|| match self.expected_outcome {
            Outcome::Positive => {
                let (ref mut context, async_result, mut handles) =
                    match self.create_context(harness, optimizer_options, console) {
                        Ok(r) => r,
                        Err(e) => return (false, e),
                    };

                // TODO: timeout
                let value = if self.is_module() {
                    let module = match parse_module_and_register(source, &self.path, context) {
                        Ok(module) => module,
                        Err(err) => return error_result(false, &err),
                    };

                    let promise = match module.load_link_evaluate(context.access()) {
                        Ok(promise) => promise,
                        Err(err) => return error_result(false, &err),
                    };

                    if let Err(err) = context.run_jobs() {
                        return error_result(false, &err);
                    }

                    match promise.state() {
                        Ok(PromiseState::Pending) => {
                            return (false, "module should have been executed".to_string());
                        }
                        Ok(PromiseState::Fulfilled(v)) => v,
                        Ok(PromiseState::Rejected(err)) => {
                            let error = match JsError::from_opaque(err.clone()) {
                                Ok(error) => error,
                                Err(error) => {
                                    return error_result(false, &JsError::from(error));
                                }
                            };
                            let output = match error.try_native(context.access()) {
                                Ok(error) => {
                                    let cause = match error.cause() {
                                        Some(cause) => match cause.render() {
                                            Ok(text) => format!("\n  caused by {text}"),
                                            Err(error) => return render_result(false, Err(error)),
                                        },
                                        None => String::new(),
                                    };
                                    format!("Uncaught {error}{cause}")
                                }
                                Err(_) => match err.display().render() {
                                    Ok(rendered) => format!("Uncaught {rendered}"),
                                    Err(error) => return (false, format!("Uncaught {error}")),
                                },
                            };

                            return (false, output);
                        }
                        Err(err) => return error_result(false, &err),
                    }
                } else {
                    context.strict(strict);
                    match context.eval(source) {
                        Ok(v) => v,
                        Err(err) => return error_result(false, &err),
                    }
                };

                if let Err(err) = context.run_jobs() {
                    return error_result(false, &err);
                }

                match *async_result.inner.borrow() {
                    UninitResult::Err(ref e) => return (false, format!("Uncaught {e}")),
                    UninitResult::Uninit if self.flags.contains(TestFlags::ASYNC) => {
                        return (
                            false,
                            "async test did not print \"Test262:AsyncTestComplete\"".to_string(),
                        );
                    }
                    _ => {}
                }

                for result in handles.join_all() {
                    match result {
                        js262::WorkerResult::Err(msg) => return (false, msg),
                        js262::WorkerResult::Panic(msg) => panic!("Worker thread panicked: {msg}"),
                        js262::WorkerResult::Ok => {}
                    }
                }

                render_result(true, value.display().render())
            }
            Outcome::Negative {
                phase: Phase::Parse,
                error_type,
            } => {
                assert_eq!(
                    error_type,
                    ErrorType::SyntaxError,
                    "non-SyntaxError parsing/early error found in {}",
                    self.path.display()
                );

                let context = &mut match Context::builder().build() {
                    Ok(context) => context,
                    Err(error) => return error_result(false, &error),
                };

                if self.is_module() {
                    match Module::parse(source, None, context.access()) {
                        Ok(_) => (false, "ModuleItemList parsing should fail".to_owned()),
                        Err(e) => error_result(true, &e),
                    }
                } else {
                    context.strict(strict);
                    match Script::parse(source, None, context.access()) {
                        Ok(_) => (false, "StatementList parsing should fail".to_owned()),
                        Err(e) => error_result(true, &e),
                    }
                }
            }
            Outcome::Negative {
                phase: Phase::Resolution,
                error_type,
            } => {
                let context = &mut match self.create_context(harness, optimizer_options, console) {
                    Ok(r) => r,
                    Err(e) => return (false, e),
                }
                .0;

                let module = match parse_module_and_register(source, &self.path, context) {
                    Ok(module) => module,
                    Err(err) => return error_result(false, &err),
                };

                let promise = match module.load(context.access()) {
                    Ok(promise) => promise,
                    Err(err) => return error_result(false, &err),
                };

                if let Err(err) = context.run_jobs() {
                    return error_result(false, &err);
                }

                match promise.state() {
                    Ok(PromiseState::Pending) => {
                        return (false, "module didn't try to load".to_string());
                    }
                    Ok(PromiseState::Fulfilled(_)) => {
                        // Try to link to see if the resolution error shows there.
                    }
                    Ok(PromiseState::Rejected(err)) => {
                        let err = match JsError::from_opaque(err) {
                            Ok(error) => error,
                            Err(error) => {
                                return error_result(false, &JsError::from(error));
                            }
                        };
                        return error_result(is_error_type(&err, error_type, context), &err);
                    }
                    Err(err) => return error_result(false, &err),
                }

                if let Err(err) = module.link(context.access()) {
                    error_result(is_error_type(&err, error_type, context), &err)
                } else {
                    (false, "module resolution didn't fail".to_string())
                }
            }
            Outcome::Negative {
                phase: Phase::Runtime,
                error_type,
            } => {
                let (ref mut context, _async_result, mut handles) =
                    match self.create_context(harness, optimizer_options, console) {
                        Ok(r) => r,
                        Err(e) => return (false, e),
                    };

                let error = if self.is_module() {
                    let module = match parse_module_and_register(source, &self.path, context) {
                        Ok(module) => module,
                        Err(err) => return error_result(false, &err),
                    };

                    let promise = match module.load(context.access()) {
                        Ok(promise) => promise,
                        Err(err) => return error_result(false, &err),
                    };

                    if let Err(err) = context.run_jobs() {
                        return error_result(false, &err);
                    }

                    match promise.state() {
                        Ok(PromiseState::Pending) => {
                            return (false, "module didn't try to load".to_string());
                        }
                        Ok(PromiseState::Fulfilled(_)) => {}
                        Ok(PromiseState::Rejected(err)) => {
                            return match err.display().render() {
                                Ok(rendered) => (false, format!("Uncaught {rendered}")),
                                Err(error) => (false, format!("Uncaught {error}")),
                            };
                        }
                        Err(err) => return error_result(false, &err),
                    }

                    if let Err(err) = module.link(context.access()) {
                        return error_result(false, &err);
                    }

                    let promise = match module.evaluate(context.access()) {
                        Ok(p) => p,
                        Err(err) => return error_result(false, &err),
                    };

                    if let Err(err) = context.run_jobs() {
                        return error_result(false, &err);
                    }

                    match promise.state() {
                        Ok(PromiseState::Pending) => {
                            return (false, "module didn't try to evaluate".to_string());
                        }
                        Ok(PromiseState::Fulfilled(val)) => {
                            return match val.display().render() {
                                Ok(rendered) => (false, rendered),
                                Err(error) => (false, format!("Uncaught {error}")),
                            };
                        }
                        Ok(PromiseState::Rejected(err)) => match JsError::from_opaque(err) {
                            Ok(error) => error,
                            Err(error) => {
                                return error_result(false, &JsError::from(error));
                            }
                        },
                        Err(err) => return error_result(false, &err),
                    }
                } else {
                    context.strict(strict);
                    let script = match Script::parse(source, None, context.access()) {
                        Ok(code) => code,
                        Err(e) => return error_result(false, &e),
                    };

                    match script.evaluate(context.access()) {
                        Ok(_) => return (false, "Script execution should fail".to_owned()),
                        Err(e) => e,
                    }
                };

                for result in handles.join_all() {
                    match result {
                        js262::WorkerResult::Err(msg) => return (false, msg),
                        js262::WorkerResult::Panic(msg) => panic!("Worker thread panicked: {msg}"),
                        js262::WorkerResult::Ok => {}
                    }
                }

                error_result(is_error_type(&error, error_type, context), &error)
            }
        });

        let (result, result_text) = result.map_or_else(
            |_| {
                eprintln!("last panic was on test \"{}\"", self.path.display());
                (TestOutcomeResult::Panic, String::new())
            },
            |(res, text)| {
                if res {
                    (TestOutcomeResult::Passed, text)
                } else {
                    (TestOutcomeResult::Failed, text)
                }
            },
        );

        self.create_result(result, result_text, strict, verbosity)
    }

    /// Creates the context to run the test.
    fn create_context(
        &self,
        harness: &Harness,
        optimizer_options: OptimizerOptions,
        console: bool,
    ) -> Result<(Context, AsyncResult, WorkerHandles), String> {
        let async_result = AsyncResult::default();
        let handles = WorkerHandles::new();
        let loader = Rc::new(
            SimpleModuleLoader::new(self.path.parent().expect("test should have a parent dir"))
                .expect("test path should be canonicalizable"),
        );
        let mut context = Context::builder()
            .module_loader(loader.clone())
            .can_block(!self.flags.contains(TestFlags::CAN_BLOCK_IS_FALSE))
            .build()
            .map_err(|error| render_result(false, error.render()).1)?;

        context.set_optimizer_options(optimizer_options);

        // Register the print() function.
        register_print_fn(&mut context, async_result.clone())
            .map_err(|e| render_result(false, e.render()).1)?;

        // add the $262 object.
        let _js262 = js262::register_js262(handles.clone(), console, context.access())
            .map_err(|e| render_result(false, e.render()).1)?;

        if console {
            let console = surgeist_js_test_host::Console::init(context.access())
                .map_err(|e| render_result(false, e.render()).1)?;
            context
                .register_global_property(
                    surgeist_js_test_host::Console::NAME,
                    console,
                    Attribute::all(),
                )
                .expect("the console builtin shouldn't exist");
        }

        if self.flags.contains(TestFlags::RAW) {
            return Ok((context, async_result, handles));
        }

        let assert = Source::from_reader(
            harness.assert.content.as_bytes(),
            Some(&harness.assert.path),
        );
        let sta = Source::from_reader(harness.sta.content.as_bytes(), Some(&harness.sta.path));

        context.eval(assert).map_err(|e| {
            render_result(
                false,
                e.render()
                    .map(|text| format!("could not run assert.js:\n{text}")),
            )
            .1
        })?;
        context.eval(sta).map_err(|e| {
            render_result(
                false,
                e.render()
                    .map(|text| format!("could not run sta.js:\n{text}")),
            )
            .1
        })?;

        if self.flags.contains(TestFlags::ASYNC) {
            let dph = Source::from_reader(
                harness.doneprint_handle.content.as_bytes(),
                Some(&harness.doneprint_handle.path),
            );
            context.eval(dph).map_err(|e| {
                render_result(
                    false,
                    e.render()
                        .map(|text| format!("could not run doneprintHandle.js:\n{text}")),
                )
                .1
            })?;
        }

        for include_name in &self.includes {
            let include = harness
                .includes
                .get(include_name)
                .ok_or_else(|| format!("could not find the {include_name} include file."))?;
            let source = Source::from_reader(include.content.as_bytes(), Some(&include.path));
            context.eval(source).map_err(|e| {
                render_result(
                    false,
                    e.render().map(|text| {
                        format!("could not run the harness `{include_name}`:\nUncaught {text}")
                    }),
                )
                .1
            })?;
        }

        Ok((context, async_result, handles))
    }
}

/// Returns `true` if `error` is a `target_type` error.
fn is_error_type(error: &JsError, target_type: ErrorType, context: &mut Context) -> bool {
    if let Ok(error) = error.try_native(context.access()) {
        match error.kind() {
            JsNativeErrorKind::Syntax if target_type == ErrorType::SyntaxError => {}
            JsNativeErrorKind::Reference if target_type == ErrorType::ReferenceError => {}
            JsNativeErrorKind::Range if target_type == ErrorType::RangeError => {}
            JsNativeErrorKind::Type if target_type == ErrorType::TypeError => {}
            _ => return false,
        }
        true
    } else {
        error
            .as_opaque()
            .expect("try_native cannot fail if e is not opaque")
            .as_object()
            .and_then(|o| o.get(js_str!("constructor"), context.access()).ok())
            .as_ref()
            .and_then(JsValue::as_object)
            .and_then(|o| o.get(js_str!("name"), context.access()).ok())
            .as_ref()
            .and_then(JsValue::as_string)
            .is_some_and(|s| s == target_type.as_str())
    }
}

/// Registers the print function in the context.
fn register_print_fn(context: &mut Context, async_result: AsyncResult) -> JsResult<()> {
    let js_function = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_with_captures(
            |_, args, async_result, mut context| {
                let message = args
                    .get_or_undefined(0)
                    .to_string(context.reborrow())?
                    .to_std_string_escaped();
                let mut result = async_result.inner.borrow_mut();

                match *result {
                    UninitResult::Uninit | UninitResult::Ok(()) => {
                        if message == "Test262:AsyncTestComplete" {
                            *result = UninitResult::Ok(());
                        } else {
                            *result = UninitResult::Err(message);
                        }
                    }
                    UninitResult::Err(_) => {}
                }

                Ok(JsValue::undefined())
            },
            async_result,
        ),
    )
    .name("print")
    .length(1)
    .build()?;

    context.register_global_property(
        js_str!("print"),
        js_function,
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
    )
}

/// Parses a module and registers it into the `ModuleLoader` of the context.
fn parse_module_and_register(
    source: Source<'_, impl ReadChar>,
    path: &Path,
    context: &mut Context,
) -> JsResult<Module> {
    let module = Module::parse(source, None, context.access())?;

    let path = path
        .canonicalize()
        .expect("test path should be canonicalizable");

    let loader = context
        .downcast_module_loader::<SimpleModuleLoader>()
        .expect("context must use a SimpleModuleLoader");

    loader.insert(path, module.clone());

    Ok(module)
}

/// A `Result` value that is possibly uninitialized.
///
/// This is mainly used to check if an async test did call `print` to signal the termination of
/// a test. Otherwise, all async tests that result in `UninitResult::Uninit` are considered
/// failures.
///
/// The Test262 [interpreting guide][guide] contains more information about how to run async tests.
///
/// [guide]: https://github.com/tc39/test262/blob/main/INTERPRETING.md#flags
#[derive(Debug, Clone, Default)]
enum UninitResult<T, E> {
    #[default]
    Uninit,
    Ok(T),
    Err(E),
}

/// Object which includes the result of the async operation.
#[derive(Debug, Clone, Default, Trace)]
struct AsyncResult {
    #[heap_trace(skip)]
    inner: Rc<RefCell<UninitResult<(), String>>>,
}

impl NativeCapture for AsyncResult {
    type Stored = Self;

    fn store(&self) -> Self {
        self.clone()
    }

    fn restore(stored: &Self) -> Result<Self, surgeist_js::error::RootingError> {
        Ok(stored.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ErrorType, Harness, OptimizerOptions, Outcome, Path, Phase, Test, TestOutcomeResult,
        error_result,
    };
    use surgeist_js::{Context, JsError, JsObject};

    #[test]
    fn builder_refusal_fails_the_runner_instead_of_matching_expected_guest_type_error() {
        let metadata =
            serde_yaml::from_str("description: builder refusal\nflags: [raw]\n").unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/exec/mod.rs");
        let mut test = Test::new("builder refusal", path.into_boxed_path(), metadata).unwrap();
        test.expected_outcome = Outcome::Negative {
            phase: Phase::Runtime,
            error_type: ErrorType::TypeError,
        };
        let empty = crate::HarnessFile {
            content: "".into(),
            path: Path::new("unused").into(),
        };
        let harness = Harness {
            assert: empty.clone(),
            sta: empty.clone(),
            doneprint_handle: empty,
            includes: Default::default(),
        };
        let blocker = Context::builder().can_block(false).build().unwrap();

        let result = test.run_once(&harness, false, 0, OptimizerOptions::empty(), false);
        assert_eq!(result.result, TestOutcomeResult::Failed);
        assert_eq!(
            result.result_text.as_ref(),
            "TypeError: a context that can block must be the only active context in its current thread"
        );
        drop(blocker);

        assert!(
            test.create_context(&harness, OptimizerOptions::empty(), false)
                .is_ok()
        );
    }

    #[test]
    fn borrowed_error_rendering_cannot_report_an_expected_guest_failure_as_passed() {
        let context = Context::builder().build().unwrap();
        let object = JsObject::with_null_proto(context.runtime()).unwrap();
        let error = JsError::from_opaque(object.clone().into()).unwrap();
        let expected = error_result(true, &error);
        assert!(expected.0);

        let alias = object.clone();
        let held = alias.borrow_mut().unwrap();
        let refused = error_result(true, &error);
        assert_eq!(
            refused,
            (
                false,
                "Uncaught ObjectAccessError: object access is already borrowed".to_string(),
            )
        );
        drop(held);

        assert_eq!(error_result(true, &error), expected);
    }
}
