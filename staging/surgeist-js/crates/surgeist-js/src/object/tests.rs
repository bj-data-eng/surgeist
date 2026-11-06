use crate::{JsNativeErrorKind, TestAction, run_test_actions};
use indoc::indoc;

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
fn native_accessor_keeps_captures_and_outcomes_during_collection() {
    use crate::{
        Context, JsError, JsObject, NativeFunction, builtins::OrdinaryObject, js_string,
        property::PropertyDescriptor,
    };
    use surgeist_js_gc::{WeakGc, force_collect};

    let context = &mut Context::default();
    for throws in [false, true] {
        let prototype = JsObject::from_proto_and_data(None, OrdinaryObject);
        let receiver = JsObject::from_proto_and_data(Some(prototype.clone()), OrdinaryObject);
        let captured = JsObject::from_proto_and_data(None, OrdinaryObject);
        let weak_capture = WeakGc::new(captured.inner());
        let getter = NativeFunction::from_copy_closure_with_captures(
            |this, _, (captured, throws), context| {
                let receiver = this.as_object().expect("accessor receiver");
                receiver
                    .prototype()
                    .expect("accessor prototype")
                    .delete_property_or_throw(js_string!("probe"), context)?;

                let canary = {
                    let canary = JsObject::from_proto_and_data(None, OrdinaryObject);
                    WeakGc::new(canary.inner())
                };
                assert!(canary.is_upgradable());
                force_collect();
                assert!(!canary.is_upgradable());

                receiver.set(js_string!("nested"), captured.clone(), true, context)?;
                let value = receiver.get(js_string!("nested"), context)?;
                receiver.delete_property_or_throw(js_string!("nested"), context)?;
                if *throws {
                    Err(JsError::from_opaque(value))
                } else {
                    Ok(value)
                }
            },
            (captured, throws),
        )
        .to_js_function(context.realm());
        prototype
            .define_property_or_throw(
                js_string!("probe"),
                PropertyDescriptor::builder().get(getter).configurable(true),
                context,
            )
            .expect("inherited accessor");

        let outcome = receiver.get(js_string!("probe"), context);
        let value = if throws {
            outcome
                .expect_err("opaque exception")
                .as_opaque()
                .expect("object payload")
                .clone()
        } else {
            outcome.expect("accessor result")
        };
        force_collect();
        let expected = JsObject::from_inner(weak_capture.upgrade().expect("live outcome"));
        assert_eq!(value.as_object().as_ref(), Some(&expected));
        assert!(
            receiver
                .get(js_string!("probe"), context)
                .unwrap()
                .is_undefined()
        );
        drop((expected, value));
        force_collect();
        assert!(!weak_capture.is_upgradable());
    }
}
