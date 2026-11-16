//! What a plain element retains for its nonce, and the nonce a published report answers by node.
use super::*;

/// The registry rows of a parse of an ordinary Document of `elements` plain `span` elements, with
/// the detail the request records, or `None` when this build cannot record that detail.
fn registry_rows(elements: usize, detail: crate::DetailSet) -> Option<usize> {
    let source = format!("<!doctype html>{}", "<span></span>".repeat(elements));
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source.as_str()),
        g2_d2::corpus_limits(),
    )
    .with_detail(detail)
    .ok()?;
    let report = request.parse().unwrap();
    Some(report.history().storage().payloads.values.len())
}

/// The registry rows one plain element holds: with the detail of causes recorded the cause of its
/// creation, and none otherwise. The strings of its name and namespace are the record's own. The
/// element has no nonce: its nonce is the empty nonce every element of its namespaces has, which
/// no row holds. The oracle is the families an element has independently of its nonce.
#[test]
fn a_plain_element_holds_no_registry_row_for_its_nonce() {
    for (detail, per_element) in [(crate::DetailSet::NONE, 0), (crate::DetailSet::ALL, 1)] {
        if !g2_d2::details().contains(&detail) {
            continue;
        }
        let Some(empty) = registry_rows(0, detail) else {
            continue;
        };
        for elements in [1, 16, 64] {
            let rows = registry_rows(elements, detail).unwrap();
            assert_eq!(
                rows - empty,
                elements * per_element,
                "{elements} plain elements, {detail:?}"
            );
        }
    }
}

use super::g2_d2::each_corpus_parse;
use surgeist_html::values::{DocumentString, Namespace};

#[derive(Default)]
struct Tally {
    parses: usize,
    nodes: usize,
    elements: usize,
    with_row: usize,
    plain: usize,
    not_nonce_elements: usize,
    not_elements: usize,
    root_rows: usize,
    root_rows_set: usize,
    attribute_rows: usize,
    failures: Vec<String>,
}

