use std::cell::{Cell, OnceCell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::ThreadId;

use super::{
    CoercedString, DomExceptionProfile, ObjectPrototype, Operations, PropertyDefinition,
    PropertyDescriptor, PropertyKey,
};
use crate::callback_object::CallbackObjectIdentity;
use crate::conversion::ConversionAccount;
use crate::owner::OwnerState;
use crate::{
    Arguments, Callback, Completion, ConstructorArguments, ConstructorCallback,
    ConstructorCompletion, Error, Exception, ExceptionPhase, FunctionMetadata, Intrinsic,
    Primitive, PrototypeResolver, Realm, RealmIdentity, RealmKey, Root, RootIdentity, Scope, Value,
    ValueKind, WellKnownSymbol,
};

mod callback_object;
mod native_object;
#[cfg(test)]
mod tests;

static OWNER_THREAD: OnceLock<ThreadId> = OnceLock::new();
static ACTIVE: AtomicBool = AtomicBool::new(false);
static NEXT_OWNER: AtomicU64 = AtomicU64::new(0);

const ERROR_CONSTRUCTOR_FIELD: usize = 6;
const DOM_EXCEPTION_PROFILE_FIELD: usize = 7;

struct ActiveOwner;

impl Drop for ActiveOwner {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::Release);
    }
}

struct RootEntry {
    identity: Weak<RootIdentity>,
    value: v8::Global<v8::Value>,
    backing: OnceCell<Box<dyn std::any::Any>>,
}

enum NativeCallback {
    Callable(Callback, Option<i32>),
    Constructor(ConstructorCallback),
}

struct CallbackLease {
    callback: NativeCallback,
    realm: RealmKey,
}

#[derive(Clone, Copy)]
enum CallbackRole {
    Callable = 0,
    ConstructorTarget = 1,
    ConstructorTrap = 2,
}

struct CallbackEntry {
    // Release the ORIGINAL weak handle before dropping arbitrary captures.
    // Cloning an SDK Weak would lose its finalizer.
    _function: v8::Weak<v8::Function>,
    retired: Rc<Cell<bool>>,
    // The SDK finalizer previously held this same strong lease. Registry
    // custody adds no guest root: the function handle above remains weak.
    lease: Rc<CallbackLease>,
}

struct Roots {
    owner: Rc<OwnerState>,
    next: Cell<u64>,
    entries: RefCell<BTreeMap<u64, Rc<RootEntry>>>,
    callback_objects: callback_object::CallbackObjects,
    native_objects: native_object::NativeObjects,
    next_callback: Cell<u32>,
    callbacks: RefCell<BTreeMap<u32, CallbackEntry>>,
    interrupt: v8::IsolateHandle,
    dom_exception_key: v8::Global<v8::Private>,
    cleanup_interrupt: Cell<bool>,
    // User panic payloads remain owned until teardown outside native callback
    // frames. Their arbitrary destructors follow ordinary Rust drop behavior.
    panic_payloads: Cell<Vec<Box<dyn std::any::Any + Send>>>,
}

impl Roots {
    fn retain(
        &self,
        scope: &v8::PinScope,
        realm: RealmKey,
        value: v8::Local<v8::Value>,
    ) -> Result<Root, Error> {
        self.available()?;
        if realm.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let index = self
            .next
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        // Reserve identity before entering the SDK. No table borrow crosses
        // allocation, guest invocation or a potentially reentrant conversion.
        self.next.set(index);
        let root = Root(Rc::new(RootIdentity { realm, index }));
        let entry = RootEntry {
            identity: Rc::downgrade(&root.0),
            value: v8::Global::new(scope, value),
            backing: OnceCell::new(),
        };
        self.entries
            .try_borrow_mut()
            .map_err(|_| self.conflict())?
            .insert(index, Rc::new(entry));
        Ok(root)
    }

    fn local<'s>(
        &self,
        scope: &v8::PinScope<'s, '_>,
        root: &Root,
    ) -> Result<v8::Local<'s, v8::Value>, Error> {
        let entry = self.entry(root)?;
        Ok(v8::Local::new(scope, &entry.value))
    }

    fn entry(&self, root: &Root) -> Result<Rc<RootEntry>, Error> {
        self.available()?;
        if root.0.realm.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let entry = self
            .entries
            .try_borrow()
            .map_err(|_| self.conflict())?
            .get(&root.0.index)
            .cloned()
            .ok_or(Error::ExpiredIdentity)?;
        if !entry.identity.ptr_eq(&Rc::downgrade(&root.0)) {
            return Err(Error::ExpiredIdentity);
        }
        // Only an Rc clone crossed the table borrow. Opening the SDK handle
        // happens after that borrow ends, including in nested callback scopes.
        Ok(entry)
    }

    fn available(&self) -> Result<(), Error> {
        let result = self.owner.available();
        if result.is_err() {
            // Cleanup can latch the shared failure without using this SDK
            // adapter. Interrupt before returning through a native callback
            // or accepting another engine operation, even in that case.
            if matches!(result, Err(Error::ConversionCleanup(_)))
                && !self.interrupt.is_execution_terminating()
            {
                self.cleanup_interrupt.set(true);
            }
            self.interrupt.terminate_execution();
        }
        result
    }

    fn fail(&self, error: Error) -> Error {
        let first = self.owner.fail(error);
        // V8 terminates at interrupt checks. Native continuations also inspect
        // this sticky first failure before doing work after guest reentry.
        self.interrupt.terminate_execution();
        first
    }

    fn conflict(&self) -> Error {
        self.fail(Error::RegistryConflict)
    }

    fn reap(&self) {
        let retired = match self.entries.try_borrow_mut() {
            Ok(mut entries) => {
                let dead: Vec<_> = entries
                    .iter()
                    .filter_map(|(id, entry)| (entry.identity.strong_count() == 0).then_some(*id))
                    .collect();
                dead.into_iter()
                    .filter_map(|id| entries.remove(&id))
                    .collect::<Vec<_>>()
            }
            Err(_) => {
                self.conflict();
                return;
            }
        };
        drop(retired);
        self.callback_objects.reap(self);
        self.native_objects.reap(self);
        let retired = match self.callbacks.try_borrow_mut() {
            Ok(mut entries) => {
                let dead: Vec<_> = entries
                    .iter()
                    .filter_map(|(id, entry)| entry.retired.get().then_some(*id))
                    .collect();
                dead.into_iter()
                    .filter_map(|id| entries.remove(&id))
                    .collect::<Vec<_>>()
            }
            Err(_) => {
                self.conflict();
                return;
            }
        };
        // Engine lifecycle entry points call reap only after SDK/native frames
        // have returned. Extraction also ended the table borrow above. Capture
        // Drop therefore follows ordinary Rust unwind behavior here, without
        // converting destructor panics into CallbackPanicked or changing the
        // owner's preexisting first refusal.
        drop(retired);
    }
}

struct ContextEntry {
    identity: Weak<RealmIdentity>,
    context: v8::Global<v8::Context>,
    intrinsics: v8::Global<v8::Object>,
}

pub(crate) struct Engine {
    next_realm: u64,
    roots: Rc<Roots>,
    contexts: BTreeMap<u64, ContextEntry>,
    isolate: v8::OwnedIsolate,
    // Field order is deliberate: release the singleton only AFTER the entered
    // isolate has exited and been disposed. The maps are cleared by Drop first.
    _active: ActiveOwner,
}

