#![forbid(unsafe_code)]
//! Authored transform admission, lifecycle, provenance and resource contracts.
//! Normative basis: selected Transforms1 §§4–6/9, Transforms2 §§5/7–10/12,
//! Values3 §6.1, Values4 §§8.3/10.13; #564 preserves the WebKit tie-break.
//! These are specialized behavior stimuli, not a second metadata/catalog table.
//! Update common/property_expectations/records.rs for all generic facts instead.

#[path = "common/authored_property.rs"]
mod authored_property;
use CssSpecifiedValueSerializationErrorKind as K;
use CssSpecifiedValueSerializationLimits as L;
use authored_property::{ParserFront, assert_source, checked, checked_components, invalid};
use surgeist_css::*;

const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("selected property {name} must be recognized"))
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one intrinsic longhand")
    };
    let [item] = values.items() else {
        panic!("one longhand; no invented axis subproperties")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert_source(item, source);
    values
}
fn admission(name: &str, input: &str, expected: &str) {
    let p = property(name);
    for front in FRONTS {
        let source = front.valid(p, input, expected);
        // Re-enter emitted syntax, retaining the original value independently.
        let again = front.valid(p, expected, expected);
        assert_eq!(source.importance(), again.importance());
        assert_eq!(
            source.known().unwrap().grammar(),
            again.known().unwrap().grammar()
        );
    }
}

