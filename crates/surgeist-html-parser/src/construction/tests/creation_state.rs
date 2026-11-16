//! The creation state and the common state a published report answers by node, for the elements
//! whose creation gives a state other than the default of their namespace, and for the contexts
//! that hold a node: a template's contents, a fragment parse, a shadow root and a lookup registry
//! other than Null.
//!
//! The oracle is DOM's create-element, as the type of the answer states it: an element of the HTML
//! namespace created with a valid custom element name or with an `is` value has the state
//! `Undefined` and keeps the `is` value it was created with; any other element is `Uncustomized`
//! and keeps an `is` value only if it was created with one. The common scope is the one of the
//! namespace: `Html` for the HTML namespace, `Element` for any other.
use super::*;
use crate::{HtmlFragmentParseRequest, HtmlParseMetadataStep, HtmlSource};
use surgeist_html::construction::HtmlNodeId;
use surgeist_html::elements::native::{HtmlCommonScope, HtmlCustomElementState};
use surgeist_html::parsing::HtmlParserScriptingMode;
use surgeist_html_semantics::*;

pub(super) fn document_report(html: &str) -> crate::HtmlParseReport {
    crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(html),
        crate::parser::tree_builder::tests::limits(100),
    )
    .parse()
    .unwrap()
}

/// The elements of a published report named `local`, in tree-construction order.
pub(super) fn elements(report: &crate::HtmlParseReport, local: &str) -> Vec<HtmlNodeId> {
    let arena = &report.history().storage().view;
    (0..arena.len())
        .filter_map(|position| {
            let record = arena.node_at(position)?;
            match record.kind() {
                ConstructionNodeKind::Element(element)
                    if *element.name().local_name() == DocumentString::from(local) =>
                {
                    Some(record.as_node().ok()?.id())
                }
                _ => None,
            }
        })
        .collect()
}

/// What the report answers by node for the creation state of `id`: its state and its saved `is`
/// value as text.
fn creation(
    report: &crate::HtmlParseReport,
    id: &HtmlNodeId,
) -> Option<(HtmlCustomElementState, Option<String>)> {
    let answer = report.creation(id).unwrap();
    answer.value().map(|creation| {
        (
            creation.state(),
            creation
                .saved_is()
                .map(|value| String::from_utf16(value.code_units()).unwrap()),
        )
    })
}

fn scope(report: &crate::HtmlParseReport, id: &HtmlNodeId) -> Option<HtmlCommonScope> {
    report
        .common_state(id)
        .unwrap()
        .value()
        .map(|common| common.scope())
}

/// The contents of the one template of `report`.
pub(super) fn template_contents(report: &crate::HtmlParseReport) -> HtmlNodeId {
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(record) => {
                if let Some(template) = record.template() {
                    return template.contents().id();
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            _ => panic!("the report holds a template"),
        }
    }
}

fn rows(report: &crate::HtmlParseReport) -> usize {
    report.history().storage().payloads.facts.creation.len()
}

