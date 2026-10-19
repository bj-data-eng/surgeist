#![forbid(unsafe_code)]
//! Authored Shapes 1 contracts through existing public fronts, CRD 2025-06-12
//! §§3, 4, 6.1–6.3 and accepted authored source dispositions #933/#934/#935.
//! Uses decoded-name discovery rather than absent public property/type variants.
//! Threshold follows imported Color 4 opacity-value, not frozen WebKit's older
//! number-only grammar. Computed clamping, geometry and resources stay downstream.
//! New typed views/constructors and exact initial payloads require functional
//! tests alongside implementation; no missing-symbol RED is claimed here.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

const NAMES: [&str; 3] = ["shape-outside", "shape-image-threshold", "shape-margin"];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> CssKnownProperty {
    let property = CssKnownProperty::from_name(name).unwrap_or_else(|| {
        panic!("selected authored property {name} must accept public discovery")
    });
    assert_eq!(property.canonical_name(), name);
    assert_eq!(
        CssKnownProperty::from_name(&name.to_ascii_uppercase()),
        Some(property)
    );
    assert_eq!(
        CssPropertyGrammar::from_name(name).unwrap(),
        property.grammar()
    );
    property
}

fn declaration(name: &str, input: &str, front: usize) -> CssDeclaration {
    let p = property(name);
    let source = match front {
        0 => {
            let css = format!("/*😀*/{name}:{input}!important");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(validate_style_attribute(&css), Ok(report.syntax().clone()));
            let [source] = report.syntax().as_slice() else {
                panic!("one authored occurrence")
            };
            assert!(source.position().is_some());
            assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
            source.clone()
        }
        1 | 2 => {
            let components = parse_component_values(input).unwrap();
            let before = components.clone();
            let source = checked(p, components.clone(), front == 2).unwrap();
            assert_eq!(components, before);
            assert_eq!(source.value_components(), &components);
            assert!(source.position().is_none());
            assert!(source.parsed_value().is_none());
            source
        }
        3 | 4 => {
            let report = if front == 3 {
                parse_property_value_text(
                    input,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                )
            } else {
                parse_property_value_text_for_grammar(input, p.grammar(), CssImportance::Important)
            };
            assert!(
                report.is_clean(),
                "{name}:{input}: {:?}",
                report.diagnostics()
            );
            let source = report
                .syntax()
                .as_ref()
                .expect("retained direct value")
                .clone();
            assert!(source.position().is_none());
            assert_eq!(source.parsed_value().unwrap().source().as_str(), input);
            source
        }
        _ => unreachable!(),
    };
    assert_eq!(source.known().unwrap().property(), p);
    assert_eq!(source.known().unwrap().grammar(), p.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}

fn checked(
    property: CssKnownProperty,
    components: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    }
}

fn accept(name: &str, cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let before = source.clone();
            let output = format!("{name}: {expected} !important;");
            assert_eq!(
                source.to_specified_css().unwrap(),
                output,
                "{input} front {front}"
            );
            assert_eq!(source, before);
            let reparse = parse_style_attribute(&output);
            assert!(reparse.is_clean(), "{output}: {:?}", reparse.diagnostics());
            let [reparsed] = reparse.syntax().as_slice() else {
                panic!("one reentered occurrence")
            };
            assert_eq!(reparsed.known().unwrap().property(), property(name));
            assert_eq!(reparsed.importance(), CssImportance::Important);
            assert_eq!(reparsed.to_specified_css().unwrap(), output);
        }
    }
}

