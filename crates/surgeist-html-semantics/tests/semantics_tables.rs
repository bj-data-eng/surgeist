use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
    HtmlSemanticUnsupported, HtmlSemanticValue, HtmlTableCellKind, HtmlTableModel,
    HtmlTableModelError,
};
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10000, 100000, 10000, 1000000, 1000, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    let attrs = OrderedAttributes::new(attrs.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap();
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn cell(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
    text: &str,
) -> HtmlCaptureBuilderSubject {
    let e = element(b, d, local, attrs);
    let t = b.add_text(d, DocumentString::from(text)).unwrap();
    b.establish_children(&e, &[t]).unwrap();
    e
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(100000000, 100, 100000, 10000, 100000000, 100000)
}

fn run(c: &HtmlCapture, e: &HtmlCaptureBuilderSubject) -> HtmlSemanticReport {
    let cx = context(c);
    HtmlSemanticSession::new(
        c,
        &cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        c.subject_id(e).unwrap(),
        HtmlSemanticQuery::TableModel,
    )])
    .unwrap()
}
fn model(report: &HtmlSemanticReport) -> &HtmlTableModel {
    match report.results()[0].fact() {
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected complete table: {other:?}"),
    }
}
fn edges(m: &HtmlTableModel) -> Vec<(&HtmlCaptureSubjectId, &HtmlCaptureSubjectId)> {
    m.headers()
        .iter()
        .map(|edge| (edge.cell(), edge.header()))
        .collect()
}

#[test]
fn empty_table_and_first_caption_do_not_require_unrelated_children() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let one = element(&mut b, &d, "caption", &[]);
    let two = element(&mut b, &d, "caption", &[]);
    b.establish_children(&table, &[one.clone(), two]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (0, 0));
    assert_eq!(m.caption(), Some(&c.subject_id(&one).unwrap()));
    assert!(m.errors().is_empty());
    assert!(m.cells().is_empty());
}

#[test]
fn column_groups_preserve_col_sources_and_ignore_own_or_late_spans() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let cg = element(&mut b, &d, "colgroup", &[("span", "99")]);
    let col = element(&mut b, &d, "col", &[("span", " +2tail")]);
    b.establish_children(&cg, std::slice::from_ref(&col))
        .unwrap();
    let empty = element(&mut b, &d, "colgroup", &[("span", "-0")]);
    b.establish_children(&empty, &[]).unwrap();
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(&mut b, &d, "td", &[("colspan", "3")], "x");
    b.establish_children(&row, &[td]).unwrap();
    let late = element(&mut b, &d, "colgroup", &[("span", "500")]);
    b.establish_children(&table, &[cg.clone(), empty.clone(), row, late])
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (3, 1));
    assert_eq!(m.column_groups().len(), 2);
    assert_eq!(
        (m.column_groups()[0].x(), m.column_groups()[0].width()),
        (0, 2)
    );
    assert_eq!(m.column_groups()[0].subject(), &c.subject_id(&cg).unwrap());
    assert_eq!(
        (m.column_groups()[1].x(), m.column_groups()[1].width()),
        (2, 1)
    );
    assert_eq!(m.columns()[0].source(), Some(&c.subject_id(&col).unwrap()));
    assert_eq!(m.columns()[1].source(), m.columns()[0].source());
    assert_eq!(m.columns()[2].source(), None);
}

#[test]
fn cell_span_recovery_clamps_huge_values_and_keeps_negative_zero_growth() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let body = element(&mut b, &d, "tbody", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(
        &mut b,
        &d,
        "td",
        &[
            ("colspan", "999999999999999999999999999999999999"),
            ("rowspan", "-0rest"),
        ],
        "x",
    );
    let row2 = element(&mut b, &d, "tr", &[]);
    b.establish_children(&row2, &[]).unwrap();
    b.establish_children(&row, &[td]).unwrap();
    b.establish_children(&body, &[row, row2]).unwrap();
    b.establish_children(&table, &[body]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (1000, 2));
    assert_eq!((m.cells()[0].width(), m.cells()[0].height()), (1000, 2));
    assert!(
        m.errors()
            .contains(&HtmlTableModelError::UnanchoredRow { y: 1 })
    );
    assert!(
        m.errors()
            .contains(&HtmlTableModelError::UnanchoredColumn { x: 999 })
    );
}

