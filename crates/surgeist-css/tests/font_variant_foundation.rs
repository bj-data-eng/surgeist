#![forbid(unsafe_code)]

//! Font variant grammar from Fonts 4 (2026-09-07), sections 6.8, 6.11 and 9.3.
//! Expectations come from the pinned specification, not parser output.

use surgeist_css::*;

const MEMBERS: [&str; 7] = [
    "font-variant-ligatures",
    "font-variant-caps",
    "font-variant-alternates",
    "font-variant-numeric",
    "font-variant-east-asian",
    "font-variant-position",
    "font-variant-emoji",
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("missing grammar: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

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

fn rejected(name: &str, value: &str) {
    let target = grammar(name).target_property();
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid-value diagnostic: {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
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
    assert!(validate_style_attribute(&source).is_err());
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
    assert_eq!(target.canonical_name(), name);
}

#[test]
fn existing_family_and_restricted_font_prefix_remain_valid() {
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
        let unsupported = parse_style_attribute(&source);
        assert!(
            !unsupported.is_clean(),
            "font keeps its restricted variant prefix: {value}"
        );
        assert!(validate_style_attribute(&source).is_err());
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
    for value in [
        "stylistic()",
        "stylistic(A, B)",
        "styleset()",
        "styleset(A B)",
        "character-variant(A,)",
        "swash(A B)",
        "historical-forms historical-forms",
        "stylistic(A) stylistic(B)",
        "normal historical-forms",
        "unknown-function(A)",
    ] {
        rejected("font-variant-alternates", value);
    }
}

#[test]
fn emoji_and_shorthand_reject_conflicting_or_exclusive_choices() {
    for value in ["emoji text", "normal emoji", "unicode unicode", "none"] {
        rejected("font-variant-emoji", value);
    }
    for value in [
        "normal stylistic(A)",
        "none emoji",
        "emoji text",
        "stylistic(A) stylistic(B)",
        "styleset(A B)",
    ] {
        rejected("font-variant", value);
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
            expand_declaration(&declaration(name, "normal")).unwrap()
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
            expand_declaration(&source).unwrap()
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
                expand_declaration(&declaration(name, expected)).unwrap()
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
