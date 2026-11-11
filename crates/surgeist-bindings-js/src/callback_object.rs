//! Engine observations and explicit native custody for guest callback objects.

use std::rc::Rc;

use crate::{Error, Process, RealmAuthority, Scope, Value};

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
    /// The same engine Realm observed before this object was retained.
    pub fn associated_realm(&self) -> &RealmAuthority {
        &self.realm
    }
}

impl<'scope> Scope<'scope> {
    /// Observe an object's engine creation Realm without invoking guest code.
    /// The observation is independent of the scope that retained its root.
    /// A proxy or object with no supported creation context refuses explicitly.
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
        let bytes = self
            .operations
            .callback_object_storage_bytes()
            .checked_add(size_of::<Storage>())
            .ok_or(Error::Allocation)?;
        let storage = admit(bytes)?;
        let holder = self.operations.retain_callback_object(
            &callback.object.root,
            callback.realm.key,
            Box::new(storage),
        )?;
        Ok(RetainedCallbackObject {
            holder,
            realm: callback.realm,
        })
    }

    /// Open the exact retained object in a scope of its owning heap.
    pub fn open_callback_object(
        &mut self,
        callback: &RetainedCallbackObject,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.open_callback_object(&callback.holder)?;
        Ok(self.value(root))
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
