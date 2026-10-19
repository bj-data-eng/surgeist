//! Actual parser-report/DOM lifetime characterization, without physical-memory
//! or sharing-policy claims. No report graph is reconstructed or cloned.

use std::{
    mem::size_of,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use crate::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomElement, DomLimits, DomNameLimits,
    DomNativeInputPlatform, DomNodeKind, DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits,
    DomSnapshot, Namespace, NodeId,
};
use surgeist_html::{HtmlDocument, HtmlDocumentMode, HtmlNodeId, HtmlNodeKind};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

const ROWS: usize = 4;
const COLUMNS: usize = 8;
const CELLS: usize = ROWS * COLUMNS;
const SELECTED: usize = COLUMNS + 3;
// Document + doctype + html/head/body/table/tbody + rows + td/Text pairs.
const PARSED_NODES: usize = 2 + 5 + ROWS + 2 * CELLS;
const MANAGED_NODES: usize = PARSED_NODES + 1; // Actual origin-source Document.

mod arrays;
mod selected;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HolderStage {
    HarnessOnly,
    ParsedReport,
    Populated,
    BeforeTextEdit,
    TextEdited,
    BeforeClassEdit,
    ClassEdited,
    BeforeStyleEdit,
    StyleEdited,
    Edited,
    OwnerDropped,
    OldOnly,
    ReportOnly,
    Released,
}

impl HolderStage {
    const ALL: [Self; 14] = [
        Self::HarnessOnly,
        Self::ParsedReport,
        Self::Populated,
        Self::BeforeTextEdit,
        Self::TextEdited,
        Self::BeforeClassEdit,
        Self::ClassEdited,
        Self::BeforeStyleEdit,
        Self::StyleEdited,
        Self::Edited,
        Self::OwnerDropped,
        Self::OldOnly,
        Self::ReportOnly,
        Self::Released,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::HarnessOnly => "harness-only",
            Self::ParsedReport => "parsed-report",
            Self::Populated => "populated",
            Self::BeforeTextEdit => "before-text-edit",
            Self::TextEdited => "text-edited",
            Self::BeforeClassEdit => "before-class-edit",
            Self::ClassEdited => "class-edited",
            Self::BeforeStyleEdit => "before-style-edit",
            Self::StyleEdited => "style-edited",
            Self::Edited => "edited",
            Self::OwnerDropped => "owner-dropped",
            Self::OldOnly => "old-only",
            Self::ReportOnly => "report-only",
            Self::Released => "released",
        }
    }
}

fn observe(observer: &mut Option<&mut dyn FnMut(HolderStage)>, at: HolderStage) {
    if let Some(observer) = observer {
        observer(at);
    }
}

/// Exact call intervals in this fixture, not an exhaustive setup or process
/// total. In particular, setup includes the real origin-Document batch and
/// environment construction; assertions between calls remain outside timers.
#[derive(Debug, Default)]
struct Timings {
    parse: Duration,
    owner_environment_setup: Duration,
    installation: Duration,
    initial_snapshot_clone: Duration,
    final_snapshot: Duration,
    text_edit: Duration,
    class_edit: Duration,
    style_edit: Duration,
    installation_result_drop: Duration,
    owner_drop: Duration,
    current_snapshot_drop: Duration,
    old_snapshot_drop: Duration,
    report_drop: Duration,
    report_weak_drop: Duration,
}

fn elapsed(start: Option<Instant>) -> Duration {
    // A disabled clock performs neither Instant::now nor Instant::elapsed.
    start.map_or(Duration::ZERO, |start| start.elapsed())
}

fn markup() -> String {
    let mut source = String::from("<!doctype html><html><head></head><body><table><tbody>");
    for row in 0..ROWS {
        source.push_str(&format!("<tr id=\"r{row}\">"));
        for _ in 0..COLUMNS {
            // Separate source occurrences, not caller-shared attribute banks.
            source.push_str("<td class=\"cell\" style=\"color: red\">cell</td>");
        }
        source.push_str("</tr>");
    }
    source.push_str("</tbody></table></body></html>");
    source
}

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(1024)
        .managed_edges(1024)
        .managed_attributes(1024)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

