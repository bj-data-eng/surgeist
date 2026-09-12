use surgeist_css::{CssNamespaceContext, parse_rule, parse_sheet, validate_sheet};

// Fonts 4 (7 September 2026), section 6.9.1 defines these authored forms.
// The indices below satisfy both conflicting clauses in sections 6.9.1/6.9.2.
// Retention and report cleanliness are independent of font activation/cascade.
#[test]
fn font_feature_values_retains_empty_named_family_rules() {
    for source in [
        "@font-feature-values Demo {}",
        "@font-feature-values Demo, \"Other Font\" {}",
        "@font-feature-values \"serif\" {}",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(report.syntax().is_some(), "{source}");
        assert!(validate_sheet(source).is_ok(), "{source}");
    }
}

#[test]
fn font_feature_values_retains_all_defined_blocks_and_display_descriptor() {
    let source = "@font-feature-values Demo {\
        font-display: swap;\
        @stylistic { fancy: 1; }\
        @historical-forms { old: 1 2; }\
        @styleset { joined: 1 2; }\
        @character-variant { open: 1 2; }\
        @swash { flowing: 2; }\
        @ornaments { fleur: 3; }\
        @annotation { circled: 4; }\
    }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 1);
    assert!(validate_sheet(source).is_ok());
}

#[test]
fn font_feature_values_rejects_missing_or_generic_family_preludes() {
    for source in [
        "@font-feature-values {}",
        "@font-feature-values serif {}",
        "@font-feature-values Demo, {}",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean(), "{source}");
        assert!(validate_sheet(source).is_err(), "{source}");
    }
}

use surgeist_css::{
    CssErrorCode, CssFontDisplay, CssFontFaceFamily, CssFontFeatureDisplayOccurrence,
    CssFontFeatureValueBlock as Block, CssFontFeatureValueDefinition as Definition,
    CssFontFeatureValueIndex as Index, CssFontFeatureValueKind as Kind,
    CssFontFeatureValueName as Name, CssFontFeatureValuesErrorKind as ConstructionError,
    CssFontFeatureValuesItem, CssFontFeatureValuesRule, CssNormalizationLimits, CssNormalizedItem,
    CssRule, CssRuleContextKindRef, normalize_sheet, normalize_sheet_with_limits,
};