#[test]
fn malformed_negative_and_zero_colspan_default_but_rowspan_zero_is_distinct() {
    for (colspan, rowspan, width, height) in [
        ("0", "garbage", 1, 1),
        ("-7", "-2", 1, 1),
        ("+2tail", "2tail", 2, 2),
    ] {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        let row = element(&mut b, &d, "tr", &[]);
        let td = cell(
            &mut b,
            &d,
            "td",
            &[("colspan", colspan), ("rowspan", rowspan)],
            "x",
        );
        b.establish_children(&row, &[td]).unwrap();
        b.establish_children(&table, &[row]).unwrap();
        let c = b.finish().unwrap();
        let report = run(&c, &table);
        let m = model(&report);
        assert_eq!(
            (m.cells()[0].width(), m.cells()[0].height()),
            (width, height)
        );
        assert_eq!((m.width(), m.height()), (width, height));
        if height == 2 {
            assert_eq!(m.rows()[1].source(), None);
            assert!(
                m.errors()
                    .contains(&HtmlTableModelError::UnanchoredRow { y: 1 })
            );
        }
    }
}

#[test]
fn downward_cells_end_at_group_boundaries_and_footers_move_after_body() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let footer = element(&mut b, &d, "tfoot", &[]);
    let fr = element(&mut b, &d, "tr", &[]);
    let f = cell(&mut b, &d, "td", &[], "f");
    b.establish_children(&fr, std::slice::from_ref(&f)).unwrap();
    b.establish_children(&footer, std::slice::from_ref(&fr))
        .unwrap();
    let body = element(&mut b, &d, "tbody", &[]);
    let first = element(&mut b, &d, "tr", &[]);
    let zero = cell(&mut b, &d, "td", &[("rowspan", "0")], "b");
    b.establish_children(&first, std::slice::from_ref(&zero))
        .unwrap();
    let second = element(&mut b, &d, "tr", &[]);
    b.establish_children(&second, &[]).unwrap();
    b.establish_children(&body, &[first.clone(), second.clone()])
        .unwrap();
    b.establish_children(&table, &[footer.clone(), body.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!(m.cells()[0].subject(), &c.subject_id(&zero).unwrap());
    assert_eq!((m.cells()[0].y(), m.cells()[0].height()), (0, 2));
    assert_eq!(m.cells()[1].subject(), &c.subject_id(&f).unwrap());
    assert_eq!(m.cells()[1].y(), 2);
    assert_eq!(m.rows()[0].source(), Some(&c.subject_id(&first).unwrap()));
    assert_eq!(m.rows()[1].source(), Some(&c.subject_id(&second).unwrap()));
    assert_eq!(m.rows()[2].source(), Some(&c.subject_id(&fr).unwrap()));
    assert_eq!(m.row_groups()[0].subject(), &c.subject_id(&body).unwrap());
    assert_eq!((m.row_groups()[0].y(), m.row_groups()[0].height()), (0, 2));
    assert_eq!(m.row_groups()[1].subject(), &c.subject_id(&footer).unwrap());
}

#[test]
fn occupied_start_is_skipped_and_overlap_is_retained_as_recovered_data() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let r1 = element(&mut b, &d, "tr", &[]);
    let r2 = element(&mut b, &d, "tr", &[]);
    let a = cell(&mut b, &d, "td", &[], "a");
    let bcell = cell(&mut b, &d, "td", &[("rowspan", "2")], "b");
    let ccell = cell(&mut b, &d, "td", &[("colspan", "2")], "c");
    b.establish_children(&r1, &[a, bcell.clone()]).unwrap();
    b.establish_children(&r2, std::slice::from_ref(&ccell))
        .unwrap();
    b.establish_children(&table, &[r1, r2]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!(
        (m.cells()[2].x(), m.cells()[2].y(), m.cells()[2].width()),
        (0, 1, 2)
    );
    assert!(m.errors().contains(&HtmlTableModelError::Overlap {
        cell: c.subject_id(&ccell).unwrap(),
        other: c.subject_id(&bcell).unwrap(),
        x: 1,
        y: 1
    }));
    assert_eq!(m.cells().len(), 3);
}

#[test]
fn explicit_headers_use_first_document_id_even_when_it_is_not_a_cell() {
    let (mut b, d) = builder();
    let outsider = element(&mut b, &d, "div", &[("id", "h")]);
    b.establish_children(&outsider, &[]).unwrap();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = cell(&mut b, &d, "th", &[("id", "h")], "heading");
    let td = cell(&mut b, &d, "td", &[("headers", "h h")], "data");
    b.establish_children(&row, &[h, td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, &[outsider, table.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert!(model(&report).headers().is_empty());
}

#[test]
fn explicit_empty_headers_suppress_automatic_scans_without_document_facts() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = cell(&mut b, &d, "th", &[("scope", "row"), ("headers", "")], "h");
    let td = cell(&mut b, &d, "td", &[("headers", "")], "d");
    b.establish_children(&row, &[h, td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert!(model(&report).headers().is_empty());
}

#[test]
fn header_tokens_deduplicate_and_empty_filter_uses_only_ascii_child_whitespace() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h1 = cell(
        &mut b,
        &d,
        "th",
        &[("id", "one"), ("headers", "")],
        " \t\r\n",
    );
    let h2 = cell(
        &mut b,
        &d,
        "th",
        &[("id", "two"), ("headers", "")],
        "\u{a0}",
    );
    let td = cell(&mut b, &d, "td", &[("headers", "one\ttwo two")], "d");
    b.establish_children(&row, &[h1, h2.clone(), td.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&table))
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(&c.subject_id(&td).unwrap(), &c.subject_id(&h2).unwrap())]
    );
}

#[test]
fn automatic_row_column_and_group_headers_keep_source_association_order() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let body = element(&mut b, &d, "tbody", &[]);
    let r1 = element(&mut b, &d, "tr", &[]);
    let corner = cell(&mut b, &d, "th", &[("scope", "rowgroup")], "group");
    let col = cell(&mut b, &d, "th", &[("scope", "CoL")], "column");
    let r2 = element(&mut b, &d, "tr", &[]);
    let row = cell(&mut b, &d, "th", &[("scope", "row")], "row");
    let data = cell(&mut b, &d, "td", &[], "data");
    b.establish_children(&r1, &[corner.clone(), col.clone()])
        .unwrap();
    b.establish_children(&r2, &[row.clone(), data.clone()])
        .unwrap();
    b.establish_children(&body, &[r1, r2]).unwrap();
    b.establish_children(&table, &[body]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let data_id = c.subject_id(&data).unwrap();
    let actual: Vec<_> = model(&report)
        .headers()
        .iter()
        .filter(|e| e.cell() == &data_id)
        .map(|e| e.header().clone())
        .collect();
    assert_eq!(
        actual,
        vec![
            c.subject_id(&row).unwrap(),
            c.subject_id(&col).unwrap(),
            c.subject_id(&corner).unwrap()
        ]
    );
}

#[test]
fn missing_fact_identifies_the_actual_ordered_child_owner() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    b.establish_children(&table, std::slice::from_ref(&row))
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&row).unwrap())
        ))
    );
}

