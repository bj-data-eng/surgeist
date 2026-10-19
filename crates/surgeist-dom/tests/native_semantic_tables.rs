//! Native DOM semantic-table capture and canonical subject mapping.
//! HTML877d §4.9.12 table formation supplies the literal slot oracles below.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    CreatedNode, DocumentId, Dom, DomBatch, DomCommit, DomHostDomains, DomHostError,
    DomHostObservations, DomHostPublisher, DomHtmlCapture, DomHtmlCaptureError,
    DomHtmlCaptureLimits, DomHtmlCaptureResource, DomLimits, DomSnapshot, Namespace, NodeId,
    Observed,
};
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureError, HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticDependencies,
    HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported,
    HtmlSemanticValue, HtmlTableCellKind, HtmlTableModel, HtmlTableModelError,
};

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16384,
            256,
            0,
            0,
            16,
            131072,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10_000_000, 16, 4096, 256, 10_000_000, 4096)
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(128, 2048, 256, 32768, 128, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn run(c: &DomHtmlCapture, table: NodeId) -> HtmlSemanticReport {
    c.query(
        &[HtmlSemanticRequest::new(
            c.subject(table).unwrap(),
            HtmlSemanticQuery::TableModel,
        )],
        HtmlValueInterpreter::new(8192),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}
fn model(report: &HtmlSemanticReport) -> &HtmlTableModel {
    match report.results()[0].fact() {
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected actual complete table, got {other:?}"),
    }
}
fn element(
    b: &mut DomBatch,
    d: &CreatedNode,
    parent: &CreatedNode,
    local: &str,
    attrs: &[(&str, &str)],
) -> CreatedNode {
    let node = b.create_element_ns(d, Namespace::html(), local);
    for (name, value) in attrs {
        b.set_attribute(&node, *name, *value);
    }
    b.insert_before(&node, parent, None);
    node
}
fn node(commit: &DomCommit, ticket: &CreatedNode) -> NodeId {
    commit.created(ticket).unwrap().unwrap()
}

struct Grid {
    dom: Dom,
    document: DocumentId,
    table: NodeId,
    caption: NodeId,
    group: NodeId,
    columns: [NodeId; 2],
    body: NodeId,
    rows: [NodeId; 2],
    cells: [NodeId; 4],
    nested: [NodeId; 2],
    separate: [NodeId; 2],
}
fn grid() -> Grid {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = element(&mut b, &d, &d, "div", &[]);
    let table = element(&mut b, &d, &root, "table", &[]);
    let caption = element(&mut b, &d, &table, "caption", &[]);
    let group = element(&mut b, &d, &table, "colgroup", &[("span", "99")]);
    let col0 = element(&mut b, &d, &group, "col", &[("span", "2")]);
    let col1 = element(&mut b, &d, &group, "col", &[]);
    let comment = b.create_comment(&d, "not a table row");
    b.insert_before(&comment, &table, None);
    let body = element(&mut b, &d, &table, "tbody", &[]);
    let r0 = element(&mut b, &d, &body, "tr", &[]);
    let a = element(&mut b, &d, &r0, "td", &[("rowspan", "2"), ("headers", "")]);
    let h = element(&mut b, &d, &r0, "th", &[("id", "heading"), ("headers", "")]);
    let ht = b.create_text(&d, "heading");
    b.insert_before(&ht, &h, None);
    let text = b.create_text(&d, "intervening row text");
    b.insert_before(&text, &r0, None);
    element(&mut b, &d, &r0, "span", &[]);
    let other = element(&mut b, &d, &r0, "td", &[("headers", "")]);
    let nested = element(&mut b, &d, &other, "table", &[]);
    let nested_row = element(&mut b, &d, &nested, "tr", &[]);
    let nested_cell = element(&mut b, &d, &nested_row, "td", &[("headers", "")]);
    let r1 = element(&mut b, &d, &body, "tr", &[]);
    let wide = element(
        &mut b,
        &d,
        &r1,
        "td",
        &[("colspan", "2"), ("headers", "heading")],
    );
    let separate = element(&mut b, &d, &root, "table", &[]);
    let separate_row = element(&mut b, &d, &separate, "tr", &[]);
    let separate_cell = element(&mut b, &d, &separate_row, "td", &[("headers", "")]);
    let c = apply(&mut dom, b);
    Grid {
        document: c.document(&d).unwrap(),
        table: node(&c, &table),
        caption: node(&c, &caption),
        group: node(&c, &group),
        columns: [node(&c, &col0), node(&c, &col1)],
        body: node(&c, &body),
        rows: [node(&c, &r0), node(&c, &r1)],
        cells: [
            node(&c, &a),
            node(&c, &h),
            node(&c, &other),
            node(&c, &wide),
        ],
        nested: [node(&c, &nested), node(&c, &nested_cell)],
        separate: [node(&c, &separate), node(&c, &separate_cell)],
        dom,
    }
}

#[test]
fn native_table_maps_literal_slots_columns_groups_and_all_associated_subjects() {
    let g = grid();
    let snapshot = g.dom.snapshot();
    let c = capture(&snapshot, g.document);
    let report = run(&c, g.table);
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (3, 2));
    assert_eq!(c.node(m.table()).unwrap(), g.table);
    assert_eq!(c.node(m.caption().unwrap()).unwrap(), g.caption);
    // Row1 starts after occupied x0; colspan2 therefore covers x1 and x2.
    let cells: Vec<_> = m
        .cells()
        .iter()
        .map(|cell| {
            (
                c.node(cell.subject()).unwrap(),
                cell.kind(),
                cell.x(),
                cell.y(),
                cell.width(),
                cell.height(),
            )
        })
        .collect();
    assert_eq!(
        cells,
        vec![
            (g.cells[0], HtmlTableCellKind::Data, 0, 0, 1, 2),
            (g.cells[1], HtmlTableCellKind::Header, 1, 0, 1, 1),
            (g.cells[2], HtmlTableCellKind::Data, 2, 0, 1, 1),
            (g.cells[3], HtmlTableCellKind::Data, 1, 1, 2, 1),
        ]
    );
    assert_eq!(
        m.columns()
            .iter()
            .map(|col| c.node(col.source().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        vec![g.columns[0], g.columns[0], g.columns[1]]
    );
    assert_eq!(
        m.rows()
            .iter()
            .map(|row| c.node(row.source().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        g.rows
    );
    assert_eq!(
        m.column_groups()
            .iter()
            .map(|group| (c.node(group.subject()).unwrap(), group.x(), group.width()))
            .collect::<Vec<_>>(),
        vec![(g.group, 0, 3)]
    );
    assert_eq!(
        m.row_groups()
            .iter()
            .map(|group| (c.node(group.subject()).unwrap(), group.y(), group.height()))
            .collect::<Vec<_>>(),
        vec![(g.body, 0, 2)]
    );
    assert_eq!(
        m.headers()
            .iter()
            .map(|edge| (c.node(edge.cell()).unwrap(), c.node(edge.header()).unwrap()))
            .collect::<Vec<_>>(),
        vec![(g.cells[3], g.cells[1])]
    );
    assert!(m.errors().is_empty());
    // Nested and sibling table cells are not promoted into the outer grid.
    for [table, cell] in [g.nested, g.separate] {
        let small = run(&c, table);
        assert_eq!((model(&small).width(), model(&small).height()), (1, 1));
        assert_eq!(model(&small).cells().len(), 1);
        assert_eq!(c.node(model(&small).cells()[0].subject()).unwrap(), cell);
    }
    assert_eq!(report.capture(), c.capture().identity());
    assert_eq!(report.host_revision(), c.capture().host_facts_revision());
    assert_eq!(
        report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(c.basis(), snapshot.document(g.document).unwrap().stamp());
    assert_eq!(
        c.node(report.results()[0].request().subject()).unwrap(),
        g.table
    );
}

#[test]
fn overlapping_native_cells_keep_both_canonical_occupants_and_exact_error_pair() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let t = element(&mut b, &d, &d, "table", &[]);
    let r0 = element(&mut b, &d, &t, "tr", &[]);
    let a = element(&mut b, &d, &r0, "td", &[("headers", "")]);
    let down = element(&mut b, &d, &r0, "td", &[("rowspan", "2"), ("headers", "")]);
    let r1 = element(&mut b, &d, &t, "tr", &[]);
    let wide = element(&mut b, &d, &r1, "td", &[("colspan", "2"), ("headers", "")]);
    let commit = apply(&mut dom, b);
    let c = capture(&dom.snapshot(), commit.document(&d).unwrap());
    let report = run(&c, node(&commit, &t));
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (2, 2));
    assert_eq!(
        m.cells()
            .iter()
            .map(|cell| (
                c.node(cell.subject()).unwrap(),
                cell.x(),
                cell.y(),
                cell.width(),
                cell.height()
            ))
            .collect::<Vec<_>>(),
        vec![
            (node(&commit, &a), 0, 0, 1, 1),
            (node(&commit, &down), 1, 0, 1, 2),
            (node(&commit, &wide), 0, 1, 2, 1),
        ]
    );
    assert_eq!(
        m.errors(),
        &[HtmlTableModelError::Overlap {
            cell: c.subject(node(&commit, &wide)).unwrap(),
            other: c.subject(node(&commit, &down)).unwrap(),
            x: 1,
            y: 1,
        }]
    );
}

#[test]
fn semantic_span_mutation_recomputes_but_width_and_display_do_not_change_membership() {
    let mut g = grid();
    let old_snapshot = g.dom.snapshot();
    let old = capture(&old_snapshot, g.document);
    let old_report = run(&old, g.table);
    let mut b = DomBatch::new();
    b.set_attribute(g.table, "style", "display:none;width:999px");
    b.set_attribute(g.cells[3], "width", "999");
    apply(&mut g.dom, b);
    let display = capture(&g.dom.snapshot(), g.document);
    let display_report = run(&display, g.table);
    assert_ne!(old.basis(), display.basis());
    for report in [&old_report, &display_report] {
        let m = model(report);
        assert_eq!((m.width(), m.height()), (3, 2));
        assert_eq!(
            (m.cells()[3].x(), m.cells()[3].y(), m.cells()[3].width()),
            (1, 1, 2)
        );
    }
    let mut b = DomBatch::new();
    b.set_attribute(g.cells[3], "colspan", "1");
    apply(&mut g.dom, b);
    let changed = capture(&g.dom.snapshot(), g.document);
    let changed_report = run(&changed, g.table);
    assert_eq!(
        (
            model(&changed_report).cells()[3].x(),
            model(&changed_report).cells()[3].width()
        ),
        (1, 1)
    );
    assert_eq!(model(&old_report).cells()[3].width(), 2);
    assert_eq!(model(&run(&old, g.table)).cells()[3].width(), 2);
    assert_ne!(changed.basis(), old.basis());
    assert_eq!(
        changed_report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    // Old immutable capture remains valid, but its subjects cannot be mixed with a new capture.
    let old_subject = old.subject(g.table).unwrap();
    assert!(matches!(
        changed.node(&old_subject),
        Err(DomHtmlCaptureError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    let error = changed
        .query(
            &[HtmlSemanticRequest::new(
                old_subject,
                HtmlSemanticQuery::TableModel,
            )],
            HtmlValueInterpreter::new(8192),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
    let DomHtmlCaptureError::Semantic(error) = error else {
        panic!("foreign semantic request")
    };
    assert_eq!(
        error.kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert_eq!(
        error.qualification().unwrap().capture(),
        changed.capture().identity()
    );
}

#[test]
fn empty_html_table_foreign_namespace_and_nontable_are_distinct() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = element(&mut b, &d, &d, "div", &[]);
    let table = element(&mut b, &d, &root, "table", &[]);
    let foreign = b.create_element(&d, "table");
    b.insert_before(&foreign, &root, None);
    let commit = apply(&mut dom, b);
    let c = capture(&dom.snapshot(), commit.document(&d).unwrap());
    let empty = run(&c, node(&commit, &table));
    assert_eq!((model(&empty).width(), model(&empty).height()), (0, 0));
    assert!(model(&empty).cells().is_empty());
    assert!(model(&empty).errors().is_empty());
    assert_eq!(
        run(&c, node(&commit, &root)).results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        run(&c, node(&commit, &foreign)).results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn table_capture_and_query_refusals_leave_canonical_state_and_prior_result_intact() {
    let g = grid();
    let snapshot = g.dom.snapshot();
    let c = capture(&snapshot, g.document);
    let completed = run(&c, g.table);
    let usage = g.dom.usage();
    let stopped = snapshot
        .html_capture(
            g.document,
            None,
            HtmlCaptureLimits::new(128, 2048, 256, 32768, 128, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(0, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap_err();
    assert!(matches!(
        stopped,
        DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: 0
        }
    ));
    assert!(matches!(
        snapshot.html_capture(
            g.document,
            None,
            HtmlCaptureLimits::new(128, 2048, 256, 32768, 128, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &AtomicBool::new(true),
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    let request =
        HtmlSemanticRequest::new(c.subject(g.table).unwrap(), HtmlSemanticQuery::TableModel);
    let error = c
        .query(
            std::slice::from_ref(&request),
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 256, 10_000_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
    let DomHtmlCaptureError::Semantic(error) = error else {
        panic!("nested output admission")
    };
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(
        error.qualification().unwrap().capture(),
        c.capture().identity()
    );
    // The first actual numeric operand is col@span="2": one authored unit.
    let error = c
        .query(
            std::slice::from_ref(&request),
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
    let DomHtmlCaptureError::Semantic(error) = error else {
        panic!("raw attribute admission")
    };
    let HtmlSemanticFailureKind::Attribute(error) = error.kind() else {
        panic!("raw span limit")
    };
    assert_eq!(error.required(), 1);
    assert_eq!(error.capacity(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let error = c
        .query(
            &[request],
            HtmlValueInterpreter::new(8192),
            limits(),
            cancel,
        )
        .unwrap_err();
    let DomHtmlCaptureError::Semantic(error) = error else {
        panic!("semantic cancellation")
    };
    assert_eq!(error.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(g.dom.snapshot().stamp(), snapshot.stamp());
    assert_eq!(g.dom.usage(), usage);
    assert_eq!(
        (model(&completed).width(), model(&completed).height()),
        (3, 2)
    );
}

#[test]
fn stale_optional_host_basis_is_rejected_before_a_table_capture_is_published() {
    let mut g = grid();
    let before = g.dom.snapshot();
    let document = before.document(g.document).unwrap();
    let mut host = DomHostPublisher::new(DomHostDomains::DocumentTarget);
    let publication = host
        .publish(
            document,
            document.stamp(),
            DomHostObservations::DocumentTarget(Observed::Supported(None)),
        )
        .unwrap();
    let mut b = DomBatch::new();
    b.set_attribute(g.table, "colspan", "2");
    apply(&mut g.dom, b);
    let current = g.dom.snapshot();
    assert!(matches!(
        current.html_capture(
            g.document,
            Some(publication.capture()),
            HtmlCaptureLimits::new(128, 2048, 256, 32768, 128, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &AtomicBool::new(false),
        ),
        Err(DomHtmlCaptureError::Host(
            DomHostError::DomRevisionMismatch { .. }
        ))
    ));
    let c = capture(&current, g.document);
    let report = run(&c, g.table);
    assert_eq!((model(&report).width(), model(&report).height()), (3, 2));
}
