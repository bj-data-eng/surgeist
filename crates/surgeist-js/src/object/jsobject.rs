//! This module implements the `JsObject` structure.
//!
//! The `JsObject` is a garbage collected Object.

use super::{
    IndexedProperties, JsData, JsPrototype, NativeObject, NativeProjection, Object, ObjectState,
    PrivateName, PropertyMap,
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
    error::{
        EngineError, EngineResourceError, JsNativeError, ObjectAccessError,
        ObjectConstructionError, RootingError,
    },
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
    fmt::{self, Debug},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
};
use surgeist_js_gc::heap::{AccessError, ObjectId, RecordRef, Rooted, Trace as HeapTrace};

/// An immutable borrow of an object's common state and typed native payload.
pub struct ObjectRef<'a, T: NativeObject + ?Sized> {
    state: Ref<'a, ObjectState>,
    data: Ref<'a, T>,
}

impl<T: NativeObject + ?Sized> Deref for ObjectRef<'_, T> {
    type Target = ObjectState;

    fn deref(&self) -> &ObjectState {
        &self.state
    }
}

impl<T: NativeObject + ?Sized> ObjectRef<'_, T> {
    /// Borrows the object's native payload.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }
}

/// A mutable borrow of an object's common state and typed native payload.
///
/// The guard cannot replace the object's common state, which belongs to the
/// runtime that created it.
///
/// ```compile_fail
/// use surgeist_js::{Context, JsObject};
/// let context = Context::builder().build().unwrap();
/// let a = JsObject::with_object_proto(context.intrinsics()).unwrap();
/// let b = JsObject::with_object_proto(context.intrinsics()).unwrap();
/// std::mem::swap(&mut *a.borrow_mut().unwrap(), &mut *b.borrow_mut().unwrap());
/// ```
///
/// Engine-owned native payloads have their own mutation operations.
///
/// ```compile_fail
/// use surgeist_js::{Context, object::builtins::JsArrayBuffer};
/// let mut context = Context::builder().build().unwrap();
/// let buffer = JsArrayBuffer::new(8, context.access()).unwrap();
/// let mut object = buffer.borrow_mut().unwrap();
/// let _ = object.data_mut();
/// ```
pub struct ObjectRefMut<'a, T: NativeObject + ?Sized> {
    state: RefMut<'a, ObjectState>,
    data: RefMut<'a, T>,
}

impl<T: NativeObject + ?Sized> Deref for ObjectRefMut<'_, T> {
    type Target = ObjectState;

    fn deref(&self) -> &ObjectState {
        &self.state
    }
}

impl<T: NativeObject + ?Sized> ObjectRefMut<'_, T> {
    pub(crate) fn state_mut(&mut self) -> &mut ObjectState {
        &mut self.state
    }

    pub(crate) fn properties_mut(&mut self) -> &mut PropertyMap {
        self.state.properties_mut()
    }

    pub(crate) fn insert(
        &mut self,
        key: impl Into<PropertyKey>,
        property: impl Into<PropertyDescriptor>,
    ) -> JsResult<bool> {
        self.state.insert(key, property)
    }

    /// Sets the prototype if it belongs to this runtime and the object permits it.
    pub fn set_prototype(&mut self, prototype: impl Into<JsPrototype>) -> JsResult<bool> {
        self.state.set_prototype(prototype)
    }
}

impl<T: NativeObject + ?Sized> ObjectRefMut<'_, T> {
    /// Borrows the object's native payload.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Mutably borrows the object's native payload.
    #[must_use]
    pub(crate) fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: NativeObject + ?Sized> Debug for ObjectRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRef")
            .field("state", &self.state)
            .field("data_type", &self.data.deref().type_name_of_value())
            .finish()
    }
}