#[test]
fn terminal_capacity_failure_is_qualified_sticky_and_publishes_no_prefix() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(&mut b, &d, "td", &[], "d");
    b.establish_children(&row, &[td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let caps = HtmlSemanticLimits::new(100000, 2, 1000, 1000, 100000, 0);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1000),
        caps,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let req =
        HtmlSemanticRequest::new(c.subject_id(&table).unwrap(), HtmlSemanticQuery::TableModel);
    let error = session.query(std::slice::from_ref(&req)).unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(error.qualification().unwrap().capture(), c.identity());
    assert_eq!(error.qualification().unwrap().limits(), caps);
    assert_eq!(session.query(&[req]).unwrap_err(), error);
    assert_eq!(session.usage().scratch_records(), 0);
}

#[test]
fn cancellation_keeps_completed_models_and_later_failure_qualification() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    b.establish_children(&table, &[]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1000),
        limits(),
        cancel.clone(),
    )
    .unwrap();
    let req =
        HtmlSemanticRequest::new(c.subject_id(&table).unwrap(), HtmlSemanticQuery::TableModel);
    let completed = session.query(std::slice::from_ref(&req)).unwrap();
    cancel.cancel();
    let stop = session.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(stop.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(stop.qualification().unwrap().capture(), c.identity());
    assert_eq!(session.query(&[req]).unwrap_err(), stop);
    assert_eq!(
        (model(&completed).width(), model(&completed).height()),
        (0, 0)
    );
}