impl Engine {
    pub(crate) fn bootstrap() -> Result<Self, Error> {
        let current = std::thread::current().id();
        let owner = OWNER_THREAD.get_or_init(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            current
        });
        if *owner != current {
            return Err(Error::WrongThread);
        }
        ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::OwnerAlreadyActive)?;
        let active = ActiveOwner;
        let owner = NEXT_OWNER
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::IdentityExhausted)?;
        let mut isolate = v8::Isolate::new(Default::default());
        isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
        let dom_exception_key = {
            v8::scope!(let scope, &mut isolate);
            let key = v8::Private::new(scope, None);
            v8::Global::new(scope, key)
        };
        let roots = Rc::new(Roots {
            owner: Rc::new(OwnerState::new(owner)),
            next: Cell::new(0),
            entries: RefCell::new(BTreeMap::new()),
            callback_objects: callback_object::CallbackObjects::default(),
            native_objects: native_object::NativeObjects::default(),
            next_callback: Cell::new(0),
            callbacks: RefCell::new(BTreeMap::new()),
            interrupt: isolate.thread_safe_handle(),
            dom_exception_key,
            cleanup_interrupt: Cell::new(false),
            panic_payloads: Cell::new(Vec::new()),
        });
        isolate.set_slot(Rc::downgrade(&roots));
        Ok(Self {
            next_realm: 0,
            roots,
            contexts: BTreeMap::new(),
            isolate,
            _active: active,
        })
    }

    fn with_context<T>(
        &mut self,
        realm: &Realm,
        operation: impl FnOnce(&mut Entered<'_, '_, '_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.validate_realm(realm)?;
        let context = self
            .contexts
            .get(&realm.0.key.index)
            .ok_or(Error::ExpiredIdentity)?;
        let pinned = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut pinned.init();
        let local = v8::Local::new(scope, &context.context);
        let intrinsics = v8::Local::new(scope, &context.intrinsics);
        let scope = &mut v8::ContextScope::new(scope, local);
        let mut entered = Entered {
            scope,
            roots: self.roots.clone(),
            realm: realm.0.key,
            intrinsics,
        };
        let result = operation(&mut entered);
        // Preserve the chosen Rust result until the actual host boundary has
        // also reaped abandoned captures and observed their cleanup.
        match entered.check_active() {
            Err(Error::ConversionCleanup(_)) => self.roots.owner.operation_result(result),
            Err(error) => Err(error),
            Ok(()) => self.roots.owner.operation_result(result),
        }
    }

    fn validate_realm(&self, realm: &Realm) -> Result<(), Error> {
        self.roots.available()?;
        if realm.0.key.owner != self.roots.owner.id() {
            return Err(Error::ForeignOwner);
        }
        if self.isolate.is_execution_terminating() {
            return Err(Error::Terminated);
        }
        let context = self
            .contexts
            .get(&realm.0.key.index)
            .ok_or(Error::ExpiredIdentity)?;
        if context.identity.ptr_eq(&Rc::downgrade(&realm.0)) {
            Ok(())
        } else {
            Err(Error::ExpiredIdentity)
        }
    }
}

impl super::Engine for Engine {
    fn owner_state(&self) -> Rc<OwnerState> {
        self.roots.owner.clone()
    }
    fn create_realm(&mut self) -> Result<Realm, Error> {
        self.roots.available()?;
        if self.isolate.is_execution_terminating() {
            return Err(Error::Terminated);
        }
        self.reap();
        self.roots.available()?;
        let index = self
            .next_realm
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let realm = Realm(Rc::new(RealmIdentity {
            key: RealmKey {
                owner: self.roots.owner.id(),
                index,
            },
        }));
        let pinned = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut pinned.init();
        let context = v8::Context::new(scope, Default::default());
        let intrinsics = {
            let scope = &mut v8::ContextScope::new(scope, context);
            let reflect_key = v8::String::new(scope, "Reflect").ok_or(Error::BackendUnavailable)?;
            let reflect = context
                .global(scope)
                .get(scope, reflect_key.into())
                .ok_or(Error::BackendUnavailable)?;
            let reflect = v8::Local::<v8::Object>::try_from(reflect)
                .map_err(|_| Error::BackendUnavailable)?;
            let setter = pristine_method(scope, reflect, "set")?;
            // Object::set_prototype swallows proxy exceptions in this SDK.
            // The pristine Reflect method preserves both false and thrown values.
            let set_prototype = pristine_method(scope, reflect, "setPrototypeOf")?;
            let array = v8::Array::new(scope, 0);
            let prototype = array
                .get_prototype(scope)
                .ok_or(Error::BackendUnavailable)?;
            let prototype = v8::Local::<v8::Object>::try_from(prototype)
                .map_err(|_| Error::BackendUnavailable)?;
            let array_values = pristine_method(scope, prototype, "values")?;
            let global = context.global(scope);
            let object_constructor = pristine_method(scope, global, "Object")?;
            let function_constructor = pristine_method(scope, global, "Function")?;
            let error_constructor = pristine_method(scope, global, "Error")?;
            let prototype_name =
                v8::String::new(scope, "prototype").ok_or(Error::BackendUnavailable)?;
            let function_prototype = function_constructor
                .get(scope, prototype_name.into())
                .ok_or(Error::BackendUnavailable)?;
            let function_prototype = v8::Local::<v8::Object>::try_from(function_prototype)
                .map_err(|_| Error::BackendUnavailable)?;
            let bind = pristine_method(scope, function_prototype, "bind")?;
            let construct = pristine_method(scope, reflect, "construct")?;
            let template = v8::ObjectTemplate::new(scope);
            if !template.set_internal_field_count(DOM_EXCEPTION_PROFILE_FIELD + 1) {
                return Err(Error::BackendUnavailable);
            }
            let intrinsics = template
                .new_instance(scope)
                .ok_or(Error::BackendUnavailable)?;
            if !intrinsics.set_internal_field(0, setter.into())
                || !intrinsics.set_internal_field(1, array_values.into())
                || !intrinsics.set_internal_field(2, set_prototype.into())
                || !intrinsics.set_internal_field(3, bind.into())
                || !intrinsics.set_internal_field(4, construct.into())
                || !intrinsics.set_internal_field(5, object_constructor.into())
                || !intrinsics.set_internal_field(ERROR_CONSTRUCTOR_FIELD, error_constructor.into())
            {
                return Err(Error::BackendUnavailable);
            }
            v8::Global::new(scope, intrinsics)
        };
        // Context-owned JS edges preserve the intrinsic bundle after the last
        // external Realm lease disappears. The realm identity slot owns only a plain key.
        let local_intrinsics = v8::Local::new(scope, &intrinsics);
        context.set_embedder_data(0, local_intrinsics.into());
        if context.set_slot(Rc::new(realm.0.key)).is_some() {
            return Err(Error::BackendUnavailable);
        }
        self.contexts.insert(
            index,
            ContextEntry {
                identity: Rc::downgrade(&realm.0),
                context: v8::Global::new(scope, context),
                intrinsics,
            },
        );
        self.next_realm = index;
        Ok(realm)
    }

    fn with_realm(
        &mut self,
        realm: &Realm,
        action: &mut dyn FnMut(&mut dyn Operations) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.with_context(realm, |entered| action(entered))
    }

    fn checkpoint(&mut self) -> Result<(), Error> {
        self.roots.available()?;
        if self.isolate.is_execution_terminating() {
            return Err(Error::Terminated);
        }
        self.isolate.perform_microtask_checkpoint();
        self.reap();
        self.roots.available()?;
        if self.isolate.is_execution_terminating() {
            Err(Error::Terminated)
        } else {
            Ok(())
        }
    }

    fn memory_pressure(&mut self) {
        if self.roots.available().is_ok() {
            self.isolate.low_memory_notification();
        }
        self.reap();
    }

    fn reap(&mut self) {
        self.roots.reap();
        self.contexts
            .retain(|_, entry| entry.identity.strong_count() != 0);
    }

    fn realm_storage_bytes(&self) -> usize {
        std::mem::size_of::<RealmIdentity>()
            + std::mem::size_of::<(u64, ContextEntry)>()
            + std::mem::size_of::<RealmKey>()
    }

    fn managed_object_storage_bytes(&self) -> usize {
        std::mem::size_of::<RootIdentity>()
            + std::mem::size_of::<RootEntry>()
            + std::mem::size_of::<(u64, Rc<RootEntry>)>()
    }

    fn release_managed_object(&mut self, holder: Root) -> Result<(), Error> {
        if holder.0.realm.owner != self.roots.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let entry = {
            let mut entries = self
                .roots
                .entries
                .try_borrow_mut()
                .map_err(|_| self.roots.conflict())?;
            entries.remove(&holder.0.index)
        };
        let entry = entry.ok_or_else(|| self.roots.conflict())?;
        if !entry.identity.ptr_eq(&Rc::downgrade(&holder.0)) {
            return Err(self.roots.conflict());
        }
        drop(entry);
        Ok(())
    }

    fn release_callback_object(
        &mut self,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.roots.callback_objects.release(&self.roots, holder)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // No checkpoint during teardown: abandoned guest jobs must not run.
        self.roots.owner.close();
        self.isolate.remove_slot::<Weak<Roots>>();
        // The registry is the first owned resource field to drop. Callback
        // scopes keep only temporary Rc clones; slots keep Weak references.
        // Its persistent handles therefore drop before the isolate, after all
        // callback frames have returned to this exclusive owner boundary.
        self.contexts.clear();
    }
}

