#![allow(unused_crate_dependencies)]

use std::{
    cell::Cell,
    future::Future,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsValue, NativeFunction, Script, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionTermination, ProfileMode},
    js_string,
    object::builtins::JsFunction,
    property::{PropertyDescriptor, PropertyKey},
    value::{IntegerOrInfinity, PreferredType},
};

#[test]
fn access_and_reborrow_evaluate_against_the_same_committed_guest_state() {
    let mut owner = Context::default();
    {
        let mut access = owner.access();
        assert_eq!(
            access
                .eval(Source::from_bytes("globalThis.value = 40"))
                .unwrap(),
            JsValue::new(40)
        );
        {
            let mut nested = access.reborrow();
            assert_eq!(
                nested.eval(Source::from_bytes("value += 2")).unwrap(),
                JsValue::new(42)
            );
        }
        assert_eq!(
            access.eval(Source::from_bytes("value")).unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn typed_host_data_mutations_through_views_remain_owned_by_the_context() {
    let mut owner = Context::default();
    assert!(owner.insert_data(Cell::new(1_u32)).is_none());
    {
        let mut access = owner.access();
        assert!(access.has_data::<Cell<u32>>());
        access.get_data::<Cell<u32>>().unwrap().set(20);
        {
            let mut nested = access.reborrow();
            assert_eq!(nested.get_data::<Cell<u32>>().unwrap().get(), 20);
            let previous = nested.insert_data(Cell::new(40_u32)).unwrap();
            assert_eq!(previous.get(), 20);
            nested.get_data::<Cell<u32>>().unwrap().set(42);
            let retained = nested.remove_data::<Cell<u32>>().unwrap();
            assert_eq!(retained.get(), 42);
            assert!(!nested.has_data::<Cell<u32>>());
            assert!(nested.insert_data(*retained).is_none());
        }
        assert_eq!(access.get_data::<Cell<u32>>().unwrap().get(), 42);
    }
    assert!(owner.has_data::<Cell<u32>>());
    assert_eq!(owner.get_data::<Cell<u32>>().unwrap().get(), 42);
    assert_eq!(owner.remove_data::<Cell<u32>>().unwrap().get(), 42);
    assert!(!owner.has_data::<Cell<u32>>());
}

#[test]
fn idle_owner_move_and_swap_preserve_state_reports_and_selected_operation_limits() {
    let mut first = Context::default();
    let completed = {
        let mut access = first.access();
        access
            .eval(Source::from_bytes("globalThis.ownerValue = 40"))
            .unwrap();
        access
            .eval_with_budget(
                Source::from_bytes("ownerValue += 2; ownerValue"),
                ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap()
    };
    assert_eq!(completed.result.unwrap(), JsValue::new(42));
    let instructions = completed.report.instructions();
    assert!(instructions > 0);
    let retained_report = completed.report.clone();
    let retained_batches = retained_report.profile().unwrap().scopes[0].accepted_batches;
    assert!(retained_batches > 0);

    let mut second = Context::default();
    second
        .access()
        .eval(Source::from_bytes("globalThis.ownerValue = -1"))
        .unwrap();
    let mut moved = first;
    std::mem::swap(&mut moved, &mut second);
    assert_eq!(
        second
            .access()
            .eval(Source::from_bytes("ownerValue"))
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        moved
            .access()
            .eval(Source::from_bytes("ownerValue"))
            .unwrap(),
        JsValue::new(-1)
    );

    // The zero budget selects the original first owner in its new binding.
    // Its prior completed report does not attach a budget to either binding.
    let denied = second
        .access()
        .eval_with_budget(
            Source::from_bytes("globalThis.budgetEffect = 1"),
            ExecutionOptions::new(0),
        )
        .unwrap();
    assert_eq!(
        denied.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(denied.report.instructions(), 0);
    let accepted = moved
        .access()
        .eval_with_budget(
            Source::from_bytes("globalThis.budgetEffect = 42; budgetEffect"),
            ExecutionOptions::new(64),
        )
        .unwrap();
    assert_eq!(accepted.result.unwrap(), JsValue::new(42));
    assert!(accepted.report.instructions() > 0);
    assert_eq!(
        second
            .access()
            .eval(Source::from_bytes(
                "ownerValue === 42 && typeof budgetEffect === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        moved
            .access()
            .eval(Source::from_bytes(
                "ownerValue === -1 && budgetEffect === 42"
            ))
            .unwrap(),
        JsValue::new(true)
    );

    assert_eq!(retained_report.instructions(), instructions);
    assert_eq!(
        retained_report.profile().unwrap().scopes[0].accepted_batches,
        retained_batches
    );
    assert!(matches!(
        retained_report.termination(),
        ExecutionTermination::Completed
    ));
    assert!(std::ptr::eq(
        completed.report.profile().unwrap(),
        retained_report.profile().unwrap()
    ));
}

#[test]
fn script_compilation_and_async_evaluation_compose_through_one_outer_access() {
    let mut owner = Context::default();
    {
        let mut access = owner.access();
        let script = Script::parse(
            Source::from_bytes(
                "globalThis.accessEffect = 0; for (let i = 0; i < 6; i++) accessEffect += 7; accessEffect",
            ),
            None,
            access.reborrow(),
        )
        .unwrap();
        let _compiled = script.codeblock(access.reborrow()).unwrap();
        let mut future = Box::pin(script.evaluate_async_with_budget(access.reborrow(), 1));
        let mut result = None;
        let mut pending_polls = 0;
        for _ in 0..1024 {
            match future
                .as_mut()
                .poll(&mut TaskContext::from_waker(Waker::noop()))
            {
                Poll::Pending => pending_polls += 1,
                Poll::Ready(completion) => {
                    result = Some(completion);
                    break;
                }
            }
        }
        let result = result.expect("script remained pending after 1024 cooperative polls");
        drop(future);
        assert!(
            pending_polls > 1,
            "script must cross multiple real suspensions"
        );
        assert_eq!(result.unwrap(), JsValue::new(42));
        assert_eq!(
            access.eval(Source::from_bytes("accessEffect")).unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("accessEffect")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn sequential_value_conversions_reborrow_one_access_for_primitive_results() {
    let mut owner = Context::default();
    let mut access = owner.access();
    assert_eq!(
        JsValue::new(js_string!("42.75"))
            .to_number(access.reborrow())
            .unwrap(),
        42.75
    );
    assert_eq!(
        JsValue::new(-42.75)
            .to_integer_or_infinity(access.reborrow())
            .unwrap(),
        IntegerOrInfinity::Integer(-42)
    );
    assert_eq!(
        JsValue::new(4_294_967_297.0)
            .to_i32(access.reborrow())
            .unwrap(),
        1
    );
    assert_eq!(
        JsValue::new(true).to_string(access.reborrow()).unwrap(),
        js_string!("true")
    );
    assert_eq!(
        JsValue::new(js_string!("0x2a"))
            .to_bigint(access.reborrow())
            .unwrap()
            .to_string_radix(10),
        "42"
    );
    assert_eq!(
        access.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn value_conversion_hooks_receive_ordered_hints_and_preserve_exact_thrown_value() {
    let mut owner = Context::default();
    let mut access = owner.access();
    let value = access.eval(Source::from_bytes(
        "globalThis.hints = []; ({ [Symbol.toPrimitive](hint) { hints.push(hint); return hint === 'string' ? 'answer' : 41; } })",
    )).unwrap();
    assert_eq!(
        value
            .to_primitive(access.reborrow(), PreferredType::Default)
            .unwrap(),
        JsValue::new(41)
    );
    assert_eq!(value.to_number(access.reborrow()).unwrap(), 41.0);
    assert_eq!(
        value.to_string(access.reborrow()).unwrap(),
        js_string!("answer")
    );
    assert_eq!(
        access.eval(Source::from_bytes("hints.join(',')")).unwrap(),
        JsValue::new(js_string!("default,number,string"))
    );

    let throwing = access
        .eval(Source::from_bytes(
            "globalThis.sentinel = { token: 7 }; globalThis.throwHints = []; ({
            [Symbol.toPrimitive](hint) { throwHints.push(hint); throw sentinel; },
            valueOf() { globalThis.fallback = true; return 0; },
            toString() { globalThis.fallback = true; return 'fallback'; }
        })",
        ))
        .unwrap();
    let sentinel = access.eval(Source::from_bytes("sentinel")).unwrap();
    assert_eq!(
        throwing
            .to_number(access.reborrow())
            .unwrap_err()
            .as_opaque(),
        Some(&sentinel)
    );
    assert_eq!(
        throwing
            .to_string(access.reborrow())
            .unwrap_err()
            .as_opaque(),
        Some(&sentinel)
    );
    assert_eq!(
        access
            .eval(Source::from_bytes("throwHints.join(',')"))
            .unwrap(),
        JsValue::new(js_string!("number,string"))
    );
    assert_eq!(
        access
            .eval(Source::from_bytes("typeof fallback === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn value_to_property_descriptor_runs_guest_getters_in_specified_order() {
    let mut owner = Context::default();
    let mut access = owner.access();
    let value = access
        .eval(Source::from_bytes(
            "globalThis.descriptorLog = []; ({
            get writable() { descriptorLog.push('writable'); return true; },
            get value() { descriptorLog.push('value'); return 42; },
            get configurable() { descriptorLog.push('configurable'); return false; },
            get enumerable() { descriptorLog.push('enumerable'); return true; }
        })",
        ))
        .unwrap();
    let descriptor = value.to_property_descriptor(access.reborrow()).unwrap();
    assert_eq!(descriptor.enumerable(), Some(true));
    assert_eq!(descriptor.configurable(), Some(false));
    assert_eq!(descriptor.value(), Some(&JsValue::new(42)));
    assert_eq!(descriptor.writable(), Some(true));
    assert_eq!(
        access
            .eval(Source::from_bytes("descriptorLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!("enumerable,configurable,value,writable"))
    );
}

#[test]
fn descriptor_getter_work_keeps_first_stop_and_skips_later_getters_and_guest_cleanup() {
    let mut owner = Context::default();
    let value = owner
        .access()
        .eval(Source::from_bytes(
            "({
            get enumerable() {
                globalThis.descriptorStarted = 7;
                try { while (true) {} } finally { globalThis.descriptorFinally = true; }
            },
            get configurable() { globalThis.descriptorLater = true; return true; }
        })",
        ))
        .unwrap();
    let outcome = owner
        .with_execution(ExecutionOptions::new(64), |mut access| {
            let first = value.to_property_descriptor(access.reborrow()).unwrap_err();
            assert_eq!(
                first.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 64
                }))
            );
            Ok(42_u32)
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 64
        }))
    );
    assert_eq!(outcome.report.instructions(), 64);
    assert!(outcome.report.effects_started());
    assert_eq!(owner.access().eval(Source::from_bytes("descriptorStarted === 7 && typeof descriptorLater === 'undefined' && typeof descriptorFinally === 'undefined'" )).unwrap(), JsValue::new(true));
    assert_eq!(
        owner.access().eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn object_properties_getters_and_key_mutations_compose_through_reborrowed_access() {
    let mut owner = Context::default();
    let mut access = owner.access();
    let object = access
        .eval(Source::from_bytes(
            "globalThis.propertyLog = []; ({
            base: 40,
            get value() { propertyLog.push('get'); return this.base + 2; },
            set value(next) { propertyLog.push('set:' + next); this.base = next - 2; }
        })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        object.get(js_string!("value"), access.reborrow()).unwrap(),
        JsValue::new(42)
    );
    assert!(
        object
            .set(js_string!("value"), 9, true, access.reborrow())
            .unwrap()
    );
    assert_eq!(
        object.get(js_string!("base"), access.reborrow()).unwrap(),
        JsValue::new(7)
    );
    assert!(
        object
            .define_property_or_throw(
                js_string!("bonus"),
                PropertyDescriptor::builder()
                    .value(35)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                access.reborrow(),
            )
            .unwrap()
    );
    assert!(
        object
            .has_own_property(js_string!("bonus"), access.reborrow())
            .unwrap()
    );
    assert_eq!(
        object.get(js_string!("bonus"), access.reborrow()).unwrap(),
        JsValue::new(35)
    );
    assert_eq!(
        object.own_property_keys(access.reborrow()).unwrap(),
        [
            PropertyKey::from(js_string!("base")),
            PropertyKey::from(js_string!("value")),
            PropertyKey::from(js_string!("bonus")),
        ]
    );
    assert!(
        object
            .delete_property_or_throw(js_string!("bonus"), access.reborrow())
            .unwrap()
    );
    assert!(
        !object
            .has_property(js_string!("bonus"), access.reborrow())
            .unwrap()
    );
    assert_eq!(
        access
            .eval(Source::from_bytes("propertyLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!("get,set:9"))
    );
}

#[test]
fn object_operations_preserve_guest_proxy_trap_order_and_results() {
    let mut owner = Context::default();
    let mut access = owner.access();
    let proxy = access.eval(Source::from_bytes(
        "globalThis.trapLog = []; new Proxy({ a: 7 }, {
            get(target, key, receiver) { trapLog.push('get:' + String(key)); return Reflect.get(target, key, receiver); },
            set(target, key, value) { trapLog.push('set:' + String(key)); return Reflect.set(target, key, value, target); },
            defineProperty(target, key, descriptor) { trapLog.push('define:' + String(key)); return Reflect.defineProperty(target, key, descriptor); },
            ownKeys(target) { trapLog.push('keys'); return Reflect.ownKeys(target); },
            deleteProperty(target, key) { trapLog.push('delete:' + String(key)); return Reflect.deleteProperty(target, key); }
        })",
    )).unwrap().as_object().unwrap();
    assert_eq!(
        proxy.get(js_string!("a"), access.reborrow()).unwrap(),
        JsValue::new(7)
    );
    assert!(
        proxy
            .set(js_string!("a"), 42, true, access.reborrow())
            .unwrap()
    );
    assert!(
        proxy
            .define_property_or_throw(
                js_string!("b"),
                PropertyDescriptor::builder()
                    .value(7)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build(),
                access.reborrow(),
            )
            .unwrap()
    );
    assert_eq!(
        proxy.own_property_keys(access.reborrow()).unwrap(),
        [
            PropertyKey::from(js_string!("a")),
            PropertyKey::from(js_string!("b")),
        ]
    );
    assert!(
        proxy
            .delete_property_or_throw(js_string!("b"), access.reborrow())
            .unwrap()
    );
    assert_eq!(
        access
            .eval(Source::from_bytes("trapLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!("get:a,set:a,define:b,keys,delete:b"))
    );
    assert_eq!(
        proxy.get(js_string!("a"), access.reborrow()).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn object_call_and_construct_preserve_receiver_results_and_instruction_admission() {
    let mut owner = Context::default();
    let callable = owner
        .access()
        .eval(Source::from_bytes(
            "(function add(delta) { globalThis.receiverSeen = this; return this.base + delta; })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let function = JsFunction::from_object(callable).unwrap();
    let receiver = owner
        .access()
        .eval(Source::from_bytes("({ base: 40 })"))
        .unwrap();
    let control = owner
        .with_execution(ExecutionOptions::new(128), |mut access| {
            function.call(&receiver, &[JsValue::new(2)], access.reborrow())
        })
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    assert_eq!(
        owner
            .access()
            .eval(Source::from_bytes("receiverSeen"))
            .unwrap(),
        receiver
    );
    let exact = control.report.instructions();
    assert!(exact > 1);
    for limit in [exact, exact - 1] {
        let outcome = owner
            .with_execution(ExecutionOptions::new(limit), |mut access| {
                function.call(&receiver, &[JsValue::new(2)], access.reborrow())
            })
            .unwrap();
        assert_eq!(outcome.report.instructions(), limit);
        if limit == exact {
            assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit
                }))
            );
        }
    }
    let constructor = owner.access().eval(Source::from_bytes(
        "(function Box(value) { this.value = value; this.correctTarget = new.target === Box; })",
    )).unwrap().as_object().unwrap();
    let constructor = JsFunction::from_object(constructor).unwrap();
    let constructed = owner
        .with_execution(ExecutionOptions::new(128), |mut access| {
            constructor.construct(&[JsValue::new(42)], None, access.reborrow())
        })
        .unwrap();
    assert!(constructed.report.instructions() > 0);
    let instance = constructed.result.unwrap();
    let mut access = owner.access();
    assert_eq!(
        instance
            .get(js_string!("value"), access.reborrow())
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        instance
            .get(js_string!("correctTarget"), access.reborrow())
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        access.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn direct_native_function_call_accepts_access_and_keeps_reentrant_guest_work_in_scope() {
    let native = NativeFunction::from_fn_ptr(|this, args, context| {
        assert_eq!(this, &JsValue::new(7));
        context.eval(Source::from_bytes("globalThis.nativeCommitted = 7"))?;
        Ok(args[0].clone())
    });
    let mut owner = Context::default();
    let outcome = owner
        .with_execution(ExecutionOptions::new(64), |mut access| {
            native.call(&JsValue::new(7), &[JsValue::new(42)], access.reborrow())
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert!(outcome.report.instructions() > 0);
    assert_eq!(
        owner
            .access()
            .eval(Source::from_bytes("nativeCommitted"))
            .unwrap(),
        JsValue::new(7)
    );
}

#[test]
fn same_runtime_peer_context_keeps_its_selected_budget_separate_from_a_retained_stop() {
    let mut original = Context::default();
    let mut peer = Context::builder()
        .runtime(original.runtime().clone())
        .build()
        .unwrap();
    assert_ne!(original.realm(), peer.realm());
    original
        .access()
        .eval(Source::from_bytes("globalThis.ownerValue = 7"))
        .unwrap();
    peer.access()
        .eval(Source::from_bytes("globalThis.ownerValue = 40"))
        .unwrap();
    let outcome = original
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let first = access
                .eval(Source::from_bytes("globalThis.originalEffect = true"))
                .unwrap_err();
            assert_eq!(
                first.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            let selected_peer = peer
                .with_execution(ExecutionOptions::new(64), |mut peer_access| {
                    peer_access.eval(Source::from_bytes(
                        "globalThis.peerValue = ownerValue + 2; peerValue",
                    ))
                })
                .unwrap();
            assert_eq!(selected_peer.result.unwrap(), JsValue::new(42));
            assert!(selected_peer.report.instructions() > 0);
            assert!(matches!(
                selected_peer.report.termination(),
                ExecutionTermination::Completed
            ));
            Ok(7_u32)
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions(), 0);
    assert!(!outcome.report.effects_started());
    assert_eq!(original.access().eval(Source::from_bytes(
        "ownerValue === 7 && typeof originalEffect === 'undefined' && typeof peerValue === 'undefined'",
    )).unwrap(), JsValue::new(true));
    assert_eq!(
        peer.access()
            .eval(Source::from_bytes("ownerValue === 40 && peerValue === 42"))
            .unwrap(),
        JsValue::new(true)
    );
    let fresh = original
        .with_execution(ExecutionOptions::new(64), |mut access| {
            access.eval(Source::from_bytes("ownerValue + 35"))
        })
        .unwrap();
    assert_eq!(fresh.result.unwrap(), JsValue::new(42));
    assert!(fresh.report.instructions() > 0);
}
