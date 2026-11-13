use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::callback_object::CallbackObjectIdentity;
use crate::{Error, ObjectPrototype, RealmAuthority, RetainedCallbackObject, Scope, Value};

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

impl<T> NativeObjectToken<T> {
    pub(crate) fn identity_parts(&self) -> (u64, u32) {
        (self.owner, self.index)
    }
}

/// Prototype and creation realm for an unpublished native projection.
pub enum NativeProjectionPrototype<'value, 'scope> {
    /// Use the current realm and an ordinary scoped prototype choice.
    Current(ObjectPrototype<'value, 'scope>),
    /// Create the object in the retained prototype's own realm.
    Creation(&'value RetainedCallbackObject),
}

struct ProjectedNativeObject {
    holder: Rc<CallbackObjectIdentity>,
    realm: RealmAuthority,
    index: u32,
}

impl ProjectedNativeObject {
    fn into_holder(self) -> RetainedCallbackObject {
        RetainedCallbackObject::new(self.holder, self.realm)
    }
}

/// Private guest object and its admitted backing. Dropping it before
/// publication discards the native entry and persistent holder immediately.
pub struct PreparedNativeProjection<'borrow, 'scope, T> {
    value: Value<'scope>,
    native: Rc<NativeObject<T>>,
    abort: AbortProjection<'borrow, 'scope>,
}

struct AbortProjection<'borrow, 'scope> {
    scope: &'borrow mut Scope<'scope>,
    projected: ProjectedNativeObject,
    armed: bool,
}

impl<'scope, T> PreparedNativeProjection<'_, 'scope, T> {
    pub(crate) fn token(&self) -> NativeObjectToken<T> {
        NativeObjectToken {
            owner: self.abort.projected.holder.owner,
            index: self.abort.projected.index,
            marker: PhantomData,
        }
    }

    /// Publish the private projection after the host's own commit. Binding its
    /// native payload and transferring its holder require no engine operation.
    pub fn publish<R>(
        mut self,
        bind: impl FnOnce(&T) -> R,
    ) -> (Value<'scope>, RetainedCallbackObject, R) {
        let bound = bind(self.native.payload());
        let holder = RetainedCallbackObject::new(
            self.abort.projected.holder.clone(),
            self.abort.projected.realm.clone(),
        );
        self.abort.armed = false;
        (self.value, holder, bound)
    }
}

