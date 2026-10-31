use super::super::end::EndPhase;
use super::super::head::HeadStep;
use super::*;
use crate::construction::ConstructionCompletion;
use crate::construction::ConstructionReadiness;
use crate::construction::native_view::{NativeFragmentOwner, NativeKind, NativeString, NativeView};
use crate::{DocumentString, HtmlDocumentParseRequest, HtmlParseResource, HtmlSource};

#[test]
fn completed_lexical_duplicate_quotes_omissions_and_d1_keep_actual_source_fields() {
    use crate::construction::provenance::{
        AssociationRelation, LexicalDisposition, LexicalKind, LexicalRole, NativeDestination,
        OmissionKind,
    };
    let source = HtmlSource::utf8(
        " \r\n<!doctype html><HTML A='x&NotEqualTilde;' a=discard empty=\"\"><body><textarea>\nX</textarea>\0</bogus></body></html>",
    );
    let request = HtmlDocumentParseRequest::offline(source, super::super::tests::limits(100));
    let mut completed = frozen_completed(&request);
    assert!(completed.associations().is_err());
    completed.prepare_native_associations().unwrap();
    let view = completed.associations().unwrap();
    let mut duplicate = false;
    let mut ignored = false;
    let mut initial_lf = false;
    let mut null = false;
    let mut leading = false;
    let mut entity = 0;
    let mut quotes = 0;
    for index in 0..view.len() {
        let row = view.row(index).unwrap();
        view.cause(row.cause).unwrap();
        match row.destination {
            NativeDestination::Lexical {
                occurrence,
                part: None,
            } => {
                assert_eq!(row.relation, AssociationRelation::LexicalEnvelope);
                let occurrence = view.lexical_occurrence(occurrence).unwrap();
                let raw = request.source().text(&occurrence.source).unwrap();
                if raw == DocumentString::from("a=discard") {
                    assert_eq!(occurrence.kind, LexicalKind::Attribute);
                    assert_eq!(
                        occurrence.disposition,
                        LexicalDisposition::DuplicateAttribute
                    );
                    assert_eq!(
                        view.lexical_occurrence(occurrence.parent.unwrap())
                            .unwrap()
                            .kind,
                        LexicalKind::StartTag
                    );
                    duplicate = true;
                }
                if raw == DocumentString::from("</bogus>") {
                    assert_eq!(occurrence.disposition, LexicalDisposition::Ignored);
                    ignored = true;
                }
            }
            NativeDestination::Lexical {
                part: Some(part), ..
            } => {
                let part = view.lexical_part(part).unwrap();
                if matches!(
                    part.role,
                    LexicalRole::OpeningQuote(_) | LexicalRole::ClosingQuote(_)
                ) {
                    quotes += 1;
                }
                if part.kind
                    == Some(crate::parser::tokenizer::token::OriginKind::CharacterReference)
                {
                    assert_eq!(
                        request.source().text(&part.source).unwrap(),
                        DocumentString::from("&NotEqualTilde;")
                    );
                    assert!(matches!((part.start, part.end), (1, 2) | (2, 3)));
                    entity += 1;
                }
            }
            NativeDestination::Cause(index) => {
                assert_eq!(row.relation, AssociationRelation::Cause);
                assert_eq!(index, row.cause);
                view.cause(index).unwrap();
            }
            NativeDestination::Omission(index) => {
                assert_eq!(row.relation, AssociationRelation::Omission);
                let omission = view.omission(index).unwrap();
                match omission.kind {
                    OmissionKind::LeadingWhitespace => {
                        assert!(matches!(
                            omission.rule,
                            crate::construction::provenance::ParserRule::Prologue(_)
                        ));
                        leading = true;
                    }
                    OmissionKind::InitialLf => {
                        assert_eq!(
                            request.source().text(&omission.source).unwrap(),
                            DocumentString::from("\n")
                        );
                        initial_lf = true;
                    }
                    OmissionKind::NullInBody => {
                        assert_eq!(
                            request.source().text(&omission.source).unwrap(),
                            DocumentString::from_utf16(vec![0])
                        );
                        null = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    assert!(duplicate && ignored && initial_lf && null && leading);
    assert_eq!(entity, 2);
    assert_eq!(quotes, 4);
    assert!(view.group(0).is_some());
    let query = request.source().range(0, request.source().len()).unwrap();
    let mut reverse = view.reverse(&query, None).unwrap();
    let mut rows = 0;
    loop {
        match reverse.advance() {
            crate::construction::provenance::reverse::ReverseStep::Match(_)
            | crate::construction::provenance::reverse::ReverseStep::Skipped => rows += 1,
            crate::construction::provenance::reverse::ReverseStep::Complete => {
                break;
            }
            _ => panic!("uncancelled readback"),
        }
    }
    assert_eq!(rows, view.len());
    let point = request
        .source()
        .range(request.source().len(), request.source().len())
        .unwrap();
    let mut eof = view.reverse(&point, None).unwrap();
    let mut anchor = 0;
    loop {
        match eof.advance() {
            crate::construction::provenance::reverse::ReverseStep::Match(row) => {
                if matches!(row.destination, NativeDestination::LexicalEnd) {
                    assert_eq!(row.relation, AssociationRelation::LexicalAnchor);
                    assert_eq!(row.source.as_ref().unwrap().start(), request.source().len());
                    assert_eq!(row.source.as_ref().unwrap().end(), request.source().len());
                    anchor += 1;
                }
            }
            crate::construction::provenance::reverse::ReverseStep::Skipped => {}
            crate::construction::provenance::reverse::ReverseStep::Complete => {
                break;
            }
            _ => panic!("uncancelled EOF observation"),
        }
    }
    assert_eq!(anchor, 1);
    completed.release().unwrap();
    // release consumes the actual completed owner; no borrowed view remains.
    assert!(completed.driver.is_none());
}

#[test]
fn completed_lexical_already_present_attributes_and_retired_history_keep_exact_origins() {
    use crate::construction::provenance::{
        AssociationRelation as R, LexicalDisposition as D, NativeDestination as N,
    };
    let source = HtmlSource::utf8(
        "<!doctype html><html a=first><body>x&amp;y<html A=discard b=kept><p id=p>A<b>B</b>C</p></body></html>",
    );
    let request = HtmlDocumentParseRequest::offline(source, super::super::tests::limits(100));
    let mut completed = frozen_completed(&request);
    completed.prepare_native_associations().unwrap();
    let view = completed.associations().unwrap();
    let mut ignored = false;
    let mut retired = false;
    let mut original = false;
    let mut trigger = false;
    for index in 0..view.len() {
        let row = view.row(index).unwrap();
        if let N::Lexical {
            occurrence,
            part: None,
        } = row.destination
        {
            let occurrence = view.lexical_occurrence(occurrence).unwrap();
            if request.source().text(&occurrence.source).unwrap()
                == DocumentString::from("A=discard")
            {
                assert_eq!(occurrence.disposition, D::AlreadyPresentAttribute);
                ignored = true;
            }
        }
        original |= row.relation == R::Original && row.source.is_some();
        trigger |= row.relation == R::Trigger && row.source.is_some();
    }
    for index in 0..view.len() {
        let row = view.row(index).unwrap();
        let cause = view.cause(row.cause).unwrap();
        if let Some(group) = cause.original {
            let group = view.group(group).unwrap();
            if let Some(retirement) = group.retirement {
                retired = true;
                assert!(group.units > 0);
                view.cause(group.cause).unwrap();
                view.cause(retirement).unwrap();
            }
        }
    }
    assert!(ignored && retired && original && trigger);
    completed.release().unwrap();
}

fn frozen_completed(request: &HtmlDocumentParseRequest) -> CompletedStandaloneDocument<'_> {
    let mut entry = super::super::entry::OrdinaryDocumentEntry::new(request);
    let mut driver = Some(entry.take_driver().unwrap());
    for _ in 0..4096 {
        if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
            return StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
        }
    }
    panic!("real EOF was not reached");
}
fn native_child<'a>(
    document: &'a crate::HtmlDocument,
    parent: crate::HtmlNode<'a>,
    local: &str,
) -> crate::HtmlNode<'a> {
    for id in parent.children() {
        let node = document.node(&id).unwrap();
        if let crate::HtmlNodeKind::Element(element) = node.kind()
            && element.name().local_name().code_units() == local.encode_utf16().collect::<Vec<_>>()
        {
            return node;
        }
    }
    panic!("missing literal child {local}");
}
#[test]
fn frozen_parts_end_borrow_and_retain_same_source_native_identity_and_completion() {
    fn produce() -> (FrozenStandaloneParts, crate::HtmlSourceId) {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>x"),
            super::super::tests::limits(100),
        );
        let source = request.source().identity().clone();
        let mut completed = frozen_completed(&request);
        let parts = completed.freeze_parts().unwrap();
        assert!(completed.driver.is_none());
        (parts, source)
    }
    let (mut parts, source) = produce();
    assert_eq!(
        parts.source_binding(),
        (&source, crate::HtmlSourceCoordinates::Utf8Bytes, 19)
    );
    assert_eq!(
        parts.document_state().completion,
        ConstructionCompletion::Complete
    );
    assert!(parts.final_usage().actual_capacity_bytes > 0);
    assert!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::ManagedUtf16Units)
            .current
            > 0
    );
    let document = parts.document().unwrap();
    let html = native_child(document, document.root(), "html");
    let body = native_child(document, html, "body");
    let p = native_child(document, body, "p");
    let text = document.node(&p.children().next().unwrap()).unwrap();
    assert!(matches!(text.kind(), crate::HtmlNodeKind::Text(value) if value.code_units()==[120]));
    parts.release().unwrap();
    assert!(parts.after_cleanup().is_some());
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_exact_utf16_recovery_and_committed_diagnostic_owner_survive_move() {
    let mut source = "<!doctype html><p>".encode_utf16().collect::<Vec<_>>();
    source.extend([0xd83d, 0xde00, 0xd800, 13, 10, 0, 38, 97, 109, 112, 59]);
    let length = source.len();
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(crate::DocumentString::from_utf16(source)),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let pointer = completed.diagnostics().unwrap().records_pointer();
    let count = completed.diagnostics().unwrap().len();
    let mut parts = completed.freeze_parts().unwrap();
    assert_eq!(
        parts.source_binding(),
        (
            request.source().identity(),
            crate::HtmlSourceCoordinates::Utf16CodeUnits,
            length
        )
    );
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    assert_eq!(parts.diagnostics().unwrap().len(), count);
    assert!(count > 0);
    let document = parts.document().unwrap();
    let html = native_child(document, document.root(), "html");
    let body = native_child(document, html, "body");
    let p = native_child(document, body, "p");
    let text = document.node(&p.children().next().unwrap()).unwrap();
    // InBody ignores NULL; CRLF preprocesses to LF and &amp; contributes &.
    assert!(
        matches!(text.kind(),crate::HtmlNodeKind::Text(value) if value.code_units()==[0xd83d,0xde00,0xd800,10,38])
    );
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_real_arena_alias_retains_failure_diagnostics_and_retries_cleanup() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let mut parts = completed.freeze_parts().unwrap();
    let pointer = parts.diagnostics().unwrap().records_pointer();
    let count = parts.diagnostics().unwrap().len();
    let alias = parts.document().unwrap().clone();
    assert!(parts.release().is_err());
    assert!(parts.arena.is_some());
    let first = parts.at_stop().unwrap().clone();
    assert!(parts.document().is_err());
    assert!(parts.cleanup_failure().is_err());
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    assert_eq!(parts.diagnostics().unwrap().len(), count);
    drop(alias);
    parts.cleanup_failure().unwrap();
    assert!(parts.arena.is_none());
    assert_eq!(parts.at_stop(), Some(&first));
    let after = parts.after_cleanup().unwrap().clone();
    assert_eq!(
        after.resource(HtmlParseResource::Diagnostics).current,
        count as u64
    );
    parts.release().unwrap();
    assert_eq!(parts.after_cleanup(), Some(&after));
    assert_eq!(parts.at_stop(), Some(&first));
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_native_projection_cancellation_partitions_only_committed_diagnostics() {
    let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;\0");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut completed = frozen_completed(&request);
    assert!(completed.native_associations().unwrap().len() > 0);
    let pointer = completed.diagnostics().unwrap().records_pointer();
    let count = completed.diagnostics().unwrap().len();
    assert_eq!(count, 3); // missing doctype, tokenizer null, and InBody null
    let mut parts = completed.freeze_parts().unwrap();
    assert!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current
            > count as u64
    );
    let before = parts.retained_usage().clone();
    {
        let diagnostics = parts.diagnostics().unwrap();
        assert_eq!(diagnostics.records_pointer(), pointer);
        let (record, location) = diagnostics.retained_record(2).unwrap();
        assert_eq!(
            record.kind,
            crate::construction::evidence::DiagnosticKind::Tree(
                super::super::head::HeadCondition::NullInBody
            )
        );
        assert_eq!(location.source, source.range(25, 26).unwrap());
    }
    assert_eq!(parts.retained_usage(), &before); // fixed lender performs no admitted work
    request.cancellation().cancel();
    assert!(matches!(
        parts.document(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    let first = parts.at_stop().unwrap().clone();
    let work = parts
        .retained_usage()
        .resource(HtmlParseResource::WorkCredits);
    let polls = parts.control.owned_span_observation().polls;
    parts.cleanup_failure().unwrap();
    assert!(parts.arena.is_none());
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    assert_eq!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        count as u64
    );
    assert_eq!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits),
        work
    );
    assert_eq!(parts.control.owned_span_observation().polls, polls);
    assert_eq!(parts.at_stop(), Some(&first));
    parts.release().unwrap();
    assert_eq!(parts.at_stop(), Some(&first));
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_native_projection_alias_refusal_keeps_partition_and_reclaims_it_once() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf16(crate::DocumentString::from_utf16([60, 112, 62, 65, 0])),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    assert!(completed.native_associations().unwrap().len() > 0);
    let mut parts = completed.freeze_parts().unwrap();
    let pointer = parts.diagnostics().unwrap().records_pointer();
    let count = parts.diagnostics().unwrap().len();
    assert_eq!(count, 3);
    let alias = parts.document().unwrap().clone();
    assert!(parts.release().is_err());
    let first = parts.at_stop().unwrap().clone();
    let work = parts
        .retained_usage()
        .resource(HtmlParseResource::WorkCredits);
    assert!(parts.cleanup_failure().is_err());
    assert!(parts.arena.is_some());
    assert!(parts.failed_evidence.is_some());
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    assert_eq!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        count as u64
    );
    let retained = parts.retained_usage().clone();
    assert!(parts.cleanup_failure().is_err());
    assert_eq!(parts.retained_usage(), &retained);
    drop(alias);
    parts.cleanup_failure().unwrap();
    assert_eq!(
        parts
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits),
        work
    );
    assert_eq!(parts.at_stop(), Some(&first));
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    let diagnostics = parts.diagnostics().unwrap();
    let (record, location) = diagnostics.retained_record(2).unwrap();
    assert_eq!(
        record.kind,
        crate::construction::evidence::DiagnosticKind::Tree(
            super::super::head::HeadCondition::NullInBody
        )
    );
    assert_eq!(location.source, request.source().range(4, 5).unwrap());
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_active_payload_alias_rejects_before_freeze_and_preserves_completed_owner() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>x"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let html = {
        let mut view = completed.native_view().unwrap();
        let root = view.root().unwrap();
        child_named(&mut view, &root, "html")
    };
    let driver = completed.driver.as_mut().unwrap();
    let alias = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .session
        .element_name(&mut driver.control, &html)
        .unwrap()
        .unwrap()
        .local_name()
        .clone();
    assert!(matches!(
        completed.freeze_parts(),
        Err(StandaloneError::Head(HeadError::Kernel(
            KernelError::Session(SessionError::AliasedPayload)
        )))
    ));
    assert!(completed.frozen.is_none());
    assert!(completed.driver.as_ref().unwrap().control.stop().is_none());
    assert!(matches!(
        completed.native_view(),
        Err(StandaloneError::Head(HeadError::Kernel(
            KernelError::Session(SessionError::AliasedPayload)
        )))
    ));
    drop(alias);
    completed.native_view().unwrap();
    let mut parts = completed.freeze_parts().unwrap();
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_missing_transferred_evidence_cannot_invent_an_empty_store() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let evidence = completed.take_diagnostics().unwrap();
    let pointer = evidence.records_pointer();
    let count = evidence.len();
    assert!(count > 0);
    assert!(matches!(
        completed.freeze_parts(),
        Err(StandaloneError::Head(HeadError::Kernel(
            KernelError::Session(SessionError::WrongToken)
        )))
    ));
    assert!(completed.frozen.is_none());
    assert!(completed.driver.as_ref().unwrap().control.stop().is_none());
    completed.native_view().unwrap();
    let driver = completed.driver.as_mut().unwrap();
    assert_eq!(evidence.records_pointer(), pointer);
    assert_eq!(evidence.len(), count);
    evidence.release(&mut driver.control).unwrap();
    completed.release().unwrap();
    assert!(completed.driver.is_none());
}
#[test]
fn frozen_parts_cancellation_after_transfer_keeps_safe_evidence_and_first_snapshot() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let mut parts = completed.freeze_parts().unwrap();
    request.cancellation().cancel();
    assert!(matches!(
        parts.document(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    let first = parts.at_stop().unwrap().clone();
    let pointer = parts.diagnostics().unwrap().records_pointer();
    let polls = parts.control.owned_span_observation().polls;
    parts.cleanup_failure().unwrap();
    assert_eq!(parts.control.owned_span_observation().polls, polls);
    assert_eq!(parts.at_stop(), Some(&first));
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_actual_after_share_cancellation_keeps_both_owners_for_terminal_release() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let pointer = completed.diagnostics().unwrap().records_pointer();
    completed.cancel_after_freeze = Some(request.cancellation().clone());
    assert!(matches!(
        completed.freeze_parts(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert!(completed.frozen.is_some());
    assert!(completed.driver.is_some());
    assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
    assert!(completed.native_view().is_err());
    assert!(completed.freeze_parts().is_err());
    let first = completed
        .driver
        .as_ref()
        .unwrap()
        .control
        .first_stop_usage()
        .unwrap()
        .clone();
    let polls = completed
        .driver
        .as_ref()
        .unwrap()
        .control
        .owned_span_observation()
        .polls;
    completed.cleanup_failure().unwrap();
    assert_eq!(completed.at_stop(), Some(&first));
    assert!(completed.after_cleanup().unwrap().actual_capacity_bytes < first.actual_capacity_bytes);
    assert_eq!(
        completed
            .driver
            .as_ref()
            .unwrap()
            .control
            .owned_span_observation()
            .polls,
        polls
    );
    assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
    assert!(completed.frozen.is_none());
    completed.release().unwrap();
    assert!(completed.driver.is_none());
    assert!(completed.frozen.is_none());
    assert!(first.actual_capacity_bytes > 0);
}
#[test]
fn frozen_parts_exact_work_and_one_short_plus_bounded_cuts_preserve_actual_owners() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut measured = frozen_completed(&request);
    let before = measured
        .usage()
        .unwrap()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let mut result = measured.freeze_parts().unwrap();
    let cost = result
        .final_usage()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    result.release().unwrap();
    assert!(cost > 0);
    let mut active_stops = 0;
    let mut frozen_stops = 0;
    for allowance in (0..cost).step_by(97).chain([cost - 1, cost]) {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let pointer = completed.diagnostics().unwrap().records_pointer();
        let count = completed.diagnostics().unwrap().len();
        let driver = completed.driver.as_mut().unwrap();
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - allowance,
            )
            .unwrap();
        match completed.freeze_parts() {
            Ok(mut parts) => {
                assert_eq!(allowance, cost, "only exact complete allowance succeeds");
                assert_eq!(
                    parts
                        .final_usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    request.limits().work_credits()
                );
                if parts.release().is_err() {
                    parts.release().unwrap();
                }
                super::super::tests::assert_released(&parts.control);
            }
            Err(error) => {
                assert!(allowance < cost);
                let driver = completed.driver.as_ref().unwrap();
                assert!(
                    matches!(
                        driver.control.stop(),
                        Some(ParseStop::LimitExceeded {
                            resource: HtmlParseResource::WorkCredits,
                            ..
                        })
                    ),
                    "{error:?}"
                );
                let stop = driver.control.stop();
                let first = driver.control.first_stop_usage().unwrap().clone();
                let polls = driver.control.owned_span_observation().polls;
                if completed.frozen.is_some() {
                    frozen_stops += 1;
                } else {
                    active_stops += 1;
                }
                assert!(completed.freeze_parts().is_err());
                completed.cleanup_failure().unwrap();
                assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
                assert_eq!(completed.diagnostics().unwrap().len(), count);
                assert_eq!(completed.driver.as_ref().unwrap().control.stop(), stop);
                assert_eq!(completed.at_stop(), Some(&first));
                assert_eq!(
                    completed
                        .driver
                        .as_ref()
                        .unwrap()
                        .control
                        .owned_span_observation()
                        .polls,
                    polls
                );
                assert!(
                    completed.after_cleanup().unwrap().actual_capacity_bytes
                        <= first.actual_capacity_bytes
                );
                completed.release().unwrap();
                assert!(completed.driver.is_none());
            }
        }
    }
    assert!(active_stops > 0);
    assert!(frozen_stops > 0);
}
#[test]
fn frozen_parts_retains_template_control_and_script_metadata_in_same_arena() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><template><p>T</template><script>x</script><textarea>\nX</textarea><input type=hidden value=y><select><option>A</select>",
        ),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let script = {
        let mut view = completed.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let head = child_named(&mut view, &html, "head");
        child_named(&mut view, &head, "script")
    };
    let mut parts = completed.freeze_parts().unwrap();
    let document = parts.document().unwrap();
    let html = native_child(document, document.root(), "html");
    let head = native_child(document, html, "head");
    let template = native_child(document, head, "template");
    let contents = document
        .fragment(&template.template_contents().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(contents.root().children().len(), 1);
    let body = native_child(document, html, "body");
    let textarea = native_child(document, body, "textarea").id();
    let hidden = native_child(document, body, "input").id();
    drop(contents);
    let arena = parts.arena.as_ref().unwrap();
    assert!(
        arena
            .script_metadata(&mut parts.control, &script)
            .unwrap()
            .flags
            .already_started()
    );
    assert_eq!(
        arena
            .textarea_raw_value(&mut parts.control, &textarea)
            .unwrap()
            .code_units(),
        [88]
    );
    arena.input_state(&mut parts.control, &hidden).unwrap();
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}
#[test]
fn frozen_parts_stopped_evidence_foreign_owner_refusal_preserves_actual_store() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>\0"),
        super::super::tests::limits(100),
    );
    let mut completed = frozen_completed(&request);
    let mut parts = completed.freeze_parts().unwrap();
    let pointer = parts.diagnostics().unwrap().records_pointer();
    let mut foreign = ParseControl::new(
        super::super::tests::limits(100),
        crate::HtmlParseCancellation::new(),
    );
    foreign.admit_source(request.source()).unwrap();
    foreign.stop_unsupported_standalone(
        super::super::super::control::StandaloneUnsupported::Completion,
    );
    let before = foreign.usage().clone();
    assert!(
        parts
            .arena
            .as_mut()
            .unwrap()
            .take_stopped_evidence(&mut foreign)
            .is_err()
    );
    assert_eq!(foreign.usage(), &before);
    assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
    assert!(parts.control.stop().is_none());
    parts.release().unwrap();
    super::super::tests::assert_released(&parts.control);
}

