use crate::error::{EngineError, RuntimeLimitError};
use crate::vm::CallFrame;
use crate::vm::call_frame::CallFrameLocation;
use crate::vm::source_info::SourcePath;
use crate::{
    Context, JsNativeErrorKind, JsValue, NativeFunction, TestAction, js_string,
    property::Attribute, run_test_actions, run_test_actions_with,
};
use indoc::indoc;
use surgeist_js_ast::Position;
use surgeist_js_macros::js_str;
use surgeist_js_parser::Source;

#[test]
fn typeof_string() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const a = "hello";
            typeof a;
        "#},
        js_str!("string"),
    )]);
}

#[test]
fn typeof_number() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let a = 1234;
            typeof a;
        "#},
        js_str!("number"),
    )]);
}

#[test]
fn basic_op() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const a = 1;
            const b = 2;
            a + b
        "#},
        3,
    )]);
}

#[test]
fn vm_registers_use_the_active_frame_base() {
    let mut context = Context::default();
    let vm = &mut context.vm;
    vm.stack.stack.extend([11.into(), 20.into(), 30.into()]);
    vm.frame_mut().rp = 1;

    assert_eq!(vm.get_register(0).as_number(), Some(20.0));
    vm.set_register(1, 42.into());
    assert_eq!(vm.take_register(1).as_number(), Some(42.0));
    assert!(vm.get_register(1).is_undefined());
    assert_eq!(vm.stack.stack[0].as_number(), Some(11.0));
}

#[test]
fn stack_register_lookup_refuses_an_overflowing_index() {
    let mut context = Context::default();
    let vm = &mut context.vm;
    vm.frame_mut().rp = 1;

    assert!(vm.stack.get_register(vm.frame(), usize::MAX).is_none());
}

#[test]
#[should_panic(expected = "register index out of bounds")]
fn vm_register_access_reports_an_overflowing_index() {
    let mut context = Context::default();
    let vm = &mut context.vm;
    vm.frame_mut().rp = 1;

    let _ = vm.get_register(usize::MAX);
}

#[test]
fn position() {
    let context = &mut Context::default();
    context
        .register_global_callable(
            js_string!("check_stack"),
            2,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let frame = context.stack_trace().collect::<Vec<&CallFrame>>();

                assert_eq!(frame.len(), 4);
                assert_eq!(
                    frame[0].position(),
                    CallFrameLocation {
                        function_name: js_string!("myOtherFunction"),
                        path: SourcePath::None,
                        position: Some(Position::new(2, 16))
                    }
                );
                assert_eq!(
                    frame[1].position(),
                    CallFrameLocation {
                        function_name: js_string!("<eval>"),
                        path: SourcePath::Eval,
                        position: Some(Position::new(1, 16))
                    }
                );
                assert_eq!(
                    frame[2].position(),
                    CallFrameLocation {
                        function_name: js_string!("myFunction"),
                        path: SourcePath::None,
                        position: Some(Position::new(5, 9))
                    }
                );
                assert_eq!(
                    frame[3].position(),
                    CallFrameLocation {
                        function_name: js_string!("<main>"),
                        path: SourcePath::None,
                        position: Some(Position::new(8, 11))
                    }
                );
                Ok(JsValue::undefined())
            }),
        )
        .expect("Could not register function");
    run_test_actions_with(
        [TestAction::run(indoc! {r#"
            const myOtherFunction = () => {
                check_stack();
            };
            function myFunction() {
                eval("myOtherFunction()");
            }

            myFunction();
        "#})],
        context,
    );
}

#[test]
fn try_catch_finally_from_init() {
    // the initialisation of the array here emits a PopOnReturnAdd op
    //
    // here we test that the stack is not popped more than intended due to multiple catches in the
    // same function, which could lead to VM stack corruption
    run_test_actions([TestAction::assert_opaque_error(
        indoc! {r#"
            try {
                [(() => {throw "h";})()];
            } catch (x) {
                throw "h";
            } finally {
            }
        "#},
        js_str!("h"),
    )]);
}

#[test]
fn multiple_catches() {
    // see explanation on `try_catch_finally_from_init`
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                try {
                    [(() => {throw "h";})()];
                } catch (x) {
                    throw "h";
                }
            } catch (y) {
            }
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn use_last_expr_try_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                19;
                7.5;
                "Hello!";
            } catch (y) {
                14;
                "Bye!"
            }
        "#},
        js_str!("Hello!"),
    )]);
}

