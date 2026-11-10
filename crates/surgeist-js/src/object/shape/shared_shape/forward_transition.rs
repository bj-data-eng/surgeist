use std::cell::RefCell;

use rustc_hash::FxHashMap;
use surgeist_js_gc::heap::{AccessError, ObjectId, RecordRef, Rooted};

use super::{Inner as SharedShapeInner, TransitionKey};
use crate::{
    JsResult,
    error::{EngineResourceError, RootingError},
};
use std::hash::Hash;

/// Maps transition key type to a [`SharedShapeInner`] transition.
#[derive(Debug)]
struct TransitionMap<T> {
    map: FxHashMap<T, RecordRef<SharedShapeInner>>,

    /// This counts the number of insertions after a prune operation.
    insertion_count_since_prune: u8,
}

impl<T> Default for TransitionMap<T> {
    fn default() -> Self {
        Self {
            map: FxHashMap::default(),
            insertion_count_since_prune: 0,
        }
    }
}

impl<T> TransitionMap<T> {
    fn get_and_increment_count(&mut self) -> u8 {
        let result = self.insertion_count_since_prune;

        // NOTE(HalidOdat): This is done so it overflows to 0, on every 256 insertion
        // operations. Fulfills the prune condition every 256 insertions.
        self.insertion_count_since_prune = self.insertion_count_since_prune.wrapping_add(1);

        result
    }
}

/// The internal representation of [`ForwardTransition`].
#[derive(Default, Debug)]
struct Inner {
    properties: Option<TransitionMap<TransitionKey>>,
    // The weak transition owns no prototype. Its live target owns that edge.
    prototypes: Option<TransitionMap<Option<ObjectId>>>,
}

/// Holds a forward reference to a previously created transition.
///
/// The reference is weak, therefore it can be garbage collected, if it's not in use.
#[derive(Default, Debug)]
pub(super) struct ForwardTransition {
    inner: RefCell<Inner>,
}

impl ForwardTransition {
    pub(super) fn insert_property(
        &self,
        key: TransitionKey,
        value: &Rooted<SharedShapeInner>,
    ) -> JsResult<()> {
        let mut this = self
            .inner
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        this.properties
            .get_or_insert_with(TransitionMap::default)
            .insert(key, value)
    }
    pub(super) fn insert_prototype(
        &self,
        key: Option<ObjectId>,
        value: &Rooted<SharedShapeInner>,
    ) -> JsResult<()> {
        let mut this = self
            .inner
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        this.prototypes
            .get_or_insert_with(TransitionMap::default)
            .insert(key, value)
    }
    pub(super) fn get_property(
        &self,
        key: &TransitionKey,
    ) -> Result<Option<RecordRef<SharedShapeInner>>, AccessError> {
        let this = self
            .inner
            .try_borrow()
            .map_err(|_| AccessError::PayloadBorrowed)?;
        Ok(this
            .properties
            .as_ref()
            .and_then(|transitions| transitions.map.get(key))
            .cloned())
    }
    pub(super) fn get_prototype(
        &self,
        key: Option<ObjectId>,
    ) -> Result<Option<Rooted<SharedShapeInner>>, AccessError> {
        let this = self
            .inner
            .try_borrow()
            .map_err(|_| AccessError::PayloadBorrowed)?;
        this.prototypes
            .as_ref()
            .and_then(|transitions| transitions.map.get(&key))
            .map_or(Ok(None), RecordRef::upgrade)
    }

