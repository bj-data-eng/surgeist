//! Charged work of the per-query fact lookups.
//!
//! A query that finds the fact of a subject by scanning the producer's facts
//! charges work linear in them, so a session that sweeps every element of a
//! document charges work quadratic in the document. Each test sweeps a
//! document of `n` elements whose producer supplies one fact per element and
//! requires that doubling `n` at most multiplies the session's charged work by
//! 2.2 (a sweep with linear lookups multiplies it by about 2, a scanning one
//! by about 4). Results do not change; the work is the whole difference.

use super::constructor_work::{flat_page, inert_fact};
use crate::semantics::support::assert_doubling_at_most_2_2;
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureSubjectId, HtmlDocumentFlatTreeRelation, HtmlFlatTreeProducerId,
    HtmlFlatTreeRevision, HtmlInertFactLimits, HtmlInertFacts, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticLimits, HtmlSemanticQuery,
    HtmlSemanticRequest, HtmlSemanticSession,
};

const DOUBLINGS: [usize; 4] = [250, 500, 1000, 2000];

/// Limits no sweep of these sizes reaches, so the usage alone is measured.
fn roomy_limits() -> HtmlSemanticLimits {
    let big = 1 << 40;
    HtmlSemanticLimits::new(big, big, big, 0, big, big)
}

/// The work one batch of `query` over `subjects` charges.
fn sweep_work(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    subjects: &[HtmlCaptureSubjectId],
    query: HtmlSemanticQuery,
) -> usize {
    let requests: Vec<_> = subjects
        .iter()
        .map(|subject| HtmlSemanticRequest::new(subject.clone(), query.clone()))
        .collect();
    let mut session = HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(10000),
        roomy_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let report = session.query(&requests).unwrap();
    report.usage().work()
}

fn inert_sweep(count: usize) -> usize {
    let (capture, ids) = flat_page(count);
    let mut records = vec![inert_fact(
        &ids[0],
        Some(true),
        HtmlDocumentFlatTreeRelation::Root,
    )];
    for id in &ids[1..] {
        records.push(inert_fact(
            id,
            Some(true),
            HtmlDocumentFlatTreeRelation::Parent(ids[0].clone()),
        ));
    }
    let revision = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 1, 1);
    let capacity = records.capacity();
    let facts = HtmlInertFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        records,
        vec![],
        HtmlInertFactLimits::new(capacity, 1 << 40),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_inert(revision, Some(facts)),
        0,
    )
    .unwrap();
    sweep_work(&capture, &context, &ids[1..], HtmlSemanticQuery::Inertness)
}

#[test]
fn an_inertness_sweep_finds_each_flat_record_in_constant_work() {
    assert_doubling_at_most_2_2("inertness sweep", &DOUBLINGS, inert_sweep);
}
