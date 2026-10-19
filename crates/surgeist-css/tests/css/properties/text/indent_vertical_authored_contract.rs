#![forbid(unsafe_code)]
//! Existing-public-API contract for the authored indent/alignment lifecycle.
//!
//! Independent sources: Text4 WD2026-08-14 §9.1 supplies text-indent's signed
//! length-percentage && hanging? && each-line?, initial 0 and inheritance yes.
//! CSS2 REC2011 §10.8.1 supplies vertical-align's eight keywords or signed
//! length/percentage, initial baseline and inheritance no. These are separate
//! sources; this contract does not adopt the Inline3 vertical-align shorthand.
//! Shared Values4 numeric behavior and existing owner resource policy remain
//! accepted: exact coefficients/origins, specified math projection, indent
//! aggregate +1 and present flags +1 each, alignment numeric carrier +0,
//! declaration/name +2, punctuation/importance bytes only.
//!
//! No absent CssLonghandValueRef variants are named. Typed borrowed initial and
//! projected payload inspection must accompany their functional implementation.
//! Expectations never depend on catalog status, issue state, Debug text, or a
//! serializer's output being copied into a golden.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn parsed(p: P, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", p.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    source.clone()
}

fn checked(p: P, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = if grammar {
        parse_property_value_for_grammar(p.grammar(), components.clone(), CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap();
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    source
}

fn text_front(p: P, value: &str, grammar: bool) -> CssDeclaration {
    let report = if grammar {
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Important)
    } else {
        parse_property_value_text(
            value,
            CssPropertyNameRef::Known(p),
            CssImportance::Important,
        )
    };
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let source = report.syntax().as_ref().unwrap().clone();
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
    source
}

fn fronts(p: P, value: &str) -> [CssDeclaration; 5] {
    [
        parsed(p, value),
        checked(p, value, false),
        checked(p, value, true),
        text_front(p, value, false),
        text_front(p, value, true),
    ]
}

fn indent(source: &CssDeclaration) -> &CssTextIndent {
    let CssKnownPropertyValueRef::TextIndent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("owning indent payload")
    };
    value.value()
}

fn align(source: &CssDeclaration) -> &CssVerticalAlign {
    let CssKnownPropertyValueRef::VerticalAlign(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("owning CSS2 alignment payload")
    };
    value.value()
}

fn scalar(source: &CssDeclaration) -> &CssSpecifiedLengthPercentage {
    match source.known().unwrap().property() {
        P::TextIndent => indent(source).length(),
        P::VerticalAlign => {
            let CssVerticalAlign::Length(value) = align(source) else {
                panic!("numeric alignment")
            };
            value
        }
        _ => panic!("selected numeric family"),
    }
}

fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().parsed_name(), source.parsed_name());
    assert_eq!(item.source().parsed_value(), source.parsed_value());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed longhand expansion")
    };
    assert_one(&values, source);
    values
}

fn assert_one(values: &CssLonghandContributions, source: &CssDeclaration) {
    let [item] = values.items() else {
        panic!("exactly one terminal, no reset-only member")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_source(item, source);
}

fn intrinsic(p: P, inherited: bool) {
    let metadata = p.metadata().unwrap();
    assert_eq!(metadata.grammar(), p.grammar());
    let CssPropertyKindRef::Longhand(value) = metadata.kind() else {
        panic!("intrinsic longhand")
    };
    assert_eq!(value.property().known_property(), p);
    assert_eq!(value.inherited_by_default(), inherited);
    let initial = value.initial_value();
    assert_eq!(initial.property().known_property(), p);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary intrinsic initial")
    };
    assert_eq!(initial.property().known_property(), p);
    assert_eq!(p.grammar().metadata().unwrap().grammar(), p.grammar());
}

#[test]
fn indent_has_intrinsic_inherited_ordinary_longhand_metadata() {
    intrinsic(P::TextIndent, true);
}

