//! What an element retains for the strings of its name and namespace, and the origins a published
//! report answers for them.
//!
//! The name and the namespace of an element are the record's own strings, as the strings of its
//! attributes are. With the detail of the origin graph recorded their origin groups are in the
//! table of origins, keyed by the field; with no detail nothing is retained for them.
use super::*;

/// The report of a parse of an ordinary Document of `elements` plain `span` elements, with the
/// detail the request records, or `None` when this build cannot record that detail.
fn report_of(elements: usize, detail: crate::DetailSet) -> Option<crate::HtmlParseReport> {
    let source = format!("<!doctype html>{}", "<span></span>".repeat(elements));
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source.as_str()),
        g2_d2::corpus_limits(),
    )
    .with_detail(detail)
    .ok()?;
    Some(request.parse().unwrap())
}

/// The steps of the metadata cursor that carry no record, which are the registry rows the cursor
/// passes without a record to read.
fn skipped_steps(report: &crate::HtmlParseReport) -> usize {
    let mut skipped = 0;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            crate::HtmlParseMetadataStep::Record(_) => {}
            crate::HtmlParseMetadataStep::Skipped => skipped += 1,
            crate::HtmlParseMetadataStep::Complete => return skipped,
            crate::HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
}

/// The metadata cursor of a report passes one step for each registry row that has no record. A
/// plain element has no registry row for its name or its namespace, so the steps it passes without
/// a record do not depend on the plain elements of the document. The oracle is the registry rows
/// an element has independently of its strings: none without detail.
#[test]
fn the_metadata_cursor_passes_no_step_for_the_strings_of_a_plain_element() {
    let empty = skipped_steps(&report_of(0, crate::DetailSet::NONE).unwrap());
    for elements in [1, 16, 64] {
        let report = report_of(elements, crate::DetailSet::NONE).unwrap();
        assert_eq!(skipped_steps(&report), empty, "{elements} plain elements");
    }
}

/// Every element of a parse that records the origin graph has the origin group of the string of
/// its name and the group of the string of its namespace in the table of origins. With no detail
/// the table holds nothing and is never allocated.
#[test]
fn the_origin_table_holds_the_name_and_the_namespace_of_each_element() {
    use crate::construction::FieldRef;
    use surgeist_html::construction::ConstructionNodeKind;
    for detail in g2_d2::details() {
        let report = report_of(16, detail).unwrap();
        let storage = report.history().storage();
        let table = &storage.payloads.detail_origins;
        let mut elements = 0;
        for position in 0..storage.view.len() {
            let record = storage.view.node_at(position).unwrap();
            let ConstructionNodeKind::Element(element) = record.kind() else {
                continue;
            };
            elements += 1;
            let recorded = detail.records_graph();
            assert_eq!(
                table.has(FieldRef::ElementName(position)),
                recorded,
                "the name of element {position}, {detail:?}"
            );
            assert_eq!(
                table.has(FieldRef::Namespace(position)),
                recorded && element.name().namespace().uri().is_some(),
                "the namespace of element {position}, {detail:?}"
            );
        }
        assert!(elements >= 16, "the document has its elements");
        if !detail.records_graph() {
            assert!(table.is_unallocated(), "{detail:?}");
        }
    }
}

/// The retained auxiliary entries of a plain element are the one entry of its node join, whatever
/// the strings of its name and namespace are: the difference between documents of 16 and 64 plain
/// elements, without detail, is 48 entries.
#[test]
fn a_plain_element_retains_one_auxiliary_entry() {
    use crate::HtmlParseResource::AuxiliaryEntries;
    let entries = |elements: usize| {
        report_of(elements, crate::DetailSet::NONE)
            .unwrap()
            .usage()
            .resource(AuxiliaryEntries)
            .current()
    };
    assert_eq!(entries(64) - entries(16), 64 - 16);
}

/// The units of an element's name are counted once, with the element. A plain element of the HTML
/// namespace named `span` counts the units of `span` and of its namespace URI: the difference
/// between documents of 16 and 64 plain elements, without detail.
#[test]
fn a_plain_element_counts_the_units_of_its_name_and_namespace() {
    let units = |elements: usize| {
        let source = format!("<!doctype html>{}", "<span></span>".repeat(elements));
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(source.as_str()),
            g2_d2::corpus_limits(),
        )
        .parse()
        .unwrap()
        .usage()
        .resource(crate::HtmlParseResource::ManagedUtf16Units)
        .current()
    };
    let namespace = "http://www.w3.org/1999/xhtml".len() as u64;
    assert_eq!(
        units(64) - units(16),
        (64 - 16) * ("span".len() as u64 + namespace)
    );
}

/// The fragment context of an element in the namespace `urn:retained`, which no parse literal is.
pub(in crate::construction) fn retained_namespace_context() -> crate::HtmlFragmentContext {
    use crate::*;
    use surgeist_html::{
        construction::HtmlDocumentMode,
        values::{Namespace, OrderedAttributes, QualifiedName},
    };
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let context = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::from_uri("urn:retained"), None, "context"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&context).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap()
}

/// The registry rows that hold the namespace of a fragment context that no literal names, in a parse
/// of `elements` elements in it.
fn retained_namespace_rows(elements: usize) -> usize {
    let source = "<foo></foo>".repeat(elements);
    let report = crate::HtmlFragmentParseRequest::new(
        crate::HtmlSource::utf8(source.as_str()),
        retained_namespace_context(),
        g2_d2::corpus_limits(),
    )
    .unwrap()
    .with_detail(crate::DetailSet::NONE)
    .unwrap()
    .parse()
    .unwrap();
    report
        .history()
        .storage()
        .payloads
        .values
        .iter()
        .filter(|row| {
            matches!(row, super::RetainedPayload::String { value, .. }
                if value.to_utf8().is_ok_and(|text| text == "urn:retained"))
        })
        .count()
}

/// The elements of a fragment in a captured namespace share the string of that namespace, as the
/// elements of a literal namespace share its string: the registry holds one row for it, whatever
/// the number of elements.
#[test]
fn the_elements_in_a_captured_namespace_share_one_registry_row() {
    for elements in [1, 4, 16] {
        assert_eq!(retained_namespace_rows(elements), 1, "{elements} elements");
    }
}
