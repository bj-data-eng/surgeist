//! A published report is qualified once, so a reference it holds that its own
//! tables cannot resolve is a defect in that qualification. Each test corrupts
//! one such reference in a published report and requires the reader that
//! follows it to report the fault as a typed error, never to panic and never
//! to answer as though the reference were absent.
use super::*;
use history::{FrozenHistory, HistoryFrame, HistoryReadError, HistoryStep};
use provenance::{GroupBody, LeafValue, NativeDestination};
use read::{HtmlParseReverseStep, UnresolvedKind as Kind, UnresolvedReference};
use report::MetadataStep;

fn parse(html: &str) -> crate::HtmlParseReport {
    crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(html),
        crate::parser::tree_builder::tests::limits(100),
    )
    .parse()
    .unwrap()
}
fn history(report: &mut crate::HtmlParseReport) -> &mut FrozenHistory {
    report.storage_mut().history_mut()
}
fn frozen(report: &mut crate::HtmlParseReport) -> &mut FrozenStorage {
    history(report).storage_mut()
}
fn fault(kind: Kind) -> UnresolvedReference {
    UnresolvedReference::new(kind)
}

// ---- the history ----

/// Walks the history of `group` and returns its first fault, after requiring
/// the cursor to stay faulted.
fn walk(report: &crate::HtmlParseReport, group: usize) -> Option<UnresolvedReference> {
    let view = report.history_view();
    let group = view.group(group).unwrap();
    let mut scratch = vec![HistoryFrame::default(); view.len()];
    let mut cursor = view.cursor(group, &mut scratch, None).unwrap();
    for _ in 0..10_000 {
        match cursor.advance() {
            HistoryStep::Fault(first) => {
                assert!(matches!(cursor.advance(), HistoryStep::Fault(again) if again == first));
                return Some(first);
            }
            HistoryStep::Complete | HistoryStep::Cancelled => return None,
            HistoryStep::Progress | HistoryStep::Leaf(_) => {}
        }
    }
    panic!("a history walk ends")
}
fn first_group(
    report: &mut crate::HtmlParseReport,
    select: impl Fn(&provenance::OriginGroup) -> bool,
) -> usize {
    history(report)
        .evidence_mut()
        .groups_for_test()
        .iter()
        .position(select)
        .expect("a group of this kind")
}

#[test]
fn a_group_naming_a_missing_cause_has_no_header() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |_| true);
    history(&mut report).evidence_mut().groups_for_test()[index].cause = usize::MAX;
    let view = report.history_view();
    let group = view.group(index).unwrap();
    assert_eq!(
        view.header(group).err(),
        Some(HistoryReadError::Unresolved(fault(Kind::Cause)))
    );
}

#[test]
fn a_group_naming_a_missing_retirement_cause_has_no_header() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |_| true);
    history(&mut report).evidence_mut().groups_for_test()[index].retirement = Some(usize::MAX);
    let view = report.history_view();
    let group = view.group(index).unwrap();
    assert_eq!(
        view.header(group).err(),
        Some(HistoryReadError::Unresolved(fault(Kind::Cause)))
    );
}

#[test]
fn a_concat_naming_a_missing_part_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| matches!(g.body, GroupBody::Concat { .. }));
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Concat { original, .. } = &mut groups[index].body else {
        panic!("a Concat")
    };
    *original = usize::MAX;
    assert_eq!(walk(&report, index), Some(fault(Kind::Group)));
}

#[test]
fn a_concat_that_names_itself_outgrows_the_scratch_and_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| matches!(g.body, GroupBody::Concat { .. }));
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Concat {
        original, incoming, ..
    } = &mut groups[index].body
    else {
        panic!("a Concat")
    };
    (*original, *incoming) = (index, index);
    assert_eq!(walk(&report, index), Some(fault(Kind::Group)));
}

#[test]
fn a_leaf_whose_backing_is_not_a_string_row_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| {
        matches!(
            g.body,
            GroupBody::Leaf {
                value: LeafValue::Payload(_),
                ..
            }
        )
    });
    let other = frozen(&mut report)
        .payloads
        .values
        .iter()
        .position(|row| !matches!(row, RetainedPayload::String { .. }))
        .unwrap();
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Leaf { value, .. } = &mut groups[index].body else {
        panic!("a Leaf")
    };
    *value = LeafValue::Payload(other);
    assert_eq!(walk(&report, index), Some(fault(Kind::Fact)));
}