#[test]
fn document_id_search_stops_after_a_proved_first_match_before_unknown_later_branch() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")], "h");
    let data = cell(&mut b, &d, "td", &[("headers", "h")], "d");
    let unknown = element(&mut b, &d, "section", &[]);
    b.establish_children(&row, &[h.clone(), data.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, &[table.clone(), unknown]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(&c.subject_id(&data).unwrap(), &c.subject_id(&h).unwrap())]
    );
}

#[test]
fn unresolved_explicit_id_requires_document_children_but_not_table_parent() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let data = cell(&mut b, &d, "td", &[("headers", "absent")], "d");
    b.establish_children(&row, &[data]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&d).unwrap())
        ))
    );
}

#[test]
fn document_lookup_excludes_shadow_and_template_associations() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    b.establish_children(&host, &[]).unwrap();
    let shadow = b
        .add_shadow_root(
            &host,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let shadow_id = element(&mut b, &d, "div", &[("id", "h")]);
    b.establish_children(&shadow, &[shadow_id]).unwrap();
    let template = element(&mut b, &d, "template", &[]);
    b.establish_children(&template, &[]).unwrap();
    let contents = b.add_fragment(&d).unwrap();
    let template_id = element(&mut b, &d, "div", &[("id", "h")]);
    b.establish_children(&contents, &[template_id]).unwrap();
    b.establish_template_contents(&template, &contents).unwrap();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")], "h");
    let data = cell(&mut b, &d, "td", &[("headers", "h")], "d");
    b.establish_children(&row, &[h.clone(), data.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, &[host, template, table.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(&c.subject_id(&data).unwrap(), &c.subject_id(&h).unwrap())]
    );
}

#[test]
fn explicit_ids_preserve_isolated_surrogates_and_case_sensitive_tokens() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let attrs = OrderedAttributes::new([
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "id"),
            DocumentString::from_utf16(vec![0xd800, 0x41]),
        ),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "headers"), ""),
    ])
    .unwrap();
    let h = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "th"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let text = b.add_text(&d, DocumentString::from("h")).unwrap();
    b.establish_children(&h, &[text]).unwrap();
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "headers"),
        DocumentString::from_utf16(vec![0xd800, 0x41, 32, 0xd800, 0x61]),
    )])
    .unwrap();
    let data = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "td"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&data, &[]).unwrap();
    b.establish_children(&row, &[h.clone(), data.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&table))
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(&c.subject_id(&data).unwrap(), &c.subject_id(&h).unwrap())]
    );
}

#[test]
fn opaque_column_header_blocks_prevent_earlier_equal_width_headers() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    for (kind, text) in [("th", "a"), ("td", "b"), ("th", "c"), ("td", "d")] {
        let row = element(&mut b, &d, "tr", &[]);
        let c = cell(&mut b, &d, kind, &[], text);
        b.establish_children(&row, std::slice::from_ref(&c))
            .unwrap();
        rows.push(row);
        cells.push(c);
    }
    b.establish_children(&table, &rows).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let last = c.subject_id(&cells[3]).unwrap();
    let actual: Vec<_> = model(&report)
        .headers()
        .iter()
        .filter(|edge| edge.cell() == &last)
        .map(|edge| edge.header().clone())
        .collect();
    assert_eq!(actual, vec![c.subject_id(&cells[2]).unwrap()]);
}

#[test]
fn automatic_row_headers_and_column_group_headers_use_anchor_membership() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let cg = element(&mut b, &d, "colgroup", &[("span", "2")]);
    b.establish_children(&cg, &[]).unwrap();
    let first = element(&mut b, &d, "tr", &[]);
    let group = cell(&mut b, &d, "th", &[("scope", "colgroup")], "group");
    let one = cell(&mut b, &d, "td", &[], "one");
    b.establish_children(&first, &[group.clone(), one.clone()])
        .unwrap();
    let second = element(&mut b, &d, "tr", &[]);
    let automatic = cell(&mut b, &d, "th", &[], "row");
    let two = cell(&mut b, &d, "td", &[], "two");
    b.establish_children(&second, &[automatic.clone(), two.clone()])
        .unwrap();
    b.establish_children(&table, &[cg, first, second]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let data = c.subject_id(&two).unwrap();
    let actual: Vec<_> = model(&report)
        .headers()
        .iter()
        .filter(|edge| edge.cell() == &data)
        .map(|edge| edge.header().clone())
        .collect();
    assert_eq!(
        actual,
        vec![
            c.subject_id(&automatic).unwrap(),
            c.subject_id(&group).unwrap()
        ]
    );
}

