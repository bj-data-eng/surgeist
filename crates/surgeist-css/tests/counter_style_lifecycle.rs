#![forbid(unsafe_code)]
//! Source-derived Counter Styles 3 (2021-07-27) authored lifecycles, §§3–3.9.
//! Ordinary initials remain implicit; extends omissions stay distinguishable
//! from explicit values. Definition selection, target lookup, marker generation,
//! grapheme padding and speech execution belong downstream.
use surgeist_css::*;

fn counter(body: &str) -> CssCounterStyleRule {
    let source = format!("@counter-style Custom {{ {body} }}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one authored counter rule");
    };
    rule.clone()
}

fn integer(value: &str) -> CssIntegerLiteral {
    CssIntegerLiteral::try_from_component(CssComponentValue::try_number(value).unwrap()).unwrap()
}

fn string(value: &str) -> CssCounterSymbol {
    CssCounterSymbol::String(CssContentString::try_new(value).unwrap())
}

fn url(value: &str) -> CssCounterSymbol {
    CssCounterSymbol::Image(CssImage::try_new(CssImageValue::Url(CssUrl::new(value))).unwrap())
}

#[test]
fn implicit_initials_are_omitted_authored_values_without_forged_occurrences() {
    let rule = counter("symbols:a;");
    let descriptors = rule.descriptors();
    assert!(descriptors.system().is_none());
    assert!(descriptors.negative().is_none());
    assert!(descriptors.prefix().is_none());
    assert!(descriptors.suffix().is_none());
    assert!(descriptors.range().is_none());
    assert!(descriptors.pad().is_none());
    assert!(descriptors.fallback().is_none());
    assert!(descriptors.additive_symbols().is_none());
    assert!(descriptors.speak_as().is_none());
    for kind in [
        CssCounterStyleDescriptorKind::System,
        CssCounterStyleDescriptorKind::Negative,
        CssCounterStyleDescriptorKind::Prefix,
        CssCounterStyleDescriptorKind::Suffix,
        CssCounterStyleDescriptorKind::Range,
        CssCounterStyleDescriptorKind::Pad,
        CssCounterStyleDescriptorKind::Fallback,
        CssCounterStyleDescriptorKind::AdditiveSymbols,
        CssCounterStyleDescriptorKind::SpeakAs,
    ] {
        assert_eq!(rule.descriptor_specified_css(kind).unwrap(), "");
    }
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Custom { symbols: a; }"
    );
}

#[test]
fn explicit_initial_values_remain_distinct_from_omission() {
    let rule = counter(
        r#"system:symbolic; negative:"-"; prefix:""; suffix:". "; range:auto; pad:0 ""; fallback:decimal; symbols:a; speak-as:auto;"#,
    );
    assert!(matches!(
        rule.descriptors().system().unwrap().value(),
        CssCounterStyleSystem::Symbolic
    ));
    assert!(
        matches!(rule.descriptors().negative().unwrap().prefix(), CssCounterSymbol::String(value) if value.as_str() == "-")
    );
    assert!(rule.descriptors().negative().unwrap().suffix().is_none());
    assert!(
        matches!(rule.descriptors().prefix().unwrap().value(), CssCounterSymbol::String(value) if value.as_str().is_empty())
    );
    assert!(
        matches!(rule.descriptors().suffix().unwrap().value(), CssCounterSymbol::String(value) if value.as_str() == ". ")
    );
    assert!(matches!(
        rule.descriptors().range().unwrap().value(),
        CssCounterStyleRange::Auto
    ));
    assert_eq!(
        rule.descriptors()
            .pad()
            .unwrap()
            .minimum_length()
            .numeric()
            .representation(),
        "0"
    );
    assert_eq!(rule.descriptors().fallback().unwrap().as_str(), "decimal");
    assert!(matches!(
        rule.descriptors().speak_as().unwrap().value(),
        CssCounterStyleSpeakAs::Auto
    ));
    assert_eq!(
        rule.to_specified_css().unwrap(),
        concat!(
            "@counter-style Custom { system: symbolic; negative: \"-\"; prefix: \"\"; ",
            "suffix: \". \"; range: auto; pad: 0 \"\"; fallback: decimal; symbols: a; speak-as: auto; }"
        )
    );
}