#[test]
fn completed_native_field_projection_retains_atomic_merged_origins_and_later_trigger() {
    use crate::construction::provenance::ConstructionOperation;
    use crate::parser::tokenizer::token::OriginKind;
    let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut driver = super::super::entry::OrdinaryDocumentEntry::new(&request)
        .take_driver()
        .unwrap();
    finish(&mut driver);
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    let (text, root) = {
        let mut view = completed.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        let p = child_named(&mut view, &body, "p");
        let text = view.child_at(&p, 0).unwrap().unwrap();
        let value = match view.kind(&text).unwrap() {
            NativeKind::Text(value) => value,
            _ => panic!("native Text"),
        };
        assert_eq!(
            units(&mut view, value),
            [65, 0xd83d, 0xde42, 10, 0x2242, 0x338]
        );
        (text, root)
    };
    let mut rows = completed.native_associations().unwrap();
    let mut observed = Vec::new();
    let mut roots = 0;
    for index in 0..rows.len() {
        if rows.node_for(index, &root).unwrap() {
            let (_, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Request);
            assert!(cause.source.is_none());
            assert!(cause.captured.is_none());
            assert!(cause.original_node.is_none());
            roots += 1;
        }
        if rows.text_for(index, &text).unwrap() {
            let reference = rows.cause_reference(index).unwrap().unwrap();
            let cause = rows.cause(&reference).unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Merge);
            assert!(cause.original.is_some());
            assert!(cause.incoming.is_some());
            assert!(cause.trigger.is_some());
            assert!(cause.source.is_none());
            assert!(cause.rule.is_none());
            assert!(cause.family.is_none());
            let original = cause.original.unwrap();
            let trigger = cause.trigger.unwrap();
            let retirement = rows
                .group(original)
                .unwrap()
                .unwrap()
                .retirement
                .expect("actual replaced field retained");
            let mut origins = rows.group_origin_cursor(original).unwrap();
            assert!(rows.next_group_origin(&mut origins).unwrap().is_some());
            while rows.next_group_origin(&mut origins).unwrap().is_some() {}
            rows.release_group_origin_cursor(&mut Some(origins))
                .unwrap();
            let historical = rows.materialize_group(original).unwrap();
            assert!(!historical.text().is_empty());
            rows.release_materialized_group(&mut Some(historical))
                .unwrap();
            let retired = rows.cause_at(retirement).unwrap().unwrap();
            assert_eq!(retired.operation, ConstructionOperation::Retirement);
            assert_eq!(retired.original, Some(original));
            assert!(matches!(
                retired.retired_destination,
                Some(crate::construction::StringLocation::Text(_))
            ));
            assert!(retired.source.is_none());
            assert_eq!(retired.trigger, Some(trigger));
            let trigger = rows.cause_at(trigger).unwrap().unwrap();
            assert_eq!(trigger.operation, ConstructionOperation::Token);
            assert_eq!(trigger.source, Some(source.range(10, 25).unwrap()));
            let (row, _) = rows.row(index).unwrap().unwrap();
            observed.push((row.start, row.end, row.source.clone(), row.kind));
        }
    }
    assert_eq!(
        observed,
        [
            (
                0,
                1,
                Some(source.range(3, 4).unwrap()),
                Some(OriginKind::Literal)
            ),
            (
                1,
                3,
                Some(source.range(4, 8).unwrap()),
                Some(OriginKind::Literal)
            ),
            (
                3,
                4,
                Some(source.range(8, 10).unwrap()),
                Some(OriginKind::Newline)
            ),
            (
                4,
                5,
                Some(source.range(10, 25).unwrap()),
                Some(OriginKind::CharacterReference)
            ),
            (
                5,
                6,
                Some(source.range(10, 25).unwrap()),
                Some(OriginKind::CharacterReference)
            ),
        ]
    );
    assert_eq!(roots, 1);
    let diagnostics = completed.take_diagnostics().unwrap();
    completed.prepare_native_associations().unwrap();
    diagnostics
        .release(&mut completed.driver.as_mut().unwrap().control)
        .unwrap();
    drop(text);
    drop(root);
    completed.release().unwrap();
}

