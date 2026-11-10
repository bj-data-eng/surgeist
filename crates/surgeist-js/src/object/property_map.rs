use super::{
    JsPrototype, PropertyDescriptor, PropertyKey,
    shape::{
        ChangeTransitionAction, Shape, ShapeRef,
        shared_shape::TransitionKey,
        slot::{Slot, SlotAttributes},
    },
};
use crate::value::{JsVariant, StoredValue};
use crate::{
    JsResult,
    error::{EngineResourceError, ObjectConstructionError, RootingError},
    native_function::{CapturedValue, NativeCapture},
    object::shape::property_table::PropertyTableInner,
};
use crate::{
    JsValue,
    property::{PropertyDescriptorBuilder, StoredPropertyDescriptor},
};
use rustc_hash::FxHashMap;
use std::{collections::hash_map, iter::FusedIterator};
use surgeist_js_gc::heap::Trace as HeapTrace;
use thin_vec::ThinVec;

/// This represents all the indexed properties.
///
/// The index properties can be stored in two storage methods:
///
/// ## Dense Storage
///
/// Dense storage holds a contiguous array of properties where the index in the array is the key of the property.
/// These are known to be data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// Since we know the properties of the property descriptors (and they are all the same) we can omit it and just store only
/// the value field and construct the data property descriptor on demand.
///
/// ## Sparse Storage
///
/// This storage is used as a backup if the element keys are not continuous or the property descriptors
/// are not data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// This method uses more space, since we also have to store the property descriptors, not just the value.
/// It is also slower because we need to do a hash lookup.
#[derive(Debug, HeapTrace)]
pub(crate) enum IndexedProperties {
    /// Dense [`i32`] storage.
    DenseI32(ThinVec<i32>),

    /// Dense [`f64`] storage.
    DenseF64(ThinVec<f64>),

    /// Dense [`JsValue`] storage.
    DenseElement(ThinVec<StoredValue>),

    /// Sparse storage that only keeps element values.
    SparseElement(Box<FxHashMap<u32, StoredValue>>),

    /// Sparse storage that keeps full property descriptors.
    SparseProperty(Box<FxHashMap<u32, StoredPropertyDescriptor>>),
}

impl Default for IndexedProperties {
    #[inline]
    fn default() -> Self {
        Self::DenseI32(ThinVec::new())
    }
}

impl IndexedProperties {
    pub(crate) fn from_dense_js_value(elements: ThinVec<JsValue>) -> Self {
        if elements.is_empty() {
            return Self::default();
        }
        Self::DenseElement(elements.into_iter().map(JsValue::into_stored).collect())
    }

    #[inline]
    fn property_simple_value(property: &PropertyDescriptor) -> Option<&JsValue> {
        if property.writable().unwrap_or(false)
            && property.enumerable().unwrap_or(false)
            && property.configurable().unwrap_or(false)
        {
            property.value()
        } else {
            None
        }
    }

    /// Get a property descriptor if it exists.
    fn get(&self, key: u32) -> Option<PropertyDescriptor> {
        let value = match self {
            Self::DenseI32(vec) => vec.get(key as usize).copied()?.into(),
            Self::DenseF64(vec) => vec.get(key as usize).copied()?.into(),
            Self::DenseElement(vec) => vec.get(key as usize)?.to_value(),
            Self::SparseElement(map) => map.get(&key)?.to_value(),
            Self::SparseProperty(map) => {
                return map.get(&key).map(StoredPropertyDescriptor::to_descriptor);
            }
        };

        Some(
            PropertyDescriptorBuilder::new()
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .value(value)
                .build(),
        )
    }

