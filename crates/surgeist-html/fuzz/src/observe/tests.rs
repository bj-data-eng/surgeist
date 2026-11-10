//! Oracle checks for the literal observation, using handwritten spec-derived sources.
use super::*;
use surgeist_html::construction::HtmlProcessingInstruction;

fn parsed_nodes(source: &str) -> Vec<NativeNode> {
    let limits = crate::codec::limits(crate::codec::CAPS).expect("documented envelope");
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.to_owned()), limits)
        .with_detail(surgeist_html_parser::DetailSet::ALL)
        .unwrap()
        .parse()
        .expect("literal parses inside the generous envelope");
    native(&report).expect("bounded native tree")
}

fn with_instruction_data(nodes: &[NativeNode], data: &str) -> Vec<NativeNode> {
    nodes
        .iter()
        .cloned()
        .map(|mut node| {
            if let NativeKind::ProcessingInstruction(instruction) = &node.kind {
                node.kind = NativeKind::ProcessingInstruction(HtmlProcessingInstruction::new(
                    instruction.target().clone(),
                    data,
                ));
            }
            node
        })
        .collect()
}

/// HTML 877d, 13.2.5.73-76: the target ends at the space, optional whitespace is
/// skipped, data accumulates until `>`, and a `?` directly before `>` is not data.
/// Both endings therefore yield the data `data`.
fn assert_literal_requires_instruction_data(id: u8, source: &str) {
    let nodes = parsed_nodes(source);
    assert!(check_literal(id, &nodes), "literal {id} on {source:?}");
    for wrong in ["", "dat", "data?", "datum", "target data"] {
        assert!(
            !check_literal(id, &with_instruction_data(&nodes, wrong)),
            "literal {id} accepted instruction data {wrong:?}"
        );
    }
}

#[test]
fn question_mark_terminated_instruction_literal_requires_its_tokenizer_data() {
    assert_literal_requires_instruction_data(5, "<?target data?><p>P</p>");
}

#[test]
fn greater_than_terminated_instruction_literal_requires_its_tokenizer_data() {
    assert_literal_requires_instruction_data(8, "<?target data><p>P</p>");
}

fn work_cut(
    cap: u64,
) -> (
    HtmlParseFailure,
    HtmlParseLimits,
    HtmlSource,
    HtmlParseCancellation,
) {
    let mut caps = crate::codec::CAPS;
    caps[19] = cap;
    let limits = crate::codec::limits(caps).expect("documented envelope");
    let source = HtmlSource::utf8("<!DOCTYPE html><p>a</p><p>b</p>".to_owned());
    let request = HtmlDocumentParseRequest::offline(source.clone(), limits)
        .with_detail(surgeist_html_parser::DetailSet::ALL)
        .unwrap();
    let cancel = request.cancellation().clone();
    let failure = request
        .parse()
        .expect_err("a work cap of 40 credits stops this parse");
    (failure, limits, source, cancel)
}

#[test]
fn balanced_recipes_accept_only_a_stop_at_the_configured_cap() {
    let descriptor = crate::UnicodeTarget::document().descriptor.clone();
    let props = Props(&descriptor);
    let (failure, limits, source, cancel) = work_cut(40);
    let text = UnicodeSource::Utf8("<!DOCTYPE html><p>a</p><p>b</p>".to_owned());
    let spec = |work_cap: u64| {
        let mut caps = crate::codec::CAPS;
        caps[19] = work_cap;
        Spec {
            fragment: false,
            text: &text,
            original: &source,
            limits: &limits,
            caps,
            requested: HtmlParserScriptingMode::Disabled,
            capture: None,
            shadow_allowed: false,
        }
    };
    // The stop reports the cap that was configured: a product outcome.
    let ok = verify_failure(&props, &spec(40), &failure, &cancel, Expect::Balanced).unwrap();
    assert_eq!(ok.kind(), OutcomeKind::ExpectedRejection, "{ok:?}");
    // The same stop against a configured cap it did not report is a spurious
    // stop, which a balanced recipe must never accept.
    let spurious = verify_failure(&props, &spec(41), &failure, &cancel, Expect::Balanced).unwrap();
    assert_eq!(spurious.kind(), OutcomeKind::Failure, "{spurious:?}");
    assert_eq!(
        spurious.failure_key().and_then(FailureKey::discriminator),
        Some("stop-limit-mismatch")
    );
    // The permissive classification still accepts it.
    let general = verify_failure(&props, &spec(41), &failure, &cancel, Expect::General).unwrap();
    assert_eq!(general.kind(), OutcomeKind::ExpectedRejection);
    // An oracle that expects success accepts no stop at all.
    let never = verify_failure(&props, &spec(40), &failure, &cancel, Expect::Never).unwrap();
    assert_eq!(never.kind(), OutcomeKind::Failure);
}

