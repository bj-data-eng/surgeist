use crate::{
    JsObject, JsResult, JsString, JsValue,
    error::{EngineResourceError, ObjectConstructionError},
    js_string,
    native_function::{BuiltinFunctionPointer, NativeFunctionObject},
    object::{
        AllocationOrigin, BuiltinPrototype, BuiltinTargetMode, CONSTRUCTOR, FunctionBinding,
        JsFunction, JsPrototype, NamedLayout, PROTOTYPE, PreparedBuiltinFunction,
        PreparedBuiltinTarget, PreparedNamedProperties, commit_builtin_targets,
        shape::slot::SlotAttributes,
    },
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::StaticJsStrings,
};
use surgeist_js_gc::heap::AccessError;

use super::{BuiltInConstructor, IntrinsicObject, function::ConstructorKind};

/// Authored named properties retain their live values until whole-target admission.
struct NamedRecipe {
    layout: NamedLayout,
    values: Vec<JsValue>,
}

impl NamedRecipe {
    fn new(capacity: usize) -> JsResult<Self> {
        let layout = NamedLayout::new(capacity)?;
        let mut values = Vec::new();
        values
            .try_reserve(capacity)
            .map_err(|_| EngineResourceError::Allocation)?;
        Ok(Self { layout, values })
    }

    fn reserve_entry(
        &mut self,
        key: PropertyKey,
        attributes: SlotAttributes,
        width: usize,
    ) -> JsResult<()> {
        self.layout.push(key, attributes)?;
        self.values
            .try_reserve(width)
            .map_err(|_| EngineResourceError::Allocation)?;
        Ok(())
    }

    fn data(&mut self, key: PropertyKey, value: JsValue, attribute: Attribute) -> JsResult<()> {
        self.reserve_entry(key, SlotAttributes::from_bits_truncate(attribute.bits()), 1)?;
        self.values.push(value);
        Ok(())
    }

    fn accessor(
        &mut self,
        key: PropertyKey,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> JsResult<()> {
        if attribute.contains(Attribute::WRITABLE) || (get.is_none() && set.is_none()) {
            return Err(ObjectConstructionError::InvalidAccessorLayout.into());
        }
        let mut attributes = SlotAttributes::from_bits_truncate(attribute.bits());
        attributes.set(SlotAttributes::GET, get.is_some());
        attributes.set(SlotAttributes::SET, set.is_some());
        self.reserve_entry(key, attributes, 2)?;
        self.values.extend([
            get.map(JsValue::new).unwrap_or_default(),
            set.map(JsValue::new).unwrap_or_default(),
        ]);
        Ok(())
    }

    fn method(
        &mut self,
        realm: &Realm,
        function: BuiltinFunctionPointer,
        binding: FunctionBinding,
        length: usize,
    ) -> JsResult<()> {
        self.reserve_entry(
            binding.binding,
            SlotAttributes::WRITABLE | SlotAttributes::CONFIGURABLE,
            1,
        )?;
        let function = BuiltInBuilder::callable(realm, function)
            .name(binding.name)
            .length(length)
            .build()?;
        self.values.push(function.into());
        Ok(())
    }

    fn prepare(self) -> JsResult<PreparedNamedProperties> {
        self.layout.prepare(self.values)
    }
}

/// A target recipe is not a live result. Its roots survive preparation and refusal.
struct TargetRecipe {
    object: JsObject,
    properties: NamedRecipe,
    prototype: JsPrototype,
    native: Option<PreparedBuiltinFunction>,
    extensible: Option<bool>,
}

impl TargetRecipe {
    fn prepare(self, mode: BuiltinTargetMode) -> JsResult<PreparedBuiltinTarget> {
        let properties = self.properties.prepare()?;
        let prototype = BuiltinPrototype::Set(self.prototype);
        let target = match self.native {
            Some(function) => {
                PreparedBuiltinTarget::callable(self.object, mode, properties, prototype, function)
            }
            None => PreparedBuiltinTarget::ordinary(self.object, mode, properties, prototype),
        };
        Ok(match self.extensible {
            Some(extensible) => target.extensible(extensible),
            None => target,
        })
    }
}

fn check_prototype(realm: &Realm, prototype: &JsPrototype) -> JsResult<()> {
    if prototype
        .as_ref()
        .is_some_and(|prototype| !realm.runtime().same_heap(&prototype.runtime()))
    {
        return Err(ObjectConstructionError::InvalidStoredEdge(AccessError::ForeignHeap).into());
    }
    Ok(())
}

/// Marker for a recipe that returns its ordinary object.
pub(crate) struct OrdinaryObject;

/// A callable recipe records metadata without changing a live native payload.
pub(crate) struct Callable {
    function: BuiltinFunctionPointer,
    name: JsString,
    length: usize,
}

/// Fluent preparation returns its first error immediately, before later arguments run.
///
/// Every successful stage is an authored recipe. Only build admits and commits targets.
#[must_use = "Call build to admit and publish the prepared builtin"]
pub(crate) struct BuiltInBuilder<'ctx, Kind> {
    realm: &'ctx Realm,
    target: TargetRecipe,
    kind: Kind,
}