#[test]
fn css2_alignment_has_intrinsic_noninherited_ordinary_longhand_metadata() {
    intrinsic(P::VerticalAlign, false);
}

fn ordinary(p: P, input: &str, canonical: &str) {
    for source in fronts(p, input) {
        let before = source.clone();
        let values = completed(&source);
        let item = &values.items()[0];
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            p
        );
        assert!(item.replacement_components().is_none());
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{}: {canonical} !important;", p.canonical_name())
        );
        assert_eq!(source, before);
    }
}

#[test]
fn ordinary_indent_expands_once_through_all_existing_fronts() {
    for (input, canonical) in [
        ("0", "0"),
        ("each-line -2.50% hanging", "-2.5% hanging each-line"),
        ("hanging calc(-2px - 3%)", "calc(-3% - 2px) hanging"),
    ] {
        ordinary(P::TextIndent, input, canonical);
    }
}

#[test]
fn ordinary_css2_alignment_expands_once_through_all_existing_fronts() {
    for input in [
        "baseline",
        "sub",
        "super",
        "top",
        "text-top",
        "middle",
        "bottom",
        "text-bottom",
        "-2px",
        "-25%",
        "0%",
    ] {
        ordinary(P::VerticalAlign, input, input);
    }
    ordinary(P::VerticalAlign, "calc(-2px - 3%)", "calc(-3% - 2px)");
}

fn globals(p: P) {
    for (text, keyword) in GLOBALS {
        for source in fronts(p, text) {
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            let values = completed(&source);
            assert!(
                matches!(values.items()[0].value(), CssContributionValueRef::Global(v) if v == keyword)
            );
            assert!(values.items()[0].replacement_components().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: {text} !important;", p.canonical_name())
            );
        }
    }
}

#[test]
fn indent_globals_remain_symbolic_with_original_importance() {
    globals(P::TextIndent);
}

#[test]
fn alignment_globals_remain_symbolic_with_original_importance() {
    globals(P::VerticalAlign);
}

fn pending(p: P, replacement_text: &str) {
    for text in ["var(--value)", "env(value)", "attr(data-value *)"] {
        for source in fronts(p, text) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole occurrence pending")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: {text} !important;", p.canonical_name())
            );
            let replacement = parse_component_values(replacement_text).unwrap();
            let replacement_before = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed reentry")
            };
            assert_one(&values, &source);
            let item = &values.items()[0];
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                p
            );
            assert_eq!(item.replacement_components(), Some(&replacement));
            for (actual, supplied) in item
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(replacement.items())
            {
                assert_eq!(actual.origin(), supplied.origin());
                let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                    (actual.origin(), supplied.origin())
                else {
                    panic!("replacement origins")
                };
                assert!(actual.source().same_snapshot(supplied.source()));
                assert_eq!(actual.source().as_str(), replacement_text);
            }
            for (text, keyword) in GLOBALS {
                let replacement = parse_component_values(text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global reentry")
                };
                assert_one(&values, &source);
                assert!(
                    matches!(values.items()[0].value(), CssContributionValueRef::Global(v) if v == keyword)
                );
                assert_eq!(
                    values.items()[0].replacement_components(),
                    Some(&replacement)
                );
            }
            assert_eq!(replacement, replacement_before);
            assert_eq!(source, before);
            assert!(source.known().unwrap().substitution_dependent().is_some());
        }
    }
}

#[test]
fn indent_pending_reentry_retains_both_original_and_replacement_snapshots() {
    pending(P::TextIndent, "each-line calc(-2px - 3%) hanging");
}

#[test]
fn alignment_pending_reentry_retains_both_original_and_replacement_snapshots() {
    pending(P::VerticalAlign, "calc(-2px - 3%)");
}

