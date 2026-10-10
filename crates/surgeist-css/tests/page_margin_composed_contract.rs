#![forbid(unsafe_code)]
//! Selected Page3 property applicability composes with Cascade5 globals and Values3/4
//! authored lengths. Page-local custom declarations use the approved bounded
//! WebKit witness; variable lookup, fallback selection and cascade stay downstream.
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("inherit", CssGlobalKeyword::Inherit),
    ("initial", CssGlobalKeyword::Initial),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn clean<T: Clone>(report: &CssParseReport<T>) {
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(report.clone().into_validation_result().is_ok());
}

fn page_list(body: &str, front: usize) -> CssDeclarationList {
    match front {
        0 => {
            let report = parse_sheet(&format!("@page :left{{{body}}}"));
            clean(&report);
            let [CssRule::Page(page)] = report.syntax().rules() else {
                panic!("one actual Page rule")
            };
            assert_eq!(page_pseudo(page), Some(CssPagePseudo::Left));
            page.declarations().properties().clone()
        }
        1 => {
            let report = parse_rule(
                &format!("@page :right{{{body}}}"),
                &CssNamespaceContext::default(),
            );
            clean(&report);
            let Some(CssRule::Page(page)) = report.syntax() else {
                panic!("one isolated Page rule")
            };
            assert_eq!(page_pseudo(page), Some(CssPagePseudo::Right));
            page.declarations().properties().clone()
        }
        2 | 3 => {
            let context = CssParserContext::new(if front == 2 {
                CssParserMode::Standards
            } else {
                CssParserMode::Quirks
            });
            let report = context.parse_page_block(&format!("{{{body}}}"));
            clean(&report);
            let fragment = report.syntax().as_ref().expect("genuine Page body");
            for declaration in fragment.body().declarations().properties().as_slice() {
                assert_eq!(declaration.parser_context(), context);
            }
            fragment.body().declarations().properties().clone()
        }
        _ => unreachable!(),
    }
}

fn source(name: &str, value: &str, front: usize) -> CssDeclaration {
    let list = page_list(&format!("{name}:{value}!important"), front);
    let [declaration] = list.as_slice() else {
        panic!("one retained Page occurrence")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    declaration.clone()
}

fn margins(declaration: &CssDeclaration) -> Vec<&CssMarginValue> {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Margin(value) => value.value().assigned_values().to_vec(),
        CssKnownPropertyValueRef::MarginTop(value) => vec![value.value()],
        CssKnownPropertyValueRef::MarginRight(value) => vec![value.value()],
        CssKnownPropertyValueRef::MarginBottom(value) => vec![value.value()],
        CssKnownPropertyValueRef::MarginLeft(value) => vec![value.value()],
        _ => panic!("physical margin owner"),
    }
}

fn globals(name: &str) {
    for front in 0..4 {
        for (text, keyword) in GLOBALS {
            let declaration = source(name, text, front);
            assert_eq!(declaration.known().unwrap().global(), Some(keyword));
            assert_eq!(
                declaration.to_specified_css().unwrap(),
                format!("{name}: {text} !important;")
            );
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&declaration).unwrap()
            else {
                panic!("symbolic global contributions")
            };
            assert_eq!(values.items().len(), if name == "margin" { 4 } else { 1 });
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&declaration));
            }
        }
    }
}

fn calculations(name: &str) {
    for value in [
        "calc(1px + 2%)",
        "calc(1cm - 2mm)",
        "calc((1in + 2pt) / 2)",
        "calc(-3pc + 4Q)",
        "calc(1rem + 2ch)",
    ] {
        for front in 0..4 {
            let declaration = source(name, value, front);
            for margin in margins(&declaration) {
                let CssMarginValue::LengthPercentage(length) = margin else {
                    panic!("symbolic length-percentage")
                };
                assert!(length.calculation().is_some(), "{value}");
                assert!(length.literal_component().is_none());
            }
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                value
            );
            let emitted = declaration.to_specified_css().unwrap();
            clean(&parse_page_block(&format!("{{{emitted}}}")));
        }
    }
}