struct Entered<'entry, 's, 'isolate> {
    scope: &'entry mut v8::PinScope<'s, 'isolate>,
    roots: Rc<Roots>,
    realm: RealmKey,
    intrinsics: v8::Local<'s, v8::Object>,
}

impl Entered<'_, '_, '_> {
    fn attempt<T>(
        &mut self,
        phase: ExceptionPhase,
        operation: impl for<'s> FnOnce(
            &mut v8::PinScope<'s, '_>,
            &Roots,
            RealmKey,
            v8::Local<'s, v8::Object>,
        ) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.check_active()?;
        let caught = std::pin::pin!(v8::TryCatch::new(self.scope));
        let caught = &mut caught.init();
        let intrinsics = v8::Local::new(caught, self.intrinsics);
        let result = operation(caught, &self.roots, self.realm, intrinsics);
        if let Err(error) = self.roots.available() {
            return match error {
                // A Rust scoped handler can already own its exact exception
                // when a later local Drop records cleanup. Refuse continuation
                // without replacing that selected operation result.
                Error::ConversionCleanup(_) => self.roots.owner.operation_result(result),
                other => Err(other),
            };
        }
        if caught.has_terminated() || !caught.can_continue() {
            return Err(Error::Terminated);
        }
        if let Some(thrown) = caught.exception() {
            let value = self.roots.retain(caught, self.realm, thrown)?;
            return Err(Error::Exception(Exception {
                phase,
                value,
                rule: None,
            }));
        }
        result
    }
    fn native_function(
        &mut self,
        callback: NativeCallback,
        metadata: Option<FunctionMetadata<'_>>,
    ) -> Result<Root, Error> {
        self.check_active()?;
        self.attempt(ExceptionPhase::Coerce, |scope, roots, realm, intrinsics| {
            let name = metadata
                .map(|metadata| string(scope, metadata.name))
                .transpose()?;
            let id = roots
                .next_callback
                .get()
                .checked_add(1)
                .ok_or(Error::IdentityExhausted)?;
            roots.next_callback.set(id);
            let is_constructor = matches!(callback, NativeCallback::Constructor(_));
            let lease = Rc::new(CallbackLease { callback, realm });
            let context = scope.get_current_context();
            let role = if is_constructor {
                CallbackRole::ConstructorTarget
            } else {
                CallbackRole::Callable
            };
            let data = callback_data(scope, context, intrinsics, id, role)?;
            let function = v8::Function::builder(callback_trampoline)
                .data(data.into())
                // Native template metadata narrows internally. Public metadata
                // is installed independently as an exact own data property.
                .length(0)
                .constructor_behavior(if is_constructor {
                    v8::ConstructorBehavior::Allow
                } else {
                    v8::ConstructorBehavior::Throw
                })
                .build(scope)
                .ok_or(Error::BackendUnavailable)?;
            if let Some(name) = name {
                function.set_name(name);
            }
            if let Some(metadata) = metadata {
                let key = v8::String::new(scope, "length").ok_or(Error::BackendUnavailable)?;
                // Every u32 integer is exactly representable by binary64.
                let length = v8::Number::new(scope, f64::from(metadata.length));
                let mut descriptor =
                    v8::PropertyDescriptor::new_from_value_writable(length.into(), false);
                descriptor.set_enumerable(false);
                descriptor.set_configurable(true);
                if function.define_property(scope, key.into(), &descriptor) != Some(true) {
                    return Err(Error::BackendUnavailable);
                }
            }
            let (value, lifetime_function) = if is_constructor {
                let data = callback_data(
                    scope,
                    context,
                    intrinsics,
                    id,
                    CallbackRole::ConstructorTrap,
                )?;
                let trap = v8::Function::builder(callback_trampoline)
                    .data(data.into())
                    .constructor_behavior(v8::ConstructorBehavior::Throw)
                    .build(scope)
                    .ok_or(Error::BackendUnavailable)?;
                let handler = private_handler(scope, "construct", trap)?;
                let proxy = v8::Proxy::new(scope, function.into(), handler)
                    .ok_or(Error::BackendUnavailable)?;
                // Never expose the private target through its default prototype.
                let name = v8::String::new(scope, "prototype").ok_or(Error::BackendUnavailable)?;
                let prototype = function
                    .get(scope, name.into())
                    .ok_or(Error::BackendUnavailable)?;
                let prototype = v8::Local::<v8::Object>::try_from(prototype)
                    .map_err(|_| Error::BackendUnavailable)?;
                let key = v8::String::new(scope, "constructor").ok_or(Error::BackendUnavailable)?;
                let mut descriptor =
                    v8::PropertyDescriptor::new_from_value_writable(proxy.into(), true);
                descriptor.set_enumerable(false);
                descriptor.set_configurable(true);
                if prototype.define_property(scope, key.into(), &descriptor) != Some(true) {
                    return Err(Error::BackendUnavailable);
                }
                (v8::Local::<v8::Value>::from(proxy), trap)
            } else {
                (function.into(), function)
            };
            let root = roots.retain(scope, realm, value)?;
            let retired = Rc::new(Cell::new(false));
            let retirement_signal = retired.clone();
            let weak = v8::Weak::with_guaranteed_finalizer(
                scope,
                lifetime_function,
                // SDK second-pass callbacks cross an extern-C boundary. This
                // callback only marks retirement: no allocation, registry
                // access, fallible work or arbitrary capture destructor.
                Box::new(move || retirement_signal.set(true)),
            );
            roots
                .callbacks
                .try_borrow_mut()
                .map_err(|_| roots.conflict())?
                .insert(
                    id,
                    CallbackEntry {
                        _function: weak,
                        retired,
                        lease,
                    },
                );
            Ok(root)
        })
    }
}

