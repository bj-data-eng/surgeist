//! Supplementary public witness for the selected eager NaN-to-zero policy.
//! At hue zero, normalized whiteness and blackness both zero leave the pure
//! red vertex: (1, 0, 0) * (1 - 0 - 0) + 0, hence RGB (255, 0, 0).

use crate::css::support::color_property::normal_checked_color as checked_color;

#[test]
fn zero_whiteness_and_blackness_leave_the_red_vertex() {
    assert_eq!(
        checked_color("hwb(0 0% 0%)")
            .serialize_specified(Default::default())
            .unwrap(),
        "rgb(255, 0, 0)"
    );
}

#[test]
fn calculated_nan_whiteness_becomes_zero_before_rgb_conversion() {
    assert_eq!(
        checked_color("hwb(0 calc(NaN * 1%) 0%)")
            .serialize_specified(Default::default())
            .unwrap(),
        "rgb(255, 0, 0)"
    );
}

#[test]
fn calculated_nan_blackness_becomes_zero_before_rgb_conversion() {
    assert_eq!(
        checked_color("hwb(0 0% calc(NaN * 1%))")
            .serialize_specified(Default::default())
            .unwrap(),
        "rgb(255, 0, 0)"
    );
}