fn units(name: &str) {
    // Relative lengths remain authored. Font, viewport and container selection
    // remain downstream.
    for value in [
        "-4Q", "2ch", "3rem", "1rex", "2cap", "3rcap", "4rch", "5ic", "6ric", "7lh", "8rlh", "1vw",
        "2vh", "3vmin", "4vmax", "1vi", "2vb", "1svw", "2svh", "3svi", "4svb", "5svmin", "6svmax",
        "1lvw", "2lvh", "3lvi", "4lvb", "5lvmin", "6lvmax", "1dvw", "2dvh", "3dvi", "4dvb",
        "5dvmin", "6dvmax", "1cqw", "2cqh", "3cqi", "4cqb", "5cqmin", "6cqmax",
    ] {
        for front in 0..4 {
            let declaration = source(name, value, front);
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                value
            );
            for margin in margins(&declaration) {
                let CssMarginValue::LengthPercentage(length) = margin else {
                    panic!("symbolic literal length")
                };
                assert!(length.literal_component().is_some());
            }
        }
    }
}

fn pending(name: &str) {
    for front in 0..4 {
        for authored in [
            "var(--m)",
            "var(--missing,)",
            "var(--missing, var(--other, 2px))",
            "bogus var(--m, 1px)",
        ] {
            let declaration = source(name, authored, front);
            let original = declaration.clone();
            assert_eq!(
                declaration
                    .known()
                    .unwrap()
                    .substitution_dependent()
                    .unwrap()
                    .as_css(),
                authored
            );
            let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
                panic!("pending whole Page margin")
            };
            assert!(handle.source().same_occurrence(&declaration));
            for _ in 0..2 {
                for invalid in [
                    "",
                    "1deg",
                    "1px bogus",
                    "inherit 1px",
                    "1zz",
                    "1yy",
                    "calc(1px + 2zz)",
                    "calc(1px + 2yy)",
                    "calc(0 * 1zz)",
                ] {
                    let replacement = parse_component_values(invalid).unwrap();
                    let before = replacement.clone();
                    assert!(
                        matches!(
                            handle.reenter(replacement.clone()).unwrap_err().kind(),
                            CssExpansionErrorKind::InvalidReplacement(_)
                        ),
                        "{name}: {invalid}"
                    );
                    assert_eq!(replacement, before);
                }
                for residual in [
                    "var(--again)",
                    "[f(var(--again))]",
                    "calc(1px + env(foo))",
                    "f(attr(data-m))",
                ] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution
                    );
                }
                for valid in [
                    "2Q",
                    "calc(1px + 2%)",
                    "3rem",
                    "auto",
                    "1cqw",
                    "2cqh",
                    "3cqi",
                    "4cqb",
                    "5cqmin",
                    "6cqmax",
                ] {
                    let replacement = parse_component_values(&format!("/*😀*/{valid}")).unwrap();
                    let before = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("completed Page margin terminals")
                    };
                    assert_eq!(values.items().len(), if name == "margin" { 4 } else { 1 });
                    for item in values.items() {
                        assert!(item.source().same_occurrence(&declaration));
                        assert_eq!(item.source().importance(), CssImportance::Important);
                        assert_eq!(item.replacement_components(), Some(&replacement));
                        assert!(item.ordinary_value().is_some());
                    }
                    assert_eq!(replacement, before);
                }
                for (text, keyword) in GLOBALS {
                    let CssContributions::Longhands(values) = handle
                        .reenter(parse_component_values(text).unwrap())
                        .unwrap()
                    else {
                        panic!("global replacement")
                    };
                    for item in values.items() {
                        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    }
                }
            }
            assert_eq!(declaration, original);
        }
    }
}

macro_rules! physical {
    ($module:ident, $name:literal) => {
        mod $module {
            use super::*;
            #[test]
            fn all_five_globals_remain_symbolic() {
                globals($name);
            }
            #[test]
            fn composed_calculations_remain_typed_and_unresolved() {
                calculations($name);
            }
            #[test]
            fn later_units_remain_symbolic() {
                units($name);
            }
            #[test]
            fn strict_pending_reentry_preserves_page_restrictions_and_reusable_identity() {
                pending($name);
            }
        }
    };
}

physical!(shorthand, "margin");
physical!(top, "margin-top");
physical!(right, "margin-right");
physical!(bottom, "margin-bottom");
physical!(left, "margin-left");