fn invalid(name: &str, values: &[&str]) {
    let p = property(name);
    for value in values {
        let middle = format!("{name}:{value};");
        let css = format!("/*😀*/color:red!important;{middle}color:blue");
        let report = parse_style_attribute(&css);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only adjacent colors survive {css}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(before.importance(), CssImportance::Important);
        assert_eq!(after.importance(), CssImportance::Normal);
        let [diagnostic] = report.diagnostics() else {
            panic!("one atomic diagnostic {css}")
        };
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("actual property grammar diagnostic")
        };
        assert_eq!(detail.property(), p);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let start = css.find(&middle).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + middle.len()
        );
        assert_eq!(
            diagnostic.span().start().column().value() as usize,
            css[..start].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let components = parse_component_values(value).unwrap();
        let original = components.clone();
        for grammar in [false, true] {
            assert!(
                checked(p, components.clone(), grammar).is_err(),
                "{name}:{value}"
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn outside_admits_exactly_the_four_shapes_boxes_alone_and_in_either_shape_order() {
    for name in ["content-box", "padding-box", "border-box", "margin-box"] {
        accept("shape-outside", &[(name, name)]);
        for input in [format!("circle() {name}"), format!("{name} circle()")] {
            accept(
                "shape-outside",
                &[(input.as_str(), format!("circle() {name}").as_str())],
            );
        }
    }
    accept(
        "shape-outside",
        &[
            (r"BORDER-BOX", "border-box"),
            (r"m\61 rgin-box", "margin-box"),
        ],
    );
}

#[test]
fn omitted_reference_box_stays_omitted_and_explicit_margin_box_stays_authored() {
    // Default margin-box is downstream context, not an authored parser insertion.
    accept(
        "shape-outside",
        &[
            ("circle()", "circle()"),
            ("circle() margin-box", "circle() margin-box"),
        ],
    );
    let omitted = declaration("shape-outside", "circle()", 0);
    let explicit = declaration("shape-outside", "circle() margin-box", 0);
    assert_ne!(
        omitted.known().unwrap().property_value(),
        explicit.known().unwrap().property_value()
    );
}

#[test]
fn all_eight_basic_shape_providers_compose_without_reparsing_or_used_defaults() {
    accept(
        "shape-outside",
        &[
            ("inset(1px)", "inset(1px)"),
            ("circle(calc(1in + 1in))", "circle(calc(192px))"),
            (
                "ellipse(abs(-2em) hypot(3em,4em))",
                "ellipse(calc(2em) calc(5em))",
            ),
            (
                "polygon(nonzero,0 0,100% 0)",
                "polygon(nonzero, 0 0, 100% 0)",
            ),
            ("rect(1px auto 2px 0)", "rect(1px auto 2px 0)"),
            ("xywh(0 0 calc(1px - 2px) 3px)", "xywh(0 0 calc(-1px) 3px)"),
            (
                "path(evenodd,'M01.00,0 L1e2 -0 Z')",
                "path(evenodd, \"M01.00,0 L1e2 -0 Z\")",
            ),
            (
                "shape(from 0px 0px, arc by 1px 2% of 3px)",
                "shape(from 0px 0px, arc by 1px 2% of 3px)",
            ),
        ],
    );
}

#[test]
fn accepted_symbolic_position_families_and_operational_radius_choices_compose() {
    accept(
        "shape-outside",
        &[
            ("circle(25%)", "circle(25%)"),
            ("ellipse(10% closest-side)", "ellipse(10% closest-side)"),
            (
                "circle(at inline-end block-start)",
                "circle(at block-start inline-end)",
            ),
            (
                "ellipse(at end 10% start -2px)",
                "ellipse(at end 10% start -2px)",
            ),
            ("circle(at y-end x-start)", "circle(at x-start y-end)"),
        ],
    );
    invalid(
        "shape-outside",
        &[
            "circle(1px 2px)",
            "circle(-1px)",
            "ellipse(1px)",
            "shape(from 0 0 line to 1px 2px)",
            "shape(from 0 0, arc to 1px 2px)",
        ],
    );
}

#[test]
fn images_and_none_are_exclusive_outside_alternatives() {
    accept(
        "shape-outside",
        &[
            ("NONE", "none"),
            ("url(\"asset.png\")", "url(\"asset.png\")"),
            ("src(\"asset.png\")", "src(\"asset.png\")"),
            ("light-dark(none,none)", "light-dark(none, none)"),
            ("linear-gradient(red,blue)", "linear-gradient(red, blue)"),
        ],
    );
    invalid(
        "shape-outside",
        &[
            "none margin-box",
            "url(\"asset.png\") border-box",
            "circle() url(\"asset.png\")",
            "linear-gradient(red,blue) margin-box",
        ],
    );
}

#[test]
fn outside_rejects_svg_clip_boxes_duplicates_and_foreign_branches() {
    invalid(
        "shape-outside",
        &[
            "fill-box",
            "stroke-box",
            "view-box",
            "no-clip",
            "text",
            "circle() fill-box",
            "circle() stroke-box",
            "circle() view-box",
            "content-box padding-box",
            "circle() border-box padding-box",
            "circle() ellipse()",
            "margin-box circle() margin-box",
            "none circle()",
            "inherit circle()",
            "circle(),circle()",
            "1px",
            "ray(0deg)",
        ],
    );
}

#[test]
fn threshold_preserves_unclamped_numbers_percentages_and_exact_authored_components() {
    accept(
        "shape-image-threshold",
        &[
            ("-2.5", "-2.5"),
            ("1.5", "1.5"),
            ("-25%", "-0.25"),
            ("150%", "1.5"),
            (".0000005", "0.000001"),
            ("1e-47", "0"),
            ("calc(2 - 3)", "calc(-1)"),
            ("calc(25% + 25%)", "calc(50%)"),
            ("calc((1px + 1%) / 1px)", "calc((1% + 1px) / 1px)"),
        ],
    );
    for input in ["-1e-47", "1e100", "-1e-47%", "150%"] {
        let source = declaration("shape-image-threshold", input, 0);
        let serialized = source.value_components().serialize().unwrap();
        assert_eq!(serialized.as_css(), input);
        let before = source.clone();
        let _ = source.to_specified_css().unwrap();
        assert_eq!(source, before);
    }
}

#[test]
fn threshold_rejects_dimensions_keywords_and_multiple_numeric_roots() {
    invalid(
        "shape-image-threshold",
        &[
            "auto",
            "none",
            "1px",
            "1deg",
            "1 2",
            "1,2",
            "calc(1px)",
            "inherit 1",
            "calc(1deg + 2deg)",
        ],
    );
}

#[test]
fn margin_exact_sign_admission_distinguishes_literals_from_symbolic_calculations() {
    accept(
        "shape-margin",
        &[
            ("-0px", "0px"),
            ("+0%", "0%"),
            ("0", "0"),
            ("1e-999px", "0px"),
            ("2em", "2em"),
            ("25%", "25%"),
            ("calc(1px - 2px)", "calc(-1px)"),
            ("calc(1px + 2%)", "calc(2% + 1px)"),
            ("min(1px,2%)", "min(1px, 2%)"),
        ],
    );
    invalid(
        "shape-margin",
        &[
            "-1px",
            "-1e-999px",
            "-1%",
            "-1e-999%",
            "1",
            "1deg",
            "auto",
            "none",
            "2px 3px",
            "calc(1deg)",
        ],
    );
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary longhand contribution")
    };
    items
}

fn member(
    items: &CssLonghandContributions,
    source: &CssDeclaration,
    replacement: Option<&CssComponentValues>,
) {
    let [item] = items.items() else {
        panic!("one terminal; Shapes has no shorthand")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.replacement_components(), replacement);
}

fn metadata(name: &str) {
    let p = property(name);
    let meta = p.metadata().expect("Shapes intrinsic metadata");
    assert_eq!(meta.grammar(), p.grammar());
    let CssPropertyKindRef::Longhand(longhand) = meta.kind() else {
        panic!("one intrinsic longhand")
    };
    assert_eq!(longhand.property().known_property(), p);
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("fixed symbolic initial, no user-agent requirement")
    };
    assert_eq!(initial.property().known_property(), p);
    // Independent None/number-zero/LP-zero payload assertions are functional
    // new-variant tests with implementation, not inferred by comparing production.
}

fn ordinary(name: &str, value: &str) {
    for front in 0..5 {
        let source = declaration(name, value, front);
        let before = source.clone();
        let items = completed(&source);
        member(&items, &source, None);
        assert_eq!(
            items.items()[0]
                .ordinary_value()
                .unwrap()
                .property()
                .known_property(),
            property(name)
        );
        assert_eq!(source, before);
    }
}

fn globals(name: &str) {
    for (input, keyword) in GLOBALS {
        for front in 0..5 {
            let source = declaration(name, input, front);
            let items = completed(&source);
            member(&items, &source, None);
            assert_eq!(
                items.items()[0].value(),
                CssContributionValueRef::Global(keyword)
            );
            assert!(items.items()[0].ordinary_value().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {input} !important;")
            );
        }
    }
}

fn pending(name: &str, valid: &str, invalid: &str) {
    let p = property(name);
    for authored in [
        "bogus var(--shape)",
        "bogus env(shape)",
        "bogus attr(data-shape)",
    ] {
        for front in 0..5 {
            let source = declaration(name, authored, front);
            let original = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending source")
            };
            assert!(handle.source().same_occurrence(&source));
            for _ in 0..2 {
                for bad in [invalid, "inherit bogus", "bogus"] {
                    let components = parse_component_values(bad).unwrap();
                    let direct = checked(p, components.clone(), false).unwrap_err();
                    let error = handle.reenter(components).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("typed strict replacement error")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                for residual in [
                    "bogus var(--again)",
                    "[f(env(shape))]",
                    "f(attr(data-shape))",
                    r"f(v\61 r(--again))",
                ] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution
                    );
                }
                let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
                let before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one resolved terminal")
                };
                member(&values, &source, Some(&replacement));
                assert!(values.items()[0].ordinary_value().is_some());
                assert_eq!(replacement, before);
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("one global terminal")
                    };
                    member(&values, &source, Some(&replacement));
                    assert_eq!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(keyword)
                    );
                }
            }
            assert_eq!(source, original);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {authored} !important;")
            );
        }
    }
}