fn retry(p: P, valid: &str, invalid: &[&str]) {
    let source = parsed(p, "var(--value)");
    let before = source.clone();
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for text in [
        "var(--again)",
        "env(again)",
        "attr(data-again *)",
        "calc(var(--again) - 2px)",
        r"\76 ar(--again)",
        "var(--again",
    ] {
        for _ in 0..2 {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::ResidualSubstitution
            ));
        }
    }
    for text in invalid {
        for _ in 0..2 {
            let error = handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("strict invalid replacement")
            };
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
        }
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values(valid).unwrap())
        .unwrap()
    else {
        panic!("successful retry")
    };
    assert_one(&values, &source);
    assert!(matches!(
        values.items()[0].value(),
        CssContributionValueRef::Ordinary(_)
    ));
    assert!(handle.source().same_occurrence(&source));
    assert_eq!(source, before);
}

#[test]
fn indent_failed_replacements_are_atomic_and_the_handle_can_retry() {
    retry(
        P::TextIndent,
        "each-line -2px hanging",
        &[
            "hanging",
            "1px 2px",
            "1px hanging hanging",
            "1px!important",
            "1px;color:red",
            "calc(1px + 1s)",
        ],
    );
}

#[test]
fn alignment_failed_replacements_are_atomic_and_the_handle_can_retry() {
    retry(
        P::VerticalAlign,
        "-2px",
        &[
            "auto",
            "top bottom",
            "-1",
            "1px!important",
            "1px;color:red",
            "calc(1px + 1s)",
        ],
    );
}

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("implicit comment or function closure")
}

fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("original EOF origin")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}

fn original_closure(p: P, ordinary: &str) {
    let mut texts = vec![
        format!("{ordinary}/*"),
        "calc(-2px - 3%".into(),
        "var(--value".into(),
        "env(value".into(),
        "attr(data-value *".into(),
        "var(--value)/*".into(),
    ];
    texts.extend(GLOBALS.map(|(text, _)| format!("{text}/*")));
    for text in texts {
        let components = parse_component_values(&text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(p),
                components.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                p.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ] {
            assert_closure(&result.unwrap_err(), &origin, &text);
        }
        assert_eq!(components, before);
    }
    for text in [
        format!("{ordinary}/**/"),
        "calc(-2px - 3%)".into(),
        "initial/**/".into(),
        "var(--value)/**/".into(),
    ] {
        checked(p, &text, false);
        checked(p, &text, true);
    }
}

#[test]
fn indent_checked_fronts_reject_original_implicit_closure_before_classification() {
    original_closure(P::TextIndent, "each-line -2px hanging");
}

#[test]
fn alignment_checked_fronts_reject_original_implicit_closure_before_classification() {
    original_closure(P::VerticalAlign, "baseline");
}

