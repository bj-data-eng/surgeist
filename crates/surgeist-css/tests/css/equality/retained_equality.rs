//! Values that retain the origin of their components compare equal across origins, and
//! `same_provenance` compares the retained origins.

use surgeist_css::CssParser;
use surgeist_css::color::CssHexColor;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssValueTokenRef,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};
use surgeist_css::rule::counter_style::{
    CssCounterStyleDescriptorKind, CssCounterStyleDescriptorValue,
};
use surgeist_css::rule::font_face::CssFontNamedInstanceString;
use surgeist_css::source::same_provenance;
use surgeist_css::values::font::CssFontLanguageString;
use surgeist_css::values::image::{CssImage1D, CssStripe};

fn components(source: &str) -> CssComponentValues {
    CssComponentValues::parse(&CssParser::default(), source).unwrap()
}

fn token(source: &str, string: bool) -> CssComponentValue {
    components(source)
        .items()
        .iter()
        .find(|item| match item.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Ident(_)) => !string,
            CssComponentValueRef::Token(CssValueTokenRef::String(_)) => string,
            _ => false,
        })
        .unwrap()
        .clone()
}

fn assert_origin_only_difference<T: PartialEq + Clone + std::fmt::Debug>(left: &T, right: &T) {
    assert_eq!(left, right);
    assert!(!same_provenance(left, right), "{left:?}");
    assert!(same_provenance(left, &left.clone()));
}

#[test]
fn property_value_failures_equal_across_origins_and_differ_in_provenance() {
    let fail = |text: &str| {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Margin)),
            components(text),
            CssImportance::Normal,
        )
        .unwrap_err()
    };
    assert_origin_only_difference(&fail("bogus"), &fail("    bogus"));
    assert_ne!(fail("bogus"), fail("other"));
}

#[test]
fn descriptor_value_failures_equal_across_origins_and_differ_in_provenance() {
    let fail = |text: &str| {
        CssCounterStyleDescriptorValue::try_from_components(
            CssCounterStyleDescriptorKind::System,
            components(text),
            Default::default(),
        )
        .unwrap_err()
    };
    assert_origin_only_difference(&fail("bogus"), &fail("    bogus"));
}

#[test]
fn string_values_equal_across_the_origin_of_their_component() {
    let language =
        |text: &str| CssFontLanguageString::try_from_component(token(text, true)).unwrap();
    assert_origin_only_difference(&language("\"x\""), &language("   \"x\""));
    assert_ne!(language("\"x\""), language("\"y\""));
    let instance =
        |text: &str| CssFontNamedInstanceString::try_from_component(token(text, true)).unwrap();
    assert_origin_only_difference(&instance("\"x\""), &instance("   \"x\""));
    assert_ne!(instance("\"x\""), instance("\"y\""));
}

#[test]
fn stripes_equal_across_the_origin_of_their_color_and_function() {
    let stripe = |text: &str| CssStripe::try_new(token(text, false), None).unwrap();
    assert_origin_only_difference(&stripe("red"), &stripe("   red"));
    assert_ne!(stripe("red"), stripe("blue"));
    let image = |text: &str| CssImage1D::try_from_components(components(text)).unwrap();
    assert_origin_only_difference(&image("stripes(red,blue)"), &image("stripes( red , blue )"));
    assert_ne!(image("stripes(red,blue)"), image("stripes(red,red)"));
}

#[test]
fn hex_digits_keep_their_origin_free_comparison() {
    let hex = |digits: &str| CssHexColor::try_new(digits).unwrap();
    assert_origin_only_difference(&hex("ABC"), &hex("abc"));
}
