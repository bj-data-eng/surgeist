//! Boa's implementation of ECMAScript's `FinalizationRegistry` object.

use std::{rc::Rc, slice, task::Poll};

use surgeist_js_gc::heap::Trace as HeapTrace;

use crate::{
    Context, JsArgs, JsData, JsObject, JsResult, JsSymbol, JsValue, JsVariant,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    job::{Job, JobCallback, NativeJob, StoredJobCallback},
    js_error, js_string,
    object::{JsFunction, ObjectEdge, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::{Realm, RealmEdge},
    string::StaticJsStrings,
    value::StoredValue,
};

use super::{
    BuiltInConstructor, BuiltInObject, IntrinsicObject, builder::BuiltInBuilder,
    weak::require_same_runtime,
};

#[cfg(test)]
mod tests;

/// A subscription to cleanup requests for a `FinalizationRegistry`.
///
/// A job executor can retain it across idle runs without keeping the registry alive.
/// Each request yields one job that the executor can run in the engine's context.
#[derive(Debug)]
pub struct FinalizationRegistryCleanupJob {
    registry: ObjectEdge,
    receiver: async_channel::Receiver<()>,
}

impl FinalizationRegistryCleanupJob {
    pub(crate) fn is_closed(&self) -> bool {
        self.receiver.is_closed()
    }

    /// Takes the next available cleanup job without waiting.
    ///
    /// `Poll::Pending` means the executor should retain the subscription and check again later.
    /// `Poll::Ready(None)` means the registry's channel is closed.
    pub fn try_take(&self) -> Poll<Option<NativeJob>> {
        match self.receiver.try_recv() {
            Ok(()) => Poll::Ready(Some(self.cleanup_job())),
            Err(async_channel::TryRecvError::Empty) => Poll::Pending,
            Err(async_channel::TryRecvError::Closed) => Poll::Ready(None),
        }
    }

    /// Waits for a cleanup job, or returns `None` once the registry's channel closes.
    ///
    /// This can wait independently of any engine context. Run the returned job separately.
    pub async fn next(&self) -> Option<NativeJob> {
        self.receiver.recv().await.ok().map(|()| self.cleanup_job())
    }

    fn cleanup_job(&self) -> NativeJob {
        let registry = self.registry.clone();
        NativeJob::new(move |context| {
            if let Some(registry) = registry.upgrade().expect("registry heap is available") {
                let registry = registry.downcast().expect("finalization registry");
                FinalizationRegistry::cleanup(&registry, context)?;
            }
            Ok(JsValue::undefined())
        })
    }
}

/// A cell tracked by a [`FinalizationRegistry`].
#[derive(HeapTrace)]
struct RegistryCell {
    #[heap_trace(skip)]
    target: ObjectEdge,
    held_value: StoredValue,
    #[heap_trace(skip)]
    unregister_token: Option<ObjectEdge>,
    // Dropping the cell cancels its registration on the target.
    #[heap_trace(skip)]
    _retirement: Rc<dyn Fn()>,
}

/// Boa's implementation of ECMAScript's [`FinalizationRegistry`] builtin object.
///
/// `FinalizationRegistry` provides a way to request that a cleanup callback get called at some point
/// when a value registered with the registry has been reclaimed (garbage-collected).
///
/// [`FinalizationRegistry`]: https://tc39.es/ecma262/#sec-finalization-registry-objects
#[derive(HeapTrace, JsData)]
pub(crate) struct FinalizationRegistry {
    _realm: RealmEdge,
    callback: StoredJobCallback,
    #[heap_trace(skip)]
    cleanup_notifier: async_channel::Sender<()>,
    cells: Vec<RegistryCell>,
}

impl IntrinsicObject for FinalizationRegistry {
    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }

    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                js_string!("FinalizationRegistry"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::register, js_string!("register"), 2)
            .method(Self::unregister, js_string!("unregister"), 1)
            .build();
    }
}

impl BuiltInObject for FinalizationRegistry {
    const NAME: crate::JsString = StaticJsStrings::FINALIZATION_REGISTRY;

    const ATTRIBUTE: Attribute = Attribute::WRITABLE.union(Attribute::CONFIGURABLE);
}

impl BuiltInConstructor for FinalizationRegistry {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::finalization_registry;