fn reentry_closure(p: P, ordinary: &str) {
    let source = checked(p, "var(--value)", true);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for text in [
        format!("{ordinary}/*"),
        "initial/*".into(),
        "calc(-2px - 3%".into(),
    ] {
        let components = parse_component_values(&text).unwrap();
        let origin = implicit_origin(&components);
        for _ in 0..2 {
            let error = handle.reenter(components.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("original closure failure")
            };
            assert_closure(error, &origin, &text);
        }
        assert!(handle.source().same_occurrence(&source));
    }
    for text in [
        format!("{ordinary}/**/"),
        "initial/**/".into(),
        "calc(-2px - 3%)".into(),
    ] {
        assert!(
            handle
                .reenter(parse_component_values(&text).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn indent_reentry_rejects_repaired_eof_and_allows_matched_retry() {
    reentry_closure(P::TextIndent, "each-line -2px hanging");
}

#[test]
fn alignment_reentry_rejects_repaired_eof_and_allows_matched_retry() {
    reentry_closure(P::VerticalAlign, "baseline");
}

fn recovered_normalization(p: P, ordinary: &str) {
    for text in [
        format!("{ordinary}/*"),
        "initial/*".into(),
        "var(--value)/*".into(),
        "calc(-2px - 3%".into(),
        "var(--value".into(),
    ] {
        let css = format!(".a{{{}:{text}", p.canonical_name());
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(validate_sheet(&css).is_err());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("one retained recovered occurrence")
        };
        assert_eq!(item.order(), 0);
        assert_eq!(item.source().known().unwrap().property(), p);
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_one(values, item.source())
            }
            CssExpansion::Pending(handle) => {
                assert!(handle.source().same_occurrence(item.source()))
            }
            _ => panic!("one terminal or whole pending recovered occurrence"),
        }
        assert!(
            normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap()
            )
            .is_err()
        );
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}

#[test]
fn browser_indent_recovery_retains_diagnostics_and_occurrences_through_normalization() {
    recovered_normalization(P::TextIndent, "-2px hanging each-line");
}

#[test]
fn browser_alignment_recovery_retains_diagnostics_and_occurrences_through_normalization() {
    recovered_normalization(P::VerticalAlign, "baseline");
}

#[test]
fn normalization_preserves_duplicate_order_importance_pending_context_and_limits() {
    let css = "@media screen{.a{text-indent:each-line -2px hanging!important;vertical-align:middle;text-indent:var(--value);vertical-align:initial;text-indent:-25%;vertical-align:calc(-2px - 3%)}}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 6, 6).unwrap();
    let normalized = normalize_report_with_limits(&report, exact).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    let expected = [
        "text-indent: -2px hanging each-line !important;",
        "vertical-align: middle;",
        "text-indent: var(--value);",
        "vertical-align: initial;",
        "text-indent: -25%;",
        "vertical-align: calc(-3% - 2px);",
    ];
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property(),
            if index % 2 == 0 {
                P::TextIndent
            } else {
                P::VerticalAlign
            }
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert_eq!(item.source().to_specified_css().unwrap(), expected[index]);
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 2);
                assert!(handle.source().same_occurrence(item.source()));
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_one(values, item.source());
                if index == 3 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Initial)
                    ));
                } else {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Ordinary(_)
                    ));
                }
                assert!(values.items()[0].replacement_components().is_none());
            }
            _ => panic!("one terminal or whole pending occurrence"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 6, 5).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 5, 6).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_report_with_limits(&report, limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 5 }
            );
            assert_eq!(error.declaration_order(), Some(5));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[5].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn accepted_indent_permutations_signed_flags_and_tiny_origins_remain_controls() {
    for text in [
        "-1px hanging each-line",
        "-1px each-line hanging",
        "hanging -1px each-line",
        "hanging each-line -1px",
        "each-line -1px hanging",
        "each-line hanging -1px",
    ] {
        for source in fronts(P::TextIndent, text) {
            assert!(indent(&source).hanging());
            assert!(indent(&source).each_line());
            assert_eq!(
                indent(&source).serialize_specified().unwrap(),
                "-1px hanging each-line"
            );
        }
    }
    for source in fronts(P::TextIndent, "each-line -1e-999% hanging") {
        let component = scalar(&source).literal_component().unwrap();
        let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = component.view()
        else {
            panic!("exact tiny percentage")
        };
        assert_eq!(number.representation(), "-1e-999");
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("numeric original")
        };
        assert!(
            origin
                .source()
                .as_str()
                .contains("each-line -1e-999% hanging")
        );
        assert_eq!(
            indent(&source).serialize_specified().unwrap(),
            "0% hanging each-line"
        );
        assert_eq!(number.representation(), "-1e-999");
    }
}

#[test]
fn accepted_css2_keywords_signed_numeric_and_authored_zero_forms_remain_controls() {
    for input in [
        "baseline",
        "sub",
        "super",
        "top",
        "text-top",
        "middle",
        "bottom",
        "text-bottom",
        "-2px",
        "-25%",
        "0%",
        "0cm",
    ] {
        for source in fronts(P::VerticalAlign, input) {
            assert_eq!(align(&source).serialize_specified().unwrap(), input);
        }
    }
}

