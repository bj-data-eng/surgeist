//! Release-mode benchmark of semantics capture and semantic facts over the five
//! documents of the `parse` benchmark.
//!
//! Each document is parsed once, outside every timer, with generous limits. Per
//! round the benchmark then times four phases over that parse:
//!
//! - **capture:** import the construction result into a capture through the
//!   construction adapter and finish it;
//! - **facts:** build the global-inheritance facts (one spellcheck default per
//!   element) and the language-direction facts of the capture;
//! - **queries:** run batches of semantic queries over every element, and list
//!   and table queries over the elements they apply to, in one session;
//! - **storage:** measure the capture's retained storage.
//!
//! The report lists each document's size, the median, minimum and maximum time
//! of each phase over the rounds, and the accounted totals each phase reaches:
//! the capture and import usage, the session usage and the retained storage.
//! The totals depend only on the document and the limits, so they identify the
//! accounted work a timing change is measured against.
//!
//! Run with `cargo bench --offline --locked -j 1 -p surgeist-html-parser --bench semantics`.

use std::{fmt::Write as _, hint::black_box, time::Instant};

#[path = "support/documents.rs"]
mod documents;

use surgeist_html::{
    bounded::construction::HtmlConstructionResult,
    construction::{HtmlDocument, HtmlNodeId, HtmlNodeKind},
    values::HtmlValueInterpreter,
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedElement, HtmlCapturedIsValue, HtmlCapturedNodeKind,
    HtmlConstructionCaptureAdapter, HtmlConstructionCaptureDocument,
    HtmlConstructionCaptureDocumentFacts, HtmlConstructionCaptureElementFacts,
    HtmlConstructionCaptureFacts, HtmlConstructionCaptureLimits, HtmlConstructionCaptureNodeFacts,
    HtmlConstructionCaptureUsage, HtmlConstructionCapturedResult, HtmlDocumentLanguageFallback,
    HtmlGlobalInheritanceFacts, HtmlGlobalInheritanceLimits, HtmlGlobalInheritanceRevision,
    HtmlHostFactsRevision, HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits,
    HtmlPragmaLanguage, HtmlProtocolLanguage, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticLimits, HtmlSemanticQuery,
    HtmlSemanticRequest, HtmlSemanticSession, HtmlSemanticUsage, HtmlSpellcheckDefaultBehavior,
    HtmlSpellcheckDefaultFact,
};

/// Measured rounds per phase and document, after one unmeasured round.
const ROUNDS: usize = 5;
const HUGE: usize = 1 << 40;
/// Every this-many-th element is asked the queries whose work grows with depth.
const ANCESTOR_STRIDE: usize = 32;

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(1 << 30)
        .buffered_utf16_units(1 << 30)
        .token_attributes(1 << 30)
        .auxiliary_entries(1 << 30)
        .active_formatting_entries(1 << 30)
        .managed_nodes(1 << 30)
        .managed_edges(1 << 30)
        .managed_attributes(1 << 30)
        .managed_utf16_units(1 << 30)
        .source_map_entries(1 << 30)
        .source_map_payload_units(1 << 30)
        .diagnostics(1 << 30)
        .diagnostic_payload_units(1 << 30)
        .context_effects(1 << 30)
        .open_elements(1 << 30)
        .tree_depth(1 << 30)
        .managed_capacity_bytes(1 << 40)
        .peak_transient_bytes(1 << 40)
        .atomic_layout_bytes(1 << 40)
        .work_credits(u64::MAX)
        .finish()
        .expect("benchmark limits are representable")
}

fn parse(text: &str) -> HtmlDocument {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), parse_limits())
        .parse()
        .unwrap_or_else(|failure| panic!("benchmark document failed to parse: {failure:?}"))
        .document()
        .expect("the benchmark document parses to a document")
        .clone()
}

