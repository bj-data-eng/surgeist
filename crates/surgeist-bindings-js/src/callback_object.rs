//! Engine observations and explicit native custody for guest callback objects.

use std::rc::Rc;

use crate::{Error, Exception, ExceptionPhase, Process, PropertyKey, RealmAuthority, Scope, Value};

pub(crate) struct CallbackObjectIdentity {
    pub(crate) owner: u64,
    pub(crate) index: u64,
}

/// A callback object observed in one binding scope.
pub struct CallbackObject<'scope> {
    object: Value<'scope>,
    realm: RealmAuthority,
}

impl CallbackObject<'_> {
    /// The object's associated engine Realm. Native attachment authority is
    /// established by the caller separately.
    pub fn associated_realm(&self) -> &RealmAuthority {
        &self.realm
    }
}

/// One retained guest object in its original process. The backend holds its
/// admitted backing until the actual persistent handle is released.
pub struct RetainedCallbackObject {
    holder: Rc<CallbackObjectIdentity>,
    realm: RealmAuthority,
}

impl RetainedCallbackObject {
    pub(crate) fn new(holder: Rc<CallbackObjectIdentity>, realm: RealmAuthority) -> Self {
        Self { holder, realm }
    }

    /// The same engine Realm observed before this object was retained.
    pub fn associated_realm(&self) -> &RealmAuthority {
        &self.realm
    }
}

impl<'scope> Scope<'scope> {
    pub(crate) fn callback_object_bytes<Storage>(&self) -> Result<usize, Error> {
        self.operations
            .callback_object_storage_bytes()
            .checked_add(size_of::<Storage>())
            .ok_or(Error::Allocation)
    }

    /// Observe an object's engine creation Realm without invoking guest code.
    /// The observation uses the original object, including a Proxy, and is
    /// independent of the scope that retained its root. An unavailable
    /// creation context refuses explicitly.
    pub fn callback_object(
        &mut self,
        value: &Value<'scope>,
    ) -> Result<CallbackObject<'scope>, Error> {
        let key = self.operations.callback_object_realm(&value.root)?;
        let owner = self.operations.owner_state();
        Ok(CallbackObject {
            object: self.value(value.root.clone()),
            realm: RealmAuthority {
                key,
                owner: Rc::downgrade(&owner),
            },
        })
    }

    /// Retain the exact observed object under one newly admitted holder.
    /// The admission closure receives the requested Rust backing bytes,
    /// excluding allocator and engine allocations. Its returned guard travels
    /// with the actual backend handle and should contain no engine custody.
    pub fn retain_callback_object<Storage: 'static, E: From<Error>>(
        &mut self,
        callback: CallbackObject<'scope>,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<RetainedCallbackObject, E> {
        self.check_active()?;
        let storage = admit(self.callback_object_bytes::<Storage>()?)?;
        let holder = self.operations.retain_callback_object(
            &callback.object.root,
            callback.realm.key,
            Box::new(storage),
        )?;
        Ok(RetainedCallbackObject::new(holder, callback.realm))
    }

    /// Open the exact retained object in a scope of its owning heap.
    pub fn open_callback_object(
        &mut self,
        callback: &RetainedCallbackObject,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.open_callback_object(&callback.holder)?;
        Ok(self.value(root))
    }

    /// Release this managed callback handle and its backing from within a
    /// callback. Any separately opened scoped value remains valid.
    pub fn release_callback_object(
        &mut self,
        callback: RetainedCallbackObject,
    ) -> Result<(), Error> {
        self.operations.release_callback_object(&callback.holder)
    }

    /// Read one Web IDL callback-interface operation and call it with the
    /// original object as receiver. A noncallable result produces the entered
    /// Realm's intrinsic TypeError. The host qualifies and enters the callback
    /// object's Realm and prepares the retained callback context.
    pub fn call_object_operation(
        &mut self,
        object: &Value<'scope>,
        operation: &[u16],
        arguments: &[Value<'scope>],
    ) -> Result<Value<'scope>, Error> {
        let method = self.get(object, PropertyKey::String(operation))?;
        if !self.is_callable(&method)? {
            let error = self.type_error(&b"Callback operation must be callable".map(u16::from))?;
            return Err(Error::Exception(Exception {
                phase: ExceptionPhase::Invoke,
                value: error.root,
                rule: None,
            }));
        }
        self.call(&method, object, arguments)
    }
}

impl Process {
    /// Release the exact managed callback handle and its backing without guest
    /// entry. This also works after the original process refuses execution.
    /// If the process is foreign, the original owner can still reap the handle.
    pub fn release_callback_object(
        &mut self,
        callback: RetainedCallbackObject,
    ) -> Result<(), Error> {
        self.engine.release_callback_object(&callback.holder)
    }
}

#[cfg(test)]
pub(crate) fn retired_prototype_refuses_before_native_allocation() -> Result<(), Error> {
    use crate::{EngineKind, ObjectPrototype};

    let mut process = Process::bootstrap(EngineKind::V8)?;
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let value = scope.object(ObjectPrototype::Default)?;
        let observed = scope.callback_object(&value)?;
        let prototype = scope.retain_callback_object(observed, |_| Ok::<(), Error>(()))?;
        // A private alias makes the retired backend entry observable.
        let stale = RetainedCallbackObject::new(prototype.holder.clone(), prototype.realm.clone());
        scope.release_callback_object(prototype)?;
        let mut admitted = false;
        let result = scope.project_retained_native_object_in_prototype_realm(&stale, (), |_| {
            admitted = true;
            Ok::<(), Error>(())
        });
        assert!(matches!(result, Err(Error::ExpiredIdentity)));
        assert!(!admitted);
        Ok(())
    })
}
