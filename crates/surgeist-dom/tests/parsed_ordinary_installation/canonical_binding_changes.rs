//! A completed contextual report remains historical when canonical inputs change.
use super::*;
use surgeist_html_parser::{
    HtmlFragmentParseRequest, HtmlParseHyperlinkState, HtmlParseMetadataStep,
};
use surgeist_html_semantics::*;

const MARKUP: &str = "<div><a href=x></a></div>";

fn base_facts(revision: HtmlBaseFactsRevision, encoding: HtmlBaseEncoding) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(
                HtmlBaseUrlRecord::observed(
                    revision,
                    "about:blank".into(),
                    HtmlBaseUrlScheme::Other,
                    true,
                    None,
                )
                .unwrap(),
            ),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap()
}

fn parsing_facts(base: &HtmlParseBaseFacts, extra: bool) -> HtmlEncodingUrlParseFacts {
    let mut observations = vec![HtmlEncodingUrlParseObservation::new(
        "x".into(),
        HtmlEncodingUrlParseOutcome::Failure,
    )];
    if extra {
        observations.push(HtmlEncodingUrlParseObservation::new(
            "y".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        ));
    }
    HtmlEncodingUrlParseFacts::new(
        base.document().document_url().unwrap().clone(),
        base.document().encoding().unwrap().clone(),
        base.document().revision().clone(),
        observations,
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap()
}

struct Parsed {
    dom: Dom,
    document: DocumentId,
    host: NodeId,
    before: DomSnapshot,
    base: HtmlParseBaseFacts,
    parsing: HtmlEncodingUrlParseFacts,
    capture: DomHtmlCapture,
    report: Arc<HtmlParseReport>,
}

fn parsed() -> Parsed {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(256, 10_000_000)
        .unwrap()
        .with_encoding_url_processing(256, 10_000_000)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let base = base_facts(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        HtmlBaseEncoding::fresh(),
    );
    let parsing = parsing_facts(&base, false);
    let mut batch = DomBatch::new();
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    batch.bind_base_facts(stamp, &base);
    batch.bind_encoding_url_facts(stamp, &parsing);
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(4096, 8192, 4096, 1_000_000, 4096, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(4096, 10_000_000).with_base_facts(4096, 65536, 10_000_000),
            &cancel,
        )
        .unwrap();
    let selected = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(document.node()).unwrap(),
                HtmlSemanticQuery::DocumentEncodingUrlContext,
            )],
            surgeist_html::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 0, 10_000_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(selected)) =
        selected.results()[0].fact()
    else {
        panic!("actual canonical bound context")
    };
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(HtmlSource::utf8(MARKUP), context, parse_limits())
            .unwrap()
            .with_encoding_url_contexts(vec![selected.clone()])
            .with_encoding_url_facts(vec![parsing.clone()])
            .parse()
            .unwrap(),
    );
    Parsed {
        dom,
        document,
        host,
        before,
        base,
        parsing,
        capture,
        report,
    }
}

fn assert_historical_inputs(f: &Parsed) {
    assert_eq!(f.capture.basis().store(), f.before.stamp());
    assert_eq!(f.report.source().as_utf8(), Some(MARKUP));
    assert_eq!(
        f.report.context().unwrap().capture().identity(),
        f.capture.capture().identity()
    );
    let mut cursor = f.report.metadata();
    let mut observed = 0;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        panic!("literal relative URL failure was observed")
                    };
                    assert_eq!(snapshot.raw(), &DocumentString::from("x"));
                    assert_eq!(snapshot.base(), f.base.document().document_url().unwrap());
                    assert_eq!(snapshot.encoding(), f.base.document().encoding().unwrap());
                    assert_eq!(snapshot.revision(), f.base.document().revision());
                    assert_eq!(snapshot.outcome(), &HtmlEncodingUrlParseOutcome::Failure);
                    observed += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(observed, 1);
    let old_document = f.before.document(f.document).unwrap();
    assert!(old_document.base_facts().unwrap().same_owner(&f.base));
    assert_eq!(old_document.encoding_url_facts(), Some(&f.parsing));
    assert_eq!(
        f.before.node(f.document.node()).unwrap().children(),
        &[f.host]
    );
    assert_eq!(
        f.before.node(f.host).unwrap().parent(),
        Some(f.document.node())
    );
    assert!(f.before.node(f.host).unwrap().children().is_empty());
}