impl Operations for Entered<'_, '_, '_> {
    fn realm_key(&self) -> RealmKey {
        self.realm
    }

    fn owner_state(&self) -> Rc<OwnerState> {
        self.roots.owner.clone()
    }

    fn refuse(&self, error: Error) -> Error {
        self.roots.fail(error)
    }
    fn check_host_failure_report(&self) -> Result<(), Error> {
        self.roots.owner.check_host_failure_report()?;
        // An interrupt requested for pending cleanup stops guest continuation.
        // It does not choose the cause of the still-active result boundary.
        if self.scope.is_execution_terminating() && !self.roots.cleanup_interrupt.get() {
            return Err(Error::Terminated);
        }
        Ok(())
    }
    fn check_active(&self) -> Result<(), Error> {
        self.roots.available()?;
        if self.scope.is_execution_terminating() {
            Err(Error::Terminated)
        } else {
            Ok(())
        }
    }

    fn validate_root(&self, root: &Root) -> Result<(), Error> {
        self.check_active()?;
        self.roots.entry(root).map(|_| ())
    }

    fn type_error(&mut self, message: &[u16]) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let message = string(scope, message)?;
                let exception = v8::Exception::type_error(scope, message);
                roots.retain(scope, realm, exception)
            },
        )
    }

    fn function(
        &mut self,
        callback: &Callback,
        declared: Option<super::DeclaredCallable<'_>>,
    ) -> Result<Root, Error> {
        // V8 reports the supplied count as i32. A larger prefix still
        // captures every supplied argument while metadata remains exact.
        let prefix =
            declared.map(|shape| i32::try_from(shape.maximum_arguments).unwrap_or(i32::MAX));
        self.native_function(
            NativeCallback::Callable(callback.clone(), prefix),
            declared.map(|shape| shape.metadata),
        )
    }

    fn constructor(
        &mut self,
        callback: &ConstructorCallback,
        metadata: FunctionMetadata<'_>,
    ) -> Result<Root, Error> {
        self.native_function(
            NativeCallback::Constructor(callback.clone()),
            Some(metadata),
        )
    }

    fn constructor_prototype(
        &mut self,
        new_target: &Root,
        fallback: &PrototypeResolver,
    ) -> Result<Root, Error> {
        self.validate_root(new_target)?;
        let shared_roots = self.roots.clone();
        let result = self.attempt(
            ExceptionPhase::Prototype,
            |scope, roots, realm, intrinsics| {
                let target = v8::Local::<v8::Object>::try_from(roots.local(scope, new_target)?)
                    .map_err(|_| Error::ExpectedFunction)?;
                // The one original Get occurs only when the typed handler requests
                // this transition, after its argument conversions have completed.
                let name = v8::String::new(scope, "prototype").ok_or(Error::BackendUnavailable)?;
                let prototype = target
                    .get(scope, name.into())
                    .ok_or(Error::BackendUnavailable)?;
                roots.available()?;
                if prototype.is_object() {
                    return roots.retain(scope, realm, prototype);
                }
                let context = constructor_realm(scope, roots, intrinsics, target)?;
                roots.available()?;
                let key = context.get_slot::<RealmKey>().ok_or(Error::ForeignOwner)?;
                if key.owner != roots.owner.id() {
                    return Err(Error::ForeignOwner);
                }
                // The private typed key proves this is a context initialized by this
                // owner, including the embedder-data slot's initialization precondition.
                let bundle = context
                    .get_embedder_data(scope, 0)
                    .ok_or(Error::BackendUnavailable)?;
                let bundle = v8::Local::<v8::Object>::try_from(bundle)
                    .map_err(|_| Error::BackendUnavailable)?;
                let scope = &mut v8::ContextScope::new(scope, context);
                let mut entered = Entered {
                    scope,
                    roots: shared_roots,
                    realm: *key,
                    intrinsics: bundle,
                };
                let result = fallback(&mut Scope {
                    operations: &mut entered,
                    invariant: std::marker::PhantomData,
                })
                .map(|value| value.root);
                let result = match entered.check_active() {
                    Err(Error::ConversionCleanup(_)) | Ok(()) => {
                        roots.owner.operation_result(result)
                    }
                    Err(error) => Err(error),
                };
                let root = result?;
                let prototype = roots.local(entered.scope, &root)?;
                if !prototype.is_object() {
                    return Err(Error::ExpectedObject);
                }
                Ok(root)
            },
        );
        match result {
            Err(Error::Exception(exception)) => Err(Error::Exception(exception)),
            Err(error) => Err(self.roots.fail(error)),
            result => result,
        }
    }

    fn evaluate(&mut self, source: &[u16]) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Compile,
            |scope, _roots, _realm, _intrinsics| {
                let source = string(scope, source)?;
                let script =
                    v8::Script::compile(scope, source, None).ok_or(Error::BackendUnavailable)?;
                // Compilation is complete. Run catches are separated so callers can
                // distinguish a syntax exception from an exception during execution.
                Ok(v8::Global::new(scope, script))
            },
        )
        .and_then(|script| {
            self.attempt(
                ExceptionPhase::Evaluate,
                |scope, roots, realm, _intrinsics| {
                    let script = v8::Local::new(scope, &script);
                    let value = script.run(scope).ok_or(Error::BackendUnavailable)?;
                    roots.retain(scope, realm, value)
                },
            )
        })
    }

    fn validate_string_length(&self, units: usize) -> Result<(), Error> {
        validate_string_length(units)
    }

    fn string(&mut self, units: &[u16]) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let value = string(scope, units)?;
                roots.retain(scope, realm, value.into())
            },
        )
    }

    fn primitive(&mut self, primitive: Primitive) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let value = match primitive {
                    Primitive::Undefined => v8::undefined(scope).into(),
                    Primitive::Null => v8::null(scope).into(),
                    Primitive::Boolean(value) => v8::Boolean::new(scope, value).into(),
                    Primitive::Number(value) => v8::Number::new(scope, value).into(),
                };
                roots.retain(scope, realm, value)
            },
        )
    }

    fn kind(&mut self, value: &Root) -> Result<ValueKind, Error> {
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| {
                let value = roots.local(scope, value)?;
                let kind = if value.is_undefined() {
                    ValueKind::Undefined
                } else if value.is_null() {
                    ValueKind::Null
                } else if value.is_boolean() {
                    ValueKind::Boolean
                } else if value.is_number() {
                    ValueKind::Number
                } else if value.is_big_int() {
                    ValueKind::BigInt
                } else if value.is_string() {
                    ValueKind::String
                } else if value.is_symbol() {
                    ValueKind::Symbol
                } else if value.is_object() {
                    ValueKind::Object
                } else {
                    return Err(Error::BackendUnavailable);
                };
                Ok(kind)
            },
        )
    }

    fn is_callable(&mut self, value: &Root) -> Result<bool, Error> {
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| {
                // In the pinned SDK, Value::is_function delegates Value::IsFunction
                // to V8's internal IsCallable (v8/src/api/api.cc), not a JSFunction
                // shape check. It includes classes and revoked callable proxies.
                Ok(roots.local(scope, value)?.is_function())
            },
        )
    }

    fn callback_object_realm(&mut self, value: &Root) -> Result<RealmKey, Error> {
        self.observe_callback_object(value)
    }

    fn callback_object_storage_bytes(&self) -> usize {
        callback_object::CallbackObjects::storage_bytes()
    }

    fn retain_callback_object(
        &mut self,
        object: &Root,
        realm: RealmKey,
        backing: Box<dyn std::any::Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        self.pin_callback_object(object, realm, backing)
    }

    fn open_callback_object(&mut self, holder: &Rc<CallbackObjectIdentity>) -> Result<Root, Error> {
        self.open_pinned_callback_object(holder)
    }

    fn release_callback_object(
        &mut self,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.roots.callback_objects.release(&self.roots, holder)
    }

    fn argument_locals_storage_bytes(&self, count: usize) -> Option<usize> {
        std::mem::size_of::<v8::Local<'_, v8::Value>>().checked_mul(count)
    }

    fn native_error(&mut self) -> Result<Root, Error> {
        self.attempt(ExceptionPhase::Invoke, |scope, roots, realm, intrinsics| {
            let error = pristine_error(scope, intrinsics)?;
            roots.retain(scope, realm, error.into())
        })
    }

    fn dom_exception_profile(&mut self) -> Result<Option<DomExceptionProfile>, Error> {
        self.attempt(
            ExceptionPhase::Prototype,
            |scope, roots, realm, intrinsics| {
                let profile = captured_value(scope, intrinsics, DOM_EXCEPTION_PROFILE_FIELD)?;
                let account = scope
                    .get_current_context()
                    .get_slot::<Weak<ConversionAccount>>();
                if profile.is_undefined() && account.is_none() {
                    return Ok(None);
                }
                let profile = v8::Local::<v8::Array>::try_from(profile)
                    .map_err(|_| roots.fail(Error::BackendUnavailable))?;
                let account = account
                    .and_then(|account| account.upgrade())
                    .ok_or_else(|| roots.fail(Error::BackendUnavailable))?;
                let prototype = profile
                    .get_index(scope, 0)
                    .filter(|prototype| prototype.is_native_error())
                    .ok_or_else(|| roots.fail(Error::BackendUnavailable))?;
                let prototype = roots.retain(scope, realm, prototype)?;
                Ok(Some(DomExceptionProfile { prototype, account }))
            },
        )
    }

    fn publish_dom_exception(
        &mut self,
        key: &[u16],
        constructor: &Root,
        prototype: &Root,
        account: Weak<ConversionAccount>,
    ) -> Result<(), Error> {
        self.validate_root(constructor)?;
        self.validate_root(prototype)?;
        self.attempt(
            ExceptionPhase::DefineProperty,
            |scope, roots, _, intrinsics| {
                let context = scope.get_current_context();
                let previous = captured_value(scope, intrinsics, DOM_EXCEPTION_PROFILE_FIELD)?;
                if !previous.is_undefined() {
                    return Err(Error::PropertyRejected);
                }
                if account.strong_count() == 0
                    || context.get_slot::<Weak<ConversionAccount>>().is_some()
                {
                    return Err(roots.fail(Error::BackendUnavailable));
                }
                let constructor = roots.local(scope, constructor)?;
                let prototype = roots.local(scope, prototype)?;
                let key = string(scope, key)?;
                let global = context.global(scope);
                require_absent_property(scope, roots, global, key.into())?;
                let mut descriptor =
                    v8::PropertyDescriptor::new_from_value_writable(constructor, true);
                descriptor.set_enumerable(false);
                descriptor.set_configurable(true);

                // Keep the native constructor's funded account alive even if guest
                // code replaces the public binding and prototype.constructor. The
                // Context slot holds only a weak Rust edge. Arbitrary backing drops
                // stay in the existing callback registry's owned retirement path.
                let profile = v8::Array::new_with_elements(scope, &[prototype, constructor]);
                if !intrinsics.set_internal_field(DOM_EXCEPTION_PROFILE_FIELD, profile.into()) {
                    return Err(roots.fail(Error::BackendUnavailable));
                }
                if context.set_slot(Rc::new(account)).is_some() {
                    return Err(roots.fail(Error::BackendUnavailable));
                }
                // The backend creates this context's native global without guest
                // interceptors. An own definition doesn't traverse its prototype.
                // The profile is complete before the global can become visible.
                let result = define_own_property(scope, global, key.into(), &descriptor);
                if result.is_err() {
                    context.remove_slot::<Weak<ConversionAccount>>();
                    if !intrinsics.set_internal_field(DOM_EXCEPTION_PROFILE_FIELD, previous.into())
                    {
                        return Err(roots.fail(Error::BackendUnavailable));
                    }
                }
                result
            },
        )
    }

    fn create_dom_exception(
        &mut self,
        prototype: &Root,
        name: &Root,
        message: &Root,
        code: u16,
    ) -> Result<Root, Error> {
        self.validate_root(prototype)?;
        let error = self.attempt(ExceptionPhase::Invoke, |scope, roots, realm, intrinsics| {
            let name = roots.local(scope, name)?;
            let message = roots.local(scope, message)?;
            if !name.is_string() || !message.is_string() {
                return Err(Error::BackendUnavailable);
            }
            let code = v8::Integer::new_from_unsigned(scope, u32::from(code));
            let error = pristine_error(scope, intrinsics)?;
            let slots = v8::Array::new_with_elements(scope, &[name, message, code.into()]);
            let key = v8::Local::new(scope, &roots.dom_exception_key);
            if error.set_private(scope, key, slots.into()) != Some(true) {
                return Err(Error::BackendUnavailable);
            }
            roots.retain(scope, realm, error.into())
        })?;
        // The receiver is a fresh ordinary JSError. Only its proposed
        // prototype can be a Proxy; SetPrototypeOf does not call its traps.
        self.set_prototype(&error, ObjectPrototype::Object(prototype))?;
        Ok(error)
    }

    fn dom_exception_slot(
        &mut self,
        receiver: &Root,
        slot: crate::dom_exception::Slot,
    ) -> Result<Option<Root>, Error> {
        self.validate_root(receiver)?;
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, realm, _| {
            let receiver = roots.local(scope, receiver)?;
            // The SDK's native error predicate rejects proxies, including
            // proxies around branded objects before inspecting private data.
            if !receiver.is_native_error() {
                return Ok(None);
            }
            let receiver = v8::Local::<v8::Object>::try_from(receiver)
                .map_err(|_| Error::BackendUnavailable)?;
            let key = v8::Local::new(scope, &roots.dom_exception_key);
            if !receiver
                .has_private(scope, key)
                .ok_or(Error::BackendUnavailable)?
            {
                return Ok(None);
            }
            let slots = receiver
                .get_private(scope, key)
                .ok_or(Error::BackendUnavailable)?;
            let slots =
                v8::Local::<v8::Array>::try_from(slots).map_err(|_| Error::BackendUnavailable)?;
            let value = slots
                .get_index(scope, slot as u32)
                .ok_or(Error::BackendUnavailable)?;
            roots.retain(scope, realm, value).map(Some)
        })
    }

    fn global(&mut self) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::PropertyRead,
            |scope, roots, realm, _intrinsics| {
                let context = scope.get_current_context();
                let value = context.global(scope);
                roots.retain(scope, realm, value.into())
            },
        )
    }

    fn get(&mut self, object: &Root, key: PropertyKey<'_>) -> Result<Root, Error> {
        self.validate_root(object)?;
        self.attempt(
            ExceptionPhase::PropertyRead,
            |scope, roots, realm, _intrinsics| {
                let value = roots.local(scope, object)?;
                let object =
                    v8::Local::<v8::Object>::try_from(value).map_err(|_| Error::ExpectedObject)?;
                let name = property_key(scope, roots, key)?;
                let value = object
                    .get(scope, name.into())
                    .ok_or(Error::BackendUnavailable)?;
                roots.retain(scope, realm, value)
            },
        )
    }

    fn set(&mut self, object: &Root, key: PropertyKey<'_>, value: &Root) -> Result<(), Error> {
        self.validate_root(object)?;
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::PropertyWrite,
            |scope, roots, _realm, intrinsics| {
                let object = roots.local(scope, object)?;
                let object =
                    v8::Local::<v8::Object>::try_from(object).map_err(|_| Error::ExpectedObject)?;
                let name = property_key(scope, roots, key)?;
                let value = roots.local(scope, value)?;
                // Object::set returns true even when a nonwritable property silently
                // rejects the value. Retain the pristine intrinsic at realm creation
                // so its real Boolean result cannot be replaced by guest globals.
                let setter = captured_method(scope, intrinsics, 0)?;
                let receiver = v8::undefined(scope);
                let accepted = setter
                    .call(scope, receiver.into(), &[object.into(), name.into(), value])
                    .ok_or(Error::BackendUnavailable)?;
                if accepted.is_true() {
                    Ok(())
                } else {
                    Err(Error::PropertyRejected)
                }
            },
        )
    }

    fn define_property(
        &mut self,
        object: &Root,
        key: PropertyKey<'_>,
        descriptor: PropertyDescriptor<'_>,
        definition: PropertyDefinition,
    ) -> Result<(), Error> {
        self.validate_root(object)?;
        self.attempt(
            ExceptionPhase::DefineProperty,
            |scope, roots, _realm, _intrinsics| {
                let object = v8::Local::<v8::Object>::try_from(roots.local(scope, object)?)
                    .map_err(|_| Error::ExpectedObject)?;
                let key = property_key(scope, roots, key)?;
                let (mut descriptor, enumerable, configurable) = match descriptor {
                    PropertyDescriptor::Data {
                        value,
                        writable,
                        enumerable,
                        configurable,
                    } => (
                        v8::PropertyDescriptor::new_from_value_writable(
                            roots.local(scope, value)?,
                            writable,
                        ),
                        enumerable,
                        configurable,
                    ),
                    PropertyDescriptor::Accessor {
                        get,
                        set,
                        enumerable,
                        configurable,
                    } => {
                        let get = accessor(scope, roots, get)?;
                        let set = accessor(scope, roots, set)?;
                        (
                            v8::PropertyDescriptor::new_from_get_set(get, set),
                            enumerable,
                            configurable,
                        )
                    }
                };
                descriptor.set_enumerable(enumerable);
                descriptor.set_configurable(configurable);
                if matches!(definition, PropertyDefinition::RequireAbsent) {
                    require_absent_property(scope, roots, object, key)?;
                }
                define_own_property(scope, object, key, &descriptor)
            },
        )
    }

    fn object(&mut self, prototype: ObjectPrototype<'_>) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Prototype,
            |scope, roots, realm, _intrinsics| {
                let prototype = prototype_value(scope, roots, prototype)?;
                let object = v8::Object::with_prototype_and_properties(scope, prototype, &[], &[]);
                roots.retain(scope, realm, object.into())
            },
        )
    }

    fn managed_object(&mut self) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Prototype,
            |scope, roots, realm, intrinsics| {
                let template = v8::ObjectTemplate::new(scope);
                if !template.set_internal_field_count(3) {
                    return Err(Error::BackendUnavailable);
                }
                let holder = template
                    .new_instance(scope)
                    .ok_or(Error::BackendUnavailable)?;
                let object = v8::Object::new(scope);
                let context = scope.get_current_context();
                if !holder.set_internal_field(0, object.into())
                    || !holder.set_internal_field(1, context.into())
                    || !holder.set_internal_field(2, intrinsics.into())
                {
                    return Err(Error::BackendUnavailable);
                }
                roots.retain(scope, realm, holder.into())
            },
        )
    }

    fn native_object_storage_bytes(&self) -> usize {
        native_object::NativeObjects::storage_bytes()
    }

    fn project_native_object(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn std::any::Any>,
    ) -> Result<Root, Error> {
        self.project_payload(prototype, payload)
    }

    fn project_retained_native_object(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn std::any::Any>,
        backing: Box<dyn std::any::Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        self.project_pinned_payload(prototype, payload, backing)
    }

    fn project_retained_native_object_in_prototype_realm(
        &mut self,
        prototype: &Root,
        realm: RealmKey,
        payload: Rc<dyn std::any::Any>,
        backing: Box<dyn std::any::Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        self.project_pinned_payload_in_prototype_realm(prototype, realm, payload, backing)
    }

    fn native_object(&mut self, object: &Root) -> Result<Option<Rc<dyn std::any::Any>>, Error> {
        self.native_payload(object)
    }

    fn native_object_index(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
    ) -> Result<Option<u32>, Error> {
        self.native_index(object, brand)
    }

    fn open_native_object(
        &mut self,
        index: u32,
        brand: std::any::TypeId,
    ) -> Result<Option<Root>, Error> {
        self.open_native_index(index, brand)
    }

    fn native_object_reference(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
        slot: u32,
    ) -> Result<Option<Root>, Error> {
        self.read_native_reference(object, brand, slot)
    }

    fn set_native_object_reference(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
        slot: u32,
        reference: Option<&Root>,
    ) -> Result<(), Error> {
        self.write_native_reference(object, brand, slot, reference)
    }

    fn retain_root_backing(
        &mut self,
        root: &Root,
        backing: Box<dyn std::any::Any>,
    ) -> Result<(), Error> {
        let entry = self.roots.entry(root)?;
        entry
            .backing
            .set(backing)
            .map_err(|_| Error::RegistryConflict)
    }

    fn open_managed_object(&mut self, holder: &Root) -> Result<Root, Error> {
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, realm, _| {
            let holder = roots.local(scope, holder).map_err(|error| match error {
                Error::ExpiredIdentity => roots.conflict(),
                error => error,
            })?;
            let holder = v8::Local::<v8::Object>::try_from(holder).map_err(|_| roots.conflict())?;
            let object = holder
                .get_internal_field(scope, 0)
                .and_then(|value| v8::Local::<v8::Value>::try_from(value).ok())
                .ok_or_else(|| roots.conflict())?;
            roots.retain(scope, realm, object)
        })
    }

    fn set_prototype(
        &mut self,
        object: &Root,
        prototype: ObjectPrototype<'_>,
    ) -> Result<(), Error> {
        self.validate_root(object)?;
        self.attempt(
            ExceptionPhase::Prototype,
            |scope, roots, _realm, intrinsics| {
                let object = v8::Local::<v8::Object>::try_from(roots.local(scope, object)?)
                    .map_err(|_| Error::ExpectedObject)?;
                let prototype = prototype_value(scope, roots, prototype)?;
                let setter = captured_method(scope, intrinsics, 2)?;
                let receiver = v8::undefined(scope);
                let accepted = setter
                    .call(scope, receiver.into(), &[object.into(), prototype])
                    .ok_or(Error::BackendUnavailable)?;
                roots.available()?;
                if accepted.is_true() {
                    Ok(())
                } else {
                    Err(Error::PropertyRejected)
                }
            },
        )
    }

    fn array(&mut self, values: &[Root]) -> Result<Root, Error> {
        self.check_active()?;
        super::validate_array_length(values.len())?;
        for value in values {
            self.validate_root(value)?;
        }
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let mut elements = Vec::new();
                elements
                    .try_reserve_exact(values.len())
                    .map_err(|_| Error::Allocation)?;
                for value in values {
                    elements.push(roots.local(scope, value)?);
                }
                let array = v8::Array::new_with_elements(scope, &elements);
                roots.retain(scope, realm, array.into())
            },
        )
    }

    fn well_known_symbol(&mut self, symbol: WellKnownSymbol) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let symbol = match symbol {
                    WellKnownSymbol::Iterator => v8::Symbol::get_iterator(scope),
                    WellKnownSymbol::ToStringTag => v8::Symbol::get_to_string_tag(scope),
                    WellKnownSymbol::Unscopables => v8::Symbol::get_unscopables(scope),
                };
                roots.retain(scope, realm, symbol.into())
            },
        )
    }

    fn intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::PropertyRead,
            |scope, roots, realm, intrinsics| {
                let index = match intrinsic {
                    Intrinsic::ArrayValues => 1,
                };
                let method = captured_method(scope, intrinsics, index)?;
                roots.retain(scope, realm, method.into())
            },
        )
    }

    fn call(
        &mut self,
        function: &Root,
        receiver: &Root,
        arguments: &[Root],
    ) -> Result<Root, Error> {
        self.check_active()?;
        if arguments.len() > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        self.validate_root(function)?;
        self.validate_root(receiver)?;
        for argument in arguments {
            self.validate_root(argument)?;
        }
        self.attempt(
            ExceptionPhase::Invoke,
            |scope, roots, realm, _intrinsics| {
                let function = roots.local(scope, function)?;
                let function = v8::Local::<v8::Function>::try_from(function)
                    .map_err(|_| Error::ExpectedFunction)?;
                let receiver = roots.local(scope, receiver)?;
                let mut values = Vec::new();
                values
                    .try_reserve_exact(arguments.len())
                    .map_err(|_| Error::Allocation)?;
                for argument in arguments {
                    values.push(roots.local(scope, argument)?);
                }
                let value = function
                    .call(scope, receiver, &values)
                    .ok_or(Error::BackendUnavailable)?;
                roots.retain(scope, realm, value)
            },
        )
    }

    fn coerce_string(&mut self, value: &Root) -> Result<CoercedString, Error> {
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, realm, _intrinsics| {
                let value = roots.local(scope, value)?;
                let string = value.to_string(scope).ok_or(Error::BackendUnavailable)?;
                roots.available()?;
                Ok(CoercedString {
                    length: string.length(),
                    value: roots.retain(scope, realm, string.into())?,
                })
            },
        )
    }

    fn copy_string(&mut self, string: &CoercedString, units: &mut [u16]) -> Result<(), Error> {
        self.validate_root(&string.value)?;
        if units.len() != string.len() {
            return Err(Error::BackendUnavailable);
        }
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| {
                let string = roots.local(scope, &string.value)?;
                let string = v8::Local::<v8::String>::try_from(string)
                    .map_err(|_| Error::BackendUnavailable)?;
                string.write_v2(scope, 0, units, v8::WriteFlags::empty());
                Ok(())
            },
        )
    }

    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error> {
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| Ok(roots.local(scope, value)?.boolean_value(scope)),
        )
    }

    fn to_number(&mut self, value: &Root) -> Result<f64, Error> {
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| {
                roots
                    .local(scope, value)?
                    .number_value(scope)
                    .ok_or(Error::BackendUnavailable)
            },
        )
    }

    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error> {
        self.validate_root(left)?;
        self.validate_root(right)?;
        self.attempt(
            ExceptionPhase::Coerce,
            |scope, roots, _realm, _intrinsics| {
                let left = roots.local(scope, left)?;
                let right = roots.local(scope, right)?;
                Ok(left.strict_equals(right))
            },
        )
    }
}

