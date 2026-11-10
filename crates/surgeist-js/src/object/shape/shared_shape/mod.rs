mod forward_transition;
pub(crate) mod template;

#[cfg(test)]
mod tests;

use std::{collections::hash_map::RandomState, hash::Hash};

use bitflags::bitflags;
use indexmap::IndexMap;
use surgeist_js_gc::heap::{
    AccessError, ObjectId, RecordRef, Rooted, RuntimeHeap, Trace as HeapTrace,
};

use crate::{
    JsObject, JsResult,
    error::{EngineResourceError, ObjectConstructionError, RootingError},
    object::{JsPrototype, ObjectEdge},
    property::PropertyKey,
};

use self::forward_transition::ForwardTransition;

use super::{
    ChangeTransition, ChangeTransitionAction, Slot, UniqueShape, property_table::PropertyTable,
    slot::SlotAttributes,
};

/// Represent a [`SharedShape`] property transition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TransitionKey {
    pub(crate) property_key: PropertyKey,
    pub(crate) attributes: SlotAttributes,
}

const INSERT_PROPERTY_TRANSITION_TYPE: u8 = 0b0000_0000;
const CONFIGURE_PROPERTY_TRANSITION_TYPE: u8 = 0b0000_0001;
const PROTOTYPE_TRANSITION_TYPE: u8 = 0b0000_0010;

// Reserved for future use!
#[allow(unused)]
const RESERVED_TRANSITION_TYPE: u8 = 0b0000_0011;

bitflags! {
    /// Flags of a shape.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ShapeFlags: u8 {
        /// Represents the transition type of a [`SharedShape`].
        const TRANSITION_TYPE = 0b0000_0011;
    }
}

impl ShapeFlags {
    // NOTE: Remove type bits and set the new ones.
    fn insert_property_transition_from(previous: Self) -> Self {
        previous.difference(Self::TRANSITION_TYPE)
            | Self::from_bits_retain(INSERT_PROPERTY_TRANSITION_TYPE)
    }
    fn configure_property_transition_from(previous: Self) -> Self {
        previous.difference(Self::TRANSITION_TYPE)
            | Self::from_bits_retain(CONFIGURE_PROPERTY_TRANSITION_TYPE)
    }
    fn prototype_transition_from(previous: Self) -> Self {
        previous.difference(Self::TRANSITION_TYPE)
            | Self::from_bits_retain(PROTOTYPE_TRANSITION_TYPE)
    }

    const fn is_insert_transition_type(self) -> bool {
        self.intersection(Self::TRANSITION_TYPE).bits() == INSERT_PROPERTY_TRANSITION_TYPE
    }
    const fn is_prototype_transition_type(self) -> bool {
        self.intersection(Self::TRANSITION_TYPE).bits() == PROTOTYPE_TRANSITION_TYPE
    }
}

/// The internal representation of a [`SharedShape`].
#[derive(Debug, HeapTrace)]
struct Inner {
    /// See [`ForwardTransition`].
    #[heap_trace(skip)]
    forward_transitions: ForwardTransition,

    /// The count of how many properties this [`SharedShape`] holds.
    #[heap_trace(skip)]
    property_count: u32,

    /// Instance prototype `__proto__`.
    prototype: Option<ObjectEdge>,

    // SAFETY: This is safe because nothing in [`PropertyTable`]
    //         needs tracing
    #[heap_trace(skip)]
    property_table: PropertyTable,

    /// The previous shape in the transition chain.
    ///
    /// [`None`] if it is the root shape.
    previous: Option<RecordRef<Inner>>,

    /// How many transitions have happened from the root node.
    #[heap_trace(skip)]
    transition_count: u16,

    /// Flags about the shape.
    #[heap_trace(skip)]
    flags: ShapeFlags,
}

/// Represents a shared object shape.
#[derive(Debug, Clone)]
pub struct SharedShape {
    inner: Rooted<Inner>,
}