#[test]
fn unbound_external_banks_leave_the_canonical_context_installable() {
    let mut f = parsed();
    assert_historical_inputs(&f);
    let stamp = f.dom.snapshot().stamp();
    let usage = f.dom.usage();
    let external = base_facts(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 4, 5, 6),
        HtmlBaseEncoding::fresh(),
    );
    let external_parsing = parsing_facts(&external, true);
    assert_ne!(external.document().revision(), f.base.document().revision());
    assert_ne!(external_parsing.encoding(), f.parsing.encoding());
    assert_eq!(f.dom.snapshot().stamp(), stamp);
    assert_eq!(f.dom.usage(), usage);
    assert!(f.dom.changes_since(stamp).unwrap().changes().is_empty());
    assert!(
        f.dom
            .snapshot()
            .document(f.document)
            .unwrap()
            .base_facts()
            .unwrap()
            .same_owner(&f.base)
    );
    let report = f.report.clone();
    let installed = f
        .dom
        .install_parsed_html_fragment(
            report.clone(),
            &f.capture,
            limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(Arc::ptr_eq(&report, &f.report));
    assert_eq!(
        f.dom.snapshot().stamp().revision().get(),
        stamp.revision().get() + 1
    );
    let link = installed
        .bindings()
        .iter()
        .find_map(|binding| {
            installed
                .snapshot()
                .node(binding.node())
                .unwrap()
                .as_hyperlink()
        })
        .unwrap();
    assert_eq!(link.parse_document(), Some(f.document));
    assert_eq!(
        link.parse_observation().unwrap().base(),
        f.base.document().document_url().unwrap()
    );
    assert_eq!(
        link.parse_observation().unwrap().encoding(),
        f.base.document().encoding().unwrap()
    );
    assert_eq!(
        link.parse_observation().unwrap().raw(),
        &DocumentString::from("x")
    );
    assert_eq!(
        link.parse_observation().unwrap().revision(),
        f.base.document().revision()
    );
    assert_eq!(
        link.parse_observation().unwrap().outcome(),
        &HtmlEncodingUrlParseOutcome::Failure
    );
    assert_eq!(
        installed.captured_node_for(&f.capture.subject(f.document.node()).unwrap()),
        Some(f.document.node())
    );
    assert_historical_inputs(&f);
}

#[derive(Clone, Copy)]
enum Binding {
    Base,
    Encoding,
}

fn refuses_after_binding_change(binding: Binding) {
    let mut f = parsed();
    assert_historical_inputs(&f);
    let old_stamp = f.before.stamp();
    let mut batch = DomBatch::new();
    let stamp = f.before.document(f.document).unwrap().stamp();
    let (next_base, next_parsing, change) = match binding {
        Binding::Base => {
            let next = base_facts(
                HtmlBaseFactsRevision::new(
                    f.base.document().revision().producer().clone(),
                    2,
                    2,
                    3,
                ),
                f.base.document().encoding().unwrap().clone(),
            );
            batch.bind_base_facts(stamp, &next);
            (next, f.parsing.clone(), DomChange::DocumentBase(f.document))
        }
        Binding::Encoding => {
            let next = parsing_facts(&f.base, true);
            assert!(!next.same_owner(&f.parsing));
            batch.bind_encoding_url_facts(stamp, &next);
            (
                f.base.clone(),
                next,
                DomChange::DocumentEncodingUrlFacts(f.document),
            )
        }
    };
    let commit = f.dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = f.dom.snapshot();
    assert_eq!(current.stamp().owner(), old_stamp.owner());
    assert_eq!(
        current.stamp().revision().get(),
        old_stamp.revision().get() + 1
    );
    assert_eq!(commit.changes().changes(), &[change]);
    assert!(
        current
            .document(f.document)
            .unwrap()
            .base_facts()
            .unwrap()
            .same_owner(&next_base)
    );
    assert_eq!(
        current.document(f.document).unwrap().encoding_url_facts(),
        Some(&next_parsing)
    );
    assert_historical_inputs(&f);
    let usage = f.dom.usage();
    let report = f.report.clone();
    for _ in 0..2 {
        // Reuse the same immutable completed report and original capture, rather
        // than reparsing or substituting a new contextual authority.
        let error = f
            .dom
            .install_parsed_html_fragment(
                report.clone(),
                &f.capture,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        assert!(
            matches!(error, DomParsedHtmlInstallError::StaleDestination { expected, actual } if expected == old_stamp && actual == current.stamp())
        );
        assert!(Arc::ptr_eq(&report, &f.report));
        assert_eq!(f.dom.snapshot().stamp(), current.stamp());
        assert_eq!(f.dom.usage(), usage);
        assert!(
            f.dom
                .changes_since(current.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        let now = f.dom.snapshot();
        assert!(
            now.document(f.document)
                .unwrap()
                .base_facts()
                .unwrap()
                .same_owner(&next_base)
        );
        assert_eq!(
            now.document(f.document).unwrap().encoding_url_facts(),
            Some(&next_parsing)
        );
        assert_eq!(now.node(f.document.node()).unwrap().children(), &[f.host]);
        assert_eq!(now.node(f.host).unwrap().parent(), Some(f.document.node()));
        assert!(now.node(f.host).unwrap().children().is_empty());
        assert_historical_inputs(&f);
    }
}

#[test]
fn canonical_base_rebound_after_parse_refuses_the_original_report_atomically() {
    refuses_after_binding_change(Binding::Base);
}

#[test]
fn canonical_encoding_bank_rebound_after_parse_refuses_the_original_report_atomically() {
    refuses_after_binding_change(Binding::Encoding);
}
