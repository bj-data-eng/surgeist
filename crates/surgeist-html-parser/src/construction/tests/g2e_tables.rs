//! What the nodes of a parse retain in the payload registry for their template metadata, their
//! declarative shadow state and the cause of their creation.
//!
//! The metadata of a template, the state of a shadow root and the cause of a node's creation are
//! facts of the node that holds them, so no registry row is kept for them: a table of each family
//! holds a row for the nodes that have one, found by the node's position.
use super::*;

/// The registry rows of a parse of an ordinary Document with `source`, with the detail the request
/// records, or `None` when this build cannot record that detail.
fn document_rows(source: &str, detail: crate::DetailSet) -> Option<usize> {
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source),
        g2_d2::corpus_limits(),
    )
    .with_detail(detail)
    .ok()?;
    let report = request.parse().unwrap();
    Some(report.history().storage().payloads.values.len())
}

/// The report of a fragment parse of `source` in a `div` that allows declarative shadow roots, with
/// the detail the request records, or `None` when this build cannot record that detail.
fn shadow_report(source: &str, detail: crate::DetailSet) -> Option<crate::HtmlParseReport> {
    let request = crate::HtmlFragmentParseRequest::with_options(
        crate::HtmlSource::utf8(source),
        g2_d2::fragment_context("div").unwrap(),
        g2_d2::corpus_limits(),
        surgeist_html::parsing::HtmlParserScriptingMode::Inert,
        true,
    )
    .unwrap()
    .with_detail(detail)
    .ok()?;
    Some(request.parse().unwrap())
}

/// The registry rows of a report that are not strings or retired list rows: the rows that belong
/// to a node and its metadata. A retired row is a work list that an effect released, and the
/// strings are the families the later parts of the move still keep in the registry.
fn node_rows(report: &crate::HtmlParseReport) -> usize {
    report
        .history()
        .storage()
        .payloads
        .values
        .iter()
        .filter(|row| {
            !matches!(
                row,
                RetainedPayload::String { .. } | RetainedPayload::Retired
            )
        })
        .count()
}

