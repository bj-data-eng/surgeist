use std::any::Any;
use std::rc::Rc;

use crate::{Error, ObjectPrototype, Scope, Value};

/// One typed native payload and its admitted Rust backing. A lookup can keep
/// both alive after the original JavaScript object becomes unreachable.
pub struct NativeObject<T> {
    payload: T,
    _backing: Box<dyn Any>,
}

impl<T> NativeObject<T> {
    /// Borrows the original native payload.
    pub fn payload(&self) -> &T {
        &self.payload
    }
}

impl<'scope> Scope<'scope> {
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
        let bytes = self
            .operations
            .native_object_storage_bytes()
            .checked_add(size_of::<NativeObject<T>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Storage>()))
            .ok_or(Error::Allocation)?;
        let backing = admit(bytes)?;
        let root = self.operations.project_native_object(
            prototype.input(),
            Rc::new(NativeObject {
                payload,
                _backing: Box::new(backing),
            }),
        )?;
        Ok(self.value(root))
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
