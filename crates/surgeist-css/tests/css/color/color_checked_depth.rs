#![forbid(unsafe_code)]
//! Checked property construction must handle an already valid component graph at
//! the same public 256-level ceiling as authored parsing, on the default stack.
use surgeist_css::*;

#[test]
fn checked_mixed_colors_preserve_the_shared_component_depth_boundary() {
    for depth in [255, 256] {
        let mut text = "rgb(1 0 0)".to_owned();
        for level in 1..depth {
            text = if level % 2 == 0 {
                format!("rgb(from {text} r g b)")
            } else {
                format!("color-mix(in srgb, {text}, blue)")
            };
        }
        let report = parse_style_attribute(&format!("color:{text}"));
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 1);
        drop(report);
        let values = parse_component_values(&text).expect("within the shared depth ceiling");
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            values,
            CssImportance::Normal,
        )
        .expect("checked grammar has the same depth envelope");
        assert_eq!(
            declaration.known().unwrap().property(),
            CssKnownProperty::Color
        );
        // The owned graph drops normally, without stack-size overrides or leaking.
        drop(declaration);
    }
}