#[test]
fn use_last_expr_catch_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
                throw Error("generic error");
                19;
                7.5;
            } catch (y) {
                14;
                "Hello!";
            }
        "#},
        js_str!("Hello!"),
    )]);
}

#[test]
fn no_use_last_expr_finally_block() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            try {
            } catch (y) {
            } finally {
                "Unused";
            }
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn finally_block_binding_env() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let buf = "Hey hey";
            try {
            } catch (y) {
            } finally {
                let x = " people";
                buf += x;
            }
            buf
        "#},
        js_str!("Hey hey people"),
    )]);
}

#[test]
fn run_super_method_in_object() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let proto = {
                m() { return "super"; }
            };
            let obj = {
                v() { return super.m(); }
            };
            Object.setPrototypeOf(obj, proto);
            obj.v();
        "#},
        js_str!("super"),
    )]);
}

#[test]
fn get_reference_by_super() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            var fromA, fromB;
            var A = { fromA: 'a', fromB: 'a' };
            var B = { fromB: 'b' };
            Object.setPrototypeOf(B, A);
            var obj = {
                fromA: 'c',
                fromB: 'c',
                method() {
                    fromA = (() => { return super.fromA; })();
                    fromB = (() => { return super.fromB; })();
                }
            };
            Object.setPrototypeOf(obj, B);
            obj.method();
            fromA + fromB
        "#},
        js_str!("ab"),
    )]);
}

#[test]
fn super_call_constructor_null() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            class A extends Object {
                constructor() {
                    Object.setPrototypeOf(A, null);
                    super(A);
                }
            }
            new A();
        "#},
        JsNativeErrorKind::Type,
        "super constructor object must be constructor",
    )]);
}

#[test]
fn super_call_get_constructor_before_arguments_execution() {
    run_test_actions([TestAction::assert(indoc! {r#"
        class A extends Object {
            constructor() {
                super(Object.setPrototypeOf(A, null));
            }
        }
        new A() instanceof A;
    "#})]);
}

#[test]
fn order_of_execution_in_assignment() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let i = 0;
                let array = [[]];

                array[i++][i++] = i++;
            "#}),
        TestAction::assert_eq("i", 3),
        TestAction::assert_eq("array.length", 1),
        TestAction::assert_eq("array[0].length", 2),
    ]);
}

#[test]
fn order_of_execution_in_assignment_with_comma_expressions() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let result = "";
            function f(i) {
                result += i;
            }
            let a = [[]];
            (f(1), a)[(f(2), 0)][(f(3), 0)] = (f(4), 123);
            result
        "#},
        js_str!("1234"),
    )]);
}

#[test]
fn loop_runtime_limit() {
    run_test_actions([
        TestAction::assert_eq(
            indoc! {r#"
                for (let i = 0; i < 20; ++i) { }
            "#},
            JsValue::undefined(),
        ),
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_loop_iteration_limit(10);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                for (let i = 0; i < 20; ++i) { }
            "#},
            RuntimeLimitError::LoopIteration,
        ),
        TestAction::assert_eq(
            indoc! {r#"
                for (let i = 0; i < 10; ++i) { }
            "#},
            JsValue::undefined(),
        ),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                while (1) { }
            "#},
            RuntimeLimitError::LoopIteration,
        ),
    ]);
}

#[test_case::test_case("while (true) {}"; "script")]
#[test_case::test_case("function stop() { while (true) {} } stop();"; "nested_call")]
fn a_runtime_stop_releases_the_execution_stack(source: &str) {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(0);
    let stack_len = context.vm.stack.stack.len();
    let frame_count = context.vm.frames.len();

    let error = context.eval(Source::from_bytes(source)).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::LoopIteration))
    );
    assert_eq!(context.vm.frames.len(), frame_count);
    assert_eq!(context.vm.stack.stack.len(), stack_len);
    assert_eq!(
        context.eval(Source::from_bytes("1 + 2")).unwrap(),
        JsValue::new(3)
    );
}