enum CallbackOutcome<'s> {
    Undefined,
    Return(v8::Local<'s, v8::Value>),
    Throw(v8::Local<'s, v8::Value>),
}

fn callback_trampoline<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    let Some(roots) = scope.get_slot::<Weak<Roots>>().and_then(Weak::upgrade) else {
        scope.terminate_execution();
        return;
    };
    if roots.available().is_err() {
        return;
    }
    // This is the public Rust callback boundary. A panic must never unwind
    // through the SDK ABI. Arbitrary capture destructors and SDK fatal errors
    // are not promised to be recoverable or universally panic free.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match invoke_callback(scope, &arguments, &roots) {
            Ok(outcome) => Some(outcome),
            Err(error) => {
                // Selection can discard an owned native cause or storage
                // guard. Dispose of callback errors within this unwind guard.
                roots.fail(error);
                None
            }
        }
    }));
    let outcome = match outcome {
        Ok(Some(outcome)) => outcome,
        Ok(None) => return,
        Err(payload) => {
            let selected = roots.owner.finalize::<()>(Err(Error::CallbackPanicked));
            if let Err(error) = selected {
                roots.fail(error);
            }
            // Disposal happens when this owner drops, after all callback frames
            // have returned. Nested callback panics can contribute several
            // payloads while the first typed host failure stays unchanged.
            let mut payloads = roots.panic_payloads.take();
            payloads.push(payload);
            roots.panic_payloads.set(payloads);
            return;
        }
    };
    if roots.available().is_err() {
        return;
    }
    match outcome {
        CallbackOutcome::Undefined => (),
        CallbackOutcome::Return(value) => result.set(value),
        CallbackOutcome::Throw(value) => {
            // All operation-local TryCatch frames have finished by this point.
            scope.throw_exception(value);
        }
    }
}