fn container_length(unit: &str) {
    // Conditional5 WD 2025-10-30 §7 defines each unit as a length. Its
    // per-element query-container selection/fallback is a computed-value rule,
    // not an authored Page grammar restriction.
    for name in [
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
    ] {
        for front in 0..4 {
            let literal = format!("3{unit}");
            let declaration = source(name, &literal, front);
            for margin in margins(&declaration) {
                let CssMarginValue::LengthPercentage(length) = margin else {
                    panic!("container-relative length")
                };
                assert!(
                    matches!(length.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit: actual, .. }) if actual == unit)
                );
            }
            let calculation = format!("calc(1{unit} + 2%)");
            let declaration = source(name, &calculation, front);
            for margin in margins(&declaration) {
                let CssMarginValue::LengthPercentage(length) = margin else {
                    panic!("symbolic container length-percentage")
                };
                assert!(length.calculation().is_some());
            }
            let pending = source(name, "var(--container-margin)", front);
            let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
                panic!("pending Page margin")
            };
            for value in [&literal, &calculation] {
                let replacement = parse_component_values(value).unwrap();
                let before = replacement.clone();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("completed symbolic container margin")
                };
                assert_eq!(values.items().len(), if name == "margin" { 4 } else { 1 });
                for item in values.items() {
                    assert!(item.source().same_occurrence(&pending));
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    assert!(item.ordinary_value().is_some());
                }
                assert_eq!(replacement, before);
            }
            assert!(
                handle
                    .reenter(parse_component_values("1em").unwrap())
                    .is_ok()
            );
        }
    }
}

macro_rules! container_unit {
    ($test:ident, $unit:literal) => {
        #[test]
        fn $test() {
            container_length($unit);
        }
    };
}
container_unit!(
    container_width_length_remains_symbolic_in_page_and_reentry,
    "cqw"
);
container_unit!(
    container_height_length_remains_symbolic_in_page_and_reentry,
    "cqh"
);
container_unit!(
    container_inline_length_remains_symbolic_in_page_and_reentry,
    "cqi"
);
container_unit!(
    container_block_length_remains_symbolic_in_page_and_reentry,
    "cqb"
);
container_unit!(
    container_minimum_length_remains_symbolic_in_page_and_reentry,
    "cqmin"
);
container_unit!(
    container_maximum_length_remains_symbolic_in_page_and_reentry,
    "cqmax"
);

#[test]
fn page_context_restriction_does_not_escape_into_ordinary_declarations_or_reentry() {
    for name in [
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
    ] {
        for value in ["1em", "1ex", "calc(1px + 2em)"] {
            clean(&parse_declaration(&format!("{name}:{value}")));
            let report = parse_declaration(&format!("{name}:var(--m)"));
            clean(&report);
            let CssExpansion::Pending(handle) =
                expand_declaration(report.syntax().as_ref().unwrap()).unwrap()
            else {
                panic!("ordinary pending")
            };
            assert!(
                handle
                    .reenter(parse_component_values(value).unwrap())
                    .is_ok()
            );
        }
    }
}

#[test]
fn strict_pending_reentry_rejects_recovered_original_closures_with_original_provenance() {
    let declaration = source("margin-left", "var(--m)", 2);
    let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
        panic!("Page pending")
    };
    for invalid in ["calc(1px + 2%", "1px/*unfinished"] {
        let replacement = parse_component_values(invalid).unwrap();
        let before = replacement.clone();
        let error = handle.reenter(replacement.clone()).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
            panic!("strict closure failure")
        };
        assert!(matches!(
            detail.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
        ));
        assert!(matches!(
            detail.origin(),
            CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { .. }))
        ));
        assert_eq!(replacement, before);
    }
    assert!(
        handle
            .reenter(parse_component_values("2Q").unwrap())
            .is_ok()
    );
}

