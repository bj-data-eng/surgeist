#![allow(unused_crate_dependencies)]

use std::{
    cell::Cell,
    future::Future,
    io::{self, Read},
    path::Path,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, IntoJsModule, JsError, JsString, Module, NativeFunction, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ProfileMode},
    gc::heap::Trace,
    js_string,
    module::SyntheticModuleInitializer,
    native_function::NativeCapture,
};

const LIMIT: u64 = 100_000;
fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn options(limit: u64) -> ExecutionOptions {
    ExecutionOptions::new(LIMIT).with_module_metadata_work_limit(limit)
}
fn refusal(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::ModuleMetadataWork { limit }
        ))
    );
}

#[derive(Default)]
struct Observation {
    stores: Cell<usize>,
    advanced: Cell<bool>,
}
#[derive(Clone)]
struct Capture {
    observation: Rc<Observation>,
    panic: bool,
}
#[derive(Trace)]
struct StoredCapture {
    #[heap_trace(skip)]
    observation: Rc<Observation>,
    panic: bool,
}
impl NativeCapture for Capture {
    type Stored = StoredCapture;
    fn store(&self) -> StoredCapture {
        self.observation
            .stores
            .set(self.observation.stores.get() + 1);
        if !self.observation.advanced.replace(true) {
            TIME.set(TIME.get() + Duration::from_millis(7));
        }
        if self.panic {
            panic!("authored metadata capture unwind");
        }
        StoredCapture {
            observation: self.observation.clone(),
            panic: self.panic,
        }
    }
    fn restore(stored: &StoredCapture) -> Result<Self, surgeist_js::error::RootingError> {
        Ok(Self {
            observation: stored.observation.clone(),
            panic: stored.panic,
        })
    }
}
fn initializer(observation: Rc<Observation>, panic: bool) -> SyntheticModuleInitializer {
    SyntheticModuleInitializer::from_fn_with_captures(
        |_, _, _| Ok(()),
        Capture { observation, panic },
    )
}

#[derive(Default)]
struct InputObservation {
    conversions: Cell<usize>,
    pulls: Cell<usize>,
    eof: Cell<bool>,
}
#[derive(Clone)]
struct Input {
    observation: Rc<InputObservation>,
    item: Option<(JsString, NativeFunction)>,
}
struct InputIterator {
    observation: Rc<InputObservation>,
    item: Option<(JsString, NativeFunction)>,
}
impl IntoIterator for Input {
    type Item = (JsString, NativeFunction);
    type IntoIter = InputIterator;
    fn into_iter(self) -> InputIterator {
        self.observation
            .conversions
            .set(self.observation.conversions.get() + 1);
        InputIterator {
            observation: self.observation,
            item: self.item,
        }
    }
}
impl Iterator for InputIterator {
    type Item = (JsString, NativeFunction);
    fn next(&mut self) -> Option<Self::Item> {
        self.observation.pulls.set(self.observation.pulls.get() + 1);
        let item = self.item.take();
        if item.is_none() {
            self.observation.eof.set(true);
        }
        item
    }
}
fn input(observation: Rc<InputObservation>, nonempty: bool) -> Input {
    Input {
        observation,
        item: nonempty.then(|| {
            (
                js_string!("answer"),
                NativeFunction::from_fn_ptr(|_, _, _| Ok(42.into())),
            )
        }),
    }
}

#[test]
fn zero_metadata_cap_refuses_source_synthetic_and_native_construction_without_host_consumption() {
    let mut context = context();
    let source = context
        .with_execution(options(0), |access| {
            Module::parse(
                Source::from_bytes("export const answer = 42;"),
                None,
                access,
            )
        })
        .unwrap();
    refusal(&source.result.unwrap_err(), 0);
    assert_eq!(source.report.module_metadata_work().unwrap(), 0);
    assert!(!source.report.effects_started());
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation.clone(), false);
    let synthetic = context
        .with_execution(options(0), |access| {
            Module::synthetic(&[js_string!("answer")], steps, None, None, access)
        })
        .unwrap();
    refusal(&synthetic.result.unwrap_err(), 0);
    assert_eq!(synthetic.report.module_metadata_work().unwrap(), 0);
    assert!(!synthetic.report.effects_started());
    assert_eq!(observation.stores.get(), 0);
    let observed = Rc::new(InputObservation::default());
    let exports = input(observed.clone(), true);
    let native = context
        .with_execution(options(0), |access| exports.into_js_module(access))
        .unwrap();
    refusal(&native.result.unwrap_err(), 0);
    assert_eq!(native.report.module_metadata_work().unwrap(), 0);
    assert!(!native.report.effects_started());
    assert_eq!(observed.conversions.get(), 0);
    assert_eq!(observed.pulls.get(), 0);
    assert!(!observed.eof.get());
}

