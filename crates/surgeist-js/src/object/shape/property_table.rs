use crate::{
    JsResult, JsValue,
    error::{EngineResourceError, ObjectConstructionError, RootingError},
    object::{
        PreparedNamedProperties,
        shape::slot::{Slot, SlotAttributes},
    },
    property::PropertyKey,
};
use rustc_hash::FxHashMap;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use surgeist_js_gc::heap::AccessError;

/// Ordered named-property metadata. Slot counts are maintained at insertion.
#[derive(Default, Debug, Clone)]
pub(crate) struct PropertyTableInner {
    pub(crate) map: FxHashMap<PropertyKey, Cell<(u32, Slot)>>,
    pub(crate) keys: Vec<(PropertyKey, Slot)>,
    pub(crate) required_slots: u32,
}

impl PropertyTableInner {
    pub(crate) fn with_capacity(capacity: usize) -> JsResult<Self> {
        let mut this = Self::default();
        this.reserve(capacity)?;
        Ok(this)
    }

    pub(crate) fn reserve(&mut self, additional: usize) -> JsResult<()> {
        let count = self
            .keys
            .len()
            .checked_add(additional)
            .ok_or(EngineResourceError::SizeOverflow)?;
        u32::try_from(count).map_err(|_| EngineResourceError::SizeOverflow)?;
        std::alloc::Layout::array::<(PropertyKey, Slot)>(count)
            .map_err(|_| EngineResourceError::SizeOverflow)?;
        self.keys
            .try_reserve(additional)
            .map_err(|_| EngineResourceError::Allocation)?;
        self.map
            .try_reserve(additional)
            .map_err(|_| EngineResourceError::Allocation)?;
        Ok(())
    }

    pub(crate) fn keys(&self) -> JsResult<Vec<PropertyKey>> {
        self.keys_cloned_n(
            u32::try_from(self.keys.len()).map_err(|_| EngineResourceError::SizeOverflow)?,
        )
    }

    pub(crate) fn keys_cloned_n(&self, n: u32) -> JsResult<Vec<PropertyKey>> {
        let entries = self
            .keys
            .get(..n as usize)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(entries.len())
            .map_err(|_| EngineResourceError::Allocation)?;
        result.extend(
            entries
                .iter()
                .filter(|(key, _)| matches!(key, PropertyKey::String(_)))
                .map(|(key, _)| key.clone()),
        );
        result.extend(
            entries
                .iter()
                .filter(|(key, _)| matches!(key, PropertyKey::Symbol(_)))
                .map(|(key, _)| key.clone()),
        );
        Ok(result)
    }

    pub(crate) fn clone_count(&self, n: u32) -> JsResult<Self> {
        let entries = self
            .keys
            .get(..n as usize)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let mut result = Self::with_capacity(entries.len())?;
        for (key, slot) in entries {
            result.insert(key.clone(), slot.attributes)?;
        }
        Ok(result)
    }

    pub(crate) fn insert(&mut self, key: PropertyKey, attributes: SlotAttributes) -> JsResult<()> {
        if matches!(key, PropertyKey::Index(_)) {
            return Err(ObjectConstructionError::IndexedTemplateKey.into());
        }
        if self.map.contains_key(&key) {
            return Err(ObjectConstructionError::DuplicateTemplateProperty.into());
        }
        let index =
            u32::try_from(self.keys.len()).map_err(|_| EngineResourceError::SizeOverflow)?;
        let required_slots = self
            .required_slots
            .checked_add(attributes.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        self.reserve(1)?;
        let slot = Slot {
            index: self.required_slots,
            attributes,
        };
        self.keys.push((key.clone(), slot));
        self.map.insert(key, Cell::new((index, slot)));
        self.required_slots = required_slots;
        Ok(())
    }
}

/// Validated authored metadata, not a live object or mutation capability.
pub(crate) struct NamedLayout {
    table: PropertyTableInner,
}

impl NamedLayout {
    pub(crate) fn new(capacity: usize) -> JsResult<Self> {
        Ok(Self {
            table: PropertyTableInner::with_capacity(capacity)?,
        })
    }

