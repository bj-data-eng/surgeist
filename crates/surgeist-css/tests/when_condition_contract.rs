#![forbid(unsafe_code)]
//! Functional new-API qualification for the adopted #647 authored profile and
//! #642's immutable lexical model. No missing-symbol or stub RED is claimed.
//! https://github.com/bj-data-eng/surgeist/issues/647
use surgeist_css::*;

fn feature(condition: &CssWhenCondition) -> &CssWhenMediaFeature {
    let CssWhenConditionKind::MediaFeature(value) = condition.kind() else {
        panic!("one supported media feature: {:?}", condition.kind());
    };
    value
}

fn parsed(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(value) = origin else {
        panic!("genuine source origin: {origin:?}");
    };
    value
}

#[test]
fn exact_media_leaf_preserves_boolean_plain_and_directional_range_shapes() {
    for source in [
        "media(width)",
        "media(width:1px)",
        "media(width >= 1px)",
        "media(1px < width)",
        "media(1px < width <= 10px)",
    ] {
        let condition = parse_when_condition(source).unwrap();
        let leaf = feature(&condition);
        let CssWhenMediaFeatureKind::Feature(query) = leaf.kind() else {
            panic!("known width feature");
        };
        assert_eq!(query.name(), "width");
        match source {
            "media(width)" => assert!(matches!(
                query,
                CssMediaFeatureQuery::Boolean(CssMediaFeatureKind::Width)
            )),
            _ => {
                let CssMediaFeatureQuery::Width(range) = query else {
                    panic!("typed width range");
                };
                match source {
                    "media(width:1px)" => {
                        assert!(matches!(range.view(), CssMediaRangeRef::Plain { .. }))
                    }
                    "media(width >= 1px)" => assert!(matches!(
                        range.view(),
                        CssMediaRangeRef::FeatureFirst {
                            comparison: CssQueryComparison::GreaterThanOrEqual,
                            ..
                        }
                    )),
                    "media(1px < width)" => assert!(matches!(
                        range.view(),
                        CssMediaRangeRef::ValueFirst {
                            comparison: CssQueryComparison::LessThan,
                            ..
                        }
                    )),
                    _ => assert!(matches!(
                        range.view(),
                        CssMediaRangeRef::Ascending {
                            left_inclusive: false,
                            right_inclusive: true,
                            ..
                        }
                    )),
                }
            }
        }
        assert_eq!(leaf.position().unwrap().byte_offset().value(), 0);
        assert_eq!(parsed(leaf.origin()).source().as_str(), source);
        assert_eq!(condition.serialize().unwrap().as_css(), source);
        let components = parse_component_values(source).unwrap();
        let [component] = components.items() else {
            panic!("one function component");
        };
        assert_eq!(
            CssWhenMediaFeature::try_from_component(component.clone()).unwrap(),
            *leaf
        );
    }
}

