#![forbid(unsafe_code)]
use surgeist_html::{DocumentString, HtmlReferrerPolicy};
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

fn record(
    revision: &HtmlBaseFactsRevision,
    url: &str,
    blank: bool,
    fragment: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        url.into(),
        HtmlBaseUrlScheme::Other,
        blank,
        fragment,
    )
    .unwrap()
}
fn environment(base: &HtmlBaseUrlRecord, encoding: &HtmlBaseEncoding) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            base.revision().clone(),
            Some(base.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap()
}
fn bank(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    observations: Vec<HtmlEncodingUrlParseObservation>,
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        base.revision().clone(),
        observations,
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap()
}

fn rows(report: &HtmlParseReport) -> Vec<HtmlParseDocumentMeta<'_>> {
    let mut result = Vec::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(meta) = row.document_meta() {
                    result.push(meta);
                }
            }
            HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    result
}
fn parse(source: &str) -> HtmlParseReport {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
        .parse()
        .unwrap()
}
fn known_refresh(meta: &HtmlParseDocumentMeta<'_>) -> bool {
    let HtmlParseMetaRefresh::Known(value) = meta.refresh() else {
        panic!("known actual refresh history")
    };
    value
}
#[test]
fn source_created_documents_have_independent_untouched_meta_history() {
    // Template contents are not in the ordinary Document tree, including the
    // separately created inert owner. Their Meta insertions do not set either.
    let report = parse(
        "<!doctype html><template><meta http-equiv=content-language content=fr><meta name=referrer content=never><meta http-equiv=refresh content=0></template>",
    );
    let states = rows(&report);
    assert_eq!(states.len(), 2);
    assert_ne!(states[0].document().id(), states[1].document().id());
    for state in states {
        assert_eq!(state.profile(), HtmlParseMetaProfile::OfflineDoNothingNoUi);
        assert!(state.pragma_language().is_none());
        assert_eq!(
            state.referrer_policy(),
            HtmlReferrerPolicy::StrictOriginWhenCrossOrigin
        );
        assert!(!known_refresh(&state));
        assert!(state.first_missing().is_none());
    }
}
#[test]
fn actual_language_and_referrer_transitions_preserve_prior_values_on_invalid_later_meta() {
    // HTML877d11691+: comma anywhere rejects language; first non-whitespace
    // token is retained exactly. Meta referrer has no trimming and keeps prior
    // policy when later invalid, including a content attribute which is empty.
    let report = parse(
        "<!doctype html><meta http-equiv=content-language content='  EN-us other'><meta http-equiv=content-language content='de,FR'><meta http-equiv=content-language content='  '><meta name=ReFeRrEr content=always><meta name=referrer content=' origin '><meta name=referrer content=''>",
    );
    let states = rows(&report);
    assert_eq!(states.len(), 1);
    assert_eq!(
        states[0].pragma_language(),
        Some(&DocumentString::from("EN-us"))
    );
    assert_eq!(states[0].referrer_policy(), HtmlReferrerPolicy::UnsafeUrl);
    assert!(states[0].first_missing().is_none());
    assert!(!known_refresh(&states[0]));
}
#[test]
fn independent_intrinsics_continue_after_missing_external_domains_and_keep_first_cause() {
    let report = parse(
        r#"<!doctype html><head><meta http-equiv=default-style content=chosen><meta http-equiv=content-security-policy content="base-uri 'none'"><meta name=theme-color content=red><meta name=color-scheme content=dark><meta http-equiv=content-language content=ja><meta name=referrer content=never></head>"#,
    );
    let states = rows(&report);
    let state = &states[0];
    assert_eq!(state.pragma_language(), Some(&DocumentString::from("ja")));
    assert_eq!(state.referrer_policy(), HtmlReferrerPolicy::NoReferrer);
    assert!(!known_refresh(state));
    let first = state.first_missing().unwrap();
    assert_eq!(first.kind(), &HtmlParseMetaMissingKind::PreferredStyle);
    assert_eq!(first.document().id(), state.document().id());
    assert!(first.cause().is_some());
    for (domain, kind) in [
        (
            HtmlParseMetaDomain::PreferredStyle,
            HtmlParseMetaMissingKind::PreferredStyle,
        ),
        (HtmlParseMetaDomain::Csp, HtmlParseMetaMissingKind::Csp),
        (
            HtmlParseMetaDomain::ThemeColor,
            HtmlParseMetaMissingKind::ThemeColor,
        ),
        (
            HtmlParseMetaDomain::ColorScheme,
            HtmlParseMetaMissingKind::ColorScheme,
        ),
    ] {
        let HtmlParseMetaDisposition::Unavailable(missing) = state.disposition(domain) else {
            panic!("actual reached external operation")
        };
        assert_eq!(missing.kind(), &kind);
        assert_eq!(missing.document().id(), state.document().id());
    }
}
#[test]
fn known_true_refresh_returns_before_later_missing_url_observation() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    // Arbitrarily large delay is never narrowed to a machine integer. The
    // closed Do-nothing branch still sets the flag before later early return.
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><meta http-equiv=refresh content=999999999999999999999999999999999999><meta http-equiv=refresh content='0; url=unobserved'>"), limits())
        .with_base_facts(environment(&base, &encoding)).unwrap().parse().unwrap();
    let states = rows(&report);
    assert!(known_refresh(&states[0]));
    assert!(states[0].first_missing().is_none());
}
#[test]
fn unavailable_refresh_never_becomes_false_or_speculatively_runs_later_refresh() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><meta http-equiv=refresh content='0; url=unobserved'><meta http-equiv=refresh content=0><meta http-equiv=content-language content=ko>"), limits())
        .with_base_facts(environment(&base, &encoding)).unwrap().parse().unwrap();
    let states = rows(&report);
    let HtmlParseMetaRefresh::Unavailable(missing) = states[0].refresh() else {
        panic!("unknown prior refresh remains unavailable")
    };
    assert!(matches!(
        missing.kind(),
        HtmlParseMetaMissingKind::RefreshUrl(_)
    ));
    assert_eq!(
        missing.subject().id(),
        states[0].first_missing().unwrap().subject().id()
    );
    assert_eq!(
        states[0].pragma_language(),
        Some(&DocumentString::from("ko"))
    );
}
#[test]
fn refresh_uses_observed_failure_and_javascript_before_successful_actual_parse() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let js = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "javascript:0".into(),
        HtmlBaseUrlScheme::Javascript,
        false,
        None,
    )
    .unwrap();
    let good = record(&revision, "https://example.test/x", false, None);
    for (source, expected) in [
        (
            "<!doctype html><meta http-equiv=refresh content='0;url=bad'><meta http-equiv=refresh content='0;url=js'>",
            false,
        ),
        (
            "<!doctype html><meta http-equiv=refresh content='0;url=bad'><meta http-equiv=refresh content='0;url=js'><meta http-equiv=refresh content='0;url=x'>",
            true,
        ),
    ] {
        let facts = bank(
            &base,
            &encoding,
            vec![
                HtmlEncodingUrlParseObservation::new(
                    "bad".into(),
                    HtmlEncodingUrlParseOutcome::Failure,
                ),
                HtmlEncodingUrlParseObservation::new(
                    "js".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(js.clone()),
                ),
                HtmlEncodingUrlParseObservation::new(
                    "x".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(good.clone()),
                ),
            ],
        );
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .with_base_facts(environment(&base, &encoding))
            .unwrap()
            .with_encoding_url_facts(vec![facts])
            .parse()
            .unwrap();
        let states = rows(&report);
        assert_eq!(known_refresh(&states[0]), expected);
        assert!(states[0].first_missing().is_none());
    }
}
#[test]
fn base_after_unavailable_csp_is_explicitly_unavailable_and_earlier_frozen_value_survives() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let chosen = record(&revision, "https://example.test/A", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    for (source, frozen_expected) in [
        (
            r#"<!doctype html><head><meta http-equiv=content-security-policy content="base-uri 'none'"><base href=A></head>"#,
            false,
        ),
        (
            r#"<!doctype html><head><base href=A><meta http-equiv=content-security-policy content="base-uri 'none'"></head>"#,
            true,
        ),
    ] {
        let facts = HtmlParseBaseFacts::new(
            environment(&base, &encoding).document().clone(),
            vec![
                HtmlBaseParseObservation::new(
                    "A".into(),
                    base.clone(),
                    encoding.clone(),
                    revision.clone(),
                    HtmlBaseParseOutcome::Parsed {
                        record: chosen.clone(),
                        base_allowed: HtmlBaseAllowed::Allowed,
                    },
                )
                .unwrap(),
            ],
            HtmlBaseFactsLimits::new(20, 10_000, 100_000),
        )
        .unwrap();
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .with_base_facts(facts)
            .unwrap()
            .parse()
            .unwrap();
        let mut cursor = report.metadata();
        let mut count = 0;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if let Some(state) = row.base_state() {
                        count += 1;
                        match state {
                            HtmlParseBaseState::Frozen(value) if frozen_expected => {
                                assert_eq!(value.value(), &chosen)
                            }
                            HtmlParseBaseState::Unavailable { missing, previous }
                                if !frozen_expected =>
                            {
                                assert_eq!(missing.kind(), &HtmlParseMetaMissingKind::Csp);
                                assert!(previous.is_none());
                            }
                            _ => panic!("source-ordered actual Base disposition"),
                        }
                    }
                }
                HtmlParseMetadataStep::Skipped => (),
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(count, 1);
    }
}