macro_rules! lifecycle {
    ($module:ident, $name:literal, $good:literal, $bad:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn selected_metadata_is_noninherited_fixed_and_terminal() {
                metadata($name);
            }
            #[test]
            fn ordinary_expansion_keeps_occurrence_and_grammar() {
                ordinary($name, $good);
            }
            #[test]
            fn every_css_wide_keyword_remains_symbolic() {
                globals($name);
            }
            #[test]
            fn failed_and_residual_reentry_is_atomic_reusable_and_keeps_replacement_origin() {
                pending($name, $good, $bad);
            }
        }
    };
}
lifecycle!(
    outside,
    "shape-outside",
    "circle(25% at inline-end block-start) margin-box",
    "circle() view-box"
);
lifecycle!(threshold, "shape-image-threshold", "150%", "1px");
lifecycle!(margin, "shape-margin", "calc(1px + 2%)", "-1e-999px");

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    let output = components.serialize().unwrap();
    (0..output.as_css().len())
        .find_map(|offset| match output.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original fixture has implicit closure")
}

fn strict_original(name: &str, ordinary: &str, unclosed: &str) {
    let p = property(name);
    let inputs = [
        format!("/*😀*/{unclosed}"),
        format!("{ordinary}/*unfinished"),
        "inherit/*unfinished".into(),
        "var(--shape".into(),
        "env(shape".into(),
        "attr(data-shape".into(),
        "var(--shape)/*unfinished".into(),
    ];
    for input in inputs {
        let components = parse_component_values(&input).unwrap();
        let original = components.clone();
        let origin = implicit_origin(&components);
        for grammar in [false, true] {
            let error = checked(p, components.clone(), grammar).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ),
                "{input}: {error:?}"
            );
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, original);
    }
}