#[test]
fn native_iterator_prefix_is_one_state_record_then_one_visit_before_each_pull() {
    for nonempty in [false, true] {
        for cap in [0, 1, 2] {
            let mut context = context();
            let observed = Rc::new(InputObservation::default());
            let exports = input(observed.clone(), nonempty);
            let outcome = context
                .with_execution(options(cap), |access| exports.into_js_module(access))
                .unwrap();
            refusal(&outcome.result.unwrap_err(), cap);
            assert_eq!(outcome.report.module_metadata_work().unwrap(), cap);
            assert!(!outcome.report.effects_started());
            assert_eq!(observed.conversions.get(), usize::from(cap >= 1));
            assert_eq!(observed.pulls.get(), usize::from(cap >= 2));
            assert_eq!(observed.eof.get(), !nonempty && cap >= 2);
            // At cap two there is no credit for final Module publication, even if
            // the first pull is EOF. A nonempty item instead needs further owned
            // record/string/storage work before a second Visit can be admitted.
        }
    }
}

#[test]
fn metadata_refusal_is_sticky_for_direct_constructors_and_a_fresh_attempt_remains_possible() {
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation.clone(), false);
    let observed = Rc::new(InputObservation::default());
    let exports = input(observed.clone(), true);
    let mut direct = None;
    let outcome = context
        .with_execution(options(0), |mut access| {
            let first = Module::parse(Source::from_bytes("export {};"), None, access.reborrow())
                .unwrap_err();
            refusal(&first, 0);
            let second = Module::synthetic(
                &[js_string!("answer")],
                steps,
                None,
                None,
                access.reborrow(),
            )
            .unwrap_err();
            let third = exports.into_js_module(access).unwrap_err();
            assert_eq!(second, first);
            assert_eq!(third, first);
            direct = Some(first);
            Ok(())
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap_err(), direct.unwrap());
    assert_eq!(outcome.report.module_metadata_work().unwrap(), 0);
    assert_eq!(observation.stores.get(), 0);
    assert_eq!(observed.conversions.get(), 0);
    assert_eq!(observed.pulls.get(), 0);
    let retry = context
        .with_execution(ExecutionOptions::new(LIMIT), |access| {
            Module::synthetic(
                &[
                    js_string!("answer"),
                    js_string!("answer"),
                    js_string!("長🙂"),
                ],
                initializer(observation.clone(), false),
                None,
                None,
                access,
            )
        })
        .unwrap();
    retry.result.unwrap();
    assert!(retry.report.module_metadata_work().unwrap() > 0);
    assert!(!retry.report.effects_started());
    assert!(observation.stores.get() > 0);
}

#[test]
fn a_current_instruction_stop_precedes_metadata_refusal_without_capture_or_iterator_conversion() {
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation.clone(), false);
    let observed = Rc::new(InputObservation::default());
    let exports = input(observed.clone(), true);
    let mut first = None;
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0).with_module_metadata_work_limit(0),
            |mut access| {
                let prior = access.eval(Source::from_bytes("0")).unwrap_err();
                assert_eq!(
                    prior.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 0
                    }))
                );
                assert_eq!(
                    Module::synthetic(&[], steps, None, None, access.reborrow()).unwrap_err(),
                    prior
                );
                assert_eq!(exports.into_js_module(access).unwrap_err(), prior);
                first = Some(prior);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap_err(), first.unwrap());
    assert_eq!(outcome.report.module_metadata_work().unwrap(), 0);
    assert_eq!(observation.stores.get(), 0);
    assert_eq!(observed.conversions.get(), 0);
    assert_eq!(observed.pulls.get(), 0);
}

