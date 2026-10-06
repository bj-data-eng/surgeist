#![forbid(unsafe_code)]
//! Existing checked aggregate fronts must not certify browser-recovered children.
use surgeist_css::*;

fn color(text: &str, clean: bool) -> CssColor {
    let source = format!("text-decoration-color:{text}");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.is_clean(),
        clean,
        "{source}: {:?}",
        report.diagnostics()
    );
    assert_eq!(
        report.syntax().len(),
        1,
        "{source}: {:?}",
        report.diagnostics()
    );
    let CssKnownPropertyValueRef::TextDecorationColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained decoration Color")
    };
    value.value().clone()
}

fn thickness(text: &str, clean: bool) -> CssTextDecorationThickness {
    let source = format!("text-decoration-thickness:{text}");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.is_clean(),
        clean,
        "{source}: {:?}",
        report.diagnostics()
    );
    assert_eq!(
        report.syntax().len(),
        1,
        "{source}: {:?}",
        report.diagnostics()
    );
    let CssKnownPropertyValueRef::TextDecorationThickness(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained decoration thickness")
    };
    value.value().clone()
}

#[test]
fn checked_decoration_rejects_recovered_color_roots_and_nested_closures() {
    for text in [
        "rgb(1 2 3",
        "light-dark(red,blue",
        "color-mix(in srgb,red,blue",
        "light-dark(red,rgb(calc(1 + 2),0,0",
    ] {
        let value = color(text, false);
        let before = value.clone();
        assert!(
            CssTextDecoration::try_new(None, Some(value.clone()), None, None).is_none(),
            "checked construction must reject recovered original Color: {text}"
        );
        assert_eq!(value, before);
        assert!(CssTextDecoration::try_new(None, Some(value), None, None).is_none());
    }
}

#[test]
fn checked_decoration_rejects_recovered_thickness_without_consuming_the_child() {
    for text in ["calc(-1px", "calc(1px + calc(2em"] {
        let value = thickness(text, false);
        let before = value.clone();
        assert!(
            CssTextDecoration::try_new(None, None, None, Some(value.clone())).is_none(),
            "checked construction must reject recovered original thickness: {text}"
        );
        assert_eq!(value, before);
        assert!(CssTextDecoration::try_new(None, None, None, Some(value)).is_none());
    }
}

#[test]
fn complete_and_programmatic_decoration_children_keep_existing_checked_output() {
    assert!(CssTextDecoration::try_new(None, None, None, None).is_none());
    for text in [
        "red",
        "rgb(1 2 3)",
        "light-dark(red,blue)",
        "color-mix(in srgb,red,blue)",
    ] {
        let value = color(text, true);
        let aggregate = CssTextDecoration::try_new(None, Some(value.clone()), None, None).unwrap();
        assert_eq!(aggregate.color(), Some(&value));
        assert!(aggregate.serialize_specified().is_ok());
    }
    for text in ["-1px", "calc(-1px)", "calc(1px + calc(2em))"] {
        let value = thickness(text, true);
        let aggregate = CssTextDecoration::try_new(None, None, None, Some(value.clone())).unwrap();
        assert_eq!(aggregate.thickness(), Some(&value));
        assert!(aggregate.serialize_specified().is_ok());
    }
    let value = CssColor::transparent();
    let aggregate = CssTextDecoration::try_new(None, Some(value.clone()), None, None).unwrap();
    assert_eq!(aggregate.color(), Some(&value));
    assert_eq!(aggregate.serialize_specified().unwrap(), "transparent");
}