#[test]
fn selected_3d_properties_admit_keywords_positions_and_unfloored_nonnegative_lengths() {
    // Actual browser declaration admission precedes any absent-variant lookup.
    for (name, input, output) in [
        ("transform-style", "PRESERVE-3D", "preserve-3d"),
        ("transform-style", "flat", "flat"),
        ("backface-visibility", "HIDDEN", "hidden"),
        ("backface-visibility", "visible", "visible"),
        ("perspective", "none", "none"),
        ("perspective", "0", "0"),
        ("perspective", ".25px", "0.25px"),
        ("perspective", "calc(1px - 2px)", "calc(-1px)"),
        ("perspective-origin", "top left", "left top"),
        (
            "perspective-origin",
            "right 2px bottom 3%",
            "right 2px bottom 3%",
        ),
        (
            "perspective-origin",
            "calc(1px + 2em) 30%",
            "calc(2em + 1px) 30%",
        ),
    ] {
        let text = format!("color:red;{name}:{input};color:blue");
        let report = parse_style_attribute(&text);
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 3);
        admission(name, input, output);
    }
}
#[test]
fn percentage_scale_and_typed_math_extend_property_and_2d_function_domains() {
    for (name, input, output) in [
        ("scale", "50%", "0.5"),
        ("scale", "50% .5 100%", "0.5"),
        ("scale", "50% 150% 200%", "0.5 1.5 2"),
        ("scale", "calc(1 + 2)", "calc(3)"),
        ("scale", "calc(50% + 50%)", "calc(1)"),
        ("transform", "scale(50%,150%)", "scale(0.5, 1.5)"),
        (
            "transform",
            "scaleX(25%) scaleY(125%)",
            "scaleX(0.25) scaleY(1.25)",
        ),
        ("transform", "scale(calc(50% + 50%))", "scale(calc(1))"),
        ("transform", "scaleX(calc(25% * 2))", "scaleX(calc(0.5))"),
        (
            "transform",
            "scaleY(calc((1px + 1%) / 1px))",
            "scaleY(calc((1% + 1px) / 1px))",
        ),
    ] {
        admission(name, input, output);
    }
}
#[test]
fn represented_function_controls_cover_all_twenty_one_alternatives_and_canonical_arity() {
    for (input, output) in [
        ("MATRIX(1,0,0,1,2,3)", "matrix(1, 0, 0, 1, 2, 3)"),
        (
            "matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,2,3,4,1)",
            "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 2, 3, 4, 1)",
        ),
        ("perspective(.25px)", "perspective(0.25px)"),
        ("rotate(0)", "rotate(0)"),
        ("rotate3d(1,2,3,30deg)", "rotate3d(1, 2, 3, 30deg)"),
        ("rotateX(.5turn)", "rotateX(0.5turn)"),
        ("rotateY(100grad)", "rotateY(100grad)"),
        ("rotateZ(0)", "rotateZ(0)"),
        ("scale(2,2)", "scale(2)"),
        ("scale3d(50%,100%,150%)", "scale3d(0.5, 1, 1.5)"),
        ("scaleX(-2)", "scaleX(-2)"),
        ("scaleY(3)", "scaleY(3)"),
        ("scaleZ(150%)", "scaleZ(1.5)"),
        ("skew(10deg,0deg)", "skew(10deg)"),
        ("skewX(0)", "skewX(0)"),
        ("skewY(20deg)", "skewY(20deg)"),
        ("translate(10%,0%)", "translate(10%)"),
        ("translate3d(1%,2%,3px)", "translate3d(1%, 2%, 3px)"),
        ("translateX(-2px)", "translateX(-2px)"),
        ("translateY(3%)", "translateY(3%)"),
        ("translateZ(4px)", "translateZ(4px)"),
    ] {
        admission("transform", input, output);
    }
    // The specified projection keeps skew as one operation, in list order.
    admission(
        "transform",
        "skew(10deg,20deg) skewX(10deg) skewY(20deg)",
        "skew(10deg, 20deg) skewX(10deg) skewY(20deg)",
    );
}
#[test]
fn wrong_arities_separators_and_numeric_domains_reject_the_whole_function_declaration() {
    for text in [
        "matrix(1,0,0,1,2)",
        "matrix(1,0,0,1,2,3,4)",
        "matrix(1 0 0 1 2 3)",
        "matrix3d(1,0,0)",
        "perspective(-1e-999px)",
        "perspective(1%)",
        "perspective(1px,2px)",
        "rotate(1)",
        "rotate(0,0)",
        "rotate3d(1,0,30deg)",
        "rotate3d(1,0,0,30%)",
        "rotateX()",
        "rotateY(1px)",
        "rotateZ(1deg,2deg)",
        "scale()",
        "scale(1,2,3)",
        "scale3d(1,2)",
        "scaleX(1px)",
        "scaleY(1,2)",
        "scaleZ(1deg)",
        "skew()",
        "skew(1deg,2deg,3deg)",
        "skewX(1)",
        "skewY(2px)",
        "translate()",
        "translate(1px 2px)",
        "translate3d(1px,2px,3%)",
        "translateX(1deg)",
        "translateY(1px,2px)",
        "translateZ(2%)",
        "none rotate(0)",
        "rotate(0),scale(1)",
    ] {
        invalid(CssKnownProperty::Transform, text);
    }
    for (name, value) in [
        ("translate", "1px 2px 3%"),
        ("rotate", "0"),
        ("rotate", "1 2 30deg 3"),
        ("scale", "1 2 3 4"),
        ("scale", "calc(.5 + 50%)"),
        ("transform-box", "padding-box"),
        ("transform-box", "view-box fill-box"),
        ("perspective", "-1e-999px"),
        ("perspective", "25%"),
        ("perspective-origin", "left 2px top"),
        ("perspective-origin", "left top 1px"),
        ("transform-style", "preserve-3d flat"),
        ("backface-visibility", "auto"),
    ] {
        invalid(property(name), value);
    }
}
#[test]
fn selected_origin_tie_break_keeps_valid_planar_pairs_and_requires_pair_before_z() {
    for value in [
        "top 50px",
        "bottom 50px",
        "top calc(1px * 2)",
        "bottom 0",
        "left top 50%",
        "left 2px top 3px",
    ] {
        invalid(CssKnownProperty::TransformOrigin, value);
    }
    for (input, output) in [
        ("top", "center top"),
        ("bottom left", "left bottom"),
        ("left 50px", "left 50px"),
        ("top left 50px", "left top 50px"),
        ("20% 30% -2px", "20% 30% -2px"),
    ] {
        admission("transform-origin", input, output);
    }
}

