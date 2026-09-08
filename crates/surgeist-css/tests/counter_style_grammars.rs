use surgeist_css::{
    CssCounterStyleDescriptorRef, CssCounterStyleName, CssCounterStyleRange,
    CssCounterStyleRangeBound, CssCounterStyleSpeakAs, CssCounterStyleSystem, CssCounterSymbol,
    CssCounterSymbolIdent, CssErrorCode, CssRecoveryAction, CssRule, CssSupportStatus,
    feature_metadata, parse_sheet,
};

#[test]
fn counter_style_rules_retain_valid_core_definitions() {
    let report = parse_sheet(concat!(
        ".before { color: red; } ",
        "@counter-style cycle { system: cyclic; symbols: ● ○; prefix: 👍; suffix: \" \"; } ",
        "@counter-style digits { system: numeric; symbols: \"0\" \"1\"; } ",
        "@counter-style letters { system: alphabetic; symbols: a b; symbols: x y; } ",
        ".after { color: blue; }",
    ));

    assert!(
        report.is_clean(),
        "valid core counter styles should not recover: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 5);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.error().code() != CssErrorCode::UnsupportedAtRule)
    );

    let CssRule::CounterStyle(cycle) = &report.syntax().rules()[1] else {
        panic!("expected cyclic counter style")
    };
    assert_eq!(cycle.name().as_str(), "cycle");
    assert_eq!(cycle.position().byte_offset().value(), 24);
    assert!(matches!(
        cycle.descriptors().system().map(|value| value.value()),
        Some(CssCounterStyleSystem::Cyclic)
    ));
    let symbols = cycle.descriptors().symbols().expect("cyclic symbols");
    assert!(matches!(
        symbols.symbols(),
        [CssCounterSymbol::Ident(first), CssCounterSymbol::Ident(second)]
            if first.as_str() == "●" && second.as_str() == "○"
    ));
    assert!(matches!(
        cycle.descriptors().prefix().map(|value| value.value()),
        Some(CssCounterSymbol::Ident(value)) if value.as_str() == "👍"
    ));
    assert!(matches!(
        cycle.descriptors().suffix().map(|value| value.value()),
        Some(CssCounterSymbol::String(value)) if value.as_str() == " "
    ));
    assert_eq!(cycle.descriptors().occurrences().count(), 4);

    let CssRule::CounterStyle(digits) = &report.syntax().rules()[2] else {
        panic!("expected numeric counter style")
    };
    assert!(matches!(
        digits.descriptors().system().map(|value| value.value()),
        Some(CssCounterStyleSystem::Numeric)
    ));
    assert_eq!(digits.descriptors().symbols().unwrap().symbols().len(), 2);

    let CssRule::CounterStyle(letters) = &report.syntax().rules()[3] else {
        panic!("expected alphabetic counter style")
    };
    let occurrences = letters.descriptors().occurrences().collect::<Vec<_>>();
    assert!(matches!(
        occurrences.as_slice(),
        [
            CssCounterStyleDescriptorRef::System(_),
            CssCounterStyleDescriptorRef::Symbols(_),
            CssCounterStyleDescriptorRef::Symbols(_),
        ]
    ));
    assert!(matches!(
        letters.descriptors().symbols().unwrap().symbols(),
        [CssCounterSymbol::Ident(first), CssCounterSymbol::Ident(second)]
            if first.as_str() == "x" && second.as_str() == "y"
    ));
}

#[test]
fn counter_style_descriptors_enforce_domains_order_and_recovery() {
    let report = parse_sheet(concat!(
        "@counter-style base { system: cyclic; symbols: b; } ",
        "@counter-style additive { system: additive; negative: \"-\" \"(\"; ",
        "range: infinite -1, 1 infinite; pad: 2 \"0\"; fallback: base; ",
        "additive-symbols: 100 C, 10 X, 1 I, 0 N; speak-as: words; } ",
        "@counter-style inherited { system: extends base; negative: \"(\" \" )\"; ",
        "range: auto; pad: 0 \"\"; fallback: decimal; speak-as: base; }",
    ));

    assert!(
        report.is_clean(),
        "valid descriptor domains should be retained without recovery: {:?}",
        report.diagnostics()
    );
    let [
        CssRule::CounterStyle(base),
        CssRule::CounterStyle(additive),
        CssRule::CounterStyle(inherited),
    ] = report.syntax().rules()
    else {
        panic!("expected all valid counter styles to be retained")
    };
    assert_eq!(base.descriptors().occurrences().count(), 2);
    assert_eq!(additive.descriptors().occurrences().count(), 7);
    assert_eq!(inherited.descriptors().occurrences().count(), 6);
}

