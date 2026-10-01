#![forbid(unsafe_code)]
//! Authored rectangles from Shapes 1 CRD 2025-06-12 §§3.1–3.3:
//! https://www.w3.org/TR/2025/CRD-css-shapes-1-20250612/#supported-basic-shapes
//! Token separation follows Values 4 WD 2024-03-12 §2.5. Expectations retain
//! authored coordinate order, auto, round omission and symbolic calculations;
//! computed inset conversion and used rectangle geometry are downstream.
//! This suite uses existing callable APIs; new named model construction and
//! cumulative resource limits receive separate functional tests.

use surgeist_css::*;

fn clip(source: &CssDeclaration) -> &CssClipPath {
    let CssKnownPropertyValueRef::ClipPath(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed clip-path")
    };
    value.value()
}

fn declaration(value: &str) -> CssDeclaration {
    let source = format!("clip-path:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one authored declaration")
    };
    let CssKnownPropertyValueRef::ClipPath(parsed) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("clip-path property")
    };
    assert_eq!(parsed.as_css(), value, "retain authored function and order");
    declaration.clone()
}

fn accept(authored: &str, expected: &str) {
    let source = declaration(authored);
    assert_eq!(clip(&source).serialize_specified().unwrap(), expected);
    let components = parse_component_values(authored).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{authored}: {error:?}"));
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(clip(&checked).serialize_specified().unwrap(), expected);
    for declaration in [&source, &checked] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            expand_declaration(declaration).unwrap()
        else {
            panic!("ordinary rectangle terminal")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::ClipPath);
        assert!(item.source().same_occurrence(declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
        let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed rectangle terminal")
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn rect_retains_four_distinct_signed_edges_in_top_right_bottom_left_order() {
    for text in [
        "rect(1px 2% 3em 4px)",
        "rect(-1px -2% -3em -4px)",
        "rect(10px 0 0 20px)",
    ] {
        accept(text, text);
    }
    accept("rect(-1e-999px 2% 0 4px)", "rect(0px 2% 0 4px)");
}

#[test]
fn rect_retains_auto_independently_in_each_edge_without_inferred_defaults() {
    for text in [
        "rect(auto 2px 3% 4px)",
        "rect(1px auto 3% 4px)",
        "rect(1px 2% auto 4px)",
        "rect(1px 2% 3px auto)",
        "rect(auto auto auto auto)",
    ] {
        accept(text, text);
    }
}

#[test]
fn xywh_retains_signed_offsets_and_nonnegative_dimensions_in_grammar_order() {
    accept("xywh(-1px -2% 3em 4px)", "xywh(-1px -2% 3em 4px)");
    accept("xywh(-1e-999px -2% 0 0)", "xywh(0px -2% 0 0)");
    accept("xywh(1px 2% 3e-999px 4%)", "xywh(1px 2% 0px 4%)");
}

#[test]
fn rectangular_calculations_stay_symbolic_without_size_evaluation() {
    for text in [
        "rect(calc(10% - 2px) auto min(-1px, 2%) -3px)",
        "xywh(calc(10% - 2px) -3px calc(-1px) min(-1px, 2%))",
    ] {
        accept(text, text);
    }
}

#[test]
fn absent_round_and_explicit_round_zero_remain_distinct() {
    for text in [
        "rect(1px 2% 3px 4%)",
        "rect(1px 2% 3px 4% round 0)",
        "xywh(1px 2% 3px 4%)",
        "xywh(1px 2% 3px 4% round 0)",
    ] {
        accept(text, text);
    }
}

#[test]
fn rectangular_round_retains_radius_arity_and_explicit_vertical_slash() {
    for function in ["rect", "xywh"] {
        for radius in [
            "1px",
            "1px 2%",
            "1px 2% 3px",
            "1px 2% 3px 4%",
            "1px / 2%",
            "1px 2px / 3% 4%",
            "1px 2px 3px / 4% 5% 6%",
            "1px 2px 3px 4px / 5% 6% 7% 8%",
            "calc(10% - 2px) / 0",
        ] {
            let text = format!("{function}(1px 2% 3px 4% round {radius})");
            accept(&text, &text);
        }
    }
}

#[test]
fn both_reference_box_orders_serialize_shape_before_the_explicit_box() {
    for function in ["rect", "xywh"] {
        for reference_box in [
            "content-box",
            "padding-box",
            "border-box",
            "margin-box",
            "fill-box",
            "stroke-box",
            "view-box",
        ] {
            let shape = format!("{function}(1px 2% 3px 4% round 0)");
            let expected = format!("{shape} {reference_box}");
            accept(&expected, &expected);
            accept(&format!("{reference_box} {shape}"), &expected);
        }
    }
}

#[test]
fn distinguishable_adjacent_percentages_and_comment_only_separators_are_valid() {
    for function in ["rect", "xywh"] {
        accept(
            &format!("{function}(1%2%3%4%)"),
            &format!("{function}(1% 2% 3% 4%)"),
        );
        accept(
            &format!("{function}(1px/**/2%/**/3px/**/4%)"),
            &format!("{function}(1px 2% 3px 4%)"),
        );
        accept(
            &format!("{function}(1%2%3%4%/**/round/**/1px/**/2%/3px)"),
            &format!("{function}(1% 2% 3% 4% round 1px 2% / 3px)"),
        );
    }
}

#[test]
fn escaped_and_case_varied_function_names_keep_authored_text_and_canonical_output() {
    accept(
        "ReCt(AUTO 2px 3% 4px RoUnD 0)",
        "rect(auto 2px 3% 4px round 0)",
    );
    accept("r\\65 ct(1px 2% 3px 4%)", "rect(1px 2% 3px 4%)");
    accept("XYWH(1px 2% 3px 4%)", "xywh(1px 2% 3px 4%)");
    accept("x\\79 wh(1px 2% 3px 4%)", "xywh(1px 2% 3px 4%)");
}

fn reject(value: &str) {
    let prefix = "/*😀*/ ";
    let unit = format!("clip-path:{value};");
    let source = format!("{prefix}{unit} color:blue");
    let report = parse_style_attribute(&source);
    let [sibling] = report.syntax().as_slice() else {
        panic!("only sibling survives {source}: {:?}", report.syntax())
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let [diagnostic] = report.diagnostics() else {
        panic!("one diagnostic for {source}")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let end = prefix.len() + unit.len();
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        prefix.encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().end().column().value() as usize,
        source[..end].encode_utf16().count()
    );
    let position = diagnostic.error().position();
    let byte = position.byte_offset().value();
    assert!(byte >= prefix.len() + "clip-path:".len() && byte < end);
    assert_eq!(position.line().value(), 0);
    assert_eq!(
        position.column().value() as usize,
        source[..byte].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ClipPath),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err()
    );
}

#[test]
fn malformed_coordinate_arity_separators_and_domains_recover_once() {
    for function in ["rect", "xywh"] {
        for coordinates in [
            "",
            "1px",
            "1px 2px",
            "1px 2px 3px",
            "1px 2px 3px 4px 5px",
            "1px, 2px, 3px, 4px",
            "1px 2px 3px 4px,",
            "1px 2px / 3px 4px",
            "1em2em 3px 4px",
            "1 2px 3px 4px",
            "1px 2deg 3px 4px",
            "1px 2px 3s 4px",
            "inherit 2px 3px 4px",
            "1px 2px 3px initial",
            "1px 2px 3px 4px trailing",
            "round 0 1px 2px 3px 4px",
        ] {
            reject(&format!("{function}({coordinates})"));
        }
    }
    for value in [
        "xywh(auto 0 1px 2px)",
        "xywh(0 auto 1px 2px)",
        "xywh(0 0 auto 2px)",
        "xywh(0 0 1px auto)",
        "xywh(0 0 -1px 2px)",
        "xywh(0 0 1px -2%)",
        "xywh(0 0 -1e-999px 2px)",
        "xywh(0 0 1px -1e-999%)",
        "rect(1px 2px 3px 4px) border-box padding-box",
        "xywh(1px 2px 3px 4px) rect(1px 2px 3px 4px)",
        "none rect(1px 2px 3px 4px)",
        "url('#clip') xywh(1px 2px 3px 4px)",
    ] {
        reject(value);
    }
}

#[test]
fn malformed_round_arity_slash_range_and_placement_recover_once() {
    for function in ["rect", "xywh"] {
        for radius in [
            "",
            "1px 2px 3px 4px 5px",
            "/ 1px",
            "1px /",
            "1px / 2px / 3px",
            "1px / 2px 3px 4px 5px 6px",
            "1px, 2px",
            "-1px",
            "-1e-999px",
            "1px / -2%",
            "1deg",
            "1",
            "0 round 0",
        ] {
            reject(&format!("{function}(1px 2px 3px 4px round {radius})"));
        }
        reject(&format!("{function}(1px 2px round 0 3px 4px)"));
    }
}

#[test]
fn raw_rect_function_origins_distinguish_opener_closer_and_coordinate_tokens() {
    let source = "/*😀*/ clip-path:ReCt(auto -1e-999px calc(10% - 2px) 4% round 0)!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let component = &report.syntax()[0].value_components().items()[0];
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("shape function")
    };
    assert_origin(
        component.origin(),
        source,
        "ReCt(",
        source.find("ReCt(").unwrap(),
    );
    assert_origin(
        function.closing_origin(),
        source,
        ")",
        source.rfind(')').unwrap(),
    );
    for (text, kind) in [
        ("auto", "ident"),
        ("-1e-999px", "dimension"),
        ("4%", "percentage"),
    ] {
        let component = function
            .values()
            .items()
            .iter()
            .find(|item| match item.view() {
                CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) => {
                    kind == "ident" && value == "auto"
                }
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. }) => {
                    kind == "dimension"
                }
                CssComponentValueRef::Token(CssValueTokenRef::Percentage(_)) => {
                    kind == "percentage"
                }
                _ => false,
            })
            .unwrap();
        assert_origin(component.origin(), source, text, source.find(text).unwrap());
    }
    let math = function
        .values()
        .items()
        .iter()
        .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
        .unwrap();
    assert_origin(
        math.origin(),
        source,
        "calc(",
        source.find("calc(").unwrap(),
    );
    let CssComponentValueRef::Function(math) = math.view() else {
        panic!("math function")
    };
    assert_origin(
        math.closing_origin(),
        source,
        ")",
        source.find("2px)").unwrap() + 3,
    );
}

