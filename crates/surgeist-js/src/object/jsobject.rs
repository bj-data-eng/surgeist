//! This module implements the `JsObject` structure.
//!
//! The `JsObject` is a garbage collected Object.

use super::{
    IndexedProperties, JsPrototype, NativeObject, Object, ObjectState, PrivateName, PropertyMap,
    internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS},
    shape::{RootShape, UniqueShape, property_table::PropertyTableInner},
};
use crate::{
    Context, ContextAccess, JsError, JsResult, JsString, JsSymbol, JsValue, Runtime,
    builtins::{
        array::ARRAY_EXOTIC_INTERNAL_METHODS,
        array_buffer::{ArrayBuffer, BufferObject, SharedArrayBuffer},
        object::OrdinaryObject,
    },
    class::Class,
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    js_error, js_string,
    property::{PropertyDescriptor, PropertyKey},
    value::PreferredType,
};
use core::ptr::fn_addr_eq;
use std::collections::HashSet;
use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
    result::Result as StdResult,
};
use surgeist_js_gc::heap::{AccessError, ObjectId, RecordRef, Rooted, Trace as HeapTrace};

/// An immutable borrow of an object's common state and typed native payload.
#[derive(Debug)]
pub struct ObjectRef<'a, T: NativeObject + ?Sized> {
    inner: Ref<'a, Object>,
    brand: PhantomData<fn(&T) -> &T>,
}

impl<T: NativeObject + ?Sized> Deref for ObjectRef<'_, T> {
    type Target = ObjectState;

    fn deref(&self) -> &ObjectState {
        &self.inner.state
    }
}

impl<T: NativeObject> ObjectRef<'_, T> {
    /// Borrows the object's native payload.
    #[must_use]
    pub fn data(&self) -> &T {
        self.inner
            .data
            .as_ref()
            .downcast_ref()
            .expect("checked object brand")
    }
}

/// A mutable borrow of an object's common state and typed native payload.
///
/// The guard cannot replace the object's common state, which belongs to the
/// runtime that created it.
///
/// ```compile_fail
/// use surgeist_js::{Context, JsObject};
/// let context = Context::default();
/// let a = JsObject::with_object_proto(context.intrinsics());
/// let b = JsObject::with_object_proto(context.intrinsics());
/// std::mem::swap(&mut *a.borrow_mut(), &mut *b.borrow_mut());
/// ```
///
/// Engine-owned native payloads have their own mutation operations.
///
/// ```compile_fail
/// use surgeist_js::{Context, object::builtins::JsArrayBuffer};
/// let mut context = Context::default();
/// let buffer = JsArrayBuffer::new(8, context.access()).unwrap();
/// let mut object = buffer.borrow_mut();
/// let _ = object.data_mut();
/// ```
#[derive(Debug)]
pub struct ObjectRefMut<'a, T: NativeObject + ?Sized> {
    inner: RefMut<'a, Object>,
    brand: PhantomData<fn(&T) -> &T>,
}

impl<T: NativeObject + ?Sized> Deref for ObjectRefMut<'_, T> {
    type Target = ObjectState;

    fn deref(&self) -> &ObjectState {
        &self.inner.state
    }
}

impl<T: NativeObject + ?Sized> ObjectRefMut<'_, T> {
    pub(crate) fn state_mut(&mut self) -> &mut ObjectState {
        &mut self.inner.state
    }

    pub(crate) fn properties_mut(&mut self) -> &mut PropertyMap {
        self.inner.state.properties_mut()
    }

    pub(crate) fn insert(
        &mut self,
        key: impl Into<PropertyKey>,
        property: impl Into<PropertyDescriptor>,
    ) -> bool {
        self.inner.state.insert(key, property)
    }

    /// Sets the prototype if it belongs to this runtime and the object permits it.
    pub fn set_prototype(&mut self, prototype: impl Into<JsPrototype>) -> bool {
        self.inner.state.set_prototype(prototype)
    }
}

impl<T: NativeObject> ObjectRefMut<'_, T> {
    /// Borrows the object's native payload.
    #[must_use]
    pub fn data(&self) -> &T {
        self.inner
            .data
            .as_ref()
            .downcast_ref()
            .expect("checked object brand")
    }

    /// Mutably borrows the object's native payload.
    #[must_use]
    pub(crate) fn data_mut(&mut self) -> &mut T {
        self.inner
            .data
            .as_mut()
            .downcast_mut()
            .expect("checked object brand")
    }
}

/// Garbage-collected handle to a JavaScript object.
///
/// The native brand can change only through a checked downcast or erasure.
///
/// ```compile_fail
/// use std::marker::PhantomData;
/// use surgeist_js::JsObject;
///
/// type Static = PhantomData<fn(&'static ())>;
/// type AnyLifetime = PhantomData<for<'a> fn(&'a ())>;
/// fn widen(object: JsObject<Static>) -> JsObject<AnyLifetime> {
///     object
/// }
/// ```
pub struct JsObject<T: NativeObject + ?Sized = dyn NativeObject> {
    inner: Rooted<ObjectRecord>,
    brand: PhantomData<fn(&T) -> &T>,
}

/// An allocated object retained by a preparation recipe without a JavaScript handle.
/// Only a matching installed function capability can expose a callable record.
pub(crate) struct UnpublishedObject<T: NativeObject> {
    inner: Rooted<ObjectRecord>,
    brand: PhantomData<fn(&T) -> &T>,
}

impl<T: NativeObject> UnpublishedObject<T> {
    pub(crate) fn from_object(
        runtime: &Runtime,
        object: Object,
        vtable: &'static InternalObjectMethods,
    ) -> JsResult<Self> {
        Ok(Self {
            inner: JsObject::<dyn NativeObject>::allocate(runtime, object, vtable)?,
            brand: PhantomData,
        })
    }
}

impl UnpublishedObject<crate::builtins::function::OrdinaryFunction> {
    pub(crate) fn validate_publication(
        &self,
        publication: &surgeist_js_ast::scope::PublicationPermit<'_>,
    ) -> Result<(), surgeist_js_ast::scope::ScopeStateError> {
        let object = self.inner.object.borrow();
        let function = object
            .data
            .as_ref()
            .downcast_ref::<crate::builtins::function::OrdinaryFunction>()
            .expect("unpublished function retains its native brand");
        publication.authorize_handle(&function.code.scope_association)
    }

