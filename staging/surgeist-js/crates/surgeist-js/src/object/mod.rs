//! Boa's representation of a JavaScript object and builtin object wrappers
//!
//! For the builtin object wrappers, please see [`object::builtins`][builtins] for implementors.

pub use jsobject::RecursionLimiter;
pub use operations::IntegrityLevel;
pub use property_map::*;
use thin_vec::ThinVec;

use self::{internal_methods::ORDINARY_INTERNAL_METHODS, shape::Shape};
use crate::{
    Context, JsString, JsSymbol, JsValue,
    builtins::{OrdinaryObject, function::ConstructorKind},
    context::intrinsics::StandardConstructor,
    js_string,
    native_function::{NativeFunction, NativeFunctionObject},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    string::StaticJsStrings,
    value::StoredValue,
};

use std::{any::Any, fmt::Debug};
use surgeist_js_gc::{Finalize, Trace};

#[cfg(test)]
mod tests;

pub(crate) mod internal_methods;

pub mod builtins;
mod datatypes;
mod jsobject;
mod operations;
mod property_map;

pub mod shape;

pub(crate) use builtins::*;

pub use datatypes::JsData;
pub use jsobject::*;

/// Const `constructor`, usually set on prototypes as a key to point to their respective constructor object.
pub const CONSTRUCTOR: JsString = js_string!("constructor");

/// Const `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub const PROTOTYPE: JsString = js_string!("prototype");

/// A type alias for an object prototype.
///
/// A `None` values means that the prototype is the `null` value.
pub type JsPrototype = Option<JsObject>;

/// This trait allows Rust types to be passed around as objects.
///
/// This is automatically implemented when a type implements `Any`, `Trace`, and `JsData`.
pub trait NativeObject: Any + Trace + JsData {
    /// Gets the type name of the value.
    fn type_name_of_value(&self) -> &'static str;
}

impl<T: Any + Trace + JsData> NativeObject for T {
    fn type_name_of_value(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

impl dyn NativeObject {
    /// Returns whether the inner value is of type `T`.
    #[inline]
    pub fn is<T: NativeObject>(&self) -> bool {
        (self as &dyn Any).is::<T>()
    }

    /// Borrows the inner value when it is of type `T`.
    #[inline]
    pub fn downcast_ref<T: NativeObject>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }

    /// Mutably borrows the inner value when it is of type `T`.
    #[inline]
    pub fn downcast_mut<T: NativeObject>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut::<T>()
    }
}

/// A JavaScript object's properties and internal state apart from its native payload.
#[derive(Debug, Finalize, Trace)]
#[surgeist_js_gc(unsafe_no_drop)]
pub struct ObjectState {
    pub(crate) properties: PropertyMap,
    pub(crate) extensible: bool,
    private_elements: ThinVec<(PrivateName, StoredPrivateElement)>,
}

impl ObjectState {
    fn new(properties: PropertyMap) -> Self {
        Self {
            properties,
            extensible: true,
            private_elements: ThinVec::new(),
        }
    }
}

/// A JavaScript object awaiting publication.
#[derive(Finalize, Trace)]
#[surgeist_js_gc(unsafe_no_drop)]
pub(crate) struct Object {
    state: ObjectState,
    data: Box<dyn NativeObject>,
}

impl Object {
    fn new(properties: PropertyMap, data: impl NativeObject) -> Self {
        Self {
            state: ObjectState::new(properties),
            data: Box::new(data),
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new(PropertyMap::default(), OrdinaryObject)
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("state", &self.state)
            .field("data_type", &self.data.as_ref().type_name_of_value())
            .finish()
    }
}

/// A Private Name.
#[derive(Clone, Debug, PartialEq, Eq, Trace, Finalize)]
pub struct PrivateName {
    /// The `[[Description]]` internal slot of the private name.
    description: JsString,

    /// The unique identifier of the private name.
    id: usize,
}

impl PrivateName {
    /// Create a new private name.
    pub(crate) const fn new(description: JsString, id: usize) -> Self {
        Self { description, id }
    }
}

/// The representation of private object elements.
#[derive(Clone, Debug, Trace, Finalize)]
pub enum PrivateElement {
    /// A private field.
    Field(JsValue),

    /// A private method.
    Method(JsObject),

    /// A private element accessor.
    Accessor {
        /// A getter function.
        getter: Option<JsObject>,

        /// A setter function.
        setter: Option<JsObject>,
    },
}

/// A private element held by an object or class constructor.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) enum StoredPrivateElement {
    Field(StoredValue),
    Method(ObjectEdge),
    Accessor {
        getter: Option<ObjectEdge>,
        setter: Option<ObjectEdge>,
    },
}

