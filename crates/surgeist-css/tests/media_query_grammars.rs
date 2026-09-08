use surgeist_css::{
    CssDefinedFalseMediaReason, CssErrorCode, CssGridMode, CssMediaConditionKind,
    CssMediaFeatureKind, CssMediaFeatureQuery, CssMediaQuery, CssMediaQueryModifier, CssMediaRatio,
    CssMediaType, CssQueryComparison, CssRatio, CssRecoveryAction, CssResolutionUnit, CssRule,
    CssScanMode, parse_sheet,
};

fn parsed_feature(query: &str) -> CssMediaFeatureQuery {
    let source = format!("@media {query} {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{query}: {:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("{query}: expected one retained media rule")
    };
    let [CssMediaQuery::Condition(condition)] = rule.query().queries() else {
        panic!("{query}: expected one condition-only query")
    };
    let CssMediaConditionKind::Feature(feature) = condition.kind() else {
        panic!("{query}: expected one media feature")
    };
    feature.clone()
}

fn parsed_type(name: &str) -> CssMediaType {
    let source = format!("@media {name} {{}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("{name}: expected one retained media rule")
    };
    let [CssMediaQuery::Typed(query)] = rule.query().queries() else {
        panic!("{name}: expected one typed query")
    };
    query.media_type()
}

#[test]
fn defined_false_media_syntax_is_not_malformed_recovery() {
    let report = parse_sheet(concat!(
        "@media only future-screen and (unknown-feature: calc(1foo + 2px)), ",
        "(width: calc(1px)), screen {}",
    ));
    assert!(
        report.is_clean(),
        "balanced unknown MQ3 syntax is valid defined-false authored syntax: {:?}",
        report.diagnostics()
    );
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected the media rule to be retained")
    };
    assert_eq!(rule.query().queries().len(), 3);
    assert!(
        rule.query()
            .queries()
            .iter()
            .all(|query| !matches!(query, CssMediaQuery::Never(_))),
        "defined-false members are distinct from malformed recovery"
    );

    for malformed in ["layer", "not", "and", "only", "or", "???", ""] {
        let source = format!("@media screen,{malformed},print {{}} ");
        let report = parse_sheet(&source);
        let [CssRule::Media(rule)] = report.syntax().rules() else {
            panic!("{malformed:?}: expected the media rule to be retained")
        };
        assert!(
            matches!(
                rule.query().queries(),
                [
                    CssMediaQuery::Typed(_),
                    CssMediaQuery::Never(_),
                    CssMediaQuery::Typed(_)
                ]
            ),
            "{malformed:?}: reserved, unexpected, and empty members stay malformed"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{malformed:?}: expected exactly one malformed-member diagnostic")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            "{malformed:?}"
        );
    }

    let scripting = parse_sheet("@media screen,(scripting: enabled),print {}");
    let [CssRule::Media(rule)] = scripting.syntax().rules() else {
        panic!("expected the scripting media rule to be retained")
    };
    assert!(matches!(
        rule.query().queries(),
        [
            CssMediaQuery::Typed(_),
            CssMediaQuery::Never(_),
            CssMediaQuery::Typed(_)
        ]
    ));
    let [diagnostic] = scripting.diagnostics() else {
        panic!("recognized deferred scripting still diagnoses")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
}

#[test]
fn mq3_named_types_and_features_follow_exact_domains() {
    let report = parse_sheet(concat!(
        "@media speech { .speech { color: red; } } ",
        "@media (device-width: 1px) { .device { color: blue; } }",
    ));

    assert!(
        report.is_clean(),
        "MQ3 speech and device-width are valid authored syntax: {:?}",
        report.diagnostics()
    );

    let [CssRule::Media(speech), CssRule::Media(device)] = report.syntax().rules() else {
        panic!("expected both valid MQ3 media rules to be retained")
    };
    assert!(matches!(
        speech.query().queries(),
        [CssMediaQuery::Typed(_)]
    ));
    assert!(matches!(
        device.query().queries(),
        [CssMediaQuery::Condition(_)]
    ));
}

#[test]
fn mq3_exposes_all_eleven_named_media_types() {
    for (name, expected) in [
        ("all", CssMediaType::All),
        ("aural", CssMediaType::Aural),
        ("braille", CssMediaType::Braille),
        ("embossed", CssMediaType::Embossed),
        ("handheld", CssMediaType::Handheld),
        ("print", CssMediaType::Print),
        ("projection", CssMediaType::Projection),
        ("screen", CssMediaType::Screen),
        ("speech", CssMediaType::Speech),
        ("tty", CssMediaType::Tty),
        ("tv", CssMediaType::Tv),
    ] {
        assert_eq!(parsed_type(name), expected, "{name}");
    }
}