#[test]
fn outside_checked_original_shape_and_pending_closures_cannot_be_repaired() {
    strict_original("shape-outside", "circle()", "circle(");
}
#[test]
fn threshold_checked_original_math_and_pending_closures_cannot_be_repaired() {
    strict_original("shape-image-threshold", "150%", "calc(1 + 25%");
}
#[test]
fn margin_checked_original_math_and_pending_closures_cannot_be_repaired() {
    strict_original("shape-margin", "1px", "calc(1px + 2%");
}

#[test]
fn pending_reentry_checks_original_closure_before_repaired_grammar_but_after_residuals() {
    for (name, valid, open) in [
        ("shape-outside", "circle() margin-box", "circle("),
        ("shape-image-threshold", "150%", "calc(25% + 25%"),
        ("shape-margin", "1px", "calc(1px + 2%"),
    ] {
        let source = declaration(name, "var(--shape)", 0);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(open).unwrap();
        let origin = implicit_origin(&replacement);
        let error = handle.reenter(replacement).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("closure failure")
        };
        assert!(matches!(
            actual.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ));
        assert_eq!(actual.origin(), &CssSerializedOrigin::End(Some(origin)));
        assert_eq!(
            handle
                .reenter(parse_component_values("future(var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(
            handle
                .reenter(parse_component_values(valid).unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
}

#[test]
fn normalized_shape_occurrences_keep_rule_context_order_importance_and_one_member_each() {
    let input = ".a{shape-outside:circle()!important;shape-margin:var(--gap);shape-image-threshold:150%;shape-outside:unset;shape-margin:calc(1px + 2%);shape-image-threshold:initial!important}";
    let report = parse_sheet(input);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style rule")
    };
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 6, 6).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (order, (item, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                let [value] = values.items() else {
                    panic!("one Shapes terminal")
                };
                assert!(value.source().same_occurrence(source));
                assert_eq!(value.property(), source.known().unwrap().property());
            }
            CssExpansion::Pending(handle) if order == 1 => {
                assert!(handle.source().same_occurrence(source))
            }
            other => panic!("expected one Shapes occurrence, got {other:?}"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(0, 1, 5, 6).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 6, 5).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit: 5 }
        );
        assert_eq!(error.declaration_order(), Some(5));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(&style.declarations()[5])
        );
        assert!(error.rule_context().is_some());
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn all_reset_includes_each_shapes_longhand_without_evaluating_initials() {
    let source = declaration("all", "initial", 0);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("symbolic all")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property(name))));
    }
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
}