impl SharedShape {
    fn property_table(&self) -> &PropertyTable {
        &self.inner.property_table
    }
    /// Return the property count that this shape owns in the [`PropertyTable`].
    fn property_count(&self) -> u32 {
        self.inner.property_count
    }
    /// Return the index to the property in the [`PropertyTable`].
    fn property_index(&self) -> u32 {
        self.inner.property_count.saturating_sub(1)
    }
    /// Getter for the transition count field.
    #[must_use]
    pub fn transition_count(&self) -> u16 {
        self.inner.transition_count
    }
    /// Getter for the previous field.
    pub fn previous(&self) -> Result<Option<Self>, RootingError> {
        self.inner
            .previous
            .as_ref()
            .map(|previous| {
                Ok(Self {
                    inner: previous.upgrade()?.ok_or(RootingError::Expired)?,
                })
            })
            .transpose()
    }
    /// Get the prototype of the shape.
    pub fn prototype(&self) -> Result<JsPrototype, RootingError> {
        self.inner
            .prototype
            .as_ref()
            .map(|edge| edge.upgrade()?.ok_or(RootingError::Expired))
            .transpose()
    }
    /// Get the property this [`SharedShape`] refers to.
    pub(crate) fn property(&self) -> JsResult<(PropertyKey, Slot)> {
        let inner = self
            .property_table()
            .inner()
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        let (key, slot) = inner
            .keys
            .get(self.property_index() as usize)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        Ok((key.clone(), *slot))
    }
    /// Get the flags of the shape.
    fn flags(&self) -> ShapeFlags {
        self.inner.flags
    }
    /// Getter for the [`ForwardTransition`] field.
    fn forward_transitions(&self) -> &ForwardTransition {
        &self.inner.forward_transitions
    }
    /// Check if the shape has the given prototype.
    #[must_use]
    pub fn has_prototype(&self, prototype: &JsObject) -> bool {
        self.inner
            .prototype
            .as_ref()
            .is_some_and(|edge| edge.identity() == prototype.identity())
    }

    /// Create a new [`SharedShape`].
    fn new(heap: &RuntimeHeap, inner: Inner) -> Result<Self, AccessError> {
        let inner = heap.allocate(inner).map_err(|rejected| rejected.reason)?;
        Ok(Self { inner })
    }

    /// Create a root [`SharedShape`].
    pub(crate) fn root(heap: &RuntimeHeap) -> JsResult<Self> {
        Self::new(
            heap,
            Inner {
                forward_transitions: ForwardTransition::default(),
                prototype: None,
                property_count: 0,
                // Most of the time the root shape initiates with between 1-4 properties.
                property_table: PropertyTable::with_capacity(4)?,
                previous: None,
                flags: ShapeFlags::default(),
                transition_count: 0,
            },
        )
        .map_err(|error| RootingError::Access(error).into())
    }

    /// Create a [`SharedShape`] change prototype transition.
    pub(crate) fn change_prototype_transition(&self, prototype: JsPrototype) -> JsResult<Self> {
        let key = prototype.as_ref().map(JsObject::identity);
        if let Some(inner) = self
            .forward_transitions()
            .get_prototype(key)
            .map_err(RootingError::Access)?
        {
            return Ok(Self { inner });
        }
        let new_inner_shape = Inner {
            forward_transitions: ForwardTransition::default(),
            prototype: prototype.map(JsObject::into_record),
            property_table: self.property_table().clone(),
            property_count: self.property_count(),
            previous: Some(self.inner.record_ref()),
            transition_count: self
                .transition_count()
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?,
            flags: ShapeFlags::prototype_transition_from(self.flags()),
        };
        let new_shape =
            Self::new(self.inner.runtime(), new_inner_shape).map_err(RootingError::Access)?;

        self.forward_transitions()
            .insert_prototype(key, &new_shape.inner)?;

        Ok(new_shape)
    }

    /// Create a [`SharedShape`] insert property transition.
    pub(crate) fn insert_property_transition(&self, key: TransitionKey) -> JsResult<Self> {
        // Check if we have already created such a transition, if so use it!
        if let Some(shape) = self
            .forward_transitions()
            .get_property(&key)
            .map_err(RootingError::Access)?
            && let Some(inner) = shape.upgrade().map_err(RootingError::Access)?
        {
            return Ok(Self { inner });
        }
        // An expired hit is a miss; periodic insertion pruning owns full scans.

        let property_table = self.property_table().add_property_deep_clone_if_needed(
            key.property_key.clone(),
            key.attributes,
            self.property_count(),
        )?;
        let new_inner_shape = Inner {
            prototype: self.inner.prototype.clone(),
            forward_transitions: ForwardTransition::default(),
            property_table,
            property_count: self
                .property_count()
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?,
            previous: Some(self.inner.record_ref()),
            transition_count: self
                .transition_count()
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?,
            flags: ShapeFlags::insert_property_transition_from(self.flags()),
        };
        let new_shape =
            Self::new(self.inner.runtime(), new_inner_shape).map_err(RootingError::Access)?;

        self.forward_transitions()
            .insert_property(key, &new_shape.inner)?;

        Ok(new_shape)
    }

