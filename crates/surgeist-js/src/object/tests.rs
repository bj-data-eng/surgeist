use crate::{JsNativeErrorKind, TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn trusted_property_installation_distinguishes_refusal_from_operation_errors() {
    use crate::{
        Context, JsObject,
        error::{EngineError, ObjectConstructionError, RootingError},
        js_string,
    };
    use surgeist_js_gc::heap::AccessError;

    let context = &mut Context::builder().build().unwrap();
    let object = JsObject::with_null_proto(context.runtime()).unwrap();
    let runtime = context.runtime().clone();
    runtime
        .heap
        .read(|_| {
            let error = object
                .create_non_enumerable_data_property_or_throw(js_string!("probe"), 7, context)
                .unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::Rooting(RootingError::Access(
                    AccessError::HeapBorrowed
                )))
            );
        })
        .unwrap();

    object
        .create_non_enumerable_data_property_or_throw(js_string!("kept"), 7, context)
        .unwrap();
    assert_eq!(
        object.get(js_string!("kept"), context.access()).unwrap(),
        7.into()
    );
    assert!(object.__prevent_extensions__(context).unwrap());
    let error = object
        .create_non_enumerable_data_property_or_throw(js_string!("refused"), 11, context)
        .unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectConstruction(
            ObjectConstructionError::PropertyRefused
        ))
    );
    let guest_error = object
        .create_data_property_or_throw(js_string!("refused"), 11, context.access())
        .unwrap_err();
    assert!(guest_error.as_native().unwrap().is_type());
    assert!(guest_error.as_engine().is_none());
}

#[test]
fn typed_object_views_preserve_brand_and_identity() {
    use crate::{JsObject, js_string, property::PropertyDescriptor};

    let typed = JsObject::new_unique(&crate::Runtime::default(), None, 12u32).unwrap();
    let erased = typed.clone().upcast();
    let wrong = erased.clone().downcast::<u64>().expect_err("wrong brand");
    assert_eq!(wrong, erased);
    assert_eq!(
        *erased.downcast_ref::<u32>().unwrap().expect("u32 payload"),
        12
    );

    let mut view = typed.borrow_mut().unwrap();
    assert!(erased.is::<u32>());
    assert!(erased.downcast_ref::<u64>().unwrap().is_none());
    assert!(erased.borrow().is_err());
    *view.data_mut() = 19;
    view.insert(
        js_string!("marker"),
        PropertyDescriptor::builder()
            .value(27)
            .writable(true)
            .enumerable(true)
            .configurable(true),
    )
    .unwrap();
    drop(view);

    let typed_again = erased.downcast::<u32>().expect("u32 brand");
    assert_eq!(typed_again, typed);
    assert_eq!(*typed_again.borrow().unwrap().data(), 19);
    assert!(
        typed_again
            .borrow()
            .unwrap()
            .properties()
            .contains_key(&js_string!("marker").into())
            .unwrap()
    );
}

#[test]
fn object_accepts_a_naturally_aligned_native_payload() {
    #[repr(align(32))]
    #[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
    struct Aligned(u32);

    let typed = crate::JsObject::new_unique(&crate::Runtime::default(), None, Aligned(5)).unwrap();
    let erased = typed.clone().upcast();
    let mut payload = erased
        .downcast_mut::<Aligned>()
        .unwrap()
        .expect("aligned payload");
    assert_eq!(
        std::ptr::from_ref(&*payload).addr() % align_of::<Aligned>(),
        0
    );
    payload.0 = 8;
    drop(payload);
    assert_eq!(typed.borrow().unwrap().data().0, 8);
}

#[test]
fn ordinary_has_instance_nonobject_prototype() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            function C() {}
            C.prototype = 1
            String instanceof C
        "#},
        JsNativeErrorKind::Type,
        "function has non-object prototype in instanceof check",
    )]);
}

#[test]
fn object_properties_return_order() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                var o = {
                    p1: 'v1',
                    p2: 'v2',
                    p3: 'v3',
                };
                o.p4 = 'v4';
                o[2] = 'iv2';
                o[0] = 'iv0';
                o[1] = 'iv1';
                delete o.p1;
                delete o.p3;
                o.p1 = 'v1';
            "#}),
        TestAction::assert(r#"arrayEquals(Object.keys(o), [ "0", "1", "2", "p2", "p4", "p1" ])"#),
        TestAction::assert(
            r#"arrayEquals(Object.values(o), [ "iv0", "iv1", "iv2", "v2", "v4", "v1" ])"#,
        ),
    ]);
}