    fn convert_dense_to_sparse<T: Into<StoredValue>>(
        vec: &mut ThinVec<T>,
    ) -> FxHashMap<u32, StoredPropertyDescriptor> {
        std::mem::take(vec)
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index as u32, StoredPropertyDescriptor::simple(value.into())))
            .collect()
    }

    fn convert_dense_to_sparse_values<T: Into<StoredValue>>(
        vec: &mut ThinVec<T>,
    ) -> FxHashMap<u32, StoredValue> {
        std::mem::take(vec)
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index as u32, value.into()))
            .collect()
    }

    fn convert_to_sparse_and_insert(&mut self, key: u32, property: &PropertyDescriptor) -> bool {
        let mut descriptors = match self {
            Self::DenseI32(vec) => Self::convert_dense_to_sparse(vec),
            Self::DenseF64(vec) => Self::convert_dense_to_sparse(vec),
            Self::DenseElement(vec) => Self::convert_dense_to_sparse(vec),
            Self::SparseElement(map) => std::mem::take(map.as_mut())
                .into_iter()
                .map(|(index, value)| (index, StoredPropertyDescriptor::simple(value)))
                .collect(),
            Self::SparseProperty(map) => {
                return map.insert(key, property.to_stored()).is_some();
            }
        };

        let replaced = descriptors.insert(key, property.to_stored()).is_some();
        *self = Self::SparseProperty(Box::new(descriptors));
        replaced
    }

    /// Inserts a property descriptor with the specified key.
    fn insert(&mut self, key: u32, property: PropertyDescriptor) -> bool {
        let Some(value) = Self::property_simple_value(&property) else {
            return self.convert_to_sparse_and_insert(key, &property);
        };

        match self {
            // Fast Path: contiguous array access without creating holes.
            Self::DenseI32(vec) if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                if let Some(val) = value.as_i32() {
                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(val);
                        return false;
                    }

                    // If the key points at an already taken index, set it.
                    vec[key as usize] = val;
                    return true;
                }

                if let Some(num) = value.as_number() {
                    let mut vec = vec.iter().copied().map(f64::from).collect::<ThinVec<_>>();

                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(num);
                        *self = Self::DenseF64(vec);
                        return false;
                    }

                    // If the key points at an already taken index, set it.
                    vec[key as usize] = num;
                    *self = Self::DenseF64(vec);
                    return true;
                }

                let mut vec = vec
                    .iter()
                    .copied()
                    .map(StoredValue::from)
                    .collect::<ThinVec<StoredValue>>();

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    vec.push(value.clone().into_stored());
                    *self = Self::DenseElement(vec);
                    return false;
                }

                // If the key points at an already taken index, set it.
                vec[key as usize] = value.clone().into_stored();
                *self = Self::DenseElement(vec);
                true
            }
            Self::DenseF64(vec) if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                if let Some(num) = value.as_number() {
                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                    if key == len {
                        vec.push(num);
                        return false;
                    }

                    // If the key points at an already taken index, swap and return it.
                    vec[key as usize] = num;
                    return true;
                }

                let mut vec = vec
                    .iter()
                    .copied()
                    .map(StoredValue::from)
                    .collect::<ThinVec<StoredValue>>();

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    vec.push(value.clone().into_stored());
                    *self = Self::DenseElement(vec);
                    return false;
                }

                // If the key points at an already taken index, set it.
                vec[key as usize] = value.clone().into_stored();
                *self = Self::DenseElement(vec);
                true
            }
            Self::DenseElement(vec) if key <= vec.len() as u32 => {
                let len = vec.len() as u32;

                // If the key is pointing one past the last element, we push it!
                //
                // Since the previous key is the current key - 1. Meaning that the elements are continuous.
                if key == len {
                    vec.push(value.clone().into_stored());
                    return false;
                }

                // If the key points at an already taken index, set it.
                vec[key as usize] = value.clone().into_stored();
                true
            }
            // Creating a hole: transition dense storage into sparse element storage.
            Self::DenseI32(vec) => {
                let mut values = Self::convert_dense_to_sparse_values(vec);
                let replaced = values.insert(key, value.clone().into_stored()).is_some();
                *self = Self::SparseElement(Box::new(values));
                replaced
            }
            Self::DenseF64(vec) => {
                let mut values = Self::convert_dense_to_sparse_values(vec);
                let replaced = values.insert(key, value.clone().into_stored()).is_some();
                *self = Self::SparseElement(Box::new(values));
                replaced
            }
            Self::DenseElement(vec) => {
                let mut values = Self::convert_dense_to_sparse_values(vec);
                let replaced = values.insert(key, value.clone().into_stored()).is_some();
                *self = Self::SparseElement(Box::new(values));
                replaced
            }
            Self::SparseElement(map) => map.insert(key, value.clone().into_stored()).is_some(),
            Self::SparseProperty(map) => {
                let descriptor = StoredPropertyDescriptor::simple(value.clone().into_stored());
                map.insert(key, descriptor).is_some()
            }
        }
    }

    fn convert_to_sparse_and_remove(&mut self, key: u32) -> bool {
        let mut descriptors = match self {
            IndexedProperties::DenseI32(vec) => Self::convert_dense_to_sparse_values(vec),
            IndexedProperties::DenseF64(vec) => Self::convert_dense_to_sparse_values(vec),
            IndexedProperties::DenseElement(vec) => Self::convert_dense_to_sparse_values(vec),
            IndexedProperties::SparseProperty(map) => return map.remove(&key).is_some(),
            IndexedProperties::SparseElement(map) => {
                return map.remove(&key).is_some();
            }
        };

        let removed = descriptors.remove(&key).is_some();
        *self = Self::SparseElement(Box::new(descriptors));
        removed
    }

    /// Removes a property descriptor with the specified key.
    fn remove(&mut self, key: u32) -> bool {
        match self {
            // Fast Paths: contiguous storage.
            //
            // If the key is pointing at the last element, then we pop it.
            Self::DenseI32(vec) if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            // If the key is out of range then don't do anything.
            Self::DenseI32(vec) if key >= vec.len() as u32 => false,
            Self::DenseF64(vec) if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            Self::DenseF64(vec) if key >= vec.len() as u32 => false,
            Self::DenseElement(vec) if (key + 1) == vec.len() as u32 => {
                vec.pop();
                true
            }
            Self::DenseElement(vec) if key >= vec.len() as u32 => false,
            // Slow Paths: non-contiguous storage.
            Self::SparseElement(map) => map.remove(&key).is_some(),
            Self::SparseProperty(map) => map.remove(&key).is_some(),
            _ => self.convert_to_sparse_and_remove(key),
        }
    }

    /// Check if we contain the key to a property descriptor.
    fn contains_key(&self, key: u32) -> bool {
        match self {
            Self::DenseI32(vec) => (0..vec.len() as u32).contains(&key),
            Self::DenseF64(vec) => (0..vec.len() as u32).contains(&key),
            Self::DenseElement(vec) => (0..vec.len() as u32).contains(&key),
            Self::SparseElement(map) => map.contains_key(&key),
            Self::SparseProperty(map) => map.contains_key(&key),
        }
    }

    /// Pushes a value to the end of the dense indexed properties.
    ///
    /// Returns `true` if the push succeeded (storage is dense), `false` if
    /// the storage is sparse and the caller should fall back to the slow path.
    ///
    /// Handles type transitions: `DenseI32` → `DenseF64` → `DenseElement`.
    pub(crate) fn push_dense(&mut self, value: &JsValue) -> bool {
        match self {
            Self::DenseI32(vec) => {
                if let Some(i) = value.as_i32() {
                    vec.push(i);
                } else if let Some(n) = value.as_number() {
                    let mut new_vec: ThinVec<f64> = vec.iter().copied().map(f64::from).collect();
                    new_vec.push(n);
                    *self = Self::DenseF64(new_vec);
                } else {
                    let mut new_vec: ThinVec<StoredValue> =
                        vec.iter().copied().map(StoredValue::from).collect();
                    new_vec.push(value.clone().into_stored());
                    *self = Self::DenseElement(new_vec);
                }
                true
            }
            Self::DenseF64(vec) => {
                if let Some(n) = value.as_number() {
                    vec.push(n);
                } else {
                    let mut new_vec: ThinVec<StoredValue> =
                        vec.iter().copied().map(StoredValue::from).collect();
                    new_vec.push(value.clone().into_stored());
                    *self = Self::DenseElement(new_vec);
                }
                true
            }
            Self::DenseElement(vec) => {
                vec.push(value.clone().into_stored());
                true
            }
            Self::SparseElement(_) | Self::SparseProperty(_) => false,
        }
    }

    /// Transform this array into a sparse array if it isn't so already.
    pub(crate) fn transform_to_sparse(&mut self) {
        match &*self {
            Self::DenseI32(v) => {
                *self = Self::SparseElement(Box::new(
                    v.into_iter()
                        .copied()
                        .enumerate()
                        .map(|(i, v)| (i as u32, StoredValue::from(v)))
                        .collect(),
                ));
            }
            Self::DenseF64(v) => {
                *self = Self::SparseElement(Box::new(
                    v.into_iter()
                        .copied()
                        .enumerate()
                        .map(|(i, v)| (i as u32, StoredValue::from(v)))
                        .collect(),
                ));
            }
            Self::DenseElement(v) => {
                *self = Self::SparseElement(Box::new(
                    v.into_iter()
                        .enumerate()
                        .map(|(i, v)| (i as u32, v.clone()))
                        .collect(),
                ));
            }
            _ => {}
        }
    }

    fn iter(&self) -> IndexProperties<'_> {
        match self {
            Self::DenseI32(vec) => IndexProperties::DenseI32(vec.iter().enumerate()),
            Self::DenseF64(vec) => IndexProperties::DenseF64(vec.iter().enumerate()),
            Self::DenseElement(vec) => IndexProperties::DenseElement(vec.iter().enumerate()),
            Self::SparseElement(map) => IndexProperties::SparseElement(map.iter()),
            Self::SparseProperty(map) => IndexProperties::SparseProperty(map.iter()),
        }
    }

    fn keys(&self) -> IndexPropertyKeys<'_> {
        match self {
            Self::DenseI32(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::DenseF64(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::DenseElement(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::SparseElement(map) => IndexPropertyKeys::SparseElement(map.keys()),
            Self::SparseProperty(map) => IndexPropertyKeys::SparseProperty(map.keys()),
        }
    }

    fn values(&self) -> IndexPropertyValues<'_> {
        match self {
            Self::DenseI32(vec) => IndexPropertyValues::DenseI32(vec.iter()),
            Self::DenseF64(vec) => IndexPropertyValues::DenseF64(vec.iter()),
            Self::DenseElement(vec) => IndexPropertyValues::DenseElement(vec.iter()),
            Self::SparseElement(map) => IndexPropertyValues::SparseElement(map.values()),
            Self::SparseProperty(map) => IndexPropertyValues::SparseProperty(map.values()),
        }
    }
}