fn units(view: &mut NativeView<'_>, text: NativeString<'_>) -> Vec<u16> {
    (0..text.len())
        .map(|i| view.unit(&text, i).unwrap().unwrap())
        .collect()
}
#[test]
fn completed_intrinsic_raw_fields_keep_utf16_child_and_hidden_sanitization_relations() {
    use crate::construction::provenance::{ConstructionOperation, RawFieldKind};
    use crate::parser::tokenizer::token::OriginKind;
    for (input, local, kind, expected, expected_maps) in [
        (
            "<textarea>A🙂\r\n&NotEqualTilde;</textarea>",
            "textarea",
            RawFieldKind::Textarea,
            vec![65, 0xd83d, 0xde42, 10, 0x2242, 0x338],
            vec![
                (0, 1, 10, 11, OriginKind::Literal),
                (1, 3, 11, 13, OriginKind::Literal),
                (3, 4, 13, 15, OriginKind::Newline),
                (4, 5, 15, 30, OriginKind::CharacterReference),
                (5, 6, 15, 30, OriginKind::CharacterReference),
            ],
        ),
        (
            "<input value='A&#13;B&#10;C' type=hidden>",
            "input",
            RawFieldKind::Input,
            vec![65, 66, 67],
            vec![
                (0, 1, 14, 15, OriginKind::Literal),
                (1, 2, 20, 21, OriginKind::Literal),
                (2, 3, 26, 27, OriginKind::Literal),
            ],
        ),
        (
            "<input value='A&#13;B&#10;C' type=password>",
            "input",
            RawFieldKind::Input,
            vec![65, 66, 67],
            vec![
                (0, 1, 14, 15, OriginKind::Literal),
                (1, 2, 20, 21, OriginKind::Literal),
                (2, 3, 26, 27, OriginKind::Literal),
            ],
        ),
    ] {
        let source = HtmlSource::utf16(crate::DocumentString::from_utf16(
            input.encode_utf16().collect::<Vec<_>>(),
        ));
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let node = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            child_named(&mut view, &body, local)
        };
        {
            let driver = owner.driver.as_mut().unwrap();
            let session = &driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .session;
            match kind {
                RawFieldKind::Nonce => assert_eq!(
                    session
                        .cryptographic_nonce(&mut driver.control, &node)
                        .unwrap()
                        .code_units(),
                    expected
                ),
                RawFieldKind::Textarea => assert_eq!(
                    session
                        .textarea_raw_value(&mut driver.control, &node)
                        .unwrap()
                        .code_units(),
                    expected
                ),
                RawFieldKind::Input => {
                    let mut raw =
                        crate::parser::buffer::Utf16Buffer::new(&mut driver.control).unwrap();
                    session
                        .append_input_internal_value(&mut driver.control, &node, &mut raw)
                        .unwrap();
                    assert_eq!(raw.units(), expected);
                    raw.release(&mut driver.control).unwrap();
                }
            }
        }
        let mut rows = owner.native_associations().unwrap();
        let mut actual = Vec::new();
        for index in 0..rows.len() {
            if rows.raw_for(index, &node, kind).unwrap() {
                let (row, cause) = rows.row(index).unwrap().unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Replacement);
                assert!(cause.original.is_some());
                assert!(cause.trigger.is_some());
                actual.push((
                    row.start,
                    row.end,
                    row.source.as_ref().unwrap().start(),
                    row.source.as_ref().unwrap().end(),
                    row.kind.unwrap(),
                ));
            }
        }
        assert_eq!(actual, expected_maps);
        drop(node);
        owner.release().unwrap();
    }
}
#[test]
fn completed_formatting_reconstruction_distinguishes_original_creation_and_later_trigger() {
    use crate::construction::provenance::ConstructionOperation;
    let source = HtmlSource::utf8("<p><b>A</p>B");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    let (replay, body) = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        let b = view.child_at(&body, 1).unwrap().unwrap();
        assert!(matches!(view.kind(&b).unwrap(), NativeKind::Element(_)));
        let text = view.child_at(&b, 0).unwrap().unwrap();
        let NativeKind::Text(value) = view.kind(&text).unwrap() else {
            panic!("reconstructed B")
        };
        assert_eq!(units(&mut view, value), [66]);
        (b, body)
    };
    let mut rows = owner.native_associations().unwrap();
    let mut intended = None;
    for index in 0..rows.len() {
        if rows.node_for(index, &body).unwrap() {
            let (row, _) = rows.row(index).unwrap().unwrap();
            let crate::construction::provenance::NativeDestination::Node(node) = row.destination
            else {
                unreachable!()
            };
            intended = Some(node);
        }
    }
    assert!(intended.is_some());
    let mut seen = 0;
    for index in 0..rows.len() {
        if rows.node_for(index, &replay).unwrap() {
            let (_, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Replay);
            assert_eq!(cause.intended_parent, intended);
            let (original, trigger) = (cause.original_cause.unwrap(), cause.trigger.unwrap());
            assert_eq!(
                rows.cause_at(original).unwrap().unwrap().source,
                Some(source.range(3, 6).unwrap())
            );
            assert_eq!(
                rows.cause_at(trigger).unwrap().unwrap().source,
                Some(source.range(11, 12).unwrap())
            );
            seen += 1;
        }
    }
    assert_eq!(seen, 1);
    drop((replay, body));
    owner.release().unwrap();
}
#[test]
fn completed_adoption_replay_keeps_furthest_block_parent_and_original_moved_text() {
    use crate::construction::provenance::{ConstructionOperation, NativeDestination};
    use crate::parser::tokenizer::token::OriginKind;
    let source = HtmlSource::utf8("<b>1<p>2</b>3</p>");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    let (original, replay, parent, moved) = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        assert_eq!(view.child_count(&body).unwrap(), 2);
        let original = view.child_at(&body, 0).unwrap().unwrap();
        let parent = view.child_at(&body, 1).unwrap().unwrap();
        let replay = view.child_at(&parent, 0).unwrap().unwrap();
        let moved = view.child_at(&replay, 0).unwrap().unwrap();
        let NativeKind::Text(value) = view.kind(&moved).unwrap() else {
            panic!("actual adopted text")
        };
        assert_eq!(units(&mut view, value), [50]);
        let tail = view.child_at(&parent, 1).unwrap().unwrap();
        let NativeKind::Text(value) = view.kind(&tail).unwrap() else {
            panic!("actual tail")
        };
        assert_eq!(units(&mut view, value), [51]);
        (original, replay, parent, moved)
    };
    let mut rows = owner.native_associations().unwrap();
    let mut intended = None;
    let mut original_index = None;
    for index in 0..rows.len() {
        if rows.node_for(index, &parent).unwrap() {
            let (row, _) = rows.row(index).unwrap().unwrap();
            let NativeDestination::Node(node) = row.destination else {
                unreachable!()
            };
            intended = Some(node);
        }
        if rows.node_for(index, &original).unwrap() {
            let (row, _) = rows.row(index).unwrap().unwrap();
            let NativeDestination::Node(node) = row.destination else {
                unreachable!()
            };
            original_index = Some(node);
        }
    }
    assert!(intended.is_some());
    let mut seen = 0;
    let mut moved_maps = Vec::new();
    for index in 0..rows.len() {
        if rows.node_for(index, &replay).unwrap() {
            let (row, cause) = rows.row(index).unwrap().unwrap();
            let NativeDestination::Node(node) = row.destination else {
                unreachable!()
            };
            assert_ne!(Some(node), original_index);
            assert_eq!(cause.operation, ConstructionOperation::Replay);
            assert_eq!(cause.intended_parent, intended);
            let (creation, trigger) = (cause.original_cause.unwrap(), cause.trigger.unwrap());
            assert_eq!(
                rows.cause_at(creation).unwrap().unwrap().source,
                Some(source.range(0, 3).unwrap())
            );
            assert_eq!(
                rows.cause_at(trigger).unwrap().unwrap().source,
                Some(source.range(8, 12).unwrap())
            );
            seen += 1;
        }
        if rows.text_for(index, &moved).unwrap() {
            let (row, _) = rows.row(index).unwrap().unwrap();
            moved_maps.push((row.start, row.end, row.source.clone(), row.kind));
        }
    }
    assert_eq!(seen, 1);
    assert_eq!(
        moved_maps,
        [(
            0,
            1,
            Some(source.range(7, 8).unwrap()),
            Some(OriginKind::Literal)
        )]
    );
    drop((original, replay, parent, moved));
    owner.release().unwrap();
}
#[test]
fn native_projection_exact_and_one_short_admission_retains_only_safe_failed_diagnostics() {
    let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut successful = completed(&request);
    let before = successful.usage().unwrap().clone();
    successful.prepare_native_associations().unwrap();
    let after = successful.usage().unwrap().clone();
    assert!(
        after
            .resource(HtmlParseResource::SourceMapEntries)
            .high_water
            > before
                .resource(HtmlParseResource::SourceMapEntries)
                .high_water
    );
    let cost = after.resource(HtmlParseResource::WorkCredits).current;
    let map_peak = after
        .resource(HtmlParseResource::SourceMapEntries)
        .high_water;
    successful.release().unwrap();
    for resource in [
        HtmlParseResource::WorkCredits,
        HtmlParseResource::SourceMapEntries,
    ] {
        for exact in [false, true] {
            let cap = if resource == HtmlParseResource::WorkCredits {
                cost
            } else {
                map_peak
            };
            let limits = super::super::tests::limits(100).into_builder();
            let limits = if resource == HtmlParseResource::WorkCredits {
                limits.work_credits(cap - u64::from(!exact))
            } else {
                limits.source_map_entries(cap - u64::from(!exact))
            }
            .finish()
            .unwrap();
            let request = HtmlDocumentParseRequest::offline(source.clone(), limits);
            let mut owner = completed(&request);
            let pointer = owner.diagnostics().unwrap().records_pointer();
            if exact {
                owner.prepare_native_associations().unwrap();
                assert_eq!(
                    owner
                        .usage()
                        .unwrap()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current,
                    after.resource(HtmlParseResource::SourceMapEntries).current
                );
                if owner.release().is_err() {
                    owner.release().unwrap();
                }
            } else {
                assert!(owner.prepare_native_associations().is_err());
                let stop = owner.driver.as_ref().unwrap().control.stop().unwrap();
                assert!(
                    matches!(stop, ParseStop::LimitExceeded { resource: stopped, .. } if stopped == resource)
                );
                assert!(owner.native_associations().is_err());
                assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
                let mut input = owner.driver.take();
                let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
                assert_eq!(failed.stop(), stop);
                assert_eq!(failed.diagnostics().unwrap().records_pointer(), pointer);
                // ManagedNodes is a permanent cumulative admission count,
                // not retained storage and is deliberately never refunded.
                assert_eq!(
                    failed
                        .after_cleanup()
                        .resource(HtmlParseResource::ManagedNodes)
                        .current,
                    failed
                        .at_stop()
                        .resource(HtmlParseResource::ManagedNodes)
                        .current
                );
                assert_eq!(
                    failed
                        .after_cleanup()
                        .resource(HtmlParseResource::BufferedUtf16Units)
                        .current,
                    0
                );
                assert_eq!(
                    failed
                        .after_cleanup()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    failed
                        .at_stop()
                        .resource(HtmlParseResource::WorkCredits)
                        .current
                );
                assert!(
                    failed
                        .after_cleanup()
                        .resource(HtmlParseResource::SourceMapEntries)
                        .current
                        < failed
                            .at_stop()
                            .resource(HtmlParseResource::SourceMapEntries)
                            .current
                );
                assert!(failed.retained_usage().actual_capacity_bytes > 0);
                failed.release().unwrap();
                super::super::tests::assert_released(&failed.control);
            }
        }
    }
}
#[test]
fn native_projection_cancel_and_alias_refusal_preserve_the_whole_owner_and_first_stop() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>A"),
        super::super::tests::limits(100),
    );
    let mut owner = completed(&request);
    let html = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        child_named(&mut view, &root, "html")
    };
    let alias = {
        let driver = owner.driver.as_mut().unwrap();
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        state
            .before
            .kernel
            .session
            .element_name(&mut driver.control, &html)
            .unwrap()
            .unwrap()
            .local_name()
            .clone()
    };
    assert!(owner.prepare_native_associations().is_err());
    assert!(owner.driver.as_ref().unwrap().control.stop().is_none());
    drop(alias);
    drop(html);
    owner.prepare_native_associations().unwrap();
    request.cancellation().cancel();
    assert!(owner.native_associations().is_err());
    let stop = owner.driver.as_ref().unwrap().control.stop().unwrap();
    assert_eq!(stop, ParseStop::Cancelled);
    assert!(owner.native_view().is_err());
    assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
    let mut input = owner.driver.take();
    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
    assert_eq!(failed.stop(), stop);
    failed.release().unwrap();
    super::super::tests::assert_released(&failed.control);
}
#[test]
fn completed_selectedcontent_clone_retains_original_native_identity_and_scheduling_trigger() {
    use crate::construction::provenance::{ConstructionOperation, NativeDestination};
    let input = "<select><button><selectedcontent></selectedcontent></button><option selected>A</option></select>";
    let source = HtmlSource::utf8(input);
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    let (original, copy) = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        let select = child_named(&mut view, &body, "select");
        let button = child_named(&mut view, &select, "button");
        let selected = child_named(&mut view, &button, "selectedcontent");
        let option = child_named(&mut view, &select, "option");
        let original = view.child_at(&option, 0).unwrap().unwrap();
        let copy = view.child_at(&selected, 0).unwrap().unwrap();
        for node in [&original, &copy] {
            let NativeKind::Text(value) = view.kind(node).unwrap() else {
                panic!("actual cloned A")
            };
            assert_eq!(units(&mut view, value), [65]);
        }
        (original, copy)
    };
    let mut rows = owner.native_associations().unwrap();
    let mut original_identity = None;
    for index in 0..rows.len() {
        if rows.node_for(index, &original).unwrap() {
            let (row, _) = rows.row(index).unwrap().unwrap();
            let NativeDestination::Node(node) = row.destination else {
                panic!("qualified original node")
            };
            original_identity = Some(node);
        }
    }
    let mut nodes = 0;
    let mut fields = 0;
    for index in 0..rows.len() {
        if rows.node_for(index, &copy).unwrap() {
            let (_, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Copy);
            assert_eq!(cause.original_node, original_identity);
            let trigger = cause.trigger.unwrap();
            let trigger = rows.cause_at(trigger).unwrap().unwrap();
            let start = input.find("</option>").unwrap();
            assert_eq!(
                trigger.source,
                Some(source.range(start, start + 9).unwrap())
            );
            nodes += 1;
        }
        if rows.text_for(index, &copy).unwrap() {
            let (row, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Copy);
            assert!(cause.original.is_some());
            let start = input.find(">A<").unwrap() + 1;
            assert_eq!(row.source, Some(source.range(start, start + 1).unwrap()));
            assert_eq!((row.start, row.end), (0, 1));
            fields += 1;
        }
    }
    assert_eq!((nodes, fields), (1, 1));
    drop((original, copy));
    owner.release().unwrap();
}
#[test]
fn completed_template_foreign_and_rewritten_fields_keep_exact_native_bindings() {
    use crate::construction::{
        StringLocation,
        provenance::{ConstructionOperation, NativeDestination},
    };
    use crate::parser::tokenizer::token::OriginKind;
    let input = "<template><svg><lineargradient></lineargradient></svg></template><image><br></br>";
    let source = HtmlSource::utf8(input);
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    let nodes = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let head = child_named(&mut view, &html, "head");
        let template = child_named(&mut view, &head, "template");
        let contents = view.template_contents(&template).unwrap().unwrap();
        assert!(view.parent(&contents).unwrap().is_none());
        let svg = child_named(&mut view, &contents, "svg");
        let adjusted = child_named(&mut view, &svg, "linearGradient");
        let NativeKind::Element(name) = view.kind(&adjusted).unwrap() else {
            panic!("adjusted SVG")
        };
        assert_eq!(
            units(&mut view, name.local),
            "linearGradient".encode_utf16().collect::<Vec<_>>()
        );
        let body = child_named(&mut view, &html, "body");
        let image = child_named(&mut view, &body, "img");
        let end_br = view.child_at(&body, 2).unwrap().unwrap();
        let NativeKind::Element(name) = view.kind(&end_br).unwrap() else {
            panic!("recovered end br")
        };
        assert_eq!(units(&mut view, name.local), [98, 114]);
        [adjusted, image, end_br]
    };
    let mut rows = owner.native_associations().unwrap();
    let mut identities = [None; 3];
    for index in 0..rows.len() {
        for (position, node) in nodes.iter().enumerate() {
            if rows.node_for(index, node).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                let NativeDestination::Node(id) = row.destination else {
                    unreachable!()
                };
                identities[position] = Some(id);
            }
        }
    }
    assert!(identities.iter().all(Option::is_some));
    let mut names = [Vec::new(), Vec::new(), Vec::new()];
    let mut source_less_svg_namespace = false;
    for index in 0..rows.len() {
        let (row, cause) = rows.row(index).unwrap().unwrap();
        match row.destination {
            NativeDestination::Field(StringLocation::ElementName(node)) => {
                for (position, id) in identities.iter().enumerate() {
                    if Some(node) == *id {
                        if position == 0 {
                            assert_eq!(cause.operation, ConstructionOperation::Replacement);
                            assert!(cause.original.is_some());
                        }
                        names[position].push((row.start, row.end, row.source.clone(), row.kind));
                    }
                }
            }
            NativeDestination::Field(StringLocation::Namespace(node))
                if Some(node) == identities[0] =>
            {
                assert!(row.source.is_none());
                assert_eq!((row.start, row.end), (0, 26));
                source_less_svg_namespace = true;
            }
            _ => {}
        }
    }
    let expected = (0..14)
        .map(|offset| {
            (
                offset,
                offset + 1,
                Some(source.range(16 + offset, 17 + offset).unwrap()),
                Some(OriginKind::RecoverySyntax),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(names[0], expected);
    assert_eq!(
        names[1],
        [(
            0,
            3,
            Some(source.range(66, 71).unwrap()),
            Some(OriginKind::RecoverySyntax)
        )]
    );
    assert_eq!(
        names[2],
        [
            (
                0,
                1,
                Some(source.range(78, 79).unwrap()),
                Some(OriginKind::Literal)
            ),
            (
                1,
                2,
                Some(source.range(79, 80).unwrap()),
                Some(OriginKind::Literal)
            )
        ]
    );
    assert!(source_less_svg_namespace);
    drop(nodes);
    owner.release().unwrap();
}
#[test]
fn completed_fostered_table_text_keeps_original_processing_and_later_flush_trigger() {
    use crate::construction::provenance::{ConstructionOperation, ParserRule};
    use crate::parser::tokenizer::token::OriginKind;
    use crate::parser::tree_builder::head::InsertionMode;
    let source = HtmlSource::utf8("<table>AB<tr><td>Z</td></tr></table>");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    let text = {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        let text = view.child_at(&body, 0).unwrap().unwrap();
        let NativeKind::Text(value) = view.kind(&text).unwrap() else {
            panic!("actual fostered Text before table")
        };
        assert_eq!(units(&mut view, value), [65, 66]);
        let table = view.child_at(&body, 1).unwrap().unwrap();
        assert!(matches!(view.kind(&table).unwrap(), NativeKind::Element(_)));
        text
    };
    let mut rows = owner.native_associations().unwrap();
    let mut observed = Vec::new();
    for index in 0..rows.len() {
        if !rows.text_for(index, &text).unwrap() {
            continue;
        }
        let (row, cause) = rows.row(index).unwrap().unwrap();
        assert_eq!(cause.operation, ConstructionOperation::Merge);
        let groups = [cause.original.unwrap(), cause.incoming.unwrap()];
        observed.push((row.start, row.end, row.source.clone(), row.kind));
        for (group, expected) in groups.into_iter().zip([(7, 8), (8, 9)]) {
            let group = rows.group(group).unwrap().unwrap();
            let replay = group.cause;
            let replay = rows.cause_at(replay).unwrap().unwrap();
            assert_eq!(replay.operation, ConstructionOperation::Replay);
            let (original, trigger) = (replay.original_cause.unwrap(), replay.trigger.unwrap());
            let original = rows.cause_at(original).unwrap().unwrap();
            assert_eq!(original.operation, ConstructionOperation::Token);
            assert_eq!(
                original.source,
                Some(source.range(expected.0, expected.1).unwrap())
            );
            assert_eq!(
                original.rule,
                Some(ParserRule::Tree(InsertionMode::InTableText))
            );
            let trigger = rows.cause_at(trigger).unwrap().unwrap();
            assert_eq!(trigger.source, Some(source.range(9, 13).unwrap()));
        }
    }
    assert_eq!(
        observed,
        [
            (
                0,
                1,
                Some(source.range(7, 8).unwrap()),
                Some(OriginKind::Literal)
            ),
            (
                1,
                2,
                Some(source.range(8, 9).unwrap()),
                Some(OriginKind::Literal)
            ),
        ]
    );
    drop(text);
    owner.release().unwrap();
}
#[test]
fn completed_pi_pseudo_attribute_derivation_keeps_literal_and_xml_reference_groups() {
    use crate::construction::{
        StringLocation,
        provenance::{ConstructionOperation, NativeDestination},
    };
    use crate::parser::tokenizer::token::OriginKind;
    let source = HtmlSource::utf8("<body><?marker name='x&amp;y'?>");
    let request =
        HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
    let mut owner = completed(&request);
    {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        let html = child_named(&mut view, &root, "html");
        let body = child_named(&mut view, &html, "body");
        let pi = view.child_at(&body, 0).unwrap().unwrap();
        let NativeKind::ProcessingInstruction { target, data } = view.kind(&pi).unwrap() else {
            panic!("actual PI")
        };
        assert_eq!(
            units(&mut view, target),
            "marker".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            units(&mut view, data),
            "name='x&amp;y'".encode_utf16().collect::<Vec<_>>()
        );
    }
    let mut rows = owner.native_associations().unwrap();
    let mut values = Vec::new();
    let mut names = 0;
    for index in 0..rows.len() {
        let (row, cause) = rows.row(index).unwrap().unwrap();
        match row.destination {
            NativeDestination::Field(StringLocation::PiName(_, 0)) => {
                assert_eq!(cause.operation, ConstructionOperation::Replacement);
                assert!(cause.original.is_some());
                names += 1;
            }
            NativeDestination::Field(StringLocation::PiValue(_, 0)) => {
                assert_eq!(cause.operation, ConstructionOperation::Replacement);
                assert!(cause.original.is_some());
                assert!(cause.trigger.is_some());
                values.push((row.start, row.end, row.source.clone(), row.kind));
            }
            _ => {}
        }
    }
    assert_eq!(names, 4);
    assert_eq!(
        values,
        [
            (
                0,
                1,
                Some(source.range(21, 22).unwrap()),
                Some(OriginKind::Literal)
            ),
            (
                1,
                2,
                Some(source.range(22, 27).unwrap()),
                Some(OriginKind::RecoverySyntax)
            ),
            (
                2,
                3,
                Some(source.range(27, 28).unwrap()),
                Some(OriginKind::Literal)
            ),
        ]
    );
    owner.release().unwrap();
}
fn child_named(view: &mut NativeView<'_>, parent: &ParserNode, local: &str) -> ParserNode {
    for i in 0..view.child_count(parent).unwrap() {
        let child = view.child_at(parent, i).unwrap().unwrap();
        if let NativeKind::Element(name) = view.kind(&child).unwrap()
            && units(view, name.local) == local.encode_utf16().collect::<Vec<_>>()
        {
            return child;
        }
    }
    panic!("literal native child {local} missing");
}
fn completed<'a>(request: &'a HtmlDocumentParseRequest) -> CompletedStandaloneDocument<'a> {
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(request);
    finish(&mut driver);
    StandaloneDocumentDriver::take_completed(&mut Some(driver)).unwrap()
}