impl From<&PrivateElement> for StoredPrivateElement {
    fn from(element: &PrivateElement) -> Self {
        match element {
            PrivateElement::Field(value) => Self::Field(value.clone().into_stored()),
            PrivateElement::Method(method) => Self::Method(method.inner().clone()),
            PrivateElement::Accessor { getter, setter } => Self::Accessor {
                getter: getter.as_ref().map(|getter| getter.inner().clone()),
                setter: setter.as_ref().map(|setter| setter.inner().clone()),
            },
        }
    }
}

impl ObjectState {
    /// Returns the shape of the object.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.properties.shape
    }

    /// Gets the prototype instance of this object.
    #[inline]
    #[must_use]
    pub fn prototype(&self) -> JsPrototype {
        self.properties.shape.prototype()
    }

    /// Sets the prototype instance of the object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
    #[track_caller]
    pub fn set_prototype<O: Into<JsPrototype>>(&mut self, prototype: O) -> bool {
        let prototype = prototype.into();
        if self.extensible {
            self.properties.shape = self.properties.shape.change_prototype_transition(prototype);
            true
        } else {
            // If target is non-extensible, [[SetPrototypeOf]] must return false
            // unless V is the SameValue as the target's observed [[GetPrototypeOf]] value.
            self.prototype() == prototype
        }
    }

    /// Returns the properties of the object.
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    #[inline]
    pub(crate) fn properties_mut(&mut self) -> &mut PropertyMap {
        &mut self.properties
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name, then `true` is returned
    /// otherwise, `false` is returned.
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> bool
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.properties.insert(&key.into(), property.into())
    }

    /// Helper function for property removal without checking if it's configurable.
    ///
    /// Returns `true` if the property was removed, `false` otherwise.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> bool {
        self.properties.remove(key)
    }

    /// Abstract operation [`PrivateElementFind`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-privateelementfind
    #[allow(clippy::similar_names)]
    fn private_element_find(
        &self,
        name: &PrivateName,
        is_getter: bool,
        is_setter: bool,
    ) -> Option<&StoredPrivateElement> {
        self.private_elements.iter().find_map(|(key, element)| {
            if key != name {
                return None;
            }

            match element {
                StoredPrivateElement::Accessor { getter, setter }
                    if !(getter.is_some() && is_getter || setter.is_some() && is_setter) =>
                {
                    None
                }
                _ => Some(element),
            }
        })
    }

    /// Append a private element to an object.
    pub(crate) fn append_private_element(&mut self, name: PrivateName, element: &PrivateElement) {
        self.append_stored_private_element(name, element.into());
    }

    fn append_stored_private_element(&mut self, name: PrivateName, element: StoredPrivateElement) {
        if let StoredPrivateElement::Accessor { getter, setter } = &element {
            for (key, value) in &mut self.private_elements {
                if name == *key
                    && let StoredPrivateElement::Accessor {
                        getter: existing_getter,
                        setter: existing_setter,
                    } = value
                {
                    if existing_getter.is_none() {
                        existing_getter.clone_from(getter);
                    }
                    if existing_setter.is_none() {
                        existing_setter.clone_from(setter);
                    }
                    return;
                }
            }
        }

        self.private_elements.push((name, element));
    }
}

/// The functions binding.
///
/// Specifies what is the name of the function object (`name` property),
/// and the binding name of the function object which can be different
/// from the function name.
///
/// The only way to construct this is with the `From` trait.
///
/// There are two implementations:
///  - From a single type `T` which implements `Into<FunctionBinding>` which sets the binding
///    name and the function name to the same value.
///  - From a tuple `(B: Into<PropertyKey>, N: Into<JsString>)`, where the `B` is the binding name
///    and the `N` is the function name.
#[derive(Debug, Clone)]
pub struct FunctionBinding {
    pub(crate) binding: PropertyKey,
    pub(crate) name: JsString,
}

impl From<JsString> for FunctionBinding {
    #[inline]
    fn from(name: JsString) -> Self {
        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<JsSymbol> for FunctionBinding {
    #[inline]
    fn from(binding: JsSymbol) -> Self {
        Self {
            name: binding.fn_name(),
            binding: binding.into(),
        }
    }
}

impl<B, N> From<(B, N)> for FunctionBinding
where
    B: Into<PropertyKey>,
    N: Into<JsString>,
{
    fn from((binding, name): (B, N)) -> Self {
        Self {
            binding: binding.into(),
            name: name.into(),
        }
    }
}

/// Builder for creating native function objects
#[derive(Debug)]
pub struct FunctionObjectBuilder<'realm> {
    realm: &'realm Realm,
    function: NativeFunction,
    constructor: Option<ConstructorKind>,
    name: JsString,
    length: usize,
}

impl<'realm> FunctionObjectBuilder<'realm> {
    /// Create a new `FunctionBuilder` for creating a native function.
    #[inline]
    #[must_use]
    pub fn new(realm: &'realm Realm, function: NativeFunction) -> Self {
        Self {
            realm,
            function,
            constructor: None,
            name: js_string!(),
            length: 0,
        }
    }