fn setup() -> (Dom, DomParsedDocumentEnvironment) {
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            10_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let origin_document = batch.create_document();
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&origin_document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    drop(commit);
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let url = HtmlBaseUrlRecord::observed(
        revision,
        "https://example.test/memory-table/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    (dom, environment)
}

fn report_children(document: &HtmlDocument, id: &HtmlNodeId, local: &str) -> Vec<HtmlNodeId> {
    let node = document.node(id).unwrap();
    let HtmlNodeKind::Element(element) = node.kind() else {
        panic!("expected report Element {local}");
    };
    assert_eq!(element.name().namespace().uri(), Namespace::html().uri());
    assert_eq!(element.name().local_name(), &DocumentString::from(local));
    assert!(element.name().prefix().is_none());
    node.children().collect()
}

/// Re-read the actual immutable report at every lifetime boundary. The source
/// and grammar fix the expected values/topology; DOM output is not its oracle.
fn check_report(report: &HtmlParseReport, source: &str) -> Vec<HtmlNodeId> {
    assert_eq!(report.source().as_utf8(), Some(source));
    let document = report.document().unwrap();
    assert_eq!(document.mode(), HtmlDocumentMode::NoQuirks);
    let root = document.root();
    assert!(root.parent().is_none());
    let children: Vec<_> = root.children().collect();
    assert_eq!(children.len(), 2);
    let HtmlNodeKind::Doctype(doctype) = document.node(&children[0]).unwrap().kind() else {
        panic!("literal doctype");
    };
    assert_eq!(doctype.name(), &DocumentString::from("html"));
    let html = report_children(document, &children[1], "html");
    assert_eq!(html.len(), 2);
    assert!(report_children(document, &html[0], "head").is_empty());
    let body = report_children(document, &html[1], "body");
    assert_eq!(body.len(), 1);
    let table = report_children(document, &body[0], "table");
    assert_eq!(table.len(), 1);
    let rows = report_children(document, &table[0], "tbody");
    assert_eq!(rows.len(), ROWS);
    for (ordinal, row) in rows.iter().enumerate() {
        let cells = report_children(document, row, "tr");
        assert_eq!(cells.len(), COLUMNS);
        let HtmlNodeKind::Element(element) = document.node(row).unwrap().kind() else {
            unreachable!();
        };
        let attributes = element.attributes();
        assert_eq!(attributes.len(), 1);
        assert_eq!(
            attributes.iter().next().unwrap().name().local_name(),
            &DocumentString::from("id")
        );
        assert_eq!(
            attributes.iter().next().unwrap().value(),
            &DocumentString::from(format!("r{ordinal}"))
        );
        for cell in cells {
            let text = report_children(document, &cell, "td");
            assert_eq!(text.len(), 1);
            let HtmlNodeKind::Element(element) = document.node(&cell).unwrap().kind() else {
                unreachable!();
            };
            assert_eq!(element.attributes().len(), 2);
            for (actual, (name, value)) in element
                .attributes()
                .iter()
                .zip([("class", "cell"), ("style", "color: red")])
            {
                assert_eq!(actual.name().local_name(), &DocumentString::from(name));
                assert!(actual.name().namespace().uri().is_none());
                assert!(actual.name().prefix().is_none());
                assert_eq!(actual.value(), &DocumentString::from(value));
            }
            assert_eq!(
                document.node(&text[0]).unwrap().kind(),
                &HtmlNodeKind::Text("cell".into())
            );
        }
    }
    let mut order = Vec::new();
    let mut pending = vec![root.id()];
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        for child in node.children() {
            assert_eq!(document.node(&child).unwrap().parent(), Some(id.clone()));
        }
        pending.extend(node.children().rev());
        order.push(id);
    }
    assert_eq!(order.len(), PARSED_NODES);
    order
}

#[derive(Clone, Copy)]
enum Stage {
    Original,
    Text,
    Class,
    Style,
}

impl Stage {
    fn before(self) -> HolderStage {
        match self {
            Self::Text => HolderStage::BeforeTextEdit,
            Self::Class => HolderStage::BeforeClassEdit,
            Self::Style => HolderStage::BeforeStyleEdit,
            Self::Original => unreachable!("the original report is not an edit"),
        }
    }

    fn after(self) -> HolderStage {
        match self {
            Self::Text => HolderStage::TextEdited,
            Self::Class => HolderStage::ClassEdited,
            Self::Style => HolderStage::StyleEdited,
            Self::Original => unreachable!("the original report is not an edit"),
        }
    }
}

fn element<'a>(snapshot: &'a DomSnapshot, id: NodeId, local: &str) -> &'a DomElement {
    let value = snapshot.node(id).unwrap().as_element().unwrap();
    assert_eq!(value.name().namespace_uri(), Namespace::html().uri());
    assert_eq!(value.name().local_name(), &DocumentString::from(local));
    assert!(value.name().prefix().is_none());
    value
}