#[test]
fn shorthand_one_to_four_values_expand_in_physical_order() {
    for (authored, expected) in [
        ("1Q", ["1q", "1q", "1q", "1q"]),
        ("1Q 2ch", ["1q", "2ch", "1q", "2ch"]),
        ("1Q 2ch 3rem", ["1q", "2ch", "3rem", "2ch"]),
        ("1Q 2ch 3rem auto", ["1q", "2ch", "3rem", "auto"]),
    ] {
        for front in 0..4 {
            let declaration = source("margin", authored, front);
            assert_eq!(
                margins(&declaration)
                    .into_iter()
                    .map(|v| v.serialize_specified().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&declaration).unwrap()
            else {
                panic!("four physical contributions")
            };
            assert_eq!(
                values
                    .items()
                    .iter()
                    .map(|v| v.property().canonical_name())
                    .collect::<Vec<_>>(),
                ["margin-top", "margin-right", "margin-bottom", "margin-left"]
            );
            for item in values.items() {
                assert!(item.source().same_occurrence(&declaration));
            }
        }
    }
}

#[test]
fn strict_shorthand_reentry_checks_every_assigned_side_and_whole_arity() {
    let declaration = source("margin", "var(--m)", 0);
    let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
        panic!("pending shorthand")
    };
    for invalid in [
        "1px 2px 3px 4px 5px",
        "1px 2zz",
        "1px 2px 3yy",
        "1px 2px 3px calc(1zz + 2px)",
        "logical 1px",
        "1px initial",
    ] {
        assert!(
            matches!(
                handle
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ),
            "{invalid}"
        );
    }
    for valid in ["1Q", "1Q 2ch", "1Q 2ch 3rem", "1Q 2ch 3rem calc(1px + 2%)"] {
        let CssContributions::Longhands(values) = handle
            .reenter(parse_component_values(valid).unwrap())
            .unwrap()
        else {
            panic!("four completed terminals")
        };
        assert_eq!(values.items().len(), 4);
    }
}

#[test]
fn page_local_custom_declarations_preserve_case_tokens_duplicates_priority_and_selector_scope() {
    let input = "--M: 1em;--m: var(--other, 2Q) !important;--empty:;--M: 3rem;margin:var(--m, 1px)!important";
    for front in 0..4 {
        let list = page_list(input, front);
        assert_eq!(list.len(), 5);
        for (index, name) in ["--M", "--m", "--empty", "--M"].into_iter().enumerate() {
            assert!(
                matches!(list[index].property_name(), CssPropertyNameRef::Custom(value) if value.as_str() == name)
            );
            assert!(matches!(
                expand_declaration(&list[index]).unwrap(),
                CssExpansion::Contributions(CssContributions::Custom(_))
            ));
        }
        assert_eq!(list[1].importance(), CssImportance::Important);
        assert_eq!(
            list[0].custom().unwrap().value().value().unwrap().as_css(),
            "1em"
        );
        assert_eq!(
            list[1].custom().unwrap().value().value().unwrap().as_css(),
            "var(--other, 2Q)"
        );
        assert_eq!(
            list[2].custom().unwrap().value().value().unwrap().as_css(),
            ""
        );
        assert!(matches!(
            expand_declaration(&list[4]).unwrap(),
            CssExpansion::Pending(_)
        ));
    }
    let report = parse_sheet(
        ":root{--m:9px}@page :left{--m:2Q;margin:var(--m)}@page :right{margin:var(--m,1px)}",
    );
    clean(&report);
    let [
        CssRule::Style(root),
        CssRule::Page(left),
        CssRule::Page(right),
    ] = report.syntax().rules()
    else {
        panic!("separate authored scopes")
    };
    assert_eq!(root.declarations().len(), 1);
    assert_eq!(left.declarations().properties().len(), 2);
    assert_eq!(right.declarations().properties().len(), 1);
    assert!(matches!(
        expand_declaration(&right.declarations().properties()[0]).unwrap(),
        CssExpansion::Pending(_)
    ));
}

