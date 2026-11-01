use crate::parser::support::{fragment_context, limits};
use surgeist_html::*;
use surgeist_html_parser::{
    HtmlColorProcessingError, HtmlDocumentParseRequest, HtmlFragmentParseRequest,
    HtmlParseAssociationRelation, HtmlParseConstructionOperation, HtmlParseDestination,
    HtmlParseDiagnosticContext, HtmlParseDiagnosticKind, HtmlParseDiagnosticLocationRole,
    HtmlParseDiagnosticSeverity, HtmlParseEffect, HtmlParseEffectHost, HtmlParseFailureReason,
    HtmlParseField, HtmlParseHistoryError, HtmlParseHistoryFrame, HtmlParseHistoryStep,
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, HtmlParseLimits,
    HtmlParseMetadataStep, HtmlParseOriginKind, HtmlParseRawField, HtmlParseResource,
    HtmlParseRetiredDestination, HtmlParseReverseError, HtmlParseReverseStep, HtmlParseStop,
    HtmlParseTokenFamily, HtmlParseUnsupportedState, HtmlPrologueDiagnostic, HtmlPrologueMode,
    HtmlSource, HtmlSourceCoordinates, HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticPhase,
    HtmlTokenizerState,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureFact, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCapturedIsValue, HtmlFragmentContext, HtmlScriptingStatus,
};

#[test]
fn recovered_references_public_reader_preserves_typed_chronology_and_exact_dual_coordinates() {
    // HTML877d numeric-end and named-reference rules: missing ';' precedes
    // NULL replacement, C1 0x80 maps to EURO, and NotEqualTilde emits two
    // code points with the same authored interval. é separates UTF-8 bytes
    // from UTF-16 coordinates. These literals are not another parse's output.
    let raw = "<!doctype html><p>é&#0x&#x80;&NotEqualTilde;</p>";
    for utf16 in [false, true] {
        let at = |units: usize| {
            if utf16 || units <= 18 {
                units
            } else {
                units + 1
            }
        };
        let source = if utf16 {
            HtmlSource::utf16(DocumentString::from(raw))
        } else {
            HtmlSource::utf8(raw)
        };
        let request = HtmlDocumentParseRequest::offline(source, limits());
        let identity = request.source().identity().clone();
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancellation.cancel();
        assert_eq!(report.source().identity(), &identity);
        assert_eq!(
            report.source().coordinates(),
            if utf16 {
                HtmlSourceCoordinates::Utf16CodeUnits
            } else {
                HtmlSourceCoordinates::Utf8Bytes
            }
        );
        if utf16 {
            assert_eq!(
                report.source().as_utf16().unwrap().code_units(),
                raw.encode_utf16().collect::<Vec<_>>()
            );
        } else {
            assert_eq!(report.source().as_utf8(), Some(raw));
        }
        let document = report.document().unwrap();
        assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
        let roots: Vec<_> = document.root().children().collect();
        assert_eq!(roots.len(), 2);
        assert!(matches!(
            document.node(&roots[0]).unwrap().kind(),
            HtmlNodeKind::Doctype(_)
        ));
        let html = document.node(&roots[1]).unwrap();
        let branches: Vec<_> = html.children().collect();
        assert_eq!(branches.len(), 2);
        let head = document.node(&branches[0]).unwrap();
        assert_eq!(head.children().count(), 0);
        let body = document.node(&branches[1]).unwrap();
        assert_eq!(body.children().count(), 1);
        let p = document.node(&body.children().next().unwrap()).unwrap();
        let HtmlNodeKind::Element(element) = p.kind() else {
            panic!("literal p")
        };
        assert_eq!(element.name().local_name(), &DocumentString::from("p"));
        assert!(element.attributes().is_empty());
        assert_eq!(p.children().count(), 1);
        let text_id = p.children().next().unwrap();
        let HtmlNodeKind::Text(text) = document.node(&text_id).unwrap().kind() else {
            panic!("literal text")
        };
        assert_eq!(
            text.code_units(),
            &[0xE9, 0xFFFD, 0x78, 0x20AC, 0x2242, 0x338]
        );
        let expected_errors = [
            (
                HtmlTokenizerDiagnostic::MissingSemicolonAfterCharacterReference,
                "parse-error-missing-semicolon-after-character-reference",
                "missing-semicolon-after-character-reference",
                19,
                22,
            ),
            (
                HtmlTokenizerDiagnostic::NullCharacterReference,
                "parse-error-null-character-reference",
                "null-character-reference",
                19,
                22,
            ),
            (
                HtmlTokenizerDiagnostic::ControlCharacterReference,
                "parse-error-control-character-reference",
                "control-character-reference",
                23,
                29,
            ),
        ];
        let diagnostics = report.diagnostics();
        assert_eq!(diagnostics.len(), expected_errors.len());
        for (i, (code, anchor, branch, start, end)) in expected_errors.into_iter().enumerate() {
            let (record, location) = diagnostics.retained_record(i).unwrap().unwrap();
            assert_eq!(record.kind(), HtmlParseDiagnosticKind::Tokenizer(code));
            assert_eq!(
                record.context(),
                HtmlParseDiagnosticContext::Tokenizer {
                    phase: HtmlTokenizerDiagnosticPhase::Tokenizer,
                    state: HtmlTokenizerState::Data,
                }
            );
            assert_eq!(record.severity(), HtmlParseDiagnosticSeverity::Recoverable);
            assert_eq!(record.rule().anchor(), anchor);
            assert_eq!(record.rule().branch(), branch);
            assert_eq!(
                location.role(),
                HtmlParseDiagnosticLocationRole::InputFrontier
            );
            assert_eq!(location.token(), None);
            assert_eq!(
                location.source_range(),
                &report.source().range(at(start), at(end)).unwrap()
            );
            assert_eq!(location.source_range().source(), &identity);
        }
        use HtmlParseOriginKind::{CharacterReference as R, Literal as L};
        let expected = [
            (0, 1, 18, 19, L),
            (1, 2, 19, 22, R),
            (2, 3, 22, 23, L),
            (3, 4, 23, 29, R),
            (4, 5, 29, 44, R),
            (5, 6, 29, 44, R),
        ]
        .map(|(a, b, c, d, k)| (a, b, at(c), at(d), k));
        let associations = report.associations();
        let mut actual = Vec::new();
        for i in 0..associations.len() {
            let row = associations.row(i).unwrap().unwrap();
            if row.relation() == HtmlParseAssociationRelation::Output
                && let HtmlParseDestination::Field {
                    node,
                    field: HtmlParseField::Text,
                } = row.destination()
            {
                assert_eq!(node.id(), text_id);
                let source = row.source().unwrap();
                assert_eq!(source.source(), &identity);
                actual.push((
                    row.start(),
                    row.end(),
                    source.start(),
                    source.end(),
                    row.origin_kind().unwrap(),
                ));
                assert!(row.group().is_some());
            }
        }
        actual.sort_by_key(|r| (r.0, r.2));
        assert_eq!(actual, expected);
        // The two distinct output units overlap one input interval. Reverse
        // traversal may also report lexical/cause/history rows; qualify this
        // literal comparison specifically to the current text Output relation.
        let range = report.source().range(at(29), at(44)).unwrap();
        let mut cursor = associations.reverse(&range, None).unwrap();
        let mut reverse = Vec::new();
        let mut complete = false;
        for _ in 0..=associations.len() {
            match cursor.advance() {
                HtmlParseReverseStep::Match(row) => {
                    if row.relation() == HtmlParseAssociationRelation::Output
                        && let HtmlParseDestination::Field {
                            node,
                            field: HtmlParseField::Text,
                        } = row.destination()
                    {
                        assert_eq!(node.id(), text_id);
                        let source = row.source().unwrap();
                        reverse.push((
                            row.start(),
                            row.end(),
                            source.start(),
                            source.end(),
                            row.origin_kind().unwrap(),
                        ));
                    }
                }
                HtmlParseReverseStep::Skipped => {}
                HtmlParseReverseStep::Complete => {
                    complete = true;
                    break;
                }
                HtmlParseReverseStep::Cancelled => {
                    panic!("original cancellation cannot revoke immutable read")
                }
                HtmlParseReverseStep::Fault(fault) => panic!("{fault}"),
            }
        }
        assert!(complete, "one flat row per action must reach completion");
        reverse.sort_by_key(|r| r.0);
        assert_eq!(reverse, expected[4..]);
        assert_eq!(report.usage(), &usage);
    }
}

#[test]
fn document_parse_owns_exact_source_native_and_complete_d1_after_cancellation() {
    let literal = "<!doctype html><p id=x>A&amp;B\r\nC</p>";
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
    let identity = request.source().identity().clone();
    let cancellation = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancellation.cancel();
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(report.source().as_utf8(), Some(literal));
    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
    let html = document
        .root()
        .children()
        .find(|n| matches!(document.node(n).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let html = document.node(&html).unwrap();
    let children: Vec<_> = html.children().collect();
    let body = document.node(&children[1]).unwrap();
    let p = body.children().next().unwrap();
    let p = document.node(&p).unwrap();
    let HtmlNodeKind::Element(element) = p.kind() else {
        panic!("p")
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("p"));
    assert_eq!(
        element.attributes().as_slice()[0].value(),
        &DocumentString::from("x")
    );
    let text = p.children().next().unwrap();
    assert!(
        matches!(document.node(&text).unwrap().kind(),HtmlNodeKind::Text(t) if t==&DocumentString::from("A&B\nC"))
    );
    let associations = report.associations();
    assert!(!associations.is_empty());
    let mut character_reference = false;
    for index in 0..associations.len() {
        let row = associations.row(index).unwrap().unwrap();
        if row.origin_kind() == Some(HtmlParseOriginKind::CharacterReference) {
            character_reference = true;
            assert_eq!(
                report.source().text(row.source().unwrap()).unwrap(),
                DocumentString::from("&amp;")
            );
        }
    }
    assert!(character_reference);
    assert_eq!(report.usage(), &usage);
}

#[test]
fn public_resource_and_cancel_failures_retain_original_source_without_partial_output() {
    let source = HtmlSource::utf8("<p>x");
    let identity = source.identity().clone();
    let failure = HtmlDocumentParseRequest::offline(
        source,
        limits()
            .into_builder()
            .source_units(0)
            .work_credits(0)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap_err();
    assert_eq!(failure.source().identity(), &identity);
    assert!(matches!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    assert_eq!(failure.cleanup_error(), None);
    let source_denial = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("x"),
        limits().into_builder().source_units(0).finish().unwrap(),
    )
    .parse()
    .unwrap_err();
    assert_eq!(source_denial.source_admitted(), Some(false));
    assert!(matches!(
        source_denial.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::SourceUnits,
            ..
        })
    ));
    assert!(source_denial.diagnostics().is_none());
    assert_eq!(source_denial.cleanup_error(), None);
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(vec![0xd800])),
        limits(),
    );
    request.cancellation().cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled)
    );
    assert_eq!(
        failure.source().as_utf16().map(|s| s.code_units()),
        Some(&[0xd800][..])
    );
    assert_eq!(failure.cleanup_error(), None);
}

