#![forbid(unsafe_code)]
//! Characterization of the dated CSSOM media-feature cells under the adopted
//! authored-value policy. MQ5 supplies the domains; Values4 supplies both ratio
//! operands; numeric lexemes and slash spacing remain the selected media policy.

use surgeist_css::{
    CssCalculationExpressionRef, CssMediaConditionKind, CssMediaCssomSerializationError,
    CssMediaFeatureQuery, CssMediaGridRef, CssMediaQuery, CssMediaRangeRef, CssMediaResolutionRef,
    CssMediaSerializationError, CssQueryComparison, CssRecoveryAction, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind as ResourceKind,
    CssSpecifiedValueSerializationLimits as Limits, CssUnknownMediaFeatureReason, CssValueOrigin,
    parse_cssom_media_query, parse_media_query_list,
};

fn query(source: &str) -> CssMediaQuery {
    let report = parse_cssom_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.into_parts().0.expect("one clean media query")
}

fn feature(query: &CssMediaQuery) -> &CssMediaFeatureQuery {
    let CssMediaQuery::Condition(condition) = query else {
        panic!("expected a media condition");
    };
    let CssMediaConditionKind::Feature(feature) = condition.kind() else {
        panic!("expected an admitted known feature");
    };
    feature
}

fn resource_kind(error: &CssMediaCssomSerializationError) -> ResourceKind {
    let CssMediaCssomSerializationError::Resource { error, .. } = error else {
        panic!("expected resource failure: {error:?}");
    };
    error.kind()
}

fn parsed_span(origin: &CssValueOrigin, source: &str, start: usize, end: usize) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("expected the original parsed source");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

fn exact_projection(source: &str, expected: &str, nodes: usize) {
    let authored = query(source);
    let original = authored.clone();
    let bytes = expected.len();
    let adequate = Limits::new(nodes, nodes, bytes);
    assert_eq!(
        authored
            .serialize_cssom_with_limits(adequate)
            .unwrap()
            .as_css(),
        expected,
        "{source}"
    );
    assert_eq!(authored.serialize_cssom().unwrap().as_css(), expected);
    for (limits, kind) in [
        (
            Limits::new(nodes - 1, nodes, bytes),
            ResourceKind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, bytes),
            ResourceKind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, bytes - 1),
            ResourceKind::ByteLimit,
        ),
        (Limits::new(0, nodes, bytes), ResourceKind::InputNodeLimit),
        (
            Limits::new(nodes, 0, bytes),
            ResourceKind::ProjectionNodeLimit,
        ),
        (Limits::new(nodes, nodes, 0), ResourceKind::ByteLimit),
    ] {
        let error = authored.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(resource_kind(&error), kind, "{source}: {limits:?}");
        assert_eq!(authored, original, "failed projection must be atomic");
        assert_eq!(
            authored
                .serialize_cssom_with_limits(adequate)
                .unwrap()
                .as_css(),
            expected,
            "an adequate retry must retain the authored query"
        );
    }
    assert_eq!(authored, original);
}