#[test]
fn a_leaf_whose_backing_is_past_the_registry_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| {
        matches!(
            g.body,
            GroupBody::Leaf {
                value: LeafValue::Payload(_),
                ..
            }
        )
    });
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Leaf { value, .. } = &mut groups[index].body else {
        panic!("a Leaf")
    };
    *value = LeafValue::Payload(usize::MAX);
    assert_eq!(walk(&report, index), Some(fault(Kind::Fact)));
}

#[test]
fn a_leaf_that_is_still_pending_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| {
        matches!(
            g.body,
            GroupBody::Leaf {
                value: LeafValue::Payload(_),
                ..
            }
        )
    });
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Leaf { value, .. } = &mut groups[index].body else {
        panic!("a Leaf")
    };
    *value = LeafValue::Pending;
    assert_eq!(walk(&report, index), Some(fault(Kind::Fact)));
}

#[test]
fn a_leaf_whose_origins_are_past_the_table_faults_the_cursor() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| {
        matches!(
            g.body,
            GroupBody::Leaf {
                value: LeafValue::Payload(_),
                ..
            }
        )
    });
    let groups = history(&mut report).evidence_mut().groups_for_test();
    let GroupBody::Leaf { start, .. } = &mut groups[index].body else {
        panic!("a Leaf")
    };
    *start = usize::MAX;
    assert_eq!(walk(&report, index), Some(fault(Kind::Group)));
}

#[test]
fn an_uncorrupted_history_walks_to_its_end() {
    let mut report = parse("<p>A&amp;B");
    let index = first_group(&mut report, |g| matches!(g.body, GroupBody::Concat { .. }));
    assert_eq!(walk(&report, index), None);
}

// ---- the association rows ----

fn first_sourced_row(report: &mut crate::HtmlParseReport) -> usize {
    history(report)
        .rows_for_test()
        .iter()
        .position(|row| row.source.is_some())
        .expect("a sourced row")
}
fn row_fault(report: &crate::HtmlParseReport, index: usize) -> Option<UnresolvedReference> {
    report.associations().row(index).err()
}

#[test]
fn a_row_naming_a_missing_cause_is_reported_when_it_is_lent() {
    let mut report = parse("<p>A&amp;B");
    let index = first_sourced_row(&mut report);
    history(&mut report).rows_for_test()[index].cause = usize::MAX;
    assert_eq!(row_fault(&report, index), Some(fault(Kind::Cause)));
}

#[test]
fn a_row_naming_a_missing_group_is_reported_when_it_is_lent() {
    let mut report = parse("<p>A&amp;B");
    let index = first_sourced_row(&mut report);
    history(&mut report).rows_for_test()[index].group = Some(usize::MAX);
    assert_eq!(row_fault(&report, index), Some(fault(Kind::Group)));
}

#[test]
fn a_row_with_an_unresolvable_destination_is_reported_when_it_is_lent() {
    use crate::construction::StringLocation as Location;
    let cases: [(&str, NativeDestination, Kind); 9] = [
        ("node", NativeDestination::Node(usize::MAX), Kind::Node),
        (
            "raw",
            NativeDestination::Raw {
                node: usize::MAX,
                kind: provenance::RawFieldKind::Input,
            },
            Kind::Node,
        ),
        (
            "text field",
            NativeDestination::Field(Location::Text(usize::MAX)),
            Kind::Node,
        ),
        (
            "attribute field",
            NativeDestination::Field(Location::AttributeName(usize::MAX, 0)),
            Kind::Fact,
        ),
        (
            "detached field",
            NativeDestination::Field(Location::Detached),
            Kind::Fact,
        ),
        (
            "lexical occurrence",
            NativeDestination::Lexical {
                occurrence: usize::MAX,
                part: None,
            },
            Kind::LexicalOccurrence,
        ),
        (
            "lexical part",
            NativeDestination::Lexical {
                occurrence: 0,
                part: Some(usize::MAX),
            },
            Kind::LexicalPart,
        ),
        (
            "omission",
            NativeDestination::Omission(usize::MAX),
            Kind::Omission,
        ),
        ("cause", NativeDestination::Cause(usize::MAX), Kind::Cause),
    ];
    for (name, destination, kind) in cases {
        let mut report = parse("<p class=a>A&amp;B<!--c-->");
        let index = first_sourced_row(&mut report);
        history(&mut report).rows_for_test()[index].destination = destination;
        assert_eq!(row_fault(&report, index), Some(fault(kind)), "{name}");
    }
}