#[test]
fn indexed_values_survive_collection_and_release_their_edges_on_removal() {
    use crate::{
        Context, JsObject, NativeFunction,
        object::{FunctionObjectBuilder, builtins::JsArray},
        property::PropertyDescriptor,
    };

    let context = &mut Context::builder().build().unwrap();
    for key in [0, 4, 8] {
        let child = JsObject::with_null_proto(context.runtime()).unwrap();
        let weak = child.edge();
        let object = if key == 0 {
            JsObject::from(JsArray::from_iter([child.into()], context.access()).unwrap())
        } else if key == 4 {
            let object = JsObject::with_null_proto(context.runtime()).unwrap();
            object
                .define_property_or_throw(
                    key,
                    PropertyDescriptor::builder()
                        .value(child)
                        .writable(true)
                        .enumerable(true)
                        .configurable(true),
                    context.access(),
                )
                .unwrap();
            object
        } else {
            let object = JsObject::with_null_proto(context.runtime()).unwrap();
            let getter = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_with_captures(
                    |_, _, child, _| Ok(child.clone().into()),
                    child,
                ),
            )
            .build()
            .unwrap();
            object
                .define_property_or_throw(
                    key,
                    PropertyDescriptor::builder().get(getter).configurable(true),
                    context.access(),
                )
                .unwrap();
            object
        };

        context.runtime().collect().unwrap();
        assert!(weak.is_live().unwrap());
        let extracted = object.get(key, context.access()).unwrap();
        assert!(
            object
                .delete_property_or_throw(key, context.access())
                .unwrap()
        );
        context.runtime().collect().unwrap();
        assert_eq!(
            extracted.as_object(),
            Some(
                weak.upgrade()
                    .unwrap()
                    .expect("extracted value owns the child")
            )
        );
        drop(extracted);
        context.runtime().collect().unwrap();
        assert!(!weak.is_live().unwrap());
    }
}

#[test]
fn private_field_keeps_its_value_until_replaced() {
    use crate::{Context, JsObject, JsValue, js_string};

    let context = &mut Context::builder().build().unwrap();
    let object = JsObject::with_null_proto(context.runtime()).unwrap();
    let name = object.private_name(js_string!("value"));
    let child = JsObject::with_null_proto(context.runtime()).unwrap();
    let weak = child.edge();
    object
        .private_field_add(&name, child.into(), context)
        .unwrap();
    context.runtime().collect().unwrap();
    assert!(weak.is_live().unwrap());

    let extracted = object.private_get(&name, context).unwrap();
    object
        .private_set(&name, JsValue::undefined(), context)
        .unwrap();
    context.runtime().collect().unwrap();
    assert_eq!(
        extracted.as_object(),
        Some(weak.upgrade().unwrap().expect("live extracted value"))
    );
    drop(extracted);
    context.runtime().collect().unwrap();
    assert!(!weak.is_live().unwrap());
}

