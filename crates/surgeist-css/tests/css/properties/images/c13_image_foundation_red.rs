use surgeist_css::{CssKnownProperty, parse_style_attribute};

#[test]
fn valid_authored_linear_gradient_is_retained_as_background_image() {
    let report = parse_style_attribute("background-image: linear-gradient(red, blue)");

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().map(|known| known.property()),
        Some(CssKnownProperty::BackgroundImage),
    );
}