#[test]
fn an_uncorrupted_row_is_lent_and_a_row_past_the_end_is_absent() {
    let report = parse("<p class=a>A&amp;B<!--c-->");
    let rows = report.associations();
    for index in 0..rows.len() {
        assert!(matches!(rows.row(index), Ok(Some(_))), "row {index}");
    }
    assert!(matches!(rows.row(rows.len()), Ok(None)));
}

#[test]
fn a_reverse_query_reaching_an_unresolvable_row_faults_and_stays_faulted() {
    let mut report = parse("<p>A&amp;B");
    let index = first_sourced_row(&mut report);
    let source = history(&mut report).rows_for_test()[index]
        .source
        .clone()
        .unwrap();
    history(&mut report).rows_for_test()[index].cause = usize::MAX;
    let rows = report.associations();
    let mut cursor = rows.reverse(&source, None).unwrap();
    let mut found = None;
    for _ in 0..=rows.len() {
        match cursor.advance() {
            HtmlParseReverseStep::Fault(first) => {
                assert!(
                    matches!(cursor.advance(), HtmlParseReverseStep::Fault(again) if again == first)
                );
                found = Some(first);
                break;
            }
            HtmlParseReverseStep::Complete => break,
            _ => {}
        }
    }
    assert_eq!(found, Some(fault(Kind::Cause)));
}

// ---- the cause and what it links ----

/// The cause of the first sourced row, corrupted by `edit`, and the row.
fn corrupted_cause(
    report: &mut crate::HtmlParseReport,
    edit: impl FnOnce(&mut provenance::ConstructionCause),
) -> usize {
    let index = first_sourced_row(report);
    let cause = history(report).rows_for_test()[index].cause;
    edit(&mut history(report).evidence_mut().causes_for_test()[cause]);
    index
}

#[test]
fn a_cause_naming_a_missing_cause_reports_it_from_the_link_readers() {
    for field in ["original_cause", "trigger"] {
        let mut report = parse("<p>A&amp;B");
        let index = corrupted_cause(&mut report, |cause| match field {
            "original_cause" => cause.original_cause = Some(usize::MAX),
            _ => cause.trigger = Some(usize::MAX),
        });
        let cause = report.associations().row(index).unwrap().unwrap().cause();
        let result = match field {
            "original_cause" => cause.original_cause(),
            _ => cause.trigger(),
        };
        assert_eq!(result.err(), Some(fault(Kind::Cause)), "{field}");
    }
}

#[test]
fn a_cause_naming_a_missing_node_reports_it_from_the_node_readers() {
    for field in ["original_node", "intended_parent"] {
        let mut report = parse("<p>A&amp;B");
        let index = corrupted_cause(&mut report, |cause| match field {
            "original_node" => cause.original_node = Some(usize::MAX),
            _ => cause.intended_parent = Some(usize::MAX),
        });
        let cause = report.associations().row(index).unwrap().unwrap().cause();
        let result = match field {
            "original_node" => cause.original_node(),
            _ => cause.intended_parent(),
        };
        assert_eq!(result.err(), Some(fault(Kind::Node)), "{field}");
    }
}

#[test]
fn a_cause_naming_a_missing_group_reports_it_from_the_group_readers() {
    for field in ["original", "incoming"] {
        let mut report = parse("<p>A&amp;B");
        let index = corrupted_cause(&mut report, |cause| match field {
            "original" => cause.original = Some(usize::MAX),
            _ => cause.incoming = Some(usize::MAX),
        });
        let cause = report.associations().row(index).unwrap().unwrap().cause();
        let result = match field {
            "original" => cause.original_group(),
            _ => cause.incoming_group(),
        };
        assert_eq!(result.err(), Some(fault(Kind::Group)), "{field}");
    }
}