fn assert_origin(origin: &CssValueOrigin, source: &str, text: &str, start: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(
        origin.span().end().byte_offset().value(),
        start + text.len()
    );
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        origin.span().end().column().value() as usize,
        source[..start + text.len()].encode_utf16().count()
    );
}

#[test]
fn rectangle_pending_reentry_checks_full_grammar_and_preserves_replacement_origins() {
    let report = parse_style_attribute("/*😀*/ clip-path:var(--rectangle)!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("pending rectangle")
    };
    for invalid in [
        "rect(1px 2px 3px)",
        "rect(1px, 2px, 3px, 4px)",
        "xywh(0 0 -1e-999px 2px)",
        "xywh(0 0 1px 2px round / 0)",
    ] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("rect(var(--again) 2px 3px 4px)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for (authored, expected) in [
        (
            "rect(auto -2px 3% 4px round 0)",
            "rect(auto -2px 3% 4px round 0)",
        ),
        (
            "border-box xywh(-1px 2% 3px 4px round 1px / 2%)",
            "xywh(-1px 2% 3px 4px round 1px / 2%) border-box",
        ),
        ("xywh(1%2%3%4%)", "xywh(1% 2% 3% 4%)"),
    ] {
        let replacement = parse_component_values(authored).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("rectangle terminal")
            };
            let [item] = items.items() else {
                panic!("one rectangle terminal")
            };
            assert_eq!(item.property(), CssKnownProperty::ClipPath);
            assert!(item.source().same_occurrence(source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(
                item.replacement_components().unwrap().items()[0].origin(),
                replacement.items()[0].origin()
            );
            let CssLonghandValueRef::ClipPath(value) = item.ordinary_value().unwrap().view() else {
                panic!("clip-path replacement")
            };
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
    }
    let replacement = parse_component_values("inherit").unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("global terminal")
    };
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
    assert!(items.items()[0].source().same_occurrence(source));
    assert_eq!(
        items.items()[0].replacement_components(),
        Some(&replacement)
    );
    assert!(pending.source().same_occurrence(source));
}

