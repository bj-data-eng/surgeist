#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::sync::Arc;

use surgeist_animation::{
    CompositeValue, DiscreteValue, InterpolationError, PropertyKey, TransformValue,
};

#[test]
fn all_text_routes_preserve_nonblank_contents_and_content_equality() {
    for text in ["opacity", " \tvisible\u{2003} ", "\u{200b}", "é陰影"] {
        let property = PropertyKey::new(text).unwrap();
        assert_eq!(property.as_str(), text);
        assert_eq!(PropertyKey::new(text.to_owned()).unwrap(), property);
        assert_eq!(PropertyKey::from_text(text).unwrap(), property);
        assert_eq!(PropertyKey::from_shared(Arc::from(text)).unwrap(), property);

        let discrete = DiscreteValue::new(text).unwrap();
        assert_eq!(discrete.token(), text);
        assert_eq!(DiscreteValue::new(text.to_owned()).unwrap(), discrete);
        assert_eq!(DiscreteValue::from_text(text).unwrap(), discrete);
        assert_eq!(
            DiscreteValue::from_shared(Arc::from(text)).unwrap(),
            discrete
        );

        let transform = TransformValue::unsupported(text);
        assert_eq!(transform.kind(), text);
        assert_eq!(TransformValue::unsupported(text.to_owned()), transform);
        assert_eq!(TransformValue::unsupported_from_text(text), transform);
        assert_eq!(
            TransformValue::unsupported_from_shared(Arc::from(text)),
            transform
        );

        let composite = CompositeValue::unsupported(text);
        assert_eq!(composite.kind(), text);
        assert_eq!(CompositeValue::unsupported(text.to_owned()), composite);
        assert_eq!(CompositeValue::unsupported_from_text(text), composite);
        assert_eq!(
            CompositeValue::unsupported_from_shared(Arc::from(text)),
            composite
        );
    }
}

#[test]
fn empty_and_unicode_whitespace_routes_preserve_typed_rejection_and_marker_fallback() {
    for text in ["", " \t\r\n", "\u{00a0}\u{2003}\u{3000}"] {
        assert_eq!(
            PropertyKey::new(text),
            Err(InterpolationError::EmptyPropertyKey)
        );
        assert_eq!(
            PropertyKey::from_text(text),
            Err(InterpolationError::EmptyPropertyKey)
        );
        assert_eq!(
            PropertyKey::from_shared(Arc::from(text)),
            Err(InterpolationError::EmptyPropertyKey)
        );
        assert_eq!(
            DiscreteValue::new(text),
            Err(InterpolationError::EmptyDiscreteValue)
        );
        assert_eq!(
            DiscreteValue::from_text(text),
            Err(InterpolationError::EmptyDiscreteValue)
        );
        assert_eq!(
            DiscreteValue::from_shared(Arc::from(text)),
            Err(InterpolationError::EmptyDiscreteValue)
        );

        for transform in [
            TransformValue::unsupported(text),
            TransformValue::unsupported_from_text(text),
            TransformValue::unsupported_from_shared(Arc::from(text)),
        ] {
            assert_eq!(transform.kind(), "unsupported");
        }
        for composite in [
            CompositeValue::unsupported(text),
            CompositeValue::unsupported_from_text(text),
            CompositeValue::unsupported_from_shared(Arc::from(text)),
        ] {
            assert_eq!(composite.kind(), "unsupported");
        }
    }
}

#[test]
fn equal_property_names_from_each_route_find_the_same_hash_set_entry() {
    let stored = PropertyKey::new(" \u{2003}opacity ").unwrap();
    let mut keys = HashSet::new();
    keys.insert(stored);

    assert!(keys.contains(&PropertyKey::from_text(" \u{2003}opacity ").unwrap()));
    assert!(keys.contains(&PropertyKey::from_shared(Arc::from(" \u{2003}opacity ")).unwrap()));
    assert!(!keys.contains(&PropertyKey::from_text("opacity").unwrap()));
}

