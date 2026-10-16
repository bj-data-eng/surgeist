//! Actual moved String capacity is distinct from logical decoded-name length.
#![forbid(unsafe_code)]

use surgeist_css::{CssCustomIdent, CssViewTransitionIdent, CssViewTransitionName};

#[test]
fn checked_transition_name_retains_original_spare_capacity_and_value_semantics() {
    let mut raw = String::with_capacity(128);
    raw.push_str("Card");
    let capacity = raw.capacity();
    let pointer = raw.as_ptr();
    assert!(capacity >= 128);
    let value = CssViewTransitionIdent::try_new(CssCustomIdent::try_new(raw).unwrap()).unwrap();
    let equal = CssViewTransitionIdent::try_new(CssCustomIdent::try_new("Card").unwrap()).unwrap();
    for _ in 0..3 {
        assert_eq!(value.retained_buffer_bytes(), capacity);
        assert!(value.retained_buffer_bytes() > value.as_str().len());
        assert_eq!(value.as_str(), "Card");
        assert_eq!(value.as_str().as_ptr(), pointer);
        assert_eq!(value, equal);
    }
    assert_eq!(
        CssViewTransitionName::Custom(value)
            .serialize_specified()
            .unwrap(),
        "Card"
    );
}

#[test]
fn transition_name_capacity_counts_utf8_bytes_without_changing_case_or_admission() {
    let mut raw = String::with_capacity(96);
    raw.push_str("é中");
    let capacity = raw.capacity();
    let value = CssViewTransitionIdent::try_new(CssCustomIdent::try_new(raw).unwrap()).unwrap();
    assert_eq!(value.as_str(), "é中");
    assert_eq!(value.as_str().len(), 5);
    assert_eq!(value.retained_buffer_bytes(), capacity);
    assert!(value.retained_buffer_bytes() >= 96);
    for reserved in ["none", "NONE", "auto", "AUTO"] {
        assert!(
            CssViewTransitionIdent::try_new(CssCustomIdent::try_new(reserved).unwrap()).is_none()
        );
    }
}
