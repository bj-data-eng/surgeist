//! Implements object shapes.

pub(crate) mod property_table;
mod root_shape;
pub(crate) mod shared_shape;
pub(crate) mod slot;
pub(crate) mod unique_shape;

pub use root_shape::RootShape;
pub use shared_shape::SharedShape;
pub(crate) use unique_shape::UniqueShape;

use std::fmt::Debug;

use surgeist_js_gc::heap::{AccessError, ObjectId, Trace as HeapTrace};

use crate::{JsResult, property::PropertyKey};

use self::{
    shared_shape::{SharedShapeRef, TransitionKey},
    slot::Slot,
    unique_shape::UniqueShapeRef,
};

use super::JsPrototype;

/// Action to be performed after a property attribute change
//
// Example: of { get/set x() { ... }, y: ... } into { x: ..., y: ... }
//
//                 0       1       2
//    Storage: | get x | set x |   y   |
//
// We delete at position of x which is index 0 (it spans two elements) + 1:
//
//                 0      1
//    Storage: |   x  |   y   |
pub(crate) enum ChangeTransitionAction {
    /// Do nothing to storage.
    Nothing,

    /// Remove element at (index + 1) from storage.
    Remove,

    /// Insert element at (index + 1) into storage.
    Insert,
}

/// The result of a change property attribute transition.
pub(crate) struct ChangeTransition<T> {
    /// The shape after transition.
    pub(crate) shape: T,

    /// The needed action to be performed after transition to the object storage.
    pub(crate) action: ChangeTransitionAction,
}

/// The internal representation of [`Shape`].
#[derive(Debug, Clone)]
enum Inner {
    Unique(UniqueShape),
    Shared(SharedShape),
}

/// Represents the shape of an object.
#[derive(Debug, Clone)]
pub struct Shape {
    inner: Inner,
}

impl Shape {
    /// The max transition count of a [`SharedShape`] from the root node,
    /// before the shape will be converted into a [`UniqueShape`]
    ///
    /// NOTE: This only applies to [`SharedShape`].
    const TRANSITION_COUNT_MAX: u16 = 1024;

    /// Returns `true` if it's a shared shape, `false` otherwise.
    #[inline]
    #[must_use]
    pub const fn is_shared(&self) -> bool {
        matches!(self.inner, Inner::Shared(_))
    }

    /// Returns `true` if it's a unique shape, `false` otherwise.
    #[inline]
    #[must_use]
    pub const fn is_unique(&self) -> bool {
        matches!(self.inner, Inner::Unique(_))
    }

    pub(crate) const fn as_unique(&self) -> Option<&UniqueShape> {
        if let Inner::Unique(shape) = &self.inner {
            return Some(shape);
        }
        None
    }

    /// Create an insert property transitions returning the new transitioned [`Shape`].
    ///
    /// NOTE: This assumes that there is no property with the given key!
    pub(crate) fn insert_property_transition(&self, key: TransitionKey) -> JsResult<Self> {
        match &self.inner {
            Inner::Shared(shape) => {
                let shape = shape.insert_property_transition(key)?;
                if shape.transition_count() >= Self::TRANSITION_COUNT_MAX {
                    return Ok(shape.to_unique()?.into());
                }
                Ok(shape.into())
            }
            Inner::Unique(shape) => Ok(shape.insert_property_transition(key)?.into()),
        }
    }

    /// Create a change attribute property transitions returning [`ChangeTransition`] containing the new [`Shape`]
    /// and actions to be performed
    ///
    /// NOTE: This assumes that there already is a property with the given key!
    pub(crate) fn change_attributes_transition(
        &self,
        key: TransitionKey,
    ) -> JsResult<ChangeTransition<Self>> {
        match &self.inner {
            Inner::Shared(shape) => {
                let change_transition = shape.change_attributes_transition(key)?;
                let shape =
                    if change_transition.shape.transition_count() >= Self::TRANSITION_COUNT_MAX {
                        change_transition.shape.to_unique()?.into()
                    } else {
                        change_transition.shape.into()
                    };
                Ok(ChangeTransition {
                    shape,
                    action: change_transition.action,
                })
            }
            Inner::Unique(shape) => shape.change_attributes_transition(&key),
        }
    }

