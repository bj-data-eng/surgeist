//! Genuine producer/fence and actual output graph custody, without Runtime mirrors.
use super::super::tests::{begin_await, binding, envelope, flag, install, parse, qualifier, setup};
use super::*;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParser, CssParserContext};
use surgeist_cssom::*;
use surgeist_html::HtmlCspOwner;
fn ready() -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    DomStyleCssomRoot,
    HtmlParseAssociationOccurrenceId,
) {
    let report = parse(
        "<!doctype html><style title=A media=''>a{color:red}</style><meta http-equiv=default-style content='B𝄞'>",
    );
    let mut cursor = report.pending_associations();
    let wanted = loop {
        match cursor.advance() {
            Step::Fault(fault) => panic!("{fault}"),
            Step::Occurrence(r) if r.kind() == Kind::ChangePreferred => break r.id(),
            Step::Complete => panic!("real Meta occurrence"),
            _ => (),
        }
    };
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, report, env);
    begin_await(&mut dom, id, envelope());
    let binding = binding(&dom, id);
    let mut admit = |_| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut admit).unwrap();
    let mut policy = Some(owner.initial_document_with(&mut admit).unwrap());
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.bind_initial_policy(&binding, &mut policy, &flag())
            .unwrap();
        hold.advance(&flag()).unwrap();
    }
    let cap = DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    for invocation in 0..2 {
        let q = qualifier(&dom, id);
        let mut input = Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            dom.parsed_html_style_prefix(id)
                .unwrap()
                .inline_policy_request()
                .unwrap(),
            true,
        ));
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.accept_inline_policy(&mut input, &flag()).unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        let parser = CssParserContext::default();
        let sheet = CssParser::new(parser, Default::default())
            .parse_sheet_utf16(
                prep.request().text().shared_code_units().clone(),
                CssUtf16InputLimits::default(),
                || prep.step(&flag()),
            )
            .unwrap();
        let media = CssMediaQueryList::parse_utf16(
            &CssParser::new(parser, Default::default()),
            prep.request().media().unwrap().shared_code_units().clone(),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
        let mut supplied = Some(DomParsedHtmlStyleSheetInput::new(
            sheet,
            media,
            CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "authentic-original".into(),
                revision: 1,
            },
            "owner".into(),
            false,
        ));
        prep.stage_sheet(&mut supplied, &flag()).unwrap();
        let (_, retired) = prep.commit(&flag()).unwrap().into_parts();
        drop(retired);
        hold.exclude_script_bookkeeping(
            &q,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
        drop(
            hold.process_rendering(
                &q,
                crate::DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag(),
            )
            .unwrap(),
        );
        if invocation == 0 {
            let (_, retired) = hold
                .prepare_successor(Some(&cap), &flag())
                .unwrap()
                .commit(&flag())
                .unwrap()
                .into_parts();
            drop(retired);
        }
    }
    (dom, document, id, cap, wanted)
}
#[test]
fn post_producer_original_work_refusal_preserves_marker_flags_and_raised_retry() {
    let (mut dom, _, id, cap, wanted) = ready();
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let sheet = cap.snapshot().unwrap().sheets()[0].clone();
    let spent;
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut prep = hold
            .prepare_meta_occurrence(&wanted, &cap, &flag())
            .unwrap();
        prep.set_disabled(&sheet, true, &flag()).unwrap();
        let available =
            prep.meter.entry().control.limits().work() - prep.meter.entry().control.spent_work();
        prep.meter.spend(available - 2, &flag()).unwrap();
        let before = PREPARED.with(|n| n.get());
        assert!(matches!(
            prep.commit(
                DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
                &flag()
            ),
            Err(Error::Style(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    ..
                })
            )))
        ));
        assert_eq!(PREPARED.with(|n| n.get()), before + 1);
        spent = hold.spent_work();
        {
            let graph = hold.meter.entry().style_prefix.try_lock().unwrap();
            let current = graph.as_ref().unwrap().value().qualification().occurrence();
            let marker = hold.meter.entry().executed_prefix.try_lock().unwrap();
            assert_eq!(&marker.as_ref().unwrap().last, current);
            assert_ne!(
                current, &wanted,
                "refused Meta cannot replace completed Style prefix"
            );
        }
    }
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert!(
        !cap.snapshot()
            .unwrap()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    let raised =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 2_000_000, 2048).unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() >= spent);
    let mut prep = hold
        .prepare_meta_occurrence(&wanted, &cap, &flag())
        .unwrap();
    prep.set_disabled(&sheet, true, &flag()).unwrap();
    let (done, dead) = prep
        .commit(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
            &flag(),
        )
        .unwrap()
        .into_parts();
    drop(dead);
    drop(hold);
    assert!(
        cap.snapshot()
            .unwrap()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    assert_eq!(done.occurrence().id(), &wanted);
}
#[test]
fn owning_name_output_survives_command_and_dom_until_its_original_receipt_drops() {
    let (mut dom, _, id, cap, wanted) = ready();
    let ledger = dom.custody_registry.ledger().clone();
    // The actual external name holder disposes before the completion receipt.
    let outcome = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let runtime =
            DomParsedHtmlAssociationRuntimeUsage::new(size_of::<DocumentString>(), 0, 1).unwrap();
        let initial = hold.meter.own_usage();
        hold.admit_runtime(runtime, &flag()).unwrap();
        let prep = hold
            .prepare_meta_occurrence(&wanted, &cap, &flag())
            .unwrap();
        let candidate = prep.occurrence().content().clone();
        let (done, dead) = prep.commit(runtime, &flag()).unwrap().into_parts();
        assert_eq!(hold.meter.own_usage(), initial);
        assert_eq!(hold.meter.runtime_usage().bytes(), 0);
        assert_eq!(hold.meter.runtime_usage().outputs(), 0);
        drop(dead);
        (candidate, done)
    };
    let done = &outcome.1;
    let weak = Arc::downgrade(done.occurrence().content().shared_code_units());
    let capacity = done.occurrence().content().shared_code_units().capacity();
    assert_eq!(
        done._meter.own_usage(),
        custody::Usage::new(
            size_of::<DomParsedHtmlMetaOccurrenceCompletion>()
                + size_of::<Vec<u16>>()
                + capacity * size_of::<u16>()
                + size_of::<DocumentString>(),
            capacity
        )
    );
    drop(cap);
    drop(dom);
    let retained = ledger.total();
    assert!(weak.upgrade().is_some());
    assert_eq!(
        outcome.0.code_units(),
        "B𝄞".encode_utf16().collect::<Vec<_>>()
    );
    drop(outcome);
    assert!(weak.upgrade().is_none());
    assert!(ledger.total().bytes() < retained.bytes());
}

