#![forbid(unsafe_code)]

//! Flexbox 1 Appendix B (normative), 2025-10-14: exactly twelve `-webkit-`
//! legacy property names alias their standard names, without new value grammars.
//! https://www.w3.org/TR/2025/CRD-css-flexbox-1-20251014/#webkit-aliases

use surgeist_css::*;

const CASES: &[(&str, &str, &str, CssKnownProperty)] = &[
    (
        "-webkit-align-content",
        "align-content",
        "space-between",
        CssKnownProperty::AlignContent,
    ),
    (
        "-webkit-align-items",
        "align-items",
        "safe center",
        CssKnownProperty::AlignItems,
    ),
    (
        "-webkit-align-self",
        "align-self",
        "auto",
        CssKnownProperty::AlignSelf,
    ),
    ("-webkit-flex", "flex", "2", CssKnownProperty::Flex),
    (
        "-webkit-flex-basis",
        "flex-basis",
        "10px",
        CssKnownProperty::FlexBasis,
    ),
    (
        "-webkit-flex-direction",
        "flex-direction",
        "column",
        CssKnownProperty::FlexDirection,
    ),
    (
        "-webkit-flex-flow",
        "flex-flow",
        "wrap-reverse column",
        CssKnownProperty::FlexFlow,
    ),
    (
        "-webkit-flex-grow",
        "flex-grow",
        "2",
        CssKnownProperty::FlexGrow,
    ),
    (
        "-webkit-flex-shrink",
        "flex-shrink",
        "3",
        CssKnownProperty::FlexShrink,
    ),
    (
        "-webkit-flex-wrap",
        "flex-wrap",
        "wrap-reverse",
        CssKnownProperty::FlexWrap,
    ),
    (
        "-webkit-justify-content",
        "justify-content",
        "space-evenly",
        CssKnownProperty::JustifyContent,
    ),
    ("-webkit-order", "order", "-2", CssKnownProperty::Order),
];

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).expect("alias grammar"),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("canonical terminal contributions")
    };
    items.items().to_vec()
}

fn targets(property: CssKnownProperty) -> &'static [CssKnownProperty] {
    match property {
        CssKnownProperty::Flex => &[
            CssKnownProperty::FlexGrow,
            CssKnownProperty::FlexShrink,
            CssKnownProperty::FlexBasis,
        ],
        CssKnownProperty::FlexFlow => {
            &[CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap]
        }
        CssKnownProperty::AlignContent => &[CssKnownProperty::AlignContent],
        CssKnownProperty::AlignItems => &[CssKnownProperty::AlignItems],
        CssKnownProperty::AlignSelf => &[CssKnownProperty::AlignSelf],
        CssKnownProperty::FlexBasis => &[CssKnownProperty::FlexBasis],
        CssKnownProperty::FlexDirection => &[CssKnownProperty::FlexDirection],
        CssKnownProperty::FlexGrow => &[CssKnownProperty::FlexGrow],
        CssKnownProperty::FlexShrink => &[CssKnownProperty::FlexShrink],
        CssKnownProperty::FlexWrap => &[CssKnownProperty::FlexWrap],
        CssKnownProperty::JustifyContent => &[CssKnownProperty::JustifyContent],
        CssKnownProperty::Order => &[CssKnownProperty::Order],
        _ => panic!("not an Appendix B property"),
    }
}

