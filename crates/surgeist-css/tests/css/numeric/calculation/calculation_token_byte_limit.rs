//! `serialize_tokens` honours the output byte limit of its component limits:
//! the exact serialized length succeeds with the unlimited text, and one byte
//! less refuses with a byte-limit failure at the calculation's origin.

use crate::css::support::byte_limits;
use surgeist_css::{CssCalculation, CssLength, parse_component_values};

#[test]
fn calculation_tokens_refuse_one_byte_below_their_serialized_length() {
    let values = parse_component_values("calc(1px + 2em)").unwrap();
    let calculation =
        CssCalculation::<CssLength>::try_from_components(values, Default::default()).unwrap();
    let unlimited = calculation.serialize_tokens(Default::default()).unwrap();
    let length = unlimited.as_css().len();
    assert!(length > 0);
    assert_eq!(
        calculation
            .serialize_tokens(byte_limits(length))
            .unwrap()
            .as_css(),
        unlimited.as_css()
    );
    let error = calculation
        .serialize_tokens(byte_limits(length - 1))
        .unwrap_err();
    assert!(error.is_byte_limit(), "{error:?}");
    assert_eq!(error.origin(), calculation.origin());
    assert!(
        calculation
            .serialize_tokens(byte_limits(0))
            .unwrap_err()
            .is_byte_limit()
    );
}
