//! A Rust API wrapper for the `Set` Builtin ECMAScript Object
use crate::ContextAccess;
use std::ops::Deref;

use crate::{
    JsResult, JsValue,
    builtins::{
        Set, canonicalize_keyed_collection_key, iterable::IteratorHint,
        set::ordered_set::OrderedSet,
    },
    error::{JsNativeError, ObjectAccessError},
    object::{JsFunction, JsObject, JsSetIterator},
    value::TryFromJs,
};

/// `JsSet` provides a wrapper for Boa's implementation of the ECMAScript `Set` object.
#[derive(Debug, Clone)]
pub struct JsSet {
    inner: JsObject,
}

impl JsSet {
    /// Create a new empty set.
    ///
    /// Doesn't matches JavaScript `new Set()` as it doesn't takes an iterator
    /// similar to Rust initialization.
    #[inline]
    pub fn new(context: ContextAccess<'_>) -> Self {
        let context = context.into_owner();
        Self {
            inner: Set::create_from_data(OrderedSet::new(), context).upcast(),
        }
    }

    /// Returns the size of the `Set` as an integer.
    ///
    /// Same as JavaScript's `set.size`.
    #[inline]
    pub fn size(&self) -> Result<usize, ObjectAccessError> {
        Ok(self
            .inner
            .downcast_ref::<OrderedSet>()?
            .ok_or(ObjectAccessError::WrongNativeBrand)?
            .len())
    }

    /// Appends value to the Set object.
    /// Returns the Set object with added value.
    ///
    /// Same as JavaScript's `set.add(value)`.
    pub fn add<T>(&self, value: T, context: ContextAccess<'_>) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        let context = context.into_owner();
        self.add_items(&[value.into()], context.access())
    }

    /// Adds slice as a single element.
    /// Returns the Set object with added slice.
    ///
    /// Same as JavaScript's `set.add(["one", "two", "three"])`
    #[inline]
    pub fn add_items(&self, items: &[JsValue], context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        Set::add(&self.inner.clone().into(), items, context)
    }

    /// Removes all elements from the Set object.
    ///
    /// Same as JavaScript's `set.clear()`.
    #[inline]
    pub fn clear(&self) -> Result<(), ObjectAccessError> {
        self.inner
            .downcast_mut::<OrderedSet>()?
            .ok_or(ObjectAccessError::WrongNativeBrand)?
            .clear();
        Ok(())
    }

    /// Removes the element associated to the value.
    /// Returns a boolean asserting whether an element was
    /// successfully removed or not.
    ///
    /// Same as JavaScript's `set.delete(value)`.
    pub fn delete<T>(&self, value: T) -> Result<bool, ObjectAccessError>
    where
        T: Into<JsValue>,
    {
        let value = canonicalize_keyed_collection_key(value.into());
        Ok(self
            .inner
            .downcast_mut::<OrderedSet>()?
            .ok_or(ObjectAccessError::WrongNativeBrand)?
            .delete(&value))
    }

    /// Returns a boolean asserting whether an element is present
    /// with the given value in the Set object or not.
    ///
    /// Same as JavaScript's `set.has(value)`.
    pub fn has<T>(&self, value: T) -> Result<bool, ObjectAccessError>
    where
        T: Into<JsValue>,
    {
        let value = canonicalize_keyed_collection_key(value.into());
        Ok(self
            .inner
            .downcast_ref::<OrderedSet>()?
            .ok_or(ObjectAccessError::WrongNativeBrand)?
            .contains(&value))
    }

    /// Returns a new iterator object that yields the values
    /// for each element in the Set object in insertion order.
    ///
    /// Same as JavaScript's `set.values()`.
    #[inline]
    pub fn values(&self, context: ContextAccess<'_>) -> JsResult<JsSetIterator> {
        let context = context.into_owner();
        let iterator_object = Set::values(&self.inner.clone().into(), &[JsValue::null()], context)?
            .get_iterator(IteratorHint::Sync, context.access())?;

        JsSetIterator::from_object(iterator_object.iterator().clone())
    }

    /// Alias for `Set.prototype.values()`
    /// Returns a new iterator object that yields the values
    /// for each element in the Set object in insertion order.
    ///
    /// Same as JavaScript's `set.keys()`.
    #[inline]
    pub fn keys(&self, context: ContextAccess<'_>) -> JsResult<JsSetIterator> {
        let context = context.into_owner();
        let iterator_object = Set::values(&self.inner.clone().into(), &[JsValue::null()], context)?
            .get_iterator(IteratorHint::Sync, context.access())?;

        JsSetIterator::from_object(iterator_object.iterator().clone())
    }

    /// Calls callbackFn once for each value present in the Set object,
    /// in insertion order.
    /// Returns `Undefined`.
    ///
    /// Same as JavaScript's `set.forEach(values)`.
    #[inline]
    pub fn for_each(
        &self,
        callback: JsFunction,
        this_arg: JsValue,
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        Set::for_each(
            &self.inner.clone().into(),
            &[callback.into(), this_arg],
            context,
        )
    }

    /// Executes the provided callback function for each key-value pair within the [`JsSet`].
    #[inline]
    pub fn for_each_native<F>(&self, f: F) -> JsResult<()>
    where
        F: FnMut(JsValue) -> JsResult<()>,
    {
        let this = self.inner.clone().into();
        Set::for_each_native(&this, f)
    }

    /// Utility: Creates `JsSet` from `JsObject`, otherwise returns the original object as an `Err`.
    #[inline]
    pub fn from_object(object: JsObject) -> Result<Self, JsObject> {
        if object.is::<OrderedSet>() {
            Ok(Self { inner: object })
        } else {
            Err(object)
        }
    }

    /// Utility: Creates a `JsSet` from a `<IntoIterator<Item = JsValue>` convertible object.
    ///
    /// Returns a `TypeError` if an element is an object from another runtime.
    pub fn from_iter<I>(elements: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = JsValue>,
    {
        let context = context.into_owner();
        let elements = elements.into_iter();

        let inner =
            Set::create_from_data(OrderedSet::with_capacity(elements.size_hint().0), context);

        for element in elements {
            let element = canonicalize_keyed_collection_key(element);
            Set::add_to_data(&inner, element)?;
        }

        Ok(Self {
            inner: inner.upcast(),
        })
    }
}

impl From<JsSet> for JsObject {
    #[inline]
    fn from(o: JsSet) -> Self {
        o.inner.clone()
    }
}

impl From<JsSet> for JsValue {
    #[inline]
    fn from(o: JsSet) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsSet {
    type Target = JsObject;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsSet {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object()
            && let Ok(set) = Self::from_object(o.clone())
        {
            Ok(set)
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a Set object")
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn set_access_refuses_an_alias_without_removing_members() {
        let context = &mut Context::default();
        let set = JsSet::from_iter([JsValue::new(7)], context.access()).unwrap();
        let alias: JsObject = set.clone().into();
        let view = alias.borrow_mut().unwrap();

        assert_eq!(set.size(), Err(ObjectAccessError::Borrowed));
        assert_eq!(set.has(7), Err(ObjectAccessError::Borrowed));
        assert_eq!(set.delete(7), Err(ObjectAccessError::Borrowed));
        assert_eq!(set.clear(), Err(ObjectAccessError::Borrowed));
        drop(view);

        assert_eq!(set.size().unwrap(), 1);
        assert!(set.has(7).unwrap());
        assert!(set.delete(7).unwrap());
        assert_eq!(set.size().unwrap(), 0);
    }
}