#[test]
fn a_custom_element_name_has_a_row_and_a_plain_name_has_none() {
    let report = document_report("<!doctype html><x-a></x-a><span></span><svg><x-b></x-b></svg>");
    let custom = &elements(&report, "x-a")[0];
    let plain = &elements(&report, "span")[0];
    let foreign = &elements(&report, "x-b")[0];
    assert_eq!(
        creation(&report, custom),
        Some((HtmlCustomElementState::Undefined, None))
    );
    assert_eq!(scope(&report, custom), Some(HtmlCommonScope::Html));
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    assert_eq!(scope(&report, plain), Some(HtmlCommonScope::Html));
    // A name that is valid for a custom element in another namespace than HTML is no custom
    // element, so it holds no row either.
    assert_eq!(
        creation(&report, foreign),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    assert_eq!(scope(&report, foreign), Some(HtmlCommonScope::Element));
    assert_eq!(rows(&report), 1);
}

#[test]
fn an_is_attribute_has_a_row_with_the_value_it_was_created_with() {
    let report =
        document_report("<!doctype html><div is=x-b></div><svg><circle is=q></circle></svg><p>");
    let html = &elements(&report, "div")[0];
    let svg = &elements(&report, "circle")[0];
    let plain = &elements(&report, "p")[0];
    assert_eq!(
        creation(&report, html),
        Some((HtmlCustomElementState::Undefined, Some("x-b".into())))
    );
    // Outside the HTML namespace the element stays uncustomized, and still keeps the value.
    assert_eq!(
        creation(&report, svg),
        Some((HtmlCustomElementState::Uncustomized, Some("q".into())))
    );
    assert_eq!(scope(&report, svg), Some(HtmlCommonScope::Element));
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    assert_eq!(rows(&report), 2);
}

#[test]
fn the_elements_of_a_templates_contents_are_answered_like_any_other() {
    let report = document_report("<!doctype html><template><x-a is=y><b></b></x-a></template>");
    let template = &elements(&report, "template")[0];
    let inner = &elements(&report, "x-a")[0];
    let plain = &elements(&report, "b")[0];
    assert_eq!(
        creation(&report, template),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    assert_eq!(
        creation(&report, inner),
        Some((HtmlCustomElementState::Undefined, Some("y".into())))
    );
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    // The contents of the template are a fragment: no creation state and no common state.
    let contents = template_contents(&report);
    assert_eq!(creation(&report, &contents), None);
    assert_eq!(scope(&report, &contents), None);
    assert_eq!(rows(&report), 1);
}

#[test]
fn nodes_that_are_not_elements_have_no_creation_state() {
    let report = document_report("<!doctype html><!--c-->text<p>");
    let document = report.document().unwrap().root().id();
    assert_eq!(creation(&report, &document), None);
    assert_eq!(scope(&report, &document), None);
    let arena = &report.history().storage().view;
    for position in 0..arena.len() {
        let record = arena.node_at(position).unwrap();
        if !matches!(record.kind(), ConstructionNodeKind::Element(_)) {
            let id = record.as_node().unwrap().id();
            assert_eq!(creation(&report, &id), None, "node {position}");
            assert_eq!(scope(&report, &id), None, "node {position}");
        }
    }
}

/// A fragment parse in the context of a `div`. With `registry` the document and the element name
/// a global registry as their lookup registry, and otherwise the Null registry. With `shadow` the
/// parse allows declarative shadow roots.
pub(super) fn fragment_report(html: &str, registry: bool, shadow: bool) -> crate::HtmlParseReport {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let global = registry.then(|| {
        let id = builder.add_registry(Some(Vec::new())).unwrap();
        builder
            .establish_registry_scope(&id, HtmlRegistryScope::Global)
            .unwrap();
        id
    });
    let input = |global: &Option<_>| match global {
        Some(id) => HtmlCaptureRegistryInput::Registry(std::clone::Clone::clone(id)),
        None => HtmlCaptureRegistryInput::Null,
    };
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            input(&global),
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            input(&global),
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
    HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8(html),
        context,
        crate::parser::tree_builder::tests::limits(100),
        HtmlParserScriptingMode::Inert,
        shadow,
    )
    .unwrap()
    .parse()
    .unwrap()
}

#[test]
fn a_fragment_parse_answers_by_node_and_its_implicit_root_has_the_default() {
    let report = fragment_report("<x-a is=z></x-a><i></i>", false, false);
    let custom = &elements(&report, "x-a")[0];
    let plain = &elements(&report, "i")[0];
    assert_eq!(
        creation(&report, custom),
        Some((HtmlCustomElementState::Undefined, Some("z".into())))
    );
    assert_eq!(scope(&report, custom), Some(HtmlCommonScope::Html));
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    // The implicit `html` root the parse creates in its own parser Document.
    let root = &elements(&report, "html")[0];
    assert_eq!(
        creation(&report, root),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    assert_eq!(scope(&report, root), Some(HtmlCommonScope::Html));
    assert_eq!(rows(&report), 1);
}

#[test]
fn an_element_whose_registry_is_not_null_has_no_common_state_and_keeps_its_creation_state() {
    let report = fragment_report("<x-a></x-a><i></i>", true, false);
    let custom = &elements(&report, "x-a")[0];
    let plain = &elements(&report, "i")[0];
    // The registry of the context is the registry of every element created in it, so the
    // closed common state, which holds with the Null registry only, is not answered.
    assert_eq!(scope(&report, custom), None);
    assert_eq!(scope(&report, plain), None);
    assert_eq!(
        creation(&report, custom),
        Some((HtmlCustomElementState::Undefined, None))
    );
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    // The record of an element that has a row agrees with the answer by node.
    let mut cursor = report.metadata();
    let mut seen = 0;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(record) if record.creation().is_some() => {
                assert_eq!(record.node().id(), *custom);
                assert!(record.common_state().is_none());
                seen += 1;
            }
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            _ => {}
        }
    }
    assert_eq!(seen, 1);
    // Each element, with the implicit root, is a node the effects bind to its registry.
    let effects = report.effects().unwrap().unwrap();
    let mut bound = Vec::new();
    for slot in 0..effects.registry_slots() {
        if let Some(binding) = effects.registry(slot).unwrap() {
            assert!(matches!(
                binding.registry(),
                crate::HtmlParseRegistry::Captured(_)
            ));
            bound.push(binding.node().id());
        }
    }
    let root = elements(&report, "html")[0];
    for id in [&root, custom, plain] {
        assert!(bound.contains(id), "an element has no registry binding");
    }
}

#[test]
fn the_contents_of_a_shadow_root_are_a_fragment_and_their_elements_are_answered() {
    let report = fragment_report(
        "<template shadowrootmode=open><x-a is=s></x-a><b></b></template>",
        false,
        true,
    );
    let custom = &elements(&report, "x-a")[0];
    let plain = &elements(&report, "b")[0];
    assert_eq!(
        creation(&report, custom),
        Some((HtmlCustomElementState::Undefined, Some("s".into())))
    );
    assert_eq!(
        creation(&report, plain),
        Some((HtmlCustomElementState::Uncustomized, None))
    );
    let effects = report.effects().unwrap().unwrap();
    let mut contents = None;
    for index in 0..effects.len() {
        if let Some(crate::HtmlParseEffect::AttachShadow {
            contents: found, ..
        }) = effects.effect(index).unwrap()
        {
            contents = Some(found.id());
        }
    }
    let contents = contents.expect("the template attaches a shadow root");
    assert_eq!(creation(&report, &contents), None);
    assert_eq!(scope(&report, &contents), None);
}

#[test]
fn a_node_of_another_result_is_foreign_to_both_reads() {
    let one = document_report("<!doctype html><p>");
    let other = document_report("<!doctype html><p>");
    let foreign = elements(&other, "p")[0];
    assert_eq!(
        one.creation(&foreign).err(),
        Some(crate::HtmlParseNodeReadError::Foreign)
    );
    assert_eq!(
        one.common_state(&foreign).err(),
        Some(crate::HtmlParseNodeReadError::Foreign)
    );
}