/// Observable stage of an ordinary document parse that a work stop reached.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ReachedStage {
    /// The original request is retained but no execution owner exists yet.
    Initialization,
    /// Execution owns diagnostic evidence and the tree is still being built.
    Construction,
    /// Every node is allocated and closed; only publication work remains.
    Transfer,
}

#[test]
fn selected_ordinary_work_cuts_retain_first_stop_and_cleanup_evidence() {
    let literal =
        "<!doctype html><form id=f><input type=hidden value=x><textarea>A&amp;B</textarea></form>";
    let complete = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let total = complete
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    // HTML 877d tree construction yields Document, doctype, html, head, body,
    // form, input, textarea and the textarea's one text node. The allocation
    // count is the contract that identifies a finished tree at a work stop.
    let tree_nodes = complete
        .usage()
        .resource(HtmlParseResource::ManagedNodes)
        .current();
    assert_eq!(tree_nodes, 9);

    let reached = |failure: &surgeist_html_parser::HtmlParseFailure| {
        let at_stop = failure
            .at_stop()
            .expect("every work stop keeps its snapshot");
        if failure.diagnostics().is_none() {
            ReachedStage::Initialization
        } else if at_stop.resource(HtmlParseResource::ManagedNodes).current() == tree_nodes
            && at_stop.resource(HtmlParseResource::OpenElements).current() == 0
        {
            ReachedStage::Transfer
        } else {
            ReachedStage::Construction
        }
    };
    let cut = |work: u64| {
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(literal),
            limits().into_builder().work_credits(work).finish().unwrap(),
        )
        .parse()
        .unwrap_err()
    };
    // Locate each stage boundary at run time: the first budget whose stop
    // reaches the stage. Stages only advance as the budget grows.
    let first_reaching = |stage: ReachedStage| {
        assert!(reached(&cut(0)) < stage, "{stage:?} needs work");
        assert!(reached(&cut(total - 1)) >= stage, "{stage:?} is reachable");
        let (mut short, mut enough) = (0, total - 1);
        while short + 1 < enough {
            let middle = short + (enough - short) / 2;
            if reached(&cut(middle)) >= stage {
                enough = middle;
            } else {
                short = middle;
            }
        }
        enough
    };
    let construction = first_reaching(ReachedStage::Construction);
    let transfer = first_reaching(ReachedStage::Transfer);
    assert!(0 < construction && construction < transfer && transfer < total);

    // The short initialization prefix, both stage boundaries, 32 evenly
    // spaced cuts across initialization's remainder, construction and
    // publication, and the final writes, without every intermediate prefix.
    let spaced = |from: u64, to: u64| (0..32).map(move |i| from + (to - from) * i / 32);
    let cuts = (0..2048)
        .chain(construction.saturating_sub(2)..construction + 2)
        .chain(transfer.saturating_sub(2)..transfer + 2)
        .chain(spaced(2048, construction))
        .chain(spaced(construction, transfer))
        .chain(spaced(transfer, total))
        .chain(total - 32..total);
    for work in cuts {
        let failure = cut(work);
        let expected = if work < construction {
            ReachedStage::Initialization
        } else if work < transfer {
            ReachedStage::Construction
        } else {
            ReachedStage::Transfer
        };
        assert_eq!(reached(&failure), expected, "work {work}");
        let first = failure
            .first_stop()
            .expect("a work stop is an execution stop");
        assert!(
            matches!(
                first,
                HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit,
                    ..
                } if limit == work
            ),
            "work {work}: {first:?}"
        );
        assert_eq!(failure.reason(), HtmlParseFailureReason::Stop(first));
        assert!(
            failure
                .at_stop()
                .unwrap()
                .resource(HtmlParseResource::WorkCredits)
                .current()
                <= work,
            "work {work}: the budget is never overspent"
        );
        assert_eq!(failure.cleanup_error(), None, "work {work}");
        // Cleanup is the contract on every cut, so its evidence must exist.
        let after = failure
            .after_cleanup()
            .unwrap_or_else(|| panic!("work {work}: cleanup evidence is missing"));
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::SourceMapPayloadUnits,
            HtmlParseResource::Diagnostics,
            HtmlParseResource::DiagnosticPayloadUnits,
            HtmlParseResource::ContextEffects,
            HtmlParseResource::OpenElements,
            HtmlParseResource::ManagedCapacityBytes,
        ] {
            assert_eq!(
                after.resource(resource).current(),
                0,
                "work {work}: {resource:?}"
            );
        }
    }
    // The budget the complete parse consumed is sufficient.
    HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits()
            .into_builder()
            .work_credits(total)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap();
}

#[test]
fn public_failure_keeps_first_stop_snapshot_separate_from_cleanup_and_cancel() {
    // Document, doctype, html, head, body, p and Text A are seven actual nodes.
    // The following div must refuse before allocation, after text is retained.
    let literal = "<!doctype html><p>A</p><div>";
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits().into_builder().managed_nodes(7).finish().unwrap(),
    );
    let cancel = request.cancellation().clone();
    let failure = request.parse().unwrap_err();
    let first = failure.first_stop().unwrap();
    assert_eq!(
        first,
        HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedNodes,
            limit: 7,
            current: 7,
            requested_delta: 1,
        }
    );
    assert_eq!(failure.reason(), HtmlParseFailureReason::Stop(first));
    assert_eq!(failure.source_admitted(), Some(true));
    let at_stop = failure.at_stop().unwrap().clone();
    let after = failure.after_cleanup().unwrap().clone();
    assert_eq!(
        at_stop.resource(HtmlParseResource::ManagedNodes).current(),
        7
    );
    assert!(
        at_stop
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current()
            > 0
    );
    assert_eq!(
        after
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        0
    );
    assert_eq!(
        at_stop.resource(HtmlParseResource::WorkCredits),
        after.resource(HtmlParseResource::WorkCredits)
    );
    cancel.cancel();
    assert_eq!(failure.first_stop(), Some(first));
    assert_eq!(failure.at_stop(), Some(&at_stop));
    assert_eq!(failure.after_cleanup(), Some(&after));
    assert_eq!(failure.cleanup_error(), None);
}

#[test]
fn canonical_control_metadata_lends_actual_nodes_flags_raw_values_and_form_identity() {
    let literal = "<!doctype html><form id=f><input type=hidden value=x><textarea>A&amp;B</textarea><select><option selected>Q</option></select></form>";
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let document = report.document().unwrap();
    let mut form = None;
    let mut stack = vec![document.root().id()];
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("form"))
        {
            form = Some(id.clone());
        }
        stack.extend(node.children());
    }
    let form = form.unwrap();
    let mut metadata = report.metadata();
    let (mut input, mut textarea, mut select, mut option) = (false, false, false, false);
    let mut associated = Vec::new();
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(owner) = record.form_owner() {
                    assert_eq!(owner.unwrap().id(), form);
                    associated.push(record.node().id());
                }
                if let Some(flags) = record.input_flags() {
                    input = true;
                    assert_eq!(flags.input_type(), HtmlInputType::Hidden);
                    assert!(!flags.dirty_value());
                    assert_eq!(record.input_raw_value(), Some(&DocumentString::from("x")));
                    assert!(record.form_owner().is_none());
                }
                if let Some(flags) = record.textarea_flags() {
                    textarea = true;
                    assert!(!flags.dirty_value());
                    assert_eq!(
                        record.textarea_raw_value(),
                        Some(&DocumentString::from("A&B"))
                    );
                    assert!(record.form_owner().is_none());
                }
                if record.select_flags().is_some() {
                    select = true;
                    assert!(record.form_owner().is_none());
                }
                if let Some(flags) = record.option_flags() {
                    option = true;
                    assert!(flags.selectedness());
                    let nearest = record.option_nearest_select().flatten().unwrap();
                    assert!(
                        matches!(nearest.kind(),HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("select"))
                    );
                    assert!(record.form_owner().is_none());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!((input, textarea, select, option), (true, true, true, true));
    assert_eq!(associated.len(), 3);
    for id in associated {
        let node = document.node(&id).unwrap();
        assert!(
            matches!(node.kind(),HtmlNodeKind::Element(e) if ["input","textarea","select"].iter().any(|name|e.name().local_name()==&DocumentString::from(*name)))
        );
    }
    let rows = report.associations();
    let mut attribute_rows = 0;
    for i in 0..rows.len() {
        let row = rows.row(i).unwrap().unwrap();
        if let HtmlParseDestination::Field {
            node,
            field: HtmlParseField::AttributeValue(slot),
        } = row.destination()
        {
            let HtmlNodeKind::Element(element) = node.kind() else {
                panic!("qualified element")
            };
            assert!(slot < element.attributes().len());
            attribute_rows += 1;
        }
    }
    assert!(attribute_rows >= 3);
}

#[test]
fn a_form_inserted_after_its_controls_owns_only_the_control_naming_it_by_id() {
    // The id event of the later form re-resolves the form attribute of the
    // first input; the sixty inputs without a form attribute are not owned by
    // the form, which is not their ancestor either.
    let literal = format!(
        "<!doctype html><body><input form=f>{}<form id=f></form>",
        "<input name=a>".repeat(60)
    );
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let document = report.document().unwrap();
    let mut form = None;
    let mut stack = vec![document.root().id()];
    while let Some(id) = stack.pop() {
        let node = document.node(&id).unwrap();
        if matches!(node.kind(),HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("form"))
        {
            form = Some(id.clone());
        }
        stack.extend(node.children());
    }
    let form = form.unwrap();
    let mut metadata = report.metadata();
    let (mut associations, mut owned) = (0, 0);
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(owner) = record.form_owner() {
                    associations += 1;
                    if let Some(owner) = owner {
                        assert_eq!(owner.id(), form);
                        owned += 1;
                    }
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!((associations, owned), (61, 1));
}

#[test]
fn handoff_resource_stop_remains_typed_in_public_failure() {
    let literal =
        "<!doctype html><form id=f><input type=hidden value=x><textarea>A&amp;B</textarea></form>";
    let failure = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits()
            .into_builder()
            .work_credits(199_923)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap_err();
    assert!(matches!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    assert_eq!(failure.cleanup_error(), None);
}

#[test]
fn completed_owner_transfer_work_stop_remains_a_public_failure() {
    let literal =
        "<!doctype html><form id=f><input type=hidden value=x><textarea>A&amp;B</textarea></form>";
    let failure = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(literal),
        limits()
            .into_builder()
            .work_credits(539_305)
            .finish()
            .unwrap(),
    )
    .parse()
    .unwrap_err();
    assert!(matches!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        })
    ));
    assert_eq!(failure.cleanup_error(), None);
}

