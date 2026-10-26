//! Containment2 authored checked/reentry fronts must retain original EOF failures.
//! Browser recovery remains available and does not turn a recovered report clean.
use crate::css::support::property_admission::assert_implicit_closure;
use surgeist_css::*;
const PROPERTY: CssKnownProperty = CssKnownProperty::ContentVisibility;
const GLOBALS: [&str; 5] = ["initial", "inherit", "unset", "revert", "revert-layer"];
const PENDING: [&str; 3] = ["var(--visible)", "env(visible)", "attr(data-visible)"];
fn implicit(components: &CssComponentValues) -> CssValueOrigin {
    for item in components.items() {
        let closing = match item.view() {
            CssComponentValueRef::Function(value) => Some(value.closing_origin()),
            CssComponentValueRef::Block(value) => Some(value.closing_origin()),
            _ => None,
        };
        if let Some(value @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return value.clone();
        }
    }
    let text = components.serialize_tokens(Default::default()).unwrap();
    (0..text.as_css().len())
        .find_map(|offset| match text.origin_at(offset) {
            Some(CssSerializedOrigin::Token(value @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(value.clone())
            }
            _ => None,
        })
        .expect("independently selected original EOF stimulus")
}
fn original_points(origin: &CssValueOrigin, text: &str) {
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert_eq!(at.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    let (start, end) = if let Some(start) = text.rfind("/*unfinished") {
        (start, text.len())
    } else {
        let start = text
            .rfind("var(")
            .or_else(|| text.rfind("env("))
            .or_else(|| text.rfind("attr("))
            .or_else(|| text.rfind("future("))
            .expect("independent unfinished function witness");
        (start, start + text[start..].find('(').unwrap() + 1)
    };
    assert_eq!(opening.span().start().byte_offset().value(), start);
    assert_eq!(opening.span().end().byte_offset().value(), end);
    assert_eq!(
        opening.span().start().column().value() as usize,
        text[..start].encode_utf16().count()
    );
    assert_eq!(
        opening.span().end().column().value() as usize,
        text[..end].encode_utf16().count()
    );
    assert_eq!(at.span().start(), at.span().end());
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(
        at.span().start().column().value() as usize,
        text.encode_utf16().count()
    );
}
fn expected(error: &CssPropertyValueError, origin: &CssValueOrigin) -> bool {
    error.code() == Some(CssErrorCode::UnexpectedEnd)
        && error.origin() == CssSerializedOrigin::End(Some(origin.clone()))
}
fn reject_original(inputs: Vec<String>) {
    let mut failures = Vec::new();
    for text in inputs {
        let components = CssComponentValues::parse(&CssParser::default(), &text).unwrap();
        let before = components.clone();
        let origin = implicit(&components);
        original_points(&origin, &text);
        for (front, result) in [
            (
                "property",
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Property(CssPropertyNameRef::Known(PROPERTY)),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
            (
                "grammar",
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Grammar(PROPERTY.grammar()),
                    components.clone(),
                    CssImportance::Important,
                ),
            ),
        ] {
            if !matches!(result, Err(ref error) if expected(error, &origin)) {
                failures.push(format!(
                    "{front}/{text}: original EOF was admitted or lost its typed failure/origin"
                ));
            }
            assert_eq!(components, before);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn ordinary_and_all_globals_reject_original_unclosed_comments_at_both_checked_fronts() {
    reject_original(
        ["visible", "auto", "hidden"]
            .into_iter()
            .chain(GLOBALS)
            .map(|text| format!("/*😀*/{text}/*unfinished"))
            .collect(),
    );
}
#[test]
fn pending_substitutions_reject_original_unclosed_comments_at_both_checked_fronts() {
    reject_original(
        PENDING
            .into_iter()
            .map(|text| format!("/*😀*/{text}/*unfinished"))
            .collect(),
    );
}
#[test]
fn pending_functions_reject_original_unclosed_delimiters_at_both_checked_fronts() {
    reject_original(
        ["var(--visible", "env(visible", "attr(data-visible"]
            .into_iter()
            .map(|text| format!("/*😀*/{text}"))
            .collect(),
    );
}
#[test]
fn pending_reentry_rejects_original_closure_but_remains_residual_first_and_retryable() {
    let mut failures = Vec::new();
    for text in PENDING {
        let source = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(PROPERTY)),
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        let source_before = source.clone();
        let CssExpansion::Pending(handle) = source.expand().unwrap() else {
            panic!("pending")
        };
        for replacement_text in ["auto", "hidden", "visible"]
            .into_iter()
            .chain(GLOBALS)
            .map(|text| format!("/*😀*/{text}/*unfinished"))
            .chain(["/*😀*/future(value".to_owned()])
        {
            let components =
                CssComponentValues::parse(&CssParser::default(), &replacement_text).unwrap();
            let before = components.clone();
            let origin = implicit(&components);
            original_points(&origin, &replacement_text);
            for _ in 0..2 {
                if !matches!(handle.reenter(components.clone()), Err(ref error)
                    if error.replacement_error().is_some_and(|value| expected(value, &origin)))
                {
                    failures.push(format!(
                        "{text}/{replacement_text}: strict original replacement EOF lost"
                    ));
                }
                assert_eq!(components, before);
                assert_eq!(source, source_before);
                assert!(handle.source().same_occurrence(&source));
            }
        }
        for replacement_text in [
            "var(--again",
            "env(again",
            "attr(data-again",
            "[future(var(--again",
        ] {
            let components =
                CssComponentValues::parse(&CssParser::default(), replacement_text).unwrap();
            let before = components.clone();
            assert!(
                handle
                    .reenter(components.clone())
                    .unwrap_err()
                    .is_residual_substitution()
            );
            assert_eq!(components, before);
        }
        let components =
            CssComponentValues::parse(&CssParser::default(), "/*😀*/auto/**/").unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(components.clone()).unwrap()
            else {
                panic!("ordinary retry")
            };
            let [item] = values.items() else {
                panic!("one terminal")
            };
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&components));
            assert_eq!(
                item.ordinary_value().unwrap().view(),
                CssLonghandValueRef::ContentVisibility(&CssContentVisibility::Auto)
            );
            assert_eq!(source, source_before);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn closed_original_components_remain_callable_for_ordinary_global_and_pending_values() {
    for text in ["auto", "hidden", "visible"]
        .into_iter()
        .chain(GLOBALS)
        .chain(PENDING)
    {
        let components =
            CssComponentValues::parse(&CssParser::default(), format!("/*😀*/{text}/**/")).unwrap();
        let source = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(PROPERTY.grammar()),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(source.expand().is_ok());
    }
}
#[test]
fn browser_recovery_retains_diagnostics_and_occurrences_through_normalization() {
    for text in [
        "auto/*unfinished",
        "initial/*unfinished",
        "var(--visible",
        "env(visible",
        "attr(data-visible",
    ] {
        let css = format!("/*😀*/.a{{color:red;content-visibility:{text}");
        let report = CssParser::default().parse_sheet(&css);
        let before = report.clone();
        assert_implicit_closure(&css, &report);
        assert_eq!(
            CssParser::default()
                .parse_sheet(&css)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
        let normalized = report
            .normalize(CssNormalizationLimits::try_new(0, 1, 2, 2).unwrap())
            .unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let CssRule::Style(rule) = &report.syntax().rules()[0] else {
            panic!("recovered rule")
        };
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 2);
        for (order, item) in declarations.iter().enumerate() {
            assert_eq!(item.order(), order);
            assert!(item.source().same_occurrence(&rule.declarations()[order]));
        }
        // Color and the recovered declaration each contribute one longhand.
        let error = report
            .normalize(CssNormalizationLimits::try_new(0, 1, 2, 1).unwrap())
            .unwrap_err();
        assert_eq!(
            error.exhausted_limit(),
            Some((CssNormalizationResource::Contributions, 1))
        );
        assert_eq!(error.declaration_order(), Some(1));
        assert_eq!(report, before);
    }
}