#[test]
fn normalization_preserves_rectangle_family_importance_and_surviving_order() {
    let source = ".a{clip-path:rect(auto -2px 3% 4px)!important;clip-path:xywh(0 0 -1px 2px);clip-path:border-box xywh(-1px 2% 3px 4px round 0);clip-path:inherit;clip-path:var(--rectangle)}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (order, (item, authored)) in declarations
        .iter()
        .zip([
            "rect(auto -2px 3% 4px)",
            "border-box xywh(-1px 2% 3px 4px round 0)",
            "inherit",
            "var(--rectangle)",
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert_eq!(
            item.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            source.find(authored).unwrap() - "clip-path:".len()
        );
        if order == 3 {
            let CssExpansion::Pending(pending) = item.expansion() else {
                panic!("pending terminal")
            };
            assert!(pending.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(items)) = item.expansion()
            else {
                panic!("completed terminal")
            };
            let [value] = items.items() else {
                panic!("one terminal")
            };
            assert!(value.source().same_occurrence(item.source()));
            assert!(value.replacement_components().is_none());
            if order == 2 {
                assert_eq!(
                    value.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            } else {
                let CssLonghandValueRef::ClipPath(value) = value.ordinary_value().unwrap().view()
                else {
                    panic!("rectangle")
                };
                assert_eq!(
                    value.serialize_specified().unwrap(),
                    if order == 0 {
                        "rect(auto -2px 3% 4px)"
                    } else {
                        "xywh(-1px 2% 3px 4px round 0) border-box"
                    }
                );
            }
        }
    }
}