impl Drop for AbortProjection<'_, '_> {
    fn drop(&mut self) {
        if self.armed {
            self.scope
                .operations
                .discard_unpublished_native_object(self.projected.index, &self.projected.holder);
        }
    }
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
        self.project_payload(prototype, payload, None, admit)
    }

    /// Project a native object with one engine-traced reference already set.
    /// The reference must be an object. If it cannot be installed, the weak
    /// native entry is never published and its admitted backing is released.
    pub fn project_native_object_with_reference<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        payload: T,
        slot: u32,
        reference: &Value<'scope>,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Value<'scope>, E> {
        self.project_payload(prototype, payload, Some((slot, &reference.root)), admit)
    }

    fn project_payload<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        payload: T,
        reference: Option<(u32, &crate::Root)>,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Value<'scope>, E> {
        self.check_active()?;
        let backing = admit(self.native_projection_bytes::<T, Storage>()?)?;
        let root = self.operations.project_native_object(
            prototype.input(),
            Rc::new(NativeObject::new(payload, backing)),
            reference,
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
        let (holder, _) = self.operations.project_retained_native_object(
            prototype.input(),
            Rc::new(NativeObject::new(payload, projection)),
            Box::new(holder_backing),
        )?;
        Ok(RetainedCallbackObject::new(holder, realm))
    }

    /// Project a typed native object in the retained prototype's actual
    /// creation realm, including from a callback entered in another realm.
    /// The prototype must belong to this engine owner. Admission receives
    /// the payload and persistent holder sizes together, in that order,
    /// and returns their respective guards before the engine forms either.
    /// Each guard follows the backing it funds and may not retain engine
    /// custody.
    pub fn project_retained_native_object_in_prototype_realm<
        T: 'static,
        Storage: 'static,
        E: From<Error>,
    >(
        &mut self,
        prototype: &RetainedCallbackObject,
        payload: T,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
    ) -> Result<RetainedCallbackObject, E> {
        self.form_retained_native_object(
            NativeProjectionPrototype::Creation(prototype),
            payload,
            admit,
        )
        .map(ProjectedNativeObject::into_holder)
    }

    /// Form and validate a private retained projection before attempting the
    /// host operation. The host closure receives only the authenticated Rust
    /// payload. The guest value and holder escape only after it succeeds.
    /// Failure releases their admitted Rust backing without a collection.
    pub fn try_project_retained_native_object<T: 'static, Storage: 'static, R, E: From<Error>>(
        &mut self,
        prototype: NativeProjectionPrototype<'_, 'scope>,
        payload: T,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
        commit: impl FnOnce(&T) -> Result<R, E>,
    ) -> Result<(Value<'scope>, RetainedCallbackObject, R), E> {
        let prepared = self.prepare_retained_native_object(prototype, payload, admit)?;
        let result = commit(prepared.native.payload())?;
        Ok(prepared.publish(|_| result))
    }

    /// Requested Rust backing for the payload and persistent holder, in order.
    pub fn retained_native_object_bytes<T, Storage>(&self) -> Result<(usize, usize), Error> {
        Ok((
            self.native_projection_bytes::<T, Storage>()?,
            self.callback_object_bytes::<Storage>()?,
        ))
    }

    /// Form and validate a private projection while the host's output is still
    /// tentative. Holding it borrows this Scope through publish or abort.
    pub fn prepare_retained_native_object<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: NativeProjectionPrototype<'_, 'scope>,
        payload: T,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
    ) -> Result<PreparedNativeProjection<'_, 'scope, T>, E> {
        let projected = self.form_retained_native_object(prototype, payload, admit)?;
        let abort = AbortProjection {
            scope: self,
            projected,
            armed: true,
        };
        let root = abort
            .scope
            .operations
            .open_callback_object(&abort.projected.holder)?;
        let value = abort.scope.value(root);
        let native = abort
            .scope
            .native_object::<T>(&value)?
            .ok_or(Error::ExpectedNativeObject)?;
        Ok(PreparedNativeProjection {
            value,
            native,
            abort,
        })
    }

    fn form_retained_native_object<T: 'static, Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: NativeProjectionPrototype<'_, 'scope>,
        payload: T,
        admit: impl FnOnce(usize, usize) -> Result<(Storage, Storage), E>,
    ) -> Result<ProjectedNativeObject, E> {
        enum Resolved<'value> {
            Current(crate::backend::ObjectPrototype<'value>),
            Creation(&'value Rc<CallbackObjectIdentity>),
        }
        let (realm, prototype) = match prototype {
            NativeProjectionPrototype::Current(prototype) => (
                self.realm_authority()?,
                Resolved::Current(prototype.input()),
            ),
            NativeProjectionPrototype::Creation(prototype) => (
                prototype.associated_realm().clone(),
                Resolved::Creation(prototype.validated_holder(self)?),
            ),
        };
        let (projection_bytes, holder_bytes) = self.retained_native_object_bytes::<T, Storage>()?;
        let (projection, holder_backing) = admit(projection_bytes, holder_bytes)?;
        let payload = Rc::new(NativeObject::new(payload, projection));
        let backing = Box::new(holder_backing);
        let (holder, index) = match prototype {
            Resolved::Current(prototype) => self
                .operations
                .project_retained_native_object(prototype, payload, backing)?,
            Resolved::Creation(prototype) => self
                .operations
                .project_retained_native_object_in_prototype_realm(
                    prototype, realm.key, payload, backing,
                )?,
        };
        Ok(ProjectedNativeObject {
            holder,
            realm,
            index,
        })
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
