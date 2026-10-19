#![forbid(unsafe_code)]

//! Font reset dependencies from pinned CSS Fonts 4 (2026-09-07), §§2.6, 4.6,
//! 6.3, 6.12, 6.13, 8.1 and 8.2. Expectations are independent of parser output.

use surgeist_css::*;

const SIX: [(&str, &str, &str); 6] = [
    ("font-feature-settings", "normal", "normal"),
    ("font-kerning", "auto", "auto"),
    ("font-size-adjust", "none", "none"),
    ("font-language-override", "normal", "normal"),
    ("font-optical-sizing", "auto", "auto"),
    ("font-variation-settings", "normal", "normal"),
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

fn accepted(name: &str, value: &str) {
    let parsed = declaration(name, value);
    let checked = parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"));
    assert_eq!(
        parsed.known().unwrap().property(),
        grammar(name).target_property()
    );
    assert_eq!(
        checked.known().unwrap().property(),
        grammar(name).target_property()
    );
}

fn rejected(name: &str, value: &str) {
    // Checking the known grammar first distinguishes a malformed value from an
    // unregistered property, even when the parser preserves valid neighbors.
    let target = grammar(name).target_property();
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one value error for {source}: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(
        report.syntax().len(),
        2,
        "valid neighbors survive: {source}"
    );
    for neighbor in report.syntax().as_slice() {
        assert_eq!(
            neighbor.known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
    assert!(validate_style_attribute(&source).is_err());
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked constructor accepted {source}"
    );
    assert_eq!(target.canonical_name(), name);
}

fn descriptor_accepted(name: &str, value: &str) {
    let source = format!("@font-face{{font-family:Demo;{name}:{value};src:url(face)}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("one font face: {source}")
    };
    assert_eq!(face.descriptors().occurrences().count(), 3);
}

fn descriptor_rejected(name: &str, value: &str) {
    let source = format!("@font-face{{font-family:Demo;{name}:{value};src:url(face)}}");
    let report = parse_sheet(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one descriptor error for {source}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("valid font face remains")
    };
    assert_eq!(face.descriptors().occurrences().count(), 2);
    assert!(validate_sheet(&source).is_err());
}

#[test]
fn existing_font_controls_remain_accepted() {
    for (name, value) in [
        ("font-feature-settings", "normal"),
        ("font-feature-settings", "\"kern\" on, \"liga\" 0"),
        ("font-kerning", "auto"),
        ("font-kerning", "normal"),
        ("font-size-adjust", "none"),
        ("font-size-adjust", "0.5"),
        ("font", "italic small-caps 700 16px serif"),
    ] {
        accepted(name, value);
    }
    descriptor_accepted("font-feature-settings", "\"kern\" on");
}

#[test]
fn feature_tags_use_four_printable_ascii_characters_and_preserve_case() {
    for value in ["\" abc\" on", "\"~ABC\" off", "\"aBcD\" 2, \"ABCD\" 3"] {
        accepted("font-feature-settings", value);
        descriptor_accepted("font-feature-settings", value);
    }
    for value in [
        r#""\1 abc" on"#,
        r#""\7f abc" on"#,
        "\"abc\" on",
        "\"abcde\" on",
        "\"éabc\" on",
    ] {
        rejected("font-feature-settings", value);
        descriptor_rejected("font-feature-settings", value);
    }
}

#[test]
fn feature_indices_accept_unbounded_integer_literals_and_symbolic_integer_math() {
    for value in [
        "\"cv01\" 2147483648",
        "\"cv01\" 999999999999999999999999999999999999",
        "\"cv01\" calc(2 + 3)",
        "\"cv01\" -0",
    ] {
        accepted("font-feature-settings", value);
        descriptor_accepted("font-feature-settings", value);
    }
    for value in [
        "\"cv01\" -1",
        "\"cv01\" 1.5",
        "\"cv01\" 1e2",
        "\"cv01\" 2%",
        "\"cv01\" 2px",
    ] {
        rejected("font-feature-settings", value);
        descriptor_rejected("font-feature-settings", value);
    }
}

#[test]
fn feature_settings_reject_empty_and_malformed_lists() {
    for value in ["", "normal, \"kern\"", "\"kern\",", "\"kern\" \"liga\""] {
        rejected("font-feature-settings", value);
        descriptor_rejected("font-feature-settings", value);
    }
}

#[test]
fn kerning_has_only_three_keywords() {
    for value in ["auto", "normal", "none"] {
        accepted("font-kerning", value);
    }
    for value in ["auto none", "optimizeSpeed", "1"] {
        rejected("font-kerning", value);
    }
}

#[test]
fn size_adjust_keeps_exact_nonnegative_numbers_and_symbolic_math() {
    for value in ["none", "0", "-0", "0.5", "1e999", "calc(1 / 2)"] {
        accepted("font-size-adjust", value);
    }
}

#[test]
fn size_adjust_rejects_negative_values_even_when_float_narrowing_underflows() {
    for value in ["-1e-999", "-0.000000000000000000000000000000000000001"] {
        rejected("font-size-adjust", value);
    }
}

#[test]
fn size_adjust_rejects_wrong_domains_and_unselected_forms() {
    for value in ["1px", "50%", "from-font", "ex-height 0.5"] {
        rejected("font-size-adjust", value);
    }
}

#[test]
fn language_override_accepts_general_strings_and_rejects_other_domains() {
    for value in [
        "normal",
        "\"\"",
        "\"SRB \u{e9} Long\"",
        "\"trK\"",
        r#""a\22 b""#,
    ] {
        accepted("font-language-override", value);
    }
    for value in ["none", "SRB", "\"x\" \"y\""] {
        rejected("font-language-override", value);
    }
}

#[test]
fn optical_sizing_accepts_only_auto_or_none() {
    for value in ["auto", "none"] {
        accepted("font-optical-sizing", value);
    }
    for value in ["normal", "auto none", "1"] {
        rejected("font-optical-sizing", value);
    }
}

#[test]
fn variation_settings_accept_signed_fractional_and_duplicate_axes_in_both_contexts() {
    for value in [
        "normal",
        "\"wght\" -1.25",
        "\"wdth\" +0.5, \"wdth\" -2, \"WDTH\" 1e3",
        "\" abc\" 0, \"~ABC\" 12",
        "\"wght\" calc(1 - 2)",
    ] {
        accepted("font-variation-settings", value);
        descriptor_accepted("font-variation-settings", value);
    }
}

#[test]
fn variation_settings_reject_bad_tags_and_number_domains_in_both_contexts() {
    for value in [
        "\"abc\" 1",
        "\"abcde\" 1",
        "\"éabc\" 1",
        r#""\1 abc" 1"#,
        r#""\7f abc" 1"#,
        "\"wght\" 2%",
        "\"wght\" 2px",
        "\"wght\"",
        "\"wght\" 1,",
        "normal, \"wght\" 1",
    ] {
        rejected("font-variation-settings", value);
        descriptor_rejected("font-variation-settings", value);
    }
}

#[test]
fn variation_descriptor_has_independent_ordered_recovery_and_css_wide_rejection() {
    descriptor_accepted("font-variation-settings", "\"wght\" -1");
    for value in ["inherit", "unset", "revert-layer", "var(--axis)"] {
        descriptor_rejected("font-variation-settings", value);
    }
    let source = "@font-face{font-variation-settings:\"wght\" 1;font-variation-settings:bad;font-variation-settings:\"wght\" 2}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face retained")
    };
    assert_eq!(face.descriptors().occurrences().count(), 2);
}