#[test]
fn mq3_boolean_features_preserve_their_typed_names() {
    for (name, expected) in [
        ("width", CssMediaFeatureKind::Width),
        ("height", CssMediaFeatureKind::Height),
        ("device-width", CssMediaFeatureKind::DeviceWidth),
        ("device-height", CssMediaFeatureKind::DeviceHeight),
        ("orientation", CssMediaFeatureKind::Orientation),
        ("aspect-ratio", CssMediaFeatureKind::AspectRatio),
        (
            "device-aspect-ratio",
            CssMediaFeatureKind::DeviceAspectRatio,
        ),
        ("color", CssMediaFeatureKind::Color),
        ("color-index", CssMediaFeatureKind::ColorIndex),
        ("monochrome", CssMediaFeatureKind::Monochrome),
        ("resolution", CssMediaFeatureKind::Resolution),
        ("scan", CssMediaFeatureKind::Scan),
        ("grid", CssMediaFeatureKind::Grid),
    ] {
        let feature = parsed_feature(&format!("({name})"));
        assert_eq!(feature.name(), name);
        assert!(matches!(feature, CssMediaFeatureQuery::Boolean(value) if value == expected));
        assert_eq!(expected.name(), name);
    }
}

#[test]
fn mq3_range_features_follow_length_ratio_integer_and_resolution_domains() {
    assert!(matches!(
        parsed_feature("(device-width: 800px)"),
        CssMediaFeatureQuery::DeviceWidth(value)
            if value.comparison() == Some(CssQueryComparison::Equal)
                && value.value().value().value() == 800.0
    ));
    assert!(matches!(
        parsed_feature("(max-device-height: 7em)"),
        CssMediaFeatureQuery::DeviceHeight(value)
            if value.comparison() == Some(CssQueryComparison::LessThanOrEqual)
                && value.value().value().value() == 7.0
    ));
    assert!(matches!(
        parsed_feature("(width: 0)"),
        CssMediaFeatureQuery::Width(value)
            if value.value().value().value() == 0.0
                && value.value().authored_unit().is_none()
                && value.value().unit() == surgeist_css::CssLengthUnit::Px
    ));
    assert!(matches!(
        parsed_feature("(width: 0px)"),
        CssMediaFeatureQuery::Width(value)
            if value.value().authored_unit() == Some(surgeist_css::CssLengthUnit::Px)
    ));
    assert!(matches!(
        parsed_feature("(min-aspect-ratio: 16/9)"),
        CssMediaFeatureQuery::AspectRatio(value)
            if value.comparison() == Some(CssQueryComparison::GreaterThanOrEqual)
                && value.value().numerator() == 16
                && value.value().denominator() == 9
    ));
    assert!(matches!(
        parsed_feature("(device-aspect-ratio > 4/3)"),
        CssMediaFeatureQuery::DeviceAspectRatio(value)
            if value.comparison() == Some(CssQueryComparison::GreaterThan)
                && value.value().numerator() == 4
                && value.value().denominator() == 3
    ));
    assert!(matches!(
        parsed_feature("(min-color-index: 256)"),
        CssMediaFeatureQuery::ColorIndex(value)
            if value.comparison() == Some(CssQueryComparison::GreaterThanOrEqual)
                && value.value().value() == 256
    ));

    for (css, unit) in [
        ("(resolution: 96dpi)", CssResolutionUnit::Dpi),
        ("(resolution: 38dpcm)", CssResolutionUnit::Dpcm),
        ("(resolution: 2dppx)", CssResolutionUnit::Dppx),
    ] {
        assert!(matches!(
            parsed_feature(css),
            CssMediaFeatureQuery::Resolution(value) if value.value().unit() == unit
        ));
    }
}

#[test]
fn mq3_ratio_is_positive_integer_only_without_narrowing_general_ratio() {
    let ratio = CssMediaRatio::try_new(16, 9).expect("positive MQ3 ratio");
    assert_eq!(ratio.numerator(), 16);
    assert_eq!(ratio.denominator(), 9);
    assert_eq!(CssMediaRatio::try_new(0, 9), None);
    assert_eq!(CssMediaRatio::try_new(16, 0), None);

    let general = CssRatio::try_new(0.0, 1.5).expect("existing general ratio remains broader");
    assert_eq!(general.numerator().value(), 0.0);
    assert_eq!(general.denominator().value(), 1.5);
}

