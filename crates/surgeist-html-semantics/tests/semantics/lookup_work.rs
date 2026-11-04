//! Charged work of the per-query fact lookups.
//!
//! A query that finds the fact of a subject by scanning the producer's facts
//! charges work linear in them, so a session that sweeps every element of a
//! document charges work quadratic in the document. Each test sweeps a
//! document of `n` elements whose producer supplies one fact per element and
//! requires that doubling `n` at most multiplies the session's charged work by
//! 2.2 (a sweep with linear lookups multiplies it by about 2, a scanning one
//! by about 4). Results do not change; the work is the whole difference.

use super::constructor_work::{big_builder, flat_page, html, inert_fact};
use crate::semantics::support::assert_doubling_at_most_2_2;
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureSubjectId, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlCustomControlClassification, HtmlDocumentFlatTreeRelation, HtmlElementReferenceFact,
    HtmlElementReferenceFacts, HtmlElementReferenceFactsLimits, HtmlElementReferenceKey,
    HtmlExplicitElementReference, HtmlFlatTreeProducerId, HtmlFlatTreeRevision,
    HtmlFormAssociation, HtmlFormAssociationState, HtmlInertFactLimits, HtmlInertFacts,
    HtmlInputControlState, HtmlOptionControlState, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextInput, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlTextareaControlState,
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
        .map(|subject| HtmlSemanticRequest::new(subject.clone(), query))
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

/// A document of `count` elements named `name`, all its children.
fn flat_elements(name: &str, count: usize) -> (HtmlCapture, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = big_builder(count);
    let elements: Vec<_> = (0..count).map(|_| html(&mut b, &d, name)).collect();
    for element in &elements {
        b.establish_children(element, &[]).unwrap();
    }
    b.establish_children(&d, &elements).unwrap();
    let capture = b.finish().unwrap();
    let ids = elements
        .iter()
        .map(|element| capture.subject_id(element).unwrap())
        .collect();
    (capture, ids)
}

fn element_reference_sweep(count: usize) -> usize {
    let (capture, ids) = flat_elements("button", count);
    let records: Vec<_> = ids
        .iter()
        .map(|id| {
            HtmlElementReferenceFact::new(
                id.clone(),
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )
        })
        .collect();
    let capacity = records.capacity();
    let facts = HtmlElementReferenceFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        records,
        HtmlElementReferenceFactsLimits::new(capacity, 1 << 40),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_element_references(facts),
        0,
    )
    .unwrap();
    sweep_work(
        &capture,
        &context,
        &ids,
        HtmlSemanticQuery::CommandForElement,
    )
}

#[test]
fn an_element_reference_sweep_finds_each_pointer_in_constant_work() {
    assert_doubling_at_most_2_2(
        "element reference sweep",
        &DOUBLINGS,
        element_reference_sweep,
    );
}

/// Context over `controls` built from the given records for every element.
fn control_sweep(
    name: &str,
    count: usize,
    query: HtmlSemanticQuery,
    classify: bool,
    associate: bool,
    states: bool,
) -> usize {
    let (capture, ids) = flat_elements(name, count);
    let associations: Vec<_> = if associate {
        ids.iter()
            .map(|id| {
                HtmlFormAssociation::new(
                    id.clone(),
                    HtmlFormAssociationState::EstablishedCurrent(None),
                )
            })
            .collect()
    } else {
        vec![]
    };
    let (mut inputs, mut options, mut textareas) = (vec![], vec![], vec![]);
    if states {
        for id in &ids {
            match name {
                "input" => inputs.push(HtmlInputControlState::new(
                    id.clone(),
                    Some(true),
                    Some(false),
                    Some(false),
                    Some(false),
                )),
                "option" => options.push(HtmlOptionControlState::new(
                    id.clone(),
                    Some(true),
                    Some(false),
                )),
                _ => textareas.push(HtmlTextareaControlState::new(id.clone(), Some(false))),
            }
        }
    }
    let records =
        associations.capacity() + inputs.capacity() + options.capacity() + textareas.capacity();
    let facts = HtmlControlFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        associations,
        inputs,
        options,
        textareas,
        HtmlControlFactsLimits::new(records, 1 << 40),
    )
    .unwrap();
    // Reversed, so that finding a subject by scanning is not accidentally cheap.
    let classifications: Vec<_> = if classify {
        ids.iter()
            .rev()
            .map(|id| HtmlCustomControlClassification::new(id.clone(), true))
            .collect()
    } else {
        vec![]
    };
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(classifications).with_controls(facts),
        count,
    )
    .unwrap();
    sweep_work(&capture, &context, &ids, query)
}

#[test]
fn a_form_association_sweep_finds_each_association_in_constant_work() {
    assert_doubling_at_most_2_2("form association sweep", &DOUBLINGS, |n| {
        control_sweep("input", n, HtmlSemanticQuery::FormOwner, false, true, false)
    });
}
#[test]
fn a_custom_control_sweep_finds_each_classification_in_constant_work() {
    assert_doubling_at_most_2_2("custom control sweep", &DOUBLINGS, |n| {
        control_sweep(
            "x-widget",
            n,
            HtmlSemanticQuery::FormOwner,
            true,
            true,
            false,
        )
    });
}
#[test]
fn a_control_state_sweep_finds_each_state_in_constant_work() {
    assert_doubling_at_most_2_2("input state sweep", &DOUBLINGS, |n| {
        control_sweep(
            "input",
            n,
            HtmlSemanticQuery::CurrentCheckedness,
            false,
            false,
            true,
        )
    });
    assert_doubling_at_most_2_2("option state sweep", &DOUBLINGS, |n| {
        control_sweep(
            "option",
            n,
            HtmlSemanticQuery::CurrentSelectedness,
            false,
            false,
            true,
        )
    });
    assert_doubling_at_most_2_2("textarea state sweep", &DOUBLINGS, |n| {
        control_sweep(
            "textarea",
            n,
            HtmlSemanticQuery::DirtyValueFlag,
            false,
            false,
            true,
        )
    });
}
