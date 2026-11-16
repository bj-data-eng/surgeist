//! The creation state and the common state a published report answers by node, for every node of
//! every retained html5lib tree-construction case, with detail off and on.
//!
//! An element that has a record answers what its record holds. An element that has none answers
//! the creation state of its namespace: no custom element state, no saved `is` value, and the
//! common scope the classification of its name gives. A node that is not an element answers none.
use super::g2_d2::each_corpus_parse;
use super::*;
use surgeist_html::elements::native::{HtmlCommonScope, HtmlCustomElementState};
use surgeist_html::values::Namespace;

#[derive(Default)]
struct Tally {
    parses: usize,
    nodes: usize,
    elements: usize,
    with_record: usize,
    plain: usize,
    custom_state: usize,
    saved_is: usize,
    not_elements: usize,
    failures: Vec<String>,
}

/// The records of a report that hold a creation state, by node position.
fn creation_records(
    report: &crate::HtmlParseReport,
) -> std::collections::HashMap<
    surgeist_html::construction::HtmlNodeId,
    (
        HtmlCustomElementState,
        Option<surgeist_html::values::DocumentString>,
        Option<HtmlCommonScope>,
    ),
> {
    let mut records = std::collections::HashMap::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            crate::HtmlParseMetadataStep::Record(record) => {
                if let Some((state, saved)) = record.creation() {
                    let scope = record.common_state().map(|common| common.scope());
                    records.insert(record.node().id(), (state, saved.cloned(), scope));
                }
            }
            crate::HtmlParseMetadataStep::Skipped => {}
            crate::HtmlParseMetadataStep::Complete => return records,
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// The scope the classification of the element's name gives, which does not read the namespace
/// that the report's default reads.
fn classified_scope(name: &surgeist_html::values::QualifiedName) -> HtmlCommonScope {
    surgeist_html::bounded::native::element_classification_with(name, |_| {
        Ok::<(), std::convert::Infallible>(())
    })
    .unwrap()
    .common_scope()
}

fn check(report: &crate::HtmlParseReport, label: &str, tally: &mut Tally) {
    let storage = report.history().storage();
    let arena = &storage.view;
    let records = creation_records(report);
    tally.parses += 1;
    let mut fail = |message: String| tally.failures.push(format!("{label}: {message}"));
    if records.len() != storage.payloads.facts.creation.len() {
        fail(format!(
            "{} records for {} rows",
            records.len(),
            storage.payloads.facts.creation.len()
        ));
    }
    let mut counts = [0_usize; 7];
    for position in 0..arena.len() {
        counts[0] += 1;
        let record = arena.node_at(position).unwrap();
        let id = record.as_node().unwrap().id();
        let creation = report.creation(&id).unwrap();
        let common = report.common_state(&id).unwrap();
        let ConstructionNodeKind::Element(element) = record.kind() else {
            counts[5] += 1;
            if creation.value().is_some() || common.value().is_some() {
                fail(format!(
                    "node {position} is not an element but has an answer"
                ));
            }
            continue;
        };
        counts[1] += 1;
        let Some(creation) = *creation.value() else {
            fail(format!("element {position} has no creation state"));
            continue;
        };
        let (state, saved) = (creation.state(), creation.saved_is());
        // The closed profile of these requests has the Null registry for every element.
        let Some(common) = *common.value() else {
            fail(format!("element {position} has no common state"));
            continue;
        };
        match records.get(&id) {
            Some((record_state, record_saved, record_scope)) => {
                counts[2] += 1;
                if (state, saved) != (*record_state, record_saved.as_ref())
                    || Some(common.scope()) != *record_scope
                {
                    fail(format!("element {position} differs from its record"));
                }
                // A record is held only where the creation state is not the default of the
                // namespace, so each one has a custom element state or a saved is value.
                if state == HtmlCustomElementState::Uncustomized && saved.is_none() {
                    fail(format!("element {position} has a record of the default"));
                }
                if state != HtmlCustomElementState::Uncustomized {
                    counts[4] += 1;
                }
                if saved.is_some() {
                    counts[6] += 1;
                }
            }
            None => {
                counts[3] += 1;
                if state != HtmlCustomElementState::Uncustomized || saved.is_some() {
                    fail(format!("plain element {position} has a state"));
                }
                let namespace_scope = if *element.name().namespace() == Namespace::html() {
                    HtmlCommonScope::Html
                } else {
                    HtmlCommonScope::Element
                };
                if common.scope() != classified_scope(element.name())
                    || common.scope() != namespace_scope
                {
                    fail(format!("plain element {position} has the wrong scope"));
                }
            }
        }
    }
    tally.nodes += counts[0];
    tally.elements += counts[1];
    tally.with_record += counts[2];
    tally.plain += counts[3];
    tally.custom_state += counts[4];
    tally.not_elements += counts[5];
    tally.saved_is += counts[6];
}

/// Sources with the elements the retained corpus holds none of: custom element names, `is`
/// attributes in both namespaces, a template's contents and a clone made by a select.
const CUSTOM: [&str; 6] = [
    "<!doctype html><x-a></x-a><p>a</p><span></span>",
    "<!doctype html><div is=b></div><svg><circle is=q></circle><x-c></x-c></svg>",
    "<!doctype html><template><x-a is=y><b></b></x-a></template>",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
     <option selected><x-b is=c>t</x-b><i></i></option></select>",
    "<!doctype html><body is=z><x-a x-b=c is=d></x-a>",
    "<!doctype html><table><x-a is=e></x-a><tr><td><x-f></x-f></table>",
];

#[test]
fn the_creation_state_a_report_answers_by_node_is_its_record_or_the_default_of_its_namespace() {
    let tally = std::cell::RefCell::new(Tally::default());
    each_corpus_parse(
        |report, label| check(report, label, &mut tally.borrow_mut()),
        |label| {
            tally
                .borrow_mut()
                .failures
                .push(format!("{label}: parse failed"))
        },
    );
    let mut tally = tally.into_inner();
    let corpus = (tally.parses, tally.nodes, tally.with_record);
    for source in CUSTOM {
        for detail in [crate::DetailSet::NONE, crate::DetailSet::ALL] {
            let label = format!("{source} detail={detail:?}");
            let report = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                super::g2_d2::corpus_limits(),
            )
            .with_detail(detail)
            .unwrap()
            .parse()
            .unwrap();
            check(&report, &label, &mut tally);
            // The fragment parse of the same markup, in the context of a `div`.
            let context = super::g2_d2::fragment_context("div").unwrap();
            let fragment = source.strip_prefix("<!doctype html>").unwrap_or(source);
            let report = crate::HtmlFragmentParseRequest::new(
                crate::HtmlSource::utf8(fragment),
                context,
                super::g2_d2::corpus_limits(),
            )
            .unwrap()
            .with_detail(detail)
            .unwrap()
            .parse()
            .unwrap();
            check(&report, &format!("fragment {label}"), &mut tally);
        }
    }
    eprintln!(
        "D3 parses {} (corpus {}) nodes {} (corpus {}) elements {} with-record {} (corpus {}) \
         plain {} custom-state {} saved-is {} not-elements {} failures {}",
        tally.parses,
        corpus.0,
        tally.nodes,
        corpus.1,
        tally.elements,
        tally.with_record,
        corpus.2,
        tally.plain,
        tally.custom_state,
        tally.saved_is,
        tally.not_elements,
        tally.failures.len()
    );
    for line in tally.failures.iter().take(30) {
        eprintln!("{line}");
    }
    assert!(tally.failures.is_empty());
    // The corpus is pinned: one parse with details off and one with all of them for each case,
    // and the extra sources add a document parse and a fragment parse in each of the two modes.
    assert_eq!(corpus.0, 3_534);
    assert_eq!(tally.parses, corpus.0 + 4 * CUSTOM.len());
    // Every node is an element or is not, and every element has a record or is plain.
    assert_eq!(tally.elements + tally.not_elements, tally.nodes);
    assert_eq!(tally.with_record + tally.plain, tally.elements);
    // The extra sources hold custom element names and `is` attributes, and the corpus holds far
    // more plain elements than records.
    assert!(tally.with_record > corpus.2);
    assert!(tally.custom_state > 0);
    assert!(tally.saved_is > 0);
    assert!(tally.plain > 10 * tally.with_record);
}
