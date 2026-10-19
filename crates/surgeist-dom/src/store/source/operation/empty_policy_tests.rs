use super::*;

pub(in crate::store) fn style_fixture() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            16,
            100_000,
            32,
            0,
            0,
            8,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, crate::Namespace::html(), "style");
    let text = batch.create_text(&document, "p {}");
    batch.insert_before(&text, &style, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&style).unwrap().unwrap(),
    )
}
pub(in crate::store) fn begin_style(
    dom: &mut Dom,
    document: DocumentId,
    style: NodeId,
) -> (DomSourceLease<'_>, DomStyleRequest) {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(
            crate::DomSourceEdit::insert_before(style, document.node(), None),
            DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 16, 64, 64, 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual connected native Style CSP request")
    };
    assert_eq!(
        request.action(),
        DomStyleAction::Update(Action::CheckInlineCsp)
    );
    assert_eq!(request.source().element(), style);
    assert_eq!(
        request.source().text().as_slice(),
        &[0x70, 0x20, 0x7b, 0x7d]
    );
    (lease, request)
}

fn raw_string(value: &DocumentString) -> (usize, usize) {
    let capacity = value.shared_code_units().capacity();
    (
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    )
}
fn raw_source(source: &DomStyleSource) -> (usize, usize) {
    // This fixture has native, unhosted Document metadata and no producer report.
    // Count each retained field occurrence, even when string storage aliases.
    let metadata = source.document_metadata();
    assert!(matches!(
        metadata.creation_cause(),
        crate::DomDocumentCreationCause::Native
    ));
    assert!(metadata.document_environment().hosted.is_none());
    assert!(metadata.hosted_inputs.is_none());
    assert!(
        metadata
            .meta_color_scheme
            .as_ref()
            .is_none_or(Option::is_none)
    );
    assert!(
        metadata
            .meta_state()
            .and_then(|m| m.pragma_language())
            .is_none()
    );
    let mut bytes = size_of::<Vec<u16>>() + source.text().capacity() * size_of::<u16>();
    let mut units = source.text().capacity();
    for string in [
        Some(source.base_url()),
        source.media(),
        source.title(),
        source.blocking(),
        Some(source.nonce()),
        Some(metadata.content_type()),
        Some(metadata.encoding()),
        Some(metadata.url()),
    ]
    .into_iter()
    .flatten()
    {
        let (b, u) = raw_string(string);
        bytes += b;
        units += u;
    }
    (bytes, units)
}
fn assert_gate(
    lease: &mut DomSourceLease<'_>,
    request: &DomStyleRequest,
    observation: &DomStyleInlineCspObservation,
    present: bool,
) {
    let cancel = AtomicBool::new(false);
    let (mut expected_bytes, mut expected_units) = raw_source(request.source());
    if present {
        expected_bytes += size_of::<Vec<DocumentString>>()
            + observation.policies.capacity() * size_of::<DocumentString>();
    }
    for policy in observation.policies() {
        let (b, u) = raw_string(policy);
        expected_bytes += b;
        expected_units += u;
    }
    let op = lease.dom.source_operation.as_mut().unwrap();
    op.refresh_admission(&cancel).unwrap();
    let original = op.limits;
    let base = op.draft.source_storage_bytes(&cancel).unwrap() + op.draft.source_other_bytes;
    let boundary = base + expected_bytes;
    let stamp = lease.snapshot().stamp();
    for short in [true, false] {
        let limit = boundary - usize::from(short);
        let op = lease.dom.source_operation.as_mut().unwrap();
        op.limits.retained_bytes = limit;
        op.draft.source_limits.as_mut().unwrap().retained_bytes = limit;
        let result = lease.accept_inline_csp(request, observation, &cancel);
        if short {
            assert!(matches!(result, Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes, limit: actual
            }) if actual == limit));
            assert!(
                lease
                    .dom
                    .source_operation
                    .as_ref()
                    .unwrap()
                    .receipt
                    .is_none()
            );
            assert_eq!(lease.snapshot().stamp(), stamp);
        } else {
            assert!(
                result.is_ok(),
                "actual receipt must fit independently priced B={boundary}, present={present}: {result:?}"
            );
            let Some(Receipt::Inline {
                bytes,
                units,
                observation: stored,
            }) = &lease.dom.source_operation.as_ref().unwrap().receipt
            else {
                panic!("accepted original Inline receipt")
            };
            assert_eq!((*bytes, *units), (expected_bytes, expected_units));
            assert_eq!(stored.policies(), observation.policies());
            assert_eq!(stored.policy_revision(), observation.policy_revision());
            assert_eq!(stored.allowed(), observation.allowed());
        }
    }
    let op = lease.dom.source_operation.as_mut().unwrap();
    op.limits = original;
    op.draft.source_limits = Some(original);
}

#[test]
fn source_complete_empty_policy_receipt_has_no_policy_bank_extent() {
    let (mut dom, document, style) = style_fixture();
    let (mut lease, request) = begin_style(&mut dom, document, style);
    let observation = DomStyleInlineCspObservation::complete_empty(request.source(), 11);
    assert_gate(&mut lease, &request, &observation, false);
}

#[test]
fn source_evaluated_policy_receipts_price_the_actual_supplied_banks() {
    for case in 0..3 {
        let (mut dom, document, style) = style_fixture();
        let (mut lease, request) = begin_style(&mut dom, document, style);
        let mut policies = if case == 0 {
            Vec::new()
        } else {
            Vec::with_capacity(7)
        };
        if case == 2 {
            let mut value = Vec::with_capacity(19);
            value.extend([0xd800, 0, 0xdc00]);
            policies.extend([
                DocumentString::from("style-src 'none'"),
                DocumentString::from_utf16(value),
            ]);
        }
        let observation = DomStyleInlineCspObservation::evaluated(
            request.source(),
            Arc::new(policies),
            17 + case,
            case != 2,
        );
        assert_gate(&mut lease, &request, &observation, true);
    }
}
