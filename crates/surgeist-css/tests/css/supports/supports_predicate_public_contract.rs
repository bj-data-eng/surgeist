//! Functional qualification of the additive authored predicate payloads, beyond
//! the existing-callable classification regression checkpoint. Independent
//! oracles: Conditional 5 WD20251030 §2 and Fonts 4 WD20260907 §4.3 vocabularies.
//! Support truth, font loading and actual at-rule admission remain downstream.
use crate::css::support::authored_syntax::checked_supports_condition as checked;
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::*;

fn both(source: &str, inspect: impl Fn(&CssCondition<CssSupportsLeaf>)) {
    let condition = checked(source);
    inspect(&condition);
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    let clone = condition.clone();
    assert_eq!(condition, clone);
    let sheet = format!("@supports {source} {{}}");
    let report = CssParser::default().parse_sheet(&sheet);
    assert!(report.is_clean(), "{sheet}: {:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("retained group")
    };
    inspect(rule.condition());
    assert_eq!(
        rule.condition()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        format!(" {source} ")
    );
    let expected = format!("@supports {source} {{ }}");
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    let replay = CssParser::default().parse_sheet(&expected);
    assert!(replay.is_clean(), "{:?}", replay.diagnostics());
    let [CssRule::Supports(rule)] = replay.syntax().rules() else {
        panic!("replayed group")
    };
    inspect(rule.condition());
    assert_eq!(
        replay
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
}

fn assert_token_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original token origin")
    };
    let start = source.find(token).unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + token.len()
    );
}

#[test]
fn every_font_technology_keyword_has_its_distinct_authored_payload() {
    for (name, expected) in [
        ("features-opentype", CssFontTechHint::FeaturesOpenType),
        ("features-aat", CssFontTechHint::FeaturesAAT),
        ("features-graphite", CssFontTechHint::FeaturesGraphite),
        ("color-COLRv0", CssFontTechHint::ColorCOLRv0),
        ("color-COLRv1", CssFontTechHint::ColorCOLRv1),
        ("color-SVG", CssFontTechHint::ColorSVG),
        ("color-sbix", CssFontTechHint::ColorSbix),
        ("color-CBDT", CssFontTechHint::ColorCBDT),
        ("variations", CssFontTechHint::Variations),
        ("palettes", CssFontTechHint::Palettes),
        ("incremental", CssFontTechHint::Incremental),
    ] {
        both(&format!("font-tech({name})"), |condition| {
            assert!(
                matches!(condition.kind(), CssConditionKind::Leaf(CssSupportsLeaf::FontTech(value)) if *value == expected)
            );
        });
    }
    both(r"FoNt-\74 ech( /**/ VaRiAtIoNs /**/ )", |condition| {
        assert!(matches!(
            condition.kind(),
            CssConditionKind::Leaf(CssSupportsLeaf::FontTech(CssFontTechHint::Variations))
        ));
    });
}

#[test]
fn every_font_format_keyword_is_preserved_without_merging_synonymous_formats() {
    for (name, expected) in [
        ("collection", CssFontFormatHint::Collection),
        ("embedded-opentype", CssFontFormatHint::EmbeddedOpenType),
        ("opentype", CssFontFormatHint::OpenType),
        ("svg", CssFontFormatHint::Svg),
        ("truetype", CssFontFormatHint::TrueType),
        ("woff", CssFontFormatHint::Woff),
        ("woff2", CssFontFormatHint::Woff2),
    ] {
        both(&format!("font-format({name})"), |condition| {
            assert!(
                matches!(condition.kind(), CssConditionKind::Leaf(CssSupportsLeaf::FontFormat(CssFontFormat::Keyword(value))) if *value == expected)
            );
        });
    }
    both(r"FONT-\66 ormat( /**/ W\4f FF2 /**/ )", |condition| {
        assert!(matches!(
            condition.kind(),
            CssConditionKind::Leaf(CssSupportsLeaf::FontFormat(CssFontFormat::Keyword(
                CssFontFormatHint::Woff2
            )))
        ));
    });
}

