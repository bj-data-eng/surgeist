//! CSS Syntax preprocessing, raw block recovery and source ownership oracles.

use std::sync::Arc;
use surgeist_css::*;

fn parse_units(context: CssParserContext, units: Arc<Vec<u16>>) -> CssDeclarationBlockParseReport {
    context
        .parse_declaration_block_contents_utf16_with_limits(
            units,
            CssUtf16InputLimits::default(),
            CssComponentValueLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
}

fn parse(text: &str) -> CssDeclarationBlockParseReport {
    parse_units(
        CssParserContext::default(),
        Arc::new(text.encode_utf16().collect()),
    )
}

fn names(report: &CssDeclarationBlockParseReport) -> Vec<&str> {
    report
        .report()
        .syntax()
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("ordinary property name"),
        })
        .collect()
}

#[test]
fn preprocessing_keeps_original_units_and_exact_normalized_scalar_mapping() {
    let mut raw: Vec<_> = "--x:".encode_utf16().collect();
    raw.extend([13, 10, 13, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00]);
    let original = Arc::new(raw);
    let report = parse_units(CssParserContext::default(), original.clone());
    assert!(report.report().is_clean(), "{report:?}");
    assert_eq!(names(&report), ["--x"]);
    let source = report.source().source();
    assert_eq!(source.as_str(), "--x:\n\n\n\u{fffd}😀\u{fffd}a\u{fffd}");
    assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &original));
    assert_eq!(
        original.as_slice(),
        [
            45, 45, 120, 58, 13, 10, 13, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00
        ]
    );
    for (bytes, units) in [
        (0..4, 0..4),
        (4..5, 4..6),
        (5..6, 6..7),
        (6..7, 7..8),
        (7..10, 8..9),
        (10..14, 9..11),
        (14..17, 11..12),
        (17..18, 12..13),
        (18..21, 13..14),
        (21..21, 14..14),
    ] {
        assert_eq!(source.original_utf16_range(bytes), Some(units));
    }
    assert_eq!(source.original_utf16_range(11..14), None);
    assert_eq!(source.original_utf16_range(22..22), None);
}

#[test]
fn declarations_values_and_diagnostics_share_the_complete_preprocessed_source() {
    let original = Arc::new(
        "/*😀*/\r\nwidth:2px; mystery:bad; --é:var(--size)"
            .encode_utf16()
            .collect(),
    );
    let weak = Arc::downgrade(&original);
    let report = parse_units(CssParserContext::default(), original);
    let root = report.source().source();
    assert_eq!(
        root.as_str(),
        "/*😀*/\nwidth:2px; mystery:bad; --é:var(--size)"
    );
    assert_eq!(names(&report), ["width", "--é"]);
    for declaration in report.report().syntax().iter() {
        assert!(
            declaration
                .parsed_name()
                .unwrap()
                .source()
                .same_snapshot(root)
        );
        assert!(
            declaration
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(root)
        );
        assert_eq!(declaration.parser_context(), report.source().context());
        let mut pending = vec![declaration.value_components().items()];
        while let Some(items) = pending.pop() {
            for item in items {
                let CssValueOrigin::Parsed(origin) = item.origin() else {
                    panic!("actual token origin")
                };
                assert!(origin.source().same_snapshot(root));
                let children = match item.view() {
                    CssComponentValueRef::Function(function) => Some(function.values()),
                    CssComponentValueRef::Block(block) => Some(block.values()),
                    _ => None,
                };
                if let Some(children) = children {
                    pending.push(children.items());
                }
            }
        }
    }
    let [diagnostic] = report.report().diagnostics() else {
        panic!("unknown property diagnostic")
    };
    assert!(diagnostic.source().unwrap().same_snapshot(root));
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 20);
    assert_eq!(root.original_utf16_range(20..27), Some(19..26));
    let declaration = report.report().syntax()[0].clone();
    let diagnostic = diagnostic.clone();
    drop(report);
    assert!(weak.upgrade().is_some());
    assert!(
        declaration
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(diagnostic.source().unwrap())
    );
    drop(declaration);
    drop(diagnostic);
    assert!(weak.upgrade().is_none());
}

