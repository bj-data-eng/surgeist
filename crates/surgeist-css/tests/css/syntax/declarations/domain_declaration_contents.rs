#![forbid(unsafe_code)]
//! Consumer oracles for exact unbraced CSSOM replacement inputs (#1084).
//! These are new public boundaries; no preimplementation RED is claimed.
use surgeist_css::*;

fn page_names(block: &CssPageDeclarationBlock) -> Vec<&str> {
    block
        .occurrences()
        .iter()
        .map(|entry| match entry {
            CssPageDeclaration::Descriptor(d) => d.value().kind().css_name(),
            CssPageDeclaration::Property(d) => match d.property_name() {
                CssPropertyNameRef::Known(p) => p.canonical_name(),
                CssPropertyNameRef::Custom(n) => n.as_str(),
                _ => panic!("Page property"),
            },
            _ => panic!("Page declaration"),
        })
        .collect()
}
fn face_kinds(block: &CssFontFaceDescriptors) -> Vec<CssFontFaceDescriptorKind> {
    block.occurrences().map(|d| d.value().kind()).collect()
}
fn resource(diagnostics: &[CssRecoveryDiagnostic], kind: CssComponentValueErrorKind) {
    assert!(
        diagnostics.iter().any(|d| matches!(d.error().kind(),
        ErrorKind::InvalidComponentValue(error) if error.kind() == kind)),
        "{diagnostics:?}"
    );
}

#[test]
fn page_recovers_mixed_domains_and_preserves_duplicate_occurrences_before_selection() {
    let report = parse_page_declaration_block_contents(
        "size:A4; margin:1px; --Theme: RAW; size:bogus; color:red; size:A3 !important; --theme: lower",
    );
    let block = report.syntax().as_ref().unwrap();
    assert!(!report.is_clean());
    assert_eq!(
        page_names(block),
        ["size", "margin", "--Theme", "color", "size", "--theme"]
    );
    assert_eq!(block.properties().len(), 4);
    assert_eq!(
        block
            .effective_descriptor(CssPageDescriptorKind::Size)
            .unwrap()
            .importance(),
        CssImportance::Important
    );
    let selected = block.try_specified().unwrap();
    assert_eq!(
        selected.serialize_cssom().unwrap(),
        "margin: 1px; --Theme: RAW; color: red; size: a3 !important; --theme: lower;"
    );
    let CssPageDeclaration::Property(property) = &block.occurrences()[1] else {
        panic!()
    };
    assert!(property.same_occurrence(&block.properties()[0]));
}

#[test]
fn page_drops_nested_rules_as_units_without_creating_margin_children() {
    let report = parse_page_declaration_block_contents(
        "marks:crop; @top-left { content:'child'; size:A3; } margin:2px; .x { color:red; @x{} } bleed:1pt; @unknown; color:blue",
    );
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(page_names(block), ["marks", "margin", "bleed", "color"]);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::DropAtRule)
            .count(),
        2
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropQualifiedRule)
    );
    assert_eq!(
        block.try_specified().unwrap().serialize_cssom().unwrap(),
        "marks: crop; margin: 2px; bleed: 1pt; color: blue;"
    );
}

#[test]
fn page_uses_domain_applicability_and_selected_syntax_fallback() {
    let report = parse_page_declaration_block_contents(
        "margin-top:-1px; margin-left:7; display:block; size:var(--x){}margin-bottom:3px; --x:{ a:b }; marks:cross",
    );
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(
        page_names(block),
        ["margin-top", "margin-bottom", "--x", "marks"]
    );
    assert!(!report.is_clean());
}

#[test]
fn font_face_recovers_invalid_neighbors_priority_and_duplicate_order() {
    use CssFontFaceDescriptorKind as K;
    let report = parse_font_face_declaration_block_contents(
        "font-family:Example; font-display:swap; bad:value; font-weight:0; src:local(Example); font-display:optional; font-style:italic !important; --x:raw; font-weight:400",
    );
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(
        face_kinds(block),
        [
            K::FontFamily,
            K::FontDisplay,
            K::Src,
            K::FontDisplay,
            K::FontWeight
        ]
    );
    assert!(!report.is_clean());
    assert!(matches!(
        block.effective(K::FontDisplay).unwrap().value(),
        CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::FontDisplay(
            CssFontDisplay::Optional
        ))
    ));
    // An honest programmatic containing rule uses existing effective-descriptor formatting.
    let rule = CssRule::FontFace(CssFontFaceRule::new(block.clone()));
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@font-face { font-family: Example; src: local(\"Example\"); font-weight: 400; font-display: optional; }"
    );
}