    /// Constructor [`FinalizationRegistry ( cleanupCallback )`][cons]
    ///
    /// [cons]: https://tc39.es/ecma262/#sec-finalization-registry-cleanup-callback
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(js_error!(
                TypeError: "FinalizationRegistry: cannot call constructor without `new`"
            ));
        }

        // 2. If IsCallable(cleanupCallback) is false, throw a TypeError exception.
        let callback = args
            .get_or_undefined(0)
            .as_object()
            .and_then(JsFunction::from_object)
            .ok_or_else(|| {
                js_error!(
                    TypeError: "FinalizationRegistry: \
                        cleanup callback of registry must be callable"
                )
            })?;

        // 3. Let finalizationRegistry be ? OrdinaryCreateFromConstructor(NewTarget,
        //    "%FinalizationRegistry.prototype%", « [[Realm]], [[CleanupCallback]], [[Cells]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::finalization_registry,
            context,
        )?;

        // 4. Let fn be the active function object.
        // 5. Set finalizationRegistry.[[Realm]] to fn.[[Realm]].
        let realm = context.vm.frame().realm.clone();

        // 6. Set finalizationRegistry.[[CleanupCallback]] to HostMakeJobCallback(cleanupCallback).
        let callback = JobCallback::from_host(callback, context)?;

        // 7. Set finalizationRegistry.[[Cells]] to a new empty List.
        let cells = Vec::new();

        let (sender, receiver) = async_channel::bounded(1);

        let registry = JsObject::new_unique(
            context.runtime(),
            prototype,
            FinalizationRegistry {
                _realm: realm.edge(),
                callback: callback.to_stored(),
                cells,
                cleanup_notifier: sender,
            },
        )?;
        drop(callback);

        context.enqueue_job(Job::FinalizationRegistryCleanupJob(
            FinalizationRegistryCleanupJob {
                registry: registry.edge(),
                receiver,
            },
        ));

        // 8. Return finalizationRegistry.
        Ok(registry.upcast().into())
    }
}

impl FinalizationRegistry {
    /// [`FinalizationRegistry.prototype.register ( target, heldValue [ , unregisterToken ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/sec-finalization-registry.prototype.register
    fn register(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. Let finalizationRegistry be the this value.
        // 2. Perform ? RequireInternalSlot(finalizationRegistry, [[Cells]]).
        let this = this
            .as_object()
            .and_then(|object| object.downcast::<Self>().ok())
            .ok_or_else(|| {
                js_error!(
                    TypeError: "FinalizationRegistry.prototype.register: \
                        invalid object type for `this`",
                )
            })?;

        let target = args.get_or_undefined(0);
        let held_value = args.get_or_undefined(1);
        let unregister_token = args.get_or_undefined(2);

        // 3. If CanBeHeldWeakly(target) is false, throw a TypeError exception.
        //
        // [`CanBeHeldWeakly ( v )`](https://tc39.es/ecma262/#sec-canbeheldweakly)
        //
        // 1. If v is an Object, return true.
        // 2. If v is a Symbol and KeyForSymbol(v) is undefined, return true.
        // 3. Return false.
        //
        // TODO: support Symbols
        let Some(target_obj) = target.as_object() else {
            return Err(js_error!(
                TypeError: "FinalizationRegistry.prototype.register: \
                    `target` must be an Object or Symbol",
            ));
        };

        // 4. If SameValue(target, heldValue) is true, throw a TypeError exception.
        if target == held_value {
            return Err(js_error!(
                TypeError: "FinalizationRegistry.prototype.register: \
                    `heldValue` cannot be the same as `target`"
            ));
        }

        // 5. If CanBeHeldWeakly(unregisterToken) is false, then
        //
        // // [`CanBeHeldWeakly ( v )`](https://tc39.es/ecma262/#sec-canbeheldweakly)
        //
        // 1. If v is an Object, return true.
        // 2. If v is a Symbol and KeyForSymbol(v) is undefined, return true.
        // 3. Return false.
        //
        // TODO: support Symbols
        let runtime = this.runtime();
        require_same_runtime(&runtime, &target_obj)?;
        let unregister_token = match unregister_token.variant() {
            JsVariant::Object(obj) => {
                require_same_runtime(&runtime, &obj)?;
                Some(obj.edge())
            }
            // b. Set unregisterToken to empty.
            JsVariant::Undefined => None,
            // a. If unregisterToken is not undefined, throw a TypeError exception.
            _ => {
                return Err(js_error!(
                    TypeError: "FinalizationRegistry.prototype.register: \
                        `unregisterToken` must be an Object, a Symbol, or undefined",
                ));
            }
        };

        if let Some(held_object) = held_value.as_object()
            && !runtime.same_heap(&held_object.runtime())
        {
            return Err(js_error!(
                TypeError: "FinalizationRegistry.prototype.register: \
                    `heldValue` belongs to a different JavaScript runtime",
            ));
        }

        // 6. Let cell be the Record { [[WeakRefTarget]]: target, [[HeldValue]]: heldValue, [[UnregisterToken]]: unregisterToken }.
        let mut registry = this.borrow_data_mut();
        let sender = registry.cleanup_notifier.downgrade();
        let retirement: Rc<dyn Fn()> = Rc::new(move || {
            if let Some(sender) = sender.upgrade() {
                // A full channel already has a cleanup request. A closed one has no listener.
                let _ = sender.try_send(());
            }
        });
        target_obj.observe_retirement(&retirement);
        let cell = RegistryCell {
            target: target_obj.edge(),
            held_value: held_value.clone().into_stored(),
            unregister_token,
            _retirement: retirement,
        };

        // 7. Append cell to finalizationRegistry.[[Cells]].
        registry.cells.push(cell);

        // 8. Return undefined.
        Ok(JsValue::undefined())
    }