#[test]
fn dated_media_feature_cells_preserve_lexical_quantities_and_defined_keywords() {
    // Query + condition + backing block = 3 visits. A plain name/colon/value
    // adds 3; the spaced <= case adds 6; an explicit ratio adds 5. Generated
    // denominator bytes do not invent an additional input/projection visit.
    for (source, expected, name, nodes) in [
        ("(WIDTH:+001.5PX)", "(width: +001.5PX)", "width", 6),
        ("(HEIGHT <= -100px)", "(height <= -100px)", "height", 9),
        (
            "(DEVICE-WIDTH:-0px)",
            "(device-width: -0px)",
            "device-width",
            6,
        ),
        (
            "(device-height:4em)",
            "(device-height: 4em)",
            "device-height",
            6,
        ),
        (
            "(ASPECT-RATIO:2)",
            "(aspect-ratio: 2 / 1)",
            "aspect-ratio",
            6,
        ),
        (
            "(device-aspect-ratio:32/18)",
            "(device-aspect-ratio: 32 / 18)",
            "device-aspect-ratio",
            8,
        ),
        ("(COLOR:-1)", "(color: -1)", "color", 6),
        ("(color-index:8)", "(color-index: 8)", "color-index", 6),
        ("(MONOCHROME:0)", "(monochrome: 0)", "monochrome", 6),
        ("(resolution:96dpi)", "(resolution: 96dpi)", "resolution", 6),
        ("(GRID:-0)", "(grid: -0)", "grid", 6),
        (
            "(ORIENTATION:LANDSCAPE)",
            "(orientation: landscape)",
            "orientation",
            6,
        ),
        ("(SCAN:INTERLACE)", "(scan: interlace)", "scan", 6),
        // The other two values explicitly defined by the dated table.
        (
            "(ORIENTATION:PORTRAIT)",
            "(orientation: portrait)",
            "orientation",
            6,
        ),
        ("(SCAN:PROGRESSIVE)", "(scan: progressive)", "scan", 6),
    ] {
        let authored = query(source);
        assert_eq!(feature(&authored).name(), name);
        assert_eq!(authored.serialize().unwrap().as_css(), expected);
        exact_projection(source, expected, nodes);
    }
}

#[test]
fn ratios_emit_both_operands_without_division_reduction_or_zero_rejection() {
    for (source, expected, nodes) in [
        ("(aspect-ratio:1.5/0)", "(aspect-ratio: 1.5 / 0)", 8),
        ("(aspect-ratio:0/0)", "(aspect-ratio: 0 / 0)", 8),
        ("(device-aspect-ratio:2)", "(device-aspect-ratio: 2 / 1)", 6),
        ("(aspect-ratio:+002/0001)", "(aspect-ratio: +002 / 0001)", 8),
        // Whitespace is retained work even where slash formatting consumes it.
        ("(aspect-ratio:2 / 1)", "(aspect-ratio: 2 / 1)", 10),
        // Functions and their retained children are charged, without evaluation.
        (
            "(aspect-ratio:calc(1 + 1)/calc(2))",
            "(aspect-ratio: calc(1 + 1) / calc(2))",
            14,
        ),
        ("(aspect-ratio:calc(2))", "(aspect-ratio: calc(2) / 1)", 7),
        (
            "(aspect-ratio:calc(-1)/2)",
            "(aspect-ratio: calc(-1) / 2)",
            9,
        ),
    ] {
        assert!(matches!(
            feature(&query(source)),
            CssMediaFeatureQuery::AspectRatio(_) | CssMediaFeatureQuery::DeviceAspectRatio(_)
        ));
        exact_projection(source, expected, nodes);
    }
}

#[test]
fn signed_units_ranges_and_symbolic_math_remain_in_the_authored_phase() {
    // Each total is 3 plus the retained token/function graph: 4 for >=-2x,
    // 8 for the calc sum, 6/5 for the two chains, and 4 for calc(0.5).
    for (source, expected, nodes) in [
        ("(resolution>=-2x)", "(resolution >= -2x)", 7),
        (
            "(resolution:calc(1dppx + 96dpi))",
            "(resolution: calc(1dppx + 96dpi))",
            11,
        ),
        ("(80em>=width>-100px)", "(80em >= width > -100px)", 9),
        ("(100px<width<10px)", "(100px < width < 10px)", 8),
        ("(MIN-COLOR:-1)", "(min-color: -1)", 6),
        ("(grid:calc(0.5))", "(grid: calc(0.5))", 7),
        (
            "(infinite>=resolution>0dppx)",
            "(infinite >= resolution > 0dppx)",
            9,
        ),
    ] {
        exact_projection(source, expected, nodes);
    }

    let authored = query("(resolution>=-2x)");
    let CssMediaFeatureQuery::Resolution(range) = feature(&authored) else {
        panic!("expected resolution");
    };
    let CssMediaRangeRef::FeatureFirst { comparison, value } = range.view() else {
        panic!("expected feature-first range");
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThanOrEqual);
    let CssMediaResolutionRef::Numeric(calculation) = value.view() else {
        panic!("expected a retained resolution calculation");
    };
    let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
        panic!("expected a signed literal");
    };
    assert_eq!(value.literal().representation(), "-2");
    assert_eq!(value.literal().unit(), Some("x"));

    let authored = query("(grid:calc(0.5))");
    let CssMediaFeatureQuery::Grid(grid) = feature(&authored) else {
        panic!("expected grid");
    };
    let CssMediaGridRef::Calculation(calculation) = grid.view() else {
        panic!("expected deferred integer calculation");
    };
    assert!(calculation.requires_rounding());
    assert!(matches!(
        calculation.expression(),
        CssCalculationExpressionRef::NestedCalc(_)
    ));

    let authored = query("(100px<width<10px)");
    let CssMediaFeatureQuery::Width(range) = feature(&authored) else {
        panic!("expected width");
    };
    let CssMediaRangeRef::Ascending {
        left,
        left_inclusive,
        right,
        right_inclusive,
    } = range.view()
    else {
        panic!("expected the original empty ascending interval");
    };
    assert!(!left_inclusive);
    assert!(!right_inclusive);
    for (value, expected) in [(left, "100"), (right, "10")] {
        let CssCalculationExpressionRef::Value(value) = value.calculation().expression() else {
            panic!("expected exact range bound");
        };
        assert_eq!(value.literal().representation(), expected);
        assert_eq!(value.literal().unit(), Some("px"));
    }
}

