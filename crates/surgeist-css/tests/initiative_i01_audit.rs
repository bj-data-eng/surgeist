use surgeist_css::{
    CssErrorCode, CssImportance, CssKnownProperty, CssRecoveryAction, CssRule, CssSupportStatus,
    ErrorKind, feature_metadata, parse_sheet, parse_style_attribute, property_metadata,
};

fn actions(source: &str) -> Vec<CssRecoveryAction> {
    parse_sheet(source)
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.action())
        .collect()
}

#[test]
fn recovery_front_doors_retain_valid_siblings() {
    let sheet_source = ".before { color: red; } @unknown x; .middle { mystery: 1; width: 2px; } ??? { color: black; } .after { height: 3px; }";
    let sheet = parse_sheet(sheet_source);
    assert_eq!(sheet.syntax().rules().len(), 3, "retained rules");
    assert_eq!(sheet.diagnostics().len(), 3, "diagnosed units");
    assert_eq!(
        sheet
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.action())
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropAtRule,
            CssRecoveryAction::DropDeclaration,
            CssRecoveryAction::DropQualifiedRule,
        ],
        "recovery boundaries"
    );

    let attribute = parse_style_attribute("color: red; broken; width: 2px");
    assert_eq!(attribute.syntax().len(), 2, "style retention");
    assert_eq!(attribute.diagnostics().len(), 1, "style recovery");
    assert_eq!(
        attribute.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration,
        "style boundary"
    );
}

#[test]
fn malformed_units_emit_each_recovery_action() {
    let cases = [
        (
            CssRecoveryAction::DropDeclaration,
            ".x { mystery: 1; color: red; }",
        ),
        (
            CssRecoveryAction::DropDescriptor,
            "@font-face { font-family: Demo; src: url(demo); mystery: 1; }",
        ),
        (
            CssRecoveryAction::DropQualifiedRule,
            "??? { color: red; } .after { color: blue; }",
        ),
        (CssRecoveryAction::DropAtRule, "@unknown value;"),
        (
            CssRecoveryAction::DropKeyframeBlock,
            "@keyframes fade { fn(a) { opacity: .5; } to { opacity: 1; } }",
        ),
        (
            CssRecoveryAction::DropSelectorListItem,
            ":is(.kept,???) { color: red; }",
        ),
        (
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            "@media screen, ??? { .x { color: red; } }",
        ),
        (
            CssRecoveryAction::RetainWithImplicitClosure,
            ".x { color: red;",
        ),
        (
            CssRecoveryAction::IgnoreLegacyToken,
            "<!-- .x { color: red; }",
        ),
    ];

    for (expected, source) in cases {
        assert!(
            actions(source).contains(&expected),
            "missing {expected:?} for {source}"
        );
    }

    let mut over_limit = ":is(".repeat(257);
    over_limit.push_str(".leaf");
    over_limit.push_str(&")".repeat(257));
    over_limit.push_str(" { color: red; }");
    assert!(
        actions(&over_limit).contains(&CssRecoveryAction::StopAtNestingLimit),
        "missing StopAtNestingLimit"
    );
}

#[test]
fn malformed_units_report_typed_diagnostics_in_source_order() {
    let source = ".😀 { mystery: 1; width: bogus; color: red; } @unknown x;";
    let report = parse_sheet(source);
    let diagnostics = report.diagnostics();
    assert_eq!(diagnostics.len(), 3, "diagnostic count");
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect::<Vec<_>>(),
        [
            CssErrorCode::UnknownProperty,
            CssErrorCode::InvalidPropertyValue,
            CssErrorCode::UnknownAtRule,
        ],
        "structured error order"
    );

    let offsets = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.error().position().byte_offset().value())
        .collect::<Vec<_>>();
    assert_eq!(offsets[0], source.find("mystery").expect("mystery"));
    assert_eq!(offsets[1], source.find("bogus").expect("bogus"));
    assert_eq!(
        offsets[2],
        source.find("@unknown").expect("unknown at-rule")
    );
    assert!(offsets.windows(2).all(|pair| pair[0] < pair[1]));

    let first = &diagnostics[0];
    assert_eq!(first.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(first.span().start().byte_offset().value(), offsets[0]);
    assert_eq!(
        first.error().position().column().value(),
        u32::try_from(source[..offsets[0]].encode_utf16().count()).expect("UTF-16 column"),
        "UTF-16 coordinate"
    );
    assert_ne!(
        offsets[0],
        first.error().position().column().value() as usize
    );
    let ErrorKind::UnknownProperty(detail) = first.error().kind() else {
        panic!("expected structured unknown-property detail");
    };
    assert_eq!(detail.name().as_str(), "mystery");
}

