use surgeist_js_gc::heap::Trace as HeapTrace;

use crate::{
    JsValue, Runtime,
    object::{
        IndexedProperties, JsObject, NativeObject, Object, PropertyMap, shape::slot::SlotAttributes,
    },
    property::{Attribute, PropertyKey},
};

use super::{SharedShape, SharedShapeRef, TransitionKey};

/// Prepares a template of an object's properties and prototype.
/// This is used to construct as many objects  as needed from a predefined [`SharedShape`].
#[derive(Clone)]
pub(crate) struct ObjectTemplateBuilder {
    shape: SharedShape,
}

impl ObjectTemplateBuilder {
    /// Create a new [`ObjectTemplate`]
    pub(crate) fn new(root: &crate::object::shape::RootShape) -> Self {
        Self {
            shape: root.shape().clone(),
        }
    }

    /// Create and [`ObjectTemplate`] with a prototype.
    pub(crate) fn with_prototype(
        root: &crate::object::shape::RootShape,
        prototype: JsObject,
    ) -> Self {
        let shape = root.shape().change_prototype_transition(Some(prototype));
        Self { shape }
    }

    /// Set the prototype of the [`ObjectTemplate`].
    ///
    /// This assumes that the prototype has not been set yet.
    pub(crate) fn set_prototype(&mut self, prototype: JsObject) -> &mut Self {
        self.shape = self.shape.change_prototype_transition(Some(prototype));
        self
    }

    /// Add a data property to the [`ObjectTemplate`].
    ///
    /// This assumes that the property with the given key was not previously set
    /// and that it's a string or symbol.
    pub(crate) fn property(&mut self, key: PropertyKey, attributes: Attribute) -> &mut Self {
        debug_assert!(!matches!(&key, PropertyKey::Index(_)));

        let attributes = SlotAttributes::from_bits_truncate(attributes.bits());
        self.shape = self.shape.insert_property_transition(TransitionKey {
            property_key: key,
            attributes,
        });
        self
    }

    /// Add a accessor property to the [`ObjectTemplate`].
    ///
    /// This assumes that the property with the given key was not previously set
    /// and that it's a string or symbol.
    pub(crate) fn accessor(
        &mut self,
        key: PropertyKey,
        get: bool,
        set: bool,
        attributes: Attribute,
    ) -> &mut Self {
        // TODO: We don't support indexed keys.
        debug_assert!(!matches!(&key, PropertyKey::Index(_)));

        let attributes = {
            let mut result = SlotAttributes::empty();
            result.set(
                SlotAttributes::CONFIGURABLE,
                attributes.contains(Attribute::CONFIGURABLE),
            );
            result.set(
                SlotAttributes::ENUMERABLE,
                attributes.contains(Attribute::ENUMERABLE),
            );

            result.set(SlotAttributes::GET, get);
            result.set(SlotAttributes::SET, set);

            result
        };

        self.shape = self.shape.insert_property_transition(TransitionKey {
            property_key: key,
            attributes,
        });
        self
    }

    /// Retain the prepared shape until its containing realm is published.
    pub(crate) fn build(self, roots: &mut crate::realm::BootstrapRoots) -> ObjectTemplate {
        let shape = SharedShapeRef::from(&self.shape);
        roots.retain_shape(self.shape);
        ObjectTemplate { shape }
    }
}

/// A prepared object shape stored in its realm.
#[derive(Debug, HeapTrace)]
pub(crate) struct ObjectTemplate {
    shape: SharedShapeRef,
}

impl ObjectTemplate {
    pub(crate) fn shape(&self) -> SharedShape {
        self.shape
            .upgrade()
            .expect("live realm retains its template")
    }

    pub(crate) fn has_prototype(&self, prototype: &JsObject) -> bool {
        self.shape().has_prototype(prototype)
    }

    pub(crate) fn create<T: NativeObject>(&self, data: T, storage: Vec<JsValue>) -> JsObject {
        Self::create_on_shape(self.shape(), data, storage, IndexedProperties::default())
    }

    pub(crate) fn try_create<T: NativeObject>(
        &self,
        data: T,
        storage: Vec<JsValue>,
    ) -> crate::JsResult<JsObject> {
        let shape = self.shape();
        let runtime = Runtime {
            heap: shape.heap().clone(),
        };
        let internal_methods = data.internal_methods();
        let object = Self::object_on_shape(shape, data, storage, IndexedProperties::default());
        JsObject::try_from_object_and_vtable(&runtime, object, internal_methods)
    }

    pub(crate) fn create_with_prototype<T: NativeObject>(
        &self,
        prototype: JsObject,
        data: T,
        storage: Vec<JsValue>,
    ) -> JsObject {
        let shape = self.shape().change_prototype_transition(Some(prototype));
        Self::create_on_shape(shape, data, storage, IndexedProperties::default())
    }

    /// Create an object from the [`ObjectTemplate`]
    ///
    /// The storage must match the properties provided. It does not apply to
    /// the indexed properties.
    pub(crate) fn create_with_indexed_properties<T: NativeObject>(
        &self,
        data: T,
        storage: Vec<JsValue>,
        indexed_properties: IndexedProperties,
    ) -> JsObject {
        Self::create_on_shape(self.shape(), data, storage, indexed_properties)
    }

    fn create_on_shape<T: NativeObject>(
        shape: SharedShape,
        data: T,
        storage: Vec<JsValue>,
        indexed_properties: IndexedProperties,
    ) -> JsObject {
        let runtime = Runtime {
            heap: shape.heap().clone(),
        };
        let internal_methods = data.internal_methods();
        let object = Self::object_on_shape(shape, data, storage, indexed_properties);
        JsObject::from_object_and_vtable(&runtime, object, internal_methods)
    }

    fn object_on_shape<T: NativeObject>(
        shape: SharedShape,
        data: T,
        storage: Vec<JsValue>,
        indexed_properties: IndexedProperties,
    ) -> Object {
        let shape = shape.into();
        let mut object = Object::new(PropertyMap::new(&shape, indexed_properties), data);

        object.state.properties.initialize_named_slots(storage);
        object
    }
}