#[test]
fn counter_style_descriptor_models_preserve_authored_duplicates_and_effective_last_values() {
    let source = concat!(
        "@counter-style base { system: cyclic; symbols: b; } ",
        "@counter-style rich { system: additive; ",
        "negative: \"-\"; negative: \"(\" \" )\"; ",
        "range: auto; range: infinite -1, 1 infinite; ",
        "pad: 2 \"0\"; pad: \"_\" 3; ",
        "fallback: decimal; fallback: base; ",
        "additive-symbols: 10 X, 1 I; additive-symbols: C 100, X 10, I 1, N 0; ",
        "speak-as: words; speak-as: base; }",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(_), CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("expected base and rich counter styles")
    };

    let negative = rule.descriptors().negative().unwrap();
    assert!(matches!(
        negative.prefix(),
        CssCounterSymbol::String(value) if value.as_str() == "("
    ));
    assert!(matches!(
        negative.suffix(),
        Some(CssCounterSymbol::String(value)) if value.as_str() == " )"
    ));
    let CssCounterStyleRange::Ranges(ranges) = rule.descriptors().range().unwrap().value() else {
        panic!("expected explicit effective ranges")
    };
    assert_eq!(ranges.ranges().len(), 2);
    assert_eq!(
        ranges.ranges()[0].lower(),
        CssCounterStyleRangeBound::Infinite
    );
    assert_eq!(
        ranges.ranges()[0].upper(),
        CssCounterStyleRangeBound::Integer(-1)
    );
    assert_eq!(
        ranges.ranges()[1].lower(),
        CssCounterStyleRangeBound::Integer(1)
    );
    assert_eq!(
        ranges.ranges()[1].upper(),
        CssCounterStyleRangeBound::Infinite
    );

    let pad = rule.descriptors().pad().unwrap();
    assert_eq!(pad.minimum_length(), 3);
    assert!(matches!(
        pad.symbol(),
        CssCounterSymbol::String(value) if value.as_str() == "_"
    ));
    assert_eq!(rule.descriptors().fallback().unwrap().as_str(), "base");
    let tuples = rule.descriptors().additive_symbols().unwrap().tuples();
    assert_eq!(
        tuples
            .iter()
            .map(|tuple| tuple.weight())
            .collect::<Vec<_>>(),
        vec![100, 10, 1, 0]
    );
    assert!(matches!(
        rule.descriptors().speak_as().map(|value| value.value()),
        Some(CssCounterStyleSpeakAs::CounterStyle(name)) if name.as_str() == "base"
    ));
    assert_eq!(rule.descriptors().occurrences().count(), 13);
}

#[test]
fn invalid_counter_style_descriptor_values_drop_only_the_descriptor_and_keep_effective_values() {
    let source = concat!(
        ".before {} ",
        "@counter-style kept { system: additive; ",
        "negative: \"-\" \"+\" extra; negative: \"-\"; ",
        "range: 3 1; range: 1 3; ",
        "pad: -1 \"0\"; pad: \"0\" 2; ",
        "fallback: none; fallback: decimal; ",
        "additive-symbols: 10 X, 10 I; additive-symbols: 10 X, 1 I; ",
        "speak-as: inherit; speak-as: numbers; mystery: x; } ",
        ".after {}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Style(_),
            CssRule::CounterStyle(_),
            CssRule::Style(_)
        ]
    ));
    let CssRule::CounterStyle(rule) = &report.syntax().rules()[1] else {
        panic!("expected retained counter style")
    };
    assert_eq!(rule.descriptors().occurrences().count(), 7);
    assert!(
        matches!(rule.descriptors().range().unwrap().value(), CssCounterStyleRange::Ranges(ranges) if ranges.ranges().len() == 1)
    );
    assert_eq!(rule.descriptors().pad().unwrap().minimum_length(), 2);
    assert_eq!(rule.descriptors().fallback().unwrap().as_str(), "decimal");
    assert!(matches!(
        rule.descriptors().speak_as().map(|value| value.value()),
        Some(CssCounterStyleSpeakAs::Numbers)
    ));

    assert_eq!(report.diagnostics().len(), 7);
    assert!(report.diagnostics()[..6].iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue
            && diagnostic.action() == CssRecoveryAction::DropDescriptor
    }));
    assert_eq!(
        report.diagnostics()[6].error().code(),
        CssErrorCode::UnknownDescriptor
    );
    assert_eq!(
        report.diagnostics()[6].action(),
        CssRecoveryAction::DropDescriptor
    );
}

