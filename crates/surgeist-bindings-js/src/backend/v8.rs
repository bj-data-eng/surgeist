use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::ThreadId;

use crate::{Error, Exception, ExceptionPhase, Realm, RealmIdentity, Root, RootIdentity};

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

struct Roots {
    owner: u64,
    next: Cell<u64>,
    entries: RefCell<BTreeMap<u64, Rc<RootEntry>>>,
    closed: Cell<bool>,
    conflicted: Cell<bool>,
    interrupt: v8::IsolateHandle,
}

impl Roots {
    fn retain(
        &self,
        scope: &v8::PinScope,
        realm: &Realm,
        value: v8::Local<v8::Value>,
    ) -> Result<Root, Error> {
        self.available()?;
        if realm.0.owner != self.owner {
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
        let root = Root(Rc::new(RootIdentity {
            realm: realm.clone(),
            index,
        }));
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
        if root.0.realm.0.owner != self.owner {
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
        if self.closed.get() {
            Err(Error::BackendUnavailable)
        } else if self.conflicted.get() {
            Err(Error::RegistryConflict)
        } else {
            Ok(())
        }
    }

    fn conflict(&self) -> Error {
        self.conflicted.set(true);
        // This is an engine integrity failure, not a catchable guest exception.
        // V8 observes termination at interrupt checks, not necessarily at the
        // next callback return. Every native entry and post-coercion mutation
        // must also check this sticky failure before accepting further work.
        self.interrupt.terminate_execution();
        Error::RegistryConflict
    }

    fn reap(&self) {
        match self.entries.try_borrow_mut() {
            Ok(mut entries) => entries.retain(|_, entry| entry.identity.strong_count() != 0),
            Err(_) => {
                self.conflict();
            }
        }
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
            conflicted: Cell::new(false),
            interrupt: isolate.thread_safe_handle(),
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

    fn enter<T>(
        &mut self,
        realm: &Realm,
        phase: ExceptionPhase,
        operation: impl for<'s> FnOnce(&mut v8::PinScope<'s, '_>, &Roots) -> Result<T, Error>,
    ) -> Result<T, Error> {
        <Self as super::Engine>::validate_realm(self, realm)?;
        if self.isolate.is_execution_terminating() {
            return Err(Error::Terminated);
        }
        let context = &self
            .contexts
            .get(&realm.0.index)
            .ok_or(Error::ExpiredIdentity)?
            .context;
        // SDK scope! and tc_scope! expand to unsafe. Safe pinning keeps both the
        // handle frame and its exception catcher immobile without those macros.
        let pinned = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut pinned.init();
        let context = v8::Local::new(scope, context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let caught = std::pin::pin!(v8::TryCatch::new(scope));
        let caught = &mut caught.init();
        let result = operation(caught, &self.roots);
        if self.roots.conflicted.get() {
            return Err(Error::RegistryConflict);
        }
        if caught.has_terminated() || !caught.can_continue() {
            return Err(Error::Terminated);
        }
        if let Some(thrown) = caught.exception() {
            let value = self.roots.retain(caught, realm, thrown)?;
            return Err(Error::Exception(Exception { phase, value }));
        }
        result
    }

    fn validate_owner(&self, realm: &Realm) -> Result<(), Error> {
        if realm.0.owner == self.owner {
            Ok(())
        } else {
            Err(Error::ForeignOwner)
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
            owner: self.owner,
            index,
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

    fn validate_realm(&self, realm: &Realm) -> Result<(), Error> {
        self.roots.available()?;
        self.validate_owner(realm)?;
        let context = self
            .contexts
            .get(&realm.0.index)
            .ok_or(Error::ExpiredIdentity)?;
        if context.identity.ptr_eq(&Rc::downgrade(&realm.0)) {
            Ok(())
        } else {
            Err(Error::ExpiredIdentity)
        }
    }

    fn validate_root(&self, root: &Root) -> Result<(), Error> {
        self.roots.entry(root).map(|_| ())
    }

    fn evaluate(&mut self, realm: &Realm, source: &[u16]) -> Result<Root, Error> {
        self.enter(realm, ExceptionPhase::Compile, |scope, _roots| {
            let source = string(scope, source)?;
            let script =
                v8::Script::compile(scope, source, None).ok_or(Error::BackendUnavailable)?;
            // Compilation is complete. Run catches are separated so callers can
            // distinguish a syntax exception from an exception during execution.
            Ok(v8::Global::new(scope, script))
        })
        .and_then(|script| {
            self.enter(realm, ExceptionPhase::Evaluate, |scope, roots| {
                let script = v8::Local::new(scope, &script);
                let value = script.run(scope).ok_or(Error::BackendUnavailable)?;
                roots.retain(scope, realm, value)
            })
        })
    }

    fn string(&mut self, realm: &Realm, units: &[u16]) -> Result<Root, Error> {
        self.enter(realm, ExceptionPhase::Coerce, |scope, roots| {
            let value = string(scope, units)?;
            roots.retain(scope, realm, value.into())
        })
    }

    fn global(&mut self, realm: &Realm) -> Result<Root, Error> {
        self.enter(realm, ExceptionPhase::PropertyRead, |scope, roots| {
            let context = scope.get_current_context();
            let value = context.global(scope);
            roots.retain(scope, realm, value.into())
        })
    }

    fn get(&mut self, realm: &Realm, object: &Root, name: &[u16]) -> Result<Root, Error> {
        self.validate_root(object)?;
        self.enter(realm, ExceptionPhase::PropertyRead, |scope, roots| {
            let value = roots.local(scope, object)?;
            let object =
                v8::Local::<v8::Object>::try_from(value).map_err(|_| Error::ExpectedObject)?;
            let name = string(scope, name)?;
            let value = object
                .get(scope, name.into())
                .ok_or(Error::BackendUnavailable)?;
            roots.retain(scope, realm, value)
        })
    }

    fn set(
        &mut self,
        realm: &Realm,
        object: &Root,
        name: &[u16],
        value: &Root,
    ) -> Result<(), Error> {
        self.validate_root(object)?;
        self.validate_root(value)?;
        self.validate_realm(realm)?;
        let setter = self
            .contexts
            .get(&realm.0.index)
            .ok_or(Error::ExpiredIdentity)?
            .set
            .clone();
        self.enter(realm, ExceptionPhase::PropertyWrite, |scope, roots| {
            let object = roots.local(scope, object)?;
            let object =
                v8::Local::<v8::Object>::try_from(object).map_err(|_| Error::ExpectedObject)?;
            let name = string(scope, name)?;
            let value = roots.local(scope, value)?;
            // Object::set returns true even when a nonwritable property silently
            // rejects the value. Retain the pristine intrinsic at realm creation
            // so its real Boolean result cannot be replaced by guest globals.
            let setter = v8::Local::new(scope, &setter);
            let receiver = v8::undefined(scope);
            let accepted = setter
                .call(scope, receiver.into(), &[object.into(), name.into(), value])
                .ok_or(Error::BackendUnavailable)?;
            if accepted.is_true() {
                Ok(())
            } else {
                Err(Error::PropertyRejected)
            }
        })
    }

    fn call(
        &mut self,
        realm: &Realm,
        function: &Root,
        receiver: &Root,
        arguments: &[Root],
    ) -> Result<Root, Error> {
        if arguments.len() > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        self.validate_root(function)?;
        self.validate_root(receiver)?;
        for argument in arguments {
            self.validate_root(argument)?;
        }
        self.enter(realm, ExceptionPhase::Invoke, |scope, roots| {
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

    fn to_utf16(&mut self, realm: &Realm, value: &Root) -> Result<Vec<u16>, Error> {
        self.validate_root(value)?;
        self.enter(realm, ExceptionPhase::Coerce, |scope, roots| {
            let value = roots.local(scope, value)?;
            let string = value.to_string(scope).ok_or(Error::BackendUnavailable)?;
            let mut units = Vec::new();
            units
                .try_reserve_exact(string.length())
                .map_err(|_| Error::Allocation)?;
            units.resize(string.length(), 0);
            string.write_v2(scope, 0, &mut units, v8::WriteFlags::empty());
            Ok(units)
        })
    }

    fn to_number(&mut self, realm: &Realm, value: &Root) -> Result<f64, Error> {
        self.validate_root(value)?;
        self.enter(realm, ExceptionPhase::Coerce, |scope, roots| {
            roots
                .local(scope, value)?
                .number_value(scope)
                .ok_or(Error::BackendUnavailable)
        })
    }

    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error> {
        self.validate_root(left)?;
        self.validate_root(right)?;
        self.enter(&left.0.realm, ExceptionPhase::Coerce, |scope, roots| {
            let left = roots.local(scope, left)?;
            let right = roots.local(scope, right)?;
            Ok(left.strict_equals(right))
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
