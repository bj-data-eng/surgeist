//! Behavioral qualification of compiler admission at its real entry adapters.

use std::{collections::HashSet, convert::Infallible, rc::Rc};

use surgeist_js_ast::scope::{BindingLocator, PreparedScopes, ScopeAnalysisError};

use crate::{
    Context, JsResult, JsValue, Module, NativeFunction, Script, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionOutcome, ExecutionReport, ExecutionWork, ProfileMode},
    js_string,
    module::{MapModuleLoader, SyntheticModuleInitializer},
    native_function::NativeCapture,
    object::{JsFunction, JsPromise},
    property::Attribute,
    script::ScriptEdge,
    vm::{ActivatedCode, CodeBlock, Constant, InstructionIterator},
};

const DRIVER: &str = "entry()";
const PENDING: &str = "
    let pending = existing + (function add(value = 37) { return value; })();
    pending;
";
const AFTER_CALLBACK: &str = "let pending = (() => between + 1)(); pending;";
const NOOP: u32 = 0;
const CODE_ONCE: u32 = 1;
const CODE_TWICE: u32 = 2;
const CODE_AROUND_EFFECT: u32 = 3;

/// Reuses Script's existing traceable custody; the callback retains no hidden roots.
struct ScriptCapture(Script);

impl NativeCapture for ScriptCapture {
    type Stored = ScriptEdge;

    fn store(&self) -> Self::Stored {
        self.0.edge()
    }

    fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
        Ok(Self(Script::from_edge(stored.clone())?))
    }
}

fn script_entry(
    _: &JsValue,
    _: &[JsValue],
    (script, action): &(ScriptCapture, u32),
    mut context: crate::ContextAccess<'_>,
) -> JsResult<JsValue> {
    if *action == NOOP {
        return Ok(42.into());
    }
    let first = script.0.codeblock(context.reborrow())?;
    if *action == CODE_AROUND_EFFECT {
        context.eval(Source::from_bytes("let between = 41;"))?;
        let refreshed = script.0.codeblock(context.reborrow())?;
        assert!(!Rc::ptr_eq(&first, &refreshed));
        assert!(Rc::ptr_eq(
            &refreshed,
            &script.0.codeblock(context.reborrow())?
        ));
    } else if *action == CODE_TWICE {
        assert!(Rc::ptr_eq(&first, &script.0.codeblock(context.reborrow())?));
    }
    Ok(42.into())
}

fn next_scope_identity(context: &Context) -> u32 {
    // Observe the next canonical identity using real private staging. Dropping
    // this proposal neither spends an identity nor changes a pending basis.
    let mut scopes = PreparedScopes::new(context.realm().scope());
    scopes
        .with_access::<_, Infallible, ScopeAnalysisError<Infallible>>(None, |access| {
            let root = access.root_handle()?;
            let (_, names) = access.create_module_scope(&root, &[js_string!("identityProbe")])?;
            Ok(names[0].unique_scope_id())
        })
        .unwrap()
}

struct RealmSnapshot {
    bindings: u32,
    next_identity: u32,
    existing: BindingLocator,
}

impl RealmSnapshot {
    fn new(context: &Context) -> Self {
        Self {
            bindings: context.realm().scope().num_bindings(),
            next_identity: next_scope_identity(context),
            existing: context
                .realm()
                .scope()
                .get_binding(&js_string!("existing"))
                .unwrap(),
        }
    }

    fn assert_existing(&self, context: &Context) {
        assert_eq!(
            context.realm().scope().get_binding(&js_string!("existing")),
            Some(self.existing.clone())
        );
        assert_eq!(
            context
                .realm()
                .environment()
                .get(self.existing.binding_index())
                .unwrap(),
            5.into()
        );
    }

    fn assert_private(&self, context: &Context, committed_bindings: u32) {
        self.assert_existing(context);
        assert_eq!(
            context.realm().scope().num_bindings(),
            self.bindings + committed_bindings
        );
        assert_eq!(next_scope_identity(context), self.next_identity);
        assert!(!context.realm().scope().has_binding(&js_string!("pending")));
    }
}

fn seeded() -> Context {
    let mut context = Context::builder().build().unwrap();
    context
        .eval(Source::from_bytes("let existing = 5;"))
        .unwrap();
    context
}

