#![forbid(unsafe_code)]
//! Counter Styles 3 (2021-07-27) §3: decoded counter-style names, predefined
//! identity, and the six rule-definition exclusions. Section 3.8 defines
//! identifier symbols through Values 4 (2024-03-12) §4.2 `<custom-ident>`.
//! CSS Syntax 3 (2021-12-24) §4.3.9 and §4.3.11 establish that valid escapes
//! contribute decoded code points to one identifier token.

use surgeist_css::{
    CssComponentValueRef, CssCounterStyleName, CssCounterStyleRule, CssCounterStyleSpeakAs,
    CssCounterStyleSystem, CssCounterStyleValue, CssCounterSymbol, CssCounterSymbolIdent,
    CssKnownPropertyValueRef, CssListStyleTypeValue, CssParsedOrigin, CssRecoveryAction, CssRule,
    CssValueOrigin, CssValueTokenRef, parse_sheet, parse_style_attribute,
};

// Independent fixture transcribed from the selected Counter Styles 3 §6 and §7
// definitions, including aliases and the algorithm-defined Chinese/Ethiopic
// styles. This fixture is not obtained from a production keyword catalog.
const PREDEFINED_NAMES: [&str; 55] = [
    "decimal",
    "decimal-leading-zero",
    "arabic-indic",
    "armenian",
    "upper-armenian",
    "lower-armenian",
    "bengali",
    "cambodian",
    "khmer",
    "cjk-decimal",
    "devanagari",
    "georgian",
    "gujarati",
    "gurmukhi",
    "hebrew",
    "kannada",
    "lao",
    "malayalam",
    "mongolian",
    "myanmar",
    "oriya",
    "persian",
    "lower-roman",
    "upper-roman",
    "tamil",
    "telugu",
    "thai",
    "tibetan",
    "lower-alpha",
    "lower-latin",
    "upper-alpha",
    "upper-latin",
    "lower-greek",
    "hiragana",
    "hiragana-iroha",
    "katakana",
    "katakana-iroha",
    "disc",
    "circle",
    "square",
    "disclosure-open",
    "disclosure-closed",
    "cjk-earthly-branch",
    "cjk-heavenly-stem",
    "japanese-informal",
    "japanese-formal",
    "korean-hangul-formal",
    "korean-hanja-informal",
    "korean-hanja-formal",
    "simp-chinese-informal",
    "simp-chinese-formal",
    "trad-chinese-informal",
    "trad-chinese-formal",
    "cjk-ideographic",
    "ethiopic-numeric",
];

const FORBIDDEN_RULE_NAMES: [&str; 6] = [
    "decimal",
    "disc",
    "square",
    "circle",
    "disclosure-open",
    "disclosure-closed",
];

const ESCAPED_IDENTIFIERS: [(&str, &str); 5] = [
    (r"\31Name", "1Name"),
    (r"-\31Name", "-1Name"),
    (r"chapter\ name", "chapter name"),
    (r"chapter\;name", "chapter;name"),
    (r"chapter\\name", r"chapter\name"),
];

const GENERIC_RESERVED: [&str; 8] = [
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
    "default",
    "DeFaUlT",
    "InHeRiT",
];

fn parsed_counter(source: &str) -> CssCounterStyleRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("expected one counter style for {source:?}");
    };
    rule.clone()
}

fn origin_text(origin: &CssParsedOrigin) -> &str {
    &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()]
}

#[test]
fn checked_counter_names_admit_decoded_identifiers_requiring_escapes() {
    for (_, decoded) in ESCAPED_IDENTIFIERS {
        let name = CssCounterStyleName::try_new(decoded)
            .unwrap_or_else(|| panic!("valid decoded counter name {decoded:?}"));
        assert_eq!(name.as_str(), decoded);
    }
}

#[test]
fn escaped_counter_rule_names_retain_decoded_identity_and_authored_positions() {
    for (spelling, decoded) in ESCAPED_IDENTIFIERS {
        let source = format!("/* authored */\n@counter-style {spelling} {{ symbols: a; }}");
        let rule = parsed_counter(&source);
        assert_eq!(rule.name().as_str(), decoded, "{spelling}");
        assert_eq!(
            rule.position().byte_offset().value(),
            source.find("@counter-style").unwrap()
        );
        assert_eq!(
            rule.descriptors()
                .symbols()
                .unwrap()
                .position()
                .byte_offset()
                .value(),
            source.find("symbols:").unwrap()
        );
    }
}

#[test]
fn checked_counter_symbols_admit_decoded_identifiers_requiring_escapes() {
    for (_, decoded) in ESCAPED_IDENTIFIERS {
        let symbol = CssCounterSymbolIdent::try_new(decoded)
            .unwrap_or_else(|| panic!("valid decoded counter symbol {decoded:?}"));
        assert_eq!(symbol.as_str(), decoded);
    }
}

