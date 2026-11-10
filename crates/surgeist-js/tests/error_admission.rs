#![allow(unused_crate_dependencies)]

use surgeist_js::{
    Context, JsError, JsNativeError, JsNativeErrorKind, JsValue, NativeFunction, error::EngineError,
};

fn return_staged_error(owner: &mut Context, error: JsError) -> JsError {
    assert!(owner.insert_data(error).is_none());
    let native = NativeFunction::from_fn_ptr(|_, _, mut context| {
        Err(*context.remove_data::<JsError>().expect("staged host error"))
    });
    native
        .call(&JsValue::undefined(), &[], owner.access())
        .expect_err("the native callback returned an error")
}

fn contains_busy(error: &JsError) -> bool {
    if error.as_engine() == Some(&EngineError::ContextAccessBusy) {
        return true;
    }
    let Some(native) = error.as_native() else {
        return false;
    };
    native.cause().is_some_and(contains_busy)
        || matches!(native.kind(), JsNativeErrorKind::Aggregate(errors) if errors.iter().any(contains_busy))
}

fn assert_engine_survives_admission(owner: &mut Context, error: JsError) {
    // Public materialization refuses engine-containing errors even when an
    // ordinary foreign object elsewhere in the tree would also be refused.
    let refusal = error.clone().into_opaque(owner.access()).unwrap_err();
    assert_eq!(refusal.as_engine(), Some(&EngineError::ContextAccessBusy));

    let admitted = return_staged_error(owner, error);
    assert!(
        contains_busy(&admitted),
        "callback admission lost the typed engine error: {admitted:?}"
    );
    let refusal = admitted.into_opaque(owner.access()).unwrap_err();
    assert_eq!(refusal.as_engine(), Some(&EngineError::ContextAccessBusy));
}

#[test]
fn foreign_aggregate_member_before_engine_member_cannot_make_it_catchable() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let error = JsNativeError::aggregate(vec![
        JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap(),
        EngineError::ContextAccessBusy.into(),
    ]);
    assert_engine_survives_admission(&mut owner, error.into());
}

#[test]
fn foreign_aggregate_member_after_engine_member_cannot_make_it_catchable() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let error = JsNativeError::aggregate(vec![
        EngineError::ContextAccessBusy.into(),
        JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap(),
    ]);
    assert_engine_survives_admission(&mut owner, error.into());
}

#[test]
fn foreign_cause_cannot_hide_an_engine_aggregate_member() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let error = JsNativeError::aggregate(vec![EngineError::ContextAccessBusy.into()])
        .with_cause(JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap());
    assert_engine_survives_admission(&mut owner, error.into());
}

#[test]
fn engine_nested_in_a_native_cause_survives_a_foreign_aggregate_member() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let cause = JsNativeError::aggregate(vec![EngineError::ContextAccessBusy.into()]);
    let error = JsNativeError::aggregate(vec![
        JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap(),
    ])
    .with_cause(cause);
    assert_engine_survives_admission(&mut owner, error.into());
}

#[test]
fn local_nested_engine_error_preserves_its_tree_and_refuses_materialization() {
    let mut owner = Context::builder().build().unwrap();
    let error: JsError = JsNativeError::error()
        .with_cause(JsNativeError::aggregate(vec![
            JsNativeError::error()
                .with_message("ordinary local member")
                .into(),
            EngineError::ContextAccessBusy.into(),
        ]))
        .into();
    assert_eq!(return_staged_error(&mut owner, error.clone()), error);
    assert_engine_survives_admission(&mut owner, error);
}

#[test]
fn foreign_only_error_remains_a_catchable_type_error() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let error: JsError = JsNativeError::aggregate(vec![
        JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap(),
    ])
    .into();
    let refusal = error.clone().into_opaque(owner.access()).unwrap_err();
    assert_eq!(
        refusal.as_native().unwrap().kind(),
        &JsNativeErrorKind::Type
    );

    let admitted = return_staged_error(&mut owner, error);
    assert_eq!(
        admitted.as_native().unwrap().kind(),
        &JsNativeErrorKind::Type
    );
    assert!(admitted.into_opaque(owner.access()).is_ok());
}
