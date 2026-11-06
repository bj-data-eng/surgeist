#![allow(unused_crate_dependencies)]

use surgeist_js::{
    Context, JsError, JsNativeError, JsNativeErrorKind, JsObject, JsString, JsValue, Source,
    builtins::{
        error::{Error, ErrorKind},
        iterable::create_iter_result_object,
    },
    context::HostHooks,
    error::{EngineError, PanicError},
    js_string,
    object::builtins::{JsArray, JsArrayBuffer, JsSharedArrayBuffer, JsTypedArray, JsUint8Array},
    property::Attribute,
    realm::Realm,
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