    /// Completes one function's scope publication before exposing its record.
    pub(crate) fn publish(
        self,
        publication: surgeist_js_ast::scope::PublicationPermit<'_>,
    ) -> JsResult<JsObject<crate::builtins::function::OrdinaryFunction>> {
        self.validate_publication(&publication)?;
        publication.commit()?;
        Ok(JsObject::with_record(self.inner))
    }

    /// Sets the prototype of a private, fresh function without invoking code.
    pub(crate) fn set_private_prototype(&self, prototype: JsObject) -> JsResult<()> {
        let runtime = Runtime {
            heap: self.inner.runtime().clone(),
        };
        prototype.check_runtime(&runtime)?;
        let changed = self
            .inner
            .object
            .borrow_mut()
            .state
            .set_prototype(Some(prototype));
        assert!(changed, "fresh unpublished functions are extensible");
        Ok(())
    }

    /// Exposes a module recipe under the publication owner's group capability.
    /// The recipe's code and capture were paired before the group could commit.
    pub(crate) fn activate_module(
        self,
        activation: &crate::module::ModuleActivation,
    ) -> JsObject<crate::builtins::function::OrdinaryFunction> {
        {
            let object = self.inner.object.borrow();
            let function = object
                .data
                .as_ref()
                .downcast_ref::<crate::builtins::function::OrdinaryFunction>()
                .expect("unpublished function retains its native brand");
            function
                .code
                .scope_association
                .resolve_live(activation.installed_scopes())
                .expect("module activation installs the recipe's prevalidated code association");
        }
        JsObject::with_record(self.inner)
    }

    /// Links the private prototype without constructing an executable-looking handle.
    pub(crate) fn link_constructor(&self, prototype: &JsObject) -> JsResult<()> {
        let runtime = Runtime {
            heap: self.inner.runtime().clone(),
        };
        prototype.check_runtime(&runtime)?;
        let mut prototype = prototype.borrow_mut();
        let mut slot = super::shape::slot::Slot::new();
        let descriptor = prototype
            .properties
            .get_with_slot(&super::CONSTRUCTOR.into(), &mut slot)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("private function prototype lacks constructor slot")
            })?;
        if !descriptor.is_data_descriptor()
            || !descriptor.value().is_some_and(JsValue::is_undefined)
        {
            return Err(JsNativeError::typ()
                .with_message("private function prototype constructor slot is not empty")
                .into());
        }
        prototype
            .properties_mut()
            .write_named_object_edge(slot.index as usize, ObjectEdge(self.inner.record_ref()));
        Ok(())
    }

    /// Exposes the already allocated callable after its exact code association installs.
    pub(crate) fn activate(
        self,
        code: crate::vm::ActivatedCode,
    ) -> JsResult<JsObject<crate::builtins::function::OrdinaryFunction>> {
        let matches = {
            let object = self.inner.object.borrow();
            let function = object
                .data
                .as_ref()
                .downcast_ref::<crate::builtins::function::OrdinaryFunction>()
                .expect("unpublished function retains its native brand");
            Rc::ptr_eq(&function.code, code.code())
        };
        if !matches {
            return Err(crate::error::EngineError::InvalidEntryPermit.into());
        }
        Ok(JsObject::with_record(self.inner))
    }
}

impl<T: NativeObject + ?Sized> Clone for JsObject<T> {
    fn clone(&self) -> Self {
        Self::with_record(self.inner.clone())
    }
}

/// Strong record reference stored inside the JavaScript object graph.
#[derive(Clone, Debug, Eq, PartialEq, HeapTrace)]
pub(crate) struct ObjectEdge(RecordRef<ObjectRecord>);

impl ObjectEdge {
    pub(crate) const fn identity(&self) -> ObjectId {
        self.0.identity()
    }

    pub(crate) fn is_live(&self) -> Result<bool, AccessError> {
        self.0.is_live()
    }

    pub(crate) fn upgrade(&self) -> Result<Option<JsObject>, AccessError> {
        self.0.upgrade().map(|root| root.map(JsObject::with_record))
    }
}

/// Shared allocation for typed and erased handles to the same object.
#[derive(HeapTrace)]
pub(crate) struct ObjectRecord {
    #[heap_trace(skip)]
    vtable: &'static InternalObjectMethods,
    #[heap_trace(skip)]
    native_type: TypeId,
    object: RefCell<Object>,
    #[heap_trace(skip)]
    retirement: RetirementWatchers,
}

#[derive(Default)]
struct RetirementWatchers(RefCell<Vec<Weak<dyn Fn()>>>);

impl Drop for RetirementWatchers {
    fn drop(&mut self) {
        for watcher in self.0.get_mut().iter().filter_map(Weak::upgrade) {
            watcher();
        }
    }
}

impl Debug for ObjectRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRecord")
            .field("native_type", &self.native_type)
            .finish_non_exhaustive()
    }
}

impl ObjectRecord {
    fn new(object: Object, vtable: &'static InternalObjectMethods) -> Self {
        let native_type = (object.data.as_ref() as &dyn Any).type_id();
        Self {
            vtable,
            native_type,
            object: RefCell::new(object),
            retirement: RetirementWatchers::default(),
        }
    }
}

impl JsObject {
    /// Creates a new ordinary object with its prototype set to the `Object` prototype.
    ///
    /// This is an alias for [`Self::with_object_proto`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::default();
    /// let obj = JsObject::default(context.intrinsics());
    ///
    /// assert!(obj.is_ordinary());
    /// ```
    #[inline]
    #[must_use]
    pub fn default(intrinsics: &Intrinsics) -> Self {
        Self::with_object_proto(intrinsics)
    }

    /// Creates a new `JsObject` from its inner object and its vtable.
    pub(crate) fn from_object_and_vtable(
        runtime: &Runtime,
        object: Object,
        vtable: &'static InternalObjectMethods,
    ) -> Self {
        Self::try_from_object_and_vtable(runtime, object, vtable)
            .expect("internal object template has valid heap edges")
    }

    pub(crate) fn try_from_object_and_vtable(
        runtime: &Runtime,
        object: Object,
        vtable: &'static InternalObjectMethods,
    ) -> JsResult<Self> {
        Self::allocate(runtime, object, vtable).map(Self::with_record)
    }