impl<'a> IntoIterator for &'a IndexedProperties {
    type IntoIter = IndexProperties<'a>;
    type Item = (u32, PropertyDescriptor);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`PropertyMap`] contains all the properties of an object.
///
/// The property values are stored in different data structures based on keys.
#[derive(Debug, HeapTrace)]
pub struct PropertyMap {
    /// Properties stored with integers as keys.
    pub(crate) indexed_properties: IndexedProperties,

    shape: ShapeRef,
    storage: Vec<StoredValue>,
}

impl PropertyMap {
    /// Create a new [`PropertyMap`].
    #[must_use]
    #[inline]
    pub(crate) fn new(shape: &Shape, indexed_properties: IndexedProperties) -> Self {
        Self {
            indexed_properties,
            shape: ShapeRef::from(shape),
            storage: Vec::default(),
        }
    }

    pub(crate) fn shape(&self) -> JsResult<Shape> {
        self.shape
            .upgrade()
            .map_err(RootingError::Access)?
            .ok_or_else(|| RootingError::Expired.into())
    }
    pub(crate) fn set_prototype(&mut self, prototype: JsPrototype) -> JsResult<()> {
        self.shape = ShapeRef::from(&self.shape()?.change_prototype_transition(prototype)?);
        Ok(())
    }
    pub(crate) fn named_slot(&self, index: usize) -> JsResult<JsValue> {
        let value = self
            .storage
            .get(index)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        Ok(JsValue::restore(&CapturedValue(value.clone()))?)
    }
    pub(crate) fn write_named_slot(&mut self, index: usize, value: JsValue) -> JsResult<()> {
        *self
            .storage
            .get_mut(index)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)? = value.into_stored();
        Ok(())
    }
    pub(crate) fn write_named_object_edge(
        &mut self,
        index: usize,
        value: super::ObjectEdge,
    ) -> JsResult<()> {
        *self
            .storage
            .get_mut(index)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)? = StoredValue::Object(value);
        Ok(())
    }
    pub(crate) fn initialize_named_slots(
        &mut self,
        slots: &mut PreparedNamedSlots,
    ) -> JsResult<()> {
        if !self.storage.is_empty() {
            return Err(ObjectConstructionError::NamedSlotsAlreadyInitialized.into());
        }
        self.storage = std::mem::take(&mut slots.storage);
        Ok(())
    }
    pub(crate) fn same_shape(&self, shape: &Shape) -> bool {
        self.shape == ShapeRef::from(shape)
    }
    pub(crate) fn named_storage_mut(&mut self) -> &mut Vec<StoredValue> {
        &mut self.storage
    }

    pub fn get(&self, key: &PropertyKey) -> JsResult<Option<PropertyDescriptor>> {
        if let PropertyKey::Index(index) = key {
            return Ok(self.indexed_properties.get(index.get()));
        }
        self.shape()?
            .lookup(key)?
            .map(|slot| self.get_storage(slot))
            .transpose()
    }
    pub(crate) fn get_with_slot(
        &self,
        key: &PropertyKey,
        out_slot: &mut Slot,
    ) -> JsResult<Option<PropertyDescriptor>> {
        if let PropertyKey::Index(index) = key {
            return Ok(self.indexed_properties.get(index.get()));
        }
        if let Some(slot) = self.shape()?.lookup(key)? {
            let descriptor = self.get_storage(slot)?;
            out_slot.index = slot.index;
            out_slot.attributes = (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS)
                | slot.attributes
                | SlotAttributes::FOUND;
            return Ok(Some(descriptor));
        }
        Ok(None)
    }
    pub(crate) fn get_storage(
        &self,
        Slot { index, attributes }: Slot,
    ) -> JsResult<PropertyDescriptor> {
        let index = index as usize;
        let mut builder = PropertyDescriptor::builder()
            .configurable(attributes.contains(SlotAttributes::CONFIGURABLE))
            .enumerable(attributes.contains(SlotAttributes::ENUMERABLE));
        if attributes.is_accessor_descriptor() {
            if attributes.has_get() {
                builder = builder.get(self.named_slot(index)?);
            }
            if attributes.has_set() {
                builder = builder.set(
                    self.named_slot(
                        index
                            .checked_add(1)
                            .ok_or(EngineResourceError::SizeOverflow)?,
                    )?,
                );
            }
        } else {
            builder = builder
                .writable(attributes.contains(SlotAttributes::WRITABLE))
                .value(self.named_slot(index)?);
        }
        Ok(builder.build())
    }
    pub fn insert(&mut self, key: &PropertyKey, property: PropertyDescriptor) -> JsResult<bool> {
        self.insert_with_slot(key, property, &mut Slot::new())
    }
    pub(crate) fn insert_with_slot(
        &mut self,
        key: &PropertyKey,
        property: PropertyDescriptor,
        out_slot: &mut Slot,
    ) -> JsResult<bool> {
        if let PropertyKey::Index(index) = key {
            return Ok(self.indexed_properties.insert(index.get(), property));
        }
        let attributes = property.to_slot_attributes();
        let shape = self.shape()?;
        if let Some(slot) = shape.lookup(key)? {
            let index = slot.index as usize;
            let end = index
                .checked_add(slot.width() as usize)
                .ok_or(EngineResourceError::SizeOverflow)?;
            if self.storage.get(index..end).is_none() {
                return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
            }
            if slot.attributes != attributes {
                if !slot.attributes.is_accessor_descriptor() && attributes.is_accessor_descriptor()
                {
                    self.storage
                        .try_reserve(1)
                        .map_err(|_| EngineResourceError::Allocation)?;
                }
                let transition = shape.change_attributes_transition(TransitionKey {
                    property_key: key.clone(),
                    attributes,
                })?;
                match transition.action {
                    ChangeTransitionAction::Nothing => {}
                    ChangeTransitionAction::Remove => {
                        self.storage.remove(index + 1);
                    }
                    ChangeTransitionAction::Insert => {
                        self.storage.insert(index + 1, StoredValue::Undefined);
                    }
                }
                self.shape = ShapeRef::from(&transition.shape);
            }
            if attributes.is_accessor_descriptor() {
                self.write_named_slot(index, property.get().cloned().unwrap_or_default())?;
                self.write_named_slot(index + 1, property.set().cloned().unwrap_or_default())?;
            } else {
                self.write_named_slot(index, property.value().cloned().unwrap_or_default())?;
            }
            out_slot.index = slot.index;
            out_slot.attributes =
                (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;
            return Ok(true);
        }
        let index =
            u32::try_from(self.storage.len()).map_err(|_| EngineResourceError::SizeOverflow)?;
        self.storage
            .try_reserve(attributes.width() as usize)
            .map_err(|_| EngineResourceError::Allocation)?;
        let new_shape = shape.insert_property_transition(TransitionKey {
            property_key: key.clone(),
            attributes,
        })?;
        if new_shape.lookup(key)? != Some(Slot { index, attributes }) {
            return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
        }
        self.shape = ShapeRef::from(&new_shape);
        out_slot.index = index;
        out_slot.attributes =
            (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;
        if attributes.is_accessor_descriptor() {
            self.storage
                .push(property.get().cloned().unwrap_or_default().into_stored());
            self.storage
                .push(property.set().cloned().unwrap_or_default().into_stored());
        } else {
            self.storage
                .push(property.value().cloned().unwrap_or_default().into_stored());
        }
        Ok(false)
    }
    pub fn remove(&mut self, key: &PropertyKey) -> JsResult<bool> {
        if let PropertyKey::Index(index) = key {
            return Ok(self.indexed_properties.remove(index.get()));
        }
        let shape = self.shape()?;
        if let Some(slot) = shape.lookup(key)? {
            let index = slot.index as usize;
            let end = index
                .checked_add(slot.width() as usize)
                .ok_or(EngineResourceError::SizeOverflow)?;
            if self.storage.get(index..end).is_none() {
                return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
            }
            let new_shape = shape.remove_property_transition(key)?;
            self.storage.drain(index..end);
            self.shape = ShapeRef::from(&new_shape);
            return Ok(true);
        }
        Ok(false)
    }

    /// Overrides all the indexed properties, setting it to dense storage.
    pub(crate) fn override_indexed_properties(&mut self, properties: ThinVec<JsValue>) {
        self.indexed_properties = IndexedProperties::DenseElement(
            properties.into_iter().map(JsValue::into_stored).collect(),
        );
    }

    pub(crate) fn get_dense_property(&self, index: u32) -> Option<JsValue> {
        let index = index as usize;
        match &self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                properties.get(index).copied().map(JsValue::from)
            }
            IndexedProperties::DenseF64(properties) => {
                properties.get(index).copied().map(JsValue::from)
            }
            IndexedProperties::DenseElement(properties) => {
                properties.get(index).map(StoredValue::to_value)
            }
            IndexedProperties::SparseProperty(_) | IndexedProperties::SparseElement(_) => None,
        }
    }

    pub(crate) fn set_dense_property(&mut self, index: u32, value: &JsValue) -> bool {
        let index = index as usize;

        match &mut self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                let Some(element) = properties.get_mut(index) else {
                    return false;
                };

                // If it can fit in a i32 and the truncated version is
                // equal to the original then it is an integer.
                let is_rational_integer = |n: f64| n.to_bits() == f64::from(n as i32).to_bits();

                let value = match value.variant() {
                    JsVariant::Integer32(n) => n,
                    JsVariant::Float64(n) if is_rational_integer(n) => n as i32,
                    JsVariant::Float64(value) => {
                        let mut properties = properties
                            .iter()
                            .copied()
                            .map(f64::from)
                            .collect::<ThinVec<_>>();
                        properties[index] = value;
                        self.indexed_properties = IndexedProperties::DenseF64(properties);

                        return true;
                    }
                    _ => {
                        let mut properties = properties
                            .iter()
                            .copied()
                            .map(StoredValue::from)
                            .collect::<ThinVec<_>>();
                        properties[index] = value.clone().into_stored();
                        self.indexed_properties = IndexedProperties::DenseElement(properties);

                        return true;
                    }
                };

                *element = value;
                true
            }
            IndexedProperties::DenseF64(properties) => {
                let Some(element) = properties.get_mut(index) else {
                    return false;
                };

                let Some(value) = value.as_number() else {
                    let mut properties = properties
                        .iter()
                        .copied()
                        .map(StoredValue::from)
                        .collect::<ThinVec<_>>();
                    properties[index] = value.clone().into_stored();
                    self.indexed_properties = IndexedProperties::DenseElement(properties);
                    return true;
                };

                *element = value;
                true
            }
            IndexedProperties::DenseElement(properties) => {
                let Some(element) = properties.get_mut(index) else {
                    return false;
                };
                *element = value.clone().into_stored();
                true
            }
            IndexedProperties::SparseProperty(_) | IndexedProperties::SparseElement(_) => false,
        }
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn to_dense_indexed_properties(&self) -> Option<ThinVec<JsValue>> {
        match &self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                Some(properties.iter().copied().map(JsValue::from).collect())
            }
            IndexedProperties::DenseF64(properties) => {
                Some(properties.iter().copied().map(JsValue::from).collect())
            }
            IndexedProperties::DenseElement(properties) => {
                Some(properties.iter().map(StoredValue::to_value).collect())
            }
            IndexedProperties::SparseProperty(_) | IndexedProperties::SparseElement(_) => None,
        }
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn dense_indexed_properties_mut(&mut self) -> Option<&mut ThinVec<StoredValue>> {
        if let IndexedProperties::DenseElement(properties) = &mut self.indexed_properties {
            Some(properties)
        } else {
            None
        }
    }

    /// An iterator over indexed keys and their descriptors in arbitrary order.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_properties(
        &self,
    ) -> impl ExactSizeIterator<Item = (u32, PropertyDescriptor)>
    + FusedIterator
    + Clone
    + std::fmt::Debug
    + '_ {
        self.indexed_properties.iter()
    }

    /// An iterator over indexed keys in arbitrary order.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_keys(
        &self,
    ) -> impl ExactSizeIterator<Item = u32> + FusedIterator + Clone + std::fmt::Debug + '_ {
        self.indexed_properties.keys()
    }

    /// An iterator over indexed property descriptors in arbitrary order.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_values(
        &self,
    ) -> impl ExactSizeIterator<Item = PropertyDescriptor> + Clone + std::fmt::Debug + '_ {
        self.indexed_properties.values()
    }

    /// Returns `true` if the given key is contained in the [`PropertyMap`].
    #[inline]
    pub fn contains_key(&self, key: &PropertyKey) -> JsResult<bool> {
        if let PropertyKey::Index(index) = key {
            return Ok(self.indexed_properties.contains_key(index.get()));
        }
        Ok(self.shape()?.lookup(key)?.is_some())
    }
}