impl<'ctx> BuiltInBuilder<'ctx, OrdinaryObject> {
    pub(crate) fn with_intrinsic<I: IntrinsicObject>(
        realm: &'ctx Realm,
    ) -> JsResult<BuiltInBuilder<'ctx, OrdinaryObject>> {
        let object = I::get(realm.intrinsics())?;
        let prototype = realm.intrinsics().constructors().object().prototype()?;
        Ok(BuiltInBuilder {
            realm,
            target: TargetRecipe {
                object,
                properties: NamedRecipe::new(0)?,
                prototype: Some(prototype),
                native: None,
                extensible: None,
            },
            kind: OrdinaryObject,
        })
    }

    pub(crate) fn callable(
        realm: &'ctx Realm,
        function: BuiltinFunctionPointer,
    ) -> BuiltInCallable<'ctx> {
        BuiltInCallable {
            realm,
            function,
            name: js_string!(),
            length: 0,
        }
    }

    pub(crate) fn callable_with_intrinsic<I: IntrinsicObject>(
        realm: &'ctx Realm,
        function: BuiltinFunctionPointer,
    ) -> JsResult<BuiltInBuilder<'ctx, Callable>> {
        let object = I::get(realm.intrinsics())?;
        Self::callable_with_object(realm, object, function)
    }

    pub(crate) fn callable_with_object(
        realm: &'ctx Realm,
        object: JsObject,
        function: BuiltinFunctionPointer,
    ) -> JsResult<BuiltInBuilder<'ctx, Callable>> {
        let prototype = realm.intrinsics().constructors().function().prototype()?;
        Ok(BuiltInBuilder {
            realm,
            target: TargetRecipe {
                object,
                properties: NamedRecipe::new(0)?,
                prototype: Some(prototype),
                native: None,
                extensible: None,
            },
            kind: Callable {
                function,
                name: js_string!(),
                length: 0,
            },
        })
    }

    pub(crate) fn from_standard_constructor<SC: BuiltInConstructor>(
        realm: &'ctx Realm,
    ) -> JsResult<BuiltInConstructorWithPrototype<'ctx>> {
        let standard = SC::STANDARD_CONSTRUCTOR(realm.intrinsics().constructors());
        let constructor_properties = NamedRecipe::new(
            SC::CONSTRUCTOR_STORAGE_SLOTS
                .checked_add(3)
                .ok_or(EngineResourceError::SizeOverflow)?,
        )?;
        let constructor = standard.constructor()?;
        let prototype_properties = NamedRecipe::new(
            SC::PROTOTYPE_STORAGE_SLOTS
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?,
        )?;
        let prototype = standard.prototype()?;
        let constructor_parent = realm.intrinsics().constructors().function().prototype()?;
        let prototype_parent = realm.intrinsics().constructors().object().prototype()?;
        Ok(BuiltInConstructorWithPrototype {
            realm,
            function: SC::constructor,
            name: js_string!(SC::NAME),
            length: SC::CONSTRUCTOR_ARGUMENTS,
            constructor: TargetRecipe {
                object: constructor,
                properties: constructor_properties,
                prototype: Some(constructor_parent),
                native: None,
                extensible: None,
            },
            prototype: TargetRecipe {
                object: prototype,
                properties: prototype_properties,
                prototype: Some(prototype_parent),
                native: None,
                extensible: None,
            },
            attributes: Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        })
    }

    /// Records native metadata without adding callable default properties.
    ///
    /// This supports intrinsic shells with explicitly authored length/name attributes.
    pub(crate) fn native_function(
        mut self,
        function: BuiltinFunctionPointer,
        constructor: Option<ConstructorKind>,
    ) -> Self {
        self.target.native = Some(PreparedBuiltinFunction::new(
            function,
            constructor,
            self.realm.clone(),
        ));
        self
    }

    pub(crate) fn build(self) -> JsResult<JsObject> {
        let object = self.target.object.clone();
        let target = self.target.prepare(BuiltinTargetMode::Update)?;
        commit_builtin_targets(target, None)?;
        Ok(object)
    }
}