#[test]
fn a_runtime_stop_completes_the_async_generator_and_reaches_the_host() {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(0);
    let error = context
        .eval(Source::from_bytes(
            "let stopped = (async function* () { while (true) {} })(); stopped.next();",
        ))
        .unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::LoopIteration))
    );

    context
        .eval(Source::from_bytes(
            "stopped.next().then(result => globalThis.generatorDone = result.done);",
        ))
        .unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("globalThis.generatorDone"))
            .unwrap(),
        JsValue::new(true)
    );
}

fn instructions_through(source: &str, end: super::opcode::Opcode, context: &mut Context) -> u64 {
    let script =
        crate::script::Script::parse(Source::from_bytes(source), None, context.access()).unwrap();
    let code = script.codeblock(context.access()).unwrap();
    let mut pc = 0;
    let mut limit = 0;
    while let Some(byte) = code.bytecode.bytes.get(pc) {
        limit += 1;
        if super::opcode::Opcode::decode(*byte) == end {
            return limit;
        }
        (_, pc) = code.bytecode.next_instruction(pc);
    }
    panic!("fixture did not contain {end:?}");
}

#[test]
fn an_instruction_stop_before_the_catch_discards_the_pending_exception() {
    let source = "try { throw 42; } catch { globalThis.caught = true; }";
    let mut context = Context::default();
    let limit = instructions_through(source, super::opcode::Opcode::Throw, &mut context);

    let outcome = context
        .eval_with_budget(
            Source::from_bytes(source),
            crate::execution::ExecutionOptions::new(limit),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit,
        }))
    );
    assert!(context.vm.frame().pending_exception.is_none());
    assert_eq!(
        context
            .eval(Source::from_bytes("typeof caught === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn an_instruction_stop_discards_the_unreturned_result() {
    let source = "({})";
    let mut context = Context::default();
    let limit = instructions_through(source, super::opcode::Opcode::SetAccumulator, &mut context);
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(source),
            crate::execution::ExecutionOptions::new(limit),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit,
        }))
    );
    assert!(context.vm.get_return_value().is_undefined());
}

#[test]
fn a_script_declaration_error_releases_the_prepared_stack() {
    let mut context = Context::default();
    let stack_len = context.vm.stack.stack.len();
    let frame_count = context.vm.frames.len();

    let error = context
        .eval(Source::from_bytes("let undefined = 0;"))
        .unwrap_err();
    let native = error.as_native().unwrap();
    assert_eq!(native.kind(), &JsNativeErrorKind::Syntax);
    assert_eq!(
        native.message(),
        "cannot redefine non-configurable global property"
    );
    assert_eq!(context.vm.frames.len(), frame_count);
    assert_eq!(context.vm.stack.stack.len(), stack_len);
}

#[test]
fn recursion_runtime_limit() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function factorial(n) {
                if (n == 0) {
                    return 1;
                }

                return n * factorial(n - 1);
            }
        "#}),
        TestAction::assert_eq("factorial(8)", JsValue::new(40_320)),
        TestAction::assert_eq("factorial(11)", JsValue::new(39_916_800)),
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(10);
        }),
        TestAction::assert_runtime_limit_error("factorial(11)", RuntimeLimitError::Recursion),
        TestAction::assert_eq("factorial(8)", JsValue::new(40_320)),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                function x() {
                    x()
                }

                x()
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

#[test]
fn arguments_object_constructor_valid_index() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let args;
            function F(a = 1) {
                args = arguments;
            }
            new F();
            typeof args
        "#},
        js_str!("object"),
    )]);
}

