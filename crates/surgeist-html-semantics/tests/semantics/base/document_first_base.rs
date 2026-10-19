//! An established serialized fallback is selected by the same first-Base rule.
use surgeist_html::DocumentString;
use surgeist_html_semantics::*;
#[test]
fn serialization_selection_preserves_missing_first_and_historical_frozen_record() {
    let fallback: DocumentString = "about:blank".into();
    assert!(std::ptr::eq(
        select_document_base_serialization(&fallback, HtmlDocumentFirstBase::Absent).unwrap(),
        &fallback
    ));
    assert!(matches!(
        select_document_base_serialization(&fallback, HtmlDocumentFirstBase::Present(None)),
        Err(HtmlBaseMissing::FrozenBase)
    ));
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 3, 7, 11);
    let historical = HtmlBaseUrlRecord::observed(
        revision,
        "https://historical.example/base/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let first = HtmlDocumentFirstBase::Present(Some(&historical));
    assert!(std::ptr::eq(
        select_document_base(None, first).unwrap(),
        &historical
    ));
    assert!(std::ptr::eq(
        select_document_base_serialization(&fallback, first).unwrap(),
        historical.serialization()
    ));
}