#[test]
fn defined_font_relative_replacement_is_admitted_without_parser_time_fallback_selection() {
    let list = page_list("--m:1em;margin:var(--m,1px)", 2);
    let CssExpansion::Pending(handle) = expand_declaration(&list[1]).unwrap() else {
        panic!("pending margin")
    };
    // The caller supplies the defined variable's replacement. The Page3 domain
    // retains font-relative syntax without selecting a variable or fallback.
    let replacement = list[0].value_components().clone();
    let CssContributions::Longhands(values) = handle.reenter(replacement).unwrap() else {
        panic!("four physical margins")
    };
    assert_eq!(values.items().len(), 4);
    for value in values.items() {
        let CssContributionValueRef::Ordinary(value) = value.value() else {
            panic!("ordinary retained length")
        };
        let margin = match value {
            CssLonghandValueRef::MarginTop(v)
            | CssLonghandValueRef::MarginRight(v)
            | CssLonghandValueRef::MarginBottom(v)
            | CssLonghandValueRef::MarginLeft(v) => v,
            _ => panic!("physical margin terminal"),
        };
        assert_eq!(margin.serialize_specified().unwrap(), "1em");
    }
    assert_eq!(
        handle
            .source()
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "var(--m,1px)"
    );
    assert!(
        handle
            .reenter(parse_component_values("1px").unwrap())
            .is_ok()
    );
}

#[test]
fn composed_values_preserve_unicode_snapshot_spans_and_occurrence_order() {
    let input =
        "/*é*/\r\n@page :first{--M:2Q;margin-left:calc(1px + 2%)!important;margin-left:var(--M)}";
    let report = parse_sheet(input);
    clean(&report);
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("actual Page")
    };
    assert_eq!(page_pseudo(page), Some(CssPagePseudo::First));
    assert_eq!(page.declarations().properties().len(), 3);
    for (declaration, name, value) in [
        (&page.declarations().properties()[0], "--M", "2Q"),
        (
            &page.declarations().properties()[1],
            "margin-left",
            "calc(1px + 2%)",
        ),
        (
            &page.declarations().properties()[2],
            "margin-left",
            "var(--M)",
        ),
    ] {
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.source().as_str(), input);
        assert_eq!(
            &input[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
            value
        );
        let name_origin = declaration.parsed_name().unwrap();
        assert_eq!(
            &input[name_origin.span().start().byte_offset().value()
                ..name_origin.span().end().byte_offset().value()],
            name
        );
        assert!(name_origin.source().same_snapshot(origin.source()));
        assert_eq!(
            declaration.value_components().serialize().unwrap().as_css(),
            value
        );
    }
    assert_eq!(
        page.declarations().properties()[1].importance(),
        CssImportance::Important
    );
    assert_eq!(
        page.declarations().properties()[2].importance(),
        CssImportance::Normal
    );
}

#[test]
fn recovery_drops_invalid_units_unknown_properties_and_children_in_source_order() {
    let input = "{margin-top:initial;margin-right:1zz;--m:2Q;noise:red;@bad-left{content:'x'}margin-left:var(--m);margin-bottom:calc(1px + 2yy);margin:1Q 2ch 3rem auto}";
    let report = parse_page_block(input);
    assert!(!report.is_clean());
    assert!(report.clone().into_validation_result().is_err());
    let list = report
        .syntax()
        .as_ref()
        .unwrap()
        .body()
        .declarations()
        .properties();
    assert_eq!(list.len(), 4, "{:?}", report.diagnostics());
    assert_eq!(
        list[0].known().unwrap().global(),
        Some(CssGlobalKeyword::Initial)
    );
    assert!(list[1].custom().is_some());
    assert!(list[2].known().unwrap().substitution_dependent().is_some());
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropDeclaration,
            CssRecoveryAction::DropDeclaration,
            CssRecoveryAction::DropAtRule,
            CssRecoveryAction::DropDeclaration
        ]
    );
    assert!(
        report
            .diagnostics()
            .windows(2)
            .all(|v| v[0].error().position().byte_offset().value()
                <= v[1].error().position().byte_offset().value())
    );
}

#[test]
fn malformed_margin_types_and_global_mixtures_recover_only_the_bad_occurrence() {
    for value in [
        "1deg",
        "calc(1px + 1s)",
        "inherit 1px",
        "1px initial",
        "1px 2px 3px 4px 5px",
        "logical 1px",
        "1zz",
        "1yy",
        "calc(1zz + 2px)",
    ] {
        let report = parse_page_block(&format!("{{margin-top:0;margin:{value};margin-left:auto}}"));
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .body()
                .declarations()
                .properties()
                .len(),
            2,
            "{value}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{value}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert!(!report.is_clean());
    }
}