    pub(crate) fn from_inner(inner: ObjectEdge) -> Self {
        inner
            .upgrade()
            .expect("object heap is available")
            .expect("live owner retains its object")
    }

    pub(crate) fn ordinary_prototype(
        runtime: &Runtime,
        vtable: &'static InternalObjectMethods,
    ) -> Self {
        JsObject::<OrdinaryObject>::new_unique_with_vtable(runtime, None, OrdinaryObject, |_| {
            vtable
        })
        .expect("ordinary prototype has no external heap edges")
        .upcast()
    }

    /// Creates a new ordinary object with its prototype set to the `Object` prototype.
    ///
    /// This is equivalent to calling the specification's abstract operation
    /// [`OrdinaryObjectCreate(%Object.prototype%)`][call].
    ///
    /// [call]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::default();
    /// let obj = JsObject::with_object_proto(context.intrinsics());
    ///
    /// assert!(obj.is_ordinary());
    /// assert!(obj.prototype().is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn with_object_proto(intrinsics: &Intrinsics) -> Self {
        Self::from_proto_and_data(
            &intrinsics.runtime(),
            intrinsics.constructors().object().prototype(),
            OrdinaryObject,
        )
        .expect("ordinary object uses its runtime's intrinsic prototype")
    }

    /// Creates a new ordinary object, with its prototype set to null.
    ///
    /// This is equivalent to calling the specification's abstract operation
    /// [`OrdinaryObjectCreate(null)`][call].
    ///
    /// [call]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsObject, Runtime};
    /// let obj = JsObject::with_null_proto(&Runtime::default());
    /// assert!(obj.prototype().is_none());
    /// assert!(obj.is_ordinary());
    /// ```
    #[inline]
    #[must_use]
    pub fn with_null_proto(runtime: &Runtime) -> Self {
        Self::from_proto_and_data(runtime, None, OrdinaryObject)
            .expect("null-prototype ordinary object has no external heap edges")
    }

    /// Creates a new object with the provided prototype and object data.
    ///
    /// This is equivalent to calling the specification's abstract operation [`OrdinaryObjectCreate`],
    /// with the difference that the `additionalInternalSlotsList` parameter is determined by
    /// the provided `data`.
    ///
    /// [`OrdinaryObjectCreate`]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the prototype belongs to another runtime or the
    /// heap rejects an edge reported by the native payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let context = &mut Context::default();
    /// let obj = JsObject::from_proto_and_data(
    ///     context.runtime(),
    ///     context.intrinsics().constructors().object().prototype(),
    ///     OrdinaryObject,
    /// )
    /// .unwrap();
    ///
    /// assert!(obj.is_ordinary());
    /// assert!(obj.prototype().is_some());
    ///
    /// // Create an object with no prototype.
    /// let null_obj =
    ///     JsObject::from_proto_and_data(context.runtime(), None, OrdinaryObject)
    ///         .unwrap();
    /// assert!(null_obj.prototype().is_none());
    /// ```
    pub fn from_proto_and_data<O: Into<Option<Self>>, T: NativeObject>(
        runtime: &Runtime,
        prototype: O,
        data: T,
    ) -> JsResult<Self> {
        JsObject::new_unique(runtime, prototype, data).map(JsObject::upcast)
    }

    /// Creates a new object with the provided prototype and object data.
    ///
    /// This is equivalent to calling the specification's abstract operation [`OrdinaryObjectCreate`],
    /// with the difference that the `additionalInternalSlotsList` parameter is determined by
    /// the provided `data`.
    ///
    /// [`OrdinaryObjectCreate`]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    pub(crate) fn from_proto_and_data_with_shared_shape<O: Into<Option<Self>>, T: NativeObject>(
        root_shape: &RootShape,
        prototype: O,
        data: T,
    ) -> JsObject<T> {
        JsObject::new(root_shape, prototype, data).expect("internal object has valid heap edges")
    }

    /// Downcasts the object's inner data if the object is of type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsData, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// #[derive(Debug, JsData, surgeist_js::gc::heap::Trace)]
    /// struct CustomStruct;
    ///
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Downcast consumes the object on success.
    /// let typed = obj.downcast::<OrdinaryObject>();
    /// assert!(typed.is_ok());
    ///
    /// // Downcast fails for a wrong type, returning the original object.
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    /// let result = obj.downcast::<CustomStruct>();
    /// assert!(result.is_err());
    /// ```
    pub fn downcast<T: NativeObject>(self) -> Result<JsObject<T>, Self> {
        if self.is::<T>() {
            Ok(JsObject::with_record(self.inner))
        } else {
            Err(self)
        }
    }

    /// Downcasts a reference to the object,
    /// if the object is of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsData, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// #[derive(Debug, JsData, surgeist_js::gc::heap::Trace)]
    /// struct CustomStruct;
    ///
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Downcast ref succeeds for the correct type.
    /// assert!(obj.downcast_ref::<OrdinaryObject>().is_some());
    ///
    /// // Returns `None` for a wrong type.
    /// assert!(obj.downcast_ref::<CustomStruct>().is_none());
    /// ```
    #[must_use]
    #[track_caller]
    pub fn downcast_ref<T: NativeObject>(&self) -> Option<Ref<'_, T>> {
        if !self.is::<T>() {
            return None;
        }
        Ref::filter_map(self.borrow().inner, |object| {
            object.data.as_ref().downcast_ref()
        })
        .ok()
    }

    /// Mutably borrows the payload of a host-defined class.
    ///
    /// Returns `None` if the object has a different native brand.
    /// The caller must preserve the heap edges described by [`Class`].
    ///
    /// # Panics
    ///
    /// Panics if the object is currently borrowed.
    ///
    /// Engine-owned payloads cannot be borrowed mutably through this method.
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, JsObject, builtins::array_buffer::ArrayBuffer};
    /// use surgeist_js::object::builtins::JsArrayBuffer;
    /// let mut context = Context::default();
    /// let object: JsObject = JsArrayBuffer::new(8, context.access()).unwrap().into();
    /// let _ = object.downcast_class_mut::<ArrayBuffer>();
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, JsObject, builtins::array_buffer::ArrayBuffer};
    /// use surgeist_js::object::builtins::JsArrayBuffer;
    /// let mut context = Context::default();
    /// let object: JsObject = JsArrayBuffer::new(8, context.access()).unwrap().into();
    /// let _ = object.downcast_mut::<ArrayBuffer>();
    /// ```
    #[must_use]
    #[track_caller]
    pub fn downcast_class_mut<C: Class>(&self) -> Option<RefMut<'_, C>> {
        self.downcast_mut()
    }

    #[must_use]
    #[track_caller]
    pub(crate) fn downcast_mut<T: NativeObject>(&self) -> Option<RefMut<'_, T>> {
        if !self.is::<T>() {
            return None;
        }
        RefMut::filter_map(self.borrow_mut().inner, |object| {
            object.data.as_mut().downcast_mut()
        })
        .ok()
    }

    /// Checks if this object is an instance of a certain `NativeObject`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{JsData, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// #[derive(Debug, JsData, surgeist_js::gc::heap::Trace)]
    /// struct CustomStruct;
    ///
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// assert!(obj.is::<OrdinaryObject>());
    /// assert!(!obj.is::<CustomStruct>());
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn is<T: NativeObject>(&self) -> bool {
        self.inner.native_type == TypeId::of::<T>()
    }

    /// Checks if it's an ordinary object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::default();
    /// let obj = JsObject::with_object_proto(context.intrinsics());
    ///
    /// assert!(obj.is_ordinary());
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn is_ordinary(&self) -> bool {
        self.is::<OrdinaryObject>()
    }

    /// Checks if it's an `Array` object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject, JsValue ,JsResult};
    /// # use surgeist_js::object::builtins::JsArray;
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    ///
    /// let array = JsArray::new(context.access())?;
    /// // A JsArray's inner JsObject is an array.
    /// assert!(JsObject::from(array).is_array());
    ///
    /// // An ordinary object is not an array.
    /// let obj = JsObject::with_object_proto(context.intrinsics());
    /// assert!(!obj.is_array());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn is_array(&self) -> bool {
        std::ptr::eq(self.vtable(), &raw const ARRAY_EXOTIC_INTERNAL_METHODS)
    }

    /// The inner implementation of `deep_strict_equals`, which keeps a list of values we've
    /// seen to avoid recursive objects.
    pub(crate) fn deep_strict_equals_inner(
        lhs: &Self,
        rhs: &Self,
        encounters: &mut HashSet<ObjectId>,
        context: &mut Context,
    ) -> JsResult<bool> {
        // Loop through all the keys and if one is not equal, return false.
        fn key_loop(
            lhs: &JsObject,
            rhs: &JsObject,
            encounters: &mut HashSet<ObjectId>,
            context: &mut Context,
        ) -> JsResult<bool> {
            let l_keys = lhs.own_property_keys(context.access())?;
            let r_keys = rhs.own_property_keys(context.access())?;

            if l_keys.len() != r_keys.len() {
                return Ok(false);
            }

            for key in &l_keys {
                let vl = lhs.get_property(key);
                let vr = rhs.get_property(key);

                match (vl, vr) {
                    (None, None) => {}
                    (Some(vl), Some(vr)) => match (vl.value(), vr.value()) {
                        (None, None) => {}
                        (Some(lv), Some(rv)) => {
                            if !lv.deep_strict_equals_inner(rv, encounters, context)? {
                                return Ok(false);
                            }
                        }
                        _ => {
                            return Ok(false);
                        }
                    },
                    _ => {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }

        let addr_l = lhs.identity();
        let addr_r = rhs.identity();

        if addr_r == addr_l {
            return Ok(true);
        }

        let contains_l = encounters.contains(&addr_l);
        let contains_r = encounters.contains(&addr_r);
        if contains_l || contains_r {
            return Ok(false);
        }

        encounters.insert(addr_l);
        encounters.insert(addr_r);

        // Make sure we clean up after the recursion.
        let result = key_loop(lhs, rhs, encounters, context);
        encounters.remove(&addr_l);
        encounters.remove(&addr_r);
        result
    }

    /// Checks that all own property keys and values are equal (recursively).
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject, JsResult, js_string};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    ///
    /// let obj1 = JsObject::with_object_proto(context.intrinsics());
    /// obj1.set(js_string!("key"), 42, false, context.access())?;
    ///
    /// let obj2 = JsObject::with_object_proto(context.intrinsics());
    /// obj2.set(js_string!("key"), 42, false, context.access())?;
    ///
    /// assert!(JsObject::deep_strict_equals(&obj1, &obj2, context.access())?);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn deep_strict_equals(
        lhs: &Self,
        rhs: &Self,
        context: ContextAccess<'_>,
    ) -> JsResult<bool> {
        let context = context.into_owner();
        Self::deep_strict_equals_inner(lhs, rhs, &mut HashSet::new(), context)
    }

    /// The abstract operation `ToPrimitive` takes an input argument and an optional argument
    /// `PreferredType`.
    ///
    /// <https://tc39.es/ecma262/#sec-toprimitive>
    pub fn to_primitive(
        &self,
        context: ContextAccess<'_>,
        preferred_type: PreferredType,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        let Some(exotic_to_prim) = self.get_method(JsSymbol::to_primitive(), context)? else {
            // c. If preferredType is not present, let preferredType be number.
            let preferred_type = match preferred_type {
                PreferredType::Default | PreferredType::Number => PreferredType::Number,
                PreferredType::String => PreferredType::String,
            };
            return self.ordinary_to_primitive(context, preferred_type);
        };

        // b. If exoticToPrim is not undefined, then
        //    i. If preferredType is not present, let hint be "default".
        //    ii. Else if preferredType is string, let hint be "string".
        //    iii. Else,
        //        1. Assert: preferredType is number.
        //        2. Let hint be "number".
        let hint = match preferred_type {
            PreferredType::Default => js_string!("default"),
            PreferredType::String => js_string!("string"),
            PreferredType::Number => js_string!("number"),
        }
        .into();

        //    iv. Let result be ? Call(exoticToPrim, input, « hint »).
        let result = exotic_to_prim.call(&self.clone().into(), &[hint], context.access())?;

        //    v. If Type(result) is not Object, return result.
        //    vi. Throw a TypeError exception.
        if result.is_object() {
            Err(js_error!(
                TypeError: "method `[Symbol.toPrimitive]` cannot return an object"
            ))
        } else {
            Ok(result)
        }
    }

    /// Converts an object to a primitive.
    ///
    /// Diverges from the spec to prevent a stack overflow when the object is recursive.
    /// For example,
    /// ```javascript
    /// let a = [1];
    /// a[1] = a;
    /// console.log(a.toString()); // We print "1,"
    /// ```
    /// The spec doesn't mention what to do in this situation, but a naive implementation
    /// would overflow the stack recursively calling `toString()`. We follow v8 and SpiderMonkey
    /// instead by returning a default value for the given `hint` -- either `0.` or `""`.
    /// Example in v8: <https://repl.it/repls/IvoryCircularCertification#index.js>
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinarytoprimitive
    pub(crate) fn ordinary_to_primitive(
        &self,
        context: &mut Context,
        hint: PreferredType,
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(O) is Object.
        //      Already is JsObject by type.
        // 2. Assert: Type(hint) is String and its value is either "string" or "number".
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);

        // Diverge from the spec here to make sure we aren't going to overflow the stack by converting
        // a recursive structure
        // We can follow v8 & SpiderMonkey's lead and return a default value for the hint in this situation
        // (see https://repl.it/repls/IvoryCircularCertification#index.js)
        let recursion_limiter = RecursionLimiter::new(&self.inner.object);
        if recursion_limiter.live {
            // we're in a recursive object, bail
            return Ok(match hint {
                PreferredType::Number => JsValue::new(0),
                PreferredType::String => JsValue::new(js_string!()),
                PreferredType::Default => unreachable!("checked type hint in step 2"),
            });
        }

        // 3. If hint is "string", then
        //    a. Let methodNames be « "toString", "valueOf" ».
        // 4. Else,
        //    a. Let methodNames be « "valueOf", "toString" ».
        let method_names = if hint == PreferredType::String {
            [js_string!("toString"), js_string!("valueOf")]
        } else {
            [js_string!("valueOf"), js_string!("toString")]
        };

        // 5. For each name in methodNames in List order, do
        for name in method_names {
            // a. Let method be ? Get(O, name).
            let method = self.get(name, context.access())?;

            // b. If IsCallable(method) is true, then
            if let Some(method) = method.as_callable() {
                // i. Let result be ? Call(method, O).
                let result = method.call(&self.clone().into(), &[], context.access())?;

                // ii. If Type(result) is not Object, return result.
                if !result.is_object() {
                    return Ok(result);
                }
            }
        }

        // 6. Throw a TypeError exception.
        Err(JsNativeError::typ()
            .with_message("cannot convert object to primitive value")
            .into())
    }

    /// The abstract operation `ToPropertyDescriptor`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-topropertydescriptor
    pub fn to_property_descriptor(
        &self,
        context: ContextAccess<'_>,
    ) -> JsResult<PropertyDescriptor> {
        let context = context.into_owner();
        // 1 is implemented on the method `to_property_descriptor` of value

        // 2. Let desc be a new Property Descriptor that initially has no fields.
        let mut desc = PropertyDescriptor::builder();

        // 3. Let hasEnumerable be ? HasProperty(Obj, "enumerable").
        // 4. If hasEnumerable is true, then ...
        if let Some(enumerable) = self.try_get(js_string!("enumerable"), context)? {
            // a. Let enumerable be ! ToBoolean(? Get(Obj, "enumerable")).
            // b. Set desc.[[Enumerable]] to enumerable.
            desc = desc.enumerable(enumerable.to_boolean());
        }

        // 5. Let hasConfigurable be ? HasProperty(Obj, "configurable").
        // 6. If hasConfigurable is true, then ...
        if let Some(configurable) = self.try_get(js_string!("configurable"), context)? {
            // a. Let configurable be ! ToBoolean(? Get(Obj, "configurable")).
            // b. Set desc.[[Configurable]] to configurable.
            desc = desc.configurable(configurable.to_boolean());
        }

        // 7. Let hasValue be ? HasProperty(Obj, "value").
        // 8. If hasValue is true, then ...
        if let Some(value) = self.try_get(js_string!("value"), context)? {
            // a. Let value be ? Get(Obj, "value").
            // b. Set desc.[[Value]] to value.
            desc = desc.value(value);
        }

        // 9. Let hasWritable be ? HasProperty(Obj, ).
        // 10. If hasWritable is true, then ...
        if let Some(writable) = self.try_get(js_string!("writable"), context)? {
            // a. Let writable be ! ToBoolean(? Get(Obj, "writable")).
            // b. Set desc.[[Writable]] to writable.
            desc = desc.writable(writable.to_boolean());
        }

        // 11. Let hasGet be ? HasProperty(Obj, "get").
        // 12. If hasGet is true, then
        // 12.a. Let getter be ? Get(Obj, "get").
        let get = if let Some(getter) = self.try_get(js_string!("get"), context)? {
            // b. If IsCallable(getter) is false and getter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !getter.is_undefined() && getter.as_object().is_none_or(|o| !o.is_callable()) {
                return Err(JsNativeError::typ()
                    .with_message("Property descriptor getter must be callable")
                    .into());
            }
            // c. Set desc.[[Get]] to getter.
            Some(getter)
        } else {
            None
        };

        // 13. Let hasSet be ? HasProperty(Obj, "set").
        // 14. If hasSet is true, then
        // 14.a. Let setter be ? Get(Obj, "set").
        let set = if let Some(setter) = self.try_get(js_string!("set"), context)? {
            // 14.b. If IsCallable(setter) is false and setter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !setter.is_undefined() && setter.as_object().is_none_or(|o| !o.is_callable()) {
                return Err(JsNativeError::typ()
                    .with_message("Property descriptor setter must be callable")
                    .into());
            }
            // 14.c. Set desc.[[Set]] to setter.
            Some(setter)
        } else {
            None
        };

        // 15. If desc.[[Get]] is present or desc.[[Set]] is present, then ...
        // a. If desc.[[Value]] is present or desc.[[Writable]] is present, throw a TypeError exception.
        if get.as_ref().or(set.as_ref()).is_some() && desc.inner().is_data_descriptor() {
            return Err(JsNativeError::typ()
                .with_message(
                    "Invalid property descriptor.\
Cannot both specify accessors and a value or writable attribute",
                )
                .into());
        }

        desc = desc.maybe_get(get).maybe_set(set);

        // 16. Return desc.
        Ok(desc.build())
    }

    // Allow lint, false positive.
    #[allow(clippy::assigning_clones)]
    pub(crate) fn get_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let mut obj = Some(self.clone());

        while let Some(o) = obj {
            if let Some(v) = o.borrow().properties.get(key) {
                return Some(v);
            }
            obj = o.borrow().prototype().clone();
        }
        None
    }

    /// Casts to a `BufferObject` if the object is an `ArrayBuffer` or a `SharedArrayBuffer`.
    #[inline]
    pub(crate) fn into_buffer_object(self) -> Result<BufferObject, JsObject> {
        match self.downcast::<ArrayBuffer>() {
            Ok(buffer) => Ok(BufferObject::Buffer(buffer)),
            Err(object) => object
                .downcast::<SharedArrayBuffer>()
                .map(BufferObject::SharedBuffer),
        }
    }
}