#[test]
fn completed_native_view_retains_exact_payloads_and_template_backlinks() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!DOCTYPE html SYSTEM 'about:legacy-compat'><?probe data><html lang='en'><head><template id='t'><svg><a xlink:href='x'>A&#x1F600;</a><!--C--></svg></template></head><body><p>B</p></body></html>",
        ),
        super::super::tests::limits(100),
    );
    let mut owner = completed(&request);
    {
        let mut view = owner.native_view().unwrap();
        let root = view.root().unwrap();
        assert!(matches!(
            view.kind(&root).unwrap(),
            NativeKind::Document(crate::HtmlDocumentMode::NoQuirks)
        ));
        assert!(view.parent(&root).unwrap().is_none());
        assert_eq!(view.child_count(&root).unwrap(), 3);
        let doctype = view.child_at(&root, 0).unwrap().unwrap();
        let NativeKind::Doctype {
            name,
            public_id,
            system_id,
        } = view.kind(&doctype).unwrap()
        else {
            panic!("doctype")
        };
        assert_eq!(
            units(&mut view, name),
            "html".encode_utf16().collect::<Vec<_>>()
        );
        assert!(units(&mut view, public_id.unwrap()).is_empty());
        assert_eq!(
            units(&mut view, system_id.unwrap()),
            "about:legacy-compat".encode_utf16().collect::<Vec<_>>()
        );
        let pi = view.child_at(&root, 1).unwrap().unwrap();
        let NativeKind::ProcessingInstruction { target, data } = view.kind(&pi).unwrap() else {
            panic!("PI")
        };
        assert_eq!(
            units(&mut view, target),
            "probe".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            units(&mut view, data),
            "data".encode_utf16().collect::<Vec<_>>()
        );
        let html = child_named(&mut view, &root, "html");
        let attr = view.attribute_at(&html, 0).unwrap().unwrap();
        assert_eq!(view.attribute_count(&html).unwrap(), 1);
        assert!(attr.name.namespace.is_none());
        assert!(attr.name.prefix.is_none());
        assert_eq!(
            units(&mut view, attr.name.local),
            "lang".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(units(&mut view, attr.value), vec![101, 110]);
        assert!(view.attribute_at(&html, 1).unwrap().is_none());
        let parent = view.parent(&html).unwrap().unwrap();
        assert!(view.same_node(&parent, &root).unwrap());
        assert!(view.shadow(&html).unwrap().is_none());
        let head = child_named(&mut view, &html, "head");
        let template = child_named(&mut view, &head, "template");
        assert_eq!(view.child_count(&template).unwrap(), 0);
        let contents = view.template_contents(&template).unwrap().unwrap();
        assert!(matches!(
            view.kind(&contents).unwrap(),
            NativeKind::Fragment
        ));
        assert!(view.parent(&contents).unwrap().is_none());
        let Some(NativeFragmentOwner::Template(backlink)) = view.fragment_owner(&contents).unwrap()
        else {
            panic!("template owner")
        };
        assert!(view.same_node(&backlink, &template).unwrap());
        let svg = child_named(&mut view, &contents, "svg");
        let a = child_named(&mut view, &svg, "a");
        let NativeKind::Element(name) = view.kind(&a).unwrap() else {
            panic!("element")
        };
        assert_eq!(
            units(&mut view, name.namespace.unwrap()),
            "http://www.w3.org/2000/svg"
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        let href = view.attribute_at(&a, 0).unwrap().unwrap();
        assert_eq!(
            units(&mut view, href.name.namespace.unwrap()),
            "http://www.w3.org/1999/xlink"
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            units(&mut view, href.name.prefix.unwrap()),
            "xlink".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            units(&mut view, href.name.local),
            "href".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(units(&mut view, href.value), vec![120]);
        let text_node = view.child_at(&a, 0).unwrap().unwrap();
        let NativeKind::Text(text) = view.kind(&text_node).unwrap() else {
            panic!("text")
        };
        assert_eq!(units(&mut view, text), vec![65, 0xd83d, 0xde00]);
        let comment_node = view.child_at(&svg, 1).unwrap().unwrap();
        let NativeKind::Comment(comment) = view.kind(&comment_node).unwrap() else {
            panic!("comment")
        };
        assert_eq!(units(&mut view, comment), vec![67]);
        assert!(view.child_at(&svg, 2).unwrap().is_none());
        let body = child_named(&mut view, &html, "body");
        let p = child_named(&mut view, &body, "p");
        assert!(view.template_contents(&p).unwrap().is_none());
        assert!(view.fragment_owner(&p).unwrap().is_none());
        assert!(matches!(
            view.attribute_count(&root),
            Err(SessionError::WrongToken)
        ));
    }
    assert_eq!(
        owner.document_state().unwrap().completion,
        crate::construction::ConstructionCompletion::Complete
    );
    owner.release().unwrap();
    assert!(matches!(
        owner.native_view(),
        Err(StandaloneError::Consumed)
    ));
}

#[test]
fn completed_native_units_are_exact_and_admitted_before_each_read() {
    // The work one unit read needs is measured from an unconstrained read.
    // The same read is then given exactly that much spare work and one credit
    // less, so admission before the read is a boundary property rather than a
    // count of private record visits.
    let mut needed = None;
    for attempt in 0..3 {
        let mut input = "<!doctype html><p>".encode_utf16().collect::<Vec<_>>();
        input.extend([0xd800, 0, 0xdc00, 13, 10]);
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(crate::DocumentString::from_utf16(input)),
            super::super::tests::limits(100),
        );
        let mut owner = completed(&request);
        {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let p = child_named(&mut view, &body, "p");
            let text_node = view.child_at(&p, 0).unwrap().unwrap();
            let NativeKind::Text(text) = view.kind(&text_node).unwrap() else {
                panic!("text")
            };
            // HTML 877d: in body U+0000 is ignored, so the high and low
            // surrogate units become adjacent, and input preprocessing
            // normalizes CR LF to one LF.
            assert_eq!(units(&mut view, text), vec![0xd800, 0xdc00, 10]);
            let used = view
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            let reads = view.usage().work(WorkCategory::InspectByte);
            match (attempt, needed) {
                (0, _) => {
                    assert_eq!(view.unit(&text, 0).unwrap(), Some(0xd800));
                    let cost = view
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current
                        - used;
                    assert!(cost > 0);
                    assert!(view.usage().work(WorkCategory::InspectByte) > reads);
                    needed = Some(cost);
                    // An index past the end is a lookup, not a unit read.
                    let reads = view.usage().work(WorkCategory::InspectByte);
                    assert_eq!(view.unit(&text, text.len()).unwrap(), None);
                    assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
                }
                (_, Some(cost)) => {
                    let short = attempt == 2;
                    let remaining = cost - u64::from(short);
                    view.control()
                        .charge(
                            WorkCategory::StateStep,
                            request.limits().work_credits() - used - remaining,
                        )
                        .unwrap();
                    if short {
                        let Err(SessionError::Stop(first)) = view.unit(&text, 0) else {
                            panic!("one short")
                        };
                        assert!(matches!(
                            first,
                            ParseStop::LimitExceeded {
                                resource: HtmlParseResource::WorkCredits,
                                ..
                            }
                        ));
                        // Admission precedes the read: nothing was read.
                        assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
                        assert!(
                            matches!(view.kind(&root), Err(SessionError::Stop(stop)) if stop == first)
                        );
                    } else {
                        assert_eq!(view.unit(&text, 0).unwrap(), Some(0xd800));
                        assert_eq!(
                            view.usage()
                                .resource(HtmlParseResource::WorkCredits)
                                .current,
                            request.limits().work_credits()
                        );
                        assert!(view.control().stop().is_none());
                    }
                }
                (_, None) => unreachable!("the measuring attempt runs first"),
            }
        }
        // An exact budget leaves nothing for release itself; a refused
        // release keeps the owner, and the latched stop makes the retry free.
        if owner.release().is_err() {
            owner.release().unwrap();
        }
        assert!(owner.driver.is_none());
    }
}

#[test]
fn completed_native_unit_read_costs_the_same_for_any_index_text_and_document_size() {
    // NativeView::unit (construction/native_view.rs) charges one record
    // visit and, for an in-range index, the bytes of one unit; it neither
    // loops nor consults any other node or string, so its cost is the same
    // for every index, any text length and any amount of unrelated document.
    let cost = |siblings: usize, text_units: usize, last: bool| {
        let source = format!(
            "<!doctype html><body><p>{}</p>{}",
            "a".repeat(text_units),
            "<i>t</i>".repeat(siblings)
        );
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(source.as_str()),
            super::super::tests::limits(100),
        );
        let mut owner = completed(&request);
        let cost = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let p = child_named(&mut view, &body, "p");
            let text_node = view.child_at(&p, 0).unwrap().unwrap();
            let NativeKind::Text(text) = view.kind(&text_node).unwrap() else {
                panic!("text")
            };
            assert_eq!(text.len(), text_units);
            let before = view
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            let index = if last { text_units - 1 } else { 0 };
            assert_eq!(view.unit(&text, index).unwrap(), Some(u16::from(b'a')));
            view.usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
                - before
        };
        owner.release().unwrap();
        cost
    };
    let baseline = cost(0, 1, false);
    assert!(baseline > 0);
    assert_eq!(cost(0, 1, true), baseline);
    assert_eq!(cost(0, 500, false), baseline);
    assert_eq!(cost(0, 500, true), baseline);
    assert_eq!(cost(100, 500, true), baseline);
}

