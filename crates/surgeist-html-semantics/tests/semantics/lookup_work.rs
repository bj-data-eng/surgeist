//! Charged work of the per-query fact lookups.
//!
//! A query that finds the fact of a subject by scanning the producer's facts
//! charges work linear in them, so a session that sweeps every element of a
//! document charges work quadratic in the document. Each test sweeps a
//! document of `n` elements whose producer supplies one fact per element and
//! requires that doubling `n` at most multiplies the session's charged work by
//! 2.2 (a sweep with linear lookups multiplies it by about 2, a scanning one
//! by about 4). Results do not change; the work is the whole difference.

use super::constructor_work::{attributed, big_builder, flat_page, html, inert_fact, shadow_root};
use crate::semantics::support::{assert_doubling_at_most_2_2, builder_with};
use surgeist_html::construction::HtmlDocumentMode;
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlControlCurrentValue, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlCustomControlClassification, HtmlDesignMode, HtmlDocumentDesignModeFact,
    HtmlDocumentFlatTreeRelation, HtmlDocumentLanguageFallback, HtmlElementReferenceFact,
    HtmlElementReferenceFacts, HtmlElementReferenceFactsLimits, HtmlElementReferenceKey,
    HtmlExplicitElementReference, HtmlFlatTreeProducerId, HtmlFlatTreeRevision,
    HtmlFormAssociation, HtmlFormAssociationState, HtmlGlobalInheritanceFacts,
    HtmlGlobalInheritanceLimits, HtmlGlobalInheritanceProducerId, HtmlGlobalInheritanceRevision,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInputControlState, HtmlLanguageDirectionFacts,
    HtmlLanguageDirectionLimits, HtmlListParticipation, HtmlListParticipationFact,
    HtmlListParticipationRevision, HtmlMediaAttributeInput, HtmlMediaContextRevision,
    HtmlMediaEnvironmentRevision, HtmlMediaFact, HtmlMediaFacts, HtmlMediaFactsLimits,
    HtmlMediaGrammarRevision, HtmlMediaProducerId, HtmlMicrodataFacts, HtmlMicrodataPropertyFact,
    HtmlMicrodataPropertyPermission, HtmlMicrodataProviderId, HtmlMicrodataProviderRevision,
    HtmlMicrodataRawBinding, HtmlMicrodataRawSource, HtmlMicrodataRevision, HtmlMicrodataTypeFact,
    HtmlMicrodataUrlFact, HtmlMicrodataUrlOperation, HtmlMicrodataUrlResult,
    HtmlMicrodataVocabularyId, HtmlOptionControlState, HtmlOutputDefaultFact,
    HtmlOutputDefaultFacts, HtmlOutputDefaultLimits, HtmlOutputDefaultValueOverride,
    HtmlParticipationProducerId, HtmlPragmaLanguage, HtmlProtocolLanguage, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticSession, HtmlSlotAssignedNodes,
    HtmlSpellcheckDefaultBehavior, HtmlSpellcheckDefaultFact, HtmlTextareaControlState,
};

const DOUBLINGS: [usize; 4] = [250, 500, 1000, 2000];

/// Limits no sweep of these sizes reaches, so the usage alone is measured.
pub fn roomy_limits() -> HtmlSemanticLimits {
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

fn language_context(
    capture: &HtmlCapture,
    fallbacks: Vec<HtmlDocumentLanguageFallback>,
    values: Vec<HtmlControlCurrentValue>,
    assignments: Vec<HtmlSlotAssignedNodes>,
) -> HtmlSemanticContext {
    let records = fallbacks.capacity() + values.capacity() + assignments.capacity();
    let facts = HtmlLanguageDirectionFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        fallbacks,
        values,
        assignments,
        HtmlLanguageDirectionLimits::new(records, 0, 1 << 20, 1 << 40),
    )
    .unwrap();
    HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_language_direction(facts),
        0,
    )
    .unwrap()
}

/// `count` documents, each holding one `div`: the documents and the divs.
fn document_page(
    count: usize,
) -> (
    HtmlCapture,
    Vec<HtmlCaptureSubjectId>,
    Vec<HtmlCaptureSubjectId>,
) {
    let (mut b, first) = big_builder(2 * count);
    let mut documents = vec![first];
    for _ in 1..count {
        documents.push(
            b.add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap(),
        );
    }
    let divs: Vec<_> = documents.iter().map(|d| html(&mut b, d, "div")).collect();
    for (document, div) in documents.iter().zip(&divs) {
        b.establish_children(div, &[]).unwrap();
        b.establish_children(document, std::slice::from_ref(div))
            .unwrap();
    }
    let capture = b.finish().unwrap();
    let documents = documents
        .iter()
        .map(|d| capture.subject_id(d).unwrap())
        .collect();
    let divs = divs
        .iter()
        .map(|div| capture.subject_id(div).unwrap())
        .collect();
    (capture, documents, divs)
}