impl<T: NativeObject + ?Sized> JsObject<T> {
    /// Upcasts this object's inner data from a specific type `T` to an erased type
    /// `dyn NativeObject`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// // Create a typed JsObject<OrdinaryObject>.
    /// let typed_obj =
    ///     JsObject::new_unique(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Upcast erases the type, producing an untyped JsObject.
    /// let obj: JsObject = typed_obj.upcast();
    /// assert!(obj.is_ordinary());
    /// ```
    #[must_use]
    pub fn upcast(self) -> JsObject {
        JsObject::with_record(self.inner)
    }

    fn with_record(inner: Rooted<ObjectRecord>) -> Self {
        Self {
            inner,
            brand: PhantomData,
        }
    }

    /// Immutably borrows the `Object`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Multiple immutable borrows are allowed.
    /// let borrowed = obj.borrow();
    /// assert!(borrowed.prototype().is_none());
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn borrow(&self) -> ObjectRef<'_, T> {
        self.try_borrow().expect("Object already mutably borrowed")
    }

    /// Mutably borrows the Object.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    /// The object cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    /// Panics if the object is currently borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let context = &mut Context::default();
    /// let obj = JsObject::from_proto_and_data(
    ///     context.runtime(),
    ///     context.intrinsics().constructors().object().prototype(),
    ///     OrdinaryObject,
    /// )
    /// .unwrap();
    ///
    /// // Set the prototype to `None` via a mutable borrow.
    /// obj.borrow_mut().set_prototype(None);
    /// assert!(obj.prototype().is_none());
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn borrow_mut(&self) -> ObjectRefMut<'_, T> {
        self.try_borrow_mut().expect("Object already borrowed")
    }

    /// Immutably borrows the `Object`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts until the returned `ObjectRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Non-panicking immutable borrow.
    /// let result = obj.try_borrow();
    /// assert!(result.is_ok());
    /// ```
    #[inline]
    pub fn try_borrow(&self) -> StdResult<ObjectRef<'_, T>, BorrowError> {
        self.inner
            .object
            .try_borrow()
            .map(|inner| ObjectRef {
                inner,
                brand: PhantomData,
            })
            .map_err(|_| BorrowError)
    }

    /// Mutably borrows the object, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `ObjectRefMut` exits scope.
    /// The object be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Non-panicking mutable borrow.
    /// let result = obj.try_borrow_mut();
    /// assert!(result.is_ok());
    /// ```
    #[inline]
    pub fn try_borrow_mut(&self) -> StdResult<ObjectRefMut<'_, T>, BorrowMutError> {
        self.inner
            .object
            .try_borrow_mut()
            .map(|inner| ObjectRefMut {
                inner,
                brand: PhantomData,
            })
            .map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let obj =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    /// let clone = obj.clone();
    ///
    /// // A clone points to the same GC allocation.
    /// assert!(JsObject::equals(&obj, &clone));
    ///
    /// // A separate object is different, even with identical data.
    /// let other =
    ///     JsObject::from_proto_and_data(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    /// assert!(!JsObject::equals(&obj, &other));
    /// ```
    #[must_use]
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        lhs.identity() == rhs.identity()
    }

    /// Get the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::default();
    ///
    /// let obj = JsObject::with_object_proto(context.intrinsics());
    /// assert!(obj.prototype().is_some());
    ///
    /// let null_obj = JsObject::with_null_proto(context.runtime());
    /// assert!(null_obj.prototype().is_none());
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn prototype(&self) -> JsPrototype {
        self.borrow().prototype()
    }

    /// Get the extensibility of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    pub(crate) fn extensible(&self) -> bool {
        self.borrow().extensible
    }

    /// Set the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::default();
    /// let obj = JsObject::with_object_proto(context.intrinsics());
    ///
    /// assert!(obj.prototype().is_some());
    ///
    /// // Set the prototype to `None`.
    /// obj.set_prototype(None);
    /// assert!(obj.prototype().is_none());
    /// ```
    #[inline]
    #[track_caller]
    #[allow(clippy::must_use_candidate)]
    pub fn set_prototype(&self, prototype: JsPrototype) -> bool {
        self.borrow_mut().set_prototype(prototype)
    }

    /// Helper function for property insertion.
    #[track_caller]
    pub(crate) fn insert<K, P>(&self, key: K, property: P) -> bool
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.borrow_mut().insert(key, property)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// Missing descriptor fields take their ECMAScript defaults.
    /// Returns whether the key previously existed. A value, getter or setter
    /// from a different runtime is rejected before insertion.
    pub fn insert_property<K, P>(&self, key: K, property: P) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.runtime().check_descriptor(&property)?;
        Ok(self.insert(key, property.complete_property_descriptor()))
    }

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    #[must_use]
    pub fn is_callable(&self) -> bool {
        !fn_addr_eq(
            self.inner.vtable.__call__,
            ORDINARY_INTERNAL_METHODS.__call__,
        )
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    #[must_use]
    pub fn is_constructor(&self) -> bool {
        !fn_addr_eq(
            self.inner.vtable.__construct__,
            ORDINARY_INTERNAL_METHODS.__construct__,
        )
    }

    pub(crate) fn vtable(&self) -> &'static InternalObjectMethods {
        self.inner.vtable
    }

    /// Observes this allocation's retirement. The caller owns the registration.
    pub(crate) fn observe_retirement(&self, watcher: &Rc<dyn Fn()>) {
        let mut watchers = self.inner.retirement.0.borrow_mut();
        if watchers.len() == watchers.capacity() {
            watchers.retain(|watcher| watcher.strong_count() != 0);
        }
        watchers.push(Rc::downgrade(watcher));
    }

    pub(crate) fn check_runtime(&self, runtime: &Runtime) -> JsResult<()> {
        if !self.inner.runtime().same_heap(&runtime.heap) {
            return Err(JsNativeError::typ()
                .with_message("object belongs to a different runtime")
                .into());
        }
        Ok(())
    }

    pub(crate) fn runtime(&self) -> Runtime {
        Runtime {
            heap: self.inner.runtime().clone(),
        }
    }

    pub(crate) fn edge(&self) -> ObjectEdge {
        ObjectEdge(self.inner.record_ref())
    }

    pub(crate) fn into_record(self) -> ObjectEdge {
        self.edge()
    }

    /// Returns an opaque identity for this allocation.
    ///
    /// Every handle to the object has this identity. Keeping the identity
    /// does not keep the object alive. Later allocations have different identities.
    #[must_use]
    pub fn identity(&self) -> ObjectId {
        self.inner.handle().erase()
    }

    fn allocate(
        runtime: &Runtime,
        object: Object,
        vtable: &'static InternalObjectMethods,
    ) -> JsResult<Rooted<ObjectRecord>> {
        runtime
            .heap
            .allocate(ObjectRecord::new(object, vtable))
            .map_err(|rejected| object_allocation_error(rejected.reason))
    }

    /// Create a new private name with this object as the unique identifier.
    pub(crate) fn private_name(&self, description: JsString) -> PrivateName {
        PrivateName::new(description, self.identity())
    }
}