fn check_dom(
    snapshot: &DomSnapshot,
    document: DocumentId,
    stage: Stage,
) -> (Vec<NodeId>, Vec<NodeId>) {
    assert_eq!(snapshot.nodes().count(), MANAGED_NODES);
    assert_eq!(
        snapshot.document(document).unwrap().metadata().mode(),
        HtmlDocumentMode::NoQuirks
    );
    let root = snapshot.node(document.node()).unwrap();
    assert!(root.parent().is_none());
    assert_eq!(root.children().len(), 2);
    assert!(matches!(
        snapshot.node(root.children()[0]).unwrap().kind(),
        DomNodeKind::DocumentType(_)
    ));
    let html = root.children()[1];
    element(snapshot, html, "html");
    let children = snapshot.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    element(snapshot, children[0], "head");
    assert!(snapshot.node(children[0]).unwrap().children().is_empty());
    element(snapshot, children[1], "body");
    let body = snapshot.node(children[1]).unwrap().children();
    assert_eq!(body.len(), 1);
    element(snapshot, body[0], "table");
    let table = snapshot.node(body[0]).unwrap().children();
    assert_eq!(table.len(), 1);
    element(snapshot, table[0], "tbody");
    let rows = snapshot.node(table[0]).unwrap().children();
    assert_eq!(rows.len(), ROWS);
    let mut cell_ids = Vec::new();
    let mut text_ids = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        let row_element = element(snapshot, *row, "tr");
        assert_eq!(row_element.attributes().len(), 1);
        assert_eq!(
            row_element.attribute(&"id".into()),
            Some(&DocumentString::from(format!("r{row_index}")))
        );
        let cells = snapshot.node(*row).unwrap().children();
        assert_eq!(cells.len(), COLUMNS);
        for cell in cells {
            let selected = cell_ids.len() == SELECTED;
            let class = if selected && matches!(stage, Stage::Class | Stage::Style) {
                "chosen"
            } else {
                "cell"
            };
            let style = if selected && matches!(stage, Stage::Style) {
                "color: blue"
            } else {
                "color: red"
            };
            let text = if selected && !matches!(stage, Stage::Original) {
                "edited"
            } else {
                "cell"
            };
            let value = element(snapshot, *cell, "td");
            assert_eq!(value.attributes().len(), 2);
            for (actual, (name, expected)) in value
                .attributes()
                .iter()
                .zip([("class", class), ("style", style)])
            {
                assert_eq!(actual.name().local_name(), &DocumentString::from(name));
                assert!(actual.name().namespace_uri().is_none());
                assert!(actual.name().prefix().is_none());
                assert_eq!(actual.value(), &DocumentString::from(expected));
            }
            let children = snapshot.node(*cell).unwrap().children();
            assert_eq!(children.len(), 1);
            assert!(matches!(
                snapshot.node(children[0]).unwrap().kind(),
                DomNodeKind::Text(_)
            ));
            assert_eq!(
                snapshot.node(children[0]).unwrap().data(),
                Some(&DocumentString::from(text))
            );
            assert_eq!(snapshot.node(children[0]).unwrap().parent(), Some(*cell));
            assert_eq!(snapshot.node(*cell).unwrap().parent(), Some(*row));
            assert_eq!(
                snapshot.node(*cell).unwrap().owner_document(),
                Some(document)
            );
            cell_ids.push(*cell);
            text_ids.push(children[0]);
        }
    }
    (cell_ids, text_ids)
}

