use crate::{
    JsObject, JsSymbol, Runtime, object::shape::slot::SlotAttributes, property::PropertyKey,
};

use super::{SharedShape, TransitionKey};

#[test]
fn cached_prototype_transition_releases_an_unused_prototype() {
    let runtime = Runtime::default();
    let root = SharedShape::root(&runtime.heap).unwrap();
    let prototype = JsObject::with_null_proto(&runtime).unwrap();
    let weak_prototype = prototype.edge();
    let transitioned = root.change_prototype_transition(Some(prototype)).unwrap();

    runtime.collect().unwrap();
    assert!(
        weak_prototype.is_live().unwrap(),
        "a live shape owns its prototype"
    );

    drop(transitioned);
    runtime.collect().unwrap();
    assert!(
        !weak_prototype.is_live().unwrap(),
        "the forward transition cache retained an unused prototype"
    );
    drop(root);
}

#[test]
fn test_prune_property_on_counter_limit() {
    let runtime = Runtime::default();
    let shape = SharedShape::root(&runtime.heap).unwrap();

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().property_transitions_count(),
            (i, i as u8)
        );

        shape
            .insert_property_transition(TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            })
            .unwrap();
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (255, 255)
    );

    runtime.heap.collect().unwrap();

    {
        shape
            .insert_property_transition(TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            })
            .unwrap();
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (1, 0)
    );

    {
        shape
            .insert_property_transition(TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            })
            .unwrap();
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );

    runtime.heap.collect().unwrap();

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );
}

#[test]
fn test_prune_prototype_on_counter_limit() {
    let runtime = Runtime::default();
    let shape = SharedShape::root(&runtime.heap).unwrap();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (0, 0)
    );

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().prototype_transitions_count(),
            (i, i as u8)
        );

        shape
            .change_prototype_transition(Some(JsObject::with_null_proto(&runtime).unwrap()))
            .unwrap();
    }

    runtime.heap.collect().unwrap();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (255, 255)
    );

    {
        shape
            .change_prototype_transition(Some(JsObject::with_null_proto(&runtime).unwrap()))
            .unwrap();
    }

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (1, 0)
    );

    {
        shape
            .change_prototype_transition(Some(JsObject::with_null_proto(&runtime).unwrap()))
            .unwrap();
    }

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );

    runtime.heap.collect().unwrap();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );
}

#[test]
fn a_live_transition_roots_its_predecessor_during_allocation() {
    use surgeist_js_gc::heap::AllocationCollection;

    let runtime = Runtime::default();
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let root = SharedShape::root(&runtime.heap).unwrap();
    let predecessor = root
        .insert_property_transition(TransitionKey {
            property_key: crate::js_string!("first").into(),
            attributes: SlotAttributes::all(),
        })
        .unwrap();
    let weak = super::SharedShapeRef::from(&predecessor);
    let descendant = predecessor
        .insert_property_transition(TransitionKey {
            property_key: crate::js_string!("second").into(),
            attributes: SlotAttributes::all(),
        })
        .unwrap();
    drop(predecessor);

    assert_eq!(runtime.heap.collect().unwrap(), 0);
    let restored = descendant.previous().unwrap().expect("predecessor edge");
    assert_eq!(
        restored.keys().unwrap(),
        [PropertyKey::from(crate::js_string!("first"))]
    );
    assert!(weak.upgrade().unwrap().is_some());
    drop(restored);
    drop(descendant);

    assert_eq!(runtime.heap.collect().unwrap(), 2);
    assert!(weak.upgrade().unwrap().is_none());
}
