#![forbid(unsafe_code)]
//! Logical1 WD20251204 §§2–3; source-authored page selector goldens.
//! Authored page selectors retain their logical classification until progression is known.
use surgeist_css::*;

fn pages(sheet: &CssNormalizedSheet) -> Vec<&CssPageRule> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::Page(page) => Some(page),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn logical_page_pseudos_retain_symbolic_spelling_without_becoming_physical_sides() {
    for (pseudo, canonical) in [
        ("recto", "recto"),
        ("verso", "verso"),
        ("RECTO", "recto"),
        ("VeRsO", "verso"),
        (r"\72 ecto", "recto"),
        (r"\76 erso", "verso"),
    ] {
        for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
            let context = CssParserContext::new(mode);
            let source = format!("@page :{pseudo}{{margin-left:-1px!important}}");
            let report = context.parse_sheet(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let [CssRule::Page(page)] = report.syntax().rules() else {
                panic!("logical page retained")
            };
            assert!(page.selector().is_some());
            assert_eq!(page.declarations().len(), 1);
            assert_eq!(
                page.declarations()[0].importance(),
                CssImportance::Important
            );
            let expected = format!("@page :{canonical} {{ margin-left: -1px !important; }}");
            assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
            assert_eq!(
                report.syntax().rules()[0].to_specified_css().unwrap(),
                expected
            );
            let reparsed = parse_sheet(&expected);
            assert!(reparsed.is_clean());
            assert_eq!(reparsed.syntax().to_specified_css().unwrap(), expected);
            let one = context.parse_rule(&source, &CssNamespaceContext::default());
            assert!(one.is_clean());
            assert_eq!(
                one.syntax().as_ref().unwrap().to_specified_css().unwrap(),
                expected
            );
        }
    }
}

#[test]
fn logical_page_pairs_preserve_neighbors_origins_margin_order_and_normalized_payloads() {
    let source = "/*😀*/\r\n@page :recto{margin-left:1px;margin-right:2%!important}@page :verso{margin-left:3px}.a{float:inline-start}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let [
        CssRule::Page(recto),
        CssRule::Page(verso),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("two pages and style neighbor")
    };
    assert_ne!(recto.selector(), verso.selector());
    assert_eq!(
        recto.position().byte_offset().value(),
        source.find("@page :recto").unwrap()
    );
    assert_eq!(recto.position().line().value(), 1);
    assert_eq!(recto.position().column().value(), 0);
    assert_eq!(
        verso.position().byte_offset().value(),
        source.find("@page :verso").unwrap()
    );
    for (declaration, marker) in [
        (&recto.declarations()[0], "margin-left:1px"),
        (&recto.declarations()[1], "margin-right:2%"),
        (&verso.declarations()[0], "margin-left:3px"),
    ] {
        let offset = source.find(marker).unwrap();
        assert_eq!(
            declaration.position().unwrap().byte_offset().value(),
            offset
        );
        assert_eq!(
            declaration.position().unwrap().column().value() as usize,
            source[source.find('\n').unwrap() + 1..offset]
                .encode_utf16()
                .count()
        );
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(pages(&normalized), vec![recto, verso]);
    // Page margin payloads remain pages; only the element float is contributed.
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 1);
    assert_eq!(
        declarations[0].source().known().unwrap().property(),
        CssKnownProperty::Float
    );
    assert_eq!(declarations[0].order(), 0);
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@page :recto { margin-left: 1px; margin-right: 2% !important; }\n@page :verso { margin-left: 3px; }\n.a { float: inline-start; }"
    );
    assert_eq!(report, before);
}

#[test]
fn logical_pages_compose_in_ordinary_and_scoped_groups_without_element_mapping() {
    let source = "@media print{@page:recto{margin:1px}@page:verso{margin:2px}}@scope(.host){@media all{@page:recto{margin-left:3px}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert_eq!(pages(&normalized).len(), 3);
    assert!(
        normalized
            .items()
            .iter()
            .all(|item| !matches!(item, CssNormalizedItem::Declaration(_)))
    );
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@media print { @page :recto { margin: 1px; } @page :verso { margin: 2px; } }\n@scope (.host) { @media all { @page :recto { margin-left: 3px; } } }"
    );
}

#[test]
fn logical_page_body_recovery_keeps_valid_margin_payload_and_siblings() {
    let source =
        ".before{}@page:recto{margin-left:1px;margin-right:bogus;color:red;margin-top:2%} .after{}";
    let report = parse_sheet(source);
    let [CssRule::Style(_), CssRule::Page(page), CssRule::Style(_)] = report.syntax().rules()
    else {
        panic!("page and neighbors")
    };
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropDeclaration)
    );
    assert_eq!(page.declarations().len(), 2);
    assert_eq!(
        page.declarations()[0].known().unwrap().property(),
        CssKnownProperty::MarginLeft
    );
    assert_eq!(
        page.declarations()[1].known().unwrap().property(),
        CssKnownProperty::MarginTop
    );
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".before { }\n@page :recto { margin-left: 1px; margin-top: 2%; }\n.after { }"
    );
    let eof = parse_sheet("@page:verso{margin-left:1px");
    assert!(matches!(eof.syntax().rules(), [CssRule::Page(_)]));
    assert_eq!(eof.diagnostics().len(), 1);
    assert_eq!(
        eof.diagnostics()[0].action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        eof.syntax().to_specified_css().unwrap(),
        "@page :verso { margin-left: 1px; }"
    );
}

