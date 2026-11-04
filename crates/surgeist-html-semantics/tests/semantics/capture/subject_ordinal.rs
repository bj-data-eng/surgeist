//! A subject's ordinal is its dense position within its own capture.
use crate::semantics::support::builder;
use surgeist_html::values::{Namespace, OrderedAttributes, QualifiedName};
use surgeist_html_semantics::{HtmlCaptureRegistryInput, HtmlCapturedIsValue};

#[test]
fn ordinals_follow_the_order_subjects_were_added_without_gaps() {
    let (mut builder, document) = builder();
    let mut handles = vec![document.clone()];
    for _ in 0..4 {
        handles.push(
            builder
                .add_element(
                    &document,
                    QualifiedName::new(Namespace::html(), None, "div"),
                    OrderedAttributes::default(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap(),
        );
    }
    let capture = builder.finish().unwrap();
    for (position, handle) in handles.iter().enumerate() {
        let id = capture.subject_id(handle).unwrap();
        assert_eq!(id.ordinal(), position);
        assert_eq!(id.capture(), capture.identity());
    }
}

#[test]
fn equal_ordinals_of_different_captures_are_different_subjects() {
    let make = || {
        let (builder, document) = builder();
        let capture = builder.finish().unwrap();
        let id = capture.subject_id(&document).unwrap();
        (capture, id)
    };
    let (first_capture, first) = make();
    let (second_capture, second) = make();
    assert_eq!(first.ordinal(), second.ordinal());
    assert_ne!(first, second);
    assert_ne!(first.capture(), second.capture());
    assert_eq!(first.capture(), first_capture.identity());
    assert_eq!(second.capture(), second_capture.identity());
}