#[test]
fn mq3_scan_and_grid_expose_exact_keyword_and_binary_domains() {
    assert!(matches!(
        parsed_feature("(scan: progressive)"),
        CssMediaFeatureQuery::Scan(CssScanMode::Progressive)
    ));
    assert!(matches!(
        parsed_feature("(scan: interlace)"),
        CssMediaFeatureQuery::Scan(CssScanMode::Interlace)
    ));
    assert!(matches!(
        parsed_feature("(grid: 0)"),
        CssMediaFeatureQuery::Grid(CssGridMode::Bitmap)
    ));
    assert!(matches!(
        parsed_feature("(grid: -0)"),
        CssMediaFeatureQuery::Grid(CssGridMode::Bitmap)
    ));
    assert!(matches!(
        parsed_feature("(grid: 1)"),
        CssMediaFeatureQuery::Grid(CssGridMode::Grid)
    ));
    assert_eq!(CssGridMode::Bitmap.value(), 0);
    assert_eq!(CssGridMode::Grid.value(), 1);
}

#[test]
fn mq3_named_queries_keep_exact_non_bmp_source_positions() {
    let source = "@media /*😀*/ speech, /*😀*/ (device-width: 1px) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected one media rule")
    };
    let [
        CssMediaQuery::Typed(speech),
        CssMediaQuery::Condition(device),
    ] = rule.query().queries()
    else {
        panic!("expected typed speech and device-width condition")
    };

    let speech_offset = source.find("speech").expect("speech position");
    assert_eq!(speech.position().byte_offset().value(), speech_offset);
    assert_eq!(
        speech.position().column().value(),
        u32::try_from(source[..speech_offset].encode_utf16().count()).expect("UTF-16 column")
    );
    let device_offset = source.find("(device-width").expect("device position");
    assert_eq!(device.position().byte_offset().value(), device_offset);
    assert_eq!(
        device.position().column().value(),
        u32::try_from(source[..device_offset].encode_utf16().count()).expect("UTF-16 column")
    );
}

#[test]
fn mq3_structurally_malformed_features_recover_one_member_and_retain_siblings() {
    for (invalid, responsible) in [
        ("(min-width)", "min-width"),
        ("(min-color)", "min-color"),
        ("(width:)", ")"),
        ("(width 1px)", "1px"),
    ] {
        let source = format!("@media screen,{invalid},print {{ .x {{ color: red; }} }}");
        let report = parse_sheet(&source);
        let [CssRule::Media(rule)] = report.syntax().rules() else {
            panic!("{invalid}: expected retained media rule")
        };
        assert!(matches!(
            rule.query().queries(),
            [
                CssMediaQuery::Typed(_),
                CssMediaQuery::Never(_),
                CssMediaQuery::Typed(_)
            ]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("{invalid}: expected exactly one query diagnostic")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            "{invalid}"
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(responsible).expect("responsible token"),
            "{invalid}"
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_sheet(&source)
                .expect_err("strict mode rejects recovered media syntax")
                .diagnostics(),
            report.diagnostics(),
            "{invalid}"
        );
    }
}

#[test]
fn mq3_unknown_features_and_complete_unknown_values_are_defined_false() {
    for (css, reason) in [
        (
            "(future-feature)",
            CssDefinedFalseMediaReason::UnknownFeature,
        ),
        (
            "(min-future-feature: 1px)",
            CssDefinedFalseMediaReason::UnknownFeature,
        ),
        (
            "(future-feature: calc(1foo + 2px))",
            CssDefinedFalseMediaReason::UnknownFeature,
        ),
        (
            "(width: calc(1px))",
            CssDefinedFalseMediaReason::UnknownValue,
        ),
        ("(width: -1px)", CssDefinedFalseMediaReason::UnknownValue),
        ("(width: 2qu)", CssDefinedFalseMediaReason::UnknownValue),
        (
            "(orientation: diagonal)",
            CssDefinedFalseMediaReason::UnknownValue,
        ),
        (
            "(aspect-ratio: 0/1)",
            CssDefinedFalseMediaReason::UnknownValue,
        ),
        (
            "(aspect-ratio: 1.5/1)",
            CssDefinedFalseMediaReason::UnknownValue,
        ),
        (
            "(resolution: 0dpi)",
            CssDefinedFalseMediaReason::UnknownValue,
        ),
        ("(scan: raster)", CssDefinedFalseMediaReason::UnknownValue),
        ("(grid: 2)", CssDefinedFalseMediaReason::UnknownValue),
    ] {
        let source = format!("@media screen,{css},print {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let [CssRule::Media(rule)] = report.syntax().rules() else {
            panic!("{css}: expected retained media rule")
        };
        let [
            CssMediaQuery::Typed(_),
            CssMediaQuery::Condition(condition),
            CssMediaQuery::Typed(_),
        ] = rule.query().queries()
        else {
            panic!("{css}: expected comma-local defined-false condition")
        };
        let CssMediaConditionKind::DefinedFalse(defined_false) = condition.kind() else {
            panic!("{css}: expected defined-false authored condition")
        };
        assert_eq!(defined_false.as_css(), css, "{css}");
        assert_eq!(defined_false.reason(), reason, "{css}");
        assert_eq!(defined_false.position(), condition.position(), "{css}");
        assert!(!rule.query().queries()[1].is_guaranteed_false(), "{css}");
    }
}

#[test]
fn mq3_unknown_media_types_preserve_modifiers_exact_spelling_and_positions() {
    let source = "@media only F\\75ture-Screen, not Future-Print and (width: 1px) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected retained media rule")
    };
    let [CssMediaQuery::Typed(only), CssMediaQuery::Typed(not)] = rule.query().queries() else {
        panic!("expected two typed unknown media queries")
    };

    assert_eq!(only.modifier(), Some(CssMediaQueryModifier::Only));
    assert_eq!(only.media_type(), CssMediaType::Unknown);
    let only_type = only.unknown_media_type().expect("unknown type details");
    assert_eq!(only_type.as_css(), "F\\75ture-Screen");
    assert_eq!(only_type.reason(), CssDefinedFalseMediaReason::UnknownType);
    assert_eq!(
        only.position().byte_offset().value(),
        source.find("only").unwrap()
    );
    assert_eq!(
        only_type.position().byte_offset().value(),
        source.find("F\\75ture-Screen").unwrap()
    );

    assert_eq!(not.modifier(), Some(CssMediaQueryModifier::Not));
    assert_eq!(not.media_type(), CssMediaType::Unknown);
    assert_eq!(
        not.unknown_media_type()
            .expect("unknown type details")
            .as_css(),
        "Future-Print"
    );
    assert!(not.condition().is_some());
}

