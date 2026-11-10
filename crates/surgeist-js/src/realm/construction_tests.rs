//! Functional refusal and cleanup coverage at the realm construction owner.

use super::*;
use crate::{
    Context, ContextAccess, JsError, JsValue, Runtime, class::ClassBuilder, error::EngineError,
    js_string, object::ObjectEdge,
};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, crate::JsData, HeapTrace)]
struct RegistryClass;

impl Class for RegistryClass {
    const NAME: &'static str = "RealmConstructionRegistryClass";

    fn init(_class: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        Ok(Self)
    }
}

fn registry_context() -> Context {
    let mut context = Context::builder().build().unwrap();
    context.register_global_class::<RegistryClass>().unwrap();
    context
}

fn identities(
    class: &ClassConstructor,
) -> (
    surgeist_js_gc::heap::ObjectId,
    surgeist_js_gc::heap::ObjectId,
) {
    (class.constructor().identity(), class.prototype().identity())
}

fn assert_rooting(error: &JsError, failure: RootingError) {
    assert_eq!(error.as_engine(), Some(&EngineError::Rooting(failure)));
}

#[test]
fn exclusive_registry_loan_refuses_queries_and_removal_without_changing_identity() {
    let context = registry_context();
    let realm = context.realm().clone();
    let registered = realm.get_class::<RegistryClass>().unwrap().unwrap();
    let expected = identities(&registered);
    let failure = RootingError::Access(AccessError::PayloadBorrowed);
    let held = realm.inner.host_classes.borrow_mut();
    let count = held.len();
    assert!(held.contains_key(&TypeId::of::<RegistryClass>()));

    assert_eq!(realm.has_class::<RegistryClass>(), Err(failure));
    assert!(matches!(realm.get_class::<RegistryClass>(), Err(error) if error == failure));
    assert!(matches!(realm.unregister_class::<RegistryClass>(), Err(error) if error == failure));
    assert_eq!(held.len(), count);
    assert!(held.contains_key(&TypeId::of::<RegistryClass>()));
    drop(held);

    assert!(realm.has_class::<RegistryClass>().unwrap());
    assert_eq!(
        identities(&realm.get_class::<RegistryClass>().unwrap().unwrap()),
        expected
    );
    let removed = realm.unregister_class::<RegistryClass>().unwrap().unwrap();
    assert_eq!(identities(&removed), expected);
    assert!(!realm.has_class::<RegistryClass>().unwrap());
    assert!(realm.get_class::<RegistryClass>().unwrap().is_none());
}

#[test]
fn shared_registry_loan_allows_lookup_but_refuses_replacement_and_removal() {
    let mut context = registry_context();
    let realm = context.realm().clone();
    let expected = identities(&realm.get_class::<RegistryClass>().unwrap().unwrap());
    let mut replacement = ClassBuilder::new::<RegistryClass>(context.access());
    RegistryClass::init(&mut replacement).unwrap();
    let replacement = replacement.build().unwrap();
    let replacement_ids = identities(&replacement);
    assert_ne!(replacement_ids, expected);
    let failure = RootingError::Access(AccessError::PayloadBorrowed);
    let held = realm.inner.host_classes.borrow();
    let count = held.len();

    assert!(realm.has_class::<RegistryClass>().unwrap());
    assert_eq!(
        identities(&realm.get_class::<RegistryClass>().unwrap().unwrap()),
        expected
    );
    let error = realm
        .register_class::<RegistryClass>(replacement.clone())
        .unwrap_err();
    assert_rooting(&error, failure);
    assert!(matches!(realm.unregister_class::<RegistryClass>(), Err(error) if error == failure));
    assert_eq!(held.len(), count);
    assert_eq!(
        identities(&realm.get_class::<RegistryClass>().unwrap().unwrap()),
        expected
    );
    drop(held);

    realm.register_class::<RegistryClass>(replacement).unwrap();
    assert_eq!(
        identities(&realm.get_class::<RegistryClass>().unwrap().unwrap()),
        replacement_ids
    );
    assert_eq!(
        identities(&realm.unregister_class::<RegistryClass>().unwrap().unwrap()),
        replacement_ids
    );
    assert!(!realm.has_class::<RegistryClass>().unwrap());
}

#[derive(Default)]
struct CountingHooks {
    objects: Cell<usize>,
    this_values: Cell<usize>,
}

impl HostHooks for CountingHooks {
    fn create_global_object(&self, intrinsics: &Intrinsics) -> JsResult<JsObject> {
        self.objects.set(self.objects.get() + 1);
        JsObject::with_object_proto(intrinsics)
    }

