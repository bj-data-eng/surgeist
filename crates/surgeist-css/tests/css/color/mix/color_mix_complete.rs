//! Selected Color5 #funcdef-color-mix and Values4 #calc-range.
//! Complete authored list, interpolation, and weight grammar.
use crate::css::support::color_and_selector_fixtures::{
    assert_color_value_rejected_at, parsed_important_declaration as parsed,
};
use crate::css::support::color_property::color_property_value as wrapper;
use crate::css::support::detached_checked_declaration;
use surgeist_color::{ColorSpace, PredefinedColorSpace};
use surgeist_css::CssParser;
use surgeist_css::color::CssColorMix;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssPropertyTarget,
};
use surgeist_css::values::common::CssBoxSideKind;

fn admitted_mix(text: &str) -> [CssColorMix; 2] {
    let mut mixes = Vec::new();
    for declaration in [
        parsed("color", text),
        detached_checked_declaration("color", text, false),
    ] {
        let value = wrapper(&declaration);
        assert_eq!(value.as_css(), text);
        mixes.push(value.value().color_mix_value().expect("typed mix").clone());
    }
    mixes.try_into().unwrap()
}

#[test]
fn omitted_interpolation_method_admits_the_default_mix() {
    for mix in admitted_mix("color-mix(red, blue)") {
        assert!(mix.interpolation().is_none());
        assert_eq!(mix.components().len(), 2);
        assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
        assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
        assert!(mix.components().iter().all(|item| item.weight().is_none()));
    }
}
#[test]
fn a_single_color_is_a_nonempty_mix_list() {
    for mix in admitted_mix("color-mix(in srgb, red)") {
        assert_eq!(
            mix.interpolation().unwrap().predefined().unwrap().space(),
            ColorSpace::Predefined(PredefinedColorSpace::Srgb)
        );
        assert_eq!(mix.components().len(), 1);
        assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
    }
}
#[test]
fn more_than_two_colors_keep_the_authored_list_and_duplicates() {
    for mix in admitted_mix("color-mix(in oklab, red, green, red)") {
        assert_eq!(
            mix.interpolation().unwrap().predefined().unwrap().space(),
            ColorSpace::Oklab
        );
        assert_eq!(
            mix.components()
                .iter()
                .map(|item| item.color().named().unwrap().name())
                .collect::<Vec<_>>(),
            ["red", "green", "red"]
        );
    }
}
#[test]
fn percentage_can_precede_its_color() {
    for mix in admitted_mix("color-mix(in srgb, 30% red, blue 70%)") {
        assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
        assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
        assert_eq!(
            mix.components()[0]
                .weight()
                .unwrap()
                .literal_value()
                .unwrap()
                .literal()
                .numeric()
                .representation(),
            "30"
        );
        assert_eq!(
            mix.components()[1]
                .weight()
                .unwrap()
                .literal_value()
                .unwrap()
                .literal()
                .numeric()
                .representation(),
            "70"
        );
    }
}
#[test]
fn percentage_math_weights_keep_specified_range_deferral() {
    for text in [
        "color-mix(in srgb, red calc(30%), blue)",
        "color-mix(in srgb, calc(30%) red, blue)",
        "color-mix(in srgb, red calc(-5%), blue calc(120%))",
    ] {
        for mix in admitted_mix(text) {
            assert_eq!(mix.components().len(), 2);
            assert!(
                mix.components()[0]
                    .weight()
                    .unwrap()
                    .calculation()
                    .is_some()
            );
            assert_eq!(
                mix.components()[0]
                    .weight()
                    .unwrap()
                    .calculation()
                    .unwrap()
                    .components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                if text.contains("-5%") {
                    "calc(-5%)"
                } else {
                    "calc(30%)"
                }
            );
            if text.contains("120%") {
                assert_eq!(
                    mix.components()[1]
                        .weight()
                        .unwrap()
                        .calculation()
                        .unwrap()
                        .components()
                        .serialize_tokens(Default::default())
                        .unwrap()
                        .as_css(),
                    "calc(120%)"
                );
            } else {
                assert!(mix.components()[1].weight().is_none());
            }
        }
    }
}
#[test]
fn custom_interpolation_names_are_symbolic_authored_references() {
    for (text, name) in [
        ("color-mix(in --Profile, red, blue)", "--Profile"),
        ("color-mix(in --profile, red, blue)", "--profile"),
        ("color-mix(in --Pr\\6f file, red, blue)", "--Profile"),
        ("color-mix(in --, red, blue)", "--"),
    ] {
        for mix in admitted_mix(text) {
            assert_eq!(
                mix.interpolation()
                    .unwrap()
                    .custom_profile()
                    .unwrap()
                    .as_str(),
                name
            );
            assert_eq!(mix.components().len(), 2);
        }
    }
}
#[test]
fn one_component_border_color_uses_complete_shared_mix_grammar() {
    let text = "color-mix(red, green, blue)";
    let supplied = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::BorderColor)),
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
        let colors = value.value();
        assert_eq!(colors.kind(), CssBoxSideKind::Physical);
        assert_eq!(colors.authored().len(), 1);
        let [top, right, bottom, left] = colors.assigned();
        assert_eq!(top.color_mix_value().unwrap().components().len(), 3);
        assert_eq!(top, right);
        assert_eq!(top, bottom);
        assert_eq!(top, left);
    }
}

