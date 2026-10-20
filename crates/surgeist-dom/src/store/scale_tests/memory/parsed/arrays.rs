//! The simultaneously held HTML and DOM attribute arrays only. String targets,
//! element bindings, indexes, pool Weak targets, Arc headers and allocator data
//! are outside this inventory; scratch maps retain no semantic owners.

use std::collections::BTreeMap;
use std::mem::size_of_val;

use super::*;

#[derive(Default)]
struct Arrays {
    // Address keys are used only while every borrowed semantic holder is live.
    // Values describe one immutable target: length, capacity, target bytes.
    targets: BTreeMap<usize, (usize, usize, usize)>,
}

impl Arrays {
    fn record(&mut self, address: usize, length: usize, capacity: usize, body: usize) {
        assert!(length <= capacity);
        if let Some(previous) = self.targets.insert(address, (length, capacity, body)) {
            assert_eq!(previous, (length, capacity, body));
        }
    }

    fn report(&self, layer: &str, stage: &str, row_bytes: usize) {
        let target_bytes: usize = self.targets.values().map(|v| v.2).sum();
        let capacity_rows: usize = self.targets.values().map(|v| v.1).sum();
        let stored_rows: usize = self.targets.values().map(|v| v.0).sum();
        let buffers = self.targets.values().filter(|v| v.1 != 0).count();
        let empty_targets = self.targets.values().filter(|v| v.0 == 0).count();
        eprintln!(
            "memory_parsed_arrays stage={stage} layer={layer} unique_targets={} empty_targets={empty_targets} nonempty_capacity_buffers={buffers} stored_rows={stored_rows} capacity_rows={capacity_rows} target_bytes={target_bytes} capacity_bytes={} row_bytes={row_bytes} physical_total=false",
            self.targets.len(),
            capacity_rows.checked_mul(row_bytes).unwrap()
        );
    }
}

pub(super) fn report(
    stage: &str,
    parsed: Option<&HtmlParseReport>,
    report_ids: &[HtmlNodeId],
    snapshots: &[&DomSnapshot],
) {
    // Four row IDs and two attributes on each of the literal 32 cells.
    let expected_occurrences = ROWS + 2 * ROWS * COLUMNS;
    let mut html = Arrays::default();
    if let Some(parsed) = parsed {
        let document = parsed.document().unwrap();
        let mut occurrences = 0;
        for id in report_ids {
            if let HtmlNodeKind::Element(element) = document.node(id).unwrap().kind() {
                let bank = element.attributes().shared_attributes();
                occurrences += bank.len();
                html.record(
                    Arc::as_ptr(bank) as usize,
                    bank.len(),
                    bank.capacity(),
                    size_of_val(bank.as_ref()),
                );
            }
        }
        assert_eq!(occurrences, expected_occurrences);
    }
    let mut dom = Arrays::default();
    for snapshot in snapshots {
        let mut occurrences = 0;
        for id in snapshot.nodes() {
            if let DomNodeKind::Element(element) = snapshot.node(id).unwrap().kind() {
                occurrences += element.attributes().len();
                if let Some(bank) = element
                    .attributes_backing()
                    .and_then(crate::name::AttributeStorage::payload)
                {
                    dom.record(
                        Arc::as_ptr(bank) as usize,
                        bank.value.entries().len(),
                        bank.value.capacity(),
                        size_of_val(bank.as_ref()),
                    );
                }
            }
        }
        assert_eq!(occurrences, expected_occurrences);
    }
    // These are actual simultaneous array targets of different owned element
    // types. UTF-16 child sharing does not make the arrays the same allocation.
    assert!(html.targets.keys().all(|id| !dom.targets.contains_key(id)));
    eprintln!(
        "memory_parsed_arrays stage={stage} report_held={} snapshot_holders={} authored_attributes_per_document={expected_occurrences} inventory=strongly_reachable_attribute_arrays_only exclusions=UTF16_children,element_bindings,indexes,pool_Weak_targets,Arc_headers,allocator_metadata,instrumentation_storage",
        parsed.is_some(),
        snapshots.len()
    );
    html.report("html_report", stage, size_of::<surgeist_html::Attribute>());
    dom.report(
        "dom_snapshot_union",
        stage,
        size_of::<crate::DomAttribute>(),
    );
}