fn script_context(source: &str, action: u32) -> (Context, Script, RealmSnapshot) {
    let mut context = seeded();
    let script = Script::parse(Source::from_bytes(source), None, context.access()).unwrap();
    context
        .register_global_builtin_callable(
            js_string!("entry"),
            0,
            NativeFunction::from_fn_with_captures(
                script_entry,
                (ScriptCapture(script.clone()), action),
            ),
        )
        .unwrap();
    let snapshot = RealmSnapshot::new(&context);
    (context, script, snapshot)
}

fn run(context: &mut Context, limit: Option<u64>) -> ExecutionOutcome<JsValue> {
    let mut options =
        ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false });
    if let Some(limit) = limit {
        options = options.with_compiler_work_limit(limit);
    }
    context
        .eval_with_budget(Source::from_bytes(DRIVER), options)
        .unwrap()
}

fn assert_stop(outcome: &ExecutionOutcome<JsValue>, limit: u64) {
    assert_eq!(
        outcome.result.as_ref().unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::CompilerWork { limit },
        ))
    );
    assert!(outcome.report.compiler_work().unwrap() <= limit);
    // The accepted native entry is a real host effect; private compilation
    // still must not publish the inner script/module's scopes or declarations.
    assert!(outcome.report.effects_started());
}

fn origin(report: &ExecutionReport, name: &str) -> u64 {
    let profile = report.profile().unwrap().unwrap();
    let phase = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "bytecode-compilation")
        .unwrap();
    profile
        .scopes
        .iter()
        .find(|scope| scope.parent == Some(phase.scope) && scope.name == name)
        .unwrap()
        .usage
        .work(ExecutionWork::CompilerWork)
}

fn assert_compilation_attribution(report: &ExecutionReport) {
    let profile = report.profile().unwrap().unwrap();
    let phase = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "bytecode-compilation")
        .unwrap();
    let compiler = ExecutionWork::CompilerWork;
    assert_eq!(phase.usage.work(compiler), report.compiler_work().unwrap());
    assert_eq!(
        profile.scopes[0].usage.work(compiler),
        report.compiler_work().unwrap()
    );
    for dimension in [
        ExecutionWork::NameVisit,
        ExecutionWork::NameRecord,
        ExecutionWork::NameComparison,
        ExecutionWork::ScopeWork,
        ExecutionWork::CompilerWork,
    ] {
        let attributed: u64 = profile
            .scopes
            .iter()
            .filter(|scope| scope.parent == Some(phase.scope))
            .map(|scope| scope.usage.work(dimension))
            .sum();
        assert_eq!(attributed, phase.usage.work(dimension));
    }
}

fn encoded_work(code: &Rc<CodeBlock>) -> (u64, u64) {
    fn visit(code: &Rc<CodeBlock>, seen: &mut HashSet<*const CodeBlock>) -> (u64, u64) {
        if !seen.insert(Rc::as_ptr(code)) {
            return (0, 0);
        }
        let mut opcodes = u64::try_from(InstructionIterator::new(&code.bytecode).count()).unwrap();
        let mut bytes = u64::try_from(code.bytecode.bytes.len()).unwrap();
        for constant in &code.constants {
            if let Constant::Function(child) = constant {
                let (child_opcodes, child_bytes) = visit(child, seen);
                opcodes += child_opcodes;
                bytes += child_bytes;
            }
        }
        (opcodes, bytes)
    }
    visit(code, &mut HashSet::new())
}