fn invoke_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: &v8::FunctionCallbackArguments<'s>,
    roots: &Rc<Roots>,
) -> Result<CallbackOutcome<'s>, Error> {
    roots.available()?;
    let boundary = crate::owner::Boundary::enter(roots.owner.clone())?;
    let data = v8::Local::<v8::Object>::try_from(arguments.data())
        .map_err(|_| Error::BackendUnavailable)?;
    let id = data
        .get_internal_field(scope, 0)
        .ok_or(Error::BackendUnavailable)?;
    let id = v8::Local::<v8::Value>::try_from(id).map_err(|_| Error::BackendUnavailable)?;
    let id = v8::Local::<v8::Uint32>::try_from(id)
        .map_err(|_| Error::BackendUnavailable)?
        .value();
    let context = data
        .get_internal_field(scope, 1)
        .ok_or(Error::BackendUnavailable)?;
    let context =
        v8::Local::<v8::Context>::try_from(context).map_err(|_| Error::BackendUnavailable)?;
    let intrinsics = data
        .get_internal_field(scope, 2)
        .ok_or(Error::BackendUnavailable)?;
    let intrinsics =
        v8::Local::<v8::Value>::try_from(intrinsics).map_err(|_| Error::BackendUnavailable)?;
    let intrinsics =
        v8::Local::<v8::Object>::try_from(intrinsics).map_err(|_| Error::BackendUnavailable)?;
    let role = data
        .get_internal_field(scope, 3)
        .ok_or(Error::BackendUnavailable)?;
    let role = v8::Local::<v8::Value>::try_from(role).map_err(|_| Error::BackendUnavailable)?;
    let role = v8::Local::<v8::Uint32>::try_from(role)
        .map_err(|_| Error::BackendUnavailable)?
        .value();
    // Only a strong Rust lease clone leaves the borrow. User invocation and
    // SDK calls happen after it ends; retirement is drained at owned host
    // boundaries after all SDK frames and registry borrows have ended.
    let lease = roots
        .callbacks
        .try_borrow()
        .map_err(|_| roots.conflict())?
        .get(&id)
        .filter(|entry| !entry.retired.get())
        .map(|entry| entry.lease.clone())
        .ok_or(Error::ExpiredIdentity)?;
    if lease.realm.owner != roots.owner.id() {
        return Err(Error::ForeignOwner);
    }
    let scope = &mut v8::ContextScope::new(scope, context);
    let mut entered = Entered {
        scope,
        roots: roots.clone(),
        realm: lease.realm,
        intrinsics,
    };
    entered.check_active()?;
    enum OwnedCompletion {
        Undefined,
        Return(Root),
        Construct(Root),
        Throw(Root),
    }
    let outcome = match (&lease.callback, role) {
        (NativeCallback::Callable(callback, prefix), 0) => {
            let receiver = Value {
                root: roots.retain(entered.scope, lease.realm, arguments.this().into())?,
                invariant: std::marker::PhantomData,
            };
            let mut values = Vec::new();
            let supplied = arguments.length();
            let selected = prefix.map_or(supplied, |prefix| prefix.min(supplied));
            let count = usize::try_from(selected).map_err(|_| Error::TooManyArguments)?;
            values
                .try_reserve_exact(count)
                .map_err(|_| Error::Allocation)?;
            for index in 0..selected {
                values.push(Value {
                    root: roots.retain(entered.scope, lease.realm, arguments.get(index))?,
                    invariant: std::marker::PhantomData,
                });
            }
            match (callback.0)(
                &mut Scope {
                    operations: &mut entered,
                    invariant: std::marker::PhantomData,
                },
                Arguments { receiver, values },
            ) {
                Ok(Completion::Undefined) => Ok(OwnedCompletion::Undefined),
                Ok(Completion::Return(value)) => Ok(OwnedCompletion::Return(value.root)),
                Ok(Completion::Throw(value)) => Ok(OwnedCompletion::Throw(value.root)),
                Err(error) => Err(error),
            }
        }
        (NativeCallback::Constructor(_), 1) => {
            let error = entered.type_error(
                &"Constructor requires 'new'"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?;
            Ok(OwnedCompletion::Throw(error))
        }
        (NativeCallback::Constructor(callback), 2) => {
            if arguments.length() != 3 {
                return Err(Error::BackendUnavailable);
            }
            let list = v8::Local::<v8::Array>::try_from(arguments.get(1))
                .map_err(|_| Error::BackendUnavailable)?;
            let length = list.length();
            let selected = callback
                .0
                .argument_limit
                .map_or(length, |limit| length.min(limit));
            let count = usize::try_from(selected).map_err(|_| Error::TooManyArguments)?;
            if count > i32::MAX as usize {
                return Err(Error::TooManyArguments);
            }
            let mut values = Vec::new();
            values
                .try_reserve_exact(count)
                .map_err(|_| Error::Allocation)?;
            for index in 0..selected {
                let value = list
                    .get_index(entered.scope, index)
                    .ok_or(Error::BackendUnavailable)?;
                values.push(Value {
                    root: roots.retain(entered.scope, lease.realm, value)?,
                    invariant: std::marker::PhantomData,
                });
            }
            let new_target = Value {
                root: roots.retain(entered.scope, lease.realm, arguments.get(2))?,
                invariant: std::marker::PhantomData,
            };
            let outcome = (callback.0.action)(
                &mut Scope {
                    operations: &mut entered,
                    invariant: std::marker::PhantomData,
                },
                ConstructorArguments {
                    values,
                    new_target,
                    fallback: callback.0.fallback.clone(),
                },
            );
            match outcome {
                Ok(ConstructorCompletion::Return(value)) => {
                    Ok(OwnedCompletion::Construct(value.root))
                }
                Ok(ConstructorCompletion::Throw(value)) => Ok(OwnedCompletion::Throw(value.root)),
                Err(error) => Err(error),
            }
        }
        _ => Err(Error::BackendUnavailable),
    };
    let outcome = match outcome {
        Ok(OwnedCompletion::Throw(value)) => Err(Error::Exception(Exception {
            phase: ExceptionPhase::Invoke,
            value,
            rule: None,
        })),
        other => other,
    };
    let outcome = match entered.check_active() {
        Err(Error::ConversionCleanup(_)) | Ok(()) => outcome,
        Err(error) => Err(error),
    };
    match boundary.finish(outcome) {
        Ok(OwnedCompletion::Undefined) => Ok(CallbackOutcome::Undefined),
        Ok(OwnedCompletion::Return(root)) => {
            Ok(CallbackOutcome::Return(roots.local(entered.scope, &root)?))
        }
        Ok(OwnedCompletion::Construct(root)) => {
            let value = roots.local(entered.scope, &root)?;
            if value.is_object() {
                Ok(CallbackOutcome::Return(value))
            } else {
                Err(Error::ExpectedObject)
            }
        }
        Ok(OwnedCompletion::Throw(_)) => Err(Error::BackendUnavailable),
        Err(Error::Exception(exception)) => Ok(CallbackOutcome::Throw(
            roots.local(entered.scope, exception.value())?,
        )),
        Err(error) => Err(error),
    }
}