impl<Kind> BuiltInBuilder<'_, Kind> {
    pub(crate) fn static_method<B>(
        mut self,
        function: BuiltinFunctionPointer,
        binding: B,
        length: usize,
    ) -> JsResult<Self>
    where
        B: Into<FunctionBinding>,
    {
        self.target
            .properties
            .method(self.realm, function, binding.into(), length)?;
        Ok(self)
    }

    pub(crate) fn static_property<K, V>(
        mut self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.target
            .properties
            .data(key.into(), value.into(), attribute)?;
        Ok(self)
    }

    pub(crate) fn static_accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        self.target
            .properties
            .accessor(key.into(), get, set, attribute)?;
        Ok(self)
    }

    pub(crate) fn prototype(mut self, prototype: JsObject) -> JsResult<Self> {
        let prototype = Some(prototype);
        check_prototype(self.realm, &prototype)?;
        self.target.prototype = prototype;
        Ok(self)
    }

    /// Records extensibility for the same admitted target commit.
    pub(crate) fn extensible(mut self, extensible: bool) -> Self {
        self.target.extensible = Some(extensible);
        self
    }
}

impl BuiltInBuilder<'_, Callable> {
    pub(crate) const fn length(mut self, length: usize) -> Self {
        self.kind.length = length;
        self
    }

    pub(crate) fn name(mut self, name: JsString) -> Self {
        self.kind.name = name;
        self
    }

    pub(crate) fn build(mut self) -> JsResult<JsFunction> {
        self.target.properties.data(
            StaticJsStrings::LENGTH.into(),
            self.kind.length.into(),
            Attribute::CONFIGURABLE,
        )?;
        self.target.properties.data(
            js_string!("name").into(),
            self.kind.name.into(),
            Attribute::CONFIGURABLE,
        )?;
        self.target.native = Some(PreparedBuiltinFunction::new(
            self.kind.function,
            None,
            self.realm.clone(),
        ));
        let object = self.target.object.clone();
        let target = self.target.prepare(BuiltinTargetMode::Update)?;
        commit_builtin_targets(target, None)?;
        Ok(JsFunction::from_object_unchecked(object))
    }
}

/// A standard constructor has two fresh targets committed as one operation.
pub(crate) struct BuiltInConstructorWithPrototype<'ctx> {
    realm: &'ctx Realm,
    function: BuiltinFunctionPointer,
    name: JsString,
    length: usize,
    constructor: TargetRecipe,
    prototype: TargetRecipe,
    attributes: Attribute,
}

impl BuiltInConstructorWithPrototype<'_> {
    pub(crate) fn static_method<B>(
        mut self,
        function: BuiltinFunctionPointer,
        binding: B,
        length: usize,
    ) -> JsResult<Self>
    where
        B: Into<FunctionBinding>,
    {
        self.constructor
            .properties
            .method(self.realm, function, binding.into(), length)?;
        Ok(self)
    }

    pub(crate) fn static_property<K, V>(
        mut self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.constructor
            .properties
            .data(key.into(), value.into(), attribute)?;
        Ok(self)
    }

    pub(crate) fn static_accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        self.constructor
            .properties
            .accessor(key.into(), get, set, attribute)?;
        Ok(self)
    }

    pub(crate) fn prototype(mut self, prototype: JsObject) -> JsResult<Self> {
        let prototype = Some(prototype);
        check_prototype(self.realm, &prototype)?;
        self.constructor.prototype = prototype;
        Ok(self)
    }

    pub(crate) fn method<B>(
        mut self,
        function: BuiltinFunctionPointer,
        binding: B,
        length: usize,
    ) -> JsResult<Self>
    where
        B: Into<FunctionBinding>,
    {
        self.prototype
            .properties
            .method(self.realm, function, binding.into(), length)?;
        Ok(self)
    }

    pub(crate) fn property<K, V>(mut self, key: K, value: V, attribute: Attribute) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.prototype
            .properties
            .data(key.into(), value.into(), attribute)?;
        Ok(self)
    }

    pub(crate) fn accessor<K>(
        mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        self.prototype
            .properties
            .accessor(key.into(), get, set, attribute)?;
        Ok(self)
    }

    pub(crate) fn inherits(mut self, prototype: JsPrototype) -> JsResult<Self> {
        check_prototype(self.realm, &prototype)?;
        self.prototype.prototype = prototype;
        Ok(self)
    }

    pub(crate) const fn constructor_attributes(mut self, attributes: Attribute) -> Self {
        self.attributes = attributes;
        self
    }

    fn prepare_constructor(&mut self) -> JsResult<()> {
        self.constructor.properties.data(
            js_string!("length").into(),
            self.length.into(),
            Attribute::CONFIGURABLE,
        )?;
        self.constructor.properties.data(
            js_string!("name").into(),
            self.name.clone().into(),
            Attribute::CONFIGURABLE,
        )?;
        self.constructor.native = Some(PreparedBuiltinFunction::new(
            self.function,
            Some(ConstructorKind::Base),
            self.realm.clone(),
        ));
        Ok(())
    }

    pub(crate) fn build(mut self) -> JsResult<()> {
        self.prepare_constructor()?;
        self.constructor.properties.data(
            PROTOTYPE.into(),
            self.prototype.object.clone().into(),
            Attribute::empty(),
        )?;
        self.prototype.properties.data(
            CONSTRUCTOR.into(),
            self.constructor.object.clone().into(),
            self.attributes,
        )?;
        let constructor = self.constructor.prepare(BuiltinTargetMode::Fresh)?;
        let prototype = self.prototype.prepare(BuiltinTargetMode::Fresh)?;
        commit_builtin_targets(constructor, Some(prototype))
    }

    pub(crate) fn build_without_prototype(mut self) -> JsResult<()> {
        self.prepare_constructor()?;
        let constructor = self.constructor.prepare(BuiltinTargetMode::Fresh)?;
        commit_builtin_targets(constructor, None)
    }
}

