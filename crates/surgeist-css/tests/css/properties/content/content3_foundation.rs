#![forbid(unsafe_code)]

//! Existing-API expectations from CSS Generated Content 3 (2025-12-04) §2:
//! https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-content
//! Counter-style names and `symbols()` follow Counter Styles 3 (2021-07-27).

use surgeist_css::*;

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("content:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one content declaration: {source}")
    };
    declaration.clone()
}

fn ordinary(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("content expands to one longhand")
    };
    let [item] = items.items() else {
        panic!("one content contribution")
    };
    item.clone()
}

fn assert_recovered(invalid: &str) {
    let source = format!("color:red;content:{invalid};height:2px");
    let report = parse_style_attribute(&source);
    let properties = report
        .syntax()
        .iter()
        .map(|declaration| declaration.known().unwrap().property())
        .collect::<Vec<_>>();
    assert_eq!(
        properties,
        [CssKnownProperty::Color, CssKnownProperty::Height],
        "{invalid}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one whole-declaration diagnostic for {invalid}: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(validate_style_attribute(&source).is_err());
}

#[test]
fn content_is_one_noninherited_longhand_with_symbolic_normal_initial() {
    let feature = feature_metadata("baseline.property.content").unwrap();
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.source().id().as_str(), "X-CONTENT3");
    assert_eq!(feature.production(), "#propdef-content");
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::Content.metadata().unwrap().kind()
    else {
        panic!("content is a longhand")
    };
    assert!(!metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("content has a normal initial")
    };
    assert_eq!(
        initial,
        ordinary(&parsed("normal")).ordinary_value().unwrap()
    );
}

#[test]
fn existing_normal_none_strings_counters_and_attr_pending_remain_admitted() {
    for value in [
        "normal",
        "none",
        "\"caption\"",
        "\"caption\" open-quote counter(chapter) counters(chapter, \".\") url(\"#icon\")",
    ] {
        let source = parsed(value);
        assert_eq!(
            source.known().unwrap().property(),
            CssKnownProperty::Content
        );
        assert!(source.known().unwrap().property_value().is_some());
    }
    let attr = parsed("\"caption\" attr(data-label)");
    assert!(attr.known().unwrap().substitution_dependent().is_some());
    assert_eq!(
        attr.value_components().serialize().unwrap().as_css(),
        "\"caption\" attr(data-label)"
    );
}

#[test]
fn generated_content_replacement_lists_and_alternatives_are_admitted() {
    for value in [
        "contents",
        "contents contents",
        "linear-gradient(red, blue)",
        "repeating-linear-gradient(red, blue)",
        "repeating-radial-gradient(red, blue)",
        "radial-gradient(red, blue) \"caption\" url(\"#icon\")",
        "\"caption\" / \"spoken\" counter(chapter)",
        "url(\"#icon\") / \"spoken\"",
        "src(\"#icon\") \"caption\" / counters(chapter, \".\")",
        "leader(dotted) leader(solid) leader(space) leader(\"·\")",
    ] {
        let source = parsed(value);
        let known = source.known().unwrap();
        assert_eq!(known.property(), CssKnownProperty::Content);
        assert!(known.property_value().is_some(), "{value}");
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value,
            "retained authored form: {value}"
        );
    }
}

