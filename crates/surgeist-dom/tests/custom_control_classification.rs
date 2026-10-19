//! HTML877d54588–54591 requires an associated form-associated definition.
//! A native undefined custom name with an actual null definition is not listed.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomElementDefinition, DomHtmlCaptureLimits, DomLimits, Namespace,
};
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

#[test]
fn null_definition_custom_element_is_excluded_from_actual_fieldset_controls() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(8, 2048, 16, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fieldset = batch.create_element_ns(&document, Namespace::html(), "fieldset");
    let custom = batch.create_element_ns(&document, Namespace::html(), "x-widget");
    batch.insert_before(&custom, &fieldset, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let fieldset = commit.created(&fieldset).unwrap().unwrap();
    let custom = commit.created(&custom).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot
            .node(custom)
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_definition(),
        DomElementDefinition::Null
    );
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(16, 4096, 16, 4096, 16, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(16, 100000),
            &cancel,
        )
        .unwrap();
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(fieldset).unwrap(),
                HtmlSemanticQuery::FieldsetControls,
            )],
            HtmlValueInterpreter::new(4096),
            HtmlSemanticLimits::new(100000, 4, 64, 32, 100000, 32),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::FieldsetControls(HtmlSemanticValue::Known(controls)) =
        report.results()[0].fact()
    else {
        panic!(
            "actual null definition establishes a known nonparticipating custom element: {:?}",
            report.results()[0].fact()
        );
    };
    assert!(controls.controls().is_empty());
}