#[test]
fn malformed_page_pseudos_drop_each_rule_atomically_and_ordinary_pages_are_controls() {
    for prelude in [
        ":recto:verso",
        ":verso:left",
        ":recto extra",
        "named:recto",
        ":recto()",
        "::verso",
        ":unknown",
    ] {
        let source =
            format!(".before{{}}@page {prelude}{{margin:1px}}@page:right{{margin:2px}}.after{{}}");
        let report = parse_sheet(&source);
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Style(_), CssRule::Page(_), CssRule::Style(_)]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("one malformed page")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePrelude
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            report.syntax().to_specified_css().unwrap(),
            ".before { }\n@page :right { margin: 2px; }\n.after { }"
        );
    }
    for pseudo in ["left", "right", "first"] {
        let report = parse_sheet(&format!("@page:{pseudo}{{margin:1px}}"));
        assert!(report.is_clean());
        assert_eq!(
            report.syntax().to_specified_css().unwrap(),
            format!("@page :{pseudo} {{ margin: 1px; }}")
        );
    }
    let statement = parse_sheet("@page:recto; .after{}");
    assert!(matches!(statement.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(statement.diagnostics().len(), 1);
    assert_eq!(
        statement.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn logical_page_names_do_not_become_property_keywords_or_style_block_rules() {
    for pseudo in ["recto", "verso"] {
        let report = parse_sheet(&format!(
            ".host{{@page:{pseudo}{{margin:1px}}color:red}} .after{{}}"
        ));
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Style(_), CssRule::Style(_)]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("invalid style-block page")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            report.syntax().to_specified_css().unwrap(),
            ".host { color: red; }\n.after { }"
        );
        let report = parse_style_attribute(&format!("float:{pseudo};color:red"));
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
    // Generic selector semantics belong to their owner; no .a:recto assumption.
}

#[test]
fn selected_directional_consumers_remain_symbolic_and_reject_forbidden_keyword_mixtures() {
    for (name, text, canonical) in [
        ("float", "INLINE-START", "inline-start"),
        ("float", "inline-end", "inline-end"),
        ("clear", "inline-start", "inline-start"),
        ("clear", "INLINE-END", "inline-end"),
        ("text-align", "START", "start"),
        ("text-align", "end", "end"),
        ("page-break-before", "recto", "recto"),
        ("page-break-after", "verso", "verso"),
    ] {
        let source = format!("{name}:{text}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(
            report.syntax()[0].to_specified_css().unwrap(),
            format!("{name}: {canonical};")
        );
        assert_eq!(
            report.syntax()[0].parsed_value().unwrap().source().as_str(),
            source
        );
    }
    for (name, text) in [
        ("float", "start"),
        ("clear", "end"),
        ("float", "block-start"),
        ("text-align", "inline-start"),
        ("float", "left inline-start"),
        ("clear", "right inline-end"),
        ("text-align", "left start"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{text};color:red"));
        assert!(!report.is_clean());
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
    // caption-side logical additions are conditional on left/right support.
    for text in ["top", "bottom"] {
        assert!(parse_style_attribute(&format!("caption-side:{text}")).is_clean());
    }
}

#[test]
fn logical_and_physical_property_occurrences_keep_relative_order_and_original_positions() {
    let source = ".a{margin-left:1px;margin-inline-start:2px;margin-top:3px;margin-block-start:4px!important}";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    for (order, (name, declaration)) in [
        "margin-left",
        "margin-inline-start",
        "margin-top",
        "margin-block-start",
    ]
    .into_iter()
    .zip(declarations.iter())
    .enumerate()
    {
        assert_eq!(declaration.order(), order);
        assert_eq!(
            declaration.source().known().unwrap().property(),
            CssKnownProperty::from_name(name).unwrap()
        );
        assert_eq!(
            declaration
                .source()
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source.find(name).unwrap()
        );
    }
    assert_eq!(declarations.len(), 4);
    assert_eq!(
        declarations[3].source().importance(),
        CssImportance::Important
    );
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".a { margin-left: 1px; margin-inline-start: 2px; margin-top: 3px; margin-block-start: 4px !important; }"
    );
}

#[test]
fn logical_page_output_and_normalization_share_atomic_cumulative_limits() {
    let report = parse_sheet("@page:recto{margin-left:1px}@page:verso{margin-left:2px}");
    assert!(report.is_clean());
    let before = report.clone();
    let expected = "@page :recto { margin-left: 1px; }\n@page :verso { margin-left: 2px; }";
    // Page rule + pseudo + declaration-list + declaration + name + numeric leaf.
    for rule in report.syntax().rules() {
        assert!(
            rule.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                6,
                6,
                expected.len()
            ))
            .is_ok()
        );
    }
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, usize::MAX, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            let error = report
                .syntax()
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(kind)
            );
            assert_eq!(report, before);
        }
    }
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let exact = CssNormalizationLimits::try_new(0, 2, 0, 0).unwrap();
    assert_eq!(
        pages(&normalize_sheet_with_limits(report.syntax(), exact).unwrap()).len(),
        2
    );
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 0, 0).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Rules,
            limit: 1
        }
    );
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