fn object_allocation_error(reason: AccessError) -> JsError {
    JsNativeError::typ().with_message(reason.to_string()).into()
}

impl<T: NativeObject> JsObject<T> {
    /// Creates a new `JsObject` from its root shape, prototype, and data.
    ///
    /// Note that the returned object will not be erased to be convertible to a
    /// `JsValue`. To erase the native brand, call [`JsObject::upcast`].
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the prototype belongs to another runtime or the
    /// heap rejects an edge reported by the native payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let context = &mut Context::default();
    ///
    /// let typed_obj = JsObject::new(
    ///     context.root_shape(),
    ///     context.intrinsics().constructors().object().prototype(),
    ///     OrdinaryObject,
    /// )
    /// .unwrap();
    ///
    /// // Upcast to an erased JsObject to use with JsValue.
    /// let obj = typed_obj.upcast();
    /// assert!(obj.is_ordinary());
    /// ```
    pub fn new<O: Into<Option<JsObject>>>(
        root_shape: &RootShape,
        prototype: O,
        data: T,
    ) -> JsResult<Self> {
        let runtime = root_shape.runtime();
        runtime
            .heap
            .with_pending_edges(data, |data| {
                let internal_methods = data.internal_methods();
                let shape = root_shape
                    .shape()
                    .try_change_prototype_transition(prototype.into())
                    .map_err(object_allocation_error)?
                    .into();
                let object =
                    Object::new(PropertyMap::new(&shape, IndexedProperties::default()), data);
                Self::allocate(runtime, object, internal_methods).map(Self::with_record)
            })
            .map_err(object_allocation_error)?
    }