/// `count` documents, each holding one `div`, with a language fallback each.
fn language_fallback_sweep(count: usize) -> usize {
    let (capture, documents, divs) = document_page(count);
    let fallbacks = documents
        .iter()
        .map(|d| {
            HtmlDocumentLanguageFallback::new(
                d.clone(),
                HtmlPragmaLanguage::Tag("en".into()),
                HtmlProtocolLanguage::Absent,
            )
        })
        .collect();
    let context = language_context(&capture, fallbacks, vec![], vec![]);
    sweep_work(&capture, &context, &divs, HtmlSemanticQuery::Language)
}

fn current_value_sweep(count: usize) -> usize {
    let (mut b, d) = big_builder(count);
    let areas: Vec<_> = (0..count)
        .map(|_| attributed(&mut b, &d, "textarea", &[("required", "")]))
        .collect();
    for area in &areas {
        b.establish_children(area, &[]).unwrap();
    }
    b.establish_children(&d, &areas).unwrap();
    let capture = b.finish().unwrap();
    let ids: Vec<_> = areas
        .iter()
        .map(|area| capture.subject_id(area).unwrap())
        .collect();
    let values = ids
        .iter()
        .map(|id| HtmlControlCurrentValue::new(id.clone(), Some("typed".into())))
        .collect();
    let context = language_context(&capture, vec![], values, vec![]);
    sweep_work(
        &capture,
        &context,
        &ids,
        HtmlSemanticQuery::TextareaValueMissing,
    )
}

/// `count` `dir=auto` slots of one shadow root, each with an empty assignment.
fn slot_assignment_sweep(count: usize) -> usize {
    let (mut b, d) = big_builder(count + 2);
    let host = html(&mut b, &d, "div");
    let root = shadow_root(&mut b, &d, &host);
    let slots: Vec<_> = (0..count)
        .map(|_| attributed(&mut b, &d, "slot", &[("dir", "auto")]))
        .collect();
    for slot in &slots {
        b.establish_children(slot, &[]).unwrap();
    }
    b.establish_children(&root, &slots).unwrap();
    let capture = b.finish().unwrap();
    let ids: Vec<_> = slots
        .iter()
        .map(|slot| capture.subject_id(slot).unwrap())
        .collect();
    let assignments = ids
        .iter()
        .map(|id| HtmlSlotAssignedNodes::new(id.clone(), Some(vec![])))
        .collect();
    let context = language_context(&capture, vec![], vec![], assignments);
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::Directionality)
}

#[test]
fn a_language_sweep_finds_each_document_fallback_in_constant_work() {
    assert_doubling_at_most_2_2(
        "language fallback sweep",
        &DOUBLINGS,
        language_fallback_sweep,
    );
}
#[test]
fn a_current_value_sweep_finds_each_control_value_in_constant_work() {
    assert_doubling_at_most_2_2("current value sweep", &DOUBLINGS, current_value_sweep);
}
#[test]
fn a_directionality_sweep_finds_each_slot_assignment_in_constant_work() {
    assert_doubling_at_most_2_2(
        "slot assignment sweep",
        &[500, 1000, 2000, 4000],
        slot_assignment_sweep,
    );
}

fn inheritance_context(
    capture: &HtmlCapture,
    design_modes: Vec<HtmlDocumentDesignModeFact>,
    spellcheck_defaults: Vec<HtmlSpellcheckDefaultFact>,
) -> HtmlSemanticContext {
    let revision =
        HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 1, 1);
    let records = design_modes.capacity() + spellcheck_defaults.capacity();
    let facts = HtmlGlobalInheritanceFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        design_modes,
        spellcheck_defaults,
        HtmlGlobalInheritanceLimits::new(records, 1 << 40),
    )
    .unwrap();
    HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_global_inheritance(revision, Some(facts)),
        0,
    )
    .unwrap()
}

fn design_mode_sweep(count: usize) -> usize {
    let (capture, documents, divs) = document_page(count);
    let modes = documents
        .iter()
        .map(|d| HtmlDocumentDesignModeFact::new(d.clone(), HtmlDesignMode::Disabled))
        .collect();
    let context = inheritance_context(&capture, modes, vec![]);
    sweep_work(&capture, &context, &divs, HtmlSemanticQuery::EditingHost)
}

