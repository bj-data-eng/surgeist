use std::panic::{AssertUnwindSafe, catch_unwind};

use surgeist_js::{
    Context, ContextAccess, JsData, JsObject, JsResult, JsValue, Source,
    class::{Class, ClassBuilder},
    execution::ExecutionOptions,
};
use surgeist_js_gc::heap::Trace;

#[derive(Debug, JsData, Trace)]
struct AccessPayload {
    value: u32,
}

impl Class for AccessPayload {
    const NAME: &'static str = "AccessPayload";

    fn init(_: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(_: &JsValue, _: &[JsValue], _: ContextAccess<'_>) -> JsResult<Self> {
        Ok(Self { value: 17 })
    }
}

fn fixture() -> (Context, JsObject<AccessPayload>) {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    let mut context = builder.build().unwrap();
    context.register_global_class::<AccessPayload>().unwrap();
    let object = context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.eval(Source::from_bytes("new AccessPayload()"))
        })
        .unwrap()
        .result
        .unwrap()
        .as_object()
        .unwrap()
        .downcast::<AccessPayload>()
        .unwrap();
    (context, object)
}

#[test]
fn immutable_object_access_under_mutable_alias_refuses_without_rust_unwind() {
    let (_context, object) = fixture();
    let identity = object.identity();
    let alias = object.clone();
    let held = alias.borrow_mut();
    assert_eq!(held.data().value, 17);
    let outcome = catch_unwind(AssertUnwindSafe(|| object.borrow()));
    assert_eq!(held.data().value, 17);
    drop(held);
    assert!(
        outcome.is_ok(),
        "conflicting object read must not unwind Rust"
    );
    drop(outcome);

    assert_eq!(object.identity(), identity);
    let retried = object.borrow();
    assert_eq!(retried.data().value, 17);
}

#[test]
fn mutable_object_access_under_shared_or_exclusive_alias_refuses_without_rust_unwind() {
    for exclusive in [false, true] {
        let (_context, object) = fixture();
        let identity = object.identity();
        let alias = object.clone();
        let read = if exclusive {
            None
        } else {
            Some(alias.borrow())
        };
        let write = if exclusive {
            Some(alias.borrow_mut())
        } else {
            None
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| object.borrow_mut()));
        drop(read);
        drop(write);
        assert!(
            outcome.is_ok(),
            "conflicting object write must not unwind Rust"
        );
        drop(outcome);

        assert_eq!(object.identity(), identity);
        let retried = object.borrow_mut();
        assert_eq!(retried.data().value, 17);
        drop(retried);
        assert_eq!(object.borrow().data().value, 17);
    }
}

#[test]
fn correct_brand_read_under_mutable_alias_refuses_without_rust_unwind() {
    let (_context, object) = fixture();
    let identity = object.identity();
    let untyped = object.clone().upcast();
    assert!(untyped.is::<AccessPayload>());
    let alias = object.clone();
    let held = alias.borrow_mut();
    assert_eq!(held.data().value, 17);
    let outcome = catch_unwind(AssertUnwindSafe(|| untyped.downcast_ref::<AccessPayload>()));
    assert_eq!(held.data().value, 17);
    drop(held);
    assert!(
        outcome.is_ok(),
        "correct-brand read refusal must not unwind Rust"
    );
    drop(outcome);

    assert_eq!(untyped.identity(), identity);
    assert_eq!(object.identity(), identity);
    let retried = untyped.downcast_ref::<AccessPayload>().unwrap();
    assert_eq!(retried.value, 17);
}

#[test]
fn correct_brand_class_write_under_shared_or_exclusive_alias_refuses_without_rust_unwind() {
    for exclusive in [false, true] {
        let (_context, object) = fixture();
        let identity = object.identity();
        let untyped = object.clone().upcast();
        assert!(untyped.is::<AccessPayload>());
        let alias = object.clone();
        let read = if exclusive {
            None
        } else {
            Some(alias.borrow())
        };
        let write = if exclusive {
            Some(alias.borrow_mut())
        } else {
            None
        };
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            untyped.downcast_class_mut::<AccessPayload>()
        }));
        drop(read);
        drop(write);
        assert!(
            outcome.is_ok(),
            "correct-brand class write refusal must not unwind Rust"
        );
        drop(outcome);

        assert_eq!(untyped.identity(), identity);
        {
            let mut retried = untyped.downcast_class_mut::<AccessPayload>().unwrap();
            assert_eq!(retried.value, 17);
            retried.value = 29;
        }
        assert_eq!(object.identity(), identity);
        assert_eq!(object.borrow().data().value, 29);
    }
}

#[test]
fn multiple_immutable_object_and_payload_reads_preserve_a_later_class_update() {
    let (_context, object) = fixture();
    let identity = object.identity();
    let alias = object.clone();
    let untyped = object.clone().upcast();
    let first = object.borrow();
    let second = alias.borrow();
    let payload = untyped.downcast_ref::<AccessPayload>().unwrap();
    assert_eq!(first.data().value, 17);
    assert_eq!(second.data().value, 17);
    assert_eq!(payload.value, 17);
    drop(first);
    drop(second);
    drop(payload);

    {
        let mut payload = untyped.downcast_class_mut::<AccessPayload>().unwrap();
        payload.value = 31;
    }
    assert_eq!(object.identity(), identity);
    assert_eq!(alias.identity(), identity);
    assert_eq!(object.borrow().data().value, 31);
    assert_eq!(alias.borrow().data().value, 31);
}

#[test]
fn unborrowed_wrong_brand_is_absent_without_changing_either_payload_or_identity() {
    let (context, object) = fixture();
    let identity = object.identity();
    let ordinary = JsObject::with_null_proto(context.runtime());
    let ordinary_identity = ordinary.identity();
    assert!(!ordinary.is::<AccessPayload>());
    assert!(ordinary.downcast_ref::<AccessPayload>().is_none());
    assert!(ordinary.downcast_class_mut::<AccessPayload>().is_none());
    assert_eq!(ordinary.identity(), ordinary_identity);
    assert!(ordinary.borrow().prototype().is_none());
    assert_eq!(object.identity(), identity);
    assert_eq!(object.borrow().data().value, 17);
}