#[test]
fn invalid_effective_counter_style_combinations_drop_only_the_at_rule() {
    let source = concat!(
        ".before {} ",
        "@counter-style inherited-symbols { system: extends decimal; symbols: x; } ",
        "@counter-style inherited-additive { system: extends decimal; additive-symbols: 1 I; } ",
        "@counter-style missing-additive { system: additive; } ",
        "@counter-style missing-symbols { range: auto; } ",
        "@counter-style kept { system: additive; additive-symbols: 1 I; } ",
        ".after {}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Style(_),
            CssRule::CounterStyle(_),
            CssRule::Style(_)
        ]
    ));
    assert_eq!(report.diagnostics().len(), 4);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidDescriptorCombination
            && diagnostic.action() == CssRecoveryAction::DropAtRule
    }));
}

#[test]
fn counter_style_system_model_retains_fixed_and_extends_forms() {
    assert!(CssCounterStyleName::try_new("default").is_none());
    assert_eq!(
        CssCounterSymbolIdent::try_new("auto").unwrap().as_str(),
        "auto"
    );
    assert!(CssCounterSymbolIdent::try_new("default").is_none());

    let report = parse_sheet(concat!(
        "@counter-style fixedish { system: fixed -2; symbols: \"a\" \"b\"; } ",
        "@counter-style child { system: extends fixedish; }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::CounterStyle(fixed),
        CssRule::CounterStyle(extended),
    ] = report.syntax().rules()
    else {
        panic!("expected two counter styles")
    };
    assert!(matches!(
        fixed.descriptors().system().map(|value| value.value()),
        Some(CssCounterStyleSystem::Fixed(value)) if value.first_symbol_value() == Some(-2)
    ));
    assert!(matches!(
        extended.descriptors().system().map(|value| value.value()),
        Some(CssCounterStyleSystem::Extends(name)) if name.as_str() == "fixedish"
    ));
    assert!(extended.descriptors().symbols().is_none());
}

#[test]
fn counter_style_descriptor_recovery_keeps_valid_occurrences_and_siblings() {
    let source = concat!(
        ".before { color: red; } ",
        "@counter-style kept { system: cyclic; prefix: \"bad\" \"extra\"; ",
        "symbols: a; mystery: 1; suffix: \"ok\"; } ",
        "@counter-style too-short { system: numeric; symbols: a; } ",
        "@counter-style mixed { system: extends kept; symbols: a; } ",
        "@counter-style missing { system: additive; } ",
        ".after { color: blue; }",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Style(_),
            CssRule::CounterStyle(_),
            CssRule::Style(_)
        ]
    ));
    let CssRule::CounterStyle(kept) = &report.syntax().rules()[1] else {
        panic!("expected retained counter style")
    };
    assert!(kept.descriptors().prefix().is_none());
    assert!(matches!(
        kept.descriptors().suffix().map(|value| value.value()),
        Some(CssCounterSymbol::String(value)) if value.as_str() == "ok"
    ));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>(),
        vec![
            (
                CssErrorCode::InvalidDescriptorValue,
                CssRecoveryAction::DropDescriptor
            ),
            (
                CssErrorCode::UnknownDescriptor,
                CssRecoveryAction::DropDescriptor
            ),
            (
                CssErrorCode::InvalidDescriptorCombination,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidDescriptorCombination,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidDescriptorCombination,
                CssRecoveryAction::DropAtRule
            ),
        ]
    );
}