impl<T: NativeObject + ?Sized> Debug for ObjectRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRefMut")
            .field("state", &self.state)
            .field("data_type", &self.data.deref().type_name_of_value())
            .finish()
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
        origin: AllocationOrigin,
    ) -> JsResult<Self> {
        Ok(Self {
            inner: JsObject::<dyn NativeObject>::allocate(runtime, object, vtable, origin)?,
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
            .try_borrow_mut()
            .map_err(|_| ObjectAccessError::Borrowed)?
            .state
            .set_prototype(Some(prototype))?;
        if !changed {
            return Err(ObjectConstructionError::PrototypeRefused.into());
        }
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
        let mut prototype = prototype.borrow_mut()?;
        let mut slot = super::shape::slot::Slot::new();
        let descriptor = prototype
            .properties
            .get_with_slot(&super::CONSTRUCTOR.into(), &mut slot)?
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
            .write_named_object_edge(slot.index as usize, ObjectEdge(self.inner.record_ref()))?;
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

    /// Returns the origin runtime while its heap exists. This weak-locator
    /// projection neither borrows the heap nor proves the record live.
    pub(crate) fn owning_runtime(&self) -> Option<Runtime> {
        self.0.owning_heap().map(|heap| Runtime { heap })
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
    /// let context = &mut Context::builder().build().unwrap();
    /// let obj = JsObject::default(context.intrinsics()).unwrap();
    ///
    /// assert!(obj.is_ordinary());
    /// ```
    #[inline]
    pub fn default(intrinsics: &Intrinsics) -> JsResult<Self> {
        Self::with_object_proto(intrinsics)
    }

    /// Creates a new `JsObject` from its inner object and its vtable.
    pub(crate) fn from_object_and_vtable(
        runtime: &Runtime,
        object: Object,
        vtable: &'static InternalObjectMethods,
        origin: AllocationOrigin,
    ) -> JsResult<Self> {
        Self::allocate(runtime, object, vtable, origin).map(Self::with_record)
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
    ) -> JsResult<Self> {
        JsObject::<OrdinaryObject>::new_unique_with_vtable(
            runtime,
            None,
            OrdinaryObject,
            |_| vtable,
            AllocationOrigin::TrustedConstruction,
        )
        .map(JsObject::upcast)
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
    /// let context = &mut Context::builder().build().unwrap();
    /// let obj = JsObject::with_object_proto(context.intrinsics()).unwrap();
    ///
    /// assert!(obj.is_ordinary());
    /// assert!(obj.prototype().unwrap().is_some());
    /// ```
    #[inline]
    pub fn with_object_proto(intrinsics: &Intrinsics) -> JsResult<Self> {
        JsObject::<OrdinaryObject>::new_unique_with_vtable(
            &intrinsics.runtime()?,
            intrinsics.constructors().object().prototype()?,
            OrdinaryObject,
            OrdinaryObject::internal_methods,
            AllocationOrigin::TrustedConstruction,
        )
        .map(JsObject::upcast)
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
    /// let obj = JsObject::with_null_proto(&Runtime::default()).unwrap();
    /// assert!(obj.prototype().unwrap().is_none());
    /// assert!(obj.is_ordinary());
    /// ```
    #[inline]
    pub fn with_null_proto(runtime: &Runtime) -> JsResult<Self> {
        Self::from_proto_and_data(runtime, None, OrdinaryObject)
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
    /// let context = &mut Context::builder().build().unwrap();
    /// let obj = JsObject::from_proto_and_data(
    ///     context.runtime(),
    ///     context.intrinsics().constructors().object().prototype().unwrap(),
    ///     OrdinaryObject,
    /// )
    /// .unwrap();
    ///
    /// assert!(obj.is_ordinary());
    /// assert!(obj.prototype().unwrap().is_some());
    ///
    /// // Create an object with no prototype.
    /// let null_obj =
    ///     JsObject::from_proto_and_data(context.runtime(), None, OrdinaryObject)
    ///         .unwrap();
    /// assert!(null_obj.prototype().unwrap().is_none());
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
    ) -> JsResult<JsObject<T>> {
        JsObject::new_with_shared_shape(
            root_shape,
            prototype,
            data,
            AllocationOrigin::TrustedConstruction,
        )
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
    /// Returns a typed access failure if a mutable alias is active or the
    /// required native projection is invalid.
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
    /// assert!(obj.downcast_ref::<OrdinaryObject>().unwrap().is_some());
    ///
    /// // Returns `None` for a wrong type.
    /// assert!(obj.downcast_ref::<CustomStruct>().unwrap().is_none());
    /// ```
    #[track_caller]
    pub fn downcast_ref<T: NativeObject>(&self) -> Result<Option<Ref<'_, T>>, ObjectAccessError> {
        if !self.is::<T>() {
            return Ok(None);
        }
        let ObjectRef { data, .. } = self.borrow()?;
        Ref::filter_map(data, |data| data.downcast_ref())
            .map(Some)
            .map_err(|_| ObjectAccessError::WrongNativeBrand)
    }

    /// Mutably borrows the payload of a host-defined class.
    ///
    /// Returns `None` if the object has a different native brand.
    /// The caller must preserve the heap edges described by [`Class`].
    ///
    /// Conflicting access or an invalid native projection returns a typed
    /// access failure.
    ///
    /// Engine-owned payloads cannot be borrowed mutably through this method.
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, JsObject, builtins::array_buffer::ArrayBuffer};
    /// use surgeist_js::object::builtins::JsArrayBuffer;
    /// let mut context = Context::builder().build().unwrap();
    /// let object: JsObject = JsArrayBuffer::new(8, context.access()).unwrap().into();
    /// let _ = object.downcast_class_mut::<ArrayBuffer>();
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, JsObject, builtins::array_buffer::ArrayBuffer};
    /// use surgeist_js::object::builtins::JsArrayBuffer;
    /// let mut context = Context::builder().build().unwrap();
    /// let object: JsObject = JsArrayBuffer::new(8, context.access()).unwrap().into();
    /// let _ = object.downcast_mut::<ArrayBuffer>();
    /// ```
    #[track_caller]
    pub fn downcast_class_mut<C: Class>(&self) -> Result<Option<RefMut<'_, C>>, ObjectAccessError> {
        self.downcast_mut()
    }

    #[track_caller]
    pub(crate) fn downcast_mut<T: NativeObject>(
        &self,
    ) -> Result<Option<RefMut<'_, T>>, ObjectAccessError> {
        if !self.is::<T>() {
            return Ok(None);
        }
        let ObjectRefMut { data, .. } = self.borrow_mut()?;
        RefMut::filter_map(data, |data| data.downcast_mut())
            .map(Some)
            .map_err(|_| ObjectAccessError::WrongNativeBrand)
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
    /// let context = &mut Context::builder().build().unwrap();
    /// let obj = JsObject::with_object_proto(context.intrinsics()).unwrap();
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
    /// let context = &mut Context::builder().build().unwrap();
    ///
    /// let array = JsArray::new(context.access())?;
    /// // A JsArray's inner JsObject is an array.
    /// assert!(JsObject::from(array).is_array());
    ///
    /// // An ordinary object is not an array.
    /// let obj = JsObject::with_object_proto(context.intrinsics()).unwrap();
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
                let vl = lhs.get_property(key)?;
                let vr = rhs.get_property(key)?;

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
    /// let context = &mut Context::builder().build().unwrap();
    ///
    /// let obj1 = JsObject::with_object_proto(context.intrinsics()).unwrap();
    /// obj1.set(js_string!("key"), 42, false, context.access())?;
    ///
    /// let obj2 = JsObject::with_object_proto(context.intrinsics()).unwrap();
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
    pub(crate) fn get_property(&self, key: &PropertyKey) -> JsResult<Option<PropertyDescriptor>> {
        let mut obj = Some(self.clone());

        while let Some(o) = obj {
            let borrowed = o.borrow()?;
            if let Some(v) = borrowed.properties.get(key)? {
                return Ok(Some(v));
            }
            obj = borrowed.prototype()?;
        }
        Ok(None)
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

    /// Immutably borrows the object's common state and checked native payload.
    ///
    /// Multiple immutable guards may coexist. Conflicting mutable access returns
    /// [`ObjectAccessError::Borrowed`]; an invalid native projection returns
    /// [`ObjectAccessError::WrongNativeBrand`].
    ///
    /// ```
    /// # use surgeist_js::{JsObject, Runtime, builtins::object::OrdinaryObject};
    /// let obj = JsObject::from_proto_and_data(&Runtime::default(), None, OrdinaryObject).unwrap();
    /// let borrowed = obj.borrow().unwrap();
    /// assert!(borrowed.prototype().unwrap().is_none());
    /// ```
    #[inline]
    pub fn borrow(&self) -> Result<ObjectRef<'_, T>, ObjectAccessError>
    where
        T: NativeProjection,
    {
        let inner = self
            .inner
            .object
            .try_borrow()
            .map_err(|_| ObjectAccessError::Borrowed)?;
        let (state, data) = Ref::map_split(inner, |object| (&object.state, object.data.as_ref()));
        let data =
            Ref::filter_map(data, T::project).map_err(|_| ObjectAccessError::WrongNativeBrand)?;
        Ok(ObjectRef { state, data })
    }

    /// Mutably borrows the object's common state and checked native payload.
    ///
    /// Any existing guard returns [`ObjectAccessError::Borrowed`]. An invalid
    /// native projection returns [`ObjectAccessError::WrongNativeBrand`].
    /// Common state and engine-owned native payloads cannot be replaced through
    /// this guard.
    ///
    /// ```
    /// # use surgeist_js::{JsObject, Runtime, builtins::object::OrdinaryObject};
    /// let obj = JsObject::from_proto_and_data(&Runtime::default(), None, OrdinaryObject).unwrap();
    /// obj.borrow_mut().unwrap().set_prototype(None).unwrap();
    /// assert!(obj.prototype().unwrap().is_none());
    /// ```
    #[inline]
    pub fn borrow_mut(&self) -> Result<ObjectRefMut<'_, T>, ObjectAccessError>
    where
        T: NativeProjection,
    {
        let inner = self
            .inner
            .object
            .try_borrow_mut()
            .map_err(|_| ObjectAccessError::Borrowed)?;
        let (state, data) =
            RefMut::map_split(inner, |object| (&mut object.state, object.data.as_mut()));
        let data = RefMut::filter_map(data, T::project_mut)
            .map_err(|_| ObjectAccessError::WrongNativeBrand)?;
        Ok(ObjectRefMut { state, data })
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

    /// Gets the prototype, or returns a typed access failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::builder().build().unwrap();
    ///
    /// let obj = JsObject::with_object_proto(context.intrinsics()).unwrap();
    /// assert!(obj.prototype().unwrap().is_some());
    ///
    /// let null_obj = JsObject::with_null_proto(context.runtime()).unwrap();
    /// assert!(null_obj.prototype().unwrap().is_none());
    /// ```
    #[inline]
    #[track_caller]
    pub fn prototype(&self) -> JsResult<JsPrototype>
    where
        T: NativeProjection,
    {
        self.borrow()?.prototype()
    }

    /// Gets extensibility, or returns a typed access failure.
    pub(crate) fn extensible(&self) -> Result<bool, ObjectAccessError>
    where
        T: NativeProjection,
    {
        Ok(self.borrow()?.extensible)
    }

    /// Sets the prototype, or returns a typed access failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsObject};
    /// let context = &mut Context::builder().build().unwrap();
    /// let obj = JsObject::with_object_proto(context.intrinsics()).unwrap();
    ///
    /// assert!(obj.prototype().unwrap().is_some());
    ///
    /// // Set the prototype to `None`.
    /// obj.set_prototype(None).unwrap();
    /// assert!(obj.prototype().unwrap().is_none());
    /// ```
    #[inline]
    #[track_caller]
    pub fn set_prototype(&self, prototype: JsPrototype) -> JsResult<bool>
    where
        T: NativeProjection,
    {
        self.borrow_mut()?.set_prototype(prototype)
    }

    /// Helper function for property insertion.
    #[track_caller]
    pub(crate) fn insert<K, P>(&self, key: K, property: P) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
        T: NativeProjection,
    {
        self.borrow_mut()?.insert(key, property)
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
        T: NativeProjection,
    {
        let property = property.into();
        self.runtime().check_descriptor(&property)?;
        self.insert(key, property.complete_property_descriptor())
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
        origin: AllocationOrigin,
    ) -> JsResult<Rooted<ObjectRecord>> {
        runtime
            .heap
            .allocate(ObjectRecord::new(object, vtable))
            .map_err(|rejected| origin.access_error(rejected.reason))
    }

    /// Create a new private name with this object as the unique identifier.
    pub(crate) fn private_name(&self, description: JsString) -> PrivateName {
        PrivateName::new(description, self.identity())
    }
}

/// The selected construction owner knows whether edges came from authored
/// ingestion or from a closed represented engine recipe.
#[derive(Clone, Copy)]
pub(crate) enum AllocationOrigin {
    AuthoredInput,
    TrustedConstruction,
}
impl AllocationOrigin {
    pub(crate) fn access_error(self, reason: AccessError) -> JsError {
        match reason {
            AccessError::HeapBorrowed | AccessError::PayloadBorrowed => {
                RootingError::Access(reason).into()
            }
            AccessError::ForeignHeap | AccessError::StaleHandle | AccessError::WrongType => {
                match self {
                    Self::AuthoredInput => {
                        JsNativeError::typ().with_message(reason.to_string()).into()
                    }
                    Self::TrustedConstruction => {
                        ObjectConstructionError::InvalidStoredEdge(reason).into()
                    }
                }
            }
        }
    }
    pub(crate) fn shape_error(self, error: JsError) -> JsError {
        if let Some(EngineError::Rooting(RootingError::Access(reason))) = error.as_engine() {
            self.access_error(*reason)
        } else {
            error
        }
    }
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
    /// let context = &mut Context::builder().build().unwrap();
    ///
    /// let typed_obj = JsObject::new(
    ///     context.root_shape(),
    ///     context.intrinsics().constructors().object().prototype().unwrap(),
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
        Self::new_with_shared_shape(root_shape, prototype, data, AllocationOrigin::AuthoredInput)
    }
    fn new_with_shared_shape<O: Into<Option<JsObject>>>(
        root_shape: &RootShape,
        prototype: O,
        data: T,
        origin: AllocationOrigin,
    ) -> JsResult<Self> {
        let runtime = root_shape.runtime();
        runtime
            .heap
            .with_pending_edges(data, |data| {
                let internal_methods = data.internal_methods();
                let shape = root_shape
                    .shape()
                    .change_prototype_transition(prototype.into())
                    .map_err(|error| origin.shape_error(error))?
                    .into();
                let object =
                    Object::new(PropertyMap::new(&shape, IndexedProperties::default()), data);
                Self::allocate(runtime, object, internal_methods, origin).map(Self::with_record)
            })
            .map_err(|error| origin.access_error(error))?
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
    /// assert!(obj.prototype().unwrap().is_none());
    /// ```
    pub fn new_unique<O: Into<Option<JsObject>>>(
        runtime: &Runtime,
        prototype: O,
        data: T,
    ) -> JsResult<Self> {
        Self::new_unique_with_vtable(
            runtime,
            prototype,
            data,
            T::internal_methods,
            AllocationOrigin::AuthoredInput,
        )
    }

    pub(crate) fn new_unique_with_vtable(
        runtime: &Runtime,
        prototype: impl Into<JsPrototype>,
        data: T,
        vtable: impl FnOnce(&T) -> &'static InternalObjectMethods,
        origin: AllocationOrigin,
    ) -> JsResult<Self> {
        runtime
            .heap
            .with_pending_edges(data, |data| {
                let vtable = vtable(&data);
                let shape = UniqueShape::new(
                    &runtime.heap,
                    prototype.into(),
                    PropertyTableInner::default(),
                )
                .map_err(|error| origin.access_error(error))?
                .into();
                let object =
                    Object::new(PropertyMap::new(&shape, IndexedProperties::default()), data);
                Self::allocate(runtime, object, vtable, origin).map(Self::with_record)
            })
            .map_err(|error| origin.access_error(error))?
    }

    pub(crate) fn borrow_data(&self) -> Result<Ref<'_, T>, ObjectAccessError> {
        let ObjectRef { data, .. } = self.borrow()?;
        Ok(data)
    }

    pub(crate) fn borrow_data_mut(&self) -> Result<RefMut<'_, T>, ObjectAccessError> {
        let ObjectRefMut { data, .. } = self.borrow_mut()?;
        Ok(data)
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
        let obj = self.inner.object.try_borrow().map_err(|_| fmt::Error)?;
        let limiter = RecursionLimiter::new(&self.inner.object);

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repetition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            let identity = self.identity();
            let kind = obj.data.as_ref().type_name_of_value();
            if self.is_callable() {
                let name_prop = obj
                    .state
                    .properties()
                    .get(&PropertyKey::String(js_string!("name")))
                    .map_err(|_| fmt::Error)?;
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

    #[derive(Debug, crate::JsData, HeapTrace)]
    struct HostPayload(u32);

    #[derive(Debug, crate::JsData, HeapTrace)]
    struct OtherHostPayload;

    fn native_type<T: NativeProjection + ?Sized>(
        object: &JsObject<T>,
    ) -> Result<&'static str, ObjectAccessError> {
        Ok(object.borrow()?.data().type_name_of_value())
    }

    #[test]
    fn checked_guards_preserve_sized_and_erased_payload_access() {
        let object = JsObject::new_unique(&Runtime::default(), None, HostPayload(7)).unwrap();
        let erased = object.clone().upcast();
        assert_eq!(object.identity(), erased.identity());
        assert_eq!(
            native_type(&object).unwrap(),
            std::any::type_name::<HostPayload>()
        );
        assert_eq!(
            native_type(&erased).unwrap(),
            std::any::type_name::<HostPayload>()
        );
        {
            let typed = object.borrow().unwrap();
            let erased = erased.borrow().unwrap();
            assert_eq!(typed.data().0, 7);
            assert!(std::ptr::eq(
                typed.data(),
                erased.data().downcast_ref::<HostPayload>().unwrap(),
            ));
        }
        object.borrow_mut().unwrap().data_mut().0 = 11;
        assert_eq!(object.borrow_data().unwrap().0, 11);
        object.borrow_data_mut().unwrap().0 = 13;
        assert_eq!(object.borrow().unwrap().data().0, 13);
        assert_eq!(erased.downcast_ref::<HostPayload>().unwrap().unwrap().0, 13);
        assert!(erased.downcast_ref::<OtherHostPayload>().unwrap().is_none());
    }

    #[test]
    fn checked_guards_report_alias_refusal_then_allow_same_object_retry() {
        use std::fmt::Write;

        let object = JsObject::new_unique(&Runtime::default(), None, HostPayload(17)).unwrap();
        let alias = object.clone().upcast();
        let identity = object.identity();
        let read = object.borrow().unwrap();
        assert_eq!(
            object.borrow_mut().unwrap_err(),
            ObjectAccessError::Borrowed
        );
        assert_eq!(
            alias.downcast_mut::<HostPayload>().unwrap_err(),
            ObjectAccessError::Borrowed
        );
        assert_eq!(
            alias
                .borrow()
                .unwrap()
                .data()
                .downcast_ref::<HostPayload>()
                .unwrap()
                .0,
            17
        );
        drop(read);

        let mut write = object.borrow_mut().unwrap();
        assert_eq!(object.borrow().unwrap_err(), ObjectAccessError::Borrowed);
        assert_eq!(
            alias.downcast_ref::<HostPayload>().unwrap_err(),
            ObjectAccessError::Borrowed
        );
        assert!(alias.downcast_ref::<OtherHostPayload>().unwrap().is_none());
        assert_eq!(
            object.prototype().unwrap_err().as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
        assert_eq!(
            object.extensible().unwrap_err(),
            ObjectAccessError::Borrowed
        );
        let mut text = String::new();
        assert!(write!(&mut text, "{object:?}").is_err());
        write.data_mut().0 = 19;
        drop(write);

        assert_eq!(object.identity(), identity);
        assert_eq!(
            alias
                .borrow_mut()
                .unwrap()
                .data()
                .downcast_ref::<HostPayload>()
                .unwrap()
                .0,
            19
        );
        assert_eq!(object.borrow().unwrap().data().0, 19);
        assert!(write!(&mut text, "{object:?}").is_ok());
    }

    #[test]
    fn invalid_private_typed_witness_releases_both_split_guards() {
        let object = JsObject::new_unique(&Runtime::default(), None, HostPayload(23)).unwrap();
        // Only this owning test constructs the inconsistent witness. The public
        // downcast front cannot produce a brand that disagrees with its record.
        let invalid = JsObject::<OtherHostPayload>::with_record(object.inner.clone());
        assert_eq!(
            invalid.borrow().unwrap_err(),
            ObjectAccessError::WrongNativeBrand
        );
        assert_eq!(
            invalid.borrow_mut().unwrap_err(),
            ObjectAccessError::WrongNativeBrand
        );
        assert_eq!(
            invalid.borrow_data().unwrap_err(),
            ObjectAccessError::WrongNativeBrand
        );
        assert_eq!(
            invalid.borrow_data_mut().unwrap_err(),
            ObjectAccessError::WrongNativeBrand
        );
        let mut retry = object.borrow_mut().unwrap();
        assert_eq!(retry.data().0, 23);
        retry.data_mut().0 = 29;
        drop(retry);
        assert_eq!(object.borrow().unwrap().data().0, 29);
    }

    #[test]
    fn native_object_creation_rejects_foreign_prototypes() {
        let runtime = Runtime::default();
        let root_shape = RootShape::new(runtime.clone()).unwrap();
        let foreign = JsObject::with_null_proto(&Runtime::default()).unwrap();

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
        let root_shape = RootShape::new(runtime.clone()).unwrap();
        let foreign = JsObject::with_null_proto(&Runtime::default()).unwrap();
        let error = JsObject::new(&root_shape, None, NativePayload(foreign.store())).unwrap_err();
        assert!(error.as_native().unwrap().is_type());

        let reclaimed = JsObject::with_null_proto(&runtime).unwrap().store();
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
        let root_shape = RootShape::new(runtime.clone()).unwrap();
        let prototype = JsObject::with_null_proto(&runtime).unwrap();
        for unique in [false, true] {
            let target = JsObject::with_null_proto(&runtime).unwrap();
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

            assert_eq!(object.prototype().unwrap(), Some(prototype.clone()));
            let target: JsObject = JsObject::restore(&object.borrow().unwrap().data().0).unwrap();
            assert_eq!(target.identity(), identity);

            let edge = target.into_record();
            drop(object);
            runtime.collect().unwrap();
            assert!(!edge.is_live().unwrap());
        }
    }
}

/// Private whole-target construction intent. It grants no access to a live guard.
#[derive(Clone, Copy)]
pub(crate) enum BuiltinTargetMode {
    Fresh,
    Update,
}
pub(crate) enum BuiltinPrototype {
    Keep,
    Set(JsPrototype),
}
pub(crate) struct PreparedBuiltinFunction {
    function: crate::native_function::StoredNativeFunction,
    constructor: Option<crate::builtins::function::ConstructorKind>,
    realm: crate::realm::Realm,
}
impl PreparedBuiltinFunction {
    pub(crate) fn new(
        function: crate::native_function::BuiltinFunctionPointer,
        constructor: Option<crate::builtins::function::ConstructorKind>,
        realm: crate::realm::Realm,
    ) -> Self {
        Self {
            function: crate::NativeFunction::from_builtin(function).stored(),
            constructor,
            realm,
        }
    }
}
pub(crate) struct PreparedBuiltinTarget {
    target: JsObject,
    mode: BuiltinTargetMode,
    properties: super::PreparedNamedProperties,
    prototype: BuiltinPrototype,
    function: Option<PreparedBuiltinFunction>,
    extensible: Option<bool>,
}
impl PreparedBuiltinTarget {
    pub(crate) fn ordinary(
        target: JsObject,
        mode: BuiltinTargetMode,
        properties: super::PreparedNamedProperties,
        prototype: BuiltinPrototype,
    ) -> Self {
        Self {
            target,
            mode,
            properties,
            prototype,
            function: None,
            extensible: None,
        }
    }
    pub(crate) fn callable(
        target: JsObject,
        mode: BuiltinTargetMode,
        properties: super::PreparedNamedProperties,
        prototype: BuiltinPrototype,
        function: PreparedBuiltinFunction,
    ) -> Self {
        Self {
            function: Some(function),
            ..Self::ordinary(target, mode, properties, prototype)
        }
    }
    pub(crate) fn extensible(mut self, value: bool) -> Self {
        self.extensible = Some(value);
        self
    }
}

struct BuiltinMapPlan<'a> {
    values: Vec<(&'a mut crate::value::StoredValue, crate::value::StoredValue)>,
    keys: Vec<(
        &'a mut (PropertyKey, super::shape::slot::Slot),
        super::shape::slot::Slot,
    )>,
    post: BuiltinMapPost,
}
struct BuiltinMapPost {
    records: Vec<(PropertyKey, u32, super::shape::slot::Slot)>,
    appended: Vec<(PropertyKey, super::shape::slot::Slot)>,
    values: Vec<crate::value::StoredValue>,
    displaced: Vec<crate::value::StoredValue>,
    required_slots: u32,
}

/// Bind checked, disjoint cells while access can still refuse. Sorting touches
/// only the recipe's updates, never the existing property's whole storage bank.
fn bind_builtin_cells<T, V>(
    mut bank: &mut [T],
    mut writes: Vec<(usize, V)>,
) -> JsResult<Vec<(&mut T, V)>> {
    writes.sort_unstable_by_key(|(index, _)| *index);
    let mut result = Vec::new();
    result
        .try_reserve_exact(writes.len())
        .map_err(|_| EngineResourceError::Allocation)?;
    let mut offset = 0usize;
    for (index, value) in writes {
        let relative = index
            .checked_sub(offset)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (_, rest) = bank
            .split_at_mut_checked(relative)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (cell, tail) = rest
            .split_first_mut()
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        result.push((cell, value));
        bank = tail;
        offset = index
            .checked_add(1)
            .ok_or(EngineResourceError::SizeOverflow)?;
    }
    Ok(result)
}

fn prepare_builtin_map<'a>(
    properties: &'a mut PropertyMap,
    table: &'a mut PropertyTableInner,
    recipe: &super::PreparedNamedProperties,
    mode: BuiltinTargetMode,
) -> JsResult<BuiltinMapPlan<'a>> {
    use super::shape::slot::Slot;
    let storage = properties.named_storage_mut();
    if matches!(mode, BuiltinTargetMode::Fresh) && (!storage.is_empty() || !table.keys.is_empty()) {
        return Err(ObjectConstructionError::NamedSlotsAlreadyInitialized.into());
    }
    if storage.len() != table.required_slots as usize {
        return Err(ObjectConstructionError::NamedSlotCount {
            expected: table.required_slots as usize,
            actual: storage.len(),
        }
        .into());
    }
    let count = recipe.table.keys.len();
    table.reserve(count)?;
    storage
        .try_reserve(recipe.slots.storage.len())
        .map_err(|_| EngineResourceError::Allocation)?;
    let mut value_writes = Vec::new();
    let mut key_writes = Vec::new();
    let mut post = BuiltinMapPost {
        records: Vec::new(),
        appended: Vec::new(),
        values: Vec::new(),
        displaced: Vec::new(),
        required_slots: table.required_slots,
    };
    value_writes
        .try_reserve_exact(recipe.slots.storage.len())
        .map_err(|_| EngineResourceError::Allocation)?;
    key_writes
        .try_reserve_exact(count)
        .map_err(|_| EngineResourceError::Allocation)?;
    post.records
        .try_reserve_exact(count)
        .map_err(|_| EngineResourceError::Allocation)?;
    post.appended
        .try_reserve_exact(count)
        .map_err(|_| EngineResourceError::Allocation)?;
    post.values
        .try_reserve_exact(recipe.slots.storage.len())
        .map_err(|_| EngineResourceError::Allocation)?;
    post.displaced
        .try_reserve_exact(recipe.slots.storage.len())
        .map_err(|_| EngineResourceError::Allocation)?;
    for (key, source_slot) in &recipe.table.keys {
        let start = source_slot.index as usize;
        let end = start
            .checked_add(source_slot.width() as usize)
            .ok_or(EngineResourceError::SizeOverflow)?;
        let values = recipe
            .slots
            .storage
            .get(start..end)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        if let Some((index, old)) = table.map.get(key).map(std::cell::Cell::get) {
            if !old.attributes.width_match(source_slot.attributes) {
                return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
            }
            if table.keys.get(index as usize) != Some(&(key.clone(), old)) {
                return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
            }
            let slot = Slot {
                index: old.index,
                attributes: source_slot.attributes,
            };
            for (relative, value) in values.iter().enumerate() {
                let index = (old.index as usize)
                    .checked_add(relative)
                    .ok_or(EngineResourceError::SizeOverflow)?;
                value_writes.push((index, value.clone()));
            }
            key_writes.push((index as usize, slot));
            post.records.push((key.clone(), index, slot));
        } else {
            let index = u32::try_from(
                table
                    .keys
                    .len()
                    .checked_add(post.appended.len())
                    .ok_or(EngineResourceError::SizeOverflow)?,
            )
            .map_err(|_| EngineResourceError::SizeOverflow)?;
            let slot = Slot {
                index: post.required_slots,
                attributes: source_slot.attributes,
            };
            post.required_slots = post
                .required_slots
                .checked_add(slot.width())
                .ok_or(EngineResourceError::SizeOverflow)?;
            post.records.push((key.clone(), index, slot));
            post.appended.push((key.clone(), slot));
            post.values.extend(values.iter().cloned());
        }
    }
    let values = bind_builtin_cells(storage.as_mut_slice(), value_writes)?;
    let keys = bind_builtin_cells(table.keys.as_mut_slice(), key_writes)?;
    Ok(BuiltinMapPlan { values, keys, post })
}
impl BuiltinMapPlan<'_> {
    fn commit(mut self) -> BuiltinMapPost {
        for (cell, value) in self.values {
            self.post.displaced.push(std::mem::replace(cell, value));
        }
        for (cell, slot) in self.keys {
            cell.1 = slot;
        }
        self.post
    }
}
impl BuiltinMapPost {
    fn finish(
        mut self,
        properties: &mut PropertyMap,
        table: &mut PropertyTableInner,
    ) -> Vec<crate::value::StoredValue> {
        for (key, index, slot) in self.records {
            table.map.insert(key, std::cell::Cell::new((index, slot)));
        }
        table.keys.append(&mut self.appended);
        properties.named_storage_mut().append(&mut self.values);
        table.required_slots = self.required_slots;
        self.displaced
    }
}
fn builtin_prototype(
    target: &PreparedBuiltinTarget,
    extensible: bool,
    current: &Option<ObjectEdge>,
) -> JsResult<Option<Option<ObjectEdge>>> {
    match &target.prototype {
        BuiltinPrototype::Keep => Ok(None),
        BuiltinPrototype::Set(value) => {
            if value
                .as_ref()
                .is_some_and(|value| !target.target.identity().same_heap(value.identity()))
            {
                return Err(ObjectConstructionError::PrototypeRefused.into());
            }
            let edge = value.as_ref().map(JsObject::edge);
            if !extensible && *current != edge {
                return Err(ObjectConstructionError::PrototypeRefused.into());
            }
            Ok(Some(edge))
        }
    }
}

/// All roots/intents outlive every guard on every return path. Only checked host
/// reservations and closed scalar admission occur under the guards. No callback,
/// GC allocation, fallible lookup or opaque capture disposal occurs after commit.
fn validate_builtin_roots(target: &PreparedBuiltinTarget) -> JsResult<()> {
    let identity = target.target.identity();
    for value in &target.properties.slots._roots {
        if value
            .as_object()
            .is_some_and(|object| !identity.same_heap(object.identity()))
        {
            return Err(
                ObjectConstructionError::InvalidStoredEdge(AccessError::ForeignHeap).into(),
            );
        }
    }
    if target
        .function
        .as_ref()
        .is_some_and(|metadata| !identity.same_heap(metadata.realm.identity()))
    {
        return Err(ObjectConstructionError::InvalidStoredEdge(AccessError::ForeignHeap).into());
    }
    Ok(())
}

pub(crate) fn commit_builtin_targets(
    first: PreparedBuiltinTarget,
    second: Option<PreparedBuiltinTarget>,
) -> JsResult<()> {
    validate_builtin_roots(&first)?;
    if let Some(second) = &second {
        validate_builtin_roots(second)?;
    }
    match second {
        Some(second) => commit_builtin_pair(first, second),
        None => commit_builtin_single(first),
    }
}
fn commit_builtin_single(target: PreparedBuiltinTarget) -> JsResult<()> {
    use crate::native_function::NativeFunctionObject;
    let shape = target.target.borrow()?.shape()?;
    let unique = shape
        .as_unique()
        .ok_or(ObjectConstructionError::ExpectedUniqueShape)?;
    let mut object = target.target.borrow_mut()?;
    let mut guards = unique.borrow_for_builtin().map_err(RootingError::Access)?;
    if !object.properties.same_shape(&shape) {
        return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
    }
    let prototype = builtin_prototype(&target, object.extensible, &guards.prototype)?;
    let native = target
        .function
        .as_ref()
        .map(|metadata| {
            object
                .data
                .downcast_mut::<NativeFunctionObject>()
                .ok_or(ObjectAccessError::WrongNativeBrand)
                .map(|native| (native, metadata))
        })
        .transpose()?;
    let plan = prepare_builtin_map(
        &mut object.state.properties,
        &mut guards.table,
        &target.properties,
        target.mode,
    )?;
    let displaced = plan
        .commit()
        .finish(&mut object.state.properties, &mut guards.table);
    let old_prototype =
        prototype.map(|prototype| std::mem::replace(&mut *guards.prototype, prototype));
    let old_function = native.map(|(native, metadata)| {
        native.constructor = metadata.constructor;
        native.realm = Some(metadata.realm.edge());
        std::mem::replace(&mut native.f, metadata.function.clone())
    });
    if let Some(value) = target.extensible {
        object.state.extensible = value;
    }
    drop(guards);
    drop(object);
    drop((displaced, old_prototype, old_function));
    Ok(())
}
fn commit_builtin_pair(
    first: PreparedBuiltinTarget,
    second: PreparedBuiltinTarget,
) -> JsResult<()> {
    use crate::native_function::NativeFunctionObject;
    let first_shape = first.target.borrow()?.shape()?;
    let second_shape = second.target.borrow()?.shape()?;
    let first_unique = first_shape
        .as_unique()
        .ok_or(ObjectConstructionError::ExpectedUniqueShape)?;
    let second_unique = second_shape
        .as_unique()
        .ok_or(ObjectConstructionError::ExpectedUniqueShape)?;
    let mut first_object = first.target.borrow_mut()?;
    let mut second_object = second.target.borrow_mut()?;
    let mut first_guards = first_unique
        .borrow_for_builtin()
        .map_err(RootingError::Access)?;
    let mut second_guards = second_unique
        .borrow_for_builtin()
        .map_err(RootingError::Access)?;
    if !first_object.properties.same_shape(&first_shape)
        || !second_object.properties.same_shape(&second_shape)
    {
        return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
    }
    let first_proto = builtin_prototype(&first, first_object.extensible, &first_guards.prototype)?;
    let second_proto =
        builtin_prototype(&second, second_object.extensible, &second_guards.prototype)?;
    let first_native = first
        .function
        .as_ref()
        .map(|metadata| {
            first_object
                .data
                .downcast_mut::<NativeFunctionObject>()
                .ok_or(ObjectAccessError::WrongNativeBrand)
                .map(|native| (native, metadata))
        })
        .transpose()?;
    let second_native = second
        .function
        .as_ref()
        .map(|metadata| {
            second_object
                .data
                .downcast_mut::<NativeFunctionObject>()
                .ok_or(ObjectAccessError::WrongNativeBrand)
                .map(|native| (native, metadata))
        })
        .transpose()?;
    let first_plan = prepare_builtin_map(
        &mut first_object.state.properties,
        &mut first_guards.table,
        &first.properties,
        first.mode,
    )?;
    let second_plan = prepare_builtin_map(
        &mut second_object.state.properties,
        &mut second_guards.table,
        &second.properties,
        second.mode,
    )?;
    // No refusal remains after the first cell write. Both targets, including
    // native payloads and prototype cells, remain guarded until both commits end.
    let first_post = first_plan.commit();
    let second_post = second_plan.commit();
    let first_displaced =
        first_post.finish(&mut first_object.state.properties, &mut first_guards.table);
    let second_displaced = second_post.finish(
        &mut second_object.state.properties,
        &mut second_guards.table,
    );
    let first_old_proto =
        first_proto.map(|prototype| std::mem::replace(&mut *first_guards.prototype, prototype));
    let second_old_proto =
        second_proto.map(|prototype| std::mem::replace(&mut *second_guards.prototype, prototype));
    let first_old_function = first_native.map(|(native, metadata)| {
        native.constructor = metadata.constructor;
        native.realm = Some(metadata.realm.edge());
        std::mem::replace(&mut native.f, metadata.function.clone())
    });
    let second_old_function = second_native.map(|(native, metadata)| {
        native.constructor = metadata.constructor;
        native.realm = Some(metadata.realm.edge());
        std::mem::replace(&mut native.f, metadata.function.clone())
    });
    if let Some(value) = first.extensible {
        first_object.state.extensible = value;
    }
    if let Some(value) = second.extensible {
        second_object.state.extensible = value;
    }
    drop(second_guards);
    drop(first_guards);
    drop(second_object);
    drop(first_object);
    drop((
        first_displaced,
        second_displaced,
        first_old_proto,
        second_old_proto,
        first_old_function,
        second_old_function,
    ));
    Ok(())
}

#[cfg(test)]
mod construction_tests {
    use super::*;
    use crate::object::shape::slot::SlotAttributes;
    use crate::{js_string, object::NamedLayout};
    use surgeist_js_gc::heap::AllocationCollection;

    #[test]
    fn displaced_native_capture_drop_reenters_both_targets_after_complete_commit() {
        use crate::{
            NativeFunction,
            builtins::function::ConstructorKind,
            native_function::{CapturedObject, NativeCapture, NativeFunctionObject},
            object::JsFunction,
        };
        use std::cell::Cell;

        #[derive(Debug, PartialEq, Eq)]
        struct Snapshot {
            first_value: Option<i32>,
            second_value: Option<i32>,
            first_prototype: Option<ObjectId>,
            second_prototype: Option<ObjectId>,
            first_extensible: bool,
            second_extensible: bool,
            native_is_base_constructor: bool,
            native_has_realm: bool,
            native_name: JsString,
        }

        #[derive(Default)]
        struct Observation {
            calls: Cell<usize>,
            result: RefCell<Option<JsResult<Snapshot>>>,
        }

        struct LiveCapture {
            first: JsObject,
            second: JsObject,
            observation: Rc<Observation>,
        }

        #[derive(surgeist_js_gc::heap::Trace)]
        struct StoredCapture {
            first: CapturedObject,
            second: CapturedObject,
            #[heap_trace(skip)]
            observation: Rc<Observation>,
        }

        impl NativeCapture for LiveCapture {
            type Stored = StoredCapture;

            fn store(&self) -> Self::Stored {
                StoredCapture {
                    first: self.first.store(),
                    second: self.second.store(),
                    observation: self.observation.clone(),
                }
            }

            fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
                Ok(Self {
                    first: JsObject::restore(&stored.first)?,
                    second: JsObject::restore(&stored.second)?,
                    observation: stored.observation.clone(),
                })
            }
        }

        impl StoredCapture {
            fn observe(&self) -> JsResult<Snapshot> {
                let first: JsObject = JsObject::restore(&self.first)?;
                let second: JsObject = JsObject::restore(&self.second)?;
                let first_shape = first.borrow()?.shape()?;
                let second_shape = second.borrow()?.shape()?;
                let first_unique = first_shape
                    .as_unique()
                    .ok_or(ObjectConstructionError::ExpectedUniqueShape)?;
                let second_unique = second_shape
                    .as_unique()
                    .ok_or(ObjectConstructionError::ExpectedUniqueShape)?;
                let first = first
                    .downcast::<NativeFunctionObject>()
                    .map_err(|_| ObjectAccessError::WrongNativeBrand)?;

                // Keep every real object/native and shape/table/prototype guard
                // together, rather than using a preflight or observer-only flag.
                let first_object = first.borrow_mut()?;
                let second_object = second.borrow_mut()?;
                let first_guards = first_unique
                    .borrow_for_builtin()
                    .map_err(RootingError::Access)?;
                let second_guards = second_unique
                    .borrow_for_builtin()
                    .map_err(RootingError::Access)?;

                // Read storage via the already-held table's slot. A normal
                // lookup would conflict with the table guard acquired above.
                let first_value = first_guards
                    .table
                    .map
                    .get(&js_string!("left").into())
                    .map(|entry| first_object.properties().get_storage(entry.get().1))
                    .transpose()?
                    .and_then(|property| property.value().and_then(JsValue::as_i32));
                let second_value = second_guards
                    .table
                    .map
                    .get(&js_string!("right").into())
                    .map(|entry| second_object.properties().get_storage(entry.get().1))
                    .transpose()?
                    .and_then(|property| property.value().and_then(JsValue::as_i32));
                let native = first_object.data();
                Ok(Snapshot {
                    first_value,
                    second_value,
                    first_prototype: first_guards.prototype.as_ref().map(ObjectEdge::identity),
                    second_prototype: second_guards.prototype.as_ref().map(ObjectEdge::identity),
                    first_extensible: first_object.extensible,
                    second_extensible: second_object.extensible,
                    native_is_base_constructor: matches!(
                        native.constructor,
                        Some(ConstructorKind::Base)
                    ),
                    native_has_realm: native.realm.is_some(),
                    native_name: native.name.clone(),
                })
            }
        }

        impl Drop for StoredCapture {
            fn drop(&mut self) {
                // Report a refused reentry to the outer test without panicking
                // from a destructor or swallowing the exact checked failure.
                let result = self.observe();
                self.observation.calls.set(self.observation.calls.get() + 1);
                drop(self.observation.result.replace(Some(result)));
            }
        }

        let mut context = Context::builder().build().unwrap();
        let runtime = context.runtime().clone();
        let first: JsObject = JsFunction::empty_intrinsic_function(&runtime, true)
            .unwrap()
            .into();
        let second = JsObject::with_null_proto(&runtime).unwrap();
        let first_parent = JsObject::with_null_proto(&runtime).unwrap();
        let second_parent = JsObject::with_null_proto(&runtime).unwrap();
        let observation = Rc::new(Observation::default());
        let old_function = NativeFunction::from_fn_with_captures(
            |_, _, _, _| Ok(JsValue::undefined()),
            LiveCapture {
                first: first.clone(),
                second: second.clone(),
                observation: observation.clone(),
            },
        );
        let old_pointer = {
            let mut native = first
                .downcast_mut::<NativeFunctionObject>()
                .unwrap()
                .unwrap();
            native.name = js_string!("retained native name");
            std::mem::replace(&mut native.f, old_function.stored())
        };
        drop(old_pointer);
        // Leave the actual target payload as the sole owner of the old stored
        // closure. Its traced capture holds edges, not rooted heap owners.
        drop(old_function);
        assert_eq!(observation.calls.get(), 0);

        commit_builtin_targets(
            PreparedBuiltinTarget::callable(
                first.clone(),
                BuiltinTargetMode::Fresh,
                properties("left", 11.into()),
                BuiltinPrototype::Set(Some(first_parent.clone())),
                PreparedBuiltinFunction::new(
                    |_, _, _| Ok(JsValue::from(73)),
                    Some(ConstructorKind::Base),
                    context.realm().clone(),
                ),
            )
            .extensible(false),
            Some(PreparedBuiltinTarget::ordinary(
                second,
                BuiltinTargetMode::Fresh,
                properties("right", 22.into()),
                BuiltinPrototype::Set(Some(second_parent.clone())),
            )),
        )
        .unwrap();

        // No collection triggers the observation: disposal is synchronous with
        // replacement, and both targets must already expose the completed state.
        assert_eq!(observation.calls.get(), 1);
        assert_eq!(
            observation.result.take().unwrap().unwrap(),
            Snapshot {
                first_value: Some(11),
                second_value: Some(22),
                first_prototype: Some(first_parent.identity()),
                second_prototype: Some(second_parent.identity()),
                first_extensible: false,
                second_extensible: true,
                native_is_base_constructor: true,
                native_has_realm: true,
                native_name: js_string!("retained native name"),
            }
        );
        assert_eq!(
            first
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            JsValue::from(73)
        );
    }

    fn properties(key: &str, value: JsValue) -> super::super::PreparedNamedProperties {
        let mut layout = NamedLayout::new(1).unwrap();
        layout
            .push(JsString::from(key).into(), SlotAttributes::CONFIGURABLE)
            .unwrap();
        layout.prepare(vec![value]).unwrap()
    }
    #[test]
    fn edge_runtime_locator_uses_metadata_without_rooting_or_proving_liveness() {
        let runtime = Runtime::default();
        let object = JsObject::with_null_proto(&runtime).unwrap();
        let edge = object.edge();
        runtime
            .heap
            .read(|_| {
                assert!(edge.owning_runtime().unwrap().same_heap(&runtime));
            })
            .unwrap();
        drop(object);
        runtime.collect().unwrap();
        assert!(!edge.is_live().unwrap());
        assert!(edge.owning_runtime().unwrap().same_heap(&runtime));
        drop(runtime);
        assert!(edge.owning_runtime().is_none());
    }

    #[test]
    fn named_layout_refuses_index_duplicate_and_wrong_storage_count() {
        let mut layout = NamedLayout::new(2).unwrap();
        assert_eq!(
            layout
                .push(1u32.into(), SlotAttributes::empty())
                .unwrap_err()
                .as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::IndexedTemplateKey
            ))
        );
        layout
            .push(js_string!("first").into(), SlotAttributes::empty())
            .unwrap();
        assert_eq!(
            layout
                .push(js_string!("first").into(), SlotAttributes::empty())
                .unwrap_err()
                .as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::DuplicateTemplateProperty
            ))
        );
        let error = layout.prepare(vec![]).err().unwrap();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::NamedSlotCount {
                    expected: 1,
                    actual: 0
                }
            ))
        );
    }
    #[test]
    fn pair_admission_refusal_preserves_both_targets_and_can_retry() {
        let runtime = Runtime::default();
        let first = JsObject::with_null_proto(&runtime).unwrap();
        let second = JsObject::with_null_proto(&runtime).unwrap();
        let held = second.borrow().unwrap();
        let attempt = || {
            commit_builtin_targets(
                PreparedBuiltinTarget::ordinary(
                    first.clone(),
                    BuiltinTargetMode::Fresh,
                    properties("left", 1.into()),
                    BuiltinPrototype::Keep,
                ),
                Some(PreparedBuiltinTarget::ordinary(
                    second.clone(),
                    BuiltinTargetMode::Fresh,
                    properties("right", 2.into()),
                    BuiltinPrototype::Keep,
                )),
            )
        };
        assert_eq!(
            attempt().unwrap_err().as_engine(),
            Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
        );
        assert!(
            first
                .get_property(&js_string!("left").into())
                .unwrap()
                .is_none()
        );
        assert!(
            held.properties()
                .get(&js_string!("right").into())
                .unwrap()
                .is_none()
        );
        drop(held);
        attempt().unwrap();
        assert_eq!(
            first
                .get_property(&js_string!("left").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&JsValue::from(1))
        );
        assert_eq!(
            second
                .get_property(&js_string!("right").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&JsValue::from(2))
        );
    }
    #[test]
    fn populated_update_preserves_members_overwrites_same_width_and_appends() {
        let runtime = Runtime::default();
        let object = JsObject::with_null_proto(&runtime).unwrap();
        object
            .insert_property(
                js_string!("method"),
                PropertyDescriptor::builder().value(9).configurable(true),
            )
            .unwrap();
        object
            .insert_property(
                js_string!("length"),
                PropertyDescriptor::builder().value(1).configurable(true),
            )
            .unwrap();
        let mut layout = NamedLayout::new(2).unwrap();
        layout
            .push(js_string!("length").into(), SlotAttributes::CONFIGURABLE)
            .unwrap();
        layout
            .push(js_string!("name").into(), SlotAttributes::CONFIGURABLE)
            .unwrap();
        commit_builtin_targets(
            PreparedBuiltinTarget::ordinary(
                object.clone(),
                BuiltinTargetMode::Update,
                layout
                    .prepare(vec![2.into(), js_string!("new").into()])
                    .unwrap(),
                BuiltinPrototype::Keep,
            ),
            None,
        )
        .unwrap();
        assert_eq!(
            object
                .get_property(&js_string!("method").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&JsValue::from(9))
        );
        assert_eq!(
            object
                .get_property(&js_string!("length").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&JsValue::from(2))
        );
        assert_eq!(
            object.borrow().unwrap().shape().unwrap().keys().unwrap(),
            vec![
                PropertyKey::from(js_string!("method")),
                PropertyKey::from(js_string!("length")),
                PropertyKey::from(js_string!("name"))
            ]
        );
    }
    #[test]
    fn incompatible_update_width_is_refused_without_changing_existing_value() {
        let runtime = Runtime::default();
        let object = JsObject::with_null_proto(&runtime).unwrap();
        object
            .insert_property(
                js_string!("length"),
                PropertyDescriptor::builder().value(3).configurable(true),
            )
            .unwrap();
        let mut layout = NamedLayout::new(1).unwrap();
        layout
            .push(js_string!("length").into(), SlotAttributes::GET)
            .unwrap();
        let error = commit_builtin_targets(
            PreparedBuiltinTarget::ordinary(
                object.clone(),
                BuiltinTargetMode::Update,
                layout
                    .prepare(vec![JsValue::undefined(), JsValue::undefined()])
                    .unwrap(),
                BuiltinPrototype::Keep,
            ),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::IncompatibleSlotLayout
            ))
        );
        assert_eq!(
            object
                .get_property(&js_string!("length").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&JsValue::from(3))
        );
    }
    #[test]
    fn prepared_children_remain_rooted_across_later_allocations() {
        let runtime = Runtime::default();
        runtime
            .heap
            .set_allocation_collection(AllocationCollection::BeforeAllocation)
            .unwrap();
        let object = JsObject::with_null_proto(&runtime).unwrap();
        let child = JsObject::with_null_proto(&runtime).unwrap();
        let identity = child.identity();
        let prepared = PreparedBuiltinTarget::ordinary(
            object.clone(),
            BuiltinTargetMode::Fresh,
            properties("child", child.into()),
            BuiltinPrototype::Keep,
        );
        for _ in 0..8 {
            drop(JsObject::with_null_proto(&runtime).unwrap());
        }
        commit_builtin_targets(prepared, None).unwrap();
        let value = object
            .get_property(&js_string!("child").into())
            .unwrap()
            .unwrap()
            .value()
            .unwrap()
            .clone();
        assert_eq!(value.as_object().unwrap().identity(), identity);
    }
    #[test]
    fn authored_and_trusted_access_origins_preserve_distinct_refusals() {
        for reason in [AccessError::HeapBorrowed, AccessError::PayloadBorrowed] {
            for origin in [
                AllocationOrigin::AuthoredInput,
                AllocationOrigin::TrustedConstruction,
            ] {
                assert_eq!(
                    origin.access_error(reason).as_engine(),
                    Some(&EngineError::Rooting(RootingError::Access(reason)))
                );
            }
        }
        for reason in [
            AccessError::ForeignHeap,
            AccessError::StaleHandle,
            AccessError::WrongType,
        ] {
            let authored = AllocationOrigin::AuthoredInput.access_error(reason);
            assert!(authored.as_native().unwrap().is_type());
            assert_eq!(authored.as_native().unwrap().message(), reason.to_string());
            assert_eq!(
                AllocationOrigin::TrustedConstruction
                    .access_error(reason)
                    .as_engine(),
                Some(&EngineError::ObjectConstruction(
                    ObjectConstructionError::InvalidStoredEdge(reason)
                ))
            );
        }
    }
}