fn spellcheck_default_sweep(count: usize) -> usize {
    let (capture, ids) = flat_elements("div", count);
    let defaults = ids
        .iter()
        .map(|id| {
            HtmlSpellcheckDefaultFact::new(
                id.clone(),
                Some(HtmlSpellcheckDefaultBehavior::TrueByDefault),
            )
        })
        .collect();
    let context = inheritance_context(&capture, vec![], defaults);
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::Spellcheck)
}

#[test]
fn a_design_mode_sweep_finds_each_document_in_constant_work() {
    assert_doubling_at_most_2_2("design mode sweep", &DOUBLINGS, design_mode_sweep);
}
#[test]
fn a_spellcheck_sweep_finds_each_default_in_constant_work() {
    assert_doubling_at_most_2_2(
        "spellcheck default sweep",
        &DOUBLINGS,
        spellcheck_default_sweep,
    );
}

fn microdata_context(
    capture: &HtmlCapture,
    url: HtmlMicrodataProviderRevision,
    vocabulary: HtmlMicrodataProviderRevision,
    urls: Vec<HtmlMicrodataUrlFact>,
    types: Vec<HtmlMicrodataTypeFact>,
    properties: Vec<HtmlMicrodataPropertyFact>,
) -> HtmlSemanticContext {
    let revision = HtmlMicrodataRevision::new(Some(url), Some(vocabulary));
    let records = urls.len() + types.len() + properties.len();
    let facts = HtmlMicrodataFacts::new(
        capture,
        revision.clone(),
        urls,
        types,
        properties,
        records,
        1 << 20,
    )
    .unwrap();
    HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_microdata(revision, Some(facts)),
        0,
    )
    .unwrap()
}
fn provider() -> HtmlMicrodataProviderRevision {
    HtmlMicrodataProviderRevision::new(HtmlMicrodataProviderId::fresh(), 1, 1)
}
fn absolute_url(
    capture: &HtmlCapture,
    revision: &HtmlMicrodataProviderRevision,
    subject: &HtmlCaptureSubjectId,
    source: HtmlMicrodataRawSource,
) -> HtmlMicrodataUrlFact {
    let operation = HtmlMicrodataUrlOperation::AbsoluteAuthorToken(revision.clone());
    HtmlMicrodataUrlFact::new(
        HtmlMicrodataRawBinding::new(capture, subject.clone(), source, 1 << 20).unwrap(),
        operation.clone(),
        Some(operation),
        Some(HtmlMicrodataUrlResult::AuthorSyntax(true)),
    )
    .unwrap()
}

/// `count` items with one item type each, with a URL fact and a type fact each.
fn microdata_type_sweep(count: usize) -> usize {
    let (mut b, d) = big_builder(count);
    let items: Vec<_> = (0..count)
        .map(|_| {
            attributed(
                &mut b,
                &d,
                "div",
                &[("itemscope", ""), ("itemtype", "https://vocab.example/a")],
            )
        })
        .collect();
    for item in &items {
        b.establish_children(item, &[]).unwrap();
    }
    b.establish_children(&d, &items).unwrap();
    let capture = b.finish().unwrap();
    let ids: Vec<_> = items
        .iter()
        .map(|item| capture.subject_id(item).unwrap())
        .collect();
    let (url, vocabulary) = (provider(), provider());
    let vocabulary_id = HtmlMicrodataVocabularyId::fresh();
    let urls = ids
        .iter()
        .map(|id| absolute_url(&capture, &url, id, HtmlMicrodataRawSource::ItemTypeToken(0)))
        .collect();
    let types = ids
        .iter()
        .map(|id| {
            HtmlMicrodataTypeFact::new(
                HtmlMicrodataRawBinding::new(
                    &capture,
                    id.clone(),
                    HtmlMicrodataRawSource::ItemTypeToken(0),
                    1 << 20,
                )
                .unwrap(),
                vocabulary.clone(),
                Some(true),
                Some(vocabulary_id.clone()),
                Some(true),
            )
            .unwrap()
        })
        .collect();
    let context = microdata_context(&capture, url, vocabulary, urls, types, vec![]);
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::MicrodataAuthor)
}

