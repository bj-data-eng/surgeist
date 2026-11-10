use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::ThreadId;

use super::Operations;
use crate::{
    Arguments, Callback, Completion, Error, Exception, ExceptionPhase, Realm, RealmIdentity,
    RealmKey, Root, RootIdentity, Scope, Value,
};

#[cfg(test)]
mod tests;

static OWNER_THREAD: OnceLock<ThreadId> = OnceLock::new();
static ACTIVE: AtomicBool = AtomicBool::new(false);
static NEXT_OWNER: AtomicU64 = AtomicU64::new(0);

struct ActiveOwner;

impl Drop for ActiveOwner {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::Release);
    }
}

struct RootEntry {
    identity: Weak<RootIdentity>,
    value: v8::Global<v8::Value>,
}

struct CallbackLease {
    callback: Callback,
    realm: RealmKey,
}

struct CallbackEntry {
    lease: Weak<CallbackLease>,
    // Keep the ORIGINAL weak handle. Cloning an SDK Weak loses its finalizer.
    _function: v8::Weak<v8::Function>,
}

struct Roots {
    owner: u64,
    next: Cell<u64>,
    entries: RefCell<BTreeMap<u64, Rc<RootEntry>>>,
    closed: Cell<bool>,
    failure: Cell<Option<Error>>,
    next_callback: Cell<u32>,
    callbacks: RefCell<BTreeMap<u32, CallbackEntry>>,
    interrupt: v8::IsolateHandle,
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
        if realm.owner != self.owner {
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
        if root.0.realm.owner != self.owner {
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
        let failure = self.failure.take();
        let result = failure.clone();
        self.failure.set(failure);
        if let Some(error) = result {
            Err(error)
        } else if self.closed.get() {
            Err(Error::BackendUnavailable)
        } else {
            Ok(())
        }
    }

    fn fail(&self, error: Error) -> Error {
        let first = self.failure.take().unwrap_or(error);
        self.failure.set(Some(first.clone()));
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
        let retired = match self.callbacks.try_borrow_mut() {
            Ok(mut entries) => {
                let dead: Vec<_> = entries
                    .iter()
                    .filter_map(|(id, entry)| (entry.lease.strong_count() == 0).then_some(*id))
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
    }
}

struct ContextEntry {
    identity: Weak<RealmIdentity>,
    context: v8::Global<v8::Context>,
    set: v8::Global<v8::Function>,
}

pub(crate) struct Engine {
    owner: u64,
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
        let roots = Rc::new(Roots {
            owner,
            next: Cell::new(0),
            entries: RefCell::new(BTreeMap::new()),
            closed: Cell::new(false),
            failure: Cell::new(None),
            next_callback: Cell::new(0),
            callbacks: RefCell::new(BTreeMap::new()),
            interrupt: isolate.thread_safe_handle(),
            panic_payloads: Cell::new(Vec::new()),
        });
        isolate.set_slot(Rc::downgrade(&roots));
        Ok(Self {
            owner,
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
        let setter = v8::Local::new(scope, &context.set);
        let scope = &mut v8::ContextScope::new(scope, local);
        let mut entered = Entered {
            scope,
            roots: self.roots.clone(),
            realm: realm.0.key,
            setter,
        };
        let result = operation(&mut entered);
        entered.check_active()?;
        result
    }

    fn validate_realm(&self, realm: &Realm) -> Result<(), Error> {
        self.roots.available()?;
        if realm.0.key.owner != self.owner {
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
                owner: self.owner,
                index,
            },
        }));
        let pinned = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut pinned.init();
        let context = v8::Context::new(scope, Default::default());
        let setter = {
            let scope = &mut v8::ContextScope::new(scope, context);
            let reflect_key = v8::String::new(scope, "Reflect").ok_or(Error::BackendUnavailable)?;
            let reflect = context
                .global(scope)
                .get(scope, reflect_key.into())
                .ok_or(Error::BackendUnavailable)?;
            let reflect = v8::Local::<v8::Object>::try_from(reflect)
                .map_err(|_| Error::BackendUnavailable)?;
            let set_key = v8::String::new(scope, "set").ok_or(Error::BackendUnavailable)?;
            let setter = reflect
                .get(scope, set_key.into())
                .ok_or(Error::BackendUnavailable)?;
            let setter = v8::Local::<v8::Function>::try_from(setter)
                .map_err(|_| Error::BackendUnavailable)?;
            v8::Global::new(scope, setter)
        };
        self.contexts.insert(
            index,
            ContextEntry {
                identity: Rc::downgrade(&realm.0),
                context: v8::Global::new(scope, context),
                set: setter,
            },
        );
        self.next_realm = index;
        Ok(realm)
    }

    fn with_realm(
        &mut self,
        realm: &Realm,
        action: &mut dyn FnMut(&mut dyn Operations),
    ) -> Result<(), Error> {
        self.with_context(realm, |entered| {
            action(entered);
            Ok(())
        })
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
}

impl Drop for Engine {
    fn drop(&mut self) {
        // No checkpoint during teardown: abandoned guest jobs must not run.
        self.roots.closed.set(true);
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
    setter: v8::Local<'s, v8::Function>,
}

impl Entered<'_, '_, '_> {
    fn attempt<T>(
        &mut self,
        phase: ExceptionPhase,
        operation: impl for<'s> FnOnce(
            &mut v8::PinScope<'s, '_>,
            &Roots,
            RealmKey,
            v8::Local<'s, v8::Function>,
        ) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.check_active()?;
        let caught = std::pin::pin!(v8::TryCatch::new(self.scope));
        let caught = &mut caught.init();
        let setter = v8::Local::new(caught, self.setter);
        let result = operation(caught, &self.roots, self.realm, setter);
        self.roots.available()?;
        if caught.has_terminated() || !caught.can_continue() {
            return Err(Error::Terminated);
        }
        if let Some(thrown) = caught.exception() {
            let value = self.roots.retain(caught, self.realm, thrown)?;
            return Err(Error::Exception(Exception { phase, value }));
        }
        result
    }
}

impl Operations for Entered<'_, '_, '_> {
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
        self.attempt(ExceptionPhase::Coerce, |scope, roots, realm, _setter| {
            let message = string(scope, message)?;
            let exception = v8::Exception::type_error(scope, message);
            roots.retain(scope, realm, exception)
        })
    }

    fn function(&mut self, callback: &Callback) -> Result<Root, Error> {
        self.attempt(ExceptionPhase::Coerce, |scope, roots, realm, setter| {
            let id = roots
                .next_callback
                .get()
                .checked_add(1)
                .ok_or(Error::IdentityExhausted)?;
            roots.next_callback.set(id);
            let lease = Rc::new(CallbackLease {
                callback: callback.clone(),
                realm,
            });
            let template = v8::ObjectTemplate::new(scope);
            if !template.set_internal_field_count(3) {
                return Err(Error::BackendUnavailable);
            }
            let data = template
                .new_instance(scope)
                .ok_or(Error::BackendUnavailable)?;
            let identity = v8::Integer::new_from_unsigned(scope, id);
            let context = scope.get_current_context();
            if !data.set_internal_field(0, identity.into())
                || !data.set_internal_field(1, context.into())
                || !data.set_internal_field(2, setter.into())
            {
                return Err(Error::BackendUnavailable);
            }
            let function = v8::Function::builder(callback_trampoline)
                .data(data.into())
                .constructor_behavior(v8::ConstructorBehavior::Throw)
                .build(scope)
                .ok_or(Error::BackendUnavailable)?;
            let weak_lease = Rc::downgrade(&lease);
            let weak =
                v8::Weak::with_guaranteed_finalizer(scope, function, Box::new(move || drop(lease)));
            roots
                .callbacks
                .try_borrow_mut()
                .map_err(|_| roots.conflict())?
                .insert(
                    id,
                    CallbackEntry {
                        lease: weak_lease,
                        _function: weak,
                    },
                );
            roots.retain(scope, realm, function.into())
        })
    }

    fn evaluate(&mut self, source: &[u16]) -> Result<Root, Error> {
        self.attempt(ExceptionPhase::Compile, |scope, _roots, _realm, _setter| {
            let source = string(scope, source)?;
            let script =
                v8::Script::compile(scope, source, None).ok_or(Error::BackendUnavailable)?;
            // Compilation is complete. Run catches are separated so callers can
            // distinguish a syntax exception from an exception during execution.
            Ok(v8::Global::new(scope, script))
        })
        .and_then(|script| {
            self.attempt(ExceptionPhase::Evaluate, |scope, roots, realm, _setter| {
                let script = v8::Local::new(scope, &script);
                let value = script.run(scope).ok_or(Error::BackendUnavailable)?;
                roots.retain(scope, realm, value)
            })
        })
    }

    fn string(&mut self, units: &[u16]) -> Result<Root, Error> {
        self.attempt(ExceptionPhase::Coerce, |scope, roots, realm, _setter| {
            let value = string(scope, units)?;
            roots.retain(scope, realm, value.into())
        })
    }

    fn global(&mut self) -> Result<Root, Error> {
        self.attempt(
            ExceptionPhase::PropertyRead,
            |scope, roots, realm, _setter| {
                let context = scope.get_current_context();
                let value = context.global(scope);
                roots.retain(scope, realm, value.into())
            },
        )
    }

    fn get(&mut self, object: &Root, name: &[u16]) -> Result<Root, Error> {
        self.validate_root(object)?;
        self.attempt(
            ExceptionPhase::PropertyRead,
            |scope, roots, realm, _setter| {
                let value = roots.local(scope, object)?;
                let object =
                    v8::Local::<v8::Object>::try_from(value).map_err(|_| Error::ExpectedObject)?;
                let name = string(scope, name)?;
                let value = object
                    .get(scope, name.into())
                    .ok_or(Error::BackendUnavailable)?;
                roots.retain(scope, realm, value)
            },
        )
    }

    fn set(&mut self, object: &Root, name: &[u16], value: &Root) -> Result<(), Error> {
        self.validate_root(object)?;
        self.validate_root(value)?;
        self.attempt(
            ExceptionPhase::PropertyWrite,
            |scope, roots, _realm, setter| {
                let object = roots.local(scope, object)?;
                let object =
                    v8::Local::<v8::Object>::try_from(object).map_err(|_| Error::ExpectedObject)?;
                let name = string(scope, name)?;
                let value = roots.local(scope, value)?;
                // Object::set returns true even when a nonwritable property silently
                // rejects the value. Retain the pristine intrinsic at realm creation
                // so its real Boolean result cannot be replaced by guest globals.
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
        self.attempt(ExceptionPhase::Invoke, |scope, roots, realm, _setter| {
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
        })
    }

    fn to_utf16(&mut self, value: &Root) -> Result<Vec<u16>, Error> {
        self.validate_root(value)?;
        self.attempt(ExceptionPhase::Coerce, |scope, roots, _realm, _setter| {
            let value = roots.local(scope, value)?;
            let string = value.to_string(scope).ok_or(Error::BackendUnavailable)?;
            roots.available()?;
            let mut units = Vec::new();
            units
                .try_reserve_exact(string.length())
                .map_err(|_| Error::Allocation)?;
            units.resize(string.length(), 0);
            string.write_v2(scope, 0, &mut units, v8::WriteFlags::empty());
            Ok(units)
        })
    }

    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error> {
        self.validate_root(value)?;
        self.attempt(ExceptionPhase::Coerce, |scope, roots, _realm, _setter| {
            Ok(roots.local(scope, value)?.boolean_value(scope))
        })
    }

    fn to_number(&mut self, value: &Root) -> Result<f64, Error> {
        self.validate_root(value)?;
        self.attempt(ExceptionPhase::Coerce, |scope, roots, _realm, _setter| {
            roots
                .local(scope, value)?
                .number_value(scope)
                .ok_or(Error::BackendUnavailable)
        })
    }

    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error> {
        self.validate_root(left)?;
        self.validate_root(right)?;
        self.attempt(ExceptionPhase::Coerce, |scope, roots, _realm, _setter| {
            let left = roots.local(scope, left)?;
            let right = roots.local(scope, right)?;
            Ok(left.strict_equals(right))
        })
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
        invoke_callback(scope, &arguments, &roots)
    }));
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(payload) => {
            roots.fail(Error::CallbackPanicked);
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
        Ok(CallbackOutcome::Undefined) => (),
        Ok(CallbackOutcome::Return(value)) => result.set(value),
        Ok(CallbackOutcome::Throw(value)) => {
            // All operation-local TryCatch frames have finished by this point.
            scope.throw_exception(value);
        }
        Err(error) => {
            roots.fail(error);
        }
    }
}

