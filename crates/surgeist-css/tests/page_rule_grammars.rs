use surgeist_css::{
    CssErrorCode, CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssMarginValue,
    CssPagePseudo, CssRecoveryAction, CssRule, CssSupportStatus, ErrorKind, feature_metadata,
    parse_sheet,
};

#[test]
fn page_rules_and_pseudos_retain_valid_authored_structure() {
    let report = parse_sheet(concat!(
        "@import \"print.css\"; ",
        "@page { margin: 1cm; } ",
        "@page :left { margin-left: -12mm !important; } ",
        "@page :right { margin-right: 10%; } ",
        "@page :first { margin-top: auto; margin-bottom: 0; }",
    ));

    assert!(
        report.is_clean(),
        "valid page rules and pseudos should not recover: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 5);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.error().code() != CssErrorCode::UnsupportedAtRule)
    );

    let [
        CssRule::Import(_),
        CssRule::Page(default),
        CssRule::Page(left),
        CssRule::Page(right),
        CssRule::Page(first),
    ] = report.syntax().rules()
    else {
        panic!("expected the import and four page rules")
    };
    assert_eq!(page_pseudo(default), None);
    assert_eq!(page_pseudo(left), Some(CssPagePseudo::Left));
    assert_eq!(page_pseudo(right), Some(CssPagePseudo::Right));
    assert_eq!(page_pseudo(first), Some(CssPagePseudo::First));
    assert_eq!(left.declarations().properties().len(), 1);
    assert_eq!(
        left.declarations().properties()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginLeft
    );
    assert_eq!(
        left.declarations().properties()[0].importance(),
        CssImportance::Important
    );
    assert_eq!(first.declarations().properties().len(), 2);
    assert_eq!(default.position().byte_offset().value(), 21);
}

#[test]
fn page_margin_composition_keeps_exact_literals_and_math_but_rejects_logical_sides() {
    let source =
        "@page { margin-left:-1e999px; margin-right:1e-999%; margin-top:auto; margin-bottom:0 }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("one page rule")
    };
    assert_eq!(page.declarations().properties().len(), 4);
    let Some(CssKnownPropertyValueRef::MarginLeft(left)) = page.declarations().properties()[0]
        .known()
        .and_then(|known| known.property_value())
    else {
        panic!("exact signed page margin")
    };
    let CssMarginValue::LengthPercentage(left) = left.value() else {
        panic!("signed length")
    };
    assert_eq!(left.literal_component().unwrap().origin(), left.origin());
    let specified = left.serialize_specified().unwrap();
    assert_eq!(specified.len(), 1003);
    assert!(specified.starts_with("-1") && specified.ends_with("px"));

    let invalid = parse_sheet(
        "@page { margin-top:1px; margin:logical 1px; margin-right:calc(1px + 2%); margin-bottom:auto }",
    );
    assert_eq!(invalid.diagnostics().len(), 1);
    assert_eq!(invalid.syntax().rules().len(), 1);
    let [CssRule::Page(page)] = invalid.syntax().rules() else {
        panic!("one recovered page rule")
    };
    assert_eq!(page.declarations().properties().len(), 3);
    assert_eq!(
        page.declarations().properties()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginTop
    );
    assert_eq!(
        page.declarations().properties()[1]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginRight
    );
    assert_eq!(
        page.declarations().properties()[2]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginBottom
    );
}

#[test]
fn page_margin_declarations_preserve_css2_physical_literal_semantics() {
    let report = parse_sheet(concat!(
        "@page { ",
        "margin: -1px 2% auto 3cm; ",
        "margin-top: -4mm; margin-right: 5in; ",
        "margin-bottom: 6pc; margin-left: 7pt; ",
        "}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected one page rule")
    };
    assert_eq!(rule.declarations().properties().len(), 5);

    let margin = rule.declarations().properties()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap();
    let surgeist_css::CssKnownPropertyValueRef::Margin(margin) = margin else {
        panic!("expected typed margin")
    };
    let [top, right, bottom, left] = margin.value().assigned_values();
    assert_eq!(top.serialize_specified().unwrap(), "-1px");
    assert_eq!(right.serialize_specified().unwrap(), "2%");
    assert!(matches!(bottom, CssMarginValue::Auto));
    assert_eq!(left.serialize_specified().unwrap(), "3cm");
}

