#![forbid(unsafe_code)]

//! Independent authored contracts: Color 5 WD 2026-09-08 §5.3,
//! Env 1 WD 2025-09-23 §3, Variables 1 CR 2022-06-16 §3,
//! Conditional Rules 3 CRD 2024-08-15 §3 and Nesting 1 WD 2026-01-22 §3.3.
//! Canonical spelling and fail-closed composition follow the CSS public specified
//! serialization contract. No resource loading or substitution is performed.

use surgeist_css::{
    CssNamespaceContext, CssNormalizedItem, CssRecoveryAction, CssRule, CssRuleContextKindRef,
    CssSpecifiedRuleSerializationErrorKind as RuleError,
    CssSpecifiedValueSerializationErrorKind as ValueError,
    CssSpecifiedValueSerializationLimits as Limits, normalize_report, parse_rule, parse_sheet,
    validate_sheet,
};

fn canonical(source: &str, expected: &str) {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    assert_eq!(
        validate_sheet(expected)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        expected
    );
}

#[test]
fn custom_profile_is_retained_before_following_style() {
    let source = "@color-profile --Press { src: url(press.icc); components: Cyan, Magenta, Yellow, Black; } .after { color: red; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 2);
    assert!(matches!(report.syntax().rules()[1], CssRule::Style(_)));
    assert_eq!(validate_sheet(source).unwrap().rules().len(), 2);
}

#[test]
fn reserved_device_profile_and_dashed_device_identity_remain_distinct() {
    canonical(
        "@COLOR-PROFILE DEVICE-CMYK {} @color-profile --device-cmyk {}",
        "@color-profile device-cmyk { }\n@color-profile --device-cmyk { }",
    );
}

#[test]
fn bare_two_dash_profile_name_is_a_valid_dashed_identifier() {
    // Values 4 WD 2024-03-12 §4.3 defines <dashed-ident> as a <custom-ident>
    // starting with two dashes, with no additional minimum-length restriction.
    // https://www.w3.org/TR/2024/WD-css-values-4-20240312/#typedef-dashed-ident
    canonical("@color-profile -- {}", "@color-profile -- { }");
}

#[test]
fn empty_and_incomplete_profiles_do_not_manufacture_descriptors() {
    canonical("@color-profile --Empty {}", "@color-profile --Empty { }");
    canonical(
        "@color-profile --Incomplete { components: red; }",
        "@color-profile --Incomplete { components: red; }",
    );
}

#[test]
fn isolated_rule_admission_agrees_with_clean_sheet_admission() {
    let source = "@color-profile --One { rendering-intent: PERCEPTUAL; }";
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "@color-profile --One { rendering-intent: perceptual; }"
    );
}

#[test]
fn all_intents_and_duplicate_occurrences_keep_authored_order() {
    canonical(
        "@color-profile --x { rendering-intent: RELATIVE-COLORIMETRIC; rendering-intent: ABSOLUTE-COLORIMETRIC; rendering-intent: PERCEPTUAL; rendering-intent: SATURATION; src:url(a); src:url(b); } @color-profile --x {}",
        "@color-profile --x { rendering-intent: relative-colorimetric; rendering-intent: absolute-colorimetric; rendering-intent: perceptual; rendering-intent: saturation; src: url(\"a\"); src: url(\"b\"); }\n@color-profile --x { }",
    );
}

#[test]
fn ordinary_component_identifiers_preserve_case_duplicates_and_escaped_punctuation() {
    canonical(
        r"@color-profile --Case { components: initial, inherit, unset, revert, revert-layer, default, alpha, pi, A, a, A, 日本, a\+b; }",
        r"@color-profile --Case { components: initial, inherit, unset, revert, revert-layer, default, alpha, pi, A, a, A, 日本, a\+b; }",
    );
}

#[test]
fn shared_url_forms_accept_empty_relative_fragment_and_function_like_data() {
    for (input, expected) in [
        ("url()", "url(\"\")"),
        ("url(relative.icc)", "url(\"relative.icc\")"),
        ("url(#profile)", "url(\"#profile\")"),
        ("src(\"var(--data)\")", "src(\"var(--data)\")"),
    ] {
        canonical(
            &format!("@color-profile --x {{ src: {input}; }}"),
            &format!("@color-profile --x {{ src: {expected}; }}"),
        );
    }
}