#[test]
fn a_cause_naming_an_unresolvable_retired_destination_reports_it() {
    use crate::construction::StringLocation as Location;
    let cases = [
        ("node", Location::Text(usize::MAX), Kind::Node),
        (
            "attribute",
            Location::AttributeValue(usize::MAX, 0),
            Kind::Fact,
        ),
    ];
    for (name, location, kind) in cases {
        let mut report = parse("<p>A&amp;B");
        let index = corrupted_cause(&mut report, |cause| {
            cause.retired_destination = Some(location);
        });
        let cause = report.associations().row(index).unwrap().unwrap().cause();
        assert_eq!(
            cause.retired_destination().err(),
            Some(fault(kind)),
            "{name}"
        );
    }
}

#[test]
fn a_detached_retired_destination_is_an_answer_not_a_fault() {
    use crate::construction::StringLocation as Location;
    let mut report = parse("<p>A&amp;B");
    let index = corrupted_cause(&mut report, |cause| {
        cause.retired_destination = Some(Location::Detached);
    });
    let cause = report.associations().row(index).unwrap().unwrap().cause();
    assert!(matches!(
        cause.retired_destination(),
        Ok(Some(crate::HtmlParseRetiredDestination::Detached))
    ));
}

#[test]
fn a_lexical_occurrence_naming_a_missing_parent_or_cause_reports_it() {
    for field in ["parent", "cause"] {
        let mut report = parse("<p class=a>A<!--c-->");
        let evidence = history(&mut report).evidence_mut();
        let occurrences = evidence.lexical_occurrences_for_test();
        match field {
            "parent" => occurrences[0].parent = Some(usize::MAX),
            _ => occurrences[0].corrupt_cause_for_test(),
        }
        let rows = history(&mut report).rows_for_test();
        let (index, part) = rows
            .iter()
            .enumerate()
            .find_map(|(i, row)| match row.destination {
                NativeDestination::Lexical {
                    occurrence: 0,
                    part,
                } => Some((i, part)),
                _ => None,
            })
            .expect("a row that names the first occurrence");
        let _ = part;
        let row = report.associations().row(index).unwrap().unwrap();
        let crate::HtmlParseDestination::Lexical { occurrence, .. } = row.destination() else {
            panic!("a lexical destination")
        };
        let result = match field {
            "parent" => occurrence.parent().err(),
            _ => occurrence.cause().err(),
        };
        let kind = if field == "parent" {
            Kind::LexicalOccurrence
        } else {
            Kind::Cause
        };
        assert_eq!(result, Some(fault(kind)), "{field}");
    }
}

#[test]
fn a_lexical_part_naming_a_missing_occurrence_reports_it() {
    let mut report = parse("<p class=a>A<!--c-->");
    let parts = history(&mut report).evidence_mut().lexical_parts_for_test();
    parts[0].occurrence = usize::MAX;
    let rows = history(&mut report).rows_for_test();
    let index = rows
        .iter()
        .position(|row| {
            matches!(
                row.destination,
                NativeDestination::Lexical { part: Some(0), .. }
            )
        })
        .expect("a row that names the first part");
    let row = report.associations().row(index).unwrap().unwrap();
    let crate::HtmlParseDestination::Lexical { part, .. } = row.destination() else {
        panic!("a lexical destination")
    };
    assert_eq!(
        part.unwrap().occurrence().err(),
        Some(fault(Kind::LexicalOccurrence))
    );
}

fn computation_report() -> crate::HtmlParseReport {
    parse("<input type=range value=50 min=0 step=20><input type=color value=red>")
}
/// The group whose cause carries a computation of the selected kind.
fn computation_group(
    report: &mut crate::HtmlParseReport,
    select: impl Fn(provenance::InputValueComputation) -> bool,
) -> (usize, usize) {
    let causes = history(report).evidence_mut().causes_for_test();
    let cause = causes
        .iter()
        .position(|cause| cause.computation.is_some_and(&select))
        .expect("a cause with this computation");
    let group = first_group(report, |g| g.cause == cause);
    (cause, group)
}

