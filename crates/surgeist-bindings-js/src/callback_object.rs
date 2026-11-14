//! Engine observations and explicit native custody for guest callback objects.

use std::rc::Rc;

use crate::{Error, Exception, ExceptionPhase, Process, PropertyKey, RealmAuthority, Scope, Value};

pub(crate) struct CallbackObjectIdentity {
    pub(crate) owner: u64,
    pub(crate) index: u64,
    pub(crate) _backing: Rc<HolderBacking>,
}

/// Shared caller funding for Rust holder metadata. SDK handles remain solely
/// in the backend entry; a surviving identity cannot keep that entry alive.
pub(crate) struct HolderBacking {
    _storage: Box<dyn std::any::Any>,
}

impl HolderBacking {
    pub(crate) fn new(storage: Box<dyn std::any::Any>) -> Rc<Self> {
        Rc::new(Self { _storage: storage })
    }
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

/// One retained guest object in its original process. Clones share one holder
/// and its caller-funded Rust metadata. Releasing the holder or Process drops
/// the SDK handles immediately; metadata backing remains admitted until the
/// final identity and backend entry are gone. An identity retains no engine.
#[derive(Clone)]
pub struct RetainedCallbackObject {
    holder: Rc<CallbackObjectIdentity>,
    realm: RealmAuthority,
}

impl RetainedCallbackObject {
    pub(crate) fn new(holder: Rc<CallbackObjectIdentity>, realm: RealmAuthority) -> Self {
        Self { holder, realm }
    }

    pub(crate) fn validated_holder(
        &self,
        scope: &Scope<'_>,
    ) -> Result<&Rc<CallbackObjectIdentity>, Error> {
        scope.operations.validate_callback_object(&self.holder)?;
        Ok(&self.holder)
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
    /// with the holder identity and backend entry through their actual final
    /// release. It must contain only caller funding, without engine custody or
    /// retained binding identities. Cloning a retained holder shares this
    /// admission without allocating a new holder record.
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

    /// Open an existing retained object with root backing supplied from a
    /// current native read. This performs no guest conversion or invocation.
    pub fn open_callback_object_funded<T: crate::preparation::Retention>(
        &mut self,
        callback: &RetainedCallbackObject,
        controller: &crate::preparation::Controller<T>,
        fund: impl FnOnce(crate::preparation::Request) -> Result<T, Error>,
    ) -> Result<Value<'scope>, Error> {
        self.operations.validate_callback_object(&callback.holder)?;
        let backing = self.fund_root(controller, fund)?;
        let root = self
            .operations
            .open_callback_object_funded(&callback.holder, backing)?;
        Ok(self.value(root))
    }

    /// Release this managed callback handle from within a callback. Surviving
    /// holder aliases keep metadata backing and become unavailable for opening.
    /// Any separately opened scoped value remains valid.
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
    /// Release the exact managed callback handle without guest entry. Surviving
    /// aliases keep metadata backing and become unavailable for opening. This
    /// also works after the original process refuses execution. If the process
    /// is foreign, the original owner can still reap the handle.
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
        let result = scope.project_retained_native_object_in_prototype_realm(&stale, (), |_, _| {
            admitted = true;
            Ok::<_, Error>(((), ()))
        });
        assert!(matches!(result, Err(Error::ExpiredIdentity)));
        assert!(!admitted);
        Ok(())
    })
}
