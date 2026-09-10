use surgeist_css::{
    CssAuthoredColor, CssAuthoredColorComponent, CssBorderColors, CssColor, CssGlobalKeyword,
    CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, parse_style_attribute,
};

// CSS Backgrounds and Borders 3, 2024-03-11, section 3.1 defines <color>{1,4}.
// The selected Color 4 publication supplies the component color grammar.
#[test]
fn one_through_four_border_colors_are_retained_with_their_valid_sibling() {
    for authored in [
        "red",
        "red blue",
        "red blue green",
        "red blue green currentcolor",
        "lab(50% 20 30) /**/ rgb(1 2 3 / none)",
    ] {
        let source = format!("border-color: {authored}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2, "{source}");
        let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
            .known()
            .expect("known border-color declaration")
            .property_value()
            .expect("ordinary border-color value")
        else {
            panic!("expected border-color wrapper for {source}");
        };
        assert_eq!(value.as_css(), authored);
        assert_eq!(
            report.syntax()[1].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{source}",
        );
    }
}

#[test]
fn single_border_color_preserves_its_typed_compatibility_value() {
    let report = parse_style_attribute("border-color: currentcolor");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .expect("known border-color declaration")
        .property_value()
        .expect("ordinary border-color value")
    else {
        panic!("expected border-color wrapper");
    };
    assert_eq!(value.i01_subset(), Some(&CssColor::CurrentColor));
}

#[test]
fn invalid_border_color_counts_and_components_recover_at_the_next_declaration() {
    for authored in [
        "",
        "red blue green black white",
        "red blue green black 1px",
        "red 1px",
        "red, blue",
        "red / blue",
        "red initial",
    ] {
        let source = format!("border-color: {authored}; opacity: 0.5");
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "{source}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().map(|known| known.property()),
            Some(CssKnownProperty::Opacity),
            "{source}",
        );
    }
}

#[test]
fn border_color_globals_and_pending_substitution_keep_their_declared_value_branches() {
    for (authored, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = format!("border-color: {authored}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{source}");
        let known = report.syntax()[0].known().expect("known border-color");
        assert_eq!(known.property(), CssKnownProperty::BorderColor);
        assert_eq!(known.global(), Some(expected), "{source}");
        assert!(known.property_value().is_none(), "{source}");
    }

    let source = "border-color: red var(--remaining-colors, blue green)";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    let known = report.syntax()[0].known().expect("known border-color");
    assert_eq!(known.property(), CssKnownProperty::BorderColor);
    assert!(matches!(
        known.declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
    assert!(known.property_value().is_none());
}

fn border_color_value(authored: &str) -> surgeist_css::CssBorderColorPropertyValue {
    let source = format!("border-color: {authored}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .expect("known border-color declaration")
        .property_value()
        .expect("ordinary border-color value")
    else {
        panic!("expected border-color wrapper for {source}");
    };
    value.clone()
}

fn named_sides(colors: &CssBorderColors) -> [&str; 4] {
    [colors.top(), colors.right(), colors.bottom(), colors.left()]
        .map(|color| color.named().expect("named side color").name())
}

#[test]
fn border_color_components_expand_in_top_right_bottom_left_order() {
    for (authored, expected) in [
        ("red", ["red", "red", "red", "red"]),
        ("red blue", ["red", "blue", "red", "blue"]),
        ("red blue green", ["red", "blue", "green", "blue"]),
        ("red blue green black", ["red", "blue", "green", "black"]),
    ] {
        let value = border_color_value(authored);
        assert_eq!(named_sides(value.current()), expected, "{authored}");
    }
}

#[test]
fn border_color_sides_preserve_current_color_and_exact_authored_color_components() {
    let authored = concat!(
        "red rgb(1 2 3 / none) currentcolor ",
        "lab(calc(50% + 10%) 20 -30 / 120%)",
    );
    let value = border_color_value(authored);
    assert_eq!(value.as_css(), authored);
    let colors = value.current();
    assert_eq!(colors.top().named().unwrap().name(), "red");
    let rgb = colors.right().rgb_value().expect("authored rgb side");
    let [
        CssAuthoredColorComponent::Number(red),
        CssAuthoredColorComponent::Number(green),
        CssAuthoredColorComponent::Number(blue),
    ] = rgb.channels()
    else {
        panic!("expected three authored numeric RGB channels");
    };
    assert_eq!([red.value(), green.value(), blue.value()], [1.0, 2.0, 3.0]);
    assert!(matches!(rgb.alpha(), Some(CssAuthoredColorComponent::None)));
    assert!(colors.bottom().is_current_color());
    let lab = colors.left().lab_value().expect("authored lab side");
    assert!(matches!(
        lab.lightness(),
        CssAuthoredColorComponent::PercentageCalculation(_)
    ));
    assert!(matches!(lab.a(), CssAuthoredColorComponent::Number(value) if value.value() == 20.0));
    assert!(matches!(lab.b(), CssAuthoredColorComponent::Number(value) if value.value() == -30.0));
    assert!(matches!(
        lab.alpha(),
        Some(CssAuthoredColorComponent::Percentage(value)) if (value.value() - 120.0).abs() < 0.001
    ));
    assert!(value.i01_subset().is_none());
}

#[test]
fn checked_border_colors_enforce_component_count_and_the_same_side_expansion() {
    let values: Vec<CssAuthoredColor> = ["red", "blue", "green", "black"]
        .map(|name| border_color_value(name).current().top().clone())
        .into();

    assert!(CssBorderColors::try_new(Vec::new()).is_none());
    assert!(CssBorderColors::try_new(vec![values[0].clone(); 5]).is_none());
    for (count, expected) in [
        (1, ["red", "red", "red", "red"]),
        (2, ["red", "blue", "red", "blue"]),
        (3, ["red", "blue", "green", "blue"]),
        (4, ["red", "blue", "green", "black"]),
    ] {
        let colors = CssBorderColors::try_new(values[..count].to_vec())
            .expect("one through four already checked colors");
        assert_eq!(named_sides(&colors), expected, "{count} components");
    }
}

#[test]
fn border_color_compatibility_requires_one_exactly_representable_authored_component() {
    assert_eq!(
        border_color_value("black").i01_subset(),
        Some(&CssColor::BLACK)
    );
    for authored in ["black black", "lab(calc(50% + 10%) 20 -30 / 120%)"] {
        assert!(
            border_color_value(authored).i01_subset().is_none(),
            "{authored}"
        );
    }
    let value = border_color_value("currentcolor");
    let colors = value.current();
    for side in [colors.top(), colors.right(), colors.bottom(), colors.left()] {
        assert!(side.is_current_color());
    }
}
