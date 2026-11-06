use crate::{JsObject, JsSymbol, object::shape::slot::SlotAttributes, property::PropertyKey};

use super::{SharedShape, TransitionKey};

#[test]
fn cached_prototype_transition_releases_an_unused_prototype() {
    use surgeist_js_gc::{WeakGc, force_collect};

    let root = SharedShape::root();
    let prototype = JsObject::with_null_proto();
    let weak_prototype = WeakGc::new(prototype.inner());
    let transitioned = root.change_prototype_transition(Some(prototype));

    force_collect();
    assert!(
        weak_prototype.is_upgradable(),
        "a live shape owns its prototype"
    );

    drop(transitioned);
    force_collect();
    assert!(
        !weak_prototype.is_upgradable(),
        "the forward transition cache retained an unused prototype"
    );
    drop(root);
}

#[test]
fn test_prune_property_on_counter_limit() {
    let shape = SharedShape::root();

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().property_transitions_count(),
            (i, i as u8)
        );

        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (255, 255)
    );

    surgeist_js_gc::force_collect();

    {
        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (1, 0)
    );

    {
        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );

    surgeist_js_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );
}

#[test]
fn test_prune_prototype_on_counter_limit() {
    let shape = SharedShape::root();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (0, 0)
    );

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().prototype_transitions_count(),
            (i, i as u8)
        );

        shape.change_prototype_transition(Some(JsObject::with_null_proto()));
    }

    surgeist_js_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (255, 255)
    );

    {
        shape.change_prototype_transition(Some(JsObject::with_null_proto()));
    }

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (1, 0)
    );

    {
        shape.change_prototype_transition(Some(JsObject::with_null_proto()));
    }

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );

    surgeist_js_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );
}