#[test]
fn cell_elements_prove_nonempty_without_descendant_text_or_template_contents() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = element(&mut b, &d, "th", &[("scope", "row")]);
    let nested = element(&mut b, &d, "template", &[]);
    b.establish_children(&h, &[nested]).unwrap();
    let data = cell(&mut b, &d, "td", &[], "d");
    b.establish_children(&row, &[h.clone(), data.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(&c.subject_id(&data).unwrap(), &c.subject_id(&h).unwrap())]
    );
}

#[test]
fn scratch_cap_fails_at_footer_admission_and_cleanup_remains_balanced() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let footer = element(&mut b, &d, "tfoot", &[]);
    b.establish_children(&table, &[footer]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1000),
        HtmlSemanticLimits::new(100000, 1, 0, 1000, 100000, 1000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let stop = session
        .query(&[HtmlSemanticRequest::new(
            c.subject_id(&table).unwrap(),
            HtmlSemanticQuery::TableModel,
        )])
        .unwrap_err();
    assert!(matches!(
        stop.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            required: 1,
            ..
        }
    ));
    assert_eq!(session.usage().scratch_records(), 0);
}

#[test]
fn exact_work_cutoff_and_duplicate_requests_preserve_atomic_batch_results() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(&mut b, &d, "td", &[], "d");
    b.establish_children(&row, &[td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let req =
        HtmlSemanticRequest::new(c.subject_id(&table).unwrap(), HtmlSemanticQuery::TableModel);
    let baseline = run(&c, &table);
    let work = baseline.usage().work();
    for cap in [work - 1, work] {
        let mut session = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(10000),
            HtmlSemanticLimits::new(cap, 100, 100000, 10000, 100000000, 100000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let result = session.query(std::slice::from_ref(&req));
        if cap == work {
            assert_eq!(model(&result.unwrap()), model(&baseline));
        } else {
            assert!(matches!(
                result.unwrap_err().kind(),
                HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::Work,
                    ..
                }
            ));
        }
        assert_eq!(session.usage().scratch_records(), 0);
    }
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let repeated = session.query(&[req.clone(), req.clone()]).unwrap();
    assert_eq!(repeated.results().len(), 2);
    assert_eq!(repeated.results()[0].fact(), repeated.results()[1].fact());
    let nested = baseline.usage().nested_output_records();
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(100000000, 100, 100000, 10000, 100000000, nested),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        session.query(&[req.clone(), req]).unwrap_err().kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
}

#[test]
fn table_attribute_input_is_admitted_before_numeric_recovery() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(&mut b, &d, "td", &[("colspan", "9999")], "d");
    b.establish_children(&row, &[td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(3),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let stop = session
        .query(&[HtmlSemanticRequest::new(
            c.subject_id(&table).unwrap(),
            HtmlSemanticQuery::TableModel,
        )])
        .unwrap_err();
    match stop.kind() {
        HtmlSemanticFailureKind::Attribute(error) => {
            assert_eq!(error.required(), 4);
            assert_eq!(error.capacity(), 3);
        }
        other => panic!("expected raw admission: {other:?}"),
    }
}

#[test]
fn known_non_table_is_inapplicable_and_foreign_table_is_unsupported() {
    let (mut b, d) = builder();
    let div = element(&mut b, &d, "div", &[]);
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::none(), None, "table"),
            OrderedAttributes::new([]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        run(&c, &div).results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        run(&c, &foreign).results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}

#[test]
fn enormous_rowspan_clamps_to_65534_and_implied_rows_have_no_source() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let td = cell(
        &mut b,
        &d,
        "td",
        &[("rowspan", "999999999999999999999999999999999999999999999")],
        "d",
    );
    b.establish_children(&row, &[td]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(100000000, 100, 100000, 10000, 100000000, 200000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let report = session
        .query(&[HtmlSemanticRequest::new(
            c.subject_id(&table).unwrap(),
            HtmlSemanticQuery::TableModel,
        )])
        .unwrap();
    let m = model(&report);
    assert_eq!((m.height(), m.cells()[0].height()), (65534, 65534));
    assert_eq!(m.rows()[65533].source(), None);
    assert!(
        m.errors()
            .contains(&HtmlTableModelError::UnanchoredRow { y: 65533 })
    );
}

#[test]
fn occupied_first_slot_moves_the_next_row_cell_to_the_next_free_column() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let r1 = element(&mut b, &d, "tr", &[]);
    let r2 = element(&mut b, &d, "tr", &[]);
    let a = cell(&mut b, &d, "td", &[("rowspan", "2")], "a");
    let next = cell(&mut b, &d, "td", &[], "b");
    b.establish_children(&r1, &[a]).unwrap();
    b.establish_children(&r2, std::slice::from_ref(&next))
        .unwrap();
    b.establish_children(&table, &[r1, r2]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!((m.cells()[1].x(), m.cells()[1].y()), (1, 1));
    assert_eq!(m.cells()[1].subject(), &c.subject_id(&next).unwrap());
    assert!(m.errors().is_empty());
}

#[test]
fn missing_header_children_are_required_only_when_filtering_that_candidate() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = element(&mut b, &d, "th", &[("scope", "row")]);
    let data = cell(&mut b, &d, "td", &[], "d");
    b.establish_children(&row, &[h.clone(), data]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&h).unwrap())
        ))
    );
}

