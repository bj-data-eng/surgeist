//! Font variant grammar from Fonts 4 (2026-09-07), sections 6.8, 6.11 and 9.3.
//! Expectations come from the pinned specification, not parser output.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::CssParser;
use surgeist_css::declaration::CssImportance;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssPropertyKindRef,
};

const MEMBERS: [&str; 7] = [
    "font-variant-ligatures",
    "font-variant-caps",
    "font-variant-alternates",
    "font-variant-numeric",
    "font-variant-east-asian",
    "font-variant-position",
    "font-variant-emoji",
];

fn accepted(name: &str, value: &str) {
    let parsed = declaration(name, value);
    let constructed = checked(name, value);
    assert_eq!(
        parsed.known().unwrap().property(),
        grammar(name).target_property()
    );
    assert_eq!(
        constructed.known().unwrap().property(),
        grammar(name).target_property()
    );
}

fn rejected<'a>(name: &str, value: &str, rest: impl Into<Option<&'a str>>) {
    let rest = rest.into();
    let target = grammar(name).target_property();
    let source = format!("color:red;{name}:{value};color:blue");
    let report = CssParser::default().parse_block_contents(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid-value diagnostic: {source}: {:?}",
            report.diagnostics()
        )
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
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .is_err()
    );
    assert_checked_value_rejected_at(known_property(name), value, rest);
    assert_eq!(target.canonical_name(), name);
}

#[test]
fn family_and_restricted_font_prefix_are_valid() {
    for (name, value) in [
        ("font-variant-ligatures", "common-ligatures no-contextual"),
        ("font-variant-caps", "small-caps"),
        ("font-variant-numeric", "oldstyle-nums tabular-nums"),
        ("font-variant-east-asian", "jis04 ruby"),
        ("font-variant-position", "super"),
        (
            "font-variant",
            "common-ligatures small-caps oldstyle-nums jis04 super",
        ),
        ("font", "italic small-caps 700 16px serif"),
    ] {
        accepted(name, value);
    }
    for value in [
        "italic petite-caps 16px serif",
        "italic stylistic(Style) 16px serif",
        "italic emoji 16px serif",
    ] {
        let source = format!("font:{value}");
        let unsupported = CssParser::default().parse_block_contents(&source);
        assert!(
            !unsupported.is_clean(),
            "font keeps its restricted variant prefix: {value}"
        );
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
    }
}

#[test]
fn alternates_accept_seven_distinct_components_and_case_sensitive_names() {
    for value in [
        "normal",
        "stylistic(normal)",
        "historical-forms",
        "styleset(Alpha, Alpha, beta)",
        "character-variant(First, second)",
        "swash(unset)",
        "ornaments(revert)",
        "annotation(unknownName)",
        "annotation(mark) ornaments(orn) swash(sw) character-variant(CV1, CV2) styleset(Set1, Set2) historical-forms stylistic(Style)",
    ] {
        accepted("font-variant-alternates", value);
    }
    // Escaped punctuation is an identifier, and its decoded spelling remains distinct.
    accepted("font-variant-alternates", r"styleset(A\+B, A\+B)");
}

#[test]
fn emoji_has_four_independent_keyword_choices() {
    for value in ["normal", "text", "emoji", "unicode"] {
        accepted("font-variant-emoji", value);
    }
}

#[test]
fn alternates_reject_empty_extra_duplicate_and_bad_separator_forms() {
    // Fonts 4: stylistic() takes one feature-value name, so the comma after
    // it is responsible.
    rejected("font-variant-alternates", "stylistic(A, B)", ", B)");
    for (value, rest) in [
        ("stylistic()", ")"),
        ("styleset()", ")"),
        ("styleset(A B)", "B)"),
        ("character-variant(A,)", ")"),
        ("swash(A B)", "B)"),
        ("historical-forms historical-forms", "historical-forms"),
        ("stylistic(A) stylistic(B)", "stylistic(B)"),
        ("normal historical-forms", "historical-forms"),
        ("unknown-function(A)", "unknown-function(A)"),
    ] {
        rejected("font-variant-alternates", value, rest);
    }
}

#[test]
fn emoji_and_shorthand_reject_conflicting_or_exclusive_choices() {
    for (value, rest) in [
        ("emoji text", "text"),
        ("normal emoji", "emoji"),
        ("unicode unicode", "unicode"),
        ("none", "none"),
    ] {
        rejected("font-variant-emoji", value, rest);
    }
    for (value, rest) in [
        ("normal stylistic(A)", "stylistic(A)"),
        ("none emoji", "emoji"),
        ("emoji text", "text"),
        ("stylistic(A) stylistic(B)", "stylistic(B)"),
        ("styleset(A B)", "B)"),
    ] {
        rejected("font-variant", value, rest);
    }
}

#[test]
fn full_shorthand_accepts_all_seven_groups_and_exclusive_top_level_values() {
    for value in [
        "normal",
        "none",
        "stylistic(Style)",
        "historical-forms text",
        "common-ligatures small-caps stylistic(Style) oldstyle-nums jis04 super emoji",
        "unicode super ruby tabular-nums annotation(Mark) all-small-caps no-contextual",
    ] {
        accepted("font-variant", value);
    }
}

#[test]
fn seven_inherited_longhands_have_normal_initials_and_exact_shorthand_targets() {
    for name in MEMBERS {
        let metadata = grammar(name)
            .metadata()
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{name} must be an intrinsic longhand")
        };
        assert!(longhand.inherited_by_default(), "{name}");
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} must have a fixed normal initial")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(normal)) =
            declaration(name, "normal").expand().unwrap()
        else {
            panic!("{name} must expand to its own longhand")
        };
        let [normal] = normal.items() else {
            panic!("{name} must contribute once")
        };
        assert_eq!(normal.ordinary_value(), Some(initial), "{name}");
    }

    let CssPropertyKindRef::Shorthand(shorthand) =
        grammar("font-variant").metadata().unwrap().kind()
    else {
        panic!("font-variant must have intrinsic shorthand metadata")
    };
    assert!(shorthand.reset_only_members().is_empty());
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        MEMBERS,
    );
    for (value, ligatures) in [
        ("normal", "normal"),
        ("none", "none"),
        ("small-caps", "normal"),
    ] {
        let source = declaration("font-variant", value);
        let CssExpansion::Contributions(CssContributions::Longhands(contributions)) =
            source.expand().unwrap()
        else {
            panic!("{value} must expand")
        };
        assert_eq!(contributions.items().len(), 7, "{value}");
        for (item, name) in contributions.items().iter().zip(MEMBERS) {
            assert_eq!(item.property(), grammar(name).target_property());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            let expected = if name == "font-variant-ligatures" {
                ligatures
            } else if name == "font-variant-caps" && value == "small-caps" {
                "small-caps"
            } else {
                "normal"
            };
            let CssExpansion::Contributions(CssContributions::Longhands(one)) =
                declaration(name, expected).expand().unwrap()
            else {
                panic!("{name} expected longhand")
            };
            assert_eq!(
                item.ordinary_value(),
                one.items()[0].ordinary_value(),
                "{value}: {name}"
            );
        }
    }
}