fn specified(item: &CssLonghandContribution) -> String {
    let value = item.ordinary_value().expect("ordinary terminal");
    match value.view() {
        CssLonghandValueRef::AlignContent(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::AlignItems(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::AlignSelf(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::FlexBasis(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::FlexDirection(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::FlexGrow(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::FlexShrink(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::FlexWrap(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::JustifyContent(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::Order(value) => value.serialize_specified().unwrap(),
        _ => panic!("unexpected Appendix B terminal"),
    }
}

#[test]
fn normative_twelve_names_resolve_to_their_existing_canonical_grammars_and_support() {
    for &(alias, canonical, _, property) in CASES {
        assert_eq!(
            CssKnownProperty::from_name(alias),
            Some(property),
            "{alias}"
        );
        assert_eq!(
            CssKnownProperty::from_name(&alias.to_ascii_uppercase()),
            Some(property),
            "{alias}"
        );
        let grammar = CssPropertyGrammar::from_name(alias).expect("alias grammar");
        let standard = CssPropertyGrammar::from_name(canonical).unwrap();
        assert_eq!(grammar, standard, "{alias}");
        assert_eq!(grammar.target_property(), property);
        assert_eq!(grammar.name(), canonical);
        assert_eq!(grammar.feature_id(), standard.feature_id());
        assert_eq!(property.aliases(), &[alias]);
        for name in [alias, canonical] {
            let support = property_support_metadata(name).expect("canonical support lookup");
            assert_eq!(support.property(), property);
            assert_eq!(support.canonical_name(), canonical);
            assert_eq!(support.aliases(), &[alias]);
            assert_eq!(support.feature().status(), CssSupportStatus::Complete);
            assert_eq!(support.feature().id(), standard.feature_id());
            let source = if canonical.starts_with("align-") || canonical == "justify-content" {
                "S-ALIGN3"
            } else if canonical == "order" {
                "S-DISPLAY3"
            } else {
                "O-FLEXBOX1"
            };
            assert_eq!(support.feature().source().id().as_str(), source);
        }
    }
    assert!(conformance_exclusion("excluded.O-FLEXBOX1.webkit-legacy").is_none());
}

#[test]
fn every_alias_parses_and_checked_construction_expands_to_exact_canonical_terminals() {
    for &(alias, _, value, property) in CASES {
        for source in [declaration(alias, value), checked(alias, value)] {
            assert_eq!(source.known().unwrap().property(), property);
            assert_eq!(source.importance(), CssImportance::Important);
            let items = expanded(&source);
            assert_eq!(
                items.iter().map(|item| item.property()).collect::<Vec<_>>(),
                targets(property)
            );
            let expected_values: &[&str] = match property {
                CssKnownProperty::Flex => &["2", "1", "0"],
                CssKnownProperty::FlexFlow => &["column", "wrap-reverse"],
                _ => &[value],
            };
            assert_eq!(
                items.iter().map(specified).collect::<Vec<_>>(),
                expected_values
            );
            for item in items {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn mixed_case_aliases_keep_authored_name_origin_in_style_and_sheet_order() {
    let source = "-WeBkIt-FlEx-GrOw:2!important;flex-grow:3;-webkit-flex-flow:wrap-reverse column";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 3);
    let first = &report.syntax()[0];
    assert_eq!(
        first.known().unwrap().property(),
        CssKnownProperty::FlexGrow
    );
    let origin = first.parsed_name().expect("authored alias origin");
    let span = origin.span();
    assert_eq!(
        &origin.source().as_str()
            [span.start().byte_offset().value()..span.end().byte_offset().value()],
        "-WeBkIt-FlEx-GrOw"
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::FlexGrow
    );
    assert_eq!(
        report.syntax()[2].known().unwrap().property(),
        CssKnownProperty::FlexFlow
    );
    assert!(!first.same_occurrence(&report.syntax()[1]));

    let sheet = parse_sheet(&format!(".x{{{source}}}"));
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
    let normalized = normalize_sheet(sheet.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, expected) in [
        &[CssKnownProperty::FlexGrow][..],
        &[CssKnownProperty::FlexGrow][..],
        &[CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap][..],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[index].expansion()
        else {
            panic!("normalized aliases")
        };
        assert_eq!(
            items
                .items()
                .iter()
                .map(|item| item.property())
                .collect::<Vec<_>>(),
            expected
        );
        for item in items.items() {
            assert!(item.source().same_occurrence(declarations[index].source()));
        }
    }
    assert_eq!(
        declarations[0].source().importance(),
        CssImportance::Important
    );
    assert_eq!(declarations[1].source().importance(), CssImportance::Normal);
}

#[test]
fn aliases_keep_all_five_css_wide_keywords_symbolic_for_every_terminal() {
    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for &(alias, _, _, property) in CASES {
            let source = declaration(alias, spelling);
            let items = expanded(&source);
            assert_eq!(
                items.iter().map(|item| item.property()).collect::<Vec<_>>(),
                targets(property)
            );
            for item in items {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_aliases_reenter_canonical_grammars_strictly_with_original_provenance() {
    for (alias, valid, invalid, expected) in [
        (
            "-webkit-flex",
            "2",
            "-1",
            &[
                CssKnownProperty::FlexGrow,
                CssKnownProperty::FlexShrink,
                CssKnownProperty::FlexBasis,
            ][..],
        ),
        (
            "-webkit-flex-flow",
            "wrap-reverse column",
            "row column",
            &[CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap][..],
        ),
        (
            "-webkit-align-items",
            "safe center",
            "safe left",
            &[CssKnownProperty::AlignItems][..],
        ),
    ] {
        let source = declaration(alias, "var(--pending)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending alias")
        };
        assert!(pending.source().same_occurrence(&source));
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(valid).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentered alias")
            };
            assert_eq!(
                items
                    .items()
                    .iter()
                    .map(|item| item.property())
                    .collect::<Vec<_>>(),
                expected
            );
            for item in items.items() {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn invalid_alias_values_drop_atomically_and_unlisted_webkit_names_remain_unknown() {
    for (alias, invalid) in [
        ("-webkit-flex-grow", "-1"),
        ("-webkit-flex-flow", "row column"),
        ("-webkit-justify-content", "baseline"),
        ("-webkit-order", "1.5"),
    ] {
        let source = format!("color:red;{alias}:{invalid};height:2px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Height
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid alias value")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    }
    let report = parse_style_attribute("color:red;-webkit-box-flex:2;height:2px");
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("unknown prefixed name")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(CssPropertyGrammar::from_name("-webkit-box-flex").is_none());
}
