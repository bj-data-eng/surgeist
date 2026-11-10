use super::{SharedShape, SharedShapeRef, TransitionKey};
use crate::object::NamedSlotSource;
use crate::{
    JsResult, Runtime,
    error::{ObjectConstructionError, RootingError},
    object::{
        AllocationOrigin, IndexedProperties, JsObject, NativeObject, Object, PreparedNamedSlots,
        PropertyMap, shape::slot::SlotAttributes,
    },
    property::{Attribute, PropertyKey},
};
use surgeist_js_gc::heap::Trace as HeapTrace;

#[derive(Clone)]
pub(crate) struct ObjectTemplateBuilder {
    shape: SharedShape,
}
impl ObjectTemplateBuilder {
    pub(crate) fn new(root: &crate::object::shape::RootShape) -> Self {
        Self {
            shape: root.shape().clone(),
        }
    }
    pub(crate) fn with_prototype(
        root: &crate::object::shape::RootShape,
        prototype: JsObject,
    ) -> JsResult<Self> {
        Ok(Self {
            shape: root.shape().change_prototype_transition(Some(prototype))?,
        })
    }
    pub(crate) fn set_prototype(&mut self, prototype: JsObject) -> JsResult<&mut Self> {
        self.shape = self.shape.change_prototype_transition(Some(prototype))?;
        Ok(self)
    }
    pub(crate) fn property(
        &mut self,
        key: PropertyKey,
        attributes: Attribute,
    ) -> JsResult<&mut Self> {
        self.insert(key, SlotAttributes::from_bits_truncate(attributes.bits()))
    }
    fn insert(&mut self, key: PropertyKey, attributes: SlotAttributes) -> JsResult<&mut Self> {
        if matches!(key, PropertyKey::Index(_)) {
            return Err(ObjectConstructionError::IndexedTemplateKey.into());
        }
        if self.shape.lookup(&key)?.is_some() {
            return Err(ObjectConstructionError::DuplicateTemplateProperty.into());
        }
        self.shape = self.shape.insert_property_transition(TransitionKey {
            property_key: key,
            attributes,
        })?;
        Ok(self)
    }
    pub(crate) fn accessor(
        &mut self,
        key: PropertyKey,
        get: bool,
        set: bool,
        attributes: Attribute,
    ) -> JsResult<&mut Self> {
        if !get && !set {
            return Err(ObjectConstructionError::InvalidAccessorLayout.into());
        }
        let mut flags = SlotAttributes::empty();
        flags.set(
            SlotAttributes::CONFIGURABLE,
            attributes.contains(Attribute::CONFIGURABLE),
        );
        flags.set(
            SlotAttributes::ENUMERABLE,
            attributes.contains(Attribute::ENUMERABLE),
        );
        flags.set(SlotAttributes::GET, get);
        flags.set(SlotAttributes::SET, set);
        self.insert(key, flags)
    }
    pub(crate) fn build(
        self,
        roots: &mut crate::realm::BootstrapRoots,
    ) -> JsResult<ObjectTemplate> {
        let shape = SharedShapeRef::from(&self.shape);
        roots.retain_shape(self.shape)?;
        Ok(ObjectTemplate { shape })
    }
}