#[test]
fn numeric_checked_components_retain_programmatic_origin_and_original_token() {
    for p in [P::TextIndent, P::VerticalAlign] {
        let component = CssComponentValue::try_dimension("-0.50", "em").unwrap();
        let components = CssComponentValues::try_new(vec![component.clone()]).unwrap();
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(p),
                components.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                p.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ] {
            let source = result.unwrap();
            assert_eq!(scalar(&source).literal_component(), Some(&component));
            assert_eq!(scalar(&source).origin(), &CssValueOrigin::Programmatic);
            assert_eq!(source.value_components(), &components);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{}: -0.5em !important;", p.canonical_name())
            );
            assert_eq!(scalar(&source).literal_component(), Some(&component));
        }
    }
}

#[test]
fn grammar_failure_preserves_unrelated_siblings_and_checked_inputs() {
    for (p, invalid) in [
        (
            P::TextIndent,
            &[
                "hanging",
                "each-line hanging",
                "1px 2px",
                "1px hanging hanging",
                "1px sideways",
            ][..],
        ),
        (
            P::VerticalAlign,
            &["auto", "top bottom", "-1", "hanging", "1fr"][..],
        ),
    ] {
        for value in invalid {
            let css = format!("color:red;{}:{value};color:blue", p.canonical_name());
            let report = parse_style_attribute(&css);
            assert_eq!(report.syntax().len(), 2);
            assert!(
                report
                    .syntax()
                    .iter()
                    .all(|v| v.known().unwrap().property() == P::Color)
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one grammar failure")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
                panic!("owning grammar error")
            };
            assert_eq!(detail.property(), p);
            assert_eq!(
                validate_style_attribute(&css).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            let components = parse_component_values(value).unwrap();
            let before = components.clone();
            for result in [
                parse_property_value(
                    CssPropertyNameRef::Known(p),
                    components.clone(),
                    CssImportance::Normal,
                ),
                parse_property_value_for_grammar(
                    p.grammar(),
                    components.clone(),
                    CssImportance::Normal,
                ),
            ] {
                assert!(matches!(
                    result.unwrap_err().kind(),
                    CssPropertyValueErrorKind::Grammar(_)
                ));
            }
            assert_eq!(components, before);
        }
    }
}

#[test]
fn declaration_resource_costs_accumulate_without_changing_the_owning_payload() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    for (p, input, canonical, inputs, projections) in [
        (P::TextIndent, "0", "0", 2, 2),
        (P::TextIndent, "hanging -2px", "-2px hanging", 3, 3),
        (
            P::TextIndent,
            "each-line -2px hanging",
            "-2px hanging each-line",
            4,
            4,
        ),
        (
            P::TextIndent,
            "each-line calc(1px + 2px) hanging",
            "calc(3px) hanging each-line",
            7,
            6,
        ),
        (P::VerticalAlign, "baseline", "baseline", 1, 1),
        (P::VerticalAlign, "-25%", "-25%", 1, 1),
        (P::VerticalAlign, "calc(1px + 2px)", "calc(3px)", 4, 3),
    ] {
        let source = checked(p, input, true);
        let before = source.clone();
        let expected = format!("{}: {canonical} !important;", p.canonical_name());
        let exact = L::new(inputs + 2, projections + 2, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                L::new(inputs + 1, projections + 2, expected.len()),
                K::InputNodeLimit,
            ),
            (
                L::new(inputs + 2, projections + 1, expected.len()),
                K::ProjectionNodeLimit,
            ),
            (
                L::new(inputs + 2, projections + 2, expected.len() - 1),
                K::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
}

#[test]
fn sheet_declarations_share_one_byte_budget_and_failure_is_atomic_and_retryable() {
    use CssSpecifiedValueSerializationLimits as L;
    let report = parse_sheet(
        ".a{text-indent:each-line -2px hanging!important}.b{vertical-align:calc(1px + 2px)}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected =
        ".a { text-indent: -2px hanging each-line !important; }\n.b { vertical-align: calc(3px); }";
    let exact = L::new(usize::MAX, usize::MAX, expected.len());
    let short = L::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
