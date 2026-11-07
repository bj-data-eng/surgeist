#![allow(unused_crate_dependencies)]

use std::{cell::Cell, rc::Rc};

use surgeist_js::{
    Context, JsError, JsNativeError, JsNativeErrorKind, JsObject, JsString, JsValue,
    NativeFunction, Source,
    builtins::{
        error::{Error, ErrorKind},
        iterable::create_iter_result_object,
    },
    context::HostHooks,
    error::{EngineError, PanicError},
    job::{JobExecutor, NativeAsyncJob, SimpleJobExecutor},
    js_string,
    object::FunctionObjectBuilder,
    object::builtins::{JsArray, JsArrayBuffer, JsSharedArrayBuffer, JsTypedArray, JsUint8Array},
    property::Attribute,
    realm::Realm,
    script::Script,
};

fn peer_context(context: &Context) -> Context {
    Context::builder()
        .runtime(context.runtime().clone())
        .build()
        .unwrap()
}

#[test]
fn native_call_and_construct_keep_arguments_ordered_and_rooted_during_reentry() {
    let mut context = Context::default();
    context
        .register_global_callable(
            js_string!("captureArguments"),
            0,
            NativeFunction::from_fn_ptr(|_, args, context| {
                let nested = context.eval(Source::from_bytes("Math.max(2, 7, 3)"))?;
                assert_eq!(nested, 7.into());
                context.runtime().collect().unwrap();
                Ok(JsArray::from_iter(args.iter().cloned(), context)?.into())
            }),
        )
        .unwrap();

    for call in ["captureArguments", "new captureArguments"] {
        for (arguments, expected) in [
            ("fresh(1), fresh(2), fresh(3), fresh(4)", "1,2,3,4"),
            (
                "fresh(1), fresh(2), fresh(3), fresh(4), fresh(5)",
                "1,2,3,4,5",
            ),
        ] {
            let script = format!(
                "{{ const fresh = n => ({{n}}); ({call}({arguments})).map(value => value.n).join(',') }}"
            );
            let result = context.eval(Source::from_bytes(&script)).unwrap();
            assert_eq!(result, JsValue::from(JsString::from(expected)));
        }
    }
}

#[test]
fn native_call_and_construct_restore_the_caller_after_a_callback_switches_realms() {
    let mut context = Context::default();
    let caller = context.realm().clone();
    let other = context.create_realm().unwrap();
    let destination = context.create_realm().unwrap();
    context.enter_realm(destination.clone()).unwrap();
    let error_prototype = context
        .eval(Source::from_bytes("TypeError.prototype"))
        .unwrap()
        .as_object();
    context.enter_realm(caller.clone()).unwrap();

    for target in [&caller, &other] {
        let previous = context.enter_realm(target.clone()).unwrap();
        let expected = context.global_object();
        context.enter_realm(previous).unwrap();
        let function = FunctionObjectBuilder::new(
            target,
            NativeFunction::from_fn_with_captures(
                |_, args, (target, destination), context| {
                    assert!(context.realm() == target);
                    let object = context.global_object();
                    context.enter_realm(destination.clone())?;
                    context.runtime().collect().unwrap();
                    if args.is_empty() {
                        Ok(object.into())
                    } else {
                        Err(JsNativeError::typ().with_message("callback failed").into())
                    }
                },
                (target.clone(), destination.clone()),
            ),
        )
        .constructor(true)
        .build()
        .unwrap();
        let object: JsObject = function.into();

        for construct in [false, true] {
            let mut invoke = |args: &[JsValue]| {
                let result = if construct {
                    object.construct(args, None, &mut context).map(Into::into)
                } else {
                    object.call(&JsValue::undefined(), args, &mut context)
                };
                assert!(context.realm() == &caller);
                result
            };
            assert_eq!(invoke(&[]).unwrap().as_object(), Some(expected.clone()));
            let error = invoke(&[JsValue::undefined()]).unwrap_err();
            assert_eq!(error.as_native().unwrap().message(), "callback failed");
            let error = error
                .into_opaque(&mut context)
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(error.prototype(), error_prototype);
        }
    }
}

fn assert_foreign_runtime(error: JsError) {
    let native = error.as_native().expect("a native runtime admission error");
    assert_eq!(native.kind(), &JsNativeErrorKind::Type);
    assert_eq!(native.message(), "object belongs to a different runtime");
}