/// An iterator over the indexed property entries of an `Object`.
#[derive(Debug, Clone)]
pub(crate) enum IndexProperties<'a> {
    /// An iterator over dense i32, Vec backed indexed property entries of an `Object`.
    DenseI32(std::iter::Enumerate<std::slice::Iter<'a, i32>>),

    /// An iterator over dense f64, Vec backed indexed property entries of an `Object`.
    DenseF64(std::iter::Enumerate<std::slice::Iter<'a, f64>>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::iter::Enumerate<std::slice::Iter<'a, StoredValue>>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing raw values.
    SparseElement(hash_map::Iter<'a, u32, StoredValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing descriptors.
    SparseProperty(hash_map::Iter<'a, u32, StoredPropertyDescriptor>),
}

impl Iterator for IndexProperties<'_> {
    type Item = (u32, PropertyDescriptor);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, value) = match self {
            Self::DenseI32(vec) => vec
                .next()
                .map(|(index, value)| (index, JsValue::from(*value)))?,
            Self::DenseF64(vec) => vec
                .next()
                .map(|(index, value)| (index, JsValue::from(*value)))?,
            Self::DenseElement(vec) => {
                vec.next().map(|(index, value)| (index, value.to_value()))?
            }
            Self::SparseProperty(map) => {
                return map
                    .next()
                    .map(|(index, value)| (*index, value.to_descriptor()));
            }
            Self::SparseElement(map) => map
                .next()
                .map(|(index, value)| (*index as usize, value.to_value()))?,
        };

        Some((
            index as u32,
            PropertyDescriptorBuilder::new()
                .writable(true)
                .configurable(true)
                .enumerable(true)
                .value(value)
                .build(),
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseF64(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseF64(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexProperties<'_> {}

/// An iterator over the index keys (`u32`) of an `Object`.
#[derive(Debug, Clone)]
enum IndexPropertyKeys<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    Dense(std::ops::Range<u32>),

    /// An iterator over sparse, `HashMap` backed indexed property entries of an `Object`.
    SparseProperty(hash_map::Keys<'a, u32, StoredPropertyDescriptor>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing values.
    SparseElement(hash_map::Keys<'a, u32, StoredValue>),
}

impl Iterator for IndexPropertyKeys<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(vec) => vec.next(),
            Self::SparseProperty(map) => map.next().copied(),
            Self::SparseElement(map) => map.next().copied(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Dense(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Dense(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexPropertyKeys<'_> {}

/// An iterator over the index values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
#[allow(variant_size_differences)]
enum IndexPropertyValues<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseI32(std::slice::Iter<'a, i32>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseF64(std::slice::Iter<'a, f64>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::slice::Iter<'a, StoredValue>),

    /// An iterator over sparse, `HashMap` backed indexed property entries of an `Object`.
    SparseProperty(hash_map::Values<'a, u32, StoredPropertyDescriptor>),

    /// An iterator over sparse, `HashMap` backed indexed property entries storing values.
    SparseElement(hash_map::Values<'a, u32, StoredValue>),
}

impl Iterator for IndexPropertyValues<'_> {
    type Item = PropertyDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self {
            Self::DenseI32(vec) => vec.next().copied()?.into(),
            Self::DenseF64(vec) => vec.next().copied()?.into(),
            Self::DenseElement(vec) => vec.next()?.to_value(),
            Self::SparseProperty(map) => {
                return map.next().map(StoredPropertyDescriptor::to_descriptor);
            }
            Self::SparseElement(map) => map.next()?.to_value(),
        };

        Some(
            PropertyDescriptorBuilder::new()
                .writable(true)
                .configurable(true)
                .enumerable(true)
                .value(value)
                .build(),
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseF64(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::SparseProperty(map) => map.size_hint(),
            Self::SparseElement(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseF64(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::SparseProperty(map) => map.len(),
            Self::SparseElement(map) => map.len(),
        }
    }
}

/// Prepared storage retains the original live roots until publication or refusal
/// cleanup finishes. Stored edges alone are not a preparation root bank.
pub(crate) struct PreparedNamedSlots {
    pub(super) storage: Vec<StoredValue>,
    pub(super) _roots: Vec<JsValue>,
}
mod named_slot_source {
    pub trait Sealed {}
    impl Sealed for Vec<crate::JsValue> {}
    impl<const N: usize> Sealed for [crate::JsValue; N] {}
}
/// Closed engine storage sources cannot run authored iterator code during
/// template preparation. Live values remain owned until publication completes.
pub(crate) trait NamedSlotSource: named_slot_source::Sealed {
    fn into_values(self) -> JsResult<Vec<JsValue>>;
}
impl NamedSlotSource for Vec<JsValue> {
    fn into_values(self) -> JsResult<Vec<JsValue>> {
        Ok(self)
    }
}
impl<const N: usize> NamedSlotSource for [JsValue; N] {
    fn into_values(self) -> JsResult<Vec<JsValue>> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(N)
            .map_err(|_| EngineResourceError::Allocation)?;
        values.extend(self);
        Ok(values)
    }
}
impl PreparedNamedSlots {
    pub(crate) fn new(values: impl NamedSlotSource, expected: usize) -> JsResult<Self> {
        let roots = values.into_values()?;
        if roots.len() != expected {
            return Err(ObjectConstructionError::NamedSlotCount {
                expected,
                actual: roots.len(),
            }
            .into());
        }
        let mut storage = Vec::new();
        storage
            .try_reserve_exact(expected)
            .map_err(|_| EngineResourceError::Allocation)?;
        storage.extend(roots.iter().cloned().map(JsValue::into_stored));
        Ok(Self {
            storage,
            _roots: roots,
        })
    }
}

pub(crate) struct PreparedNamedProperties {
    pub(super) table: PropertyTableInner,
    pub(super) slots: PreparedNamedSlots,
}
impl PreparedNamedProperties {
    pub(crate) fn new(table: PropertyTableInner, values: Vec<JsValue>) -> JsResult<Self> {
        let slots = PreparedNamedSlots::new(values, table.required_slots as usize)?;
        Ok(Self { table, slots })
    }
}