#[test]
fn pending_script_miss_hit_stale_and_installed_cache_charge_only_actual_emission() {
    let (mut control, _, _) = script_context(PENDING, NOOP);
    let overhead = run(&mut control, None);
    assert_eq!(overhead.result.unwrap(), 42.into());
    let (mut context, script, snapshot) = script_context(PENDING, CODE_TWICE);
    let cold = run(&mut context, None);
    assert_eq!(cold.result.unwrap(), 42.into());
    assert_compilation_attribution(&cold.report);
    snapshot.assert_private(&context, 0);
    let first = script.codeblock(context.access()).unwrap();
    assert!(ActivatedCode::from_installed(first.clone()).is_err());
    let (opcodes, bytes) = encoded_work(&first);
    assert!(opcodes > 0 && bytes > 0);
    assert_eq!(
        origin(&cold.report, "compiler-opcode") - origin(&overhead.report, "compiler-opcode"),
        opcodes
    );
    assert_eq!(
        origin(&cold.report, "compiler-emit-bytes")
            - origin(&overhead.report, "compiler-emit-bytes"),
        bytes
    );

    let hit = run(&mut context, None);
    assert_eq!(hit.result.unwrap(), 42.into());
    assert!(Rc::ptr_eq(
        &first,
        &script.codeblock(context.access()).unwrap()
    ));
    assert_eq!(
        origin(&hit.report, "compiler-opcode"),
        origin(&overhead.report, "compiler-opcode")
    );
    assert_eq!(
        origin(&hit.report, "compiler-emit-bytes"),
        origin(&overhead.report, "compiler-emit-bytes")
    );
    assert!(
        origin(&hit.report, "compiler-metadata-visit")
            > origin(&overhead.report, "compiler-metadata-visit")
    );
    snapshot.assert_private(&context, 0);

    context
        .eval(Source::from_bytes("let between = 41;"))
        .unwrap();
    let stale = run(&mut context, None);
    assert_eq!(stale.result.unwrap(), 42.into());
    assert_compilation_attribution(&stale.report);
    let refreshed = script.codeblock(context.access()).unwrap();
    assert!(!Rc::ptr_eq(&first, &refreshed));
    let (opcodes, bytes) = encoded_work(&refreshed);
    assert_eq!(
        origin(&stale.report, "compiler-opcode") - origin(&overhead.report, "compiler-opcode"),
        opcodes
    );
    assert_eq!(
        origin(&stale.report, "compiler-emit-bytes")
            - origin(&overhead.report, "compiler-emit-bytes"),
        bytes
    );
    snapshot.assert_private(&context, 1);

    assert_eq!(script.evaluate(context.access()).unwrap(), 42.into());
    let installed = script.codeblock(context.access()).unwrap();
    assert!(Rc::ptr_eq(&refreshed, &installed));
    ActivatedCode::from_installed(installed.clone()).unwrap();
    let installed_next = next_scope_identity(&context);
    assert!(installed_next > snapshot.next_identity);
    context
        .eval(Source::from_bytes("let unrelated = 9;"))
        .unwrap();
    let cached = run(&mut context, None);
    assert_eq!(cached.result.unwrap(), 42.into());
    assert!(Rc::ptr_eq(
        &installed,
        &script.codeblock(context.access()).unwrap()
    ));
    assert_eq!(
        origin(&cached.report, "compiler-opcode"),
        origin(&overhead.report, "compiler-opcode")
    );
    assert_eq!(
        origin(&cached.report, "compiler-emit-bytes"),
        origin(&overhead.report, "compiler-emit-bytes")
    );
    assert_eq!(next_scope_identity(&context), installed_next);
    snapshot.assert_existing(&context);
    assert_compilation_attribution(&cached.report);
}

#[test]
fn pending_script_one_short_preserves_slots_and_identities_for_retained_and_fresh_retry() {
    let (mut baseline, _, _) = script_context(PENDING, CODE_ONCE);
    let complete = run(&mut baseline, None);
    assert_eq!(complete.result.unwrap(), 42.into());
    let work = complete.report.compiler_work().unwrap();
    let (mut exact, _, snapshot) = script_context(PENDING, CODE_ONCE);
    assert_eq!(run(&mut exact, Some(work)).result.unwrap(), 42.into());
    snapshot.assert_private(&exact, 0);
    for retained in [true, false] {
        let (mut context, script, snapshot) = script_context(PENDING, CODE_ONCE);
        let stopped = run(&mut context, Some(work - 1));
        assert_stop(&stopped, work - 1);
        snapshot.assert_private(&context, 0);
        let result = if retained {
            script.codeblock(context.access()).unwrap();
            snapshot.assert_private(&context, 0);
            script.evaluate(context.access()).unwrap()
        } else {
            context.eval(Source::from_bytes(PENDING)).unwrap()
        };
        assert_eq!(result, 42.into());
        assert!(context.realm().scope().has_binding(&js_string!("pending")));
        snapshot.assert_existing(&context);
    }
}