fn authored(source: &str) -> CssFontFeatureValuesRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("expected one font rule")
    };
    rule.clone()
}
fn block(item: &CssFontFeatureValuesItem) -> &Block {
    let CssFontFeatureValuesItem::Block(block) = item else {
        panic!("expected block")
    };
    block
}
fn definition(indexes: &[u32]) -> Definition {
    Definition::try_new(
        Name::try_new("Fancy").unwrap(),
        indexes.iter().copied().map(Index::from).collect(),
    )
    .unwrap()
}
#[test]
fn typed_body_retains_order_duplicates_case_and_escaped_names() {
    let rule = authored(
        r#"@FONT-FEATURE-VALUES Demo, "serif", Demo {
        font-display: swap;
        @STYLISTIC { Fancy: +0002; fancy: -000; Fancy: 3; inherit: 4; a\+b: 5; }
        font-display: optional;
        @historical-forms { Old: 0 4 4; }
        @styleset { joined: 0 20; }
        @character-variant { open: 99 4294967296; }
        @swash { flowing: 2; }
        @ornaments { fleur: 3; }
        @annotation { circled: 4; }
        @stylistic {}
    }"#,
    );
    assert_eq!(
        rule.families()
            .iter()
            .map(CssFontFaceFamily::as_str)
            .collect::<Vec<_>>(),
        ["Demo", "serif", "Demo"]
    );
    assert_eq!(rule.items().len(), 10);
    let CssFontFeatureValuesItem::FontDisplay(display) = &rule.items()[0] else {
        panic!()
    };
    assert_eq!(display.value(), CssFontDisplay::Swap);
    let CssFontFeatureValuesItem::FontDisplay(display) = &rule.items()[2] else {
        panic!()
    };
    assert_eq!(display.value(), CssFontDisplay::Optional);
    let definitions = block(&rule.items()[1]).definitions();
    assert_eq!(
        definitions
            .iter()
            .map(|value| value.name().as_str())
            .collect::<Vec<_>>(),
        ["Fancy", "fancy", "Fancy", "inherit", "a+b"]
    );
    assert_eq!(definitions[0].indexes()[0].as_decimal_str(), "2");
    assert_eq!(definitions[1].indexes()[0].as_decimal_str(), "0");
    let kinds = rule
        .items()
        .iter()
        .filter_map(|item| match item {
            CssFontFeatureValuesItem::Block(block) => Some(block.kind()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            Kind::Stylistic,
            Kind::HistoricalForms,
            Kind::Styleset,
            Kind::CharacterVariant,
            Kind::Swash,
            Kind::Ornaments,
            Kind::Annotation,
            Kind::Stylistic
        ]
    );
    assert!(block(&rule.items()[9]).definitions().is_empty());
}
#[test]
fn exact_indices_have_no_machine_bound_and_preserve_numeric_origins() {
    let huge = "9876543210".repeat(200);
    let source = format!(
        "/*😀*/ @font-feature-values Demo {{ @swash {{ a\\+b: +000{huge}; }} @historical-forms {{ old: 4294967295 4294967296; }} }}"
    );
    let rule = authored(&source);
    let first = &block(&rule.items()[0]).definitions()[0];
    let index = &first.indexes()[0];
    assert_eq!(index.as_decimal_str(), huge);
    assert_eq!(index.to_u32(), None);
    let origin = index.origin().unwrap();
    assert_eq!(origin.source().as_str(), source);
    let span = origin.span();
    assert_eq!(
        &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
        format!("+000{huge}")
    );
    assert_eq!(
        first.position().unwrap().byte_offset().value(),
        source.find("a\\+b").unwrap()
    );
    assert_eq!(
        rule.position().unwrap().byte_offset().value(),
        source.find("@font").unwrap()
    );
    let other = &block(&rule.items()[1]).definitions()[0].indexes();
    assert_eq!(other[0].to_u32(), Some(u32::MAX));
    assert_eq!(other[1].to_u32(), None);
    assert!(
        origin
            .source()
            .same_snapshot(other[1].origin().unwrap().source())
    );
}
#[test]
fn checked_construction_normalizes_decimal_syntax_without_inventing_source() {
    for (input, expected) in [
        ("+0002", "2"),
        ("-000", "0"),
        ("000", "0"),
        ("4294967296", "4294967296"),
    ] {
        let index = Index::try_from_decimal(input).unwrap();
        assert_eq!(index.as_decimal_str(), expected);
        assert!(index.origin().is_none());
    }
    for input in ["", "+", "-", " 1", "1 ", "1.0", "1e2", "1,2", "++1", "١"] {
        assert_eq!(
            Index::try_from_decimal(input).unwrap_err().kind(),
            ConstructionError::InvalidIntegerSyntax,
            "{input}"
        );
    }
    assert_eq!(
        Index::try_from_decimal("-1").unwrap_err().kind(),
        ConstructionError::NegativeIndex
    );
    for input in ["a+b", "inherit", "initial", "unset", "revert", "default"] {
        assert_eq!(Name::try_new(input).unwrap().as_str(), input);
    }
    for input in ["", "a\0b"] {
        assert_eq!(
            Name::try_new(input).unwrap_err().kind(),
            ConstructionError::InvalidIdentifier
        );
    }
    assert_eq!(
        Definition::try_new(Name::try_new("a").unwrap(), vec![])
            .unwrap_err()
            .kind(),
        ConstructionError::EmptyIndexes
    );
    assert_eq!(
        CssFontFeatureValuesRule::try_new(vec![], vec![])
            .unwrap_err()
            .kind(),
        ConstructionError::EmptyFamilies
    );
    let definition = definition(&[1]);
    assert!(definition.position().is_none());
    let block = Block::try_new(Kind::Swash, vec![definition]).unwrap();
    assert!(block.position().is_none());
    let display = CssFontFeatureDisplayOccurrence::new(CssFontDisplay::Swap);
    assert!(display.position().is_none());
    let rule = CssFontFeatureValuesRule::try_new(
        vec![CssFontFaceFamily::try_new("serif").unwrap()],
        vec![
            CssFontFeatureValuesItem::FontDisplay(display),
            CssFontFeatureValuesItem::Block(block),
        ],
    )
    .unwrap();
    assert!(rule.position().is_none());
    assert_eq!(rule.families()[0].as_str(), "serif");
}
#[test]
fn provisional_6_9_1_character_variant_requires_two_indexes() {
    let error = Block::try_new(
        Kind::CharacterVariant,
        vec![definition(&[1, 2]), definition(&[1])],
    )
    .unwrap_err();
    assert_eq!(error.kind(), ConstructionError::InvalidIndexCount);
    assert_eq!(error.block(), Some(Kind::CharacterVariant));
    assert_eq!(error.definition_index(), Some(1));
    for value in ["1", "1 2 3"] {
        assert_invalid_value("character-variant", value);
    }
}
#[test]
fn provisional_6_9_1_character_variant_first_index_is_at_most_99() {
    let error = Block::try_new(
        Kind::CharacterVariant,
        vec![definition(&[1, 2]), definition(&[100, 1])],
    )
    .unwrap_err();
    assert_eq!(error.kind(), ConstructionError::IndexOutOfRange);
    assert_eq!(error.block(), Some(Kind::CharacterVariant));
    assert_eq!(error.definition_index(), Some(1));
    assert_eq!(error.index(), Some(0));
    assert_invalid_value("character-variant", "100 1");
    assert!(Block::try_new(Kind::CharacterVariant, vec![definition(&[99, u32::MAX])]).is_ok());
}
#[test]
fn provisional_6_9_1_styleset_range_is_shared() {
    let error = Block::try_new(Kind::Styleset, vec![definition(&[0, 20, 21])]).unwrap_err();
    assert_eq!(error.kind(), ConstructionError::IndexOutOfRange);
    assert_eq!(error.index(), Some(2));
    assert_invalid_value("styleset", "21");
    assert!(Block::try_new(Kind::Styleset, vec![definition(&[0, 20, 20])]).is_ok());
}
fn assert_invalid_value(kind: &str, value: &str) {
    let source = format!(
        "@font-feature-values Demo {{ @{kind} {{ bad: {value}; good: {}; }} }}",
        if kind == "character-variant" {
            "1 2"
        } else {
            "1"
        }
    );
    let report = parse_sheet(&source);
    assert!(!report.is_clean(), "{source}");
    assert!(validate_sheet(&source).is_err());
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("{source}: {:?}", report.diagnostics())
    };
    assert_eq!(block(&rule.items()[0]).definitions().len(), 1);
    assert_eq!(
        block(&rule.items()[0]).definitions()[0].name().as_str(),
        "good"
    );
}
#[test]
fn invalid_values_recover_only_the_definition() {
    for value in [
        "-1",
        "1.0",
        "1e0",
        "",
        "1,2",
        "1 2",
        "1 !important",
        "var(--x)",
        "1px",
        "1%",
        "1 x",
    ] {
        assert_invalid_value("swash", value);
    }
    assert!(Block::try_new(Kind::HistoricalForms, vec![definition(&[0, 1, u32::MAX])]).is_ok());
    for kind in [
        Kind::Stylistic,
        Kind::Swash,
        Kind::Ornaments,
        Kind::Annotation,
    ] {
        assert_eq!(
            Block::try_new(kind, vec![definition(&[1, 2])])
                .unwrap_err()
                .kind(),
            ConstructionError::InvalidIndexCount
        );
    }
}
#[test]
fn malformed_blocks_and_descriptors_leave_valid_siblings_and_empty_containers() {
    let source = "@font-feature-values Demo { font-display: invalid; @unknown { nested: [1]; } @swash extra { a: 1; } font-display: swap !important; font-display: block; @annotation { @swash { bad: 1; } keep: 2; } @swash {} }";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("{:?}", report.diagnostics())
    };
    assert_eq!(rule.items().len(), 3, "{:?}", report.diagnostics());
    assert_eq!(
        block(&rule.items()[1]).definitions()[0].name().as_str(),
        "keep"
    );
    assert!(block(&rule.items()[2]).definitions().is_empty());
    assert!(validate_sheet("@swash { a: 1; }").is_err());
    for source in [
        "@font-feature-values Demo;",
        "@font-feature-values Demo, serif {}",
        "@font-feature-values Demo,,Other {}",
    ] {
        assert!(parse_sheet(source).syntax().rules().is_empty());
        assert!(validate_sheet(source).is_err());
    }
}
#[test]
fn ordinary_groups_preserve_complete_font_payload_and_parent_context() {
    for (prefix, suffix) in [
        ("", ""),
        ("@media all {", "}"),
        ("@supports (color: red) {", "}"),
        ("@container (width > 1px) {", "}"),
        ("@layer theme {", "}"),
        ("@scope (.root) {", "}"),
        ("@scope (.root) { @media all {", "}}"),
    ] {
        let source = format!(
            "{prefix}@font-feature-values Demo {{ @swash {{ Fancy: 2; Fancy: 3; }} font-display: swap; }}{suffix}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let fonts = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Rule(context) => match context.kind() {
                    CssRuleContextKindRef::FontFeatureValues(rule) => Some((context, rule)),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(fonts.len(), 1, "{source}");
        assert_eq!(fonts[0].1.items().len(), 2);
        assert_eq!(block(&fonts[0].1.items()[0]).definitions().len(), 2);
        assert_eq!(fonts[0].0.parent().is_some(), !prefix.is_empty());
        assert!(
            normalized
                .items()
                .iter()
                .all(|item| matches!(item, CssNormalizedItem::Rule(_)))
        );
    }
}
#[test]
fn style_ancestry_rejects_font_rules_through_intervening_groups() {
    for (prefix, suffix) in [
        ("", ""),
        ("@media all {", "}"),
        ("@supports (color: red) {", "}"),
        ("@container (width > 1px) {", "}"),
        ("@layer theme {", "}"),
        ("@scope (.inner) {", "}"),
        ("@scope (.inner) { @media all {", "}}"),
        ("@scope (.inner) { @scope (.deeper) {", "}}"),
    ] {
        let source = format!(
            ".root {{ color: red; {prefix}@font-feature-values Demo {{ @swash {{ a: 1; }} }}{suffix} color: blue; }}"
        );
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(report.diagnostics().iter().any(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().rules().len(), 1);
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert!(normalized.items().iter().all(|item| !matches!(item, CssNormalizedItem::Rule(context) if matches!(context.kind(), CssRuleContextKindRef::FontFeatureValues(_)))));
        if prefix.starts_with("@scope") {
            assert!(normalized.items().iter().any(|item| matches!(item, CssNormalizedItem::Rule(context) if matches!(context.kind(), CssRuleContextKindRef::Scope { .. }))));
        }
    }
}
#[test]
fn normalization_counts_one_opaque_font_rule_and_fails_atomically_at_rule_limit() {
    let report =
        parse_sheet("@font-feature-values Demo { @swash { a: 1; b: 2; } font-display: swap; }");
    let limits = CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap();
    assert_eq!(
        normalize_sheet_with_limits(report.syntax(), limits)
            .unwrap()
            .items()
            .len(),
        1
    );
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 0, 0, 0).unwrap()
        )
        .is_err()
    );
}
#[test]
fn eof_retains_accepted_rules_with_real_closure_diagnostics() {
    let report = parse_sheet("@font-feature-values Demo { @swash { a: 1;");
    let [CssRule::FontFeatureValues(rule)] = report.syntax().rules() else {
        panic!("{:?}", report.diagnostics())
    };
    assert_eq!(
        block(&rule.items()[0]).definitions()[0].indexes()[0].as_decimal_str(),
        "1"
    );
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action()
                == surgeist_css::CssRecoveryAction::RetainWithImplicitClosure)
    );
}
#[test]
fn structural_limits_scan_even_discarded_subsidiary_contents() {
    for label in ["swash", "unknown"] {
        let source = format!(
            "@font-feature-values Demo {{ @{label} {{ a: {}1{}; }} @swash {{ keep: 2; }} }}",
            "(".repeat(260),
            ")".repeat(260)
        );
        let report = parse_sheet(&source);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.error().code() == CssErrorCode::NestingLimit),
            "{label}: {:?}",
            report.diagnostics()
        );
    }
}

