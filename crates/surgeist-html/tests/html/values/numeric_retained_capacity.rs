//! Actual owned primitive capacity, independently of logical decimal length.
#![forbid(unsafe_code)]

use crate::html::support::non_negative_integer as integer;
use surgeist_html::{DocumentString, HtmlNonNegativeInteger};

#[test]
fn public_integer_construction_keeps_canonical_values_and_observes_owned_buffers() {
    for (raw, canonical) in [
        ("0000", "0"),
        ("000123", "123"),
        (
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211456",
        ),
    ] {
        let value = integer(raw);
        assert_eq!(value.decimal_digits(), canonical);
        assert!(value.retained_buffer_bytes() >= canonical.len());
        assert_eq!(value.as_document_string(), DocumentString::from(canonical));
    }
    for native in [0, 1, 123, u128::MAX] {
        let value = HtmlNonNegativeInteger::from(native);
        assert_eq!(value.to_u128(), Ok(native));
        assert!(value.retained_buffer_bytes() >= value.decimal_digits().len());
    }
}

#[test]
fn actual_public_arithmetic_matches_its_single_allocation_receipt() {
    let left = integer("000999");
    let right = integer("1");
    let added = left.try_added_to(&right, 4, 100).unwrap();
    assert_eq!(added.value().decimal_digits(), "1000");
    assert_eq!(
        added.value().retained_buffer_bytes(),
        added.capacity_bytes()
    );
    let (added, capacity) = added.into_parts();
    assert_eq!(added.retained_buffer_bytes(), capacity);
    assert_eq!(added.to_u128(), Ok(1000));

    let multiplied = added.try_multiplied_by(3, 4, 100).unwrap();
    assert_eq!(multiplied.value().decimal_digits(), "3000");
    assert_eq!(
        multiplied.value().retained_buffer_bytes(),
        multiplied.capacity_bytes()
    );
    assert_eq!(left.decimal_digits(), "999");
    assert_eq!(right.decimal_digits(), "1");
}
