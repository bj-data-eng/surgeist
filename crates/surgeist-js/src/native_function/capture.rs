//! Separates live invocation state from edges retained by a function object.

use surgeist_js_gc::heap::{RecordRef, Rooted, Trace};

use crate::{
    JsObject,
    error::RootingError,
    object::{JsFunction, NativeObject, ObjectEdge},
    realm::RealmEdge,
    value::StoredValue,
};

/// Custody of an explicit native callback capture.
///
/// The live value owns its roots. The stored form reports edges to the
/// function's heap, and each invocation restores its own roots. These
/// custody conversions must not reenter the engine or mutate the owning
/// record. The owner may be borrowed until the new roots are installed.
pub trait NativeCapture: 'static {
    /// Non-owning form of the capture when installed in a function record.
    type Stored: Trace;

    /// Stores the capture without transferring the live roots.
    fn store(&self) -> Self::Stored;

    /// Restores the capture while the owning function is rooted.
    ///
    /// Returns only after acquiring all roots required by the live capture.
    fn restore(stored: &Self::Stored) -> Result<Self, RootingError>
    where
        Self: Sized;
}

impl<T: Trace> NativeCapture for Rooted<T> {
    type Stored = RecordRef<T>;

    fn store(&self) -> Self::Stored {
        self.record_ref()
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        stored
            .upgrade()
            .map_err(RootingError::Access)?
            .ok_or(RootingError::Expired)
    }
}

/// An object captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedObject(pub(crate) ObjectEdge);

impl NativeCapture for JsObject {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        stored
            .0
            .upgrade()
            .map_err(RootingError::Access)?
            .ok_or(RootingError::Expired)
    }
}

impl<T: NativeObject> NativeCapture for JsObject<T> {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        JsObject::restore(stored)?
            .downcast()
            .map_err(|_| RootingError::WrongNativeBrand)
    }
}

impl NativeCapture for JsFunction {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.clone().into_record())
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        Self::from_object(JsObject::restore(stored)?).ok_or(RootingError::NotCallable)
    }
}

/// A JavaScript value captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedValue(pub(crate) StoredValue);

/// A realm captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedRealm(pub(crate) RealmEdge);

macro_rules! capture_leaf {
    ($($type:ty),+ $(,)?) => {
        $(
            impl NativeCapture for $type {
                type Stored = Self;

                fn store(&self) -> Self {
                    self.clone()
                }

                fn restore(stored: &Self) -> Result<Self, RootingError> {
                    Ok(stored.clone())
                }
            }
        )+
    };
}

capture_leaf!((), bool, u32, u64, i32, usize, crate::JsString);

impl<T: NativeCapture> NativeCapture for Option<T> {
    type Stored = Option<T::Stored>;

    fn store(&self) -> Self::Stored {
        self.as_ref().map(T::store)
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        stored.as_ref().map(T::restore).transpose()
    }
}

impl<T: NativeCapture> NativeCapture for Vec<T> {
    type Stored = Vec<T::Stored>;

    fn store(&self) -> Self::Stored {
        self.iter().map(T::store).collect()
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        let bytes = stored
            .len()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(RootingError::SizeOverflow)?;
        if bytes > isize::MAX as usize {
            return Err(RootingError::SizeOverflow);
        }
        let mut restored = Self::new();
        restored
            .try_reserve_exact(stored.len())
            .map_err(|_| RootingError::Allocation)?;
        for capture in stored {
            restored.push(T::restore(capture)?);
        }
        Ok(restored)
    }
}

macro_rules! capture_tuple {
    ($($type:ident $field:tt),+) => {
        impl<$($type: NativeCapture),+> NativeCapture for ($($type,)+) {
            type Stored = ($($type::Stored,)+);

            fn store(&self) -> Self::Stored {
                ($(self.$field.store(),)+)
            }

            fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
                Ok(($($type::restore(&stored.$field)?,)+))
            }
        }
    };
}

capture_tuple!(A 0, B 1);
capture_tuple!(A 0, B 1, C 2);
capture_tuple!(A 0, B 1, C 2, D 3);