#[test]
fn a_cold_script_retains_its_literal_in_a_fresh_peer_context() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let script = Script::parse(
        Source::from_bytes("'surgeist-owner-literal'"),
        None,
        &mut owner,
    )
    .unwrap();

    assert_eq!(
        script.evaluate(&mut peer).unwrap(),
        JsValue::from(js_string!("surgeist-owner-literal"))
    );
}

#[test]
fn a_peer_cannot_substitute_another_literal_in_a_cold_script() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    peer.eval(Source::from_bytes("'surgeist-peer-literal'"))
        .unwrap();
    let script = Script::parse(
        Source::from_bytes("'surgeist-owner-literal'"),
        None,
        &mut owner,
    )
    .unwrap();

    for context in [&mut peer, &mut owner] {
        assert_eq!(
            script.evaluate(context).unwrap(),
            JsValue::from(js_string!("surgeist-owner-literal"))
        );
    }
}

#[test]
fn a_cached_script_reuses_code_only_within_its_runtime() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let mut foreign = Context::default();
    let script = Script::parse(Source::from_bytes("42"), None, &mut owner).unwrap();

    let code = script.codeblock(&mut owner).unwrap();
    drop(owner);
    peer.runtime().collect().unwrap();

    let error = script.codeblock(&mut foreign).unwrap_err();
    let native = error.as_native().expect("a native runtime admission error");
    assert_eq!(native.kind(), &JsNativeErrorKind::Type);
    assert_eq!(native.message(), "script belongs to a different runtime");
    assert!(Rc::ptr_eq(&code, &script.codeblock(&mut peer).unwrap()));
    assert_eq!(script.evaluate(&mut peer).unwrap(), 42.into());
}

#[test]
fn direct_eval_inside_a_compiled_class_resolves_only_its_own_symbols() {
    let mut context = Context::default();
    let value = context
        .eval(Source::from_bytes(
            "class Box {
                #surgeistPrivate = 17;
                get() { eval('0'); return eval('this.#surgeistPrivate'); }
             }
             new Box().get()",
        ))
        .unwrap();
    assert_eq!(value, 17.into());
}

#[test]
fn separate_contexts_parsing_in_one_realm_get_distinct_template_objects() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    peer.enter_realm(owner.realm().clone()).unwrap();
    let alpha = Script::parse(
        Source::from_bytes("(parts => parts)`surgeist-alpha`"),
        None,
        &mut owner,
    )
    .unwrap();
    let beta = Script::parse(
        Source::from_bytes("(parts => parts)`surgeist-beta`"),
        None,
        &mut peer,
    )
    .unwrap();
    let first = alpha.evaluate(&mut owner).unwrap();
    let second = beta.evaluate(&mut peer).unwrap();
    assert_ne!(first, second);
    assert_eq!(alpha.evaluate(&mut owner).unwrap(), first);
    assert_eq!(beta.evaluate(&mut peer).unwrap(), second);
    assert_eq!(
        second.as_object().unwrap().get(0, &mut peer).unwrap(),
        JsValue::from(js_string!("surgeist-beta"))
    );
}

#[test]
fn script_preparation_defers_explicit_realm_bindings_until_evaluation() {
    let mut context = Context::default();
    let initial = context.realm().clone();
    let target = context.create_realm().unwrap();
    let name = js_string!("surgeistExplicitRealmBinding");
    let script = Script::parse(
        Source::from_bytes("let surgeistExplicitRealmBinding = 11; surgeistExplicitRealmBinding"),
        Some(target.clone()),
        &mut context,
    )
    .unwrap();

    assert!(!initial.scope().has_binding(&name));
    assert!(!target.scope().has_binding(&name));
    script.codeblock(&mut context).unwrap();
    assert!(!initial.scope().has_binding(&name));
    assert!(!target.scope().has_binding(&name));
    assert_eq!(script.evaluate(&mut context).unwrap(), 11.into());
    assert!(!initial.scope().has_binding(&name));
    assert!(target.scope().has_binding(&name));
}