/// The work that the URL fact and the permission fact of each of `count` names
/// add to one property element naming `count` properties of the item around it.
/// The names themselves cost a little more than linear work whatever the facts,
/// so the difference with the same query without facts isolates the lookups.
fn microdata_property_fact_work(count: usize) -> usize {
    microdata_property_names(count, true) - microdata_property_names(count, false)
}
fn microdata_property_names(count: usize, with_facts: bool) -> usize {
    let names: Vec<String> = (0..count).map(|n| format!("n{n}")).collect();
    let raw = names.join(" ");
    let (mut b, d) = big_builder(4);
    let item = attributed(&mut b, &d, "div", &[("itemscope", "")]);
    let property = attributed(&mut b, &d, "span", &[("itemprop", raw.as_str())]);
    b.establish_children(&property, &[]).unwrap();
    b.establish_children(&item, std::slice::from_ref(&property))
        .unwrap();
    b.establish_children(&d, std::slice::from_ref(&item))
        .unwrap();
    let capture = b.finish().unwrap();
    let (item, property) = (
        capture.subject_id(&item).unwrap(),
        capture.subject_id(&property).unwrap(),
    );
    let (url, vocabulary) = (provider(), provider());
    let source = |n: usize| HtmlMicrodataRawSource::ItemPropToken(n);
    let facts = if with_facts { count } else { 0 };
    let urls = (0..facts)
        .map(|n| absolute_url(&capture, &url, &property, source(n)))
        .collect();
    let properties = (0..facts)
        .map(|n| {
            HtmlMicrodataPropertyFact::new(
                &capture,
                HtmlMicrodataRawBinding::new(&capture, property.clone(), source(n), 1 << 20)
                    .unwrap(),
                vocabulary.clone(),
                item.clone(),
                None,
                HtmlMicrodataPropertyPermission::new(
                    Some(true),
                    Some(true),
                    Some(false),
                    Some(true),
                ),
                1 << 20,
            )
            .unwrap()
        })
        .collect();
    let context = microdata_context(&capture, url, vocabulary, urls, vec![], properties);
    sweep_work(
        &capture,
        &context,
        std::slice::from_ref(&property),
        HtmlSemanticQuery::MicrodataAuthor,
    )
}

#[test]
fn a_microdata_type_sweep_finds_each_url_and_type_fact_in_constant_work() {
    assert_doubling_at_most_2_2("microdata type sweep", &DOUBLINGS, microdata_type_sweep);
}
#[test]
fn a_property_with_many_names_finds_each_url_and_permission_fact_in_constant_work() {
    assert_doubling_at_most_2_2(
        "microdata property names",
        &[100, 200, 400, 800],
        microdata_property_fact_work,
    );
}

/// A document of `items` untyped microdata items, each holding one property
/// element, and the item and property subjects in document order.
fn microdata_page(
    items: usize,
) -> (
    HtmlCapture,
    Vec<HtmlCaptureSubjectId>,
    Vec<HtmlCaptureSubjectId>,
) {
    let (mut b, d) = big_builder(2 * items);
    let scopes: Vec<_> = (0..items)
        .map(|_| attributed(&mut b, &d, "div", &[("itemscope", "")]))
        .collect();
    let properties: Vec<_> = (0..items)
        .map(|_| attributed(&mut b, &d, "span", &[("itemprop", "name")]))
        .collect();
    for (scope, property) in scopes.iter().zip(&properties) {
        b.establish_children(property, &[]).unwrap();
        b.establish_children(scope, std::slice::from_ref(property))
            .unwrap();
    }
    b.establish_children(&d, &scopes).unwrap();
    let capture = b.finish().unwrap();
    let ids = |nodes: &[surgeist_html_semantics::HtmlCaptureBuilderSubject]| {
        nodes
            .iter()
            .map(|node| capture.subject_id(node).unwrap())
            .collect::<Vec<_>>()
    };
    let (scopes, properties) = (ids(&scopes), ids(&properties));
    (capture, scopes, properties)
}

/// A batch of `queries` of the same kind over distinct elements of one tree
/// builds the tree's item graph once: it charges less than twice the work of
/// the first query alone, where a graph built for each query charges about
/// `queries` times as much.
fn assert_graph_is_built_once(
    label: &str,
    query: HtmlSemanticQuery,
    pick: impl Fn(&[HtmlCaptureSubjectId], &[HtmlCaptureSubjectId]) -> Vec<HtmlCaptureSubjectId>,
) {
    let (capture, items, properties) = microdata_page(100);
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let subjects = pick(&items, &properties);
    let one = sweep_work(&capture, &context, &subjects[..1], query);
    let eight = sweep_work(&capture, &context, &subjects[..8], query);
    assert!(
        eight < 2 * one,
        "{label}: one query charged {one} but a batch of eight charged {eight}"
    );
}

