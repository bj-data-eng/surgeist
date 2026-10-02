#![forbid(unsafe_code)]
//! Supplementary public witness for the selected eager NaN-to-zero policy.
//! At hue zero, normalized whiteness and blackness both zero leave the pure
//! red vertex: (1, 0, 0) * (1 - 0 - 0) + 0, hence RGB (255, 0, 0).

use surgeist_css::*;

fn checked_color(source: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(source).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("checked color property");
    };
    value.value().clone()
}

#[test]
fn zero_whiteness_and_blackness_leave_the_red_vertex() {
    assert_eq!(
        checked_color("hwb(0 0% 0%)").to_specified_css().unwrap(),
        "rgb(255, 0, 0)"
    );
}

#[test]
fn calculated_nan_whiteness_becomes_zero_before_rgb_conversion() {
    assert_eq!(
        checked_color("hwb(0 calc(NaN * 1%) 0%)")
            .to_specified_css()
            .unwrap(),
        "rgb(255, 0, 0)"
    );
}

#[test]
fn calculated_nan_blackness_becomes_zero_before_rgb_conversion() {
    assert_eq!(
        checked_color("hwb(0 0% calc(NaN * 1%))")
            .to_specified_css()
            .unwrap(),
        "rgb(255, 0, 0)"
    );
}