#[test]
fn fragment_parse_retains_actual_captured_attachment_and_fallback_after_cancel() {
    let literal = "<template shadowrootmode=closed shadowrootclonable><p>A</p></template><template shadowrootmode=open><p>B</p></template><p>Z</p>";
    let context = fragment_context();
    let capture = context.capture().identity().clone();
    let request = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(literal),
        context,
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let cancel = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancel.cancel();
    assert_eq!(report.source().as_utf8(), Some(literal));
    assert_eq!(report.context().unwrap().capture().identity(), &capture);
    assert!(report.document().is_none());
    let fragment = report.fragment().unwrap();
    let children: Vec<_> = fragment.root().children().collect();
    assert_eq!(children.len(), 2);
    let fallback = fragment.node(&children[0]).unwrap();
    assert!(
        matches!(fallback.kind(),HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("template"))
    );
    assert_eq!(fallback.children().len(), 0);
    let p = fragment.node(&children[1]).unwrap();
    assert!(
        matches!(p.kind(),HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("p"))
    );
    assert!(
        matches!(fragment.node(&p.children().next().unwrap()).unwrap().kind(),HtmlNodeKind::Text(t) if t==&DocumentString::from("Z"))
    );
    let effects = report.effects().unwrap();
    assert_eq!(effects.len(), 2);
    let Some(HtmlParseEffect::AttachShadow {
        host: HtmlParseEffectHost::Captured(host),
        contents,
        source,
        ..
    }) = effects.effect(0).unwrap()
    else {
        panic!("captured shadow attachment")
    };
    assert_eq!(host, report.context().unwrap().context_element());
    assert_eq!(
        report.source().text(source).unwrap(),
        DocumentString::from("<template shadowrootmode=closed shadowrootclonable>")
    );
    let p = contents.children().next().unwrap();
    let p = fragment.node(&p).unwrap();
    assert!(
        matches!(fragment.node(&p.children().next().unwrap()).unwrap().kind(),HtmlNodeKind::Text(t) if t==&DocumentString::from("A"))
    );
    assert!(matches!(
        effects.effect(1),
        Ok(Some(HtmlParseEffect::TemplateCache { .. }))
    ));
    assert_eq!(report.usage(), &usage);
}

#[test]
fn immutable_history_retains_literal_old_leaves_and_one_action_reverse() {
    let literal = "<!doctype html><p>A&amp;B</p>";
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let history = report.history_view();
    let rows = report.associations();
    let mut final_group = None;
    let mut retired = false;
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap().unwrap();
        if matches!(
            row.destination(),
            HtmlParseDestination::Field {
                field: HtmlParseField::Text,
                ..
            }
        ) && row.relation() == HtmlParseAssociationRelation::Output
        {
            final_group = row.group();
        }
        if row.cause().retired_destination().unwrap().is_some() {
            retired = true;
        }
    }
    assert!(retired);
    let group = final_group.unwrap();
    assert_eq!(history.header(group).unwrap().units, 3);
    let mut empty = [];
    assert!(matches!(
        history.cursor(group, &mut empty, None),
        Err(HtmlParseHistoryError::InsufficientScratch { supplied: 0, .. })
    ));
    let foreign_request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
    let query_cancel = foreign_request.cancellation().clone();
    let foreign_report = foreign_request.parse().unwrap();
    assert!(matches!(
        foreign_report.history_view().header(group),
        Err(HtmlParseHistoryError::ForeignOwner)
    ));
    let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
    let mut cursor = history.cursor(group, &mut scratch, None).unwrap();
    let mut units = Vec::new();
    loop {
        match cursor.advance() {
            HtmlParseHistoryStep::Progress => {}
            HtmlParseHistoryStep::Leaf(leaf) => {
                assert_eq!(leaf.offset(), units.len());
                units.extend_from_slice(leaf.units());
            }
            HtmlParseHistoryStep::Complete => break,
            HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
            HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!(units, [65, 38, 66]);
    let reference = report.source().range(19, 24).unwrap();
    assert_eq!(
        report.source().text(&reference).unwrap(),
        DocumentString::from("&amp;")
    );
    let mut reverse = rows.reverse(&reference, None).unwrap();
    let mut count = 0;
    loop {
        match reverse.advance() {
            HtmlParseReverseStep::Match(row) => {
                assert!(row.source().is_some());
                count += 1;
            }
            HtmlParseReverseStep::Skipped => {}
            HtmlParseReverseStep::Complete => break,
            HtmlParseReverseStep::Cancelled => panic!("no query cancellation"),
            HtmlParseReverseStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert!(count > 0);
    query_cancel.cancel();
    let mut reverse = rows.reverse(&reference, Some(&query_cancel)).unwrap();
    assert!(matches!(reverse.advance(), HtmlParseReverseStep::Cancelled));
    let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
    let mut cancelled = history
        .cursor(group, &mut scratch, Some(&query_cancel))
        .unwrap();
    assert!(matches!(
        cancelled.advance(),
        HtmlParseHistoryStep::Cancelled
    ));
    assert_eq!(history.header(group).unwrap().units, 3);
    let foreign = HtmlSource::utf8(literal).range(19, 24).unwrap();
    assert!(matches!(
        rows.reverse(&foreign, None),
        Err(HtmlParseReverseError::ForeignSource)
    ));
}

#[test]
fn attribute_and_pi_map_destinations_resolve_real_container_owners_and_retired_values() {
    let literal = "<input value='A&#13;B&#10;C' type=hidden><?marker name='x&amp;y'?>";
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(literal),
        fragment_context(),
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap();
    let fragment = report.fragment().unwrap();
    let ids: Vec<_> = fragment.root().children().collect();
    assert_eq!(ids.len(), 2);
    let input = fragment.node(&ids[0]).unwrap();
    let HtmlNodeKind::Element(element) = input.kind() else {
        panic!("input")
    };
    assert_eq!(
        element.attributes().as_slice()[0].value(),
        &DocumentString::from("ABC")
    );
    assert!(matches!(
        fragment.node(&ids[1]).unwrap().kind(),
        HtmlNodeKind::ProcessingInstruction(_)
    ));
    let rows = report.associations();
    let history = report.history_view();
    let (mut attribute, mut pi, mut original) = (false, false, false);
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap().unwrap();
        match row.destination() {
            HtmlParseDestination::Field {
                node,
                field: HtmlParseField::AttributeValue(0),
            } => {
                attribute = true;
                assert_eq!(node.id(), ids[0]);
            }
            HtmlParseDestination::Field {
                node,
                field: HtmlParseField::ProcessingInstructionAttributeValue(0),
            } => {
                pi = true;
                assert_eq!(node.id(), ids[1]);
            }
            _ => {}
        }
        let cause = row.cause();
        if let Some(HtmlParseRetiredDestination::StagedField {
            field: HtmlParseField::AttributeValue(0),
        }) = cause.retired_destination().unwrap()
        {
            let group = cause.original_group().unwrap().unwrap();
            let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
            let mut cursor = history.cursor(group, &mut scratch, None).unwrap();
            let mut units = Vec::new();
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Progress => {}
                    HtmlParseHistoryStep::Leaf(leaf) => units.extend_from_slice(leaf.units()),
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                }
            }
            assert_eq!(units, [65, 13, 66, 10, 67]);
            original = true;
        }
    }
    assert_eq!((attribute, pi, original), (true, true, true));
    let mut metadata = report.metadata();
    let mut value = None;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if record.pi_attribute_count() == Some(1) {
                    assert_eq!(record.node().id(), ids[1]);
                    value = Some(record.pi_attribute(0).unwrap().1);
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!(value, Some(&DocumentString::from("x&y")));
}

#[test]
fn retained_source_and_capture_aliases_do_not_alias_terminal_parser_backing() {
    let literal = "<p id=x>A&amp;B</p><textarea>C</textarea>";
    let source = HtmlSource::utf16(DocumentString::from(literal));
    let context = fragment_context();
    let complete = HtmlFragmentParseRequest::with_options(
        source.clone(),
        context.clone(),
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap();
    let final_credits = complete
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    drop(complete);
    for cut in [0, 256, final_credits - 1] {
        let failure = HtmlFragmentParseRequest::with_options(
            source.clone(),
            context.clone(),
            limits().into_builder().work_credits(cut).finish().unwrap(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .parse()
        .unwrap_err();
        assert!(matches!(
            failure.reason(),
            HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            })
        ));
        assert_eq!(failure.cleanup_error(), None);
        assert_eq!(
            failure.requested_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        assert!(!failure.allow_declarative_shadow_roots());
        assert!(failure.capture().is_none() && failure.document().is_none());
        assert_eq!(failure.source().identity(), source.identity());
        assert_eq!(
            failure.context().unwrap().capture().identity(),
            context.capture().identity()
        );
        let usage = failure.retained_usage().unwrap();
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::OpenElements,
        ] {
            assert_eq!(usage.resource(resource).current(), 0);
        }
    }
    let document_source = HtmlSource::utf16(DocumentString::from(
        "<!doctype html><p>A&amp;B<input type=color value=currentcolor>",
    ));
    let failure = HtmlDocumentParseRequest::offline(document_source.clone(), limits())
        .parse()
        .unwrap_err();
    assert!(matches!(
        failure.reason(),
        HtmlParseFailureReason::Stop(HtmlParseStop::ColorProcessing(
            HtmlColorProcessingError::Unavailable(
                surgeist_css::color::CssColorUnavailable::Palette(
                    surgeist_css::color::CssSystemColor::CanvasText,
                )
            ),
        ))
    ));
    assert_eq!(failure.cleanup_error(), None);
    assert_eq!(failure.source().identity(), document_source.identity());
    assert_eq!(
        failure
            .retained_usage()
            .unwrap()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        0
    );
}

#[test]
fn captured_document_rejection_retains_actual_request_context_without_execution_facts() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 100, 10, 1000, 100, 10, 100, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let input = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&input).unwrap())
        .unwrap();
    let source = HtmlSource::utf8("<p>A");
    let request = HtmlDocumentParseRequest::captured(
        source.clone(),
        capture.clone(),
        document.clone(),
        limits()
            .into_builder()
            .work_credits(0)
            .source_units(0)
            .finish()
            .unwrap(),
    )
    .unwrap();
    let cancellation = request.cancellation().clone();
    let failure = request.parse().unwrap_err();
    cancellation.cancel();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::UnsupportedDocumentInitialization
    );
    assert_eq!(failure.source().identity(), source.identity());
    assert_eq!(failure.capture().unwrap().identity(), capture.identity());
    assert_eq!(failure.document(), Some(&document));
    assert!(failure.context().is_none());
    assert_eq!(
        failure.requested_scripting_mode(),
        HtmlParserScriptingMode::Disabled
    );
    assert!(!failure.allow_declarative_shadow_roots());
    assert_eq!(failure.cleanup_error(), None);
    assert_eq!(failure.first_stop(), None);
    assert_eq!(failure.source_admitted(), None);
    assert!(failure.retained_usage().is_none());
    assert!(
        failure.at_stop().is_none()
            && failure.after_cleanup().is_none()
            && failure.diagnostics().is_none()
    );
}

