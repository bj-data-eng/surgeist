//! All original-component closure, symbolic reset and provenance contract.
//! Cascade5 CR20220113 #all-shorthand/#propdef-all; Syntax3 original recovery
//! provenance; Variables1 pending reentry; current public authored contracts.
use crate::css::support::authored_property::checked_components;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::declaration::{
    CssCustomPropertyName, CssDeclaration, CssGlobalKeyword, CssImportance,
};
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssPropertyNameRef, CssPropertyTextTarget,
    CssPropertyValueError, CssUniversalReset,
};
use surgeist_css::serialization::CssSerializationLimits;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
fn source(text: &str, importance: CssImportance, grammar: bool) -> CssDeclaration {
    let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let before = values.clone();
    let declaration =
        checked_components(CssKnownProperty::All, values.clone(), grammar, importance).unwrap();
    assert_eq!(declaration.value_components(), &values);
    assert_eq!(values, before);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    declaration
}
fn strict_error(error: &CssPropertyValueError, original: &str) {
    assert!(error.code() == Some(CssErrorCode::UnexpectedEnd));
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
        declaration.expand().unwrap()
    else {
        panic!("one symbolic universal-reset contribution")
    };
    assert_reset(&value, declaration, keyword, None);
}

#[test]
fn checked_all_globals_reject_original_comment_eof_before_serialized_recovery() {
    for (keyword, _) in GLOBALS {
        let original = format!("{keyword}/*");
        let values = CssComponentValues::parse(&CssParser::default(), &original).unwrap();
        let before = values.clone();
        for grammar in [false, true] {
            for importance in [CssImportance::Normal, CssImportance::Important] {
                for _ in 0..2 {
                    strict_error(
                        &checked_components(
                            CssKnownProperty::All,
                            values.clone(),
                            grammar,
                            importance,
                        )
                        .unwrap_err(),
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
        let values = CssComponentValues::parse(&CssParser::default(), original).unwrap();
        let before = values.clone();
        for grammar in [false, true] {
            for _ in 0..2 {
                strict_error(
                    &checked_components(
                        CssKnownProperty::All,
                        values.clone(),
                        grammar,
                        CssImportance::Important,
                    )
                    .unwrap_err(),
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
        let CssExpansion::Pending(handle) = declaration.expand().unwrap() else {
            panic!("pending All")
        };
        for (keyword, expected) in GLOBALS {
            let original = format!("{keyword}/*");
            let replacement = CssComponentValues::parse(&CssParser::default(), &original).unwrap();
            let snapshot = replacement.clone();
            for _ in 0..2 {
                let error = handle.reenter(replacement.clone()).unwrap_err();
                let Some(detail) = error.replacement_error() else {
                    panic!("strict replacement error")
                };
                strict_error(detail, &original);
                assert_eq!(replacement, snapshot);
                assert!(handle.source().same_occurrence(&declaration));
                assert_eq!(handle.source().importance(), CssImportance::Important);
                let complete =
                    CssComponentValues::parse(&CssParser::default(), format!("{keyword}/**/"))
                        .unwrap();
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
        CssComponentValues::parse(&CssParser::default(), "/*")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let values = CssComponentValues::try_new(items, Default::default()).unwrap();
    let before = values.clone();
    assert_eq!(values.items()[0].origin(), &CssValueOrigin::Programmatic);
    for grammar in [false, true] {
        strict_error(
            &checked_components(
                CssKnownProperty::All,
                values.clone(),
                grammar,
                CssImportance::Important,
            )
            .unwrap_err(),
            "/*",
        );
        assert_eq!(values, before);
    }
    let declaration = source("var(--reset)", CssImportance::Important, true);
    let CssExpansion::Pending(handle) = declaration.expand().unwrap() else {
        panic!("pending")
    };
    let error = handle.reenter(values.clone()).unwrap_err();
    let Some(detail) = error.replacement_error() else {
        panic!("replacement failure")
    };
    strict_error(detail, "/*");
    assert_eq!(values, before);
    let mut complete_items = vec![CssComponentValue::try_ident("inherit").unwrap()];
    complete_items.extend(
        CssComponentValues::parse(&CssParser::default(), "/**/")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let complete = CssComponentValues::try_new(complete_items, Default::default()).unwrap();
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
        let report = CssParser::default().parse_block_contents(&css);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(
            &CssParser::default()
                .parse_block_contents(&css)
                .into_validation_result()
                .unwrap(),
            report.syntax()
        );
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
            CssParser::default().parse_property_value(
                &text,
                CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::All)),
                CssImportance::Important,
            ),
            CssParser::default().parse_property_value(
                &text,
                CssPropertyTextTarget::Grammar(CssKnownProperty::All.grammar()),
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
                declaration
                    .value_components()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                text
            );
            reset(&declaration, expected);
            assert_eq!(
                declaration.serialize_specified(Default::default()).unwrap(),
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
            let report = CssParser::default().parse_block_contents(&css);
            assert!(report.is_clean());
            for declaration in [
                report.syntax()[0].clone(),
                source(text, importance, false),
                source(text, importance, true),
            ] {
                let before = declaration.clone();
                let CssExpansion::Pending(handle) = declaration.expand().unwrap() else {
                    panic!("whole pending All")
                };
                let cloned = handle.clone();
                assert!(cloned.source().same_occurrence(&declaration));
                let parsed =
                    CssComponentValues::parse(&CssParser::default(), "revert-layer/**/").unwrap();
                let programmatic = CssComponentValues::try_new(
                    vec![CssComponentValue::try_ident("revert-layer").unwrap()],
                    Default::default(),
                )
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
        let report = CssParser::default().parse_block_contents(&css);
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
            CssParser::default()
                .parse_block_contents(&css)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
        if text.starts_with("inherit") {
            reset(declaration, CssGlobalKeyword::Inherit);
        } else if text.starts_with("revert-layer") {
            reset(declaration, CssGlobalKeyword::RevertLayer);
        } else {
            assert!(matches!(
                declaration.expand().unwrap(),
                CssExpansion::Pending(_)
            ));
        }
    }
}
#[test]
fn invalid_all_replacements_remain_atomic_and_residual_substitution_is_distinct() {
    let declaration = source("var(--reset)", CssImportance::Important, true);
    let before = declaration.clone();
    let CssExpansion::Pending(handle) = declaration.expand().unwrap() else {
        panic!("pending")
    };
    for text in [
        "",
        "red",
        "inherit initial",
        "inherit!important",
        "inherit;color:red",
    ] {
        let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let snapshot = replacement.clone();
        for _ in 0..2 {
            let error = handle.reenter(replacement.clone()).unwrap_err();
            let Some(detail) = error.replacement_error() else {
                panic!("owned grammar failure")
            };
            assert!(detail.code().is_some());
            assert_eq!(replacement, snapshot);
            let complete = CssComponentValues::parse(&CssParser::default(), "unset").unwrap();
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
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), text).unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
    }
    assert_eq!(declaration, before);
    assert!(handle.source().same_occurrence(&declaration));
}
#[test]
fn all_remains_one_symbolic_normalization_unit_per_occurrence_with_atomic_retry() {
    let css = ".a{all:inherit!important;all:revert-layer}";
    let report = CssParser::default().parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 1, 2, 2).unwrap();
    let normalized = report.syntax().normalize(exact).unwrap();
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
    let error = report
        .syntax()
        .normalize(CssNormalizationLimits::try_new(1, 1, 2, 1).unwrap())
        .unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 1))
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(second.source())
    );
    assert_eq!(report, before);
    assert!(report.syntax().normalize(exact).is_ok());
}
#[test]
fn global_all_canonical_writer_preserves_three_node_prices_and_atomic_source() {
    for (keyword, _) in GLOBALS {
        let declaration = source(&format!("{keyword}/**/"), CssImportance::Important, true);
        let before = declaration.clone();
        let expected = format!("all: {keyword} !important;");
        let exact = CssSerializationLimits::new(3, 3, expected.len());
        assert_eq!(declaration.serialize_specified(exact).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSerializationLimits::new(2, 3, expected.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(3, 2, expected.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(3, 3, expected.len() - 1),
                SerializationFailure::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    serialization_failure(&declaration.serialize_specified(limits).unwrap_err()),
                    kind
                );
                assert_eq!(declaration, before);
            }
        }
        assert_eq!(declaration.serialize_specified(exact).unwrap(), expected);
    }
}