    /// Creates a new `JsObject` with a unique shape from a prototype and native data.
    ///
    /// Note that the returned object will not be erased to be convertible to a
    /// `JsValue`. To erase the native brand, call [`JsObject::upcast`].
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the prototype belongs to another runtime or the
    /// heap rejects an edge reported by the native payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::JsObject;
    /// # use surgeist_js::builtins::object::OrdinaryObject;
    /// let typed_obj =
    ///     JsObject::new_unique(&surgeist_js::Runtime::default(), None, OrdinaryObject)
    ///         .unwrap();
    ///
    /// // Upcast to an erased JsObject.
    /// let obj = typed_obj.upcast();
    /// assert!(obj.is_ordinary());
    /// assert!(obj.prototype().is_none());
    /// ```
    pub fn new_unique<O: Into<Option<JsObject>>>(
        runtime: &Runtime,
        prototype: O,
        data: T,
    ) -> JsResult<Self> {
        Self::new_unique_with_vtable(runtime, prototype, data, T::internal_methods)
    }

    fn new_unique_with_vtable(
        runtime: &Runtime,
        prototype: impl Into<JsPrototype>,
        data: T,
        vtable: impl FnOnce(&T) -> &'static InternalObjectMethods,
    ) -> JsResult<Self> {
        runtime
            .heap
            .with_pending_edges(data, |data| {
                let vtable = vtable(&data);
                let shape = UniqueShape::try_new(
                    &runtime.heap,
                    prototype.into(),
                    PropertyTableInner::default(),
                )
                .map_err(object_allocation_error)?
                .into();
                let object =
                    Object::new(PropertyMap::new(&shape, IndexedProperties::default()), data);
                Self::allocate(runtime, object, vtable).map(Self::with_record)
            })
            .map_err(object_allocation_error)?
    }

