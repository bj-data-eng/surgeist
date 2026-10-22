//! Typed `color` property payloads shared by color declaration fixtures.

use super::{declaration_color, normal_checked_declaration};
use surgeist_css::*;

/// The `color` property value carried by a known declaration.
pub fn color_property_value(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color property: {declaration:?}")
    };
    value
}

/// The typed color of `source` checked from component values by the
/// `color` property at normal importance.
pub fn normal_checked_color(source: &str) -> CssColor {
    declaration_color(&normal_checked_declaration(CssKnownProperty::Color, source)).clone()
}