#[test]
fn counter_style_rules_enforce_top_level_body_phase_and_exact_positions() {
    let source = concat!(
        "@counter-style 🧭 { symbols: \"x\"; suffix: \"🚀\"; } ",
        "@import \"late.css\"; ",
        "@media screen { @counter-style nested { symbols: n; } .inside {} } ",
        ".after {}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::CounterStyle(_),
            CssRule::Media(_),
            CssRule::Style(_)
        ]
    ));
    let CssRule::CounterStyle(rule) = &report.syntax().rules()[0] else {
        panic!("expected counter style")
    };
    assert_eq!(rule.name().as_str(), "🧭");
    let suffix_offset = source.find("suffix").unwrap();
    let suffix = rule.descriptors().suffix().unwrap();
    assert_eq!(suffix.position().byte_offset().value(), suffix_offset);
    assert_eq!(
        suffix.position().column().value() as usize,
        source[..suffix_offset].encode_utf16().count()
    );
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect::<Vec<_>>(),
        vec![
            CssErrorCode::InvalidAtRulePlacement,
            CssErrorCode::InvalidAtRulePlacement,
        ]
    );
    let CssRule::Media(media) = &report.syntax().rules()[1] else {
        panic!("expected media rule")
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
}

#[test]
fn counter_style_block_recovery_handles_reserved_names_eof_and_depth_boundaries() {
    for name in ["none", "default", "inherit"] {
        let reserved = parse_sheet(&format!(
            "@counter-style {name} {{ symbols: x; }} .after {{}}"
        ));
        assert!(matches!(reserved.syntax().rules(), [CssRule::Style(_)]));
        assert_eq!(
            reserved.diagnostics()[0].error().code(),
            CssErrorCode::InvalidAtRulePrelude
        );
    }

    let statement = parse_sheet("@counter-style valid; .after {}");
    assert!(matches!(statement.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(
        statement.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
    let missing_block = parse_sheet("@counter-style valid");
    assert!(missing_block.syntax().rules().is_empty());
    assert_eq!(
        missing_block.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );

    let eof_source = "@counter-style eof { system: symbolic; symbols: 🚀";
    let eof = parse_sheet(eof_source);
    assert!(matches!(eof.syntax().rules(), [CssRule::CounterStyle(_)]));
    assert_eq!(
        eof.diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            })
            .count(),
        1
    );

    for depth in [255_usize, 256, 257] {
        let source = format!(
            "@counter-style deep{{system:cyclic;symbols:{}x{};}}.after{{}}",
            "f(".repeat(depth),
            ")".repeat(depth),
        );
        let report = parse_sheet(&source);
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
        let stopped = report.diagnostics().iter().any(|diagnostic| {
            diagnostic.error().code() == CssErrorCode::NestingLimit
                && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
        });
        assert_eq!(
            stopped,
            depth >= 256,
            "depth {depth}: {:?}",
            report.diagnostics()
        );
    }
}

#[test]
fn c11_rule_recovery_preserves_siblings_and_boundaries() {
    let source = concat!(
        "/* 😀 */\n",
        "@counter-style root { system: cyclic; ",
        "@page :left { margin: 1cm; } ",
        "@counter-style descriptor-child { symbols: d; } ",
        "@font-feature-values Demo { } ",
        "@mystery-descriptor-rule { } ",
        "symbols: r; } ",
        "@page { @counter-style page-child { symbols: p; } margin: 1cm; } ",
        "@media print { ",
        "@counter-style media-child { symbols: m; } ",
        "@page :right { margin: 2cm; } ",
        ".media-kept { color: red; } } ",
        ".host { ",
        "@counter-style style-child { symbols: s; } ",
        "@page :first { margin: 3cm; } ",
        "color: blue; } ",
        "@scope { ",
        "@counter-style scope-child { symbols: c; } ",
        "@page :left { margin: 4cm; } ",
        ".scope-kept { color: green; } } ",
        "@font-feature-values Tail { } ",
        "@mystery-tail { } ",
        ".tail { color: black; }",
    );
    let report = parse_sheet(source);

    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::CounterStyle(_),
            CssRule::Page(_),
            CssRule::Media(_),
            CssRule::Style(_),
            CssRule::Scope(_),
            CssRule::Style(_),
        ]
    ));
    let CssRule::CounterStyle(counter_style) = &report.syntax().rules()[0] else {
        panic!("expected recovered counter style")
    };
    assert_eq!(counter_style.descriptors().occurrences().count(), 2);
    let CssRule::Page(page) = &report.syntax().rules()[1] else {
        panic!("expected recovered page")
    };
    assert_eq!(page.declarations().len(), 1);
    let CssRule::Media(media) = &report.syntax().rules()[2] else {
        panic!("expected recovered media parent")
    };
    assert!(matches!(media.rules(), [CssRule::Style(_)]));

    let expected = [
        (
            "@page :left { margin: 1cm; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@counter-style descriptor-child { symbols: d; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@font-feature-values Demo { }",
            CssErrorCode::UnsupportedAtRule,
        ),
        ("@mystery-descriptor-rule { }", CssErrorCode::UnknownAtRule),
        (
            "@counter-style page-child { symbols: p; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@counter-style media-child { symbols: m; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@page :right { margin: 2cm; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@counter-style style-child { symbols: s; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@page :first { margin: 3cm; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@counter-style scope-child { symbols: c; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@page :left { margin: 4cm; }",
            CssErrorCode::InvalidAtRulePlacement,
        ),
        (
            "@font-feature-values Tail { }",
            CssErrorCode::UnsupportedAtRule,
        ),
        ("@mystery-tail { }", CssErrorCode::UnknownAtRule),
    ];
    assert_eq!(
        report.diagnostics().len(),
        expected.len(),
        "{:#?}",
        report.diagnostics()
    );
    for (diagnostic, (authored, code)) in report.diagnostics().iter().zip(expected) {
        let start = source.find(authored).expect("authored invalid unit");
        assert_eq!(
            diagnostic.error().code(),
            code,
            "{authored}: {diagnostic:#?}"
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + authored.len()
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            start,
            "{authored}"
        );
        assert_eq!(diagnostic.error().position().line().value(), 1);
        let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[line_start..start].encode_utf16().count(),
            "{authored}"
        );
    }

    for depth in [255_usize, 256, 257] {
        let counter_source = format!(
            "@counter-style deep{{symbols:x;prefix:{}x{};suffix:\".\";}}.tail{{}}",
            "f(".repeat(depth),
            ")".repeat(depth),
        );
        let counter_report = parse_sheet(&counter_source);
        assert!(matches!(
            counter_report.syntax().rules(),
            [CssRule::CounterStyle(_), CssRule::Style(_)]
        ));
        let [counter_diagnostic] = counter_report.diagnostics() else {
            panic!("depth {depth} must drop exactly the invalid descriptor")
        };
        assert_eq!(
            counter_diagnostic.error().code() == CssErrorCode::NestingLimit,
            depth >= 256,
            "counter-style depth {depth}: {:?}",
            counter_report.diagnostics()
        );
        assert_eq!(
            counter_diagnostic.action(),
            if depth >= 256 {
                CssRecoveryAction::StopAtNestingLimit
            } else {
                CssRecoveryAction::DropDescriptor
            }
        );

        let page_source = format!(
            "@page{{margin-top:{}x{};margin-left:1cm;}}.tail{{}}",
            "f(".repeat(depth),
            ")".repeat(depth),
        );
        let page_report = parse_sheet(&page_source);
        assert!(matches!(
            page_report.syntax().rules(),
            [CssRule::Page(_), CssRule::Style(_)]
        ));
        let [page_diagnostic] = page_report.diagnostics() else {
            panic!("depth {depth} must drop exactly the invalid page declaration")
        };
        assert_eq!(
            page_diagnostic.error().code() == CssErrorCode::NestingLimit,
            depth >= 256,
            "page depth {depth}: {:?}",
            page_report.diagnostics()
        );
        assert_eq!(
            page_diagnostic.action(),
            if depth >= 256 {
                CssRecoveryAction::StopAtNestingLimit
            } else {
                CssRecoveryAction::DropDeclaration
            }
        );
    }

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_sheet(source)
            .expect_err("strict validation rejects every recovered C11 rule context");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn later_counter_style_rule_metadata_matches_retained_public_behavior() {
    let metadata = feature_metadata("later.rule.counter-style").expect("counter-style metadata");
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    assert_eq!(metadata.recognized_unsupported_code(), None);
    let report =
        parse_sheet("@counter-style thumbs { system: cyclic; symbols: 👍; suffix: \" \"; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::CounterStyle(_)]
    ));
}
