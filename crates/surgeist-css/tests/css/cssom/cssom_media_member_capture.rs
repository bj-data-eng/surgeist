#![forbid(unsafe_code)]
//! Ordered captures for the selected CSSOM MediaList byte-comparison algorithm.
use surgeist_css::{
    CssMediaCssomSerializationError, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, parse_media_query_list,
};

fn kind(error: CssMediaCssomSerializationError) -> Kind {
    let CssMediaCssomSerializationError::Resource { error, .. } = error else {
        panic!("expected resource failure: {error:?}");
    };
    error.kind()
}

#[test]
fn ordered_captures_keep_duplicates_recovery_and_owned_input_origins() {
    let source = String::from("SCREEN, ???, print, SCREEN, (future: AbC)");
    let report = parse_media_query_list(&source);
    let original = report.clone();
    let captures = report.syntax().serialize_cssom_members().unwrap();
    assert_eq!(
        captures
            .iter()
            .map(|capture| capture.as_css())
            .collect::<Vec<_>>(),
        ["screen", "not all", "print", "screen", "(future: AbC)"]
    );
    for (capture, query) in captures.iter().zip(report.syntax().queries()) {
        assert!(
            matches!(capture.origin_at(0), Some(CssSerializedOrigin::Token(origin))
            if origin == query.origin())
        );
    }
    assert_eq!(report, original);
    assert_eq!(report.diagnostics().len(), 1);
    drop(report);
    drop(original);
    drop(source);
    assert_eq!(captures[1].as_css(), "not all");
    assert!(captures[1].origin_at(0).is_some());
}

#[test]
fn cumulative_member_limits_exclude_separators_and_retry_independently() {
    let report = parse_media_query_list("screen, print");
    let exact = Limits::new(5, 5, 11);
    let captures = report
        .syntax()
        .serialize_cssom_members_with_limits(exact)
        .unwrap();
    assert_eq!(captures[0].as_css(), "screen");
    assert_eq!(captures[1].as_css(), "print");
    for (limits, expected) in [
        (Limits::new(4, 5, 11), Kind::InputNodeLimit),
        (Limits::new(5, 4, 11), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, 10), Kind::ByteLimit),
    ] {
        assert_eq!(
            kind(
                report
                    .syntax()
                    .serialize_cssom_members_with_limits(limits)
                    .unwrap_err()
            ),
            expected
        );
        let retry = report
            .syntax()
            .serialize_cssom_members_with_limits(exact)
            .unwrap();
        assert_eq!(
            retry
                .iter()
                .map(|capture| capture.as_css())
                .collect::<Vec<_>>(),
            ["screen", "print"]
        );
        assert_eq!(retry[0].origin_at(0), captures[0].origin_at(0));
    }
}

#[test]
fn recovered_expansion_and_empty_aggregate_have_their_real_tariffs() {
    let recovered = parse_media_query_list("???, screen");
    let exact = Limits::new(4, 6, 13);
    assert_eq!(
        recovered
            .syntax()
            .serialize_cssom_members_with_limits(exact)
            .unwrap()
            .len(),
        2
    );
    for (limits, expected) in [
        (Limits::new(3, 6, 13), Kind::InputNodeLimit),
        (Limits::new(4, 5, 13), Kind::ProjectionNodeLimit),
        (Limits::new(4, 6, 12), Kind::ByteLimit),
    ] {
        assert_eq!(
            kind(
                recovered
                    .syntax()
                    .serialize_cssom_members_with_limits(limits)
                    .unwrap_err()
            ),
            expected
        );
    }
    let empty = parse_media_query_list("");
    assert!(
        empty
            .syntax()
            .serialize_cssom_members_with_limits(Limits::new(1, 1, 0))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        kind(
            empty
                .syntax()
                .serialize_cssom_members_with_limits(Limits::new(0, 1, 0))
                .unwrap_err()
        ),
        Kind::InputNodeLimit
    );
}
