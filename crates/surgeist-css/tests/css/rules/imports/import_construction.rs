//! Whole-rule checked import contracts: Cascade optional-clause interpretation,
//! exact component provenance, and strict admission independent of loading.
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use crate::css::support::rules::import_from_components as construct;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::numeric::CssCalculationExpressionRef;
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::query::CssConditionKind;
use surgeist_css::query::media::{
    CssMediaConstructionError, CssMediaFeatureQuery, CssMediaLeaf, CssMediaQuery, CssMediaRangeRef,
    CssMediaType,
};
use surgeist_css::query::supports::CssSupportsLeaf;
use surgeist_css::rule::{CssImportLayer, CssImportRule, CssImportTarget, CssRule};
use surgeist_css::selector::{CssNamespaceConstraint, CssNamespaceContext, CssSelector};
use surgeist_css::serialization::CssSerializationError;
use surgeist_css::values::effects::CssOpacityValue;

fn values(items: Vec<CssComponentValue>) -> CssComponentValues {
    CssComponentValues::try_new(items, Default::default()).unwrap()
}
fn token(source: &str) -> CssComponentValue {
    CssComponentValue::try_token(source).unwrap()
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
    let CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(value)) = condition.kind() else {
        panic!("opaque media")
    };
    assert_eq!(
        value.serialize_tokens(Default::default()).unwrap().as_css(),
        expected
    );
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
        assert_eq!(
            import
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            "@import 'x';"
        );
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
            CssImportRule::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                &CssNamespaceContext::default(),
                Default::default(),
            )
            .as_ref()
            .is_err_and(|error| error.is_invalid_rule_grammar()),
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
            CssConditionKind::Leaf(CssSupportsLeaf::Declaration(_))
        ));
        assert!(
            matches!(import.media().unwrap().queries(), [CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print)
        );
        assert_eq!(
            import
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
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
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(declaration)) =
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
            .serialize_tokens(Default::default())
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
        let CssConditionKind::And(children) = condition.kind() else {
            panic!("complete conjunction")
        };
        assert!(matches!(
            children.conditions()[0].kind(),
            CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(_))
        ));
        assert!(matches!(
            children.conditions()[1].kind(),
            CssConditionKind::Leaf(CssMediaLeaf::Feature(_))
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
        CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(_))
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
            CssComponentValues::parse(&CssParser::default(), format!("@import 'x' {source};"))
                .unwrap(),
            &CssNamespaceContext::default(),
            Default::default(),
        )
        .unwrap_err();
        assert!(
            error
                .media_error()
                .is_some_and(CssMediaConstructionError::is_invalid_query_grammar),
            "{source}: {error:?}"
        );
    }
}

#[test]
fn namespace_context_changes_selector_admission_without_resolving_the_namespace() {
    let sheet =
        CssParser::default().parse_sheet("@namespace svg 'urn:svg'; @namespace 'urn:default';");
    assert!(sheet.is_clean());
    let context = CssNamespaceContext::from_sheet(sheet.syntax());
    let unbound = tail("supports(selector(svg|a))");
    assert!(matches!(
        unbound.supports().unwrap().condition().kind(),
        CssConditionKind::Leaf(CssSupportsLeaf::GeneralEnclosed(_))
    ));
    for (source, named) in [
        ("@import 'x' supports(selector(svg|a));", true),
        ("@import 'x' supports(selector(a));", false),
    ] {
        let import = CssImportRule::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            &context,
            Default::default(),
        )
        .unwrap();
        let CssConditionKind::Leaf(CssSupportsLeaf::Selector(CssSelector::Compound(selector))) =
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
        assert_eq!(
            import
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
    }
}

#[test]
fn programmatic_import_and_mixed_numeric_children_keep_honest_independent_origins() {
    let target = CssComponentValues::parse(&CssParser::default(), "/*😀*/'x'")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let supports_leaf = CssComponentValues::parse(&CssParser::default(), "/*😀*/1e999")
        .unwrap()
        .items()
        .last()
        .unwrap()
        .clone();
    let media_leaf = CssComponentValues::parse(&CssParser::default(), "/*λ*/\n+001.5PX")
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
        surgeist_css::component::CssBlockKind::Parenthesis,
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
        Default::default(),
    )
    .unwrap();
    assert_eq!(import.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(import.position(), None);
    let CssConditionKind::Leaf(CssSupportsLeaf::Declaration(declaration)) =
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
    let CssOpacityValue::NumberCalculation(calculation) = opacity.value() else {
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
    let CssConditionKind::Leaf(CssMediaLeaf::Feature(CssMediaFeatureQuery::Width(range))) =
        condition.kind()
    else {
        panic!("width")
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("plain width")
    };
    original(value.calculation().origin(), &media_origin);
    let output = import.serialize_tokens(Default::default()).unwrap();
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
        supports
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "(opacity:calc(1e999))"
    );
    assert_eq!(
        media.serialize_tokens(Default::default()).unwrap().as_css(),
        "(width: +001.5PX)"
    );
}

#[test]
fn parsed_at_keyword_and_terminator_keep_their_snapshots_among_programmatic_children() {
    let parsed = CssComponentValues::parse(&CssParser::default(), "/*😀*/@IMPORT;").unwrap();
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
        Default::default(),
    )
    .unwrap();
    original(import.origin(), &at_origin);
    let CssValueOrigin::Parsed(at) = &at_origin else {
        panic!("at")
    };
    assert_eq!(import.position(), Some(at.span().start()));
    let output = import.serialize_tokens(Default::default()).unwrap();
    assert_eq!(output.as_css(), "@import \"\";");
    let Some(CssSerializedOrigin::Token(origin)) = output.origin_at(output.as_css().len() - 1)
    else {
        panic!("terminator")
    };
    original(origin, &end_origin);
}