#[test]
fn scoped_chunk_conversion_preserves_font_payload_at_the_structural_ceiling() {
    let source = format!(
        "@scope (.root) {{ {} @font-feature-values Demo {{ @swash {{ a: 2; }} }} {} }}",
        "@media all {".repeat(253),
        "}".repeat(253)
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let fonts = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::FontFeatureValues(rule) => Some(rule),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fonts.len(), 1);
    let index = &block(&fonts[0].items()[0]).definitions()[0].indexes()[0];
    assert_eq!(index.as_decimal_str(), "2");
    assert_eq!(index.origin().unwrap().source().as_str(), source);
    assert_eq!(
        index.origin().unwrap().span().start().byte_offset().value(),
        source.find("2;").unwrap()
    );
}

#[test]
fn style_ancestry_survives_scoped_chunk_splitting() {
    let source = format!(
        ".root {{ @scope (.inner) {{ {} @font-feature-values Demo {{ @swash {{ a: 1; }} }} {} }} }}",
        "@media all {".repeat(70),
        "}".repeat(70)
    );
    let report = parse_sheet(&source);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement),
        "{:?}",
        report.diagnostics()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert!(normalized.items().iter().all(|item| !matches!(item, CssNormalizedItem::Rule(context) if matches!(context.kind(), CssRuleContextKindRef::FontFeatureValues(_)))));
}

