#![forbid(unsafe_code)]
use surgeist_html::{DocumentString, HtmlNodeKind};
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
fn has_source(cause: &HtmlParseCause<'_>) -> bool {
    fn walk(cause: &HtmlParseCause<'_>, depth: usize) -> bool {
        assert!(depth < 100, "finite literal fixture provenance");
        cause.source().is_some()
            || cause.original_cause().is_some_and(|c| walk(&c, depth + 1))
            || cause.trigger().is_some_and(|c| walk(&c, depth + 1))
    }
    walk(cause, 0)
}
fn hyperlinks(report: &HtmlParseReport) -> Vec<(String, HtmlBaseUrlRecord, HtmlBaseUrlRecord)> {
    let mut cursor = report.metadata();
    let mut found = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        panic!("expected real observed URL");
                    };
                    let HtmlNodeKind::Element(element) = row.node().kind() else {
                        panic!("actual hyperlink");
                    };
                    let HtmlParseMetadataDocument::Owned(document) = link.document() else {
                        panic!("actual ordinary owner");
                    };
                    assert_eq!(document.id(), report.document().unwrap().root().id());
                    assert!(has_source(link.cause().unwrap()));
                    found.push((
                        element.name().local_name().to_utf8().unwrap(),
                        snapshot.base().clone(),
                        snapshot.url().unwrap().clone(),
                    ));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    found
}
#[test]
fn earlier_anchor_preserves_actual_cached_record_when_later_base_changes() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let old = record(&revision, "https://example.test/old/", false, None);
    let new = record(&revision, "https://example.test/new/", false, None);
    let old_x = record(&revision, "https://example.test/old/x", false, None);
    let new_x = record(&revision, "https://example.test/new/x", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let base_observation = HtmlBaseParseObservation::new(
        "https://example.test/new/".into(),
        old.clone(),
        encoding.clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Parsed {
            record: new.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    )
    .unwrap();
    let base_facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision,
            Some(old.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![base_observation],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><a href=x></a><base href='https://example.test/new/'><area href=x>",
        ),
        limits(),
    )
    .with_base_facts(base_facts)
    .unwrap()
    .with_encoding_url_facts(vec![
        bank(
            &old,
            &encoding,
            vec![HtmlEncodingUrlParseObservation::new(
                "x".into(),
                HtmlEncodingUrlParseOutcome::Parsed(old_x.clone()),
            )],
        ),
        bank(
            &new,
            &encoding,
            vec![HtmlEncodingUrlParseObservation::new(
                "x".into(),
                HtmlEncodingUrlParseOutcome::Parsed(new_x.clone()),
            )],
        ),
    ]);
    let cancel = request.cancellation().clone();
    let report = request.parse().unwrap();
    cancel.cancel();
    assert_eq!(
        hyperlinks(&report),
        vec![("a".into(), old, old_x), ("area".into(), new, new_x)]
    );
}
#[test]
fn absent_empty_failed_and_unavailable_href_are_distinct_completed_states() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = bank(
        &base,
        &encoding,
        vec![
            HtmlEncodingUrlParseObservation::new(
                "".into(),
                HtmlEncodingUrlParseOutcome::Parsed(record(
                    &revision,
                    "https://example.test/",
                    false,
                    None,
                )),
            ),
            HtmlEncodingUrlParseObservation::new(
                "http://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            ),
        ],
    );
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><a></a><a href=''></a><a href='http://['></a><area href=missing>",
        ),
        limits(),
    )
    .with_base_facts(environment(&base, &encoding))
    .unwrap()
    .with_encoding_url_facts(vec![facts])
    .parse()
    .unwrap();
    let mut cursor = report.metadata();
    let mut states = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    states.push(match link.state() {
                        HtmlParseHyperlinkState::AbsentHref => {
                            assert!(link.raw().is_none());
                            "absent"
                        }
                        HtmlParseHyperlinkState::Observed(snapshot) => {
                            if snapshot.url().is_some() {
                                assert_eq!(snapshot.raw(), &DocumentString::from(""));
                                "empty-parsed"
                            } else {
                                assert_eq!(snapshot.raw(), &DocumentString::from("http://["));
                                "failed"
                            }
                        }
                        HtmlParseHyperlinkState::Unavailable(_) => {
                            assert_eq!(link.raw(), Some(&DocumentString::from("missing")));
                            "unavailable"
                        }
                    });
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(states, ["absent", "empty-parsed", "failed", "unavailable"]);
}