#[test]
fn balanced_recipes_reject_a_stop_on_a_cap_the_case_did_not_lower() {
    // A source one unit over the documented 100000-unit envelope really stops
    // the parse at the default cap. The stop is self-consistent (it reports the
    // configured cap and an attempted total above it), but no resource was
    // lowered, so an oracle that expects a tree must not accept it.
    let descriptor = crate::UnicodeTarget::document().descriptor.clone();
    let props = Props(&descriptor);
    let limits = crate::codec::limits(crate::codec::CAPS).expect("documented envelope");
    let text = UnicodeSource::Utf8("a".repeat(100_001));
    let source = HtmlSource::utf8("a".repeat(100_001));
    let request = HtmlDocumentParseRequest::offline(source.clone(), limits)
        .with_detail(surgeist_html_parser::DetailSet::ALL)
        .unwrap();
    let cancel = request.cancellation().clone();
    let failure = request
        .parse()
        .expect_err("the source exceeds the envelope");
    let spec = Spec {
        fragment: false,
        text: &text,
        original: &source,
        limits: &limits,
        caps: crate::codec::CAPS,
        requested: HtmlParserScriptingMode::Disabled,
        capture: None,
        shadow_allowed: false,
    };
    let balanced = verify_failure(&props, &spec, &failure, &cancel, Expect::Balanced).unwrap();
    assert_eq!(balanced.kind(), OutcomeKind::Failure, "{balanced:?}");
    assert_eq!(
        balanced.failure_key().and_then(FailureKey::discriminator),
        Some("stop-limit-mismatch")
    );
    let general = verify_failure(&props, &spec, &failure, &cancel, Expect::General).unwrap();
    assert_eq!(
        general.kind(),
        OutcomeKind::ExpectedRejection,
        "{general:?}"
    );
}

#[test]
fn retained_and_dropped_diagnostics_are_checked_against_the_configured_cap() {
    // HTML 877d, 13.2.5.6 tag open state: `< ` is one parse error, and a
    // document with no DOCTYPE adds the missing-doctype error of 13.2.6.4.1.
    // Ten pairs are therefore eleven errors; a cap of three retains the first
    // three and drops eight.
    let descriptor = crate::UnicodeTarget::document().descriptor.clone();
    let props = Props(&descriptor);
    let mut caps = crate::codec::CAPS;
    caps[DIAGNOSTICS_CAP] = 3;
    let limits = crate::codec::limits(caps).expect("documented envelope");
    let literal = "< ".repeat(10);
    let text = UnicodeSource::Utf8(literal.clone());
    let source = HtmlSource::utf8(literal);
    let report = HtmlDocumentParseRequest::offline(source.clone(), limits)
        .with_detail(surgeist_html_parser::DetailSet::ALL)
        .unwrap()
        .parse()
        .expect("a diagnostics cap bounds retention, not the parse");
    let spec = |diagnostics_cap: u64| {
        let mut caps = caps;
        caps[DIAGNOSTICS_CAP] = diagnostics_cap;
        Spec {
            fragment: false,
            text: &text,
            original: &source,
            limits: &limits,
            caps,
            requested: HtmlParserScriptingMode::Disabled,
            capture: None,
            shadow_allowed: false,
        }
    };
    let Ok(seen) = verify_report(&props, &spec(3), &report).unwrap() else {
        panic!("the retention obeys the configured cap");
    };
    assert_eq!(seen.diagnostics.records.len(), 3);
    assert_eq!(seen.diagnostics.dropped, 8);
    // Dropped diagnostics under a larger configured cap mean the cap was not
    // filled, which no correct parse produces.
    let Err(outcome) = verify_report(&props, &spec(100), &report).unwrap() else {
        panic!("a dropped diagnostic below the configured cap is a violation");
    };
    assert_eq!(
        outcome.failure_key().and_then(FailureKey::discriminator),
        Some("diagnostic-cap-retention")
    );
    // A retained count past the configured cap is a violation as well.
    let Err(outcome) = verify_report(&props, &spec(2), &report).unwrap() else {
        panic!("retaining more than the configured cap is a violation");
    };
    assert_eq!(
        outcome.failure_key().and_then(FailureKey::discriminator),
        Some("diagnostic-cap-retention")
    );
}