    /// Specify the name property of object function object.
    ///
    /// The default is `""` (empty string).
    #[must_use]
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<JsString>,
    {
        self.name = name.into();
        self
    }

    /// Specify the length property of object function object.
    ///
    /// How many arguments this function takes.
    ///
    /// The default is `0`.
    #[inline]
    #[must_use]
    pub const fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Specify whether the object function object can be called with `new` keyword.
    ///
    /// The default is `false`.
    #[must_use]
    pub fn constructor(mut self, yes: bool) -> Self {
        self.constructor = yes.then_some(ConstructorKind::Base);
        self
    }

    /// Build the function object.
    #[must_use]
    pub fn build(self) -> JsFunction {
        let object = self.realm.intrinsics().templates().function().create(
            NativeFunctionObject {
                f: self.function,
                name: self.name.clone(),
                constructor: self.constructor,
                realm: Some(self.realm.clone()),
            },
            vec![self.length.into(), self.name.into()],
        );

        JsFunction::from_object_unchecked(object)
    }
}

/// Builder for creating objects with properties.
///
/// # Examples
///
/// ```
/// # use surgeist_js::{
/// #     Context,
/// #     JsValue,
/// #     NativeFunction,
/// #     object::ObjectInitializer,
/// #     property::Attribute,
/// #     js_string,
/// # };
/// let mut context = Context::default();
/// let object = ObjectInitializer::new(&mut context)
///     .property(js_string!("hello"), js_string!("world"), Attribute::all())
///     .property(1, 1, Attribute::all())
///     .function(
///         NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
///         js_string!("func"),
///         0,
///     )
///     .build();
/// ```
///
/// The equivalent in JavaScript would be:
/// ```text
/// let object = {
///     hello: "world",
///     "1": 1,
///     func: function() {}
/// }
/// ```
#[derive(Debug)]
pub struct ObjectInitializer<'ctx> {
    context: &'ctx mut Context,
    object: JsObject,
}

impl<'ctx> ObjectInitializer<'ctx> {
    /// Create a new `ObjectBuilder`.
    #[inline]
    pub fn new(context: &'ctx mut Context) -> Self {
        let object = JsObject::with_object_proto(context.intrinsics());
        Self { context, object }
    }

    /// Create a new `ObjectBuilder` with custom [`NativeObject`] data.
    pub fn with_native_data<T: NativeObject>(data: T, context: &'ctx mut Context) -> Self {
        let object = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            data,
        )
        .upcast();
        Self { context, object }
    }

    /// Create a new `ObjectBuilder` with custom [`NativeObject`] data and custom prototype.
    pub fn with_native_data_and_proto<T: NativeObject>(
        data: T,
        proto: JsObject,
        context: &'ctx mut Context,
    ) -> Self {
        let object =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), proto, data)
                .upcast();
        Self { context, object }
    }

    /// Add a function to the object.
    pub fn function<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context.realm(), function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.object.borrow_mut().insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add a property to the object.
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Add new accessor property to the object.
    ///
    /// # Panics
    ///
    /// If both getter or setter are [`None`].
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        // Accessors should have at least one function.
        assert!(set.is_some() || get.is_some());

        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Build the object.
    #[inline]
    pub fn build(&mut self) -> JsObject {
        self.object.clone()
    }

    /// Gets the context used to create the object.
    #[inline]
    pub fn context(&mut self) -> &mut Context {
        self.context
    }
}

/// Builder for creating constructors objects, like `Array`.
#[derive(Debug)]
pub struct ConstructorBuilder<'ctx> {
    context: &'ctx mut Context,
    function: NativeFunction,
    constructor_object: ObjectState,
    has_prototype_property: bool,
    prototype: Object,
    name: JsString,
    length: usize,
    callable: bool,
    kind: Option<ConstructorKind>,
    inherit: Option<JsPrototype>,
    custom_prototype: Option<JsPrototype>,
}