#[test]
fn recovered_components_reject_before_invalid_envelope_and_preserve_eof_provenance() {
    let recovered = CssComponentValues::parse(&CssParser::default(), "Future(").unwrap();
    let CssComponentValueRef::Function(function) = recovered.items()[0].view() else {
        panic!("function")
    };
    let expected = function.closing_origin().clone();
    let error = CssImportRule::try_from_components(
        recovered.clone(),
        &CssNamespaceContext::default(),
        Default::default(),
    )
    .unwrap_err();
    assert!(error.is_recovered_input());
    assert_eq!(error.origin(), Some(&expected));
    let mut components = vec![token("@import"), token(" "), token("'x'"), token(" ")];
    components.extend_from_slice(recovered.items());
    components.push(token(";"));
    assert!(
        CssImportRule::try_from_components(
            values(components),
            &CssNamespaceContext::default(),
            Default::default()
        )
        .as_ref()
        .is_err_and(|error| error.is_recovered_input())
    );
    let zero = CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap();
    let error =
        CssImportRule::try_from_components(recovered, &CssNamespaceContext::default(), zero)
            .unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|error| error.is_component_limit())
    );
}

#[test]
fn all_input_trivia_counts_toward_limits_before_canonical_trivia_is_omitted() {
    let source = "@import 'x'; /**/";
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap();
    let error = CssImportRule::try_from_components(
        components.clone(),
        &CssNamespaceContext::default(),
        limits,
    )
    .unwrap_err();
    let Some(error) = error.component_error() else {
        panic!("whole input byte budget")
    };
    assert!(error.is_byte_limit(), "{error:?}");
    original(error.origin(), components.items().last().unwrap().origin());
    let limits =
        CssComponentValueLimits::try_new(0, components.items().len(), source.len()).unwrap();
    assert_eq!(
        CssImportRule::try_from_components(
            components.clone(),
            &CssNamespaceContext::default(),
            limits
        )
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css(),
        "@import 'x';"
    );
    let limits =
        CssComponentValueLimits::try_new(0, components.items().len() - 1, source.len()).unwrap();
    assert!(
        CssImportRule::try_from_components(components, &CssNamespaceContext::default(), limits)
            .as_ref()
            .is_err_and(|error| error
                .component_error()
                .is_some_and(|error| error.is_component_limit()))
    );
    let components =
        CssComponentValues::parse(&CssParser::default(), "@import 'x' future();").unwrap();
    let limits = CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap();
    assert!(
        CssImportRule::try_from_components(components, &CssNamespaceContext::default(), limits)
            .as_ref()
            .is_err_and(|error| error
                .component_error()
                .is_some_and(|error| error.is_nesting_limit()))
    );
}

#[test]
fn byte_limits_include_canonical_media_expansion_and_the_output_terminator() {
    let source = "@import 'x' (ASPECT-RATIO:2);";
    let expected = "@import 'x' (aspect-ratio: 2 / 1);";
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
    let tight = CssComponentValueLimits::try_new(256, usize::MAX, source.len()).unwrap();
    let error = CssImportRule::try_from_components(
        components.clone(),
        &CssNamespaceContext::default(),
        tight,
    )
    .unwrap_err();
    assert!(
        error
            .serialization_error()
            .is_some_and(|error| error.is_byte_limit()),
        "{error:?}"
    );
    let exact = CssComponentValueLimits::try_new(256, usize::MAX, expected.len()).unwrap();
    let import =
        CssImportRule::try_from_components(components, &CssNamespaceContext::default(), exact)
            .unwrap();
    assert_eq!(
        import
            .serialize_tokens(crate::css::support::byte_limits(expected.len()))
            .unwrap()
            .as_css(),
        expected
    );
    let error = import
        .serialize_tokens(crate::css::support::byte_limits(expected.len() - 1))
        .unwrap_err();
    assert!(error.is_byte_limit() && component_kind(&error) == Some(ComponentFailure::ByteLimit));
    let Some(CssValueOrigin::Parsed(origin)) = error.origin() else {
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
    let error = rule
        .serialize_tokens(crate::css::support::byte_limits(0))
        .unwrap_err();
    assert!(error.is_byte_limit(), "{error:?}");
    assert_eq!(component_kind(&error), Some(ComponentFailure::ByteLimit));
    // The complete output starts with @import. A speculative tail's whitespace
    // or media token is not the first token overflowing the whole-rule budget.
    original(
        error.origin().expect("overflowing token origin"),
        rule.origin(),
    );
}

/// The lexical component failure behind a serialization failure, if any.
fn component_kind(error: &CssSerializationError) -> Option<ComponentFailure> {
    std::error::Error::source(error)
        .and_then(|source| source.downcast_ref::<CssComponentValueError>())
        .map(component_failure)
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
        let output = import.serialize_tokens(Default::default()).unwrap();
        assert_eq!(output.as_css(), format!("@import 'x' {source};"));
        let reparsed = construct(output.as_css());
        assert!(reparsed.layer().is_none());
        assert!(reparsed.supports().is_none());
        assert_eq!(reparsed.media().unwrap().queries().len(), 1);
        assert_eq!(
            reparsed
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            output.as_css()
        );
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
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        import
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "@import 'x' Future(1/**/e2);"
    );
    let report = CssParser::default().parse_sheet(
        import
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
    );
    assert!(report.is_clean());
    assert!(matches!(report.syntax().rules(), [CssRule::Import(_)]));
}