#[test]
fn ratio_metadata_separates_original_numeric_tokens_from_the_default_denominator() {
    let source = "/*😀*/\n(ASPECT-RATIO:+002)";
    let authored = query(source);
    let original = authored.clone();
    let CssMediaFeatureQuery::AspectRatio(range) = feature(&authored) else {
        panic!("expected ratio");
    };
    let CssMediaRangeRef::Plain { value } = range.view() else {
        panic!("expected plain ratio");
    };
    assert!(value.denominator_is_omitted());
    assert_eq!(value.denominator().origin(), &CssValueOrigin::Programmatic);
    let position = value.numerator().position().unwrap();
    assert_eq!(position.byte_offset().value(), source.find("+002").unwrap());
    assert_eq!(position.line().value(), 1);
    assert_eq!(position.column().value(), 14);
    let expected = "(aspect-ratio: +002 / 1)";
    let output = authored
        .serialize_cssom_with_limits(Limits::new(6, 6, 24))
        .unwrap();
    assert_eq!(output.as_css(), expected);
    let Some(CssSerializedOrigin::Token(origin)) = output.origin_at(expected.find("+002").unwrap())
    else {
        panic!("numeric output must retain its original token");
    };
    let start = source.find("+002").unwrap();
    parsed_span(origin, source, start, start + 4);
    for offset in [expected.find('/').unwrap(), expected.rfind('1').unwrap()] {
        assert_eq!(
            output.origin_at(offset).cloned(),
            Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
        );
    }
    // B23 fails on the closing parenthesis after the generated denominator.
    let error = authored
        .serialize_cssom_with_limits(Limits::new(6, 6, 23))
        .unwrap_err();
    assert_eq!(resource_kind(&error), ResourceKind::ByteLimit);
    assert_eq!(authored, original);
    assert_eq!(authored.serialize_cssom().unwrap().as_css(), expected);

    let source = "/*😀*/\n(device-aspect-ratio:32/18)";
    let authored = query(source);
    let expected = "(device-aspect-ratio: 32 / 18)";
    let output = authored.serialize_cssom().unwrap();
    assert_eq!(output.as_css(), expected);
    for spelling in ["32", "18"] {
        let Some(CssSerializedOrigin::Token(origin)) =
            output.origin_at(expected.find(spelling).unwrap())
        else {
            panic!("explicit ratio operand must retain its token");
        };
        let start = source.find(spelling).unwrap();
        parsed_span(origin, source, start, start + spelling.len());
    }
}