#[test]
fn page_context_composes_shared_values_including_font_relative_lengths() {
    let source = concat!(
        "@page { ",
        "margin-top: 1em; margin-right: 2ex; margin-bottom: 3rem; ",
        "margin-left: 4q; margin: calc(1px + 2%); margin: inherit; ",
        "margin-top: var(--page-margin); margin-bottom: -5px; ",
        "}"
    );
    let report = parse_sheet(source);
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected recovered page rule")
    };
    assert_eq!(rule.declarations().properties().len(), 8);
    assert_eq!(
        rule.declarations().properties()[0]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginTop
    );
    assert_eq!(
        rule.declarations().properties()[5]
            .known()
            .unwrap()
            .global(),
        Some(surgeist_css::CssGlobalKeyword::Inherit)
    );
    assert_eq!(
        rule.declarations().properties()[7]
            .known()
            .unwrap()
            .property(),
        CssKnownProperty::MarginBottom
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
}

#[test]
fn page_context_distinguishes_known_non_margin_unknown_and_invalid_margin_declarations() {
    let report = parse_sheet(concat!(
        "@page { ",
        "opacity: .5; mystery: 1; margin-top: bogus; ",
        "margin-left: 1cm !important; margin-right: 2%; ",
        "}"
    ));
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected recovered page rule")
    };
    assert_eq!(rule.declarations().properties().len(), 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>(),
        vec![
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::UnknownProperty,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
        ]
    );
    assert!(matches!(
        report.diagnostics()[0].error().kind(),
        ErrorKind::InvalidPropertyValue(detail) if detail.property() == CssKnownProperty::Opacity
    ));
    assert!(matches!(
        report.diagnostics()[1].error().kind(),
        ErrorKind::UnknownProperty(detail) if detail.name().as_str() == "mystery"
    ));
    assert!(matches!(
        report.diagnostics()[2].error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::MarginTop
    ));
}

#[test]
fn page_rules_preserve_import_phase_and_distinguish_group_from_style_context() {
    let invalid_then_import = parse_sheet(
        "@page :unknown { margin: 1cm; } @import \"still-early.css\"; @page { margin: 2cm; }",
    );
    assert!(matches!(
        invalid_then_import.syntax().rules(),
        [CssRule::Import(_), CssRule::Page(_)]
    ));
    assert_eq!(invalid_then_import.diagnostics().len(), 1);
    assert_eq!(
        invalid_then_import.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );

    let report = parse_sheet(concat!(
        "@import \"valid.css\"; ",
        "@page :first { margin: 1cm; } ",
        "@import \"late.css\"; ",
        "@media print { @page :left { margin: 2cm; } .inside {} } ",
        ".host { @page :right { margin: 3cm; } color: red; }"
    ));
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::Page(_),
            CssRule::Media(_),
            CssRule::Style(_)
        ]
    ));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect::<Vec<_>>(),
        vec![
            CssErrorCode::InvalidAtRulePlacement,
            CssErrorCode::InvalidAtRulePlacement,
        ]
    );
    let CssRule::Media(media) = &report.syntax().rules()[2] else {
        panic!("expected media group")
    };
    let [CssRule::Page(page), CssRule::Style(_)] = media.rules() else {
        panic!("expected retained page and following style child")
    };
    assert_eq!(page_pseudo(page), Some(CssPagePseudo::Left));
    assert_eq!(page.declarations().properties().len(), 1);
}