#[test]
fn fragment_template_missing_fact_is_conditional_and_returns_original_context_options() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 100, 10, 1000, 100, 10, 100, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
    let simple = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<p>A"),
        context.clone(),
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap()
    .parse()
    .unwrap();
    assert!(simple.fragment().is_some());
    let source = HtmlSource::utf8("<template>A</template>");
    let request = HtmlFragmentParseRequest::with_options(
        source.clone(),
        context.clone(),
        limits(),
        HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap();
    let cancellation = request.cancellation().clone();
    let failure = request.parse().unwrap_err();
    cancellation.cancel();
    assert_eq!(
        failure.reason(),
        HtmlParseFailureReason::MissingContextFact(HtmlCaptureFact::TemplateDocumentRole)
    );
    assert_eq!(failure.source().identity(), source.identity());
    assert_eq!(failure.context().unwrap().target(), context.target());
    assert_eq!(
        failure.context().unwrap().capture().identity(),
        capture.identity()
    );
    assert_eq!(
        failure.requested_scripting_mode(),
        HtmlParserScriptingMode::Inert
    );
    assert!(failure.allow_declarative_shadow_roots());
    assert_eq!(failure.cleanup_error(), None);
    assert!(matches!(
        failure.first_stop(),
        Some(HtmlParseStop::UnsupportedStandaloneState(
            HtmlParseUnsupportedState::Provenance
        ))
    ));
    assert_eq!(
        failure
            .retained_usage()
            .unwrap()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current(),
        0
    );
}

#[test]
fn successful_utf16_keeps_bom_surrogate_newline_origins_and_ordered_recovery_diagnostics() {
    // Already-decoded input: FEFF is literal, D800 is a code point with a
    // recoverable input-stream error, and CRLF is one preprocessed LF.
    // Missing doctype recovery creates a Quirks html/head/body; the first
    // FEFF text precedes p rather than disappearing as a decoder BOM.
    let raw = vec![
        0xfeff, 0x3c, 0x70, 0x3e, 0x41, 0xd800, 0x0d, 0x0a, 0x42, 0x3c, 0x2f, 0x70, 0x3e,
    ];
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(raw.clone())),
        limits(),
    );
    let identity = request.source().identity().clone();
    let cancel = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancel.cancel();
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(
        report.source().coordinates(),
        HtmlSourceCoordinates::Utf16CodeUnits
    );
    assert_eq!(report.source().as_utf16().unwrap().code_units(), raw);
    assert_eq!(usage.resource(HtmlParseResource::SourceUnits).current(), 13);
    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::Quirks);
    let root_children: Vec<_> = document.root().children().collect();
    assert_eq!(root_children.len(), 1);
    let html = document.node(&root_children[0]).unwrap();
    let HtmlNodeKind::Element(element) = html.kind() else {
        panic!("implied html")
    };
    assert_eq!(element.name().local_name(), &DocumentString::from("html"));
    let html_children: Vec<_> = html.children().collect();
    assert_eq!(html_children.len(), 2);
    let head = document.node(&html_children[0]).unwrap();
    let body = document.node(&html_children[1]).unwrap();
    assert!(
        matches!(head.kind(), HtmlNodeKind::Element(e) if e.name().local_name() == &DocumentString::from("head"))
    );
    assert_eq!(head.children().count(), 0);
    assert!(
        matches!(body.kind(), HtmlNodeKind::Element(e) if e.name().local_name() == &DocumentString::from("body"))
    );
    let body_children: Vec<_> = body.children().collect();
    assert_eq!(body_children.len(), 2);
    let bom_text = document.node(&body_children[0]).unwrap();
    assert!(matches!(bom_text.kind(), HtmlNodeKind::Text(t) if t.code_units() == [0xfeff]));
    let p = document.node(&body_children[1]).unwrap();
    assert!(
        matches!(p.kind(), HtmlNodeKind::Element(e) if e.name().local_name() == &DocumentString::from("p"))
    );
    let p_children: Vec<_> = p.children().collect();
    assert_eq!(p_children.len(), 1);
    let p_text = document.node(&p_children[0]).unwrap();
    assert!(
        matches!(p_text.kind(), HtmlNodeKind::Text(t) if t.code_units() == [0x41, 0xd800, 0x0a, 0x42])
    );
    assert_ne!(bom_text.id(), p_text.id());

    let mut actual_origins = Vec::new();
    let associations = report.associations();
    for i in 0..associations.len() {
        let row = associations.row(i).unwrap().unwrap();
        if row.relation() != HtmlParseAssociationRelation::Output {
            continue;
        }
        let HtmlParseDestination::Field {
            node,
            field: HtmlParseField::Text,
        } = row.destination()
        else {
            continue;
        };
        let is_p = if node.id() == p_text.id() {
            true
        } else {
            assert_eq!(node.id(), bom_text.id());
            false
        };
        let source = row.source().unwrap();
        assert_eq!(source.source(), &identity);
        assert_eq!(source.coordinates(), HtmlSourceCoordinates::Utf16CodeUnits);
        actual_origins.push((
            is_p,
            row.start(),
            row.end(),
            source.start(),
            source.end(),
            row.origin_kind().unwrap(),
        ));
    }
    actual_origins.sort_by_key(|row| (row.0, row.1));
    assert_eq!(
        actual_origins,
        vec![
            (false, 0, 1, 0, 1, HtmlParseOriginKind::Literal),
            (true, 0, 1, 4, 5, HtmlParseOriginKind::Literal),
            (true, 1, 2, 5, 6, HtmlParseOriginKind::Literal),
            (true, 2, 3, 6, 8, HtmlParseOriginKind::Newline),
            (true, 3, 4, 8, 9, HtmlParseOriginKind::Literal),
        ]
    );
    let diagnostics = report.diagnostics();
    assert_eq!(diagnostics.len(), 2);
    for (i, (kind, context, anchor, branch, span, role, token)) in [
        (
            HtmlParseDiagnosticKind::Prologue(HtmlPrologueDiagnostic::MissingDoctype),
            HtmlParseDiagnosticContext::Prologue(HtmlPrologueMode::Initial),
            "the-initial-insertion-mode",
            "anything-else/missing-doctype",
            (0, 1),
            HtmlParseDiagnosticLocationRole::TokenEnvelope,
            Some(HtmlParseTokenFamily::Text),
        ),
        (
            HtmlParseDiagnosticKind::Tokenizer(HtmlTokenizerDiagnostic::SurrogateInInputStream),
            HtmlParseDiagnosticContext::Tokenizer {
                phase: HtmlTokenizerDiagnosticPhase::InputStream,
                state: HtmlTokenizerState::Data,
            },
            "parse-error-surrogate-in-input-stream",
            "surrogate-in-input-stream",
            (5, 6),
            HtmlParseDiagnosticLocationRole::InputFrontier,
            None,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (record, location) = diagnostics.retained_record(i).unwrap().unwrap();
        assert_eq!(record.kind(), kind);
        assert_eq!(record.context(), context);
        assert_eq!(record.severity(), HtmlParseDiagnosticSeverity::Recoverable);
        assert_eq!(record.rule().anchor(), anchor);
        assert_eq!(record.rule().branch(), branch);
        assert_eq!(
            location.source_range(),
            &report.source().range(span.0, span.1).unwrap()
        );
        assert_eq!(location.role(), role);
        assert_eq!(location.token(), token);
    }
    assert!(diagnostics.retained_record(2).unwrap().is_none());
    assert_eq!(report.usage(), &usage);
}

#[test]
fn empty_utf16_recovers_implied_document_with_one_source_qualified_eof_diagnostic() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(Vec::<u16>::new())),
        limits(),
    );
    let identity = request.source().identity().clone();
    let cancel = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancel.cancel();
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(
        report.source().coordinates(),
        HtmlSourceCoordinates::Utf16CodeUnits
    );
    assert_eq!(report.source().as_utf16().unwrap().code_units(), &[]);
    assert_eq!(usage.resource(HtmlParseResource::SourceUnits).current(), 0);
    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::Quirks);
    let root_children: Vec<_> = document.root().children().collect();
    assert_eq!(root_children.len(), 1);
    let html = document.node(&root_children[0]).unwrap();
    assert!(
        matches!(html.kind(), HtmlNodeKind::Element(e) if e.name().local_name() == &DocumentString::from("html"))
    );
    let children: Vec<_> = html.children().collect();
    assert_eq!(children.len(), 2);
    for (id, expected) in children.iter().zip(["head", "body"]) {
        let node = document.node(id).unwrap();
        assert!(
            matches!(node.kind(), HtmlNodeKind::Element(e) if e.name().local_name() == &DocumentString::from(expected))
        );
        assert_eq!(node.children().count(), 0);
    }
    let diagnostics = report.diagnostics();
    assert_eq!(diagnostics.len(), 1);
    let (record, location) = diagnostics.retained_record(0).unwrap().unwrap();
    assert_eq!(
        record.kind(),
        HtmlParseDiagnosticKind::Prologue(HtmlPrologueDiagnostic::MissingDoctype)
    );
    assert_eq!(
        record.context(),
        HtmlParseDiagnosticContext::Prologue(HtmlPrologueMode::Initial)
    );
    assert_eq!(record.severity(), HtmlParseDiagnosticSeverity::Recoverable);
    assert_eq!(record.rule().anchor(), "the-initial-insertion-mode");
    assert_eq!(record.rule().branch(), "anything-else/missing-doctype");
    assert_eq!(
        location.source_range(),
        &report.source().range(0, 0).unwrap()
    );
    assert_eq!(
        location.role(),
        HtmlParseDiagnosticLocationRole::TokenEnvelope
    );
    assert_eq!(location.token(), Some(HtmlParseTokenFamily::Eof));
    assert!(diagnostics.retained_record(1).unwrap().is_none());
    assert_eq!(report.usage(), &usage);
}

#[test]
fn retired_removed_attribute_keeps_historical_ordinal_without_reading_reindexed_current_slot() {
    let literal = "<!doctype html><details name=g open></details><details id=b name=g open data-tail=z></details>";
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits())
        .parse()
        .unwrap();
    let document = report.document().unwrap();
    let html = document
        .root()
        .children()
        .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
        .unwrap();
    let body = document.node(&html).unwrap().children().nth(1).unwrap();
    let details: Vec<_> = document.node(&body).unwrap().children().collect();
    assert_eq!(details.len(), 2);
    let second = document.node(&details[1]).unwrap();
    let HtmlNodeKind::Element(element) = second.kind() else {
        panic!("details")
    };
    let attributes = element.attributes().as_slice();
    assert_eq!(attributes.len(), 3);
    assert_eq!(
        attributes[2].name().local_name(),
        &DocumentString::from("data-tail")
    );
    assert_eq!(attributes[2].value(), &DocumentString::from("z"));
    let rows = report.associations();
    let history = report.history_view();
    let mut historical_name = false;
    let mut current_tail = false;
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap().unwrap();
        if let HtmlParseDestination::Field {
            node,
            field: HtmlParseField::AttributeName(2),
        } = row.destination()
            && node.id() == details[1]
        {
            current_tail = true;
        }
        let cause = row.cause();
        if let Some(HtmlParseRetiredDestination::Field {
            node,
            field: HtmlParseField::AttributeName(2),
        }) = cause.retired_destination().unwrap()
        {
            assert_eq!(node.id(), details[1]);
            let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
            let mut cursor = history
                .cursor(cause.original_group().unwrap().unwrap(), &mut scratch, None)
                .unwrap();
            let mut units = Vec::new();
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Progress => {}
                    HtmlParseHistoryStep::Leaf(leaf) => units.extend_from_slice(leaf.units()),
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                }
            }
            assert_eq!(units, [111, 112, 101, 110]);
            historical_name = true;
        }
    }
    assert!(historical_name && current_tail);
}