#[test]
fn numeric_domains_do_not_turn_unknown_features_into_ignored_queries() {
    for (source, expected, reason) in [
        (
            "(FuTuRe:ACTIVE)",
            "(future: ACTIVE)",
            CssUnknownMediaFeatureReason::UnknownName,
        ),
        (
            "(-vendor-future:ACTIVE)",
            "(-vendor-future: ACTIVE)",
            CssUnknownMediaFeatureReason::UnknownName,
        ),
        (
            "(grid:2)",
            "(grid: 2)",
            CssUnknownMediaFeatureReason::InvalidValue,
        ),
        (
            "(grid>0)",
            "(grid > 0)",
            CssUnknownMediaFeatureReason::InvalidOperation,
        ),
        (
            "(min-grid:1)",
            "(min-grid: 1)",
            CssUnknownMediaFeatureReason::UnknownName,
        ),
        (
            "(aspect-ratio:-1)",
            "(aspect-ratio: -1)",
            CssUnknownMediaFeatureReason::InvalidValue,
        ),
    ] {
        let authored = query(source);
        let original = authored.clone();
        let CssMediaQuery::Condition(condition) = &authored else {
            panic!("expected retained condition");
        };
        let CssMediaConditionKind::UnknownFeature(value) = condition.kind() else {
            panic!("expected unknown feature, not ignored query: {source}");
        };
        assert_eq!(value.reason(), reason);
        assert_eq!(value.authored(), Some(source));
        assert_eq!(authored.serialize_cssom().unwrap().as_css(), expected);
        assert_eq!(authored, original);
    }
    // Pinned MQ5 WD20260219 section 3 admits -1 as an <mf-value> number.
    // Values4 WD20240312 section 5.7 excludes negative literal ratio operands;
    // the complete slash pair therefore takes MQ5's <general-enclosed> fallback.
    for (source, expected) in [
        ("(aspect-ratio:-1/2)", "(aspect-ratio:-1/2)"),
        ("(aspect-ratio:1/-2)", "(aspect-ratio:1/-2)"),
    ] {
        let authored = query(source);
        let original = authored.clone();
        let CssMediaQuery::Condition(condition) = &authored else {
            panic!("expected retained condition");
        };
        assert!(matches!(
            condition.kind(),
            CssMediaConditionKind::GeneralEnclosed(_)
        ));
        assert_eq!(authored.serialize_cssom().unwrap().as_css(), expected);
        assert_eq!(authored, original);
    }
    assert_eq!(
        query("(FuTuRe:ACTIVE)").serialize().unwrap().as_css(),
        "(FuTuRe: ACTIVE)"
    );
    let authored = query("Future(\"A\")");
    let CssMediaQuery::Condition(condition) = &authored else {
        panic!("expected condition");
    };
    assert!(matches!(
        condition.kind(),
        CssMediaConditionKind::GeneralEnclosed(_)
    ));
    assert_eq!(
        authored.serialize_cssom().unwrap().as_css(),
        "Future(\"A\")"
    );
    assert_eq!(
        query("not (grid:2)").serialize_cssom().unwrap().as_css(),
        "not (grid: 2)"
    );
}

#[test]
fn cssom_quantity_equality_compares_lexical_bytes_without_unit_conversion() {
    let first = query("(RESOLUTION:96dpi)");
    let same = query("(resolution:96dpi)");
    let equivalent_unit = query("(resolution:1dppx)");
    let originals = (first.clone(), same.clone(), equivalent_unit.clone());
    // Each query has six visits and emits 19 bytes. Comparison shares I12/P12/B38.
    for (other, expected) in [(&same, true), (&equivalent_unit, false)] {
        assert_eq!(
            first
                .cssom_equals_with_limits(other, Limits::new(12, 12, 38))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(11, 12, 38), ResourceKind::InputNodeLimit),
            (Limits::new(12, 11, 38), ResourceKind::ProjectionNodeLimit),
            (Limits::new(12, 12, 37), ResourceKind::ByteLimit),
        ] {
            let error = first.cssom_equals_with_limits(other, limits).unwrap_err();
            assert_eq!(resource_kind(&error), kind);
            assert_eq!(
                first
                    .cssom_equals_with_limits(other, Limits::new(12, 12, 38))
                    .unwrap(),
                expected
            );
        }
    }
    assert_ne!(first, same);
    assert!(
        !query("(width:+001.5PX)")
            .cssom_equals(&query("(width:1.5px)"))
            .unwrap()
    );
    assert_eq!((first, same, equivalent_unit), originals);
}

