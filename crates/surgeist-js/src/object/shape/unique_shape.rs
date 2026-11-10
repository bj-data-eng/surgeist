use super::{
    ChangeTransition, ChangeTransitionAction, JsPrototype, Shape, Slot,
    property_table::PropertyTableInner, shared_shape::TransitionKey,
};
use crate::{
    JsObject, JsResult,
    error::{EngineResourceError, ObjectConstructionError, RootingError},
    object::ObjectEdge,
    property::PropertyKey,
};
use std::{
    cell::{Cell, RefCell, RefMut},
    fmt::Debug,
};
use surgeist_js_gc::heap::{
    AccessError, ObjectId, RecordRef, Rooted, RuntimeHeap, Trace as HeapTrace,
};

#[derive(Debug, HeapTrace)]
struct Inner {
    #[heap_trace(skip)]
    property_table: RefCell<PropertyTableInner>,
    prototype: RefCell<Option<ObjectEdge>>,
}
#[derive(Debug, Clone)]
pub(crate) struct UniqueShape {
    inner: Rooted<Inner>,
}
/// Borrows only; the caller separately retains the rooted shape.
pub(crate) struct UniqueShapeGuards<'a> {
    pub(crate) table: RefMut<'a, PropertyTableInner>,
    pub(crate) prototype: RefMut<'a, Option<ObjectEdge>>,
}
struct TailEdit<'a> {
    entry: &'a mut (PropertyKey, Slot),
    cell: &'a Cell<(u32, Slot)>,
    index: u32,
    slot: Slot,
}