    fn create_global_this(&self, _intrinsics: &Intrinsics) -> JsResult<Option<JsObject>> {
        self.this_values.set(self.this_values.get() + 1);
        Ok(None)
    }
}

#[test]
fn supplied_heap_loan_refuses_bootstrap_before_hooks_and_allows_fresh_retry() {
    let runtime = Runtime::default();
    let hooks = Rc::new(CountingHooks::default());
    let failed = runtime
        .heap
        .read(|_| {
            Context::builder()
                .runtime(runtime.clone())
                .host_hooks(hooks.clone())
                .build()
        })
        .unwrap();
    let error = match failed {
        Err(error) => error,
        Ok(_) => panic!("a supplied runtime read loan must refuse bootstrap"),
    };
    assert_rooting(&error, RootingError::Access(AccessError::HeapBorrowed));
    assert_eq!(hooks.objects.get(), 0);
    assert_eq!(hooks.this_values.get(), 0);

    let context = Context::builder()
        .runtime(runtime.clone())
        .host_hooks(hooks.clone())
        .build()
        .unwrap();
    assert!(context.runtime().same_heap(&runtime));
    assert_eq!(hooks.objects.get(), 1);
    assert_eq!(hooks.this_values.get(), 1);
    assert!(
        context
            .global_object()
            .unwrap()
            .runtime()
            .same_heap(&runtime)
    );
}

#[derive(Default)]
struct EscapingHooks {
    objects: Cell<usize>,
    this_values: Cell<usize>,
    escaped: RefCell<Option<JsObject>>,
    escaped_identity: Cell<Option<surgeist_js_gc::heap::ObjectId>>,
    promise_probes: RefCell<Option<(ObjectEdge, ObjectEdge)>>,
}

impl HostHooks for EscapingHooks {
    fn create_global_object(&self, intrinsics: &Intrinsics) -> JsResult<JsObject> {
        self.objects.set(self.objects.get() + 1);
        let constructor = intrinsics.constructors().promise().constructor()?;
        let prototype = intrinsics.constructors().promise().prototype()?;
        *self.promise_probes.borrow_mut() = Some((constructor.edge(), prototype.edge()));
        let object = JsObject::with_object_proto(intrinsics)?;
        self.escaped_identity.set(Some(object.identity()));
        *self.escaped.borrow_mut() = Some(object.clone());
        Ok(object)
    }

    fn create_global_this(&self, intrinsics: &Intrinsics) -> JsResult<Option<JsObject>> {
        self.this_values.set(self.this_values.get() + 1);
        let runtime = intrinsics.runtime()?;
        runtime
            .heap
            .read(|_| {
                intrinsics
                    .constructors()
                    .promise()
                    .prototype()
                    .map(Some)
                    .map_err(JsError::from)
            })
            .map_err(RootingError::Access)?
    }
}

#[test]
fn later_hook_refusal_keeps_escaped_effects_and_releases_bootstrap_only_roots() {
    let runtime = Runtime::default();
    let hooks = Rc::new(EscapingHooks::default());
    let failed = Context::builder()
        .runtime(runtime.clone())
        .host_hooks(hooks.clone())
        .build();
    let error = match failed {
        Err(error) => error,
        Ok(_) => panic!("the global-this hook must return its real projection refusal"),
    };
    assert_rooting(&error, RootingError::Access(AccessError::HeapBorrowed));
    assert_eq!(hooks.objects.get(), 1);
    assert_eq!(hooks.this_values.get(), 1);
    let escaped = hooks.escaped.borrow().as_ref().unwrap().clone();
    assert_eq!(Some(escaped.identity()), hooks.escaped_identity.get());
    assert!(escaped.runtime().same_heap(&runtime));
    // No live Context or realm initialization is returned. The accepted global
    // shell has not acquired the later default global bindings.
    assert!(
        escaped
            .borrow()
            .unwrap()
            .properties()
            .get(&js_string!("Promise").into())
            .unwrap()
            .is_none()
    );
    let (constructor, prototype) = hooks.promise_probes.borrow().clone().unwrap();
    assert!(constructor.is_live().unwrap());
    assert!(prototype.is_live().unwrap());

    runtime.collect().unwrap();
    assert!(!constructor.is_live().unwrap());
    assert!(!prototype.is_live().unwrap());
    assert_eq!(Some(escaped.identity()), hooks.escaped_identity.get());
    assert!(escaped.prototype().unwrap().is_some());
    assert_eq!(
        hooks.objects.get(),
        1,
        "accepted host effects are not replayed"
    );
    assert_eq!(hooks.this_values.get(), 1);
}
