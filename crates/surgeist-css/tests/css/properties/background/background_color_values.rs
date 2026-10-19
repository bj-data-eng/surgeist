#![forbid(unsafe_code)]
//! Functional typed lifecycle assertions for Backgrounds 3 §2.2's selected color grammar.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#background-color

use surgeist_css::*;

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn typed_color(item: &CssLonghandContribution) -> &CssColor {
    let CssLonghandValueRef::BackgroundColor(color) = item.ordinary_value().unwrap().view() else {
        panic!("typed background-color contribution")
    };
    color
}

#[test]
fn intrinsic_initial_is_typed_transparent_without_an_authored_occurrence() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BackgroundColor.metadata().unwrap().kind()
    else {
        panic!("longhand metadata")
    };
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::BackgroundColor
    );
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("typed initial")
    };
    let CssLonghandValueRef::BackgroundColor(color) = value.view() else {
        panic!("background color initial")
    };
    assert_eq!(color, &CssColor::transparent());
    assert!(color.is_transparent());
    assert_eq!(color.to_specified_css().unwrap(), "transparent");
}

#[test]
fn ordinary_typed_payloads_retain_literal_and_context_dependent_color_meaning() {
    for (text, expected, canonical) in [
        (
            "red",
            CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
            "red",
        ),
        (
            "#abc",
            CssColor::from_hex(CssHexColor::try_new("abc").unwrap()),
            "rgb(170, 187, 204)",
        ),
        ("currentcolor", CssColor::current_color(), "currentcolor"),
        ("transparent", CssColor::transparent(), "transparent"),
    ] {
        let source = declaration(&format!("background-color:{text}!important"));
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("ordinary contribution")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(typed_color(item), &expected);
        assert_eq!(typed_color(item).to_specified_css().unwrap(), canonical);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn reusable_reentry_retains_typed_programmatic_and_parsed_colors_and_symbolic_globals() {
    let source = declaration("background-color:var(--paint)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending color")
    };
    let parsed = parse_component_values("red").unwrap();
    let programmatic =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("currentcolor").unwrap()])
            .unwrap();
    for (replacement, expected) in [
        (
            parsed,
            CssColor::from_named(CssNamedColor::try_new("red").unwrap()),
        ),
        (programmatic, CssColor::current_color()),
    ] {
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("replacement contribution")
            };
            let [item] = items.items() else {
                panic!("one terminal")
            };
            assert_eq!(typed_color(item), &expected);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(
                item.replacement_components().unwrap().items()[0].origin(),
                replacement.items()[0].origin()
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("initial").unwrap())
        .unwrap()
    else {
        panic!("global contribution")
    };
    let [item] = items.items() else {
        panic!("one global")
    };
    assert_eq!(
        item.value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Initial)
    );
    assert!(item.ordinary_value().is_none());
    assert!(pending.source().same_occurrence(&source));
}