/// One actual parse/install/edit/release path. Observers receive only the stage;
/// all oracle/array temporaries have gone before a holder-stage observation.
/// Before/after-edit observations bracket the real batch construction/apply;
/// they do not by themselves qualify a process high-water mark as an edit peak.
fn run_sample(
    inventories: bool,
    mut observer: Option<&mut dyn FnMut(HolderStage)>,
    timing: bool,
) -> Option<Timings> {
    assert!(
        !timing || (!inventories && observer.is_none()),
        "operation timings require an ungated fixture without inventories"
    );
    let clock = timing && !super::scale_metrics::ENABLED;
    let mut times = clock.then(Timings::default);
    observe(&mut observer, HolderStage::HarnessOnly);
    let source = markup();
    let request =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(source.clone()), parse_limits());
    let started = clock.then(Instant::now);
    let parsed = request.parse();
    let parse_time = elapsed(started);
    let report = Arc::new(parsed.unwrap());
    if let Some(times) = &mut times {
        times.parse = parse_time;
    }
    let report_ids = check_report(&report, &source);
    let report_liveness = Arc::downgrade(&report);
    observe(&mut observer, HolderStage::ParsedReport);
    let started = clock.then(Instant::now);
    let (mut dom, environment) = setup();
    let setup_time = elapsed(started);
    if let Some(times) = &mut times {
        times.owner_environment_setup = setup_time;
    }
    assert_eq!(dom.usage().managed_nodes(), 1);
    let before = dom.snapshot().stamp();
    let measurement = if super::scale_metrics::ENABLED {
        Some(super::scale_metrics::Measurement::begin().unwrap())
    } else {
        None
    };
    let incoming_report = report.clone();
    let install_limits = DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000);
    let cancelled = AtomicBool::new(false);
    let started = clock.then(Instant::now);
    let installation = dom.install_parsed_html_document(
        before,
        incoming_report,
        environment,
        install_limits,
        &cancelled,
    );
    let install_time = elapsed(started);
    let installed = installation.unwrap();
    if let Some(times) = &mut times {
        times.installation = install_time;
    }
    if let Some(measurement) = measurement {
        let counters = measurement
            .finish(super::scale_metrics::Hooks::default())
            .unwrap();
        assert_eq!(
            counters.parsed_attribute_rows_materialized,
            ROWS + 2 * CELLS
        );
        assert_eq!(
            counters.parsed_attribute_row_materialized_bytes,
            (ROWS + 2 * CELLS) * size_of::<crate::DomAttribute>()
        );
        eprintln!(
            "memory_parsed_conversion completed_attribute_rows={} row_metadata_bytes={} UTF16_buffer_copy_claim=false",
            counters.parsed_attribute_rows_materialized,
            counters.parsed_attribute_row_materialized_bytes
        );
    }
    assert!(std::ptr::eq(installed.report(), report.as_ref()));
    assert_eq!(installed.bindings().len(), PARSED_NODES);
    let document = installed.document();
    let started = clock.then(Instant::now);
    let old = installed.snapshot().clone();
    let snapshot_time = elapsed(started);
    if let Some(times) = &mut times {
        times.initial_snapshot_clone = snapshot_time;
    }
    assert_eq!(old.revision().get(), before.revision().get() + 1);
    assert_eq!(installed.node_for(&report_ids[0]), Some(document.node()));
    for source_id in &report_ids {
        let mapped = installed.node_for(source_id).unwrap();
        let original = report.document().unwrap().node(source_id).unwrap();
        assert_eq!(
            old.node(mapped).unwrap().children(),
            original
                .children()
                .map(|id| installed.node_for(&id).unwrap())
                .collect::<Vec<_>>()
        );
    }
    let (cells, text) = check_dom(&old, document, Stage::Original);
    // The installation owns both a snapshot and the same report Arc. Dispose
    // it now so the explicit holders below determine the later release order.
    let started = clock.then(Instant::now);
    drop(installed);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.installation_result_drop = drop_time;
    }
    observe(&mut observer, HolderStage::Populated);
    for stage in [Stage::Text, Stage::Class, Stage::Style] {
        observe(&mut observer, stage.before());
        let revision = dom.revision();
        let mut batch = DomBatch::new();
        let expected = match stage {
            Stage::Text => {
                batch.replace_data(text[SELECTED], 0, usize::MAX, "edited");
                DomChange::Data(text[SELECTED])
            }
            Stage::Class => {
                batch.set_attribute(cells[SELECTED], "class", "chosen");
                DomChange::Attributes(cells[SELECTED])
            }
            Stage::Style => {
                batch.set_attribute(cells[SELECTED], "style", "color: blue");
                DomChange::Attributes(cells[SELECTED])
            }
            Stage::Original => unreachable!(),
        };
        let cancelled = AtomicBool::new(false);
        let started = clock.then(Instant::now);
        let applied = dom.apply(batch, &cancelled);
        let edit_time = elapsed(started);
        let commit = applied.unwrap();
        if let Some(times) = &mut times {
            match stage {
                Stage::Text => times.text_edit = edit_time,
                Stage::Class => times.class_edit = edit_time,
                Stage::Style => times.style_edit = edit_time,
                Stage::Original => unreachable!(),
            }
        }
        assert_eq!(commit.changes().changes(), [expected]);
        drop(commit);
        assert_eq!(dom.revision().get(), revision.get() + 1);
        // No current-snapshot clone or full oracle runs inside this bracket.
        observe(&mut observer, stage.after());
        let current = dom.snapshot();
        assert_eq!(
            check_dom(&current, document, stage),
            (cells.clone(), text.clone())
        );
        assert!(current.nodes().eq(old.nodes()));
        for id in old.nodes() {
            assert_eq!(
                current.node(id).unwrap().children(),
                old.node(id).unwrap().children()
            );
            assert_eq!(
                current.node(id).unwrap().parent(),
                old.node(id).unwrap().parent()
            );
        }
        assert_eq!(
            check_dom(&old, document, Stage::Original),
            (cells.clone(), text.clone())
        );
        assert_eq!(check_report(&report, &source), report_ids);
    }
    let started = clock.then(Instant::now);
    let current = dom.snapshot();
    let snapshot_time = elapsed(started);
    if let Some(times) = &mut times {
        times.final_snapshot = snapshot_time;
    }
    let managed_nodes = dom.usage().managed_nodes();
    assert_eq!(managed_nodes, MANAGED_NODES);
    if !timing {
        eprintln!(
            "memory_parsed rows={ROWS} columns={COLUMNS} cells={} parsed_nodes={} managed_nodes={managed_nodes} origin_documents=1 phase=owner_report_old_current_held physical_measurement=false",
            cells.len(),
            report_ids.len()
        );
    }
    if inventories {
        arrays::report(
            "owner_report_old_current",
            Some(&report),
            &report_ids,
            &[&old, &current],
        );
    }
    observe(&mut observer, HolderStage::Edited);
    let started = clock.then(Instant::now);
    drop(dom);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.owner_drop = drop_time;
    }
    assert_eq!(
        check_dom(&current, document, Stage::Style),
        (cells.clone(), text.clone())
    );
    assert_eq!(
        check_dom(&old, document, Stage::Original),
        (cells.clone(), text.clone())
    );
    assert_eq!(check_report(&report, &source), report_ids);
    if inventories {
        arrays::report(
            "report_old_current",
            Some(&report),
            &report_ids,
            &[&old, &current],
        );
    }
    observe(&mut observer, HolderStage::OwnerDropped);
    let started = clock.then(Instant::now);
    drop(current);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.current_snapshot_drop = drop_time;
    }
    if inventories {
        arrays::report("report_old", Some(&report), &report_ids, &[&old]);
    }
    assert_eq!(check_dom(&old, document, Stage::Original), (cells, text));
    assert_eq!(check_report(&report, &source), report_ids);
    observe(&mut observer, HolderStage::OldOnly);
    let started = clock.then(Instant::now);
    drop(old);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.old_snapshot_drop = drop_time;
    }
    if inventories {
        arrays::report("report_only", Some(&report), &report_ids, &[]);
    }
    assert_eq!(check_report(&report, &source), report_ids);
    // HtmlNodeId holds the construction identity, and source is a separate
    // authored String. Neither is needed after the final independent oracle.
    drop(report_ids);
    drop(source);
    observe(&mut observer, HolderStage::ReportOnly);
    let started = clock.then(Instant::now);
    drop(report);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.report_drop = drop_time;
    }
    assert!(report_liveness.upgrade().is_none());
    // This proves wrapper liveness, not reclamation of every nested shared
    // allocation or allocator/Weak control block. Drop the observer last.
    let started = clock.then(Instant::now);
    drop(report_liveness);
    let drop_time = elapsed(started);
    if let Some(times) = &mut times {
        times.report_weak_drop = drop_time;
    }
    if inventories {
        arrays::report("no_strong_fixture_arrays", None, &[], &[]);
    }
    observe(&mut observer, HolderStage::Released);
    times
}

#[test]
fn parsed_table_report_and_snapshots_survive_isolated_edits_and_ordered_release() {
    let mut stages = Vec::new();
    assert!(run_sample(true, Some(&mut |stage| stages.push(stage)), false).is_none());
    assert_eq!(
        stages,
        [
            HolderStage::HarnessOnly,
            HolderStage::ParsedReport,
            HolderStage::Populated,
            HolderStage::BeforeTextEdit,
            HolderStage::TextEdited,
            HolderStage::BeforeClassEdit,
            HolderStage::ClassEdited,
            HolderStage::BeforeStyleEdit,
            HolderStage::StyleEdited,
            HolderStage::Edited,
            HolderStage::OwnerDropped,
            HolderStage::OldOnly,
            HolderStage::ReportOnly,
            HolderStage::Released,
        ]
    );
}

#[test]
fn parsed_operation_clock_preserves_the_real_lifecycle_and_respects_counter_mode() {
    let times = run_sample(false, None, true);
    assert_eq!(times.is_some(), !super::scale_metrics::ENABLED);
    // Clock resolution may legitimately produce zero; timings have no numeric
    // pass threshold and never claim allocator reclamation or an edit peak.
}