    pub(crate) fn borrow_data(&self) -> Ref<'_, T> {
        Ref::map(self.borrow().inner, |object| {
            object
                .data
                .as_ref()
                .downcast_ref()
                .expect("checked object brand")
        })
    }

    pub(crate) fn borrow_data_mut(&self) -> RefMut<'_, T> {
        RefMut::map(self.borrow_mut().inner, |object| {
            object
                .data
                .as_mut()
                .downcast_mut()
                .expect("checked object brand")
        })
    }
}

impl From<ObjectEdge> for JsObject {
    #[inline]
    fn from(inner: ObjectEdge) -> Self {
        Self::from_inner(inner)
    }
}

impl<T: NativeObject + ?Sized> PartialEq for JsObject<T> {
    fn eq(&self, other: &Self) -> bool {
        Self::equals(self, other)
    }
}

impl<T: NativeObject + ?Sized> Eq for JsObject<T> {}

impl<T: NativeObject + ?Sized> Hash for JsObject<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identity().hash(state);
    }
}

/// An error returned by [`JsObject::try_borrow`](struct.JsObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

impl Error for BorrowError {}

/// An error returned by [`JsObject::try_borrow_mut`](struct.JsObject.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already borrowed", f)
    }
}

impl Error for BorrowMutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RecursionValueState {
    /// This value is "live": there's an active `RecursionLimiter` that hasn't been dropped.
    Live,
    /// This value has been seen before, but the recursion limiter has been dropped.
    /// For example:
    /// ```javascript
    /// let b = [];
    /// JSON.stringify([ // Create a recursion limiter for the root here
    ///    b,            // state for b's &JsObject here is None
    ///    b,            // state for b's &JsObject here is Visited
    /// ]);
    /// ```
    Visited,
}