#[test]
fn six_inherited_longhands_have_initials_and_one_coupled_contribution() {
    for (name, initial_css, ordinary_css) in SIX {
        let metadata = grammar(name).metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{name} must have intrinsic longhand metadata")
        };
        assert!(longhand.inherited_by_default(), "{name}");
        let initial_value = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("{name} has fixed intrinsic initial")
        };
        let source = declaration(name, ordinary_css);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} must expand")
        };
        let [contribution] = values.items() else {
            panic!("{name} contributes once")
        };
        assert_eq!(contribution.property(), grammar(name).target_property());
        assert!(contribution.source().same_occurrence(&source));
        assert_eq!(contribution.source().importance(), CssImportance::Important);
        assert!(contribution.replacement_components().is_none());
        let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
            expand_declaration(&declaration(name, initial_css)).unwrap()
        else {
            panic!("{name} initial expansion")
        };
        assert_eq!(expected.items()[0].ordinary_value(), Some(initial));
    }
}

#[test]
fn six_longhands_preserve_css_wide_and_pending_reentry() {
    for (name, _, _) in SIX {
        let source = declaration(name, "inherit");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("{name} inherit expands")
        };
        let [contribution] = values.items() else {
            panic!("one {name} inherit contribution")
        };
        assert_eq!(
            contribution.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );

        let pending_source = declaration(name, "var(--font-control)");
        let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
            panic!("{name} must defer var()")
        };
        assert!(pending.source().same_occurrence(&pending_source));
        assert!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .is_err()
        );
        let replacement = match name {
            "font-kerning" | "font-optical-sizing" => "auto",
            "font-size-adjust" => "0.5",
            _ => "normal",
        };
        let CssContributions::Longhands(values) = pending
            .reenter(parse_component_values(replacement).unwrap())
            .unwrap()
        else {
            panic!("{name} reentry contributes")
        };
        let [contribution] = values.items() else {
            panic!("one {name} reentry contribution")
        };
        assert_eq!(contribution.property(), grammar(name).target_property());
        assert!(contribution.source().same_occurrence(&pending_source));
        assert!(contribution.replacement_components().is_some());
    }
}

#[test]
fn feature_and_variation_descriptors_defer_env_and_reenter_strictly() {
    for (name, ordinary) in [
        ("font-feature-settings", "\"cv01\" 2147483648"),
        ("font-variation-settings", "\"wght\" -1.25"),
    ] {
        let source = format!("@font-face{{{name}:env(axis, fallback)}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("font face retained")
        };
        let descriptors = face.descriptors().occurrences().collect::<Vec<_>>();
        let [descriptor] = descriptors.as_slice() else {
            panic!("one descriptor")
        };
        let CssAuthoredFontFaceDescriptorValue::Pending(pending) = descriptor.value() else {
            panic!("{name} env pending")
        };
        assert!(
            pending
                .reparse_after_substitution(parse_component_values("env(axis)").unwrap())
                .is_err()
        );
        assert!(
            pending
                .reparse_after_substitution(parse_component_values("inherit").unwrap())
                .is_err()
        );
        assert!(
            pending
                .reparse_after_substitution(parse_component_values(ordinary).unwrap())
                .is_ok()
        );
    }
}