/// The nodes of a document in the order the benchmark visits them.
fn nodes(document: &HtmlDocument) -> Vec<(HtmlNodeId, bool, bool)> {
    let source = HtmlConstructionResult::Document(document.clone());
    let mut pending = vec![document.root().id()];
    let mut nodes = Vec::new();
    while let Some(id) = pending.pop() {
        let node = source.node(&id).expect("a reachable node is readable");
        pending.extend(node.children());
        nodes.push((
            id,
            matches!(node.kind(), HtmlNodeKind::Document(_)),
            matches!(node.kind(), HtmlNodeKind::Element(_)),
        ));
    }
    nodes
}

fn facts(
    nodes: &[(HtmlNodeId, bool, bool)],
    revision: &HtmlHostFactsRevision,
) -> HtmlConstructionCaptureFacts {
    let rows = nodes
        .iter()
        .map(|(id, is_document, is_element)| {
            HtmlConstructionCaptureNodeFacts::new(
                *id,
                HtmlConstructionCaptureDocument::ResultDocument,
                is_document.then(|| {
                    HtmlConstructionCaptureDocumentFacts::new(
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Null,
                    )
                }),
                is_element.then(|| {
                    HtmlConstructionCaptureElementFacts::new(
                        HtmlCapturedIsValue::Absent,
                        HtmlCaptureRegistryInput::Null,
                    )
                }),
                None,
            )
        })
        .collect();
    HtmlConstructionCaptureFacts::new(revision.clone(), rows)
}

fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(HUGE, HUGE, HUGE, HUGE, HUGE, 0, 0, HUGE)
}

fn import_limits() -> HtmlConstructionCaptureLimits {
    HtmlConstructionCaptureLimits::new(HUGE, HUGE, HUGE)
}

/// One capture import, finished.
fn capture(
    document: &HtmlDocument,
    nodes: &[(HtmlNodeId, bool, bool)],
    revision: &HtmlHostFactsRevision,
) -> (HtmlConstructionCapturedResult, HtmlConstructionCaptureUsage) {
    let builder = HtmlCaptureBuilder::new(capture_limits(), revision.clone());
    let import = HtmlConstructionCaptureAdapter::document(
        builder,
        document.clone(),
        facts(nodes, revision),
        import_limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap_or_else(|failure| panic!("benchmark capture import failed: {:?}", failure.reason()));
    let usage = import.usage();
    let result = import.finish().unwrap_or_else(|failure| {
        panic!("benchmark capture finish failed: {:?}", failure.reason())
    });
    (result, usage)
}

fn element_name(capture: &HtmlCapture, subject: &HtmlCaptureSubjectId) -> Option<String> {
    let node = capture.node(subject).ok()?;
    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
        return None;
    };
    Some(local_name(element))
}

fn local_name(element: &HtmlCapturedElement) -> String {
    String::from_utf16_lossy(element.name().local_name().code_units())
}

/// The semantic requests the benchmark batches, grouped by query.
fn requests(
    result: &HtmlConstructionCapturedResult,
    nodes: &[(HtmlNodeId, bool, bool)],
) -> Vec<Vec<HtmlSemanticRequest>> {
    // Language and directionality walk every ancestor, so their work grows with
    // depth times elements. They are requested for every `ANCESTOR_STRIDE`-th
    // element to keep the deeply nested document's round short.
    let general = [
        (HtmlSemanticQuery::Hidden, 1),
        (HtmlSemanticQuery::Disabledness, 1),
        (HtmlSemanticQuery::TabIndex, 1),
        (HtmlSemanticQuery::Language, ANCESTOR_STRIDE),
        (HtmlSemanticQuery::Directionality, ANCESTOR_STRIDE),
    ];
    let mut batches: Vec<Vec<HtmlSemanticRequest>> = general.iter().map(|_| Vec::new()).collect();
    let mut specific: Vec<(&str, HtmlSemanticQuery, Vec<HtmlSemanticRequest>)> = vec![
        ("li", HtmlSemanticQuery::ListOrdinal, Vec::new()),
        ("table", HtmlSemanticQuery::TableModel, Vec::new()),
        ("table", HtmlSemanticQuery::TableRows, Vec::new()),
        ("tr", HtmlSemanticQuery::TableRowCells, Vec::new()),
        ("tr", HtmlSemanticQuery::TableRowIndex, Vec::new()),
        ("td", HtmlSemanticQuery::TableCellIndex, Vec::new()),
        ("th", HtmlSemanticQuery::TableCellIndex, Vec::new()),
    ];
    for (position, (id, _, is_element)) in nodes.iter().enumerate() {
        if !is_element {
            continue;
        }
        let subject = result
            .mapping()
            .subject(id, result.capture())
            .expect("an imported node has a subject");
        for (batch, (query, stride)) in batches.iter_mut().zip(general) {
            if position % stride == 0 {
                batch.push(HtmlSemanticRequest::new(subject.clone(), query));
            }
        }
        let name = element_name(result.capture(), &subject).unwrap_or_default();
        for (local, query, batch) in &mut specific {
            if name == *local {
                batch.push(HtmlSemanticRequest::new(subject.clone(), *query));
            }
        }
    }
    batches.extend(specific.into_iter().map(|(_, _, batch)| batch));
    batches.retain(|batch| !batch.is_empty());
    batches
}

