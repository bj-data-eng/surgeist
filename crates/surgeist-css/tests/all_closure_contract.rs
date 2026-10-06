#![forbid(unsafe_code)]
//! All original-component closure, symbolic reset and provenance contract.
//! Cascade5 CR20220113 #all-shorthand/#propdef-all; Syntax3 original recovery
//! provenance; Variables1 pending reentry; current public authored contracts.
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn checked(
    values: CssComponentValues,
    grammar: bool,
    importance: CssImportance,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    if grammar {
        parse_property_value_for_grammar(CssKnownProperty::All.grammar(), values, importance)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::All),
            values,
            importance,
        )
    }
}
fn source(text: &str, importance: CssImportance, grammar: bool) -> CssDeclaration {
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    let declaration = checked(values.clone(), grammar, importance).unwrap();
    assert_eq!(declaration.value_components(), &values);
    assert_eq!(values, before);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}
fn strict_error(error: &CssPropertyValueParseError, original: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
        error.origin()
    else {
        panic!("original implicit EOF origin: {error:?}")
    };
    assert_eq!(opening.source().as_str(), original);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), original.len());
    assert_eq!(at.span().end().byte_offset().value(), original.len());
}
fn assert_reset(
    reset: &CssUniversalReset,
    declaration: &CssDeclaration,
    keyword: CssGlobalKeyword,
    replacement: Option<&CssComponentValues>,
) {
    assert_eq!(reset.keyword(), keyword);
    assert!(reset.source().same_occurrence(declaration));
    assert_eq!(reset.source().importance(), declaration.importance());
    assert_eq!(
        reset.source().value_components(),
        declaration.value_components()
    );
    assert_eq!(reset.replacement_components(), replacement);
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
    let custom = CssCustomPropertyName::try_new("--Theme").unwrap();
    assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));
    // One independently selected ordinary terminal is a control, not an inventory.
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Color)));
}
fn reset(declaration: &CssDeclaration, keyword: CssGlobalKeyword) {
    let CssExpansion::Contributions(CssContributions::UniversalReset(value)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("one symbolic universal-reset contribution")
    };
    assert_reset(&value, declaration, keyword, None);
}

#[test]
fn checked_all_globals_reject_original_comment_eof_before_serialized_recovery() {
    for (keyword, _) in GLOBALS {
        let original = format!("{keyword}/*");
        let values = parse_component_values(&original).unwrap();
        let before = values.clone();
        for grammar in [false, true] {
            for importance in [CssImportance::Normal, CssImportance::Important] {
                for _ in 0..2 {
                    strict_error(
                        &checked(values.clone(), grammar, importance).unwrap_err(),
                        &original,
                    );
                    assert_eq!(values, before);
                }
            }
        }
    }
}
#[test]
fn checked_all_pending_values_reject_original_function_eof_before_admission() {
    for original in ["var(--reset", "env(reset", "attr(data-reset *"] {
        let values = parse_component_values(original).unwrap();
        let before = values.clone();
        for grammar in [false, true] {
            for _ in 0..2 {
                strict_error(
                    &checked(values.clone(), grammar, CssImportance::Important).unwrap_err(),
                    original,
                );
                assert_eq!(values, before);
            }
        }
    }
}
#[test]
fn all_pending_reentry_rejects_recovered_globals_then_accepts_matched_complete_retry() {
    for grammar in [false, true] {
        let declaration = source("var(--reset)", CssImportance::Important, grammar);
        let before = declaration.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
            panic!("pending All")
        };
        for (keyword, expected) in GLOBALS {
            let original = format!("{keyword}/*");
            let replacement = parse_component_values(&original).unwrap();
            let snapshot = replacement.clone();
            for _ in 0..2 {
                let error = handle.reenter(replacement.clone()).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
                    panic!("strict replacement error")
                };
                strict_error(detail, &original);
                assert_eq!(replacement, snapshot);
                assert!(handle.source().same_occurrence(&declaration));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                let complete = parse_component_values(&format!("{keyword}/**/")).unwrap();
                let CssContributions::UniversalReset(value) =
                    handle.reenter(complete.clone()).unwrap()
                else {
                    panic!("completed All retry")
                };
                assert_reset(&value, &declaration, expected, Some(&complete));
            }
        }
        assert_eq!(declaration, before);
    }
}
#[test]
fn mixed_origin_all_recovery_rejects_the_actual_comment_source_without_fabricating_coordinates() {
    let mut items = vec![CssComponentValue::try_ident("inherit").unwrap()];
    items.extend(
        parse_component_values("/*")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let values = CssComponentValues::try_new(items).unwrap();
    let before = values.clone();
    assert_eq!(values.items()[0].origin(), &CssValueOrigin::Programmatic);
    for grammar in [false, true] {
        strict_error(
            &checked(values.clone(), grammar, CssImportance::Important).unwrap_err(),
            "/*",
        );
        assert_eq!(values, before);
    }
    let declaration = source("var(--reset)", CssImportance::Important, true);
    let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
        panic!("pending")
    };
    let error = handle.reenter(values.clone()).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
        panic!("replacement failure")
    };
    strict_error(detail, "/*");
    assert_eq!(values, before);
    let mut complete_items = vec![CssComponentValue::try_ident("inherit").unwrap()];
    complete_items.extend(
        parse_component_values("/**/")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let complete = CssComponentValues::try_new(complete_items).unwrap();
    let CssContributions::UniversalReset(value) = handle.reenter(complete.clone()).unwrap() else {
        panic!("mixed complete retry")
    };
    assert_reset(
        &value,
        &declaration,
        CssGlobalKeyword::Inherit,
        Some(&complete),
    );
}