#[test]
fn closure_keeps_its_environment_binding_alive() {
    use crate::{Context, JsValue, Source};

    let context = &mut Context::builder().build().unwrap();
    let closure = context
        .eval(Source::from_bytes(
            "(function () { let captured = {}; return () => captured; })()",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let weak = {
        let captured = closure
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap()
            .as_object()
            .unwrap();
        captured.edge()
    };

    context.runtime().collect().unwrap();
    {
        let expected = weak.upgrade().unwrap().expect("closure owns its binding");
        let captured = closure
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap();
        assert_eq!(captured.as_object(), Some(expected));
    }
    drop(closure);
    context.runtime().collect().unwrap();
    assert!(!weak.is_live().unwrap());
}

#[test]
fn native_accessor_keeps_captures_and_outcomes_during_collection() {
    use crate::{
        Context, JsError, JsObject, NativeFunction, builtins::OrdinaryObject, js_string,
        property::PropertyDescriptor,
    };

    let context = &mut Context::builder().build().unwrap();
    use std::num::NonZeroUsize;
    use surgeist_js_gc::heap::AllocationCollection;

    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::Opportunistic(NonZeroUsize::MIN))
        .unwrap();
    for throws in [false, true] {
        let allocation_canary = JsObject::with_null_proto(context.runtime()).unwrap().edge();
        assert!(allocation_canary.is_live().unwrap());
        let prototype = JsObject::with_null_proto(context.runtime()).unwrap();
        let receiver = JsObject::from_proto_and_data(
            context.runtime(),
            Some(prototype.clone()),
            OrdinaryObject,
        )
        .unwrap();
        assert!(!allocation_canary.is_live().unwrap());
        let captured = JsObject::with_null_proto(context.runtime()).unwrap();
        let weak_capture = captured.edge();
        let getter = NativeFunction::from_fn_with_captures(
            |this, _, (captured, throws), mut context| {
                let receiver = this.as_object().expect("accessor receiver");
                let active = {
                    let function = context
                        .reborrow()
                        .into_owner()
                        .active_function_object()
                        .expect("active getter");
                    function.edge()
                };
                receiver
                    .prototype()?
                    .expect("accessor prototype")
                    .delete_property_or_throw(js_string!("probe"), context.reborrow())?;

                let canary = {
                    let canary = JsObject::with_null_proto(context.runtime()).unwrap();
                    canary.edge()
                };
                assert!(canary.is_live().unwrap());
                context.runtime().collect().unwrap();
                assert!(!canary.is_live().unwrap());
                assert!(active.is_live().unwrap());

                receiver.set(
                    js_string!("nested"),
                    captured.clone(),
                    true,
                    context.reborrow(),
                )?;
                let value = receiver.get(js_string!("nested"), context.reborrow())?;
                receiver.delete_property_or_throw(js_string!("nested"), context.reborrow())?;
                if *throws {
                    Err(JsError::from_opaque(value)?)
                } else {
                    Ok(value)
                }
            },
            (captured, throws),
        )
        .to_js_function(context.realm())
        .unwrap();
        let weak_getter = {
            let object = JsObject::from(getter.clone());
            object.edge()
        };
        prototype
            .define_property_or_throw(
                js_string!("probe"),
                PropertyDescriptor::builder().get(getter).configurable(true),
                context.access(),
            )
            .expect("inherited accessor");

        let outcome = receiver.get(js_string!("probe"), context.access());
        let value = if throws {
            outcome
                .expect_err("opaque exception")
                .as_opaque()
                .expect("object payload")
                .clone()
        } else {
            outcome.expect("accessor result")
        };
        context.runtime().collect().unwrap();
        assert!(!weak_getter.is_live().unwrap());
        let expected = weak_capture.upgrade().unwrap().expect("live outcome");
        assert_eq!(value.as_object().as_ref(), Some(&expected));
        assert!(
            receiver
                .get(js_string!("probe"), context.access())
                .unwrap()
                .is_undefined()
        );
        drop((expected, value));
        context.runtime().collect().unwrap();
        assert!(!weak_capture.is_live().unwrap());
    }
}

#[test]
fn property_admission_distinguishes_peer_and_foreign_runtimes() {
    use crate::{Context, JsObject, JsValue};

    let mut context = Context::builder().build().unwrap();
    let peer = Context::builder()
        .runtime(context.runtime().clone())
        .build()
        .unwrap();
    let foreign = Context::builder().build().unwrap();
    let object = JsObject::with_null_proto(context.runtime()).unwrap();
    let local_child = JsObject::with_null_proto(peer.runtime()).unwrap();
    let foreign_child = JsObject::with_null_proto(foreign.runtime()).unwrap();

    assert!(
        object
            .set(0, local_child.clone(), true, context.access())
            .unwrap()
    );
    let error = object
        .set(0, foreign_child.clone(), true, context.access())
        .unwrap_err();
    assert!(
        error
            .as_native()
            .is_some_and(|error| error.kind() == &JsNativeErrorKind::Type)
    );
    assert!(
        object
            .insert_property(
                0,
                crate::property::PropertyDescriptor::builder().value(foreign_child.clone())
            )
            .is_err()
    );
    assert!(
        !object
            .borrow_mut()
            .unwrap()
            .set_prototype(Some(foreign_child))
            .unwrap()
    );
    context.runtime().collect().unwrap();
    assert_eq!(
        object.get(0, context.access()).unwrap(),
        JsValue::from(local_child)
    );
}

#[test]
fn object_builders_reject_foreign_properties_and_prototypes() {
    use crate::{
        Context, JsObject, JsValue, NativeFunction, js_string,
        object::{ConstructorBuilder, FunctionObjectBuilder, ObjectInitializer},
        property::{Attribute, PropertyDescriptor},
    };

    let mut context = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let foreign_object = JsObject::with_null_proto(foreign.runtime()).unwrap();
    let callback =
        |_: &JsValue, _: &[JsValue], _: crate::ContextAccess<'_>| Ok(JsValue::undefined());
    let foreign_getter =
        FunctionObjectBuilder::new(foreign.realm(), NativeFunction::from_fn_ptr(callback))
            .build()
            .unwrap();

    let result = ObjectInitializer::new(context.access())
        .property(
            js_string!("foreign"),
            foreign_object.clone(),
            Attribute::all(),
        )
        .build();
    assert!(result.unwrap_err().as_native().unwrap().is_type());

    let result = ObjectInitializer::new(context.access())
        .accessor(
            js_string!("foreign"),
            Some(foreign_getter),
            None,
            Attribute::all(),
        )
        .build();
    assert!(result.unwrap_err().as_native().unwrap().is_type());

    let mut builder =
        ConstructorBuilder::new(context.access(), NativeFunction::from_fn_ptr(callback));
    builder.static_property_descriptor(
        js_string!("foreign"),
        PropertyDescriptor::builder().value(foreign_object.clone()),
    );
    assert!(builder.build().unwrap_err().as_native().unwrap().is_type());

    let mut builder =
        ConstructorBuilder::new(context.access(), NativeFunction::from_fn_ptr(callback));
    builder.inherit(foreign_object);
    assert!(builder.build().unwrap_err().as_native().unwrap().is_type());

    let peer = Context::builder()
        .runtime(context.runtime().clone())
        .build()
        .unwrap();
    let peer_object = JsObject::with_null_proto(peer.runtime()).unwrap();
    let object = ObjectInitializer::new(context.access())
        .property(js_string!("peer"), peer_object.clone(), Attribute::all())
        .build()
        .unwrap();
    context.runtime().collect().unwrap();
    assert_eq!(
        object.get(js_string!("peer"), context.access()).unwrap(),
        peer_object.into()
    );
}

#[test]
fn function_builder_rejects_foreign_captures_before_publication() {
    use crate::{
        Context, JsObject, JsValue, NativeFunction, js_string, object::FunctionObjectBuilder,
    };

    let mut context = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let captured = JsObject::with_null_proto(foreign.runtime()).unwrap();
    let callback = NativeFunction::from_fn_with_captures(
        |_, _, object, _| Ok(JsValue::from(object.clone())),
        captured,
    );
    let error = FunctionObjectBuilder::new(context.realm(), callback.clone())
        .build()
        .unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    let key = js_string!("foreign");
    let error = context
        .register_global_builtin_callable(key.clone(), 0, callback)
        .unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    assert!(
        !context
            .global_object()
            .unwrap()
            .has_own_property(key, context.access())
            .unwrap()
    );
    context.runtime().collect().unwrap();
}

#[test]
fn object_builders_reject_access_from_a_mismatched_runtime() {
    use crate::{
        Context, JsValue, NativeFunction,
        object::{ConstructorBuilder, ObjectInitializer},
    };

    let mut context = Context::builder().build().unwrap();
    let original = context.runtime().clone();
    let mut foreign_object_access = Context::builder().build().unwrap();
    let mut object = ObjectInitializer::new(context.access());
    // Owning-unit injection retains defensive admission for mismatched custody.
    // Public builder getters cannot move or replace their Context owner.
    object.context = foreign_object_access.access();
    assert!(object.build().unwrap_err().as_native().unwrap().is_type());
    drop(object);

    let mut constructor = ConstructorBuilder::new(
        foreign_object_access.access(),
        NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
    );
    let second = constructor.context().runtime().clone();
    let mut foreign_constructor_access = Context::builder().build().unwrap();
    constructor.context = foreign_constructor_access.access();
    assert!(
        constructor
            .build()
            .unwrap_err()
            .as_native()
            .unwrap()
            .is_type()
    );

    original.collect().unwrap();
    second.collect().unwrap();
    foreign_constructor_access.runtime().collect().unwrap();
}

#[test]
fn constructor_static_descriptor_converts_the_descriptor_before_the_key() {
    use crate::{
        Context, JsValue, NativeFunction, js_string,
        object::ConstructorBuilder,
        property::{PropertyDescriptor, PropertyKey},
    };
    use std::cell::Cell;

    struct Descriptor<'a>(&'a Cell<u8>);
    struct Key<'a>(&'a Cell<u8>);

    impl From<Descriptor<'_>> for PropertyDescriptor {
        fn from(descriptor: Descriptor<'_>) -> Self {
            assert_eq!(descriptor.0.replace(1), 0);
            Self::builder().value(true).build()
        }
    }

    impl From<Key<'_>> for PropertyKey {
        fn from(key: Key<'_>) -> Self {
            assert_eq!(key.0.replace(2), 1);
            js_string!("property").into()
        }
    }

    let mut context = Context::builder().build().unwrap();
    let mut builder = ConstructorBuilder::new(
        context.access(),
        NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
    );
    let sequence = Cell::new(0);
    builder.static_property_descriptor(Key(&sequence), Descriptor(&sequence));
    assert_eq!(sequence.get(), 2);
    let constructor = builder.build().unwrap().constructor();
    assert_eq!(
        constructor
            .get(js_string!("property"), context.access())
            .unwrap(),
        true.into()
    );
}
