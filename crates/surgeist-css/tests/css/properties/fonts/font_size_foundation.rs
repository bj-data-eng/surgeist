//! Existing-public-API behavior from CSS Fonts 4 WD (2026-09-07) §§2.5, 2.7.
//! Exact typed payload and serialization contracts follow with the functional model.

use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::{checked_grammar_declaration, selected_grammar};
use surgeist_css::*;

fn one_size_contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        source.expand().expect("font-size intrinsic expansion")
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
fn keywords_sizes_and_shorthand_forms_are_valid() {
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
            checked_grammar_declaration("font-size", value)
                .known()
                .unwrap()
                .property(),
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
    assert_eq!(
        selected_grammar("font-size").target_property(),
        CssKnownProperty::FontSize
    );
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
        for source in [
            declaration("font-size", value),
            checked_grammar_declaration("font-size", value),
        ] {
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
    for (invalid, rest) in [
        ("-1e-50px", "-1e-50px"),
        ("-1e-50%", "-1e-50%"),
        ("-0.01px", "-0.01px"),
        ("-0.01%", "-0.01%"),
        ("1", "1"),
        ("1deg", "1deg"),
        ("calc(10deg)", "calc(10deg)"),
        ("calc(1)", "calc(1)"),
        ("medium large", "large"),
        ("math 16px", "16px"),
    ] {
        let source = format!("color:red;font-size:{invalid};color:blue");
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid size diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
        assert_checked_value_rejected_at(known_property("font-size"), invalid, rest);
    }
}

#[test]
fn inherited_medium_expands_once_with_globals_and_strict_pending_reentry() {
    let property = CssKnownProperty::FontSize;
    let CssPropertyKindRef::Longhand(longhand) =
        selected_grammar("font-size").metadata().unwrap().kind()
    else {
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
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("unresolved size remains pending");
    };
    assert!(pending.source().same_occurrence(&source));
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "-1e-50px").unwrap())
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
    for valid in ["xxx-large", "math", "125%"] {
        let replacement = CssComponentValues::parse(&CssParser::default(), valid).unwrap();
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
        declaration("all", "initial").expand().unwrap()
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
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid shorthand diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
    }
}
