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
        Err(CssSourceRetentionError::Limit("source visits"))
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
        Err(CssSourceRetentionError::Limit("source depth"))
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
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
    let mut refused = CssSourceRetention::new(0, 10_000, 100);
    assert_eq!(
        refused.visit(report.syntax()),
        Err(CssSourceRetentionError::Limit("retained source bytes"))
    );
    // Root payload and its actual source admission are the only reached actions.
    assert_eq!(refused.steps(), 2);
    assert_eq!(refused.usage(), CssSourceStorageUsage::default());
}
