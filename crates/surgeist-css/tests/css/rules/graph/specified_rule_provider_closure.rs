//! Wrapper consumption of provider owners #621 and #553/#555. These tests get
//! behavior RED through existing CssSheet emission, not through missing leaf APIs.
//! Their initial UnsupportedRule failure does not prove direct leaf-provider RED.
use crate::css::support::rules::assert_exact_specified_sheet as exact;

#[test]
fn animation_direction_consumer_emits_typed_keywords_in_list_order() {
    exact(
        ".x{animation-direction:REVERSE,ALTERNATE-REVERSE}",
        ".x { animation-direction: reverse, alternate-reverse; }",
    );
}

#[test]
fn transform_consumer_emits_typed_functions_in_authored_list_order() {
    exact(
        ".x{transform:translateX(+001.5PX) rotate(+090DEG)}",
        ".x { transform: translateX(1.5px) rotate(90deg); }",
    );
}
