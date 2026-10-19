#![forbid(unsafe_code)]

//! CSS Generated Content 3 §2.4.1, selected 2025-12-04 Working Draft:
//! https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-quotes
//! `quotes` accepts `auto | none | match-parent | [<string> <string>]+`.
//! Its inherited initial `auto` remains symbolic until language is known.

use surgeist_css::*;

fn declaration(value: &str) -> CssDeclaration {
    let source = format!("quotes:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one quotes declaration: {source}")
    };
    declaration.clone()
}

fn expansion(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("quotes expands to one terminal")
    };
    let [item] = items.items() else {
        panic!("one quotes contribution")
    };
    item.clone()
}

#[test]
fn content_three_quotes_is_an_inherited_auto_longhand_with_stable_identity() {
    let feature = feature_metadata("official.property.quotes").unwrap();
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.source().id().as_str(), "X-CONTENT3");
    assert_eq!(feature.production(), "#propdef-quotes");
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::Quotes.metadata().unwrap().kind()
    else {
        panic!("quotes is one longhand")
    };
    assert_eq!(
        metadata.property().known_property(),
        CssKnownProperty::Quotes
    );
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("quotes has a symbolic auto initial")
    };
    let auto = declaration("auto");
    assert_eq!(initial, expansion(&auto).ordinary_value().unwrap());
}

#[test]
fn three_keywords_and_string_pairs_parse_as_distinct_authored_values() {
    for keyword in ["auto", "none", "match-parent"] {
        let source = declaration(keyword);
        let CssKnownPropertyValueRef::Quotes(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("quotes wrapper")
        };
        assert_eq!(value.as_css(), keyword);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            keyword
        );
        let item = expansion(&source);
        assert_eq!(item.property(), CssKnownProperty::Quotes);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let source = declaration("\"«\" \"»\" \"‹\" \"›\"");
    let CssKnownPropertyValueRef::Quotes(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("quotes wrapper")
    };
    let CssQuotes::Pairs(pairs) = value.quotes() else {
        panic!("two authored pairs")
    };
    assert_eq!(pairs.pairs().len(), 2);
    assert_eq!(pairs.pairs()[0].open().as_str(), "«");
    assert_eq!(pairs.pairs()[0].close().as_str(), "»");
    assert_eq!(pairs.pairs()[1].open().as_str(), "‹");
    assert_eq!(pairs.pairs()[1].close().as_str(), "›");
}

#[test]
fn escaped_and_empty_strings_are_values_but_odd_pairs_are_invalid() {
    let escaped = declaration(r#""\22 " "\27 ""#);
    let CssKnownPropertyValueRef::Quotes(value) =
        escaped.known().unwrap().property_value().unwrap()
    else {
        panic!("escaped quotes")
    };
    let CssQuotes::Pairs(pairs) = value.quotes() else {
        panic!("escaped pair")
    };
    assert_eq!(pairs.pairs()[0].open().as_str(), "\"");
    assert_eq!(pairs.pairs()[0].close().as_str(), "'");
    let empty = declaration("\"\" \"\"");
    let CssKnownPropertyValueRef::Quotes(value) = empty.known().unwrap().property_value().unwrap()
    else {
        panic!("empty pair")
    };
    let CssQuotes::Pairs(pairs) = value.quotes() else {
        panic!("empty strings are still a pair")
    };
    assert_eq!(pairs.pairs()[0].open().as_str(), "");
    assert_eq!(pairs.pairs()[0].close().as_str(), "");

    for invalid in [
        "",
        "\"open\"",
        "\"a\" \"b\" \"c\"",
        "auto \"a\" \"b\"",
        "match-parent \"a\" \"b\"",
        "none \"a\" \"b\"",
        "auto match-parent",
    ] {
        let source = format!("color:red;quotes:{invalid};height:2px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Height
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one whole-declaration diagnostic: {invalid}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(validate_style_attribute(&source).is_err());
    }
}

#[test]
fn css_wide_keywords_expand_to_one_symbolic_terminal() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(spelling);
        let item = expansion(&source);
        assert_eq!(item.property(), CssKnownProperty::Quotes);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_reentry_is_strict_repeatable_and_preserves_replacement_provenance() {
    let source = declaration("var(--quotes)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("quotes substitution is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["\"open\"", "auto \"a\" \"b\"", "match-parent none"] {
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
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    let replacement = parse_component_values("match-parent").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one reentered longhand")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::Quotes);
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssContributions::Longhands(items) = pending
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("reentered symbolic global")
    };
    assert_eq!(items.items().len(), 1);
    assert_eq!(
        items.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn normalized_quotes_preserve_order_and_fail_atomically_at_contribution_limit() {
    let report = parse_sheet(".a{quotes:auto;color:red;quotes:\"<\" \">\"}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, property) in [
        CssKnownProperty::Quotes,
        CssKnownProperty::Color,
        CssKnownProperty::Quotes,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            property
        );
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[index].expansion()
        else {
            panic!("one terminal")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(declarations[index].source()));
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
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
        CssKnownProperty::Quotes
    );
}
