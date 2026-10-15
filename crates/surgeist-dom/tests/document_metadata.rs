use std::sync::atomic::AtomicBool;

use surgeist_dom::{Dom, DomBatch, DomError, DomLimits, DomResource};

// DOM's native Document defaults retain three strings: application/xml (15),
// UTF-8 (5) and about:blank (11). These are stored slots, not derived display.
const DEFAULT_METADATA_UNITS: usize = 31;

#[test]
fn native_document_charges_its_source_default_string_slots() {
    let mut dom = Dom::new(DomLimits::new(1, DEFAULT_METADATA_UNITS, 1, 0, 0).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    batch.create_document();
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.usage().string_code_units(), DEFAULT_METADATA_UNITS);
}

#[test]
fn insufficient_default_metadata_budget_rejects_atomic_creation() {
    let mut dom = Dom::new(DomLimits::new(1, 30, 1, 0, 0).unwrap()).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.create_document();
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 30
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().nodes().count(), 0);
    assert_eq!(dom.usage(), usage);
}