#[test]
fn font_face_raw_structural_recovery_keeps_braced_fragment_policy_distinct() {
    use CssFontFaceDescriptorKind as K;
    let raw = "font-family:Example; @unknown {src:local(Bad)} src:local(Good); .x {font-display:swap} font-display:optional";
    let report = parse_font_face_declaration_block_contents(raw);
    assert_eq!(
        face_kinds(report.syntax().as_ref().unwrap()),
        [K::FontFamily, K::Src, K::FontDisplay]
    );
    assert!(!report.is_clean());
    assert!(
        parse_font_face_block("{font-family:Example; @unknown {src:local(Bad)} src:local(Good)}")
            .syntax()
            .is_none()
    );
}

#[test]
fn src_member_recovery_and_mixed_curly_fallback_use_current_owners() {
    use CssFontFaceDescriptorKind as K;
    let report = parse_font_face_declaration_block_contents(
        "src:local(A), bogus(), local(B); font-weight:env(weight){}font-display:swap",
    );
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(face_kinds(block), [K::Src, K::FontDisplay]);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropFontSourceListItem)
    );
    assert_eq!(
        CssRule::FontFace(CssFontFaceRule::new(block.clone()))
            .to_specified_css()
            .unwrap(),
        "@font-face { src: local(\"A\"), local(\"B\"); font-display: swap; }"
    );
}

#[test]
fn root_closer_stops_both_domains_without_diagnostics_from_later_input() {
    let page = parse_page_declaration_block_contents("size:A4} size:A3; \"unterminated");
    assert!(page.is_clean(), "{page:?}");
    assert_eq!(page_names(page.syntax().as_ref().unwrap()), ["size"]);
    let face = parse_font_face_declaration_block_contents(
        "font-display:swap} font-display:optional; \"unterminated",
    );
    assert!(face.is_clean(), "{face:?}");
    assert_eq!(
        face_kinds(face.syntax().as_ref().unwrap()),
        [CssFontFaceDescriptorKind::FontDisplay]
    );
}

#[test]
fn genuine_unbraced_origins_outlive_input_and_keep_unicode_coordinates() {
    let raw = "/*😀*/\r\nsize:A4; margin:1px; bogus:bad";
    let report = {
        let input = raw.to_owned();
        parse_page_declaration_block_contents(&input)
    };
    let block = report.syntax().as_ref().unwrap();
    let CssPageDeclaration::Descriptor(descriptor) = &block.occurrences()[0] else {
        panic!()
    };
    let name = descriptor.occurrence().unwrap().parsed_name().unwrap();
    assert_eq!(name.source().as_str(), raw);
    assert_eq!(name.span().start().byte_offset().value(), 10);
    assert_eq!(name.span().start().line().value(), 1);
    assert_eq!(descriptor.value().origin().source().as_str(), raw);
    assert_eq!(
        descriptor
            .value()
            .origin()
            .span()
            .start()
            .byte_offset()
            .value(),
        15
    );
    let property = &block.properties()[0];
    assert_eq!(property.parsed_name().unwrap().source().as_str(), raw);
    assert_eq!(
        property
            .parsed_name()
            .unwrap()
            .span()
            .start()
            .byte_offset()
            .value(),
        19
    );
    assert_eq!(
        report.diagnostics()[0].span().start().byte_offset().value(),
        31
    );

    let raw = "/*😀*/\r\nfont-weight:env(weight); bogus:value";
    let report = {
        let input = raw.to_owned();
        parse_font_face_declaration_block_contents(&input)
    };
    let descriptor = report
        .syntax()
        .as_ref()
        .unwrap()
        .occurrences()
        .next()
        .unwrap();
    assert_eq!(descriptor.position().unwrap().byte_offset().value(), 10);
    let CssAuthoredFontFaceDescriptorValue::Pending(pending) = descriptor.value() else {
        panic!()
    };
    let CssValueOrigin::Parsed(origin) = pending
        .components()
        .items()
        .iter()
        .find_map(|item| {
            matches!(item.view(), CssComponentValueRef::Function(_)).then_some(item.origin())
        })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(origin.source().as_str(), raw);
    assert_eq!(origin.span().start().byte_offset().value(), 22);
    assert_eq!(
        report.diagnostics()[0].span().start().byte_offset().value(),
        35
    );
}