#[test]
fn malformed_page_preludes_and_statement_forms_drop_only_each_rule() {
    let source = concat!(
        ".before {} ",
        "@page named:unknown { margin: 1cm; } ",
        "@page :unknown { margin: 1cm; } ",
        "@page :left:unknown { margin: 1cm; } ",
        "@page :left extra { margin: 1cm; } ",
        "@page :first; ",
        "@page :right { margin: 2cm; } ",
        ".after {}"
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Page(_), CssRule::Style(_)]
    ));
    assert_eq!(report.diagnostics().len(), 5);
    assert!(report.diagnostics()[..4].iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidAtRulePrelude
            && diagnostic.action() == CssRecoveryAction::DropAtRule
    }));
    assert_eq!(
        report.diagnostics()[4].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
}

#[test]
fn page_body_recovery_rejects_invalid_margin_boxes_nested_rules_and_repeated_failures() {
    let report = parse_sheet(concat!(
        "@page { ",
        "margin-top: bogus; ",
        "@bad-left { content: \"title\"; } ",
        "@mystery { color: red; } ",
        ".nested { color: blue; } ",
        "margin-left: 1cm; mystery: 1; margin-right: 2cm; ",
        "} .after {}"
    ));
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Page(_), CssRule::Style(_)]
    ));
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("expected retained page")
    };
    assert_eq!(page.declarations().properties().len(), 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>(),
        vec![
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::InvalidAtRuleBody,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidAtRuleBody,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidAtRuleBody,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::UnknownProperty,
                CssRecoveryAction::DropDeclaration
            ),
        ]
    );
}

#[test]
fn page_eof_recovery_retains_valid_authored_structure_and_later_complete_rules() {
    let unterminated = parse_sheet("@page :first { margin: -1cm");
    assert!(matches!(unterminated.syntax().rules(), [CssRule::Page(_)]));
    assert_eq!(unterminated.diagnostics().len(), 1);
    assert_eq!(
        unterminated.diagnostics()[0].error().code(),
        CssErrorCode::UnexpectedEnd
    );

    let malformed = parse_sheet("@page :left :right");
    assert!(malformed.syntax().rules().is_empty());
    assert_eq!(malformed.diagnostics().len(), 1);
    assert_eq!(
        malformed.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
}

#[test]
fn page_rule_and_selector_named_metadata_match_retained_behavior() {
    for (id, kind, spelling, production) in [
        (
            "later.rule.page",
            surgeist_css::CssFeatureKind::Rule,
            "@page",
            "#syntax-page-selector",
        ),
        (
            "official.selector.page-pseudo",
            surgeist_css::CssFeatureKind::Selector,
            ":left|:right|:first|:blank",
            "#page-selectors",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(metadata.kind(), kind);
        assert_eq!(metadata.spelling(), spelling);
        assert_eq!(metadata.source().id().as_str(), "L-PAGE3-20181018");
        assert_eq!(metadata.production(), production);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.recognized_unsupported_code(), None);
    }
}

#[test]
fn page_rules_preserve_non_bmp_source_coordinates_for_rules_and_declarations() {
    let source = "/*😀*/\n@page :left { margin-left: -1cm !important; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected one page rule")
    };
    let rule_offset = source.find("@page").unwrap();
    assert_eq!(rule.position().byte_offset().value(), rule_offset);
    assert_eq!(rule.position().line().value(), 1);
    assert_eq!(rule.position().column().value(), 0);

    let declaration = &rule.declarations().properties()[0];
    let declaration_offset = source.find("margin-left").unwrap();
    let position = declaration.position().expect("parsed declaration position");
    assert_eq!(position.byte_offset().value(), declaration_offset);
    assert_eq!(position.line().value(), 1);
    assert_eq!(
        position.column().value() as usize,
        source[source.rfind('\n').unwrap() + 1..declaration_offset]
            .encode_utf16()
            .count()
    );
}

fn page_pseudo(page: &surgeist_css::CssPageRule) -> Option<surgeist_css::CssPagePseudo> {
    page.selectors()
        .selectors()
        .first()
        .and_then(|s| s.pseudos().first().copied())
}
