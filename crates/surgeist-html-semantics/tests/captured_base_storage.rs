//! Owner storage receipts include unused inputs and capacity, independently of queries.
use surgeist_html::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlBaseFactsLimits {
    HtmlBaseFactsLimits::new(1000, 10000, 100000)
}
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn padded(value: &str, spare: usize) -> DocumentString {
    let units: Vec<_> = value.encode_utf16().collect();
    let mut backing = Vec::with_capacity(units.len() + spare);
    backing.extend(units);
    DocumentString::from_utf16(backing)
}
fn url(revision: &HtmlBaseFactsRevision, spare: usize) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        padded("https://example.test/", spare),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap()
}
fn capture() -> (
    HtmlCapture,
    Vec<HtmlCaptureDocumentId>,
    Vec<HtmlCaptureSubjectId>,
) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10),
        HtmlHostFactsRevision::fresh(),
    );
    let mut documents = Vec::new();
    let mut bases = Vec::new();
    for _ in 0..2 {
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let base = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "base"),
                OrderedAttributes::new(vec![Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "href"),
                    "x",
                )])
                .unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_children(&base, &[]).unwrap();
        builder
            .establish_children(&document, std::slice::from_ref(&base))
            .unwrap();
        documents.push(document);
        bases.push(base);
    }
    let capture = builder.finish().unwrap();
    let docs = documents
        .iter()
        .map(|id| {
            capture
                .document_id(&capture.subject_id(id).unwrap())
                .unwrap()
        })
        .collect();
    let bases = bases
        .iter()
        .map(|id| capture.subject_id(id).unwrap())
        .collect();
    (capture, docs, bases)
}
fn bank(
    capture: &HtmlCapture,
    revision: &HtmlBaseFactsRevision,
    documents: Vec<HtmlCapturedBaseDocument>,
    frozen: Vec<HtmlCapturedFrozenBase>,
) -> HtmlCapturedBaseFacts {
    HtmlCapturedBaseFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        documents,
        frozen,
        limits(),
    )
    .unwrap()
}

#[test]
fn empty_and_spare_row_banks_count_outer_payload_and_actual_capacities() {
    let (capture, _, _) = capture();
    let revision = revision();
    let empty = bank(&capture, &revision, vec![], vec![]);
    assert_eq!(
        empty.retained_storage_usage().retained_bytes(),
        size_of::<HtmlCapturedBaseFacts>()
    );
    assert_eq!(empty.retained_storage_usage().utf16_extent(), 0);
    let documents = Vec::with_capacity(7);
    let frozen = Vec::with_capacity(11);
    let expected = size_of::<HtmlCapturedBaseFacts>()
        + documents.capacity() * size_of::<HtmlCapturedBaseDocument>()
        + frozen.capacity() * size_of::<HtmlCapturedFrozenBase>();
    let spare = bank(&capture, &revision, documents, frozen);
    assert_eq!(spare.retained_storage_usage().retained_bytes(), expected);
    assert_eq!(spare.retained_storage_usage().utf16_extent(), 0);
    assert_eq!(spare.formation_usage().utf16_extent(), 0);
}

#[test]
fn every_unused_nested_observation_and_repeated_owner_occurrence_is_retained() {
    let (capture, docs, _) = capture();
    let revision = revision();
    let fallback = url(&revision, 13);
    let result = url(&revision, 19);
    let encoding = HtmlBaseEncoding::fresh();
    let mut observations = Vec::with_capacity(9);
    observations.push(
        HtmlBaseParseObservation::new(
            padded("unselected", 31),
            fallback.clone(),
            encoding.clone(),
            revision.clone(),
            HtmlBaseParseOutcome::Parsed {
                record: result,
                base_allowed: HtmlBaseAllowed::Allowed,
            },
        )
        .unwrap(),
    );
    observations.push(
        HtmlBaseParseObservation::new(
            padded("https://[", 37),
            fallback.clone(),
            encoding,
            revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap(),
    );
    let nested = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(fallback),
            HtmlAboutBaseUrl::KnownNull,
            false,
            None,
        )
        .unwrap(),
        observations,
        limits(),
    )
    .unwrap();
    let receipt = nested.retained_storage_usage();
    let mut rows = Vec::with_capacity(5);
    rows.push(HtmlCapturedBaseDocument::new(
        docs[0].clone(),
        nested.clone(),
    ));
    rows.push(HtmlCapturedBaseDocument::new(docs[1].clone(), nested));
    let expected = size_of::<HtmlCapturedBaseFacts>()
        + rows.capacity() * size_of::<HtmlCapturedBaseDocument>()
        + 2 * receipt.retained_bytes();
    let outer = bank(&capture, &revision, rows, vec![]);
    drop(capture);
    assert_eq!(outer.retained_storage_usage().retained_bytes(), expected);
    assert_eq!(
        outer.retained_storage_usage().utf16_extent(),
        2 * receipt.utf16_extent()
    );
    // Nested units never changed the outer bank's existing logical tariff.
    assert_eq!(outer.formation_usage().utf16_extent(), 0);
}