#[test]
fn inherited_scope_keeps_scoped_declaration_and_selector_semantics_across_chunks() {
    for depth in [1, 70] {
        let source = format!(
            ".root {{ @scope (.inner) {{ {} &.leaf {{ color: red; }} color: blue; {} }} }}",
            "@media all {".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&source);
        assert!(
            !report.is_clean(),
            "scope rule lists reject bare declarations, depth={depth}"
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let scoped_styles = normalized.items().iter().filter(|item| matches!(item, CssNormalizedItem::Rule(context) if matches!(context.kind(), CssRuleContextKindRef::ScopedStyle(_)))).count();
        assert_eq!(
            scoped_styles,
            1,
            "depth={depth}: {:?}",
            report.diagnostics()
        );
        let selectors = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(context) => match context.kind() {
                    CssRuleContextKindRef::ScopedStyle(selectors) => Some(selectors),
                    _ => None,
                },
                _ => None,
            })
            .unwrap();
        assert_eq!(
            selectors.selectors()[0].binding(),
            surgeist_css::CssSelectorBinding::ScopeAnchors
        );
        assert!(selectors.parent().is_none());
        assert!(selectors.scope_context().is_some());
        let declarations = normalized
            .items()
            .iter()
            .filter(|item| matches!(item, CssNormalizedItem::Declaration(_)))
            .count();
        assert_eq!(declarations, 1, "depth={depth}");
    }
}