#[test]
fn a_later_compiler_stop_preserves_callback_committed_bindings_and_stale_retry() {
    let (mut baseline, _, _) = script_context(AFTER_CALLBACK, CODE_AROUND_EFFECT);
    let complete = run(&mut baseline, None);
    assert_eq!(complete.result.unwrap(), 42.into());
    let work = complete.report.compiler_work().unwrap();
    let (mut exact, _, snapshot) = script_context(AFTER_CALLBACK, CODE_AROUND_EFFECT);
    assert_eq!(run(&mut exact, Some(work)).result.unwrap(), 42.into());
    snapshot.assert_private(&exact, 1);
    for retained in [true, false] {
        let (mut context, script, snapshot) = script_context(AFTER_CALLBACK, CODE_AROUND_EFFECT);
        let stopped = run(&mut context, Some(work - 1));
        assert_stop(&stopped, work - 1);
        snapshot.assert_private(&context, 1);
        assert_eq!(
            context.eval(Source::from_bytes("between")).unwrap(),
            41.into()
        );
        let result = if retained {
            script.evaluate(context.access()).unwrap()
        } else {
            context.eval(Source::from_bytes(AFTER_CALLBACK)).unwrap()
        };
        assert_eq!(result, 42.into());
        snapshot.assert_existing(&context);
    }
}

#[derive(Clone, Copy)]
enum ModuleCase {
    Source,
    Synthetic,
    SourceWithCommittedDependency,
}

struct ModuleHarness {
    context: Context,
    module: Module,
    dependency: Option<Module>,
    snapshot: RealmSnapshot,
    case: ModuleCase,
}

const MODULE_SOURCE: &str = "
    import { offset } from 'dependency';
    export function answer(value = 1) { return existing + offset + value; }
    export const result = answer();
";

fn load(module: &Module, context: &mut Context) {
    let loaded = module.load(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
}

fn result_module(context: &mut Context) -> Module {
    Module::synthetic(
        &[js_string!("result")],
        SyntheticModuleInitializer::from_fn_with_captures(
            |module, (), _| module.set_export(&js_string!("result"), 42.into()),
            (),
        ),
        None,
        None,
        context.access(),
    )
    .unwrap()
}

fn module_harness(case: ModuleCase) -> ModuleHarness {
    let loader = Rc::new(MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes("let existing = 5;"))
        .unwrap();
    let (module, dependency) = match case {
        ModuleCase::Synthetic => (result_module(&mut context), None),
        ModuleCase::Source | ModuleCase::SourceWithCommittedDependency => {
            context
                .register_global_property(js_string!("dependencyRuns"), 0, Attribute::all())
                .unwrap();
            let dependency = Module::synthetic(
                &[js_string!("offset")],
                SyntheticModuleInitializer::from_fn_with_captures(
                    |module, (), mut context| {
                        context.eval(Source::from_bytes("dependencyRuns += 1;"))?;
                        module.set_export(&js_string!("offset"), 36.into())
                    },
                    (),
                ),
                None,
                None,
                context.access(),
            )
            .unwrap();
            if matches!(case, ModuleCase::SourceWithCommittedDependency) {
                load(&dependency, &mut context);
                dependency.link(context.access()).unwrap();
                let initialized = dependency.evaluate(context.access()).unwrap();
                context.run_jobs().unwrap();
                assert_eq!(
                    initialized.state().unwrap(),
                    PromiseState::Fulfilled(JsValue::undefined())
                );
            }
            assert!(loader.insert("dependency", dependency.clone()).is_none());
            let module =
                Module::parse(Source::from_bytes(MODULE_SOURCE), None, context.access()).unwrap();
            (module, Some(dependency))
        }
    };
    load(&module, &mut context);
    context
        .register_global_builtin_callable(
            js_string!("entry"),
            0,
            NativeFunction::from_fn_with_captures(
                |_, _, module, mut context| {
                    module.link(context.reborrow())?;
                    Ok(42.into())
                },
                module.clone(),
            ),
        )
        .unwrap();
    let snapshot = RealmSnapshot::new(&context);
    ModuleHarness {
        context,
        module,
        dependency,
        snapshot,
        case,
    }
}

fn assert_unpublished(harness: &mut ModuleHarness) {
    assert!(harness.module.environment().is_none());
    harness.snapshot.assert_private(&harness.context, 0);
    if let Some(dependency) = &harness.dependency {
        let committed = matches!(harness.case, ModuleCase::SourceWithCommittedDependency);
        assert_eq!(dependency.environment().is_some(), committed);
        let global = harness.context.global_object().unwrap().clone();
        assert_eq!(
            global
                .get(js_string!("dependencyRuns"), harness.context.access())
                .unwrap(),
            JsValue::new(if committed { 1 } else { 0 })
        );
        if committed {
            assert_eq!(
                dependency
                    .get_value(js_string!("offset"), harness.context.access())
                    .unwrap(),
                36.into()
            );
        }
    }
}

fn evaluate_module(module: &Module, context: &mut Context) {
    let evaluated = module.evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        evaluated.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("result"), context.access())
            .unwrap(),
        42.into()
    );
}