#[test]
fn pure_metadata_refusal_preserves_an_effect_already_started_by_the_outer_operation() {
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation.clone(), false);
    let outcome = context
        .with_execution(options(0), |mut access| {
            access.eval(Source::from_bytes("globalThis.outerMetadataEffect = 1"))?;
            Module::synthetic(&[js_string!("answer")], steps, None, None, access)
        })
        .unwrap();
    refusal(&outcome.result.unwrap_err(), 0);
    assert!(outcome.report.effects_started());
    assert_eq!(outcome.report.module_metadata_work().unwrap(), 0);
    assert_eq!(observation.stores.get(), 0);
    assert_eq!(
        context
            .global_object()
            .get(js_string!("outerMetadataEffect"), context.access())
            .unwrap(),
        1.into()
    );
}

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static CLOCK_READS: Cell<usize> = const { Cell::new(0) };
}
fn clock() -> Duration {
    CLOCK_READS.set(CLOCK_READS.get() + 1);
    TIME.get()
}
fn modes() -> [ProfileMode; 5] {
    [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::ScopeName("module-metadata"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::ScopeName("module-metadata"),
        },
    ]
}
fn timed(mode: ProfileMode) -> bool {
    matches!(
        mode,
        ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
    )
}

struct DelayedReader<'a> {
    bytes: &'a [u8],
    advanced: bool,
    reads: &'a Cell<usize>,
}
impl Read for DelayedReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if !self.advanced {
            self.advanced = true;
            TIME.set(TIME.get() + Duration::from_millis(100));
        }
        self.bytes.read(output)
    }
}

#[test]
fn source_path_metadata_gate_refuses_before_the_reader_is_consumed() {
    let mut context = context();
    let reads = Cell::new(0);
    let source = Source::from_reader(
        DelayedReader {
            bytes: b"export {};",
            advanced: false,
            reads: &reads,
        },
        Some(Path::new("metadata-source.js")),
    );
    let outcome = context
        .with_execution(options(0), |access| Module::parse(source, None, access))
        .unwrap();
    refusal(&outcome.result.unwrap_err(), 0);
    assert_eq!(outcome.report.module_metadata_work().unwrap(), 0);
    assert_eq!(reads.get(), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn source_metadata_segments_exclude_reader_time_and_untimed_modes_read_no_clock() {
    for mode in modes() {
        let mut context = context();
        let reads = Cell::new(0);
        let path = Path::new("metadata-source.js");
        let source = Source::from_reader(
            DelayedReader {
                bytes: b"export const answer = 42;",
                advanced: false,
                reads: &reads,
            },
            Some(path),
        );
        TIME.set(Duration::ZERO);
        CLOCK_READS.set(0);
        let outcome = context
            .with_execution(
                ExecutionOptions::new(LIMIT)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                |access| Module::parse(source, None, access),
            )
            .unwrap();
        let module = outcome.result.unwrap();
        assert_eq!(module.path(), Some(path));
        assert!(reads.get() > 0);
        assert!(outcome.report.module_metadata_work().unwrap() > 0);
        assert!(!outcome.report.effects_started());
        if mode == ProfileMode::Off {
            assert!(outcome.report.profile().unwrap().is_none());
        } else {
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(
                profile.elapsed,
                timed(mode).then_some(Duration::from_millis(100))
            );
            let metadata: Vec<_> = profile
                .scopes
                .iter()
                .filter(|scope| scope.name == "module-metadata")
                .collect();
            assert!(
                metadata.iter().map(|scope| scope.phases).sum::<u64>() >= 2,
                "path work occurs in distinct before-parser and after-parser segments"
            );
            assert_eq!(
                metadata
                    .iter()
                    .filter_map(|scope| scope.phase_elapsed)
                    .sum::<Duration>(),
                Duration::ZERO
            );
            assert!(
                metadata
                    .iter()
                    .all(|scope| scope.phase_elapsed.is_some() == timed(mode))
            );
        }
        if !timed(mode) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}

#[test]
fn lawful_capture_storage_is_inside_the_metadata_phase_but_the_later_host_gap_is_not() {
    for mode in modes() {
        let mut context = context();
        let observation = Rc::new(Observation::default());
        let steps = initializer(observation.clone(), false);
        TIME.set(Duration::ZERO);
        CLOCK_READS.set(0);
        let outcome = context
            .with_execution(
                ExecutionOptions::new(LIMIT)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                |access| {
                    let module =
                        Module::synthetic(&[js_string!("captured")], steps, None, None, access)?;
                    TIME.set(TIME.get() + Duration::from_millis(100));
                    Ok(module)
                },
            )
            .unwrap();
        outcome.result.unwrap();
        assert!(observation.stores.get() > 0);
        assert!(!outcome.report.effects_started());
        if mode == ProfileMode::Off {
            assert!(outcome.report.profile().unwrap().is_none());
        } else {
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(
                profile.elapsed,
                timed(mode).then_some(Duration::from_millis(107))
            );
            let metadata: Vec<_> = profile
                .scopes
                .iter()
                .filter(|scope| scope.name == "module-metadata")
                .collect();
            assert!(metadata.iter().map(|scope| scope.phases).sum::<u64>() > 0);
            if timed(mode) {
                assert_eq!(
                    metadata
                        .iter()
                        .filter_map(|scope| scope.phase_elapsed)
                        .sum::<Duration>(),
                    Duration::from_millis(7)
                );
            } else {
                assert!(metadata.iter().all(|scope| scope.phase_elapsed.is_none()));
            }
        }
        if !timed(mode) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}

#[test]
fn capture_unwind_closes_metadata_timing_and_preserves_pure_construction_effects() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation.clone(), true);
    TIME.set(Duration::ZERO);
    let mut task = context
        .execution_task(
            ExecutionOptions::new(LIMIT)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
            async move |access| {
                Module::synthetic(&[js_string!("captured")], steps, None, None, access)
            },
        )
        .unwrap();
    let receipt = task.receipt();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = Pin::new(&mut task).poll(&mut TaskContext::from_waker(Waker::noop()));
    }))
    .unwrap_err();
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"authored metadata capture unwind")
    );
    TIME.set(TIME.get() + Duration::from_millis(100));
    drop(task);
    let report = receipt.report().unwrap();
    assert!(report.module_metadata_work().unwrap() > 0);
    assert!(!report.effects_started());
    assert!(observation.stores.get() > 0);
    let profile = report.profile().unwrap().unwrap();
    assert_eq!(
        profile
            .scopes
            .iter()
            .filter(|scope| scope.name == "module-metadata")
            .filter_map(|scope| scope.phase_elapsed)
            .sum::<Duration>(),
        Duration::from_millis(7)
    );
}