#[test]
fn every_system_retains_its_valid_minimum_symbol_boundary() {
    for (body, expected) in [
        ("system:cyclic; symbols:a;", "cyclic"),
        ("system:numeric; symbols:a b;", "numeric"),
        ("system:alphabetic; symbols:a b;", "alphabetic"),
        ("system:symbolic; symbols:a;", "symbolic"),
        ("system:additive; additive-symbols:0 N;", "additive"),
        ("system:fixed; symbols:a;", "fixed"),
        ("system:extends Unknown;", "extends Unknown"),
    ] {
        let rule = counter(body);
        assert_eq!(
            rule.descriptor_specified_css(CssCounterStyleDescriptorKind::System)
                .unwrap(),
            expected
        );
        match (expected, rule.descriptors().system().unwrap().value()) {
            ("cyclic", CssCounterStyleSystem::Cyclic)
            | ("numeric", CssCounterStyleSystem::Numeric)
            | ("alphabetic", CssCounterStyleSystem::Alphabetic)
            | ("symbolic", CssCounterStyleSystem::Symbolic)
            | ("additive", CssCounterStyleSystem::Additive) => {}
            ("fixed", CssCounterStyleSystem::Fixed(value)) => {
                assert!(value.first_symbol_value().is_none())
            }
            ("extends Unknown", CssCounterStyleSystem::Extends(name)) => {
                assert_eq!(name.as_str(), "Unknown")
            }
            _ => panic!("wrong typed system for {body}"),
        }
    }
}

