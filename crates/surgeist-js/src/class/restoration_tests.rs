use std::{cell::Cell, rc::Rc};

use super::{Class, ClassBuilder, ClassConstructor};
use crate::{
    Context, ContextAccess, JsObject, JsResult, JsValue, NativeFunction,
    context::intrinsics::StandardConstructor, error::RootingError, object::FunctionObjectBuilder,
};
use surgeist_js_gc::heap::{AccessError, AllocationCollection};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct Leaf(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for Leaf {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn stored_pair(
    context: &Context,
    capture_drops: &Rc<Cell<usize>>,
    prototype_drops: &Rc<Cell<usize>>,
) -> (StandardConstructor, crate::object::JsFunction, JsObject) {
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::Manual)
        .unwrap();
    let captured = JsObject::new_unique(context.runtime(), None, Leaf(capture_drops.clone()))
        .unwrap()
        .upcast();
    let constructor = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_with_captures(
            |_, _, captured, _| Ok(captured.clone().into()),
            captured,
        ),
    )
    .constructor(true)
    .build()
    .unwrap();
    let prototype = JsObject::new_unique(context.runtime(), None, Leaf(prototype_drops.clone()))
        .unwrap()
        .upcast();
    let stored = StandardConstructor::new(constructor.clone(), prototype.clone());
    (stored, constructor, prototype)
}

#[test]
fn expired_class_prototype_releases_the_already_restored_constructor_prefix() {
    let context = context();
    let capture_drops = Rc::new(Cell::new(0));
    let prototype_drops = Rc::new(Cell::new(0));
    let (stored, constructor, prototype) = stored_pair(&context, &capture_drops, &prototype_drops);
    let weak_constructor = constructor.edge();
    let weak_prototype = prototype.edge();
    drop(prototype);
    context.runtime().collect().unwrap();
    assert_eq!(prototype_drops.get(), 1);
    assert!(!weak_prototype.is_live().unwrap());
    assert!(weak_constructor.is_live().unwrap());
    assert_eq!(capture_drops.get(), 0);
    drop(constructor);

    assert_eq!(
        ClassConstructor::from_stored(&stored).unwrap_err(),
        RootingError::Expired
    );
    assert_eq!(capture_drops.get(), 0);
    context.runtime().collect().unwrap();
    assert!(!weak_constructor.is_live().unwrap());
    assert_eq!(
        capture_drops.get(),
        1,
        "failed class projection releases its prefix"
    );
    assert_eq!(prototype_drops.get(), 1);
}

#[test]
fn rooted_class_clone_getters_need_no_heap_upgrade_and_release_their_final_roots() {
    let context = context();
    let capture_drops = Rc::new(Cell::new(0));
    let prototype_drops = Rc::new(Cell::new(0));
    let (stored, constructor, prototype) = stored_pair(&context, &capture_drops, &prototype_drops);
    let constructor_id = constructor.identity();
    let prototype_id = prototype.identity();
    let weak_constructor = constructor.edge();
    let weak_prototype = prototype.edge();
    let rooted = ClassConstructor::from_stored(&stored).unwrap();
    drop(constructor);
    drop(prototype);
    context.runtime().collect().unwrap();
    assert_eq!(capture_drops.get(), 0);
    assert_eq!(prototype_drops.get(), 0);
    context
        .runtime()
        .heap
        .read(|_| {
            assert_eq!(
                context.runtime().heap.allocate(()).unwrap_err().reason,
                AccessError::HeapBorrowed
            );
            assert_eq!(rooted.constructor().identity(), constructor_id);
            assert_eq!(rooted.prototype().identity(), prototype_id);
        })
        .unwrap();
    drop(rooted);
    context.runtime().collect().unwrap();
    assert!(!weak_constructor.is_live().unwrap());
    assert!(!weak_prototype.is_live().unwrap());
    assert_eq!(capture_drops.get(), 1);
    assert_eq!(prototype_drops.get(), 1);
}

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct RegisteredClass;

impl Class for RegisteredClass {
    const NAME: &'static str = "ProjectionRegisteredClass";

    fn init(_: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(_: &JsValue, _: &[JsValue], _: ContextAccess<'_>) -> JsResult<Self> {
        Ok(Self)
    }
}

#[test]
fn class_registry_lookup_and_unregister_refusal_preserve_presence_and_retry_identity() {
    let mut context = context();
    context.register_global_class::<RegisteredClass>().unwrap();
    let realm = context.realm().clone();
    let initial = context
        .get_global_class::<RegisteredClass>()
        .unwrap()
        .unwrap();
    let constructor_id = initial.constructor().identity();
    let prototype_id = initial.prototype().identity();
    drop(initial);
    let heap = context.runtime().heap.clone();
    heap.read(|_| {
        assert!(context.has_global_class::<RegisteredClass>().unwrap());
        assert!(
            context
                .access()
                .has_global_class::<RegisteredClass>()
                .unwrap()
        );
        assert!(realm.has_class::<RegisteredClass>().unwrap());
        assert_eq!(
            heap.allocate(()).unwrap_err().reason,
            AccessError::HeapBorrowed
        );
        assert_eq!(
            context.get_global_class::<RegisteredClass>().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            context
                .access()
                .get_global_class::<RegisteredClass>()
                .unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            realm.get_class::<RegisteredClass>().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            realm.unregister_class::<RegisteredClass>().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
    })
    .unwrap();
    let retried = context
        .get_global_class::<RegisteredClass>()
        .unwrap()
        .unwrap();
    assert_eq!(retried.constructor().identity(), constructor_id);
    assert_eq!(retried.prototype().identity(), prototype_id);
    drop(retried);
    let removed = realm
        .unregister_class::<RegisteredClass>()
        .unwrap()
        .unwrap();
    assert_eq!(removed.constructor().identity(), constructor_id);
    assert_eq!(removed.prototype().identity(), prototype_id);
    assert!(realm.get_class::<RegisteredClass>().unwrap().is_none());
    assert!(!realm.has_class::<RegisteredClass>().unwrap());
    assert!(!context.has_global_class::<RegisteredClass>().unwrap());
    assert!(
        context
            .get_global_class::<RegisteredClass>()
            .unwrap()
            .is_none()
    );
}
