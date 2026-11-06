#![allow(unused_crate_dependencies)]

use std::{cell::Cell, rc::Rc};

use surgeist_js::{
    Context, JsError, JsNativeError, JsNativeErrorKind, JsObject, JsString, JsValue, Source,
    builtins::{
        error::{Error, ErrorKind},
        iterable::create_iter_result_object,
    },
    context::HostHooks,
    error::{EngineError, PanicError},
    job::{JobExecutor, NativeAsyncJob, SimpleJobExecutor},
    js_string,
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
fn script_parsing_uses_the_explicit_realm_scope() {
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
    assert!(target.scope().has_binding(&name));
    assert_eq!(script.evaluate(&mut context).unwrap(), 11.into());
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