#[test]
fn reached_meta_csp_cannot_reuse_pre_policy_hyperlink_facts() {
    // HTML877d11896–11903: only a direct-head, nonempty CSP pragma reaches
    // parse/filter/enforce. Its resulting policy revision is not established by
    // the independently supplied pre-policy URL context, even if URL spelling
    // would happen to remain equal. Guarded-out pragmas do not change it.
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let resolved = record(&revision, "https://example.test/x", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = bank(
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Parsed(resolved.clone()),
        )],
    );
    for (source, policy_unavailable) in [
        (
            "<!doctype html><head><meta http-equiv=content-security-policy content=''></head><body><a href=x></a><area href=x>",
            false,
        ),
        (
            "<!doctype html><head></head><body><meta http-equiv=content-security-policy content=\"base-uri 'none'\"><a href=x></a><area href=x>",
            false,
        ),
        (
            "<!doctype html><head><meta http-equiv=content-security-policy content=\"base-uri 'none'\"></head><body><a href=x></a><area href=x>",
            true,
        ),
    ] {
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .with_base_facts(environment(&base, &encoding))
            .unwrap()
            .with_encoding_url_facts(vec![facts.clone()])
            .parse()
            .expect("missing Meta execution facts preserve completed structural report");
        let mut cursor = report.metadata();
        let mut names = Vec::new();
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    let Some(link) = row.hyperlink() else {
                        continue;
                    };
                    let HtmlNodeKind::Element(element) = row.node().kind() else {
                        panic!("actual A/Area metadata subject");
                    };
                    names.push(element.name().local_name().to_utf8().unwrap());
                    assert_eq!(link.raw(), Some(&DocumentString::from("x")));
                    if policy_unavailable {
                        assert!(
                            matches!(link.state(), HtmlParseHyperlinkState::Unavailable(_)),
                            "reached CSP leaves current policy qualification unavailable: {source}"
                        );
                    } else {
                        let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                            panic!("guarded-out CSP preserves the independently qualified context");
                        };
                        assert_eq!(snapshot.base(), &base);
                        assert_eq!(snapshot.url(), Some(&resolved));
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(names, ["a", "area"]);
    }
}

#[test]
fn standalone_img_starts_before_disabled_guard_and_keeps_only_latest_inactive_wait() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><img><img src=x srcset=y><img src=x src=x>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut cursor = report.metadata();
    let mut generations = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(image) = row.image() {
                    let state = image.state();
                    assert_eq!(
                        state.current_request().state(),
                        surgeist_html::HtmlImgFreshRequestState::Unavailable
                    );
                    assert!(state.current_request().current_url().is_empty());
                    assert_eq!(state.current_request().current_pixel_density(), 1);
                    assert!(
                        state.pending_request_is_null() && state.last_selected_source_is_null()
                    );
                    assert_eq!(image.dimension_attribute_source().id(), row.node().id());
                    let HtmlParseImageUpdate::Waiting { generation, flags } = image.update() else {
                        panic!("actual no-browsing-context wait");
                    };
                    assert_eq!(generation, image.latest_started());
                    assert!(!flags.restart_animation() && !flags.maybe_omit_events());
                    assert!(has_source(&image.cause().unwrap()));
                    generations.push(generation);
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(generations, [1, 3, 2]);
}
#[test]
fn template_url_facts_bind_only_actual_independent_inert_document() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let encoding = HtmlBaseEncoding::fresh();
    let blank = record(&revision, "about:blank", true, None);
    let not_blank = record(&revision, "https://outer.test/", false, None);
    assert!(matches!(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits())
            .with_template_base_facts(environment(&not_blank, &encoding)),
        Err(HtmlParseRequestError::Base(
            HtmlBaseFactsError::ContradictoryObservation
        ))
    ));
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><template><a href=x></a></template>"),
        limits(),
    )
    .with_template_base_facts(environment(&blank, &encoding))
    .unwrap()
    .with_encoding_url_facts(vec![bank(
        &blank,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
    )])
    .parse()
    .unwrap();
    let root = report.document().unwrap().root().id();
    let mut cursor = report.metadata();
    let mut found = false;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let HtmlParseMetadataDocument::Owned(owner) = link.document() else {
                        panic!("actual inert Document");
                    };
                    assert_ne!(owner.id(), root);
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        panic!("actual attempted parse");
                    };
                    assert_eq!(snapshot.base(), &blank);
                    assert!(snapshot.url().is_none());
                    found = true;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert!(found);
}
#[test]
fn unused_execution_banks_are_released_before_successful_report_usage() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let mut banks = Vec::with_capacity(32);
    banks.push(bank(
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "unused".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
    ));
    let report =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><p>ok</p>"), limits())
            .with_encoding_url_facts(banks)
            .parse()
            .unwrap();
    let usage = report
        .usage()
        .resource(HtmlParseResource::ManagedCapacityBytes);
    assert!(usage.high_water() > usage.current());
}