#[test]
fn multiple_email_count_retains_produced_empty_values_and_immutable_source_after_cancel() {
    // Splitting precedes per-value whitespace trimming: a final comma does
    // not add a value, but a trailing whitespace segment does. The joined
    // current string cannot recover that distinction.
    for (original, current, count) in [
        ("", "", 0),
        (" ", "", 1),
        ("a,", "a", 1),
        ("a, ", "a,", 2),
        (",,", ",", 2),
        (", , ", ",,", 3),
    ] {
        let literal = format!("<!doctype html><input type=email multiple value='{original}'>");
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal.as_str()), limits());
        let identity = request.source().identity().clone();
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancellation.cancel();
        assert_eq!(report.source().identity(), &identity);
        assert_eq!(report.source().as_utf8(), Some(literal.as_str()));
        let document = report.document().unwrap();
        assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let html_children: Vec<_> = document.node(&html).unwrap().children().collect();
        assert_eq!(html_children.len(), 2);
        let body = document.node(&html_children[1]).unwrap();
        let body_children: Vec<_> = body.children().collect();
        assert_eq!(body_children.len(), 1);
        let input_id = &body_children[0];
        let node = document.node(input_id).unwrap();
        let HtmlNodeKind::Element(element) = node.kind() else {
            panic!("input")
        };
        assert_eq!(element.name().local_name(), &DocumentString::from("input"));
        let attributes = element.attributes().as_slice();
        assert_eq!(attributes.len(), 3);
        for (attribute, (name, value)) in
            attributes
                .iter()
                .zip([("type", "email"), ("multiple", ""), ("value", original)])
        {
            assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
            assert_eq!(attribute.value(), &DocumentString::from(value));
        }
        assert_eq!(node.children().count(), 0);
        let mut metadata = report.metadata();
        let mut found = 0;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(flags) = record.input_flags() {
                        found += 1;
                        assert_eq!(record.node().id(), *input_id);
                        assert_eq!(flags.input_type(), HtmlInputType::Email);
                        assert!(!flags.dirty_value());
                        assert_eq!(
                            record.input_raw_value(),
                            Some(&DocumentString::from(current))
                        );
                        assert_eq!(record.input_email_value_count(), Some(count));
                    } else {
                        assert_eq!(record.input_email_value_count(), None);
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
                HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            }
        }
        assert_eq!(found, 1);
        assert_eq!(report.usage(), &usage);
    }

    let report = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><input type=email value='a,'><input type=hidden value=x>"),
        limits(),
    )
    .parse()
    .unwrap();
    let mut metadata = report.metadata();
    let mut inputs = 0;
    loop {
        match metadata.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if record.input_flags().is_some() {
                    inputs += 1;
                }
                assert_eq!(record.input_email_value_count(), None);
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    assert_eq!(inputs, 2);
}

#[test]
fn button_default_value_preserves_ordered_content_and_retained_raw_after_cancel() {
    for (literal, expected, value_slot, writeback) in [
        ("<!doctype html><input type=button>", "", None, false),
        (
            "<!doctype html><input type=button value='&#13;&#10;X'>",
            "\r\nX",
            Some(1),
            false,
        ),
        (
            "<!doctype html><input value='&#13;&#10;X' type=button>",
            "X",
            Some(0),
            true,
        ),
    ] {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
        let identity = request.source().identity().clone();
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancellation.cancel();
        assert_eq!(report.source().identity(), &identity);
        assert_eq!(report.source().as_utf8(), Some(literal));
        let document = report.document().unwrap();
        assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let children: Vec<_> = document.node(&body).unwrap().children().collect();
        assert_eq!(children.len(), 1);
        let input = document.node(&children[0]).unwrap();
        let HtmlNodeKind::Element(element) = input.kind() else {
            panic!("input")
        };
        assert_eq!(element.name().local_name(), &DocumentString::from("input"));
        let default = value_slot.map(|slot| {
            let attribute = &element.attributes().as_slice()[slot];
            assert_eq!(
                attribute.name().local_name(),
                &DocumentString::from("value")
            );
            attribute.value()
        });
        // Default mode reads the content attribute, with the empty fallback.
        assert_eq!(
            default.map(DocumentString::code_units).unwrap_or(&[]),
            DocumentString::from(expected).code_units()
        );
        assert_eq!(input.children().count(), 0);
        let mut metadata = report.metadata();
        let mut inputs = 0;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(flags) = record.input_flags() {
                        inputs += 1;
                        assert_eq!(record.node().id(), input.id());
                        assert_eq!(flags.input_type(), HtmlInputType::Button);
                        assert_eq!(flags.input_type().value_mode(), HtmlInputValueMode::Default);
                        assert!(!flags.dirty_value());
                        assert_eq!(
                            record.input_raw_value(),
                            Some(&DocumentString::from(expected))
                        );
                        assert_eq!(record.input_creation_writeback(), Some(writeback));
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
                HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            }
        }
        assert_eq!(inputs, 1);
        assert_eq!(report.usage(), &usage);
    }
}

#[test]
fn local_datetime_normalization_retains_exact_space_origin_and_original_value_history() {
    for (literal, source_end, spelling) in [
        (
            "<!doctype html><input type=datetime-local value='2024-02-29 12:34:00.000'>",
            60,
            " ",
        ),
        (
            "<!doctype html><input type=datetime-local value='2024-02-29&#32;12:34:00.000'>",
            64,
            "&#32;",
        ),
    ] {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(literal), limits());
        let identity = request.source().identity().clone();
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancellation.cancel();
        assert_eq!(report.source().identity(), &identity);
        assert_eq!(report.source().as_utf8(), Some(literal));
        let document = report.document().unwrap();
        assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let body = document.node(&html).unwrap().children().nth(1).unwrap();
        let children: Vec<_> = document.node(&body).unwrap().children().collect();
        assert_eq!(children.len(), 1);
        let input = document.node(&children[0]).unwrap();
        let HtmlNodeKind::Element(element) = input.kind() else {
            panic!("input")
        };
        assert_eq!(element.name().local_name(), &DocumentString::from("input"));
        let default = &element.attributes().as_slice()[1];
        assert_eq!(default.name().local_name(), &DocumentString::from("value"));
        assert_eq!(
            default.value(),
            &DocumentString::from("2024-02-29 12:34:00.000")
        );
        let mut metadata = report.metadata();
        let mut inputs = 0;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(flags) = record.input_flags() {
                        inputs += 1;
                        assert_eq!(record.node().id(), input.id());
                        assert_eq!(flags.input_type(), HtmlInputType::LocalDateTime);
                        assert_eq!(flags.input_type().value_mode(), HtmlInputValueMode::Value);
                        assert!(!flags.dirty_value());
                        assert_eq!(
                            record.input_raw_value(),
                            Some(&DocumentString::from("2024-02-29T12:34"))
                        );
                        assert_eq!(record.input_creation_writeback(), Some(false));
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
                HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            }
        }
        assert_eq!(inputs, 1);
        let range = report.source().range(59, source_end).unwrap();
        assert_eq!(
            report.source().text(&range).unwrap(),
            DocumentString::from(spelling)
        );
        let rows = report.associations();
        let history = report.history_view();
        let mut normalized = 0;
        for index in 0..rows.len() {
            let row = rows.row(index).unwrap().unwrap();
            if row.origin_kind() != Some(HtmlParseOriginKind::InputValueNormalization)
                || row.relation() != HtmlParseAssociationRelation::Output
            {
                continue;
            }
            let HtmlParseDestination::Raw { node, kind } = row.destination() else {
                panic!("normalization belongs to retained current input value")
            };
            normalized += 1;
            assert_eq!(node.id(), input.id());
            assert_eq!(kind, HtmlParseRawField::Input);
            assert_eq!((row.start(), row.end()), (10, 11));
            assert_eq!(row.source(), Some(&range));
            assert_eq!(row.source().unwrap().source(), &identity);
            let cause = row.cause();
            assert_eq!(
                cause.operation(),
                HtmlParseConstructionOperation::Replacement
            );
            let original = cause.original_group().unwrap().unwrap();
            let mut scratch = vec![HtmlParseHistoryFrame::default(); history.len()];
            let mut cursor = history.cursor(original, &mut scratch, None).unwrap();
            let mut units = Vec::new();
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Progress => {}
                    HtmlParseHistoryStep::Leaf(leaf) => {
                        assert_eq!(leaf.offset(), units.len());
                        units.extend_from_slice(leaf.units());
                    }
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                }
            }
            assert_eq!(
                units,
                DocumentString::from("2024-02-29 12:34:00.000").code_units()
            );
        }
        assert_eq!(normalized, 1);
        assert_eq!(report.usage(), &usage);
    }
}

#[test]
fn owned_fragment_details_publishes_canonical_exclusivity_without_changing_capture() {
    let literal =
        "<details name=g open=v></details><details name=g open=w data-tail=z></details><p>Z</p>";
    let context = fragment_context();
    let capture = context.capture().identity().clone();
    let request =
        HtmlFragmentParseRequest::new(HtmlSource::utf8(literal), context, limits()).unwrap();
    let cancellation = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancellation.cancel();
    assert_eq!(report.source().as_utf8(), Some(literal));
    assert_eq!(report.context().unwrap().capture().identity(), &capture);
    assert!(report.document().is_none());
    let fragment = report.fragment().unwrap();
    let nodes = fragment.root().children().collect::<Vec<_>>();
    assert_eq!(nodes.len(), 3);
    for (index, expected) in [
        (0, vec![("name", "g"), ("open", "v")]),
        (1, vec![("name", "g"), ("data-tail", "z")]),
    ] {
        let HtmlNodeKind::Element(element) = fragment.node(&nodes[index]).unwrap().kind() else {
            panic!("literal Details")
        };
        assert_eq!(
            element.name().local_name(),
            &DocumentString::from("details")
        );
        assert_eq!(
            element
                .attributes()
                .iter()
                .map(|a| (
                    a.name().local_name().code_units().to_vec(),
                    a.value().code_units().to_vec()
                ))
                .collect::<Vec<_>>(),
            expected
                .into_iter()
                .map(|(n, v)| (n.encode_utf16().collect(), v.encode_utf16().collect()))
                .collect::<Vec<(Vec<u16>, Vec<u16>)>>()
        );
    }
    let p = fragment.node(&nodes[2]).unwrap();
    assert!(
        matches!(p.kind(), HtmlNodeKind::Element(e) if e.name().local_name()==&DocumentString::from("p"))
    );
    assert!(
        matches!(fragment.node(&p.children().next().unwrap()).unwrap().kind(), HtmlNodeKind::Text(t) if t==&DocumentString::from("Z"))
    );
    assert_eq!(report.effects().unwrap().len(), 0);
    assert_eq!(report.usage(), &usage);
}