#[test]
fn item_queries_of_one_tree_share_its_item_graph() {
    assert_graph_is_built_once("item", HtmlSemanticQuery::MicrodataItem, |items, _| {
        items.to_vec()
    });
}
#[test]
fn author_queries_of_one_tree_share_its_item_graph() {
    assert_graph_is_built_once(
        "author",
        HtmlSemanticQuery::MicrodataAuthor,
        |_, properties| properties.to_vec(),
    );
}

/// Keeping a tree's graph for the rest of a batch must not raise the scratch a
/// batch needs: eight graph queries of one tree run within the scratch cap that
/// admits one, and the batch releases everything when it ends.
#[test]
fn a_held_item_graph_adds_nothing_to_the_scratch_peak_and_is_released() {
    let (capture, items, _) = microdata_page(20);
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let requests = |count: usize| -> Vec<HtmlSemanticRequest> {
        items[..count]
            .iter()
            .map(|item| HtmlSemanticRequest::new(item.clone(), HtmlSemanticQuery::MicrodataItem))
            .collect()
    };
    let run = |limits: HtmlSemanticLimits, count: usize| {
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(10000),
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let report = session.query(&requests(count));
        (report, session.usage())
    };
    let (one, _) = run(roomy_limits(), 1);
    let one = one.unwrap().usage();
    assert!(one.peak_scratch_records() > 0);
    let big = 1 << 40;
    let capped = HtmlSemanticLimits::new(big, big, one.peak_scratch_records(), 0, big, big);
    let (eight, after) = run(capped, 8);
    let eight = eight.unwrap();
    assert_eq!(
        eight.usage().peak_scratch_records(),
        one.peak_scratch_records()
    );
    assert_eq!(eight.usage().scratch_records(), 0);
    assert_eq!(after.scratch_records(), 0);
}

/// `count` custom elements with an established classification each, as not
/// form-associated, listed in reverse so that scanning for one is not cheap.
fn custom_disabledness_sweep(count: usize) -> usize {
    let (capture, ids) = flat_elements("x-widget", count);
    let classifications = ids
        .iter()
        .rev()
        .map(|id| HtmlCustomControlClassification::new(id.clone(), false))
        .collect();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(classifications),
        count,
    )
    .unwrap();
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::Disabledness)
}

#[test]
fn a_custom_disabledness_sweep_finds_each_classification_in_constant_work() {
    assert_doubling_at_most_2_2(
        "custom disabledness sweep",
        &DOUBLINGS,
        custom_disabledness_sweep,
    );
}

fn participation_sweep(count: usize) -> usize {
    let (capture, ids) = flat_elements("li", count);
    let revision = HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        1,
        2,
        HtmlParticipationProducerId::fresh(),
        3,
        4,
    );
    // Listed in the reverse order, so that scanning for a subject is not cheap.
    let facts = ids
        .iter()
        .rev()
        .map(|id| HtmlListParticipationFact::new(id.clone(), Some(true), Some(true), Some(true)))
        .collect();
    let participation =
        HtmlListParticipation::new(&capture, revision.clone(), facts, 8 * count).unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![])
            .with_list_participation(revision, Some(participation)),
        0,
    )
    .unwrap();
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::ListOwner)
}

fn output_default_sweep(count: usize) -> usize {
    let (capture, ids) = flat_elements("output", count);
    let records = ids
        .iter()
        .rev()
        .map(|id| HtmlOutputDefaultFact::new(id.clone(), HtmlOutputDefaultValueOverride::Null))
        .collect();
    let facts = HtmlOutputDefaultFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        records,
        HtmlOutputDefaultLimits::new(8 * count, 1 << 20, 1 << 40),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_output_defaults(facts),
        0,
    )
    .unwrap();
    sweep_work(
        &capture,
        &context,
        &ids,
        HtmlSemanticQuery::OutputDefaultValue,
    )
}