#[test]
fn escaped_counter_symbols_retain_decoded_identity_and_descriptor_positions() {
    for (spelling, decoded) in ESCAPED_IDENTIFIERS {
        let source = format!(
            "/* authored */\n@counter-style custom {{ system: cyclic; symbols: {spelling}; }}"
        );
        let rule = parsed_counter(&source);
        let symbols = rule.descriptors().symbols().unwrap();
        assert!(matches!(
            symbols.symbols(),
            [CssCounterSymbol::Ident(symbol)] if symbol.as_str() == decoded
        ));
        assert_eq!(
            symbols.position().byte_offset().value(),
            source.find("symbols:").unwrap()
        );
    }
}

#[test]
fn escaped_counter_name_references_retain_decoded_identity() {
    for (spelling, decoded) in ESCAPED_IDENTIFIERS {
        let source = format!(
            "@counter-style child {{ system: extends {spelling}; fallback: {spelling}; speak-as: {spelling}; }}"
        );
        let rule = parsed_counter(&source);
        assert!(matches!(
            rule.descriptors().system().unwrap().value(),
            CssCounterStyleSystem::Extends(name) if name.as_str() == decoded
        ));
        assert_eq!(
            rule.descriptors().fallback().unwrap().value().as_str(),
            decoded
        );
        assert!(matches!(
            rule.descriptors().speak_as().unwrap().value(),
            CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == decoded
        ));
    }
}

#[test]
fn bare_digit_and_multiple_identifier_rule_names_are_rejected() {
    for spelling in ["1Name", "chapter name"] {
        let source = format!("@counter-style {spelling} {{ symbols: a; }} .after {{}}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{source}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert_eq!(
            report.diagnostics()[0].span().start().byte_offset().value(),
            0
        );
        assert_eq!(
            report.diagnostics()[0].span().end().byte_offset().value(),
            source.find(" .after").unwrap()
        );
    }
}

#[test]
fn checked_counter_names_reject_generic_reserved_none_empty_and_nul() {
    for name in GENERIC_RESERVED
        .into_iter()
        .chain(["none", "NoNe", "", "a\0b"])
    {
        assert!(CssCounterStyleName::try_new(name).is_none(), "{name:?}");
    }
}

#[test]
fn reserved_counter_rule_names_are_rejected_even_when_escaped() {
    for spelling in GENERIC_RESERVED
        .into_iter()
        .chain(["none", "NoNe", r"\6e one", r"\64 efault"])
    {
        let source = format!("@counter-style {spelling} {{ symbols: a; }} .after {{}}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{source}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
    }
}

#[test]
fn checked_counter_symbols_allow_none_auto_and_span() {
    for symbol in ["none", "NoNe", "auto", "AuTo", "span", "SpAn"] {
        assert_eq!(
            CssCounterSymbolIdent::try_new(symbol).unwrap().as_str(),
            symbol
        );
    }
}

#[test]
fn parsed_counter_symbols_allow_none_auto_and_span_without_case_folding() {
    let rule = parsed_counter(
        r"@counter-style custom { system: cyclic; symbols: none AuTo SpAn \6e one; }",
    );
    let symbols = rule.descriptors().symbols().unwrap().symbols();
    let decoded: Vec<_> = symbols
        .iter()
        .map(|symbol| {
            let CssCounterSymbol::Ident(symbol) = symbol else {
                panic!("identifier symbol");
            };
            symbol.as_str()
        })
        .collect();
    assert_eq!(decoded, ["none", "AuTo", "SpAn", "none"]);
}

#[test]
fn checked_counter_symbols_reject_generic_reserved_empty_and_nul() {
    for symbol in GENERIC_RESERVED.into_iter().chain(["", "a\0b"]) {
        assert!(
            CssCounterSymbolIdent::try_new(symbol).is_none(),
            "{symbol:?}"
        );
    }
}

#[test]
fn reserved_identifier_symbols_drop_only_the_responsible_descriptor() {
    for spelling in GENERIC_RESERVED
        .into_iter()
        .chain([r"\69 nherit", r"\64 efault"])
    {
        // A valid symbols descriptor isolates rejection of this prefix from
        // counter-style effective-combination validation.
        let source =
            format!("@counter-style custom {{ system: cyclic; symbols: a; prefix: {spelling}; }}");
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
            panic!("retained rule for {source}");
        };
        assert!(rule.descriptors().prefix().is_none(), "{source}");
        assert!(matches!(
            rule.descriptors().symbols().unwrap().symbols(),
            [CssCounterSymbol::Ident(symbol)] if symbol.as_str() == "a"
        ));
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find("prefix:").unwrap()
        );
    }
}

#[test]
fn arbitrary_counter_names_and_symbols_preserve_case() {
    for custom in ["MyStyle", "myStyle", "MY-PRIVATE-STYLE", "auto", "span"] {
        assert_eq!(
            CssCounterStyleName::try_new(custom).unwrap().as_str(),
            custom
        );
        let source = format!("@counter-style {custom} {{ symbols: MiXeD; }}");
        let rule = parsed_counter(&source);
        assert_eq!(rule.name().as_str(), custom);
        assert!(matches!(
            rule.descriptors().symbols().unwrap().symbols(),
            [CssCounterSymbol::Ident(symbol)] if symbol.as_str() == "MiXeD"
        ));
    }
}

