#![forbid(unsafe_code)]
//! Whole-rule checked import contracts: Cascade optional-clause interpretation,
//! exact component provenance, and strict admission independent of loading.
use surgeist_css::{
    CssCalculationExpressionRef, CssComponentValue, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues, CssImportConstructionError,
    CssImportLayer, CssImportRule, CssImportSerializationError, CssImportTarget,
    CssKnownPropertyValueRef, CssMediaConditionKind, CssMediaConstructionError,
    CssMediaFeatureQuery, CssMediaQuery, CssMediaRangeRef, CssMediaType, CssNamespaceConstraint,
    CssNamespaceContext, CssOpacityValue, CssRule, CssSelector, CssSerializedOrigin,
    CssSupportsConditionKind, CssValueOrigin, parse_component_values, parse_sheet,
};

fn values(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items).unwrap()
}
fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).unwrap()
}
fn construct(source: &str) -> CssImportRule {
    CssImportRule::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}
fn tail(source: &str) -> CssImportRule {
    construct(&format!("@import 'x' {source};"))
}
fn original(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("original parsed provenance")
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}
fn opaque(import: &CssImportRule, expected: &str) {
    let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
        panic!("one media condition")
    };
    let CssMediaConditionKind::GeneralEnclosed(value) = condition.kind() else {
        panic!("opaque media")
    };
    assert_eq!(value.serialize().unwrap().as_css(), expected);
}

#[test]
fn whole_rule_admission_requires_one_import_and_an_explicit_terminator() {
    for source in [
        "@import 'x';",
        " /**/ @IMPORT 'x'; /*tail*/ ",
        r"@\69mport 'x';",
    ] {
        let import = construct(source);
        assert!(
            matches!(import.target(), CssImportTarget::String(target) if target.as_str() == "x")
        );
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        assert!(import.media().is_none());
        assert_eq!(import.serialize().unwrap().as_css(), "@import 'x';");
        let CssValueOrigin::Parsed(origin) = import.origin() else {
            panic!("at-keyword origin")
        };
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find('@').unwrap()
        );
        assert_eq!(import.position(), Some(origin.span().start()));
    }
    for source in [
        "",
        "'x';",
        "@media 'x';",
        "@import;",
        "@import 1;",
        "@import 'x'",
        "@import 'x' {}",
        "@import 'x';;",
        "@import 'x'; extra",
        "@import 'x'; @import 'y';",
    ] {
        assert!(
            matches!(
                CssImportRule::try_from_components(
                    parse_component_values(source).unwrap(),
                    &CssNamespaceContext::default()
                ),
                Err(CssImportConstructionError::InvalidRuleGrammar { .. })
            ),
            "{source:?}"
        );
    }
}