#[test]
fn empty_return_values() {
    run_test_actions([
        TestAction::run(indoc! {r#"do {{}} while (false);"#}),
        TestAction::run(indoc! {r#"do try {{}} catch {} while (false);"#}),
        TestAction::run(indoc! {r#"do {} while (false);"#}),
        TestAction::run(indoc! {r#"do try {{}{}} catch {} while (false);"#}),
        TestAction::run(indoc! {r#"do {{}{}} while (false);"#}),
        TestAction::run(indoc! {r#"do {;{}} while (false);"#}),
        TestAction::run(indoc! {r#"do {e: {}} while (false);"#}),
        TestAction::run(indoc! {r#"do {e: ;} while (false);"#}),
        TestAction::run(indoc! {r#"do { break } while (false);"#}),
        TestAction::run(indoc! {r#"while (true) a: break"#}),
        TestAction::run(indoc! {r#"while (true) a: {"a"; break};"#}),
        TestAction::run(indoc! {r#"do {"a";{}} while (false);"#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: {}
            }
        "#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: {}{}
            }
        "#}),
        TestAction::run(indoc! {r#"
            switch (false) {
                default: ;{}{}
            }
        "#}),
    ]);
}

#[test]
fn truncate_environments_on_non_caught_native_error() {
    let source = "with (new Proxy({}, {has: p => false})) {a}";
    run_test_actions([
        TestAction::assert_native_error(source, JsNativeErrorKind::Reference, "a is not defined"),
        TestAction::assert_native_error(source, JsNativeErrorKind::Reference, "a is not defined"),
    ]);
}

#[test]
fn super_construction_with_parameter_expression() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            class Person {
                constructor(name) {
                    this.name = name;
                }
            }

            class Student extends Person {
                constructor(name = 'unknown') {
                    super(name);
                }
            }
        "#}),
        TestAction::assert_eq("new Student().name", js_str!("unknown")),
        TestAction::assert_eq("new Student('Jack').name", js_str!("Jack")),
    ]);
}

#[test]
fn cross_context_function_call() {
    let context1 = &mut Context::default();
    let result = context1.eval(Source::from_bytes(indoc! {r"
        var global = 100;

        (function x() {
            return global;
        })
    "}));

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_callable());

    let context2 = &mut Context::builder()
        .runtime(context1.runtime().clone())
        .build()
        .unwrap();

    context2
        .register_global_property(js_string!("func"), result, Attribute::all())
        .unwrap();

    let result = context2.eval(Source::from_bytes("func()"));

    assert_eq!(result, Ok(JsValue::new(100)));
}

// See: https://github.com/boa-dev/boa/issues/1848
#[test]
fn long_object_chain_gc_trace_stack_overflow() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            let old = {};
            for (let i = 0; i < 100000; i++) {
                old = { old };
            }
        "#}),
        TestAction::inspect_context(|context| {
            context.runtime().collect().unwrap();
        }),
    ]);
}

// See: https://github.com/boa-dev/boa/issues/4515
#[test]
fn recursion_in_async_gen_throws_uncatchable_error() {
    run_test_actions([
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(128);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                async function* f() {}
                f().return({
                  get then() {
                    this.then;
                  },
                });
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

#[test]
fn recursion_in_setter_throws_uncatchable_error() {
    run_test_actions([
        TestAction::inspect_context(|context| {
            context.runtime_limits_mut().set_recursion_limit(128);
        }),
        TestAction::assert_runtime_limit_error(
            indoc! {r#"
                const obj = {
                  set x(value) {
                    this.x = value;
                  },
                };
                obj.x = 1;
            "#},
            RuntimeLimitError::Recursion,
        ),
    ]);
}

fn installed_code_fixture(context: &Context) -> std::rc::Rc<super::CodeBlock> {
    let mut code = super::CodeBlock::new(js_string!("entry fixture"), 0, true);
    code.scope_association = context.realm().scope().handle();
    std::rc::Rc::new(code)
}

#[test]
fn activation_refuses_unprepared_pending_and_aborted_associations() {
    use surgeist_js_ast::scope::{PreparedScopes, ScopeStateError};
    let context = Context::default();
    let unprepared = std::rc::Rc::new(super::CodeBlock::new(js_string!("raw"), 0, true));
    assert_eq!(
        super::ActivatedCode::from_installed(unprepared)
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::ScopeAnalysis(ScopeStateError::Unprepared))
    );
    let pending = PreparedScopes::new(context.realm().scope());
    let mut code = super::CodeBlock::new(js_string!("pending"), 0, true);
    code.scope_association = pending.root_handle();
    let code = std::rc::Rc::new(code);
    assert_eq!(
        super::ActivatedCode::from_installed(code.clone())
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::ScopeAnalysis(ScopeStateError::Unprepared))
    );
    drop(pending);
    assert_eq!(
        super::ActivatedCode::from_installed(code)
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::ScopeAnalysis(
            ScopeStateError::AssociationMismatch
        ))
    );
}

#[test]
fn activation_retains_the_original_code_and_rejects_foreign_scope_constants() {
    let context = Context::default();
    let mut code = super::CodeBlock::new(js_string!("scoped"), 0, true);
    code.scope_association = context.realm().scope().handle();
    code.constants.push(super::Constant::Scope(
        surgeist_js_ast::scope::Scope::new_global().handle(),
        super::ScopeDisplay {
            index: 0,
            bindings: 0,
        },
    ));
    let code = std::rc::Rc::new(code);
    let active = super::ActivatedCode::from_installed(code.clone()).unwrap();
    assert!(std::rc::Rc::ptr_eq(active.code(), &code));
    assert_eq!(
        active.constant_scope(0).unwrap_err().as_engine(),
        Some(&EngineError::ScopeAnalysis(
            surgeist_js_ast::scope::ScopeStateError::AssociationMismatch,
        ))
    );
}

#[test]
fn host_start_and_dummy_frames_refuse_sync_and_async_bytecode_entry() {
    use std::future::Future as _;
    let mut context = Context::default();
    assert!(
        matches!(context.run(), super::CompletionRecord::Throw(error)
        if matches!(error.as_engine(), Some(EngineError::InvalidEntryPermit)))
    );
    let frame = CallFrame::host_start(
        installed_code_fixture(&context),
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    );
    context.vm.push_host_start(frame).unwrap();
    assert!(
        matches!(context.run(), super::CompletionRecord::Throw(error)
        if matches!(error.as_engine(), Some(EngineError::InvalidEntryPermit)))
    );
    {
        let mut future = std::pin::pin!(context.run_async_with_budget(1));
        let mut task = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(
            matches!(future.as_mut().poll(&mut task), std::task::Poll::Ready(
            super::CompletionRecord::Throw(error))
            if matches!(error.as_engine(), Some(EngineError::InvalidEntryPermit)))
        );
    }
    context.vm.pop_frame().unwrap();
}

#[test]
fn first_dispatch_reservation_rejects_another_code_or_instruction() {
    let mut context = Context::default();
    let code = installed_code_fixture(&context);
    let other = installed_code_fixture(&context);
    let permit = context.reserve_guest_entry(code.clone(), 0).unwrap();
    permit.validate_for(&code, 0).unwrap();
    assert_eq!(
        permit.validate_for(&other, 0).unwrap_err().as_engine(),
        Some(&EngineError::InvalidEntryPermit)
    );
    assert_eq!(
        permit.validate_for(&code, 1).unwrap_err().as_engine(),
        Some(&EngineError::InvalidEntryPermit)
    );
    assert_eq!(
        CallFrame::new(
            super::ActivatedCode::from_installed(other).unwrap(),
            permit,
            None,
            crate::environments::EnvironmentStack::new(),
            context.realm().clone(),
        )
        .unwrap_err()
        .as_engine(),
        Some(&EngineError::InvalidEntryPermit)
    );
}

#[test]
fn reserved_first_dispatch_consumes_one_instruction_and_the_next_dispatch_refuses() {
    let mut context = Context::default();
    context.execution =
        Some(crate::execution::Execution::new(crate::execution::ExecutionOptions::new(1)).unwrap());
    let code = installed_code_fixture(&context);
    let permit = context.reserve_guest_entry(code.clone(), 0).unwrap();
    let frame = CallFrame::new(
        super::ActivatedCode::from_installed(code).unwrap(),
        permit,
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    )
    .unwrap()
    .with_flags(super::CallFrameFlags::EXIT_EARLY);
    context
        .vm
        .push_frame_with_stack(frame, JsValue::undefined(), JsValue::undefined());
    let mut dispatched = 0;
    assert!(
        context
            .execute_one(
                |context, _| {
                    dispatched += 1;
                    context.vm.frame_mut().pc = 1;
                    std::ops::ControlFlow::Continue(())
                },
                super::Opcode::Pop
            )
            .is_continue()
    );
    assert_eq!(dispatched, 1);
    let outcome = context.execute_one(
        |_, _| {
            dispatched += 1;
            std::ops::ControlFlow::Continue(())
        },
        super::Opcode::Pop,
    );
    assert!(matches!(outcome, std::ops::ControlFlow::Break(
        super::CompletionRecord::Throw(error)) if error.as_engine() == Some(
            &EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 1 }))));
    assert_eq!(dispatched, 1);
}

#[test]
#[cfg(feature = "fuzz")]
fn first_dispatch_reservation_checks_fuzz_before_instruction_and_ticks_once() {
    let mut context = Context::default();
    context.execution =
        Some(crate::execution::Execution::new(crate::execution::ExecutionOptions::new(0)).unwrap());
    context.instructions_remaining = 0;
    let code = installed_code_fixture(&context);
    assert_eq!(
        context
            .reserve_guest_entry(code.clone(), 0)
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::NoInstructionsRemain)
    );

    context.execution =
        Some(crate::execution::Execution::new(crate::execution::ExecutionOptions::new(1)).unwrap());
    context.instructions_remaining = 1;
    let permit = context.reserve_guest_entry(code.clone(), 0).unwrap();
    assert_eq!(context.instructions_remaining, 0);
    let frame = CallFrame::new(
        super::ActivatedCode::from_installed(code).unwrap(),
        permit,
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    )
    .unwrap()
    .with_flags(super::CallFrameFlags::EXIT_EARLY);
    context
        .vm
        .push_frame_with_stack(frame, JsValue::undefined(), JsValue::undefined());
    let mut dispatched = false;
    assert!(
        context
            .execute_one(
                |_, _| {
                    dispatched = true;
                    std::ops::ControlFlow::Continue(())
                },
                super::Opcode::Pop
            )
            .is_continue()
    );
    assert!(dispatched);
    assert_eq!(context.instructions_remaining, 0);
}

#[test]
fn a_frame_rejects_a_matching_code_permit_from_another_invocation() {
    let mut context = Context::default();
    let code = installed_code_fixture(&context);
    let first = context.reserve_guest_entry(code.clone(), 0).unwrap();
    let second = context.reserve_guest_entry(code.clone(), 0).unwrap();
    let frame = CallFrame::new(
        super::ActivatedCode::from_installed(code).unwrap(),
        first,
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    )
    .unwrap();
    assert_eq!(
        second.validate_dispatch(&frame).unwrap_err().as_engine(),
        Some(&EngineError::InvalidEntryPermit)
    );
}

#[test]
fn suspended_and_cloned_frames_retain_their_actual_scope_activation() {
    let mut context = Context::default();
    let handle = context.realm().scope().handle();
    let mut code = super::CodeBlock::new(js_string!("resume"), 0, true);
    code.scope_association = handle.clone();
    code.constants.push(super::Constant::Scope(
        handle.clone(),
        super::ScopeDisplay {
            index: 0,
            bindings: 0,
        },
    ));
    let code = std::rc::Rc::new(code);
    let permit = context.reserve_guest_entry(code.clone(), 0).unwrap();
    let mut frame = CallFrame::new(
        super::ActivatedCode::from_installed(code.clone()).unwrap(),
        permit,
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    )
    .unwrap();
    frame
        .take_first_dispatch()
        .unwrap()
        .validate_dispatch(&frame)
        .unwrap();
    for resumed in [frame.clone(), frame.stored().to_frame().unwrap()] {
        assert!(std::rc::Rc::ptr_eq(resumed.code_block(), &code));
        assert_eq!(resumed.constant_scope(0).unwrap().handle(), handle);
        resumed.require_executable().unwrap();
    }
}

#[test]
fn a_prepared_function_recipe_preserves_properties_and_constructor_back_reference() {
    let mut context = Context::default();
    let mut code = super::CodeBlock::new(js_string!("prepared"), 2, true);
    code.scope_association = context.realm().scope().handle();
    code.flags
        .set(super::CodeBlockFlags::STRICT | super::CodeBlockFlags::HAS_PROTOTYPE_PROPERTY);
    let code = std::rc::Rc::new(code);
    let recipe = super::prepare_function_object_fast(
        code.clone(),
        &crate::environments::EnvironmentStack::new(),
        None,
        context.realm().clone(),
        &mut context,
    )
    .unwrap();
    let function = recipe
        .activate(super::ActivatedCode::from_installed(code).unwrap())
        .unwrap();
    assert_eq!(
        function.get(js_string!("name"), context.access()).unwrap(),
        JsValue::from(js_string!("prepared"))
    );
    assert_eq!(
        function
            .get(js_string!("length"), context.access())
            .unwrap(),
        JsValue::from(2)
    );
    let prototype = function
        .get(js_string!("prototype"), context.access())
        .unwrap()
        .as_object()
        .unwrap();
    let constructor = prototype
        .get(js_string!("constructor"), context.access())
        .unwrap()
        .as_object()
        .unwrap();
    assert!(crate::JsObject::equals(&constructor, &function));
}

#[test]
fn a_prepared_function_recipe_cannot_expose_a_different_activated_codeblock() {
    let mut context = Context::default();
    let code = installed_code_fixture(&context);
    let other = installed_code_fixture(&context);
    let recipe = super::prepare_function_object_fast(
        code,
        &crate::environments::EnvironmentStack::new(),
        None,
        context.realm().clone(),
        &mut context,
    )
    .unwrap();
    assert_eq!(
        recipe
            .activate(super::ActivatedCode::from_installed(other).unwrap())
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::InvalidEntryPermit)
    );
}

#[test]
fn a_reserved_dispatch_preserves_an_intervening_retained_stop() {
    let mut context = Context::default();
    context.execution =
        Some(crate::execution::Execution::new(crate::execution::ExecutionOptions::new(1)).unwrap());
    let code = installed_code_fixture(&context);
    let permit = context.reserve_guest_entry(code.clone(), 0).unwrap();
    let frame = CallFrame::new(
        super::ActivatedCode::from_installed(code).unwrap(),
        permit,
        None,
        crate::environments::EnvironmentStack::new(),
        context.realm().clone(),
    )
    .unwrap()
    .with_flags(super::CallFrameFlags::EXIT_EARLY);
    context
        .vm
        .push_frame_with_stack(frame, JsValue::undefined(), JsValue::undefined());
    let stop = crate::JsError::from(EngineError::RuntimeLimit(RuntimeLimitError::Recursion));
    context.execution.as_mut().unwrap().retain_stop(&stop);
    let mut dispatched = false;
    let result = context.execute_one(
        |_, _| {
            dispatched = true;
            std::ops::ControlFlow::Continue(())
        },
        super::Opcode::Pop,
    );
    assert!(!dispatched);
    assert!(matches!(result, std::ops::ControlFlow::Break(
        super::CompletionRecord::Throw(error)) if error.as_engine() == stop.as_engine()));
}

#[test]
fn deferred_guest_arguments_check_the_normal_call_stack_boundary_before_entry() {
    let mut context = Context::default();
    context.runtime_limits_mut().set_stack_size_limit(3);
    // Normal ordinary-call admission sees this, function and one argument.
    assert_eq!(
        context
            .prepare_guest_call_storage(1, 0)
            .unwrap_err()
            .as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::StackSize))
    );
    // Register-file capacity is reserved, but the existing semantic gate measures
    // the calling convention slots, before that register file is pushed.
    context.prepare_guest_call_storage(0, 8).unwrap();
    context.runtime_limits_mut().set_stack_size_limit(4);
    context.prepare_guest_call_storage(1, 8).unwrap();
    assert_eq!(context.vm.stack.stack.len(), 0);
}