#[test]
fn arbitrary_counter_name_references_preserve_case() {
    let rule = parsed_counter(
        "@counter-style Child { system: extends MyStyle; fallback: MyFallback; speak-as: MyVoice; }",
    );
    assert_eq!(rule.name().as_str(), "Child");
    assert!(matches!(
        rule.descriptors().system().unwrap().value(),
        CssCounterStyleSystem::Extends(name) if name.as_str() == "MyStyle"
    ));
    assert_eq!(
        rule.descriptors().fallback().unwrap().value().as_str(),
        "MyFallback"
    );
    assert!(matches!(
        rule.descriptors().speak_as().unwrap().value(),
        CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == "MyVoice"
    ));
}

#[test]
fn legal_predefined_counter_rule_names_are_lowercased_on_parse() {
    for canonical in PREDEFINED_NAMES {
        if FORBIDDEN_RULE_NAMES.contains(&canonical) {
            continue;
        }
        let source = format!(
            "@counter-style {} {{ symbols: a; }}",
            canonical.to_ascii_uppercase()
        );
        let rule = parsed_counter(&source);
        assert_eq!(rule.name().as_str(), canonical, "{source}");
    }
}

#[test]
fn predefined_fallback_names_are_lowercased_on_parse() {
    for canonical in PREDEFINED_NAMES {
        let source = format!(
            "@counter-style custom {{ symbols: a; fallback: {}; }}",
            canonical.to_ascii_uppercase()
        );
        let rule = parsed_counter(&source);
        assert_eq!(
            rule.descriptors().fallback().unwrap().value().as_str(),
            canonical,
            "{source}"
        );
    }
}

#[test]
fn predefined_extended_names_are_lowercased_on_parse() {
    for canonical in PREDEFINED_NAMES {
        let source = format!(
            "@counter-style custom {{ system: extends {}; }}",
            canonical.to_ascii_uppercase()
        );
        let rule = parsed_counter(&source);
        assert!(
            matches!(
                rule.descriptors().system().unwrap().value(),
                CssCounterStyleSystem::Extends(name) if name.as_str() == canonical
            ),
            "{source}"
        );
    }
}

#[test]
fn predefined_spoken_names_are_lowercased_on_parse() {
    for canonical in PREDEFINED_NAMES {
        let source = format!(
            "@counter-style custom {{ symbols: a; speak-as: {}; }}",
            canonical.to_ascii_uppercase()
        );
        let rule = parsed_counter(&source);
        assert!(
            matches!(
                rule.descriptors().speak_as().unwrap().value(),
                CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == canonical
            ),
            "{source}"
        );
    }
}

#[test]
fn escaped_predefined_rule_names_are_normalized_after_decoding() {
    let rule = parsed_counter(r"@counter-style \55PPER-ROMAN { symbols: I; }");
    assert_eq!(rule.name().as_str(), "upper-roman");
}

#[test]
fn protected_predefined_names_are_rejected_as_rule_definitions() {
    for name in FORBIDDEN_RULE_NAMES {
        for spelling in [name.to_owned(), name.to_ascii_uppercase()] {
            let source = format!("@counter-style {spelling} {{ symbols: a; }} .after {{}}");
            let report = parse_sheet(&source);
            assert!(
                matches!(report.syntax().rules(), [CssRule::Style(_)]),
                "{source}"
            );
            assert_eq!(report.diagnostics().len(), 1, "{source}");
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropAtRule
            );
        }
    }
}

#[test]
fn protected_predefined_names_remain_valid_checked_references() {
    for name in FORBIDDEN_RULE_NAMES {
        assert!(CssCounterStyleName::try_new(name).is_some(), "{name}");
    }
}

#[test]
fn parsed_counter_style_reference_preserves_original_declaration_and_token_origins() {
    // Counter rules expose authored positions but no lexical name token. This
    // existing named-reference declaration boundary also exposes genuine name,
    // value and component origins, so their source identity can be checked.
    let source = r"list-style-type: \31Name;";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one list-style-type declaration");
    };
    let CssKnownPropertyValueRef::ListStyleType(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed list-style-type");
    };
    assert!(matches!(
        value.value(),
        CssListStyleTypeValue::CounterStyle(CssCounterStyleValue::Named(name))
            if name.as_str() == "1Name"
    ));

    let name_origin = declaration.parsed_name().unwrap();
    let value_origin = declaration.parsed_value().unwrap();
    assert_eq!(origin_text(name_origin), "list-style-type");
    assert_eq!(origin_text(value_origin).trim(), r"\31Name");
    assert_eq!(name_origin.source().as_str(), source);
    assert!(name_origin.source().same_snapshot(value_origin.source()));

    let identifiers: Vec<_> = declaration
        .value_components()
        .items()
        .iter()
        .filter(|component| {
            matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident(_))
            )
        })
        .collect();
    let [component] = identifiers.as_slice() else {
        panic!("one retained identifier token");
    };
    assert!(matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("1Name"))
    ));
    let CssValueOrigin::Parsed(token_origin) = component.origin() else {
        panic!("genuine parsed token origin");
    };
    assert_eq!(origin_text(token_origin), r"\31Name");
    assert!(token_origin.source().same_snapshot(value_origin.source()));
}