fn callback_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<v8::Context>,
    intrinsics: v8::Local<v8::Object>,
    id: u32,
    role: CallbackRole,
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let template = v8::ObjectTemplate::new(scope);
    if !template.set_internal_field_count(4) {
        return Err(Error::BackendUnavailable);
    }
    let data = template
        .new_instance(scope)
        .ok_or(Error::BackendUnavailable)?;
    let id = v8::Integer::new_from_unsigned(scope, id);
    let role = v8::Integer::new_from_unsigned(scope, role as u32);
    if !data.set_internal_field(0, id.into())
        || !data.set_internal_field(1, context.into())
        || !data.set_internal_field(2, intrinsics.into())
        || !data.set_internal_field(3, role.into())
    {
        return Err(Error::BackendUnavailable);
    }
    Ok(data)
}

fn private_handler<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
    function: v8::Local<'s, v8::Function>,
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let key = v8::String::new(scope, name).ok_or(Error::BackendUnavailable)?;
    let null = v8::null(scope);
    Ok(v8::Object::with_prototype_and_properties(
        scope,
        null.into(),
        &[key.into()],
        &[function.into()],
    ))
}

fn constructor_realm<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    intrinsics: v8::Local<v8::Object>,
    target: v8::Local<v8::Object>,
) -> Result<v8::Local<'s, v8::Context>, Error> {
    let mut target = v8::Local::<v8::Value>::from(target);
    while target.is_proxy() {
        roots.available()?;
        let proxy =
            v8::Local::<v8::Proxy>::try_from(target).map_err(|_| Error::BackendUnavailable)?;
        if proxy.is_revoked() {
            let message = v8::String::new(scope, "Cannot resolve a revoked constructor realm")
                .ok_or(Error::BackendUnavailable)?;
            let error = v8::Exception::type_error(scope, message);
            scope.throw_exception(error);
            return Err(Error::BackendUnavailable);
        }
        target = proxy.get_target(scope);
    }
    let target = v8::Local::<v8::Object>::try_from(target).map_err(|_| Error::ExpectedFunction)?;
    // bind reads GetPrototype and CopyNameAndLength. Sanitize name/length reads
    // to ordinary own DATA values, with no getter or descriptor-trap invocation.
    let get = v8::Function::builder(sanitized_constructor_get)
        .data(target.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
        .ok_or(Error::BackendUnavailable)?;
    let handler = private_handler(scope, "get", get)?;
    let sanitized = v8::Proxy::new(scope, target, handler).ok_or(Error::BackendUnavailable)?;
    let bind = captured_method(scope, intrinsics, 3)?;
    let undefined = v8::undefined(scope);
    let shell = bind
        .call(scope, sanitized.into(), &[undefined.into()])
        .ok_or(Error::BackendUnavailable)?;
    roots.available()?;
    let shell = v8::Local::<v8::Object>::try_from(shell).map_err(|_| Error::BackendUnavailable)?;
    // The fresh bound shell has no own prototype. Never force through a guest
    // target's potentially frozen data property and violate Proxy invariants.
    let get = v8::Function::builder(undefined_constructor_get)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
        .ok_or(Error::BackendUnavailable)?;
    let handler = private_handler(scope, "get", get)?;
    let forcing = v8::Proxy::new(scope, shell, handler).ok_or(Error::BackendUnavailable)?;
    let construct = captured_method(scope, intrinsics, 4)?;
    let object = captured_method(scope, intrinsics, 5)?;
    let arguments = v8::Array::new(scope, 0);
    let probe = construct
        .call(
            scope,
            undefined.into(),
            &[object.into(), arguments.into(), forcing.into()],
        )
        .ok_or(Error::BackendUnavailable)?;
    roots.available()?;
    let probe = v8::Local::<v8::Object>::try_from(probe).map_err(|_| Error::BackendUnavailable)?;
    let prototype = probe
        .get_prototype(scope)
        .ok_or(Error::BackendUnavailable)?;
    let prototype =
        v8::Local::<v8::Object>::try_from(prototype).map_err(|_| Error::BackendUnavailable)?;
    // The probe object can have A's map context. Its forced intrinsic fallback
    // prototype is B.Object.prototype and carries the true function realm B.
    prototype
        .get_creation_context(scope)
        .ok_or(Error::BackendUnavailable)
}

