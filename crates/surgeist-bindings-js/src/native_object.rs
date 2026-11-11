use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::{Error, ObjectPrototype, RetainedCallbackObject, Scope, Value};

/// One typed native payload and its admitted Rust backing. A lookup can keep
/// both alive after the original JavaScript object becomes unreachable.
pub struct NativeObject<T> {
    payload: T,
    _backing: Box<dyn Any>,
}

impl<T> NativeObject<T> {
    fn new<Storage: 'static>(payload: T, backing: Storage) -> Self {
        Self {
            payload,
            _backing: Box::new(backing),
        }
    }

    /// Borrows the original native payload.
    pub fn payload(&self) -> &T {
        &self.payload
    }
}

/// Weak identity of one exact native projection. It retains neither the
/// guest object nor its payload, and cannot reopen it after collection.
pub struct NativeObjectToken<T> {
    owner: u64,
    index: u32,
    marker: PhantomData<Rc<T>>,
}

impl<'scope> Scope<'scope> {
    fn native_projection_bytes<T, Storage>(&self) -> Result<usize, Error> {
        self.operations
            .native_object_storage_bytes()
            .checked_add(size_of::<NativeObject<T>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Storage>()))
            .ok_or(Error::Allocation)
    }

    /// Create a new object with a native-only typed payload. Admission receives
    /// the requested Rust record, payload and backing sizes before allocation,
    /// excluding allocator and engine storage. The object weakly indexes its
    /// payload, which must not retain a guest root that can reach this object.
    pub fn project_native_object<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        payload: T,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Value<'scope>, E> {
        self.check_active()?;
        let backing = admit(self.native_projection_bytes::<T, Storage>()?)?;
        let root = self.operations.project_native_object(
            prototype.input(),
            Rc::new(NativeObject::new(payload, backing)),
        )?;
        Ok(self.value(root))
    }

    /// Project a typed native object and retain its exact guest identity as
    /// one operation. The admission closure funds the weak payload and the
    /// persistent holder separately, before either is published. Each guard
    /// follows the backing it funds. Neither may retain engine custody.
    pub fn project_retained_native_object<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        payload: T,
        mut admit: impl FnMut(usize) -> Result<Storage, E>,
    ) -> Result<RetainedCallbackObject, E> {
        let realm = self.realm_authority()?;
        let projection = admit(self.native_projection_bytes::<T, Storage>()?)?;
        let holder_backing = admit(self.callback_object_bytes::<Storage>()?)?;
        let holder = self.operations.project_retained_native_object(
            prototype.input(),
            Rc::new(NativeObject::new(payload, projection)),
            Box::new(holder_backing),
        )?;
        Ok(RetainedCallbackObject::new(holder, realm))
    }

    /// Project a typed native object in the retained prototype's actual
    /// creation realm, including from a callback entered in another realm.
    /// The prototype must belong to this engine owner. Admission funds the
    /// payload and persistent holder before the new object is published.
    pub fn project_retained_native_object_in_prototype_realm<
        T: 'static,
        Storage: 'static,
        E: From<Error>,
    >(
        &mut self,
        prototype: &RetainedCallbackObject,
        payload: T,
        mut admit: impl FnMut(usize) -> Result<Storage, E>,
    ) -> Result<RetainedCallbackObject, E> {
        let realm = prototype.associated_realm().clone();
        let prototype = self.open_callback_object(prototype)?;
        let projection = admit(self.native_projection_bytes::<T, Storage>()?)?;
        let holder_backing = admit(self.callback_object_bytes::<Storage>()?)?;
        let holder = self
            .operations
            .project_retained_native_object_in_prototype_realm(
                &prototype.root,
                realm.key,
                Rc::new(NativeObject::new(payload, projection)),
                Box::new(holder_backing),
            )?;
        Ok(RetainedCallbackObject::new(holder, realm))
    }

    /// Observe the exact branded object's weak identity without retaining it.
    /// An ordinary, proxy, or differently branded object yields `None`.
    pub fn native_object_token<T: 'static>(
        &mut self,
        object: &Value<'scope>,
    ) -> Result<Option<NativeObjectToken<T>>, Error> {
        let index = self
            .operations
            .native_object_index(&object.root, TypeId::of::<NativeObject<T>>())?;
        let owner = self.operations.owner_state().id();
        Ok(index.map(|index| NativeObjectToken {
            owner,
            index,
            marker: PhantomData,
        }))
    }

    /// Reopen a live weak projection in its own heap. Collection yields
    /// `None`. A token from a different heap is refused before backend access.
    pub fn open_native_object<T: 'static>(
        &mut self,
        token: &NativeObjectToken<T>,
    ) -> Result<Option<Value<'scope>>, Error> {
        self.check_active()?;
        if token.owner != self.operations.owner_state().id() {
            return Err(Error::ForeignOwner);
        }
        let root = self
            .operations
            .open_native_object(token.index, TypeId::of::<NativeObject<T>>())?;
        Ok(root.map(|root| self.value(root)))
    }

    /// Read a private engine-traced object reference from the exact projection.
    /// Ordinary properties, inherited wrappers and proxies cannot supply it.
    /// Returns `None` for an empty slot and refuses an incorrect receiver brand.
    pub fn native_object_reference<T: 'static>(
        &mut self,
        object: &Value<'scope>,
        slot: u32,
    ) -> Result<Option<Value<'scope>>, Error> {
        let root = self.operations.native_object_reference(
            &object.root,
            TypeId::of::<NativeObject<T>>(),
            slot,
        )?;
        Ok(root.map(|root| self.value(root)))
    }

    /// Set or clear a private engine-traced object reference. A slot does not
    /// root the receiver or its reference outside the guest heap. Native
    /// consumers own slot assignment and admission; guest property changes,
    /// including freezing the receiver, do not change private slots.
    pub fn set_native_object_reference<T: 'static>(
        &mut self,
        object: &Value<'scope>,
        slot: u32,
        reference: Option<&Value<'scope>>,
    ) -> Result<(), Error> {
        self.operations.set_native_object_reference(
            &object.root,
            TypeId::of::<NativeObject<T>>(),
            slot,
            reference.map(|reference| &reference.root),
        )
    }

    /// Resolve only the exact branded object and requested native payload type.
    /// This performs no property lookup, coercion or Proxy unwrapping.
    pub fn native_object<T: 'static>(
        &mut self,
        object: &Value<'scope>,
    ) -> Result<Option<Rc<NativeObject<T>>>, Error> {
        Ok(self
            .operations
            .native_object(&object.root)?
            .and_then(|payload| payload.downcast().ok()))
    }
}
