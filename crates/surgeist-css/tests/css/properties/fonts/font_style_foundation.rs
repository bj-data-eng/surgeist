//! Existing-public-API contracts from CSS Fonts 4 WD (2026-09-07) §§2.4, 2.7.
//! Typed angle and authored-value contracts follow with the functional model.

use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::{checked_grammar_declaration, selected_grammar};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssPropertyKindRef, CssPropertyNameRef,
};

#[test]
fn normal_and_italic_remain_valid_style_controls() {
    for value in ["normal", "italic", "oblique"] {
        let parsed = declaration("font-style", value);
        assert_eq!(
            parsed.known().unwrap().property(),
            CssKnownProperty::FontStyle
        );
        assert_eq!(parsed.importance(), CssImportance::Important);
        assert_eq!(
            checked_grammar_declaration("font-style", value)
                .known()
                .unwrap()
                .property(),
            CssKnownProperty::FontStyle
        );
    }
}

#[test]
fn property_accepts_direction_keywords_bounded_angles_and_symbolic_angle_math() {
    for value in [
        "left",
        "right",
        "oblique 90deg",
        "oblique -90deg",
        "oblique 100grad",
        "oblique .25turn",
        "oblique 1.5707963267948966rad",
        "oblique calc(100deg)",
    ] {
        for source in [
            declaration("font-style", value),
            checked_grammar_declaration("font-style", value),
        ] {
            assert_eq!(
                source.known().unwrap().property(),
                CssKnownProperty::FontStyle
            );
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
}

#[test]
fn exact_outside_angles_and_wrong_domains_drop_only_the_style_declaration() {
    for (invalid, rest) in [
        (
            "oblique 90.00000000000000000001deg",
            "90.00000000000000000001deg",
        ),
        (
            "oblique 100.00000000000000000001grad",
            "100.00000000000000000001grad",
        ),
        (
            "oblique .25000000000000000001turn",
            ".25000000000000000001turn",
        ),
        ("oblique 1.5707963267948968rad", "1.5707963267948968rad"),
        ("oblique 0", "0"),
        ("oblique 1px", "1px"),
        ("oblique 1%", "1%"),
        ("oblique calc(16px)", "calc(16px)"),
        ("oblique 10deg 20deg", "20deg"),
        ("left right", "right"),
    ] {
        let source = format!("color:red;font-style:{invalid};color:blue");
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid style diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(
            report.syntax().len(),
            2,
            "valid neighbors survive: {source}"
        );
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
        assert_checked_value_rejected_at(known_property("font-style"), invalid, rest);
    }
}

#[test]
fn inherited_normal_style_has_one_longhand_contribution_and_strict_reentry() {
    let property = CssKnownProperty::FontStyle;
    let CssPropertyKindRef::Longhand(longhand) =
        selected_grammar("font-style").metadata().unwrap().kind()
    else {
        panic!("font-style is a longhand");
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("normal is the fixed initial value");
    };
    let normal = declaration("font-style", "normal");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        normal.expand().expect("intrinsic style expansion")
    else {
        panic!("one style contribution");
    };
    let [item] = values.items() else {
        panic!("one style contribution");
    };
    assert_eq!(item.property(), property);
    assert_eq!(item.ordinary_value(), Some(initial));
    assert!(item.source().same_occurrence(&normal));
    assert_eq!(item.source().importance(), CssImportance::Important);

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
    ] {
        let source = declaration("font-style", spelling);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            source.expand().unwrap()
        else {
            panic!("symbolic global contributes once");
        };
        let [item] = values.items() else {
            panic!("one global contribution");
        };
        assert_eq!(item.property(), property);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
    }

    let source = declaration("font-style", "var(--slant)");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("unresolved style remains pending");
    };
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "oblique 0").unwrap())
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "oblique 90deg").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("strict replacement contributes a style");
    };
    let [item] = values.items() else {
        panic!("one substituted style contribution");
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        declaration("all", "initial").expand().unwrap()
    else {
        panic!("all contributes its universal reset");
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
}

#[test]
fn shorthand_consumes_angle_math_but_leaves_size_math_for_font_size() {
    for value in [
        "italic 16px serif",
        "oblique 16px serif",
        "oblique 90deg 16px serif",
        "oblique calc(10deg) 16px serif",
        "oblique calc(16px) serif",
    ] {
        declaration("font", value);
    }
    for invalid in [
        "oblique 90.00000000000000000001deg 16px serif",
        "oblique calc(10deg) italic 16px serif",
    ] {
        let source = format!("color:red;font:{invalid};color:blue");
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid shorthand diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2);
    }
}