#[test]
fn spare_execution_capacity_does_not_overlap_successful_retained_receipt_and_failure_keeps_original_bank()
 {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let fact = bank(
        &base,
        &encoding,
        vec![HtmlEncodingUrlParseObservation::new(
            "unused".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
    );
    let parse = |capacity| {
        let mut inputs = Vec::with_capacity(capacity);
        inputs.push(fact.clone());
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><p>ok</p>"), limits())
            .with_encoding_url_facts(inputs)
            .parse()
            .unwrap()
    };
    let sparse = parse(1);
    let spare = parse(128);
    assert_eq!(
        sparse.retained_construction_bytes(),
        spare.retained_construction_bytes(),
        "discarded input capacity cannot inflate the historical graph"
    );
    assert_eq!(
        spare.retained_construction_bytes(),
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current()
    );
    assert!(
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water()
            > sparse
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water()
    );
    let mut inputs = Vec::with_capacity(128);
    inputs.push(fact.clone());
    let denied = limits().into_builder().source_units(0).finish().unwrap();
    let failure = HtmlDocumentParseRequest::offline(HtmlSource::utf8("x"), denied)
        .with_encoding_url_facts(inputs)
        .parse()
        .unwrap_err();
    assert_eq!(failure.encoding_url_fact_capacity(), Some(128));
    assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&fact));
}
#[test]
fn source_insertion_updates_actual_picture_images_in_source_order() {
    for (source, expected) in [
        (
            "<!doctype html><picture><img src=x><source srcset=y></picture>",
            4,
        ),
        (
            "<!doctype html><picture><source srcset=y><img src=x></picture>",
            3,
        ),
    ] {
        let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits())
            .parse()
            .unwrap();
        let mut cursor = report.metadata();
        let mut found = false;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if let Some(image) = row.image() {
                        assert_eq!(image.latest_started(), expected);
                        assert!(
                            matches!(image.update(), HtmlParseImageUpdate::Waiting { generation, .. } if generation == expected)
                        );
                        found = true;
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(found);
    }
}

