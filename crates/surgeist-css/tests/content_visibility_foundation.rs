#![forbid(unsafe_code)]

//! Authored `content-visibility` from CSS Containment 2 WD (2022-09-17) §4.
//! Skipping, user relevance, and used containment require downstream context.

use surgeist_css::*;

const NAME: &str = "content-visibility";

fn property() -> CssKnownProperty {
    CssKnownProperty::from_name(NAME).expect("content-visibility is recognized")
}

fn declaration(value: &str) -> CssDeclaration {
    let text = format!("{NAME}:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one authored declaration: {text}")
    };
    assert!(validate_style_attribute(&text).is_ok());
    source.clone()
}

fn contribution(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("content-visibility contributes one terminal")
    };
    let [item] = values.items() else {
        panic!("one content-visibility terminal")
    };
    item.clone()
}

#[test]
fn content_visibility_is_a_noninherited_containment2_longhand_initially_visible() {
    let property = property();
    let grammar = CssPropertyGrammar::from_name(NAME).unwrap();
    assert_eq!(grammar.target_property(), property);
    assert_eq!(grammar.name(), NAME);
    assert_eq!(
        grammar.feature_id().as_str(),
        "baseline.property.content-visibility"
    );
    let support = property_support_metadata(NAME).unwrap();
    assert_eq!(support.property(), property);
    assert_eq!(support.feature().source().id().as_str(), "I-CONTAIN2");
    assert_eq!(
        support.feature().source().url(),
        Some("https://www.w3.org/TR/2022/WD-css-contain-2-20220917/")
    );
    assert_eq!(
        support.feature().production(),
        "#propdef-content-visibility"
    );

    let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
        panic!("content-visibility is a longhand")
    };
    assert_eq!(metadata.property().known_property(), property);
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("visible is an intrinsic ordinary value")
    };
    assert_eq!(
        initial,
        contribution(&declaration("visible"))
            .ordinary_value()
            .unwrap()
    );
}

#[test]
fn only_three_keywords_construct_exact_typed_authored_values_and_one_terminal() {
    let mut terminals = Vec::new();
    for (text, expected) in [
        ("visible", CssContentVisibility::Visible),
        ("HIDDEN", CssContentVisibility::Hidden),
        (r"a\75 to", CssContentVisibility::Auto),
    ] {
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property()),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        for source in [declaration(text), checked] {
            let CssKnownPropertyValueRef::ContentVisibility(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("checked content-visibility value")
            };
            assert_eq!(value.i01_subset(), Some(&expected));
            assert_eq!(value.as_css(), text);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                text
            );
            assert_eq!(source.importance(), CssImportance::Important);
            let item = contribution(&source);
            assert_eq!(item.property(), property());
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            terminals.push(item.ordinary_value().unwrap().clone());
        }
    }
    assert_eq!(terminals[0], terminals[1]);
    assert_eq!(terminals[2], terminals[3]);
    assert_eq!(terminals[4], terminals[5]);
    assert_ne!(terminals[0], terminals[2]);
    assert_ne!(terminals[0], terminals[4]);
    assert_ne!(terminals[2], terminals[4]);
}

#[test]
fn invalid_values_drop_only_their_occurrence_and_fail_checked_construction() {
    for value in [
        "",
        "collapse",
        "none",
        "auto hidden",
        "visible,hidden",
        "10px",
    ] {
        let text = format!("color:red;{NAME}:{value};color:blue");
        let report = parse_style_attribute(&text);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid-value diagnostic: {text}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(report.syntax().len(), 2, "valid neighbors survive: {text}");
        assert!(validate_style_attribute(&text).is_err());
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property()),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked construction accepted {value:?}"
        );
    }
}

#[test]
fn css_wide_keywords_and_all_keep_symbolic_reset_semantics() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(text);
        assert_eq!(
            contribution(&source).value(),
            CssContributionValueRef::Global(keyword)
        );
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property()),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(
            contribution(&checked).value(),
            CssContributionValueRef::Global(keyword)
        );
    }
    let source = parse_style_attribute("all:unset!important");
    assert!(source.is_clean());
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source.syntax()[0]).unwrap()
    else {
        panic!("all remains symbolic")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::Unset);
    assert!(!reset.excludes(CssPropertyNameRef::Known(property())));
}

#[test]
fn pending_substitutions_reenter_whole_grammar_without_losing_original_occurrence() {
    for pending_text in [
        "var(--visibility)",
        "env(--visibility)",
        "attr(data-visibility)",
    ] {
        let source = declaration(pending_text);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("{pending_text} remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        for invalid in ["", "collapse", "auto hidden"] {
            assert!(matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        for unresolved in ["var(--again)", "env(--again)", "attr(data-again)"] {
            assert_eq!(
                pending
                    .reenter(parse_component_values(unresolved).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        for (replacement_text, expected) in [
            ("hidden", CssContentVisibility::Hidden),
            ("auto", CssContentVisibility::Auto),
        ] {
            let replacement = parse_component_values(replacement_text).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("valid keyword completes the pending declaration")
            };
            let [item] = values.items() else {
                panic!("one substituted terminal")
            };
            assert_eq!(item.property(), property());
            assert_eq!(
                item.ordinary_value(),
                contribution(&declaration(replacement_text)).ordinary_value()
            );
            let replacement_source = declaration(replacement_text);
            let CssKnownPropertyValueRef::ContentVisibility(parsed) = replacement_source
                .known()
                .unwrap()
                .property_value()
                .unwrap()
            else {
                panic!("valid replacement has its expected typed value")
            };
            assert_eq!(parsed.i01_subset(), Some(&expected));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssContributions::Longhands(retried) =
                pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("same replacement remains retryable")
            };
            assert_eq!(retried.items()[0].ordinary_value(), item.ordinary_value());
            assert!(retried.items()[0].source().same_occurrence(&source));
            assert_eq!(
                retried.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
        let CssContributions::Longhands(values) = pending
            .reenter(parse_component_values("revert-layer").unwrap())
            .unwrap()
        else {
            panic!("whole-value CSS-wide replacement completes")
        };
        assert_eq!(
            values.items()[0].value(),
            CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
        );
        assert!(pending.source().same_occurrence(&source));
    }
}

#[test]
fn normalization_preserves_valid_order_and_fails_atomically_at_contribution_budget() {
    let text = ".a{content-visibility:visible;content-visibility:collapse;content-visibility:auto!important;content-visibility:hidden}";
    let report = parse_sheet(text);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert!(validate_sheet(text).is_err());
    let original = report.syntax().clone();
    let limits = CssNormalizationLimits::try_new(0, 1, 3, 2).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2,
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        property()
    );
    assert_eq!(report.syntax(), &original);

    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 3, 3).unwrap(),
    )
    .unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (order, (item, (keyword, importance))) in declarations
        .iter()
        .zip([
            ("visible", CssImportance::Normal),
            ("auto", CssImportance::Important),
            ("hidden", CssImportance::Normal),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert_eq!(item.source().importance(), importance);
        assert_eq!(
            item.source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            keyword
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("normalized declaration has one terminal")
        };
        assert_eq!(values.items().len(), 1);
        assert!(values.items()[0].source().same_occurrence(item.source()));
    }
}