#[test]
fn empty_recovered_contents_are_successful_and_resource_failures_are_not_clearing() {
    for source in ["", " ; /**/ ;", "bogus:value", "@unknown {}", "}"] {
        assert!(
            parse_page_declaration_block_contents(source)
                .syntax()
                .as_ref()
                .unwrap()
                .occurrences()
                .is_empty(),
            "{source}"
        );
        assert_eq!(
            parse_font_face_declaration_block_contents(source)
                .syntax()
                .as_ref()
                .unwrap()
                .occurrences()
                .len(),
            0,
            "{source}"
        );
    }
    for (source, kind, limits) in [
        (
            "size:A4; margin:2px",
            CssComponentValueErrorKind::ByteLimit,
            CssComponentValueLimits::try_new(32, 100, 8).unwrap(),
        ),
        (
            "size:A4; margin:2px",
            CssComponentValueErrorKind::ComponentLimit,
            CssComponentValueLimits::try_new(32, 7, 100).unwrap(),
        ),
        (
            "size:A4; --x:f(g(1))",
            CssComponentValueErrorKind::NestingLimit,
            CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
        ),
        (
            "size:A4} /*charged trailing source*/",
            CssComponentValueErrorKind::ByteLimit,
            CssComponentValueLimits::try_new(32, 100, 8).unwrap(),
        ),
    ] {
        let failed = parse_page_declaration_block_contents_with_limits(source, limits);
        assert!(failed.syntax().is_none(), "{failed:?}");
        resource(failed.diagnostics(), kind);
        let retry = parse_page_declaration_block_contents(source);
        assert!(retry.syntax().is_some());
        assert_eq!(retry, parse_page_declaration_block_contents(source));
    }
    for (source, kind, limits) in [
        (
            "font-display:swap; src:local(A)",
            CssComponentValueErrorKind::ByteLimit,
            CssComponentValueLimits::try_new(32, 100, 17).unwrap(),
        ),
        (
            "font-display:swap; src:local(A)",
            CssComponentValueErrorKind::ComponentLimit,
            CssComponentValueLimits::try_new(32, 8, 100).unwrap(),
        ),
        (
            "font-display:swap; src:env(x, f(g(1)))",
            CssComponentValueErrorKind::NestingLimit,
            CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
        ),
        (
            "font-display:swap} /*charged trailing source*/",
            CssComponentValueErrorKind::ByteLimit,
            CssComponentValueLimits::try_new(32, 100, 17).unwrap(),
        ),
    ] {
        let failed = parse_font_face_declaration_block_contents_with_limits(source, limits);
        assert!(failed.syntax().is_none(), "{failed:?}");
        resource(failed.diagnostics(), kind);
        let retry = parse_font_face_declaration_block_contents(source);
        assert!(retry.syntax().is_some());
        assert_eq!(retry, parse_font_face_declaration_block_contents(source));
    }
}

#[test]
fn exact_component_allowances_admit_the_full_block_and_one_less_rejects_atomically() {
    let source = "size:A4; margin:2px";
    // Ident, colon, ident, semicolon, whitespace, ident, colon, dimension.
    let adequate = CssComponentValueLimits::try_new(0, 8, source.len()).unwrap();
    let report = parse_page_declaration_block_contents_with_limits(source, adequate);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        page_names(report.syntax().as_ref().unwrap()),
        ["size", "margin"]
    );
    let failed = parse_page_declaration_block_contents_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 7, source.len()).unwrap(),
    );
    assert!(failed.syntax().is_none());
    resource(
        failed.diagnostics(),
        CssComponentValueErrorKind::ComponentLimit,
    );

    let source = "font-display:swap; src:local(A)";
    // The second descriptor's function and its child both consume allowance.
    let adequate = CssComponentValueLimits::try_new(1, 9, source.len()).unwrap();
    let report = parse_font_face_declaration_block_contents_with_limits(source, adequate);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        face_kinds(report.syntax().as_ref().unwrap()),
        [
            CssFontFaceDescriptorKind::FontDisplay,
            CssFontFaceDescriptorKind::Src
        ]
    );
    let failed = parse_font_face_declaration_block_contents_with_limits(
        source,
        CssComponentValueLimits::try_new(1, 8, source.len()).unwrap(),
    );
    assert!(failed.syntax().is_none());
    resource(
        failed.diagnostics(),
        CssComponentValueErrorKind::ComponentLimit,
    );
}

#[test]
fn page_context_and_original_eof_component_recovery_are_retained() {
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let source = "margin:1px; size:A4; --x:func(1";
    let report = quirks.parse_page_declaration_block_contents(source);
    let block = report.syntax().as_ref().unwrap();
    assert_eq!(page_names(block), ["margin", "size", "--x"]);
    assert_eq!(block.properties()[0].parser_context(), quirks);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert_eq!(
        report,
        quirks.parse_page_declaration_block_contents_with_limits(
            source,
            CssComponentValueLimits::default()
        )
    );
    let face = parse_font_face_declaration_block_contents("font-display:swap; src:local(Example");
    assert_eq!(
        face_kinds(face.syntax().as_ref().unwrap()),
        [
            CssFontFaceDescriptorKind::FontDisplay,
            CssFontFaceDescriptorKind::Src
        ]
    );
    assert!(!face.is_clean());
}