#[test]
fn pi_recovery_chronology_and_abandoned_eof_are_public_exact_source_evidence() {
    // HTML877d §§13.2.5.72–76. Relative UTF16 frontiers after the prologue:
    // invalid first21, colon27, reserved-target separator35, EOF54.
    // é adds one UTF8 byte, while data "&amp;" in other PI witnesses stays
    // literal rather than invoking character-reference processing.
    let literal = "<!doctype html><p>é<?1?><?a:b><?xml d?><?x unfinished?";
    for utf16 in [false, true] {
        let source = if utf16 {
            HtmlSource::utf16(DocumentString::from(literal))
        } else {
            HtmlSource::utf8(literal)
        };
        let identity = source.identity().clone();
        let request = HtmlDocumentParseRequest::offline(source, limits());
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancellation.cancel();
        assert_eq!(report.source().identity(), &identity);
        let offset = |units| units + usize::from(!utf16);
        let document = report.document().unwrap();
        let html = document
            .root()
            .children()
            .find(|id| matches!(document.node(id).unwrap().kind(), HtmlNodeKind::Element(_)))
            .unwrap();
        let html_children: Vec<_> = document.node(&html).unwrap().children().collect();
        let body = document.node(&html_children[1]).unwrap();
        let p = document.node(&body.children().next().unwrap()).unwrap();
        let children: Vec<_> = p.children().collect();
        assert_eq!(children.len(), 4);
        assert!(
            matches!(document.node(&children[0]).unwrap().kind(),HtmlNodeKind::Text(t) if t==&DocumentString::from("é"))
        );
        for (node, expected) in children[1..].iter().zip(["?1?", "?a:b", "?xml d?"]) {
            assert!(
                matches!(document.node(node).unwrap().kind(),HtmlNodeKind::Comment(t) if t==&DocumentString::from(expected))
            );
        }
        let expected = [
            (
                HtmlTokenizerDiagnostic::InvalidFirstCharacterOfProcessingInstructionTarget,
                HtmlTokenizerState::PiOpen,
                "parse-error-invalid-first-character-of-processing-instruction-target",
                "invalid-first-character-of-processing-instruction-target",
                21,
                22,
            ),
            (
                HtmlTokenizerDiagnostic::InvalidProcessingInstructionTarget,
                HtmlTokenizerState::PiTarget,
                "parse-error-invalid-processing-instruction-target",
                "invalid-processing-instruction-target",
                27,
                28,
            ),
            (
                HtmlTokenizerDiagnostic::DisallowedProcessingInstructionTarget,
                HtmlTokenizerState::PiTarget,
                "parse-error-disallowed-processing-instruction-target",
                "disallowed-processing-instruction-target",
                35,
                36,
            ),
            (
                HtmlTokenizerDiagnostic::EofInProcessingInstruction,
                HtmlTokenizerState::PiQuestionable,
                "parse-error-eof-in-processing-instruction",
                "eof-in-processing-instruction",
                54,
                54,
            ),
        ];
        let diagnostics = report.diagnostics();
        assert_eq!(diagnostics.len(), expected.len());
        for (index, (code, state, anchor, branch, start, end)) in expected.into_iter().enumerate() {
            let (record, location) = diagnostics.retained_record(index).unwrap().unwrap();
            assert_eq!(record.kind(), HtmlParseDiagnosticKind::Tokenizer(code));
            assert_eq!(
                record.context(),
                HtmlParseDiagnosticContext::Tokenizer {
                    phase: HtmlTokenizerDiagnosticPhase::Tokenizer,
                    state
                }
            );
            assert_eq!(record.severity(), HtmlParseDiagnosticSeverity::Recoverable);
            assert_eq!(record.rule().anchor(), anchor);
            assert_eq!(record.rule().branch(), branch);
            assert_eq!(
                location.role(),
                HtmlParseDiagnosticLocationRole::InputFrontier
            );
            assert_eq!(location.token(), None);
            assert_eq!(
                location.source_range(),
                &report.source().range(offset(start), offset(end)).unwrap()
            );
        }
        let rows = report.associations();
        let mut target = false;
        let mut data = Vec::new();
        let mut end = false;
        for index in 0..rows.len() {
            let row = rows.row(index).unwrap().unwrap();
            if let HtmlParseDestination::Lexical {
                occurrence,
                part: Some(part),
            } = row.destination()
                && occurrence.source() == &report.source().range(offset(39), offset(54)).unwrap()
            {
                assert_eq!(
                    occurrence.kind(),
                    HtmlParseLexicalKind::ProcessingInstruction
                );
                assert_eq!(
                    occurrence.disposition(),
                    HtmlParseLexicalDisposition::Abandoned
                );
                match part.role() {
                    HtmlParseLexicalRole::Target => {
                        assert_eq!(
                            part.source(),
                            &report.source().range(offset(41), offset(42)).unwrap()
                        );
                        assert_eq!((part.start(), part.end()), (0, 1));
                        assert_eq!(part.origin_kind(), Some(HtmlParseOriginKind::Literal));
                        target = true;
                    }
                    HtmlParseLexicalRole::Data => {
                        assert_eq!(part.origin_kind(), Some(HtmlParseOriginKind::Literal));
                        data.push((
                            part.start(),
                            part.end(),
                            part.source().start(),
                            part.source().end(),
                        ));
                    }
                    HtmlParseLexicalRole::EndAnchor => {
                        assert_eq!(
                            part.source(),
                            &report.source().range(offset(54), offset(54)).unwrap()
                        );
                        assert_eq!((part.start(), part.end()), (0, 0));
                        end = true;
                    }
                    _ => {}
                }
            }
        }
        assert!(target && end);
        data.sort();
        assert_eq!(
            data,
            (0..10)
                .map(|i| (i, i + 1, offset(43 + i), offset(44 + i)))
                .collect::<Vec<_>>()
        );
        let eof = report.source().range(offset(54), offset(54)).unwrap();
        let mut reverse = rows.reverse(&eof, None).unwrap();
        let mut reverse_anchor = false;
        loop {
            match reverse.advance() {
                HtmlParseReverseStep::Match(row) => {
                    if let HtmlParseDestination::Lexical {
                        occurrence,
                        part: Some(part),
                    } = row.destination()
                        && occurrence.source()
                            == &report.source().range(offset(39), offset(54)).unwrap()
                        && part.role() == HtmlParseLexicalRole::EndAnchor
                    {
                        reverse_anchor = true;
                        assert_eq!(part.source(), &eof);
                        assert_eq!(
                            occurrence.disposition(),
                            HtmlParseLexicalDisposition::Abandoned
                        );
                    }
                    assert!(!matches!(
                        row.destination(),
                        HtmlParseDestination::Field {
                            field: HtmlParseField::ProcessingInstructionTarget
                                | HtmlParseField::ProcessingInstructionData,
                            ..
                        }
                    ));
                }
                HtmlParseReverseStep::Skipped => {}
                HtmlParseReverseStep::Complete => break,
                HtmlParseReverseStep::Cancelled => {
                    panic!("completed immutable evidence has no parse cancellation lease")
                }
                HtmlParseReverseStep::Fault(fault) => panic!("{fault}"),
            }
        }
        assert!(reverse_anchor);
        assert_eq!(report.usage(), &usage);
    }
}

fn value_sharing_report(fragment: bool, markup: &str) -> surgeist_html_parser::HtmlParseReport {
    value_sharing_report_with_limits(fragment, markup, limits())
}

fn value_sharing_report_with_limits(
    fragment: bool,
    markup: &str,
    limits: HtmlParseLimits,
) -> surgeist_html_parser::HtmlParseReport {
    let literal = if fragment {
        markup.to_owned()
    } else {
        format!("<!doctype html>{markup}")
    };
    let source = HtmlSource::utf8(literal.clone());
    let identity = source.identity().clone();
    let report = if fragment {
        let context = fragment_context();
        let capture = context.capture().identity().clone();
        let request = HtmlFragmentParseRequest::new(source, context, limits).unwrap();
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap_or_else(|failure| {
            panic!(
                "value-sharing fixture: {:?}; nodes admitted at stop: {:?}",
                failure.reason(),
                failure
                    .at_stop()
                    .map(|usage| usage.resource(HtmlParseResource::ManagedNodes).current())
            )
        });
        cancellation.cancel();
        assert!(report.document().is_none());
        assert!(report.fragment().is_some());
        assert_eq!(report.context().unwrap().capture().identity(), &capture);
        report
    } else {
        let request = HtmlDocumentParseRequest::offline(source, limits);
        let cancellation = request.cancellation().clone();
        let report = request.parse().unwrap_or_else(|failure| {
            panic!(
                "value-sharing fixture: {:?}; nodes admitted at stop: {:?}",
                failure.reason(),
                failure
                    .at_stop()
                    .map(|usage| usage.resource(HtmlParseResource::ManagedNodes).current())
            )
        });
        cancellation.cancel();
        assert_eq!(
            report.document().unwrap().mode(),
            HtmlDocumentMode::NoQuirks
        );
        assert!(report.fragment().is_none());
        report
    };
    assert_eq!(report.source().identity(), &identity);
    assert_eq!(report.source().as_utf8(), Some(literal.as_str()));
    report
}

fn value_sharing_node<'a>(
    report: &'a surgeist_html_parser::HtmlParseReport,
    id: &HtmlNodeId,
) -> HtmlNode<'a> {
    if let Some(document) = report.document() {
        document.node(id).unwrap()
    } else {
        report.fragment().unwrap().node(id).unwrap()
    }
}

fn value_sharing_elements(
    report: &surgeist_html_parser::HtmlParseReport,
    local_name: &str,
) -> Vec<HtmlNodeId> {
    let root = if let Some(document) = report.document() {
        document.root()
    } else {
        report.fragment().unwrap().root()
    };
    let expected = DocumentString::from(local_name);
    let mut pending = vec![root.id()];
    let mut found = Vec::new();
    while let Some(id) = pending.pop() {
        let node = value_sharing_node(report, &id);
        if matches!(node.kind(), HtmlNodeKind::Element(element) if element.name().local_name() == &expected)
        {
            found.push(id);
        }
        let children: Vec<_> = node.children().collect();
        pending.extend(children.into_iter().rev());
    }
    found
}

fn value_sharing_element<'a>(
    report: &'a surgeist_html_parser::HtmlParseReport,
    id: &HtmlNodeId,
) -> &'a HtmlElement {
    let HtmlNodeKind::Element(element) = value_sharing_node(report, id).kind() else {
        panic!("literal Element")
    };
    assert_eq!(element.name().namespace(), &Namespace::html());
    assert!(element.name().prefix().is_none());
    element
}

fn value_sharing_text<'a>(
    report: &'a surgeist_html_parser::HtmlParseReport,
    parent: &HtmlNodeId,
) -> (HtmlNodeId, &'a DocumentString) {
    let children: Vec<_> = value_sharing_node(report, parent).children().collect();
    assert_eq!(children.len(), 1);
    let HtmlNodeKind::Text(text) = value_sharing_node(report, &children[0]).kind() else {
        panic!("literal Text")
    };
    (children[0].clone(), text)
}