fn undefined_constructor_get(
    _scope: &mut v8::PinScope,
    _arguments: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    result.set_undefined();
}

fn sanitized_constructor_get(
    scope: &mut v8::PinScope,
    arguments: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    let Some(roots) = scope.get_slot::<Weak<Roots>>().and_then(Weak::upgrade) else {
        scope.terminate_execution();
        return;
    };
    if roots.available().is_err() {
        return;
    }
    let Ok(target) = v8::Local::<v8::Object>::try_from(arguments.data()) else {
        roots.fail(Error::BackendUnavailable);
        return;
    };
    let Ok(key) = v8::Local::<v8::Name>::try_from(arguments.get(1)) else {
        roots.fail(Error::ExpectedPropertyKey);
        return;
    };
    let Some(descriptor) = target.get_own_property_descriptor(scope, key) else {
        return;
    };
    if roots.available().is_err() || descriptor.is_undefined() {
        return;
    }
    let Ok(descriptor) = v8::Local::<v8::Object>::try_from(descriptor) else {
        roots.fail(Error::BackendUnavailable);
        return;
    };
    let Some(key) = v8::String::new(scope, "value") else {
        roots.fail(Error::BackendUnavailable);
        return;
    };
    // Do not read an inherited descriptor.value getter for an accessor record.
    if descriptor.has_own_property(scope, key.into()) != Some(true) {
        return;
    }
    if roots.available().is_err() {
        return;
    }
    if let Some(value) = descriptor.get(scope, key.into()) {
        result.set(value);
    }
}

fn pristine_method<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<v8::Object>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, Error> {
    let name = v8::String::new(scope, name).ok_or(Error::BackendUnavailable)?;
    let method = object
        .get(scope, name.into())
        .ok_or(Error::BackendUnavailable)?;
    v8::Local::<v8::Function>::try_from(method).map_err(|_| Error::BackendUnavailable)
}

// The immutable constructor prototype belongs to the selected realm.
// Stock Error construction with zero arguments creates the native error and
// stack state without defining an own message or invoking guest coercion.
fn pristine_error<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    intrinsics: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let constructor = captured_method(scope, intrinsics, ERROR_CONSTRUCTOR_FIELD)?;
    let error = constructor
        .new_instance(scope, &[])
        .ok_or(Error::BackendUnavailable)?;
    if error.is_native_error() {
        Ok(error)
    } else {
        Err(Error::BackendUnavailable)
    }
}

fn captured_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    intrinsics: v8::Local<'s, v8::Object>,
    index: usize,
) -> Result<v8::Local<'s, v8::Value>, Error> {
    let value = intrinsics
        .get_internal_field(scope, index)
        .ok_or(Error::BackendUnavailable)?;
    v8::Local::<v8::Value>::try_from(value).map_err(|_| Error::BackendUnavailable)
}

fn captured_method<'s>(
    scope: &v8::PinScope<'s, '_>,
    intrinsics: v8::Local<'s, v8::Object>,
    index: usize,
) -> Result<v8::Local<'s, v8::Function>, Error> {
    let method = captured_value(scope, intrinsics, index)?;
    v8::Local::<v8::Function>::try_from(method).map_err(|_| Error::BackendUnavailable)
}

fn require_absent_property(
    scope: &v8::PinScope,
    roots: &Roots,
    object: v8::Local<v8::Object>,
    key: v8::Local<v8::Name>,
) -> Result<(), Error> {
    match object.has_own_property(scope, key) {
        Some(false) => (),
        Some(true) => return Err(Error::PropertyRejected),
        None => return Err(Error::BackendUnavailable),
    }
    // A generic own-property probe may have invoked a proxy trap.
    roots.available()?;
    if scope.is_execution_terminating() {
        return Err(Error::Terminated);
    }
    Ok(())
}

fn define_own_property(
    scope: &v8::PinScope,
    object: v8::Local<v8::Object>,
    key: v8::Local<v8::Name>,
    descriptor: &v8::PropertyDescriptor,
) -> Result<(), Error> {
    match object.define_property(scope, key, descriptor) {
        Some(true) => Ok(()),
        Some(false) => Err(Error::PropertyRejected),
        None => Err(Error::BackendUnavailable),
    }
}

fn property_key<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    key: PropertyKey<'_>,
) -> Result<v8::Local<'s, v8::Name>, Error> {
    match key {
        PropertyKey::String(units) => Ok(string(scope, units)?.into()),
        PropertyKey::Value(root) => {
            // Owner validation precedes handle opening; this safe predicate
            // never performs ToPropertyKey or runs guest conversion hooks.
            let value = roots.local(scope, root)?;
            v8::Local::<v8::Name>::try_from(value).map_err(|_| Error::ExpectedPropertyKey)
        }
    }
}

fn accessor<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    value: Option<&Root>,
) -> Result<v8::Local<'s, v8::Value>, Error> {
    let value = match value {
        Some(root) => roots.local(scope, root)?,
        None => v8::undefined(scope).into(),
    };
    if value.is_undefined() || value.is_function() {
        Ok(value)
    } else {
        Err(Error::ExpectedAccessor)
    }
}

fn prototype_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    prototype: ObjectPrototype<'_>,
) -> Result<v8::Local<'s, v8::Value>, Error> {
    match prototype {
        ObjectPrototype::Null => Ok(v8::null(scope).into()),
        ObjectPrototype::Default => v8::Object::new(scope)
            .get_prototype(scope)
            .ok_or(Error::BackendUnavailable),
        ObjectPrototype::Object(root) => {
            let value = roots.local(scope, root)?;
            if value.is_object() {
                Ok(value)
            } else {
                Err(Error::ExpectedObject)
            }
        }
    }
}

fn validate_string_length(units: usize) -> Result<(), Error> {
    if units > v8::String::MAX_LENGTH {
        Err(Error::StringTooLong)
    } else {
        Ok(())
    }
}

fn string<'s>(
    scope: &v8::PinScope<'s, '_>,
    units: &[u16],
) -> Result<v8::Local<'s, v8::String>, Error> {
    validate_string_length(units.len())?;
    v8::String::new_from_two_byte(scope, units, v8::NewStringType::Normal)
        .ok_or(Error::BackendUnavailable)
}