#[test]
fn a_shared_executor_resolves_atomic_waits_only_on_their_runtime() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut waiting = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut other = Context::builder().job_executor(executor).build().unwrap();
    waiting
        .eval(Source::from_bytes(
            "var status = 'pending';
             var words = new Int32Array(new SharedArrayBuffer(4));
             Atomics.waitAsync(words, 0, 0).value.then(result => { status = result; });
             Atomics.notify(words, 0);",
        ))
        .unwrap();

    other.run_jobs().unwrap();
    assert_eq!(
        waiting.eval(Source::from_bytes("status")).unwrap(),
        JsValue::from(js_string!("pending"))
    );
    waiting.run_jobs().unwrap();
    assert_eq!(
        waiting.eval(Source::from_bytes("status")).unwrap(),
        JsValue::from(js_string!("ok"))
    );
}

#[test]
fn dropping_the_last_context_removes_its_pending_atomic_waiter() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let backing = {
        let mut waiting = Context::builder()
            .job_executor(executor.clone())
            .build()
            .unwrap();
        let buffer = JsSharedArrayBuffer::new(4, &mut waiting).unwrap();
        let backing = buffer.inner();
        waiting
            .register_global_property(js_string!("buffer"), buffer, Attribute::all())
            .unwrap();
        assert_eq!(
            waiting
                .eval(Source::from_bytes(
                    "Atomics.waitAsync(new Int32Array(buffer), 0, 0).async",
                ))
                .unwrap(),
            true.into()
        );
        backing
    };

    let mut observer = Context::builder().job_executor(executor).build().unwrap();
    let buffer = JsSharedArrayBuffer::from_buffer(backing, &mut observer);
    observer
        .register_global_property(js_string!("buffer"), buffer, Attribute::all())
        .unwrap();
    let notified = observer
        .eval(Source::from_bytes(
            "Atomics.notify(new Int32Array(buffer), 0)",
        ))
        .unwrap();
    assert_eq!(notified, 0.into());
}

#[test]
fn a_surviving_peer_context_keeps_its_runtimes_queued_jobs() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut origin = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut peer = Context::builder()
        .runtime(origin.runtime().clone())
        .job_executor(executor)
        .build()
        .unwrap();
    let realm = origin.realm().clone();
    origin
        .eval(Source::from_bytes(
            "var status = 'pending';
             var words = new Int32Array(new SharedArrayBuffer(4));
             Atomics.waitAsync(words, 0, 0).value.then(result => { status = result; });
             Atomics.notify(words, 0);",
        ))
        .unwrap();
    drop(origin);

    peer.run_jobs().unwrap();
    peer.enter_realm(realm).unwrap();
    assert_eq!(
        peer.eval(Source::from_bytes("status")).unwrap(),
        JsValue::from(js_string!("ok"))
    );
}

#[test]
fn dropping_a_direct_executor_releases_job_captures() {
    struct Probe(Rc<Cell<bool>>);

    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let mut context = Context::default();
    let dropped = Rc::new(Cell::new(false));
    let probe = Probe(dropped.clone());
    Rc::new(SimpleJobExecutor::new()).enqueue_job(
        NativeAsyncJob::new(async move |_| {
            drop(probe);
            Ok(JsValue::undefined())
        })
        .into(),
        &mut context,
    );
    assert!(dropped.get());
}

#[test]
fn a_shared_executor_keeps_finalization_with_the_registering_runtime() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut other = Context::builder().job_executor(executor).build().unwrap();
    owner
        .eval(Source::from_bytes(
            "let total = 0;
             const registry = new FinalizationRegistry(value => { total += value; });
             registry.register({}, 31);",
        ))
        .unwrap();
    owner.runtime().collect().unwrap();
    other.run_jobs().unwrap();
    assert_eq!(owner.eval(Source::from_bytes("total")).unwrap(), 0.into());
    owner.run_jobs().unwrap();
    assert_eq!(owner.eval(Source::from_bytes("total")).unwrap(), 31.into());
}