impl<'ctx> ConstructorBuilder<'ctx> {
    /// Create a new `ConstructorBuilder`.
    #[inline]
    pub fn new(context: &'ctx mut Context, function: NativeFunction) -> ConstructorBuilder<'ctx> {
        Self {
            context,
            function,
            constructor_object: ObjectState::new(PropertyMap::default()),
            prototype: Object::default(),
            length: 0,
            name: js_string!(),
            callable: true,
            kind: Some(ConstructorKind::Base),
            inherit: None,
            custom_prototype: None,
            has_prototype_property: true,
        }
    }

    /// Add new method to the constructors prototype.
    pub fn method<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context.realm(), function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.prototype.state.insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new static method to the constructors object itself.
    pub fn static_method<B>(
        &mut self,
        function: NativeFunction,
        binding: B,
        length: usize,
    ) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionObjectBuilder::new(self.context.realm(), function)
            .name(binding.name)
            .length(length)
            .constructor(false)
            .build();

        self.constructor_object.insert(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new data property to the constructor's prototype.
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.state.insert(key, property);
        self
    }

    /// Add new static data property to the constructor object itself.
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.insert(key, property);
        self
    }

    /// Add new accessor property to the constructor's prototype.
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.state.insert(key, property);
        self
    }

    /// Add new static accessor property to the constructor object itself.
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.insert(key, property);
        self
    }

    /// Add new property to the constructor's prototype.
    pub fn property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.prototype.state.insert(key, property);
        self
    }

    /// Add new static property to the constructor object itself.
    pub fn static_property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.constructor_object.insert(key, property);
        self
    }

    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub fn length(&mut self, length: usize) -> &mut Self {
        self.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `"[object]"`
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.name = name.as_ref().into();
        self
    }

    /// Specify whether the constructor function can be called.
    ///
    /// Default is `true`
    #[inline]
    pub fn callable(&mut self, callable: bool) -> &mut Self {
        self.callable = callable;
        self
    }

    /// Specify whether the constructor function can be called with `new` keyword.
    ///
    /// Default is `true`
    #[inline]
    pub fn constructor(&mut self, constructor: bool) -> &mut Self {
        self.kind = constructor.then_some(ConstructorKind::Base);
        self
    }

    /// Specify the parent prototype which objects created by this constructor
    /// inherit from.
    ///
    /// Default is `Object.prototype`
    pub fn inherit<O: Into<JsPrototype>>(&mut self, prototype: O) -> &mut Self {
        self.inherit = Some(prototype.into());
        self
    }

    /// Specify the `[[Prototype]]` internal field of this constructor.
    ///
    /// Default is `Function.prototype`
    pub fn custom_prototype<O: Into<JsPrototype>>(&mut self, prototype: O) -> &mut Self {
        self.custom_prototype = Some(prototype.into());
        self
    }

    /// Specify whether the constructor function has a 'prototype' property.
    ///
    /// Default is `true`
    #[inline]
    pub fn has_prototype_property(&mut self, has_prototype_property: bool) -> &mut Self {
        self.has_prototype_property = has_prototype_property;
        self
    }

    /// Return the current context.
    #[inline]
    pub fn context(&mut self) -> &mut Context {
        self.context
    }

    /// Build the constructor function object.
    #[must_use]
    pub fn build(mut self) -> StandardConstructor {
        let length = PropertyDescriptor::builder()
            .value(self.length)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        let name = PropertyDescriptor::builder()
            .value(self.name.clone())
            .writable(false)
            .enumerable(false)
            .configurable(true);

        let prototype = {
            if let Some(proto) = self.inherit.take() {
                self.prototype.state.set_prototype(proto);
            } else {
                self.prototype.state.set_prototype(
                    self.context
                        .intrinsics()
                        .constructors()
                        .object()
                        .prototype(),
                );
            }

            JsObject::from_object_and_vtable(self.prototype, &ORDINARY_INTERNAL_METHODS)
        };

        let constructor = {
            let data = NativeFunctionObject {
                f: self.function,
                name: self.name.clone(),
                constructor: self.kind,
                realm: Some(self.context.realm().clone()),
            };
            let internal_methods = data.internal_methods();
            let mut constructor = Object {
                state: self.constructor_object,
                data: Box::new(data),
            };

            constructor.state.insert(StaticJsStrings::LENGTH, length);
            constructor.state.insert(js_string!("name"), name);

            if let Some(proto) = self.custom_prototype.take() {
                constructor.state.set_prototype(proto);
            } else {
                constructor.state.set_prototype(
                    self.context
                        .intrinsics()
                        .constructors()
                        .function()
                        .prototype(),
                );
            }

            if self.has_prototype_property {
                constructor.state.insert(
                    PROTOTYPE,
                    PropertyDescriptor::builder()
                        .value(prototype.clone())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                );
            }

            JsObject::from_object_and_vtable(constructor, internal_methods)
        };

        {
            let mut prototype = prototype.borrow_mut();
            prototype.insert(
                CONSTRUCTOR,
                PropertyDescriptor::builder()
                    .value(constructor.clone())
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
            );
        }

        StandardConstructor::new(JsFunction::from_object_unchecked(constructor), prototype)
    }
}