#[test]
fn completed_native_view_rejects_foreign_nodes_and_cancel_without_mutation() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><p>A"),
        super::super::tests::limits(100),
    );
    let mut first = completed(&request);
    let other_request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><p>A"),
        super::super::tests::limits(100),
    );
    let mut second = completed(&other_request);
    let foreign = second.native_view().unwrap().root().unwrap();
    {
        let mut view = first.native_view().unwrap();
        assert!(matches!(
            view.kind(&foreign),
            Err(SessionError::ForeignNode)
        ));
        let root = view.root().unwrap();
        let work = view
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        request.cancellation().cancel();
        assert!(matches!(
            view.child_count(&root),
            Err(SessionError::Stop(ParseStop::Cancelled))
        ));
        assert_eq!(
            view.usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
    }
    first.release().unwrap();
    second.release().unwrap();
}

#[test]
fn completed_native_view_preserves_backing_alias_rejection_and_incomplete_take() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><p>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::InsertionPoint);
    let node = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .current(&mut driver.control)
        .unwrap();
    let mut input = Some(driver);
    assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
    let mut driver = input.take().unwrap();
    finish(&mut driver);
    let mut owner = StandaloneDocumentDriver::take_completed(&mut Some(driver)).unwrap();
    let alias = {
        let driver = owner.driver.as_mut().unwrap();
        let (c, session, _) = driver.standalone_parts().unwrap();
        session
            .element_name(c, &node)
            .unwrap()
            .unwrap()
            .local_name()
            .clone()
    };
    assert!(owner.native_view().is_err());
    assert!(owner.release().is_err());
    assert!(owner.driver.is_some());
    drop(alias);
    {
        let mut view = owner.native_view().unwrap();
        let NativeKind::Element(name) = view.kind(&node).unwrap() else {
            panic!("element")
        };
        assert_eq!(units(&mut view, name.local), vec![112]);
    }
    owner.release().unwrap();
}