#[test]
fn dynamic_function_host_hook_receives_coerced_fragments_before_parsing() {
    struct RejectSource;

    impl HostHooks for RejectSource {
        fn ensure_can_compile_strings(
            &self,
            _: Realm,
            parameters: &[JsString],
            body: &JsString,
            direct: bool,
            _: &mut Context,
        ) -> surgeist_js::JsResult<()> {
            assert_eq!(parameters, &[js_string!("π"), js_string!("")]);
            assert_eq!(body, &js_string!("return @"));
            assert!(!direct);
            Err(JsNativeError::syntax().with_message("host veto").into())
        }
    }

    for constructor in [
        "Function",
        "(function* () {}).constructor",
        "(async function () {}).constructor",
        "(async function* () {}).constructor",
    ] {
        let mut context = Context::builder()
            .host_hooks(Rc::new(RejectSource))
            .build()
            .unwrap();
        let script = format!(
            "globalThis.order = '';
             const arg = (label, source) => ({{ toString() {{ order += label; return source; }} }});
             ({constructor})(arg('p', 'π'), arg('q', ''), arg('b', 'return @'));"
        );
        let error = context.eval(Source::from_bytes(&script)).unwrap_err();
        assert_eq!(error.as_native().unwrap().message(), "host veto");
        let order = context.eval(Source::from_bytes("order")).unwrap();
        assert_eq!(order, js_string!("pqb").into());
    }
}

#[test]
fn compile_hook_engine_panic_remains_uncatchable_with_a_foreign_diagnostic() {
    struct Hook(JsError);

    impl HostHooks for Hook {
        fn ensure_can_compile_strings(
            &self,
            _: Realm,
            _: &[JsString],
            _: &JsString,
            _: bool,
            _: &mut Context,
        ) -> surgeist_js::JsResult<()> {
            Err(self.0.clone())
        }
    }

    let foreign = Context::default();
    let diagnostic = JsError::from_opaque(foreign.global_object().into());
    let panic = PanicError::new("compile hook panic").with_source(diagnostic);
    for error in [
        panic.clone().into(),
        JsNativeError::aggregate(vec![panic.into()]).into(),
    ] {
        let mut context = Context::builder()
            .host_hooks(std::rc::Rc::new(Hook(error)))
            .build()
            .unwrap();
        let error = context
            .eval(Source::from_bytes(
                "try { eval('0'); } catch (error) { error.name; }",
            ))
            .unwrap_err();
        let Some(EngineError::Panic(panic)) = error.as_engine() else {
            panic!("an engine panic became catchable");
        };
        assert_eq!(panic.message(), "compile hook panic");
        context.runtime().collect().unwrap();
    }
}

#[test]
fn aggregate_conversion_preserves_an_engine_member_with_a_foreign_diagnostic() {
    let mut context = Context::default();
    let foreign = Context::default();
    let diagnostic = JsError::from_opaque(foreign.global_object().into());
    let engine = EngineError::from(PanicError::new("aggregate panic").with_source(diagnostic));
    let error = JsNativeError::aggregate(vec![engine.clone().into()])
        .into_opaque(&mut context)
        .unwrap_err();
    assert_eq!(error.as_engine(), Some(&engine));
}

#[test]
fn failed_aggregate_conversion_leaves_existing_error_stack_unmodified() {
    let mut context = Context::default();
    let prototype = context
        .eval(Source::from_bytes("Error.prototype"))
        .unwrap()
        .as_object()
        .unwrap();
    let retained = JsObject::new_unique(context.runtime(), prototype, Error::new(ErrorKind::Error))
        .unwrap()
        .upcast();
    context
        .register_global_property(js_string!("retained"), retained.clone(), Attribute::all())
        .unwrap();
    let thrown = context
        .eval(Source::from_bytes(
            "function raise() { throw retained; } raise();",
        ))
        .unwrap_err();
    assert!(
        retained
            .get(js_string!("stack"), &mut context)
            .unwrap()
            .is_undefined()
    );

    let engine = EngineError::from(PanicError::new("unmaterializable member"));
    let error = JsNativeError::aggregate(vec![thrown, engine.clone().into()])
        .into_opaque(&mut context)
        .unwrap_err();
    assert_eq!(error.as_engine(), Some(&engine));
    assert!(
        retained
            .get(js_string!("stack"), &mut context)
            .unwrap()
            .is_undefined()
    );
}

fn next_value(iterator: JsValue, context: &mut Context) -> JsValue {
    let next = iterator
        .as_object()
        .unwrap()
        .get(js_string!("next"), context)
        .unwrap()
        .as_callable()
        .unwrap();
    let result = next
        .call(&iterator, &[], context)
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        result.get(js_string!("done"), context).unwrap(),
        false.into()
    );
    result.get(js_string!("value"), context).unwrap()
}