#[test]
fn actual_root_busy_and_tight_retained_output_overlap_leave_the_same_row_retryable() {
    let (mut dom, _, id, cap, wanted) = ready();
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let sheet = cap.snapshot().unwrap().sheets()[0].clone();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    {
        let locked = cap.acquire().unwrap();
        assert!(matches!(
            hold.prepare_meta_occurrence(&wanted, &cap, &flag()),
            Err(Error::OwnerBusy { .. })
        ));
        drop(locked);
    }
    let mut prep = hold
        .prepare_meta_occurrence(&wanted, &cap, &flag())
        .unwrap();
    prep.set_disabled(&sheet, true, &flag()).unwrap();
    let admission = prep
        .meter
        .admission(prep.dom.custody_registry.total(), &flag())
        .unwrap();
    let available = admission.remaining_bytes().unwrap();
    let scratch_meter = prep.meter.fork();
    let requested = available - 32;
    scratch_meter
        .retain(requested + size_of::<Vec<u8>>(), 0, 0, &flag())
        .unwrap();
    let mut scratch = Vec::<u8>::new();
    scratch.try_reserve_exact(requested).unwrap();
    scratch_meter
        .retain(scratch.capacity() - requested, 0, 0, &flag())
        .unwrap();
    let before = PREPARED.with(|n| n.get());
    assert!(matches!(
        prep.commit(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
            &flag()
        ),
        Err(Error::Style(DomParsedHtmlStyleError::Admission(
            crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                resource: DomResource::ParsedAssociationProcessingRetainedBytes,
                ..
            })
        )))
    ));
    assert_eq!(
        PREPARED.with(|n| n.get()),
        before,
        "output backing must refuse before producer preparation"
    );
    {
        let graph = hold.meter.entry().style_prefix.try_lock().unwrap();
        let current = graph.as_ref().unwrap().value().qualification().occurrence();
        let marker = hold.meter.entry().executed_prefix.try_lock().unwrap();
        assert_eq!(&marker.as_ref().unwrap().last, current);
        assert_ne!(
            current, &wanted,
            "refused Meta cannot replace completed Style prefix"
        );
    }
    let spent = hold.spent_work();
    drop(scratch);
    drop(scratch_meter);
    drop(admission);
    drop(hold);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert!(
        !cap.snapshot()
            .unwrap()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() >= spent);
    let mut prep = hold
        .prepare_meta_occurrence(&wanted, &cap, &flag())
        .unwrap();
    prep.set_disabled(&sheet, true, &flag()).unwrap();
    let (done, dead) = prep
        .commit(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 0).unwrap(),
            &flag(),
        )
        .unwrap()
        .into_parts();
    drop(dead);
    drop(hold);
    assert!(
        cap.snapshot()
            .unwrap()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    assert_eq!(done.occurrence().id(), &wanted);
}
#[test]
fn consuming_commit_refusal_keeps_external_name_candidate_custody_on_the_live_hold() {
    let (mut dom, _, id, cap, wanted) = ready();
    let ledger = dom.custody_registry.ledger().clone();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let runtime =
        DomParsedHtmlAssociationRuntimeUsage::new(size_of::<DocumentString>(), 0, 1).unwrap();
    hold.admit_runtime(runtime, &flag()).unwrap();
    let before = hold.meter.own_usage();
    let prep = hold
        .prepare_meta_occurrence(&wanted, &cap, &flag())
        .unwrap();
    let candidate = prep.occurrence().content().clone();
    let capacity = candidate.shared_code_units().capacity();
    let name_usage = custody::Usage::new(
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    );
    let available =
        prep.meter.entry().control.limits().work() - prep.meter.entry().control.spent_work();
    prep.meter.spend(available - 2, &flag()).unwrap();
    let prepared = PREPARED.with(|n| n.get());
    assert!(matches!(
        prep.commit(runtime, &flag()),
        Err(Error::Style(DomParsedHtmlStyleError::Admission(
            crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                ..
            })
        )))
    ));
    assert_eq!(PREPARED.with(|n| n.get()), prepared + 1);
    assert_eq!(
        hold.meter.own_usage(),
        before.checked_add(name_usage).unwrap()
    );
    assert_eq!(hold.meter.runtime_usage().bytes(), runtime.bytes());
    assert_eq!(hold.meter.runtime_usage().outputs(), runtime.outputs());
    let charged = ledger.total();
    assert_eq!(
        candidate.code_units(),
        "B𝄞".encode_utf16().collect::<Vec<_>>()
    );
    drop(candidate);
    drop(hold);
    assert!(ledger.total().bytes() < charged.bytes());
}