/// The color grammar rejects `text` through the style attribute, dropping
/// only its declaration, and through checked components. `responsible` is
/// the first token that violates the grammar with its byte offset in `text`;
/// a truncated argument list or a calculation names no single token, so only
/// the color grammar kind is checked.
fn rejected(text: &str, responsible: Option<(&str, usize)>) {
    let report = CssParser::default().parse_block_contents(format!("color:{text};opacity:.5"));
    let [diagnostic] = report.diagnostics() else {
        panic!("{text}: one diagnostic: {report:?}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidColorSyntax,
        "{text}"
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    if let Some((_, at)) = responsible {
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            "color:".len() + at,
            "{text}"
        );
    }
    assert_eq!(report.syntax().len(), 1, "{text}");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
        CssComponentValues::parse(&CssParser::default(), text).unwrap(),
        CssImportance::Normal,
    );
    match responsible {
        Some((token, at)) => {
            assert_color_value_rejected_at(checked, CssKnownProperty::Color, text, token, at)
        }
        None => assert!(
            checked.unwrap_err().code() == Some(CssErrorCode::InvalidColorSyntax),
            "{text}"
        ),
    }
}
#[test]
fn malformed_items_wrong_weight_types_and_literal_ranges_stay_invalid() {
    for (text, responsible) in [
        ("color-mix()", None),
        ("color-mix(, red)", Some((",", 10))),
        ("color-mix(in srgb red, blue)", Some(("red", 18))),
        ("color-mix(red,)", None),
        ("color-mix(red blue)", Some(("blue", 14))),
        ("color-mix(20% red 30%, blue)", Some(("30%", 18))),
        (
            "color-mix(in srgb longer hue, red, blue)",
            Some(("longer", 18)),
        ),
        (
            "color-mix(in --Profile shorter hue, red, blue)",
            Some(("shorter", 23)),
        ),
        ("color-mix(in srgb, red calc(30), blue)", None),
        ("color-mix(in srgb, red calc(1px), blue)", None),
        (
            "color-mix(in srgb, red -1e-100%, blue)",
            Some(("-1e-100%", 23)),
        ),
        (
            "color-mix(in srgb, red 100.0000000000000000000001%, blue)",
            Some(("100.0000000000000000000001%", 23)),
        ),
    ] {
        rejected(text, responsible);
    }
}
#[test]
fn explicit_safe_two_color_and_all_zero_weights_remain_valid() {
    for text in [
        "color-mix(in srgb, red 25%, blue 50%)",
        "color-mix(in oklch, red 0%, blue 0%)",
        "color-mix(in srgb, red -0%, blue 100%)",
    ] {
        for declaration in [
            parsed("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            let mix = wrapper(&declaration).value().color_mix_value().unwrap();
            assert_eq!(mix.components().len(), 2);
            let weights = [
                mix.components()[0]
                    .weight()
                    .unwrap()
                    .literal_value()
                    .unwrap()
                    .literal()
                    .numeric()
                    .representation(),
                mix.components()[1]
                    .weight()
                    .unwrap()
                    .literal_value()
                    .unwrap()
                    .literal()
                    .numeric()
                    .representation(),
            ];
            assert!(matches!(weights, ["25", "50"] | ["0", "0"] | ["-0", "100"]));
        }
    }
}

#[test]
fn qualified_substitutions_keep_the_whole_mix_pending_and_reentry_strict() {
    for text in [
        "color-mix(var(--items))",
        "color-mix(red env(weight), blue)",
    ] {
        for declaration in [
            parsed("color", text),
            detached_checked_declaration("color", text, false),
        ] {
            let known = declaration.known().unwrap();
            assert!(known.property_value().is_none());
            assert_eq!(known.substitution_dependent().unwrap().as_css(), text);
            let CssExpansion::Pending(pending) = declaration.expand().unwrap() else {
                panic!("pending mix")
            };
            assert!(pending.source().same_occurrence(&declaration));
            for replacement in ["color-mix(var(--items))", "color-mix(red env(), blue)"] {
                assert!(
                    pending
                        .reenter(
                            CssComponentValues::parse(&CssParser::default(), replacement).unwrap()
                        )
                        .unwrap_err()
                        .is_residual_substitution()
                );
            }
            assert!(
                pending
                    .reenter(
                        CssComponentValues::parse(
                            &CssParser::default(),
                            "color-mix(in srgb, red calc(30), blue)"
                        )
                        .unwrap()
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
        detached_checked_declaration("color", "color-mix(var(--items))", false),
    ] {
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("pending source")
        };
        let replacement =
            CssComponentValues::parse(&CssParser::default(), "color-mix(red, green, blue)")
                .unwrap();
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
