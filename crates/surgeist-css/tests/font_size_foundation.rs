#![forbid(unsafe_code)]

//! Existing-public-API behavior from CSS Fonts 4 WD (2026-09-07) §§2.5, 2.7.
//! Exact typed payload and serialization contracts follow with the functional model.

use surgeist_css::*;

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name("font-size").expect("font-size grammar")
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}");
    };
    declaration.clone()
}

fn checked(value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(),
        parse_component_values(value).expect("component stream"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked font-size:{value}: {error:?}"))
}

fn one_size_contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("font-size intrinsic expansion")
    else {
        panic!("one font-size contribution");
    };
    let [item] = values.items() else {
        panic!("one font-size contribution");
    };
    assert_eq!(item.property(), CssKnownProperty::FontSize);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    item.clone()
}

#[test]
fn existing_keywords_sizes_and_shorthand_forms_remain_valid_controls() {
    for value in [
        "xx-small",
        "x-small",
        "small",
        "medium",
        "large",
        "x-large",
        "xx-large",
        "larger",
        "smaller",
        "0",
        "0px",
        "125%",
        "16px",
        "1em",
        "calc(-1px)",
    ] {
        let parsed = declaration("font-size", value);
        assert_eq!(
            parsed.known().unwrap().property(),
            CssKnownProperty::FontSize
        );
        assert_eq!(
            checked(value).known().unwrap().property(),
            CssKnownProperty::FontSize
        );
    }
    for value in ["italic 16px serif", "16px math", "16px/normal serif"] {
        assert_eq!(
            declaration("font", value).known().unwrap().property(),
            CssKnownProperty::Font
        );
    }
}

#[test]
fn property_accepts_all_size_keywords_and_exact_nonnegative_values() {
    assert_eq!(grammar().target_property(), CssKnownProperty::FontSize);
    for value in [
        "xx-small",
        "x-small",
        "small",
        "medium",
        "large",
        "x-large",
        "xx-large",
        "xxx-large",
        "larger",
        "smaller",
        "math",
        "0",
        "-0",
        "0px",
        "-0px",
        "-0%",
        "125%",
        "1e999px",
        "1e-999px",
        "1e-999%",
        "16px",
        "1em",
        "1rem",
        "1ch",
        "1lh",
        "1vw",
        "1cqw",
        "1cm",
        "calc(-1px)",
    ] {
        for source in [declaration("font-size", value), checked(value)] {
            assert_eq!(
                source.known().unwrap().property(),
                CssKnownProperty::FontSize,
                "{value}"
            );
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
}

#[test]
fn exact_negative_literals_wrong_domains_and_extra_tokens_drop_locally() {
    for invalid in [
        "-1e-50px",
        "-1e-50%",
        "-0.01px",
        "-0.01%",
        "1",
        "1deg",
        "calc(10deg)",
        "calc(1)",
        "medium large",
        "math 16px",
    ] {
        let source = format!("color:red;font-size:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid size diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(validate_style_attribute(&source).is_err());
        assert!(
            parse_property_value_for_grammar(
                grammar(),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked construction accepted {invalid}"
        );
    }
}

#[test]
fn inherited_medium_expands_once_with_globals_and_strict_pending_reentry() {
    let property = CssKnownProperty::FontSize;
    let CssPropertyKindRef::Longhand(longhand) = grammar().metadata().unwrap().kind() else {
        panic!("font-size is a longhand");
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial_value) = initial.view() else {
        panic!("medium is the fixed initial");
    };
    let medium = declaration("font-size", "medium");
    assert_eq!(
        one_size_contribution(&medium).ordinary_value(),
        Some(initial_value)
    );
    for value in ["xxx-large", "math", "125%"] {
        let source = declaration("font-size", value);
        let contribution = one_size_contribution(&source);
        assert!(contribution.ordinary_value().is_some());
        assert!(contribution.replacement_components().is_none());
    }

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration("font-size", spelling);
        assert_eq!(
            one_size_contribution(&source).value(),
            CssContributionValueRef::Global(keyword)
        );
    }

    let source = declaration("font-size", "var(--size)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("unresolved size remains pending");
    };
    assert!(pending.source().same_occurrence(&source));
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(matches!(
        pending
            .reenter(parse_component_values("-1e-50px").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    for valid in ["xxx-large", "math", "125%"] {
        let replacement = parse_component_values(valid).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("strict size replacement contributes a longhand");
        };
        let [item] = values.items() else {
            panic!("one substituted size contribution");
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("all contributes a universal reset");
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
}

#[test]
fn shorthand_shares_size_grammar_without_consuming_following_family_or_line_height() {
    for value in [
        "italic 16px serif",
        "xxx-large serif",
        "math serif",
        "calc(-1px) serif",
        "16px math",
        "oblique 25deg 350.5 math serif",
        "oblique calc(10deg) calc(350 + 50) xxx-large serif",
        "16px/normal serif",
        "16px/1.2 serif",
        "16px/120% serif",
        "16px/1em serif",
    ] {
        let source = declaration("font", value);
        assert_eq!(source.known().unwrap().property(), CssKnownProperty::Font);
    }
    for invalid in [
        "-1e-50px serif",
        "-1e-50% serif",
        "1deg serif",
        "xxx-large",
        "math",
        "math serif/1.2",
        "small-caps small-caps 16px serif",
    ] {
        let source = format!("color:red;font:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid shorthand diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(validate_style_attribute(&source).is_err());
    }
}
