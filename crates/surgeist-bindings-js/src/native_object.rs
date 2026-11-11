use std::any::Any;
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