    /// Remove a property property from the [`Shape`] returning the new transitioned [`Shape`].
    ///
    /// NOTE: This assumes that there already is a property with the given key!
    pub(crate) fn remove_property_transition(&self, key: &PropertyKey) -> JsResult<Self> {
        match &self.inner {
            Inner::Shared(shape) => {
                let shape = shape.remove_property_transition(key)?;
                if shape.transition_count() >= Self::TRANSITION_COUNT_MAX {
                    return Ok(shape.to_unique()?.into());
                }
                Ok(shape.into())
            }
            Inner::Unique(shape) => Ok(shape.remove_property_transition(key)?.into()),
        }
    }

    /// Create a prototype transitions returning the new transitioned [`Shape`].
    pub(crate) fn change_prototype_transition(&self, prototype: JsPrototype) -> JsResult<Self> {
        match &self.inner {
            Inner::Shared(shape) => {
                let shape = shape.change_prototype_transition(prototype)?;
                if shape.transition_count() >= Self::TRANSITION_COUNT_MAX {
                    return Ok(shape.to_unique()?.into());
                }
                Ok(shape.into())
            }
            Inner::Unique(shape) => Ok(shape.change_prototype_transition(prototype)?.into()),
        }
    }

    /// Get the [`JsPrototype`] of the [`Shape`].
    pub fn prototype(&self) -> JsResult<JsPrototype> {
        match &self.inner {
            Inner::Shared(shape) => shape.prototype().map_err(Into::into),
            Inner::Unique(shape) => shape.prototype(),
        }
    }

    /// Lookup a property in the shape
    #[inline]
    pub(crate) fn lookup(&self, key: &PropertyKey) -> JsResult<Option<Slot>> {
        match &self.inner {
            Inner::Shared(shape) => shape.lookup(key),
            Inner::Unique(shape) => shape.lookup(key),
        }
    }

    /// Returns the keys of the [`Shape`], in insertion order.
    #[inline]
    pub fn keys(&self) -> JsResult<Vec<PropertyKey>> {
        match &self.inner {
            Inner::Shared(shape) => shape.keys(),
            Inner::Unique(shape) => shape.keys(),
        }
    }

    /// Returns this shape's runtime-scoped allocation identity.
    #[inline]
    #[must_use]
    pub fn identity(&self) -> ObjectId {
        match &self.inner {
            Inner::Shared(shape) => shape.identity(),
            Inner::Unique(shape) => shape.identity(),
        }
    }
}

impl From<UniqueShape> for Shape {
    fn from(shape: UniqueShape) -> Self {
        Self {
            inner: Inner::Unique(shape),
        }
    }
}

impl From<SharedShape> for Shape {
    fn from(shape: SharedShape) -> Self {
        Self {
            inner: Inner::Shared(shape),
        }
    }
}

/// Non-owning identity of an object's shape.
#[derive(Debug, Clone, PartialEq, HeapTrace)]
pub(crate) enum ShapeRef {
    Unique(UniqueShapeRef),
    Shared(SharedShapeRef),
}

impl ShapeRef {
    /// Roots the shape if its allocation is still live.
    #[inline]
    pub(crate) fn upgrade(&self) -> Result<Option<Shape>, AccessError> {
        match self {
            ShapeRef::Shared(shape) => Ok(shape.upgrade()?.map(Into::into)),
            ShapeRef::Unique(shape) => Ok(shape.upgrade()?.map(Into::into)),
        }
    }
}

impl From<&Shape> for ShapeRef {
    fn from(value: &Shape) -> Self {
        match &value.inner {
            Inner::Shared(shape) => ShapeRef::Shared(shape.into()),
            Inner::Unique(shape) => ShapeRef::Unique(shape.into()),
        }
    }
}