#[test]
fn ordinary_common_state_is_subject_qualified_and_nonce_is_independent() {
    // HTML877d common slots: offline parsing has no activation, reactions,
    // attachInternals or popover interaction. Authored attributes do not mutate
    // these slots. Namespace applicability is independent of nonce storage.
    let parse = || {
        HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><x-card nonce=html popover><svg nonce=svg><g nonce=g></g></svg><math nonce=math><mi nonce=mi></mi></math></x-card>"),
        limits(),
    ).parse().unwrap()
    };
    let first = parse();
    let second = parse();
    let collect = |report: &HtmlParseReport| {
        let mut cursor = report.metadata();
        let mut subjects = Vec::new();
        let mut nonces = Vec::new();
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    let HtmlNodeKind::Element(element) = row.node().kind() else {
                        continue;
                    };
                    let local = element.name().local_name().to_utf8().unwrap();
                    if let Some(common) = row.common_state() {
                        let element = common.element();
                        assert!(element.reaction_queue_is_empty());
                        assert!(!element.click_in_progress());
                        if ["x-card", "svg", "g", "math", "mi"].contains(&local.as_str()) {
                            let html = local == "x-card";
                            assert_eq!(
                                common.scope(),
                                if html {
                                    surgeist_html::HtmlCommonScope::Html
                                } else {
                                    surgeist_html::HtmlCommonScope::Element
                                }
                            );
                            assert_eq!(common.html().is_some(), html);
                            if let Some(state) = common.html() {
                                assert!(!state.has_previously_focused_element());
                                assert!(!state.has_attached_internals());
                                assert!(state.popover_is_hidden());
                                assert!(!state.has_popover_trigger());
                                assert!(!state.popover_is_hiding());
                                assert!(!state.has_popover_toggle_task_tracker());
                                assert!(!state.has_popover_close_watcher());
                                assert!(!state.has_popover_opened_in_mode());
                            }
                            subjects.push((local.clone(), row.node().id()));
                        }
                    }
                    if let Some(nonce) = row.cryptographic_nonce()
                        && ["x-card", "svg", "g", "math", "mi"].contains(&local.as_str())
                    {
                        nonces.push((local, nonce.to_utf8().unwrap()));
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        (subjects, nonces)
    };
    let (a, nonces) = collect(&first);
    let (b, other_nonces) = collect(&second);
    assert_eq!(a.len(), 5);
    assert_eq!(
        nonces,
        [
            ("x-card".into(), "html".into()),
            ("svg".into(), "svg".into()),
            ("g".into(), "g".into()),
            ("math".into(), "math".into()),
            ("mi".into(), "mi".into())
        ]
    );
    assert_eq!(nonces, other_nonces);
    for ((name, actual), (other_name, other)) in a.iter().zip(b.iter()) {
        assert_eq!(name, other_name);
        assert_ne!(
            actual, other,
            "value equality cannot qualify a foreign report subject"
        );
    }
}

#[test]
fn associated_resource_boundaries_and_cancellation_preserve_original_inputs_and_cleanup() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let facts = bank(
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
    );
    let request = |limits| {
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><a href=x></a><img src=x crossorigin=anonymous referrerpolicy=origin>"), limits)
        .with_base_facts(environment(&base, &encoding)).unwrap()
        .with_encoding_url_facts(vec![facts.clone()])
    };
    let report = request(limits()).parse().unwrap();
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::AuxiliaryEntries,
        HtmlParseResource::WorkCredits,
    ] {
        let peak = report.usage().resource(resource).high_water();
        let limited = |cap| {
            let builder = limits().into_builder();
            match resource {
                HtmlParseResource::ManagedCapacityBytes => builder.managed_capacity_bytes(cap),
                HtmlParseResource::ManagedUtf16Units => builder.managed_utf16_units(cap),
                HtmlParseResource::AuxiliaryEntries => builder.auxiliary_entries(cap),
                HtmlParseResource::WorkCredits => builder.work_credits(cap),
                _ => unreachable!(),
            }
            .finish()
            .unwrap()
        };
        request(limited(peak)).parse().unwrap();
        let failure = request(limited(peak - 1)).parse().unwrap_err();
        assert!(
            matches!(failure.first_stop(), Some(HtmlParseStop::LimitExceeded { resource: actual, .. }) if actual == resource)
        );
        assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&facts));
        assert_eq!(failure.cleanup_error(), None);
        for resource in [
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::AuxiliaryEntries,
        ] {
            assert_eq!(
                failure
                    .after_cleanup()
                    .unwrap()
                    .resource(resource)
                    .current(),
                0,
                "{resource:?}"
            );
        }
    }
    let request = request(limits());
    request.cancellation().cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(failure.first_stop(), Some(HtmlParseStop::Cancelled));
    assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&facts));
    assert_eq!(failure.cleanup_error(), None);
}