#[test]
fn refresh_uses_source_time_base_and_requires_independent_full_revision() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let old = record(&revision, "https://example.test/old/", false, None);
    let new = record(&revision, "https://example.test/new/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlParseBaseFacts::new(
        environment(&old, &encoding).document().clone(),
        vec![
            HtmlBaseParseObservation::new(
                "new".into(),
                old.clone(),
                encoding.clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: new.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    for (source, expected) in [
        (
            "<!doctype html><meta http-equiv=refresh content='0;url=x'><base href=new>",
            false,
        ),
        (
            "<!doctype html><base href=new><meta http-equiv=refresh content='0;url=x'>",
            true,
        ),
    ] {
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .with_base_facts(facts.clone())
            .unwrap()
            .with_encoding_url_facts(vec![
                bank(
                    &old,
                    &encoding,
                    vec![HtmlEncodingUrlParseObservation::new(
                        "x".into(),
                        HtmlEncodingUrlParseOutcome::Failure,
                    )],
                ),
                bank(
                    &new,
                    &encoding,
                    vec![HtmlEncodingUrlParseObservation::new(
                        "x".into(),
                        HtmlEncodingUrlParseOutcome::Parsed(record(
                            &revision,
                            "https://example.test/new/x",
                            false,
                            None,
                        )),
                    )],
                ),
            ])
            .parse()
            .unwrap();
        assert_eq!(known_refresh(&rows(&report)[0]), expected);
    }
    let different_revision = HtmlBaseFactsRevision::new(revision.producer().clone(), 1, 2, 4);
    let wrong = HtmlEncodingUrlParseFacts::new(
        old.clone(),
        encoding.clone(),
        different_revision,
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><meta http-equiv=refresh content='0;url=x'>"),
        limits(),
    )
    .with_base_facts(environment(&old, &encoding))
    .unwrap()
    .with_encoding_url_facts(vec![wrong])
    .parse()
    .unwrap();
    let states = rows(&report);
    let HtmlParseMetaRefresh::Unavailable(missing) = states[0].refresh() else {
        panic!("candidate cannot certify its own revision")
    };
    assert_eq!(
        missing.kind(),
        &HtmlParseMetaMissingKind::RefreshUrl(HtmlParseHyperlinkUnavailable::Observation(
            HtmlEncodingUrlFactsError::RevisionMismatch
        ))
    );
}

#[test]
fn unknown_csp_invalidates_required_url_context_without_erasing_known_document_url() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    for (content, expected) in [("0", true), ("0;url=x", false)] {
        let source = format!(
            "<!doctype html><head><meta http-equiv=content-security-policy content=\"base-uri 'none'\"><meta http-equiv=refresh content='{content}'></head>"
        );
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .with_base_facts(environment(&base, &encoding))
            .unwrap()
            .with_encoding_url_facts(vec![bank(
                &base,
                &encoding,
                vec![HtmlEncodingUrlParseObservation::new(
                    "x".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(record(
                        &revision,
                        "https://example.test/x",
                        false,
                        None,
                    )),
                )],
            )])
            .parse()
            .unwrap();
        let states = rows(&report);
        assert_eq!(
            states[0].first_missing().unwrap().kind(),
            &HtmlParseMetaMissingKind::Csp
        );
        if expected {
            assert!(known_refresh(&states[0]));
        } else {
            let HtmlParseMetaRefresh::Unavailable(missing) = states[0].refresh() else {
                panic!("current policy context is unavailable")
            };
            assert_eq!(
                missing.kind(),
                &HtmlParseMetaMissingKind::RefreshUrl(HtmlParseHyperlinkUnavailable::PolicyHistory)
            );
        }
    }
}

#[test]
fn real_selectedcontent_clone_processes_meta_again_against_current_document_base() {
    // The original Meta observes failure before Base changes. A real general
    // clone is inserted under selectedcontent afterward and processes refresh
    // against the then-current base, rather than copying/resetting Doc history.
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let old = record(&revision, "https://example.test/old/", false, None);
    let new = record(&revision, "https://example.test/new/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlParseBaseFacts::new(
        environment(&old, &encoding).document().clone(),
        vec![
            HtmlBaseParseObservation::new(
                "new".into(),
                old.clone(),
                encoding.clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: new.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><meta http-equiv=refresh content='0;url=x'><base href=new></option></select>"), limits())
        .with_base_facts(facts).unwrap().with_encoding_url_facts(vec![
            bank(&old, &encoding, vec![HtmlEncodingUrlParseObservation::new("x".into(), HtmlEncodingUrlParseOutcome::Failure)]),
            bank(&new, &encoding, vec![HtmlEncodingUrlParseObservation::new("x".into(), HtmlEncodingUrlParseOutcome::Parsed(record(&revision, "https://example.test/new/x", false, None)))])
        ]).parse().unwrap();
    let states = rows(&report);
    assert!(known_refresh(&states[0]));
    assert!(states[0].first_missing().is_none());
    let mut cursor = report.metadata();
    let mut subjects = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.creation().is_some() => {
                if let surgeist_html::HtmlNodeKind::Element(element) = row.node().kind()
                    && element.name().local_name() == &DocumentString::from("meta")
                {
                    subjects.push(row.node().id());
                }
            }
            HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(subjects.len(), 2);
    assert_ne!(subjects[0], subjects[1]);
}