/// Validate only the tail the ordinary operation already has to relocate. The
/// resulting edits retain actual cells; commit neither re-looks-up nor indexes
/// those cells. No copy of the untouched prefix/table is needed.
fn prepare_tail<'a>(
    mut edits: Vec<TailEdit<'a>>,
    map: &'a rustc_hash::FxHashMap<PropertyKey, Cell<(u32, Slot)>>,
    tail: &'a mut [(PropertyKey, Slot)],
    first_index: usize,
    new_first_index: usize,
    mut old_offset: u32,
    mut new_offset: u32,
) -> JsResult<(Vec<TailEdit<'a>>, u32, u32)> {
    for (relative, entry) in tail.iter_mut().enumerate() {
        let index = first_index
            .checked_add(relative)
            .ok_or(EngineResourceError::SizeOverflow)?;
        let index = u32::try_from(index).map_err(|_| EngineResourceError::SizeOverflow)?;
        let cell = map
            .get(&entry.0)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        if cell.get() != (index, entry.1) || entry.1.index != old_offset {
            return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
        }
        old_offset = old_offset
            .checked_add(entry.1.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        let slot = Slot {
            index: new_offset,
            attributes: entry.1.attributes,
        };
        new_offset = new_offset
            .checked_add(slot.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        let index = u32::try_from(
            new_first_index
                .checked_add(relative)
                .ok_or(EngineResourceError::SizeOverflow)?,
        )
        .map_err(|_| EngineResourceError::SizeOverflow)?;
        edits.push(TailEdit {
            cell,
            entry,
            index,
            slot,
        });
    }
    Ok((edits, old_offset, new_offset))
}
fn validate_prefix(
    keys: &[(PropertyKey, Slot)],
    index: usize,
    key: &PropertyKey,
    slot: Slot,
) -> JsResult<()> {
    if keys.get(index) != Some(&(key.clone(), slot)) {
        return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
    }
    let expected = match index.checked_sub(1) {
        Some(previous) => {
            let (_, previous) = keys
                .get(previous)
                .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
            previous
                .index
                .checked_add(previous.width())
                .ok_or(EngineResourceError::SizeOverflow)?
        }
        None => 0,
    };
    if slot.index != expected {
        return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
    }
    Ok(())
}
impl UniqueShape {
    pub(crate) fn new(
        heap: &RuntimeHeap,
        prototype: JsPrototype,
        table: PropertyTableInner,
    ) -> Result<Self, AccessError> {
        Self::from_stored(heap, prototype.map(JsObject::into_record), table)
    }
    fn from_stored(
        heap: &RuntimeHeap,
        prototype: Option<ObjectEdge>,
        table: PropertyTableInner,
    ) -> Result<Self, AccessError> {
        let inner = heap
            .allocate(Inner {
                property_table: RefCell::new(table),
                prototype: RefCell::new(prototype),
            })
            .map_err(|rejected| rejected.reason)?;
        Ok(Self { inner })
    }
    pub(crate) fn borrow_for_builtin(&self) -> Result<UniqueShapeGuards<'_>, AccessError> {
        let table = self
            .inner
            .property_table
            .try_borrow_mut()
            .map_err(|_| AccessError::PayloadBorrowed)?;
        let prototype = self
            .inner
            .prototype
            .try_borrow_mut()
            .map_err(|_| AccessError::PayloadBorrowed)?;
        Ok(UniqueShapeGuards { table, prototype })
    }
    pub(crate) fn prototype(&self) -> JsResult<JsPrototype> {
        let edge = self
            .inner
            .prototype
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .clone();
        edge.map(|edge| {
            edge.upgrade()
                .map_err(RootingError::Access)?
                .ok_or(RootingError::Expired)
        })
        .transpose()
        .map_err(Into::into)
    }
    pub(crate) fn insert_property_transition(&self, key: TransitionKey) -> JsResult<Self> {
        self.inner
            .property_table
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .insert(key.property_key, key.attributes)?;
        Ok(self.clone())
    }
    pub(crate) fn remove_property_transition(&self, key: &PropertyKey) -> JsResult<Self> {
        if self.lookup(key)?.is_none() {
            return Ok(self.clone());
        }
        // Allocate before touching the source. GC and any resulting disposal
        // finish before we reacquire the actual source/destination guards.
        let destination = Self::new(self.inner.runtime(), None, PropertyTableInner::default())
            .map_err(RootingError::Access)?;
        let mut source = self.borrow_for_builtin().map_err(RootingError::Access)?;
        let mut target = destination
            .borrow_for_builtin()
            .map_err(RootingError::Access)?;
        let Some((index, slot)) = source.table.map.get(key).map(Cell::get) else {
            return Ok(self.clone());
        };
        let index = index as usize;
        validate_prefix(&source.table.keys, index, key, slot)?;
        let old_end = slot
            .index
            .checked_add(slot.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        let required = source.table.required_slots;
        let first_index = index
            .checked_add(1)
            .ok_or(EngineResourceError::SizeOverflow)?;
        let tail_len = source
            .table
            .keys
            .len()
            .checked_sub(first_index)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let mut edits = Vec::new();
        edits
            .try_reserve_exact(tail_len)
            .map_err(|_| EngineResourceError::Allocation)?;
        let PropertyTableInner { map, keys, .. } = &mut *source.table;
        let suffix = keys
            .get_mut(index..)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (removed, tail) = suffix
            .split_first_mut()
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (edits, old_end, new_end) =
            prepare_tail(edits, map, tail, first_index, index, old_end, slot.index)?;
        if old_end != required {
            return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
        }
        // All fallible work ends here. Adjacent swaps move the removed cell to
        // the end without a checked-index-then-panicking Vec::remove call.
        let mut previous = removed;
        for edit in edits {
            edit.entry.1 = edit.slot;
            edit.cell.set((edit.index, edit.slot));
            std::mem::swap(previous, &mut *edit.entry);
            previous = edit.entry;
        }
        let old_map_entry = map.remove(key);
        let removed_key = source.table.keys.pop();
        source.table.required_slots = new_end;
        *target.table = std::mem::take(&mut *source.table);
        *target.prototype = source.prototype.take();
        drop(target);
        drop(source);
        drop((old_map_entry, removed_key));
        Ok(destination)
    }
    pub(crate) fn lookup(&self, key: &PropertyKey) -> JsResult<Option<Slot>> {
        let table = self
            .inner
            .property_table
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        Ok(table.map.get(key).map(|cell| cell.get().1))
    }
    pub(crate) fn change_attributes_transition(
        &self,
        key: &TransitionKey,
    ) -> JsResult<ChangeTransition<Shape>> {
        let old = self
            .lookup(&key.property_key)?
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        if old.attributes.width_match(key.attributes) {
            let mut table = self
                .inner
                .property_table
                .try_borrow_mut()
                .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
            let (index, slot) = table
                .map
                .get(&key.property_key)
                .map(Cell::get)
                .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
            validate_prefix(&table.keys, index as usize, &key.property_key, slot)?;
            let PropertyTableInner { map, keys, .. } = &mut *table;
            let cell = map
                .get(&key.property_key)
                .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
            let entry = keys
                .get_mut(index as usize)
                .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
            entry.1.attributes = key.attributes;
            cell.set((
                index,
                Slot {
                    index: slot.index,
                    attributes: key.attributes,
                },
            ));
            return Ok(ChangeTransition {
                shape: self.clone().into(),
                action: ChangeTransitionAction::Nothing,
            });
        }
        let destination = Self::new(self.inner.runtime(), None, PropertyTableInner::default())
            .map_err(RootingError::Access)?;
        let mut source = self.borrow_for_builtin().map_err(RootingError::Access)?;
        let mut target = destination
            .borrow_for_builtin()
            .map_err(RootingError::Access)?;
        let (index, old) = source
            .table
            .map
            .get(&key.property_key)
            .map(Cell::get)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        validate_prefix(&source.table.keys, index as usize, &key.property_key, old)?;
        let slot = Slot {
            index: old.index,
            attributes: key.attributes,
        };
        let old_end = old
            .index
            .checked_add(old.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        let new_end = slot
            .index
            .checked_add(slot.width())
            .ok_or(EngineResourceError::SizeOverflow)?;
        let required = source.table.required_slots;
        let first_index = (index as usize)
            .checked_add(1)
            .ok_or(EngineResourceError::SizeOverflow)?;
        let tail_len = source
            .table
            .keys
            .len()
            .checked_sub(first_index)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let mut edits = Vec::new();
        edits
            .try_reserve_exact(tail_len)
            .map_err(|_| EngineResourceError::Allocation)?;
        let PropertyTableInner { map, keys, .. } = &mut *source.table;
        let suffix = keys
            .get_mut(index as usize..)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (changed, tail) = suffix
            .split_first_mut()
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let changed_cell = map
            .get(&key.property_key)
            .ok_or(ObjectConstructionError::IncompatibleSlotLayout)?;
        let (edits, old_end, new_end) =
            prepare_tail(edits, map, tail, first_index, first_index, old_end, new_end)?;
        if old_end != required {
            return Err(ObjectConstructionError::IncompatibleSlotLayout.into());
        }
        let action = if old.attributes.width_match(key.attributes) {
            ChangeTransitionAction::Nothing
        } else if key.attributes.is_accessor_descriptor() {
            ChangeTransitionAction::Insert
        } else {
            ChangeTransitionAction::Remove
        };
        changed.1 = slot;
        changed_cell.set((index, slot));
        for edit in edits {
            edit.entry.1 = edit.slot;
            edit.cell.set((edit.index, edit.slot));
        }
        source.table.required_slots = new_end;
        *target.table = std::mem::take(&mut *source.table);
        *target.prototype = source.prototype.take();
        drop(target);
        drop(source);
        Ok(ChangeTransition {
            shape: destination.into(),
            action,
        })
    }
    pub(crate) fn change_prototype_transition(&self, prototype: JsPrototype) -> JsResult<Self> {
        let destination = Self::new(
            self.inner.runtime(),
            prototype,
            PropertyTableInner::default(),
        )
        .map_err(RootingError::Access)?;
        let mut source = self.borrow_for_builtin().map_err(RootingError::Access)?;
        let mut target = destination
            .borrow_for_builtin()
            .map_err(RootingError::Access)?;
        *target.table = std::mem::take(&mut *source.table);
        drop(target);
        drop(source);
        Ok(destination)
    }
    pub(crate) fn keys(&self) -> JsResult<Vec<PropertyKey>> {
        self.inner
            .property_table
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .keys()
    }
    pub(crate) fn identity(&self) -> ObjectId {
        self.inner.handle().erase()
    }
}
#[derive(Debug, Clone, PartialEq, HeapTrace)]
pub(crate) struct UniqueShapeRef {
    inner: RecordRef<Inner>,
}
impl UniqueShapeRef {
    pub(crate) fn upgrade(&self) -> Result<Option<UniqueShape>, AccessError> {
        Ok(self.inner.upgrade()?.map(|inner| UniqueShape { inner }))
    }
}
impl From<&UniqueShape> for UniqueShapeRef {
    fn from(value: &UniqueShape) -> Self {
        Self {
            inner: value.inner.record_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::slot::SlotAttributes;
    use super::*;
    use crate::{Runtime, builtins::object::OrdinaryObject, error::EngineError, js_string};
    use surgeist_js_gc::heap::AllocationCollection;

    fn shape(runtime: &Runtime, prototype: JsObject) -> UniqueShape {
        let mut table = PropertyTableInner::default();
        for name in ["first", "middle", "last"] {
            table
                .insert(
                    crate::JsString::from(name).into(),
                    SlotAttributes::CONFIGURABLE,
                )
                .unwrap();
        }
        UniqueShape::new(&runtime.heap, Some(prototype), table).unwrap()
    }
    #[test]
    fn destination_preparation_then_source_refusal_leaves_original_for_retry() {
        for operation in 0..3 {
            let runtime = Runtime::default();
            runtime
                .heap
                .set_allocation_collection(AllocationCollection::Manual)
                .unwrap();
            let prototype = JsObject::from_proto_and_data(&runtime, None, OrdinaryObject).unwrap();
            let source = shape(&runtime, prototype.clone());
            let key = PropertyKey::from(js_string!("middle"));
            let held = source.inner.prototype.borrow_mut();
            let result = match operation {
                0 => source.remove_property_transition(&key).map(Shape::from),
                1 => source
                    .change_attributes_transition(&TransitionKey {
                        property_key: key.clone(),
                        attributes: SlotAttributes::GET,
                    })
                    .map(|change| change.shape),
                _ => source.change_prototype_transition(None).map(Shape::from),
            };
            assert_eq!(
                result.unwrap_err().as_engine(),
                Some(&EngineError::Rooting(RootingError::Access(
                    AccessError::PayloadBorrowed
                )))
            );
            drop(held);
            assert_eq!(
                source.keys().unwrap(),
                vec![
                    PropertyKey::from(js_string!("first")),
                    PropertyKey::from(js_string!("middle")),
                    PropertyKey::from(js_string!("last"))
                ]
            );
            assert_eq!(source.prototype().unwrap(), Some(prototype));
            assert_eq!(source.lookup(&key).unwrap().unwrap().index, 1);
            match operation {
                0 => {
                    let changed = source.remove_property_transition(&key).unwrap();
                    assert!(changed.lookup(&key).unwrap().is_none());
                    assert_eq!(
                        changed
                            .lookup(&js_string!("last").into())
                            .unwrap()
                            .unwrap()
                            .index,
                        1
                    );
                }
                1 => {
                    let changed = source
                        .change_attributes_transition(&TransitionKey {
                            property_key: key.clone(),
                            attributes: SlotAttributes::GET,
                        })
                        .unwrap()
                        .shape;
                    assert_eq!(changed.lookup(&key).unwrap().unwrap().width(), 2);
                    assert_eq!(
                        changed
                            .lookup(&js_string!("last").into())
                            .unwrap()
                            .unwrap()
                            .index,
                        3
                    );
                }
                _ => {
                    let changed = source.change_prototype_transition(None).unwrap();
                    assert!(changed.prototype().unwrap().is_none());
                    assert_eq!(
                        changed
                            .lookup(&js_string!("last").into())
                            .unwrap()
                            .unwrap()
                            .index,
                        2
                    );
                }
            }
        }
    }
    #[test]
    fn last_property_remove_and_width_change_keep_earlier_slot_positions() {
        let runtime = Runtime::default();
        let prototype = JsObject::with_null_proto(&runtime).unwrap();
        let source = shape(&runtime, prototype);
        let key = PropertyKey::from(js_string!("last"));
        let changed = source
            .change_attributes_transition(&TransitionKey {
                property_key: key.clone(),
                attributes: SlotAttributes::GET,
            })
            .unwrap()
            .shape;
        assert_eq!(
            changed
                .lookup(&js_string!("middle").into())
                .unwrap()
                .unwrap()
                .index,
            1
        );
        assert_eq!(changed.lookup(&key).unwrap().unwrap().index, 2);
        let removed = changed.remove_property_transition(&key).unwrap();
        assert!(removed.lookup(&key).unwrap().is_none());
        assert_eq!(
            removed
                .lookup(&js_string!("middle").into())
                .unwrap()
                .unwrap()
                .index,
            1
        );
    }
}