#[test]
fn parsed_style_retains_actual_parser_document_and_pop_while_general_clone_is_fresh() {
    // HTML877d12131/12142: actual parser creation and actual pop are independent
    // of current node-document; DOM cloning does not copy parser ownership.
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(
        "<!doctype html><style>a{}</style><template><style>b{}</style></template><select><button><selectedcontent></selectedcontent></button><option selected><style>c{}</style></option></select>"
    ), limits()).parse().unwrap();
    let root = report.document().unwrap().root().id();
    let mut cursor = report.metadata();
    let mut rows = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(style) = row.style() {
                    assert!(!style.state().on_parser_stack());
                    assert_eq!(style.association(), HtmlParseStyleAssociation::Pending);
                    assert!(has_source(style.cause().unwrap()));
                    let node = row.node();
                    let parent = node.parent().unwrap();
                    let parent_node = report.document().unwrap().node(&parent).unwrap();
                    let parent_name = match parent_node.kind() {
                        HtmlNodeKind::Element(element) => {
                            element.name().local_name().to_utf8().unwrap()
                        }
                        HtmlNodeKind::Fragment => "template-contents".into(),
                        _ => panic!("literal Style parent"),
                    };
                    if parent_name == "selectedcontent" {
                        assert!(!style.state().created_by_document_parser());
                        assert!(style.creating_parser_document().is_none());
                        assert_eq!(
                            style.cause().unwrap().operation(),
                            HtmlParseConstructionOperation::Copy
                        );
                    } else {
                        assert!(style.state().created_by_document_parser());
                        let Some(HtmlParseMetadataDocument::Owned(document)) =
                            style.creating_parser_document()
                        else {
                            panic!("actual parser Document");
                        };
                        assert_eq!(document.id(), root);
                    }
                    rows.push(parent_name);
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    rows.sort();
    assert_eq!(
        rows,
        ["head", "option", "selectedcontent", "template-contents"]
    );
}

#[test]
fn inline_snapshot_header_is_counted_once_and_unused_observation_capacity_is_released() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let make = |raw: &str, capacity| {
        let mut observations = Vec::with_capacity(capacity);
        observations.push(HtmlEncodingUrlParseObservation::new(
            raw.into(),
            HtmlEncodingUrlParseOutcome::Failure,
        ));
        let fact = HtmlEncodingUrlParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            observations,
            HtmlBaseFactsLimits::new(128, 10_000, 100_000),
        )
        .unwrap();
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><a href=x>"), limits())
            .with_base_facts(environment(&base, &encoding))
            .unwrap()
            .with_encoding_url_facts(vec![fact])
            .parse()
            .unwrap()
    };
    let unavailable = make("unused", 1);
    let selected = make("x", 1);
    let spare = make("x", 128);
    let mut cursor = selected.metadata();
    let mut actual_snapshot = None;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        panic!("actual attempted URL parse");
                    };
                    let footprint = snapshot.retained_storage_usage().retained_bytes() as u64;
                    actual_snapshot = Some(footprint);
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    // Both reports own identical retained payload Vec layouts and one retired
    // execution-bank slot. Only the observed state's external graph differs;
    // the public snapshot's inline header already occupies that SAME Vec slot.
    let footprint = actual_snapshot.unwrap();
    assert_eq!(
        selected.retained_construction_bytes() - unavailable.retained_construction_bytes(),
        footprint - std::mem::size_of::<HtmlHyperlinkUrlSnapshot>() as u64
    );
    assert_eq!(
        selected.retained_construction_bytes(),
        spare.retained_construction_bytes()
    );
    assert!(
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water()
            > selected
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water()
    );
}

#[test]
fn unavailable_contexts_cannot_self_validate_candidate_revision_or_equal_serialization() {
    let producer = HtmlBaseProducerId::fresh();
    let revision = HtmlBaseFactsRevision::new(producer.clone(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let observation = || {
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )]
    };
    let facts = |base, encoding, revision| {
        HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            observation(),
            HtmlBaseFactsLimits::new(20, 10_000, 100_000),
        )
        .unwrap()
    };
    for (candidate, expected) in [
        (
            facts(
                base.clone(),
                encoding.clone(),
                HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
            ),
            HtmlEncodingUrlFactsError::ForeignProducer,
        ),
        (
            facts(
                base.clone(),
                encoding.clone(),
                HtmlBaseFactsRevision::new(producer, 2, 2, 3),
            ),
            HtmlEncodingUrlFactsError::RevisionMismatch,
        ),
        (
            facts(
                record(&revision, "https://example.test/", false, None),
                encoding.clone(),
                revision.clone(),
            ),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            facts(base.clone(), HtmlBaseEncoding::fresh(), revision.clone()),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
    ] {
        let report = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><a href=x>"),
            limits(),
        )
        .with_base_facts(environment(&base, &encoding))
        .unwrap()
        .with_encoding_url_facts(vec![candidate])
        .parse()
        .unwrap();
        let mut cursor = report.metadata();
        let mut found = false;
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if let Some(link) = row.hyperlink() {
                        let HtmlParseHyperlinkState::Unavailable(
                            HtmlParseHyperlinkUnavailable::Observation(actual),
                        ) = link.state()
                        else {
                            panic!("actual context must remain unavailable");
                        };
                        assert_eq!(actual, &expected);
                        found = true;
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(found);
    }
}

