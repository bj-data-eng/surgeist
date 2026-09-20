#![forbid(unsafe_code)]
//! Selected Color5 #funcdef-color-mix and Values4 #calc-range.
//! Existing public boundaries only; typed list/weight APIs are tested separately.
use surgeist_css::*;

fn parsed(property: &str, text: &str) -> CssDeclaration {
    let source = format!("{property}:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    let declaration = declaration.clone();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("original function")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        property.len() + 1
    );
    declaration
}
fn checked(text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}
fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary color")
    };
    value
}
fn admits_new_mix(text: &str) {
    for declaration in [parsed("color", text), checked(text)] {
        let value = wrapper(&declaration);
        assert!(value.current().color_mix_value().is_some());
        assert_eq!(value.as_css(), text);
        // New spellings are deliberately outside the unchanged frozen parser.
        assert!(value.i01_subset().is_none(), "{text}");
    }
}

#[test]
fn omitted_interpolation_method_admits_the_default_mix() {
    admits_new_mix("color-mix(red, blue)");
}
#[test]
fn a_single_color_is_a_nonempty_mix_list() {
    admits_new_mix("color-mix(in srgb, red)");
}
#[test]
fn more_than_two_colors_keep_the_authored_list_and_duplicates() {
    admits_new_mix("color-mix(in oklab, red, green, red)");
}
#[test]
fn percentage_can_precede_its_color() {
    admits_new_mix("color-mix(in srgb, 30% red, blue 70%)");
}
#[test]
fn percentage_math_weights_keep_specified_range_deferral() {
    for text in [
        "color-mix(in srgb, red calc(30%), blue)",
        "color-mix(in srgb, calc(30%) red, blue)",
        "color-mix(in srgb, red calc(-5%), blue calc(120%))",
    ] {
        admits_new_mix(text);
    }
}
#[test]
fn custom_interpolation_names_are_symbolic_authored_references() {
    // These checks establish grammar admission without a profile registry.
    // Decoded case/escape identity is inspected by later typed API evidence.
    for text in [
        "color-mix(in --Profile, red, blue)",
        "color-mix(in --profile, red, blue)",
        "color-mix(in --Pr\\6f file, red, blue)",
        "color-mix(in --, red, blue)",
    ] {
        admits_new_mix(text);
    }
}
#[test]
fn one_component_border_color_uses_complete_shared_mix_grammar() {
    let text = "color-mix(red, green, blue)";
    let supplied = parse_component_values(text).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::BorderColor),
        supplied.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &supplied);
    for declaration in [parsed("border-color", text), checked] {
        let CssKnownPropertyValueRef::BorderColor(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("border color")
        };
        let colors = value.current();
        assert!(colors.top().color_mix_value().is_some());
        assert_eq!(colors.top(), colors.right());
        assert_eq!(colors.top(), colors.bottom());
        assert_eq!(colors.top(), colors.left());
        assert!(value.i01_subset().is_none());
    }
}

fn rejected(text: &str) {
    let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
    assert_eq!(report.diagnostics().len(), 1, "{text}");
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            parse_component_values(text).unwrap(),
            CssImportance::Normal
        )
        .is_err(),
        "{text}"
    );
}
#[test]
fn malformed_items_wrong_weight_types_and_literal_ranges_stay_invalid() {
    for text in [
        "color-mix()",
        "color-mix(, red)",
        "color-mix(in srgb red, blue)",
        "color-mix(red,)",
        "color-mix(red blue)",
        "color-mix(20% red 30%, blue)",
        "color-mix(in srgb longer hue, red, blue)",
        "color-mix(in --Profile shorter hue, red, blue)",
        "color-mix(in srgb, red calc(30), blue)",
        "color-mix(in srgb, red calc(1px), blue)",
        "color-mix(in srgb, red -1e-100%, blue)",
        "color-mix(in srgb, red 100.0000000000000000000001%, blue)",
    ] {
        rejected(text);
    }
}
#[test]
fn explicit_safe_two_color_and_all_zero_weights_remain_valid() {
    for text in [
        "color-mix(in srgb, red 25%, blue 50%)",
        "color-mix(in oklch, red 0%, blue 0%)",
        "color-mix(in srgb, red -0%, blue 100%)",
    ] {
        for declaration in [parsed("color", text), checked(text)] {
            assert!(wrapper(&declaration).current().color_mix_value().is_some());
            assert!(wrapper(&declaration).i01_subset().is_some(), "{text}");
        }
    }
}

#[test]
fn qualified_substitutions_keep_the_whole_mix_pending_and_reentry_strict() {
    for text in [
        "color-mix(var(--items))",
        "color-mix(red env(weight), blue)",
    ] {
        for declaration in [parsed("color", text), checked(text)] {
            let known = declaration.known().unwrap();
            assert!(known.property_value().is_none());
            assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
            let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
                panic!("pending mix")
            };
            assert!(pending.source().same_occurrence(&declaration));
            for replacement in ["color-mix(var(--items))", "color-mix(red env(), blue)"] {
                assert_eq!(
                    pending
                        .reenter(parse_component_values(replacement).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            }
            assert!(
                pending
                    .reenter(
                        parse_component_values("color-mix(in srgb, red calc(30), blue)").unwrap()
                    )
                    .is_err()
            );
        }
    }
}
#[test]
fn pending_mix_reenters_complete_ordinary_grammar_with_original_occurrence() {
    for source in [
        parsed("color", "color-mix(var(--items))"),
        checked("color-mix(var(--items))"),
    ] {
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending source")
        };
        let replacement = parse_component_values("color-mix(red, green, blue)").unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("color longhand")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::Color);
        assert!(value.ordinary_value().is_some());
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
    }
}