#[test]
fn page_payload_normalization_retains_pending_custom_and_composed_occurrences() {
    let report = parse_sheet(
        "@page :first{--m:2Q;margin:var(--m)!important;margin-top:calc(1px + 2%);margin-left:revert-layer}",
    );
    clean(&report);
    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap(),
    )
    .unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("one Page context payload")
    };
    let CssRuleContextKindRef::Page(page) = context.kind() else {
        panic!("Page payload")
    };
    assert_eq!(page_pseudo(page), Some(CssPagePseudo::First));
    assert_eq!(page.declarations().properties().len(), 4);
    assert!(page.declarations().properties()[0].custom().is_some());
    let CssExpansion::Pending(handle) =
        expand_declaration(&page.declarations().properties()[1]).unwrap()
    else {
        panic!("pending survived normalization")
    };
    assert!(
        handle
            .reenter(parse_component_values("1em").unwrap())
            .is_ok()
    );
    assert!(
        handle
            .reenter(parse_component_values("2Q").unwrap())
            .is_ok()
    );
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 0, 0, 0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn specified_output_and_checked_list_assembly_share_atomic_cumulative_limits() {
    let list = page_list(
        "--m:2Q;margin-left:var(--m)!important;margin-top:initial",
        0,
    );
    let before = list.clone();
    let expected = "--m: 2Q; margin-left: var(--m) !important; margin-top: initial;";
    assert_eq!(list.to_specified_css().unwrap(), expected);
    let ample = CssSpecifiedValueSerializationLimits::new(1000, 1000, expected.len());
    assert_eq!(list.to_specified_css_with_limits(ample).unwrap(), expected);
    let short = CssSpecifiedValueSerializationLimits::new(1000, 1000, expected.len() - 1);
    assert_eq!(
        list.to_specified_css_with_limits(short).unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        CssDeclarationList::try_new_with_limits(list.as_slice().to_vec(), short)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1000, 1000),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1000, 0, 1000),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(list, before);
    let checked = CssDeclarationList::try_new_with_limits(list.as_slice().to_vec(), ample).unwrap();
    let repeated = vec![checked[1].clone(), checked[1].clone()];
    let one_bytes = "margin-left: var(--m) !important;".len();
    let one_budget = CssSpecifiedValueSerializationLimits::new(1000, 1000, one_bytes);
    assert!(checked[1].to_specified_css_with_limits(one_budget).is_ok());
    assert_eq!(
        CssDeclarationList::try_new_with_limits(repeated, one_budget)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    let CssExpansion::Pending(handle) = expand_declaration(&checked[1]).unwrap() else {
        panic!("checked assembly preserves pending context")
    };
    assert!(
        handle
            .reenter(parse_component_values("1ex").unwrap())
            .is_ok()
    );
    assert!(
        handle
            .reenter(parse_component_values("2Q").unwrap())
            .is_ok()
    );
}

#[test]
fn actual_group_page_values_compose_without_lifting_style_ancestor_placement() {
    let report = parse_sheet(
        "@media print{@page :right{--m:2Q;margin:var(--m)}}@scope (.root){@media print{@page{margin-left:calc(1px + 2%)}}}",
    );
    clean(&report);
    let [CssRule::Media(media), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("ordinary group owners")
    };
    let [CssRule::Page(page)] = media.rules() else {
        panic!("Page in ordinary media")
    };
    assert_eq!(page.declarations().properties().len(), 2);
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("scoped ordinary media")
    };
    let [CssScopedRule::Page(page)] = media.rules().rules() else {
        panic!("scoped ordinary Page")
    };
    assert_eq!(page.declarations().properties().len(), 1);
    let ns = CssNamespaceContext::default();
    let input = "{@page{--m:2Q;margin:var(--m)}.after{color:red}}";
    for report in [
        parse_scope_block(input, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(input, &ns, CssStyleAncestor::Present),
    ] {
        assert!(!report.is_clean());
        let [CssScopedRule::Style(style)] = report.syntax().as_ref().unwrap().body().rules() else {
            panic!("following style survives excluded Page")
        };
        assert_eq!(
            style.declarations()[0].to_specified_css().unwrap(),
            "color: red;"
        );
    }
}

fn page_pseudo(page: &surgeist_css::CssPageRule) -> Option<surgeist_css::CssPagePseudo> {
    page.selectors()
        .selectors()
        .first()
        .and_then(|s| s.pseudos().first().copied())
}