#[test]
fn specified_scalar_and_keyword_limits_fail_atomically_and_retry_with_full_budget() {
    for (name, input, value) in [
        ("shape-outside", "none", "none"),
        ("shape-image-threshold", "150%", "1.5"),
        ("shape-margin", "1px", "1px"),
    ] {
        let source = declaration(name, input, 0);
        let original = source.clone();
        let expected = format!("{name}: {value} !important;");
        // One scalar/keyword plus the independently defined two declaration nodes.
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(3, 3, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(2, 3, expected.len()), K::InputNodeLimit),
            (L::new(3, 2, expected.len()), K::ProjectionNodeLimit),
            (L::new(3, 3, expected.len() - 1), K::ByteLimit),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, original);
            }
        }
        assert_eq!(source.to_specified_css().unwrap(), expected);
    }
}

#[test]
fn composed_shape_numeric_children_and_pending_output_share_bounded_atomic_writers() {
    for (name, input, expected) in [
        (
            "shape-outside",
            "ellipse(abs(-2em) hypot(3em,4em) at 1px 2%) border-box",
            "ellipse(calc(2em) calc(5em) at 1px 2%) border-box",
        ),
        ("shape-margin", "calc(1px + 2%)", "calc(2% + 1px)"),
        (
            "shape-image-threshold",
            "bogus var(--shape)",
            "bogus var(--shape)",
        ),
    ] {
        let source = declaration(name, input, 0);
        let original = source.clone();
        let output = format!("{name}: {expected} !important;");
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(65_536, 262_144, output.len()))
                .unwrap(),
            output
        );
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(65_536, 262_144, output.len() - 1))
                .unwrap_err()
                .kind(),
            K::ByteLimit
        );
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(0, 262_144, output.len()))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(65_536, 0, output.len()))
                .unwrap_err()
                .kind(),
            K::ProjectionNodeLimit
        );
        assert_eq!(source, original);
        assert_eq!(source.to_specified_css().unwrap(), output);
    }
}

#[test]
fn accepted_clip_shape_provider_controls_use_shared_projection_and_keep_exact_payloads() {
    accept(
        "clip-path",
        &[
            ("circle(calc(1in + 1in))", "circle(calc(192px))"),
            (
                "path(evenodd,'M01.00,0 L1e2 -0 Z')",
                "path(evenodd, \"M01.00,0 L1e2 -0 Z\")",
            ),
            (
                "circle(at inline-end block-start)",
                "circle(at block-start inline-end)",
            ),
            ("ellipse(10% closest-side)", "ellipse(10% closest-side)"),
        ],
    );
}

#[test]
fn accepted_opacity_and_nonnegative_lp_provider_controls_keep_authored_range_phase() {
    accept(
        "opacity",
        &[
            ("150%", "1.5"),
            ("-25%", "-0.25"),
            ("calc(2 - 3)", "calc(-1)"),
        ],
    );
    accept(
        "padding-left",
        &[
            ("calc(1px - 2px)", "calc(-1px)"),
            ("calc(1px + 2%)", "calc(2% + 1px)"),
        ],
    );
    invalid("padding-left", &["-1e-999px", "-1e-999%"]);
}

#[test]
fn accepted_image_provider_control_keeps_image_lists_separate_from_box_grammar() {
    accept(
        "background-image",
        &[
            ("linear-gradient(red,blue)", "linear-gradient(red, blue)"),
            ("url(\"asset.png\")", "url(\"asset.png\")"),
        ],
    );
}

#[test]
fn outside_shape_box_and_numeric_siblings_have_one_cumulative_visit_budget() {
    let input = "ellipse(abs(-2em) hypot(3em,4em) at 1px 2%) border-box";
    let output = "shape-outside: ellipse(calc(2em) calc(5em) at 1px 2%) border-box !important;";
    // Existing ellipse/radius pair: two nodes; abs: two; hypot: three;
    // position/axes: three; literal offsets: two = twelve. Shape-plus-box
    // composition adds one aggregate and one explicit box; declaration adds two.
    for front in [0, 1, 2] {
        let source = declaration("shape-outside", input, front);
        let original = source.clone();
        assert_eq!(
            source
                .to_specified_css_with_limits(L::new(16, 16, output.len()))
                .unwrap(),
            output
        );
        for (limits, kind) in [
            (L::new(15, 16, output.len()), K::InputNodeLimit),
            (L::new(16, 15, output.len()), K::ProjectionNodeLimit),
            (L::new(16, 16, output.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(source, original);
        }
        assert_eq!(source.to_specified_css().unwrap(), output);
    }
}
