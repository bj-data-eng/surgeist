use std::sync::Arc;
use surgeist_html::DocumentString;
use surgeist_html_semantics::{
    HtmlBaseFactsError, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}

#[test]
fn preformation_extent_matches_the_actual_retained_record() {
    // Storage measurement accepts an empty input without claiming that it is
    // the serialization of an actual URL record.
    assert_eq!(
        HtmlBaseUrlRecord::observed_storage_usage(&DocumentString::from_utf16(Vec::<u16>::new()))
            .unwrap()
            .utf16_extent(),
        0,
    );
    for raw in ["about:blank", "https://example.test/path"] {
        let serialization = DocumentString::from(raw);
        let before = HtmlBaseUrlRecord::observed_storage_usage(&serialization).unwrap();
        let record = HtmlBaseUrlRecord::observed(
            revision(),
            serialization,
            HtmlBaseUrlScheme::Other,
            raw == "about:blank",
        )
        .unwrap();
        assert_eq!(before, record.retained_storage_usage());
        assert_eq!(
            before.utf16_extent(),
            record.serialization().shared_code_units().capacity(),
        );
    }
}

#[test]
fn actual_spare_utf16_capacity_is_charged_without_retaining_or_replacing_the_input() {
    let mut small = "about:blank".encode_utf16().collect::<Vec<_>>();
    small.shrink_to_fit();
    let mut spare = Vec::with_capacity(4096);
    spare.extend_from_slice(&small);
    let small = DocumentString::from_utf16(small);
    let spare_backing = Arc::new(spare);
    let spare = DocumentString::from_shared_code_units(spare_backing.clone());
    let count = Arc::strong_count(&spare_backing);
    let small_usage = HtmlBaseUrlRecord::observed_storage_usage(&small).unwrap();
    let usage = HtmlBaseUrlRecord::observed_storage_usage(&spare).unwrap();
    assert_eq!(Arc::strong_count(&spare_backing), count);
    assert!(Arc::ptr_eq(spare.shared_code_units(), &spare_backing));
    assert_eq!(spare.code_units(), small.code_units());
    let extra = spare_backing.capacity() - small.shared_code_units().capacity();
    assert!(extra > 0);
    assert_eq!(usage.utf16_extent(), spare_backing.capacity());
    assert_eq!(usage.utf16_extent() - small_usage.utf16_extent(), extra);
    assert_eq!(
        usage.retained_bytes() - small_usage.retained_bytes(),
        extra * size_of::<u16>(),
    );
    let record =
        HtmlBaseUrlRecord::observed(revision(), spare, HtmlBaseUrlScheme::Other, true).unwrap();
    assert_eq!(record.retained_storage_usage(), usage);
    assert!(Arc::ptr_eq(
        record.serialization().shared_code_units(),
        &spare_backing
    ));
    assert_eq!(record, record.clone());
    let independent = HtmlBaseUrlRecord::observed(
        record.revision().clone(),
        record.serialization().clone(),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    assert_ne!(record, independent);
}

#[test]
fn measuring_storage_does_not_attest_url_truth_or_change_factory_validation() {
    let serialization = DocumentString::from("about:blank");
    let before = HtmlBaseUrlRecord::observed_storage_usage(&serialization).unwrap();
    for scheme in [HtmlBaseUrlScheme::Data, HtmlBaseUrlScheme::Javascript] {
        assert!(matches!(
            HtmlBaseUrlRecord::observed(revision(), serialization.clone(), scheme, true),
            Err(HtmlBaseFactsError::ContradictoryObservation),
        ));
    }
    assert_eq!(
        HtmlBaseUrlRecord::observed_storage_usage(&serialization).unwrap(),
        before,
    );
}
