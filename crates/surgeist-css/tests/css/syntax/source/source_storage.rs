use surgeist_css::*;

#[test]
fn source_traversal_charges_exact_work_and_independent_representation_depth() {
    let report = parse_sheet(".a { --token: calc(1 + var(--other)); }");
    let source = report.syntax().parse_source().unwrap().source();
    let bytes = source.storage_usage().retained_bytes();
    let mut measured = CssSourceRetention::new(bytes, 10_000, 100);
    measured.visit(report.syntax()).unwrap();
    assert_eq!(measured.usage(), source.storage_usage());
    let steps = measured.steps();
    let mut exact = CssSourceRetention::new(bytes, steps, 100);
    exact.visit(report.syntax()).unwrap();
    let mut short = CssSourceRetention::new(bytes, steps - 1, 100);
    assert_eq!(
        short.visit(report.syntax()),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Visits
        ))
    );

    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("style");
    };
    let origin = rule.declarations()[0].parsed_value().unwrap();
    let mut exact = CssSourceRetention::new(bytes, 100, 5);
    exact.at_depth(4, |sources| sources.visit(origin)).unwrap();
    let mut short = CssSourceRetention::new(bytes, 100, 4);
    assert_eq!(
        short.at_depth(4, |sources| sources.visit(origin)),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Depth
        ))
    );
}

#[test]
fn source_storage_deduplicates_identity_and_stops_before_descendants_on_failure() {
    let report = parse_sheet(".a { --token: value }");
    let source = report.syntax().parse_source().unwrap().source();
    let bytes = source.storage_usage().retained_bytes();
    let mut admission = CssSourceRetention::new(bytes, 10_000, 100);
    admission.visit(report.syntax()).unwrap();
    admission.visit(report.syntax()).unwrap();
    assert_eq!(admission.usage(), source.storage_usage());
    let other = parse_sheet(".a { --token: value }");
    assert_eq!(report.syntax(), other.syntax());
    assert_eq!(
        admission.visit(other.syntax()),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
    );
    let mut refused = CssSourceRetention::new(0, 10_000, 100);
    assert_eq!(
        refused.visit(report.syntax()),
        Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::RetainedBytes
        ))
    );
    // Root payload and its actual source admission are the only reached actions.
    assert_eq!(refused.steps(), 2);
    assert_eq!(refused.usage(), CssSourceStorageUsage::default());
}

#[test]
fn source_retention_refusals_display_their_resource_or_cause_without_a_source() {
    use std::error::Error as _;
    let limit = CssSourceRetentionError::limit;
    let cases = [
        (
            limit(CssSourceRetentionResource::Depth),
            "CSS source retention source depth limit exceeded",
        ),
        (
            limit(CssSourceRetentionResource::Visits),
            "CSS source retention source visits limit exceeded",
        ),
        (
            limit(CssSourceRetentionResource::RetainedBytes),
            "CSS source retention retained source bytes limit exceeded",
        ),
        (
            limit(CssSourceRetentionResource::Storage),
            "CSS source retention source storage limit exceeded",
        ),
        (
            CssSourceRetentionError::allocation(),
            "CSS source retention allocation failed",
        ),
        (
            CssSourceRetentionError::caller_admission(),
            "CSS source retention was refused by the caller's admission",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
        assert!(error.source().is_none(), "{text}");
    }

    // A refusal reached by an actual traversal displays the same text.
    let report = parse_sheet(".a { color: red; }");
    let bytes = report
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    let refused = CssSourceRetention::new(bytes, 0, 100)
        .visit(report.syntax())
        .unwrap_err();
    assert_eq!(
        refused.to_string(),
        "CSS source retention source visits limit exceeded"
    );
}