fn assert_value_sharing_origin(
    report: &surgeist_html_parser::HtmlParseReport,
    id: &HtmlNodeId,
    field: HtmlParseField,
    source_start: usize,
    source_end: usize,
    kind: HtmlParseOriginKind,
) {
    let rows = report.associations();
    let mut actual = Vec::new();
    for index in 0..rows.len() {
        let row = rows.row(index).unwrap().unwrap();
        if row.relation() == HtmlParseAssociationRelation::Output
            && let HtmlParseDestination::Field {
                node,
                field: actual_field,
            } = row.destination()
            && node.id() == *id
            && actual_field == field
        {
            let source = row.source().unwrap();
            assert_eq!(source.source(), report.source().identity());
            assert!(row.group().is_some());
            actual.push((
                row.start(),
                row.end(),
                source.start(),
                source.end(),
                row.origin_kind().unwrap(),
            ));
        }
    }
    assert_eq!(actual, [(0, 1, source_start, source_end, kind)]);
}

fn assert_equal_report_attribute_banks(fragment: bool) {
    let report = value_sharing_report(
        fragment,
        "<table><tbody><tr><td class=k title=v>A</td><td class=k title=v>B</td></tr></tbody></table>",
    );
    let cells = value_sharing_elements(&report, "td");
    assert_eq!(cells.len(), 2);
    assert_ne!(cells[0], cells[1]);
    let elements = [&cells[0], &cells[1]].map(|id| value_sharing_element(&report, id));
    let literal = report.source().as_utf8().unwrap();
    let class_sources: Vec<_> = literal
        .match_indices("class=k")
        .map(|(i, _)| i + 6)
        .collect();
    let title_sources: Vec<_> = literal
        .match_indices("title=v")
        .map(|(i, _)| i + 6)
        .collect();
    assert_eq!(class_sources.len(), 2);
    assert_eq!(title_sources.len(), 2);
    let mut text_ids = Vec::new();
    for index in 0..2 {
        let attributes = elements[index].attributes();
        assert_eq!(attributes.len(), 2);
        for (ordinal, (name, value)) in [("class", "k"), ("title", "v")].into_iter().enumerate() {
            let attribute = &attributes.as_slice()[ordinal];
            assert_eq!(attribute.name().namespace(), &Namespace::none());
            assert!(attribute.name().prefix().is_none());
            assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
            assert_eq!(attribute.value(), &DocumentString::from(value));
            let source = if ordinal == 0 {
                class_sources[index]
            } else {
                title_sources[index]
            };
            assert_value_sharing_origin(
                &report,
                &cells[index],
                HtmlParseField::AttributeValue(ordinal),
                source,
                source + 1,
                HtmlParseOriginKind::Literal,
            );
        }
        let (id, text) = value_sharing_text(&report, &cells[index]);
        let expected = if index == 0 { "A" } else { "B" };
        assert_eq!(text, &DocumentString::from(expected));
        let source = literal
            .find(if index == 0 { ">A<" } else { ">B<" })
            .unwrap()
            + 1;
        assert_value_sharing_origin(
            &report,
            &id,
            HtmlParseField::Text,
            source,
            source + 1,
            HtmlParseOriginKind::Literal,
        );
        text_ids.push(id);
    }
    assert_ne!(text_ids[0], text_ids[1]);
    // The report itself must share complete banks; later DOM copies cannot
    // reclaim duplicate backing retained by this original immutable owner.
    assert!(std::sync::Arc::ptr_eq(
        elements[0].attributes().shared_attributes(),
        elements[1].attributes().shared_attributes(),
    ));
}

fn assert_distinct_report_banks_share_strings(fragment: bool) {
    let report = value_sharing_report(fragment, "<p id=a class=k></p><p id=b class=k></p>");
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 2);
    assert_ne!(paragraphs[0], paragraphs[1]);
    let elements = [&paragraphs[0], &paragraphs[1]].map(|id| value_sharing_element(&report, id));
    let literal = report.source().as_utf8().unwrap();
    let class_sources: Vec<_> = literal
        .match_indices("class=k")
        .map(|(i, _)| i + 6)
        .collect();
    assert_eq!(class_sources.len(), 2);
    for index in 0..2 {
        let attributes = elements[index].attributes();
        assert_eq!(attributes.len(), 2);
        for (ordinal, name) in ["id", "class"].into_iter().enumerate() {
            let attribute = &attributes.as_slice()[ordinal];
            assert_eq!(attribute.name().namespace(), &Namespace::none());
            assert!(attribute.name().prefix().is_none());
            assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
        }
        assert_eq!(
            attributes.as_slice()[0].value(),
            &DocumentString::from(if index == 0 { "a" } else { "b" })
        );
        assert_eq!(attributes.as_slice()[1].value(), &DocumentString::from("k"));
        assert_eq!(
            value_sharing_node(&report, &paragraphs[index])
                .children()
                .count(),
            0
        );
        let id_source = literal
            .find(if index == 0 { "id=a" } else { "id=b" })
            .unwrap()
            + 3;
        assert_value_sharing_origin(
            &report,
            &paragraphs[index],
            HtmlParseField::AttributeValue(0),
            id_source,
            id_source + 1,
            HtmlParseOriginKind::Literal,
        );
        assert_value_sharing_origin(
            &report,
            &paragraphs[index],
            HtmlParseField::AttributeValue(1),
            class_sources[index],
            class_sources[index] + 1,
            HtmlParseOriginKind::Literal,
        );
    }
    assert!(!std::sync::Arc::ptr_eq(
        elements[0].attributes().shared_attributes(),
        elements[1].attributes().shared_attributes()
    ));
    let attributes = elements.map(HtmlElement::attributes);
    assert!(std::sync::Arc::ptr_eq(
        attributes[0].as_slice()[1].value().shared_code_units(),
        attributes[1].as_slice()[1].value().shared_code_units()
    ));
    for ordinal in 0..2 {
        assert!(std::sync::Arc::ptr_eq(
            attributes[0].as_slice()[ordinal]
                .name()
                .local_name()
                .shared_code_units(),
            attributes[1].as_slice()[ordinal]
                .name()
                .local_name()
                .shared_code_units()
        ));
    }
    assert!(std::sync::Arc::ptr_eq(
        elements[0].name().local_name().shared_code_units(),
        elements[1].name().local_name().shared_code_units()
    ));
    assert!(std::sync::Arc::ptr_eq(
        elements[0]
            .name()
            .namespace()
            .uri()
            .unwrap()
            .shared_code_units(),
        elements[1]
            .name()
            .namespace()
            .uri()
            .unwrap()
            .shared_code_units()
    ));
}

fn assert_independent_report_text_shares_strings(fragment: bool) {
    let report = value_sharing_report(fragment, "<p>X</p><p>&#88;</p>");
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 2);
    assert_ne!(paragraphs[0], paragraphs[1]);
    let (first_id, first) = value_sharing_text(&report, &paragraphs[0]);
    let (second_id, second) = value_sharing_text(&report, &paragraphs[1]);
    assert_ne!(first_id, second_id);
    assert_eq!(first.code_units(), &[88]);
    assert_eq!(second.code_units(), &[88]);
    let prefix = if fragment { 0 } else { "<!doctype html>".len() };
    assert_value_sharing_origin(
        &report,
        &first_id,
        HtmlParseField::Text,
        prefix + 3,
        prefix + 4,
        HtmlParseOriginKind::Literal,
    );
    assert_value_sharing_origin(
        &report,
        &second_id,
        HtmlParseField::Text,
        prefix + 11,
        prefix + 16,
        HtmlParseOriginKind::CharacterReference,
    );
    assert!(std::sync::Arc::ptr_eq(
        first.shared_code_units(),
        second.shared_code_units()
    ));
}

#[test]
fn report_value_sharing_equal_attribute_banks_in_document() {
    assert_equal_report_attribute_banks(false);
}

#[test]
fn report_value_sharing_equal_attribute_banks_in_fragment() {
    assert_equal_report_attribute_banks(true);
}

#[test]
fn report_value_sharing_distinct_banks_share_components_in_document() {
    assert_distinct_report_banks_share_strings(false);
}

#[test]
fn report_value_sharing_distinct_banks_share_components_in_fragment() {
    assert_distinct_report_banks_share_strings(true);
}

#[test]
fn report_value_sharing_independent_text_in_document() {
    assert_independent_report_text_shares_strings(false);
}

#[test]
fn report_value_sharing_independent_text_in_fragment() {
    assert_independent_report_text_shares_strings(true);
}

#[test]
fn report_value_sharing_preserves_raw_surrogates_and_each_source_interval() {
    let mut raw: Vec<u16> = "<!doctype html><p>".encode_utf16().collect();
    raw.push(0xd800);
    raw.extend("</p><p>".encode_utf16());
    raw.push(0xd800);
    raw.extend("</p>".encode_utf16());
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(DocumentString::from_utf16(raw.clone())),
        limits(),
    );
    let source = request.source().identity().clone();
    let cancel = request.cancellation().clone();
    let report = request.parse().unwrap();
    assert_eq!(report.source().identity(), &source);
    assert_eq!(report.source().as_utf16().unwrap().code_units(), raw);
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 2);
    let (first_id, first) = value_sharing_text(&report, &paragraphs[0]);
    let (second_id, second) = value_sharing_text(&report, &paragraphs[1]);
    assert_ne!(first_id, second_id);
    assert_eq!(first.code_units(), &[0xd800]);
    assert_eq!(second.code_units(), &[0xd800]);
    for (id, start) in [(&first_id, 18), (&second_id, 26)] {
        assert_value_sharing_origin(
            &report,
            id,
            HtmlParseField::Text,
            start,
            start + 1,
            HtmlParseOriginKind::Literal,
        );
    }
    let diagnostics = report.diagnostics();
    assert_eq!(diagnostics.len(), 2);
    for index in 0..2 {
        let (record, location) = diagnostics.retained_record(index).unwrap().unwrap();
        assert_eq!(
            record.kind(),
            HtmlParseDiagnosticKind::Tokenizer(HtmlTokenizerDiagnostic::SurrogateInInputStream)
        );
        assert_eq!(location.source_range().source(), &source);
        assert_eq!(location.source_range().start(), [18, 26][index]);
        assert_eq!(location.source_range().end(), [19, 27][index]);
    }
    assert!(std::sync::Arc::ptr_eq(
        first.shared_code_units(),
        second.shared_code_units()
    ));
    let held = first.clone();
    let weak = std::sync::Arc::downgrade(held.shared_code_units());
    let usage = report.usage().clone();
    cancel.cancel();
    assert_eq!(report.usage(), &usage);
    drop(report);
    assert_eq!(held.code_units(), &[0xd800]);
    drop(held);
    assert!(weak.upgrade().is_none());
}