fn media_external_sweep(count: usize) -> usize {
    let (mut b, d) = big_builder(count);
    let styles: Vec<_> = (0..count)
        .map(|_| attributed(&mut b, &d, "style", &[("media", "screen")]))
        .collect();
    for style in &styles {
        b.establish_children(style, &[]).unwrap();
    }
    b.establish_children(&d, &styles).unwrap();
    let capture = b.finish().unwrap();
    let ids: Vec<_> = styles
        .iter()
        .map(|style| capture.subject_id(style).unwrap())
        .collect();
    let revision = HtmlMediaContextRevision::new(
        HtmlMediaGrammarRevision::new(HtmlMediaProducerId::fresh(), 1, 2),
        HtmlMediaEnvironmentRevision::new(HtmlMediaProducerId::fresh(), 3, 4),
    );
    let facts = ids
        .iter()
        .rev()
        .map(|id| {
            HtmlMediaFact::new(
                HtmlMediaAttributeInput::new(&capture, id.clone(), 0).unwrap(),
                None,
                None,
            )
        })
        .collect();
    let facts = HtmlMediaFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        facts,
        HtmlMediaFactsLimits::new(8 * count, 1 << 30, 1 << 40),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_media(revision, Some(facts)),
        0,
    )
    .unwrap();
    sweep_work(
        &capture,
        &context,
        &ids,
        HtmlSemanticQuery::MediaAttributeRules,
    )
}

#[test]
fn a_participation_sweep_finds_each_fact_in_constant_work() {
    assert_doubling_at_most_2_2("participation sweep", &DOUBLINGS, participation_sweep);
}
#[test]
fn an_output_default_sweep_finds_each_record_in_constant_work() {
    assert_doubling_at_most_2_2("output default sweep", &DOUBLINGS, output_default_sweep);
}
#[test]
fn a_media_sweep_finds_each_external_fact_in_constant_work() {
    assert_doubling_at_most_2_2("media external sweep", &DOUBLINGS, media_external_sweep);
}

/// `count` untyped microdata items, all children of the document, swept by item
/// query: the graph of the tree is built once for the batch, so what grows is
/// the type walk of each item, which costs the items and edges above it (none).
fn item_type_sweep(count: usize) -> usize {
    let (mut b, d) = big_builder(count);
    let items: Vec<_> = (0..count)
        .map(|_| attributed(&mut b, &d, "div", &[("itemscope", "")]))
        .collect();
    for item in &items {
        b.establish_children(item, &[]).unwrap();
    }
    b.establish_children(&d, &items).unwrap();
    let capture = b.finish().unwrap();
    let ids: Vec<_> = items
        .iter()
        .map(|item| capture.subject_id(item).unwrap())
        .collect();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    sweep_work(&capture, &context, &ids, HtmlSemanticQuery::MicrodataItem)
}

#[test]
fn an_item_sweep_walks_the_types_above_each_item_in_work_of_that_walk() {
    assert_doubling_at_most_2_2("item type sweep", &DOUBLINGS, item_type_sweep);
}

/// Two chains of `depth` nested elements below the document: a button at the
/// bottom of the first has a reference to a `div` at the bottom of the second.
/// They share only the document, so no ancestor of the button but that one is an
/// ancestor of the target.
fn reference_scope_work(depth: usize) -> usize {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        2 * depth + 10,
        8 * depth + 100,
        100,
        40 * depth + 100_000,
        2 * depth + 10,
        0,
        0,
        16_777_216,
    ));
    let chain = |b: &mut HtmlCaptureBuilder, last: &str| {
        let elements: Vec<_> = (0..depth)
            .map(|n| html(b, &d, if n + 1 == depth { last } else { "div" }))
            .collect();
        for pair in elements.windows(2) {
            b.establish_children(&pair[0], std::slice::from_ref(&pair[1]))
                .unwrap();
        }
        b.establish_children(&elements[depth - 1], &[]).unwrap();
        (elements[0].clone(), elements[depth - 1].clone())
    };
    let (first_top, button) = chain(&mut b, "button");
    let (second_top, target) = chain(&mut b, "div");
    b.establish_children(&d, &[first_top, second_top]).unwrap();
    let capture = b.finish().unwrap();
    let (button, target) = (
        capture.subject_id(&button).unwrap(),
        capture.subject_id(&target).unwrap(),
    );
    let facts = HtmlElementReferenceFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        vec![HtmlElementReferenceFact::new(
            button.clone(),
            HtmlElementReferenceKey::CommandFor,
            HtmlExplicitElementReference::Element(target),
        )],
        HtmlElementReferenceFactsLimits::new(8, 1 << 40),
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
        std::slice::from_ref(&button),
        HtmlSemanticQuery::CommandForElement,
    )
}

#[test]
fn a_reference_scope_check_walks_each_chain_of_ancestors_once() {
    assert_doubling_at_most_2_2(
        "reference scope",
        &[100, 200, 400, 800],
        reference_scope_work,
    );
}
