//! Fonts4 src-list recovery retains valid members in authored order.
//! Independent grammar authority:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! Diagnostic and validation expectations are Surgeist's recovery contract.

use surgeist_css::{
    CssErrorCode, CssFontDisplay, CssFontFaceRule, CssFontFaceSource, CssParseReport,
    CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSheet, parse_sheet, validate_sheet,
};

fn font_face(report: &CssParseReport<CssSheet>) -> &CssFontFaceRule {
    let [CssRule::FontFace(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected retained font-face and following style rule");
    };
    assert_eq!(rule.descriptors().font_family().unwrap().as_str(), "Demo");
    assert_eq!(
        rule.descriptors().font_display().unwrap().value(),
        &CssFontDisplay::Swap
    );
    rule
}

fn assert_sources(rule: &CssFontFaceRule, expected: &[(&str, &str)]) {
    let actual = rule
        .descriptors()
        .src()
        .expect("valid fallback sources must survive")
        .sources()
        .iter()
        .map(|source| match source {
            CssFontFaceSource::Local(local) => ("local", local.as_str()),
            CssFontFaceSource::Url(url) => ("url", url.url()),
            other => panic!("unexpected font source kind: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_span(source: &str, diagnostic: &CssRecoveryDiagnostic, start: usize, end: usize) {
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    let responsible = diagnostic.error().position().byte_offset().value();
    assert!(
        (start..=end).contains(&responsible),
        "responsible offset {responsible} lies outside {:?}",
        &source[start..end]
    );
}

fn assert_member_diagnostics(
    source: &str,
    report: &CssParseReport<CssSheet>,
    discarded_members: &[&str],
) {
    assert_eq!(report.diagnostics().len(), discarded_members.len());
    for (diagnostic, member) in report.diagnostics().iter().zip(discarded_members) {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        // This observable assertion compiles before the new action is introduced.
        assert_eq!(
            format!("{:?}", diagnostic.action()),
            "DropFontSourceListItem"
        );
        let start = source
            .find(member)
            .expect("independently supplied member exists");
        assert_span(source, diagnostic, start, start + member.len());
    }
}

fn assert_validation_parity(source: &str, report: &CssParseReport<CssSheet>) {
    match validate_sheet(source) {
        Ok(sheet) => {
            assert!(report.is_clean());
            assert_eq!(&sheet, report.syntax());
        }
        Err(failure) => {
            assert!(!report.is_clean());
            assert_eq!(failure.diagnostics(), report.diagnostics());
        }
    }
}

#[test]
fn invalid_source_members_preserve_valid_fallback_order_and_diagnostics() {
    let source = concat!(
        "@font-face{font-family:Demo;src:",
        "local(First),local(inherit),url(second.woff2) format(woff2),",
        "url(bad.woff2) tech(unknown),local(Last);font-display:swap}",
        ".after{color:red}",
    );
    let report = parse_sheet(source);

    assert_sources(
        font_face(&report),
        &[
            ("local", "First"),
            ("url", "second.woff2"),
            ("local", "Last"),
        ],
    );
    assert_member_diagnostics(
        source,
        &report,
        &["local(inherit)", "url(bad.woff2) tech(unknown)"],
    );
    assert!(!report.is_clean());
    assert_validation_parity(source, &report);
}

#[test]
fn empty_source_members_use_delimiting_comma_spans() {
    let source = concat!(
        "@font-face{font-family:Demo;src:,url(first),,local(Last),;",
        "font-display:swap}.after{color:red}",
    );
    let report = parse_sheet(source);

    assert_sources(font_face(&report), &[("url", "first"), ("local", "Last")]);
    let expected_commas = [
        source.find("src:,").unwrap() + "src:".len(),
        source.find(",,").unwrap() + 1,
        source.find(",;").unwrap(),
    ];
    assert_eq!(report.diagnostics().len(), expected_commas.len());
    for (diagnostic, start) in report.diagnostics().iter().zip(expected_commas) {
        assert_eq!(
            format!("{:?}", diagnostic.action()),
            "DropFontSourceListItem"
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_span(source, diagnostic, start, start + 1);
    }
    assert_validation_parity(source, &report);
}

#[test]
fn source_member_boundaries_handle_bad_urls_suffixes_and_nested_commas() {
    let source = concat!(
        "@font-face{font-family:Demo;src:",
        "url(not a valid url),local(Discarded) format(woff2),dummy(a,b),",
        "url(\"last,part.woff2\") tech(variations,color-colrv1),local(\"A,B\");",
        "font-display:swap}.after{color:red}",
    );
    let report = parse_sheet(source);

    assert_sources(
        font_face(&report),
        &[("url", "last,part.woff2"), ("local", "A,B")],
    );
    assert_member_diagnostics(
        source,
        &report,
        &[
            "url(not a valid url)",
            "local(Discarded) format(woff2)",
            "dummy(a,b)",
        ],
    );
    assert_validation_parity(source, &report);
}

#[test]
fn all_invalid_source_members_drop_one_descriptor_and_preserve_earlier_source() {
    for invalid in [
        "local(),junk",
        "local(A) format(woff2),local(B) tech(variations)",
        ",,",
    ] {
        let rejected = format!("src:{invalid};");
        let source = format!(
            "@font-face{{font-family:Demo;src:url(previous);{rejected}font-display:swap}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);

        assert_sources(font_face(&report), &[("url", "previous")]);
        let [diagnostic] = report.diagnostics() else {
            panic!("all-invalid descriptor must have one whole-descriptor diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        let start = source.find(&rejected).unwrap();
        assert_span(&source, diagnostic, start, start + rejected.len());
        assert_validation_parity(&source, &report);
    }
}

#[test]
fn rejected_descriptor_annotations_discard_pending_member_diagnostics() {
    for annotation in ["!important", "!priority", "!important extra"] {
        let rejected = format!("src:local(),url(replacement){annotation};");
        let source = format!(
            "@font-face{{font-family:Demo;src:url(previous);{rejected}font-display:swap}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);

        assert_sources(font_face(&report), &[("url", "previous")]);
        let [diagnostic] = report.diagnostics() else {
            panic!("rejected descriptor must not publish its staged member diagnostics: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDeclarationAnnotation
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find('!').unwrap()
        );
        let start = source.find(&rejected).unwrap();
        assert_span(&source, diagnostic, start, start + rejected.len());
        assert_validation_parity(&source, &report);
    }
}

#[test]
fn recovered_source_member_span_preserves_unicode_source_coordinates() {
    let source = concat!(
        "@font-face{font-family:Demo;\n",
        "src:local(\"😀\"),local(inherit),url(last);font-display:swap}",
        ".after{color:red}",
    );
    let report = parse_sheet(source);

    assert_sources(font_face(&report), &[("local", "😀"), ("url", "last")]);
    assert_member_diagnostics(source, &report, &["local(inherit)"]);
    let span = report.diagnostics()[0].span();
    assert_eq!(span.start().line().value(), 1);
    assert_eq!(span.start().column().value(), 16);
    assert_eq!(span.end().line().value(), 1);
    assert_eq!(span.end().column().value(), 30);
    assert_validation_parity(source, &report);
}

#[test]
fn clean_source_list_has_no_recovery_and_passes_validation() {
    let source = concat!(
        "@font-face{font-family:Demo;src:local(First),",
        "url(last) format(woff2) tech(variations,color-colrv1);",
        "font-display:swap}.after{color:red}",
    );
    let report = parse_sheet(source);

    assert_sources(font_face(&report), &[("local", "First"), ("url", "last")]);
    assert!(report.is_clean());
    assert_validation_parity(source, &report);
}

#[test]
fn discarded_source_members_do_not_claim_implicit_closures() {
    // RetainWithImplicitClosure describes retained syntax. Discarding a source
    // member also discards its unclosed functions, even if another member survives.
    for discarded in ["local(", "dummy(", "dummy(nested(", "url(bad) format("] {
        let source = format!("@font-face{{src:url(valid),{discarded}");
        let report = parse_sheet(&source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected retained font-face for {source}");
        };
        assert_sources(rule, &[("url", "valid")]);
        let discarded_count = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropFontSourceListItem)
            .count();
        assert_eq!(discarded_count, 1, "{source}: {:?}", report.diagnostics());
        let closures: Vec<_> = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            })
            .collect();
        assert_eq!(
            closures.len(),
            1,
            "only the retained font-face block closes at EOF: {source}: {:?}",
            report.diagnostics()
        );
        assert_span(&source, closures[0], source.len(), source.len());
        assert_eq!(report.diagnostics().len(), 2);
        assert_validation_parity(&source, &report);
    }
}

#[test]
fn retained_source_members_keep_their_implicit_closures() {
    for (source, expected_discarded) in [
        ("@font-face{src:url(first),local(Last", 0),
        ("@font-face{src:url(first),dummy(),local(Last", 1),
    ] {
        let report = parse_sheet(source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected retained font-face for {source}");
        };
        assert_sources(rule, &[("url", "first"), ("local", "Last")]);
        let discarded_count = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropFontSourceListItem)
            .count();
        assert_eq!(discarded_count, expected_discarded);
        let closures: Vec<_> = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            })
            .collect();
        assert_eq!(
            closures.len(),
            2,
            "both retained font-face and local() close at EOF: {source}: {:?}",
            report.diagnostics()
        );
        for diagnostic in closures {
            assert_span(source, diagnostic, source.len(), source.len());
        }
        assert_eq!(report.diagnostics().len(), expected_discarded + 2);
        assert_validation_parity(source, &report);
    }
}