#[test]
fn completed_native_view_rejects_a_string_from_the_other_control() {
    let first_request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>A"),
        super::super::tests::limits(100),
    );
    let second_request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p>B"),
        super::super::tests::limits(100),
    );
    let mut first = completed(&first_request);
    let mut second = completed(&second_request);
    {
        let mut other = second.native_view().unwrap();
        let root = other.root().unwrap();
        let html = child_named(&mut other, &root, "html");
        let NativeKind::Element(name) = other.kind(&html).unwrap() else {
            panic!("element")
        };
        let mut view = first.native_view().unwrap();
        let reads = view.usage().work(WorkCategory::InspectByte);
        let Err(SessionError::Stop(first_stop)) = view.unit(&name.local, 0) else {
            panic!("foreign control")
        };
        assert!(matches!(
            first_stop,
            ParseStop::InternalInvariant(crate::parser::control::Invariant::ForeignControl)
        ));
        assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
        assert!(matches!(view.root(), Err(SessionError::Stop(stop)) if stop == first_stop));
        assert_eq!(other.unit(&name.local, 0).unwrap(), Some(104));
    }
    first.release().unwrap();
    second.release().unwrap();
}

#[test]
fn completed_native_lender_one_short_retains_whole_owner_for_cleanup() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><p>A"),
        super::super::tests::limits(100),
    );
    let mut first = completed(&request);
    let used = first
        .usage()
        .unwrap()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let cost = {
        let view = first.native_view().unwrap();
        view.usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - used
    };
    assert!(cost > 0);
    first.release().unwrap();
    let mut owner = completed(&request);
    let driver = owner.driver.as_mut().unwrap();
    let used = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    driver
        .control
        .charge(
            WorkCategory::StateStep,
            request.limits().work_credits() - used - cost + 1,
        )
        .unwrap();
    assert!(owner.native_view().is_err());
    let driver = owner.driver.as_ref().unwrap();
    assert!(driver.head.is_some());
    let stop = driver.control.stop().unwrap();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        }
    ));
    assert!(owner.native_view().is_err());
    assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
    owner.release().unwrap();
    assert!(owner.driver.is_none());
}

fn phase(driver: &StandaloneDocumentDriver<'_>) -> Option<EndPhase> {
    driver
        .head
        .as_ref()
        .and_then(|h| h.state.as_ref())
        .and_then(|s| s.end)
}
fn to_phase(driver: &mut StandaloneDocumentDriver<'_>, expected: EndPhase) {
    for _ in 0..1_000 {
        if phase(driver) == Some(expected) {
            return;
        }
        assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
    }
    panic!("actual end phase {expected:?} was not reached");
}
fn state(driver: &mut StandaloneDocumentDriver<'_>) -> StandaloneDocumentState {
    let (c, session, _) = driver.standalone_parts().unwrap();
    session.standalone_document_state(c).unwrap()
}
fn finish(driver: &mut StandaloneDocumentDriver<'_>) {
    for _ in 0..1_000 {
        match driver.step().unwrap() {
            HeadStep::Consumed => {}
            HeadStep::Complete => return,
            other => panic!("unexpected ordinary boundary {other:?}"),
        }
    }
    panic!("finite ordinary source did not finish");
}
fn cleanup(driver: &mut StandaloneDocumentDriver<'_>) {
    if driver.release().is_err() {
        driver.release().unwrap();
    }
    super::super::tests::assert_released(&driver.control);
}

#[test]
fn ordered_evidence_keeps_actual_delayed_table_and_lexer_emission_order() {
    use crate::construction::evidence::{
        DiagnosticContext as C, DiagnosticKind as K, DiagnosticSeverity,
    };
    use crate::parser::tokenizer::token::{DiagnosticCode as L, DiagnosticPhase};
    use head::{HeadCondition as T, InsertionMode as M};
    use prologue::{PrologueCondition as P, PrologueMode};
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<table>A<!--\0-->\0</table>"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    finish(&mut driver);
    let pointer = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .session
        .observed_evidence()
        .records_pointer();
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    let evidence = completed.diagnostics().unwrap();
    assert_eq!(evidence.records_pointer(), pointer);
    // HTML Initial; comment-state NULL; delayed InTableText list error,
    // InTable anything-else; Data NULL; InTableText NULL. Source offsets
    // intentionally go backwards after the lexical comment error.
    let expected = [
        (
            K::Prologue(P::MissingDoctype),
            C::Prologue(PrologueMode::Initial),
            0,
            7,
            "the-initial-insertion-mode",
        ),
        (
            K::Tokenizer(L::UnexpectedNullCharacter),
            C::Tokenizer {
                phase: DiagnosticPhase::Tokenizer,
                state: crate::parser::tokenizer::states::State::Comment,
            },
            12,
            13,
            "parse-error-unexpected-null-character",
        ),
        (
            K::Tree(T::NonWhitespaceInTable),
            C::Tree(M::InTableText),
            7,
            8,
            "parsing-main-intabletext",
        ),
        (
            K::Tree(T::UnexpectedTableToken),
            C::Tree(M::InTableText),
            7,
            8,
            "parsing-main-intable",
        ),
        (
            K::Tokenizer(L::UnexpectedNullCharacter),
            C::Tokenizer {
                phase: DiagnosticPhase::Tokenizer,
                state: crate::parser::tokenizer::states::State::Data,
            },
            16,
            17,
            "parse-error-unexpected-null-character",
        ),
        (
            K::Tree(T::NullInTable),
            C::Tree(M::InTableText),
            16,
            17,
            "parsing-main-intabletext",
        ),
    ];
    assert_eq!(evidence.len(), expected.len());
    for (index, (kind, context, start, end, anchor)) in expected.into_iter().enumerate() {
        let (record, source) = evidence.retained_record(index).unwrap();
        assert_eq!(
            (record.kind, record.context, record.severity),
            (kind, context, DiagnosticSeverity::Recoverable)
        );
        assert_eq!((source.start(), source.end()), (start, end));
        assert_eq!(source.source(), request.source().identity());
        assert_eq!(record.rule.anchor, anchor);
        use crate::construction::evidence::{DiagnosticLocationRole as R, DiagnosticToken};
        assert_eq!(
            source.role,
            match index {
                0 => R::TokenEnvelope,
                1 | 4 => R::InputFrontier,
                2 | 3 => R::BufferedTableCharacter,
                _ => R::TextOrigin,
            }
        );
        assert_eq!(
            source.token,
            match index {
                0 => Some(DiagnosticToken::Tag),
                1 | 4 => None,
                _ => Some(DiagnosticToken::Text),
            }
        );
        assert!(!record.rule.branch.is_empty());
    }
    assert!(evidence.retained_record(expected.len()).is_none());
    completed.release().unwrap();
}

#[test]
fn completed_evidence_move_preserves_both_buffers_and_requires_same_completion_owner() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<p/>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    finish(&mut driver);
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    let before = completed.diagnostics().unwrap().records_pointer();
    let evidence = completed.take_diagnostics().unwrap();
    assert_eq!(evidence.records_pointer(), before);
    assert!(completed.diagnostics().is_err());
    assert_eq!(evidence.len(), 2); // missing doctype; nonvoid self-closing p
    let c = &mut completed.driver.as_mut().unwrap().control;
    evidence.release(c).unwrap();
    completed.release().unwrap();
}