#[test]
fn empty_and_wholly_recovered_reports_retain_and_admit_the_root_source() {
    for text in ["", " \r\n/**/;", "@unknown x;", "mystery:bad"] {
        let report = parse(text);
        assert!(report.report().syntax().is_empty(), "{text}: {report:?}");
        let root = report.source().source();
        assert_eq!(
            root.original_utf16().unwrap().as_slice(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        let bytes = root.storage_usage().retained_bytes();
        let mut sources = CssSourceRetention::new(bytes, 1000, 20);
        sources.visit(&report).unwrap();
        sources.visit(&report.clone()).unwrap();
        sources.visit(report.report().syntax()).unwrap();
        sources.visit(report.report().diagnostics()).unwrap();
        assert_eq!(sources.usage().retained_bytes(), bytes);
        if bytes > 0 {
            let mut refused = CssSourceRetention::new(bytes - 1, 1000, 20);
            assert_eq!(
                refused.visit(&report),
                Err(CssSourceRetentionError::Limit("retained source bytes"))
            );
        }
        let (source, ordinary) = report.into_parts();
        assert!(ordinary.syntax().is_empty());
        assert_eq!(source.context(), CssParserContext::default());
    }
}

#[test]
fn authored_order_custom_case_importance_and_symbolic_values_are_preserved() {
    let report = parse(
        "WIDTH:2px !IMPORTANT; width:3px; --Case:A/**/ B; --case:C; --Case:var(--x) !/**/important; color:inherit; width:var(--size,2px)",
    );
    assert!(report.report().is_clean(), "{report:?}");
    assert_eq!(
        names(&report),
        [
            "width", "width", "--Case", "--case", "--Case", "color", "width"
        ]
    );
    let declarations = report.report().syntax();
    assert_eq!(declarations[0].importance(), CssImportance::Important);
    assert_eq!(declarations[1].importance(), CssImportance::Normal);
    assert_eq!(declarations[4].importance(), CssImportance::Important);
    assert!(!declarations[0].same_occurrence(&declarations[1]));
    assert_eq!(
        declarations[2]
            .custom()
            .unwrap()
            .value()
            .value()
            .unwrap()
            .as_css(),
        "A/**/ B"
    );
    assert_eq!(
        declarations[5].known().unwrap().global(),
        Some(CssGlobalKeyword::Inherit)
    );
    assert_eq!(
        declarations[6]
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "var(--size,2px)"
    );
}

#[test]
fn actual_parser_mode_and_independent_svg_choice_select_the_owning_grammar() {
    let raw: Arc<Vec<u16>> = Arc::new("color:ABC; width:2px".encode_utf16().collect());
    let standards = parse_units(CssParserContext::default(), raw.clone());
    assert_eq!(names(&standards), ["width"]);
    assert_eq!(standards.report().diagnostics().len(), 1);
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let report = parse_units(quirks, raw);
    assert_eq!(report.source().context(), quirks);
    assert_eq!(names(&report), ["color", "width"]);
    assert!(report.report().is_clean(), "{report:?}");
    assert_eq!(
        report.report().syntax()[0].to_specified_css().unwrap(),
        "color: rgb(170, 187, 204);"
    );
    let svg = quirks.with_svg_glyph_orientation_vertical();
    let report = parse_units(
        svg,
        Arc::new("glyph-orientation-vertical:90".encode_utf16().collect()),
    );
    assert!(report.report().is_clean(), "{report:?}");
    assert_eq!(report.source().context(), svg);
    assert_eq!(report.report().syntax()[0].parser_context(), svg);
    assert_eq!(
        report.report().syntax()[0].property_name(),
        CssPropertyNameRef::SvgGlyphOrientationVertical
    );
}

#[test]
fn raw_recovery_drops_nested_units_and_root_closer_stops_without_later_diagnostics() {
    let report = parse(
        ";width:2px; @unknown {color:red} .x {height:3px} color:blue; --x:fn([a;b]); } height:4px; \"unterminated",
    );
    assert_eq!(names(&report), ["width", "color", "--x"]);
    let diagnostics = report.report().diagnostics();
    assert_eq!(diagnostics.len(), 2, "{report:?}");
    assert_eq!(diagnostics[0].action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostics[1].action(),
        CssRecoveryAction::DropQualifiedRule
    );
    for diagnostic in diagnostics {
        assert!(
            diagnostic
                .source()
                .unwrap()
                .same_snapshot(report.source().source())
        );
    }
    assert!(
        report
            .source()
            .source()
            .as_str()
            .ends_with("\"unterminated")
    );
    let implicit = parse("width:2px; --x:fn([a");
    assert_eq!(names(&implicit), ["width", "--x"]);
    assert_eq!(implicit.report().diagnostics().len(), 2);
    assert!(implicit.report().diagnostics().iter().all(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            && diagnostic
                .source()
                .unwrap()
                .same_snapshot(implicit.source().source())
    }));
}

#[test]
fn component_admission_counts_the_complete_input_after_a_root_stop() {
    let raw = Arc::new("width:2px} a b c d".encode_utf16().collect());
    let report = CssParserContext::default()
        .parse_declaration_block_contents_utf16_with_limits(
            raw,
            CssUtf16InputLimits::default(),
            CssComponentValueLimits::try_new(20, 4, 100).unwrap(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert!(report.report().syntax().is_empty());
    assert!(report.report().diagnostics().iter().any(|diagnostic| {
        diagnostic
            .source()
            .unwrap()
            .same_snapshot(report.source().source())
            && matches!(diagnostic.error().kind(), ErrorKind::InvalidComponentValue(error)
                if error.kind() == CssComponentValueErrorKind::ComponentLimit)
    }));
    let prefix = CssParserContext::default()
        .parse_declaration_block_contents_utf16_with_limits(
            Arc::new("width:2px}".encode_utf16().collect()),
            CssUtf16InputLimits::default(),
            CssComponentValueLimits::try_new(20, 4, 100).unwrap(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(names(&prefix), ["width"]);
    assert!(prefix.report().is_clean(), "{prefix:?}");
}

#[test]
fn normalized_byte_and_depth_limits_refuse_the_whole_declaration_projection() {
    for (text, limits, kind) in [
        (
            "--é:😀",
            CssComponentValueLimits::try_new(20, 100, 8).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
        (
            "width:2px; --x:f(g(1))",
            CssComponentValueLimits::try_new(1, 100, 100).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
    ] {
        let report = CssParserContext::default()
            .parse_declaration_block_contents_utf16_with_limits(
                Arc::new(text.encode_utf16().collect()),
                CssUtf16InputLimits::default(),
                limits,
                || Ok::<_, ()>(()),
            )
            .unwrap();
        assert!(report.report().syntax().is_empty(), "{report:?}");
        let [diagnostic] = report.report().diagnostics() else {
            panic!("one complete-input resource refusal")
        };
        assert!(
            diagnostic
                .source()
                .unwrap()
                .same_snapshot(report.source().source())
        );
        assert!(
            matches!(diagnostic.error().kind(), ErrorKind::InvalidComponentValue(error)
            if error.kind() == kind)
        );
        let retry = parse(text);
        assert!(retry.report().is_clean(), "{retry:?}");
        assert!(!retry.report().syntax().is_empty());
    }
}

#[test]
fn preparation_refusal_and_interruption_return_no_partial_parse() {
    let original: Arc<Vec<u16>> = Arc::new("width:2px".encode_utf16().collect());
    let base = CssUtf16InputLimits::default();
    for limits in [
        CssUtf16InputLimits {
            max_input_units: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_output_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_map_entries: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_checkpoints: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_scratch_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_retained_bytes: 0,
            ..base
        },
        CssUtf16InputLimits {
            max_work: 0,
            ..base
        },
    ] {
        assert!(matches!(
            CssParserContext::default().parse_declaration_block_contents_utf16_with_limits(
                original.clone(),
                limits,
                CssComponentValueLimits::default(),
                || Ok::<_, ()>(())
            ),
            Err(CssUtf16InputError::Limit(_))
        ));
    }
    let mut total = 0;
    CssParserContext::default()
        .parse_declaration_block_contents_utf16_with_limits(
            original.clone(),
            base,
            CssComponentValueLimits::default(),
            || {
                total += 1;
                Ok::<_, usize>(())
            },
        )
        .unwrap();
    // Include both observations bracketing the opaque grammar, as well as
    // preparation itself; no polling frequency inside grammar is assumed.
    for stop in [1, 2, 5, total - 1, total] {
        let mut calls = 0;
        let result = CssParserContext::default()
            .parse_declaration_block_contents_utf16_with_limits(
                original.clone(),
                base,
                CssComponentValueLimits::default(),
                || {
                    calls += 1;
                    if calls == stop { Err(stop) } else { Ok(()) }
                },
            );
        assert_eq!(result.unwrap_err(), CssUtf16InputError::Interrupted(stop));
        assert_eq!(calls, stop);
        assert_eq!(Arc::strong_count(&original), 1);
    }
}