#[test]
fn a_range_computation_naming_a_missing_original_or_operand_reports_it() {
    let mut report = computation_report();
    let (cause, group) = computation_group(
        &mut report,
        |c| matches!(c, provenance::InputValueComputation::Range(r) if matches!(r.original, provenance::RangeOriginal::Group(_))),
    );
    let edge = provenance::RangeEdge {
        group: usize::MAX,
        cause: usize::MAX,
    };
    let Some(provenance::InputValueComputation::Range(range)) =
        &mut history(&mut report).evidence_mut().causes_for_test()[cause].computation
    else {
        panic!("a Range computation")
    };
    range.original = provenance::RangeOriginal::Group(edge);
    *range
        .operands
        .get_mut(provenance::RangeOperandRole::Minimum) = provenance::RangeOperand::Finite(edge);
    let view = report.history_view();
    let header = view.header(view.group(group).unwrap()).unwrap();
    let computation = header.cause.range_computation().unwrap();
    assert_eq!(computation.original_group().err(), Some(fault(Kind::Group)));
    assert_eq!(computation.original_cause().err(), Some(fault(Kind::Cause)));
    let operand = computation.operand(provenance::RangeOperandRole::Minimum);
    assert_eq!(operand.group().err(), Some(fault(Kind::Group)));
    assert_eq!(operand.cause().err(), Some(fault(Kind::Cause)));
}

#[test]
fn a_color_computation_naming_a_missing_group_reports_it() {
    let mut report = computation_report();
    let (cause, group) = computation_group(&mut report, |c| {
        matches!(c, provenance::InputValueComputation::Color(_))
    });
    let edge = provenance::RangeEdge {
        group: usize::MAX,
        cause: usize::MAX,
    };
    let Some(provenance::InputValueComputation::Color(color)) =
        &mut history(&mut report).evidence_mut().causes_for_test()[cause].computation
    else {
        panic!("a Color computation")
    };
    color.original = Some(edge);
    *color.operands.get_mut(provenance::ColorOperandRole::Alpha) = Some(edge);
    let view = report.history_view();
    let header = view.header(view.group(group).unwrap()).unwrap();
    let computation = header.cause.color_computation().unwrap();
    assert_eq!(computation.original_group().err(), Some(fault(Kind::Group)));
    assert_eq!(
        computation
            .operand_group(provenance::ColorOperandRole::Alpha)
            .err(),
        Some(fault(Kind::Group))
    );
}

// ---- the metadata records ----

/// The fault a metadata cursor reports, requiring it to stay faulted, and the
/// number of records lent before it.
fn metadata_fault(report: &crate::HtmlParseReport) -> Option<UnresolvedReference> {
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            MetadataStep::Fault(first) => {
                assert!(matches!(cursor.advance(), MetadataStep::Fault(again) if again == first));
                return Some(first);
            }
            MetadataStep::Complete => return None,
            MetadataStep::Record(_) | MetadataStep::Skipped => {}
        }
    }
}
/// Edits the first payload row `edit` accepts, and requires one to exist.
fn corrupt_row(report: &mut crate::HtmlParseReport, edit: impl Fn(&mut RetainedPayload) -> bool) {
    assert!(
        frozen(report).payloads.values.iter_mut().any(edit),
        "a row of this kind"
    );
}
fn missing_node() -> Option<usize> {
    Some(usize::MAX)
}

#[test]
fn an_uncorrupted_report_issues_every_metadata_record() {
    let report = parse("<!doctype html><p class=a>A<input value=x><textarea>t</textarea>");
    assert_eq!(metadata_fault(&report), None);
}