    /// [`FinalizationRegistry.prototype.unregister ( unregisterToken )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-finalization-registry.prototype.unregister
    fn unregister(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. Let finalizationRegistry be the this value.
        // 2. Perform ? RequireInternalSlot(finalizationRegistry, [[Cells]]).
        let this = this.as_object();
        let mut registry = this
            .as_ref()
            .and_then(JsObject::downcast_mut::<Self>)
            .ok_or_else(|| {
                js_error!(
                    TypeError: "FinalizationRegistry.prototype.unregister: \
                    invalid object type for `this`",
                )
            })?;

        // 3. If CanBeHeldWeakly(unregisterToken) is false, throw a TypeError exception.\
        //
        // // [`CanBeHeldWeakly ( v )`](https://tc39.es/ecma262/#sec-canbeheldweakly)
        //
        // 1. If v is an Object, return true.
        // 2. If v is a Symbol and KeyForSymbol(v) is undefined, return true.
        // 3. Return false.
        //
        // TODO: support Symbols
        let unregister_token = args.get_or_undefined(0).as_object().ok_or_else(|| {
            js_error!(
                TypeError: "FinalizationRegistry.prototype.unregister: \
                            `unregisterToken` must be an Object or a Symbol.",
            )
        })?;

        // 4. Let removed be false.
        let original_len = registry.cells.len();
        // 5. For each Record { [[WeakRefTarget]], [[HeldValue]], [[UnregisterToken]] } cell of finalizationRegistry.[[Cells]], do
        registry.cells.retain(|cell| {
            cell.unregister_token
                .as_ref()
                .is_none_or(|token| token.identity() != unregister_token.identity())
        });

        // 6. Return removed.
        Ok((registry.cells.len() != original_len).into())
    }

    /// Abstract operation [`CleanupFinalizationRegistry ( finalizationRegistry )`][spec].
    ///
    /// Cleans up all the cells of the finalization registry that are determined to be
    /// unreachable by the garbage collector.
    ///
    /// # Panics
    ///
    /// Panics if `obj` is not a `FinalizationRegistry` object.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-cleanup-finalization-registry
    pub(crate) fn cleanup(
        obj: &JsObject<FinalizationRegistry>,
        context: &mut Context,
    ) -> JsResult<()> {
        obj.check_runtime(context.runtime())?;
        // 2. Let callback be finalizationRegistry.[[CleanupCallback]].
        let stored_callback = obj.borrow().data().callback.clone();
        let callback = stored_callback.to_callback();
        loop {
            let held_value = {
                let mut registry = obj.borrow_mut();
                let cells = &mut registry.data_mut().cells;
                // 3. Choose and remove a cell with an empty [[WeakRefTarget]].
                let Some(index) = cells
                    .iter()
                    .position(|cell| !cell.target.is_live().expect("target heap is available"))
                else {
                    break;
                };
                cells.swap_remove(index).held_value.to_value()
            };

            // The callback may modify the registry or collect again.
            callback.call(&JsValue::undefined(), slice::from_ref(&held_value), context)?;
        }

        // 4. Return unused.
        Ok(())
    }
}