#[test]
fn failed_extraction_cleanup_refusal_keeps_actual_evidence_for_retry_or_disposal() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::InsertionPoint);
    let node = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .current(&mut driver.control)
        .unwrap();
    let (c, session, _) = driver.standalone_parts().unwrap();
    let alias = session
        .element_name(c, &node)
        .unwrap()
        .unwrap()
        .local_name()
        .clone();
    request.cancellation().cancel();
    assert!(driver.step().is_err());
    let stop = driver.control.stop();
    let pointer = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .session
        .observed_evidence()
        .records_pointer();
    let mut input = Some(driver);
    assert!(StandaloneDocumentDriver::take_failed(&mut input).is_err());
    assert!(input.is_some());
    assert_eq!(
        input
            .as_ref()
            .unwrap()
            .failed_evidence
            .as_ref()
            .unwrap()
            .records_pointer(),
        pointer
    );
    assert_eq!(input.as_ref().unwrap().control.stop(), stop);
    drop(alias);
    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
    assert!(input.is_none());
    assert_eq!(failed.diagnostics().unwrap().records_pointer(), pointer);
    assert_eq!(Some(failed.stop()), stop);
    failed.release().unwrap();
    super::super::tests::assert_released(&failed.control);
}

#[test]
fn failed_evidence_retains_only_committed_prefix_and_actual_storage_tickets() {
    let limits = super::super::tests::limits(100)
        .into_builder()
        .diagnostics(1)
        .finish()
        .unwrap();
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<table>\0"), limits);
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    loop {
        if driver.step().is_err() {
            break;
        }
    }
    let stop = driver.control.stop().unwrap();
    assert!(matches!(
        stop,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            ..
        }
    ));
    let before = driver.control.usage().clone();
    let polls = driver.control.owned_span_observation().polls;
    let mut input = Some(driver);
    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
    assert!(input.is_none());
    assert_eq!(failed.stop(), stop);
    assert_eq!(failed.source().identity(), request.source().identity());
    assert_eq!(failed.at_stop(), &before);
    // ManagedNodes is the nonrefundable cumulative allocation count;
    // native reclamation must not rewrite this execution history.
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::ManagedNodes),
        before.resource(HtmlParseResource::ManagedNodes)
    );
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::ManagedEdges)
            .current,
        0
    );
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::OpenElements)
            .current,
        0
    );
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::Diagnostics)
            .current,
        1
    );
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::SourceMapEntries)
            .current,
        1
    );
    assert!(failed.after_cleanup().actual_capacity_bytes > 0);
    assert_eq!(
        failed
            .after_cleanup()
            .resource(HtmlParseResource::WorkCredits),
        before.resource(HtmlParseResource::WorkCredits)
    );
    assert_eq!(failed.control.owned_span_observation().polls, polls);
    let evidence = failed.diagnostics().unwrap();
    assert_eq!(evidence.len(), 1);
    let (record, source) = evidence.retained_record(0).unwrap();
    assert_eq!(
        record.kind,
        crate::construction::evidence::DiagnosticKind::Prologue(
            prologue::PrologueCondition::MissingDoctype
        )
    );
    assert_eq!((source.start(), source.end()), (0, 7));
    assert_eq!(failed.retained_usage(), failed.after_cleanup());
    failed.release().unwrap();
    assert!(failed.diagnostics().is_none());
    super::super::tests::assert_released(&failed.control);
    assert_eq!(failed.control.owned_span_observation().polls, polls);
    assert_eq!(failed.stop(), stop);
    assert_eq!(
        failed
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits),
        before.resource(HtmlParseResource::WorkCredits)
    );
}

#[test]
fn cancellation_at_final_completion_retains_prior_diagnostics_without_useful_cleanup() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::VerifyComplete);
    request.cancellation().cancel();
    assert!(driver.step().is_err());
    let mut input = Some(driver);
    assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
    assert_eq!(failed.stop(), ParseStop::Cancelled);
    assert_eq!(
        failed.at_stop().resource(HtmlParseResource::WorkCredits),
        failed
            .after_cleanup()
            .resource(HtmlParseResource::WorkCredits)
    );
    assert_eq!(failed.diagnostics().unwrap().len(), 2); // missing doctype; open div at EOF
    failed.release().unwrap();
    super::super::tests::assert_released(&failed.control);
}

#[test]
fn interactive_precedes_all_pops_and_postload_precedes_complete_continuation() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::InsertionPoint);
    assert_eq!(
        state(&mut driver),
        StandaloneDocumentState {
            readiness: ConstructionReadiness::Complete,
            ready_for_post_load_tasks: false,
            completion: ConstructionCompletion::Parsing
        }
    );
    let open = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .stack()
        .unwrap()
        .len();
    to_phase(&mut driver, EndPhase::PopOpen);
    assert_eq!(
        state(&mut driver).readiness,
        ConstructionReadiness::Interactive
    );
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .stack()
            .unwrap()
            .len(),
        open
    );
    assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
    assert_eq!(
        driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .stack()
            .unwrap()
            .len(),
        open - 1
    );
    to_phase(&mut driver, EndPhase::PostLoadMilestone);
    assert_eq!(
        state(&mut driver).readiness,
        ConstructionReadiness::Interactive
    );
    assert!(!state(&mut driver).ready_for_post_load_tasks);
    driver.step().unwrap();
    assert_eq!(phase(&driver), Some(EndPhase::CompleteReadiness));
    assert_eq!(
        state(&mut driver),
        StandaloneDocumentState {
            readiness: ConstructionReadiness::Interactive,
            ready_for_post_load_tasks: true,
            completion: ConstructionCompletion::Parsing
        }
    );
    driver.step().unwrap();
    assert_eq!(
        state(&mut driver).readiness,
        ConstructionReadiness::Complete
    );
    assert_eq!(
        state(&mut driver).completion,
        ConstructionCompletion::Parsing
    );
    finish(&mut driver);
    assert_eq!(
        state(&mut driver).completion,
        ConstructionCompletion::Complete
    );
    let input_reads = driver.control.usage().work(WorkCategory::InputUnitRead);
    let writes = driver.control.usage().work(WorkCategory::RecordWrite);
    assert_eq!(driver.step().unwrap(), HeadStep::Complete);
    assert_eq!(
        driver.control.usage().work(WorkCategory::InputUnitRead),
        input_reads
    );
    assert_eq!(
        driver.control.usage().work(WorkCategory::RecordWrite),
        writes
    );
    cleanup(&mut driver);
}

#[test]
fn both_spin_checkpoints_run_when_empty_and_do_not_write_readiness_early() {
    let request =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), super::super::tests::limits(100));
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::DomContentLoadedDisposition);
    let writes = driver.control.usage().work(WorkCategory::RecordWrite);
    driver.step().unwrap();
    assert_eq!(phase(&driver), Some(EndPhase::ScriptsCheckpoint));
    // Step6's eliminated external task has only its actual cursor write.
    assert_eq!(
        driver.control.usage().work(WorkCategory::RecordWrite) - writes,
        1
    );
    for next in [
        EndPhase::LoadBlockersCheckpoint,
        EndPhase::RetainCompleteContinuation,
    ] {
        let writes = driver.control.usage().work(WorkCategory::RecordWrite);
        driver.step().unwrap();
        assert_eq!(phase(&driver), Some(next));
        // Actual FIFO entry/exit plus retained triggering-cause reference,
        // restored cause and final phase write: five fixed owned writes.
        assert_eq!(
            driver.control.usage().work(WorkCategory::RecordWrite) - writes,
            5
        );
        assert_eq!(
            state(&mut driver).readiness,
            ConstructionReadiness::Interactive
        );
        assert!(!state(&mut driver).ready_for_post_load_tasks);
    }
    finish(&mut driver);
    cleanup(&mut driver);
}

#[test]
fn eof_drains_real_option_removal_task_and_clears_selectedcontent() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A</option></select>",
        ),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let option = loop {
        driver.step().unwrap();
        let Some(state) = driver.head.as_mut().and_then(|h| h.state.as_mut()) else {
            continue;
        };
        let Some(last) = state.before.kernel.stack().unwrap().len().checked_sub(1) else {
            continue;
        };
        let node = state.before.kernel.at(last, &mut driver.control).unwrap();
        if state
            .before
            .kernel
            .named(&node, "option", &mut driver.control)
            .unwrap()
        {
            break node;
        }
    };
    to_phase(&mut driver, EndPhase::InsertionPoint);
    let (c, session, capability) = driver.standalone_parts().unwrap();
    session.remove(c, &option, Some(capability)).unwrap();
    assert_eq!(
        session.freeze(c).err(),
        Some(SessionError::PendingIntrinsicWork)
    );
    to_phase(&mut driver, EndPhase::ScriptsCheckpoint);
    driver.step().unwrap();
    assert_eq!(phase(&driver), Some(EndPhase::LoadBlockersCheckpoint));
    let (c, session, _) = driver.standalone_parts().unwrap();
    session.require_standalone_effects_finished(c).unwrap();
    finish(&mut driver);
    let (mut c, mut frozen) = super::super::tests::body_document(driver);
    let actual = super::super::tests::outline(frozen.document().unwrap());
    frozen.release(&mut c).unwrap();
    super::super::tests::assert_released(&c);
    assert_eq!(
        actual,
        super::super::tests::expected_outline(&[
            (0, "html"),
            (1, "head"),
            (1, "body"),
            (2, "select"),
            (3, "button"),
            (4, "selectedcontent")
        ])
    );
}

const PHASES: [EndPhase; 13] = [
    EndPhase::InsertionPoint,
    EndPhase::Interactive,
    EndPhase::PopOpen,
    EndPhase::DeferredScriptsAbsent,
    EndPhase::DomContentLoadedDisposition,
    EndPhase::ScriptsCheckpoint,
    EndPhase::LoadBlockersCheckpoint,
    EndPhase::RetainCompleteContinuation,
    EndPhase::PrintDisposition,
    EndPhase::PostLoadMilestone,
    EndPhase::CompleteReadiness,
    EndPhase::NullBrowsingContextTail,
    EndPhase::VerifyComplete,
];

#[test]
fn every_end_phase_cancellation_preserves_state_and_has_zero_useful_cleanup() {
    for target in PHASES {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A",
            ),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, target);
        let work = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        request.cancellation().cancel();
        assert!(matches!(
            driver.step(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        assert_eq!(phase(&driver), Some(target));
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
        let driver = input.as_mut().unwrap();
        assert_eq!(phase(driver), Some(target));
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
        cleanup(driver);
        assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            work
        );
    }
}

#[test]
fn exact_one_short_end_phase_work_keeps_first_stop_and_reclaims_all_real_owners() {
    for target in PHASES {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A",
            ),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, target);
        let before = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver.step().unwrap();
        let cost = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        assert!(cost > 0);
        cleanup(&mut driver);
        for allowance in [0, cost - 1] {
            let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
            to_phase(&mut driver, target);
            let used = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            driver
                .control
                .charge(
                    WorkCategory::StateStep,
                    request.limits().work_credits() - used - allowance,
                )
                .unwrap();
            assert!(driver.step().is_err(), "{target:?}/{allowance}");
            let first = driver.control.stop().unwrap();
            assert!(matches!(
                first,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ));
            let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
            assert!(matches!(driver.step(), Err(StandaloneError::Stop(stop)) if stop == first));
            assert_eq!(
                driver.control.usage().work(WorkCategory::InputUnitRead),
                reads
            );
            let work = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            cleanup(&mut driver);
            assert_eq!(driver.control.stop(), Some(first));
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                work
            );
        }
    }
}