fn session_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(HUGE, HUGE, HUGE, 1 << 20, HUGE, HUGE)
}

/// One session over every batch; returns its usage.
fn query(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    batches: &[Vec<HtmlSemanticRequest>],
) -> HtmlSemanticUsage {
    let mut session = HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(1 << 20),
        session_limits(),
        HtmlSemanticCancellation::new(),
    )
    .expect("the context belongs to the capture");
    for batch in batches {
        let report = session
            .query(batch)
            .unwrap_or_else(|failure| panic!("benchmark query failed: {:?}", failure.kind()));
        black_box(report);
    }
    session.usage()
}

/// The facts the benchmark builds over a capture.
fn build_facts(
    result: &HtmlConstructionCapturedResult,
    nodes: &[(HtmlNodeId, bool, bool)],
    revision: &HtmlHostFactsRevision,
) -> (HtmlGlobalInheritanceFacts, HtmlLanguageDirectionFacts) {
    let mut defaults = Vec::new();
    let mut documents = Vec::new();
    for (id, is_document, is_element) in nodes {
        let subject = result
            .mapping()
            .subject(id, result.capture())
            .expect("an imported node has a subject");
        if *is_element {
            defaults.push(HtmlSpellcheckDefaultFact::new(
                subject,
                Some(HtmlSpellcheckDefaultBehavior::TrueByDefault),
            ));
        } else if *is_document {
            documents.push(HtmlDocumentLanguageFallback::new(
                subject,
                HtmlPragmaLanguage::Absent,
                HtmlProtocolLanguage::Absent,
            ));
        }
    }
    let inheritance = HtmlGlobalInheritanceFacts::new(
        result.capture(),
        revision.clone(),
        HtmlGlobalInheritanceRevision::new(
            surgeist_html_semantics::HtmlGlobalInheritanceProducerId::fresh(),
            0,
            0,
        ),
        Vec::new(),
        defaults,
        HtmlGlobalInheritanceLimits::new(HUGE, HUGE),
    )
    .unwrap_or_else(|error| panic!("benchmark inheritance facts failed: {error:?}"));
    let language = HtmlLanguageDirectionFacts::new(
        result.capture(),
        revision.clone(),
        documents,
        Vec::new(),
        Vec::new(),
        HtmlLanguageDirectionLimits::new(HUGE, HUGE, HUGE, HUGE),
    )
    .unwrap_or_else(|error| panic!("benchmark language facts failed: {error:?}"));
    (inheritance, language)
}

fn context(
    result: &HtmlConstructionCapturedResult,
    revision: &HtmlHostFactsRevision,
) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        result.capture(),
        revision.clone(),
        HtmlSemanticContextInput::new(Vec::new()),
        0,
    )
    .unwrap_or_else(|error| panic!("benchmark context failed: {error:?}"))
}