#[test]
fn mq3_defined_false_balanced_nesting_obeys_the_255_256_257_boundary() {
    fn source_at_depth(depth: usize) -> String {
        let functions = depth.saturating_sub(1);
        format!(
            "@media (unknown: {}x{}) {{}}",
            "f(".repeat(functions),
            ")".repeat(functions)
        )
    }

    for depth in [255, 256] {
        let source = source_at_depth(depth);
        let report = parse_sheet(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let [CssRule::Media(rule)] = report.syntax().rules() else {
            panic!("depth {depth}: expected retained media rule")
        };
        assert!(matches!(
            rule.query().queries(),
            [CssMediaQuery::Condition(condition)]
                if matches!(condition.kind(), CssMediaConditionKind::DefinedFalse(_))
        ));
    }

    let source = source_at_depth(257);
    let report = parse_sheet(&source);
    assert!(!report.is_clean());
    assert!(report.diagnostics().iter().any(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::NestingLimit
            && diagnostic.action() == CssRecoveryAction::StopAtNestingLimit
    }));
}

#[test]
fn mq3_defined_false_condition_survives_rule_eof_implicit_closure() {
    let source = "@media (unknown: yes) {";
    let report = parse_sheet(source);
    let [CssRule::Media(rule)] = report.syntax().rules() else {
        panic!("expected the implicitly closed media rule to be retained")
    };
    assert!(matches!(
        rule.query().queries(),
        [CssMediaQuery::Condition(condition)]
            if matches!(condition.kind(), CssMediaConditionKind::DefinedFalse(_))
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected only the rule-block EOF closure diagnostic")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
}

#[test]
fn empty_media_list_is_valid_authored_syntax_but_public_construction_stays_checked() {
    let report = parse_sheet("@media {} .after { color: red; }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Media(media), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected empty media rule and following style sibling")
    };
    assert!(media.query().queries().is_empty());
    assert!(surgeist_css::CssMediaQueryList::try_new(Vec::new()).is_none());

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_sheet("@media {} .after { color: red; }")
            .expect("empty media list is valid"),
        report.syntax().clone()
    );
}

#[test]
fn mq4_comparisons_and_discrete_features_remain_typed() {
    assert!(matches!(
        parsed_feature("(width >= 600px)"),
        CssMediaFeatureQuery::Width(value)
            if value.comparison() == Some(CssQueryComparison::GreaterThanOrEqual)
    ));
    assert!(matches!(
        parsed_feature("(hover: hover)"),
        CssMediaFeatureQuery::Hover(surgeist_css::CssHoverCapability::Hover)
    ));
    assert!(matches!(
        parsed_feature("(prefers-color-scheme: dark)"),
        CssMediaFeatureQuery::PrefersColorScheme(surgeist_css::CssColorSchemePreference::Dark)
    ));
    assert!(matches!(
        parsed_feature("(display-mode: standalone)"),
        CssMediaFeatureQuery::DisplayMode(surgeist_css::CssDisplayMode::Standalone)
    ));
}