#[test]
fn synthetic_spelling_work_does_not_consume_ast_name_credits() {
    let mut context = context();
    let observation = Rc::new(Observation::default());
    let steps = initializer(observation, false);
    let outcome = context
        .with_execution(
            ExecutionOptions::new(LIMIT)
                .with_name_visit_limit(0)
                .with_name_record_limit(0)
                .with_name_comparison_limit(0),
            |access| {
                Module::synthetic(
                    &[
                        js_string!("answer"),
                        js_string!("answer"),
                        js_string!("長🙂"),
                    ],
                    steps,
                    None,
                    None,
                    access,
                )
            },
        )
        .unwrap();
    outcome.result.unwrap();
    assert!(outcome.report.module_metadata_work().unwrap() > 0);
    assert_eq!(outcome.report.name_visits().unwrap(), 0);
    assert_eq!(outcome.report.name_records().unwrap(), 0);
    assert_eq!(outcome.report.name_comparisons().unwrap(), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn source_name_refusal_remains_separate_and_before_parser_path_work_wins_when_both_caps_are_zero() {
    let mut context = context();
    let names = context
        .with_execution(
            ExecutionOptions::new(LIMIT).with_name_record_limit(0),
            |access| Module::parse(Source::from_bytes("export const named = 1;"), None, access),
        )
        .unwrap();
    assert_eq!(
        names.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::NameRecord {
            limit: 0
        }))
    );
    assert_eq!(names.report.name_records().unwrap(), 0);
    assert!(!names.report.effects_started());
    let both = context
        .with_execution(options(0).with_name_record_limit(0), |access| {
            Module::parse(
                Source::from_bytes("export const named = 1;").with_path(Path::new("a")),
                None,
                access,
            )
        })
        .unwrap();
    refusal(&both.result.unwrap_err(), 0);
    assert_eq!(both.report.module_metadata_work().unwrap(), 0);
    assert_eq!(both.report.name_records().unwrap(), 0);
    assert!(!both.report.effects_started());
}