    /// Create a [`SharedShape`] change prototype transition, returning [`ChangeTransition`].
    pub(crate) fn change_attributes_transition(
        &self,
        key: TransitionKey,
    ) -> JsResult<ChangeTransition<Self>> {
        let slot = self.property_table().get(&key.property_key)?;

        // Check if we have already created such a transition, if so use it!
        if let Some(shape) = self
            .forward_transitions()
            .get_property(&key)
            .map_err(RootingError::Access)?
            && let Some(inner) = shape.upgrade().map_err(RootingError::Access)?
        {
            let action = if slot.attributes.width_match(key.attributes) {
                ChangeTransitionAction::Nothing
            } else if slot.attributes.is_accessor_descriptor() {
                // Accessor property --> Data property
                ChangeTransitionAction::Remove
            } else {
                // Data property --> Accessor property
                ChangeTransitionAction::Insert
            };

            return Ok(ChangeTransition {
                shape: Self { inner },
                action,
            });
        }
        // An expired hit is a miss; periodic insertion pruning owns full scans.

        // The attribute change transitions, didn't change from accessor to data property or vice-versa.
        if slot.attributes.width_match(key.attributes) {
            let property_table = self.property_table().deep_clone_all()?;
            property_table.set_attributes_at_index(&key.property_key, key.attributes)?;
            let inner_shape = Inner {
                forward_transitions: ForwardTransition::default(),
                prototype: self.inner.prototype.clone(),
                property_table,
                property_count: self.property_count(),
                previous: Some(self.inner.record_ref()),
                transition_count: self
                    .transition_count()
                    .checked_add(1)
                    .ok_or(EngineResourceError::SizeOverflow)?,
                flags: ShapeFlags::configure_property_transition_from(self.flags()),
            };
            let shape =
                Self::new(self.inner.runtime(), inner_shape).map_err(RootingError::Access)?;

            self.forward_transitions()
                .insert_property(key, &shape.inner)?;

            return Ok(ChangeTransition {
                shape,
                action: ChangeTransitionAction::Nothing,
            });
        }

        // Rollback before the property has added.
        let (mut base, prototype, transitions) = self.rollback_before(&key.property_key)?;

        // Apply prototype transition, if it was found.
        if let Some(prototype) = prototype {
            base = base.change_prototype_transition(prototype)?;
        }

        // Apply this property.
        base = base.insert_property_transition(key)?;

        // Apply previous properties.
        for (property_key, attributes) in transitions.into_iter().rev() {
            let transition = TransitionKey {
                property_key,
                attributes,
            };
            base = base.insert_property_transition(transition)?;
        }

        // Determine action to be performed on the storage.
        let action = if slot.attributes.is_accessor_descriptor() {
            // Accessor property --> Data property
            ChangeTransitionAction::Remove
        } else {
            // Data property --> Accessor property
            ChangeTransitionAction::Insert
        };

        Ok(ChangeTransition {
            shape: base,
            action,
        })
    }

    /// Rollback to shape before the insertion of the [`PropertyKey`] that is provided.
    ///
    /// This returns the shape before the insertion, if it sees a prototype transition it will return the latest one,
    /// ignoring any others, [`None`] otherwise. It also will return the property transitions ordered from
    /// latest to oldest that it sees.
    ///
    /// NOTE: In the transitions it does not include the property that we are rolling back.
    ///
    /// NOTE: The prototype transitions if it sees a property insert and then later an attribute change it will condense
    /// into one property insert transition with the new attribute in the change attribute transition,
    /// in the same place that the property was inserted initially.
    //
    // For example with the following chain:
    //
    //        INSERT(x)             INSERT(y)                INSERT(z)
    // { }  ------------>  { x }  ------------>  { x, y }  ------------>  { x, y, z }
    //
    // Then we call rollback on `y`:
    //
    //        INSERT(x)             INSERT(y)                INSERT(z)
    // { }  ------------>  { x }  ------------>  { x, y }  ------------>  { x, y, z }
    //                       ^
    //                       \--- base (with array of transitions to be performed: INSERT(z),
    //                                                 and protortype: None )
    fn rollback_before(
        &self,
        key: &PropertyKey,
    ) -> JsResult<(
        Self,
        Option<JsPrototype>,
        IndexMap<PropertyKey, SlotAttributes>,
    )> {
        let mut prototype = None;
        let mut transitions: IndexMap<PropertyKey, SlotAttributes, RandomState> =
            IndexMap::default();

        let mut current = Some(self.clone());
        let base = loop {
            let Some(current_shape) = current else {
                return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
            };

            // We only take the latest prototype change it, if it exists.
            if current_shape.flags().is_prototype_transition_type() {
                if prototype.is_none() {
                    prototype = Some(current_shape.prototype()?);
                }

                // Skip when it is a prototype transition.
                current = current_shape.previous()?;
                continue;
            }

            let (current_property_key, slot) = current_shape.property()?;

            if current_shape.flags().is_insert_transition_type() && &current_property_key == key {
                let base = if let Some(base) = current_shape.previous()? {
                    base
                } else {
                    // It's the root, because it doesn't have previous.
                    current_shape
                };
                break base;
            }

            // Do not add property that we are trying to delete.
            // this can happen if a configure was called after inserting it into the shape
            if &current_property_key != key {
                // Only take the latest changes to a property. To try to build a smaller tree.
                transitions
                    .try_reserve(1)
                    .map_err(|_| EngineResourceError::Allocation)?;
                transitions
                    .entry(current_property_key)
                    .or_insert(slot.attributes);
            }

            current = current_shape.previous()?;
        };

        Ok((base, prototype, transitions))
    }