#[test]
fn borrowed_text_values_remain_owned_after_the_source_is_cleared() {
    let mut source = String::from(" \u{2003}owned text ");
    let property = PropertyKey::from_text(&source).unwrap();
    let discrete = DiscreteValue::from_text(&source).unwrap();
    let transform = TransformValue::unsupported_from_text(&source);
    let composite = CompositeValue::unsupported_from_text(&source);
    source.clear();
    drop(source);

    assert_eq!(property.as_str(), " \u{2003}owned text ");
    assert_eq!(discrete.token(), " \u{2003}owned text ");
    assert_eq!(transform.kind(), " \u{2003}owned text ");
    assert_eq!(composite.kind(), " \u{2003}owned text ");
}

#[test]
fn shared_text_ownership_and_clones_keep_the_consumed_allocation_alive() {
    assert_shared_ownership(
        |text| PropertyKey::from_shared(text).unwrap(),
        PropertyKey::as_str,
    );
    assert_shared_ownership(
        |text| DiscreteValue::from_shared(text).unwrap(),
        DiscreteValue::token,
    );
    assert_shared_ownership(
        TransformValue::unsupported_from_shared,
        TransformValue::kind,
    );
    assert_shared_ownership(
        CompositeValue::unsupported_from_shared,
        CompositeValue::kind,
    );
}

#[test]
fn rejected_or_replaced_blank_shared_text_releases_the_consumed_allocation() {
    let blank: Arc<str> = Arc::from("\u{2003}\u{3000}");
    let weak = Arc::downgrade(&blank);
    assert_eq!(
        PropertyKey::from_shared(blank),
        Err(InterpolationError::EmptyPropertyKey)
    );
    assert!(weak.upgrade().is_none());

    let blank: Arc<str> = Arc::from("\u{2003}\u{3000}");
    let weak = Arc::downgrade(&blank);
    assert_eq!(
        DiscreteValue::from_shared(blank),
        Err(InterpolationError::EmptyDiscreteValue)
    );
    assert!(weak.upgrade().is_none());

    let blank: Arc<str> = Arc::from("\u{2003}\u{3000}");
    let weak = Arc::downgrade(&blank);
    let transform = TransformValue::unsupported_from_shared(blank);
    assert_eq!(transform.kind(), "unsupported");
    assert!(weak.upgrade().is_none());

    let blank: Arc<str> = Arc::from("\u{2003}\u{3000}");
    let weak = Arc::downgrade(&blank);
    let composite = CompositeValue::unsupported_from_shared(blank);
    assert_eq!(composite.kind(), "unsupported");
    assert!(weak.upgrade().is_none());
}

#[test]
fn shared_text_values_support_cross_thread_borrows_and_owned_transfer() {
    let shared: Arc<str> = Arc::from(" \u{2003}shared text ");
    let values = (
        PropertyKey::from_shared(Arc::clone(&shared)).unwrap(),
        DiscreteValue::from_shared(Arc::clone(&shared)).unwrap(),
        TransformValue::unsupported_from_shared(Arc::clone(&shared)),
        CompositeValue::unsupported_from_shared(shared),
    );

    std::thread::scope(|scope| {
        scope.spawn(|| assert_text_values(&values)).join().unwrap();
    });
    let retained = std::thread::spawn(move || {
        assert_text_values(&values);
        values
    })
    .join()
    .unwrap();
    assert_text_values(&retained);
}

fn assert_shared_ownership<T: Clone>(
    construct: impl FnOnce(Arc<str>) -> T,
    text: impl Fn(&T) -> &str,
) {
    let shared: Arc<str> = Arc::from(" \u{2003}retained text ");
    let weak = Arc::downgrade(&shared);
    let value = construct(shared);
    assert_eq!(text(&value), " \u{2003}retained text ");
    assert!(weak.upgrade().is_some());

    let retained = value.clone();
    drop(value);
    assert!(weak.upgrade().is_some());
    assert_eq!(text(&retained), " \u{2003}retained text ");
    drop(retained);
    assert!(weak.upgrade().is_none());
}

fn assert_text_values(values: &(PropertyKey, DiscreteValue, TransformValue, CompositeValue)) {
    assert_eq!(values.0.as_str(), " \u{2003}shared text ");
    assert_eq!(values.1.token(), " \u{2003}shared text ");
    assert_eq!(values.2.kind(), " \u{2003}shared text ");
    assert_eq!(values.3.kind(), " \u{2003}shared text ");
}