#[test]
fn source_and_synthetic_exact_one_short_and_retry_publish_only_the_admitted_group() {
    for case in [
        ModuleCase::Source,
        ModuleCase::Synthetic,
        ModuleCase::SourceWithCommittedDependency,
    ] {
        let mut baseline = module_harness(case);
        let complete = run(&mut baseline.context, None);
        assert_eq!(complete.result.unwrap(), 42.into());
        assert_compilation_attribution(&complete.report);
        let work = complete.report.compiler_work().unwrap();
        let mut exact = module_harness(case);
        let exact_result = run(&mut exact.context, Some(work));
        assert_eq!(exact_result.result.unwrap(), 42.into());
        assert!(exact.module.environment().is_some());
        assert!(next_scope_identity(&exact.context) > exact.snapshot.next_identity);
        exact.snapshot.assert_existing(&exact.context);
        assert_eq!(
            exact.context.realm().scope().num_bindings(),
            exact.snapshot.bindings
        );
        evaluate_module(&exact.module, &mut exact.context);
        for retained in [true, false] {
            let mut harness = module_harness(case);
            let stopped = run(&mut harness.context, Some(work - 1));
            assert_stop(&stopped, work - 1);
            assert_unpublished(&mut harness);
            let selected = if retained {
                harness.module.clone()
            } else if matches!(case, ModuleCase::Synthetic) {
                let fresh = result_module(&mut harness.context);
                load(&fresh, &mut harness.context);
                fresh
            } else {
                let fresh = Module::parse(
                    Source::from_bytes(MODULE_SOURCE),
                    None,
                    harness.context.access(),
                )
                .unwrap();
                load(&fresh, &mut harness.context);
                fresh
            };
            selected.link(harness.context.access()).unwrap();
            evaluate_module(&selected, &mut harness.context);
            harness.snapshot.assert_existing(&harness.context);
            assert_eq!(
                harness.context.realm().scope().num_bindings(),
                harness.snapshot.bindings
            );
            if harness.dependency.is_some() {
                let global = harness.context.global_object().unwrap().clone();
                assert_eq!(
                    global
                        .get(js_string!("dependencyRuns"), harness.context.access())
                        .unwrap(),
                    1.into()
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Family {
    Ordinary,
    Async,
    Generator,
    AsyncGenerator,
}

fn constructor_context(expression: &str) -> (Context, JsFunction, RealmSnapshot) {
    let mut context = seeded();
    let constructor = context
        .eval(Source::from_bytes(expression))
        .unwrap()
        .as_function()
        .unwrap();
    context
        .register_global_builtin_callable(
            js_string!("entry"),
            0,
            NativeFunction::from_fn_with_captures(
                |_, _, constructor, mut context| {
                    constructor.call(
                        &JsValue::undefined(),
                        &[
                            js_string!("value").into(),
                            js_string!("return value + 1;").into(),
                        ],
                        context.reborrow(),
                    )
                },
                constructor.clone(),
            ),
        )
        .unwrap();
    let snapshot = RealmSnapshot::new(&context);
    (context, constructor, snapshot)
}

fn fulfilled(value: JsValue, context: &mut Context) -> JsValue {
    let promise = JsPromise::from_object(value.as_object().unwrap()).unwrap();
    context.run_jobs().unwrap();
    let PromiseState::Fulfilled(value) = promise.state().unwrap() else {
        panic!("the generated async body must fulfill");
    };
    value
}

fn verify_function(value: JsValue, family: Family, context: &mut Context) {
    let function = value.as_function().unwrap();
    assert_eq!(
        function
            .get(js_string!("length"), context.access())
            .unwrap(),
        1.into()
    );
    assert_eq!(
        function.get(js_string!("name"), context.access()).unwrap(),
        js_string!("anonymous").into()
    );
    let result = function
        .call(&JsValue::undefined(), &[41.into()], context.access())
        .unwrap();
    match family {
        Family::Ordinary => assert_eq!(result, 42.into()),
        Family::Async => assert_eq!(fulfilled(result, context), 42.into()),
        Family::Generator | Family::AsyncGenerator => {
            let iterator = result.as_object().unwrap();
            let next = iterator
                .get(js_string!("next"), context.access())
                .unwrap()
                .as_function()
                .unwrap();
            let next = next.call(&result, &[], context.access()).unwrap();
            let next = if matches!(family, Family::AsyncGenerator) {
                fulfilled(next, context)
            } else {
                next
            };
            let next = next.as_object().unwrap();
            assert_eq!(
                next.get(js_string!("value"), context.access()).unwrap(),
                42.into()
            );
            assert_eq!(
                next.get(js_string!("done"), context.access()).unwrap(),
                true.into()
            );
        }
    }
}

#[test]
fn all_dynamic_constructor_families_share_compiler_admission_and_fresh_retry() {
    for (expression, family) in [
        ("Function", Family::Ordinary),
        ("(async function () {}).constructor", Family::Async),
        ("(function* () {}).constructor", Family::Generator),
        (
            "(async function* () {}).constructor",
            Family::AsyncGenerator,
        ),
    ] {
        let (mut baseline, _, _) = constructor_context(expression);
        let complete = run(&mut baseline, None);
        assert_compilation_attribution(&complete.report);
        let work = complete.report.compiler_work().unwrap();
        verify_function(complete.result.unwrap(), family, &mut baseline);
        let (mut exact, _, snapshot) = constructor_context(expression);
        let exact_result = run(&mut exact, Some(work));
        verify_function(exact_result.result.unwrap(), family, &mut exact);
        snapshot.assert_existing(&exact);

        let (mut retry, constructor, snapshot) = constructor_context(expression);
        let stopped = run(&mut retry, Some(work - 1));
        assert_stop(&stopped, work - 1);
        snapshot.assert_private(&retry, 0);
        let fresh = constructor
            .call(
                &JsValue::undefined(),
                &[
                    js_string!("value").into(),
                    js_string!("return value + 1;").into(),
                ],
                retry.access(),
            )
            .unwrap();
        verify_function(fresh, family, &mut retry);
        snapshot.assert_existing(&retry);
    }
}

#[test]
fn admitted_throw_source_maps_preserve_parentheses_and_leftmost_operand_positions() {
    for (source, expected) in [
        ("throw (\n  1 + 2\n);", (1, 7)),
        ("throw 1 +\n  2 +\n  3;", (1, 7)),
        ("\n  throw (\n    1 + 2\n  );", (2, 9)),
    ] {
        let (mut context, script, snapshot) = script_context(source, CODE_ONCE);
        let compiled = run(&mut context, None);
        assert_eq!(compiled.result.unwrap(), 42.into());
        snapshot.assert_private(&context, 0);
        let code = script.codeblock(context.access()).unwrap();
        let throws: Vec<_> = InstructionIterator::new(&code.bytecode)
            .filter(|(_, opcode, _)| *opcode == crate::vm::opcode::Opcode::Throw)
            .collect();
        assert_eq!(throws.len(), 1);
        let pc = u32::try_from(throws[0].0).unwrap();
        assert_eq!(
            code.source_info.map().find(pc),
            Some(expected.into()),
            "{source}"
        );
    }
}

#[test]
fn late_source_start_visit_refusal_keeps_preparation_private_and_allows_retry() {
    const SOURCE: &str = "throw 1+2+3+4;";
    let (mut baseline, _, _) = script_context(SOURCE, CODE_ONCE);
    let complete = run(&mut baseline, None);
    assert_eq!(complete.result.unwrap(), 42.into());
    let visits = complete.report.name_visits().unwrap();
    assert!(visits > 0);
    for retained in [true, false] {
        let (mut context, script, snapshot) = script_context(SOURCE, CODE_ONCE);
        let limit = visits - 1;
        let stopped = context
            .eval_with_budget(
                Source::from_bytes(DRIVER),
                ExecutionOptions::new(4096).with_name_visit_limit(limit),
            )
            .unwrap();
        assert_eq!(
            stopped.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::NameVisit {
                limit
            },))
        );
        assert_eq!(stopped.report.name_visits().unwrap(), limit);
        assert!(stopped.report.effects_started());
        snapshot.assert_private(&context, 0);
        let retry = if retained {
            script
        } else {
            Script::parse(Source::from_bytes(SOURCE), None, context.access()).unwrap()
        };
        let code = retry.codeblock(context.access()).unwrap();
        assert!(!code.bytecode.bytes.is_empty());
        snapshot.assert_private(&context, 0);
        let thrown = retry.evaluate(context.access()).unwrap_err();
        assert_eq!(thrown.as_opaque(), Some(&JsValue::new(10)));
    }
}
