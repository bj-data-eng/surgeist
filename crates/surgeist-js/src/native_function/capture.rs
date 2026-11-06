//! Separates live invocation state from edges retained by a function object.

use surgeist_js_gc::heap::{RecordRef, Rooted, Trace};

use crate::{
    JsObject, JsValue,
    object::{JsFunction, NativeObject, ObjectEdge},
    realm::{Realm, RealmEdge},
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
    fn restore(stored: &Self::Stored) -> Self;
}

impl<T: Trace> NativeCapture for Rooted<T> {
    type Stored = RecordRef<T>;

    fn store(&self) -> Self::Stored {
        self.record_ref()
    }

    fn restore(stored: &Self::Stored) -> Self {
        stored
            .upgrade()
            .expect("capture heap is available")
            .expect("live function retains capture")
    }
}

/// An object captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedObject(ObjectEdge);

impl NativeCapture for JsObject {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self::from_inner(stored.0.clone())
    }
}

impl<T: NativeObject> NativeCapture for JsObject<T> {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Self {
        JsObject::from_inner(stored.0.clone())
            .downcast()
            .expect("capture retains its native brand")
    }
}

impl NativeCapture for JsFunction {
    type Stored = CapturedObject;

    fn store(&self) -> Self::Stored {
        CapturedObject(self.clone().into_record())
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self::from_record(stored.0.clone())
    }
}

/// A JavaScript value captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedValue(StoredValue);

impl NativeCapture for JsValue {
    type Stored = CapturedValue;

    fn store(&self) -> Self::Stored {
        CapturedValue(self.clone().into_stored())
    }

    fn restore(stored: &Self::Stored) -> Self {
        stored.0.to_value()
    }
}

/// A realm captured in a native function record.
#[derive(Debug, Clone, Trace)]
pub struct CapturedRealm(RealmEdge);

impl NativeCapture for Realm {
    type Stored = CapturedRealm;

    fn store(&self) -> Self::Stored {
        CapturedRealm(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self::from_edge(stored.0.clone())
    }
}

macro_rules! capture_leaf {
    ($($type:ty),+ $(,)?) => {
        $(
            impl NativeCapture for $type {
                type Stored = Self;

                fn store(&self) -> Self {
                    self.clone()
                }

                fn restore(stored: &Self) -> Self {
                    stored.clone()
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

    fn restore(stored: &Self::Stored) -> Self {
        stored.as_ref().map(T::restore)
    }
}

impl<T: NativeCapture> NativeCapture for Vec<T> {
    type Stored = Vec<T::Stored>;

    fn store(&self) -> Self::Stored {
        self.iter().map(T::store).collect()
    }

    fn restore(stored: &Self::Stored) -> Self {
        stored.iter().map(T::restore).collect()
    }
}

macro_rules! capture_tuple {
    ($($type:ident $field:tt),+) => {
        impl<$($type: NativeCapture),+> NativeCapture for ($($type,)+) {
            type Stored = ($($type::Stored,)+);

            fn store(&self) -> Self::Stored {
                ($(self.$field.store(),)+)
            }

            fn restore(stored: &Self::Stored) -> Self {
                ($($type::restore(&stored.$field),)+)
            }
        }
    };
}

capture_tuple!(A 0, B 1);
capture_tuple!(A 0, B 1, C 2);
capture_tuple!(A 0, B 1, C 2, D 3);
