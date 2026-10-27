//! Backgrounds 3 (CRD 2024-03-11) §2.3: background-image is <bg-image>#, and
//! Images 3 (CRD 2023-12-18) §3.1 defines linear-gradient(), whose omitted
//! direction defaults to `to bottom`.

use surgeist_css::CssParser;
use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef};
use surgeist_css::values::image::{CssGradient, CssImageValue};

#[test]
fn valid_authored_linear_gradient_is_retained_as_background_image() {
    let report =
        CssParser::default().parse_block_contents("background-image: linear-gradient(red, blue)");

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration");
    };
    let known = declaration.known().expect("known background-image");
    assert_eq!(known.property(), CssKnownProperty::BackgroundImage);
    let Some(CssKnownPropertyValueRef::BackgroundImage(value)) = known.property_value() else {
        panic!("typed background-image value");
    };
    let [CssImageValue::Gradient(gradient @ CssGradient::Linear(_))] = value.value().images()
    else {
        panic!("one linear gradient: {:?}", value.value().images());
    };
    // CSSOM 1 (WD 2021-08-26) §6.7.2 omits a component that does not change
    // the meaning, so the default direction stays omitted; both stops keep
    // their named-color keywords in order.
    assert_eq!(
        gradient.serialize_specified(Default::default()).unwrap(),
        "linear-gradient(red, blue)"
    );
}
