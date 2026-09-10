use surgeist_css::{
    CssColor, CssGlobalKeyword, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, parse_style_attribute,
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
