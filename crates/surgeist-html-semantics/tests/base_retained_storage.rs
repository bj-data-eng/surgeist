//! Actual backing is distinct from logical formation admission.
use surgeist_html::DocumentString;
use surgeist_html_semantics::*;
fn text(value: &str, spare: usize) -> DocumentString {
    let mut units = Vec::with_capacity(value.len() + spare);
    units.extend(value.encode_utf16());
    DocumentString::from_utf16(units)
}
fn facts(
    spare_raw: usize,
    spare_fallback: usize,
    spare_result: usize,
    spare_records: usize,
) -> HtmlParseBaseFacts {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let fallback = HtmlBaseUrlRecord::observed(
        revision.clone(),
        text("about:blank", spare_fallback),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    let parsed = HtmlBaseUrlRecord::observed(
        revision.clone(),
        text("https://example.test/", spare_result),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    // No document URL: the observation's private fallback is still retained.
    let document = HtmlBaseDocumentFacts::new(
        revision.clone(),
        None,
        HtmlAboutBaseUrl::Unavailable,
        false,
        Some(encoding.clone()),
    )
    .unwrap();
    let mut observations = Vec::with_capacity(1 + spare_records);
    observations.push(
        HtmlBaseParseObservation::new(
            text("x", spare_raw),
            fallback,
            encoding,
            revision,
            HtmlBaseParseOutcome::Parsed {
                record: parsed,
                base_allowed: HtmlBaseAllowed::Allowed,
            },
        )
        .unwrap(),
    );
    HtmlParseBaseFacts::new(
        document,
        observations,
        HtmlBaseFactsLimits::new(256, 1024, 10000),
    )
    .unwrap()
}
#[test]
fn spare_raw_private_fallback_and_result_capacity_do_not_change_logical_formation() {
    let small = facts(0, 0, 0, 0);
    let large = facts(37, 43, 59, 0);
    assert_eq!(small.formation_usage(), large.formation_usage());
    let a = small.retained_storage_usage();
    let b = large.retained_storage_usage();
    assert_eq!(b.utf16_extent() - a.utf16_extent(), 37 + 43 + 59);
    assert_eq!(
        b.retained_bytes() - a.retained_bytes(),
        (37 + 43 + 59) * size_of::<u16>()
    );
}
#[test]
fn empty_spare_observation_bank_is_retained_and_checked_without_logical_string_units() {
    let make = |capacity| {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 0, 0, 0);
        let document =
            HtmlBaseDocumentFacts::new(revision, None, HtmlAboutBaseUrl::Unavailable, false, None)
                .unwrap();
        HtmlParseBaseFacts::new(
            document,
            Vec::with_capacity(capacity),
            HtmlBaseFactsLimits::new(32, 0, 0),
        )
        .unwrap()
    };
    let empty = make(0);
    let spare = make(19);
    assert_eq!(spare.formation_usage().utf16_extent(), 0);
    assert_eq!(spare.retained_storage_usage().utf16_extent(), 0);
    assert_eq!(
        spare.retained_storage_usage().retained_bytes()
            - empty.retained_storage_usage().retained_bytes(),
        19 * size_of::<HtmlBaseParseObservation>()
    );
}

#[test]
fn hyperlink_snapshot_footprint_keeps_only_selected_observation_and_is_not_semantic_equality() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
    let base = HtmlBaseUrlRecord::observed(
        revision.clone(),
        text("https://base/", 23),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let result = HtmlBaseUrlRecord::observed(
        revision.clone(),
        text("https://result/", 41),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let make = |spare, extra| {
        let mut rows = vec![HtmlHyperlinkParseObservation::new(
            text("chosen", spare),
            HtmlHyperlinkParseOutcome::Parsed(result.clone()),
        )];
        if extra {
            rows.push(HtmlHyperlinkParseObservation::new(
                text("unselected", 311),
                HtmlHyperlinkParseOutcome::Failure,
            ));
        }
        HtmlHyperlinkParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            rows,
            HtmlBaseFactsLimits::new(128, 10000, 10000),
        )
        .unwrap()
    };
    let small = make(0, false);
    let large = make(67, true);
    let resolve = |facts: &HtmlHyperlinkParseFacts| {
        facts
            .resolve_with(&"chosen".into(), &base, &encoding, &revision, || {
                Ok::<_, ()>(())
            })
            .unwrap()
    };
    let a = resolve(&small);
    let b = resolve(&large);
    assert_eq!(a, b, "derived footprint is not URL/parse identity");
    assert_eq!(
        b.retained_storage_usage().utf16_extent() - a.retained_storage_usage().utf16_extent(),
        67
    );
    assert_eq!(
        b.retained_storage_usage().retained_bytes() - a.retained_storage_usage().retained_bytes(),
        67 * size_of::<u16>()
    );
    assert!(
        large.retained_storage_usage().retained_bytes()
            > b.retained_storage_usage().retained_bytes() + 311 * size_of::<u16>()
    );
}
#[test]
fn hyperlink_empty_bank_spare_capacity_is_counted() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
    let base = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "x".into(),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let make = |capacity| {
        HtmlHyperlinkParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            Vec::with_capacity(capacity),
            HtmlBaseFactsLimits::new(128, 10000, 10000),
        )
        .unwrap()
    };
    let a = make(0);
    let b = make(17);
    assert_eq!(
        a.formation_usage().utf16_extent(),
        b.formation_usage().utf16_extent()
    );
    assert_eq!(
        b.retained_storage_usage().retained_bytes() - a.retained_storage_usage().retained_bytes(),
        17 * size_of::<HtmlHyperlinkParseObservation>()
    );
}

#[test]
fn standalone_url_record_footprint_preserves_actual_spare_backing_without_bank() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 0, 0, 0);
    let make = |spare| {
        HtmlBaseUrlRecord::observed(
            revision.clone(),
            text("about:blank", spare),
            HtmlBaseUrlScheme::Other,
            true,
        )
        .unwrap()
    };
    let a = make(0);
    let b = make(71);
    assert_eq!(
        b.retained_storage_usage().utf16_extent() - a.retained_storage_usage().utf16_extent(),
        71
    );
    assert_eq!(
        b.retained_storage_usage().retained_bytes() - a.retained_storage_usage().retained_bytes(),
        71 * size_of::<u16>()
    );
}