#[test]
fn array_values_refuses_a_foreign_receiver_and_iterates_in_a_peer_context() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let mut foreign = Context::default();
    let element: JsValue = JsObject::with_null_proto(owner.runtime()).into();
    let array = JsArray::from_iter([element.clone()], &mut owner).unwrap();

    assert_foreign_runtime(array.values(&mut foreign).unwrap_err());

    let iterator = array.values(&mut peer).unwrap();
    assert_eq!(next_value(iterator, &mut peer), element);
}

#[test]
fn array_with_refuses_a_foreign_replacement_and_retains_a_peer_replacement() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let foreign = Context::default();
    let array = JsArray::from_iter([1.into(), 2.into()], &mut owner).unwrap();

    let foreign_replacement = JsObject::with_null_proto(foreign.runtime());
    assert_foreign_runtime(
        array
            .with(1, foreign_replacement.into(), &mut owner)
            .unwrap_err(),
    );

    let replacement: JsValue = JsObject::with_null_proto(peer.runtime()).into();
    let copy = array.with(1, replacement.clone(), &mut peer).unwrap();
    assert_eq!(array.get(1, &mut owner).unwrap(), 2.into());
    assert_eq!(copy.get(0, &mut peer).unwrap(), 1.into());
    assert_eq!(copy.get(1, &mut peer).unwrap(), replacement);
}

#[test]
fn typed_array_iterators_refuse_a_foreign_receiver_and_iterate_in_a_peer_context() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let mut foreign = Context::default();
    let array = JsUint8Array::from_iter([19], &mut owner).unwrap();

    for make_iterator in [
        JsTypedArray::entries as fn(&JsTypedArray, &mut Context) -> _,
        JsTypedArray::keys,
        JsTypedArray::values,
        JsTypedArray::iterator,
    ] {
        assert_foreign_runtime(make_iterator(&array, &mut foreign).unwrap_err());
    }

    let iterator = array.entries(&mut peer).unwrap();
    let entry = next_value(iterator, &mut peer).as_object().unwrap();
    assert_eq!(entry.get(0, &mut peer).unwrap(), 0.into());
    assert_eq!(entry.get(1, &mut peer).unwrap(), 19.into());
    let iterator = array.keys(&mut peer).unwrap();
    assert_eq!(next_value(iterator, &mut peer), 0.into());
    let iterator = array.values(&mut peer).unwrap();
    assert_eq!(next_value(iterator, &mut peer), 19.into());
    let iterator = array.iterator(&mut peer).unwrap();
    assert_eq!(next_value(iterator, &mut peer), 19.into());
}

#[test]
fn typed_array_from_array_buffer_refuses_a_foreign_buffer_and_retains_a_peer_buffer() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let mut foreign = Context::default();
    let foreign_buffer = JsArrayBuffer::new(1, &mut foreign).unwrap();
    assert_foreign_runtime(
        JsUint8Array::from_array_buffer(foreign_buffer, &mut owner).unwrap_err(),
    );

    let buffer = JsArrayBuffer::from_byte_block(vec![23], &mut peer).unwrap();
    let view = JsUint8Array::from_array_buffer(buffer.clone(), &mut owner).unwrap();
    assert_eq!(view.buffer(&mut owner).unwrap(), JsValue::from(buffer));
    assert_eq!(view.get(0, &mut owner).unwrap(), 23.into());
}

#[test]
fn typed_array_from_shared_array_buffer_refuses_a_foreign_buffer_and_retains_a_peer_buffer() {
    let mut owner = Context::default();
    let mut peer = peer_context(&owner);
    let mut foreign = Context::default();
    // Empty shared buffers also work on targets without lock-free 64-bit atomics.
    let foreign_buffer = JsSharedArrayBuffer::new(0, &mut foreign).unwrap();
    assert_foreign_runtime(
        JsUint8Array::from_shared_array_buffer(foreign_buffer, &mut owner).unwrap_err(),
    );

    let buffer = JsSharedArrayBuffer::new(0, &mut peer).unwrap();
    let view = JsUint8Array::from_shared_array_buffer(buffer.clone(), &mut owner).unwrap();
    assert_eq!(view.buffer(&mut owner).unwrap(), JsValue::from(buffer));
    assert_eq!(view.length(&mut owner).unwrap(), 0);
}