// This specialized lifecycle list supplies ordinary/reentry stimuli only.
// Initial values, inheritance, support/source IDs and wrapper expectations live
// in the existing independent common records and their real consumers.
const LIFECYCLES: [(&str, &str, &str); 10] = [
    (
        "transform",
        "translateX(1px) scale(2)",
        "translateX(1px) scale(2)",
    ),
    ("transform-box", "view-box", "view-box"),
    ("transform-origin", "top left 2px", "left top 2px"),
    ("translate", "10% 0px 0px", "10%"),
    ("rotate", "30deg x", "x 30deg"),
    ("scale", "50% 50% 100%", "0.5"),
    ("transform-style", "preserve-3d", "preserve-3d"),
    ("perspective", ".25px", "0.25px"),
    ("perspective-origin", "top left", "left top"),
    ("backface-visibility", "hidden", "hidden"),
];
#[test]
fn ordinary_and_all_css_wide_occurrences_expand_without_axis_shorthands_or_resets() {
    for (name, input, output) in LIFECYCLES {
        let p = property(name);
        for front in FRONTS {
            let source = front.valid(p, input, output);
            let values = completed(&source);
            assert!(matches!(
                values.items()[0].value(),
                CssContributionValueRef::Ordinary(_)
            ));
            assert!(values.items()[0].replacement_components().is_none());
            for (text, keyword) in GLOBALS {
                let source = front.valid(p, text, text);
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                let values = completed(&source);
                assert!(
                    matches!(values.items()[0].value(), CssContributionValueRef::Global(k) if k == keyword)
                );
            }
        }
    }
}
#[test]
fn all_pending_forms_reenter_strictly_and_reuse_original_identity_and_replacement_origins() {
    for (name, input, output) in LIFECYCLES {
        let p = property(name);
        for front in FRONTS {
            for pending in ["var(--t)", "env(t)", "attr(data-t *)"] {
                let source = front.valid(p, pending, pending);
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("supported pending transform")
                };
                assert!(handle.source().same_occurrence(&source));
                for invalid in ["var(--again)", "env(again)", "attr(data-again *)"] {
                    assert!(matches!(
                        handle
                            .reenter(parse_component_values(invalid).unwrap())
                            .unwrap_err()
                            .kind(),
                        CssExpansionErrorKind::ResidualSubstitution
                    ));
                }
                for invalid in [
                    "__invalid__",
                    "initial trailing",
                    "initial!important",
                    "initial;color:red",
                ] {
                    for _ in 0..2 {
                        assert!(matches!(
                            handle
                                .reenter(parse_component_values(invalid).unwrap())
                                .unwrap_err()
                                .kind(),
                            CssExpansionErrorKind::InvalidReplacement(_)
                        ));
                    }
                }
                for (text, expected_global) in
                    std::iter::once((input, None)).chain(GLOBALS.map(|(text, k)| (text, Some(k))))
                {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("terminal replacement")
                    };
                    let [item] = values.items() else {
                        panic!("single longhand")
                    };
                    assert_source(item, &source);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    for (actual, original) in item
                        .replacement_components()
                        .unwrap()
                        .items()
                        .iter()
                        .zip(replacement.items())
                    {
                        assert_eq!(actual.origin(), original.origin());
                        let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
                            (actual.origin(), original.origin())
                        else {
                            panic!("replacement has original parsed provenance")
                        };
                        assert!(a.source().same_snapshot(b.source()));
                    }
                    if let Some(k) = expected_global {
                        assert!(
                            matches!(item.value(), CssContributionValueRef::Global(v) if v == k)
                        );
                    } else {
                        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    }
                }
                admission(name, input, output);
                assert_eq!(source, before);
            }
        }
    }
}
fn implicit_origin(values: &CssComponentValues) -> CssValueOrigin {
    let serialized = values.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(v @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(v.clone())
            }
            _ => None,
        })
        .or_else(|| {
            values.items().iter().find_map(|value| match value.view() {
                CssComponentValueRef::Function(v) => match v.closing_origin() {
                    v @ CssValueOrigin::ImplicitClosure { .. } => Some(v.clone()),
                    _ => None,
                },
                _ => None,
            })
        })
        .expect("original implicit closure")
}
fn closed_error(error: &CssPropertyValueParseError, origin: &CssValueOrigin, text: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit closure")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
}
#[test]
fn checked_original_closures_reject_ordinary_global_and_pending_values_at_original_eof() {
    // No metadata dependency: the six represented-property cases can execute at RED.
    for (name, value, _) in LIFECYCLES {
        let p = property(name);
        // Existing Scale literal isolates closure rejection from its new domain.
        let value = if name == "scale" { "2 2 1" } else { value };
        for text in std::iter::once(format!("{value}/*"))
            .chain(GLOBALS.map(|(v, _)| format!("{v}/*")))
            .chain(
                [
                    "var(--t)/*",
                    "env(t)/*",
                    "attr(data-t *)/*",
                    "var(--t",
                    "env(t",
                    "attr(data-t *",
                ]
                .map(str::to_owned),
            )
        {
            let values = parse_component_values(&text).unwrap();
            let before = values.clone();
            let origin = implicit_origin(&values);
            for grammar in [false, true] {
                let error =
                    checked_components(p, values.clone(), grammar, CssImportance::Important)
                        .unwrap_err();
                closed_error(&error, &origin, &text);
            }
            assert_eq!(values, before);
        }
        checked(p, &format!("{value}/**/"), false);
    }
}
#[test]
fn pending_reentry_closures_reject_without_consuming_the_handle_and_complete_comments_succeed() {
    for (name, value, _) in LIFECYCLES {
        let p = property(name);
        let value = if name == "scale" { "2 2 1" } else { value };
        let source = checked(p, "var(--t)", false);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in [format!("{value}/*"), "initial/*".to_owned()] {
            let replacement = parse_component_values(&text).unwrap();
            let origin = implicit_origin(&replacement);
            for _ in 0..2 {
                let error = handle.reenter(replacement.clone()).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("strict closure")
                };
                closed_error(error, &origin, &text);
            }
        }
        assert!(
            handle
                .reenter(parse_component_values(&format!("{value}/**/")).unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
}
#[test]
fn normalized_composed_transform_families_preserve_occurrences_order_importance_and_pending_identity()
 {
    let report = parse_sheet(
        ".a{transform:translateX(10%);rotate:x 30deg!important;scale:50%;perspective:.25px;transform:var(--t)}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 5);
    for (index, name) in ["transform", "rotate", "scale", "perspective", "transform"]
        .iter()
        .enumerate()
    {
        assert_eq!(items[index].order(), index);
        assert_eq!(
            items[index].source().known().unwrap().property(),
            property(name)
        );
        assert_eq!(
            items[index].source().importance(),
            if index == 1 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        if index < 4 {
            let CssExpansion::Contributions(CssContributions::Longhands(v)) =
                items[index].expansion()
            else {
                panic!("ordinary")
            };
            assert_source(&v.items()[0], items[index].source());
        } else {
            let CssExpansion::Pending(v) = items[index].expansion() else {
                panic!("pending")
            };
            assert!(v.source().same_occurrence(items[index].source()));
        }
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 4).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 4
        }
    );
}
fn budgets(source: &CssDeclaration, output: &str, input: usize, projection: usize) {
    let expected = format!(
        "{}: {output} !important;",
        source.known().unwrap().property().canonical_name()
    );
    let exact = L::new(input, projection, expected.len());
    let before = source.clone();
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        for _ in 0..2 {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(source, &before);
    }
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
#[test]
fn retained_provider_controls_charge_suppressed_defaults_and_specified_math_cumulatively() {
    // Independent accepted tariff: declaration/name 2; transform carrier/list 1;
    // each function 1; each numeric literal 1. Suppressed defaults remain charged.
    budgets(
        &checked(
            CssKnownProperty::Transform,
            "translate(1px,0px) scale(2,2)",
            false,
        ),
        "translate(1px) scale(2)",
        9,
        9,
    );
    budgets(
        &checked(CssKnownProperty::Translate, "10px 0px 0px", false),
        "10px",
        6,
        6,
    );
    budgets(&checked(CssKnownProperty::Scale, "2 2 1", false), "2", 6, 6);
    budgets(
        &checked(CssKnownProperty::TransformBox, "view-box", false),
        "view-box",
        3,
        3,
    );
    // Calc input is 1 wrapper + 1 sum + 2 leaves, projection is 3 numeric-owner
    // nodes; enclosing transform/function 2 and declaration/name 2.
    budgets(
        &checked(
            CssKnownProperty::Transform,
            "translateX(calc(1px + 2px))",
            false,
        ),
        "translateX(calc(3px))",
        8,
        7,
    );
}
#[test]
fn composed_sheets_share_bytes_and_retry_after_late_failure_without_mutation() {
    let report = parse_sheet(
        ".a{transform:scale(50%,150%);perspective:.25px!important}.b{rotate:x 30deg;transform-origin:top left 2px}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { transform: scale(0.5, 1.5); perspective: 0.25px !important; }\n.b { rotate: x 30deg; transform-origin: left top 2px; }";
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
            CssSpecifiedRuleSerializationErrorKind::Resource(K::ByteLimit)
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn checked_nested_transform_values_clone_compare_serialize_and_drop_on_joined_ordinary_stack() {
    // Join before returning so construction, comparison and destruction all
    // complete on the selected ordinary stack.
    let worker = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for depth in [254, 255] {
                // transform function is one structural level, yielding total 255/256.
                let value = format!(
                    "translateX({}1px{})",
                    "calc(".repeat(depth),
                    ")".repeat(depth)
                );
                let components = parse_component_values(&value).unwrap();
                let a = parse_property_value(
                    CssPropertyNameRef::Known(CssKnownProperty::Transform),
                    components.clone(),
                    CssImportance::Normal,
                )
                .unwrap();
                let b = a.clone();
                assert_eq!(a, b);
                assert_eq!(
                    a.to_specified_css().unwrap(),
                    "transform: translateX(calc(1px));"
                );
                drop(b);
                drop(a);
                drop(components);
            }
        })
        .unwrap();
    if let Err(panic) = worker.join() {
        std::panic::resume_unwind(panic);
    }
}
#[test]
fn checked_programmatic_tokens_and_parsed_numeric_children_keep_exact_original_provenance() {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_dimension("25", "px").unwrap()])
            .unwrap();
    for grammar in [false, true] {
        let source = checked_components(
            CssKnownProperty::Translate,
            components.clone(),
            grammar,
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert!(source.parsed_name().is_none());
        assert!(source.parsed_value().is_none());
        assert!(source.position().is_none());
        assert!(matches!(
            source.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        assert_eq!(
            source.to_specified_css().unwrap(),
            "translate: 25px !important;"
        );
    }
    let text = "/*😀*/\r\ntranslate:25% 1e-999px 0px!important";
    let report = parse_style_attribute(text);
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    let before = source.clone();
    let CssKnownPropertyValueRef::Translate(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("represented translate")
    };
    let CssTranslate::Values(values) = wrapper.value() else {
        panic!("axes")
    };
    let y = values.y().unwrap();
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        y.literal_component().unwrap().view()
    else {
        panic!("exact retained length")
    };
    assert_eq!(number.representation(), "1e-999");
    assert_eq!(unit, "px");
    let CssValueOrigin::Parsed(origin) = y.origin() else {
        panic!("parsed operand origin")
    };
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        text.find("1e-999").unwrap()
    );
    assert_eq!(
        source.to_specified_css().unwrap(),
        "translate: 25% 0px !important;"
    );
    assert_eq!(
        source
            .to_specified_css_with_limits(L::new(6, 6, 1))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(source, &before);
}
#[test]
fn global_output_prices_one_symbolic_keyword_plus_occurrence_and_name_for_every_transform_property()
{
    for (name, _, _) in LIFECYCLES {
        for (text, _) in GLOBALS {
            budgets(&checked(property(name), text, true), text, 3, 3);
        }
    }
}