    /// Remove a property from [`SharedShape`], returning the new [`SharedShape`].
    pub(crate) fn remove_property_transition(&self, key: &PropertyKey) -> JsResult<Self> {
        let (mut base, prototype, transitions) = self.rollback_before(key)?;

        // Apply prototype transition, if it was found.
        if let Some(prototype) = prototype {
            base = base.change_prototype_transition(prototype)?;
        }

        for (property_key, attributes) in transitions.into_iter().rev() {
            let transition = TransitionKey {
                property_key,
                attributes,
            };
            base = base.insert_property_transition(transition)?;
        }

        Ok(base)
    }

    /// Do a property lookup, returns [`None`] if property not found.
    pub(crate) fn lookup(&self, key: &PropertyKey) -> JsResult<Option<Slot>> {
        let property_count = self.property_count();
        if property_count == 0 {
            return Ok(None);
        }

        let property_table_inner = self
            .property_table()
            .inner()
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        // Check if we are trying to access properties that belong to another shape.
        if let Some((property_table_index, slot)) =
            property_table_inner.map.get(key).map(std::cell::Cell::get)
            && property_table_index < self.property_count()
        {
            return Ok(Some(slot));
        }
        Ok(None)
    }

    /// Gets all keys first strings then symbols in creation order.
    pub(crate) fn keys(&self) -> JsResult<Vec<PropertyKey>> {
        let property_table = self
            .property_table()
            .inner()
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        property_table.keys_cloned_n(self.property_count())
    }

    /// Returns a new [`UniqueShape`] with the properties of the [`SharedShape`].
    pub(crate) fn required_slots(&self) -> JsResult<usize> {
        let table = self
            .property_table()
            .inner()
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        if self.property_count() == 0 {
            return Ok(0);
        }
        let (_, slot) = table
            .keys
            .get(self.property_index() as usize)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        Ok(slot
            .index
            .checked_add(slot.width())
            .ok_or(EngineResourceError::SizeOverflow)? as usize)
    }
    pub(crate) fn to_unique(&self) -> JsResult<UniqueShape> {
        let table = self
            .property_table()
            .inner()
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .clone_count(self.property_count())?;
        UniqueShape::new(self.inner.runtime(), self.prototype()?, table)
            .map_err(|error| RootingError::Access(error).into())
    }

    pub(crate) const fn heap(&self) -> &RuntimeHeap {
        self.inner.runtime()
    }

    pub(crate) fn identity(&self) -> ObjectId {
        self.inner.handle().erase()
    }
}

/// Non-owning identity of a [`SharedShape`].
#[derive(Debug, Clone, PartialEq, HeapTrace)]
pub(crate) struct SharedShapeRef {
    inner: RecordRef<Inner>,
}

impl SharedShapeRef {
    /// Upgrade returns a [`SharedShape`] pointer for the internal value if the pointer is still live,
    /// or [`None`] if the value was already garbage collected.
    #[inline]
    pub(crate) fn upgrade(&self) -> Result<Option<SharedShape>, AccessError> {
        Ok(self.inner.upgrade()?.map(|inner| SharedShape { inner }))
    }
}

impl From<&SharedShape> for SharedShapeRef {
    fn from(value: &SharedShape) -> Self {
        SharedShapeRef {
            inner: value.inner.record_ref(),
        }
    }
}