    #[cfg(test)]
    pub(crate) fn property_transitions_count(&self) -> (usize, u8) {
        let this = self.inner.borrow();
        this.properties.as_ref().map_or((0, 0), |transitions| {
            (
                transitions.map.len(),
                transitions.insertion_count_since_prune,
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn prototype_transitions_count(&self) -> (usize, u8) {
        let this = self.inner.borrow();
        this.prototypes.as_ref().map_or((0, 0), |transitions| {
            (
                transitions.map.len(),
                transitions.insertion_count_since_prune,
            )
        })
    }
}

impl<T: Eq + Hash> TransitionMap<T> {
    fn insert(&mut self, key: T, value: &Rooted<SharedShapeInner>) -> JsResult<()> {
        self.map
            .try_reserve(1)
            .map_err(|_| EngineResourceError::Allocation)?;
        if self.get_and_increment_count() == u8::MAX {
            // The periodic prune is the only whole-cache traversal. Once access
            // refuses, preserve every unexamined entry and the first cause.
            let mut failure = None;
            self.map.retain(|_, edge| {
                if failure.is_some() {
                    return true;
                }
                match edge.is_live() {
                    Ok(live) => live,
                    Err(error) => {
                        failure = Some(error);
                        true
                    }
                }
            });
            if let Some(error) = failure {
                return Err(RootingError::Access(error).into());
            }
        }
        self.map.insert(key, value.record_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Runtime,
        error::EngineError,
        js_string,
        object::shape::{SharedShape, slot::SlotAttributes},
        property::PropertyKey,
    };
    #[test]
    fn transition_cache_borrow_refuses_without_erasing_existing_entry() {
        let runtime = Runtime::default();
        let root = SharedShape::root(&runtime.heap).unwrap();
        let key = TransitionKey {
            property_key: PropertyKey::from(js_string!("first")),
            attributes: SlotAttributes::CONFIGURABLE,
        };
        let first = root.insert_property_transition(key.clone()).unwrap();
        let held = root.forward_transitions().inner.borrow_mut();
        let error = root.insert_property_transition(key.clone()).unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Rooting(RootingError::Access(
                AccessError::PayloadBorrowed
            )))
        );
        drop(held);
        let again = root.insert_property_transition(key).unwrap();
        assert_eq!(again.identity(), first.identity());
        assert_eq!(
            root.forward_transitions().property_transitions_count(),
            (1, 1)
        );
    }

    #[test]
    fn periodic_prune_refusal_preserves_old_entries_and_rejects_new_entry_until_retry() {
        use surgeist_js_gc::heap::AllocationCollection;

        let runtime = Runtime::default();
        runtime
            .heap
            .set_allocation_collection(AllocationCollection::Manual)
            .unwrap();
        let root = SharedShape::root(&runtime.heap).unwrap();
        let mut transitions = TransitionMap::<TransitionKey>::default();
        let mut accepted = Vec::new();

        // Generate real, matching transition keys and records through the shape
        // owner. This private cache receives the same valid entries; no record,
        // counter or liveness result is fabricated for the failure path.
        for ordinal in 0..255 {
            let key = TransitionKey {
                property_key: crate::JsString::from(format!("accepted_{ordinal}")).into(),
                attributes: SlotAttributes::CONFIGURABLE,
            };
            let shape = root.insert_property_transition(key.clone()).unwrap();
            transitions.insert(key.clone(), &shape.inner).unwrap();
            accepted.push((key, shape));
        }
        assert_eq!(transitions.insertion_count_since_prune, 255);
        let new_key = TransitionKey {
            property_key: js_string!("new_entry").into(),
            attributes: SlotAttributes::CONFIGURABLE,
        };
        let new_shape = root.insert_property_transition(new_key.clone()).unwrap();

        // Prepare every record before the loan. Cache insertion itself needs no
        // heap allocation, so this reaches the periodic liveness check rather than
        // an earlier shape-allocation refusal.
        let error = runtime
            .heap
            .read(|_| transitions.insert(new_key.clone(), &new_shape.inner))
            .unwrap()
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Rooting(RootingError::Access(
                AccessError::HeapBorrowed
            )))
        );
        assert_eq!(transitions.insertion_count_since_prune, 0);
        assert_eq!(transitions.map.len(), 255);
        assert!(!transitions.map.contains_key(&new_key));
        for (key, shape) in &accepted {
            assert_eq!(
                transitions.map.get(key).unwrap().identity(),
                shape.identity()
            );
        }

        transitions
            .insert(new_key.clone(), &new_shape.inner)
            .unwrap();
        assert_eq!(transitions.insertion_count_since_prune, 1);
        assert_eq!(transitions.map.len(), 256);
        assert_eq!(
            transitions.map.get(&new_key).unwrap().identity(),
            new_shape.identity()
        );
        for (key, shape) in accepted {
            assert_eq!(
                transitions.map.get(&key).unwrap().identity(),
                shape.identity()
            );
        }
    }
}
