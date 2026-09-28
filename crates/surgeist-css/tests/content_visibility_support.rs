#![forbid(unsafe_code)]

//! The pinned Containment 2 §4 authored value production is fully recognized.

use surgeist_css::{CssSupportStatus, property_support_metadata};

#[test]
fn content_visibility_authored_property_support_is_complete() {
    let support = property_support_metadata("content-visibility").unwrap();
    assert_eq!(support.feature().status(), CssSupportStatus::Complete);
}