fn invoke_callback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: &v8::FunctionCallbackArguments<'s>,
    roots: &Rc<Roots>,
) -> Result<CallbackOutcome<'s>, Error> {
    roots.available()?;
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
    let setter = data
        .get_internal_field(scope, 2)
        .ok_or(Error::BackendUnavailable)?;
    let setter = v8::Local::<v8::Value>::try_from(setter).map_err(|_| Error::BackendUnavailable)?;
    let setter =
        v8::Local::<v8::Function>::try_from(setter).map_err(|_| Error::BackendUnavailable)?;
    // Only a Rust Weak clone leaves the borrow. Upgrading, user invocation,
    // SDK calls and finalizer-driven capture drops happen after it ends.
    let lease = roots
        .callbacks
        .try_borrow()
        .map_err(|_| roots.conflict())?
        .get(&id)
        .map(|entry| entry.lease.clone())
        .ok_or(Error::ExpiredIdentity)?;
    let lease = lease.upgrade().ok_or(Error::ExpiredIdentity)?;
    if lease.realm.owner != roots.owner {
        return Err(Error::ForeignOwner);
    }
    let scope = &mut v8::ContextScope::new(scope, context);
    let mut entered = Entered {
        scope,
        roots: roots.clone(),
        realm: lease.realm,
        setter,
    };
    entered.check_active()?;
    let receiver = Value {
        root: roots.retain(entered.scope, lease.realm, arguments.this().into())?,
        invariant: std::marker::PhantomData,
    };
    let mut values = Vec::new();
    let count = usize::try_from(arguments.length()).map_err(|_| Error::TooManyArguments)?;
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    for index in 0..arguments.length() {
        values.push(Value {
            root: roots.retain(entered.scope, lease.realm, arguments.get(index))?,
            invariant: std::marker::PhantomData,
        });
    }
    enum OwnedCompletion {
        Undefined,
        Return(Root),
        Throw(Root),
    }
    // Erase the scope brand only inside this backend, after the callback has
    // returned. Its public completion can never escape to an outside caller.
    let outcome = match (lease.callback.0)(
        &mut Scope {
            operations: &mut entered,
            invariant: std::marker::PhantomData,
        },
        Arguments { receiver, values },
    ) {
        Ok(Completion::Undefined) => Ok(OwnedCompletion::Undefined),
        Ok(Completion::Return(value)) => Ok(OwnedCompletion::Return(value.root)),
        Ok(Completion::Throw(value)) => Ok(OwnedCompletion::Throw(value.root)),
        Err(Error::Exception(exception)) => Ok(OwnedCompletion::Throw(exception.value)),
        Err(error) => Err(error),
    };
    entered.check_active()?;
    match outcome? {
        OwnedCompletion::Undefined => Ok(CallbackOutcome::Undefined),
        OwnedCompletion::Return(root) => {
            Ok(CallbackOutcome::Return(roots.local(entered.scope, &root)?))
        }
        OwnedCompletion::Throw(root) => {
            Ok(CallbackOutcome::Throw(roots.local(entered.scope, &root)?))
        }
    }
}

fn string<'s>(
    scope: &v8::PinScope<'s, '_>,
    units: &[u16],
) -> Result<v8::Local<'s, v8::String>, Error> {
    if units.len() > v8::String::MAX_LENGTH {
        return Err(Error::StringTooLong);
    }
    v8::String::new_from_two_byte(scope, units, v8::NewStringType::Normal)
        .ok_or(Error::BackendUnavailable)
}