/// The nonce of every record of a report that has one, by node.
fn nonce_records(
    report: &crate::HtmlParseReport,
) -> std::collections::HashMap<surgeist_html::construction::HtmlNodeId, DocumentString> {
    let mut records = std::collections::HashMap::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            crate::HtmlParseMetadataStep::Record(record) => {
                if let Some(value) = record.cryptographic_nonce() {
                    assert!(
                        records.insert(record.node().id(), value.clone()).is_none(),
                        "two nonce records for one node"
                    );
                }
            }
            crate::HtmlParseMetadataStep::Skipped => {}
            crate::HtmlParseMetadataStep::Complete => return records,
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// Whether the units are the ASCII literal.
fn is(units: &[u16], literal: &str) -> bool {
    units.len() == literal.len()
        && units
            .iter()
            .zip(literal.bytes())
            .all(|(&unit, byte)| unit == u16::from(byte))
}

fn check(report: &crate::HtmlParseReport, label: &str, tally: &mut Tally) {
    let storage = report.history().storage();
    let arena = &storage.view;
    let records = nonce_records(report);
    tally.parses += 1;
    let mut fail = |message: String| tally.failures.push(format!("{label}: {message}"));
    if records.len() != storage.payloads.facts.nonces.len() {
        fail(format!(
            "{} records for {} rows",
            records.len(),
            storage.payloads.facts.nonces.len()
        ));
    }
    // The bound DOM's row map relies on: every id the result issued has a position below the
    // node count of the result's usage, which is the number of records.
    let nodes = match (report.document(), report.fragment()) {
        (Some(document), _) => document.usage().nodes(),
        (None, Some(fragment)) => fragment.usage().nodes(),
        (None, None) => 0,
    };
    if nodes != arena.len() {
        fail(format!(
            "usage counts {nodes} nodes for {} records",
            arena.len()
        ));
    }
    let mut counts = [0_usize; 9];
    for position in 0..arena.len() {
        counts[0] += 1;
        let record = arena.node_at(position).unwrap();
        let id = record.as_node().unwrap().id();
        if id.position() != position || id.position() >= nodes {
            fail(format!(
                "node {position} has the position {}",
                id.position()
            ));
        }
        let answer = report.cryptographic_nonce(&id).unwrap();
        let ConstructionNodeKind::Element(element) = record.kind() else {
            counts[5] += 1;
            if answer.value().is_some() {
                fail(format!("node {position} is not an element but has a nonce"));
            }
            continue;
        };
        counts[1] += 1;
        let namespace = element.name().namespace();
        let nonce_element =
            [Namespace::html(), Namespace::svg(), Namespace::mathml()].contains(namespace);
        let Some(value) = *answer.value() else {
            counts[4] += 1;
            if nonce_element {
                fail(format!("element {position} has no nonce"));
            }
            continue;
        };
        if !nonce_element {
            fail(format!(
                "element {position} of another namespace has a nonce"
            ));
        }
        // The oracle for a row, independent of the parse's decision: the element has a
        // no-namespace `nonce` attribute, or is an HTML `html` or `body`.
        let attribute = matches!(
            record.as_node().unwrap().kind(),
            surgeist_html::construction::HtmlNodeKind::Element(published)
                if published.attributes().iter().any(|attribute| {
                    attribute.name().namespace().uri().is_none()
                        && is(attribute.name().local_name().code_units(), "nonce")
                })
        );
        let root = *namespace == Namespace::html()
            && (is(element.name().local_name().code_units(), "html")
                || is(element.name().local_name().code_units(), "body"));
        match records.get(&id) {
            Some(held) => {
                counts[2] += 1;
                if value != held {
                    fail(format!("element {position} differs from its record"));
                }
                if !(attribute || root) {
                    fail(format!(
                        "element {position} has a row and no reason for one"
                    ));
                }
                if root {
                    counts[6] += 1;
                    if !held.is_empty() {
                        counts[7] += 1;
                    }
                }
                if attribute {
                    counts[8] += 1;
                }
            }
            None => {
                counts[3] += 1;
                if !value.is_empty() {
                    fail(format!("plain element {position} has a nonce"));
                }
                if attribute || root {
                    fail(format!(
                        "element {position} has a nonce attribute or is a root and no row"
                    ));
                }
            }
        }
    }
    tally.nodes += counts[0];
    tally.elements += counts[1];
    tally.with_row += counts[2];
    tally.plain += counts[3];
    tally.not_nonce_elements += counts[4];
    tally.not_elements += counts[5];
    tally.root_rows += counts[6];
    tally.root_rows_set += counts[7];
    tally.attribute_rows += counts[8];
}

/// Sources with the elements the retained corpus holds none of: nonces on ordinary, foreign and
/// ordered-creation elements, a nonce merged onto `html` and `body` by a second tag, a template's
/// contents, a clone made by a select, an attribute that is not a nonce, and a repeated nonce.
// One style element in each source: a contextual parse of two consecutive style elements trips the
// style order assertion, which is #2888 and holds at the commit this test started from.
const NONCES: [&str; 13] = [
    "<!doctype html><div nonce=a></div><span></span><script nonce=b></script>",
    "<!doctype html><html><body><html nonce=m><body nonce=n><p>",
    "<!doctype html><html nonce=x><body nonce=y><html nonce=q><p nonce=z>",
    "<!doctype html><svg nonce=s><g nonce=t></g><g></g></svg><math nonce=u><mi></mi></math>",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
     <option selected><span nonce=c>t</span><i></i></option></select>",
    "<!doctype html><template><i nonce=a></i><b></b></template>",
    "<!doctype html><input type=hidden nonce=a><input><details nonce=d></details>",
    "<!doctype html><style nonce=k></style><p></p>",
    "<!doctype html><div></div><style></style>",
    "<!doctype html><svg xlink:nonce=a></svg><div NONCE=A></div><div nonce=a nonce=b></div>",
    "<!doctype html><div nonce></div><div nonce=\"\"></div>",
    "<!doctype html><table><tr><td nonce=t></td></tr></table><p nonce=p>",
    "<!doctype html><a nonce=1><b nonce=2><i nonce=3>x</a></b>",
];

#[test]
fn the_nonce_a_report_answers_by_node_is_its_row_or_the_empty_nonce() {
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
    let corpus = (tally.parses, tally.nodes, tally.with_row);
    for source in NONCES {
        for detail in g2_d2::details() {
            let label = format!("{source} detail={detail:?}");
            let report = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source),
                g2_d2::corpus_limits(),
            )
            .with_detail(detail)
            .unwrap()
            .parse()
            .unwrap();
            check(&report, &label, &mut tally);
            // The fragment parse of the same markup, in the context of a `div`.
            let context = g2_d2::fragment_context("div").unwrap();
            let fragment = source.strip_prefix("<!doctype html>").unwrap_or(source);
            let report = crate::HtmlFragmentParseRequest::new(
                crate::HtmlSource::utf8(fragment),
                context,
                g2_d2::corpus_limits(),
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
        "D3 nonce parses {} (corpus {}) nodes {} (corpus {}) elements {} rows {} (corpus {}) \
         plain {} other-namespace {} not-elements {} root-rows {} root-rows-set {} \
         attribute-rows {} failures {}",
        tally.parses,
        corpus.0,
        tally.nodes,
        corpus.1,
        tally.elements,
        tally.with_row,
        corpus.2,
        tally.plain,
        tally.not_nonce_elements,
        tally.not_elements,
        tally.root_rows,
        tally.root_rows_set,
        tally.attribute_rows,
        tally.failures.len()
    );
    for line in tally.failures.iter().take(30) {
        eprintln!("{line}");
    }
    assert!(tally.failures.is_empty());
    assert_eq!(corpus.0, 1_767 * g2_d2::details().len());
    assert_eq!(
        tally.parses,
        corpus.0 + 2 * g2_d2::details().len() * NONCES.len()
    );
    // Every node is an element or is not, and every element has a row, is plain, or is of a
    // namespace with no nonce.
    assert_eq!(tally.elements + tally.not_elements, tally.nodes);
    assert_eq!(
        tally.with_row + tally.plain + tally.not_nonce_elements,
        tally.elements
    );
    // The extra sources hold nonces on ordinary elements and a nonce merged onto `html` and
    // `body`, and the corpus holds more plain elements than rows, most of which are those of `html` and `body`.
    assert!(tally.attribute_rows > 0);
    assert!(tally.root_rows > 0);
    assert!(tally.root_rows_set > 0);
    assert!(tally.plain > tally.with_row);
}
