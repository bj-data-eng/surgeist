#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

fn fixture(ns: Namespace, value: Option<DocumentString>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(10, 10, 10, 10000, 10, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let e = b
        .add_element(
            &d,
            QualifiedName::new(ns, None, "arbitrary"),
            OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, "nonce"),
                "different content much longer than raw cap zero",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    if let Some(value) = value {
        b.establish_element_nonce(&e, value).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}

fn limits(work: usize, output: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(work, output, 10, 0, 100000, 10)
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::Nonce)
}
#[test]
fn getter_observes_exact_slot_without_content_or_policy_dependency() {
    for ns in [Namespace::html(), Namespace::svg(), Namespace::mathml()] {
        for units in [vec![], vec![0, 32, 0xd800, 0xdc00, 0xffff]] {
            let expected = DocumentString::from_utf16(units);
            let (c, id) = fixture(ns.clone(), Some(expected.clone()));
            let cx = context(&c);
            let mut s = HtmlSemanticSession::new(
                &c,
                &cx,
                HtmlValueInterpreter::new(0),
                limits(10000, 10),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
            let r = s.query(&[request(&id)]).unwrap();
            assert_eq!(
                r.results()[0].fact(),
                &HtmlSemanticFact::Nonce(HtmlSemanticValue::Known(expected))
            );
        }
    }
}
#[test]
fn missing_and_inapplicable_are_not_empty() {
    for ns in [Namespace::html(), Namespace::none()] {
        let (c, id) = fixture(ns.clone(), None);
        let cx = context(&c);
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(0),
            limits(10000, 10),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let r = s.query(&[request(&id)]).unwrap();
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::Nonce(if ns == Namespace::html() {
                HtmlSemanticValue::Missing(HtmlSemanticMissing::NonceSlot(id))
            } else {
                HtmlSemanticValue::Inapplicable
            })
        );
    }
}
#[test]
fn batch_refusal_cancellation_and_sticky_stop_retain_no_partial_results() {
    let (c, id) = fixture(Namespace::html(), Some("S".into()));
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        limits(10000, 1),
        cancel.clone(),
    )
    .unwrap();
    let first = s.query(&[request(&id), request(&id)]).unwrap_err();
    cancel.cancel();
    assert_eq!(s.query(&[request(&id)]).unwrap_err(), first);
    assert_eq!(s.usage().scratch_records(), 0);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        limits(10000, 10),
        cancel,
    )
    .unwrap();
    assert!(s.query(&[request(&id)]).is_err());
    assert_eq!(s.usage().scratch_records(), 0);
}