#[test]
fn report_value_sharing_occurs_after_real_body_attribute_merge() {
    let report = value_sharing_report(
        false,
        "<body class=k><body id=x class=ignored><p class=k id=x></p>",
    );
    let bodies = value_sharing_elements(&report, "body");
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(bodies.len(), 1);
    assert_eq!(paragraphs.len(), 1);
    let body = value_sharing_element(&report, &bodies[0]);
    let paragraph = value_sharing_element(&report, &paragraphs[0]);
    for element in [body, paragraph] {
        let actual: Vec<_> = element
            .attributes()
            .iter()
            .map(|attribute| {
                (
                    attribute.name().local_name().code_units(),
                    attribute.value().code_units(),
                )
            })
            .collect();
        assert_eq!(
            actual,
            [
                (&[99, 108, 97, 115, 115][..], &[107][..]),
                (&[105, 100][..], &[120][..])
            ]
        );
    }
    let literal = report.source().as_utf8().unwrap();
    for (id, value, field) in [
        (&bodies[0], "class=k", HtmlParseField::AttributeValue(0)),
        (&bodies[0], "id=x", HtmlParseField::AttributeValue(1)),
    ] {
        let start = literal.find(value).unwrap() + value.len() - 1;
        assert_value_sharing_origin(
            &report,
            id,
            field,
            start,
            start + 1,
            HtmlParseOriginKind::Literal,
        );
    }
    assert!(std::sync::Arc::ptr_eq(
        body.attributes().shared_attributes(),
        paragraph.attributes().shared_attributes()
    ));
}

fn assert_late_report_patterns(fragment: bool) {
    let mut markup = String::new();
    for index in 0..384 {
        markup.push_str(&format!("<p id=v{index:03} class=k></p>"));
    }
    markup.push_str("<p id=q class=z>R</p><p id=q class=z>R</p>");
    // Native association projection prepares growing fixed-record banks before
    // report finalization. This input-only setup envelope allows 64 copies or
    // visits of a 512-byte record per authored paragraph pair, with 16 fixed
    // root/late records. It is not a sharing-performance or output-size oracle.
    // All retained-storage limits and the ordinary limit tests stay unchanged.
    let work = 32_768 * (384_u64 + 16).pow(2);
    let report = value_sharing_report_with_limits(
        fragment,
        &markup,
        limits().into_builder().work_credits(work).finish().unwrap(),
    );
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 386);
    for (index, id) in paragraphs.iter().take(384).enumerate() {
        let element = value_sharing_element(&report, id);
        assert_eq!(
            element.attributes().as_slice()[0].value(),
            &DocumentString::from(format!("v{index:03}"))
        );
        assert_eq!(
            element.attributes().as_slice()[1].value().code_units(),
            &[107]
        );
        assert_eq!(value_sharing_node(&report, id).children().count(), 0);
    }
    let literal = report.source().as_utf8().unwrap();
    let id_sources: Vec<_> = literal
        .match_indices("id=q")
        .map(|(start, _)| start + 3)
        .collect();
    let class_sources: Vec<_> = literal
        .match_indices("class=z")
        .map(|(start, _)| start + 6)
        .collect();
    let text_sources: Vec<_> = literal
        .match_indices(">R<")
        .map(|(start, _)| start + 1)
        .collect();
    assert_eq!(
        (id_sources.len(), class_sources.len(), text_sources.len()),
        (2, 2, 2)
    );
    let late = [&paragraphs[384], &paragraphs[385]].map(|id| value_sharing_element(&report, id));
    let mut texts = Vec::new();
    for index in 0..2 {
        assert_eq!(
            late[index].attributes().as_slice()[0].value().code_units(),
            &[113]
        );
        assert_eq!(
            late[index].attributes().as_slice()[1].value().code_units(),
            &[122]
        );
        assert_value_sharing_origin(
            &report,
            &paragraphs[384 + index],
            HtmlParseField::AttributeValue(0),
            id_sources[index],
            id_sources[index] + 1,
            HtmlParseOriginKind::Literal,
        );
        assert_value_sharing_origin(
            &report,
            &paragraphs[384 + index],
            HtmlParseField::AttributeValue(1),
            class_sources[index],
            class_sources[index] + 1,
            HtmlParseOriginKind::Literal,
        );
        let (id, value) = value_sharing_text(&report, &paragraphs[384 + index]);
        assert_eq!(value.code_units(), &[82]);
        assert_value_sharing_origin(
            &report,
            &id,
            HtmlParseField::Text,
            text_sources[index],
            text_sources[index] + 1,
            HtmlParseOriginKind::Literal,
        );
        texts.push((id, value));
    }
    assert_ne!(paragraphs[384], paragraphs[385]);
    assert_ne!(texts[0].0, texts[1].0);
    assert!(std::sync::Arc::ptr_eq(
        late[0].attributes().shared_attributes(),
        late[1].attributes().shared_attributes()
    ));
    assert!(std::sync::Arc::ptr_eq(
        texts[0].1.shared_code_units(),
        texts[1].1.shared_code_units()
    ));
}

fn assert_empty_report_values(fragment: bool) {
    let report = value_sharing_report(fragment, "<p></p><p></p><p a='' b=''></p><p a='' b=''></p>");
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 4);
    let elements = paragraphs
        .iter()
        .map(|id| value_sharing_element(&report, id))
        .collect::<Vec<_>>();
    for id in &paragraphs {
        assert_eq!(value_sharing_node(&report, id).children().count(), 0);
    }
    assert!(elements[0].attributes().is_empty() && elements[1].attributes().is_empty());
    assert!(std::sync::Arc::ptr_eq(
        elements[0].attributes().shared_attributes(),
        elements[1].attributes().shared_attributes()
    ));
    for element in &elements[2..] {
        assert_eq!(element.attributes().len(), 2);
        for (attribute, name) in element.attributes().iter().zip(["a", "b"]) {
            assert_eq!(attribute.name().local_name(), &DocumentString::from(name));
            assert_eq!(attribute.name().namespace(), &Namespace::none());
            assert!(attribute.name().prefix().is_none());
            assert!(attribute.value().is_empty());
        }
    }
    assert!(std::sync::Arc::ptr_eq(
        elements[2].attributes().shared_attributes(),
        elements[3].attributes().shared_attributes()
    ));
    assert!(std::sync::Arc::ptr_eq(
        elements[2].attributes().as_slice()[0]
            .value()
            .shared_code_units(),
        elements[2].attributes().as_slice()[1]
            .value()
            .shared_code_units()
    ));
}

fn assert_report_prefix_roles(fragment: bool) {
    let report = value_sharing_report(
        fragment,
        "<svg><use xlink:href=x href=x></use><use xlink:href=x href=x></use></svg>",
    );
    let uses = value_sharing_elements(&report, "use");
    assert_eq!(uses.len(), 2);
    let elements = uses
        .iter()
        .map(|id| {
            let HtmlNodeKind::Element(element) = value_sharing_node(&report, id).kind() else {
                panic!("literal SVG use")
            };
            element
        })
        .collect::<Vec<_>>();
    for element in &elements {
        assert_eq!(element.name().namespace(), &Namespace::svg());
        assert!(element.name().prefix().is_none());
        assert_eq!(element.attributes().len(), 2);
        let qualified = &element.attributes().as_slice()[0];
        let plain = &element.attributes().as_slice()[1];
        assert_eq!(
            qualified.name().namespace(),
            &Namespace::from_uri("http://www.w3.org/1999/xlink")
        );
        assert_eq!(
            qualified.name().prefix(),
            Some(&DocumentString::from("xlink"))
        );
        assert_eq!(plain.name().namespace(), &Namespace::none());
        assert!(plain.name().prefix().is_none());
        for attribute in [qualified, plain] {
            assert_eq!(
                attribute.name().local_name().code_units(),
                &[104, 114, 101, 102]
            );
            assert_eq!(attribute.value().code_units(), &[120]);
        }
    }
    assert_ne!(uses[0], uses[1]);
    let literal = report.source().as_utf8().unwrap();
    let sources: Vec<_> = literal
        .match_indices("xlink:href=x href=x")
        .map(|(start, _)| start)
        .collect();
    assert_eq!(sources.len(), 2);
    for index in 0..2 {
        for (ordinal, offset) in [(0, 11), (1, 18)] {
            assert_value_sharing_origin(
                &report,
                &uses[index],
                HtmlParseField::AttributeValue(ordinal),
                sources[index] + offset,
                sources[index] + offset + 1,
                HtmlParseOriginKind::Literal,
            );
        }
    }
    assert!(std::sync::Arc::ptr_eq(
        elements[0].attributes().shared_attributes(),
        elements[1].attributes().shared_attributes()
    ));
    assert!(std::sync::Arc::ptr_eq(
        elements[0].attributes().as_slice()[0]
            .value()
            .shared_code_units(),
        elements[0].attributes().as_slice()[1]
            .value()
            .shared_code_units()
    ));
}

fn assert_oversized_report_banks(fragment: bool) {
    let mut tag = String::from("<p");
    for ordinal in 0..65 {
        tag.push_str(&format!(" a{ordinal:02}=v"));
    }
    tag.push_str("></p>");
    let report = value_sharing_report(fragment, &(tag.clone() + &tag));
    let paragraphs = value_sharing_elements(&report, "p");
    assert_eq!(paragraphs.len(), 2);
    assert_ne!(paragraphs[0], paragraphs[1]);
    let literal = report.source().as_utf8().unwrap();
    for (index, id) in paragraphs.iter().enumerate() {
        let element = value_sharing_element(&report, id);
        assert_eq!(element.attributes().len(), 65);
        assert_eq!(value_sharing_node(&report, id).children().count(), 0);
        assert!(
            report.retained_construction_bytes()
                >= (element.attributes().shared_attributes().capacity()
                    * std::mem::size_of::<Attribute>()) as u64
        );
        for (ordinal, attribute) in element.attributes().iter().enumerate() {
            let name = format!("a{ordinal:02}");
            assert_eq!(
                attribute.name().local_name(),
                &DocumentString::from(name.clone())
            );
            assert_eq!(attribute.name().namespace(), &Namespace::none());
            assert!(attribute.name().prefix().is_none());
            assert_eq!(attribute.value().code_units(), &[118]);
            let marker = format!("{name}=v");
            let start = literal.match_indices(&marker).nth(index).unwrap().0 + marker.len() - 1;
            assert_value_sharing_origin(
                &report,
                id,
                HtmlParseField::AttributeValue(ordinal),
                start,
                start + 1,
                HtmlParseOriginKind::Literal,
            );
        }
    }
    // Larger banks may pass through optional sharing. Exact values, order,
    // selected capacity and each original origin remain report contracts.
}

#[test]
fn report_value_sharing_late_patterns_in_document() {
    assert_late_report_patterns(false);
}
#[test]
fn report_value_sharing_late_patterns_in_fragment() {
    assert_late_report_patterns(true);
}
#[test]
fn report_value_sharing_empty_banks_and_values_in_document() {
    assert_empty_report_values(false);
}
#[test]
fn report_value_sharing_empty_banks_and_values_in_fragment() {
    assert_empty_report_values(true);
}
#[test]
fn report_value_sharing_preserves_prefix_roles_in_document() {
    assert_report_prefix_roles(false);
}
#[test]
fn report_value_sharing_preserves_prefix_roles_in_fragment() {
    assert_report_prefix_roles(true);
}
#[test]
fn report_value_sharing_oversized_bank_pass_through_in_document() {
    assert_oversized_report_banks(false);
}
#[test]
fn report_value_sharing_oversized_bank_pass_through_in_fragment() {
    assert_oversized_report_banks(true);
}