#[test]
fn full_format_string_grammar_retains_decoded_strings_instead_of_keyword_truth() {
    // Conditional5 §2.1.1 excludes strings from a positive support result;
    // Fonts4 still admits their authored grammar, including empty/unknown text.
    for (source, decoded) in [
        (r#"font-format("")"#, ""),
        (r#"font-format("future-format")"#, "future-format"),
        (r#"font-format("woff2")"#, "woff2"),
        (r#"font-format("woff2-variations")"#, "woff2-variations"),
        (r#"font-format("\77 off2")"#, "woff2"),
        ("font-format('  WOFF2  ')", "  WOFF2  "),
    ] {
        both(source, |condition| {
            let CssConditionKind::Leaf(CssSupportsLeaf::FontFormat(CssFontFormat::String(value))) =
                condition.kind()
            else {
                panic!("distinct authored String for {source}")
            };
            assert_eq!(value.as_str(), decoded);
        });
    }
}

#[test]
fn arbitrary_at_keywords_keep_original_tokens_decoded_names_and_function_delimiters() {
    for (argument, decoded) in [
        ("@unknown-future", "unknown-future"),
        ("@charset", "charset"),
        ("@MeDiA", "MeDiA"),
        (r"@\4d eDiA", "MeDiA"),
        ("@--Future", "--Future"),
    ] {
        let function = format!(r"aT-\72 ule( /**/ {argument} /**/ )");
        both(&function, |condition| {
            let CssConditionKind::Leaf(CssSupportsLeaf::AtRule(value)) = condition.kind() else {
                panic!("genuine AtKeyword predicate")
            };
            assert_eq!(value.name(), decoded);
            assert!(
                matches!(value.component().view(), CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(name)) if name == decoded)
            );
            assert_eq!(value.origin(), value.component().origin());
            let serialized = condition.serialize_tokens(Default::default()).unwrap();
            let lexical = serialized.as_css();
            let CssValueOrigin::Parsed(origin) = value.origin() else {
                panic!("argument origin")
            };
            let original = origin.source().as_str();
            assert_token_origin(value.origin(), original, argument);
            assert_eq!(value.position(), Some(origin.span().start()));
            assert_eq!(
                serialized.origin_at(lexical.find(argument).unwrap()),
                Some(&CssSerializedOrigin::Token(value.origin().clone()))
            );
            let component = condition
                .components()
                .iter()
                .find(|value| matches!(value.view(), CssComponentValueRef::Function(_)))
                .unwrap();
            let CssComponentValueRef::Function(full) = component.view() else {
                unreachable!()
            };
            let actual_argument = full
                .values()
                .items()
                .iter()
                .find(|value| {
                    matches!(
                        value.view(),
                        CssComponentValueRef::Token(CssValueTokenRef::AtKeyword(_))
                    )
                })
                .unwrap();
            assert_eq!(value.component(), actual_argument);
            assert_token_origin(component.origin(), original, r"aT-\72 ule(");
            let CssValueOrigin::Parsed(close) = full.closing_origin() else {
                panic!("original closing delimiter")
            };
            assert_eq!(close.source().as_str(), original);
            assert_eq!(
                close.span().start().byte_offset().value(),
                original.find(')').unwrap()
            );
            assert_eq!(
                serialized.origin_at(lexical.find(')').unwrap()),
                Some(&CssSerializedOrigin::Token(full.closing_origin().clone()))
            );
        });
    }
}

#[test]
fn programmatic_function_wrapper_keeps_the_independently_parsed_at_keyword_origin() {
    let source = "/*😀*/\n@--Mixed";
    let argument = CssComponentValues::parse(&CssParser::default(), source)
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let origin = argument.origin().clone();
    let function = CssComponentValue::try_function(
        "at-rule",
        CssComponentValues::try_new(vec![argument.clone()], Default::default()).unwrap(),
    )
    .unwrap();
    let condition = CssCondition::<CssSupportsLeaf>::try_from_components(
        CssComponentValues::try_new(vec![function], Default::default()).unwrap(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(condition.position(), None);
    let CssConditionKind::Leaf(CssSupportsLeaf::AtRule(value)) = condition.kind() else {
        panic!("mixed-origin AtKeyword")
    };
    assert_eq!(value.component(), &argument);
    assert_eq!(value.name(), "--Mixed");
    assert_eq!(value.origin(), &origin);
    assert_eq!(value.position().unwrap().line().value(), 1);
    assert_eq!(value.position().unwrap().column().value(), 0);
    let output = condition.serialize_tokens(Default::default()).unwrap();
    assert_eq!(output.as_css(), "at-rule(@--Mixed)");
    assert_eq!(
        output.origin_at(0),
        Some(&CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    );
    assert_eq!(
        output.origin_at(8),
        Some(&CssSerializedOrigin::Token(origin))
    );
}

const COMPOSED: &str = r#"font-tech(variations) and (font-format("woff2") or at-rule(@charset)) and (--\46 eature) and selector(.probe) and (display:grid)"#;

fn inspect_composed(condition: &CssCondition<CssSupportsLeaf>) {
    let CssConditionKind::And(children) = condition.kind() else {
        panic!("authored conjunction")
    };
    let [tech, grouped, named, selector, declaration] = children.conditions() else {
        panic!("five ordered authored operands")
    };
    assert!(matches!(
        tech.kind(),
        CssConditionKind::Leaf(CssSupportsLeaf::FontTech(CssFontTechHint::Variations))
    ));
    let CssConditionKind::Parenthesized(group) = grouped.kind() else {
        panic!("explicitly grouped alternatives")
    };
    let CssConditionKind::Or(alternatives) = group.kind() else {
        panic!("explicitly grouped alternatives")
    };
    let [format, rule] = alternatives.conditions() else {
        panic!("two alternatives")
    };
    assert!(
        matches!(format.kind(), CssConditionKind::Leaf(CssSupportsLeaf::FontFormat(CssFontFormat::String(value))) if value.as_str() == "woff2")
    );
    assert!(
        matches!(rule.kind(), CssConditionKind::Leaf(CssSupportsLeaf::AtRule(value)) if value.name() == "charset")
    );
    assert!(
        matches!(named.kind(), CssConditionKind::Leaf(CssSupportsLeaf::Named(value)) if value.as_str() == "--Feature")
    );
    assert!(
        matches!(selector.kind(), CssConditionKind::Leaf(CssSupportsLeaf::Selector(CssSelector::Class(value))) if value == "probe")
    );
    assert!(
        matches!(declaration.kind(), CssConditionKind::Leaf(CssSupportsLeaf::Declaration(value)) if value.property() == "display")
    );
    assert_eq!(
        condition
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css()
            .trim(),
        COMPOSED
    );
}

#[test]
fn predicates_compose_with_booleans_names_and_selectors_across_group_consumers() {
    inspect_composed(&checked(COMPOSED));
    both(
        "not (font-tech(palettes) or at-rule(@unknown))",
        |condition| {
            let CssConditionKind::Not(child) = condition.kind() else {
                panic!("unary not")
            };
            let CssConditionKind::Parenthesized(group) = child.kind() else {
                panic!("explicit group")
            };
            let CssConditionKind::Or(children) = group.kind() else {
                panic!("grouped or")
            };
            assert!(matches!(
                children.conditions()[0].kind(),
                CssConditionKind::Leaf(CssSupportsLeaf::FontTech(CssFontTechHint::Palettes))
            ));
            assert!(
                matches!(children.conditions()[1].kind(), CssConditionKind::Leaf(CssSupportsLeaf::AtRule(value)) if value.name() == "unknown")
            );
        },
    );
    for (source, expected) in [
        (
            format!("@supports {COMPOSED}{{.child{{color:red}}}}"),
            format!("@supports {COMPOSED} {{ .child {{ color: red; }} }}"),
        ),
        (
            format!(".parent{{@supports {COMPOSED}{{color:red}}}}"),
            format!(".parent {{ @supports {COMPOSED} {{ color: red; }} }}"),
        ),
        (
            format!("@scope(.root){{@supports {COMPOSED}{{.child{{color:red}}}}}}"),
            format!("@scope (.root) {{ @supports {COMPOSED} {{ .child {{ color: red; }} }} }}"),
        ),
    ] {
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let before = report.clone();
        let normalized = report.syntax().normalize(Default::default()).unwrap();
        let conditions: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Rule(context) => match context.kind() {
                    CssRuleContextKindRef::Supports(condition) => Some(condition),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let [condition] = conditions.as_slice() else {
            panic!("one normalized supports carrier")
        };
        inspect_composed(condition);
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        let [value] = values.as_slice() else {
            panic!("retained body declaration")
        };
        assert_eq!(
            value.source().known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css()
                .trim(),
            "red"
        );
        let reused = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
        assert_eq!(reused.rules(), report.syntax().rules());
        assert_eq!(
            reused.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(
            report
                .syntax()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
        assert_eq!(report, before);
        let replay = CssParser::default().parse_sheet(&expected);
        assert!(replay.is_clean(), "{:?}", replay.diagnostics());
        assert_eq!(
            replay
                .syntax()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn import_supports_uses_the_same_typed_predicates_and_checked_lexical_owner() {
    let source = format!("@import 'x.css' supports({COMPOSED});");
    let report = CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(parsed)] = report.syntax().rules() else {
        panic!("retained import")
    };
    let constructed = CssImportRule::try_from_components(
        CssComponentValues::parse(&CssParser::default(), &source).unwrap(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap();
    for rule in [parsed, &constructed] {
        assert!(rule.media().is_none());
        inspect_composed(rule.supports().unwrap().condition());
        assert_eq!(
            rule.serialize_tokens(Default::default()).unwrap().as_css(),
            source
        );
    }
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        source
    );
    let replay = CssParser::default().parse_sheet(&source);
    let [CssRule::Import(rule)] = replay.syntax().rules() else {
        panic!("replayed import")
    };
    inspect_composed(rule.supports().unwrap().condition());
}

#[test]
fn balanced_invalid_predicate_arguments_remain_opaque_but_invalid_outer_algebra_rejects() {
    for source in [
        "font-tech()",
        "font-tech(unknown)",
        "font-tech(variations palettes)",
        "font-tech(variations, palettes)",
        "font-tech(\"variations\")",
        "font-format()",
        "font-format(future-format)",
        "font-format(woff2-variations)",
        "font-format(woff2, opentype)",
        "font-format(1)",
        "at-rule()",
        "at-rule(media)",
        "at-rule(\"@media\")",
        "at-rule(@media;)",
        "at-rule(@media @supports)",
        "future(font-tech(variations))",
    ] {
        both(source, |condition| {
            assert!(
                matches!(
                    condition.kind(),
                    CssConditionKind::Leaf(CssSupportsLeaf::GeneralEnclosed(_))
                ),
                "{source}"
            );
        });
    }
    for source in [
        "font-tech(variations) and",
        "not not at-rule(@media)",
        "font-tech(variations) and at-rule(@media) or font-format(woff2)",
    ] {
        let error = CssCondition::<CssSupportsLeaf>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            &CssNamespaceContext::default(),
            Default::default(),
        )
        .unwrap_err();
        assert!(error.is_invalid_condition_grammar());
        let report =
            CssParser::default().parse_sheet(format!("@supports {source}{{}} .after{{color:red}}"));
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
        let [diagnostic] = report.diagnostics() else {
            panic!("one failed outer condition")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePrelude
        );
    }
}

#[test]
fn lexical_failures_in_function_predicates_are_terminal_and_recover_to_the_later_sibling() {
    for (predicate, bad, kind) in [
        ("font-tech(url(a b))", "url(", ComponentFailure::BadUrl),
        ("at-rule(url(a b))", "url(", ComponentFailure::BadUrl),
        (
            "font-format(\"broken\n)",
            "\"broken",
            ComponentFailure::BadString,
        ),
    ] {
        let component_error =
            CssComponentValues::parse(&CssParser::default(), predicate).unwrap_err();
        assert_eq!(component_failure(&component_error), kind);
        for (prefix, ending) in [("@supports ", "{}"), ("@import 'x' supports(", ");")] {
            let source = format!("/*😀*/\r\n{prefix}{predicate}{ending} .after{{color:red}}");
            let report = CssParser::default().parse_sheet(&source);
            assert!(
                matches!(report.syntax().rules(), [CssRule::Style(_)]),
                "{source}: {report:?}"
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one terminal lexical failure: {report:?}")
            };
            let Some(detail) = diagnostic.error().component_error() else {
                panic!("lexical error survives predicate probes")
            };
            assert_eq!(component_failure(detail), kind);
            let CssValueOrigin::Parsed(origin) = detail.origin() else {
                panic!("original bad token")
            };
            let start = source.find(bad).unwrap();
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(origin.span().start().byte_offset().value(), start);
            assert_eq!(origin.span().start().line().value(), 1);
            let line_start = source.find('\n').unwrap() + 1;
            assert_eq!(
                origin.span().start().column().value() as usize,
                source[line_start..start].encode_utf16().count()
            );
            assert_eq!(diagnostic.error().position(), origin.span().start());
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        }
    }
}

#[test]
fn complete_predicate_graph_limits_are_cumulative_and_retries_preserve_origins() {
    let source = "font-tech(variations) and at-rule(@unknown)";
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
    // Two functions and their arguments, two spaces and one operator: seven
    // components at depth one. Closing delimiters affect bytes, not node count.
    let exact = CssComponentValueLimits::try_new(1, 7, source.len()).unwrap();
    let condition = CssCondition::<CssSupportsLeaf>::try_from_components(
        components.clone(),
        &CssNamespaceContext::default(),
        exact,
    )
    .unwrap();
    assert_eq!(
        condition
            .serialize_tokens(crate::css::support::byte_limits(source.len()))
            .unwrap()
            .as_css(),
        source
    );
    let before = condition.clone();
    for (limits, kind, offset) in [
        (
            CssComponentValueLimits::try_new(0, 7, source.len()).unwrap(),
            ComponentFailure::NestingLimit,
            0,
        ),
        (
            CssComponentValueLimits::try_new(1, 6, source.len()).unwrap(),
            ComponentFailure::ComponentLimit,
            source.find("@unknown").unwrap(),
        ),
        (
            CssComponentValueLimits::try_new(1, 7, source.len() - 1).unwrap(),
            ComponentFailure::ByteLimit,
            source.len() - 1,
        ),
    ] {
        let error = CssCondition::<CssSupportsLeaf>::try_from_components(
            components.clone(),
            &CssNamespaceContext::default(),
            limits,
        )
        .unwrap_err();
        let Some(error) = error.component_error() else {
            panic!("terminal component resource failure")
        };
        assert_eq!(component_failure(error), kind);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("original exhausted component")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), offset);
    }
    assert!(
        condition
            .serialize_tokens(crate::css::support::byte_limits(source.len() - 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(condition, before);
    assert_eq!(
        condition
            .serialize_tokens(crate::css::support::byte_limits(source.len()))
            .unwrap()
            .as_css(),
        source
    );
    assert_eq!(
        CssCondition::<CssSupportsLeaf>::try_from_components(
            components,
            &CssNamespaceContext::default(),
            exact
        )
        .unwrap(),
        condition
    );
}

#[test]
fn recovered_predicate_delimiters_keep_parsed_import_payloads_but_fail_strict_reuse() {
    for predicate in [
        "font-tech(variations",
        "font-format(woff2",
        "at-rule(@media",
    ] {
        let components = CssComponentValues::parse(&CssParser::default(), predicate).unwrap();
        let error = CssCondition::<CssSupportsLeaf>::try_from_components(
            components,
            &CssNamespaceContext::default(),
            Default::default(),
        )
        .unwrap_err();
        assert!(error.is_recovered_input());
        let CssValueOrigin::ImplicitClosure { at, .. } = error.origin() else {
            panic!("original implied closure")
        };
        assert_eq!(at.source().as_str(), predicate);
        assert_eq!(at.span().start().byte_offset().value(), predicate.len());
        let source = format!("@import 'x' supports({predicate}");
        let report = CssParser::default().parse_sheet(&source);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let [CssRule::Import(rule)] = report.syntax().rules() else {
            panic!("retained recovered import")
        };
        let condition = rule.supports().unwrap().condition();
        match predicate {
            "font-tech(variations" => assert!(matches!(
                condition.kind(),
                CssConditionKind::Leaf(CssSupportsLeaf::FontTech(CssFontTechHint::Variations))
            )),
            "font-format(woff2" => assert!(matches!(
                condition.kind(),
                CssConditionKind::Leaf(CssSupportsLeaf::FontFormat(CssFontFormat::Keyword(
                    CssFontFormatHint::Woff2
                )))
            )),
            _ => assert!(
                matches!(condition.kind(), CssConditionKind::Leaf(CssSupportsLeaf::AtRule(value)) if value.name() == "media")
            ),
        }
        let expected = format!("@import 'x' supports({predicate}));");
        assert_eq!(
            rule.serialize_tokens(Default::default()).unwrap().as_css(),
            expected
        );
        let serialized = condition.serialize_tokens(Default::default()).unwrap();
        assert!(matches!(
            serialized.origin_at(serialized.as_css().len() - 1),
            Some(CssSerializedOrigin::Token(
                CssValueOrigin::ImplicitClosure { .. }
            ))
        ));
        let recovered =
            CssComponentValues::try_new(condition.components().to_vec(), Default::default())
                .unwrap();
        assert!(
            CssCondition::<CssSupportsLeaf>::try_from_components(
                recovered,
                &CssNamespaceContext::default(),
                Default::default()
            )
            .as_ref()
            .is_err_and(|error| error.is_recovered_input())
        );
        assert!(CssParser::default().parse_sheet(&expected).is_clean());
    }
}