/// The metadata records of a report that carry the template metadata and the shadow state of a
/// node: the public read of both.
fn template_and_shadow_records(report: &crate::HtmlParseReport) -> (usize, usize) {
    let (mut templates, mut shadows) = (0, 0);
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            crate::HtmlParseMetadataStep::Record(record) => {
                templates += usize::from(record.template().is_some());
                shadows += usize::from(record.shadow().is_some());
            }
            crate::HtmlParseMetadataStep::Skipped => {}
            crate::HtmlParseMetadataStep::Complete => return (templates, shadows),
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// A `template` element holds no registry row: not for its metadata, and not, with the detail of
/// causes recorded, for the causes of its creation and of its contents. The oracle is that the
/// element and its contents are the only nodes the template adds, and that every one of the
/// elements the parse creates has its template metadata among the records the report answers.
#[test]
fn a_template_element_holds_no_registry_row() {
    for detail in g2_d2::details() {
        let Some(empty) = document_rows("<!doctype html>", detail) else {
            continue;
        };
        for templates in [1_usize, 16, 64] {
            let source = format!(
                "<!doctype html>{}",
                "<template></template>".repeat(templates)
            );
            let rows = document_rows(&source, detail).unwrap();
            assert_eq!(rows, empty, "{templates} templates, {detail:?}");
            let report = crate::HtmlDocumentParseRequest::offline(
                crate::HtmlSource::utf8(source.as_str()),
                g2_d2::corpus_limits(),
            )
            .with_detail(detail)
            .unwrap()
            .parse()
            .unwrap();
            assert_eq!(
                template_and_shadow_records(&report).0,
                templates,
                "{templates} templates, {detail:?}"
            );
        }
    }
}

/// A template inside a template holds no registry row either.
#[test]
fn nested_templates_hold_no_registry_row() {
    for detail in g2_d2::details() {
        let Some(empty) = document_rows("<!doctype html>", detail) else {
            continue;
        };
        let source = format!(
            "<!doctype html>{}",
            "<template><template><template></template></template></template>".repeat(8)
        );
        assert_eq!(document_rows(&source, detail).unwrap(), empty, "{detail:?}");
    }
}

/// A shadow host holds no registry row for its shadow state, nor for its template element or the
/// contents of its shadow root, whatever the detail. The oracle is that every unit the source
/// repeats has one shadow state and one template among the records the report answers.
#[test]
fn a_declarative_shadow_host_holds_no_registry_row() {
    for detail in g2_d2::details() {
        let Some(empty) = shadow_report("", detail) else {
            continue;
        };
        let empty_rows = node_rows(&empty);
        for hosts in [1_usize, 16, 64] {
            let source = "<div><template shadowrootmode=open></template></div>".repeat(hosts);
            let report = shadow_report(&source, detail).unwrap();
            assert_eq!(node_rows(&report), empty_rows, "{hosts} hosts, {detail:?}");
            assert_eq!(
                template_and_shadow_records(&report),
                (hosts, hosts),
                "{hosts} hosts, {detail:?}"
            );
        }
    }
}

/// The growth of the parse facts by the three tables is the sum of their sizes: a table is its
/// key bank and its row bank, a vector and a capacity record for each and the count of admitted
/// rows, whatever its row, and the facts are the sum of their fields. The oracle is the
/// layout of the standard `Vec`: three words, and of the capacity record: its owner, its
/// capacity and its admitted count.
#[test]
fn the_three_tables_add_the_sum_of_their_sizes_to_the_parse_facts() {
    use super::super::family_table::FamilyTable;
    use super::super::payload_registry::ParseFacts;
    use crate::parser::control::VectorCapacity;
    use std::mem::size_of;
    let word = size_of::<usize>();
    let capacity = 3 * word;
    assert_eq!(size_of::<Vec<u32>>(), 3 * word);
    assert_eq!(size_of::<VectorCapacity<u32>>(), capacity);
    // The key bank (vector and capacity) and the row bank (vector, capacity, admitted count).
    let table = (3 * word + capacity) + (3 * word + capacity + word);
    assert_eq!(size_of::<FamilyTable<u8>>(), table);
    assert_eq!(size_of::<FamilyTable<template::TemplateMetadata>>(), table);
    assert_eq!(size_of::<FamilyTable<context::ShadowState>>(), table);
    assert_eq!(
        size_of::<FamilyTable<provenance::ConstructionCauseRef>>(),
        table
    );
    assert_eq!(table, 104);
    // The facts hold three tables more than the one of each earlier family: their size is the
    // sum of the sizes of their fields, rounded to the alignment.
    let mut c = control();
    let storage = ParserArenaSession::new(
        &mut c,
        RootKind::Document(surgeist_html::construction::HtmlDocumentMode::NoQuirks),
    )
    .unwrap();
    let facts = &storage.active().unwrap().payloads.facts;
    let fields = std::mem::size_of_val(&facts.completion)
        + std::mem::size_of_val(&facts.documents)
        + std::mem::size_of_val(&facts.csp)
        + std::mem::size_of_val(&facts.missing)
        + std::mem::size_of_val(&facts.associations)
        + std::mem::size_of_val(&facts.contexts)
        + std::mem::size_of_val(&facts.creation)
        + std::mem::size_of_val(&facts.nonces)
        + std::mem::size_of_val(&facts.templates)
        + std::mem::size_of_val(&facts.shadows)
        + std::mem::size_of_val(&facts.causes);
    assert_eq!(
        size_of::<ParseFacts>(),
        fields.next_multiple_of(std::mem::align_of::<ParseFacts>())
    );
    let new_tables = std::mem::size_of_val(&facts.templates)
        + std::mem::size_of_val(&facts.shadows)
        + std::mem::size_of_val(&facts.causes);
    assert_eq!(new_tables, 3 * table);
    let mut storage = storage;
    storage.release(&mut c).unwrap();
}
