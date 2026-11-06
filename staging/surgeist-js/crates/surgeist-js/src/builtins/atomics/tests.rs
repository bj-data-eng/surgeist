use std::{rc::Rc, time::Duration};

use crate::{
    Context, JsValue, TestAction,
    builtins::atomics::Atomics,
    js_string,
    module::IdleModuleLoader,
    object::{JsInt32Array, JsPromise, JsSharedArrayBuffer},
    run_test_actions,
    value::TryFromJs,
};

#[test]
fn atomics_revalidate_resizable_buffers_after_argument_coercion() {
    run_test_actions([
        TestAction::run(
            r#"
            var backing = new ArrayBuffer(8, {maxByteLength: 8});
            var tracking = new Int32Array(backing);
            tracking[0] = 42;
            var loaded = Atomics.load(tracking, { valueOf() { backing.resize(4); return 0; } });

            backing.resize(8);
            var rangeError = false;
            try {
                Atomics.store(tracking, 1, { valueOf() { backing.resize(4); return 7; } });
            } catch (error) {
                rangeError = error instanceof RangeError;
            }

            backing.resize(8);
            var fixed = new Int32Array(backing, 0, 2);
            var typeError = false;
            try {
                Atomics.add(fixed, 0, { valueOf() { backing.resize(4); return 1; } });
            } catch (error) {
                typeError = error instanceof TypeError;
            }

            backing.resize(8);
            var conversions = [];
            var previous = Atomics.compareExchange(
                tracking,
                0,
                { valueOf() { conversions.push('expected'); return 42; } },
                { valueOf() { conversions.push('replacement'); backing.resize(4); return 9; } },
            );
            "#,
        ),
        TestAction::assert("loaded === 42 && rangeError && typeError"),
        TestAction::assert("previous === 42 && tracking[0] === 9"),
        TestAction::assert("conversions.join(',') === 'expected,replacement'"),
    ]);
}

#[test]
fn waiterlist_block_indexedposition_wake() {
    const NUMAGENT: i32 = 2;
    const RUNNING: i32 = 4;

    let context = &mut Context::builder()
        .can_block(true)
        .module_loader(Rc::new(IdleModuleLoader))
        .build()
        .unwrap();
    let buffer = JsSharedArrayBuffer::new(size_of::<i32>() * 5, context).unwrap();
    let inner_buffer = buffer.inner();
    let i32a = JsInt32Array::from_shared_array_buffer(buffer.clone(), context).unwrap();

    std::thread::scope(|s| {
        let mut threads = Vec::new();
        for idx in [2, 0] {
            let buffer = inner_buffer.clone();
            let handle = s.spawn(move || {
                let context = &mut Context::builder()
                    .can_block(true)
                    .module_loader(Rc::new(IdleModuleLoader))
                    .build()
                    .unwrap();
                let buffer = JsSharedArrayBuffer::from_buffer(buffer, context);
                let i32a = JsInt32Array::from_shared_array_buffer(buffer, context).unwrap();

                Atomics::add(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), RUNNING.into(), 1.into()],
                    context,
                )
                .unwrap();

                let promise = JsPromise::try_from_js(
                    &Atomics::wait::<true>(
                        &JsValue::undefined(),
                        &[
                            i32a.clone().into(),
                            idx.into(),
                            0.into(),
                            f64::INFINITY.into(),
                        ],
                        context,
                    )
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .get(js_string!("value"), context)
                    .unwrap(),
                    context,
                )
                .unwrap();

                assert_eq!(
                    promise
                        .await_blocking(context)
                        .unwrap()
                        .to_string(context)
                        .unwrap()
                        .to_std_string_lossy(),
                    "ok"
                );
            });
            threads.push(handle);
        }

        while Atomics::load(
            &JsValue::undefined(),
            &[i32a.clone().into(), RUNNING.into()],
            context,
        )
        .unwrap()
        .to_i32(context)
        .unwrap()
            != NUMAGENT
        {}

        std::thread::sleep(Duration::from_millis(100));

        assert_eq!(
            Atomics::load(
                &JsValue::undefined(),
                &[i32a.clone().into(), RUNNING.into()],
                context,
            )
            .unwrap()
            .to_i32(context)
            .unwrap(),
            NUMAGENT
        );

        for missing in [1, 3] {
            assert_eq!(
                Atomics::notify(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), missing.into()],
                    context
                )
                .unwrap()
                .to_i32(context)
                .unwrap(),
                0
            );
        }

        for exists in [2, 0] {
            assert_eq!(
                Atomics::notify(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), exists.into()],
                    context
                )
                .unwrap()
                .to_i32(context)
                .unwrap(),
                1
            );
        }
    });
}