    pub(crate) fn push(&mut self, key: PropertyKey, attributes: SlotAttributes) -> JsResult<()> {
        if attributes.is_accessor_descriptor() && attributes.contains(SlotAttributes::WRITABLE) {
            return Err(ObjectConstructionError::InvalidAccessorLayout.into());
        }
        self.table.insert(key, attributes)
    }

    pub(crate) fn prepare(self, values: Vec<JsValue>) -> JsResult<PreparedNamedProperties> {
        PreparedNamedProperties::new(self.table, values)
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct PropertyTable {
    pub(super) inner: Rc<RefCell<PropertyTableInner>>,
}

impl PropertyTable {
    pub(crate) fn with_capacity(capacity: usize) -> JsResult<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(PropertyTableInner::with_capacity(capacity)?)),
        })
    }
    pub(super) fn inner(&self) -> &RefCell<PropertyTableInner> {
        &self.inner
    }

    pub(crate) fn add_property_deep_clone_if_needed(
        &self,
        key: PropertyKey,
        attributes: SlotAttributes,
        property_count: u32,
    ) -> JsResult<Self> {
        {
            let mut inner = self
                .inner
                .try_borrow_mut()
                .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
            if property_count as usize == inner.keys.len() && !inner.map.contains_key(&key) {
                inner.insert(key, attributes)?;
                return Ok(self.clone());
            }
        }
        let this = self.deep_clone(property_count)?;
        this.inner
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .insert(key, attributes)?;
        Ok(this)
    }
    pub(crate) fn deep_clone(&self, n: u32) -> JsResult<Self> {
        Ok(Self {
            inner: Rc::new(RefCell::new(
                self.inner
                    .try_borrow()
                    .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
                    .clone_count(n)?,
            )),
        })
    }
    pub(crate) fn deep_clone_all(&self) -> JsResult<Self> {
        let inner = self
            .inner
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        let count =
            u32::try_from(inner.keys.len()).map_err(|_| EngineResourceError::SizeOverflow)?;
        Ok(Self {
            inner: Rc::new(RefCell::new(inner.clone_count(count)?)),
        })
    }
    pub(crate) fn set_attributes_at_index(
        &self,
        key: &PropertyKey,
        attributes: SlotAttributes,
    ) -> JsResult<()> {
        let mut inner = self
            .inner
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        let PropertyTableInner { map, keys, .. } = &mut *inner;
        let cell = map
            .get(key)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (index, old) = cell.get();
        if !old.attributes.width_match(attributes) {
            return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
        }
        let entry = keys
            .get_mut(index as usize)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        entry.1.attributes = attributes;
        cell.set((
            index,
            Slot {
                index: old.index,
                attributes,
            },
        ));
        Ok(())
    }
    pub(crate) fn get(&self, key: &PropertyKey) -> JsResult<Slot> {
        let inner = self
            .inner
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        inner
            .map
            .get(key)
            .map(|cell| cell.get().1)
            .ok_or_else(|| ObjectConstructionError::IncompatibleSlotLayout.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js_string;
    #[test]
    fn cloned_table_cells_have_independent_attribute_and_index_storage() {
        let original = PropertyTable::with_capacity(1).unwrap();
        let key = PropertyKey::from(js_string!("value"));
        original
            .inner
            .borrow_mut()
            .insert(key.clone(), SlotAttributes::CONFIGURABLE)
            .unwrap();
        let copied = original.inner.borrow().clone();
        let cloned = PropertyTable {
            inner: Rc::new(RefCell::new(copied)),
        };
        original
            .set_attributes_at_index(&key, SlotAttributes::WRITABLE)
            .unwrap();
        assert_eq!(
            original.get(&key).unwrap().attributes,
            SlotAttributes::WRITABLE
        );
        assert_eq!(
            cloned.get(&key).unwrap().attributes,
            SlotAttributes::CONFIGURABLE
        );
        assert_eq!(
            cloned.inner.borrow().keys.first().unwrap().1.attributes,
            SlotAttributes::CONFIGURABLE
        );
        cloned
            .set_attributes_at_index(&key, SlotAttributes::ENUMERABLE)
            .unwrap();
        assert_eq!(
            original.get(&key).unwrap().attributes,
            SlotAttributes::WRITABLE
        );
        assert_eq!(
            cloned.get(&key).unwrap().attributes,
            SlotAttributes::ENUMERABLE
        );
    }
}