#[derive(Debug, HeapTrace)]
pub(crate) struct ObjectTemplate {
    shape: SharedShapeRef,
}
impl ObjectTemplate {
    pub(crate) fn shape(&self) -> Result<SharedShape, RootingError> {
        self.shape.upgrade()?.ok_or(RootingError::Expired)
    }
    pub(crate) fn has_prototype(&self, prototype: &JsObject) -> Result<bool, RootingError> {
        Ok(self.shape()?.has_prototype(prototype))
    }
    pub(crate) fn create<T: NativeObject>(
        &self,
        data: T,
        storage: impl NamedSlotSource,
        origin: AllocationOrigin,
    ) -> JsResult<JsObject> {
        Self::create_on_shape(
            self.shape()?,
            data,
            storage,
            IndexedProperties::default(),
            origin,
        )
    }
    pub(crate) fn create_unpublished<T: NativeObject>(
        &self,
        data: T,
        storage: impl NamedSlotSource,
        origin: AllocationOrigin,
    ) -> JsResult<crate::object::UnpublishedObject<T>> {
        let shape = self.shape()?;
        let runtime = Runtime {
            heap: shape.heap().clone(),
        };
        let mut slots = PreparedNamedSlots::new(storage, shape.required_slots()?)?;
        let internal_methods = data.internal_methods();
        let object = Self::object_on_shape(shape, data, &mut slots, IndexedProperties::default())?;
        crate::object::UnpublishedObject::from_object(&runtime, object, internal_methods, origin)
    }
    pub(crate) fn create_with_prototype<T: NativeObject>(
        &self,
        prototype: JsObject,
        data: T,
        storage: impl NamedSlotSource,
        origin: AllocationOrigin,
    ) -> JsResult<JsObject> {
        let shape = self.shape()?;
        let runtime = Runtime {
            heap: shape.heap().clone(),
        };
        // Payload capture edges must remain roots during the GC allocation used
        // to prepare a different shape, before the final object exists.
        runtime
            .heap
            .with_pending_edges(data, |data| {
                let shape = shape
                    .change_prototype_transition(Some(prototype))
                    .map_err(|error| origin.shape_error(error))?;
                Self::create_on_shape(shape, data, storage, IndexedProperties::default(), origin)
            })
            .map_err(|error| origin.access_error(error))?
    }
    pub(crate) fn create_with_indexed_properties<T: NativeObject>(
        &self,
        data: T,
        storage: impl NamedSlotSource,
        indexed_properties: IndexedProperties,
        origin: AllocationOrigin,
    ) -> JsResult<JsObject> {
        Self::create_on_shape(self.shape()?, data, storage, indexed_properties, origin)
    }
    fn create_on_shape<T: NativeObject>(
        shape: SharedShape,
        data: T,
        storage: impl NamedSlotSource,
        indexed_properties: IndexedProperties,
        origin: AllocationOrigin,
    ) -> JsResult<JsObject> {
        let runtime = Runtime {
            heap: shape.heap().clone(),
        };
        let mut slots = PreparedNamedSlots::new(storage, shape.required_slots()?)?;
        let internal_methods = data.internal_methods();
        let object = Self::object_on_shape(shape, data, &mut slots, indexed_properties)?;
        JsObject::from_object_and_vtable(&runtime, object, internal_methods, origin)
    }
    fn object_on_shape<T: NativeObject>(
        shape: SharedShape,
        data: T,
        slots: &mut PreparedNamedSlots,
        indexed_properties: IndexedProperties,
    ) -> JsResult<Object> {
        let shape = shape.into();
        let mut object = Object::new(PropertyMap::new(&shape, indexed_properties), data);
        object.state.properties.initialize_named_slots(slots)?;
        Ok(object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        builtins::object::OrdinaryObject, error::EngineError, js_string, object::shape::RootShape,
    };
    use surgeist_js_gc::heap::AccessError;

    #[test]
    fn template_heap_refusal_keeps_authoritative_shape_and_allows_same_template_retry() {
        let runtime = Runtime::default();
        let root = RootShape::new(runtime.clone()).unwrap();
        let shape = root
            .shape()
            .insert_property_transition(TransitionKey {
                property_key: js_string!("value").into(),
                attributes: SlotAttributes::CONFIGURABLE,
            })
            .unwrap();
        let template = ObjectTemplate {
            shape: SharedShapeRef::from(&shape),
        };
        runtime
            .heap
            .read(|_| {
                let error = template
                    .create(
                        OrdinaryObject,
                        [crate::JsValue::from(7)],
                        AllocationOrigin::TrustedConstruction,
                    )
                    .unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::Rooting(RootingError::Access(
                        AccessError::HeapBorrowed
                    )))
                );
            })
            .unwrap();
        let object = template
            .create(
                OrdinaryObject,
                [crate::JsValue::from(7)],
                AllocationOrigin::TrustedConstruction,
            )
            .unwrap();
        assert_eq!(
            object
                .get_property(&js_string!("value").into())
                .unwrap()
                .unwrap()
                .value(),
            Some(&crate::JsValue::from(7))
        );
    }
    #[test]
    fn template_storage_count_refusal_does_not_poison_later_creation() {
        let runtime = Runtime::default();
        let root = RootShape::new(runtime).unwrap();
        let shape = root
            .shape()
            .insert_property_transition(TransitionKey {
                property_key: js_string!("value").into(),
                attributes: SlotAttributes::CONFIGURABLE,
            })
            .unwrap();
        let template = ObjectTemplate {
            shape: SharedShapeRef::from(&shape),
        };
        let error = template
            .create(OrdinaryObject, [], AllocationOrigin::TrustedConstruction)
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ObjectConstruction(
                ObjectConstructionError::NamedSlotCount {
                    expected: 1,
                    actual: 0
                }
            ))
        );
        template
            .create(
                OrdinaryObject,
                [crate::JsValue::from(1)],
                AllocationOrigin::TrustedConstruction,
            )
            .unwrap();
    }

    // The authoritative template and weak cache retain the SAME actually collected
    // shape identity, making their distinct expired/miss outcomes directly comparable.
    #[test]
    fn expired_template_refuses_while_stale_transition_cache_rebuilds_the_same_layout() {
        use surgeist_js_gc::heap::AllocationCollection;

        let runtime = Runtime::default();
        runtime
            .heap
            .set_allocation_collection(AllocationCollection::Manual)
            .unwrap();
        let root = RootShape::new(runtime.clone()).unwrap();
        let key = TransitionKey {
            property_key: js_string!("value").into(),
            attributes: SlotAttributes::CONFIGURABLE,
        };
        let shape = root
            .shape()
            .insert_property_transition(key.clone())
            .unwrap();
        let retired_identity = shape.identity();
        let template = ObjectTemplate {
            shape: SharedShapeRef::from(&shape),
        };
        let cached = root
            .shape()
            .forward_transitions()
            .get_property(&key)
            .unwrap()
            .unwrap();
        assert_eq!(cached.identity(), retired_identity);
        drop(shape);
        runtime.collect().unwrap();
        assert!(!cached.is_live().unwrap());

        assert_eq!(template.shape().unwrap_err(), RootingError::Expired);
        let error = template
            .create(
                OrdinaryObject,
                [crate::JsValue::from(11)],
                AllocationOrigin::TrustedConstruction,
            )
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Rooting(RootingError::Expired))
        );

        let replacement = root
            .shape()
            .insert_property_transition(key.clone())
            .unwrap();
        assert_ne!(replacement.identity(), retired_identity);
        let slot = replacement.lookup(&key.property_key).unwrap().unwrap();
        assert_eq!(slot.index, 0);
        assert_eq!(slot.attributes, SlotAttributes::CONFIGURABLE);
        assert_eq!(
            root.shape()
                .forward_transitions()
                .get_property(&key)
                .unwrap()
                .unwrap()
                .identity(),
            replacement.identity()
        );
        // A replacement cache entry must not silently rebind the stale authoritative
        // template to a different allocation or generation.
        assert_eq!(template.shape().unwrap_err(), RootingError::Expired);
        let rebuilt = ObjectTemplate {
            shape: SharedShapeRef::from(&replacement),
        };
        let object = rebuilt
            .create(
                OrdinaryObject,
                [crate::JsValue::from(11)],
                AllocationOrigin::TrustedConstruction,
            )
            .unwrap();
        assert_eq!(
            object
                .get_property(&key.property_key)
                .unwrap()
                .unwrap()
                .value(),
            Some(&crate::JsValue::from(11))
        );
    }
}
