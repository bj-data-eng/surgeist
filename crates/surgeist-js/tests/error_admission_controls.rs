#![allow(unused_crate_dependencies)]

use surgeist_js::{
    Context, JsError, JsNativeError, JsNativeErrorKind, JsValue, NativeFunction, Source,
    error::{EngineError, PanicError},
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

fn materialized_engine(owner: &mut Context, error: JsError) -> EngineError {
    let refusal = error.into_opaque(owner.access()).unwrap_err();
    refusal
        .as_engine()
        .expect("materialization must return the selected typed engine error")
        .clone()
}

fn foreign_opaque(foreign: &Context) -> JsError {
    JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap()
}

fn foreign_native(foreign: &mut Context) -> JsNativeError {
    let value = foreign
        .eval(Source::from_bytes("new Error('foreign native realm')"))
        .unwrap();
    JsError::from_opaque(value)
        .unwrap()
        .try_native(foreign.access())
        .unwrap()
}

#[test]
fn engine_in_native_cause_precedes_engine_aggregate_members_during_admission() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let error: JsError = JsNativeError::aggregate(vec![
        foreign_opaque(&foreign),
        PanicError::new("aggregate member must lose to cause").into(),
    ])
    .with_cause(JsNativeError::aggregate(vec![
        EngineError::ContextAccessBusy.into(),
    ]))
    .into();
    assert_eq!(
        materialized_engine(&mut owner, error.clone()),
        EngineError::ContextAccessBusy
    );
    let admitted = return_staged_error(&mut owner, error);
    assert_eq!(
        materialized_engine(&mut owner, admitted),
        EngineError::ContextAccessBusy
    );
}

#[test]
fn first_engine_aggregate_member_wins_in_authored_order_after_foreign_member() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let first = EngineError::from(PanicError::new("first authored engine member"));
    let error: JsError = JsNativeError::aggregate(vec![
        foreign_opaque(&foreign),
        first.clone().into(),
        EngineError::ContextAccessBusy.into(),
        PanicError::new("later authored engine member").into(),
    ])
    .into();
    assert_eq!(materialized_engine(&mut owner, error.clone()), first);
    let admitted = return_staged_error(&mut owner, error);
    assert_eq!(materialized_engine(&mut owner, admitted), first);
}

#[test]
fn selected_panic_sanitizes_foreign_diagnostic_despite_foreign_sibling() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let selected =
        PanicError::new("selected panic retains identity").with_source(foreign_opaque(&foreign));
    let error: JsError = JsNativeError::aggregate(vec![
        foreign_opaque(&foreign),
        selected.clone().into(),
        EngineError::ContextAccessBusy.into(),
    ])
    .into();
    assert_eq!(
        materialized_engine(&mut owner, error.clone()),
        EngineError::from(selected)
    );
    let admitted = return_staged_error(&mut owner, error);
    let expected = PanicError::new("selected panic retains identity")
        .with_source(JsNativeError::typ().with_message("object belongs to a different runtime"));
    assert_eq!(
        materialized_engine(&mut owner, admitted),
        EngineError::from(expected)
    );
}

#[test]
fn diagnostic_engine_cannot_replace_enclosing_panic_when_sibling_is_foreign() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let selected = EngineError::from(
        PanicError::new("outer panic wins over its diagnostic")
            .with_source(EngineError::ContextAccessBusy),
    );
    let error: JsError = JsNativeError::aggregate(vec![
        foreign_opaque(&foreign),
        selected.clone().into(),
        EngineError::InvalidEntryPermit.into(),
    ])
    .into();
    assert_eq!(materialized_engine(&mut owner, error.clone()), selected);
    let admitted = return_staged_error(&mut owner, error);
    assert_eq!(materialized_engine(&mut owner, admitted), selected);
}

#[test]
fn direct_panic_keeps_its_identity_while_foreign_diagnostic_becomes_type_error() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let selected =
        PanicError::new("direct panic diagnostic control").with_source(foreign_opaque(&foreign));
    let admitted = return_staged_error(&mut owner, selected.into());
    let expected = EngineError::from(
        PanicError::new("direct panic diagnostic control").with_source(
            JsNativeError::typ().with_message("object belongs to a different runtime"),
        ),
    );
    assert_eq!(admitted.as_engine(), Some(&expected));
}

#[test]
fn direct_panic_is_not_replaced_by_its_engine_diagnostic() {
    let mut owner = Context::builder().build().unwrap();
    let selected = EngineError::from(
        PanicError::new("direct panic with engine diagnostic")
            .with_source(EngineError::InvalidEntryPermit),
    );
    let admitted = return_staged_error(&mut owner, selected.clone().into());
    assert_eq!(admitted.as_engine(), Some(&selected));
}

#[test]
fn engine_beneath_foreign_native_realm_is_found_before_later_engine_member() {
    let mut owner = Context::builder().build().unwrap();
    let mut foreign = Context::builder().build().unwrap();
    let foreign_parent = foreign_native(&mut foreign).with_cause(JsNativeError::aggregate(vec![
        EngineError::ContextAccessBusy.into(),
    ]));
    let error: JsError = JsNativeError::aggregate(vec![
        foreign_parent.into(),
        PanicError::new("later engine must not win over foreign parent child").into(),
    ])
    .into();
    assert_eq!(
        materialized_engine(&mut owner, error.clone()),
        EngineError::ContextAccessBusy
    );
    let admitted = return_staged_error(&mut owner, error);
    assert_eq!(
        materialized_engine(&mut owner, admitted),
        EngineError::ContextAccessBusy
    );
}

#[test]
fn local_mixed_native_tree_is_preserved_with_cause_first_engine_selection() {
    let mut owner = Context::builder().build().unwrap();
    let selected = EngineError::from(
        PanicError::new("local cause panic")
            .with_source(JsNativeError::error().with_message("local diagnostic")),
    );
    let error: JsError = JsNativeError::aggregate(vec![
        EngineError::ContextAccessBusy.into(),
        JsNativeError::range().with_message("local member").into(),
    ])
    .with_cause(JsNativeError::aggregate(vec![selected.clone().into()]))
    .into();
    let admitted = return_staged_error(&mut owner, error.clone());
    assert_eq!(admitted, error);
    assert_eq!(materialized_engine(&mut owner, admitted), selected);
}

#[test]
fn foreign_native_realm_without_engine_remains_a_catchable_type_error() {
    let mut owner = Context::builder().build().unwrap();
    let mut foreign = Context::builder().build().unwrap();
    let error: JsError = foreign_native(&mut foreign).into();
    let conversion_refusal = error.clone().into_opaque(owner.access()).unwrap_err();
    let admitted = return_staged_error(&mut owner, error);
    for refusal in [conversion_refusal, admitted] {
        let native = refusal.as_native().unwrap();
        assert_eq!(native.kind(), &JsNativeErrorKind::Type);
        assert_eq!(
            native.message(),
            "error realm belongs to a different runtime"
        );
        assert!(refusal.into_opaque(owner.access()).is_ok());
    }
}