#[test]
fn empty_and_whitespace_targets_remain_authored_values_without_loading() {
    for (spelling, url, decoded) in [
        ("\"\"", false, ""),
        ("''", false, ""),
        ("url()", true, ""),
        ("url(   )", true, ""),
        ("url(\"\")", true, ""),
        ("\" \"", false, " "),
        ("url(\" \")", true, " "),
        (r"url(\20)", true, " "),
    ] {
        let source = format!("@import {spelling} layer(theme) supports(display:grid) print;");
        let import = construct(&source);
        match import.target() {
            CssImportTarget::Url(target) if url => assert_eq!(target.as_str(), decoded),
            CssImportTarget::String(target) if !url => assert_eq!(target.as_str(), decoded),
            other => panic!("wrong target: {other:?}"),
        }
        assert!(
            matches!(import.layer(), Some(CssImportLayer::Named(layer)) if layer.components() == ["theme"])
        );
        assert!(matches!(
            import.supports().unwrap().condition().kind(),
            CssSupportsConditionKind::Declaration(_)
        ));
        assert!(
            matches!(import.media().unwrap().queries(), [CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print)
        );
        assert_eq!(import.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn specific_optional_clauses_win_only_when_the_complete_derivation_matches() {
    let anonymous = tail("layer");
    assert!(matches!(anonymous.layer(), Some(CssImportLayer::Anonymous)));
    assert!(anonymous.media().is_none());
    let import = tail("layer(theme) supports(display:grid) print");
    assert!(
        matches!(import.layer(), Some(CssImportLayer::Named(layer)) if layer.components() == ["theme"])
    );
    let CssSupportsConditionKind::Declaration(declaration) =
        import.supports().unwrap().condition().kind()
    else {
        panic!("specific supports")
    };
    assert_eq!(declaration.property(), "display");
    assert_eq!(declaration.authored(), None);
    assert!(declaration.known().is_some());
    assert!(
        matches!(import.media().unwrap().queries(), [CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print)
    );
    assert_eq!(
        import
            .supports()
            .unwrap()
            .condition()
            .serialize()
            .unwrap()
            .as_css(),
        "(display:grid)"
    );
    for source in [
        "layer()",
        "supports()",
        "layer(initial)",
        "supports(not)",
        "supports(2px)",
    ] {
        let import = tail(source);
        assert!(import.layer().is_none(), "{source}");
        assert!(import.supports().is_none(), "{source}");
        opaque(&import, source);
    }
    for source in ["layer(theme)", "supports(display:grid)"] {
        let import = tail(&format!("{source} and (color)"));
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
            panic!("media condition")
        };
        let CssMediaConditionKind::And(children) = condition.kind() else {
            panic!("complete conjunction")
        };
        assert!(matches!(
            children.conditions()[0].kind(),
            CssMediaConditionKind::GeneralEnclosed(_)
        ));
        assert!(matches!(
            children.conditions()[1].kind(),
            CssMediaConditionKind::Feature(_)
        ));
    }
}

#[test]
fn later_clause_shaped_functions_remain_media_in_the_selected_derivation() {
    let import = tail("supports(display:grid) layer(theme)");
    assert!(import.layer().is_none());
    assert!(import.supports().is_some());
    opaque(&import, "layer(theme)");
    let import = tail("layer(theme) layer(other)");
    assert!(
        matches!(import.layer(), Some(CssImportLayer::Named(layer)) if layer.components() == ["theme"])
    );
    assert!(import.supports().is_none());
    opaque(&import, "layer(other)");
    let import = tail("supports(display:grid) supports(color:red)");
    assert!(import.supports().is_some());
    opaque(&import, "supports(color:red)");
    let import = tail("screen and supports(x)");
    assert!(import.layer().is_none());
    assert!(import.supports().is_none());
    let [CssMediaQuery::Typed(query)] = import.media().unwrap().queries() else {
        panic!("screen query")
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(matches!(
        query.condition().unwrap().kind(),
        CssMediaConditionKind::GeneralEnclosed(_)
    ));
}

#[test]
fn malformed_media_rejects_the_whole_import_instead_of_retaining_never_members() {
    for source in [
        "layer(theme) screen and, print",
        "layer layer",
        "screen and",
        "print, screen and",
    ] {
        let error = CssImportRule::try_from_components(
            parse_component_values(&format!("@import 'x' {source};")).unwrap(),
            &CssNamespaceContext::default(),
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                CssImportConstructionError::Media(
                    CssMediaConstructionError::InvalidQueryGrammar { .. }
                )
            ),
            "{source}: {error:?}"
        );
    }
}

#[test]
fn namespace_context_changes_selector_admission_without_resolving_the_namespace() {
    let sheet = parse_sheet("@namespace svg 'urn:svg'; @namespace 'urn:default';");
    assert!(sheet.is_clean());
    let context = CssNamespaceContext::from_sheet(sheet.syntax());
    let unbound = tail("supports(selector(svg|a))");
    assert!(matches!(
        unbound.supports().unwrap().condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
    for (source, named) in [
        ("@import 'x' supports(selector(svg|a));", true),
        ("@import 'x' supports(selector(a));", false),
    ] {
        let import =
            CssImportRule::try_from_components(parse_component_values(source).unwrap(), &context)
                .unwrap();
        let CssSupportsConditionKind::Selector(CssSelector::Compound(selector)) =
            import.supports().unwrap().condition().kind()
        else {
            panic!("typed selector")
        };
        let namespace = selector.type_selector().unwrap().namespace();
        if named {
            assert!(
                matches!(namespace, CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg")
            );
        } else {
            assert_eq!(namespace, &CssNamespaceConstraint::Default);
        }
        assert_eq!(import.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn programmatic_import_and_mixed_numeric_children_keep_honest_independent_origins() {
    let target = parse_component_values("/*😀*/'x'")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let supports_leaf = parse_component_values("/*😀*/1e999")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let media_leaf = parse_component_values("/*λ*/\n+001.5PX")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let target_origin = target.origin().clone();
    let supports_origin = supports_leaf.origin().clone();
    let media_origin = media_leaf.origin().clone();
    let calc = CssComponentValue::try_function("calc", values(vec![supports_leaf])).unwrap();
    let supports = CssComponentValue::try_function(
        "supports",
        values(vec![token("opacity"), token(":"), calc]),
    )
    .unwrap();
    let media = CssComponentValue::try_block(
        surgeist_css::CssBlockKind::Parenthesis,
        values(vec![token("width"), token(":"), media_leaf]),
    )
    .unwrap();
    let import = CssImportRule::try_from_components(
        values(vec![
            token("@import"),
            token(" "),
            target,
            token(" "),
            supports,
            token(" "),
            media,
            token(";"),
        ]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(import.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(import.position(), None);
    let CssSupportsConditionKind::Declaration(declaration) =
        import.supports().unwrap().condition().kind()
    else {
        panic!("supports declaration")
    };
    assert_eq!(declaration.authored(), None);
    let CssKnownPropertyValueRef::Opacity(opacity) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("opacity")
    };
    let CssOpacityValue::Calculation(calculation) = opacity.value() else {
        panic!("calc")
    };
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let CssCalculationExpressionRef::NestedCalc(calc) = calculation.expression() else {
        panic!("calc")
    };
    let CssCalculationExpressionRef::Value(leaf) = calc.operand() else {
        panic!("numeric leaf")
    };
    assert_eq!(leaf.literal().representation(), "1e999");
    original(leaf.literal().origin(), &supports_origin);
    let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
        panic!("media")
    };
    let CssMediaConditionKind::Feature(CssMediaFeatureQuery::Width(range)) = condition.kind()
    else {
        panic!("width")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain width")
    };
    original(value.calculation().origin(), &media_origin);
    let output = import.serialize().unwrap();
    assert_eq!(
        output.as_css(),
        "@import 'x' supports(opacity:calc(1e999)) (width: +001.5PX);"
    );
    for (spelling, expected) in [
        ("'x'", &target_origin),
        ("1e999", &supports_origin),
        ("+001.5PX", &media_origin),
    ] {
        let Some(CssSerializedOrigin::Token(actual)) =
            output.origin_at(output.as_css().find(spelling).unwrap())
        else {
            panic!("mapped original token")
        };
        original(actual, expected);
    }
    let supports = import.supports().unwrap().condition().clone();
    let media = import.media().unwrap().clone();
    drop(import);
    assert_eq!(
        supports.serialize().unwrap().as_css(),
        "(opacity:calc(1e999))"
    );
    assert_eq!(media.serialize().unwrap().as_css(), "(width: +001.5PX)");
}

#[test]
fn parsed_at_keyword_and_terminator_keep_their_snapshots_among_programmatic_children() {
    let parsed = parse_component_values("/*😀*/@IMPORT;").unwrap();
    let at = parsed.items()[1].clone();
    let end = parsed.items()[2].clone();
    let at_origin = at.origin().clone();
    let end_origin = end.origin().clone();
    let import = CssImportRule::try_from_components(
        values(vec![
            at,
            token(" "),
            CssComponentValue::try_string("").unwrap(),
            end,
        ]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    original(import.origin(), &at_origin);
    let CssValueOrigin::Parsed(at) = &at_origin else {
        panic!("at")
    };
    assert_eq!(import.position(), Some(at.span().start()));
    let output = import.serialize().unwrap();
    assert_eq!(output.as_css(), "@import \"\";");
    let Some(CssSerializedOrigin::Token(origin)) = output.origin_at(output.as_css().len() - 1)
    else {
        panic!("terminator")
    };
    original(origin, &end_origin);
}

#[test]
fn recovered_components_reject_before_invalid_envelope_and_preserve_eof_provenance() {
    let recovered = parse_component_values("Future(").unwrap();
    let CssComponentValueRef::Function(function) = recovered.items()[0].view() else {
        panic!("function")
    };
    let expected = function.closing_origin().clone();
    let error =
        CssImportRule::try_from_components(recovered.clone(), &CssNamespaceContext::default())
            .unwrap_err();
    assert!(matches!(
        error,
        CssImportConstructionError::RecoveredInput { .. }
    ));
    assert_eq!(error.origin(), &expected);
    let mut components = vec![token("@import"), token(" "), token("'x'"), token(" ")];
    components.extend_from_slice(recovered.items());
    components.push(token(";"));
    assert!(matches!(
        CssImportRule::try_from_components(values(components), &CssNamespaceContext::default()),
        Err(CssImportConstructionError::RecoveredInput { .. })
    ));
    let zero = CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap();
    let error = CssImportRule::try_from_components_with_limits(
        recovered,
        &CssNamespaceContext::default(),
        zero,
    )
    .unwrap_err();
    assert!(
        matches!(error, CssImportConstructionError::Component(ref error) if error.kind() == CssComponentValueErrorKind::ComponentLimit)
    );
}

#[test]
fn all_input_trivia_counts_toward_limits_before_canonical_trivia_is_omitted() {
    let source = "@import 'x'; /**/";
    let components = parse_component_values(source).unwrap();
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap();
    let error = CssImportRule::try_from_components_with_limits(
        components.clone(),
        &CssNamespaceContext::default(),
        limits,
    )
    .unwrap_err();
    let CssImportConstructionError::Component(error) = error else {
        panic!("whole input byte budget")
    };
    assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
    original(error.origin(), components.items().last().unwrap().origin());
    let limits =
        CssComponentValueLimits::try_new(0, components.items().len(), source.len()).unwrap();
    assert_eq!(
        CssImportRule::try_from_components_with_limits(
            components.clone(),
            &CssNamespaceContext::default(),
            limits
        )
        .unwrap()
        .serialize()
        .unwrap()
        .as_css(),
        "@import 'x';"
    );
    let limits =
        CssComponentValueLimits::try_new(0, components.items().len() - 1, source.len()).unwrap();
    assert!(
        matches!(CssImportRule::try_from_components_with_limits(components, &CssNamespaceContext::default(), limits), Err(CssImportConstructionError::Component(error)) if error.kind() == CssComponentValueErrorKind::ComponentLimit)
    );
    let components = parse_component_values("@import 'x' future();").unwrap();
    let limits = CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap();
    assert!(
        matches!(CssImportRule::try_from_components_with_limits(components, &CssNamespaceContext::default(), limits), Err(CssImportConstructionError::Component(error)) if error.kind() == CssComponentValueErrorKind::NestingLimit)
    );
}

#[test]
fn byte_limits_include_canonical_media_expansion_and_the_output_terminator() {
    let source = "@import 'x' (ASPECT-RATIO:2);";
    let expected = "@import 'x' (aspect-ratio: 2 / 1);";
    let components = parse_component_values(source).unwrap();
    let tight = CssComponentValueLimits::try_new(256, usize::MAX, source.len()).unwrap();
    let error = CssImportRule::try_from_components_with_limits(
        components.clone(),
        &CssNamespaceContext::default(),
        tight,
    )
    .unwrap_err();
    assert!(
        matches!(error, CssImportConstructionError::Serialization(CssImportSerializationError::Media(surgeist_css::CssMediaSerializationError::Component(ref error))) if error.kind() == CssComponentValueErrorKind::ByteLimit),
        "{error:?}"
    );
    let exact = CssComponentValueLimits::try_new(256, usize::MAX, expected.len()).unwrap();
    let import = CssImportRule::try_from_components_with_limits(
        components,
        &CssNamespaceContext::default(),
        exact,
    )
    .unwrap();
    assert_eq!(
        import
            .serialize_with_limit(expected.len())
            .unwrap()
            .as_css(),
        expected
    );
    let error = import.serialize_with_limit(expected.len() - 1).unwrap_err();
    assert!(
        matches!(error, CssImportSerializationError::Component(ref error) if error.kind() == CssComponentValueErrorKind::ByteLimit)
    );
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("original terminator")
    };
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.len() - 1
    );
}

#[test]
fn complete_output_byte_budget_identifies_the_first_overflowing_rule_token() {
    let rule = construct("@import 'x' (width:1px);");
    let error = rule.serialize_with_limit(0).unwrap_err();
    assert!(matches!(
        error,
        CssImportSerializationError::Component(ref error)
            if error.kind() == CssComponentValueErrorKind::ByteLimit
    ));
    // The complete output starts with @import. A speculative tail's whitespace
    // or media token is not the first token overflowing the whole-rule budget.
    original(error.origin(), rule.origin());
}

#[test]
fn canonical_output_keeps_complete_clause_interpretation_and_needed_token_boundaries() {
    for source in [
        "layer()",
        "supports(2px)",
        "layer(initial)",
        "layer(theme) and (color)",
        "supports(display:grid) and (color)",
    ] {
        let import = tail(source);
        let output = import.serialize().unwrap();
        assert_eq!(output.as_css(), format!("@import 'x' {source};"));
        let reparsed = construct(output.as_css());
        assert!(reparsed.layer().is_none());
        assert!(reparsed.supports().is_none());
        assert_eq!(reparsed.media().unwrap().queries().len(), 1);
        assert_eq!(reparsed.serialize().unwrap().as_css(), output.as_css());
    }
    let function =
        CssComponentValue::try_function("Future", values(vec![token("1"), token("e2")])).unwrap();
    let import = CssImportRule::try_from_components(
        values(vec![
            token("@import"),
            token(" "),
            token("'x'"),
            token(" "),
            function,
            token(";"),
        ]),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(
        import.serialize().unwrap().as_css(),
        "@import 'x' Future(1/**/e2);"
    );
    let report = parse_sheet(import.serialize().unwrap().as_css());
    assert!(report.is_clean());
    assert!(matches!(report.syntax().rules(), [CssRule::Import(_)]));
}
