//! What a plain element retains for its nonce, and the nonce a published report answers by node.
use super::*;

/// The registry rows of a parse of an ordinary Document of `elements` plain `span` elements, with
/// the detail the request records, or `None` when this build cannot record that detail.
fn registry_rows(elements: usize, detail: crate::DetailSet) -> Option<usize> {
    let source = format!("<!doctype html>{}", "<span></span>".repeat(elements));
    let request = crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source.as_str()),
        g2_d2::corpus_limits(),
    )
    .with_detail(detail)
    .ok()?;
    let report = request.parse().unwrap();
    Some(report.history().storage().payloads.values.len())
}

/// The registry rows one plain element holds: the string of its name and the string of its
/// namespace, and with the detail of causes recorded the cause of its creation. The element has
/// no nonce: its nonce is the empty nonce every element of its namespaces has, which no row holds.
/// The oracle is the families an element has independently of its nonce.
#[test]
fn a_plain_element_holds_no_registry_row_for_its_nonce() {
    for (detail, per_element) in [(crate::DetailSet::NONE, 2), (crate::DetailSet::ALL, 3)] {
        if !g2_d2::details().contains(&detail) {
            continue;
        }
        let Some(empty) = registry_rows(0, detail) else {
            continue;
        };
        for elements in [1, 16, 64] {
            let rows = registry_rows(elements, detail).unwrap();
            assert_eq!(
                rows - empty,
                elements * per_element,
                "{elements} plain elements, {detail:?}"
            );
        }
    }
}