#[test]
fn a_metadata_record_naming_a_missing_node_faults_the_cursor() {
    let mut report = parse("<p>A<input value=x>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Input { node, .. } => {
            *node = missing_node();
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

#[test]
fn a_metadata_record_naming_a_raw_value_that_is_not_a_string_faults_the_cursor() {
    let mut report = parse("<p>A<input value=x>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Input { raw, .. } => {
            *raw = usize::MAX;
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_textarea_naming_a_raw_value_that_is_not_a_string_faults_the_cursor() {
    let mut report = parse("<textarea>t</textarea>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Textarea { state, .. } => {
            state.raw = usize::MAX;
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_node_document_naming_a_missing_node_faults_the_cursor() {
    let mut report = parse("<p>A");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::NodeDocument { document, .. } => {
            *document = DocumentRef::Owned(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

#[test]
fn a_template_naming_a_missing_node_faults_the_cursor() {
    let mut report = parse("<template><p>A</template>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Template { state, .. } => {
            state.contents = usize::MAX;
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

#[test]
fn a_form_association_naming_a_missing_owner_faults_the_cursor() {
    let mut report = parse("<form><input></form>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::FormAssociation { state, .. } if state.owner.is_some() => {
            state.owner = missing_node();
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

#[test]
fn a_hyperlink_naming_a_missing_document_or_value_faults_the_cursor() {
    let mut report = parse("<a href=x>y</a>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Hyperlink { state, .. } => {
            state.document = DocumentRef::Owned(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
    let mut report = parse("<a href=x>y</a>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Hyperlink { state, .. } if state.raw.is_some() => {
            state.raw = Some(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_style_naming_a_missing_parser_document_faults_the_cursor() {
    let mut report = parse("<style>b{}</style>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Style { state, .. } => {
            state.parser_document = Some(DocumentRef::Owned(usize::MAX));
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

#[test]
fn a_style_naming_a_missing_cause_faults_the_cursor() {
    let mut report = parse("<style>b{}</style>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Style { state, .. } => {
            let Some(cause) = state.cause.as_mut() else {
                return false;
            };
            cause.corrupt_for_test();
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Cause)));
}

#[test]
fn a_dialog_naming_a_value_that_is_not_a_string_faults_the_cursor() {
    let mut report = parse("<dialog>a</dialog>");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Dialog { state, .. } => {
            state.value = usize::MAX;
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_document_naming_a_policy_row_that_is_not_one_faults_the_cursor() {
    let mut report = parse("<p>A");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Document { meta, .. } => {
            meta.csp = Some(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_document_naming_a_language_row_that_is_not_a_string_faults_the_cursor() {
    let mut report = parse("<p>A");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Document { meta, .. } => {
            meta.language = Some(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_document_naming_a_missing_gap_row_faults_the_cursor() {
    let mut report = parse("<p>A");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Document { meta, .. } => {
            *meta.gaps.get_mut(meta::MetaDomain::Refresh) = Some(usize::MAX);
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_document_whose_first_gap_has_no_row_faults_the_cursor() {
    let mut report = parse("<p>A");
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::Document { meta, .. } => {
            meta.first = Some(meta::MetaDomain::Csp);
            *meta.gaps.get_mut(meta::MetaDomain::Csp) = None;
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Fact)));
}

#[test]
fn a_base_naming_a_missing_document_faults_the_cursor() {
    let mut report = super::base::tests::request("<!doctype html><base href=A><p>A")
        .parse()
        .unwrap();
    corrupt_row(&mut report, |row| match row {
        RetainedPayload::BaseDocument { first, .. } if first.is_some() => {
            *first = missing_node();
            true
        }
        _ => false,
    });
    assert_eq!(metadata_fault(&report), Some(fault(Kind::Node)));
}

// ---- fragment effects, diagnostics and the parser Document ----

fn fragment_effects_report() -> crate::HtmlParseReport {
    super::fragment_document::tests::fragment_report("<template><p>a</p></template>")
}

#[test]
fn an_effect_naming_a_missing_inert_document_is_reported() {
    let mut report = fragment_effects_report();
    let fragment = frozen(&mut report).fragment.as_mut().unwrap();
    let effects = fragment.effects.as_mut_slice_for_test();
    let context::ContextEffect::TemplateCache { inert, .. } = &mut effects[0] else {
        panic!("a template cache")
    };
    *inert = Some(usize::MAX);
    let effects = report.effects().unwrap().unwrap();
    assert_eq!(effects.effect(0).err(), Some(fault(Kind::Node)));
}

#[test]
fn an_effect_whose_cache_was_never_bound_is_reported() {
    let mut report = fragment_effects_report();
    let fragment = frozen(&mut report).fragment.as_mut().unwrap();
    let effects = fragment.effects.as_mut_slice_for_test();
    let context::ContextEffect::TemplateCache { inert, .. } = &mut effects[0] else {
        panic!("a template cache")
    };
    *inert = None;
    let effects = report.effects().unwrap().unwrap();
    assert_eq!(effects.effect(0).err(), Some(fault(Kind::Fact)));
}

#[test]
fn an_effect_past_the_last_is_absent_and_a_bound_effect_is_lent() {
    let report = fragment_effects_report();
    let effects = report.effects().unwrap().unwrap();
    assert!(matches!(effects.effect(0), Ok(Some(_))));
    assert!(matches!(effects.effect(effects.len()), Ok(None)));
}

#[test]
fn a_registry_binding_naming_a_missing_node_is_reported() {
    let mut report = fragment_effects_report();
    let slot = frozen(&mut report)
        .payloads
        .values
        .iter()
        .position(|row| matches!(row, RetainedPayload::LookupRegistry { node: Some(_), .. }))
        .expect("a registry binding");
    let RetainedPayload::LookupRegistry { node, .. } =
        &mut frozen(&mut report).payloads.values[slot]
    else {
        panic!("a registry binding")
    };
    *node = missing_node();
    let effects = report.effects().unwrap().unwrap();
    assert_eq!(effects.registry(slot).err(), Some(fault(Kind::Node)));
}

#[test]
fn a_row_that_is_not_a_registry_binding_is_absent_not_a_fault() {
    let report = fragment_effects_report();
    let effects = report.effects().unwrap().unwrap();
    let slots = effects.registry_slots();
    assert!(matches!(effects.registry(slots), Ok(None)));
}

#[test]
fn a_diagnostic_naming_a_missing_location_is_reported() {
    let mut report = parse("<p>");
    assert!(!report.diagnostics().is_empty());
    assert!(
        history(&mut report)
            .evidence_mut()
            .corrupt_diagnostic_location_for_test(0)
    );
    assert_eq!(
        report.diagnostics().retained_record(0).err(),
        Some(fault(Kind::Diagnostic))
    );
}

#[test]
fn a_diagnostic_past_the_last_is_absent() {
    let report = parse("<p>");
    let diagnostics = report.diagnostics();
    assert!(matches!(
        diagnostics.retained_record(diagnostics.len()),
        Ok(None)
    ));
}

#[test]
fn a_contextual_report_without_its_completion_row_is_reported_not_skipped() {
    let mut report = super::fragment_document::tests::fragment_report("<p>a</p>");
    let storage = frozen(&mut report);
    let completion = storage.fragment.as_ref().unwrap().completion;
    storage.payloads.values[completion] = RetainedPayload::Retired;
    assert_eq!(
        report.fragment_parser_document().err(),
        Some(fault(Kind::Fact))
    );
}

#[test]
fn a_contextual_report_without_its_fragment_storage_is_reported_not_skipped() {
    let mut report = super::fragment_document::tests::fragment_report("<p>a</p>");
    frozen(&mut report).fragment = None;
    assert_eq!(
        report.fragment_parser_document().err(),
        Some(fault(Kind::Fragment))
    );
}

#[test]
fn an_ordinary_report_has_no_parser_document_and_that_is_not_a_fault() {
    let report = parse("<p>a</p>");
    assert!(matches!(report.fragment_parser_document(), Ok(None)));
}

// ---- how the fault describes itself ----

#[test]
fn an_unresolved_reference_names_what_it_denotes_in_words() {
    use std::error::Error;
    let cases = [
        (Kind::Node, "a node of its tree"),
        (Kind::Cause, "a construction cause"),
        (Kind::Group, "an origin group of its text history"),
        (Kind::LexicalOccurrence, "a lexical occurrence"),
        (Kind::LexicalPart, "a part of a lexical occurrence"),
        (Kind::Omission, "a lexical omission"),
        (Kind::Fact, "a fact it retains"),
        (Kind::CapturedNode, "a node of its captured context"),
        (Kind::Fragment, "its fragment context"),
        (Kind::Diagnostic, "the location of a diagnostic"),
    ];
    for (kind, words) in cases {
        let message = fault(kind).to_string();
        assert_eq!(
            message,
            format!("a published parse report refers to {words} that it does not hold")
        );
        assert!(fault(kind).source().is_none());
    }
}

#[test]
fn a_history_error_describes_itself_and_names_the_fault_as_its_source() {
    use std::error::Error;
    let unresolved = HistoryReadError::Unresolved(fault(Kind::Cause));
    assert_eq!(
        unresolved.to_string(),
        "the report's history holds a reference it cannot resolve"
    );
    assert_eq!(
        unresolved.source().map(ToString::to_string),
        Some(fault(Kind::Cause).to_string())
    );
    assert_eq!(
        HistoryReadError::ForeignOwner.to_string(),
        "the group belongs to another report"
    );
    assert!(HistoryReadError::ForeignOwner.source().is_none());
    assert_eq!(
        HistoryReadError::InsufficientScratch {
            required: 3,
            supplied: 2
        }
        .to_string(),
        "the history cursor needs 3 scratch frames and was given 2"
    );
}