#[test]
fn every_descriptor_selects_last_valid_duplicate_after_whole_value_recovery() {
    use CssCounterStyleDescriptorKind as Kind;
    for (kind, initial, invalid, last, expected) in [
        (
            Kind::System,
            "system:fixed -2",
            "system:fixed calc(1)",
            "system:CYCLIC",
            "cyclic",
        ),
        (
            Kind::Negative,
            "negative:'-'",
            "negative:'-' ')' extra",
            "negative:'(' ')'",
            "\"(\" \")\"",
        ),
        (
            Kind::Prefix,
            "prefix:'first'",
            "prefix:'partial' extra",
            "prefix:Last",
            "Last",
        ),
        (
            Kind::Suffix,
            "suffix:'first'",
            "suffix:'partial' extra",
            "suffix:'last'",
            "\"last\"",
        ),
        (
            Kind::Range,
            "range:1 3",
            "range:9 2, 1 3",
            "range:infinite -1, 1 infinite",
            "infinite -1, 1 infinite",
        ),
        (Kind::Pad, "pad:2 '0'", "pad:-1 '0'", "pad:'_' 3", "3 \"_\""),
        (
            Kind::Fallback,
            "fallback:First",
            "fallback:none",
            "fallback:Last",
            "Last",
        ),
        (
            Kind::Symbols,
            "symbols:first",
            "symbols:",
            "symbols:'last' Last",
            "\"last\" Last",
        ),
        (
            Kind::AdditiveSymbols,
            "additive-symbols:10 X, 1 I",
            "additive-symbols:10 X, 10 I",
            "additive-symbols:C 100, N 0",
            "100 C, 0 N",
        ),
        (
            Kind::SpeakAs,
            "speak-as:words",
            "speak-as:inherit",
            "speak-as:SPELL-OUT",
            "spell-out",
        ),
    ] {
        let source = format!(
            ".before{{}} @counter-style Custom {{ symbols:a; {initial}; {invalid}; {last}; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [
            CssRule::Style(_),
            CssRule::CounterStyle(rule),
            CssRule::Style(_),
        ] = report.syntax().rules()
        else {
            panic!("descriptor recovery must preserve both neighboring rules: {source}");
        };
        let retained = rule.clone();
        assert_eq!(
            rule.descriptor_specified_css(kind).unwrap(),
            expected,
            "{source}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one wholly invalid descriptor: {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let start = source.find(&format!("{invalid};")).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + invalid.len() + 1
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let _ = rule.to_specified_css().unwrap();
        assert_eq!(rule, &retained);
    }
}

#[test]
fn unknown_descriptor_and_unsupported_value_recover_independently() {
    let source = "@counter-style Custom { symbols:a; Future-Descriptor: f([unknown]); system:future-system; suffix:'.'; } .after{}";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("counter and following rule");
    };
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Custom { suffix: \".\"; symbols: a; }"
    );
    let [unknown, unsupported] = report.diagnostics() else {
        panic!("independent descriptor recovery");
    };
    assert_eq!(unknown.error().code(), CssErrorCode::UnknownDescriptor);
    assert_eq!(
        unsupported.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(unknown.action(), CssRecoveryAction::DropDescriptor);
    assert_eq!(unsupported.action(), CssRecoveryAction::DropDescriptor);
    assert!(rule.descriptors().system().is_none());
}

#[test]
fn descriptor_annotations_invalidate_only_the_annotated_descriptor() {
    for annotation in ["!important", "!other", "!important extra"] {
        let source = format!(
            "@counter-style Custom {{ symbols:a; prefix:'kept'; prefix:'bad' {annotation}; suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("annotation recovery");
        };
        assert_eq!(
            rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Prefix)
                .unwrap(),
            "\"kept\""
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one annotated descriptor");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn descriptor_body_at_rules_follow_forward_recovery_without_losing_valid_values() {
    let source = "@counter-style Custom { prefix:'before'; @future-body { nested:([x]); } @counter-style forbidden { symbols:x; } symbols:a; suffix:'after'; } .after{}";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("whole unknown nested units recover");
    };
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Custom { prefix: \"before\"; suffix: \"after\"; symbols: a; }"
    );
    let [unknown, forbidden] = report.diagnostics() else {
        panic!("two invalid nested at-rules");
    };
    assert_eq!(unknown.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(
        forbidden.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    assert_eq!(unknown.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(forbidden.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn extends_omission_and_explicit_range_auto_remain_distinct() {
    let omitted = counter("system:extends Missing;");
    let explicit = counter("system:extends Missing; range:auto;");
    assert!(omitted.descriptors().range().is_none());
    assert!(matches!(
        explicit.descriptors().range().unwrap().value(),
        CssCounterStyleRange::Auto
    ));
    assert_eq!(
        omitted
            .descriptor_specified_css(CssCounterStyleDescriptorKind::Range)
            .unwrap(),
        ""
    );
    assert_eq!(
        explicit
            .descriptor_specified_css(CssCounterStyleDescriptorKind::Range)
            .unwrap(),
        "auto"
    );
    assert!(explicit.descriptors().fallback().is_none());
    assert!(explicit.descriptors().speak_as().is_none());
    assert_eq!(
        explicit.to_specified_css().unwrap(),
        "@counter-style Custom { system: extends Missing; range: auto; }"
    );
}

#[test]
fn forbidden_extends_symbol_descriptors_drop_the_rule_but_invalid_descriptors_do_not() {
    for descriptor in ["symbols:a;", "additive-symbols:1 I;"] {
        let source = format!(
            ".before{{}} @counter-style Custom {{ system:extends Missing; {descriptor} }} .after{{}}"
        );
        let report = parse_sheet(&source);
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Style(_), CssRule::Style(_)]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("one genuinely invalid extends combination");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorCombination
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    }
    let source = "@counter-style Custom { system:extends Missing; symbols:; additive-symbols:1 I, 1 X; suffix:'.'; }";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("invalid symbol descriptors are ignored before effective combination validation");
    };
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Custom { system: extends Missing; suffix: \".\"; }"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDescriptor)
    );
}

#[test]
fn unknown_named_targets_and_speech_choices_remain_symbolic() {
    let rule = counter("system:extends MyStyle; fallback:MyFallback; speak-as:MyVoice;");
    assert!(
        matches!(rule.descriptors().system().unwrap().value(), CssCounterStyleSystem::Extends(name) if name.as_str() == "MyStyle")
    );
    assert_eq!(
        rule.descriptors().fallback().unwrap().as_str(),
        "MyFallback"
    );
    assert!(
        matches!(rule.descriptors().speak_as().unwrap().value(), CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == "MyVoice")
    );
    for (body, expected) in [
        ("AUTO", "auto"),
        ("BULLETS", "bullets"),
        ("NUMBERS", "numbers"),
        ("WORDS", "words"),
        ("SPELL-OUT", "spell-out"),
    ] {
        let rule = counter(&format!("symbols:a; speak-as:{body};"));
        assert_eq!(
            rule.descriptor_specified_css(CssCounterStyleDescriptorKind::SpeakAs)
                .unwrap(),
            expected
        );
    }
}

#[test]
fn checked_negative_construction_preserves_required_and_optional_symbol_order() {
    let first = url("minus.svg");
    let last = string(")");
    let pair = CssCounterStyleNegative::new(first.clone(), Some(last.clone()));
    assert_eq!(pair.prefix(), &first);
    assert_eq!(pair.suffix(), Some(&last));
    let single = CssCounterStyleNegative::new(first.clone(), None);
    assert_eq!(single.prefix(), &first);
    assert!(single.suffix().is_none());
    let empty = string("");
    let explicit_empty = CssCounterStyleNegative::new(first, Some(empty.clone()));
    assert_eq!(explicit_empty.suffix(), Some(&empty));
}

#[test]
fn checked_pad_and_additive_construction_keep_exact_integer_and_image_semantics() {
    let image = url("symbol.svg");
    let pad = CssCounterStylePad::try_new(integer("-000"), image.clone()).unwrap();
    assert_eq!(pad.minimum_length().numeric().representation(), "-000");
    assert_eq!(pad.symbol(), &image);
    assert!(CssCounterStylePad::try_new(integer("-1"), image.clone()).is_none());
    let huge = "999999999999999999999999999999999999999999999999999";
    let upper = CssCounterAdditiveTuple::try_new(integer(huge), image.clone()).unwrap();
    let zero = CssCounterAdditiveTuple::try_new(integer("-000"), string("zero")).unwrap();
    let list = CssCounterAdditiveSymbols::try_new(vec![upper.clone(), zero.clone()]).unwrap();
    assert_eq!(list.tuples()[0].weight().numeric().representation(), huge);
    assert_eq!(list.tuples()[0].symbol(), &image);
    assert_eq!(list.tuples()[1], zero);
    assert!(CssCounterAdditiveSymbols::try_new(vec![zero.clone(), upper]).is_none());
    let alias = CssCounterAdditiveTuple::try_new(integer("+0000"), image.clone()).unwrap();
    assert!(CssCounterAdditiveSymbols::try_new(vec![zero, alias]).is_none());
    assert!(CssCounterAdditiveTuple::try_new(integer("-1"), image).is_none());
    assert!(CssCounterAdditiveSymbols::try_new(Vec::new()).is_none());
}

#[test]
fn huge_integer_fields_and_signed_zero_keep_authored_tokens_without_range_clamping() {
    let huge = "999999999999999999999999999999999999999999999999999";
    let rule = counter(&format!(
        "system:fixed -{huge}; symbols:a; range:-{huge} {huge}, infinite -1, -000 infinite; pad:-000 '_'; additive-symbols:{huge} X, -000 N;"
    ));
    let CssCounterStyleSystem::Fixed(fixed) = rule.descriptors().system().unwrap().value() else {
        panic!("fixed");
    };
    assert_eq!(
        fixed
            .first_symbol_value()
            .unwrap()
            .numeric()
            .representation(),
        format!("-{huge}")
    );
    let CssCounterStyleRange::Ranges(ranges) = rule.descriptors().range().unwrap().value() else {
        panic!("ranges");
    };
    assert_eq!(
        ranges.ranges()[1].lower(),
        &CssCounterStyleRangeBound::Infinite
    );
    assert_eq!(
        ranges.ranges()[2].upper(),
        &CssCounterStyleRangeBound::Infinite
    );
    assert_eq!(
        rule.descriptors()
            .pad()
            .unwrap()
            .minimum_length()
            .numeric()
            .representation(),
        "-000"
    );
    assert_eq!(
        rule.descriptors().additive_symbols().unwrap().tuples()[1]
            .weight()
            .numeric()
            .representation(),
        "-000"
    );
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Range)
            .unwrap(),
        format!("-{huge} {huge}, infinite -1, 0 infinite")
    );
    assert_eq!(
        rule.descriptor_specified_css(CssCounterStyleDescriptorKind::Pad)
            .unwrap(),
        "0 \"_\""
    );
}

#[test]
fn inside_and_case_sensitive_names_remain_valid_unresolved_rule_and_reference_names() {
    let source = "@counter-style inside { symbols:MiXeD; fallback:Inside; speak-as:Voice; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("inside is a valid rule name");
    };
    assert_eq!(rule.name().as_str(), "inside");
    assert_eq!(rule.descriptors().fallback().unwrap().as_str(), "Inside");
    assert!(
        matches!(rule.descriptors().symbols().unwrap().symbols(), [CssCounterSymbol::Ident(value)] if value.as_str() == "MiXeD")
    );
}

#[test]
fn eof_body_recovery_keeps_valid_occurrences_and_matches_strict_diagnostics() {
    let source = "@counter-style Custom { symbols:a; suffix:'.'";
    let report = parse_sheet(source);
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("EOF retains completed descriptors");
    };
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@counter-style Custom { suffix: \".\"; symbols: a; }"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert!(!report.is_clean());
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert_eq!(
        normalize_report(&report).unwrap().diagnostics(),
        report.diagnostics()
    );
}