#[test]
fn frozen_href_and_url_spare_backing_are_counted_per_occurrence() {
    let (capture, docs, bases) = capture();
    let revision = revision();
    let record = url(&revision, 41);
    let href = padded("x", 43);
    let extent = href.shared_code_units().capacity();
    let nested = record.retained_storage_usage();
    let rows = vec![
        HtmlCapturedFrozenBase::new(
            bases[0].clone(),
            docs[0].clone(),
            href.clone(),
            Some(record.clone()),
            revision.clone(),
        ),
        HtmlCapturedFrozenBase::new(
            bases[1].clone(),
            docs[1].clone(),
            href,
            Some(record),
            revision.clone(),
        ),
    ];
    let expected = size_of::<HtmlCapturedBaseFacts>()
        + rows.capacity() * size_of::<HtmlCapturedFrozenBase>()
        + 2 * (size_of::<Vec<u16>>() + extent * size_of::<u16>() + nested.retained_bytes());
    let outer = bank(&capture, &revision, vec![], rows);
    assert_eq!(outer.retained_storage_usage().retained_bytes(), expected);
    assert_eq!(
        outer.retained_storage_usage().utf16_extent(),
        2 * (extent + nested.utf16_extent())
    );
    assert_eq!(
        outer.formation_usage().utf16_extent(),
        2 * (1 + "https://example.test/".len())
    );
}

#[test]
fn null_frozen_url_counts_href_and_preserves_exact_existing_formation_limits() {
    let (capture, docs, bases) = capture();
    let revision = revision();
    let make = |spare, limits| {
        let href = padded("x", spare);
        HtmlCapturedBaseFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            revision.clone(),
            vec![],
            vec![HtmlCapturedFrozenBase::new(
                bases[0].clone(),
                docs[0].clone(),
                href,
                None,
                revision.clone(),
            )],
            limits,
        )
    };
    let small = make(0, limits()).unwrap();
    let large = make(53, limits()).unwrap();
    assert_eq!(small.formation_usage(), large.formation_usage());
    assert_eq!(
        large.retained_storage_usage().utf16_extent()
            - small.retained_storage_usage().utf16_extent(),
        53
    );
    assert_eq!(
        large.retained_storage_usage().retained_bytes()
            - small.retained_storage_usage().retained_bytes(),
        53 * size_of::<u16>()
    );
    let usage = small.formation_usage();
    let exact = HtmlBaseFactsLimits::new(
        usage.record_capacity(),
        usage.utf16_extent(),
        usage.validation_work(),
    );
    assert!(make(53, exact).is_ok());
    assert_eq!(
        make(
            53,
            HtmlBaseFactsLimits::new(
                usage.record_capacity(),
                usage.utf16_extent(),
                usage.validation_work() - 1
            )
        )
        .unwrap_err(),
        HtmlBaseFactsError::WorkLimit
    );
    assert_eq!(
        make(
            53,
            HtmlBaseFactsLimits::new(
                usage.record_capacity(),
                usage.utf16_extent() - 1,
                usage.validation_work()
            )
        )
        .unwrap_err(),
        HtmlBaseFactsError::Utf16Limit
    );
    assert_eq!(
        make(
            53,
            HtmlBaseFactsLimits::new(
                usage.record_capacity() - 1,
                usage.utf16_extent(),
                usage.validation_work()
            )
        )
        .unwrap_err(),
        HtmlBaseFactsError::RecordLimit
    );
}