#[test]
fn complete_all_globals_keep_all_five_fronts_canonical_output_and_symbolic_exclusions() {
    for (keyword, expected) in GLOBALS {
        let text = format!("{keyword}/**/");
        let css = format!("/*😀*/ALL:{text}!important");
        let report = parse_style_attribute(&css);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(&validate_style_attribute(&css).unwrap(), report.syntax());
        let [parsed] = report.syntax().as_slice() else {
            panic!("one All occurrence")
        };
        assert_eq!(
            parsed.position().unwrap().byte_offset().value(),
            "/*😀*/".len()
        );
        assert_eq!(
            parsed.position().unwrap().column().value() as usize,
            "/*😀*/".encode_utf16().count()
        );
        assert_eq!(parsed.parsed_value().unwrap().source().as_str(), css);
        let mut sources = vec![
            parsed.clone(),
            source(&text, CssImportance::Important, false),
            source(&text, CssImportance::Important, true),
        ];
        for report in [
            parse_property_value_text(
                &text,
                CssPropertyNameRef::Known(CssKnownProperty::All),
                CssImportance::Important,
            ),
            parse_property_value_text_for_grammar(
                &text,
                CssKnownProperty::All.grammar(),
                CssImportance::Important,
            ),
        ] {
            assert!(report.is_clean());
            let declaration = report.syntax().as_ref().unwrap();
            assert_eq!(declaration.parsed_value().unwrap().source().as_str(), text);
            sources.push(declaration.clone());
        }
        for declaration in sources {
            assert_eq!(
                declaration.known().unwrap().property(),
                CssKnownProperty::All
            );
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                text
            );
            reset(&declaration, expected);
            assert_eq!(
                declaration.to_specified_css().unwrap(),
                format!("all: {keyword} !important;")
            );
        }
    }
}
#[test]
fn complete_all_pending_reentry_keeps_original_occurrence_and_parsed_or_programmatic_replacements()
{
    for text in ["var(--reset)", "env(reset)", "attr(data-reset *)"] {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let suffix = if importance == CssImportance::Important {
                "!important"
            } else {
                ""
            };
            let css = format!("all:{text}{suffix}");
            let report = parse_style_attribute(&css);
            assert!(report.is_clean());
            for declaration in [
                report.syntax()[0].clone(),
                source(text, importance, false),
                source(text, importance, true),
            ] {
                let before = declaration.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap()
                else {
                    panic!("whole pending All")
                };
                let cloned = handle.clone();
                assert!(cloned.source().same_occurrence(&declaration));
                let parsed = parse_component_values("revert-layer/**/").unwrap();
                let programmatic = CssComponentValues::try_new(vec![
                    CssComponentValue::try_ident("revert-layer").unwrap(),
                ])
                .unwrap();
                for replacement in [parsed, programmatic] {
                    for _ in 0..2 {
                        let CssContributions::UniversalReset(value) =
                            cloned.reenter(replacement.clone()).unwrap()
                        else {
                            panic!("one universal reset")
                        };
                        assert_reset(
                            &value,
                            &declaration,
                            CssGlobalKeyword::RevertLayer,
                            Some(&replacement),
                        );
                        assert_eq!(
                            value.replacement_components().unwrap().items()[0].origin(),
                            replacement.items()[0].origin()
                        );
                    }
                }
                assert_eq!(declaration, before);
            }
        }
    }
}
#[test]
fn browser_recovered_all_globals_and_pending_values_remain_unclean_authored_occurrences() {
    for text in [
        "inherit/*",
        "revert-layer/*",
        "var(--reset",
        "env(reset",
        "attr(data-reset *",
    ] {
        let css = format!("all:{text}");
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        let [declaration] = report.syntax().as_slice() else {
            panic!("browser-recovered All occurrence")
        };
        assert_eq!(
            declaration.known().unwrap().property(),
            CssKnownProperty::All
        );
        assert_eq!(declaration.parsed_value().unwrap().source().as_str(), css);
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        if text.starts_with("inherit") {
            reset(declaration, CssGlobalKeyword::Inherit);
        } else if text.starts_with("revert-layer") {
            reset(declaration, CssGlobalKeyword::RevertLayer);
        } else {
            assert!(matches!(
                expand_declaration(declaration).unwrap(),
                CssExpansion::Pending(_)
            ));
        }
    }
}
#[test]
fn invalid_all_replacements_remain_atomic_and_residual_substitution_is_distinct() {
    let declaration = source("var(--reset)", CssImportance::Important, true);
    let before = declaration.clone();
    let CssExpansion::Pending(handle) = expand_declaration(&declaration).unwrap() else {
        panic!("pending")
    };
    for text in [
        "",
        "red",
        "inherit initial",
        "inherit!important",
        "inherit;color:red",
    ] {
        let replacement = parse_component_values(text).unwrap();
        let snapshot = replacement.clone();
        for _ in 0..2 {
            let error = handle.reenter(replacement.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
                panic!("owned grammar failure")
            };
            assert!(matches!(
                detail.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            assert_eq!(replacement, snapshot);
            let complete = parse_component_values("unset").unwrap();
            let CssContributions::UniversalReset(value) = handle.reenter(complete.clone()).unwrap()
            else {
                panic!("valid retry")
            };
            assert_reset(
                &value,
                &declaration,
                CssGlobalKeyword::Unset,
                Some(&complete),
            );
        }
    }
    for text in ["var(--again)", "env(again)", "attr(data-again *)"] {
        assert_eq!(
            handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
    }
    assert_eq!(declaration, before);
    assert!(handle.source().same_occurrence(&declaration));
}
#[test]
fn all_remains_one_symbolic_normalization_unit_per_occurrence_with_atomic_retry() {
    let css = ".a{all:inherit!important;all:revert-layer}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 1, 2, 2).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    let [first, second] = declarations.as_slice() else {
        panic!("two ordered All occurrences")
    };
    for (order, (value, keyword)) in [
        (*first, CssGlobalKeyword::Inherit),
        (*second, CssGlobalKeyword::RevertLayer),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(
            value.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            value.expansion()
        else {
            panic!("symbolic reset")
        };
        assert_reset(reset, value.source(), keyword, None);
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            css
        );
    }
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(1, 1, 2, 1).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 1
        }
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(second.source())
    );
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn global_all_canonical_writer_preserves_three_node_prices_and_atomic_source() {
    for (keyword, _) in GLOBALS {
        let declaration = source(&format!("{keyword}/**/"), CssImportance::Important, true);
        let before = declaration.clone();
        let expected = format!("all: {keyword} !important;");
        let exact = CssSpecifiedValueSerializationLimits::new(3, 3, expected.len());
        assert_eq!(
            declaration.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    declaration
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(declaration, before);
            }
        }
        assert_eq!(
            declaration.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
}