#[test]
fn unknown_boolean_features_keep_the_actual_media_enclosure_without_custom_reference() {
    for name in ["screen", "--name", "future-feature"] {
        let source = format!("media({name})");
        let condition = parse_when_condition(&source).unwrap();
        let leaf = feature(&condition);
        let CssWhenMediaFeatureKind::UnknownFeature(unknown) = leaf.kind() else {
            panic!("unknown boolean feature");
        };
        assert_eq!(unknown.name(), name);
        assert_eq!(unknown.reason(), CssUnknownMediaFeatureReason::UnknownName);
        assert!(matches!(unknown.view(), CssUnknownMediaFeatureRef::Boolean));
        assert_eq!(unknown.authored(), Some(source.as_str()));
        assert_eq!(unknown.component(), leaf.component());
        // The existing unknown payload's own public serializer must accept its
        // genuine media() enclosure, rather than assuming synthetic parentheses.
        assert_eq!(unknown.serialize().unwrap().as_css(), source);
        assert_eq!(condition.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn unknown_feature_values_preserve_plain_and_range_syntax_without_opaque_conversion() {
    for source in [
        "media(future-feature:1px)",
        "media(future-feature > 1px)",
        "media(1px < future-feature <= 10px)",
    ] {
        let condition = parse_when_condition(source).unwrap();
        let leaf = feature(&condition);
        let CssWhenMediaFeatureKind::UnknownFeature(unknown) = leaf.kind() else {
            panic!("unknown name with exact feature grammar");
        };
        assert_eq!(unknown.reason(), CssUnknownMediaFeatureReason::UnknownName);
        let CssUnknownMediaFeatureRef::Range(range) = unknown.view() else {
            panic!("authored value-bearing feature");
        };
        match source {
            "media(future-feature:1px)" => {
                assert!(matches!(range.view(), CssMediaRangeRef::Plain { .. }))
            }
            "media(future-feature > 1px)" => assert!(matches!(
                range.view(),
                CssMediaRangeRef::FeatureFirst {
                    comparison: CssQueryComparison::GreaterThan,
                    ..
                }
            )),
            _ => assert!(matches!(
                range.view(),
                CssMediaRangeRef::Ascending {
                    left_inclusive: false,
                    right_inclusive: true,
                    ..
                }
            )),
        }
        assert_eq!(unknown.authored(), Some(source));
        assert_eq!(condition.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn broader_known_function_inputs_are_opaque_and_reject_on_leaf_only_admission() {
    for source in [
        "media()",
        "media((width))",
        "media(screen and (width:1px))",
        "media(width > = 1px)",
        "media(1px < width > 10px)",
        "supports()",
        "supports((display:grid))",
    ] {
        let condition = parse_when_condition(source).unwrap();
        let CssWhenConditionKind::GeneralEnclosed(opaque) = condition.kind() else {
            panic!("opaque fallback for {source}");
        };
        assert_eq!(opaque.authored(), Some(source));
        assert_eq!(condition.serialize().unwrap().as_css(), source);
        if source.starts_with("media") {
            let values = parse_component_values(source).unwrap();
            assert!(matches!(
                CssWhenMediaFeature::try_from_component(values.items()[0].clone()),
                Err(CssWhenConstructionError::InvalidMediaFeatureGrammar { .. })
            ));
        }
    }
}

#[test]
fn supports_leaf_retains_one_declaration_and_the_selected_document_mode() {
    for (source, property, known) in [
        ("supports(display:grid)", "display", true),
        ("supports(future-prop:value)", "future-prop", false),
    ] {
        let condition = parse_when_condition(source).unwrap();
        let CssWhenConditionKind::SupportsDeclaration(declaration) = condition.kind() else {
            panic!("one supports declaration");
        };
        assert_eq!(declaration.property(), property);
        assert_eq!(declaration.known().is_some(), known);
        assert_eq!(
            declaration.position().unwrap().byte_offset().value(),
            "supports(".len()
        );
        assert_eq!(parsed(declaration.origin()).source().as_str(), source);
        assert_eq!(condition.serialize().unwrap().as_css(), source);
    }
    let source = "supports(color:123)";
    let standard = parse_when_condition(source).unwrap();
    let quirks = CssParserContext::new(CssParserMode::Quirks)
        .parse_when_condition(source)
        .unwrap();
    let CssWhenConditionKind::SupportsDeclaration(standard) = standard.kind() else {
        panic!("standard declaration");
    };
    let CssWhenConditionKind::SupportsDeclaration(quirks) = quirks.kind() else {
        panic!("quirks declaration");
    };
    assert!(standard.known().is_none());
    assert!(quirks.known().is_some());
}

#[test]
fn homogeneous_operators_and_redundant_grouping_remain_ordered_and_symbolic() {
    let source = "((media(width) or media(height))) and not(supports(display:grid))";
    let condition = parse_when_condition(source).unwrap();
    let CssWhenConditionKind::And(and) = condition.kind() else {
        panic!("outer And");
    };
    let [first, second] = and.conditions() else {
        panic!("two ordered terms");
    };
    let CssWhenConditionKind::Parenthesized(first) = first.kind() else {
        panic!("first explicit pair");
    };
    let CssWhenConditionKind::Parenthesized(first) = first.kind() else {
        panic!("second explicit pair");
    };
    let CssWhenConditionKind::Or(or) = first.kind() else {
        panic!("grouped Or");
    };
    let [width, height] = or.conditions() else {
        panic!("ordered feature terms");
    };
    assert_eq!(width.serialize().unwrap().as_css(), "media(width)");
    assert_eq!(height.serialize().unwrap().as_css(), "media(height)");
    // No separating space before '(' makes not(...) a function token.
    assert!(matches!(
        second.kind(),
        CssWhenConditionKind::GeneralEnclosed(_)
    ));
    assert_eq!(condition.serialize().unwrap().as_css(), source);
    let unary = parse_when_condition("not (supports(display:grid))").unwrap();
    let CssWhenConditionKind::Not(term) = unary.kind() else {
        panic!("one unary Not");
    };
    let CssWhenConditionKind::Parenthesized(term) = term.kind() else {
        panic!("explicit grouped operand");
    };
    assert!(matches!(
        term.kind(),
        CssWhenConditionKind::SupportsDeclaration(_)
    ));
}

#[test]
fn empty_and_failed_group_probes_are_opaque_while_outer_algebra_rejects() {
    for source in [
        "()",
        "future()",
        "font-tech(color-COLRv1)",
        "future(;!)",
        "(media(width) and supports(display:grid) or media(height))",
    ] {
        let condition = parse_when_condition(source).unwrap();
        assert!(
            matches!(condition.kind(), CssWhenConditionKind::GeneralEnclosed(_)),
            "{source}"
        );
        assert_eq!(condition.serialize().unwrap().as_css(), source);
    }
    for source in [
        "",
        "not",
        "media(width) and",
        "not media(width) and supports(display:grid)",
        "media(width) and supports(display:grid) or media(height)",
        "media(width),supports(display:grid)",
        "media(width) trailing",
        "media (width)",
    ] {
        assert!(
            matches!(
                parse_when_condition(source),
                Err(CssWhenConstructionError::InvalidConditionGrammar { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn lexical_failures_are_terminal_component_errors_with_original_origins() {
    for (source, kind, responsible) in [
        (
            "future(\"bad\n)",
            CssComponentValueErrorKind::BadString,
            "\"",
        ),
        (
            "future(url(a b))",
            CssComponentValueErrorKind::BadUrl,
            "url(",
        ),
        (
            "future(])",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
            "]",
        ),
    ] {
        let error = parse_when_condition(source).unwrap_err();
        let CssWhenConstructionError::Component(component) = &error else {
            panic!("terminal component cause: {error:?}");
        };
        assert_eq!(component.kind(), kind);
        let origin = parsed(component.origin());
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(responsible).unwrap()
        );
        assert!(std::error::Error::source(&error).is_some());
    }
}

#[test]
fn checked_recovered_input_rejects_while_parsed_rule_retains_owned_eof_closures() {
    let values = parse_component_values("media(width").unwrap();
    assert!(matches!(
        CssWhenCondition::try_from_components(values.clone()),
        Err(CssWhenConstructionError::RecoveredInput {
            origin: CssValueOrigin::ImplicitClosure { .. }
        })
    ));
    assert!(matches!(
        CssWhenMediaFeature::try_from_component(values.items()[0].clone()),
        Err(CssWhenConstructionError::RecoveredInput { .. })
    ));
    let report = parse_sheet("@when media(width){.A{color:red");
    let [CssRule::When(rule)] = report.syntax().rules() else {
        panic!("retained when with EOF-recovered body");
    };
    assert!(matches!(
        rule.condition().kind(),
        CssWhenConditionKind::MediaFeature(_)
    ));
    // Lexical condition output retains the actual prelude's root trivia;
    // compact specified rule emission owns trimming that edge whitespace.
    assert_eq!(
        rule.condition().serialize().unwrap().as_css(),
        " media(width)"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert_eq!(report.diagnostics().len(), 2);
    assert!(report.into_validation_result().is_err());
}

#[test]
fn lexical_spelling_comments_coordinates_and_serialized_origins_survive_clone() {
    let source = "/*🦀*/ MEDIA(width) AnD (supports(display:grid) or future()) ";
    let condition = parse_when_condition(source).unwrap();
    let before = condition.clone();
    assert!(matches!(condition.kind(), CssWhenConditionKind::And(_)));
    let position = condition.position().unwrap();
    assert_eq!(
        position.byte_offset().value(),
        source.find("MEDIA").unwrap()
    );
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value(),
        source[..source.find("MEDIA").unwrap()]
            .encode_utf16()
            .count() as u32
    );
    let output = condition.serialize().unwrap();
    assert_eq!(output.as_css(), source);
    let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))) =
        output.origin_at(source.find("MEDIA").unwrap())
    else {
        panic!("serialized original keyword origin");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("MEDIA").unwrap()
    );
    assert_eq!(condition, before);
    assert_eq!(
        CssWhenCondition::try_from_components(
            CssComponentValues::try_new(condition.components().to_vec()).unwrap()
        )
        .unwrap(),
        condition
    );
}

#[test]
fn programmatic_enclosure_keeps_parsed_feature_children_without_invented_position() {
    let children = parse_component_values("width").unwrap();
    let child = children.items()[0].clone();
    let component = CssComponentValue::try_function("media", children).unwrap();
    let condition = CssWhenCondition::try_from_components(
        CssComponentValues::try_new(vec![component]).unwrap(),
    )
    .unwrap();
    assert_eq!(condition.origin(), &CssValueOrigin::Programmatic);
    assert!(condition.position().is_none());
    let leaf = feature(&condition);
    let CssComponentValueRef::Function(function) = leaf.component().view() else {
        panic!("programmatic media function");
    };
    assert_eq!(function.values().items()[0], child);
    assert_eq!(
        parsed(function.values().items()[0].origin())
            .source()
            .as_str(),
        "width"
    );
    assert_eq!(condition.serialize().unwrap().as_css(), "media(width)");
}

#[test]
fn independent_component_depth_count_and_byte_limits_fail_atomically() {
    let source = "media(width)";
    // One function and its one feature identifier are two actual components.
    let exact = CssComponentValueLimits::try_new(1, 2, source.len()).unwrap();
    let condition = parse_when_condition_with_limits(source, exact).unwrap();
    let before = condition.clone();
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, 2, source.len()).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 1, source.len()).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 2, source.len() - 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        assert!(
            matches!(parse_when_condition_with_limits(source, limits), Err(CssWhenConstructionError::Component(ref error)) if error.kind() == kind)
        );
        assert!(
            matches!(CssWhenCondition::try_from_components_with_limits(parse_component_values(source).unwrap(), limits), Err(CssWhenConstructionError::Component(ref error)) if error.kind() == kind)
        );
    }
    assert_eq!(
        condition
            .serialize_with_limit(source.len())
            .unwrap()
            .as_css(),
        source
    );
    assert_eq!(
        condition
            .serialize_with_limit(source.len() - 1)
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    assert_eq!(condition, before);
    assert_eq!(condition.serialize().unwrap().as_css(), source);
}

#[test]
fn supported_and_opaque_condition_depth_256_retain_identity_and_257_is_terminal() {
    for opaque in [false, true] {
        let source = if opaque {
            format!("future({}x{})", "f(".repeat(255), ")".repeat(255))
        } else {
            format!("{}media(width){}", "(".repeat(255), ")".repeat(255))
        };
        let condition = parse_when_condition(&source).unwrap();
        if opaque {
            assert!(matches!(
                condition.kind(),
                CssWhenConditionKind::GeneralEnclosed(_)
            ));
        } else {
            let mut current = &condition;
            for _ in 0..255 {
                let CssWhenConditionKind::Parenthesized(child) = current.kind() else {
                    panic!("every authored grouping pair retained");
                };
                current = child;
            }
            assert!(matches!(
                current.kind(),
                CssWhenConditionKind::MediaFeature(_)
            ));
        }
        assert_eq!(condition.clone(), condition);
        assert_eq!(condition.serialize().unwrap().as_css(), source);
        let too_deep = if opaque {
            format!("future({}x{})", "f(".repeat(256), ")".repeat(256))
        } else {
            format!("{}media(width){}", "(".repeat(256), ")".repeat(256))
        };
        assert!(
            matches!(parse_when_condition(&too_deep), Err(CssWhenConstructionError::Component(ref error)) if error.kind() == CssComponentValueErrorKind::NestingLimit)
        );
    }
}