#[test]
fn completed_owner_retains_actual_evidence_and_same_control_usage_without_clones() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8(
            "<!doctype bad><html><head><meta/><title>A&amp;B</title></head><body>\0<div/>X",
        ),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    finish(&mut driver);
    let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
    let lexical = state.lexical.as_slice().as_ptr();
    let diagnostics = state
        .before
        .kernel
        .session
        .observed_evidence()
        .records_pointer();

    let lengths = (
        state.lexical.len(),
        state.before.kernel.session.observed_evidence().len(),
        state.before.diagnostics().len(),
    );
    assert!(lengths.0 > 0 && lengths.1 > 0 && lengths.2 > 0);
    let usage = driver.control.usage().clone();
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    assert!(input.is_none());
    assert!(
        completed
            .usage()
            .unwrap()
            .resource(HtmlParseResource::WorkCredits)
            .current
            >= usage.resource(HtmlParseResource::WorkCredits).current
    );
    let actual = completed.document_state().unwrap();
    assert_eq!(actual.readiness, ConstructionReadiness::Complete);
    assert!(actual.ready_for_post_load_tasks);
    assert_eq!(actual.completion, ConstructionCompletion::Complete);
    let retained = completed
        .driver
        .as_ref()
        .unwrap()
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap();
    assert_eq!(
        (
            retained.lexical.len(),
            retained.before.kernel.session.observed_evidence().len(),
            retained.before.diagnostics().len()
        ),
        lengths
    );
    assert_eq!(retained.lexical.as_slice().as_ptr(), lexical);
    assert_eq!(
        retained
            .before
            .kernel
            .session
            .observed_evidence()
            .records_pointer(),
        diagnostics
    );

    completed.release().unwrap();
    assert!(completed.driver.is_none());
    assert!(matches!(
        completed.document_state(),
        Err(StandaloneError::Consumed)
    ));
}

#[test]
fn completed_take_incomplete_and_aliased_rejection_retains_exact_input_owner() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::InsertionPoint);
    let node = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .current(&mut driver.control)
        .unwrap();
    let mut input = Some(driver);
    assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
    assert!(input.is_some());
    finish(input.as_mut().unwrap());
    let driver = input.as_mut().unwrap();
    let (c, session, _) = driver.standalone_parts().unwrap();
    let alias = session
        .element_name(c, &node)
        .unwrap()
        .unwrap()
        .local_name()
        .clone();
    assert!(matches!(
        StandaloneDocumentDriver::take_completed(&mut input),
        Err(StandaloneError::Head(HeadError::Kernel(
            KernelError::Session(SessionError::AliasedPayload)
        )))
    ));
    assert!(input.is_some());
    drop(alias);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    completed.release().unwrap();
}

#[test]
fn completed_take_exact_one_short_preflight_retains_terminal_owner() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    finish(&mut driver);
    let before = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    let cost = completed
        .usage()
        .unwrap()
        .resource(HtmlParseResource::WorkCredits)
        .current
        - before;
    completed.release().unwrap();
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    finish(&mut driver);
    let used = driver
        .control
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current;
    driver
        .control
        .charge(
            WorkCategory::StateStep,
            request.limits().work_credits() - used - cost + 1,
        )
        .unwrap();
    let mut input = Some(driver);
    assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
    let driver = input.as_mut().unwrap();
    let first = driver.control.stop().unwrap();
    assert!(matches!(
        first,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        }
    ));
    cleanup(driver);
    assert_eq!(driver.control.stop(), Some(first));
}

#[test]
fn completed_release_alias_and_cancel_keep_real_borrowed_owner_until_cleanup() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><div>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    to_phase(&mut driver, EndPhase::InsertionPoint);
    let node = driver
        .head
        .as_ref()
        .unwrap()
        .state
        .as_ref()
        .unwrap()
        .before
        .kernel
        .current(&mut driver.control)
        .unwrap();
    finish(&mut driver);
    let mut input = Some(driver);
    let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
    let driver = completed.driver.as_mut().unwrap();
    let (c, session, _) = driver.standalone_parts().unwrap();
    let alias = session
        .element_name(c, &node)
        .unwrap()
        .unwrap()
        .local_name()
        .clone();
    assert!(completed.release().is_err());
    assert!(completed.driver.is_some());
    drop(alias);
    request.cancellation().cancel();
    assert!(matches!(
        completed.document_state(),
        Err(StandaloneError::Stop(ParseStop::Cancelled))
    ));
    assert_eq!(
        completed.driver.as_ref().unwrap().control.stop(),
        Some(ParseStop::Cancelled)
    );
    completed.release().unwrap();
    assert!(completed.driver.is_none());
    assert!(matches!(completed.usage(), Err(StandaloneError::Consumed)));
}

#[test]
fn completed_release_one_short_preflight_retains_owner_then_terminal_cleanup_is_free() {
    fn request() -> HtmlDocumentParseRequest {
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        )
    }
    // A completed owner whose remaining work is exactly `spare`.
    fn completed_with_spare(
        request: &HtmlDocumentParseRequest,
        spare: u64,
    ) -> CompletedStandaloneDocument<'_> {
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(request);
        finish(&mut driver);
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let driver = completed.driver.as_mut().unwrap();
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - spare,
            )
            .unwrap();
        completed
    }
    // Boundaries are found from observed outcomes by bisection, never
    // computed from the size of the driver being moved. `released` is a
    // release that finished; `started` is one that charged any work, which is
    // to say it got past its preflight admission.
    let first_release = |spare: u64| {
        let request = request();
        let mut completed = completed_with_spare(&request, spare);
        let work = |completed: &CompletedStandaloneDocument<'_>| {
            completed
                .driver
                .as_ref()
                .unwrap()
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
        };
        let before = work(&completed);
        let released = completed.release().is_ok();
        let started = released || work(&completed) > before;
        (released, started)
    };
    let search = |reached: &dyn Fn(u64) -> bool| {
        let (mut short, mut enough) = (0, 1_000_000);
        assert!(reached(enough), "the search bound reaches the boundary");
        assert!(!reached(short), "the boundary needs work");
        while short + 1 < enough {
            let middle = short + (enough - short) / 2;
            if reached(middle) {
                enough = middle;
            } else {
                short = middle;
            }
        }
        enough
    };
    let preflight = search(&|spare| first_release(spare).1);
    let needed = search(&|spare| first_release(spare).0);
    // The preflight is only the first admission of a release that goes on
    // to release the owner's parts, so completing needs strictly more.
    assert!(0 < preflight && preflight < needed);

    // The preflight does not depend on the document being released, and the
    // total grows no faster than the document: releasing walks the retained
    // records once each, so doubling the document twice must not more than
    // double the second increase over the first.
    let bounds = |paragraphs: usize| {
        let source = format!("<!doctype html>{}", "<p>a</p>".repeat(paragraphs));
        let make = || {
            HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(source.as_str()),
                super::super::tests::limits(100),
            )
        };
        let first_release = |spare: u64| {
            let request = make();
            let mut completed = completed_with_spare(&request, spare);
            let work = |completed: &CompletedStandaloneDocument<'_>| {
                completed
                    .driver
                    .as_ref()
                    .unwrap()
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current
            };
            let before = work(&completed);
            let released = completed.release().is_ok();
            let started = released || work(&completed) > before;
            (released, started)
        };
        (
            search(&|spare| first_release(spare).1),
            search(&|spare| first_release(spare).0),
        )
    };
    let (small_preflight, small) = bounds(4);
    let (medium_preflight, medium) = bounds(8);
    let (large_preflight, large) = bounds(16);
    assert_eq!(small_preflight, preflight);
    assert_eq!(medium_preflight, preflight);
    assert_eq!(large_preflight, preflight);
    assert!(small <= medium && medium <= large);
    assert!(large - medium <= 2 * (medium - small));

    // Exactly the needed work releases the owner and consumes it.
    let exact_request = request();
    let mut exact = completed_with_spare(&exact_request, needed);
    exact.release().unwrap();
    assert!(exact.driver.is_none());
    assert!(matches!(exact.usage(), Err(StandaloneError::Consumed)));

    // Anything less is refused with a latched work stop that keeps the
    // owner; the retry then succeeds without any spare work. One credit
    // short of the preflight refuses before anything is released.
    for spare in [0, preflight - 1, preflight, needed - 1] {
        let short_request = request();
        let mut completed = completed_with_spare(&short_request, spare);
        let before = completed
            .driver
            .as_ref()
            .unwrap()
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        assert!(completed.release().is_err(), "spare {spare}");
        let driver = completed.driver.as_ref().unwrap();
        let first = driver.control.stop().unwrap();
        assert!(
            matches!(
                first,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ),
            "spare {spare}"
        );
        let work = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        if spare < preflight {
            // The refused preflight charged nothing and released nothing.
            assert_eq!(work, before, "spare {spare}");
            assert!(driver.head.is_some(), "spare {spare}");
        } else {
            assert!(work > before, "spare {spare}");
        }
        completed.release().unwrap();
        assert!(completed.driver.is_none(), "spare {spare}");
        // State/usage getters are intentionally consumed along with that owner.
        assert!(matches!(completed.usage(), Err(StandaloneError::Consumed)));
        assert!(work <= short_request.limits().work_credits());
    }
}

#[test]
fn unclosed_svg_script_stays_unprocessed_during_eof_pops() {
    let request = HtmlDocumentParseRequest::offline(
        HtmlSource::utf8("<!doctype html><svg><script>A"),
        super::super::tests::limits(100),
    );
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let script = super::super::tests::svg_open_node(&mut driver);
    finish(&mut driver);
    let (c, session, _) = driver.standalone_parts().unwrap();
    assert!(
        !session
            .svg_script_state(c, &script)
            .unwrap()
            .flags
            .already_processed()
    );
    cleanup(&mut driver);
}

#[test]
fn failed_transfer_preserves_first_stop_usage_before_reference_scratch_cleanup() {
    // Missing doctype is the one retained diagnostic. The actual unknown
    // named reference then reaches the next diagnostic cap while its
    // literal text and reference-query scratch are still owned.
    let limits = super::super::tests::limits(100)
        .into_builder()
        .diagnostics(1)
        .finish()
        .unwrap();
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>&bogus;"), limits);
    let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
    let mut stopped = false;
    for _ in 0..128 {
        if driver.step().is_err() {
            stopped = true;
            break;
        }
    }
    assert!(stopped, "literal reference reaches the second diagnostic");
    let first = driver.control.stop().unwrap();
    assert!(matches!(
        first,
        ParseStop::LimitExceeded {
            resource: HtmlParseResource::Diagnostics,
            limit: 1,
            current: 1,
            requested_delta: 1,
        }
    ));
    let latched = driver.control.first_stop_usage().unwrap().clone();
    let after_unwind = driver.control.usage().clone();
    // The literal ampersand plus five letters were retained at the latch.
    // Normal reference unwinding disposes them before returning the error.
    assert!(
        latched
            .resource(HtmlParseResource::BufferedUtf16Units)
            .current
            >= after_unwind
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current
                + 6
    );
    assert!(
        latched
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current
            > after_unwind
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
    );
    assert_eq!(
        latched.resource(HtmlParseResource::WorkCredits),
        after_unwind.resource(HtmlParseResource::WorkCredits)
    );
    let mut input = Some(driver);
    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
    assert!(input.is_none());
    assert_eq!(failed.stop(), first);
    assert_eq!(failed.source().identity(), request.source().identity());
    let evidence = failed.diagnostics().unwrap();
    assert_eq!(evidence.len(), 1);
    let (record, source) = evidence.retained_record(0).unwrap();
    assert_eq!(
        record.kind,
        crate::construction::evidence::DiagnosticKind::Prologue(
            prologue::PrologueCondition::MissingDoctype
        )
    );
    assert_eq!((source.start(), source.end()), (0, 3));
    let exported_at_stop = failed.at_stop().clone();
    let after_cleanup = failed.after_cleanup().clone();
    assert_eq!(
        after_cleanup.resource(HtmlParseResource::WorkCredits),
        latched.resource(HtmlParseResource::WorkCredits)
    );
    failed.release().unwrap();
    super::super::tests::assert_released(&failed.control);
    assert_eq!(failed.stop(), first);
    assert_eq!(failed.at_stop(), &exported_at_stop);
    // This is the published failure observation contract, not a demand for
    // a particular staging layout: at_stop is the canonical latch snapshot.
    assert_eq!(exported_at_stop, latched);
}