#[test]
fn iterator_result_refuses_a_foreign_value_without_panicking() {
    let mut owner = Context::default();
    let peer = peer_context(&owner);
    let foreign = Context::default();
    let foreign_value = JsObject::with_null_proto(foreign.runtime()).into();
    assert_foreign_runtime(
        create_iter_result_object(foreign_value, false, &mut owner).unwrap_err(),
    );

    let value: JsValue = JsObject::with_null_proto(peer.runtime()).into();
    let result = create_iter_result_object(value.clone(), true, &mut owner)
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(result.get(js_string!("value"), &mut owner).unwrap(), value);
    assert_eq!(
        result.get(js_string!("done"), &mut owner).unwrap(),
        true.into()
    );
}

#[test]
fn dropping_parsed_or_compiled_scripts_preserves_the_realm_scope_identity_sequence() {
    let identities = [false, true].map(|compile| {
        let mut context = Context::default();
        let script = Script::parse(
            Source::from_bytes("(() => 42); (class {});"),
            None,
            &mut context,
        )
        .unwrap();
        if compile {
            script.codeblock(&mut context).unwrap();
        }
        drop(script);

        let next = surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false);
        (compile, next.unique_id())
    });
    assert_eq!(identities, [(false, 1), (true, 1)]);
}

#[test]
fn pending_scripts_can_share_lexical_names_until_actual_evaluation() {
    let mut context = Context::default();
    let first = Script::parse(
        Source::from_bytes("let pending = 40; pending + 2"),
        None,
        &mut context,
    )
    .unwrap();
    let second = Script::parse(
        Source::from_bytes("let pending = 41; pending + 1"),
        None,
        &mut context,
    )
    .expect("independent pending scripts do not reserve each other's names");
    let first_code = first.codeblock(&mut context).unwrap();
    second.codeblock(&mut context).unwrap();
    assert!(Rc::ptr_eq(
        &first_code,
        &first.codeblock(&mut context).unwrap()
    ));
    assert!(!context.realm().scope().has_binding(&js_string!("pending")));

    assert_eq!(first.evaluate(&mut context).unwrap(), JsValue::new(42));
    assert!(context.realm().scope().has_binding(&js_string!("pending")));
    second
        .codeblock(&mut context)
        .expect("compiled inspection does not instantiate declarations");
    let conflict = second.evaluate(&mut context).unwrap_err();
    assert!(conflict.as_native().is_some_and(JsNativeError::is_syntax));
}

#[test]
fn existing_declaration_conflicts_are_evaluation_errors_after_pure_preparation() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes("let occupied = 1;"))
        .unwrap();
    let script = Script::parse(
        Source::from_bytes("let occupied = 2; occupied"),
        None,
        &mut context,
    )
    .expect("the new script has valid syntax independently of the realm");
    script.codeblock(&mut context).unwrap();

    let conflict = script.evaluate(&mut context).unwrap_err();
    assert!(conflict.as_native().is_some_and(JsNativeError::is_syntax));
    assert_eq!(
        context.eval(Source::from_bytes("occupied")).unwrap(),
        JsValue::new(1)
    );
}

#[test]
fn installed_script_cache_survives_unrelated_realm_changes() {
    let mut context = Context::default();
    let script = Script::parse(Source::from_bytes("(() => 42)()"), None, &mut context).unwrap();
    let code = script.codeblock(&mut context).unwrap();
    assert_eq!(script.evaluate(&mut context).unwrap(), JsValue::new(42));

    context
        .eval(Source::from_bytes("let unrelated = 1;"))
        .unwrap();
    assert!(Rc::ptr_eq(&code, &script.codeblock(&mut context).unwrap()));
    assert_eq!(script.evaluate(&mut context).unwrap(), JsValue::new(42));
}