#[test]
fn content_three_function_families_accept_their_finite_arguments() {
    for value in [
        "target-counter(\"#ch\", chapter)",
        "target-counter(url(\"#ch\"), chapter, decimal)",
        "target-counter(\"#ch\", none) target-counter(\"#ch\", auto)",
        "target-counter(\"#ch\", span, symbols(cyclic \"*\"))",
        "target-counters(\"#ch\", chapter, \".\")",
        "target-counters(url(\"#ch\"), chapter, \".\", symbols(numeric \"0\" \"1\"))",
        "target-text(\"#ch\") target-text(\"#ch\", content) target-text(\"#ch\", before)",
        "target-text(\"#ch\", after) target-text(url(\"#ch\"), first-letter)",
        "string(none) string(span, first-except) string(auto, start)",
        "string(chapter, first) string(chapter, last)",
        r##"string(\73 pan) target-counter("#ch", \61 uto)"##,
        r##"string(chapter\ name) target-counter("#ch", chapter\ name)"##,
        r##"string(\31 chapter) target-counter("#ch", \31 chapter)"##,
        "content() content(text) content(before) content(after) content(first-letter) content(marker)",
        "counter(chapter, symbols(alphabetic \"a\" \"b\"))",
        "counter(chapter, symbols(\"*\"))",
        "counter(chapter, symbols(fixed \"a\"))",
        r"counter(chapter\ name) counter(\31 chapter)",
        "counters(chapter, \".\", symbols(symbolic url(\"#star\")))",
    ] {
        let source = parsed(value);
        assert_eq!(
            source.known().unwrap().property(),
            CssKnownProperty::Content
        );
    }
}

#[test]
fn ordinary_counter_names_reject_decoded_reserved_and_default_names() {
    for invalid in [
        "counter(default)",
        "counters(DEFAULT, \".\")",
        r"counter(\64 efault)",
    ] {
        assert_recovered(invalid);
    }
}

#[test]
fn invalid_combinations_and_function_arguments_recover_without_losing_neighbors() {
    for invalid in [
        "normal contents",
        "none \"caption\"",
        "normal / \"spoken\"",
        "\"caption\" /",
        "\"caption\" / url(\"#icon\")",
        "\"caption\" / open-quote",
        "\"caption\" / contents",
        "\"caption\" / leader(dotted)",
        "leader()",
        "leader(dashed)",
        "target-counter(\"#ch\")",
        "target-counter(\"#ch\", chapter, decimal, extra)",
        "target-counters(\"#ch\", chapter)",
        "target-text(\"#ch\", marker)",
        "string(chapter, middle)",
        "content(chapter)",
        "counter(none)",
        r"counter(\69 nherit)",
        "target-counter(\"#ch\", default)",
        "string(INHERIT)",
        r"string(\64 efault)",
        "counter(chapter, symbols())",
        "counter(chapter, symbols(numeric \"0\"))",
        "counter(chapter, symbols(alphabetic \"a\"))",
        "counter(chapter, symbols(cyclic none))",
        "counter(chapter, symbols(symbolic star))",
        "target-counter(\"#ch\", chapter,)",
    ] {
        assert_recovered(invalid);
    }
}

#[test]
fn global_keywords_have_one_terminal_and_preserve_source_importance() {
    for (spelling, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = parsed(spelling);
        let item = ordinary(&source);
        assert_eq!(item.property(), CssKnownProperty::Content);
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let source = parsed("\"caption\"");
    let item = ordinary(&source);
    assert_eq!(item.property(), CssKnownProperty::Content);
    assert!(item.ordinary_value().is_some());
    assert!(item.source().same_occurrence(&source));
}

#[test]
fn pending_content_reentry_is_strict_and_retains_replacement_provenance() {
    let source = parsed("var(--generated)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("content substitution is pending")
    };
    assert!(pending.source().same_occurrence(&source));
    for invalid in ["normal contents", "\"caption\" /", "leader()"] {
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
    for replacement_css in [
        "contents",
        "linear-gradient(red, blue) / \"spoken\"",
        "inherit",
    ] {
        let replacement = parse_component_values(replacement_css).unwrap();
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one reentered content terminal")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::Content);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn normalized_content_preserves_order_and_reports_atomic_limit_failure() {
    let report = parse_sheet(".a{content:normal;color:red;content:contents}");
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
    for (order, property) in [
        CssKnownProperty::Content,
        CssKnownProperty::Color,
        CssKnownProperty::Content,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[order].order(), order);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[order].expansion()
        else {
            panic!("one normalized contribution")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(declarations[order].source()));
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
        CssKnownProperty::Content
    );
}
