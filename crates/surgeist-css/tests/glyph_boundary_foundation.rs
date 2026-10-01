#![forbid(unsafe_code)]
//! Writing Modes 3 §5.1.3 legacy glyph alias boundary. The five listed
//! terminal mappings are explicit; numeric-terminal spelling and math
//! applicability remain unresolved under CSSWG issue 8032.

use surgeist_css::*;

const ALIAS: &str = "glyph-orientation-vertical";
const TARGET: CssKnownProperty = CssKnownProperty::TextOrientation;

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(ALIAS).expect("selected legacy grammar")
}

fn one(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn mapped(source: &CssDeclaration, expected: CssTextOrientation, authored: &str) {
    let known = source.known().expect("known legacy alias");
    assert_eq!(known.grammar(), grammar());
    assert_eq!(known.property(), TARGET);
    let CssKnownPropertyValueRef::TextOrientation(value) = known.property_value().unwrap() else {
        panic!("legacy alias maps to text-orientation")
    };
    assert_eq!(value.orientation(), &expected);
    assert_eq!(value.as_css(), authored);
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("one legacy target")
    else {
        panic!("terminal legacy expansion")
    };
    let [item] = values.items() else {
        panic!("one target and no resets")
    };
    assert_eq!(item.property(), TARGET);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert!(item.replacement_components().is_none());
    item.clone()
}

#[test]
fn legacy_alias_catalog_is_partial_with_a_truthful_unresolved_boundary() {
    let alias = feature_metadata("official.property-alias.glyph-orientation-vertical")
        .expect("selected alias catalog record");
    assert_eq!(alias.kind(), CssFeatureKind::PropertyAlias);
    assert_eq!(alias.spelling(), ALIAS);
    assert_eq!(alias.source().id().as_str(), "O-WRITING3");
    assert_eq!(alias.production(), "#propdef-glyph-orientation-vertical");
    assert_eq!(alias.status(), CssSupportStatus::Partial);
    let subset = alias.supported_subset().expect("implemented subset");
    for spelling in ["auto", "0deg", "90deg", "0", "90"] {
        assert!(subset.contains(spelling), "missing {spelling}: {subset}");
    }
    assert!(subset.contains("text-orientation"), "{subset}");
    let remainder = alias.unsupported_remainder().expect("unresolved scope");
    assert!(
        remainder.starts_with("Unresolved by the selected standard:"),
        "{remainder}"
    );
    for term in ["numeric", "math", "8032"] {
        assert!(
            remainder.to_ascii_lowercase().contains(term),
            "missing {term}: {remainder}"
        );
    }
    assert_eq!(alias.recognized_unsupported_code(), None);
    assert!(alias.baseline_alias_targets().is_empty());

    // The ordinary Partial record remains a known valid-but-unimplemented case.
    let ordinary = feature_metadata("ext.value.basic-shape").expect("Shapes 1 record");
    assert_eq!(ordinary.status(), CssSupportStatus::Partial);
    assert!(ordinary.supported_subset().is_some());
    let remainder = ordinary.unsupported_remainder().expect("valid remainder");
    assert!(remainder.contains("shape()"), "{remainder}");
    assert!(remainder.to_ascii_lowercase().contains("unsupported"));
    assert!(!remainder.starts_with("Unresolved by the selected standard:"));
    assert_eq!(ordinary.recognized_unsupported_code(), None);
}

#[test]
fn five_explicit_alias_mappings_retain_authored_tokens_and_one_target() {
    assert_eq!(grammar().name(), ALIAS);
    assert_eq!(grammar().target_property(), TARGET);
    assert_eq!(
        grammar().feature_id().as_str(),
        "official.property-alias.glyph-orientation-vertical"
    );
    assert_eq!(CssKnownProperty::from_name(ALIAS), None);
    assert!(TARGET.aliases().is_empty());
    let CssPropertyKindRef::Shorthand(metadata) = grammar().metadata().unwrap().kind() else {
        panic!("explicit legacy shorthand grammar")
    };
    assert!(metadata.is_legacy());
    assert_eq!(metadata.settable_members().len(), 1);
    assert_eq!(metadata.settable_members()[0].known_property(), TARGET);
    assert!(metadata.reset_only_members().is_empty());

    for (authored, orientation) in [
        ("auto", CssTextOrientation::Mixed),
        ("0deg", CssTextOrientation::Upright),
        ("90deg", CssTextOrientation::Sideways),
        ("0", CssTextOrientation::Upright),
        ("90", CssTextOrientation::Sideways),
    ] {
        let source = format!("{ALIAS}:{authored}!important");
        let parsed = one(&source);
        let components = parse_component_values(authored).unwrap();
        let checked = parse_property_value_for_grammar(
            grammar(),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        let programmatic_components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(authored).unwrap()])
                .unwrap();
        let programmatic = parse_property_value_for_grammar(
            grammar(),
            programmatic_components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        for declaration in [&parsed, &checked, &programmatic] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration.value_components().serialize().unwrap().as_css(),
                authored
            );
            mapped(declaration, orientation, authored);
            let item = contribution(declaration);
            assert!(matches!(
                item.value(),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::TextOrientation(actual))
                    if actual == &orientation
            ));
        }
        assert!(matches!(
            parsed.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(checked.value_components(), &components);
        assert!(matches!(
            checked.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(programmatic.value_components(), &programmatic_components);
        assert!(matches!(
            programmatic.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
    }
}

#[test]
fn alias_normalization_keeps_authored_order_and_occurrences() {
    let report = parse_sheet(
        ".x{glyph-orientation-vertical:90deg!important;text-orientation:mixed;glyph-orientation-vertical:0}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let sheet = normalize_sheet(report.syntax()).expect("selected intrinsic expansion");
    let declarations: Vec<_> = sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (index, (name, importance)) in [
        (ALIAS, CssImportance::Important),
        ("text-orientation", CssImportance::Normal),
        (ALIAS, CssImportance::Normal),
    ]
    .into_iter()
    .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().grammar().name(), name);
        assert_eq!(item.source().importance(), importance);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("one normalized target")
        };
        let [value] = values.items() else {
            panic!("one target, no reset")
        };
        assert_eq!(value.property(), TARGET);
        assert!(value.source().same_occurrence(item.source()));
    }
    assert!(
        !declarations[0]
            .source()
            .same_occurrence(declarations[2].source())
    );
}

#[test]
fn alias_css_wide_values_and_all_keep_symbolic_intrinsic_boundaries() {
    for (spelling, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let declaration = one(&format!("{ALIAS}:{spelling}!important"));
        assert!(matches!(
            declaration.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(actual) if actual == keyword
        ));
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&declaration).unwrap()
        else {
            panic!("global alias completes")
        };
        let [item] = values.items() else {
            panic!("only text-orientation")
        };
        assert_eq!(item.property(), TARGET);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&declaration));
    }
    let all = one("all:revert-layer!important");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all).unwrap()
    else {
        panic!("all remains a symbolic universal reset")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::RevertLayer);
    assert!(reset.source().same_occurrence(&all));
    assert!(!reset.excludes(CssPropertyNameRef::Known(TARGET)));
}