#[test]
fn an_older_captured_environment_reads_a_new_global_lexical_binding() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.shadowed = 1;
             globalThis.readShadowed = (() => {
                 const kept = 1;
                 return () => kept + shadowed;
             })();",
        ))
        .unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("readShadowed()")).unwrap(),
        JsValue::new(2)
    );

    context
        .eval(Source::from_bytes("let shadowed = 41;"))
        .unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("readShadowed()")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn an_older_captured_environment_writes_a_new_global_lexical_binding() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.shadowed = 1;
             globalThis.writeShadowed = (() => {
                 const kept = 1;
                 return value => shadowed = kept + value;
             })();
             writeShadowed(1);",
        ))
        .unwrap();
    context
        .eval(Source::from_bytes("let shadowed = 41;"))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "writeShadowed(41); shadowed === 42 && globalThis.shadowed === 2",
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn typeof_in_an_older_captured_environment_observes_a_new_global_lexical_binding() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.shadowed = 1;
             globalThis.typeOfShadowed = (() => {
                 const kept = 1;
                 return () => kept && typeof shadowed;
             })();",
        ))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("typeOfShadowed()"))
            .unwrap(),
        JsValue::from(js_string!("number"))
    );
    context
        .eval(Source::from_bytes("let shadowed = 'lexical';"))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("typeOfShadowed()"))
            .unwrap(),
        JsValue::from(js_string!("string"))
    );
}

#[test]
fn new_global_lexical_bindings_preserve_local_with_and_eval_resolution() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "globalThis.shadowed = 1;
             globalThis.readLocal = (() => {
                 const shadowed = 7;
                 return () => shadowed;
             })();
             globalThis.readWith = (() => {
                 const kept = 1;
                 with ({shadowed: 10}) { return () => kept + shadowed; }
             })();
             globalThis.readUnscopable = (() => {
                 const kept = 1;
                 with ({shadowed: 100, [Symbol.unscopables]: {shadowed: true}}) {
                     return () => kept + shadowed;
                 }
             })();
             globalThis.readEval = (() => {
                 eval('var shadowed = 13');
                 return () => shadowed;
             })();",
        ))
        .unwrap();
    let read = "[readLocal(), readWith(), readUnscopable(), readEval()].join(',')";
    assert_eq!(
        context.eval(Source::from_bytes(read)).unwrap(),
        JsValue::from(js_string!("7,11,2,13"))
    );
    context
        .eval(Source::from_bytes("let shadowed = 41;"))
        .unwrap();
    assert_eq!(
        context.eval(Source::from_bytes(read)).unwrap(),
        JsValue::from(js_string!("7,11,42,13"))
    );
}

#[cfg(feature = "annex-b")]
#[test]
fn annex_b_compilation_does_not_create_a_global_property() {
    let mut context = Context::default();
    let global = context.global_object();
    let name = js_string!("pendingAnnexFunction");
    let script = Script::parse(
        Source::from_bytes(
            "{ function pendingAnnexFunction() { return 42; } } pendingAnnexFunction()",
        ),
        None,
        &mut context,
    )
    .unwrap();
    assert!(!global.has_own_property(name.clone(), &mut context).unwrap());
    script.codeblock(&mut context).unwrap();
    assert!(!global.has_own_property(name.clone(), &mut context).unwrap());

    assert_eq!(script.evaluate(&mut context).unwrap(), JsValue::new(42));
    assert!(global.has_own_property(name, &mut context).unwrap());
}

#[test]
fn a_stale_pending_script_rebuilds_slots_without_replacing_installed_code() {
    let mut context = Context::default();
    let script = Script::parse(
        Source::from_bytes("let pending = seed + 2; pending"),
        None,
        &mut context,
    )
    .unwrap();
    let before = script.codeblock(&mut context).unwrap();
    context.eval(Source::from_bytes("let seed = 40;")).unwrap();
    let refreshed = script.codeblock(&mut context).unwrap();
    assert!(!Rc::ptr_eq(&before, &refreshed));
    assert!(!context.realm().scope().has_binding(&js_string!("pending")));
    assert_eq!(script.evaluate(&mut context).unwrap(), JsValue::new(42));
    assert!(Rc::ptr_eq(
        &refreshed,
        &script.codeblock(&mut context).unwrap()
    ));
    assert_eq!(
        context.eval(Source::from_bytes("seed")).unwrap(),
        JsValue::new(40)
    );
}