#[cfg(feature = "app-strict")]
#[test]
fn strict_validation_returns_clean_syntax_or_ordinary_diagnostics() {
    let clean_source = ".x { color: red; }";
    let clean = parse_sheet(clean_source);
    assert_eq!(
        surgeist_css::validate_sheet(clean_source),
        Ok(clean.syntax().clone()),
        "clean validation"
    );

    let recovered_source = "color: red; mystery: 1; width: 2px";
    let recovered = parse_style_attribute(recovered_source);
    let failure = surgeist_css::validate_style_attribute(recovered_source)
        .expect_err("recovered input must fail validation");
    assert!(!failure.diagnostics().is_empty());
    assert_eq!(failure.first(), &recovered.diagnostics()[0]);
    assert_eq!(failure.diagnostics(), recovered.diagnostics());
}

#[test]
fn custom_substitution_and_known_declarations_preserve_coupled_values() {
    let report = parse_style_attribute(
        "--Theme: RGB(1, 2, var(--fallback)); width: var(--size, 2px) !important; color: red",
    );
    assert!(report.is_clean(), "style-attribute report");
    assert_eq!(report.syntax().len(), 3);

    let custom = report.syntax()[0].custom().expect("custom body");
    assert_eq!(custom.name().as_str(), "--Theme");
    assert_eq!(
        custom
            .value()
            .value()
            .expect("authored custom value")
            .as_css(),
        "RGB(1, 2, var(--fallback))"
    );

    let width = &report.syntax()[1];
    assert_eq!(width.importance(), CssImportance::Important);
    assert!(matches!(
        width.property_name(),
        surgeist_css::CssPropertyNameRef::Known(CssKnownProperty::Width)
    ));
    let value = width
        .known()
        .expect("known width")
        .substitution_dependent()
        .expect("expected property-coupled width value");
    assert_eq!(value.as_css(), "var(--size, 2px)");

    let color = &report.syntax()[2];
    assert!(matches!(
        color.property_name(),
        surgeist_css::CssPropertyNameRef::Known(CssKnownProperty::Color)
    ));
    assert_eq!(
        color.known().map(|known| known.property()),
        Some(CssKnownProperty::Color)
    );
}

#[test]
fn named_property_metadata_lookup_is_exact() {
    let width = property_metadata("WiDtH").expect("width metadata");
    assert_eq!(width.property(), CssKnownProperty::Width);
    assert_eq!(width.feature().id().as_str(), "baseline.property.width");
    assert_eq!(property_metadata("--width"), None);
    assert_eq!(property_metadata("not-a-property"), None);
    assert_eq!(feature_metadata("BASELINE.PROPERTY.WIDTH"), None);
}

#[test]
fn encoding_declarations_are_retained_or_diagnosed() {
    let valid = parse_sheet("@charset \"UTF-8\"; .x { color: red; }");
    assert!(valid.is_clean(), "valid encoding");
    assert_eq!(
        valid
            .syntax()
            .encoding()
            .expect("encoding metadata")
            .label(),
        "UTF-8"
    );

    for source in [
        "@charset bogus; .x { color: red; }",
        "@charset \"UTF-8\" .x { color: red; }",
        ".x { color: red; } @charset \"UTF-8\";",
    ] {
        let report = parse_sheet(source);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.error().code()
                    == CssErrorCode::InvalidEncodingDeclaration),
            "missing encoding diagnostic for {source}"
        );
    }
}

#[test]
fn parse_reports_decompose_into_retained_syntax_and_diagnostics() {
    let recovered = parse_sheet(".kept { color: red; } @unknown x;");
    assert!(!recovered.is_clean());
    let (sheet, diagnostics) = recovered.into_parts();
    assert_eq!(sheet.rules().len(), 1);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(diagnostics[0].action(), CssRecoveryAction::DropAtRule);

    let attribute = parse_style_attribute("--Theme: var(--fallback); width: 2px !important");
    assert!(attribute.is_clean());
    assert_eq!(attribute.syntax().len(), 2);
    assert_eq!(attribute.syntax()[1].importance(), CssImportance::Important);
    assert_eq!(
        feature_metadata("foundation.declaration.importance")
            .expect("metadata lookup")
            .status(),
        CssSupportStatus::Complete
    );
}

#[test]
fn media_query_recovery_exposes_read_only_never_sentinel() {
    let report = parse_sheet("@media screen, ??? { .x { color: red; } }");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("expected retained media rule");
    };
    assert!(
        media
            .query()
            .queries()
            .iter()
            .any(|query| query.is_guaranteed_false()),
        "parser-owned Never state"
    );
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
}