#[test]
fn reached_general_clones_use_current_base_and_fresh_image_update_identity() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let old = record(&revision, "https://example.test/old/", false, None);
    let new = record(&revision, "https://example.test/new/", false, None);
    let old_x = record(&revision, "https://example.test/old/x", false, None);
    let new_x = record(&revision, "https://example.test/new/x", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let environment = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(old.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "https://example.test/new/".into(),
                old.clone(),
                encoding.clone(),
                revision,
                HtmlBaseParseOutcome::Parsed {
                    record: new.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(20, 10000, 100000),
    )
    .unwrap();
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><a href=x><img src=x></a><base href='https://example.test/new/'></option></select>"), limits())
        .with_base_facts(environment).unwrap().with_encoding_url_facts(vec![bank(&old,&encoding,vec![HtmlEncodingUrlParseObservation::new("x".into(),HtmlEncodingUrlParseOutcome::Parsed(old_x.clone()))]), bank(&new,&encoding,vec![HtmlEncodingUrlParseObservation::new("x".into(),HtmlEncodingUrlParseOutcome::Parsed(new_x.clone()))])]).parse().unwrap();
    let mut cursor = report.metadata();
    let mut links = Vec::new();
    let mut images = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        if let HtmlParseHyperlinkState::Unavailable(reason) = link.state() {
                            panic!(
                                "actual current-context observation unavailable: {reason:?}, cause {:?}",
                                link.cause().unwrap().operation()
                            );
                        }
                        panic!("actual current-context observation absent");
                    };
                    links.push((
                        row.node().id(),
                        snapshot.url().unwrap().clone(),
                        link.cause().unwrap().operation(),
                    ));
                }
                if let Some(image) = row.image() {
                    assert_eq!(image.latest_started(), 2);
                    assert_eq!(image.dimension_attribute_source().id(), row.node().id());
                    let HtmlParseImageUpdate::Waiting {
                        generation: 2,
                        flags,
                    } = image.update()
                    else {
                        panic!("fresh current wait");
                    };
                    assert!(!flags.restart_animation() && !flags.maybe_omit_events());
                    images.push(row.node().id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(links.len(), 2);
    assert_ne!(links[0].0, links[1].0);
    assert_eq!(links[0].1, old_x);
    assert_eq!(links[1].1, new_x);
    assert_eq!(links[1].2, HtmlParseConstructionOperation::Copy);
    assert_eq!(images.len(), 2);
    assert_ne!(images[0], images[1]);
}

#[test]
fn missing_inert_context_and_duplicate_execution_contexts_remain_explicit() {
    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><template><a href=x></a></template>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut cursor = report.metadata();
    let mut found = false;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    assert!(matches!(
                        link.state(),
                        HtmlParseHyperlinkState::Unavailable(HtmlParseHyperlinkUnavailable::Base(
                            HtmlBaseMissing::DocumentFacts
                        ))
                    ));
                    found = true;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert!(found);
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let fact = bank(&base, &encoding, vec![]);
    let failure =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<!doctype html><a href=x>"), limits())
            .with_encoding_url_facts(vec![fact.clone(), fact.clone()])
            .parse()
            .unwrap_err();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::InvalidAssociatedFacts)
    );
    let original = failure.encoding_url_facts().unwrap();
    assert_eq!(original.len(), 2);
    assert!(original.iter().all(|f| f.same_owner(&fact)));
    assert_eq!(failure.cleanup_error(), None);
}