#[test]
fn alias_var_and_env_reentry_are_strict_retryable_and_provenance_preserving() {
    for substitution in ["var(--glyph)", "env(--glyph)"] {
        let source = one(&format!("{ALIAS}:{substitution}!important"));
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("substitution awaits reentry")
        };
        assert!(handle.source().same_occurrence(&source));
        for invalid in ["45deg", "mixed", "inherit extra"] {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for residual in ["var(--again)", "env(--again)"] {
            assert_eq!(
                handle
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        let replacement = parse_component_values("90deg").unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentry completes the original target")
            };
            let [item] = values.items() else {
                panic!("one target")
            };
            assert_eq!(item.property(), TARGET);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert!(matches!(
                item.value(),
                CssContributionValueRef::Ordinary(CssLonghandValueRef::TextOrientation(actual))
                    if actual == &CssTextOrientation::Sideways
            ));
        }
        let CssContributions::Longhands(values) = handle
            .reenter(parse_component_values("unset").unwrap())
            .unwrap()
        else {
            panic!("CSS-wide replacement completes")
        };
        let [item] = values.items() else {
            panic!("one global target")
        };
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Unset)
        );
        assert!(item.source().same_occurrence(&source));
    }
}

#[test]
fn modern_keywords_stay_distinct_and_invalid_legacy_values_recover() {
    for (spelling, orientation) in [
        ("mixed", CssTextOrientation::Mixed),
        ("upright", CssTextOrientation::Upright),
        ("sideways", CssTextOrientation::Sideways),
    ] {
        let modern = one(&format!("text-orientation:{spelling}"));
        assert_eq!(modern.known().unwrap().property(), TARGET);
        let CssKnownPropertyValueRef::TextOrientation(value) =
            modern.known().unwrap().property_value().unwrap()
        else {
            panic!("modern text-orientation")
        };
        assert_eq!(value.orientation(), &orientation);
        assert_eq!(value.as_css(), spelling);
    }
    for invalid in ["45", "45deg", "180deg", "1.5708rad", "mixed"] {
        let source = format!("color:red;{ALIAS}:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one legacy invalid-value diagnostic: {source}")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2, "neighbors retained: {source}");
        assert!(
            report
                .syntax()
                .iter()
                .all(|item| { item.known().unwrap().property() == CssKnownProperty::Color })
        );
        assert!(
            parse_property_value_for_grammar(
                grammar(),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal,
            )
            .is_err()
        );
    }
}