/// Prevents infinite recursion during `Debug::fmt`, `JSON.stringify`, and other conversions.
/// This uses a thread local, so is not safe to use where the object graph will be traversed by
/// multiple threads!
#[derive(Debug)]
pub struct RecursionLimiter {
    /// If this was the first `JsObject` in the tree.
    top_level: bool,
    /// The ptr being kept in the `HashSet`, so we can delete it when we drop.
    ptr: usize,
    /// If this `JsObject` has been visited before in the graph, but not in the current branch.
    pub visited: bool,
    /// If this `JsObject` has been visited in the current branch of the graph.
    pub live: bool,
}

impl Drop for RecursionLimiter {
    fn drop(&mut self) {
        if self.top_level {
            // When the top level of the graph is dropped, we can free the entire map for the next traversal.
            SEEN.with(|hm| hm.borrow_mut().clear());
        } else if !self.live {
            // This was the first RL for this object to become live, so it's no longer live now that it's dropped.
            SEEN.with(|hm| {
                hm.borrow_mut()
                    .insert(self.ptr, RecursionValueState::Visited)
            });
        }
    }
}

thread_local! {
    /// The map of pointers to `JsObject` that have been visited during the current `Debug::fmt` graph,
    /// and the current state of their RecursionLimiter (dropped or live -- see `RecursionValueState`)
    static SEEN: RefCell<HashMap<usize, RecursionValueState>> = RefCell::new(HashMap::new());
}

impl RecursionLimiter {
    /// Determines if the specified `T` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `T` values that have been
    /// visited. The first `T` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    pub fn new<T: ?Sized>(o: &T) -> Self {
        let ptr: *const _ = o;
        let ptr = ptr.cast::<()>() as usize;
        let (top_level, visited, live) = SEEN.with(|hm| {
            let mut hm = hm.borrow_mut();
            let top_level = hm.is_empty();
            let old_state = hm.insert(ptr, RecursionValueState::Live);

            (
                top_level,
                old_state == Some(RecursionValueState::Visited),
                old_state == Some(RecursionValueState::Live),
            )
        });

        Self {
            top_level,
            ptr,
            visited,
            live,
        }
    }
}

impl<T: NativeObject + ?Sized> Debug for JsObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let limiter = RecursionLimiter::new(&self.inner.object);

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repetition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            let identity = self.identity();
            let obj = self.borrow();
            let kind = obj.inner.data.as_ref().type_name_of_value();
            if self.is_callable() {
                let name_prop = obj
                    .properties()
                    .get(&PropertyKey::String(js_string!("name")));
                let name = match name_prop {
                    None => JsString::default(),
                    Some(prop) => prop
                        .value()
                        .and_then(JsValue::as_string)
                        .unwrap_or_default(),
                };

                return f.write_fmt(format_args!("({:?}) {:?} {:?}", kind, name, identity));
            }

            f.write_fmt(format_args!("({:?}) {:?}", kind, identity))
        } else {
            f.write_str("{ ... }")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_function::{CapturedObject, NativeCapture};
    use surgeist_js_gc::heap::AllocationCollection;

    #[derive(Debug, crate::JsData, HeapTrace)]
    struct NativePayload(CapturedObject);

    #[test]
    fn native_object_creation_rejects_foreign_prototypes() {
        let runtime = Runtime::default();
        let root_shape = RootShape::new(runtime.clone());
        let foreign = JsObject::with_null_proto(&Runtime::default());

        for result in [
            JsObject::new(&root_shape, foreign.clone(), OrdinaryObject).map(JsObject::upcast),
            JsObject::new_unique(&runtime, foreign.clone(), OrdinaryObject).map(JsObject::upcast),
            JsObject::from_proto_and_data(&runtime, foreign, OrdinaryObject),
        ] {
            assert!(result.unwrap_err().as_native().unwrap().is_type());
        }
    }

    #[test]
    fn native_object_creation_checks_payload_edges() {
        let runtime = Runtime::default();
        let root_shape = RootShape::new(runtime.clone());
        let foreign = JsObject::with_null_proto(&Runtime::default());
        let error = JsObject::new(&root_shape, None, NativePayload(foreign.store())).unwrap_err();
        assert!(error.as_native().unwrap().is_type());

        let reclaimed = JsObject::with_null_proto(&runtime).store();
        runtime.collect().unwrap();
        let error =
            JsObject::from_proto_and_data(&runtime, None, NativePayload(reclaimed)).unwrap_err();
        let error = error.as_native().unwrap();
        assert!(error.is_type());
        assert_eq!(error.message(), AccessError::StaleHandle.to_string());
    }

    #[test]
    fn native_object_creation_keeps_pending_edges_live_through_shape_allocation() {
        let runtime = Runtime::default();
        runtime
            .heap
            .set_allocation_collection(AllocationCollection::BeforeAllocation)
            .unwrap();
        let root_shape = RootShape::new(runtime.clone());
        let prototype = JsObject::with_null_proto(&runtime);
        for unique in [false, true] {
            let target = JsObject::with_null_proto(&runtime);
            let identity = target.identity();
            let data = NativePayload(target.store());
            drop(target);

            let object = if unique {
                JsObject::new_unique(&runtime, prototype.clone(), data)
            } else {
                JsObject::new(&root_shape, prototype.clone(), data)
            }
            .unwrap();
            runtime.collect().unwrap();

            assert_eq!(object.prototype(), Some(prototype.clone()));
            let target: JsObject = JsObject::restore(&object.borrow().data().0).unwrap();
            assert_eq!(target.identity(), identity);

            let edge = target.into_record();
            drop(object);
            runtime.collect().unwrap();
            assert!(!edge.is_live().unwrap());
        }
    }
}