#[test]
fn feature_lists_share_resources_and_report_the_later_original_member() {
    let source = "/*😀*/\n(width:1px), (resolution:96dpi)";
    let report = parse_media_query_list(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let original = report.clone();
    let list = report.syntax();
    let expected = "(width: 1px), (resolution: 96dpi)";
    // One list aggregate plus two six-node feature graphs; 12 + 2 + 19 bytes.
    assert_eq!(
        list.serialize_cssom_with_limits(Limits::new(13, 13, 33))
            .unwrap()
            .as_css(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(12, 13, 33), ResourceKind::InputNodeLimit),
        (Limits::new(13, 12, 33), ResourceKind::ProjectionNodeLimit),
        (Limits::new(13, 13, 32), ResourceKind::ByteLimit),
        // Exhaust the node budget exactly after the first complete member.
        (Limits::new(7, 13, 33), ResourceKind::InputNodeLimit),
        (Limits::new(13, 7, 33), ResourceKind::ProjectionNodeLimit),
        // The first output and separator fit; the second opening token does not.
        (Limits::new(13, 13, 14), ResourceKind::ByteLimit),
    ] {
        let error = list.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(resource_kind(&error), kind);
        let CssValueOrigin::Parsed(origin) = error.origin() else {
            panic!("later-member failure must retain its source");
        };
        assert_eq!(origin.source().as_str(), source);
        assert!(origin.span().start().byte_offset().value() >= source.find("(resolution").unwrap());
        assert_eq!(report, original);
        assert_eq!(
            list.serialize_cssom_with_limits(Limits::new(13, 13, 33))
                .unwrap()
                .as_css(),
            expected
        );
    }
    for limits in [Limits::new(7, 13, 33), Limits::new(13, 7, 33)] {
        let error = list.serialize_cssom_with_limits(limits).unwrap_err();
        assert_eq!(error.origin(), list.queries()[1].origin());
    }
    assert_eq!(report, original);
}

#[test]
fn malformed_neighbor_recovery_keeps_feature_reports_and_atomic_resource_retries() {
    let source = "screen, ???, print";
    let report = parse_media_query_list(source);
    let original = report.clone();
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find("???").unwrap()
    );
    let expected = "screen, not all, print";
    // List1 + typed screen2 + ignored query1 + typed print2 = I6;
    // the two generated ignored-member keywords extend that to P8. B22.
    assert_eq!(
        report
            .syntax()
            .serialize_cssom_with_limits(Limits::new(6, 8, 22))
            .unwrap()
            .as_css(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(5, 8, 22), ResourceKind::InputNodeLimit),
        (Limits::new(6, 7, 22), ResourceKind::ProjectionNodeLimit),
        (Limits::new(6, 8, 21), ResourceKind::ByteLimit),
    ] {
        let error = report
            .syntax()
            .serialize_cssom_with_limits(limits)
            .unwrap_err();
        assert_eq!(resource_kind(&error), kind);
        assert_eq!(report, original);
        assert_eq!(
            report
                .syntax()
                .serialize_cssom_with_limits(Limits::new(6, 8, 22))
                .unwrap()
                .as_css(),
            expected
        );
    }
    assert!(matches!(
        report.syntax().serialize(),
        Err(CssMediaSerializationError::RecoveredNever { .. })
    ));
    let output = report.syntax().serialize_cssom().unwrap();
    assert_eq!(
        output.origin_at(8).cloned(),
        Some(CssSerializedOrigin::Token(
            report.syntax().queries()[1].origin().clone()
        ))
    );
    assert_eq!(report, original);

    let single = parse_cssom_media_query(source);
    assert!(single.syntax().is_none());
    assert_eq!(single.diagnostics(), report.diagnostics());
}