fn timed<T>(work: impl FnOnce() -> T) -> (f64, T) {
    let start = Instant::now();
    let value = work();
    (start.elapsed().as_secs_f64() * 1_000.0, value)
}

struct Phase {
    name: &'static str,
    rounds: Vec<f64>,
}

impl Phase {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            rounds: Vec::new(),
        }
    }
    fn line(&mut self, document: &str) -> String {
        self.rounds.sort_by(f64::total_cmp);
        let last = self.rounds.len() - 1;
        format!(
            "{document:<18} {:<9} {:>10.2} {:>10.2} {:>10.2}",
            self.name,
            self.rounds[last / 2],
            self.rounds[0],
            self.rounds[last]
        )
    }
}

fn main() {
    let documents: [(&str, String); 5] = [
        ("ordinary", documents::ordinary()),
        ("deeply-nested", documents::deeply_nested()),
        ("many-attributes", documents::many_attributes()),
        ("table-heavy", documents::table_heavy()),
        ("many-diagnostics", documents::many_diagnostics()),
    ];
    println!("rounds per phase: {ROUNDS} (after one unmeasured round)");
    println!();
    println!(
        "{:<18} {:<9} {:>10} {:>10} {:>10}",
        "document", "phase", "median ms", "min ms", "max ms"
    );
    let mut details = String::new();
    for (name, text) in &documents {
        let document = parse(text);
        let nodes = nodes(&document);
        let revision = HtmlHostFactsRevision::fresh();
        let mut phases = [
            Phase::new("capture"),
            Phase::new("facts"),
            Phase::new("queries"),
            Phase::new("storage"),
        ];
        let mut record = String::new();
        for round in 0..=ROUNDS {
            let (capture_ms, (result, import)) = timed(|| capture(&document, &nodes, &revision));
            let (facts_ms, built) = timed(|| build_facts(&result, &nodes, &revision));
            black_box(built);
            let context = context(&result, &revision);
            let batches = requests(&result, &nodes);
            let (query_ms, usage) = timed(|| query(result.capture(), &context, &batches));
            let (storage_ms, retained) = timed(|| {
                result
                    .capture()
                    .retained_storage_usage_with(|| Ok::<(), ()>(()))
                    .expect("a bounded capture measures its storage")
            });
            if round == 0 {
                let capture_usage = result.capture().usage();
                record = format!(
                    "{name} ({} bytes, {} nodes, {} requests)\n  capture usage: subjects={} relationships={} attributes={} utf16_units={} depth={} registries={} definitions={} element_value_bytes={}\n  import usage: mapping_entries={} scratch_entries={} work={}\n  session usage: work={} output_records={} scratch_records={} allocated_bytes={} peak_scratch_records={} nested_output_records={}\n  retained storage: bytes={} utf16_extent={}\n",
                    text.len(),
                    nodes.len(),
                    batches.iter().map(Vec::len).sum::<usize>(),
                    capture_usage.subjects(),
                    capture_usage.relationships(),
                    capture_usage.attributes(),
                    capture_usage.utf16_units(),
                    capture_usage.depth(),
                    capture_usage.registries(),
                    capture_usage.definitions(),
                    capture_usage.element_value_bytes(),
                    import.mapping_entries(),
                    import.scratch_entries(),
                    import.work(),
                    usage.work(),
                    usage.output_records(),
                    usage.scratch_records(),
                    usage.allocated_bytes(),
                    usage.peak_scratch_records(),
                    usage.nested_output_records(),
                    retained.retained_bytes(),
                    retained.utf16_extent(),
                );
                continue;
            }
            phases[0].rounds.push(capture_ms);
            phases[1].rounds.push(facts_ms);
            phases[2].rounds.push(query_ms);
            phases[3].rounds.push(storage_ms);
        }
        for phase in &mut phases {
            println!("{}", phase.line(name));
        }
        let _ = write!(details, "{record}");
    }
    println!();
    print!("{details}");
}