#[test]
fn foreign_capture_table_request_retains_the_admitted_session_qualification() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    b.establish_children(&table, &[]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let (mut other, od) = builder();
    let foreign = element(&mut other, &od, "table", &[]);
    let other = other.finish().unwrap();
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let stop = session
        .query(&[HtmlSemanticRequest::new(
            other.subject_id(&foreign).unwrap(),
            HtmlSemanticQuery::TableModel,
        )])
        .unwrap_err();
    assert_eq!(
        stop.kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert_eq!(stop.qualification().unwrap().capture(), c.identity());
}

#[test]
fn column_only_table_retains_empty_dimensions_and_the_source_anchor_error() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let group = element(&mut b, &d, "colgroup", &[]);
    b.establish_children(&group, &[]).unwrap();
    b.establish_children(&table, &[group]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    assert_eq!((m.width(), m.height()), (1, 0));
    assert_eq!(
        m.errors(),
        &[HtmlTableModelError::UnanchoredColumn { x: 0 }]
    );
}

#[test]
fn direct_row_run_end_does_not_add_an_unrequested_downward_growth_step() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let fixed = cell(&mut b, &d, "td", &[("rowspan", "3")], "fixed");
    let growing = cell(&mut b, &d, "td", &[("rowspan", "0")], "growing");
    b.establish_children(&row, &[fixed, growing]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    let m = model(&report);
    // Forming-a-table End goes directly to pending footers and anchor errors.
    // It does not invoke ending-a-row-group for this trailing direct-row run.
    assert_eq!(m.height(), 3);
    assert_eq!(m.cells()[0].height(), 3);
    assert_eq!(m.cells()[1].height(), 1);
    assert!(m.row_groups().is_empty());
}

#[test]
fn explicit_headers_can_name_data_cells_but_exclude_the_principal_cell() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let named = cell(
        &mut b,
        &d,
        "td",
        &[("id", "named"), ("headers", "")],
        "named data",
    );
    let principal = cell(
        &mut b,
        &d,
        "td",
        &[("id", "self"), ("headers", "named self named")],
        "principal",
    );
    b.establish_children(&row, &[named.clone(), principal.clone()])
        .unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, std::slice::from_ref(&table))
        .unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert_eq!(
        edges(model(&report)),
        vec![(
            &c.subject_id(&principal).unwrap(),
            &c.subject_id(&named).unwrap()
        )]
    );
    assert_eq!(model(&report).cells()[0].kind(), HtmlTableCellKind::Data);
}

#[test]
fn complete_empty_document_proves_no_explicit_header_id_without_missing_evidence() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")], "h");
    let data = cell(&mut b, &d, "td", &[("headers", "h")], "d");
    b.establish_children(&row, &[h, data]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    b.establish_children(&d, &[]).unwrap();
    let c = b.finish().unwrap();
    let report = run(&c, &table);
    assert!(model(&report).headers().is_empty());
    assert_eq!(model(&report).cells().len(), 2);
}