/// An authored callable recipe acquires its template only when built.
pub(crate) struct BuiltInCallable<'ctx> {
    realm: &'ctx Realm,
    function: BuiltinFunctionPointer,
    name: JsString,
    length: usize,
}

impl BuiltInCallable<'_> {
    pub(crate) const fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub(crate) fn name(mut self, name: JsString) -> Self {
        self.name = name;
        self
    }

    pub(crate) fn build(self) -> JsResult<JsFunction> {
        let object = self.realm.intrinsics().templates().function().create(
            NativeFunctionObject {
                f: crate::NativeFunction::from_builtin(self.function).stored(),
                name: self.name.clone(),
                constructor: None,
                realm: Some(self.realm.edge()),
            },
            [JsValue::new(self.length), JsValue::new(self.name)],
            AllocationOrigin::TrustedConstruction,
        )?;
        Ok(JsFunction::from_object_unchecked(object))
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltInBuilder;
    use crate::{
        Context, JsResult, JsValue, Source,
        builtins::Math,
        error::{EngineError, RootingError},
        js_string,
        property::Attribute,
    };
    use std::cell::Cell;
    use surgeist_js_gc::heap::AccessError;

    #[test]
    fn method_preparation_refusal_stops_later_arguments_and_keeps_target_unchanged() {
        let context = Context::builder().build().unwrap();
        let builder = BuiltInBuilder::with_intrinsic::<Math>(context.realm()).unwrap();
        let target = builder.target.object.clone();
        let later_arguments = Cell::new(0);
        let heap = context.runtime().heap.clone();

        heap.read(|_| {
            let result = (|| -> JsResult<()> {
                builder
                    .static_method(
                        |_, _, _| Ok(JsValue::undefined()),
                        js_string!("unpublishedMethod"),
                        0,
                    )?
                    .static_property(
                        js_string!("unpublishedProperty"),
                        {
                            later_arguments.set(later_arguments.get() + 1);
                            true
                        },
                        Attribute::CONFIGURABLE,
                    )?
                    .build()?;
                Ok(())
            })();
            assert_eq!(
                result.unwrap_err().as_engine(),
                Some(&EngineError::Rooting(RootingError::Access(
                    AccessError::HeapBorrowed,
                )))
            );
        })
        .unwrap();

        assert_eq!(later_arguments.get(), 0);
        assert!(
            target
                .get_property(&js_string!("unpublishedMethod").into())
                .unwrap()
                .is_none()
        );
        assert!(
            target
                .get_property(&js_string!("unpublishedProperty").into())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn builtin_builders_preserve_callable_constructor_and_method_behavior() {
        let mut context = Context::builder().build().unwrap();
        let result = context
            .eval(Source::from_bytes(
                "[Array.of(1, 2).join('-'), Math.max(3, 7),
                  Object.getPrototypeOf(new Array()) === Array.prototype,
                  Object.getPrototypeOf(new Proxy({}, {})) === Object.prototype,
                  Function.prototype.call.call(String, undefined, 42)] .join(',')",
            ))
            .unwrap();
        assert_eq!(result, js_string!("1-2,7,true,true,42").into());
    }
}