#[test]
fn environment_functions_defer_the_entire_descriptor_grammar_including_var_tokens() {
    for (name, value) in [
        ("src", "env(profile-url)"),
        ("rendering-intent", "env(intent, var(--x))"),
        ("rendering-intent", "env(intent) var(--x)"),
        ("components", "env(channels 0 1, red, green)"),
        ("components", r"e\6ev(channels)"),
    ] {
        let source = format!("@color-profile --x {{ {name}: {value}; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1);
        let emitted = report.syntax().to_specified_css().unwrap();
        assert!(validate_sheet(&emitted).is_ok());
    }
}

#[test]
fn ordinary_nested_groups_retain_profile_contexts_without_style_contributions() {
    let source = "@media all { @supports (color: red) { @layer ink { @color-profile --x {} @color-profile --x {} } } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.syntax().items().len(), 5);
    assert!(
        normalized
            .syntax()
            .items()
            .iter()
            .all(|item| matches!(item, CssNormalizedItem::Rule(_)))
    );
    let items = normalized.syntax().items();
    let CssNormalizedItem::Rule(layer) = &items[2] else {
        panic!("layer context")
    };
    assert!(matches!(layer.kind(), CssRuleContextKindRef::LayerBlock(_)));
    let CssNormalizedItem::Rule(first) = &items[3] else {
        panic!("first terminal")
    };
    let CssNormalizedItem::Rule(second) = &items[4] else {
        panic!("second terminal")
    };
    assert!(first.parent().unwrap().same_context(layer));
    assert!(second.parent().unwrap().same_context(layer));
    assert!(!first.same_context(second));
    assert_eq!(
        first.position().unwrap().byte_offset().value(),
        source.find("@color-profile").unwrap()
    );
    assert_eq!(
        second.position().unwrap().byte_offset().value(),
        source.rfind("@color-profile").unwrap()
    );
}

#[test]
fn scoped_group_profile_is_retained_with_scope_ancestry() {
    let source = "@scope (.host) { @media all { @color-profile --Scoped {} } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_report(&report).unwrap();
    let [
        CssNormalizedItem::Rule(scope),
        CssNormalizedItem::Rule(media),
        CssNormalizedItem::Rule(profile),
    ] = normalized.syntax().items()
    else {
        panic!("scope, media and terminal profile must survive");
    };
    assert!(matches!(scope.kind(), CssRuleContextKindRef::Scope { .. }));
    assert!(matches!(media.kind(), CssRuleContextKindRef::Media(_)));
    assert!(profile.parent().unwrap().same_context(media));
    assert!(media.parent().unwrap().same_context(scope));
    assert_eq!(
        profile.position().unwrap().byte_offset().value(),
        source.find("@color-profile").unwrap()
    );
}

#[test]
fn malformed_outer_names_and_missing_blocks_drop_only_the_outer_rule() {
    for invalid in [
        "@color-profile {}",
        "@color-profile plain {}",
        "@color-profile --x --y {}",
        "@color-profile \"--x\" {}",
        "@color-profile env(name) {}",
        "@color-profile --x;",
    ] {
        let source = format!("{invalid} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(&source).is_err());
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
    }
}

#[test]
fn style_ancestry_rejects_profiles_and_preserves_style_declarations() {
    for body in [
        "@color-profile --x {}",
        "@media all { @color-profile --x {} }",
    ] {
        let source = format!(".host {{ color: red; {body} }} .after {{ color: blue; }}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(&source).is_err());
        assert_eq!(report.syntax().rules().len(), 2);
        let CssRule::Style(style) = &report.syntax().rules()[0] else {
            panic!("host style")
        };
        assert_eq!(style.declarations().len(), 1);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
    }
}

#[test]
fn descriptor_recovery_retains_valid_siblings_profile_and_following_rule() {
    for invalid in [
        "unknown: x",
        "rendering-intent: auto",
        "components: none",
        r"components: n\6f ne",
        "components: a b",
        "components: a,",
        "components: a,,b",
        "components: \"a\"",
        "components: 1",
        "components:",
        "src: 1",
        "src: var(--x)",
        "rendering-intent: env()",
        "components: env(channels -1)",
        "rendering-intent: perceptual !important",
        "components: a !other",
    ] {
        let source = format!(
            "@color-profile --x {{ src: url(good); {invalid}; components: a, b; }} .after {{ color: red; }}"
        );
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(&source).is_err());
        assert_eq!(
            report.syntax().rules().len(),
            2,
            "profile and following style must survive: {invalid}"
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor
        );
        assert_eq!(
            report.syntax().rules()[0].to_specified_css().unwrap(),
            "@color-profile --x { src: url(\"good\"); components: a, b; }"
        );
        let normalized = normalize_report(&report).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
    }
}

#[test]
fn malformed_child_rule_is_recovered_inside_the_profile() {
    let source = "@color-profile --x { src: url(good); @media all {} components: a; } .after { color: red; }";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    assert!(validate_sheet(source).is_err());
    assert_eq!(report.syntax().rules().len(), 2);
    assert_eq!(
        report.syntax().rules()[0].to_specified_css().unwrap(),
        "@color-profile --x { src: url(\"good\"); components: a; }"
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn profile_palette_and_named_supports_leaf_sheet_has_explicit_canonical_output() {
    canonical(
        "@font-palette-values --p { font-family: Demo; } @color-profile --x {} @supports-condition --f {}",
        "@font-palette-values --p { font-family: Demo; }\n@color-profile --x { }\n@supports-condition --f { }",
    );
}

#[test]
fn profiles_compose_with_style_and_group_siblings_under_atomic_limits() {
    for (source, suffix) in [
        (".after { color: red; }", ".after { color: red; }"),
        ("@media all {}", "@media all { }"),
        ("@scope (.x) {}", "@scope (.x) { }"),
    ] {
        let report = parse_sheet(&format!("@color-profile --x {{}} {source}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let before = report.clone();
        let expected = format!("@color-profile --x {{ }}\n{suffix}");
        assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
        let error = report
            .syntax()
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
            .unwrap_err();
        assert_eq!(error.kind(), RuleError::Resource(ValueError::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(report, before);
    }
}

#[test]
fn named_supports_definitions_compose_with_style_and_group_siblings() {
    for (source, suffix) in [
        (".after { color: red; }", ".after { color: red; }"),
        ("@media all {}", "@media all { }"),
        ("@scope (.x) {}", "@scope (.x) { }"),
    ] {
        canonical(
            &format!("@supports-condition --f {{}} {source}"),
            &format!("@supports-condition --f {{ }}\n{suffix}"),
        );
    }
}

#[test]
fn cumulative_sheet_limits_include_supported_prefix_and_profile_punctuation() {
    let report =
        parse_sheet("@supports-condition --f {} @color-profile --日本 { components: red, green; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let expected = "@supports-condition --f { }\n@color-profile --日本 { components: red, green; }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let error = report
        .syntax()
        .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
        .unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(ValueError::ByteLimit));
    assert_eq!(error.rule_index(), Some(1));
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    for (input, kind) in [
        (true, ValueError::InputNodeLimit),
        (false, ValueError::ProjectionNodeLimit),
    ] {
        let found = (1..200).any(|limit| {
            let limits = Limits::new(if input { limit } else { usize::MAX }, if input { usize::MAX } else { limit }, usize::MAX);
            report.syntax().rules().iter().all(|rule| rule.to_specified_css_with_limits(limits).is_ok())
                && matches!(report.syntax().to_specified_css_with_limits(limits), Err(ref error) if error.kind() == RuleError::Resource(kind) && error.rule_index() == Some(1))
        });
        assert!(
            found,
            "individual rules must fit while their cumulative sheet fails: {kind:?}"
        );
    }
}

#[test]
fn shared_color_url_and_palette_substitution_controls_stay_clean() {
    let source = ".x { color: device-cmyk(0 0 0 1); background-image: url(relative.png); } @font-palette-values --p { font-family: Demo; base-palette: var(--p); }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(validate_sheet(source).is_ok());
}
