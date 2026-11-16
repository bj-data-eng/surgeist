//! The template metadata, the shadow state and the creation cause a published report answers for
//! every node of every retained html5lib tree-construction case and of sources with the nodes the
//! corpus holds none of, with detail off and on.
//!
//! The oracles are the trees themselves: a template has the metadata of the element whose contents
//! the record names, a shadow root's contents have the state of an attached shadow root, and a
//! parse that records its causes has a creation cause for each node it created. None is read
//! from the table that holds the row.
use super::g2_d2::each_corpus_parse;
use super::*;
use surgeist_html::construction::{ConstructionNodeKind, HtmlNodeId};

#[derive(Default)]
struct Tally {
    parses: usize,
    nodes: usize,
    templates: usize,
    template_markers: usize,
    shadows: usize,
    shadow_clones: usize,
    detailed: usize,
    creation_rows: usize,
    failures: Vec<String>,
}

/// What the report answers for the templates and the shadow roots, by node.
struct Records {
    templates: std::collections::HashMap<HtmlNodeId, Vec<TemplateAnswer>>,
    shadows: std::collections::HashMap<HtmlNodeId, Vec<(bool, bool, bool)>>,
}

struct TemplateAnswer {
    contents: HtmlNodeId,
    markers: bool,
}

fn records(report: &crate::HtmlParseReport) -> Records {
    let mut found = Records {
        templates: Default::default(),
        shadows: Default::default(),
    };
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            crate::HtmlParseMetadataStep::Record(record) => {
                let id = record.node().id();
                if let Some(template) = record.template() {
                    found.templates.entry(id).or_default().push(TemplateAnswer {
                        contents: template.contents().id(),
                        markers: template.target().is_some()
                            || template.start().is_some()
                            || template.end().is_some(),
                    });
                }
                if let Some(state) = record.shadow() {
                    found.shadows.entry(id).or_default().push(state);
                }
            }
            crate::HtmlParseMetadataStep::Skipped => {}
            crate::HtmlParseMetadataStep::Complete => return found,
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// A build without the `detail` feature records no causes.
#[cfg(not(feature = "detail"))]
fn creation_rows(
    _: &crate::HtmlParseReport,
) -> Option<std::collections::HashMap<HtmlNodeId, usize>> {
    None
}

/// The creation rows of a report that records its causes, by node: how many rows each node has.
#[cfg(feature = "detail")]
fn creation_rows(
    report: &crate::HtmlParseReport,
) -> Option<std::collections::HashMap<HtmlNodeId, usize>> {
    let associations = report.detail()?.associations()?;
    let mut rows = std::collections::HashMap::new();
    for index in 0..associations.len() {
        let row = associations.row(index).unwrap().unwrap();
        if row.relation() != crate::HtmlParseAssociationRelation::Creation {
            continue;
        }
        let crate::HtmlParseDestination::Node(node) = row.destination() else {
            continue;
        };
        *rows.entry(node.id()).or_insert(0) += 1;
    }
    Some(rows)
}

fn check(report: &crate::HtmlParseReport, label: &str, tally: &mut Tally) {
    let storage = report.history().storage();
    let arena = &storage.view;
    tally.parses += 1;
    let mut fail = |message: String| tally.failures.push(format!("{label}: {message}"));
    let found = records(report);
    let causes = creation_rows(report);
    if causes.is_some() {
        tally.detailed += 1;
    }
    let (mut templates, mut shadows) = (0, 0);
    for position in 0..arena.len() {
        tally.nodes += 1;
        let record = arena.node_at(position).unwrap();
        let id = record.as_node().unwrap().id();
        // A template is an element whose template contents the parse owns.
        if let Some(contents) = record.template_contents() {
            templates += 1;
            let contents = arena
                .node_at(contents.position())
                .unwrap()
                .as_node()
                .unwrap()
                .id();
            match found.templates.get(&id).map(Vec::as_slice) {
                Some([answer]) => {
                    if answer.contents != contents {
                        fail(format!("template {position} names other contents"));
                    }
                    tally.template_markers += usize::from(answer.markers);
                }
                other => fail(format!(
                    "template {position} has {} records",
                    other.map_or(0, <[_]>::len)
                )),
            }
        } else if found.templates.contains_key(&id) {
            fail(format!(
                "node {position} has a template record and no contents"
            ));
        }
        // A shadow root's contents are a fragment that carries the properties of the root, whether
        // its host is a node of the tree or the element a contextual parse captured.
        let shadow_root = matches!(record.kind(), ConstructionNodeKind::Fragment)
            && record.shadow_properties().is_some();
        if shadow_root {
            shadows += 1;
            match found.shadows.get(&id).map(Vec::as_slice) {
                Some([(_, declarative, available)]) => {
                    // The parser attaches declarative roots, and only roots it attached are
                    // available to internals; a clone of a root is declarative and is not.
                    if !declarative {
                        fail(format!("shadow root {position} is not declarative"));
                    }
                    tally.shadow_clones += usize::from(!available);
                }
                other => fail(format!(
                    "shadow root {position} has {} records",
                    other.map_or(0, <[_]>::len)
                )),
            }
        } else if found.shadows.contains_key(&id) {
            fail(format!(
                "node {position} has a shadow record and is no shadow root"
            ));
        }
        // Every node has the cause of its creation: the row that names it as created. A parse
        // that replaces a node's cause, as a fragment's bootstrap does, adds another.
        if let Some(causes) = &causes {
            match causes.get(&id) {
                Some(rows) if *rows >= 1 => tally.creation_rows += 1,
                _ => fail(format!("node {position} has no creation cause")),
            }
        }
    }
    tally.templates += templates;
    tally.shadows += shadows;
    let answered: usize = found.templates.values().map(Vec::len).sum();
    if answered != templates {
        fail(format!(
            "{answered} template records for {templates} templates"
        ));
    }
    let answered: usize = found.shadows.values().map(Vec::len).sum();
    if answered != shadows {
        fail(format!(
            "{answered} shadow records for {shadows} shadow roots"
        ));
    }
}

/// Sources with the nodes the corpus holds few of: templates inside templates, a template cloned
/// by a `selectedcontent` element, declarative shadow roots, a clonable one among them, cloned
/// with its host, and the templates of a table and of a head.
const SOURCES: [&str; 10] = [
    "<!doctype html><template><div><template><i></i></div></template></template>",
    "<!doctype html><template><template><template></template></template></template><p>a",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
     <option selected><template><b>t</b></template></option></select>",
    "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
     <option selected><template><template><i></i></template></template><u></u></option></select>",
    "<div><template shadowrootmode=open><slot></slot></template></div>",
    "<div><template shadowrootmode=closed shadowrootclonable><p>a</p><div>\
     <template shadowrootmode=open><b></b></template></div></template></div>",
    "<select><button><selectedcontent></selectedcontent></button><option selected>\
     <div><template shadowrootmode=open shadowrootclonable><slot></slot></template></div>\
     </option></select>",
    "<!doctype html><head><template><meta charset=x></template></head><table><template>\
     <tr><td>a</template></table>",
    "<!doctype html><svg><template><desc></desc></template></svg><math><template></template></math>",
    "<template shadowrootmode=open><template shadowrootmode=open><x-a is=b></x-a></template></template>",
];

/// The capture of an element named `context` that has no shadow root, in an ordinary Document, so
/// that a template at the top of a fragment may attach one to the context element.
fn shadow_context(context: &str) -> crate::HtmlFragmentContext {
    use crate::*;
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };
    let (namespace, local) = match context.split_once(' ') {
        Some(("svg", local)) => (Namespace::svg(), local),
        _ => (Namespace::html(), context),
    };
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
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
    let element = builder
        .add_element(
            &document,
            QualifiedName::new(namespace, None, local),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    builder.establish_no_shadow_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap()
}

fn fragment_report(
    source: &str,
    context: &str,
    shadow: bool,
    detail: crate::DetailSet,
) -> crate::HtmlParseReport {
    crate::HtmlFragmentParseRequest::with_options(
        crate::HtmlSource::utf8(source),
        shadow_context(context),
        g2_d2::corpus_limits(),
        surgeist_html::parsing::HtmlParserScriptingMode::Inert,
        shadow,
    )
    .unwrap()
    .with_detail(detail)
    .unwrap()
    .parse()
    .unwrap()
}

#[test]
fn the_template_shadow_and_cause_a_report_answers_for_a_node_are_those_of_its_tree() {
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
    let corpus = (tally.parses, tally.nodes, tally.templates);
    for source in SOURCES {
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
            let fragment = source.strip_prefix("<!doctype html>").unwrap_or(source);
            for (context, shadow) in [("div", false), ("div", true), ("svg g", true)] {
                let report = fragment_report(fragment, context, shadow, detail);
                check(
                    &report,
                    &format!("{context} fragment shadow={shadow} {label}"),
                    &mut tally,
                );
            }
        }
    }
    eprintln!(
        "g2e parses {} (corpus {}) nodes {} (corpus {}) templates {} (corpus {}) markers {} \
         shadows {} shadow-clones {} detailed {} creation-rows {} failures {}",
        tally.parses,
        corpus.0,
        tally.nodes,
        corpus.1,
        tally.templates,
        corpus.2,
        tally.template_markers,
        tally.shadows,
        tally.shadow_clones,
        tally.detailed,
        tally.creation_rows,
        tally.failures.len()
    );
    for line in tally.failures.iter().take(30) {
        eprintln!("{line}");
    }
    assert!(tally.failures.is_empty());
    assert_eq!(corpus.0, 1_767 * g2_d2::details().len());
    assert_eq!(
        tally.parses,
        corpus.0 + 4 * g2_d2::details().len() * SOURCES.len()
    );
    // No source of this test writes a `template for` marker, so no template has a target.
    assert_eq!(tally.template_markers, 0);
    assert!(corpus.2 >= 100, "the corpus has templates");
    assert!(tally.templates > corpus.2);
    assert!(tally.shadows >= 16);
    assert!(tally.shadow_clones >= 1);
    if cfg!(feature = "detail") {
        assert_eq!(tally.detailed, tally.parses / 2);
        assert!(tally.creation_rows >= 13_000);
    }
}