#[test]
fn global_var_names_preserve_source_order_and_instantiate_once_in_a_proxy_global() {
    #[derive(Debug)]
    struct Global(JsObject);
    impl HostHooks for Global {
        fn create_global_object(
            &self,
            _: &surgeist_js::context::intrinsics::Intrinsics,
        ) -> JsObject {
            self.0.clone()
        }
    }
    let mut owner = Context::default();
    let global = owner
        .eval(Source::from_bytes(
            "globalThis.events = [];
         new Proxy({}, {
             getOwnPropertyDescriptor(target, key) {
                 if (['first', 'second', 'third'].includes(key)) events.push('own:' + key);
                 return Reflect.getOwnPropertyDescriptor(target, key);
             },
             isExtensible(target) {
                 events.push('extensible');
                 return Reflect.isExtensible(target);
             },
             defineProperty(target, key, descriptor) {
                 if (['first', 'second', 'third'].includes(key)) events.push('define:' + key);
                 return Reflect.defineProperty(target, key, descriptor);
             }
         })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let mut context = Context::builder()
        .runtime(owner.runtime().clone())
        .host_hooks(Rc::new(Global(global)))
        .build()
        .unwrap();
    owner
        .eval(Source::from_bytes("events.length = 0;"))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "var first, second, third; var second; 42"
            ))
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        owner.eval(Source::from_bytes("events.join(',')")).unwrap(),
        JsValue::from(js_string!(
            "own:first,extensible,own:second,extensible,own:third,extensible,own:first,extensible,define:first,own:second,extensible,define:second,own:third,extensible,define:third"
        ))
    );
}

#[test]
fn a_host_cannot_hide_a_final_publication_conflict_from_the_outer_operation() {
    use surgeist_js::execution::ExecutionOptions;
    use surgeist_js_ast::scope::ScopeStateError;

    #[derive(Debug)]
    struct Global(JsObject);
    impl HostHooks for Global {
        fn create_global_object(
            &self,
            _: &surgeist_js::context::intrinsics::Intrinsics,
        ) -> JsObject {
            self.0.clone()
        }
    }
    struct CaughtConflict(JsError);

    let mut context = Context::default();
    let global = context
        .eval(Source::from_bytes(
            "globalThis.publicationMutationRuns = 0;
             globalThis.continuedAfterPublicationConflict = false;
             new Proxy({}, {
                 getOwnPropertyDescriptor(target, key) {
                     if (key === 'pendingPublicationFunction' && publicationMutationRuns === 0) {
                         publicationMutationRuns++;
                         mutatePendingRealm();
                     }
                     return Reflect.getOwnPropertyDescriptor(target, key);
                 }
             })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let mut peer = Context::builder()
        .runtime(context.runtime().clone())
        .host_hooks(Rc::new(Global(global.clone())))
        .build()
        .unwrap();
    let pending_realm = peer.realm().clone();
    let mutate = FunctionObjectBuilder::new(
        &pending_realm,
        NativeFunction::from_fn_ptr(|_, _, context| {
            context.eval(Source::from_bytes("let callbackOwned = 1;"))
        }),
    )
    .build()
    .unwrap();
    context
        .register_global_property(js_string!("mutatePendingRealm"), mutate, Attribute::all())
        .unwrap();

    let pending = Script::parse(
        Source::from_bytes("function pendingPublicationFunction() { return 42; } 42"),
        None,
        &mut peer,
    )
    .unwrap();
    pending.codeblock(&mut peer).unwrap();
    // Rust host custody retains the prepared Script without adding a JavaScript
    // callable or a new NativeCapture representation for pending code.
    context.insert_data(pending);
    context
        .register_global_callable(
            js_string!("catchPublicationConflict"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let pending = context.get_data::<Script>().unwrap().clone();
                let error = pending.evaluate(context).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
                );
                context.insert_data(CaughtConflict(error));
                Ok(JsValue::new(17))
            }),
        )
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes(
                "catchPublicationConflict(); continuedAfterPublicationConflict = true; 99",
            ),
            ExecutionOptions::new(4096),
        )
        .unwrap();
    let caught = &context.get_data::<CaughtConflict>().unwrap().0;
    assert_eq!(
        caught.as_engine(),
        Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
    );
    assert!(outcome.report.effects_started());
    assert_eq!(
        context
            .eval(Source::from_bytes("publicationMutationRuns"))
            .unwrap(),
        JsValue::new(1)
    );
    assert_eq!(
        peer.eval(Source::from_bytes("callbackOwned")).unwrap(),
        JsValue::new(1)
    );
    assert!(
        !global
            .has_own_property(js_string!("pendingPublicationFunction"), &mut peer)
            .unwrap()
    );
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes("continuedAfterPublicationConflict"))
            .unwrap(),
        JsValue::new(false)
    );
}
